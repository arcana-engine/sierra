pub use crate::backend::ImageView;
use crate::{
    access::Access,
    backend::Device,
    encode::Encoder,
    image::{Image, ImageExtent, ImageMemoryBarrier, Layout, SubresourceRange},
    queue::{Ownership, QueueId},
    OutOfMemory,
};

/// Kind of image view.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub enum ImageViewKind {
    /// One dimensional image view
    D1,

    /// Two dimensional image view.
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

    /// Subresource of the image view is bound to.
    pub range: SubresourceRange,

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
            range: SubresourceRange::new(
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

/// Image region with access mask,
/// specifying how it was accessed "before".
///
/// Note that "before" is loosely defined
/// as whatever previous owners did with it,
/// i.e. "earlier GPU operations which used this resource,"
/// but this crate doesn't attempt to enforce that.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ImageViewState {
    pub view: ImageView,
    pub prev_access: Access,
    pub old_layout: Option<Layout>,
    pub family: Ownership,
}

impl ImageViewState {
    ///
    pub fn access<'a>(
        &'a mut self,
        next_access: Access,
        new_layout: Layout,
        queue: QueueId,
        encoder: &mut Encoder<'a>,
    ) -> &'a ImageView {
        match self.family {
            Ownership::NotOwned => encoder.image_barriers(
                encoder.scope().to_scope([ImageMemoryBarrier {
                    image: &self.view.info().image,
                    prev_access: self.prev_access,
                    next_access,
                    old_layout: self.old_layout,
                    new_layout,
                    family_transfer: None,
                    range: self.view.info().range,
                }]),
            ),
            Ownership::Owned { family } => {
                assert_eq!(family, queue.family, "Wrong queue family owns the buffer");

                encoder.image_barriers(
                    encoder.scope().to_scope([ImageMemoryBarrier {
                        image: &self.view.info().image,
                        prev_access: self.prev_access,
                        next_access,
                        old_layout: self.old_layout,
                        new_layout,
                        family_transfer: None,
                        range: self.view.info().range,
                    }]),
                )
            }
            Ownership::Transition { from, to } => {
                assert_eq!(
                    to, queue.family,
                    "Image is being transitioned to wrong queue family"
                );

                encoder.image_barriers(
                    encoder.scope().to_scope([ImageMemoryBarrier {
                        image: &self.view.info().image,
                        prev_access: self.prev_access,
                        next_access,
                        old_layout: self.old_layout,
                        new_layout,
                        family_transfer: Some((from, to)),
                        range: self.view.info().range,
                    }]),
                )
            }
        }
        self.family = Ownership::Owned {
            family: queue.family,
        };
        self.prev_access = next_access;
        self.old_layout = Some(new_layout);
        &self.view
    }

    ///
    pub fn overwrite<'a>(
        &'a mut self,
        next_access: Access,
        new_layout: Layout,
        queue: QueueId,
        encoder: &mut Encoder<'a>,
    ) -> &'a ImageView {
        encoder.image_barriers(
            encoder.scope().to_scope([ImageMemoryBarrier {
                image: &self.view.info().image,
                prev_access: Access::None,
                next_access,
                old_layout: None,
                new_layout,
                family_transfer: None,
                range: self.view.info().range,
            }]),
        );
        self.family = Ownership::Owned {
            family: queue.family,
        };
        self.prev_access = next_access;
        self.old_layout = Some(new_layout);
        &self.view
    }
}
