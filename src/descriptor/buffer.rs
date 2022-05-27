use crate::BufferView;

use {
    super::{DescriptorBinding, DescriptorBindingFlags},
    crate::{
        buffer::{Buffer, BufferRange},
        Device, OutOfMemory,
    },
};

impl DescriptorBinding for Buffer {
    const COUNT: u32 = 1;
    const FLAGS: DescriptorBindingFlags = DescriptorBindingFlags::empty();
    type DescriptorArray = [BufferRange; 1];

    #[inline]
    fn eq(&self, range: &[BufferRange; 1]) -> bool {
        range[0].buffer == *self && range[0].offset == 0 && range[0].size == self.info().size
    }

    #[inline]
    fn get_descriptors(&self, _device: &Device) -> Result<[BufferRange; 1], OutOfMemory> {
        Ok([BufferRange::whole(self.clone())])
    }
}

impl<const N: usize> DescriptorBinding for [Buffer; N] {
    const COUNT: u32 = N as u32;
    const FLAGS: DescriptorBindingFlags = DescriptorBindingFlags::empty();
    type DescriptorArray = [BufferRange; N];

    #[inline]
    fn eq(&self, range: &[BufferRange; N]) -> bool {
        (0..N).all(|i| {
            range[i].buffer == self[i]
                && range[i].offset == 0
                && range[i].size == self[i].info().size
        })
    }

    #[inline]
    fn get_descriptors(&self, _device: &Device) -> Result<[BufferRange; N], OutOfMemory> {
        Ok(self.clone().map(BufferRange::whole))
    }
}

impl DescriptorBinding for BufferRange {
    const COUNT: u32 = 1;
    const FLAGS: DescriptorBindingFlags = DescriptorBindingFlags::empty();
    type DescriptorArray = [BufferRange; 1];

    #[inline]
    fn eq(&self, range: &[BufferRange; 1]) -> bool {
        *self == range[0]
    }

    #[inline]
    fn get_descriptors(&self, _device: &Device) -> Result<[BufferRange; 1], OutOfMemory> {
        Ok([self.clone()])
    }
}

impl<const N: usize> DescriptorBinding for [BufferRange; N] {
    const COUNT: u32 = N as u32;
    const FLAGS: DescriptorBindingFlags = DescriptorBindingFlags::empty();
    type DescriptorArray = [BufferRange; N];

    #[inline]
    fn eq(&self, range: &[BufferRange; N]) -> bool {
        *self == *range
    }

    #[inline]
    fn get_descriptors(&self, _device: &Device) -> Result<[BufferRange; N], OutOfMemory> {
        Ok(self.clone())
    }
}

impl<const N: usize> DescriptorBinding for arrayvec::ArrayVec<BufferRange, N> {
    const COUNT: u32 = N as u32;
    const FLAGS: DescriptorBindingFlags = DescriptorBindingFlags::PARTIALLY_BOUND;
    type DescriptorArray = arrayvec::ArrayVec<BufferRange, N>;

    #[inline]
    fn eq(&self, range: &arrayvec::ArrayVec<BufferRange, N>) -> bool {
        *self == *range
    }

    #[inline]
    fn get_descriptors(
        &self,
        _device: &Device,
    ) -> Result<arrayvec::ArrayVec<BufferRange, N>, OutOfMemory> {
        Ok(self.clone())
    }
}

impl DescriptorBinding for BufferView {
    const COUNT: u32 = 1;
    const FLAGS: DescriptorBindingFlags = DescriptorBindingFlags::empty();
    type DescriptorArray = [BufferView; 1];

    #[inline]
    fn eq(&self, range: &[BufferView; 1]) -> bool {
        *self == range[0]
    }

    #[inline]
    fn get_descriptors(&self, _device: &Device) -> Result<[BufferView; 1], OutOfMemory> {
        Ok([self.clone()])
    }
}

impl<const N: usize> DescriptorBinding for [BufferView; N] {
    const COUNT: u32 = N as u32;
    const FLAGS: DescriptorBindingFlags = DescriptorBindingFlags::empty();
    type DescriptorArray = [BufferView; N];

    #[inline]
    fn eq(&self, range: &[BufferView; N]) -> bool {
        *self == *range
    }

    #[inline]
    fn get_descriptors(&self, _device: &Device) -> Result<[BufferView; N], OutOfMemory> {
        Ok(self.clone())
    }
}

impl<const N: usize> DescriptorBinding for arrayvec::ArrayVec<BufferView, N> {
    const COUNT: u32 = N as u32;
    const FLAGS: DescriptorBindingFlags = DescriptorBindingFlags::PARTIALLY_BOUND;
    type DescriptorArray = arrayvec::ArrayVec<BufferView, N>;

    #[inline]
    fn eq(&self, range: &arrayvec::ArrayVec<BufferView, N>) -> bool {
        *self == *range
    }

    #[inline]
    fn get_descriptors(
        &self,
        _device: &Device,
    ) -> Result<arrayvec::ArrayVec<BufferView, N>, OutOfMemory> {
        Ok(self.clone())
    }
}
