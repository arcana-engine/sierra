use {
    super::native::ShaderNative,
    bytemuck::{Pod, Zeroable},
};

/// POD replacement for [`bool`].
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct boolean(pub u8);

unsafe impl Zeroable for boolean {}
unsafe impl Pod for boolean {}

unsafe impl ShaderNative for boolean {
    const ALIGN_MASK: usize = 0;
    const ARRAY_PADDING_140: usize = 15;
    const ARRAY_PADDING_430: usize = 0;
    type ArrayPadding140 = [u8; 15];
    type ArrayPadding430 = [u8; 0];
}

unsafe impl ShaderNative for i32 {
    const ALIGN_MASK: usize = 3;
    const ARRAY_PADDING_140: usize = 12;
    const ARRAY_PADDING_430: usize = 0;
    type ArrayPadding140 = [u8; 12];
    type ArrayPadding430 = [u8; 0];
}

unsafe impl ShaderNative for u32 {
    const ALIGN_MASK: usize = 3;
    const ARRAY_PADDING_140: usize = 12;
    const ARRAY_PADDING_430: usize = 0;
    type ArrayPadding140 = [u8; 12];
    type ArrayPadding430 = [u8; 0];
}

unsafe impl ShaderNative for f32 {
    const ALIGN_MASK: usize = 3;
    const ARRAY_PADDING_140: usize = 12;
    const ARRAY_PADDING_430: usize = 0;
    type ArrayPadding140 = [u8; 12];
    type ArrayPadding430 = [u8; 0];
}

unsafe impl ShaderNative for f64 {
    const ALIGN_MASK: usize = 7;
    const ARRAY_PADDING_140: usize = 8;
    const ARRAY_PADDING_430: usize = 0;
    type ArrayPadding140 = [u8; 8];
    type ArrayPadding430 = [u8; 0];
}
