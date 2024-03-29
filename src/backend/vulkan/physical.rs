use std::{convert::TryInto as _, ffi::CStr};

use erupt::{
    extensions::{
        ext_descriptor_indexing::EXT_DESCRIPTOR_INDEXING_EXTENSION_NAME,
        ext_scalar_block_layout::EXT_SCALAR_BLOCK_LAYOUT_EXTENSION_NAME,
        google_display_timing::GOOGLE_DISPLAY_TIMING_EXTENSION_NAME,
        // khr_16bit_storage::KHR_16BIT_STORAGE_EXTENSION_NAME,
        // khr_8bit_storage::KHR_8BIT_STORAGE_EXTENSION_NAME,
        khr_acceleration_structure::{self as acc, KHR_ACCELERATION_STRUCTURE_EXTENSION_NAME},
        khr_buffer_device_address::KHR_BUFFER_DEVICE_ADDRESS_EXTENSION_NAME,
        khr_deferred_host_operations::KHR_DEFERRED_HOST_OPERATIONS_EXTENSION_NAME,
        khr_dynamic_rendering::KHR_DYNAMIC_RENDERING_EXTENSION_NAME,
        // khr_pipeline_library::KHR_PIPELINE_LIBRARY_EXTENSION_NAME,
        // khr_push_descriptor::KHR_PUSH_DESCRIPTOR_EXTENSION_NAME,
        khr_ray_tracing_pipeline::{self as rt, KHR_RAY_TRACING_PIPELINE_EXTENSION_NAME},
        khr_swapchain::KHR_SWAPCHAIN_EXTENSION_NAME,
    },
    vk1_0, vk1_1, vk1_2, vk1_3, DeviceLoader, ExtendableFrom, LoaderError, ObjectHandle,
};
use hashbrown::HashMap;
use smallvec::SmallVec;

use crate::{
    arith_gt, assert_object, out_of_host_memory,
    physical::*,
    queue::{Family, FamilyInfo, Queue, QueueId, QueuesQuery},
    CreateDeviceError, OutOfMemory,
};

use super::{convert::from_erupt, device::Device, graphics::Graphics, unexpected_result};

#[derive(Clone, Debug)]
pub(super) struct Properties {
    pub extension: SmallVec<[vk1_0::ExtensionProperties; 8]>,
    pub family: SmallVec<[vk1_0::QueueFamilyProperties; 8]>,
    pub memory: vk1_0::PhysicalDeviceMemoryProperties,

    pub v10: vk1_0::PhysicalDeviceProperties,
    pub v11: vk1_2::PhysicalDeviceVulkan11Properties,
    pub v12: vk1_2::PhysicalDeviceVulkan12Properties,
    pub v13: vk1_3::PhysicalDeviceVulkan13Properties,

    pub acc: acc::PhysicalDeviceAccelerationStructurePropertiesKHR,
    pub rt: rt::PhysicalDeviceRayTracingPipelinePropertiesKHR,
}

// Not auto-implemented because of raw pointer in fields.
// Dereferencing said pointer requires `unsafe` and shouldn't be performed.
unsafe impl Sync for Properties {}
unsafe impl Send for Properties {}

#[derive(Clone, Debug)]
pub(super) struct Features {
    pub v10: vk1_0::PhysicalDeviceFeatures,
    pub v11: vk1_2::PhysicalDeviceVulkan11Features,
    pub v12: vk1_2::PhysicalDeviceVulkan12Features,
    pub v13: vk1_3::PhysicalDeviceVulkan13Features,

    pub acc: acc::PhysicalDeviceAccelerationStructureFeaturesKHR,
    pub rt: rt::PhysicalDeviceRayTracingPipelineFeaturesKHR,
}

// Not auto-implemented because of raw pointer in fields.
// Dereferencing said pointer requires `unsafe` and shouldn't be performed.
unsafe impl Sync for Features {}
unsafe impl Send for Features {}

