use {
    super::{
        DescriptorBindingFlags, ImageDescriptor, TypedDescriptorBinding,
        TypedImageDescriptorBinding,
    },
    crate::{
        image::{Image, Layout},
        view::{ImageView, ImageViewInfo},
        Device, OutOfMemory,
    },
};

impl TypedImageDescriptorBinding for Image {
    const COUNT: u32 = 1;
    const FLAGS: DescriptorBindingFlags = DescriptorBindingFlags::empty();
    type Descriptors = [ImageDescriptor<ImageView>; 1];

    #[inline]
    fn eq(&self, descriptors: &[ImageDescriptor<ImageView>; 1], layout: Layout) -> bool {
        descriptors[0].layout == layout && *self == descriptors[0].image.info().image
    }

    #[inline]
    fn get_descriptors(
        &self,
        device: &Device,
        layout: Layout,
    ) -> Result<[ImageDescriptor<ImageView>; 1], OutOfMemory> {
        let view = device.create_image_view(ImageViewInfo::new(self.clone()))?;
        Ok([ImageDescriptor {
            image: view,
            layout,
        }])
    }
}

impl TypedImageDescriptorBinding for ImageView {
    const COUNT: u32 = 1;
    const FLAGS: DescriptorBindingFlags = DescriptorBindingFlags::empty();
    type Descriptors = [ImageDescriptor<ImageView>; 1];

    #[inline]
    fn eq(&self, descriptors: &[ImageDescriptor<ImageView>; 1], layout: Layout) -> bool {
        descriptors[0].layout == layout && *self == descriptors[0].image
    }

    #[inline]
    fn get_descriptors(
        &self,
        _device: &Device,
        layout: Layout,
    ) -> Result<[ImageDescriptor<ImageView>; 1], OutOfMemory> {
        Ok([ImageDescriptor {
            image: self.clone(),
            layout,
        }])
    }
}

impl TypedDescriptorBinding for ImageDescriptor<Image> {
    const COUNT: u32 = 1;
    const FLAGS: DescriptorBindingFlags = DescriptorBindingFlags::empty();
    type Descriptors = [ImageDescriptor<ImageView>; 1];

    #[inline]
    fn eq(&self, descriptors: &[ImageDescriptor<ImageView>; 1]) -> bool {
        self.layout == descriptors[0].layout && self.image == descriptors[0].image.info().image
    }

    #[inline]
    fn get_descriptors(
        &self,
        device: &Device,
    ) -> Result<[ImageDescriptor<ImageView>; 1], OutOfMemory> {
        let view = device.create_image_view(ImageViewInfo::new(self.image.clone()))?;
        Ok([ImageDescriptor {
            image: view,
            layout: self.layout,
        }])
    }
}

impl TypedDescriptorBinding for ImageDescriptor<ImageView> {
    const COUNT: u32 = 1;
    const FLAGS: DescriptorBindingFlags = DescriptorBindingFlags::empty();
    type Descriptors = [ImageDescriptor<ImageView>; 1];

    #[inline]
    fn eq(&self, descriptors: &[ImageDescriptor<ImageView>; 1]) -> bool {
        *self == descriptors[0]
    }

    #[inline]
    fn get_descriptors(
        &self,
        _device: &Device,
    ) -> Result<[ImageDescriptor<ImageView>; 1], OutOfMemory> {
        Ok([self.clone()])
    }
}

impl<const N: usize> TypedImageDescriptorBinding for [Image; N] {
    const COUNT: u32 = N as u32;
    const FLAGS: DescriptorBindingFlags = DescriptorBindingFlags::empty();
    type Descriptors = [ImageDescriptor<ImageView>; N];

    #[inline]
    fn eq(&self, descriptors: &[ImageDescriptor<ImageView>; N], laoyut: Layout) -> bool {
        self.iter().zip(descriptors).all(|(me, descriptor)| {
            descriptor.layout == laoyut && *me == descriptor.image.info().image
        })
    }

