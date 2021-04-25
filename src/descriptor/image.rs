use {
    super::{
        AsDescriptors, Descriptors, ImageViewDescriptor, SampledImageDescriptor,
        StorageImageDescriptor, TypedDescriptors,
    },
    crate::{
        image::{Image, Layout},
        view::{ImageView, ImageViewInfo},
        Device, OutOfMemory,
    },
};

impl<const N: usize> TypedDescriptors<SampledImageDescriptor> for [ImageViewDescriptor; N] {
    fn descriptors(&self) -> Descriptors<'_> {
        Descriptors::SampledImage(self)
    }
}

/// Interface for all types that can be used as `UniformBuffer` or `StorageBuffer` descriptor.
impl<const N: usize> TypedDescriptors<StorageImageDescriptor> for [ImageViewDescriptor; N] {
    fn descriptors(&self) -> Descriptors<'_> {
        Descriptors::StorageImage(self)
    }
}

impl AsDescriptors for Image {
    const COUNT: u32 = 1;
    type Descriptors = [ImageViewDescriptor; 1];

    fn eq(&self, descriptors: &[ImageViewDescriptor; 1]) -> bool {
        *self == descriptors[0].view.info().image
    }

    fn get_descriptors(&self, device: &Device) -> Result<[ImageViewDescriptor; 1], OutOfMemory> {
        let view = device.create_image_view(ImageViewInfo::new(self.clone()))?;
        Ok([ImageViewDescriptor {
            view,
            layout: Layout::ShaderReadOnlyOptimal,
        }])
    }
}

impl AsDescriptors for ImageView {
    const COUNT: u32 = 1;
    type Descriptors = [ImageViewDescriptor; 1];

    fn eq(&self, descriptors: &[ImageViewDescriptor; 1]) -> bool {
        *self == descriptors[0].view
    }

    fn get_descriptors(&self, _device: &Device) -> Result<[ImageViewDescriptor; 1], OutOfMemory> {
        Ok([ImageViewDescriptor {
            view: self.clone(),
            layout: Layout::ShaderReadOnlyOptimal,
        }])
    }
}

impl<const N: usize> AsDescriptors for [ImageView; N] {
    const COUNT: u32 = N as u32;
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
                    layout: Layout::ShaderReadOnlyOptimal,
                });
            }
        }

        Ok(result.into_inner().unwrap())
    }
}
