use std::{
    collections::VecDeque,
    convert::TryInto as _,
    num::NonZeroU32,
    sync::{
        atomic::{AtomicU32, AtomicU64, Ordering::*},
        Arc,
    },
};

use erupt::{
    extensions::{
        khr_surface as vks,
        khr_swapchain::{self as vksw, SwapchainKHR},
    },
    vk1_0, ObjectHandle,
};
use smallvec::SmallVec;

use crate::{
    backend::vulkan::{convert::from_erupt, device_lost},
    format::Format,
    image::{Image, ImageInfo, ImageUsage, Samples},
    out_of_host_memory,
    semaphore::Semaphore,
    surface::{PresentMode, SurfaceCapabilities, SurfaceError},
    DisplayTimingUnavailable, OutOfMemory, PresentationTiming,
};

use super::{
    convert::ToErupt as _,
    device::{Device, WeakDevice},
    physical::surface_capabilities,
    surface::{surface_error_from_erupt, Surface},
    unexpected_result,
};
static UID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug)]
pub struct SwapchainImage<'a> {
    image: &'a Image,
    wait: &'a mut Semaphore,
    signal: &'a mut Semaphore,
    owner: WeakDevice,
    handle: SwapchainKHR,
    supported_families: Arc<[bool]>,
    acquired_counter: &'a AtomicU32,
    index: u32,
    optimal: bool,
}

impl SwapchainImage<'_> {
    #[inline]
    pub(super) fn supported_families(&self) -> &[bool] {
        &self.supported_families
    }

    /// Swapchain image.
    #[inline]
    pub fn image(&self) -> &Image {
        self.image
    }

    /// Semaphores that should be waited upon before and signaled after last image access.
    #[inline]
    pub fn wait_signal(&mut self) -> [&mut Semaphore; 2] {
        [&mut *self.wait, &mut *self.signal]
    }

    /// Returns true of this swapchain image is optimal for the surface.
    /// If image is not optimal, user still can render to it and must present.
    ///
    /// For most users this is the hint that swapchain should be reconfigured.
    #[inline]
    pub fn is_optimal(&self) -> bool {
        self.optimal
    }

    #[inline]
    pub(super) fn index(&self) -> u32 {
        self.index
    }

    #[inline]
    pub(super) fn is_owned_by(&self, owner: &impl PartialEq<WeakDevice>) -> bool {
        *owner == self.owner
    }

    #[inline]
    pub(super) fn handle(&self) -> SwapchainKHR {
        self.handle
    }

    #[inline]
    pub(super) fn presented(self) {
        self.acquired_counter.fetch_sub(1, Release);
        std::mem::forget(self);
    }
}

impl Drop for SwapchainImage<'_> {
    #[track_caller]
    fn drop(&mut self) {
        // Report usage error unless this happens due to unwinding.
        if !std::thread::panicking() {
            error!("Swapchain image is dropped. Swapchain images MUST be presented")
        }
    }
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
    acquired_counter: AtomicU32,
    format: Format,
    usage: ImageUsage,
    mode: PresentMode,
    optimal: bool,
}

#[derive(Debug)]
pub struct Swapchain {
    inner: Option<SwapchainInner>,
    retired: VecDeque<SwapchainInner>,
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

        info!("{:#?}", surface_capabilities);

        if surface_capabilities.supported_families.is_empty() {
            return Err(SurfaceError::NotSupported);
        }

        let free_semaphore = device.clone().create_semaphore()?;

        surface.mark_used()?;
        debug!("Swapchain created");
        Ok(Swapchain {
            surface: surface.clone(),
            free_semaphore,
            inner: None,
            retired: VecDeque::new(),
            device: device.downgrade(),
            surface_capabilities,
        })
    }
}

impl Swapchain {
    #[inline]
    pub fn capabilities(&self) -> &SurfaceCapabilities {
        &self.surface_capabilities
    }