    #[inline]
    fn get_descriptors(
        &self,
        device: &Device,
        laoyut: Layout,
    ) -> Result<[ImageDescriptor<ImageView>; N], OutOfMemory> {
        let mut result = arrayvec::ArrayVec::new();

        for me in self {
            let view = device.create_image_view(ImageViewInfo::new(me.clone()))?;
            unsafe {
                result.push_unchecked(ImageDescriptor {
                    image: view,
                    layout: laoyut,
                });
            }
        }

        Ok(result.into_inner().unwrap())
    }
}

impl<const N: usize> TypedImageDescriptorBinding for [ImageView; N] {
    const COUNT: u32 = N as u32;
    const FLAGS: DescriptorBindingFlags = DescriptorBindingFlags::empty();
    type Descriptors = [ImageDescriptor<ImageView>; N];

    #[inline]
    fn eq(&self, descriptors: &[ImageDescriptor<ImageView>; N], laoyut: Layout) -> bool {
        self.iter()
            .zip(descriptors)
            .all(|(me, descriptor)| descriptor.layout == laoyut && *me == descriptor.image)
    }

    #[inline]
    fn get_descriptors(
        &self,
        _device: &Device,
        laoyut: Layout,
    ) -> Result<[ImageDescriptor<ImageView>; N], OutOfMemory> {
        let mut result = arrayvec::ArrayVec::new();

        for me in self {
            unsafe {
                result.push_unchecked(ImageDescriptor {
                    image: me.clone(),
                    layout: laoyut,
                });
            }
        }

        Ok(result.into_inner().unwrap())
    }
}

impl<const N: usize> TypedDescriptorBinding for [ImageDescriptor<Image>; N] {
    const COUNT: u32 = N as u32;
    const FLAGS: DescriptorBindingFlags = DescriptorBindingFlags::empty();
    type Descriptors = [ImageDescriptor<ImageView>; N];

    #[inline]
    fn eq(&self, descriptors: &[ImageDescriptor<ImageView>; N]) -> bool {
        self.iter().zip(descriptors).all(|(me, descriptor)| {
            me.layout == descriptor.layout && me.image == descriptor.image.info().image
        })
    }

    #[inline]
    fn get_descriptors(
        &self,
        device: &Device,
    ) -> Result<[ImageDescriptor<ImageView>; N], OutOfMemory> {
        let mut result = arrayvec::ArrayVec::new();

        for me in self {
            let view = device.create_image_view(ImageViewInfo::new(me.image.clone()))?;
            unsafe {
                result.push_unchecked(ImageDescriptor {
                    image: view,
                    layout: me.layout,
                });
            }
        }

        Ok(result.into_inner().unwrap())
    }
}

impl<const N: usize> TypedDescriptorBinding for [ImageDescriptor<ImageView>; N] {
    const COUNT: u32 = N as u32;
    const FLAGS: DescriptorBindingFlags = DescriptorBindingFlags::empty();
    type Descriptors = [ImageDescriptor<ImageView>; N];

    #[inline]
    fn eq(&self, descriptors: &[ImageDescriptor<ImageView>; N]) -> bool {
        *self == *descriptors
    }

    #[inline]
    fn get_descriptors(
        &self,
        _device: &Device,
    ) -> Result<[ImageDescriptor<ImageView>; N], OutOfMemory> {
        Ok(self.clone())
    }
}

impl<const N: usize> TypedImageDescriptorBinding for arrayvec::ArrayVec<Image, N> {
    const COUNT: u32 = N as u32;
    const FLAGS: DescriptorBindingFlags = DescriptorBindingFlags::PARTIALLY_BOUND;
    type Descriptors = arrayvec::ArrayVec<ImageDescriptor<ImageView>, N>;

    #[inline]
    fn eq(
        &self,
        descriptors: &arrayvec::ArrayVec<ImageDescriptor<ImageView>, N>,
        layout: Layout,
    ) -> bool {
        self.len() == descriptors.len()
            && self.iter().zip(descriptors).all(|(me, descriptor)| {
                descriptor.layout == layout && *me == descriptor.image.info().image
            })
    }

