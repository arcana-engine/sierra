use std::{
    convert::{TryFrom as _, TryInto as _},
    ffi::CString,
    fmt::{self, Debug},
    mem::{size_of_val, MaybeUninit},
    ops::Range,
    sync::{Arc, Weak},
};

use hashbrown::hash_map::{Entry, HashMap};

use bytemuck::Pod;

#[cfg(any(feature = "glsl", feature = "wgsl"))]
use codespan_reporting::{
    diagnostic::{Diagnostic, Label},
    files::SimpleFile,
    term::termcolor::{ColorChoice, StandardStream},
};

use erupt::{
    extensions::{
        khr_acceleration_structure as vkacc, khr_deferred_host_operations as vkdho,
        khr_ray_tracing_pipeline as vkrt, khr_swapchain as vksw,
    },
    vk1_0, vk1_1, vk1_2, vk1_3, DeviceLoader, ExtendableFrom, ObjectHandle,
};
use gpu_alloc::{Dedicated, GpuAllocator, MemoryBlock};
use gpu_alloc_erupt::EruptMemoryDevice;
use gpu_descriptor::{DescriptorAllocator, DescriptorSetLayoutCreateFlags, DescriptorTotalCount};
use gpu_descriptor_erupt::EruptDescriptorDevice;

#[cfg(any(feature = "glsl", feature = "wgsl"))]
use naga::WithSpan;

use parking_lot::Mutex;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use slab::Slab;
use smallvec::SmallVec;

use crate::{
    accel::{
        AccelerationStructure, AccelerationStructureBuildFlags,
        AccelerationStructureBuildSizesInfo, AccelerationStructureGeometryInfo,
        AccelerationStructureInfo, AccelerationStructureLevel,
    },
    align_up, arith_eq, arith_le, arith_ne, assert_object,
    buffer::{
        Buffer, BufferInfo, BufferRange, BufferUsage, BufferView, BufferViewInfo, MappableBuffer,
        StridedBufferRange,
    },
    descriptor::{
        DescriptorBindingFlags, DescriptorSetInfo, DescriptorSetLayout, DescriptorSetLayoutBinding,
        DescriptorSetLayoutFlags, DescriptorSetLayoutInfo, DescriptorSlice, DescriptorType,
        DescriptorsAllocationError, UpdateDescriptorSet, WritableDescriptorSet,
    },
    fence::Fence,
    framebuffer::{Framebuffer, FramebufferInfo},
    host_memory_space_overflow,
    image::{Image, ImageInfo},
    memory::MemoryUsage,
    out_of_host_memory,
    pipeline::{
        ColorBlend, ComputePipeline, ComputePipelineInfo, GraphicsPipeline, GraphicsPipelineInfo,
        PipelineLayout, PipelineLayoutInfo, RayTracingPipeline, RayTracingPipelineInfo,
        RayTracingShaderGroupInfo, ShaderBindingTable, ShaderBindingTableInfo, State,
    },
    queue::QueueId,
    render_pass::{CreateRenderPassError, RenderPass, RenderPassInfo},
    sampler::{Sampler, SamplerInfo},
    semaphore::Semaphore,
    shader::{
        CreateShaderModuleError, InvalidShader, ShaderLanguage, ShaderModule, ShaderModuleInfo,
        ShaderStage,
    },
    surface::Surface,
    view::{ImageView, ImageViewInfo, ImageViewKind},
    CreateSurfaceError, DeviceAddress, DeviceLost, GraphicsPipelineRenderingInfo, IndexType,
    MapError, OutOfMemory, SurfaceInfo,
};

use super::{
    access::supported_access,
    convert::{buffer_memory_usage_to_gpu_alloc, from_erupt, oom_error_from_erupt, ToErupt as _},
    epochs::Epochs,
    graphics::Graphics,
    physical::{Features, Properties},
    resources::FenceState,
    unexpected_result,
};

impl From<gpu_alloc::MapError> for MapError {
    fn from(err: gpu_alloc::MapError) -> Self {
        match err {
            gpu_alloc::MapError::OutOfDeviceMemory => MapError::OutOfMemory {
                source: OutOfMemory,
            },
            gpu_alloc::MapError::OutOfHostMemory => out_of_host_memory(),
            gpu_alloc::MapError::NonHostVisible => MapError::NonHostVisible,
            gpu_alloc::MapError::MapFailed => MapError::MapFailed,
            gpu_alloc::MapError::AlreadyMapped => MapError::AlreadyMapped,
        }
    }
}

pub(crate) struct Inner {
    logical: DeviceLoader,
    physical: vk1_0::PhysicalDevice,
    properties: Properties,
    features: Features,
    allocator: Mutex<GpuAllocator<vk1_0::DeviceMemory>>,
    version: u32,
    buffers: Mutex<Slab<vk1_0::Buffer>>,
    buffer_views: Mutex<Slab<vk1_0::BufferView>>,
    descriptor_allocator: Mutex<DescriptorAllocator<vk1_0::DescriptorPool, vk1_0::DescriptorSet>>,
    descriptor_set_layouts: Mutex<Slab<vk1_0::DescriptorSetLayout>>,
    fences: Mutex<Slab<vk1_0::Fence>>,
    framebuffers: Mutex<Slab<vk1_0::Framebuffer>>,
    images: Mutex<Slab<vk1_0::Image>>,
    image_views: Mutex<Slab<vk1_0::ImageView>>,
    pipelines: Mutex<Slab<vk1_0::Pipeline>>,
    pipeline_layouts: Mutex<Slab<vk1_0::PipelineLayout>>,
    render_passes: Mutex<Slab<vk1_0::RenderPass>>,
    semaphores: Mutex<Slab<vk1_0::Semaphore>>,
    shaders: Mutex<Slab<vk1_0::ShaderModule>>,
    acceleration_strucutres: Mutex<Slab<vkacc::AccelerationStructureKHR>>,
    samplers: Mutex<Slab<vk1_0::Sampler>>,
    swapchains: Mutex<Slab<vksw::SwapchainKHR>>,

    samplers_cache: Mutex<HashMap<SamplerInfo, Sampler>>,

    epochs: Epochs,
}

impl Inner {
    fn wait_idle(&self) -> Result<(), DeviceLost> {
        let epochs = self.epochs.next_epoch_all_queues();
        let result = unsafe { self.logical.device_wait_idle() }.result();

        match result {
            Ok(()) => {
                for (queue, epoch) in epochs {
                    self.epochs.close_epoch(queue, epoch);
                }
                Ok(())
            }
            Err(vk1_0::Result::ERROR_DEVICE_LOST) => Err(DeviceLost),
            Err(result) => unexpected_result(result),
        }
    }
}

impl Debug for Inner {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        if fmt.alternate() {
            fmt.debug_struct("Device")
                .field("logical", &self.logical.handle)
                .field("physical", &self.physical)
                .finish()
        } else {
            Debug::fmt(&self.logical.handle, fmt)
        }
    }
}

impl Drop for Inner {
    fn drop(&mut self) {
        // Ignore device lost error here
        let _ = self.wait_idle();

        unsafe {
            self.allocator
                .get_mut()
                .cleanup(EruptMemoryDevice::wrap(&self.logical));
            self.descriptor_allocator
                .get_mut()
                .cleanup(EruptDescriptorDevice::wrap(&self.logical));
        }
    }
}

/// Weak reference to the device.
/// Must be upgraded to strong reference before use.
/// Upgrade will fail if last strong reference to device was dropped.
#[derive(Clone)]
#[repr(transparent)]
pub struct WeakDevice {
    inner: Weak<Inner>,
}

impl Debug for WeakDevice {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.inner.upgrade() {
            Some(device) => device.fmt(fmt),
            None => write!(fmt, "Destroyed device: {:p}", self.inner.as_ptr()),
        }
    }
}

impl WeakDevice {
    /// Upgrades to strong reference.
    pub fn upgrade(&self) -> Option<Device> {
        self.inner.upgrade().map(|inner| Device { inner })
    }

    /// Checks if this reference points to the same device.
    pub fn is(&self, device: &Device) -> bool {
        self.inner.as_ptr() == &*device.inner
    }
}

impl PartialEq<Device> for Device {
    fn eq(&self, weak: &Device) -> bool {
        Arc::ptr_eq(&weak.inner, &self.inner)
    }
}

impl PartialEq<Device> for &'_ Device {
    fn eq(&self, weak: &Device) -> bool {
        Arc::ptr_eq(&weak.inner, &self.inner)
    }
}

impl PartialEq<WeakDevice> for Device {
    fn eq(&self, weak: &WeakDevice) -> bool {
        std::ptr::eq(weak.inner.as_ptr(), &*self.inner)
    }
}

impl PartialEq<WeakDevice> for &'_ Device {
    fn eq(&self, weak: &WeakDevice) -> bool {
        std::ptr::eq(weak.inner.as_ptr(), &*self.inner)
    }
}

impl PartialEq<WeakDevice> for WeakDevice {
    fn eq(&self, weak: &WeakDevice) -> bool {
        std::ptr::eq(weak.inner.as_ptr(), self.inner.as_ptr())
    }
}

impl PartialEq<WeakDevice> for &'_ WeakDevice {
    fn eq(&self, weak: &WeakDevice) -> bool {
        std::ptr::eq(weak.inner.as_ptr(), self.inner.as_ptr())
    }
}

/// Handle to abstract device that can execute graphics, compute and ray-tracing pipelines.
#[derive(Clone)]
#[repr(transparent)]
pub struct Device {
    inner: Arc<Inner>,
}

impl Debug for Device {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        if fmt.alternate() {
            fmt.debug_struct("Device")
                .field("logical", &self.inner.logical.handle)
                .field("physical", &self.inner.physical)
                .finish()
        } else {
            Debug::fmt(&self.inner.logical.handle, fmt)
        }
    }
}

impl Device {
    pub(super) fn logical(&self) -> &DeviceLoader {
        &self.inner.logical
    }

    pub(super) fn physical(&self) -> vk1_0::PhysicalDevice {
        self.inner.physical
    }

    pub(super) fn properties(&self) -> &Properties {
        &self.inner.properties
    }

    pub(super) fn features(&self) -> &Features {
        &self.inner.features
    }

    pub(super) fn epochs(&self) -> &Epochs {
        &self.inner.epochs
    }

    pub(super) fn new(
        logical: DeviceLoader,
        physical: vk1_0::PhysicalDevice,
        properties: Properties,
        features: Features,
        version: u32,
        queues: impl Iterator<Item = QueueId>,
    ) -> Self {
        Device {
            inner: Arc::new(Inner {
                allocator: Mutex::new(GpuAllocator::new(
                    gpu_alloc::Config::i_am_prototyping(),
                    memory_device_properties(&properties, &features),
                )),

                descriptor_allocator: Mutex::new(DescriptorAllocator::new(
                    properties
                        .v12
                        .max_update_after_bind_descriptors_in_all_pools,
                )),

                // Numbers here are hints so no strong reasoning is required.
                buffers: Mutex::new(Slab::with_capacity(4096)),
                buffer_views: Mutex::new(Slab::with_capacity(4096)),
                descriptor_set_layouts: Mutex::new(Slab::with_capacity(64)),
                fences: Mutex::new(Slab::with_capacity(128)),
                framebuffers: Mutex::new(Slab::with_capacity(128)),
                images: Mutex::new(Slab::with_capacity(4096)),
                image_views: Mutex::new(Slab::with_capacity(4096)),
                pipelines: Mutex::new(Slab::with_capacity(128)),
                pipeline_layouts: Mutex::new(Slab::with_capacity(64)),
                render_passes: Mutex::new(Slab::with_capacity(32)),
                semaphores: Mutex::new(Slab::with_capacity(128)),
                shaders: Mutex::new(Slab::with_capacity(512)),
                swapchains: Mutex::new(Slab::with_capacity(32)),
                acceleration_strucutres: Mutex::new(Slab::with_capacity(1024)),
                samplers: Mutex::new(Slab::with_capacity(128)),

                logical,
                physical,
                version,
                properties,
                features,

                samplers_cache: Mutex::new(HashMap::new()),

                epochs: Epochs::new(queues),
            }),
        }
    }

