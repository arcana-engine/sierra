use {
    super::unexpected_result,
    crate::{
        out_of_host_memory,
        surface::{SurfaceError, SurfaceInfo},
        OutOfMemory,
    },
    erupt::{extensions::khr_surface::SurfaceKHR, vk1_0},
    std::{
        fmt::Debug,
        sync::atomic::{AtomicBool, Ordering},
    },
};

#[derive(Debug)]
pub(crate) struct Inner {
    pub handle: SurfaceKHR,
    pub used: AtomicBool,
    pub info: SurfaceInfo,
}

#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct Surface {
    inner: std::sync::Arc<Inner>,
}

impl std::cmp::PartialEq for Surface {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(&*self.inner, &*other.inner)
    }
}

impl std::cmp::Eq for Surface {}

impl std::hash::Hash for Surface {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::ptr::hash(&*self.inner, state)
    }
}

impl Surface {
    pub(crate) fn make(
        handle: SurfaceKHR,
        used: AtomicBool,
        info: SurfaceInfo,
    ) -> Self {
        Surface {
            inner: std::sync::Arc::new(Inner { handle, used, info }),
        }
    }

    pub(crate) fn handle(&self) -> SurfaceKHR {
        self.inner.handle
    }

    pub(crate) fn mark_used(&self) -> Result<(), SurfaceError> {
        if self.inner.used.fetch_or(true, Ordering::SeqCst) {
            return Err(SurfaceError::AlreadyUsed);
        } else {
            Ok(())
        }
    }

    pub fn info(&self) -> &SurfaceInfo {
        &self.inner.info
    }
}

pub(crate) fn surface_error_from_erupt(err: vk1_0::Result) -> SurfaceError {
    match err {
        vk1_0::Result::ERROR_OUT_OF_HOST_MEMORY => out_of_host_memory(),
        vk1_0::Result::ERROR_OUT_OF_DEVICE_MEMORY => {
            SurfaceError::OutOfMemory {
                source: OutOfMemory,
            }
        }
        vk1_0::Result::ERROR_SURFACE_LOST_KHR => SurfaceError::SurfaceLost,
        _ => unexpected_result(err),
    }
}
