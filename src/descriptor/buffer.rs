use crate::{
    access::AccessFlags,
    buffer::{Buffer, BufferRange, BufferRangeState},
    encode::Encoder,
    queue::QueueId,
    stage::PipelineStageFlags,
    Device, OutOfMemory,
};

/// Interface for all types that can be used as `UniformBuffer` descriptor.
pub trait UniformBuffer {
    /// Compare with image view currently bound to descriptor set.
    /// Returns `true` if self is equivalent specified image view,
    /// and no update is required.
    fn eq(&self, range: &BufferRange) -> bool;

    /// Returns `BufferRange` equivalent to self.
    fn get_range(&self, device: &Device) -> Result<BufferRange, OutOfMemory>;

    /// Synchronize `self` with access as sampled image in specified stages.
    /// Record commands as necessary.
    /// Any commands that may be recorded here must "happen-before" operation
    /// that will access this image.
    /// Operations must be either recorded afterward
    /// or into separate encoder that will be submitted after this one.
    fn sync<'a>(
        &'a mut self,
        stages: PipelineStageFlags,
        queue: QueueId,
        encoder: &mut Encoder<'a>,
    );
}

impl UniformBuffer for Buffer {
    #[inline]
    fn eq(&self, range: &BufferRange) -> bool {
        range.buffer == *self && range.offset == 0 && range.size == self.info().size
    }

    #[inline]
    fn get_range(&self, _device: &Device) -> Result<BufferRange, OutOfMemory> {
        Ok(BufferRange::whole(self.clone()))
    }

    #[inline]
    fn sync<'a>(
        &'a mut self,
        _stages: PipelineStageFlags,
        _queue: QueueId,
        _encoder: &mut Encoder<'a>,
    ) {
        // Must be externally synchronized.
    }
}

impl UniformBuffer for BufferRange {
    #[inline]
    fn eq(&self, range: &BufferRange) -> bool {
        *self == *range
    }

    #[inline]
    fn get_range(&self, _device: &Device) -> Result<BufferRange, OutOfMemory> {
        Ok(self.clone())
    }

    #[inline]
    fn sync<'a>(
        &'a mut self,
        _stages: PipelineStageFlags,
        _queue: QueueId,
        _encoder: &mut Encoder<'a>,
    ) {
        // Must be externally synchronized.
    }
}

impl UniformBuffer for BufferRangeState {
    #[inline]
    fn eq(&self, range: &BufferRange) -> bool {
        self.range == *range
    }

    #[inline]
    fn get_range(&self, _device: &Device) -> Result<BufferRange, OutOfMemory> {
        Ok(self.range.clone())
    }

    #[inline]
    fn sync<'a>(
        &'a mut self,
        stages: PipelineStageFlags,
        queue: QueueId,
        encoder: &mut Encoder<'a>,
    ) {
        self.access(AccessFlags::SHADER_READ, stages, queue, encoder);
    }
}

/// Interface for all types that can be used as `StorageBuffer` descriptor.
pub trait StorageBuffer {
    /// Compare with image view currently bound to descriptor set.
    /// Returns `true` if self is equivalent specified image view,
    /// and no update is required.
    fn eq(&self, range: &BufferRange) -> bool;

    /// Returns `BufferRange` equivalent to self.
    fn get_range(&self, device: &Device) -> Result<BufferRange, OutOfMemory>;

    /// Synchronize `self` with access as sampled image in specified stages.
    /// Record commands as necessary.
    /// Any commands that may be recorded here must "happen-before" operation
    /// that will access this image.
    /// Operations must be either recorded afterward
    /// or into separate encoder that will be submitted after this one.
    fn sync<'a>(
        &'a mut self,
        stages: PipelineStageFlags,
        queue: QueueId,
        encoder: &mut Encoder<'a>,
    );
}

impl StorageBuffer for Buffer {
    #[inline]
    fn eq(&self, range: &BufferRange) -> bool {
        range.buffer == *self && range.offset == 0 && range.size == self.info().size
    }

    #[inline]
    fn get_range(&self, _device: &Device) -> Result<BufferRange, OutOfMemory> {
        Ok(BufferRange::whole(self.clone()))
    }

    #[inline]
    fn sync<'a>(
        &'a mut self,
        _stages: PipelineStageFlags,
        _queue: QueueId,
        _encoder: &mut Encoder<'a>,
    ) {
        // Must be externally synchronized.
    }
}

impl StorageBuffer for BufferRange {
    #[inline]
    fn eq(&self, range: &BufferRange) -> bool {
        *self == *range
    }

    #[inline]
    fn get_range(&self, _device: &Device) -> Result<BufferRange, OutOfMemory> {
        Ok(self.clone())
    }

    #[inline]
    fn sync<'a>(
        &'a mut self,
        _stages: PipelineStageFlags,
        _queue: QueueId,
        _encoder: &mut Encoder<'a>,
    ) {
        // Must be externally synchronized.
    }
}

impl StorageBuffer for BufferRangeState {
    #[inline]
    fn eq(&self, range: &BufferRange) -> bool {
        self.range == *range
    }

    #[inline]
    fn get_range(&self, _device: &Device) -> Result<BufferRange, OutOfMemory> {
        Ok(self.range.clone())
    }

    #[inline]
    fn sync<'a>(
        &'a mut self,
        stages: PipelineStageFlags,
        queue: QueueId,
        encoder: &mut Encoder<'a>,
    ) {
        self.access(
            AccessFlags::SHADER_READ | AccessFlags::SHADER_WRITE,
            stages,
            queue,
            encoder,
        );
    }
}