    /// Returns [`Graphics`] associated with the device instance.
    pub fn graphics(&self) -> &'static Graphics {
        unsafe {
            // Device can be created only via Graphics instance.
            Graphics::get_unchecked()
        }
    }

    /// Returns weak reference to this device.
    pub fn downgrade(&self) -> WeakDevice {
        WeakDevice {
            inner: Arc::downgrade(&self.inner),
        }
    }

    /// Creates buffer with uninitialized content.
    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub fn create_buffer(&self, info: BufferInfo) -> Result<Buffer, OutOfMemory> {
        self.create_buffer_impl(info, None).map(Into::into)
    }

    /// Creates buffer with uninitialized content.
    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub fn create_mappable_buffer(
        &self,
        info: BufferInfo,
        memory_usage: MemoryUsage,
    ) -> Result<MappableBuffer, OutOfMemory> {
        self.create_buffer_impl(info, Some(memory_usage))
    }

    #[track_caller]
    fn create_buffer_impl(
        &self,
        info: BufferInfo,
        memory_usage: Option<MemoryUsage>,
    ) -> Result<MappableBuffer, OutOfMemory> {
        assert_ne!(info.size, 0, "Buffer size must be greater than 0");

        if info.usage.contains(BufferUsage::DEVICE_ADDRESS) {
            assert_ne!(self.inner.features.v12.buffer_device_address, 0);
        }

        let handle = unsafe {
            self.inner.logical.create_buffer(
                &vk1_0::BufferCreateInfoBuilder::new()
                    .size(info.size)
                    .usage(info.usage.to_erupt())
                    .sharing_mode(vk1_0::SharingMode::EXCLUSIVE),
                None,
            )
        }
        .result()
        .map_err(oom_error_from_erupt)?;

        let mut dedicated = vk1_1::MemoryDedicatedRequirementsBuilder::new();
        let mut reqs = vk1_1::MemoryRequirements2Builder::new().extend_from(&mut dedicated);
        if self.graphics().instance.enabled().vk1_1 {
            unsafe {
                self.inner.logical.get_buffer_memory_requirements2(
                    &vk1_1::BufferMemoryRequirementsInfo2Builder::new().buffer(handle),
                    &mut reqs,
                )
            }
        } else {
            reqs.memory_requirements =
                unsafe { self.inner.logical.get_buffer_memory_requirements(handle) }
        }

        debug_assert!(reqs.memory_requirements.alignment.is_power_of_two());

        let block = {
            let device = EruptMemoryDevice::wrap(&self.inner.logical);
            let request = gpu_alloc::Request {
                size: reqs.memory_requirements.size,
                align_mask: (reqs.memory_requirements.alignment - 1) | info.align,
                memory_types: reqs.memory_requirements.memory_type_bits,
                usage: buffer_memory_usage_to_gpu_alloc(info.usage, memory_usage),
            };

            let dedicated = if dedicated.requires_dedicated_allocation != 0 {
                Some(Dedicated::Required)
            } else if dedicated.prefers_dedicated_allocation != 0 {
                Some(Dedicated::Preferred)
            } else {
                None
            };

            unsafe {
                match dedicated {
                    None => self.inner.allocator.lock().alloc(device, request),
                    Some(dedicated) => self
                        .inner
                        .allocator
                        .lock()
                        .alloc_with_dedicated(device, request, dedicated),
                }
            }
            .map_err(|err| {
                unsafe {
                    self.inner.logical.destroy_buffer(handle, None);
                }

                error!("{:#}", err);
                OutOfMemory
            })?
        };

        let result = unsafe {
            self.inner
                .logical
                .bind_buffer_memory(handle, *block.memory(), block.offset())
        }
        .result();

        if let Err(err) = result {
            unsafe {
                self.inner.logical.destroy_buffer(handle, None);

                self.inner
                    .allocator
                    .lock()
                    .dealloc(EruptMemoryDevice::wrap(&self.inner.logical), block);
            }

            return Err(oom_error_from_erupt(err));
        }

        let address = if info.usage.contains(BufferUsage::DEVICE_ADDRESS) {
            Some(Option::unwrap(from_erupt(unsafe {
                self.inner.logical.get_buffer_device_address(
                    &vk1_2::BufferDeviceAddressInfoBuilder::new().buffer(handle),
                )
            })))
        } else {
            None
        };

        let buffer_index = self.inner.buffers.lock().insert(handle);

        debug!("Buffer created {:p}", handle);
        Ok(MappableBuffer::new(
            info,
            self.downgrade(),
            handle,
            address,
            buffer_index,
            block,
            memory_usage.unwrap_or_else(MemoryUsage::empty),
        ))
    }

    /// Creates static buffer pre-initialized with content from [`data`].
    /// Implies `MemoryUsage::Device`.
    ///
    /// # Panics
    ///
    /// Function will panic if specified buffer size does not equal data size.
    /// i.e. if `info.size != std::mem::size_of(data)`.
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(data)))]
    pub fn create_buffer_static<T: 'static>(
        &self,
        info: BufferInfo,
        data: &[T],
    ) -> Result<Buffer, OutOfMemory>
    where
        T: Pod,
    {
        assert!(info.is_valid());

        if arith_ne(info.size, size_of_val(data)) {
            panic!(
                "Buffer size {} does not match data size {}",
                info.size,
                size_of_val(data)
            );
        }

        debug_assert!(arith_eq(info.size, size_of_val(data)));

        let mut buffer = self.create_mappable_buffer(info, MemoryUsage::UPLOAD)?;

        match self.upload_to_memory(&mut buffer, 0, data) {
            Ok(()) => Ok(buffer.share()),
            Err(MapError::OutOfMemory { source }) => Err(source),
            Err(MapError::NonHostVisible) => unreachable!(),
            Err(MapError::AlreadyMapped) => unreachable!(),
            Err(MapError::MapFailed) => panic!("Map failed"),
        }
    }

    pub(super) unsafe fn destroy_buffer(
        &self,
        index: usize,
        block: MemoryBlock<vk1_0::DeviceMemory>,
    ) {
        self.inner
            .allocator
            .lock()
            .dealloc(EruptMemoryDevice::wrap(&self.inner.logical), block);

        let handle = self.inner.buffers.lock().remove(index);
        self.inner.logical.destroy_buffer(handle, None);
    }

    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub fn create_buffer_view(&self, info: BufferViewInfo) -> Result<BufferView, OutOfMemory> {
        assert_owner!(info.buffer, self);
        assert!(info
            .buffer
            .info()
            .usage
            .intersects(BufferUsage::UNIFORM_TEXEL | BufferUsage::STORAGE_TEXEL), "BufferView cannot be created from buffer without at least on of `UNIFORM_TEXEL` or `STORAGE_TEXEL` usage flags");

        let buffer = &info.buffer;

        let view = unsafe {
            self.inner.logical.create_buffer_view(
                &vk1_0::BufferViewCreateInfoBuilder::new()
                    .buffer(buffer.handle())
                    .format(info.format.to_erupt())
                    .offset(info.offset)
                    .range(info.size),
                None,
            )
        }
        .result()
        .map_err(oom_error_from_erupt)?;

        let index = self.inner.buffer_views.lock().insert(view);

        debug!("BufferView created {:p}", view);
        Ok(BufferView::new(info, self.downgrade(), view, index))
    }

    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub(super) unsafe fn destroy_buffer_view(&self, index: usize) {
        let handle = self.inner.buffer_views.lock().remove(index);
        self.inner.logical.destroy_buffer_view(handle, None);
    }

    /// Creates image with uninitialized content.
    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub fn create_image(&self, info: ImageInfo) -> Result<Image, OutOfMemory> {
        let handle = unsafe {
            self.inner.logical.create_image(
                &vk1_0::ImageCreateInfoBuilder::new()
                    .image_type(info.extent.to_erupt())
                    .format(info.format.to_erupt())
                    .extent(info.extent.into_3d().to_erupt())
                    .mip_levels(info.levels)
                    .array_layers(info.layers)
                    .samples(info.samples.to_erupt())
                    .tiling(vk1_0::ImageTiling::OPTIMAL)
                    .usage(info.usage.to_erupt())
                    .sharing_mode(vk1_0::SharingMode::EXCLUSIVE)
                    .initial_layout(vk1_0::ImageLayout::UNDEFINED),
                None,
            )
        }
        .result()
        .map_err(oom_error_from_erupt)?;

        let mut dedicated = vk1_1::MemoryDedicatedRequirementsBuilder::new();
        let mut reqs = vk1_1::MemoryRequirements2Builder::new().extend_from(&mut dedicated);
        if self.graphics().instance.enabled().vk1_1 {
            unsafe {
                self.inner.logical.get_image_memory_requirements2(
                    &vk1_1::ImageMemoryRequirementsInfo2Builder::new().image(handle),
                    &mut reqs,
                )
            }
        } else {
            reqs.memory_requirements =
                unsafe { self.inner.logical.get_image_memory_requirements(handle) }
        }

        debug_assert!(reqs.memory_requirements.alignment.is_power_of_two());

        let block = {
            let device = EruptMemoryDevice::wrap(&self.inner.logical);
            let request = gpu_alloc::Request {
                size: reqs.memory_requirements.size,
                align_mask: reqs.memory_requirements.alignment - 1,
                memory_types: reqs.memory_requirements.memory_type_bits,
                usage: gpu_alloc::UsageFlags::empty(),
            };

            let dedicated = if dedicated.requires_dedicated_allocation != 0 {
                Some(Dedicated::Required)
            } else if dedicated.prefers_dedicated_allocation != 0 {
                Some(Dedicated::Preferred)
            } else {
                None
            };

            unsafe {
                match dedicated {
                    None => self.inner.allocator.lock().alloc(device, request),
                    Some(dedicated) => self
                        .inner
                        .allocator
                        .lock()
                        .alloc_with_dedicated(device, request, dedicated),
                }
            }
            .map_err(|err| {
                unsafe {
                    self.inner.logical.destroy_image(handle, None);
                }

                error!("{:#}", err);
                OutOfMemory
            })?
        };

        let result = unsafe {
            self.inner
                .logical
                .bind_image_memory(handle, *block.memory(), block.offset())
        }
        .result();

        match result {
            Ok(()) => {
                let index = self.inner.images.lock().insert(handle);

                debug!("Image created {:p}", handle);
                Ok(Image::new(info, self.downgrade(), handle, block, index))
            }
            Err(err) => {
                unsafe {
                    self.inner.logical.destroy_image(handle, None);
                    self.inner
                        .allocator
                        .lock()
                        .dealloc(EruptMemoryDevice::wrap(&self.inner.logical), block);
                }

                Err(oom_error_from_erupt(err))
            }
        }
    }

    pub(super) unsafe fn destroy_image(
        &self,
        index: usize,
        block: MemoryBlock<vk1_0::DeviceMemory>,
    ) {
        self.inner
            .allocator
            .lock()
            .dealloc(EruptMemoryDevice::wrap(self.logical()), block);

        let handle = self.inner.images.lock().remove(index);
        self.inner.logical.destroy_image(handle, None);
    }

    // /// Creates static image with preinitialized content from `data`.
    // ///
    // /// # Panics
    // ///
    // /// Function will panic if creating image size does not equal data size.
    // #[cfg_attr(feature = "tracing", tracing::instrument(skip(data)))]
    // pub fn create_image_static<T>(
    //     &self,
    //     info: ImageInfo,
    //     data: &[T],
    // ) -> Result<Image, OutOfMemory>
    // where
    //     T: Pod,
    // {
    //     assert!(info.memory.intersects(
    //         MemoryUsage::HOST_ACCESS
    //             | MemoryUsage::UPLOAD
    //             | MemoryUsage::DOWNLOAD
    //     ));

    //     let image = unsafe {
    //         self.inner.logical.create_image(
    //             &vk1_0::ImageCreateInfoBuilder::new()
    //                 .image_type(info.extent.to_erupt())
    //                 .format(info.format.to_erupt())
    //                 .extent(info.extent.into_3d().to_erupt())
    //                 .mip_levels(info.levels)
    //                 .array_layers(info.layers)
    //                 .samples(info.samples.to_erupt())
    //                 .tiling(vk1_0::ImageTiling::LINEAR)
    //                 .usage(info.usage.to_erupt())
    //                 .sharing_mode(vk1_0::SharingMode::EXCLUSIVE)
    //                 .initial_layout(vk1_0::ImageLayout::UNDEFINED),
    //             None,
    //             None,
    //         )
    //     }
    //     .result()
    //     .map_err(oom_error_from_erupt)?;

    //     let reqs = unsafe {
    //         self.inner
    //             .logical
    //             .get_image_memory_requirements(image, None)
    //     };

    //     debug_assert!(arith_eq(reqs.size, data.len()));
    //     debug_assert!(reqs.alignment.is_power_of_two());

    //     let mut block = unsafe {
    //         self.inner
    //             .allocator
    //             .lock()
    //             .alloc(
    //                 EruptMemoryDevice::wrap(&self.inner.logical),
    //                 gpu_alloc::Request {
    //                     size: reqs.size,
    //                     align_mask: reqs.alignment - 1,
    //                     memory_types: reqs.memory_type_bits,
    //                     usage: image_memory_usage_to_gpu_alloc(info.usage),
    //                 },
    //             )
    //             .map_err(|err| {
    //                 self.inner.logical.destroy_image(image, None);
    //                 error!("{:#}", err);
    //                 OutOfMemory
    //             })
    //     }?;

    //     let result = unsafe {
    //         self.inner.logical.bind_image_memory(
    //             image,
    //             *block.memory(),
    //             block.offset(),
    //         )
    //     }
    //     .result();

    //     if let Err(err) = result {
    //         unsafe {
    //             self.inner.logical.destroy_image(image, None);
    //             self.inner.allocator.lock().dealloc(
    //                 EruptMemoryDevice::wrap(&self.inner.logical),
    //                 block,
    //             );
    //         }
    //         return Err(oom_error_from_erupt(err).into());
    //     }

    //     unsafe {
    //         match block.map(
    //             EruptMemoryDevice::wrap(&self.inner.logical),
    //             0,
    //             size_of_val(data),
    //         ) {
    //             Ok(ptr) => {
    //                 std::ptr::copy_nonoverlapping(
    //                     data.as_ptr() as *const u8,
    //                     ptr.as_ptr(),
    //                     size_of_val(data),
    //                 );

    //                 block.unmap(EruptMemoryDevice::wrap(&self.inner.logical));
    //             }
    //             Err(gpu_alloc::MapError::OutOfDeviceMemory) => {
    //                 return Err(OutOfMemory.into())
    //             }
    //             Err(gpu_alloc::MapError::OutOfHostMemory) => {
    //                 out_of_host_memory()
    //             }
    //             Err(gpu_alloc::MapError::NonHostVisible)
    //             | Err(gpu_alloc::MapError::AlreadyMapped) => unreachable!(),
    //             Err(gpu_alloc::MapError::MapFailed) => panic!("Map failed"),
    //         }
    //     }

    //     let index = self.inner.images.lock().insert(image);

    //     Ok(Image::new(
    //         info,
    //         self.downgrade(),
    //         image,
    //         Some(block),
    //         Some(index),
    //     ))
    // }

    /// Creates view to an image.
    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub fn create_image_view(&self, info: ImageViewInfo) -> Result<ImageView, OutOfMemory> {
        assert_owner!(info.image, self);

        let image = &info.image;

        let view = unsafe {
            self.inner.logical.create_image_view(
                &vk1_0::ImageViewCreateInfoBuilder::new()
                    .image(image.handle())
                    .format(info.image.info().format.to_erupt())
                    .view_type(info.view_kind.to_erupt())
                    .subresource_range(
                        vk1_0::ImageSubresourceRangeBuilder::new()
                            .aspect_mask(info.range.aspect.to_erupt())
                            .base_mip_level(info.range.first_level)
                            .level_count(info.range.level_count)
                            .base_array_layer(info.range.first_layer)
                            .layer_count(info.range.layer_count)
                            .build(),
                    )
                    .components(info.mapping.to_erupt()),
                None,
            )
        }
        .result()
        .map_err(oom_error_from_erupt)?;

        let index = self.inner.image_views.lock().insert(view);

        debug!("ImageView created {:p}", view);
        Ok(ImageView::new(info, self.downgrade(), view, index))
    }

    pub(super) unsafe fn destroy_image_view(&self, index: usize) {
        let handle = self.inner.image_views.lock().remove(index);
        self.inner.logical.destroy_image_view(handle, None);
    }

    /// Returns handle to newly created [`Fence`].
    /// Fences are create in un-signaled state.
    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub fn create_fence(&self) -> Result<Fence, OutOfMemory> {
        let fence = unsafe {
            self.inner
                .logical
                .create_fence(&vk1_0::FenceCreateInfoBuilder::new(), None)
        }
        .result()
        .map_err(oom_error_from_erupt)?;

        let index = self.inner.fences.lock().insert(fence);

        debug!("Fence created {:p}", fence);
        Ok(Fence::new(self.downgrade(), fence, index))
    }

    pub(super) unsafe fn destroy_fence(&self, index: usize) {
        let handle = self.inner.fences.lock().remove(index);
        self.inner.logical.destroy_fence(handle, None);
    }

    /// Returns handle to newly created [`Framebuffer`].
    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub fn create_framebuffer(&self, info: FramebufferInfo) -> Result<Framebuffer, OutOfMemory> {
        for view in &info.attachments {
            assert_owner!(view, self);
        }

        assert_owner!(info.render_pass, self);

        assert!(
            info.attachments
                .iter()
                .all(|view| view.info().view_kind == ImageViewKind::D2),
            "All image views for Framebuffer must have `view_kind == ImageViewKind::D2`",
        );

        assert!(
            info.attachments
                .iter()
                .all(|view| view.info().image.info().extent.into_2d() >= info.extent),
            "All image views for Framebuffer must be at least as large as framebuffer extent",
        );

        let render_pass = info.render_pass.handle();

        let attachments = info
            .attachments
            .iter()
            .map(|view| view.handle())
            .collect::<SmallVec<[_; 16]>>();

        let framebuffer = unsafe {
            self.inner.logical.create_framebuffer(
                &vk1_0::FramebufferCreateInfoBuilder::new()
                    .render_pass(render_pass)
                    .attachments(&attachments)
                    .width(info.extent.width)
                    .height(info.extent.height)
                    .layers(1),
                None,
            )
        }
        .result()
        .map_err(oom_error_from_erupt)?;

        let index = self.inner.framebuffers.lock().insert(framebuffer);

        debug!("Framebuffer created {:p}", framebuffer);
        Ok(Framebuffer::new(info, self.downgrade(), framebuffer, index))
    }

    pub(super) unsafe fn destroy_framebuffer(&self, index: usize) {
        let handle = self.inner.framebuffers.lock().remove(index);
        self.inner.logical.destroy_framebuffer(handle, None);
    }

    /// Creates graphics pipeline.
    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub fn create_graphics_pipeline(
        &self,
        info: GraphicsPipelineInfo,
    ) -> Result<GraphicsPipeline, OutOfMemory> {
        let desc = &info.desc;

        let color_attachments;

        let mut dynamic_rendering_info;
        let colors_count;

        let mut builder = vk1_0::GraphicsPipelineCreateInfoBuilder::new();

        match info.rendering {
            GraphicsPipelineRenderingInfo::DynamicRendering {
                ref colors,
                depth_stencil,
            } => {
                color_attachments = colors
                    .iter()
                    .map(|c| c.to_erupt())
                    .collect::<SmallVec<[_; 16]>>();

                dynamic_rendering_info = vk1_3::PipelineRenderingCreateInfoBuilder::new()
                    .color_attachment_formats(&color_attachments)
                    .depth_attachment_format(
                        depth_stencil.map_or(vk1_0::Format::UNDEFINED, |f| f.to_erupt()),
                    )
                    .stencil_attachment_format(
                        depth_stencil.map_or(vk1_0::Format::UNDEFINED, |f| f.to_erupt()),
                    );

                colors_count = colors.len();

                builder = builder.extend_from(&mut dynamic_rendering_info);
            }
            GraphicsPipelineRenderingInfo::RenderPass {
                ref render_pass,
                subpass,
            } => {
                assert_owner!(render_pass, self);
                colors_count = render_pass.info().subpasses[usize::try_from(subpass).unwrap()]
                    .colors
                    .len();

                builder = builder.render_pass(render_pass.handle()).subpass(subpass);
            }
        }

        assert_owner!(desc.layout, self);
        assert_owner!(desc.vertex_shader.module(), self);
        if let Some(fragment_shader) = desc
            .rasterizer
            .as_ref()
            .and_then(|r| r.fragment_shader.as_ref())
        {
            assert_owner!(fragment_shader.module(), self);
        }

        let mut shader_stages = Vec::with_capacity(2);
        let mut dynamic_states = Vec::with_capacity(7);

        let vertex_binding_descriptions = desc
            .vertex_bindings
            .iter()
            .enumerate()
            .map(|(i, vb)| {
                vk1_0::VertexInputBindingDescriptionBuilder::new()
                    .binding(i.try_into().unwrap())
                    .stride(vb.stride)
                    .input_rate(vb.rate.to_erupt())
            })
            .collect::<SmallVec<[_; 16]>>();

        let vertex_attribute_descriptions = desc
            .vertex_attributes
            .iter()
            .map(|attr| {
                vk1_0::VertexInputAttributeDescriptionBuilder::new()
                    .location(attr.location)
                    .binding(attr.binding)
                    .offset(attr.offset)
                    .format(attr.format.to_erupt())
            })
            .collect::<SmallVec<[_; 16]>>();

        let vertex_input_state = vk1_0::PipelineVertexInputStateCreateInfoBuilder::new()
            .vertex_binding_descriptions(&vertex_binding_descriptions)
            .vertex_attribute_descriptions(&vertex_attribute_descriptions);

        let vertex_shader_entry = entry_name_to_cstr(desc.vertex_shader.entry());

        shader_stages.push(
            vk1_0::PipelineShaderStageCreateInfoBuilder::new()
                .stage(vk1_0::ShaderStageFlagBits::VERTEX)
                .module(desc.vertex_shader.module().handle())
                .name(&vertex_shader_entry),
        );

        let input_assembly_state = vk1_0::PipelineInputAssemblyStateCreateInfoBuilder::new()
            .topology(desc.primitive_topology.to_erupt())
            .primitive_restart_enable(desc.primitive_restart_enable);

        let rasterization_state;

        let viewport;

        let scissor;

        let attachments;

        let mut viewport_state = None;

        let mut multisample_state = None;

        let mut depth_stencil_state = None;

        let mut color_blend_state = None;

        let fragment_shader_entry;

        let with_rasterizer = if let Some(rasterizer) = &desc.rasterizer {
            let mut builder = vk1_0::PipelineViewportStateCreateInfoBuilder::new();

            match &rasterizer.viewport {
                State::Static { value } => {
                    viewport = value.to_erupt().into_builder();

                    builder = builder.viewports(std::slice::from_ref(&viewport));
                }
                State::Dynamic => {
                    dynamic_states.push(vk1_0::DynamicState::VIEWPORT);
                    builder = builder.viewport_count(1);
                }
            }

            match &rasterizer.scissor {
                State::Static { value } => {
                    scissor = value.to_erupt().into_builder();

                    builder = builder.scissors(std::slice::from_ref(&scissor));
                }
                State::Dynamic => {
                    dynamic_states.push(vk1_0::DynamicState::SCISSOR);
                    builder = builder.scissor_count(1);
                }
            }

            viewport_state = Some(builder);

            rasterization_state = vk1_0::PipelineRasterizationStateCreateInfoBuilder::new()
                .rasterizer_discard_enable(false)
                .depth_clamp_enable(rasterizer.depth_clamp)
                .polygon_mode(rasterizer.polygon_mode.to_erupt())
                .cull_mode(rasterizer.culling.to_erupt())
                .front_face(rasterizer.front_face.to_erupt())
                .line_width(1.0);

            multisample_state = Some(
                vk1_0::PipelineMultisampleStateCreateInfoBuilder::new()
                    .rasterization_samples(vk1_0::SampleCountFlagBits::_1),
            );

            let mut builder = vk1_0::PipelineDepthStencilStateCreateInfoBuilder::new();

            if let Some(depth_test) = rasterizer.depth_test {
                builder = builder
                    .depth_test_enable(true)
                    .depth_write_enable(depth_test.write)
                    .depth_compare_op(depth_test.compare.to_erupt())
            };

            if let Some(depth_bounds) = rasterizer.depth_bounds {
                builder = builder.depth_bounds_test_enable(true);

                match depth_bounds {
                    State::Static { value } => {
                        builder = builder
                            .min_depth_bounds(value.offset)
                            .max_depth_bounds(value.offset + value.size)
                    }
                    State::Dynamic => dynamic_states.push(vk1_0::DynamicState::DEPTH_BOUNDS),
                }
            }

            if let Some(stencil_tests) = rasterizer.stencil_tests {
                builder = builder
                    .stencil_test_enable(true)
                    .front({
                        let mut builder = vk1_0::StencilOpStateBuilder::new()
                            .fail_op(stencil_tests.front.fail.to_erupt())
                            .pass_op(stencil_tests.front.pass.to_erupt())
                            .depth_fail_op(stencil_tests.front.depth_fail.to_erupt())
                            .compare_op(stencil_tests.front.compare.to_erupt());

                        match stencil_tests.front.compare_mask {
                            State::Static { value } => builder = builder.compare_mask(value),
                            State::Dynamic => {
                                dynamic_states.push(vk1_0::DynamicState::STENCIL_COMPARE_MASK)
                            }
                        }

                        match stencil_tests.front.write_mask {
                            State::Static { value } => builder = builder.write_mask(value),
                            State::Dynamic => {
                                dynamic_states.push(vk1_0::DynamicState::STENCIL_WRITE_MASK)
                            }
                        }

                        match stencil_tests.front.reference {
                            State::Static { value } => builder = builder.reference(value),
                            State::Dynamic => {
                                dynamic_states.push(vk1_0::DynamicState::STENCIL_REFERENCE)
                            }
                        }

                        *builder
                    })
                    .back({
                        let mut builder = vk1_0::StencilOpStateBuilder::new()
                            .fail_op(stencil_tests.back.fail.to_erupt())
                            .pass_op(stencil_tests.back.pass.to_erupt())
                            .depth_fail_op(stencil_tests.back.depth_fail.to_erupt())
                            .compare_op(stencil_tests.back.compare.to_erupt());

                        match stencil_tests.back.compare_mask {
                            State::Static { value } => builder = builder.compare_mask(value),
                            State::Dynamic => {
                                dynamic_states.push(vk1_0::DynamicState::STENCIL_COMPARE_MASK)
                            }
                        }

                        match stencil_tests.back.write_mask {
                            State::Static { value } => builder = builder.write_mask(value),
                            State::Dynamic => {
                                dynamic_states.push(vk1_0::DynamicState::STENCIL_WRITE_MASK)
                            }
                        }

                        match stencil_tests.back.reference {
                            State::Static { value } => builder = builder.reference(value),
                            State::Dynamic => {
                                dynamic_states.push(vk1_0::DynamicState::STENCIL_REFERENCE)
                            }
                        }

                        *builder
                    });
            }

            depth_stencil_state = Some(builder);

            if let Some(shader) = &rasterizer.fragment_shader {
                fragment_shader_entry = entry_name_to_cstr(shader.entry());
                shader_stages.push(
                    vk1_0::PipelineShaderStageCreateInfoBuilder::new()
                        .stage(vk1_0::ShaderStageFlagBits::FRAGMENT)
                        .module(shader.module().handle())
                        .name(&fragment_shader_entry),
                );
            }

            let mut builder = vk1_0::PipelineColorBlendStateCreateInfoBuilder::new();

            builder = match rasterizer.color_blend {
                ColorBlend::Logic { op } => builder.logic_op_enable(true).logic_op(op.to_erupt()),
                ColorBlend::Blending {
                    blending,
                    write_mask,
                    constants,
                } => {
                    builder = builder.logic_op_enable(false).attachments({
                        attachments = (0..colors_count)
                            .map(|_| {
                                if let Some(blending) = blending {
                                    vk1_0::PipelineColorBlendAttachmentStateBuilder::new()
                                        .blend_enable(true)
                                        .src_color_blend_factor(
                                            blending.color_src_factor.to_erupt(),
                                        )
                                        .dst_color_blend_factor(
                                            blending.color_dst_factor.to_erupt(),
                                        )
                                        .color_blend_op(blending.color_op.to_erupt())
                                        .src_alpha_blend_factor(
                                            blending.alpha_src_factor.to_erupt(),
                                        )
                                        .dst_alpha_blend_factor(
                                            blending.alpha_dst_factor.to_erupt(),
                                        )
                                        .alpha_blend_op(blending.alpha_op.to_erupt())
                                } else {
                                    vk1_0::PipelineColorBlendAttachmentStateBuilder::new()
                                        .blend_enable(false)
                                }
                                .color_write_mask(write_mask.to_erupt())
                            })
                            .collect::<Vec<_>>();
                        &attachments
                    });

                    match constants {
                        State::Static { value } => builder = builder.blend_constants(value),
                        State::Dynamic => dynamic_states.push(vk1_0::DynamicState::BLEND_CONSTANTS),
                    }

                    builder
                }

                ColorBlend::IndependentBlending { .. } => {
                    panic!("Unsupported yet")
                }
            };

            color_blend_state = Some(builder);

            true
        } else {
            rasterization_state = vk1_0::PipelineRasterizationStateCreateInfoBuilder::new()
                .rasterizer_discard_enable(true);

            false
        };

        builder = builder
            .vertex_input_state(&vertex_input_state)
            .input_assembly_state(&input_assembly_state)
            .rasterization_state(&rasterization_state)
            .stages(&shader_stages)
            .layout(desc.layout.handle());

        let pipeline_dynamic_state;

        if !dynamic_states.is_empty() {
            pipeline_dynamic_state =
                vk1_0::PipelineDynamicStateCreateInfoBuilder::new().dynamic_states(&dynamic_states);

            builder = builder.dynamic_state(&pipeline_dynamic_state);
        }

        if with_rasterizer {
            builder = builder
                .viewport_state(viewport_state.as_ref().unwrap())
                .multisample_state(multisample_state.as_ref().unwrap())
                .color_blend_state(color_blend_state.as_ref().unwrap())
                .depth_stencil_state(depth_stencil_state.as_ref().unwrap());
        }

        let pipelines = unsafe {
            self.inner.logical.create_graphics_pipelines(
                vk1_0::PipelineCache::null(),
                &[builder],
                None,
            )
        }
        .result()
        .map_err(oom_error_from_erupt)?;

        debug_assert_eq!(pipelines.len(), 1);

        let pipeline = pipelines[0];
        let index = self.inner.pipelines.lock().insert(pipeline);

        drop(shader_stages);

        debug!("GraphicsPipeline created {:p}", pipeline);
        Ok(GraphicsPipeline::new(
            info,
            self.downgrade(),
            pipeline,
            index,
        ))
    }

    /// Creates compute pipeline.
    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub fn create_compute_pipeline(
        &self,
        info: ComputePipelineInfo,
    ) -> Result<ComputePipeline, OutOfMemory> {
        assert_owner!(info.shader.module(), self);
        assert_owner!(info.layout, self);

        let shader_entry = entry_name_to_cstr(info.shader.entry());

        let pipelines = unsafe {
            self.inner.logical.create_compute_pipelines(
                vk1_0::PipelineCache::null(),
                &[vk1_0::ComputePipelineCreateInfoBuilder::new()
                    .stage(
                        vk1_0::PipelineShaderStageCreateInfoBuilder::new()
                            .stage(vk1_0::ShaderStageFlagBits::COMPUTE)
                            .module(info.shader.module().handle())
                            .name(&shader_entry)
                            .build_dangling(),
                    )
                    .layout(info.layout.handle())],
                None,
            )
        }
        .result()
        .map_err(oom_error_from_erupt)?;

        debug_assert_eq!(pipelines.len(), 1);

        let pipeline = pipelines[0];
        let index = self.inner.pipelines.lock().insert(pipeline);

        debug!("ComputePipeline created {:p}", pipeline);
        Ok(ComputePipeline::new(
            info,
            self.downgrade(),
            pipeline,
            index,
        ))
    }

    pub(super) unsafe fn destroy_pipeline(&self, index: usize) {
        let handle = self.inner.pipelines.lock().remove(index);
        self.inner.logical.destroy_pipeline(handle, None);
    }

    /// Creates pipeline layout.
    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub fn create_pipeline_layout(
        &self,
        info: PipelineLayoutInfo,
    ) -> Result<PipelineLayout, OutOfMemory> {
        for set in &info.sets {
            assert_owner!(set, self);
        }

        let pipeline_layout = unsafe {
            self.inner.logical.create_pipeline_layout(
                &vk1_0::PipelineLayoutCreateInfoBuilder::new()
                    .set_layouts(
                        &info
                            .sets
                            .iter()
                            .map(|set| set.handle())
                            .collect::<SmallVec<[_; 16]>>(),
                    )
                    .push_constant_ranges(
                        &info
                            .push_constants
                            .iter()
                            .map(|pc| {
                                vk1_0::PushConstantRangeBuilder::new()
                                    .stage_flags(pc.stages.to_erupt())
                                    .offset(pc.offset)
                                    .size(pc.size)
                            })
                            .collect::<SmallVec<[_; 16]>>(),
                    ),
                None,
            )
        }
        .result()
        .map_err(oom_error_from_erupt)?;

        let index = self.inner.pipeline_layouts.lock().insert(pipeline_layout);

        debug!("Pipeline layout created: {:p}", pipeline_layout);
        Ok(PipelineLayout::new(
            info,
            self.downgrade(),
            pipeline_layout,
            index,
        ))
    }

    pub(super) unsafe fn destroy_pipeline_layout(&self, index: usize) {
        let handle = self.inner.pipeline_layouts.lock().remove(index);
        self.inner.logical.destroy_pipeline_layout(handle, None);
    }

    /// Creates render pass.
    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub fn create_render_pass(
        &self,
        info: RenderPassInfo,
    ) -> Result<RenderPass, CreateRenderPassError> {
        let mut subpass_attachments = Vec::new();

        let subpasses =
            info.subpasses
                .iter()
                .enumerate()
                .map(|(si, s)| -> Result<_, CreateRenderPassError> {
                    let color_offset = subpass_attachments.len();
                    subpass_attachments.extend(
                        s.colors
                            .iter()
                            .enumerate()
                            .map(|(ci, &(c, cl))| -> Result<_, CreateRenderPassError> {
                                Ok(vk1_0::AttachmentReferenceBuilder::new()
                                .attachment(if arith_le(c, info.attachments.len()) {
                                    Some(c)
                                } else {
                                    None
                                }
                                .ok_or(
                                    CreateRenderPassError::ColorAttachmentReferenceOutOfBound {
                                        subpass: si,
                                        index: ci,
                                        attachment: c,
                                    }
                                )?)
                                .layout(cl.to_erupt())
                            )
                            })
                            .collect::<Result<SmallVec<[_; 16]>, _>>()?,
                    );

                    let depth_offset = subpass_attachments.len();
                    if let Some((d, dl)) = s.depth {
                        subpass_attachments.push(
                            vk1_0::AttachmentReferenceBuilder::new()
                                .attachment(
                                    if arith_le(d, info.attachments.len()) {
                                        Some(d)
                                    } else {
                                        None
                                    }
                                    .ok_or(
                                        CreateRenderPassError::DepthAttachmentReferenceOutOfBound {
                                            subpass: si,
                                            attachment: d,
                                        },
                                    )?,
                                )
                                .layout(dl.to_erupt()),
                        );
                    }
                    Ok((color_offset, depth_offset))
                })
                .collect::<Result<SmallVec<[_; 16]>, _>>()?;

        let subpasses = info
            .subpasses
            .iter()
            .zip(subpasses)
            .map(|(s, (color_offset, depth_offset))| {
                let builder = vk1_0::SubpassDescriptionBuilder::new()
                    .color_attachments(&subpass_attachments[color_offset..depth_offset]);

                if s.depth.is_some() {
                    builder.depth_stencil_attachment(&subpass_attachments[depth_offset])
                } else {
                    builder
                }
            })
            .collect::<Vec<_>>();

        let attachments = info
            .attachments
            .iter()
            .map(|a| {
                vk1_0::AttachmentDescriptionBuilder::new()
                    .format(a.format.to_erupt())
                    .load_op(a.load_op.to_erupt())
                    .store_op(a.store_op.to_erupt())
                    .initial_layout(a.initial_layout.to_erupt())
                    .final_layout(a.final_layout.to_erupt())
                    .samples(vk1_0::SampleCountFlagBits::_1)
            })
            .collect::<SmallVec<[_; 16]>>();

        let dependencies = info
            .dependencies
            .iter()
            .map(|d| {
                vk1_0::SubpassDependencyBuilder::new()
                    .src_subpass(d.src.unwrap_or(vk1_0::SUBPASS_EXTERNAL))
                    .dst_subpass(d.dst.unwrap_or(vk1_0::SUBPASS_EXTERNAL))
                    .src_stage_mask(d.src_stages.to_erupt())
                    .dst_stage_mask(d.dst_stages.to_erupt())
                    .src_access_mask(supported_access(d.src_stages.to_erupt()))
                    .dst_access_mask(supported_access(d.dst_stages.to_erupt()))
            })
            .collect::<SmallVec<[_; 16]>>();

        let render_passs_create_info = vk1_0::RenderPassCreateInfoBuilder::new()
            .attachments(&attachments)
            .subpasses(&subpasses)
            .dependencies(&dependencies);

        let render_pass = unsafe {
            self.inner
                .logical
                .create_render_pass(&render_passs_create_info, None)
        }
        .result()
        .map_err(create_render_pass_error_from_erupt)?;

        let index = self.inner.render_passes.lock().insert(render_pass);

        debug!("Render pass created: {:p}", render_pass);
        Ok(RenderPass::new(info, self.downgrade(), render_pass, index))
    }

    pub(super) unsafe fn destroy_render_pass(&self, index: usize) {
        let handle = self.inner.render_passes.lock().remove(index);
        self.inner.logical.destroy_render_pass(handle, None);
    }

    pub(crate) fn create_semaphore_raw(&self) -> Result<(vk1_0::Semaphore, usize), vk1_0::Result> {
        let semaphore = unsafe {
            self.inner
                .logical
                .create_semaphore(&vk1_0::SemaphoreCreateInfoBuilder::new(), None)
        }
        .result()?;

        let index = self.inner.semaphores.lock().insert(semaphore);

        Ok((semaphore, index))
    }

    /// Creates semaphore. Semaphores are created in unsignaled state.
    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub fn create_semaphore(&self) -> Result<Semaphore, OutOfMemory> {
        let (handle, index) = self.create_semaphore_raw().map_err(oom_error_from_erupt)?;

        debug!("Semaphore created: {:p}", handle);
        Ok(Semaphore::new(self.downgrade(), handle, index))
    }

    pub(super) unsafe fn destroy_semaphore(&self, index: usize) {
        let handle = self.inner.semaphores.lock().remove(index);
        self.inner.logical.destroy_semaphore(handle, None);
    }

    /// Creates new shader module from shader's code.
    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub fn create_shader_module(
        &self,
        info: ShaderModuleInfo,
    ) -> Result<ShaderModule, CreateShaderModuleError> {
        #[allow(unused)]
        let spv: Vec<u32>;

        #[cfg(feature = "naga")]
        let naga_caps = |inner: &Inner| {
            let mut caps = naga::valid::Capabilities::PUSH_CONSTANT;

            let features = &inner.features.v10;

            if features.geometry_shader != 0 {
                caps |= naga::valid::Capabilities::PRIMITIVE_INDEX;
            }

            if features.shader_float64 != 0 {
                caps |= naga::valid::Capabilities::FLOAT64;
            }

            if features.shader_clip_distance != 0 {
                caps |= naga::valid::Capabilities::CLIP_DISTANCE;
            }

            if features.shader_cull_distance != 0 {
                caps |= naga::valid::Capabilities::CULL_DISTANCE;
            }

            let v12 = &inner.properties.v12;
            if v12.shader_sampled_image_array_non_uniform_indexing_native != 0
                && v12.shader_storage_buffer_array_non_uniform_indexing_native != 0
            {
                caps |= naga::valid::Capabilities::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING;
                caps |= naga::valid::Capabilities::SAMPLER_NON_UNIFORM_INDEXING;
            };

            if v12.shader_uniform_buffer_array_non_uniform_indexing_native != 0
                && v12.shader_storage_image_array_non_uniform_indexing_native != 0
            {
                caps |= naga::valid::Capabilities::UNIFORM_BUFFER_AND_STORAGE_TEXTURE_ARRAY_NON_UNIFORM_INDEXING;
            };

            caps | naga::valid::Capabilities::all()
        };

        let code = match info.language {
            ShaderLanguage::SPIRV => &*info.code,

            #[cfg(feature = "glsl")]
            ShaderLanguage::GLSL { stage } => {
                let stage = match stage {
                    ShaderStage::Vertex => naga::ShaderStage::Vertex,
                    ShaderStage::Fragment => naga::ShaderStage::Fragment,
                    ShaderStage::Compute => naga::ShaderStage::Compute,
                    _ => {
                        return Err(CreateShaderModuleError::UnsupportedShaderLanguage {
                            language: info.language,
                        })
                    }
                };

                let code = std::str::from_utf8(&info.code)?;

                let module = naga::front::glsl::Parser::default()
                    .parse(
                        &naga::front::glsl::Options {
                            stage,
                            defines: Default::default(),
                        },
                        code,
                    )
                    .map_err(|errors| {
                        emit_glsl_parser_error(&errors, "source.glsl", code);
                        CreateShaderModuleError::NagaGlslParseError { errors }
                    })?;

                let info = naga::valid::Validator::new(
                    naga::valid::ValidationFlags::all(),
                    naga_caps(&self.inner),
                )
                .validate(&module)
                .map_err(|err| {
                    emit_annotated_error(&err, "source.glsl", code);
                    err
                })?;

                spv = naga::back::spv::write_vec(
                    &module,
                    &info,
                    &naga::back::spv::Options::default(),
                    None,
                )?;

                bytemuck::cast_slice(&spv)
            }
            #[cfg(feature = "wgsl")]
            ShaderLanguage::WGSL => {
                let code = std::str::from_utf8(&info.code)?;
                let module = naga::front::wgsl::parse_str(code).map_err(|err| {
                    err.emit_to_stderr("source.wgsl");
                    CreateShaderModuleError::NagaWgslParseError {
                        source: Box::from(err.emit_to_string(".")),
                    }
                })?;
                let info = naga::valid::Validator::new(
                    naga::valid::ValidationFlags::all(),
                    naga_caps(&self.inner),
                )
                .validate(&module)
                .map_err(|err| {
                    emit_annotated_error(&err, "source.wgsl", code);
                    err
                })?;

                spv = naga::back::spv::write_vec(
                    &module,
                    &info,
                    &naga::back::spv::Options::default(),
                    None,
                )?;

                bytemuck::cast_slice(&spv)
            }
            #[allow(unreachable_patterns)]
            _ => {
                return Err(CreateShaderModuleError::UnsupportedShaderLanguage {
                    language: info.language,
                })
            }
        };

        if code.is_empty() {
            return Err(CreateShaderModuleError::InvalidShader {
                source: InvalidShader::EmptySource,
            });
        }

        if code.len() & 3 > 0 {
            return Err(CreateShaderModuleError::InvalidShader {
                source: InvalidShader::SizeIsNotMultipleOfFour,
            });
        }

        let magic: u32 = unsafe {
            // The size is at least 4 bytes.
            std::ptr::read_unaligned(code.as_ptr() as *const u32)
        };

        if magic != 0x07230203 {
            return Err(CreateShaderModuleError::InvalidShader {
                source: InvalidShader::WrongMagic { found: magic },
            });
        }

        let mut aligned_code;

        let is_aligned = code.as_ptr() as usize & 3 == 0;

        let code_slice = if !is_aligned {
            // Copy spirv code into aligned array.
            unsafe {
                aligned_code = Vec::<u32>::with_capacity(code.len() / 4);

                // Copying array of `u8` into 4 times smaller array of `u32`.
                // They cannot overlap.
                std::ptr::copy_nonoverlapping(
                    code.as_ptr(),
                    aligned_code.as_mut_ptr() as *mut u8,
                    code.len(),
                );

                // Those values are initialized by copy operation above.
                aligned_code.set_len(code.len() / 4);
            }

            &aligned_code[..]
        } else {
            unsafe {
                // As `[u8; 4]` must be compatible with `u32`
                // `[u8; N]` must be compatible with `[u32; N / 4]
                // Resulting lifetime is bound to the function while
                // source lifetime is not less than the function.
                std::slice::from_raw_parts(code.as_ptr() as *const u32, code.len() / 4)
            }
        };

        let module = unsafe {
            // FIXME: It is still required to validate SPIR-V.
            // Othewise adheres to valid usage described in spec.
            self.inner.logical.create_shader_module(
                &vk1_0::ShaderModuleCreateInfoBuilder::new().code(code_slice),
                None,
            )
        }
        .result()
        .map_err(|err| CreateShaderModuleError::OutOfMemoryError {
            source: oom_error_from_erupt(err),
        })?;

        let index = self.inner.shaders.lock().insert(module);

        debug!("Shader module created: {:p}", module);
        Ok(ShaderModule::new(info, self.downgrade(), module, index))
    }

    pub(super) unsafe fn destroy_shader_module(&self, index: usize) {
        let handle = self.inner.shaders.lock().remove(index);
        self.inner.logical.destroy_shader_module(handle, None);
    }

    /// Creates surface for specified window.
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(window, display), fields(?window = window.raw_window_handle())))]
    pub fn create_surface(
        &self,
        window: &impl HasRawWindowHandle,
        display: &impl HasRawDisplayHandle,
    ) -> Result<Surface, CreateSurfaceError> {
        let window = window.raw_window_handle();
        let display = display.raw_display_handle();

        let surface = self.graphics().create_surface(window, display)?;
        Surface::new(surface, SurfaceInfo { window, display }, self)
    }

    pub(super) fn insert_swapchain(&self, swapchain: vksw::SwapchainKHR) -> usize {
        self.inner.swapchains.lock().insert(swapchain)
    }

    pub(super) unsafe fn destroy_swapchain(&self, index: usize) {
        let handle = self.inner.swapchains.lock().remove(index);
        self.inner.logical.destroy_swapchain_khr(handle, None);
    }

    /// Resets fences.
    /// All specified fences must be in signalled state.
    /// Fences are moved into unsignalled state.
    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub fn reset_fences(&self, fences: &mut [&mut Fence]) -> Result<(), DeviceLost> {
        for fence in fences.iter_mut() {
            assert_owner!(fence, self);
        }

        let handles =
            fences
                .iter_mut()
                .try_fold(SmallVec::<[_; 16]>::new(), |mut handles, fence| {
                    if let FenceState::Armed { .. } = fence.state() {
                        // Could be come signalled already.
                        // User may be sure because they called device or queue wait idle method.
                        if !self.is_fence_signalled(fence)? {
                            panic!("Fence must not be reset while associated submission is pending")
                        }
                    }
                    handles.push(fence.handle());
                    Ok::<_, DeviceLost>(handles)
                })?;

        match unsafe { self.inner.logical.reset_fences(&handles) }.result() {
            Ok(()) => {
                for fence in fences {
                    fence.was_reset();
                }
                Ok(())
            }
            Err(vk1_0::Result::ERROR_DEVICE_LOST) => Err(DeviceLost),
            Err(result) => unexpected_result(result),
        }
    }

    /// Checks if fence is in signalled state.
    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub fn is_fence_signalled(&self, fence: &mut Fence) -> Result<bool, DeviceLost> {
        assert_owner!(fence, *self);

        match unsafe { self.inner.logical.get_fence_status(fence.handle()) }.raw {
            vk1_0::Result::SUCCESS => {
                if let Some((queue, epoch)) = fence.signalled() {
                    self.inner.epochs.close_epoch(queue, epoch);
                }
                Ok(true)
            }
            vk1_0::Result::NOT_READY => Ok(false),
            vk1_0::Result::ERROR_DEVICE_LOST => Err(DeviceLost),
            err => unexpected_result(err),
        }
    }

    /// Wait for fences to become signaled.
    /// If `all` is `true` - waits for all specified fences to become signaled.
    /// Otherwise waits for at least on of specified fences to become signaled.
    /// May return immediately if all fences are already signaled (or at least
    /// one is signaled if `all == false`). Fences are signaled by `Queue`s.
    /// See `Queue::submit`.
    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub fn wait_fences(&self, fences: &mut [&mut Fence], all: bool) -> Result<(), DeviceLost> {
        if fences.is_empty() {
            assert!(
                all,
                "Cannot use empty fences array in `Device::wait_fences` with `all == false` as it would wait forever."
            );
            return Ok(());
        }

        for fence in fences.iter_mut() {
            assert_owner!(fence, self);
        }

        let handles = fences
            .iter()
            .filter_map(|fence| match fence.state() {
                FenceState::Signalled => None,
                FenceState::Armed { .. } => Some(fence.handle()),
                FenceState::UnSignalled => {
                    panic!("Unsignalled fences must not be used in wait function")
                }
            })
            .collect::<SmallVec<[_; 16]>>();

        if handles.is_empty() {
            // All fences are already signalled.
            // No epoch can be closed.
            return Ok(());
        }

        match unsafe { self.inner.logical.wait_for_fences(&handles, all, !0) }.result() {
            Ok(()) => {
                let all_signalled = all || handles.len() == 1;

                let mut epochs = SmallVec::<[_; 16]>::new();

                for fence in fences {
                    if all_signalled || self.is_fence_signalled(fence)? {
                        if let Some((queue, epoch)) = fence.signalled() {
                            epochs.push((queue, epoch));
                        }
                    }
                }

                if !epochs.is_empty() {
                    // Dedup. Keep largest epoch per queue.
                    epochs.sort_unstable_by_key(|(q, e)| (*q, !*e));
                    let mut last_queue = None;
                    epochs.retain(|(q, _)| {
                        if Some(*q) == last_queue {
                            false
                        } else {
                            last_queue = Some(*q);
                            true
                        }
                    });
                    for (queue, epoch) in epochs {
                        self.inner.epochs.close_epoch(queue, epoch)
                    }
                }
                Ok(())
            }
            Err(vk1_0::Result::ERROR_DEVICE_LOST) => Err(DeviceLost),
            Err(result) => unexpected_result(result),
        }
    }

    /// Wait for whole device to become idle. That is, wait for all pending
    /// operations to complete. This is equivalent to calling
    /// `Queue::wait_idle` for all queues. Typically used only before device
    /// destruction.
    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub fn wait_idle(&self) -> Result<(), DeviceLost> {
        self.inner.wait_idle()
    }

    /// Returns memory size requirements for accelelration structure build operations.
    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub fn get_acceleration_structure_build_sizes(
        &self,
        level: AccelerationStructureLevel,
        flags: AccelerationStructureBuildFlags,
        geometry: &[AccelerationStructureGeometryInfo],
    ) -> AccelerationStructureBuildSizesInfo {
        assert!(
            self.inner.logical.enabled().khr_acceleration_structure,
            "`AccelerationStructure` feature is not enabled"
        );

        assert!(u32::try_from(geometry.len()).is_ok(), "Too many geometry");

        let geometries = geometry
            .iter()
            .map(|info| match *info {
                AccelerationStructureGeometryInfo::Triangles {
                    index_type,
                    max_vertex_count,
                    vertex_format,
                    allows_transforms,
                    ..
                } => {
                    assert_eq!(
                        level,
                        AccelerationStructureLevel::Bottom,
                        "Triangles must be built into bottom level acceleration structure"
                    );

                    vkacc::AccelerationStructureGeometryKHRBuilder::new()
                        .geometry_type(vkacc::GeometryTypeKHR::TRIANGLES_KHR)
                        .geometry(vkacc::AccelerationStructureGeometryDataKHR {
                            triangles:
                                vkacc::AccelerationStructureGeometryTrianglesDataKHRBuilder::new()
                                    .vertex_format(vertex_format.to_erupt())
                                    .max_vertex(max_vertex_count)
                                    .index_type(match index_type {
                                        Some(IndexType::U16) => vk1_0::IndexType::UINT16,
                                        Some(IndexType::U32) => vk1_0::IndexType::UINT32,
                                        None => vk1_0::IndexType::NONE_KHR,
                                    })
                                    .transform_data(vkacc::DeviceOrHostAddressConstKHR {
                                        device_address: allows_transforms as u64,
                                    })
                                    .build_dangling(),
                        })
                }
                AccelerationStructureGeometryInfo::AABBs { .. } => {
                    assert_eq!(
                        level,
                        AccelerationStructureLevel::Bottom,
                        "AABBs must be built into bottom level acceleration structure"
                    );

                    vkacc::AccelerationStructureGeometryKHRBuilder::new()
                        .geometry_type(vkacc::GeometryTypeKHR::AABBS_KHR)
                        .geometry(vkacc::AccelerationStructureGeometryDataKHR {
                            aabbs: vkacc::AccelerationStructureGeometryAabbsDataKHR::default(),
                        })
                }
                AccelerationStructureGeometryInfo::Instances { .. } => {
                    assert_eq!(
                        level,
                        AccelerationStructureLevel::Top,
                        "Instances must be built into bottom level acceleration structure"
                    );

                    vkacc::AccelerationStructureGeometryKHRBuilder::new()
                        .geometry_type(vkacc::GeometryTypeKHR::INSTANCES_KHR)
                        .geometry(vkacc::AccelerationStructureGeometryDataKHR {
                            instances:
                                vkacc::AccelerationStructureGeometryInstancesDataKHR::default(),
                        })
                }
            })
            .collect::<SmallVec<[_; 4]>>();

        let max_primitive_counts = geometry
            .iter()
            .map(|info| match *info {
                AccelerationStructureGeometryInfo::Triangles {
                    max_primitive_count,
                    ..
                } => max_primitive_count,
                AccelerationStructureGeometryInfo::AABBs {
                    max_primitive_count,
                } => max_primitive_count,
                AccelerationStructureGeometryInfo::Instances {
                    max_primitive_count,
                } => max_primitive_count,
            })
            .collect::<SmallVec<[_; 4]>>();

        let build_info = vkacc::AccelerationStructureBuildGeometryInfoKHRBuilder::new()
            ._type(level.to_erupt())
            .flags(flags.to_erupt())
            .mode(vkacc::BuildAccelerationStructureModeKHR::BUILD_KHR)
            .geometries(&geometries);

        let build_sizes = unsafe {
            self.inner
                .logical
                .get_acceleration_structure_build_sizes_khr(
                    vkacc::AccelerationStructureBuildTypeKHR::DEVICE_KHR,
                    &build_info,
                    &max_primitive_counts,
                )
        };

        AccelerationStructureBuildSizesInfo {
            acceleration_structure_size: build_sizes.acceleration_structure_size,
            update_scratch_size: build_sizes.update_scratch_size,
            build_scratch_size: build_sizes.build_scratch_size,
        }
    }

    /// Creates acceleration structure.
    ///
    /// # Panics
    ///
    /// This method may panic if `Feature::RayTracing` wasn't enabled.
    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub fn create_acceleration_structure(
        &self,
        info: AccelerationStructureInfo,
    ) -> Result<AccelerationStructure, OutOfMemory> {
        assert!(
            self.inner.logical.enabled().khr_acceleration_structure,
            "`AccelerationStructure` feature is not enabled"
        );

        assert_owner!(info.region.buffer, self);

        let handle = unsafe {
            self.inner.logical.create_acceleration_structure_khr(
                &vkacc::AccelerationStructureCreateInfoKHRBuilder::new()
                    ._type(info.level.to_erupt())
                    .offset(info.region.offset)
                    .size(info.region.size)
                    .buffer(info.region.buffer.handle()),
                None,
            )
        }
        .result()
        .map_err(|result| match result {
            vk1_0::Result::ERROR_OUT_OF_HOST_MEMORY => out_of_host_memory(),
            vk1_0::Result::ERROR_INVALID_OPAQUE_CAPTURE_ADDRESS_KHR => {
                panic!("INVALID_OPAQUE_CAPTURE_ADDRESS_KHR error was unexpected")
            }
            _ => unexpected_result(result),
        })?;

        let index = self.inner.acceleration_strucutres.lock().insert(handle);

        let address = Option::unwrap(from_erupt(unsafe {
            self.inner
                .logical
                .get_acceleration_structure_device_address_khr(
                    &vkacc::AccelerationStructureDeviceAddressInfoKHR::default()
                        .into_builder()
                        .acceleration_structure(handle),
                )
        }));

        debug!("AccelerationStructure created {:p}", handle);
        Ok(AccelerationStructure::new(
            info,
            self.downgrade(),
            handle,
            address,
            index,
        ))
    }

    pub(super) unsafe fn destroy_acceleration_structure(&self, index: usize) {
        let handle = self.inner.acceleration_strucutres.lock().remove(index);
        self.inner
            .logical
            .destroy_acceleration_structure_khr(handle, None);
    }

    /// Returns buffers device address.
    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub fn get_buffer_device_address(&self, buffer: &Buffer) -> Option<DeviceAddress> {
        assert_owner!(buffer, self);

        if buffer.info().usage.contains(BufferUsage::DEVICE_ADDRESS) {
            assert_ne!(self.inner.features.v12.buffer_device_address, 0);

            Some(buffer.address().expect(
                "Device address for buffer must be set when `BufferUsage::DEVICE_ADDRESS` is specified",
            ))
        } else {
            None
        }
    }

    /// Returns device address of acceleration strucutre.
    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub fn get_acceleration_structure_device_address(
        &self,
        acceleration_structure: &AccelerationStructure,
    ) -> DeviceAddress {
        assert_owner!(acceleration_structure, self);
        acceleration_structure.address()
    }

    /// Creates ray-tracing pipeline.
    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub fn create_ray_tracing_pipeline(
        &self,
        info: RayTracingPipelineInfo,
    ) -> Result<RayTracingPipeline, OutOfMemory> {
        assert!(
            self.inner.logical.enabled().khr_ray_tracing_pipeline,
            "`RayTracing` feature is not enabled"
        );

        assert_owner!(info.layout, self);

        for shader in &info.shaders {
            assert_owner!(shader.module(), self);
        }

        let entries: Vec<_> = info
            .shaders
            .iter()
            .map(|shader| entry_name_to_cstr(shader.entry()))
            .collect();

        let mut entries = entries.iter();

        let stages: Vec<_> = info
            .shaders
            .iter()
            .map(|shader| {
                vk1_0::PipelineShaderStageCreateInfoBuilder::new()
                    .stage(shader.stage().to_erupt())
                    .module(shader.module.handle())
                    .name(entries.next().unwrap())
            })
            .collect();

        let groups: Vec<_> = info
            .groups
            .iter()
            .map(|group| {
                let builder = vkrt::RayTracingShaderGroupCreateInfoKHRBuilder::new();
                match *group {
                    RayTracingShaderGroupInfo::Raygen { raygen } => {
                        assert_ne!(raygen, vkrt::SHADER_UNUSED_KHR);
                        assert_eq!(
                            usize::try_from(raygen)
                                .ok()
                                .and_then(|raygen| info.shaders.get(raygen))
                                .expect("raygen shader index out of bounds")
                                .stage(),
                            ShaderStage::Raygen
                        );

                        builder
                            ._type(vkrt::RayTracingShaderGroupTypeKHR::GENERAL_KHR)
                            .general_shader(raygen)
                            .any_hit_shader(vkrt::SHADER_UNUSED_KHR)
                            .closest_hit_shader(vkrt::SHADER_UNUSED_KHR)
                            .intersection_shader(vkrt::SHADER_UNUSED_KHR)
                    }
                    RayTracingShaderGroupInfo::Miss { miss } => {
                        assert_ne!(miss, vkrt::SHADER_UNUSED_KHR);
                        assert_eq!(
                            usize::try_from(miss)
                                .ok()
                                .and_then(|miss| info.shaders.get(miss))
                                .expect("miss shader index out of bounds")
                                .stage(),
                            ShaderStage::Miss
                        );

                        builder
                            ._type(vkrt::RayTracingShaderGroupTypeKHR::GENERAL_KHR)
                            .general_shader(miss)
                            .any_hit_shader(vkrt::SHADER_UNUSED_KHR)
                            .closest_hit_shader(vkrt::SHADER_UNUSED_KHR)
                            .intersection_shader(vkrt::SHADER_UNUSED_KHR)
                    }
                    RayTracingShaderGroupInfo::Triangles {
                        any_hit,
                        closest_hit,
                    } => {
                        if let Some(any_hit) = any_hit {
                            assert_ne!(any_hit, vkrt::SHADER_UNUSED_KHR);
                            assert_eq!(
                                usize::try_from(any_hit)
                                    .ok()
                                    .and_then(|any_hit| info.shaders.get(any_hit))
                                    .expect("any_hit shader index out of bounds")
                                    .stage(),
                                ShaderStage::AnyHit
                            );
                        }
                        if let Some(closest_hit) = closest_hit {
                            assert_ne!(closest_hit, vkrt::SHADER_UNUSED_KHR);
                            assert_eq!(
                                usize::try_from(closest_hit)
                                    .ok()
                                    .and_then(|closest_hit| info.shaders.get(closest_hit))
                                    .expect("closest_hit shader index out of bounds")
                                    .stage(),
                                ShaderStage::ClosestHit
                            );
                        }

                        builder
                            ._type(vkrt::RayTracingShaderGroupTypeKHR::TRIANGLES_HIT_GROUP_KHR)
                            .general_shader(vkrt::SHADER_UNUSED_KHR)
                            .any_hit_shader(any_hit.unwrap_or(vkrt::SHADER_UNUSED_KHR))
                            .closest_hit_shader(closest_hit.unwrap_or(vkrt::SHADER_UNUSED_KHR))
                            .intersection_shader(vkrt::SHADER_UNUSED_KHR)
                    }
                }
            })
            .collect();

        let handles = unsafe {
            self.inner.logical.create_ray_tracing_pipelines_khr(
                vkdho::DeferredOperationKHR::null(),
                vk1_0::PipelineCache::null(),
                &[vkrt::RayTracingPipelineCreateInfoKHRBuilder::new()
                    .stages(&stages)
                    .groups(&groups)
                    .max_pipeline_ray_recursion_depth(info.max_recursion_depth)
                    .layout(info.layout.handle())],
                None,
            )
        }
        .result()
        .map_err(oom_error_from_erupt)?;

        assert_eq!(handles.len(), 1);

        let handle = handles[0];

        let group_size = self.inner.properties.rt.shader_group_handle_size;

        let group_size_usize = usize::try_from(group_size).map_err(|_| out_of_host_memory())?;

        #[allow(clippy::redundant_closure)]
        let total_size_usize = group_size_usize
            .checked_mul(info.groups.len())
            .unwrap_or_else(|| host_memory_space_overflow());

        let group_count = u32::try_from(info.groups.len()).map_err(|_| OutOfMemory)?;

        let mut bytes = vec![0u8; total_size_usize];

        unsafe {
            self.inner.logical.get_ray_tracing_shader_group_handles_khr(
                handle,
                0,
                group_count,
                bytes.len(),
                bytes.as_mut_ptr() as *mut _,
            )
        }
        .result()
        .map_err(|err| {
            unsafe { self.inner.logical.destroy_pipeline(handle, None) }

            oom_error_from_erupt(err)
        })?;

        let index = self.inner.pipelines.lock().insert(handle);

        debug!("RayTracingPipeline created {:p}", handle);
        Ok(RayTracingPipeline::new(
            info,
            self.downgrade(),
            handle,
            bytes.into(),
            index,
        ))
    }

    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub fn create_descriptor_set_layout(
        &self,
        info: DescriptorSetLayoutInfo,
    ) -> Result<DescriptorSetLayout, OutOfMemory> {
        let handle = if vk1_0::make_api_version(0, 1, 2, 0) > self.inner.version {
            assert!(
                info.bindings.iter().all(|binding| binding.flags.is_empty()),
                "Vulkan 1.2 is required for non-empty `DescriptorBindingFlags`",
            );

            if info.bindings.iter().any(|binding| {
                binding
                    .flags
                    .contains(DescriptorBindingFlags::UPDATE_AFTER_BIND)
            }) {
                assert!(info
                    .flags
                    .contains(DescriptorSetLayoutFlags::UPDATE_AFTER_BIND_POOL))
            }

            // Is it so?
            // assert!(
            //     info.bindings.iter().all(|binding| binding.count > 0),
            //     "Binding `count` must be greater than 0",
            // );

            // TODO: Validated descriptor count according to physical device properties.

            unsafe {
                self.inner.logical.create_descriptor_set_layout(
                    &vk1_0::DescriptorSetLayoutCreateInfoBuilder::new()
                        .bindings(
                            &info
                                .bindings
                                .iter()
                                .map(|binding| {
                                    vk1_0::DescriptorSetLayoutBindingBuilder::new()
                                        .binding(binding.binding)
                                        .descriptor_count(binding.count)
                                        .descriptor_type(binding.ty.to_erupt())
                                        .stage_flags(binding.stages.to_erupt())
                                })
                                .collect::<SmallVec<[_; 16]>>(),
                        )
                        .flags(info.flags.to_erupt()),
                    None,
                )
            }
        } else {
            let flags = info
                .bindings
                .iter()
                .map(|binding| binding.flags.to_erupt())
                .collect::<SmallVec<[_; 16]>>();

            unsafe {
                let bindings = info
                    .bindings
                    .iter()
                    .map(|binding| {
                        vk1_0::DescriptorSetLayoutBindingBuilder::new()
                            .binding(binding.binding)
                            .descriptor_count(binding.count)
                            .descriptor_type(binding.ty.to_erupt())
                            .stage_flags(binding.stages.to_erupt())
                    })
                    .collect::<SmallVec<[_; 16]>>();
                let mut create_info = vk1_0::DescriptorSetLayoutCreateInfoBuilder::new()
                    .bindings(&bindings)
                    .flags(info.flags.to_erupt());

                let mut flags = vk1_2::DescriptorSetLayoutBindingFlagsCreateInfoBuilder::new()
                    .binding_flags(&flags);

                create_info = create_info.extend_from(&mut flags);

                self.inner
                    .logical
                    .create_descriptor_set_layout(&create_info, None)
            }
        }
        .result()
        .map_err(oom_error_from_erupt)?;

        let index = self.inner.descriptor_set_layouts.lock().insert(handle);

        let total_count = descriptor_count_from_bindings(&info.bindings);

        debug!("DescriptorSetLayout created {:p}", handle);
        Ok(DescriptorSetLayout::new(
            info,
            self.downgrade(),
            handle,
            total_count,
            index,
        ))
    }

    pub(super) unsafe fn destroy_descriptor_set_layout(&self, index: usize) {
        let handle = self.inner.descriptor_set_layouts.lock().remove(index);
        self.inner
            .logical
            .destroy_descriptor_set_layout(handle, None);
    }

    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub fn create_descriptor_set(
        &self,
        info: DescriptorSetInfo,
    ) -> Result<WritableDescriptorSet, DescriptorsAllocationError> {
        assert_owner!(info.layout, self);

        assert!(
            !info
                .layout
                .info()
                .flags
                .contains(DescriptorSetLayoutFlags::PUSH_DESCRIPTOR),
            "Push descriptor sets must not be created. "
        );

        let layout_flags = info.layout.info().flags;
        let mut flags = DescriptorSetLayoutCreateFlags::empty();

        if layout_flags.contains(DescriptorSetLayoutFlags::UPDATE_AFTER_BIND_POOL) {
            flags |= DescriptorSetLayoutCreateFlags::UPDATE_AFTER_BIND;
        }

        let mut sets = unsafe {
            self.inner.descriptor_allocator.lock().allocate(
                EruptDescriptorDevice::wrap(&self.inner.logical),
                &info.layout.handle(),
                flags,
                info.layout.total_count(),
                1,
            )
        }
        .map_err(|err| match err {
            gpu_descriptor::AllocationError::OutOfHostMemory => out_of_host_memory(),
            gpu_descriptor::AllocationError::OutOfDeviceMemory => {
                DescriptorsAllocationError::OutOfMemory {
                    source: OutOfMemory,
                }
            }
            gpu_descriptor::AllocationError::Fragmentation => {
                DescriptorsAllocationError::Fragmentation
            }
        })?;

        let set = sets.remove(0);

        debug!("DescriptorSet created {:?}", set);
        Ok(WritableDescriptorSet::new(info, self.downgrade(), set))
    }

    pub(super) unsafe fn destroy_descriptor_set(
        &self,
        set: gpu_descriptor::DescriptorSet<vk1_0::DescriptorSet>,
    ) {
        self.inner
            .descriptor_allocator
            .lock()
            .free(EruptDescriptorDevice::wrap(&self.inner.logical), Some(set))
    }

    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub fn update_descriptor_sets<'a>(&self, updates: &mut [UpdateDescriptorSet<'a>]) {
        let mut writes_count = 0;

        for update in updates.iter() {
            assert_owner!(update.set, self);
            writes_count += update.writes.len();

            for write in update.writes.iter() {
                match write.descriptors {
                    DescriptorSlice::Sampler(samplers) => {
                        for sampler in samplers {
                            assert_owner!(sampler, self);
                        }
                    }
                    DescriptorSlice::CombinedImageSampler(combos) => {
                        for combo in combos {
                            assert_owner!(combo.view, self);
                            assert_owner!(combo.sampler, self);
                        }
                    }
                    DescriptorSlice::SampledImage(slice)
                    | DescriptorSlice::StorageImage(slice)
                    | DescriptorSlice::InputAttachment(slice) => {
                        for image in slice {
                            assert_owner!(image.0, self);
                        }
                    }
                    DescriptorSlice::UniformBuffer(regions)
                    | DescriptorSlice::StorageBuffer(regions)
                    | DescriptorSlice::UniformBufferDynamic(regions)
                    | DescriptorSlice::StorageBufferDynamic(regions) => {
                        for region in regions {
                            assert_owner!(region.buffer, self);
                            debug_assert_ne!(
                                region.size, 0,
                                "Cannot write 0 sized buffer range into descriptor"
                            );
                            debug_assert!(
                                region.offset <= region.buffer.info().size,
                                "Buffer ({:#?}) descriptor offset ({}) is out of bounds",
                                region.buffer,
                                region.offset,
                            );
                            debug_assert!(
                                region.size <= region.buffer.info().size - region.offset,
                                "Buffer ({:#?}) descriptor size ({}) is out of bounds",
                                region.buffer,
                                region.size
                            );
                        }
                    }
                    DescriptorSlice::AccelerationStructure(acceleration_structures) => {
                        for acceleration_structure in acceleration_structures {
                            assert_owner!(acceleration_structure, self);
                            assert_eq!(
                                acceleration_structure.info().level,
                                AccelerationStructureLevel::Top
                            );
                        }
                    }
                    DescriptorSlice::UniformTexelBuffer(views) => {
                        for view in views {
                            assert_owner!(view, self);
                        }
                    }
                    DescriptorSlice::StorageTexelBuffer(views) => {
                        for view in views {
                            assert_owner!(view, self);
                        }
                    }
                }
            }

            if !update.copies.is_empty() {
                unimplemented!()
            }
        }

        if writes_count == 0 {
            return;
        }

        let mut ranges = SmallVec::<[_; 64]>::new();

        let mut images = SmallVec::<[_; 16]>::new();

        let mut buffers = SmallVec::<[_; 16]>::new();

        let mut buffer_views = SmallVec::<[_; 16]>::new();

        // let mut buffer_views = SmallVec::<[_; 16]
        let mut acceleration_structures = SmallVec::<[_; 64]>::new();

        let mut write_descriptor_acceleration_structures = SmallVec::<[_; 16]>::new();

        for update in updates.iter() {
            for write in update.writes.iter() {
                match write.descriptors {
                    DescriptorSlice::Sampler(slice) => {
                        let start = images.len();

                        images.extend(slice.iter().map(|sampler| {
                            vk1_0::DescriptorImageInfoBuilder::new().sampler(sampler.handle())
                        }));

                        ranges.push(start..images.len());
                    }
                    DescriptorSlice::CombinedImageSampler(slice) => {
                        let start = images.len();

                        images.extend(slice.iter().map(|combo| {
                            vk1_0::DescriptorImageInfoBuilder::new()
                                .sampler(combo.sampler.handle())
                                .image_view(combo.view.handle())
                                .image_layout(combo.layout.to_erupt())
                        }));

                        ranges.push(start..images.len());
                    }
                    DescriptorSlice::SampledImage(slice) => {
                        let start = images.len();

                        images.extend(slice.iter().map(|(image, layout)| {
                            vk1_0::DescriptorImageInfoBuilder::new()
                                .image_view(image.handle())
                                .image_layout(layout.to_erupt())
                        }));

                        ranges.push(start..images.len());
                    }
                    DescriptorSlice::StorageImage(slice) => {
                        let start = images.len();

                        images.extend(slice.iter().map(|(image, layout)| {
                            vk1_0::DescriptorImageInfoBuilder::new()
                                .image_view(image.handle())
                                .image_layout(layout.to_erupt())
                        }));

                        ranges.push(start..images.len());
                    }
                    DescriptorSlice::UniformBuffer(slice) => {
                        let start = buffers.len();

                        buffers.extend(slice.iter().map(|region| {
                            vk1_0::DescriptorBufferInfoBuilder::new()
                                .buffer(region.buffer.handle())
                                .offset(region.offset)
                                .range(region.size)
                        }));

                        ranges.push(start..buffers.len());
                    }
                    DescriptorSlice::StorageBuffer(slice) => {
                        let start = buffers.len();

                        buffers.extend(slice.iter().map(|region| {
                            vk1_0::DescriptorBufferInfoBuilder::new()
                                .buffer(region.buffer.handle())
                                .offset(region.offset)
                                .range(region.size)
                        }));

                        ranges.push(start..buffers.len());
                    }
                    DescriptorSlice::UniformBufferDynamic(slice) => {
                        let start = buffers.len();

                        buffers.extend(slice.iter().map(|region| {
                            vk1_0::DescriptorBufferInfoBuilder::new()
                                .buffer(region.buffer.handle())
                                .offset(region.offset)
                                .range(region.size)
                        }));

                        ranges.push(start..buffers.len());
                    }
                    DescriptorSlice::StorageBufferDynamic(slice) => {
                        let start = buffers.len();

                        buffers.extend(slice.iter().map(|region| {
                            vk1_0::DescriptorBufferInfoBuilder::new()
                                .buffer(region.buffer.handle())
                                .offset(region.offset)
                                .range(region.size)
                        }));

                        ranges.push(start..buffers.len());
                    }
                    DescriptorSlice::UniformTexelBuffer(slice) => {
                        let start = buffer_views.len();

                        buffer_views.extend(slice.iter().map(|view| view.handle()));

                        ranges.push(start..buffer_views.len());
                    }

                    DescriptorSlice::StorageTexelBuffer(slice) => {
                        let start = buffer_views.len();

                        buffer_views.extend(slice.iter().map(|view| view.handle()));

                        ranges.push(start..buffer_views.len());
                    }
                    DescriptorSlice::InputAttachment(slice) => {
                        let start = images.len();

                        images.extend(slice.iter().map(|(image, layout)| {
                            vk1_0::DescriptorImageInfoBuilder::new()
                                .image_view(image.handle())
                                .image_layout(layout.to_erupt())
                        }));

                        ranges.push(start..images.len());
                    }
                    DescriptorSlice::AccelerationStructure(slice) => {
                        let start = acceleration_structures.len();

                        acceleration_structures.extend(slice.iter().map(|accs| accs.handle()));

                        ranges.push(start..acceleration_structures.len());

                        write_descriptor_acceleration_structures
                            .push(vkacc::WriteDescriptorSetAccelerationStructureKHRBuilder::new());
                    }
                }
            }
        }

        let mut ranges = ranges.into_iter();

        let mut write_descriptor_acceleration_structures =
            write_descriptor_acceleration_structures.iter_mut();

        let mut erupt_writes: SmallVec<[_; 16]> = SmallVec::with_capacity(writes_count);

        for update in updates.iter() {
            for write in update.writes {
                let builder = vk1_0::WriteDescriptorSetBuilder::new()
                    .dst_set(update.set.handle())
                    .dst_binding(write.binding)
                    .dst_array_element(write.element);

                let write = match write.descriptors {
                    DescriptorSlice::Sampler(_) => builder
                        .descriptor_type(vk1_0::DescriptorType::SAMPLER)
                        .image_info(&images[ranges.next().unwrap()]),
                    DescriptorSlice::CombinedImageSampler(_) => builder
                        .descriptor_type(vk1_0::DescriptorType::COMBINED_IMAGE_SAMPLER)
                        .image_info(&images[ranges.next().unwrap()]),
                    DescriptorSlice::SampledImage(_) => builder
                        .descriptor_type(vk1_0::DescriptorType::SAMPLED_IMAGE)
                        .image_info(&images[ranges.next().unwrap()]),
                    DescriptorSlice::StorageImage(_) => builder
                        .descriptor_type(vk1_0::DescriptorType::STORAGE_IMAGE)
                        .image_info(&images[ranges.next().unwrap()]),
                    DescriptorSlice::UniformTexelBuffer(_) => builder
                        .descriptor_type(vk1_0::DescriptorType::UNIFORM_TEXEL_BUFFER)
                        .texel_buffer_view(&buffer_views[ranges.next().unwrap()]),
                    DescriptorSlice::StorageTexelBuffer(_) => builder
                        .descriptor_type(vk1_0::DescriptorType::STORAGE_TEXEL_BUFFER)
                        .texel_buffer_view(&buffer_views[ranges.next().unwrap()]),
                    DescriptorSlice::UniformBuffer(_) => builder
                        .descriptor_type(vk1_0::DescriptorType::UNIFORM_BUFFER)
                        .buffer_info(&buffers[ranges.next().unwrap()]),
                    DescriptorSlice::StorageBuffer(_) => builder
                        .descriptor_type(vk1_0::DescriptorType::STORAGE_BUFFER)
                        .buffer_info(&buffers[ranges.next().unwrap()]),
                    DescriptorSlice::UniformBufferDynamic(_) => builder
                        .descriptor_type(vk1_0::DescriptorType::UNIFORM_BUFFER_DYNAMIC)
                        .buffer_info(&buffers[ranges.next().unwrap()]),
                    DescriptorSlice::StorageBufferDynamic(_) => builder
                        .descriptor_type(vk1_0::DescriptorType::STORAGE_BUFFER_DYNAMIC)
                        .buffer_info(&buffers[ranges.next().unwrap()]),
                    DescriptorSlice::InputAttachment(_) => builder
                        .descriptor_type(vk1_0::DescriptorType::INPUT_ATTACHMENT)
                        .image_info(&images[ranges.next().unwrap()]),
                    DescriptorSlice::AccelerationStructure(_) => {
                        let range = ranges.next().unwrap();
                        let mut write = builder
                            .descriptor_type(vk1_0::DescriptorType::ACCELERATION_STRUCTURE_KHR);
                        write.descriptor_count = range.len() as u32;

                        let acc_structure_write =
                            write_descriptor_acceleration_structures.next().unwrap();

                        *acc_structure_write =
                            vkacc::WriteDescriptorSetAccelerationStructureKHRBuilder::new()
                                .acceleration_structures(&acceleration_structures[range]);
                        write.extend_from(acc_structure_write)
                    }
                };

                erupt_writes.push(write);
            }
        }

        for update in updates {
            for write in update.writes {
                update
                    .set
                    .write_descriptors(write.binding, write.element, write.descriptors);
            }
        }

        unsafe {
            self.inner
                .logical
                .update_descriptor_sets(&erupt_writes, &[])
        }
    }

    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub fn create_sampler(&self, info: SamplerInfo) -> Result<Sampler, OutOfMemory> {
        match self.inner.samplers_cache.lock().entry(info) {
            Entry::Occupied(entry) => Ok(entry.get().clone()),
            Entry::Vacant(entry) => {
                let handle = unsafe {
                    self.inner.logical.create_sampler(
                        &vk1_0::SamplerCreateInfoBuilder::new()
                            .mag_filter(info.mag_filter.to_erupt())
                            .min_filter(info.min_filter.to_erupt())
                            .mipmap_mode(info.mipmap_mode.to_erupt())
                            .address_mode_u(info.address_mode_u.to_erupt())
                            .address_mode_v(info.address_mode_v.to_erupt())
                            .address_mode_w(info.address_mode_w.to_erupt())
                            .mip_lod_bias(info.mip_lod_bias)
                            .anisotropy_enable(info.max_anisotropy.is_some())
                            .max_anisotropy(info.max_anisotropy.unwrap_or(0.0))
                            .compare_enable(info.compare_op.is_some())
                            .compare_op(match info.compare_op {
                                Some(compare_op) => compare_op.to_erupt(),
                                None => vk1_0::CompareOp::NEVER,
                            })
                            .min_lod(info.min_lod)
                            .max_lod(info.max_lod)
                            .border_color(info.border_color.to_erupt())
                            .unnormalized_coordinates(info.unnormalized_coordinates),
                        None,
                    )
                }
                .result()
                .map_err(oom_error_from_erupt)?;

                let index = self.inner.samplers.lock().insert(handle);

                debug!("Sampler created {:p}", handle);
                let sampler = Sampler::new(info, self.downgrade(), handle, index);
                Ok(entry.insert(sampler).clone())
            }
        }
    }

    pub(super) unsafe fn destroy_sampler(&self, index: usize) {
        let handle = self.inner.samplers.lock().remove(index);
        self.inner.logical.destroy_sampler(handle, None);
    }

    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub fn create_shader_binding_table(
        &self,
        pipeline: &RayTracingPipeline,
        info: ShaderBindingTableInfo,
    ) -> Result<ShaderBindingTable, OutOfMemory> {
        assert_owner!(pipeline, self);

        let group_size = u64::from(self.inner.properties.rt.shader_group_handle_size);
        let group_align = u64::from(self.inner.properties.rt.shader_group_base_alignment - 1);

        let group_count_usize =
            info.raygen.is_some() as usize + info.miss.len() + info.hit.len() + info.callable.len();

        let group_count = u32::try_from(group_count_usize).map_err(|_| OutOfMemory)?;

        let group_stride = align_up(group_align, group_size).ok_or(OutOfMemory)?;

        let group_stride_usize = usize::try_from(group_stride).map_err(|_| OutOfMemory)?;

        let total_size = (group_stride.checked_mul(u64::from(group_count))).ok_or(OutOfMemory)?;

        let total_size_usize = usize::try_from(total_size).unwrap_or_else(|_| out_of_host_memory());

        let mut bytes = vec![0; total_size_usize];

        let mut write_offset = 0;

        let group_handlers = pipeline.group_handlers();

        let raygen_handlers = copy_group_handlers(
            group_handlers,
            &mut bytes,
            info.raygen.iter().copied(),
            &mut write_offset,
            group_size,
            group_stride_usize,
        );

        let miss_handlers = copy_group_handlers(
            group_handlers,
            &mut bytes,
            info.miss.iter().copied(),
            &mut write_offset,
            group_size,
            group_stride_usize,
        );

        let hit_handlers = copy_group_handlers(
            group_handlers,
            &mut bytes,
            info.hit.iter().copied(),
            &mut write_offset,
            group_size,
            group_stride_usize,
        );

        let callable_handlers = copy_group_handlers(
            group_handlers,
            &mut bytes,
            info.callable.iter().copied(),
            &mut write_offset,
            group_size,
            group_stride_usize,
        );

        let buffer = self.create_buffer_static(
            BufferInfo {
                align: group_align,
                size: total_size,
                usage: BufferUsage::SHADER_BINDING_TABLE | BufferUsage::DEVICE_ADDRESS,
            },
            &bytes,
        )?;

        debug!("ShaderBindingTable created");
        Ok(ShaderBindingTable {
            raygen: raygen_handlers.map(|range| StridedBufferRange {
                range: BufferRange {
                    buffer: buffer.clone(),
                    offset: range.start,
                    size: range.end - range.start,
                },
                stride: group_stride,
            }),

            miss: miss_handlers.map(|range| StridedBufferRange {
                range: BufferRange {
                    buffer: buffer.clone(),
                    offset: range.start,
                    size: range.end - range.start,
                },
                stride: group_stride,
            }),

            hit: hit_handlers.map(|range| StridedBufferRange {
                range: BufferRange {
                    buffer: buffer.clone(),
                    offset: range.start,
                    size: range.end - range.start,
                },
                stride: group_stride,
            }),

            callable: callable_handlers.map(|range| StridedBufferRange {
                range: BufferRange {
                    buffer: buffer.clone(),
                    offset: range.start,
                    size: range.end - range.start,
                },
                stride: group_stride,
            }),
        })
    }

    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub fn map_memory(
        &self,
        buffer: &mut MappableBuffer,
        offset: u64,
        size: usize,
    ) -> Result<&mut [MaybeUninit<u8>], MapError> {
        assert_owner!(buffer, self);

        Ok(unsafe {
            let ptr = buffer.memory_block().map(
                EruptMemoryDevice::wrap(&self.inner.logical),
                offset,
                size,
            )?;
            std::slice::from_raw_parts_mut(ptr.as_ptr() as _, size)
        })
    }

    pub fn unmap_memory(&self, buffer: &mut MappableBuffer) -> bool {
        assert_owner!(buffer, self);
        unsafe {
            buffer
                .memory_block()
                .unmap(EruptMemoryDevice::wrap(&self.inner.logical))
        }
    }

    pub fn upload_to_memory<T>(
        &self,
        buffer: &mut MappableBuffer,
        offset: u64,
        data: &[T],
    ) -> Result<(), MapError>
    where
        T: Pod,
    {
        let slice = self.map_memory(buffer, offset, size_of_val(data))?;

        unsafe {
            std::ptr::copy_nonoverlapping(
                data.as_ptr() as *const u8,
                slice.as_mut_ptr() as *mut u8,
                size_of_val(data),
            );
        }

        self.unmap_memory(buffer);

        Ok(())
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(skip(data)))]
    pub fn write_buffer<T>(
        &self,
        buffer: &mut MappableBuffer,
        offset: u64,
        data: &[T],
    ) -> Result<(), MapError>
    where
        T: Pod,
    {
        assert_owner!(buffer, self);

        if size_of_val(data) == 0 {
            return Ok(());
        }

        unsafe {
            buffer.memory_block().write_bytes(
                EruptMemoryDevice::wrap(&self.inner.logical),
                offset,
                bytemuck::cast_slice(data),
            )
        }
        .map_err(Into::into)
    }
}

