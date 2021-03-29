use super::{
    pad::Padded,
    repr::{ShaderRepr, Std140, Std430},
};

macro_rules! impl_unsafe_marker_for_array {
    ($( $n:expr ),*) => {
        $(
            impl<T> ShaderRepr<Std140> for [T; $n]
            where
                T: ShaderRepr<Std140>,
            {
                const ALIGN_MASK: usize = T::ALIGN_MASK | 15;
                const ARRAY_PADDING: usize = 0;

                type Type = [Padded<T::Type, T::ArrayPadding>; $n];
                type ArrayPadding = [u8; 0];

                fn copy_to_repr(&self, repr: &mut [Padded<T::Type, T::ArrayPadding>; $n]) {
                    for i in 0..$n {
                        self[i].copy_to_repr(&mut repr[i].value);
                    }
                }
            }

            impl<T> ShaderRepr<Std430> for [T; $n]
            where
                T: ShaderRepr<Std430>,
            {
                const ALIGN_MASK: usize = T::ALIGN_MASK;
                const ARRAY_PADDING: usize = 0;

                type Type = [Padded<T::Type, T::ArrayPadding>; $n];
                type ArrayPadding = [u8; 0];

                fn copy_to_repr(&self, repr: &mut [Padded<T::Type, T::ArrayPadding>; $n]) {
                    for i in 0..$n {
                        self[i].copy_to_repr(&mut repr[i].value);
                    }
                }
            }
        )*
    }
}

impl_unsafe_marker_for_array!(
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 29, 30, 31, 32, 48, 64, 96, 128, 256, 512, 1024, 2048, 4096
);
