use crate::{
    descriptor::{
        DescriptorBinding, DescriptorBindingFlags, DescriptorKind, DynamicLayout, ImageDescriptor,
        ImageLayout, Sampled, Storage, ValidLayout,
    },
    image::{Image, StaticLayout},
    view::{ImageView, ImageViewInfo},
    Device, ImageUsage, OutOfMemory,
};

trait ImageDescriptorKind: DescriptorKind<Descriptor = ImageLayout<ImageView>> {
    const USAGE: ImageUsage;
}

impl ImageDescriptorKind for ImageDescriptor<Sampled, DynamicLayout> {
    const USAGE: ImageUsage = ImageUsage::SAMPLED;
}

impl<L> ImageDescriptorKind for ImageDescriptor<Sampled, L>
where
    L: ValidLayout<Sampled>,
{
    const USAGE: ImageUsage = ImageUsage::SAMPLED;
}

impl ImageDescriptorKind for ImageDescriptor<Storage, DynamicLayout> {
    const USAGE: ImageUsage = ImageUsage::STORAGE;
}

impl<L> ImageDescriptorKind for ImageDescriptor<Storage, L>
where
    L: ValidLayout<Storage>,
{
    const USAGE: ImageUsage = ImageUsage::STORAGE;
}

impl<S, L> DescriptorBinding<ImageDescriptor<S, L>> for Image
where
    ImageDescriptor<S, L>: ImageDescriptorKind,
    L: StaticLayout,
{
    const FLAGS: DescriptorBindingFlags = DescriptorBindingFlags::empty();

    #[inline]
    fn is_compatible(&self, descriptor: &ImageLayout<ImageView>) -> bool {
        descriptor.layout == L::LAYOUT && *self == descriptor.image.info().image
    }

    #[inline]
    fn get_descriptor(&self, device: &Device) -> Result<ImageLayout<ImageView>, OutOfMemory> {
        assert!(self.info().usage.contains(<ImageDescriptor<S, L>>::USAGE));

        let view = device.create_image_view(ImageViewInfo::new(self.clone()))?;
        Ok(ImageLayout {
            image: view,
            layout: L::LAYOUT,
        })
    }
}

impl<S, L> DescriptorBinding<ImageDescriptor<S, L>> for ImageView
where
    ImageDescriptor<S, L>: ImageDescriptorKind,
    L: StaticLayout,
{
    const FLAGS: DescriptorBindingFlags = DescriptorBindingFlags::empty();

    #[inline]
    fn is_compatible(&self, descriptor: &ImageLayout<ImageView>) -> bool {
        descriptor.layout == L::LAYOUT && *self == descriptor.image
    }

    #[inline]
    fn get_descriptor(&self, _device: &Device) -> Result<ImageLayout<ImageView>, OutOfMemory> {
        assert!(self
            .info()
            .image
            .info()
            .usage
            .contains(<ImageDescriptor<S, L>>::USAGE));

        Ok(ImageLayout {
            image: self.clone(),
            layout: L::LAYOUT,
        })
    }
}

impl<S> DescriptorBinding<ImageDescriptor<S, DynamicLayout>> for ImageLayout<Image>
where
    ImageDescriptor<S, DynamicLayout>: ImageDescriptorKind,
{
    const FLAGS: DescriptorBindingFlags = DescriptorBindingFlags::empty();

    #[inline]
    fn is_compatible(&self, descriptor: &ImageLayout<ImageView>) -> bool {
        self.layout == descriptor.layout && self.image == descriptor.image.info().image
    }

    #[inline]
    fn get_descriptor(&self, device: &Device) -> Result<ImageLayout<ImageView>, OutOfMemory> {
        assert!(self
            .image
            .info()
            .usage
            .contains(<ImageDescriptor<S, DynamicLayout>>::USAGE));

        let view = device.create_image_view(ImageViewInfo::new(self.image.clone()))?;
        Ok(ImageLayout {
            image: view,
            layout: self.layout,
        })
    }
}

impl<S> DescriptorBinding<ImageDescriptor<S, DynamicLayout>> for ImageLayout<ImageView>
where
    ImageDescriptor<S, DynamicLayout>: ImageDescriptorKind,
{
    const FLAGS: DescriptorBindingFlags = DescriptorBindingFlags::empty();

    #[inline]
    fn is_compatible(&self, descriptor: &ImageLayout<ImageView>) -> bool {
        *self == *descriptor
    }

    #[inline]
    fn get_descriptor(&self, _device: &Device) -> Result<ImageLayout<ImageView>, OutOfMemory> {
        assert!(self
            .image
            .info()
            .image
            .info()
            .usage
            .contains(<ImageDescriptor<S, DynamicLayout>>::USAGE));

        Ok(self.clone())
    }
}
