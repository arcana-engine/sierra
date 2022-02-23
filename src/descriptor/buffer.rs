use crate::BufferView;

use {
    super::{DescriptorBindingFlags, TypedDescriptorBinding},
    crate::{
        buffer::{Buffer, BufferRange},
        Device, OutOfMemory,
    },
};

impl TypedDescriptorBinding for Buffer {
    const COUNT: u32 = 1;
    const FLAGS: DescriptorBindingFlags = DescriptorBindingFlags::empty();
    type Descriptors = [BufferRange; 1];

    #[inline]
    fn eq(&self, range: &[BufferRange; 1]) -> bool {
        range[0].buffer == *self && range[0].offset == 0 && range[0].size == self.info().size
    }

    #[inline]
    fn get_descriptors(&self, _device: &Device) -> Result<[BufferRange; 1], OutOfMemory> {
        Ok([BufferRange::whole(self.clone())])
    }
}

impl<const N: usize> TypedDescriptorBinding for [Buffer; N] {
    const COUNT: u32 = N as u32;
    const FLAGS: DescriptorBindingFlags = DescriptorBindingFlags::empty();
    type Descriptors = [BufferRange; N];

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

impl TypedDescriptorBinding for BufferRange {
    const COUNT: u32 = 1;
    const FLAGS: DescriptorBindingFlags = DescriptorBindingFlags::empty();
    type Descriptors = [BufferRange; 1];

    #[inline]
    fn eq(&self, range: &[BufferRange; 1]) -> bool {
        *self == range[0]
    }

    #[inline]
    fn get_descriptors(&self, _device: &Device) -> Result<[BufferRange; 1], OutOfMemory> {
        Ok([self.clone()])
    }
}

impl<const N: usize> TypedDescriptorBinding for [BufferRange; N] {
    const COUNT: u32 = N as u32;
    const FLAGS: DescriptorBindingFlags = DescriptorBindingFlags::empty();
    type Descriptors = [BufferRange; N];

    #[inline]
    fn eq(&self, range: &[BufferRange; N]) -> bool {
        *self == *range
    }

    #[inline]
    fn get_descriptors(&self, _device: &Device) -> Result<[BufferRange; N], OutOfMemory> {
        Ok(self.clone())
    }
}

impl<const N: usize> TypedDescriptorBinding for arrayvec::ArrayVec<BufferRange, N> {
    const COUNT: u32 = N as u32;
    const FLAGS: DescriptorBindingFlags = DescriptorBindingFlags::PARTIALLY_BOUND;
    type Descriptors = arrayvec::ArrayVec<BufferRange, N>;

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

impl TypedDescriptorBinding for BufferView {
    const COUNT: u32 = 1;
    const FLAGS: DescriptorBindingFlags = DescriptorBindingFlags::empty();
    type Descriptors = [BufferView; 1];

    #[inline]
    fn eq(&self, range: &[BufferView; 1]) -> bool {
        *self == range[0]
    }

    #[inline]
    fn get_descriptors(&self, _device: &Device) -> Result<[BufferView; 1], OutOfMemory> {
        Ok([self.clone()])
    }
}

impl<const N: usize> TypedDescriptorBinding for [BufferView; N] {
    const COUNT: u32 = N as u32;
    const FLAGS: DescriptorBindingFlags = DescriptorBindingFlags::empty();
    type Descriptors = [BufferView; N];

    #[inline]
    fn eq(&self, range: &[BufferView; N]) -> bool {
        *self == *range
    }

    #[inline]
    fn get_descriptors(&self, _device: &Device) -> Result<[BufferView; N], OutOfMemory> {
        Ok(self.clone())
    }
}

impl<const N: usize> TypedDescriptorBinding for arrayvec::ArrayVec<BufferView, N> {
    const COUNT: u32 = N as u32;
    const FLAGS: DescriptorBindingFlags = DescriptorBindingFlags::PARTIALLY_BOUND;
    type Descriptors = arrayvec::ArrayVec<BufferView, N>;

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
