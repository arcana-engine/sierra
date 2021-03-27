pub use crate::backend::ImageView;
use crate::{
    backend::Device,
    image::{Image, ImageExtent, ImageSubresourceRange},
    OutOfMemory,
};

/// Kind of image view.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub enum ImageViewKind {
    /// One dimensional image view
    D1,

    /// Two dimensional imave view.
    D2,

    /// Three dimensional image view.
    D3,

    /// Cube view.
    /// 6 image layers are treated as sides of a cube.
    /// Cube views can be sampled by direction vector
    /// resulting in sample at intersection of cube and
    /// a ray with origin in center of cube and direction of that vector
    Cube,
}

/// Information required to create an image view.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ImageViewInfo {
    /// Kind of the view.
    pub view_kind: ImageViewKind,

    /// Subresorce of the image view is bound to.
    pub subresource: ImageSubresourceRange,

    /// An image view is bound to.
    pub image: Image,
}

impl ImageViewInfo {
    pub fn new(image: Image) -> Self {
        let info = image.info();

        ImageViewInfo {
            view_kind: match info.extent {
                ImageExtent::D1 { .. } => ImageViewKind::D1,
                ImageExtent::D2 { .. } => ImageViewKind::D2,
                ImageExtent::D3 { .. } => ImageViewKind::D3,
            },
            subresource: ImageSubresourceRange::new(
                info.format.aspect_flags(),
                0..info.levels,
                0..info.layers,
            ),
            image,
        }
    }
}

#[doc(hidden)]
pub trait MakeImageView {
    fn make_view<'a>(&'a self, device: &Device) -> Result<ImageView, OutOfMemory>;
}

impl MakeImageView for ImageView {
    fn make_view<'a>(&'a self, _device: &Device) -> Result<ImageView, OutOfMemory> {
        Ok(self.clone())
    }
}

impl MakeImageView for Image {
    fn make_view<'a>(&'a self, device: &Device) -> Result<ImageView, OutOfMemory> {
        let view = device.create_image_view(ImageViewInfo::new(self.clone()))?;
        Ok(view)
    }
}