#[allow(dead_code)]
fn check() {
    assert_object::<Device>();
}

fn entry_name_to_cstr(name: &str) -> CString {
    CString::new(name.as_bytes()).expect("Shader names should not contain zero bytes")
}

fn copy_group_handlers(
    group_handlers: &[u8],
    write: &mut [u8],
    group_indices: impl IntoIterator<Item = u32>,
    write_offset: &mut usize,
    group_size: u64,
    group_stride: usize,
) -> Option<Range<u64>> {
    let result_start = u64::try_from(*write_offset).ok()?;
    let group_size_usize = usize::try_from(group_size).ok()?;

    for group_index in group_indices {
        let group_offset = (group_size_usize.checked_mul(usize::try_from(group_index).ok()?))?;

        let group_end = group_offset.checked_add(group_size_usize)?;
        let write_end = write_offset.checked_add(group_size_usize)?;

        let group_range = group_offset..group_end;
        let write_range = *write_offset..write_end;

        let handler = &group_handlers[group_range];
        let output = &mut write[write_range];

        output.copy_from_slice(handler);
        *write_offset = write_offset.checked_add(group_stride)?;
    }

    let result_end = u64::try_from(*write_offset).ok()?;
    Some(result_start..result_end)
}

pub(crate) fn create_render_pass_error_from_erupt(err: vk1_0::Result) -> CreateRenderPassError {
    match err {
        vk1_0::Result::ERROR_OUT_OF_HOST_MEMORY => out_of_host_memory(),
        vk1_0::Result::ERROR_OUT_OF_DEVICE_MEMORY => CreateRenderPassError::OutOfMemory {
            source: OutOfMemory,
        },
        _ => unexpected_result(err),
    }
}