    /// Update swapchain images for changed surface.
    /// Does nothing if not configured.
    #[cfg_attr(feature = "tracing", tracing::instrument)]
    #[inline]
    pub fn update(&mut self) -> Result<(), SurfaceError> {
        if let Some(inner) = &self.inner {
            let usage = inner.usage;
            let format = inner.format;
            let mode = inner.mode;

            self.configure(usage, format, mode)
        } else {
            Ok(())
        }
    }

    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub fn configure(
        &mut self,
        usage: ImageUsage,
        format: Format,
        mode: PresentMode,
    ) -> Result<(), SurfaceError> {
        let device = self.device.upgrade().ok_or(SurfaceError::SurfaceLost)?;

        debug_assert!(
            device.logical().enabled().khr_swapchain,
            "Should be enabled given that there is a Swapchain"
        );

        // TODO: Configurable count
        if self.retired.len() > 16 {
            // Too many swapchains accumulated.
            // Give resources a chance to be freed.

            warn!("Too many retired swapchains. Wait device idle");
            device.wait_idle();
        }

        self.try_dispose_retired_swapchains(&device);

        assert!(
            self.retired.len() <= 16,
            "Resources that reference old swapchain images should be freed in timely manner"
        );

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
        let logical = device.logical();

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
            .ok_or(SurfaceError::FormatUnsupported { format })?;

        let composite_alpha = {
            let raw = caps.supported_composite_alpha.to_erupt().bits();

            if raw == 0 {
                warn!("Vulkan implementation must support at least one composite alpha mode, but this one reports none. Picking OPAQUE and hope for the best");
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
            self.retired.push_back(inner);

            handle
        } else {
            vksw::SwapchainKHR::null()
        };

        let image_count = 3.clamp(
            caps.min_image_count.get(),
            caps.max_image_count.map_or(!0, NonZeroU32::get),
        );

        let handle = unsafe {
            logical.create_swapchain_khr(
                &vksw::SwapchainCreateInfoKHRBuilder::new()
                    .surface(surface)
                    .min_image_count(image_count)
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
            )
        }
        .result()
        .map_err(surface_error_from_erupt)?;

        let images = unsafe {
            logical
                .get_swapchain_images_khr(handle, None)
                .result()
                .map_err(|err| {
                    logical.destroy_swapchain_khr(handle, None);
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
                logical.destroy_swapchain_khr(handle, None);

                SurfaceError::OutOfMemory { source: err }
            })?;

        let index = device.insert_swapchain(handle);

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
                        UID.fetch_add(1, Relaxed)
                            .try_into()
                            .expect("u64 increment overflows"),
                    ),
                    acquire: a,
                    release: r,
                })
                .collect(),
            acquired_counter: AtomicU32::new(0),
            format,
            usage,
            mode,
            optimal: true,
        });

        debug!("Swapchain configured");
        Ok(())
    }

    pub fn acquire_image(&mut self) -> Result<SwapchainImage<'_>, SurfaceError> {
        let device = self.device.upgrade().ok_or(SurfaceError::SurfaceLost)?;

        debug_assert!(
            device.logical().enabled().khr_swapchain,
            "Should be enabled given that there is a Swapchain"
        );

        self.try_dispose_retired_swapchains(&device);

        let index = loop {
            let inner = self.inner.as_mut().ok_or(SurfaceError::NotConfigured)?;

            if inner.acquired_counter.load(Acquire)
                > (inner.images.len() as u32 - self.surface_capabilities.min_image_count.get())
            {
                return Err(SurfaceError::TooManyAcquired);
            }

            // FIXME: Use fences to know that acquire semaphore is unused.
            let wait = &self.free_semaphore;

            let result = unsafe {
                device.logical().acquire_next_image_khr(
                    inner.handle,
                    !0, /* wait indefinitely. This is OK as we never try to
                         * acquire more images than there is in swapchain. */
                    wait.handle(),
                    vk1_0::Fence::null(),
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
                    // Image acquired, but it is suboptimal.
                    // It must be presented either way.
                    inner.optimal = false;
                }
                vk1_0::Result::ERROR_OUT_OF_DATE_KHR => {
                    // No image acquired. Reconfigure.
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

            inner.acquired_counter.fetch_add(1, Acquire);

            break index;
        };

        let inner = self.inner.as_mut().unwrap();
        let image_and_semaphores = &mut inner.images[index as usize];
        let wait = &mut image_and_semaphores.acquire;
        let signal = &mut image_and_semaphores.release;

        Ok(SwapchainImage {
            image: &image_and_semaphores.image,
            wait,
            signal,
            owner: self.device.clone(),
            handle: inner.handle,
            supported_families: self.surface_capabilities.supported_families.clone(),
            acquired_counter: &inner.acquired_counter,
            index,
            optimal: inner.optimal,
        })
    }

    /// Returns refresh duration of the display associated with this swapchain.
    /// Returned value is the number of nanoseconds from the start of one refresh cycle to the next.
    pub fn get_refresh_cycle_duration(&self) -> Result<u64, SurfaceError> {
        let device = self.device.upgrade().ok_or(SurfaceError::SurfaceLost)?;

        debug_assert!(
            device.logical().enabled().khr_swapchain,
            "Should be enabled given that there is a Swapchain"
        );

        if !device.logical().enabled().google_display_timing {
            return Err(DisplayTimingUnavailable.into());
        }

        let inner = self.inner.as_ref().ok_or(SurfaceError::NotConfigured)?;

        let refresh_cycle_duration = unsafe {
            device
                .logical()
                .get_refresh_cycle_duration_google(inner.handle)
        }
        .result()
        .map_err(surface_error_from_erupt)?;

        Ok(refresh_cycle_duration.refresh_duration)
    }

    pub fn get_past_presentation_timing(
        &self,
    ) -> Result<SmallVec<[PresentationTiming; 8]>, SurfaceError> {
        let device = self.device.upgrade().ok_or(SurfaceError::SurfaceLost)?;

        debug_assert!(
            device.logical().enabled().khr_swapchain,
            "Should be enabled given that there is a Swapchain"
        );

        if !device.logical().enabled().google_display_timing {
            return Err(DisplayTimingUnavailable.into());
        }

        let inner = self.inner.as_ref().ok_or(SurfaceError::NotConfigured)?;

        let f = device
            .logical()
            .get_past_presentation_timing_google
            .unwrap();

        let mut timings = SmallVec::<[_; 8]>::from_elem(Default::default(), 8);

        let mut count = 8;
        let mut result = unsafe {
            f(
                device.logical().handle,
                inner.handle,
                &mut count,
                timings.as_mut_ptr(),
            )
        };

        while let vk1_0::Result::INCOMPLETE = result {
            timings.resize(count as usize, Default::default());

            result = unsafe {
                f(
                    device.logical().handle,
                    inner.handle,
                    &mut count,
                    timings.as_mut_ptr(),
                )
            };
        }

        match result {
            vk1_0::Result::INCOMPLETE => unreachable!(),
            vk1_0::Result::SUCCESS => {
                let mut timings = timings
                    .into_iter()
                    .map(from_erupt)
                    .collect::<SmallVec<[PresentationTiming; 8]>>();
                timings.sort_by_key(|timing| timing.present_id);
                Ok(timings)
            }
            vk1_0::Result::ERROR_OUT_OF_HOST_MEMORY => out_of_host_memory(),
            vk1_0::Result::ERROR_DEVICE_LOST => device_lost(),
            vk1_0::Result::ERROR_OUT_OF_DATE_KHR => Ok(SmallVec::new()),
            vk1_0::Result::ERROR_SURFACE_LOST_KHR => Err(SurfaceError::SurfaceLost),
            _ => unexpected_result(result),
        }
    }

    fn try_dispose_retired_swapchains(&mut self, device: &Device) {
        'a: while let Some(mut inner) = self.retired.pop_front() {
            while let Some(mut iws) = inner.images.pop() {
                if let Err(image) = iws.image.try_dispose() {
                    iws.image = image;
                    inner.images.push(iws);
                    self.retired.push_front(inner);
                    break 'a;
                }
            }

            debug!("Destroying retired swapchain. {} left", self.retired.len());
            unsafe {
                // This swapchain and its images are no longer in use.
                device.destroy_swapchain(inner.index)
            }
        }
    }
}
