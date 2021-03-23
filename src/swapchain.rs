pub use crate::backend::{Swapchain, SwapchainImage};
use crate::{image::Image, semaphore::Semaphore};

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
