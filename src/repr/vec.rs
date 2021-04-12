use {
    super::{native::ShaderNative, scalar::boolean},
    bytemuck::{Pod, Zeroable},
};

/// Generic vector type.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct vec<T, const N: usize>(pub [T; N]);

impl<T: Default + Copy, const N: usize> Default for vec<T, N> {
    fn default() -> Self {
        vec([Default::default(); N])
    }
}

unsafe impl<T: Zeroable, const N: usize> Zeroable for vec<T, N> {}
unsafe impl<T: Pod, const N: usize> Pod for vec<T, N> {}

impl<T: Pod, const N: usize> From<[T; N]> for vec<T, N> {
    fn from(value: [T; N]) -> Self {
        vec(value)
    }
}

/// Vector with 2-elements.
pub type vec2<T = f32> = vec<T, 2>;

unsafe impl ShaderNative for vec2<boolean> {
    const ALIGN_MASK: usize = 1;
    const ARRAY_PADDING_140: usize = 14;
    const ARRAY_PADDING_430: usize = 0;
    type ArrayPadding140 = [u8; 14];
    type ArrayPadding430 = [u8; 0];
}
unsafe impl ShaderNative for vec2<i32> {
    const ALIGN_MASK: usize = 7;
    const ARRAY_PADDING_140: usize = 8;
    const ARRAY_PADDING_430: usize = 0;
    type ArrayPadding140 = [u8; 8];
    type ArrayPadding430 = [u8; 0];
}
unsafe impl ShaderNative for vec2<u32> {
    const ALIGN_MASK: usize = 7;
    const ARRAY_PADDING_140: usize = 8;
    const ARRAY_PADDING_430: usize = 0;
    type ArrayPadding140 = [u8; 8];
    type ArrayPadding430 = [u8; 0];
}
unsafe impl ShaderNative for vec2<f32> {
    const ALIGN_MASK: usize = 7;
    const ARRAY_PADDING_140: usize = 8;
    const ARRAY_PADDING_430: usize = 0;
    type ArrayPadding140 = [u8; 8];
    type ArrayPadding430 = [u8; 0];
}
unsafe impl ShaderNative for vec2<f64> {
    const ALIGN_MASK: usize = 15;
    const ARRAY_PADDING_140: usize = 0;
    const ARRAY_PADDING_430: usize = 0;
    type ArrayPadding140 = [u8; 0];
    type ArrayPadding430 = [u8; 0];
}

/// Vector with 3-elements.
pub type vec3<T = f32> = vec<T, 3>;

unsafe impl ShaderNative for vec3<boolean> {
    const ALIGN_MASK: usize = 3;
    const ARRAY_PADDING_140: usize = 13;
    const ARRAY_PADDING_430: usize = 1;
    type ArrayPadding140 = [u8; 13];
    type ArrayPadding430 = [u8; 1];
}
unsafe impl ShaderNative for vec3<i32> {
    const ALIGN_MASK: usize = 15;
    const ARRAY_PADDING_140: usize = 4;
    const ARRAY_PADDING_430: usize = 4;
    type ArrayPadding140 = [u8; 4];
    type ArrayPadding430 = [u8; 4];
}
unsafe impl ShaderNative for vec3<u32> {
    const ALIGN_MASK: usize = 15;
    const ARRAY_PADDING_140: usize = 4;
    const ARRAY_PADDING_430: usize = 4;
    type ArrayPadding140 = [u8; 4];
    type ArrayPadding430 = [u8; 4];
}
unsafe impl ShaderNative for vec3<f32> {
    const ALIGN_MASK: usize = 15;
    const ARRAY_PADDING_140: usize = 4;
    const ARRAY_PADDING_430: usize = 4;
    type ArrayPadding140 = [u8; 4];
    type ArrayPadding430 = [u8; 4];
}
unsafe impl ShaderNative for vec3<f64> {
    const ALIGN_MASK: usize = 31;
    const ARRAY_PADDING_140: usize = 8;
    const ARRAY_PADDING_430: usize = 8;
    type ArrayPadding140 = [u8; 8];
    type ArrayPadding430 = [u8; 8];
}

/// Vector with 4-elements.
pub type vec4<T = f32> = vec<T, 4>;

unsafe impl ShaderNative for vec4<boolean> {
    const ALIGN_MASK: usize = 3;
    const ARRAY_PADDING_140: usize = 12;
    const ARRAY_PADDING_430: usize = 0;
    type ArrayPadding140 = [u8; 12];
    type ArrayPadding430 = [u8; 0];
}
unsafe impl ShaderNative for vec4<i32> {
    const ALIGN_MASK: usize = 15;
    const ARRAY_PADDING_140: usize = 0;
    const ARRAY_PADDING_430: usize = 0;
    type ArrayPadding140 = [u8; 0];
    type ArrayPadding430 = [u8; 0];
}
unsafe impl ShaderNative for vec4<u32> {
    const ALIGN_MASK: usize = 15;
    const ARRAY_PADDING_140: usize = 0;
    const ARRAY_PADDING_430: usize = 0;
    type ArrayPadding140 = [u8; 0];
    type ArrayPadding430 = [u8; 0];
}
unsafe impl ShaderNative for vec4<f32> {
    const ALIGN_MASK: usize = 15;
    const ARRAY_PADDING_140: usize = 0;
    const ARRAY_PADDING_430: usize = 0;
    type ArrayPadding140 = [u8; 0];
    type ArrayPadding430 = [u8; 0];
}
unsafe impl ShaderNative for vec4<f64> {
    const ALIGN_MASK: usize = 31;
    const ARRAY_PADDING_140: usize = 0;
    const ARRAY_PADDING_430: usize = 0;
    type ArrayPadding140 = [u8; 0];
    type ArrayPadding430 = [u8; 0];
}

pub type bvec2 = vec2<bool>;
pub type bvec3 = vec3<bool>;
pub type bvec4 = vec4<bool>;
pub type ivec2 = vec2<i32>;
pub type ivec3 = vec3<i32>;
pub type ivec4 = vec4<i32>;
pub type uvec2 = vec2<u32>;
pub type uvec3 = vec3<u32>;
pub type uvec4 = vec4<u32>;
pub type dvec2 = vec2<f64>;
pub type dvec3 = vec3<f64>;
pub type dvec4 = vec4<f64>;
