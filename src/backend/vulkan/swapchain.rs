use super::{
    convert::ToErupt as _,
    device::{Device, WeakDevice},
    physical::surface_capabilities,
    surface::{surface_error_from_erupt, Surface},
    unexpected_result,
};
use crate::{
    format::Format,
    image::{Image, ImageInfo, ImageUsage, Samples},
    out_of_host_memory,
    semaphore::Semaphore,
    surface::{PresentMode, SurfaceCapabilities, SurfaceError},
    Extent2d, OutOfMemory,
};
use erupt::{
    extensions::{
        khr_surface as vks,
        khr_swapchain::{self as vksw, SwapchainKHR},
    },
    vk1_0,
};
use std::{
    convert::TryInto as _,
    sync::{
        atomic::{AtomicU32, AtomicU64, Ordering::*},
        Arc,
    },
};

static UID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Debug)]
pub struct SwapchainImage {
    info: SwapchainImageInfo,
    owner: WeakDevice,
    handle: SwapchainKHR,
    supported_families: Arc<[bool]>,
    acquired_counter: Arc<AtomicU32>,
    index: u32,
}

impl SwapchainImage {
    pub fn info(&self) -> &SwapchainImageInfo {
        &self.info
    }

    pub(super) fn supported_families(&self) -> &[bool] {
        &*self.supported_families
    }

    pub fn index(&self) -> u32 {
        self.index
    }

    pub(super) fn is_owned_by(&self, owner: &impl PartialEq<WeakDevice>) -> bool {
        *owner == self.owner
    }

    pub(super) fn handle(&self) -> SwapchainKHR {
        self.handle
    }

    pub(super) fn presented(self) {
        self.acquired_counter.fetch_sub(1, Release);
    }
}

#[derive(Clone, Debug)]
pub struct SwapchainImageInfo {
    /// Swapchain image.
    pub image: Image,

    /// Semaphore that should be waited upon before accessing an image.
    ///
    /// Acquisition semaphore management may be rather complex,
    /// so keep that to the implementation.
    pub wait: Semaphore,

    /// Semaphore that should be signaled after last image access.
    ///
    /// Presentation semaphore management may be rather complex,
    /// so keep that to the implementation.
    pub signal: Semaphore,
}

#[derive(Debug)]
struct SwapchainImageAndSemaphores {
    image: Image,
    acquire: Semaphore,
    release: Semaphore,
}

#[derive(Debug)]
struct SwapchainInner {
    handle: vksw::SwapchainKHR,
    index: usize,
    images: Vec<SwapchainImageAndSemaphores>,
    acquired_counter: Arc<AtomicU32>,
    format: Format,
    extent: Extent2d,
    usage: ImageUsage,
    mode: PresentMode,
}

#[derive(Debug)]
pub struct Swapchain {
    inner: Option<SwapchainInner>,
    retired: Vec<SwapchainInner>,
    retired_offset: u64,
    free_semaphore: Semaphore,
    device: WeakDevice,
    surface: Surface,
    surface_capabilities: SurfaceCapabilities,
}

impl Swapchain {
    pub(crate) fn new(surface: &Surface, device: &Device) -> Result<Self, SurfaceError> {
        debug_assert!(
            device.graphics().instance.enabled().khr_surface,
            "Should be enabled given that there is a Surface"
        );

        assert!(
            device.logical().enabled().khr_swapchain,
            "`Feature::SurfacePresentation` must be enabled in order to create a `Swapchain`"
        );

        let instance = &device.graphics().instance;
        let surface_capabilities =
            surface_capabilities(instance, device.physical(), surface.handle())?;

        if surface_capabilities.supported_families.is_empty() {
            return Err(SurfaceError::NotSupported);
        }

        let free_semaphore = device.clone().create_semaphore()?;

        surface.mark_used()?;
        tracing::debug!("Swapchain created");
        Ok(Swapchain {
            surface: surface.clone(),
            free_semaphore,
            inner: None,
            retired: Vec::new(),
            retired_offset: 0,
            device: device.downgrade(),
            surface_capabilities,
        })
    }
}

