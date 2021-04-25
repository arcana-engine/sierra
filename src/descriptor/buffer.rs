use {
    super::{
        AsDescriptors, Descriptors, StorageBufferDescriptor, TypedDescriptors,
        UniformBufferDescriptor,
    },
    crate::{
        buffer::{Buffer, BufferRange},
        Device, OutOfMemory,
    },
};

impl<const N: usize> TypedDescriptors<UniformBufferDescriptor> for [BufferRange; N] {
    fn descriptors(&self) -> Descriptors<'_> {
        Descriptors::UniformBuffer(self)
    }
}

/// Interface for all types that can be used as `UniformBuffer` or `StorageBuffer` descriptor.
impl<const N: usize> TypedDescriptors<StorageBufferDescriptor> for [BufferRange; N] {
    fn descriptors(&self) -> Descriptors<'_> {
        Descriptors::StorageBuffer(self)
    }
}

impl AsDescriptors for Buffer {
    const COUNT: u32 = 1;
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

impl AsDescriptors for BufferRange {
    const COUNT: u32 = 1;
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

impl<const N: usize> AsDescriptors for [BufferRange; N] {
    const COUNT: u32 = N as u32;
    type Descriptors = [BufferRange; N];

    #[inline]
    fn eq(&self, range: &[BufferRange; N]) -> bool {
        *self == *range
    }

    #[inline]
    fn get_descriptors(&self, _device: &Device) -> Result<[BufferRange; N], OutOfMemory> {
        let mut result = arrayvec::ArrayVec::new();

        for me in self {
            unsafe {
                result.push_unchecked(me.clone());
            }
        }

        Ok(result.into_inner().unwrap())
    }
}
