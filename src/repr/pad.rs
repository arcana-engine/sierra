use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Padded<T, P> {
    pub value: T,
    pub pad: P,
}

pub unsafe trait Padding: Copy + 'static {}
unsafe impl<const N: usize> Padding for [u8; N] {}

unsafe impl<T: Zeroable, P: Padding> Zeroable for Padded<T, P> {}
unsafe impl<T: Pod, P: Padding> Pod for Padded<T, P> {}
