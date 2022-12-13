use std::{error::Error, fmt::Debug, num::NonZeroU32, sync::Arc};

use raw_window_handle::{RawDisplayHandle, RawWindowHandle};

use crate::{
    assert_error, format::Format, image::ImageUsage, DeviceLost, Extent2, IteratorExt, OutOfMemory,
};

pub use crate::backend::Surface;

#[derive(Clone, Copy, Debug, thiserror::Error, PartialEq, Eq)]
pub enum SurfaceError {
    #[error(transparent)]
    OutOfMemory {
        #[from]
        source: OutOfMemory,
    },

    #[error(transparent)]
    DeviceLost {
        #[from]
        source: DeviceLost,
    },

    #[error("Image usage {{{usage:?}}} is not supported for surface images")]
    UsageNotSupported { usage: ImageUsage },

    #[error("Surface was lost")]
    SurfaceLost,

    #[error("Format {{{format:?}}} is not supported for surface images")]
    FormatUnsupported { format: Format },

    #[error("Presentation mode {{{mode:?}}} is not supported for surface images")]
    PresentModeUnsupported { mode: PresentMode },

    #[error("Surface is already used")]
    AlreadyUsed,

    #[error("Native window is in use")]
    WindowIsInUse,

    #[error("Initialization failed")]
    InitializationFailed,

    #[error("Too many images acquired")]
    TooManyAcquired,

    #[error("Surface not configured")]
    NotConfigured,
}

#[derive(Debug, thiserror::Error)]
pub enum CreateSurfaceError {
    #[error(transparent)]
    SurfaceError {
        #[from]
        source: SurfaceError,
    },

    #[error("Surface is not supported by device")]
    NotSupported,

    #[error("Window handle of kind {{{window:?}}} is not supported")]
    UnsupportedWindow {
        window: RawWindowHandleKind,
        #[source]
        source: Option<Box<dyn Error + Send + Sync>>,
    },

    #[error("Window handle of kind {{{window:?}}} does not match display of kind {{{display:?}}}")]
    WindowDisplayMismatch {
        window: RawWindowHandleKind,
        display: RawDisplayHandleKind,
    },
}

impl From<OutOfMemory> for CreateSurfaceError {
    fn from(source: OutOfMemory) -> Self {
        CreateSurfaceError::SurfaceError {
            source: SurfaceError::OutOfMemory { source },
        }
    }
}

/// Kind of raw window handles
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum RawWindowHandleKind {
    UiKit,
    AppKit,
    Xlib,
    Xcb,
    Wayland,
    Win32,
    WinRt,
    Web,
    Android,
    Unknown,
}

impl RawWindowHandleKind {
    /// Returns kind of the raw window handle.
    pub fn of(window: &RawWindowHandle) -> Self {
        match window {
            #[cfg(target_os = "android")]
            RawWindowHandle::Android(_) => RawWindowHandleKind::Android,

            #[cfg(target_os = "ios")]
            RawWindowHandle::UiKit(_) => RawWindowHandleKind::IOS,

            #[cfg(target_os = "macos")]
            RawWindowHandle::AppKit(_) => RawWindowHandleKind::AppKit,

            #[cfg(any(
                target_os = "linux",
                target_os = "dragonfly",
                target_os = "freebsd",
                target_os = "netbsd",
                target_os = "openbsd"
            ))]
            RawWindowHandle::Wayland(_) => RawWindowHandleKind::Wayland,

            #[cfg(target_os = "windows")]
            RawWindowHandle::Win32(_) => RawWindowHandleKind::Win32,

            #[cfg(target_os = "windows")]
            RawWindowHandle::WinRt(_) => RawWindowHandleKind::WinRt,

            #[cfg(any(
                target_os = "linux",
                target_os = "dragonfly",
                target_os = "freebsd",
                target_os = "netbsd",
                target_os = "openbsd"
            ))]
            RawWindowHandle::Xcb(_) => RawWindowHandleKind::Xcb,

            #[cfg(any(
                target_os = "linux",
                target_os = "dragonfly",
                target_os = "freebsd",
                target_os = "netbsd",
                target_os = "openbsd"
            ))]
            RawWindowHandle::Xlib(_) => RawWindowHandleKind::Xlib,
            _ => RawWindowHandleKind::Unknown,
        }
    }
}

/// Kind of raw window handles
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum RawDisplayHandleKind {
    UiKit,
    AppKit,
    Xlib,
    Xcb,
    Wayland,
    Windows,
    Web,
    Android,
    Unknown,
}