impl Swapchain {
    pub fn capabilities(&self) -> &SurfaceCapabilities {
        &self.surface_capabilities
    }

    #[tracing::instrument]
    pub fn configure(
        &mut self,
        usage: ImageUsage,
        format: Format,
        mode: PresentMode,
    ) -> Result<(), SurfaceError> {
        let device = self
            .device
            .upgrade()
            .ok_or_else(|| SurfaceError::SurfaceLost)?;

        let surface = self.surface.handle();

        assert!(
            device.graphics().instance.enabled().khr_surface,
            "Should be enabled given that there is a Swapchain"
        );
        assert!(
            device.logical().enabled().khr_swapchain,
            "Should be enabled given that there is a Swapchain"
        );

        let instance = &device.graphics().instance;
        let logical = &device.logical();

        self.surface_capabilities = surface_capabilities(instance, device.physical(), surface)?;
        let caps = &self.surface_capabilities;

        if !caps.supported_usage.contains(usage) {
            return Err(SurfaceError::UsageNotSupported { usage });
        }

        let formats = unsafe {
            instance.get_physical_device_surface_formats_khr(device.physical(), surface, None)
        }
        .result()
        .map_err(surface_error_from_erupt)?;

        let erupt_format = format.to_erupt();

        let sf = formats
            .iter()
            .find(|sf| sf.format == erupt_format)
            .ok_or_else(|| SurfaceError::FormatUnsupported { format })?;

        let composite_alpha = {
            let raw = caps.supported_composite_alpha.to_erupt().bits();

            if raw == 0 {
                tracing::warn!("Vulkan implementation must support at least one composite alpha mode, but this one reports none. Picking OPAQUE and hope for the best");
                vks::CompositeAlphaFlagsKHR::OPAQUE_KHR
            } else {
                // Use lowest bit flag
                vks::CompositeAlphaFlagsKHR::from_bits_truncate(1 << raw.trailing_zeros())
            }
        };

        let modes = unsafe {
            instance.get_physical_device_surface_present_modes_khr(device.physical(), surface, None)
        }
        .result()
        .map_err(surface_error_from_erupt)?;

        let erupt_mode = mode.to_erupt();

        if modes.iter().all(|&sm| sm != erupt_mode) {
            return Err(SurfaceError::PresentModeUnsupported { mode });
        }

        let old_swapchain = if let Some(inner) = self.inner.take() {
            let handle = inner.handle;
            self.retired.push(inner);

            handle
        } else {
            vksw::SwapchainKHR::null()
        };

        let handle = unsafe {
            logical.create_swapchain_khr(
                &vksw::SwapchainCreateInfoKHRBuilder::new()
                    .surface(surface)
                    .min_image_count(3.min(caps.max_image_count).max(caps.min_image_count))
                    .image_format(sf.format)
                    .image_color_space(sf.color_space)
                    .image_extent(caps.current_extent.to_erupt())
                    .image_array_layers(1)
                    .image_usage(usage.to_erupt())
                    .image_sharing_mode(vk1_0::SharingMode::EXCLUSIVE)
                    .pre_transform(vks::SurfaceTransformFlagBitsKHR(
                        caps.current_transform.to_erupt().bits(),
                    ))
                    .composite_alpha(vks::CompositeAlphaFlagBitsKHR(composite_alpha.bits()))
                    .present_mode(erupt_mode)
                    .old_swapchain(old_swapchain),
                None,
                None,
            )
        }
        .result()
        .map_err(surface_error_from_erupt)?;

        let images = unsafe {
            logical
                .get_swapchain_images_khr(handle, None)
                .result()
                .map_err(|err| {
                    logical.destroy_swapchain_khr(Some(handle), None);
                    surface_error_from_erupt(err)
                })
        }?;

        let semaphores = (0..images.len())
            .map(|_| {
                Ok((
                    device.clone().create_semaphore()?,
                    device.clone().create_semaphore()?,
                ))
            })
            .collect::<Result<Vec<_>, _>>()
            .map_err(|err| unsafe {
                logical.destroy_swapchain_khr(Some(handle), None);

                SurfaceError::OutOfMemory { source: err }
            })?;

        let index = device.swapchains().lock().insert(handle);

        self.inner = Some(SwapchainInner {
            handle,
            index,
            images: images
                .into_iter()
                .zip(semaphores)
                .map(|(i, (a, r))| SwapchainImageAndSemaphores {
                    image: Image::new_swapchain(
                        ImageInfo {
                            extent: caps.current_extent.into(),
                            format,
                            levels: 1,
                            layers: 1,
                            samples: Samples::Samples1,
                            usage,
                        },
                        self.device.clone(),
                        i,
                        UID.fetch_add(1, Relaxed).try_into().unwrap(),
                    ),
                    acquire: a,
                    release: r,
                })
                .collect(),
            acquired_counter: Arc::new(AtomicU32::new(0)),
            extent: caps.current_extent,
            format,
            usage,
            mode,
        });

        tracing::debug!("Swapchain configured");
        Ok(())
    }

