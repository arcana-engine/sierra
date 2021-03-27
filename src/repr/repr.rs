use {
    super::{native::ShaderNative, pad::Padding},
    bytemuck::Pod,
    core::mem::size_of,
};

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
    fn copy_to_repr(&self) -> Self::Type;

    /// Copy as std140 representation into bytes slice
    fn copy_as_repr(&self, bytes: &mut [u8]) {
        let size = size_of::<Self::Type>();
        assert_eq!(bytes.len(), size);
        bytes.copy_from_slice(bytemuck::bytes_of(&self.copy_to_repr()));
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

    fn copy_to_repr(&self) -> T {
        *self
    }

    fn copy_as_repr(&self, bytes: &mut [u8]) {
        let size = size_of::<T>();
        assert_eq!(bytes.len(), size);
        bytes.copy_from_slice(bytemuck::bytes_of(self));
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

    fn copy_to_repr(&self) -> T {
        *self
    }

    fn copy_as_repr(&self, bytes: &mut [u8]) {
        let size = size_of::<T>();
        assert_eq!(bytes.len(), size);
        bytes.copy_from_slice(bytemuck::bytes_of(self));
    }
}
