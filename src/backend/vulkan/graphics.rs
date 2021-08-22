use {
    super::{physical::PhysicalDevice, unexpected_result},
    crate::{
        out_of_host_memory,
        physical::EnumerateDeviceError,
        surface::{CreateSurfaceError, RawWindowHandleKind, Surface, SurfaceInfo},
        OutOfMemory,
    },
    erupt::{
        extensions::{
            ext_debug_report::{
                DebugReportCallbackCreateInfoEXTBuilder, DebugReportFlagsEXT,
                DebugReportObjectTypeEXT, EXT_DEBUG_REPORT_EXTENSION_NAME,
            },
            ext_debug_utils::EXT_DEBUG_UTILS_EXTENSION_NAME,
            khr_surface::KHR_SURFACE_EXTENSION_NAME,
        },
        utils::loading::{EntryLoader, EntryLoaderError},
        vk1_0, InstanceLoader, LoaderError,
    },
    once_cell::sync::OnceCell,
    raw_window_handle::{HasRawWindowHandle, RawWindowHandle},
    smallvec::SmallVec,
    std::{
        ffi::{c_void, CStr},
        fmt::{self, Debug},
        os::raw::c_char,
        sync::atomic::AtomicBool,
    },
};

#[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd",
))]
use erupt::extensions::{
    khr_wayland_surface::{WaylandSurfaceCreateInfoKHR, KHR_WAYLAND_SURFACE_EXTENSION_NAME},
    khr_xcb_surface::{XcbSurfaceCreateInfoKHR, KHR_XCB_SURFACE_EXTENSION_NAME},
    khr_xlib_surface::{XlibSurfaceCreateInfoKHR, KHR_XLIB_SURFACE_EXTENSION_NAME},
};

#[cfg(target_os = "android")]
use erupt::extensions::khr_android_surface::{
    AndroidSurfaceCreateInfoKHR, KHR_ANDROID_SURFACE_EXTENSION_NAME,
};

#[cfg(target_os = "windows")]
use erupt::extensions::khr_win32_surface::{
    Win32SurfaceCreateInfoKHR, KHR_WIN32_SURFACE_EXTENSION_NAME,
};

#[cfg(any(target_os = "ios", target_os = "macos"))]
use erupt::extensions::ext_metal_surface::{
    MetalSurfaceCreateInfoEXT, EXT_METAL_SURFACE_EXTENSION_NAME,
};

/// Root object of the erupt graphics system.
pub struct Graphics {
    pub(crate) instance: InstanceLoader,
    pub(crate) version: u32,
    _entry: EntryLoader,
}

static GLOBAL_GRAPHICS: OnceCell<Graphics> = OnceCell::new();

impl Debug for Graphics {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        if fmt.alternate() {
            fmt.debug_struct("Graphics")
                .field("instance", &self.instance.handle)
                .field("version", &self.version)
                .finish()
        } else {
            Debug::fmt(&self.instance.handle, fmt)
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum InitError {
    #[error(transparent)]
    EntryLoaderError {
        #[from]
        source: EntryLoaderError,
    },

    #[error("Failed to load functions from vulkan library")]
    FunctionLoadFailed,

    #[error("Instance creation failed")]
    CreateInstanceError {
        #[from]
        source: vk1_0::Result,
    },
}

impl Graphics {
    pub fn get_or_init() -> Result<&'static Graphics, InitError> {
        GLOBAL_GRAPHICS.get_or_try_init(Self::new)
    }

    pub(crate) unsafe fn get_unchecked() -> &'static Graphics {
        GLOBAL_GRAPHICS.get_unchecked()
    }

    #[tracing::instrument]
    fn new() -> Result<Self, InitError> {
        tracing::trace!("Init erupt graphisc implementation");

        let entry = EntryLoader::new()?;

        let version = entry.instance_version();

        let layer_properties =
            unsafe { entry.enumerate_instance_layer_properties(None) }.result()?;

        let mut enable_layers = SmallVec::<[_; 1]>::new();

        // Pushes layer if it's avalable and returns if it was pushed.
        let mut push_layer = |name: &'static CStr| -> bool {
            if layer_properties
                .iter()
                .any(|p| unsafe { CStr::from_ptr(&p.layer_name[0]) } == name)
            {
                enable_layers.push(name.as_ptr());
                true
            } else {
                false
            }
        };

