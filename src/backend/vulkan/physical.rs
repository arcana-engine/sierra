use {
    super::{
        convert::from_erupt, device::Device, graphics::Graphics, surface::surface_error_from_erupt,
        unexpected_result,
    },
    crate::{
        arith_gt, assert_object, out_of_host_memory,
        physical::*,
        queue::{Family, FamilyInfo, Queue, QueueId, QueuesQuery},
        surface::{Surface, SurfaceCapabilities, SurfaceError},
        CreateDeviceError, OutOfMemory,
    },
    erupt::{
        extensions::{
            // khr_16bit_storage::KHR_16BIT_STORAGE_EXTENSION_NAME,
            // khr_8bit_storage::KHR_8BIT_STORAGE_EXTENSION_NAME,
            khr_acceleration_structure::{
                self as vkacc, KHR_ACCELERATION_STRUCTURE_EXTENSION_NAME,
            },
            khr_deferred_host_operations::KHR_DEFERRED_HOST_OPERATIONS_EXTENSION_NAME,
            // khr_pipeline_library::KHR_PIPELINE_LIBRARY_EXTENSION_NAME,
            // khr_push_descriptor::KHR_PUSH_DESCRIPTOR_EXTENSION_NAME,
            khr_ray_tracing_pipeline::{self as vkrt, KHR_RAY_TRACING_PIPELINE_EXTENSION_NAME},
            khr_swapchain::KHR_SWAPCHAIN_EXTENSION_NAME,
        },
        vk1_0, vk1_1, vk1_2, DeviceLoader, ExtendableFrom as _, LoaderError,
    },
    smallvec::SmallVec,
    std::{collections::HashMap, convert::TryInto as _, ffi::CStr},
};

#[derive(Clone, Debug)]
pub(crate) struct Properties {
    pub(crate) extension: Vec<vk1_0::ExtensionProperties>,
    pub(crate) family: Vec<vk1_0::QueueFamilyProperties>,
    pub(crate) memory: vk1_0::PhysicalDeviceMemoryProperties,

    pub(crate) v10: vk1_0::PhysicalDeviceProperties,
    pub(crate) v11: vk1_2::PhysicalDeviceVulkan11Properties,
    pub(crate) v12: vk1_2::PhysicalDeviceVulkan12Properties,
    pub(crate) acc: vkacc::PhysicalDeviceAccelerationStructurePropertiesKHR,
    pub(crate) rt: vkrt::PhysicalDeviceRayTracingPipelinePropertiesKHR,
}

// Not auto-implemented because of raw pointer in fields.
// Dereferencing said pointer requires `unsafe` and shouldn't be performed.
unsafe impl Sync for Properties {}
unsafe impl Send for Properties {}

#[derive(Clone, Debug)]
pub(crate) struct Features {
    pub(crate) v10: vk1_0::PhysicalDeviceFeatures,
    pub(crate) v11: vk1_2::PhysicalDeviceVulkan11Features,
    pub(crate) v12: vk1_2::PhysicalDeviceVulkan12Features,
    pub(crate) acc: vkacc::PhysicalDeviceAccelerationStructureFeaturesKHR,
    pub(crate) rt: vkrt::PhysicalDeviceRayTracingPipelineFeaturesKHR,
}

// Not auto-implemented because of raw pointer in fields.
// Dereferencing said pointer requires `unsafe` and shouldn't be performed.
unsafe impl Sync for Features {}
unsafe impl Send for Features {}

