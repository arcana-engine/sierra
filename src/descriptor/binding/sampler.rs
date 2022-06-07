use crate::{descriptor::SamplerDescriptor, sampler::Sampler, Device, OutOfMemory};

use super::{DescriptorBinding, DescriptorBindingFlags};

impl DescriptorBinding<SamplerDescriptor> for Sampler {
    const FLAGS: DescriptorBindingFlags = DescriptorBindingFlags::empty();

    #[inline]
    fn is_compatible(&self, sampler: &Sampler) -> bool {
        *self == *sampler
    }

    #[inline]
    fn get_descriptor(&self, _device: &Device) -> Result<Sampler, OutOfMemory> {
        Ok(self.clone())
    }
}