unsafe fn collect_properties_and_features(
    physical: vk1_0::PhysicalDevice,
) -> (Properties, Features) {
    let graphics = Graphics::get_unchecked();

    let extension_properties = graphics
        .instance
        .enumerate_device_extension_properties(physical, None, None)
        .expect("OOM on initialization");

    let has_extension = |name| -> bool {
        let name = CStr::from_ptr(name);
        extension_properties
            .iter()
            .any(|p| CStr::from_ptr(&p.extension_name[0]) == name)
    };

    let properties10;
    let mut properties11 = vk1_2::PhysicalDeviceVulkan11PropertiesBuilder::new();
    let mut properties12 = vk1_2::PhysicalDeviceVulkan12PropertiesBuilder::new();
    let mut properties13 = vk1_3::PhysicalDeviceVulkan13PropertiesBuilder::new();
    let mut properties_edi = vk1_2::PhysicalDeviceDescriptorIndexingPropertiesBuilder::new();
    let mut properties_rt = rt::PhysicalDeviceRayTracingPipelinePropertiesKHRBuilder::new();
    let mut properties_acc = acc::PhysicalDeviceAccelerationStructurePropertiesKHRBuilder::new();

    let features10;
    let mut features11 = vk1_2::PhysicalDeviceVulkan11FeaturesBuilder::new();
    let mut features12 = vk1_2::PhysicalDeviceVulkan12FeaturesBuilder::new();
    let mut features13 = vk1_3::PhysicalDeviceVulkan13FeaturesBuilder::new();
    let mut features_sbl = vk1_2::PhysicalDeviceScalarBlockLayoutFeaturesBuilder::new();
    let mut features_edi = vk1_2::PhysicalDeviceDescriptorIndexingFeaturesBuilder::new();
    let mut features_bda = vk1_2::PhysicalDeviceBufferDeviceAddressFeaturesBuilder::new();
    let mut features_acc = acc::PhysicalDeviceAccelerationStructureFeaturesKHRBuilder::new();
    let mut features_rt = rt::PhysicalDeviceRayTracingPipelineFeaturesKHRBuilder::new();
    let mut features_dr = vk1_3::PhysicalDeviceDynamicRenderingFeaturesBuilder::new();

    if graphics.instance.enabled().vk1_1
        || graphics
            .instance
            .enabled()
            .khr_get_physical_device_properties2
    {
        let mut properties2 = vk1_1::PhysicalDeviceProperties2Builder::new();
        let mut features2 = vk1_1::PhysicalDeviceFeatures2Builder::new();

        if graphics.instance.enabled().vk1_1 {
            properties2 = properties2.extend_from(&mut properties11);
            features2 = features2.extend_from(&mut features11);
        }

        if graphics.instance.enabled().vk1_2 {
            properties2 = properties2.extend_from(&mut properties12);
            features2 = features2.extend_from(&mut features12);
        }

        if graphics.instance.enabled().vk1_3 {
            properties2 = properties2.extend_from(&mut properties13);
            features2 = features2.extend_from(&mut features13);
        }

        if !graphics.instance.enabled().vk1_2
            && has_extension(EXT_SCALAR_BLOCK_LAYOUT_EXTENSION_NAME)
        {
            features2 = features2.extend_from(&mut features_sbl);
        }

        if !graphics.instance.enabled().vk1_2
            && has_extension(EXT_DESCRIPTOR_INDEXING_EXTENSION_NAME)
        {
            features2 = features2.extend_from(&mut features_edi);
            properties2 = properties2.extend_from(&mut properties_edi);
        }

        if !graphics.instance.enabled().vk1_2
            && has_extension(KHR_BUFFER_DEVICE_ADDRESS_EXTENSION_NAME)
        {
            features2 = features2.extend_from(&mut features_bda);
        }

        if !graphics.instance.enabled().vk1_3 && has_extension(KHR_DYNAMIC_RENDERING_EXTENSION_NAME)
        {
            features2 = features2.extend_from(&mut features_dr);
        }

        if has_extension(KHR_ACCELERATION_STRUCTURE_EXTENSION_NAME) {
            properties2 = properties2.extend_from(&mut properties_acc);
            features2 = features2.extend_from(&mut features_acc);
        }

        if has_extension(KHR_RAY_TRACING_PIPELINE_EXTENSION_NAME) {
            properties2 = properties2.extend_from(&mut properties_rt);
            features2 = features2.extend_from(&mut features_rt);
        }

        graphics
            .instance
            .get_physical_device_properties2(physical, &mut properties2);

        graphics
            .instance
            .get_physical_device_features2(physical, &mut features2);

        properties10 = properties2.properties;
        features10 = features2.features;
    } else {
        properties10 = graphics.instance.get_physical_device_properties(physical);
        features10 = graphics.instance.get_physical_device_features(physical);
    }

    let family_properties = graphics
        .instance
        .get_physical_device_queue_family_properties(physical, None);

    let memory_properties = graphics
        .instance
        .get_physical_device_memory_properties(physical);

    if !graphics.instance.enabled().vk1_2 && has_extension(EXT_SCALAR_BLOCK_LAYOUT_EXTENSION_NAME) {
        features12.scalar_block_layout = features_sbl.scalar_block_layout;
    }

    if !graphics.instance.enabled().vk1_2 && has_extension(EXT_DESCRIPTOR_INDEXING_EXTENSION_NAME) {
        properties12.max_update_after_bind_descriptors_in_all_pools =
            properties_edi.max_update_after_bind_descriptors_in_all_pools;
        properties12.shader_uniform_buffer_array_non_uniform_indexing_native =
            properties_edi.shader_uniform_buffer_array_non_uniform_indexing_native;
        properties12.shader_sampled_image_array_non_uniform_indexing_native =
            properties_edi.shader_sampled_image_array_non_uniform_indexing_native;
        properties12.shader_storage_buffer_array_non_uniform_indexing_native =
            properties_edi.shader_storage_buffer_array_non_uniform_indexing_native;
        properties12.shader_storage_image_array_non_uniform_indexing_native =
            properties_edi.shader_storage_image_array_non_uniform_indexing_native;
        properties12.shader_input_attachment_array_non_uniform_indexing_native =
            properties_edi.shader_input_attachment_array_non_uniform_indexing_native;
        properties12.robust_buffer_access_update_after_bind =
            properties_edi.robust_buffer_access_update_after_bind;
        properties12.quad_divergent_implicit_lod = properties_edi.quad_divergent_implicit_lod;
        properties12.max_per_stage_descriptor_update_after_bind_samplers =
            properties_edi.max_per_stage_descriptor_update_after_bind_samplers;
        properties12.max_per_stage_descriptor_update_after_bind_uniform_buffers =
            properties_edi.max_per_stage_descriptor_update_after_bind_uniform_buffers;
        properties12.max_per_stage_descriptor_update_after_bind_storage_buffers =
            properties_edi.max_per_stage_descriptor_update_after_bind_storage_buffers;
        properties12.max_per_stage_descriptor_update_after_bind_sampled_images =
            properties_edi.max_per_stage_descriptor_update_after_bind_sampled_images;
        properties12.max_per_stage_descriptor_update_after_bind_storage_images =
            properties_edi.max_per_stage_descriptor_update_after_bind_storage_images;
        properties12.max_per_stage_descriptor_update_after_bind_input_attachments =
            properties_edi.max_per_stage_descriptor_update_after_bind_input_attachments;
        properties12.max_per_stage_update_after_bind_resources =
            properties_edi.max_per_stage_update_after_bind_resources;
        properties12.max_descriptor_set_update_after_bind_samplers =
            properties_edi.max_descriptor_set_update_after_bind_samplers;
        properties12.max_descriptor_set_update_after_bind_uniform_buffers =
            properties_edi.max_descriptor_set_update_after_bind_uniform_buffers;
        properties12.max_descriptor_set_update_after_bind_uniform_buffers_dynamic =
            properties_edi.max_descriptor_set_update_after_bind_uniform_buffers_dynamic;
        properties12.max_descriptor_set_update_after_bind_storage_buffers =
            properties_edi.max_descriptor_set_update_after_bind_storage_buffers;
        properties12.max_descriptor_set_update_after_bind_storage_buffers_dynamic =
            properties_edi.max_descriptor_set_update_after_bind_storage_buffers_dynamic;
        properties12.max_descriptor_set_update_after_bind_sampled_images =
            properties_edi.max_descriptor_set_update_after_bind_sampled_images;
        properties12.max_descriptor_set_update_after_bind_storage_images =
            properties_edi.max_descriptor_set_update_after_bind_storage_images;
        properties12.max_descriptor_set_update_after_bind_input_attachments =
            properties_edi.max_descriptor_set_update_after_bind_input_attachments;

        features12.shader_input_attachment_array_dynamic_indexing =
            features_edi.shader_input_attachment_array_dynamic_indexing;
        features12.shader_uniform_texel_buffer_array_dynamic_indexing =
            features_edi.shader_uniform_texel_buffer_array_dynamic_indexing;
        features12.shader_storage_texel_buffer_array_dynamic_indexing =
            features_edi.shader_storage_texel_buffer_array_dynamic_indexing;
        features12.shader_uniform_buffer_array_non_uniform_indexing =
            features_edi.shader_uniform_buffer_array_non_uniform_indexing;
        features12.shader_sampled_image_array_non_uniform_indexing =
            features_edi.shader_sampled_image_array_non_uniform_indexing;
        features12.shader_storage_buffer_array_non_uniform_indexing =
            features_edi.shader_storage_buffer_array_non_uniform_indexing;
        features12.shader_storage_image_array_non_uniform_indexing =
            features_edi.shader_storage_image_array_non_uniform_indexing;
        features12.shader_input_attachment_array_non_uniform_indexing =
            features_edi.shader_input_attachment_array_non_uniform_indexing;
        features12.shader_uniform_texel_buffer_array_non_uniform_indexing =
            features_edi.shader_uniform_texel_buffer_array_non_uniform_indexing;
        features12.shader_storage_texel_buffer_array_non_uniform_indexing =
            features_edi.shader_storage_texel_buffer_array_non_uniform_indexing;
        features12.descriptor_binding_uniform_buffer_update_after_bind =
            features_edi.descriptor_binding_uniform_buffer_update_after_bind;
        features12.descriptor_binding_sampled_image_update_after_bind =
            features_edi.descriptor_binding_sampled_image_update_after_bind;
        features12.descriptor_binding_storage_image_update_after_bind =
            features_edi.descriptor_binding_storage_image_update_after_bind;
        features12.descriptor_binding_storage_buffer_update_after_bind =
            features_edi.descriptor_binding_storage_buffer_update_after_bind;
        features12.descriptor_binding_uniform_texel_buffer_update_after_bind =
            features_edi.descriptor_binding_uniform_texel_buffer_update_after_bind;
        features12.descriptor_binding_storage_texel_buffer_update_after_bind =
            features_edi.descriptor_binding_storage_texel_buffer_update_after_bind;
        features12.descriptor_binding_update_unused_while_pending =
            features_edi.descriptor_binding_update_unused_while_pending;
        features12.descriptor_binding_partially_bound =
            features_edi.descriptor_binding_partially_bound;
        features12.descriptor_binding_variable_descriptor_count =
            features_edi.descriptor_binding_variable_descriptor_count;
        features12.runtime_descriptor_array = features_edi.runtime_descriptor_array;
    }

    if !graphics.instance.enabled().vk1_2 && has_extension(KHR_BUFFER_DEVICE_ADDRESS_EXTENSION_NAME)
    {
        features12.buffer_device_address = features_bda.buffer_device_address;
        features12.buffer_device_address_capture_replay =
            features_bda.buffer_device_address_capture_replay;
        features12.buffer_device_address_multi_device =
            features_bda.buffer_device_address_multi_device;
    }

    if !graphics.instance.enabled().vk1_3 && has_extension(KHR_DYNAMIC_RENDERING_EXTENSION_NAME) {
        features13.dynamic_rendering = features_dr.dynamic_rendering;
    }

    let mut properties = Properties {
        extension: extension_properties,
        family: family_properties,
        memory: memory_properties,
        v10: properties10,
        v11: properties11.build_dangling(),
        v12: properties12.build_dangling(),
        v13: properties13.build_dangling(),
        acc: properties_acc.build_dangling(),
        rt: properties_rt.build_dangling(),
    };

    let mut features = Features {
        v10: features10,
        v11: features11.build_dangling(),
        v12: features12.build_dangling(),
        v13: features13.build_dangling(),
        acc: features_acc.build_dangling(),
        rt: features_rt.build_dangling(),
    };

    properties.v11.p_next = std::ptr::null_mut();
    properties.v12.p_next = std::ptr::null_mut();
    properties.v13.p_next = std::ptr::null_mut();
    properties.acc.p_next = std::ptr::null_mut();
    properties.rt.p_next = std::ptr::null_mut();
    features.v11.p_next = std::ptr::null_mut();
    features.v12.p_next = std::ptr::null_mut();
    features.v13.p_next = std::ptr::null_mut();
    features.acc.p_next = std::ptr::null_mut();
    features.rt.p_next = std::ptr::null_mut();

    (properties, features)
}

