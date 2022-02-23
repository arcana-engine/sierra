#![allow(non_camel_case_types)]

mod array;
mod mat;
mod native;
mod pad;
mod scalar;
mod vec;

use bytemuck::{Pod, Zeroable};

pub use self::{mat::*, native::*, pad::*, scalar::*, vec::*};

pub const fn pad_size(align_mask: usize, offset: usize) -> usize {
    align_mask - ((offset + align_mask) & align_mask)
}

pub const fn align_offset(align_mask: usize, offset: usize) -> usize {
    (offset + align_mask) & !align_mask
}

pub const fn next_offset(align_mask: usize, offset: usize, size: usize) -> usize {
    size + align_offset(align_mask, offset)
}

/// Default layout for [`Repr`].
/// Can be used for both uniforms storage buffers.
#[derive(Clone, Copy, Debug)]
pub enum Std140 {}

/// Can be used only for storage buffers.
#[derive(Clone, Copy, Debug)]
pub enum Std430 {}

/// Type that can be represented in shader.
pub trait ShaderRepr<T = Std140> {
    const ALIGN_MASK: usize;
    const ARRAY_PADDING: usize;

    /// Type with matching layout.
    type Type: Pod;

    /// Padding required after field of `Self::Type` for arrays.
    type ArrayPadding: Padding;

    /// Copy data in this type into its representation.
    fn copy_to_repr(&self, repr: &mut Self::Type);

    fn to_repr(&self) -> Self::Type {
        let mut repr = Zeroable::zeroed();
        self.copy_to_repr(&mut repr);
        repr
    }
}

impl<T> ShaderRepr<Std140> for T
where
    T: ShaderNative,
{
    const ALIGN_MASK: usize = <T as ShaderNative>::ALIGN_MASK;
    const ARRAY_PADDING: usize = <T as ShaderNative>::ARRAY_PADDING_140;
    type Type = T;
    type ArrayPadding = <T as ShaderNative>::ArrayPadding140;

    fn copy_to_repr(&self, repr: &mut T) {
        *repr = *self
    }
}

impl<T> ShaderRepr<Std430> for T
where
    T: ShaderNative,
{
    const ALIGN_MASK: usize = <T as ShaderNative>::ALIGN_MASK;
    const ARRAY_PADDING: usize = <T as ShaderNative>::ARRAY_PADDING_430;
    type Type = T;
    type ArrayPadding = <T as ShaderNative>::ArrayPadding430;

    fn copy_to_repr(&self, repr: &mut T) {
        *repr = *self
    }
}