        if cfg!(debug_assertions) {
            // Enable layers in debug mode.
            if !push_layer(unsafe {
                // Safe because literal has nul-byte.
                CStr::from_bytes_with_nul_unchecked(b"VK_LAYER_KHRONOS_validation\0")
            }) {
                push_layer(unsafe {
                    // Safe because literal has nul-byte.
                    CStr::from_bytes_with_nul_unchecked(b"VK_LAYER_LUNARG_standard_validation\0")
                });
            }

            // push_layer(unsafe {
            //     CStr::from_bytes_with_nul_unchecked(b"
            // VK_LAYER_LUNARG_api_dump\0") });
            // push_layer(unsafe {
            //     CStr::from_bytes_with_nul_unchecked(b"
            // VK_LAYER_LUNARG_device_simulation\0") });
            // push_layer(unsafe {
            //     CStr::from_bytes_with_nul_unchecked(b"
            // VK_LAYER_LUNARG_monitor\0") });
            // push_layer(unsafe {
            //     CStr::from_bytes_with_nul_unchecked(b"
            // VK_LAYER_LUNARG_screenshot\0") });

            // push_layer(unsafe {
            // CStr::from_bytes_with_nul_unchecked(b"VK_LAYER_NV_optimus\0")
            // });

            // push_layer(unsafe {
            //     CStr::from_bytes_with_nul_unchecked(b"
            // VK_LAYER_NV_nomad_release_public_2020_2_0\0") });

            // push_layer(unsafe {
            //     CStr::from_bytes_with_nul_unchecked(b"
            // VK_LAYER_NV_GPU_Trace_release_public_2020_2_0\0") });

            // push_layer(unsafe {
            //     CStr::from_bytes_with_nul_unchecked(b"
            // VK_LAYER_LUNARG_screenshot\0") });
        }

        let extension_properties =
            unsafe { entry.enumerate_instance_extension_properties(None, None) }.result()?;

        let mut enable_exts = SmallVec::<[_; 10]>::new();

        // Pushes extension if it's available and returns if it was pushed.
        let mut push_ext = |name| -> bool {
            let name = unsafe { CStr::from_ptr(name) };
            if extension_properties
                .iter()
                .any(|p| unsafe { CStr::from_ptr(&p.extension_name[0]) } == name)
            {
                tracing::trace!("Pick extension {:?}", name);
                enable_exts.push(name.as_ptr());
                true
            } else {
                false
            }
        };

        if cfg!(debug_assertions) {
            // Enable debug utils and report extensions in debug build.
            push_ext(EXT_DEBUG_UTILS_EXTENSION_NAME);
            push_ext(EXT_DEBUG_REPORT_EXTENSION_NAME);
        }