fn memory_device_properties(
    properties: &Properties,
    features: &Features,
) -> gpu_alloc::DeviceProperties<'static> {
    let memory_properties = &properties.memory;
    let limits = &properties.v10.limits;

    gpu_alloc::DeviceProperties {
        max_memory_allocation_count: limits.max_memory_allocation_count,
        max_memory_allocation_size: u64::max_value(), // FIXME: Can query this information if instance is v1.1

        non_coherent_atom_size: limits.non_coherent_atom_size,
        memory_types: memory_properties.memory_types
            [..memory_properties.memory_type_count as usize]
            .iter()
            .map(|memory_type| gpu_alloc::MemoryType {
                props: gpu_alloc_erupt::memory_properties_from_erupt(memory_type.property_flags),
                heap: memory_type.heap_index,
            })
            .collect(),
        memory_heaps: memory_properties.memory_heaps
            [..memory_properties.memory_heap_count as usize]
            .iter()
            .map(|&memory_heap| gpu_alloc::MemoryHeap {
                size: memory_heap.size,
            })
            .collect(),
        buffer_device_address: features.v12.buffer_device_address != 0,
    }
}

pub(super) fn descriptor_count_from_bindings(
    bindings: &[DescriptorSetLayoutBinding],
) -> DescriptorTotalCount {
    let mut result = DescriptorTotalCount::default();

    for binding in bindings {
        match binding.ty {
            DescriptorType::AccelerationStructure => result.acceleration_structure += binding.count,
            DescriptorType::CombinedImageSampler => result.combined_image_sampler += binding.count,
            DescriptorType::InputAttachment => result.input_attachment += binding.count,
            DescriptorType::SampledImage => result.sampled_image += binding.count,
            DescriptorType::StorageImage => result.storage_image += binding.count,
            DescriptorType::Sampler => result.sampler += binding.count,
            DescriptorType::UniformTexelBuffer => result.uniform_texel_buffer += binding.count,
            DescriptorType::StorageTexelBuffer => result.storage_texel_buffer += binding.count,
            DescriptorType::UniformBuffer => result.uniform_buffer += binding.count,
            DescriptorType::StorageBuffer => result.storage_buffer += binding.count,
            DescriptorType::UniformBufferDynamic => result.uniform_buffer_dynamic += binding.count,
            DescriptorType::StorageBufferDynamic => result.storage_buffer_dynamic += binding.count,
        }
    }

    result
}

