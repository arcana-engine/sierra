use {
    super::{AsDescriptors, Descriptors, SamplerDescriptor, TypedDescriptors},
    crate::{sampler::Sampler, Device, OutOfMemory},
};

impl<const N: usize> TypedDescriptors<SamplerDescriptor> for [Sampler; N] {
    fn descriptors(&self) -> Descriptors<'_> {
        Descriptors::Sampler(self)
    }
}

impl AsDescriptors for Sampler {
    const COUNT: u32 = 1;
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

impl<const N: usize> AsDescriptors for [Sampler; N] {
    const COUNT: u32 = N as u32;
    type Descriptors = [Sampler; N];

    #[inline]
    fn eq(&self, range: &[Sampler; N]) -> bool {
        *self == *range
    }

    #[inline]
    fn get_descriptors(&self, _device: &Device) -> Result<[Sampler; N], OutOfMemory> {
        let mut result = arrayvec::ArrayVec::new();

        for me in self {
            unsafe {
                result.push_unchecked(me.clone());
            }
        }

        Ok(result.into_inner().unwrap())
    }
}