        if push_ext(KHR_SURFACE_EXTENSION_NAME) {
            #[cfg(target_os = "android")]
            {
                push_ext(KHR_ANDROID_SURFACE_EXTENSION_NAME);
            }

            #[cfg(any(
                target_os = "linux",
                target_os = "dragonfly",
                target_os = "freebsd",
                target_os = "netbsd",
                target_os = "openbsd",
            ))]
            {
                push_ext(KHR_XLIB_SURFACE_EXTENSION_NAME);
                push_ext(KHR_XCB_SURFACE_EXTENSION_NAME);
                push_ext(KHR_WAYLAND_SURFACE_EXTENSION_NAME);
            }

            #[cfg(target_os = "windows")]
            {
                push_ext(KHR_WIN32_SURFACE_EXTENSION_NAME);
            }

            #[cfg(any(target_os = "ios", target_os = "macos"))]
            {
                push_ext(EXT_METAL_SURFACE_EXTENSION_NAME);
            }
        }

        let result = unsafe {
            InstanceLoader::new(
                &entry,
                &vk1_0::InstanceCreateInfoBuilder::new()
                    .application_info(
                        &vk1_0::ApplicationInfoBuilder::new()
                            .engine_name(CStr::from_bytes_with_nul(b"Illume\0").unwrap())
                            .engine_version(1)
                            .application_name(CStr::from_bytes_with_nul(b"IllumeApp\0").unwrap())
                            .application_version(1)
                            .api_version(version),
                    )
                    .enabled_layer_names(&enable_layers)
                    .enabled_extension_names(&enable_exts),
                None,
            )
        };

        let instance = match result {
            Err(LoaderError::SymbolNotAvailable) => {
                return Err(InitError::FunctionLoadFailed);
            }
            Err(LoaderError::VulkanError(err)) => {
                return Err(InitError::CreateInstanceError { source: err });
            }
            Ok(ok) => ok,
        };

        if instance.enabled().ext_debug_report {
            let _ = unsafe {
                instance.create_debug_report_callback_ext(
                    &DebugReportCallbackCreateInfoEXTBuilder::new()
                        .flags(DebugReportFlagsEXT::all())
                        .pfn_callback(Some(debug_report_callback)),
                    None,
                )
            }
            .result()?;
        }

        tracing::trace!("Instance created");

        let graphics = Graphics {
            instance,
            version,
            _entry: entry,
        };

        Ok(graphics)
    }

    pub fn name(&self) -> &str {
        "Vulkan"
    }

    #[tracing::instrument]
    pub fn devices(&self) -> Result<Vec<PhysicalDevice>, EnumerateDeviceError> {
        tracing::trace!("Enumerating physical devices");

        let devices = unsafe {
            // Using valid instance.
            self.instance.enumerate_physical_devices(None)
        }
        .result()
        .map_err(|err| match err {
            vk1_0::Result::ERROR_OUT_OF_HOST_MEMORY => out_of_host_memory(),
            vk1_0::Result::ERROR_OUT_OF_DEVICE_MEMORY => EnumerateDeviceError::OutOfMemory {
                source: OutOfMemory,
            },
            _ => unexpected_result(err),
        })?;

        tracing::trace!("Physical devices {:?}", devices);

        Ok(devices
            .into_iter()
            .map(|physical| unsafe { PhysicalDevice::new(physical) })
            .collect())
    }

    #[tracing::instrument(skip(window), fields(?window = window.raw_window_handle()))]
    pub fn create_surface(
        &self,
        window: &impl HasRawWindowHandle,
    ) -> Result<Surface, CreateSurfaceError> {
        let window = window.raw_window_handle();

        let surface = match window {
            #[cfg(target_os = "android")]
            RawWindowHandle::Android(handle) => {
                if !self.instance.enabled().khr_android_surface {
                    return Err(CreateSurfaceError::UnsupportedWindow {
                        window: RawWindowHandleKind::Android,
                        source: Box::new(RequiredExtensionIsNotAvailable {
                            extension: "VK_KHR_android_surface",
                        }),
                    });
                }

                let result = self.instance.create_android_surface_khr(
                    &AndroidSurfaceCreateInfoKHR {
                        window: handle.a_native_window,
                        ..AndroidSurfaceCreateInfoKHR::default()
                    },
                    None,
                );

                todo!()
            }

            #[cfg(target_os = "ios")]
            RawWindowHandle::IOS(handle) => todo!(),

            #[cfg(target_os = "macos")]
            RawWindowHandle::MacOS(handle) => todo!(),

            #[cfg(any(
                target_os = "linux",
                target_os = "dragonfly",
                target_os = "freebsd",
                target_os = "netbsd",
                target_os = "openbsd"
            ))]
            RawWindowHandle::Wayland(handle) => {
                if !self.instance.enabled().khr_wayland_surface {
                    return Err(CreateSurfaceError::UnsupportedWindow {
                        window: RawWindowHandleKind::Windows,
                        source: Some(Box::new(RequiredExtensionIsNotAvailable {
                            extension: "VK_KHR_wayland_surface",
                        })),
                    });
                }

                let result = unsafe {
                    self.instance.create_wayland_surface_khr(
                        &WaylandSurfaceCreateInfoKHR::default()
                            .into_builder()
                            .surface(handle.surface)
                            .display(handle.display),
                        None,
                    )
                }
                .result();

                match result {
                    Ok(surface) => surface,
                    Err(vk1_0::Result::ERROR_OUT_OF_HOST_MEMORY) => out_of_host_memory(),
                    Err(vk1_0::Result::ERROR_OUT_OF_DEVICE_MEMORY) => {
                        return Err(OutOfMemory.into())
                    }
                    Err(err) => unexpected_result(err),
                }
            }

            #[cfg(target_os = "windows")]
            RawWindowHandle::Windows(handle) => {
                if !self.instance.enabled().khr_win32_surface {
                    return Err(CreateSurfaceError::UnsupportedWindow {
                        window: RawWindowHandleKind::Windows,
                        source: Some(Box::new(RequiredExtensionIsNotAvailable {
                            extension: "VK_KHR_win32_surface",
                        })),
                    });
                }

                unsafe {
                    let mut info = Win32SurfaceCreateInfoKHR::default();
                    info.hinstance = handle.hinstance;
                    info.hwnd = handle.hwnd;
                    self.instance.create_win32_surface_khr(&info, None)
                }
                .result()
                .map_err(|err| match err {
                    vk1_0::Result::ERROR_OUT_OF_HOST_MEMORY => out_of_host_memory(),
                    vk1_0::Result::ERROR_OUT_OF_DEVICE_MEMORY => OutOfMemory,
                    _ => unexpected_result(err),
                })?
            }

            #[cfg(any(
                target_os = "linux",
                target_os = "dragonfly",
                target_os = "freebsd",
                target_os = "netbsd",
                target_os = "openbsd"
            ))]
            RawWindowHandle::Xcb(handle) => {
                if !self.instance.enabled().khr_xcb_surface {
                    return Err(CreateSurfaceError::UnsupportedWindow {
                        window: RawWindowHandleKind::Xcb,
                        source: Some(Box::new(RequiredExtensionIsNotAvailable {
                            extension: "VK_KHR_xcb_surface",
                        })),
                    });
                }

                let result = unsafe {
                    self.instance.create_xcb_surface_khr(
                        &XcbSurfaceCreateInfoKHR::default()
                            .into_builder()
                            .window(handle.window)
                            .connection(handle.connection),
                        None,
                    )
                }
                .result();

                match result {
                    Ok(surface) => surface,
                    Err(vk1_0::Result::ERROR_OUT_OF_HOST_MEMORY) => out_of_host_memory(),
                    Err(vk1_0::Result::ERROR_OUT_OF_DEVICE_MEMORY) => {
                        return Err(OutOfMemory.into())
                    }
                    Err(err) => unexpected_result(err),
                }
            }

            #[cfg(any(
                target_os = "linux",
                target_os = "dragonfly",
                target_os = "freebsd",
                target_os = "netbsd",
                target_os = "openbsd"
            ))]
            RawWindowHandle::Xlib(handle) => {
                if !self.instance.enabled().khr_xlib_surface {
                    return Err(CreateSurfaceError::UnsupportedWindow {
                        window: RawWindowHandleKind::Xlib,
                        source: Some(Box::new(RequiredExtensionIsNotAvailable {
                            extension: "VK_KHR_xlib_surface",
                        })),
                    });
                }

                unsafe {
                    self.instance.create_xlib_surface_khr(
                        &XlibSurfaceCreateInfoKHR {
                            window: handle.window.into(),
                            dpy: handle.display,
                            ..XlibSurfaceCreateInfoKHR::default()
                        },
                        None,
                    )
                }
                .result()
                .map_err(|err| match err {
                    vk1_0::Result::ERROR_OUT_OF_HOST_MEMORY => out_of_host_memory(),
                    vk1_0::Result::ERROR_OUT_OF_DEVICE_MEMORY => OutOfMemory,
                    _ => unexpected_result(err),
                })?
            }
            _ => {
                debug_assert_eq!(
                    RawWindowHandleKind::of(&window),
                    RawWindowHandleKind::Unknown,
                );

                return Err(CreateSurfaceError::UnsupportedWindow {
                    window: RawWindowHandleKind::Unknown,
                    source: None,
                });
            }
        };

        Ok(Surface::make(
            surface,
            AtomicBool::new(false),
            SurfaceInfo { window },
        ))
    }
}