unsafe fn collect_propeties_and_features(
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
    let mut properties_rt = vkrt::PhysicalDeviceRayTracingPipelinePropertiesKHRBuilder::new();
    let mut properties_acc = vkacc::PhysicalDeviceAccelerationStructurePropertiesKHRBuilder::new();
    let features10;
    let mut features11 = vk1_2::PhysicalDeviceVulkan11FeaturesBuilder::new();
    let mut features12 = vk1_2::PhysicalDeviceVulkan12FeaturesBuilder::new();
    let mut features_acc = vkacc::PhysicalDeviceAccelerationStructureFeaturesKHRBuilder::new();
    let mut features_rt = vkrt::PhysicalDeviceRayTracingPipelineFeaturesKHRBuilder::new();

    if graphics.version >= vk1_0::make_version(1, 1, 0) {
        let mut properties2 = vk1_1::PhysicalDeviceProperties2Builder::new();
        let mut features2 = vk1_1::PhysicalDeviceFeatures2Builder::new();

        properties2 = properties2.extend_from(&mut properties11);
        features2 = features2.extend_from(&mut features11);

        if graphics.version >= vk1_0::make_version(1, 2, 0) {
            properties2 = properties2.extend_from(&mut properties12);
            features2 = features2.extend_from(&mut features12);
        }

        if has_extension(KHR_ACCELERATION_STRUCTURE_EXTENSION_NAME) {
            properties2 = properties2.extend_from(&mut properties_acc);
            features2 = features2.extend_from(&mut features_acc);
        }

        if has_extension(KHR_RAY_TRACING_PIPELINE_EXTENSION_NAME) {
            properties2 = properties2.extend_from(&mut properties_rt);
            features2 = features2.extend_from(&mut features_rt);
        }

        *properties2 = graphics
            .instance
            .get_physical_device_properties2(physical, Some(*properties2));

        *features2 = graphics
            .instance
            .get_physical_device_features2(physical, Some(*features2));

        properties10 = properties2.properties;
        features10 = features2.features;
    } else {
        properties10 = graphics
            .instance
            .get_physical_device_properties(physical, None);

        features10 = graphics
            .instance
            .get_physical_device_features(physical, None);
    }

    let family_properties = graphics
        .instance
        .get_physical_device_queue_family_properties(physical, None);

    let memory_properties = graphics
        .instance
        .get_physical_device_memory_properties(physical, None);

    let mut properties = Properties {
        extension: extension_properties,
        family: family_properties,
        memory: memory_properties,
        v10: properties10,
        v11: properties11.build(),
        v12: properties12.build(),
        acc: properties_acc.build(),
        rt: properties_rt.build(),
    };

    let mut features = Features {
        v10: features10,
        v11: features11.build(),
        v12: features12.build(),
        acc: features_acc.build(),
        rt: features_rt.build(),
    };

    properties.v11.p_next = std::ptr::null_mut();
    properties.v12.p_next = std::ptr::null_mut();
    properties.rt.p_next = std::ptr::null_mut();
    features.v11.p_next = std::ptr::null_mut();
    features.v12.p_next = std::ptr::null_mut();
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
        let (properties, features) = collect_propeties_and_features(physical);
        tracing::info!("{:#?}", properties);

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

        if self.features.v12.buffer_device_address > 0 {
            features.push(Feature::BufferDeviceAddress);
        }

        if self
            .properties
            .has_extension(unsafe { CStr::from_ptr(KHR_ACCELERATION_STRUCTURE_EXTENSION_NAME) })
            && self.features.acc.acceleration_structure != 0
        {
            assert!(features.contains(&Feature::BufferDeviceAddress));
            features.push(Feature::AccelerationStructure);
        }

        if self
            .properties
            .has_extension(unsafe { CStr::from_ptr(KHR_RAY_TRACING_PIPELINE_EXTENSION_NAME) })
            && self.features.rt.ray_tracing_pipeline != 0
        {
            assert!(features.contains(&Feature::AccelerationStructure));
            features.push(Feature::RayTracingPipeline);
        }

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

        if self.graphics().instance.enabled().khr_surface
            && self
                .properties
                .has_extension(unsafe { CStr::from_ptr(KHR_SWAPCHAIN_EXTENSION_NAME) })
        {
            features.push(Feature::SurfacePresentation);
        }

        DeviceInfo {
            kind: match self.properties.v10.device_type {
                vk1_0::PhysicalDeviceType::INTEGRATED_GPU => Some(DeviceKind::Integrated),
                vk1_0::PhysicalDeviceType::DISCRETE_GPU => Some(DeviceKind::Discrete),
                vk1_0::PhysicalDeviceType::CPU => Some(DeviceKind::Software),
                vk1_0::PhysicalDeviceType::OTHER | vk1_0::PhysicalDeviceType::VIRTUAL_GPU | _ => {
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

    /// Returns surface capabilities.
    /// Returns `Ok(None)` if this device does not support surface.
    pub fn surface_capabilities(
        &self,
        surface: &Surface,
    ) -> Result<Option<SurfaceCapabilities>, SurfaceError> {
        let surface = surface.handle();
        let instance = &self.graphics().instance;

        assert!(
            instance.enabled().khr_surface,
            "Should be enabled given that there is a Surface"
        );
        let families =
            unsafe { instance.get_physical_device_queue_family_properties(self.physical, None) };

        let families = (0..families.len())
            .filter_map(|f| {
                let supported = unsafe {
                    instance.get_physical_device_surface_support_khr(
                        self.physical,
                        f.try_into().unwrap(),
                        surface,
                        None,
                    )
                }
                .result()
                .map_err(|err| match err {
                    vk1_0::Result::ERROR_OUT_OF_HOST_MEMORY => out_of_host_memory(),
                    vk1_0::Result::ERROR_OUT_OF_DEVICE_MEMORY => SurfaceError::OutOfMemory {
                        source: OutOfMemory,
                    },
                    vk1_0::Result::ERROR_SURFACE_LOST_KHR => SurfaceError::SurfaceLost,
                    _ => unreachable!(),
                });

                match supported {
                    Ok(true) => Some(Ok(f)),
                    Ok(false) => None,
                    Err(err) => Some(Err(err)),
                }
            })
            .collect::<Result<Vec<_>, SurfaceError>>()?;

        if families.is_empty() {
            return Ok(None);
        }

        let present_modes = unsafe {
            instance.get_physical_device_surface_present_modes_khr(self.physical, surface, None)
        }
        .result()
        .map_err(surface_error_from_erupt)?;

        let present_modes = present_modes
            .into_iter()
            .filter_map(from_erupt)
            .collect::<Vec<_>>();

        let caps = unsafe {
            instance.get_physical_device_surface_capabilities_khr(self.physical, surface, None)
        }
        .result()
        .map_err(surface_error_from_erupt)?;

        let formats = unsafe {
            instance.get_physical_device_surface_formats_khr(self.physical, surface, None)
        }
        .result()
        .map_err(surface_error_from_erupt)?;

        let formats = formats
            .iter()
            .filter_map(|sf| from_erupt(sf.format))
            .collect::<Vec<_>>();

        Ok(Some(SurfaceCapabilities {
            families,
            image_count: caps.min_image_count..=caps.max_image_count,
            current_extent: from_erupt(caps.current_extent),
            image_extent: from_erupt(caps.min_image_extent)..=from_erupt(caps.max_image_extent),
            supported_usage: from_erupt(caps.supported_usage_flags),
            present_modes,
            formats,
        }))
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
    #[tracing::instrument(skip(queues))]
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

        tracing::trace!("Creating device");

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

            let priorities = families_requested.entry(family).or_insert(Vec::new());

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
        let mut features = vk1_0::PhysicalDeviceFeaturesBuilder::new();
        let mut features2 = vk1_1::PhysicalDeviceFeatures2Builder::new();
        let mut features11 = vk1_2::PhysicalDeviceVulkan11FeaturesBuilder::new();
        let mut features12 = vk1_2::PhysicalDeviceVulkan12FeaturesBuilder::new();
        let mut features_acc = vkacc::PhysicalDeviceAccelerationStructureFeaturesKHRBuilder::new();
        let mut features_rt = vkrt::PhysicalDeviceRayTracingPipelineFeaturesKHRBuilder::new();
        let include_features11 = false;
        let mut include_features12 = false;
        let mut include_features_acc = false;
        let mut include_features_rt = false;

        // Enable requested extensions.
        let mut enable_exts = SmallVec::<[_; 10]>::new();

        let mut push_ext = |name| {
            let name = unsafe { CStr::from_ptr(name) };
            assert!(
                self.properties.has_extension(name),
                "Extension {:?} is missing",
                name
            );

            enable_exts.push(name.as_ptr());
        };

        if requested_features.take(Feature::SurfacePresentation) {
            push_ext(KHR_SWAPCHAIN_EXTENSION_NAME);
        }

        if requested_features.take(Feature::RayTracingPipeline) {
            assert_ne!(
                self.features.rt.ray_tracing_pipeline, 0,
                "Attempt to enable unsupported feature `RayTracing`"
            );
            assert!(
                requested_features.check(Feature::AccelerationStructure),
                "`AccelerationStructure` feature must be enabled when `RayTracingPipeline` feature is enabled"
            );
            features_rt.ray_tracing_pipeline = 1;
            include_features_rt = true;

            push_ext(KHR_RAY_TRACING_PIPELINE_EXTENSION_NAME);
            // push_ext(KHR_PIPELINE_LIBRARY_EXTENSION_NAME);
            // push_ext(KHR_DEFERRED_HOST_OPERATIONS_EXTENSION_NAME);
            // push_ext(KHR_8BIT_STORAGE_EXTENSION_NAME);
            // push_ext(KHR_16BIT_STORAGE_EXTENSION_NAME);
            // push_ext(KHR_PUSH_DESCRIPTOR_EXTENSION_NAME);
        }

        if requested_features.take(Feature::AccelerationStructure) {
            assert_ne!(
                self.features.acc.acceleration_structure, 0,
                "Attempt to enable unsupported feature `AccelerationStructure`"
            );

            assert!(
                requested_features.check(Feature::BufferDeviceAddress),
                "`BufferDeviceAddress` feature must be enabled when `AccelerationStructure` feature is enabled"
            );

            features_acc.acceleration_structure = 1;
            include_features_acc = true;

            push_ext(KHR_ACCELERATION_STRUCTURE_EXTENSION_NAME);
            push_ext(KHR_DEFERRED_HOST_OPERATIONS_EXTENSION_NAME);
        }

        if requested_features.take(Feature::BufferDeviceAddress) {
            assert_ne!(
                self.features.v12.buffer_device_address, 0,
                "Attempt to enable unsupproted feature `BufferDeviceAddress`"
            );

            features12.buffer_device_address = 1;
            include_features12 = true;
        }

        if requested_features.take(Feature::ScalarBlockLayout) {
            assert_ne!(
                self.features.v12.scalar_block_layout, 0,
                "Attempt to enable unsupported feature `ScalarBlockLayout`"
            );

            features12.scalar_block_layout = 1;
            include_features12 = true;
        }

        if requested_features.take(Feature::RuntimeDescriptorArray) {
            assert_ne!(
                self.features.v12.runtime_descriptor_array, 0,
                "Attempt to enable unsupported feature `RuntimeDescriptorArray`"
            );

            features12.runtime_descriptor_array = 1;
            include_features12 = true;
        }

        if requested_features.take(Feature::DescriptorBindingUniformBufferUpdateAfterBind) {
            assert_ne!(
                self.features
                    .v12
                    .descriptor_binding_uniform_buffer_update_after_bind,
                0,
                "Attempt to enable unsupported feature `DescriptorBindingUniformBufferUpdateAfterBind`"
            );
            features12.descriptor_binding_uniform_buffer_update_after_bind = 1;
            include_features12 = true;
        }
        if requested_features.take(Feature::DescriptorBindingSampledImageUpdateAfterBind) {
            assert_ne!(
                self.features
                    .v12
                    .descriptor_binding_sampled_image_update_after_bind,
                0,
                "Attempt to enable unsupported feature `DescriptorBindingSampledImageUpdateAfterBind`"
            );
            features12.descriptor_binding_sampled_image_update_after_bind = 1;
            include_features12 = true;
        }
        if requested_features.take(Feature::DescriptorBindingStorageImageUpdateAfterBind) {
            assert_ne!(
                self.features
                    .v12
                    .descriptor_binding_storage_image_update_after_bind,
                0,
                "Attempt to enable unsupported feature `DescriptorBindingStorageImageUpdateAfterBind`"
            );
            features12.descriptor_binding_storage_image_update_after_bind = 1;
            include_features12 = true;
        }
        if requested_features.take(Feature::DescriptorBindingStorageBufferUpdateAfterBind) {
            assert_ne!(
                self.features
                    .v12
                    .descriptor_binding_storage_buffer_update_after_bind,
                0,
                "Attempt to enable unsupported feature `DescriptorBindingStorageBufferUpdateAfterBind`"
            );
            features12.descriptor_binding_storage_buffer_update_after_bind = 1;
            include_features12 = true;
        }
        if requested_features.take(Feature::DescriptorBindingUniformTexelBufferUpdateAfterBind) {
            assert_ne!(
                self.features
                    .v12
                    .descriptor_binding_uniform_texel_buffer_update_after_bind,
                0,
                "Attempt to enable unsupported feature `DescriptorBindingUniformTexelBufferUpdateAfterBind`"
            );
            features12.descriptor_binding_uniform_texel_buffer_update_after_bind = 1;
            include_features12 = true;
        }
        if requested_features.take(Feature::DescriptorBindingStorageTexelBufferUpdateAfterBind) {
            assert_ne!(
                self.features
                    .v12
                    .descriptor_binding_storage_texel_buffer_update_after_bind,
                0,
                "Attempt to enable unsupported feature `DescriptorBindingStorageTexelBufferUpdateAfterBind`"
            );
            features12.descriptor_binding_storage_texel_buffer_update_after_bind = 1;
            include_features12 = true;
        }
        if requested_features.take(Feature::DescriptorBindingUpdateUnusedWhilePending) {
            assert_ne!(
                self.features
                    .v12
                    .descriptor_binding_update_unused_while_pending,
                0
            );
            features12.descriptor_binding_update_unused_while_pending = 1;
            include_features12 = true;
        }
        if requested_features.take(Feature::DescriptorBindingPartiallyBound) {
            assert_ne!(
                self.features.v12.descriptor_binding_partially_bound, 0,
                "Attempt to enable unsupported feature `DescriptorBindingPartiallyBound`"
            );
            features12.descriptor_binding_partially_bound = 1;
            include_features12 = true;
        }

        if requested_features.take(Feature::ShaderSampledImageNonUniformIndexing) {
            assert!(requested_features.check(Feature::ShaderSampledImageDynamicIndexing));
            assert_ne!(
                self.features
                    .v12
                    .shader_sampled_image_array_non_uniform_indexing,
                0
            );
            features12.shader_sampled_image_array_non_uniform_indexing = 1;
            include_features12 = true;
        }
        if requested_features.take(Feature::ShaderSampledImageDynamicIndexing) {
            assert_ne!(
                self.features
                    .v10
                    .shader_sampled_image_array_dynamic_indexing,
                0
            );
            features.shader_sampled_image_array_dynamic_indexing = 1;
        }
        if requested_features.take(Feature::ShaderStorageImageNonUniformIndexing) {
            assert!(requested_features.check(Feature::ShaderStorageImageDynamicIndexing));
            assert_ne!(
                self.features
                    .v12
                    .shader_storage_image_array_non_uniform_indexing,
                0
            );
            features12.shader_storage_image_array_non_uniform_indexing = 1;
            include_features12 = true;
        }
        if requested_features.take(Feature::ShaderStorageImageDynamicIndexing) {
            assert_ne!(
                self.features
                    .v10
                    .shader_storage_image_array_dynamic_indexing,
                0
            );
            features.shader_storage_image_array_dynamic_indexing = 1;
        }
        if requested_features.take(Feature::ShaderUniformBufferNonUniformIndexing) {
            assert!(requested_features.check(Feature::ShaderUniformBufferDynamicIndexing));
            assert_ne!(
                self.features
                    .v12
                    .shader_uniform_buffer_array_non_uniform_indexing,
                0
            );
            features12.shader_uniform_buffer_array_non_uniform_indexing = 1;
            include_features12 = true;
        }
        if requested_features.take(Feature::ShaderUniformBufferDynamicIndexing) {
            assert_ne!(
                self.features
                    .v10
                    .shader_uniform_buffer_array_dynamic_indexing,
                0
            );
            features.shader_uniform_buffer_array_dynamic_indexing = 1;
        }
        if requested_features.take(Feature::ShaderStorageBufferNonUniformIndexing) {
            assert!(requested_features.check(Feature::ShaderStorageBufferDynamicIndexing));
            assert_ne!(
                self.features
                    .v12
                    .shader_storage_buffer_array_non_uniform_indexing,
                0
            );
            features12.shader_storage_buffer_array_non_uniform_indexing = 1;
            include_features12 = true;
        }
        if requested_features.take(Feature::ShaderStorageBufferDynamicIndexing) {
            assert_ne!(
                self.features
                    .v10
                    .shader_storage_buffer_array_dynamic_indexing,
                0
            );
            features.shader_storage_buffer_array_dynamic_indexing = 1;
        }

        device_create_info = device_create_info.enabled_extension_names(&enable_exts);

        let version = self.graphics().version;

        if version < vk1_0::make_version(1, 1, 0) {
            device_create_info = device_create_info.enabled_features(&features);
            assert!(!include_features11);
            assert!(!include_features12);
            assert!(!include_features_rt);
        } else {
            if version < vk1_0::make_version(1, 2, 0) {
                assert!(!include_features12);
            }

            // Push structure to the list if at least one feature is enabled.
            if include_features_acc {
                device_create_info = device_create_info.extend_from(&mut features_acc);
            }

            if include_features_rt {
                device_create_info = device_create_info.extend_from(&mut features_rt);
            }

            if include_features12 {
                device_create_info = device_create_info.extend_from(&mut features12);
            }

            if include_features11 {
                device_create_info = device_create_info.extend_from(&mut features11);
            }

            device_create_info = device_create_info.extend_from(&mut features2);
        }

        // Ensure all features were consumed.
        requested_features.assert_empty();

        let instance = &self.graphics().instance;

        let result = DeviceLoader::new(instance, self.physical, &device_create_info, None);

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

        // Wrap device.
        let device = Device::new(
            logical,
            self.physical,
            self.properties,
            self.features,
            version,
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
                            let queue =
                                unsafe { device.logical().get_device_queue(index, family, None) };

                            Queue::new(
                                queue,
                                vk1_0::CommandPool::null(),
                                device.clone(),
                                QueueId {
                                    family: family as usize,
                                    index: index as usize,
                                },
                                capabilities,
                            )
                        })
                        .collect(),
                }
            })
            .collect();

        tracing::debug!("Device created");

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