impl Properties {
    pub(crate) fn has_extension(&self, name: &CStr) -> bool {
        self.extension
            .iter()
            .any(|p| unsafe { CStr::from_ptr(&p.extension_name[0]) } == name)
    }
}

/// Opaque value representing a device (software emulated of hardware).
/// Can be used to fetch information about device,
/// its support of the surface and create graphics device.
#[derive(Debug)]
pub struct PhysicalDevice {
    physical: vk1_0::PhysicalDevice,
    properties: Properties,
    features: Features,
}

impl PhysicalDevice {
    pub(crate) unsafe fn new(physical: vk1_0::PhysicalDevice) -> Self {
        let (properties, features) = collect_properties_and_features(physical);
        info!("{:#?}", properties);

        PhysicalDevice {
            properties,
            features,
            physical,
        }
    }

    pub(crate) fn graphics(&self) -> &'static Graphics {
        unsafe {
            // PhysicalDevice can be created only via Graphics instance.
            Graphics::get_unchecked()
        }
    }

    /// Returns information about this device.
    pub fn info(&self) -> DeviceInfo {
        let mut features = Vec::new();

        if self.features.v12.scalar_block_layout > 0 {
            features.push(Feature::ScalarBlockLayout);
        }

        if self.features.v12.runtime_descriptor_array > 0 {
            features.push(Feature::RuntimeDescriptorArray);
        }

        if self
            .features
            .v12
            .descriptor_binding_uniform_buffer_update_after_bind
            > 0
        {
            features.push(Feature::DescriptorBindingUniformBufferUpdateAfterBind);
        }
        if self
            .features
            .v12
            .descriptor_binding_sampled_image_update_after_bind
            > 0
        {
            features.push(Feature::DescriptorBindingSampledImageUpdateAfterBind);
        }
        if self
            .features
            .v12
            .descriptor_binding_storage_image_update_after_bind
            > 0
        {
            features.push(Feature::DescriptorBindingStorageImageUpdateAfterBind);
        }
        if self
            .features
            .v12
            .descriptor_binding_storage_buffer_update_after_bind
            > 0
        {
            features.push(Feature::DescriptorBindingStorageBufferUpdateAfterBind);
        }
        if self
            .features
            .v12
            .descriptor_binding_uniform_texel_buffer_update_after_bind
            > 0
        {
            features.push(Feature::DescriptorBindingUniformTexelBufferUpdateAfterBind);
        }
        if self
            .features
            .v12
            .descriptor_binding_storage_texel_buffer_update_after_bind
            > 0
        {
            features.push(Feature::DescriptorBindingStorageTexelBufferUpdateAfterBind);
        }
        if self
            .features
            .v12
            .descriptor_binding_update_unused_while_pending
            > 0
        {
            features.push(Feature::DescriptorBindingUpdateUnusedWhilePending);
        }
        if self.features.v12.descriptor_binding_partially_bound > 0 {
            features.push(Feature::DescriptorBindingPartiallyBound);
        }
        if self.features.v12.buffer_device_address > 0 {
            features.push(Feature::BufferDeviceAddress);
        }

        if self.features.v13.dynamic_rendering != 0 {
            features.push(Feature::DynamicRendering);
        }

        if self.features.acc.acceleration_structure != 0 {
            assert!(features.contains(&Feature::BufferDeviceAddress));
            features.push(Feature::AccelerationStructure);
        }

        if self.features.rt.ray_tracing_pipeline != 0 {
            assert!(features.contains(&Feature::AccelerationStructure));
            features.push(Feature::RayTracingPipeline);
        }

        if self.graphics().instance.enabled().khr_surface
            && self
                .properties
                .has_extension(unsafe { CStr::from_ptr(KHR_SWAPCHAIN_EXTENSION_NAME) })
        {
            features.push(Feature::SurfacePresentation);

            if self
                .properties
                .has_extension(unsafe { CStr::from_ptr(GOOGLE_DISPLAY_TIMING_EXTENSION_NAME) })
            {
                features.push(Feature::DisplayTiming);
            }
        }

        DeviceInfo {
            kind: match self.properties.v10.device_type {
                vk1_0::PhysicalDeviceType::INTEGRATED_GPU => Some(DeviceKind::Integrated),
                vk1_0::PhysicalDeviceType::DISCRETE_GPU => Some(DeviceKind::Discrete),
                vk1_0::PhysicalDeviceType::CPU => Some(DeviceKind::Software),
                vk1_0::PhysicalDeviceType::OTHER | vk1_0::PhysicalDeviceType::VIRTUAL_GPU => None,
                _ => {
                    error!(
                        "Unexpected device type value: {:?}",
                        self.properties.v10.device_type
                    );
                    None
                }
            },
            name: unsafe {
                assert!(
                    self.properties.v10.device_name.contains(&0),
                    "Valid C string expected"
                );

                CStr::from_ptr(&self.properties.v10.device_name[0])
            }
            .to_string_lossy()
            .into_owned(),
            features,
            families: self
                .properties
                .family
                .iter()
                .map(|f| FamilyInfo {
                    count: f
                        .queue_count
                        .try_into()
                        .expect("More families than memory size"),
                    capabilities: from_erupt(f.queue_flags),
                })
                .collect(),
        }
    }

    /// Create graphics API device.
    ///
    /// `features` - device will enable specifeid features.
    ///     Only features listed in `DeviceInfo` returned from `self.info()` can
    /// be specified here.     Otherwise device creation will fail.
    ///
    /// `queues` - specifies `QueuesQuery` object which will query device and
    /// initialize command queues.  
    ///  Returns initialized device and queues.
    /// Type in which queues are returned depends on type of queues query,
    /// it may be single queue, an array of queues, struct, anything.
    ///
    /// Note. `QueuesQuery` may be implemented by user, this trait is not
    /// sealed.
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(self, queues)))]
    pub fn create_device<Q>(
        self,
        features: &[Feature],
        queues: Q,
    ) -> Result<(Device, Q::Queues), CreateDeviceError<Q::Error>>
    where
        Q: QueuesQuery,
    {
        let (query, collector) = queues
            .query(&self.info().families)
            .map_err(|source| CreateDeviceError::CannotFindRequeredQueues { source })?;

        let families = query.as_ref();

        trace!("Creating device");

        let mut device_create_info = vk1_0::DeviceCreateInfoBuilder::new();

        // Convert features into cunsumable type.
        // Before creating device all features must be consumed.
        // Not-consumed features are not-supported.
        let mut requested_features = RequestedFeatures::new(features);

        // Process requested families array.
        let mut families_requested = HashMap::new();

        for &(family, count) in families {
            if self.properties.family.len() <= family {
                return Err(CreateDeviceError::BadFamiliesRequested);
            }

            let priorities = families_requested.entry(family).or_insert_with(Vec::new);

            if arith_gt(
                priorities.len() + count,
                self.properties.family[family].queue_count,
            ) {
                return Err(CreateDeviceError::BadFamiliesRequested);
            }

            priorities.resize(priorities.len() + count, 1.0f32);
        }

        let device_queue_create_infos = families_requested
            .iter()
            .map(|(&index, priorities)| {
                vk1_0::DeviceQueueCreateInfoBuilder::new()
                    .queue_family_index(
                        index
                            .try_into()
                            .expect("More families than bytes in memory space"),
                    )
                    .queue_priorities(priorities)
            })
            .collect::<Vec<_>>();

        device_create_info = device_create_info.queue_create_infos(&device_queue_create_infos);

        // Collect requested features.
        let mut features2 = vk1_1::PhysicalDeviceFeatures2Builder::new();
        let mut features11 = vk1_2::PhysicalDeviceVulkan11FeaturesBuilder::new();
        let mut features12 = vk1_2::PhysicalDeviceVulkan12FeaturesBuilder::new();
        let mut features13 = vk1_3::PhysicalDeviceVulkan13FeaturesBuilder::new();
        let mut features_sbl = vk1_2::PhysicalDeviceScalarBlockLayoutFeaturesBuilder::new();
        let mut features_edi = vk1_2::PhysicalDeviceDescriptorIndexingFeaturesBuilder::new();
        let mut features_bda = vk1_2::PhysicalDeviceBufferDeviceAddressFeaturesBuilder::new();
        let mut features_acc = acc::PhysicalDeviceAccelerationStructureFeaturesKHRBuilder::new();
        let mut features_rt = rt::PhysicalDeviceRayTracingPipelineFeaturesKHRBuilder::new();
        let mut features_dr = vk1_3::PhysicalDeviceDynamicRenderingFeaturesBuilder::new();

        let include_features11 = false;
        let mut include_features12 = false;
        let mut include_features13 = false;

        let mut include_features_sbl = false;
        let mut include_features_edi = false;
        let mut include_features_bda = false;
        let mut include_features_acc = false;
        let mut include_features_rt = false;
        let mut include_features_dr = false;

        // Enable requested extensions.
        let mut enable_exts = SmallVec::<[_; 10]>::new();

        let mut push_ext = |name| {
            let name = unsafe { CStr::from_ptr(name) };
            assert!(
                self.properties.has_extension(name),
                "Extension {:?} is missing",
                name
            );

            if !enable_exts.contains(&name.as_ptr()) {
                enable_exts.push(name.as_ptr());
            }
        };

        if requested_features.take(Feature::SurfacePresentation) {
            push_ext(KHR_SWAPCHAIN_EXTENSION_NAME);
        }

        if requested_features.take(Feature::ScalarBlockLayout) {
            if self.features.v12.scalar_block_layout > 0 {
                features12.scalar_block_layout = 1;
                features_sbl.scalar_block_layout = 1;
                include_features12 = true;
                include_features_sbl = true;
            } else {
                panic!("Attempt to enable unsupported feature `ScalarBlockLayout`");
            }
        }

        if requested_features.take(Feature::BufferDeviceAddress) {
            if self.features.v12.buffer_device_address > 0 {
                features12.buffer_device_address = 1;
                features_bda.buffer_device_address = 1;
                include_features12 = true;
                include_features_bda = true;
            } else {
                panic!("Attempt to enable unsupported feature `BufferDeviceAddress`");
            }
        }

        if requested_features.take(Feature::AccelerationStructure) {
            assert_ne!(
                self.features.acc.acceleration_structure, 0,
                "Attempt to enable unsupported feature `AccelerationStructure`"
            );

            assert_ne!(
                features12.buffer_device_address, 0,
                "`BufferDeviceAddress` feature must be enabled when `AccelerationStructure` feature is enabled"
            );

            features_acc.acceleration_structure = 1;
            include_features_acc = true;
        }

        if requested_features.take(Feature::RayTracingPipeline) {
            assert_ne!(
                self.features.rt.ray_tracing_pipeline, 0,
                "Attempt to enable unsupported feature `RayTracing`"
            );
            assert_ne!(
                features_acc.acceleration_structure, 0,
                "`AccelerationStructure` feature must be enabled when `RayTracingPipeline` feature is enabled"
            );
            features_rt.ray_tracing_pipeline = 1;
            include_features_rt = true;
        }

        if requested_features.take(Feature::RuntimeDescriptorArray) {
            if self.features.v12.runtime_descriptor_array != 0 {
                features12.runtime_descriptor_array = 1;
                features_edi.runtime_descriptor_array = 1;
                include_features12 = true;
                include_features_edi = true;
            } else {
                panic!("Attempt to enable unsupported feature `RuntimeDescriptorArray`");
            }
        }

        if requested_features.take(Feature::DescriptorBindingUniformBufferUpdateAfterBind) {
            if self
                .features
                .v12
                .descriptor_binding_uniform_buffer_update_after_bind
                > 0
            {
                features12.descriptor_binding_uniform_buffer_update_after_bind = 1;
                include_features12 = true;
                include_features_edi = true;
            } else {
                panic!("Attempt to enable unsupported feature `DescriptorBindingUniformBufferUpdateAfterBind`")
            }
        }
        if requested_features.take(Feature::DescriptorBindingSampledImageUpdateAfterBind) {
            if self
                .features
                .v12
                .descriptor_binding_sampled_image_update_after_bind
                > 0
            {
                features12.descriptor_binding_sampled_image_update_after_bind = 1;
                features_edi.descriptor_binding_sampled_image_update_after_bind = 1;
                include_features12 = true;
                include_features_edi = true;
            } else {
                panic!("Attempt to enable unsupported feature `DescriptorBindingSampledImageUpdateAfterBind`")
            }
        }
        if requested_features.take(Feature::DescriptorBindingStorageImageUpdateAfterBind) {
            if self
                .features
                .v12
                .descriptor_binding_storage_image_update_after_bind
                > 0
            {
                features12.descriptor_binding_storage_image_update_after_bind = 1;
                features_edi.descriptor_binding_storage_image_update_after_bind = 1;
                include_features12 = true;
                include_features_edi = true;
            } else {
                panic!("Attempt to enable unsupported feature `DescriptorBindingStorageImageUpdateAfterBind`")
            }
        }
        if requested_features.take(Feature::DescriptorBindingStorageBufferUpdateAfterBind) {
            if self
                .features
                .v12
                .descriptor_binding_storage_buffer_update_after_bind
                > 0
            {
                features12.descriptor_binding_storage_buffer_update_after_bind = 1;
                features_edi.descriptor_binding_storage_buffer_update_after_bind = 1;
                include_features_edi = true;
                include_features12 = true;
            } else {
                panic!("Attempt to enable unsupported feature `DescriptorBindingStorageBufferUpdateAfterBind`")
            }
        }
        if requested_features.take(Feature::DescriptorBindingUniformTexelBufferUpdateAfterBind) {
            if self
                .features
                .v12
                .descriptor_binding_uniform_texel_buffer_update_after_bind
                > 0
            {
                features12.descriptor_binding_uniform_texel_buffer_update_after_bind = 1;
                features_edi.descriptor_binding_uniform_texel_buffer_update_after_bind = 1;
                include_features_edi = true;
                include_features12 = true;
            } else {
                panic!("Attempt to enable unsupported feature `DescriptorBindingUniformTexelBufferUpdateAfterBind`")
            }
        }
        if requested_features.take(Feature::DescriptorBindingStorageTexelBufferUpdateAfterBind) {
            if self
                .features
                .v12
                .descriptor_binding_storage_texel_buffer_update_after_bind
                > 0
            {
                features12.descriptor_binding_storage_texel_buffer_update_after_bind = 1;
                features_edi.descriptor_binding_storage_texel_buffer_update_after_bind = 1;
                include_features_edi = true;
                include_features12 = true;
            } else {
                panic!("Attempt to enable unsupported feature `DescriptorBindingStorageTexelBufferUpdateAfterBind`")
            }
        }
        if requested_features.take(Feature::DescriptorBindingUpdateUnusedWhilePending) {
            if self
                .features
                .v12
                .descriptor_binding_update_unused_while_pending
                > 0
            {
                features12.descriptor_binding_update_unused_while_pending = 1;
                features_edi.descriptor_binding_update_unused_while_pending = 1;
                include_features_edi = true;
                include_features12 = true;
            } else {
                panic!("Attempt to enable unsupported feature `DescriptorBindingUpdateUnusedWhilePending`")
            }
        }
        if requested_features.take(Feature::DescriptorBindingPartiallyBound) {
            if self.features.v12.descriptor_binding_partially_bound > 0 {
                features12.descriptor_binding_partially_bound = 1;
                features_edi.descriptor_binding_partially_bound = 1;
                include_features_edi = true;
                include_features12 = true;
            } else {
                panic!("Attempt to enable unsupported feature `DescriptorBindingPartiallyBound`")
            }
        }

        if requested_features.take(Feature::ShaderSampledImageNonUniformIndexing) {
            assert!(requested_features.check(Feature::ShaderSampledImageDynamicIndexing));
            if self
                .features
                .v12
                .shader_sampled_image_array_non_uniform_indexing
                > 0
            {
                features12.shader_sampled_image_array_non_uniform_indexing = 1;
                features_edi.shader_sampled_image_array_non_uniform_indexing = 1;
                include_features_edi = true;
                include_features12 = true;
            } else {
                panic!(
                    "Attempt to enable unsupported feature `ShaderSampledImageNonUniformIndexing`"
                )
            }
        }
        if requested_features.take(Feature::ShaderSampledImageDynamicIndexing) {
            assert_ne!(
                self.features
                    .v10
                    .shader_sampled_image_array_dynamic_indexing,
                0,
                "Attempt to enable unsupported feature `ShaderSampledImageDynamicIndexing`"
            );
            features2
                .features
                .shader_sampled_image_array_dynamic_indexing = 1;
        }
        if requested_features.take(Feature::ShaderStorageImageNonUniformIndexing) {
            assert!(requested_features.check(Feature::ShaderStorageImageDynamicIndexing));
            if self
                .features
                .v12
                .shader_storage_image_array_non_uniform_indexing
                > 0
            {
                features12.shader_storage_image_array_non_uniform_indexing = 1;
                features_edi.shader_storage_image_array_non_uniform_indexing = 1;
                include_features_edi = true;
                include_features12 = true;
            } else {
                panic!(
                    "Attempt to enable unsupported feature `ShaderStorageImageNonUniformIndexing`"
                )
            }
        }
        if requested_features.take(Feature::ShaderStorageImageDynamicIndexing) {
            assert_ne!(
                self.features
                    .v10
                    .shader_storage_image_array_dynamic_indexing,
                0,
                "Attempt to enable unsupported feature `ShaderStorageImageDynamicIndexing`"
            );
            features2
                .features
                .shader_storage_image_array_dynamic_indexing = 1;
        }
        if requested_features.take(Feature::ShaderUniformBufferNonUniformIndexing) {
            assert!(requested_features.check(Feature::ShaderUniformBufferDynamicIndexing));
            if self
                .features
                .v12
                .shader_uniform_buffer_array_non_uniform_indexing
                > 0
            {
                features12.shader_uniform_buffer_array_non_uniform_indexing = 1;
                features_edi.shader_uniform_buffer_array_non_uniform_indexing = 1;
                include_features_edi = true;
                include_features12 = true;
            } else {
                panic!(
                    "Attempt to enable unsupported feature `ShaderUniformBufferNonUniformIndexing`"
                )
            }
        }
        if requested_features.take(Feature::ShaderUniformBufferDynamicIndexing) {
            assert_ne!(
                self.features
                    .v10
                    .shader_uniform_buffer_array_dynamic_indexing,
                0,
                "Attempt to enable unsupported feature `ShaderUniformBufferDynamicIndexing`"
            );
            features2
                .features
                .shader_uniform_buffer_array_dynamic_indexing = 1;
        }
        if requested_features.take(Feature::ShaderStorageBufferNonUniformIndexing) {
            assert!(requested_features.check(Feature::ShaderStorageBufferDynamicIndexing));
            if self
                .features
                .v12
                .shader_storage_buffer_array_non_uniform_indexing
                > 0
            {
                features12.shader_storage_buffer_array_non_uniform_indexing = 1;
                features_edi.shader_storage_buffer_array_non_uniform_indexing = 1;
                include_features12 = true;
                include_features_edi = true;
            } else {
                panic!(
                    "Attempt to enable unsupported feature `ShaderStorageBufferNonUniformIndexing`"
                )
            }
        }
        if requested_features.take(Feature::ShaderStorageBufferDynamicIndexing) {
            assert_ne!(
                self.features
                    .v10
                    .shader_storage_buffer_array_dynamic_indexing,
                0,
                "Attempt to enable unsupported feature `ShaderStorageBufferDynamicIndexing`"
            );
            features2
                .features
                .shader_storage_buffer_array_dynamic_indexing = 1;
        }
        if requested_features.take(Feature::DisplayTiming) {
            push_ext(GOOGLE_DISPLAY_TIMING_EXTENSION_NAME);
        }
        if requested_features.take(Feature::DynamicRendering) {
            assert_ne!(
                self.features.v13.dynamic_rendering, 0,
                "Attempt to enable unsupported feature `DynamicRendering`"
            );
            features13.dynamic_rendering = 1;
            features_dr.dynamic_rendering = 1;
            include_features13 = true;
            include_features_dr = true;
        }

        device_create_info = device_create_info.enabled_features(&features2.features);

        if self.graphics().instance.enabled().vk1_1 {
        } else {
            assert!(!include_features11);
            assert!(!include_features_rt);
        }
        if self.graphics().instance.enabled().vk1_2 {
            include_features_sbl = false;
            include_features_edi = false;
            include_features_bda = false;
        } else {
            include_features12 = false;
        }
        if self.graphics().instance.enabled().vk1_3 {
            include_features_dr = false;
        } else {
            include_features13 = false;
        }

        // Push structure to the list if at least one feature is enabled.
        if include_features_sbl {
            push_ext(EXT_SCALAR_BLOCK_LAYOUT_EXTENSION_NAME);
            device_create_info = device_create_info.extend_from(&mut features_sbl);
        }

        if include_features_edi {
            push_ext(EXT_DESCRIPTOR_INDEXING_EXTENSION_NAME);
            device_create_info = device_create_info.extend_from(&mut features_edi);
        }

        if include_features_bda {
            push_ext(KHR_BUFFER_DEVICE_ADDRESS_EXTENSION_NAME);
            device_create_info = device_create_info.extend_from(&mut features_bda);
        }

        if include_features_acc {
            push_ext(KHR_DEFERRED_HOST_OPERATIONS_EXTENSION_NAME);
            push_ext(KHR_ACCELERATION_STRUCTURE_EXTENSION_NAME);
            device_create_info = device_create_info.extend_from(&mut features_acc);
        }

        if include_features_rt {
            push_ext(KHR_RAY_TRACING_PIPELINE_EXTENSION_NAME);
            device_create_info = device_create_info.extend_from(&mut features_rt);
        }

        if include_features_dr {
            push_ext(KHR_DYNAMIC_RENDERING_EXTENSION_NAME);
            device_create_info = device_create_info.extend_from(&mut features_dr);
        }

        if include_features13 {
            device_create_info = device_create_info.extend_from(&mut features13);
        }

        if include_features12 {
            device_create_info = device_create_info.extend_from(&mut features12);
        }

        if include_features11 {
            device_create_info = device_create_info.extend_from(&mut features11);
        }

        device_create_info = device_create_info.enabled_extension_names(&enable_exts);

        // Ensure all features were consumed.
        requested_features.assert_empty();

        let instance = &self.graphics().instance;

        let result = unsafe { DeviceLoader::new(instance, self.physical, &device_create_info) };

        let logical = match result {
            Err(LoaderError::SymbolNotAvailable) => {
                return Err(CreateDeviceError::FunctionLoadFailed);
            }
            Err(LoaderError::VulkanError(vk1_0::Result::ERROR_OUT_OF_HOST_MEMORY)) => {
                out_of_host_memory()
            }
            Err(LoaderError::VulkanError(vk1_0::Result::ERROR_OUT_OF_DEVICE_MEMORY)) => {
                return Err(OutOfMemory.into())
            }
            Err(LoaderError::VulkanError(err)) => unexpected_result(err),
            Ok(ok) => ok,
        };

        let family_properties = self.properties.family.clone();

        let version = self.graphics().version;

        // Wrap device.
        let device = Device::new(
            logical,
            self.physical,
            self.properties,
            Features {
                v10: features2.features,
                v11: features11.build_dangling(),
                v12: features12.build_dangling(),
                v13: features13.build_dangling(),
                acc: features_acc.build_dangling(),
                rt: features_rt.build_dangling(),
            },
            version,
            families.iter().flat_map(|&(family, count)| {
                (0..count).map(move |index| {
                    let index = index.try_into().unwrap();
                    let family = family.try_into().unwrap();
                    QueueId { family, index }
                })
            }),
        );

        // Wrap families.
        let families = families
            .iter()
            .map(|&(family, count)| {
                let capabilities = from_erupt(family_properties[family].queue_flags);

                Family {
                    capabilities,
                    queues: (0..count)
                        .map(|index| {
                            let index = index.try_into().unwrap();
                            let family = family.try_into().unwrap();
                            let queue = unsafe { device.logical().get_device_queue(index, family) };

                            Queue::new(
                                queue,
                                vk1_0::CommandPool::null(),
                                device.clone(),
                                QueueId { family, index },
                                capabilities,
                            )
                        })
                        .collect(),
                }
            })
            .collect();

        debug!("Device created");

        Ok((device, Q::collect(collector, families)))
    }
}

struct RequestedFeatures {
    array: Vec<Feature>,
}

impl RequestedFeatures {
    fn new(features: &[Feature]) -> Self {
        RequestedFeatures {
            array: features.to_vec(),
        }
    }

    fn take(&mut self, feature: Feature) -> bool {
        if let Some(index) = self.array.iter().position(|&f| f == feature) {
            self.array.swap_remove(index);

            true
        } else {
            false
        }
    }

    fn check(&self, feature: Feature) -> bool {
        self.array.contains(&feature)
    }

    fn assert_empty(self) {
        assert!(
            self.array.is_empty(),
            "Features: {:#?} are unsupported",
            &self.array
        );
    }
}

#[allow(dead_code)]
fn check() {
    assert_object::<PhysicalDevice>();
}