#[derive(Debug)]
struct RequiredExtensionIsNotAvailable {
    extension: &'static str,
}

impl fmt::Display for RequiredExtensionIsNotAvailable {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            fmt,
            "Required extension '{}' is not available",
            self.extension
        )
    }
}

impl std::error::Error for RequiredExtensionIsNotAvailable {}

unsafe extern "system" fn debug_report_callback(
    flags: DebugReportFlagsEXT,
    object_type: DebugReportObjectTypeEXT,
    _object: u64,
    _location: usize,
    _message_code: i32,
    p_layer_prefix: *const c_char,
    p_message: *const c_char,
    _p_user_data: *mut c_void,
) -> vk1_0::Bool32 {
    let layer_prefix = CStr::from_ptr(p_layer_prefix);

    let message = CStr::from_ptr(p_message);

    if flags.contains(DebugReportFlagsEXT::ERROR_EXT) {
        tracing::error!("{:?}: {:?} | {:?}", layer_prefix, object_type, message);
        // panic!("{:?}: {:?} | {:?}", layer_prefix, object_type, message);
    } else if flags.contains(DebugReportFlagsEXT::PERFORMANCE_WARNING_EXT) {
        tracing::warn!("{:?}: {:?} | {:?}", layer_prefix, object_type, message);
    } else if flags.contains(DebugReportFlagsEXT::WARNING_EXT) {
        tracing::warn!("{:?}: {:?} | {:?}", layer_prefix, object_type, message);
    } else if flags.contains(DebugReportFlagsEXT::INFORMATION_EXT) {
        tracing::info!("{:?}: {:?} | {:?}", layer_prefix, object_type, message);
    } else if flags.contains(DebugReportFlagsEXT::DEBUG_EXT) {
        tracing::debug!("{:?}: {:?} | {:?}", layer_prefix, object_type, message);
    } else {
        tracing::trace!("{:?}: {:?} | {:?}", layer_prefix, object_type, message);
    }

    0
}
