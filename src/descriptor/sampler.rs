use {
    super::{DescriptorBindingFlags, TypedDescriptorBinding},
    crate::{sampler::Sampler, Device, OutOfMemory},
};

impl TypedDescriptorBinding for Sampler {
    const COUNT: u32 = 1;
    const FLAGS: DescriptorBindingFlags = DescriptorBindingFlags::empty();
    type Descriptors = [Sampler; 1];

    #[inline]
    fn eq(&self, range: &[Sampler; 1]) -> bool {
        *self == range[0]
    }

    #[inline]
    fn get_descriptors(&self, _device: &Device) -> Result<[Sampler; 1], OutOfMemory> {
        Ok([self.clone()])
    }
}

impl<const N: usize> TypedDescriptorBinding for [Sampler; N] {
    const COUNT: u32 = N as u32;
    const FLAGS: DescriptorBindingFlags = DescriptorBindingFlags::empty();
    type Descriptors = [Sampler; N];

    #[inline]
    fn eq(&self, range: &[Sampler; N]) -> bool {
        *self == *range
    }

    #[inline]
    fn get_descriptors(&self, _device: &Device) -> Result<[Sampler; N], OutOfMemory> {
        Ok(self.clone())
    }
}

impl<const N: usize> TypedDescriptorBinding for arrayvec::ArrayVec<Sampler, N> {
    const COUNT: u32 = N as u32;
    const FLAGS: DescriptorBindingFlags = DescriptorBindingFlags::PARTIALLY_BOUND;
    type Descriptors = arrayvec::ArrayVec<Sampler, N>;

    #[inline]
    fn eq(&self, range: &arrayvec::ArrayVec<Sampler, N>) -> bool {
        *self == *range
    }

    #[inline]
    fn get_descriptors(
        &self,
        _device: &Device,
    ) -> Result<arrayvec::ArrayVec<Sampler, N>, OutOfMemory> {
        Ok(self.clone())
    }
}
