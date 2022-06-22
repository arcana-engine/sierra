pub use crate::backend::ImageView;
use crate::{
    access::AccessFlags,
    backend::Device,
    encode::Encoder,
    image::{Image, ImageExtent, ImageMemoryBarrier, Layout, SubresourceRange},
    queue::{Ownership, QueueId},
    sealed::Sealed,
    stage::PipelineStageFlags,
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

/// Defines remapping of a color component.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum Swizzle {
    /// Maps to the same component
    Identity,

    /// Constant zero
    Zero,

    /// Constant one
    One,

    /// Maps to R component
    R,

    /// Maps to G component
    G,

    /// Maps to B component
    B,

    /// Maps to A component
    A,
}

impl Default for Swizzle {
    fn default() -> Self {
        Self::Identity
    }
}

/// Defines remapping of color components.
#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq)]
pub struct ComponentMapping {
    /// Mapping of R component
    pub r: Swizzle,

    /// Mapping of G component
    pub g: Swizzle,

    /// Mapping of B component
    pub b: Swizzle,

    /// Mapping of A component
    pub a: Swizzle,
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

    /// Defines remapping of color components.
    pub mapping: ComponentMapping,
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
            mapping: ComponentMapping::default(),
        }
    }

    pub fn is_whole_image(&self, image: &Image) -> bool {
        let info = image.info();
        if self.image != *image {
            return false;
        }

        if self.range
            != SubresourceRange::new(
                self.image.info().format.aspect_flags(),
                0..info.levels,
                0..info.layers,
            )
        {
            return false;
        }

        if self.mapping != ComponentMapping::default() {
            return false;
        }

        match (info.extent, self.view_kind) {
            (ImageExtent::D1 { .. }, ImageViewKind::D1) => {}
            (ImageExtent::D2 { .. }, ImageViewKind::D2) => {}
            (ImageExtent::D3 { .. }, ImageViewKind::D3) => {}
            _ => return false,
        }

        true
    }
}

#[doc(hidden)]
pub trait MakeImageView {
    fn make_view(&self, device: &Device) -> Result<ImageView, OutOfMemory>;
}

impl MakeImageView for ImageView {
    fn make_view(&self, _device: &Device) -> Result<ImageView, OutOfMemory> {
        Ok(self.clone())
    }
}

impl MakeImageView for Image {
    fn make_view(&self, device: &Device) -> Result<ImageView, OutOfMemory> {
        let view = device.create_image_view(ImageViewInfo::new(self.clone()))?;
        Ok(view)
    }
}

/// Image region with access mask,
/// specifying how it may be accessed "before".
///
/// Note that "before" is loosely defined,
/// as whatever previous owners do.
/// Which should be translated to "earlier GPU operations"
/// but this crate doesn't attempt to enforce that.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ImageViewState {
    pub view: ImageView,
    pub access: AccessFlags,
    pub stages: PipelineStageFlags,
    pub layout: Option<Layout>,
    pub family: Ownership,
}

impl ImageViewState {
    ///
    pub fn access<'a>(
        &'a mut self,
        access: AccessFlags,
        stages: PipelineStageFlags,
        layout: Layout,
        queue: QueueId,
        encoder: &mut Encoder<'a>,
    ) -> &'a ImageView {
        match self.family {
            Ownership::NotOwned => encoder.image_barriers(
                self.stages,
                stages,
                encoder.scope().to_scope([ImageMemoryBarrier {
                    image: &self.view.info().image,
                    old_access: self.access,
                    new_access: access,
                    old_layout: self.layout,
                    new_layout: layout,
                    family_transfer: None,
                    range: self.view.info().range,
                }]),
            ),
            Ownership::Owned { family } => {
                assert_eq!(family, queue.family, "Wrong queue family owns the buffer");

                encoder.image_barriers(
                    self.stages,
                    stages,
                    encoder.scope().to_scope([ImageMemoryBarrier {
                        image: &self.view.info().image,
                        old_access: self.access,
                        new_access: access,
                        old_layout: self.layout,
                        new_layout: layout,
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
                    self.stages,
                    stages,
                    encoder.scope().to_scope([ImageMemoryBarrier {
                        image: &self.view.info().image,
                        old_access: self.access,
                        new_access: access,
                        old_layout: self.layout,
                        new_layout: layout,
                        family_transfer: Some((from, to)),
                        range: self.view.info().range,
                    }]),
                )
            }
        }
        self.family = Ownership::Owned {
            family: queue.family,
        };
        self.stages = stages;
        self.access = access;
        self.layout = Some(layout);
        &self.view
    }

    ///
    pub fn overwrite<'a>(
        &'a mut self,
        access: AccessFlags,
        stages: PipelineStageFlags,
        layout: Layout,
        queue: QueueId,
        encoder: &mut Encoder<'a>,
    ) -> &'a ImageView {
        encoder.image_barriers(
            self.stages,
            stages,
            encoder.scope().to_scope([ImageMemoryBarrier {
                image: &self.view.info().image,
                old_access: AccessFlags::empty(),
                new_access: access,
                old_layout: None,
                new_layout: layout,
                family_transfer: None,
                range: self.view.info().range,
            }]),
        );
        self.family = Ownership::Owned {
            family: queue.family,
        };
        self.stages = stages;
        self.access = access;
        self.layout = Some(layout);
        &self.view
    }
}

impl Sealed for (ImageView, Layout) {}
