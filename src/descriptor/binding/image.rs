use crate::{
    descriptor::{
        DescriptorBinding, DescriptorBindingFlags, DescriptorKind, DynamicLayout, ImageDescriptor,
        Sampled, Storage, ValidLayout,
    },
    image::{Image, Layout, StaticLayout},
    view::{ImageView, ImageViewInfo},
    Device, ImageUsage, OutOfMemory,
};

trait ImageDescriptorKind: DescriptorKind<Descriptor = (ImageView, Layout)> {
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
    fn is_compatible(&self, descriptor: &(ImageView, Layout)) -> bool {
        descriptor.1 == L::LAYOUT && *self == descriptor.0.info().image
    }

    #[inline]
    fn get_descriptor(&self, device: &Device) -> Result<(ImageView, Layout), OutOfMemory> {
        assert!(self.info().usage.contains(<ImageDescriptor<S, L>>::USAGE));

        let view = device.create_image_view(ImageViewInfo::new(self.clone()))?;
        Ok((view, L::LAYOUT))
    }
}

impl<S, L> DescriptorBinding<ImageDescriptor<S, L>> for ImageView
where
    ImageDescriptor<S, L>: ImageDescriptorKind,
    L: StaticLayout,
{
    const FLAGS: DescriptorBindingFlags = DescriptorBindingFlags::empty();

    #[inline]
    fn is_compatible(&self, descriptor: &(ImageView, Layout)) -> bool {
        descriptor.1 == L::LAYOUT && *self == descriptor.0
    }

    #[inline]
    fn get_descriptor(&self, _device: &Device) -> Result<(ImageView, Layout), OutOfMemory> {
        assert!(self
            .info()
            .image
            .info()
            .usage
            .contains(<ImageDescriptor<S, L>>::USAGE));

        Ok((self.clone(), L::LAYOUT))
    }
}

impl<S> DescriptorBinding<ImageDescriptor<S, DynamicLayout>> for (Image, Layout)
where
    ImageDescriptor<S, DynamicLayout>: ImageDescriptorKind,
{
    const FLAGS: DescriptorBindingFlags = DescriptorBindingFlags::empty();

    #[inline]
    fn is_compatible(&self, descriptor: &(ImageView, Layout)) -> bool {
        self.1 == descriptor.1 && self.0 == descriptor.0.info().image
    }

    #[inline]
    fn get_descriptor(&self, device: &Device) -> Result<(ImageView, Layout), OutOfMemory> {
        assert!(self
            .0
            .info()
            .usage
            .contains(<ImageDescriptor<S, DynamicLayout>>::USAGE));

        let view = device.create_image_view(ImageViewInfo::new(self.0.clone()))?;
        Ok((view, self.1))
    }
}

impl<S> DescriptorBinding<ImageDescriptor<S, DynamicLayout>> for (ImageView, Layout)
where
    ImageDescriptor<S, DynamicLayout>: ImageDescriptorKind,
{
    const FLAGS: DescriptorBindingFlags = DescriptorBindingFlags::empty();

    #[inline]
    fn is_compatible(&self, descriptor: &(ImageView, Layout)) -> bool {
        *self == *descriptor
    }

    #[inline]
    fn get_descriptor(&self, _device: &Device) -> Result<(ImageView, Layout), OutOfMemory> {
        assert!(self
            .0
            .info()
            .image
            .info()
            .usage
            .contains(<ImageDescriptor<S, DynamicLayout>>::USAGE));

        Ok(self.clone())
    }
}
