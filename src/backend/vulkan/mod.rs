macro_rules! assert_owner {
    ($resource:expr, $owner:expr) => {
        assert!($resource.is_owned_by(&$owner));
    };
}

mod access;
mod convert;
mod descriptor;
mod device;
mod encode;
mod graphics;
mod physical;
mod queue;
mod resources;
mod surface;
mod swapchain;

pub use self::{
    descriptor::*, device::*, encode::*, graphics::*, physical::*, queue::*, resources::*,
    surface::*, swapchain::*,
};

#[track_caller]
fn device_lost() -> ! {
    panic!("Device lost")
}

#[track_caller]
fn unexpected_result(result: erupt::vk1_0::Result) -> ! {
    panic!("Unexpected Vulkan result {}", result)
}