#[cfg(feature = "glsl")]
fn emit_glsl_parser_error(errors: &[naga::front::glsl::Error], filename: &str, source: &str) {
    let files = SimpleFile::new(filename, source);
    let config = codespan_reporting::term::Config::default();
    let writer = StandardStream::stderr(ColorChoice::Auto);

    for err in errors {
        let mut diagnostic = Diagnostic::error().with_message(err.kind.to_string());

        if let Some(range) = err.meta.to_range() {
            diagnostic = diagnostic.with_labels(vec![Label::primary((), range)]);
        }

        let mut writer = writer.lock();

        if let Err(err) = codespan_reporting::term::emit(&mut writer, &config, &files, &diagnostic)
        {
            error!("Failed to print annotated error. {:#}", err);
        }
    }
}

#[cfg(any(feature = "glsl", feature = "wgsl"))]
fn emit_annotated_error<E: std::error::Error>(ann_err: &WithSpan<E>, filename: &str, source: &str) {
    let files = SimpleFile::new(filename, source);
    let config = codespan_reporting::term::Config::default();
    let writer = StandardStream::stderr(ColorChoice::Auto);

    let diagnostic = Diagnostic::error().with_labels(
        ann_err
            .spans()
            .map(|(span, desc)| {
                Label::primary((), span.to_range().unwrap()).with_message(desc.to_owned())
            })
            .collect(),
    );

    let mut writer = writer.lock();

    if let Err(err) = codespan_reporting::term::emit(&mut writer, &config, &files, &diagnostic) {
        error!("Failed to print annotated error. {:#}", err);
    }
}