impl RawDisplayHandleKind {
    /// Returns kind of the raw window handle.
    pub fn of(window: &RawDisplayHandle) -> Self {
        match window {
            #[cfg(target_os = "android")]
            RawDisplayHandle::Android(_) => RawDisplayHandleKind::Android,

            #[cfg(target_os = "ios")]
            RawDisplayHandle::UiKit(_) => RawDisplayHandleKind::IOS,

            #[cfg(target_os = "macos")]
            RawDisplayHandle::AppKit(_) => RawDisplayHandleKind::AppKit,

            #[cfg(any(
                target_os = "linux",
                target_os = "dragonfly",
                target_os = "freebsd",
                target_os = "netbsd",
                target_os = "openbsd"
            ))]
            RawDisplayHandle::Wayland(_) => RawDisplayHandleKind::Wayland,

            #[cfg(target_os = "windows")]
            RawDisplayHandle::Windows(_) => RawDisplayHandleKind::Windows,

            #[cfg(any(
                target_os = "linux",
                target_os = "dragonfly",
                target_os = "freebsd",
                target_os = "netbsd",
                target_os = "openbsd"
            ))]
            RawDisplayHandle::Xcb(_) => RawDisplayHandleKind::Xcb,

            #[cfg(any(
                target_os = "linux",
                target_os = "dragonfly",
                target_os = "freebsd",
                target_os = "netbsd",
                target_os = "openbsd"
            ))]
            RawDisplayHandle::Xlib(_) => RawDisplayHandleKind::Xlib,
            _ => RawDisplayHandleKind::Unknown,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub enum PresentMode {
    Immediate,
    Mailbox,
    Fifo,
    FifoRelaxed,
}

bitflags::bitflags! {
    #[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
    pub struct CompositeAlphaFlags: u32 {
        const OPAQUE = 0x1;
        const PRE_MULTIPLIED = 0x2;
        const POST_MULTIPLIED = 0x4;
        const INHERIT = 0x8;
    }
}

bitflags::bitflags! {
    #[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
    pub struct SurfaceTransformFlags: u32 {
        const IDENTITY = 0x001;
        const ROTATE_90 = 0x002;
        const ROTATE_180 = 0x004;
        const ROTATE_270 = 0x008;
        const HORIZONTAL_MIRROR = 0x010;
        const HORIZONTAL_MIRROR_ROTATE_90 = 0x020;
        const HORIZONTAL_MIRROR_ROTATE_180 = 0x040;
        const HORIZONTAL_MIRROR_ROTATE_270 = 0x080;
        const INHERIT = 0x100;
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub struct SurfaceCapabilities {
    pub supported_families: Arc<[bool]>,
    pub min_image_count: NonZeroU32,
    pub max_image_count: Option<NonZeroU32>,
    pub current_extent: Extent2,
    pub current_transform: SurfaceTransformFlags,
    pub min_image_extent: Extent2,
    pub max_image_extent: Extent2,
    pub supported_usage: ImageUsage,
    pub present_modes: Vec<PresentMode>,
    pub formats: Vec<Format>,
    pub supported_composite_alpha: CompositeAlphaFlags,
}

impl SurfaceCapabilities {
    pub fn get_basic_rendering_format(&self) -> Option<Format> {
        self.formats
            .iter()
            .filter_min_by_key(|f| match f {
                Format::RGB8Srgb | Format::BGR8Srgb => Some(0),
                Format::RGB8Unorm | Format::BGR8Unorm => Some(1),
                Format::RGBA8Srgb | Format::BGRA8Srgb => Some(2),
                Format::RGBA8Unorm | Format::BGRA8Unorm => Some(3),
                _ => None,
            })
            .copied()
    }

    pub fn get_basic_present_mode(&self) -> Option<PresentMode> {
        self.present_modes
            .iter()
            .min_by_key(|m| match m {
                PresentMode::Mailbox => 0,
                PresentMode::Fifo => 2,
                PresentMode::FifoRelaxed => 2,
                PresentMode::Immediate => 3,
            })
            .copied()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SurfaceInfo {
    pub window: RawWindowHandle,
    pub display: RawDisplayHandle,
}

unsafe impl Send for SurfaceInfo {}
unsafe impl Sync for SurfaceInfo {}

#[allow(dead_code)]
fn check_surface_error() {
    assert_error::<SurfaceError>();
    assert_error::<CreateSurfaceError>();
}

///
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PresentationTiming {
    /// An application-provided value that was given to a previous [`Queue::present_with_timing`]
    ///
    /// It can be used to uniquely identify a previous present with the [`Queue::present_with_timing`].
    pub present_id: u32,

    /// An application-provided value that was given to a previous [`Queue::present_with_timing`].
    /// If non-zero, it was used by the application to indicate that an image not be presented any sooner than [`desired_present_time`].
    pub desired_present_time: u64,

    /// The time when the image of the surface was actually displayed.
    pub actual_present_time: u64,

    /// The time when the image of the surface could have been displayed.
    /// This may differ from [`actual_present_time`] if the application requested that the image be presented no sooner than [`desired_present_time`]
    pub earliest_present_time: u64,

    /// An indication of how early the [`Queue::present_with_timing`] was processed
    /// compared to how soon it needed to be processed, and still be presented at [`earliest_present_time`].
    pub present_margin: u64,
}
