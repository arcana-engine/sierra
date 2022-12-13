macro_rules! assert_owner {
    ($resource:expr, $owner:expr) => {
        assert!($resource.is_owned_by(&$owner));
    };
}

mod access;
mod convert;
mod device;
mod encode;
mod epochs;
mod graphics;
mod physical;
mod queue;
mod resources;
mod surface;

pub use self::{
    device::*, encode::*, graphics::*, physical::*, queue::*, resources::*, surface::*,
};

#[track_caller]
fn unexpected_result(result: erupt::vk1_0::Result) -> ! {
    panic!("Unexpected Vulkan result {}", result)
}
