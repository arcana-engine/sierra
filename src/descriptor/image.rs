use crate::{
    access::AccessFlags,
    encode::Encoder,
    image::{Image, ImageSubresourceState, ImageUsage, Layout},
    queue::QueueId,
    stage::PipelineStageFlags,
    view::{ImageView, ImageViewInfo, ImageViewState},
    Device, OutOfMemory,
};

/// Interface for all types that can be used as `SampledImage` or `CombinedImageSampler` descriptor.
pub trait SampledImage {
    /// Compare with image view currently bound to descriptor set.
    /// Returns `true` if self is equivalent specified image view,
    /// and no update is required.
    fn eq(&self, view: &ImageView) -> bool;

    /// Returns `ImageView` equivalent to self.
    fn get_view(&self, device: &Device) -> Result<ImageView, OutOfMemory>;

    /// Synchronize `self` with access as sampled image in specified stages.
    /// Record commands as necessary.
    /// Any commands that may be recorded here must "happen-before" operation
    /// that will access this image.
    /// Operations must be either recorded afterward
    /// or into separate encoder that will be submitted after this one.
    fn sync<'a>(
        &'a mut self,
        encoder: &mut Encoder<'a>,
        stages: PipelineStageFlags,
        queue: QueueId,
    );
}

impl SampledImage for Image {
    fn eq(&self, view: &ImageView) -> bool {
        *self == view.info().image
    }

    fn get_view(&self, device: &Device) -> Result<ImageView, OutOfMemory> {
        assert!(self.info().usage.contains(ImageUsage::SAMPLED));
        device.create_image_view(ImageViewInfo::new(self.clone()))
    }

    fn sync<'a>(
        &'a mut self,
        _encoder: &mut Encoder<'a>,
        _stages: PipelineStageFlags,
        _queue: QueueId,
    ) {
        // Must be externally synchronized.
    }
}

impl SampledImage for ImageSubresourceState {
    fn eq(&self, view: &ImageView) -> bool {
        self.subresource.image == view.info().image
    }

    fn get_view(&self, device: &Device) -> Result<ImageView, OutOfMemory> {
        assert!(self
            .subresource
            .image
            .info()
            .usage
            .contains(ImageUsage::SAMPLED));
        device.create_image_view(ImageViewInfo::new(self.subresource.image.clone()))
    }

    fn sync<'a>(
        &'a mut self,
        encoder: &mut Encoder<'a>,
        stages: PipelineStageFlags,
        queue: QueueId,
    ) {
        self.access(
            AccessFlags::SHADER_READ,
            stages,
            Layout::ShaderReadOnlyOptimal,
            queue,
            encoder,
        );
    }
}

impl SampledImage for ImageView {
    fn eq(&self, view: &ImageView) -> bool {
        *self == *view
    }

    fn get_view(&self, _device: &Device) -> Result<ImageView, OutOfMemory> {
        assert!(self.info().image.info().usage.contains(ImageUsage::SAMPLED));
        Ok(self.clone())
    }

    fn sync<'a>(
        &'a mut self,
        _encoder: &mut Encoder<'a>,
        _stages: PipelineStageFlags,
        _queue: QueueId,
    ) {
        // Must be externally synchronized.
    }
}

impl SampledImage for ImageViewState {
    fn eq(&self, view: &ImageView) -> bool {
        self.view == *view
    }

    fn get_view(&self, _device: &Device) -> Result<ImageView, OutOfMemory> {
        assert!(self
            .view
            .info()
            .image
            .info()
            .usage
            .contains(ImageUsage::SAMPLED));
        Ok(self.view.clone())
    }

    fn sync<'a>(
        &'a mut self,
        encoder: &mut Encoder<'a>,
        stages: PipelineStageFlags,
        queue: QueueId,
    ) {
        self.access(
            AccessFlags::SHADER_READ,
            stages,
            Layout::ShaderReadOnlyOptimal,
            queue,
            encoder,
        );
    }
}

/// Interface for all types that can be used as `StorageImage` descriptor.
pub trait StorageImage {
    /// Compare with image view currently bound to descriptor set.
    /// Returns `true` if self is equivalent specified image view,
    /// and no update is required.
    fn eq(&self, view: &ImageView) -> bool;

    /// Returns `ImageView` equivalent to self.
    fn get_view(&self, device: &Device) -> Result<ImageView, OutOfMemory>;

    /// Synchronize `self` with access as sampled image in specified stages.
    /// Record commands as necessary.
    /// Any commands that may be recorded here must "happen-before" operation
    /// that will access this image.
    /// Operations must be either recorded afterward
    /// or into separate encoder that will be submitted after this one.
    fn sync<'a>(
        &'a mut self,
        encoder: &mut Encoder<'a>,
        stages: PipelineStageFlags,
        queue: QueueId,
    );
}

impl StorageImage for Image {
    fn eq(&self, view: &ImageView) -> bool {
        *self == view.info().image
    }

    fn get_view(&self, device: &Device) -> Result<ImageView, OutOfMemory> {
        assert!(self.info().usage.contains(ImageUsage::STORAGE));
        device.create_image_view(ImageViewInfo::new(self.clone()))
    }

    fn sync<'a>(
        &'a mut self,
        _encoder: &mut Encoder<'a>,
        _stages: PipelineStageFlags,
        _queue: QueueId,
    ) {
        // Must be externally synchronized.
    }
}

impl StorageImage for ImageSubresourceState {
    fn eq(&self, view: &ImageView) -> bool {
        self.subresource.image == view.info().image
    }

    fn get_view(&self, device: &Device) -> Result<ImageView, OutOfMemory> {
        assert!(self
            .subresource
            .image
            .info()
            .usage
            .contains(ImageUsage::STORAGE));
        device.create_image_view(ImageViewInfo::new(self.subresource.image.clone()))
    }

    fn sync<'a>(
        &'a mut self,
        encoder: &mut Encoder<'a>,
        stages: PipelineStageFlags,
        queue: QueueId,
    ) {
        self.access(
            AccessFlags::SHADER_READ | AccessFlags::SHADER_WRITE,
            stages,
            Layout::General,
            queue,
            encoder,
        );
    }
}

impl StorageImage for ImageView {
    fn eq(&self, view: &ImageView) -> bool {
        *self == *view
    }

    fn get_view(&self, _device: &Device) -> Result<ImageView, OutOfMemory> {
        assert!(self.info().image.info().usage.contains(ImageUsage::STORAGE));
        Ok(self.clone())
    }

    fn sync<'a>(
        &'a mut self,
        _encoder: &mut Encoder<'a>,
        _stages: PipelineStageFlags,
        _queue: QueueId,
    ) {
        // Must be externally synchronized.
    }
}

impl StorageImage for ImageViewState {
    fn eq(&self, view: &ImageView) -> bool {
        self.view == *view
    }

    fn get_view(&self, _device: &Device) -> Result<ImageView, OutOfMemory> {
        assert!(self
            .view
            .info()
            .image
            .info()
            .usage
            .contains(ImageUsage::STORAGE));
        Ok(self.view.clone())
    }

    fn sync<'a>(
        &'a mut self,
        encoder: &mut Encoder<'a>,
        stages: PipelineStageFlags,
        queue: QueueId,
    ) {
        self.access(
            AccessFlags::SHADER_READ | AccessFlags::SHADER_WRITE,
            stages,
            Layout::General,
            queue,
            encoder,
        );
    }
}
