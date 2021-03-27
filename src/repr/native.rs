use {super::pad::Padding, bytemuck::Pod};

/// Type that can be represented in shader natively.
/// i.e. with matching layout, and can be copied as-is.
pub unsafe trait ShaderNative: Pod {
    const ALIGN_MASK: usize;

    const ARRAY_PADDING_140: usize;
    const ARRAY_PADDING_430: usize;

    type ArrayPadding140: Padding;
    type ArrayPadding430: Padding;
}
