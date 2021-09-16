use {
    super::{DescriptorBindingFlags, ImageViewDescriptor, TypedDescriptorBinding},
    crate::{
        image::{Image, RawLayout},
        view::{ImageView, ImageViewInfo},
        Device, OutOfMemory,
    },
};

impl TypedDescriptorBinding for Image {
    const COUNT: u32 = 1;
    const FLAGS: DescriptorBindingFlags = DescriptorBindingFlags::empty();
    type Descriptors = [ImageViewDescriptor; 1];

    fn eq(&self, descriptors: &[ImageViewDescriptor; 1]) -> bool {
        *self == descriptors[0].view.info().image
    }

    fn get_descriptors(&self, device: &Device) -> Result<[ImageViewDescriptor; 1], OutOfMemory> {
        let view = device.create_image_view(ImageViewInfo::new(self.clone()))?;
        Ok([ImageViewDescriptor {
            view,
            layout: RawLayout::ShaderReadOnlyOptimal,
        }])
    }
}

impl TypedDescriptorBinding for ImageView {
    const COUNT: u32 = 1;
    const FLAGS: DescriptorBindingFlags = DescriptorBindingFlags::empty();
    type Descriptors = [ImageViewDescriptor; 1];

    fn eq(&self, descriptors: &[ImageViewDescriptor; 1]) -> bool {
        *self == descriptors[0].view
    }

    fn get_descriptors(&self, _device: &Device) -> Result<[ImageViewDescriptor; 1], OutOfMemory> {
        Ok([ImageViewDescriptor {
            view: self.clone(),
            layout: RawLayout::ShaderReadOnlyOptimal,
        }])
    }
}

impl TypedDescriptorBinding for ImageViewDescriptor {
    const COUNT: u32 = 1;
    const FLAGS: DescriptorBindingFlags = DescriptorBindingFlags::empty();
    type Descriptors = [ImageViewDescriptor; 1];

    fn eq(&self, descriptors: &[ImageViewDescriptor; 1]) -> bool {
        *self == descriptors[0]
    }

    fn get_descriptors(&self, _device: &Device) -> Result<[ImageViewDescriptor; 1], OutOfMemory> {
        Ok([self.clone()])
    }
}

impl<const N: usize> TypedDescriptorBinding for [ImageView; N] {
    const COUNT: u32 = N as u32;
    const FLAGS: DescriptorBindingFlags = DescriptorBindingFlags::empty();
    type Descriptors = [ImageViewDescriptor; N];

    fn eq(&self, descriptors: &[ImageViewDescriptor; N]) -> bool {
        self.iter()
            .zip(descriptors)
            .all(|(me, descriptor)| *me == descriptor.view)
    }

    fn get_descriptors(&self, _device: &Device) -> Result<[ImageViewDescriptor; N], OutOfMemory> {
        let mut result = arrayvec::ArrayVec::new();

        for me in self {
            unsafe {
                result.push_unchecked(ImageViewDescriptor {
                    view: me.clone(),
                    layout: RawLayout::ShaderReadOnlyOptimal,
                });
            }
        }

        Ok(result.into_inner().unwrap())
    }
}

impl<const N: usize> TypedDescriptorBinding for [ImageViewDescriptor; N] {
    const COUNT: u32 = N as u32;
    const FLAGS: DescriptorBindingFlags = DescriptorBindingFlags::empty();
    type Descriptors = [ImageViewDescriptor; N];

    fn eq(&self, descriptors: &[ImageViewDescriptor; N]) -> bool {
        *self == *descriptors
    }

    fn get_descriptors(&self, _device: &Device) -> Result<[ImageViewDescriptor; N], OutOfMemory> {
        Ok(self.clone())
    }
}

impl<const N: usize> TypedDescriptorBinding for arrayvec::ArrayVec<ImageView, N> {
    const COUNT: u32 = N as u32;
    const FLAGS: DescriptorBindingFlags = DescriptorBindingFlags::PARTIALLY_BOUND;
    type Descriptors = arrayvec::ArrayVec<ImageViewDescriptor, N>;

    fn eq(&self, descriptors: &arrayvec::ArrayVec<ImageViewDescriptor, N>) -> bool {
        self.len() == descriptors.len()
            && self
                .iter()
                .zip(descriptors)
                .all(|(me, descriptor)| *me == descriptor.view)
    }

    fn get_descriptors(
        &self,
        _device: &Device,
    ) -> Result<arrayvec::ArrayVec<ImageViewDescriptor, N>, OutOfMemory> {
        let mut result = arrayvec::ArrayVec::new();

        for me in self {
            unsafe {
                result.push_unchecked(ImageViewDescriptor {
                    view: me.clone(),
                    layout: RawLayout::ShaderReadOnlyOptimal,
                });
            }
        }

        Ok(result)
    }
}

impl<const N: usize> TypedDescriptorBinding for arrayvec::ArrayVec<ImageViewDescriptor, N> {
    const COUNT: u32 = N as u32;
    const FLAGS: DescriptorBindingFlags = DescriptorBindingFlags::PARTIALLY_BOUND;
    type Descriptors = arrayvec::ArrayVec<ImageViewDescriptor, N>;

    fn eq(&self, descriptors: &arrayvec::ArrayVec<ImageViewDescriptor, N>) -> bool {
        *self == *descriptors
    }

    fn get_descriptors(
        &self,
        _device: &Device,
    ) -> Result<arrayvec::ArrayVec<ImageViewDescriptor, N>, OutOfMemory> {
        Ok(self.clone())
    }
}