    pub fn acquire_image(&mut self, optimal: bool) -> Result<SwapchainImage, SurfaceError> {
        let device = self
            .device
            .upgrade()
            .ok_or_else(|| SurfaceError::SurfaceLost)?;

        assert!(
            device.logical().enabled().khr_swapchain,
            "Should be enabled given that there is a Swapchain"
        );

        loop {
            if let Some(inner) = self.inner.as_mut() {
                if inner.acquired_counter.load(Acquire)
                    >= (inner.images.len() as u32 - self.surface_capabilities.min_image_count)
                {
                    return Err(SurfaceError::TooManyAcquired);
                }

                // FIXME: Use fences to know that acquire semaphore is unused.
                let wait = self.free_semaphore.clone();

                let result = unsafe {
                    device.logical().acquire_next_image_khr(
                        inner.handle,
                        !0, /* wait indefinitely. This is OK as we never try to
                             * acquire more images than there is in swapchain. */
                        Some(wait.handle()),
                        None,
                        None,
                    )
                };

                match result.raw {
                    vk1_0::Result::SUCCESS => {}
                    vk1_0::Result::ERROR_OUT_OF_HOST_MEMORY => out_of_host_memory(),
                    vk1_0::Result::ERROR_OUT_OF_DEVICE_MEMORY => {
                        return Err(SurfaceError::OutOfMemory {
                            source: OutOfMemory,
                        });
                    }
                    vk1_0::Result::ERROR_SURFACE_LOST_KHR => {
                        return Err(SurfaceError::SurfaceLost);
                    }
                    vk1_0::Result::SUBOPTIMAL_KHR => {
                        if optimal {
                            let usage = inner.usage;
                            let format = inner.format;
                            let mode = inner.mode;

                            self.configure(usage, format, mode)?;
                            continue;
                        }
                    }
                    vk1_0::Result::ERROR_OUT_OF_DATE_KHR => {
                        let usage = inner.usage;
                        let format = inner.format;
                        let mode = inner.mode;

                        self.configure(usage, format, mode)?;
                        continue;
                    }
                    raw => unexpected_result(raw),
                }

                let index = result.unwrap();
                let image_and_semaphores = &mut inner.images[index as usize];

                std::mem::swap(&mut image_and_semaphores.acquire, &mut self.free_semaphore);

                let signal = image_and_semaphores.release.clone();

                let swapchain_image = SwapchainImage {
                    info: SwapchainImageInfo {
                        image: image_and_semaphores.image.clone(),
                        wait,
                        signal,
                    },
                    owner: self.device.clone(),
                    handle: inner.handle,
                    supported_families: self.surface_capabilities.supported_families.clone(),
                    acquired_counter: { inner.acquired_counter.clone() },
                    index,
                };

                inner.acquired_counter.fetch_add(1, Acquire);

                return Ok(swapchain_image);
            } else {
                return Err(SurfaceError::NotConfigured);
            }
        }
    }
}
