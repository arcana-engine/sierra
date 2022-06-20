pub use crate::backend::{Queue, Swapchain, SwapchainImage};

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

    /// The time when the image of the swapchain was actually displayed.
    pub actual_present_time: u64,

    /// The time when the image of the swapchain could have been displayed.
    /// This may differ from [`actual_present_time`] if the application requested that the image be presented no sooner than [`desired_present_time`]
    pub earliest_present_time: u64,

    /// An indication of how early the [`Queue::present_with_timing`] was processed
    /// compared to how soon it needed to be processed, and still be presented at [`earliest_present_time`].
    pub present_margin: u64,
}
