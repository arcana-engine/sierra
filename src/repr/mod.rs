#![allow(non_camel_case_types)]

mod array;
mod mat;
mod native;
mod pad;
mod repr;
mod scalar;
mod vec;

pub use {
    self::{mat::*, native::*, pad::*, repr::*, scalar::*, vec::*},
    bytemuck::{Pod, Zeroable},
};

pub const fn pad_size(align_mask: usize, offset: usize) -> usize {
    align_mask - ((offset + align_mask) & align_mask)
}

pub const fn next_offset(align_mask: usize, offset: usize, size: usize) -> usize {
    size + offset + pad_size(align_mask, offset)
}
