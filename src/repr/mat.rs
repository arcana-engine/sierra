use {
    super::{native::ShaderNative, pad::Padded, vec::vec, ShaderRepr, Std140, Std430},
    bytemuck::{Pod, Zeroable},
    core::{
        convert::TryFrom,
        mem::{align_of, align_of_val, size_of, size_of_val},
    },
    std::cmp::Ordering,
};

/// Generic matrix type.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct mat<T, const N: usize, const M: usize>(pub [vec<T, M>; N]);

impl<T: Default + Copy, const N: usize, const M: usize> Default for mat<T, N, M> {
    fn default() -> Self {
        mat([Default::default(); N])
    }
}

// impl<T: Pod, const N: usize, const M: usize> From<[[T; M]; N]> for mat<T, N, M> {
//     fn from(v: [[T; M]; N]) -> Self {
//         bytemuck::cast(v)
//     }
// }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WrongNumberOfElements {
    NotEnoughElements,
    TooManyElements,
}

impl<T: Pod, const N: usize, const M: usize> TryFrom<&[T]> for mat<T, N, M> {
    type Error = WrongNumberOfElements;

    fn try_from(v: &[T]) -> Result<Self, WrongNumberOfElements> {
        debug_assert_eq!(align_of_val(v), align_of::<Self>());

        let len = v.len();

        match len.cmp(&(N * M)) {
            Ordering::Less => {
                debug_assert!(size_of_val(v) < size_of::<Self>());
                Err(WrongNumberOfElements::NotEnoughElements)
            }
            Ordering::Greater => {
                debug_assert!(size_of_val(v) > size_of::<Self>());
                Err(WrongNumberOfElements::TooManyElements)
            }
            Ordering::Equal => {
                debug_assert_eq!(size_of_val(v), size_of::<Self>());
                Ok(bytemuck::cast_slice(v)[0])
            }
        }
    }
}

unsafe impl<T: Zeroable, const N: usize, const M: usize> Zeroable for mat<T, N, M> {}
unsafe impl<T: Pod, const N: usize, const M: usize> Pod for mat<T, N, M> {}

pub type mat2x2<T = f32> = mat<T, 2, 2>;
pub type mat3x2<T = f32> = mat<T, 3, 2>;
pub type mat4x2<T = f32> = mat<T, 4, 2>;

pub type mat2x3<T = f32> = mat<T, 2, 3>;
pub type mat3x3<T = f32> = mat<T, 3, 3>;
pub type mat4x3<T = f32> = mat<T, 4, 3>;
pub type mat3x4<T = f32> = mat<T, 3, 4>;
pub type mat4x4<T = f32> = mat<T, 4, 4>;
pub type mat2x4<T = f32> = mat<T, 2, 4>;

pub type bmat2 = mat2<bool>;
pub type bmat3 = mat3<bool>;
pub type bmat4 = mat4<bool>;

pub type bmat2x2 = mat2x2<bool>;
pub type bmat3x2 = mat3x2<bool>;
pub type bmat4x2 = mat4x2<bool>;

pub type bmat2x3 = mat2x3<bool>;
pub type bmat3x3 = mat3x3<bool>;
pub type bmat4x3 = mat4x3<bool>;

pub type bmat2x4 = mat2x4<bool>;
pub type bmat3x4 = mat3x4<bool>;
pub type bmat4x4 = mat4x4<bool>;

pub type imat2 = mat2<i32>;
pub type imat3 = mat3<i32>;
pub type imat4 = mat4<i32>;

pub type imat2x2 = mat2x2<i32>;
pub type imat3x2 = mat3x2<i32>;
pub type imat4x2 = mat4x2<i32>;

pub type imat2x3 = mat2x3<i32>;
pub type imat3x3 = mat3x3<i32>;
pub type imat4x3 = mat4x3<i32>;

pub type imat2x4 = mat2x4<i32>;
pub type imat3x4 = mat3x4<i32>;
pub type imat4x4 = mat4x4<i32>;

pub type umat2 = mat2<u32>;
pub type umat3 = mat3<u32>;
pub type umat4 = mat4<u32>;

pub type umat2x2 = mat2x2<u32>;
pub type umat3x2 = mat3x2<u32>;
pub type umat4x2 = mat4x2<u32>;

pub type umat2x3 = mat2x3<u32>;
pub type umat3x3 = mat3x3<u32>;
pub type umat4x3 = mat4x3<u32>;

pub type umat2x4 = mat2x4<u32>;
pub type umat3x4 = mat3x4<u32>;
pub type umat4x4 = mat4x4<u32>;

pub type dmat2 = mat2<f64>;
pub type dmat3 = mat3<f64>;
pub type dmat4 = mat4<f64>;

pub type dmat2x2 = mat2x2<f64>;
pub type dmat3x2 = mat3x2<f64>;
pub type dmat4x2 = mat4x2<f64>;

pub type dmat2x3 = mat2x3<f64>;
pub type dmat3x3 = mat3x3<f64>;
pub type dmat4x3 = mat4x3<f64>;

pub type dmat2x4 = mat2x4<f64>;
pub type dmat3x4 = mat3x4<f64>;
pub type dmat4x4 = mat4x4<f64>;

pub type mat2<T = f32> = mat2x2<T>;
pub type mat3<T = f32> = mat3x3<T>;
pub type mat4<T = f32> = mat4x4<T>;

macro_rules! impl_mats {
    ($($n:tt ( $($m:tt)* )),+) => {$($(
        impl<T> ShaderRepr<Std140> for mat<T, $n, $m>
        where
            vec<T, $m>: ShaderNative,
        {
            const ALIGN_MASK: usize = <vec<T, $m> as ShaderNative>::ALIGN_MASK | 15;
            const ARRAY_PADDING: usize = 0;

            type Type = [Padded<vec<T, $m>, <vec<T, $m> as ShaderNative>::ArrayPadding140>; $n];
            type ArrayPadding = [u8; 0];

            fn copy_to_repr(&self, repr: &mut Self::Type) {
                ShaderRepr::<Std140>::copy_to_repr(&self.0, repr)
            }
        }

        impl<T> ShaderRepr<Std430> for mat<T, $n, $m>
        where
            vec<T, $m>: ShaderNative,
        {
            const ALIGN_MASK: usize = <vec<T, $m> as ShaderNative>::ALIGN_MASK | 15;
            const ARRAY_PADDING: usize = 0;

            type Type = [Padded<vec<T, $m>, <vec<T, $m> as ShaderNative>::ArrayPadding430>; $n];
            type ArrayPadding = [u8; 0];

            fn copy_to_repr(&self, repr: &mut Self::Type) {
                ShaderRepr::<Std430>::copy_to_repr(&self.0, repr)
            }
        }

        impl<T: Pod> From<[[T; $m]; $n]> for mat<T, $n, $m> {
            fn from(v: [[T; $m]; $n]) -> Self {
                bytemuck::cast(v)
            }
        }

        impl<T: Pod> From<[T; $m * $n]> for mat<T, $n, $m> {
            fn from(v: [T; $m * $n]) -> Self {
                bytemuck::cast(v)
            }
        }

    )*)*};
}

impl_mats!(2 ( 2 3 4 ), 3 ( 2 3 4), 4 ( 2 3 4));