    #[inline]
    fn get_descriptors(
        &self,
        device: &Device,
        layout: Layout,
    ) -> Result<arrayvec::ArrayVec<ImageDescriptor<ImageView>, N>, OutOfMemory> {
        let mut result = arrayvec::ArrayVec::new();

        for me in self {
            let view = device.create_image_view(ImageViewInfo::new(me.clone()))?;
            unsafe {
                result.push_unchecked(ImageDescriptor {
                    image: view,
                    layout,
                });
            }
        }

        Ok(result)
    }
}
impl<const N: usize> TypedImageDescriptorBinding for arrayvec::ArrayVec<ImageView, N> {
    const COUNT: u32 = N as u32;
    const FLAGS: DescriptorBindingFlags = DescriptorBindingFlags::PARTIALLY_BOUND;
    type Descriptors = arrayvec::ArrayVec<ImageDescriptor<ImageView>, N>;

    #[inline]
    fn eq(
        &self,
        descriptors: &arrayvec::ArrayVec<ImageDescriptor<ImageView>, N>,
        layout: Layout,
    ) -> bool {
        self.len() == descriptors.len()
            && self
                .iter()
                .zip(descriptors)
                .all(|(me, descriptor)| descriptor.layout == layout && *me == descriptor.image)
    }

    #[inline]
    fn get_descriptors(
        &self,
        _device: &Device,
        layout: Layout,
    ) -> Result<arrayvec::ArrayVec<ImageDescriptor<ImageView>, N>, OutOfMemory> {
        let mut result = arrayvec::ArrayVec::new();

        for me in self {
            unsafe {
                result.push_unchecked(ImageDescriptor {
                    image: me.clone(),
                    layout,
                });
            }
        }

        Ok(result)
    }
}

impl<const N: usize> TypedDescriptorBinding for arrayvec::ArrayVec<ImageDescriptor<Image>, N> {
    const COUNT: u32 = N as u32;
    const FLAGS: DescriptorBindingFlags = DescriptorBindingFlags::PARTIALLY_BOUND;
    type Descriptors = arrayvec::ArrayVec<ImageDescriptor<ImageView>, N>;

    #[inline]
    fn eq(&self, descriptors: &arrayvec::ArrayVec<ImageDescriptor<ImageView>, N>) -> bool {
        self.iter().zip(descriptors).all(|(me, descriptor)| {
            me.layout == descriptor.layout && me.image == descriptor.image.info().image
        })
    }

    #[inline]
    fn get_descriptors(
        &self,
        device: &Device,
    ) -> Result<arrayvec::ArrayVec<ImageDescriptor<ImageView>, N>, OutOfMemory> {
        let mut result = arrayvec::ArrayVec::new();

        for me in self {
            let view = device.create_image_view(ImageViewInfo::new(me.image.clone()))?;
            unsafe {
                result.push_unchecked(ImageDescriptor {
                    image: view,
                    layout: me.layout,
                });
            }
        }

        Ok(result)
    }
}

impl<const N: usize> TypedDescriptorBinding for arrayvec::ArrayVec<ImageDescriptor<ImageView>, N> {
    const COUNT: u32 = N as u32;
    const FLAGS: DescriptorBindingFlags = DescriptorBindingFlags::PARTIALLY_BOUND;
    type Descriptors = arrayvec::ArrayVec<ImageDescriptor<ImageView>, N>;

    #[inline]
    fn eq(&self, descriptors: &arrayvec::ArrayVec<ImageDescriptor<ImageView>, N>) -> bool {
        *self == *descriptors
    }

    #[inline]
    fn get_descriptors(
        &self,
        _device: &Device,
    ) -> Result<arrayvec::ArrayVec<ImageDescriptor<ImageView>, N>, OutOfMemory> {
        Ok(self.clone())
    }
}
