use crate::{
    out_of_host_memory, AccelerationStructureBuildFlags, AccelerationStructureLevel, AccessFlags,
    AspectFlags, BlendFactor, BlendOp, BorderColor, BufferCopy, BufferImageCopy, BufferUsage,
    CompareOp, ComponentMask, CompositeAlphaFlags, Culling, DescriptorBindingFlags,
    DescriptorSetLayoutFlags, DescriptorType, DeviceAddress, Extent2d, Extent3d, Filter, Format,
    FrontFace, GeometryFlags, ImageBlit, ImageCopy, ImageExtent, ImageUsage, ImageViewKind,
    IndexType, Layout, LoadOp, LogicOp, MemoryUsage, MipmapMode, Offset2d, Offset3d, OutOfMemory,
    PipelineStageFlags, PolygonMode, PresentMode, PrimitiveTopology, QueueCapabilityFlags, Rect2d,
    SamplerAddressMode, Samples, ShaderStage, ShaderStageFlags, StencilOp, StoreOp, Subresource,
    SubresourceLayers, SubresourceRange, SurfaceTransformFlags, VertexInputRate, Viewport,
};
use erupt::{
    extensions::{
        khr_acceleration_structure as vkacc,
        khr_surface::{CompositeAlphaFlagsKHR, PresentModeKHR, SurfaceTransformFlagsKHR},
    },
    vk1_0, vk1_2,
};
use std::num::NonZeroU64;

pub(crate) trait ToErupt<T> {
    fn to_erupt(self) -> T;
}

pub(crate) trait FromErupt<T> {
    fn from_erupt(value: T) -> Self;
}

pub(crate) fn from_erupt<T, U: FromErupt<T>>(value: T) -> U {
    U::from_erupt(value)
}

impl FromErupt<vk1_0::Format> for Option<Format> {
    fn from_erupt(format: vk1_0::Format) -> Self {
        match format {
            vk1_0::Format::R8_UNORM => Some(Format::R8Unorm),
            vk1_0::Format::R8_SNORM => Some(Format::R8Snorm),
            vk1_0::Format::R8_USCALED => Some(Format::R8Uscaled),
            vk1_0::Format::R8_SSCALED => Some(Format::R8Sscaled),
            vk1_0::Format::R8_UINT => Some(Format::R8Uint),
            vk1_0::Format::R8_SINT => Some(Format::R8Sint),
            vk1_0::Format::R8_SRGB => Some(Format::R8Srgb),
            vk1_0::Format::R8G8_UNORM => Some(Format::RG8Unorm),
            vk1_0::Format::R8G8_SNORM => Some(Format::RG8Snorm),
            vk1_0::Format::R8G8_USCALED => Some(Format::RG8Uscaled),
            vk1_0::Format::R8G8_SSCALED => Some(Format::RG8Sscaled),
            vk1_0::Format::R8G8_UINT => Some(Format::RG8Uint),
            vk1_0::Format::R8G8_SINT => Some(Format::RG8Sint),
            vk1_0::Format::R8G8_SRGB => Some(Format::RG8Srgb),
            vk1_0::Format::R8G8B8_UNORM => Some(Format::RGB8Unorm),
            vk1_0::Format::R8G8B8_SNORM => Some(Format::RGB8Snorm),
            vk1_0::Format::R8G8B8_USCALED => Some(Format::RGB8Uscaled),
            vk1_0::Format::R8G8B8_SSCALED => Some(Format::RGB8Sscaled),
            vk1_0::Format::R8G8B8_UINT => Some(Format::RGB8Uint),
            vk1_0::Format::R8G8B8_SINT => Some(Format::RGB8Sint),
            vk1_0::Format::R8G8B8_SRGB => Some(Format::RGB8Srgb),
            vk1_0::Format::B8G8R8_UNORM => Some(Format::BGR8Unorm),
            vk1_0::Format::B8G8R8_SNORM => Some(Format::BGR8Snorm),
            vk1_0::Format::B8G8R8_USCALED => Some(Format::BGR8Uscaled),
            vk1_0::Format::B8G8R8_SSCALED => Some(Format::BGR8Sscaled),
            vk1_0::Format::B8G8R8_UINT => Some(Format::BGR8Uint),
            vk1_0::Format::B8G8R8_SINT => Some(Format::BGR8Sint),
            vk1_0::Format::B8G8R8_SRGB => Some(Format::BGR8Srgb),
            vk1_0::Format::R8G8B8A8_UNORM => Some(Format::RGBA8Unorm),
            vk1_0::Format::R8G8B8A8_SNORM => Some(Format::RGBA8Snorm),
            vk1_0::Format::R8G8B8A8_USCALED => Some(Format::RGBA8Uscaled),
            vk1_0::Format::R8G8B8A8_SSCALED => Some(Format::RGBA8Sscaled),
            vk1_0::Format::R8G8B8A8_UINT => Some(Format::RGBA8Uint),
            vk1_0::Format::R8G8B8A8_SINT => Some(Format::RGBA8Sint),
            vk1_0::Format::R8G8B8A8_SRGB => Some(Format::RGBA8Srgb),
            vk1_0::Format::B8G8R8A8_UNORM => Some(Format::BGRA8Unorm),
            vk1_0::Format::B8G8R8A8_SNORM => Some(Format::BGRA8Snorm),
            vk1_0::Format::B8G8R8A8_USCALED => Some(Format::BGRA8Uscaled),
            vk1_0::Format::B8G8R8A8_SSCALED => Some(Format::BGRA8Sscaled),
            vk1_0::Format::B8G8R8A8_UINT => Some(Format::BGRA8Uint),
            vk1_0::Format::B8G8R8A8_SINT => Some(Format::BGRA8Sint),
            vk1_0::Format::B8G8R8A8_SRGB => Some(Format::BGRA8Srgb),
            vk1_0::Format::R16_UNORM => Some(Format::R16Unorm),
            vk1_0::Format::R16_SNORM => Some(Format::R16Snorm),
            vk1_0::Format::R16_USCALED => Some(Format::R16Uscaled),
            vk1_0::Format::R16_SSCALED => Some(Format::R16Sscaled),
            vk1_0::Format::R16_UINT => Some(Format::R16Uint),
            vk1_0::Format::R16_SINT => Some(Format::R16Sint),
            vk1_0::Format::R16_SFLOAT => Some(Format::R16Sfloat),
            vk1_0::Format::R16G16_UNORM => Some(Format::RG16Unorm),
            vk1_0::Format::R16G16_SNORM => Some(Format::RG16Snorm),
            vk1_0::Format::R16G16_USCALED => Some(Format::RG16Uscaled),
            vk1_0::Format::R16G16_SSCALED => Some(Format::RG16Sscaled),
            vk1_0::Format::R16G16_UINT => Some(Format::RG16Uint),
            vk1_0::Format::R16G16_SINT => Some(Format::RG16Sint),
            vk1_0::Format::R16G16_SFLOAT => Some(Format::RG16Sfloat),
            vk1_0::Format::R16G16B16_UNORM => Some(Format::RGB16Unorm),
            vk1_0::Format::R16G16B16_SNORM => Some(Format::RGB16Snorm),
            vk1_0::Format::R16G16B16_USCALED => Some(Format::RGB16Uscaled),
            vk1_0::Format::R16G16B16_SSCALED => Some(Format::RGB16Sscaled),
            vk1_0::Format::R16G16B16_UINT => Some(Format::RGB16Uint),
            vk1_0::Format::R16G16B16_SINT => Some(Format::RGB16Sint),
            vk1_0::Format::R16G16B16_SFLOAT => Some(Format::RGB16Sfloat),
            vk1_0::Format::R16G16B16A16_UNORM => Some(Format::RGBA16Unorm),
            vk1_0::Format::R16G16B16A16_SNORM => Some(Format::RGBA16Snorm),
            vk1_0::Format::R16G16B16A16_USCALED => Some(Format::RGBA16Uscaled),
            vk1_0::Format::R16G16B16A16_SSCALED => Some(Format::RGBA16Sscaled),
            vk1_0::Format::R16G16B16A16_UINT => Some(Format::RGBA16Uint),
            vk1_0::Format::R16G16B16A16_SINT => Some(Format::RGBA16Sint),
            vk1_0::Format::R16G16B16A16_SFLOAT => Some(Format::RGBA16Sfloat),
            vk1_0::Format::R32_UINT => Some(Format::R32Uint),
            vk1_0::Format::R32_SINT => Some(Format::R32Sint),
            vk1_0::Format::R32_SFLOAT => Some(Format::R32Sfloat),
            vk1_0::Format::R32G32_UINT => Some(Format::RG32Uint),
            vk1_0::Format::R32G32_SINT => Some(Format::RG32Sint),
            vk1_0::Format::R32G32_SFLOAT => Some(Format::RG32Sfloat),
            vk1_0::Format::R32G32B32_UINT => Some(Format::RGB32Uint),
            vk1_0::Format::R32G32B32_SINT => Some(Format::RGB32Sint),
            vk1_0::Format::R32G32B32_SFLOAT => Some(Format::RGB32Sfloat),
            vk1_0::Format::R32G32B32A32_UINT => Some(Format::RGBA32Uint),
            vk1_0::Format::R32G32B32A32_SINT => Some(Format::RGBA32Sint),
            vk1_0::Format::R32G32B32A32_SFLOAT => Some(Format::RGBA32Sfloat),
            vk1_0::Format::R64_UINT => Some(Format::R64Uint),
            vk1_0::Format::R64_SINT => Some(Format::R64Sint),
            vk1_0::Format::R64_SFLOAT => Some(Format::R64Sfloat),
            vk1_0::Format::R64G64_UINT => Some(Format::RG64Uint),
            vk1_0::Format::R64G64_SINT => Some(Format::RG64Sint),
            vk1_0::Format::R64G64_SFLOAT => Some(Format::RG64Sfloat),
            vk1_0::Format::R64G64B64_UINT => Some(Format::RGB64Uint),
            vk1_0::Format::R64G64B64_SINT => Some(Format::RGB64Sint),
            vk1_0::Format::R64G64B64_SFLOAT => Some(Format::RGB64Sfloat),
            vk1_0::Format::R64G64B64A64_UINT => Some(Format::RGBA64Uint),
            vk1_0::Format::R64G64B64A64_SINT => Some(Format::RGBA64Sint),
            vk1_0::Format::R64G64B64A64_SFLOAT => Some(Format::RGBA64Sfloat),
            vk1_0::Format::D16_UNORM => Some(Format::D16Unorm),
            vk1_0::Format::D32_SFLOAT => Some(Format::D32Sfloat),
            vk1_0::Format::S8_UINT => Some(Format::S8Uint),
            vk1_0::Format::D16_UNORM_S8_UINT => Some(Format::D16UnormS8Uint),
            vk1_0::Format::D24_UNORM_S8_UINT => Some(Format::D24UnormS8Uint),
            vk1_0::Format::D32_SFLOAT_S8_UINT => Some(Format::D32SfloatS8Uint),
            _ => None,
        }
    }
}

impl ToErupt<vk1_0::Format> for Format {
    fn to_erupt(self) -> vk1_0::Format {
        match self {
            Format::R8Unorm => vk1_0::Format::R8_UNORM,
            Format::R8Snorm => vk1_0::Format::R8_SNORM,
            Format::R8Uscaled => vk1_0::Format::R8_USCALED,
            Format::R8Sscaled => vk1_0::Format::R8_SSCALED,
            Format::R8Uint => vk1_0::Format::R8_UINT,
            Format::R8Sint => vk1_0::Format::R8_SINT,
            Format::R8Srgb => vk1_0::Format::R8_SRGB,
            Format::RG8Unorm => vk1_0::Format::R8G8_UNORM,
            Format::RG8Snorm => vk1_0::Format::R8G8_SNORM,
            Format::RG8Uscaled => vk1_0::Format::R8G8_USCALED,
            Format::RG8Sscaled => vk1_0::Format::R8G8_SSCALED,
            Format::RG8Uint => vk1_0::Format::R8G8_UINT,
            Format::RG8Sint => vk1_0::Format::R8G8_SINT,
            Format::RG8Srgb => vk1_0::Format::R8G8_SRGB,
            Format::RGB8Unorm => vk1_0::Format::R8G8B8_UNORM,
            Format::RGB8Snorm => vk1_0::Format::R8G8B8_SNORM,
            Format::RGB8Uscaled => vk1_0::Format::R8G8B8_USCALED,
            Format::RGB8Sscaled => vk1_0::Format::R8G8B8_SSCALED,
            Format::RGB8Uint => vk1_0::Format::R8G8B8_UINT,
            Format::RGB8Sint => vk1_0::Format::R8G8B8_SINT,
            Format::RGB8Srgb => vk1_0::Format::R8G8B8_SRGB,
            Format::BGR8Unorm => vk1_0::Format::B8G8R8_UNORM,
            Format::BGR8Snorm => vk1_0::Format::B8G8R8_SNORM,
            Format::BGR8Uscaled => vk1_0::Format::B8G8R8_USCALED,
            Format::BGR8Sscaled => vk1_0::Format::B8G8R8_SSCALED,
            Format::BGR8Uint => vk1_0::Format::B8G8R8_UINT,
            Format::BGR8Sint => vk1_0::Format::B8G8R8_SINT,
            Format::BGR8Srgb => vk1_0::Format::B8G8R8_SRGB,
            Format::RGBA8Unorm => vk1_0::Format::R8G8B8A8_UNORM,
            Format::RGBA8Snorm => vk1_0::Format::R8G8B8A8_SNORM,
            Format::RGBA8Uscaled => vk1_0::Format::R8G8B8A8_USCALED,
            Format::RGBA8Sscaled => vk1_0::Format::R8G8B8A8_SSCALED,
            Format::RGBA8Uint => vk1_0::Format::R8G8B8A8_UINT,
            Format::RGBA8Sint => vk1_0::Format::R8G8B8A8_SINT,
            Format::RGBA8Srgb => vk1_0::Format::R8G8B8A8_SRGB,
            Format::BGRA8Unorm => vk1_0::Format::B8G8R8A8_UNORM,
            Format::BGRA8Snorm => vk1_0::Format::B8G8R8A8_SNORM,
            Format::BGRA8Uscaled => vk1_0::Format::B8G8R8A8_USCALED,
            Format::BGRA8Sscaled => vk1_0::Format::B8G8R8A8_SSCALED,
            Format::BGRA8Uint => vk1_0::Format::B8G8R8A8_UINT,
            Format::BGRA8Sint => vk1_0::Format::B8G8R8A8_SINT,
            Format::BGRA8Srgb => vk1_0::Format::B8G8R8A8_SRGB,
            Format::R16Unorm => vk1_0::Format::R16_UNORM,
            Format::R16Snorm => vk1_0::Format::R16_SNORM,
            Format::R16Uscaled => vk1_0::Format::R16_USCALED,
            Format::R16Sscaled => vk1_0::Format::R16_SSCALED,
            Format::R16Uint => vk1_0::Format::R16_UINT,
            Format::R16Sint => vk1_0::Format::R16_SINT,
            Format::R16Sfloat => vk1_0::Format::R16_SFLOAT,
            Format::RG16Unorm => vk1_0::Format::R16G16_UNORM,
            Format::RG16Snorm => vk1_0::Format::R16G16_SNORM,
            Format::RG16Uscaled => vk1_0::Format::R16G16_USCALED,
            Format::RG16Sscaled => vk1_0::Format::R16G16_SSCALED,
            Format::RG16Uint => vk1_0::Format::R16G16_UINT,
            Format::RG16Sint => vk1_0::Format::R16G16_SINT,
            Format::RG16Sfloat => vk1_0::Format::R16G16_SFLOAT,
            Format::RGB16Unorm => vk1_0::Format::R16G16B16_UNORM,
            Format::RGB16Snorm => vk1_0::Format::R16G16B16_SNORM,
            Format::RGB16Uscaled => vk1_0::Format::R16G16B16_USCALED,
            Format::RGB16Sscaled => vk1_0::Format::R16G16B16_SSCALED,
            Format::RGB16Uint => vk1_0::Format::R16G16B16_UINT,
            Format::RGB16Sint => vk1_0::Format::R16G16B16_SINT,
            Format::RGB16Sfloat => vk1_0::Format::R16G16B16_SFLOAT,
            Format::RGBA16Unorm => vk1_0::Format::R16G16B16A16_UNORM,
            Format::RGBA16Snorm => vk1_0::Format::R16G16B16A16_SNORM,
            Format::RGBA16Uscaled => vk1_0::Format::R16G16B16A16_USCALED,
            Format::RGBA16Sscaled => vk1_0::Format::R16G16B16A16_SSCALED,
            Format::RGBA16Uint => vk1_0::Format::R16G16B16A16_UINT,
            Format::RGBA16Sint => vk1_0::Format::R16G16B16A16_SINT,
            Format::RGBA16Sfloat => vk1_0::Format::R16G16B16A16_SFLOAT,
            Format::R32Uint => vk1_0::Format::R32_UINT,
            Format::R32Sint => vk1_0::Format::R32_SINT,
            Format::R32Sfloat => vk1_0::Format::R32_SFLOAT,
            Format::RG32Uint => vk1_0::Format::R32G32_UINT,
            Format::RG32Sint => vk1_0::Format::R32G32_SINT,
            Format::RG32Sfloat => vk1_0::Format::R32G32_SFLOAT,
            Format::RGB32Uint => vk1_0::Format::R32G32B32_UINT,
            Format::RGB32Sint => vk1_0::Format::R32G32B32_SINT,
            Format::RGB32Sfloat => vk1_0::Format::R32G32B32_SFLOAT,
            Format::RGBA32Uint => vk1_0::Format::R32G32B32A32_UINT,
            Format::RGBA32Sint => vk1_0::Format::R32G32B32A32_SINT,
            Format::RGBA32Sfloat => vk1_0::Format::R32G32B32A32_SFLOAT,
            Format::R64Uint => vk1_0::Format::R64_UINT,
            Format::R64Sint => vk1_0::Format::R64_SINT,
            Format::R64Sfloat => vk1_0::Format::R64_SFLOAT,
            Format::RG64Uint => vk1_0::Format::R64G64_UINT,
            Format::RG64Sint => vk1_0::Format::R64G64_SINT,
            Format::RG64Sfloat => vk1_0::Format::R64G64_SFLOAT,
            Format::RGB64Uint => vk1_0::Format::R64G64B64_UINT,
            Format::RGB64Sint => vk1_0::Format::R64G64B64_SINT,
            Format::RGB64Sfloat => vk1_0::Format::R64G64B64_SFLOAT,
            Format::RGBA64Uint => vk1_0::Format::R64G64B64A64_UINT,
            Format::RGBA64Sint => vk1_0::Format::R64G64B64A64_SINT,
            Format::RGBA64Sfloat => vk1_0::Format::R64G64B64A64_SFLOAT,
            Format::D16Unorm => vk1_0::Format::D16_UNORM,
            Format::D32Sfloat => vk1_0::Format::D32_SFLOAT,
            Format::S8Uint => vk1_0::Format::S8_UINT,
            Format::D16UnormS8Uint => vk1_0::Format::D16_UNORM_S8_UINT,
            Format::D24UnormS8Uint => vk1_0::Format::D24_UNORM_S8_UINT,
            Format::D32SfloatS8Uint => vk1_0::Format::D32_SFLOAT_S8_UINT,
        }
    }
}

impl ToErupt<vk1_0::Format> for Option<Format> {
    fn to_erupt(self) -> vk1_0::Format {
        self.map(Format::to_erupt)
            .unwrap_or(vk1_0::Format::UNDEFINED)
    }
}

impl FromErupt<vk1_0::Extent2D> for Extent2d {
    fn from_erupt(extent: vk1_0::Extent2D) -> Self {
        Extent2d {
            width: extent.width,
            height: extent.height,
        }
    }
}

impl ToErupt<vk1_0::Extent2D> for Extent2d {
    fn to_erupt(self) -> vk1_0::Extent2D {
        vk1_0::Extent2D {
            width: self.width,
            height: self.height,
        }
    }
}

impl FromErupt<vk1_0::Extent3D> for Extent3d {
    fn from_erupt(extent: vk1_0::Extent3D) -> Self {
        Extent3d {
            width: extent.width,
            height: extent.height,
            depth: extent.depth,
        }
    }
}

impl ToErupt<vk1_0::Extent3D> for Extent3d {
    fn to_erupt(self) -> vk1_0::Extent3D {
        vk1_0::Extent3D {
            width: self.width,
            height: self.height,
            depth: self.depth,
        }
    }
}

impl FromErupt<vk1_0::Offset2D> for Offset2d {
    fn from_erupt(offset: vk1_0::Offset2D) -> Offset2d {
        Offset2d {
            x: offset.x,
            y: offset.y,
        }
    }
}

impl ToErupt<vk1_0::Offset2D> for Offset2d {
    fn to_erupt(self) -> vk1_0::Offset2D {
        vk1_0::Offset2D {
            x: self.x,
            y: self.y,
        }
    }
}

impl FromErupt<vk1_0::Offset3D> for Offset3d {
    fn from_erupt(offset: vk1_0::Offset3D) -> Offset3d {
        Offset3d {
            x: offset.x,
            y: offset.y,
            z: offset.z,
        }
    }
}

impl ToErupt<vk1_0::Offset3D> for Offset3d {
    fn to_erupt(self) -> vk1_0::Offset3D {
        vk1_0::Offset3D {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }
}

impl FromErupt<vk1_0::Rect2D> for Rect2d {
    fn from_erupt(rect: vk1_0::Rect2D) -> Rect2d {
        Rect2d {
            offset: Offset2d::from_erupt(rect.offset),
            extent: Extent2d::from_erupt(rect.extent),
        }
    }
}

impl ToErupt<vk1_0::Rect2D> for Rect2d {
    fn to_erupt(self) -> vk1_0::Rect2D {
        vk1_0::Rect2D {
            offset: self.offset.to_erupt(),
            extent: self.extent.to_erupt(),
        }
    }
}

impl FromErupt<vk1_0::ImageUsageFlags> for ImageUsage {
    fn from_erupt(usage: vk1_0::ImageUsageFlags) -> ImageUsage {
        let mut result = ImageUsage::empty();

        if usage.contains(vk1_0::ImageUsageFlags::TRANSFER_SRC) {
            result |= ImageUsage::TRANSFER_SRC;
        }

        if usage.contains(vk1_0::ImageUsageFlags::TRANSFER_DST) {
            result |= ImageUsage::TRANSFER_DST;
        }

        if usage.contains(vk1_0::ImageUsageFlags::SAMPLED) {
            result |= ImageUsage::SAMPLED;
        }

        if usage.contains(vk1_0::ImageUsageFlags::STORAGE) {
            result |= ImageUsage::STORAGE;
        }

        if usage.contains(vk1_0::ImageUsageFlags::COLOR_ATTACHMENT) {
            result |= ImageUsage::COLOR_ATTACHMENT;
        }

        if usage.contains(vk1_0::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT) {
            result |= ImageUsage::DEPTH_STENCIL_ATTACHMENT;
        }

        if usage.contains(vk1_0::ImageUsageFlags::TRANSIENT_ATTACHMENT) {
            result |= ImageUsage::TRANSIENT_ATTACHMENT;
        }

        if usage.contains(vk1_0::ImageUsageFlags::INPUT_ATTACHMENT) {
            result |= ImageUsage::INPUT_ATTACHMENT;
        }

        result
    }
}

impl ToErupt<vk1_0::ImageUsageFlags> for ImageUsage {
    fn to_erupt(self) -> vk1_0::ImageUsageFlags {
        let mut result = vk1_0::ImageUsageFlags::empty();

        if self.contains(ImageUsage::TRANSFER_SRC) {
            result |= vk1_0::ImageUsageFlags::TRANSFER_SRC;
        }

        if self.contains(ImageUsage::TRANSFER_DST) {
            result |= vk1_0::ImageUsageFlags::TRANSFER_DST;
        }

        if self.contains(ImageUsage::SAMPLED) {
            result |= vk1_0::ImageUsageFlags::SAMPLED;
        }

        if self.contains(ImageUsage::STORAGE) {
            result |= vk1_0::ImageUsageFlags::STORAGE;
        }

        if self.contains(ImageUsage::COLOR_ATTACHMENT) {
            result |= vk1_0::ImageUsageFlags::COLOR_ATTACHMENT;
        }

        if self.contains(ImageUsage::DEPTH_STENCIL_ATTACHMENT) {
            result |= vk1_0::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT;
        }

        if self.contains(ImageUsage::TRANSIENT_ATTACHMENT) {
            result |= vk1_0::ImageUsageFlags::TRANSIENT_ATTACHMENT;
        }

        if self.contains(ImageUsage::INPUT_ATTACHMENT) {
            result |= vk1_0::ImageUsageFlags::INPUT_ATTACHMENT;
        }

        result
    }
}

impl FromErupt<vk1_0::BufferUsageFlags> for BufferUsage {
    fn from_erupt(usage: vk1_0::BufferUsageFlags) -> BufferUsage {
        let mut result = BufferUsage::empty();

        if usage.contains(vk1_0::BufferUsageFlags::TRANSFER_SRC) {
            result |= BufferUsage::TRANSFER_SRC;
        }

        if usage.contains(vk1_0::BufferUsageFlags::TRANSFER_DST) {
            result |= BufferUsage::TRANSFER_DST;
        }

        if usage.contains(vk1_0::BufferUsageFlags::UNIFORM_TEXEL_BUFFER) {
            result |= BufferUsage::UNIFORM_TEXEL;
        }

        if usage.contains(vk1_0::BufferUsageFlags::STORAGE_TEXEL_BUFFER) {
            result |= BufferUsage::STORAGE_TEXEL;
        }

        if usage.contains(vk1_0::BufferUsageFlags::UNIFORM_BUFFER) {
            result |= BufferUsage::UNIFORM;
        }

        if usage.contains(vk1_0::BufferUsageFlags::STORAGE_BUFFER) {
            result |= BufferUsage::STORAGE;
        }

        if usage.contains(vk1_0::BufferUsageFlags::INDEX_BUFFER) {
            result |= BufferUsage::INDEX;
        }

        if usage.contains(vk1_0::BufferUsageFlags::VERTEX_BUFFER) {
            result |= BufferUsage::VERTEX;
        }

        if usage.contains(vk1_0::BufferUsageFlags::INDIRECT_BUFFER) {
            result |= BufferUsage::INDIRECT;
        }

        if usage.contains(vk1_0::BufferUsageFlags::CONDITIONAL_RENDERING_EXT) {
            result |= BufferUsage::CONDITIONAL_RENDERING;
        }

        if usage.contains(vk1_0::BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR)
        {
            result |= BufferUsage::ACCELERATION_STRUCTURE_BUILD_INPUT;
        }

        if usage.contains(vk1_0::BufferUsageFlags::ACCELERATION_STRUCTURE_STORAGE_KHR) {
            result |= BufferUsage::ACCELERATION_STRUCTURE_STORAGE;
        }

        if usage.contains(vk1_0::BufferUsageFlags::SHADER_BINDING_TABLE_KHR) {
            result |= BufferUsage::SHADER_BINDING_TABLE;
        }

        if usage.contains(vk1_0::BufferUsageFlags::TRANSFORM_FEEDBACK_BUFFER_EXT) {
            result |= BufferUsage::TRANSFORM_FEEDBACK;
        }

        if usage.contains(vk1_0::BufferUsageFlags::TRANSFORM_FEEDBACK_COUNTER_BUFFER_EXT) {
            result |= BufferUsage::TRANSFORM_FEEDBACK_COUNTER;
        }

        if usage.contains(vk1_0::BufferUsageFlags::SHADER_DEVICE_ADDRESS) {
            result |= BufferUsage::DEVICE_ADDRESS;
        }

        result
    }
}

impl ToErupt<vk1_0::BufferUsageFlags> for BufferUsage {
    fn to_erupt(self) -> vk1_0::BufferUsageFlags {
        let mut result = vk1_0::BufferUsageFlags::empty();

        if self.contains(BufferUsage::TRANSFER_SRC) {
            result |= vk1_0::BufferUsageFlags::TRANSFER_SRC;
        }

        if self.contains(BufferUsage::TRANSFER_DST) {
            result |= vk1_0::BufferUsageFlags::TRANSFER_DST;
        }

        if self.contains(BufferUsage::UNIFORM_TEXEL) {
            result |= vk1_0::BufferUsageFlags::UNIFORM_TEXEL_BUFFER;
        }

        if self.contains(BufferUsage::STORAGE_TEXEL) {
            result |= vk1_0::BufferUsageFlags::STORAGE_TEXEL_BUFFER;
        }

        if self.contains(BufferUsage::UNIFORM) {
            result |= vk1_0::BufferUsageFlags::UNIFORM_BUFFER;
        }

        if self.contains(BufferUsage::STORAGE) {
            result |= vk1_0::BufferUsageFlags::STORAGE_BUFFER;
        }

        if self.contains(BufferUsage::INDEX) {
            result |= vk1_0::BufferUsageFlags::INDEX_BUFFER;
        }

        if self.contains(BufferUsage::VERTEX) {
            result |= vk1_0::BufferUsageFlags::VERTEX_BUFFER;
        }

        if self.contains(BufferUsage::INDIRECT) {
            result |= vk1_0::BufferUsageFlags::INDIRECT_BUFFER;
        }

        if self.contains(BufferUsage::CONDITIONAL_RENDERING) {
            result |= vk1_0::BufferUsageFlags::CONDITIONAL_RENDERING_EXT;
        }

        if self.contains(BufferUsage::ACCELERATION_STRUCTURE_BUILD_INPUT) {
            result |= vk1_0::BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR;
        }

        if self.contains(BufferUsage::ACCELERATION_STRUCTURE_STORAGE) {
            result |= vk1_0::BufferUsageFlags::ACCELERATION_STRUCTURE_STORAGE_KHR;
        }

        if self.contains(BufferUsage::SHADER_BINDING_TABLE) {
            result |= vk1_0::BufferUsageFlags::SHADER_BINDING_TABLE_KHR;
        }

        if self.contains(BufferUsage::TRANSFORM_FEEDBACK) {
            result |= vk1_0::BufferUsageFlags::TRANSFORM_FEEDBACK_BUFFER_EXT;
        }

        if self.contains(BufferUsage::TRANSFORM_FEEDBACK_COUNTER) {
            result |= vk1_0::BufferUsageFlags::TRANSFORM_FEEDBACK_COUNTER_BUFFER_EXT;
        }

        if self.contains(BufferUsage::DEVICE_ADDRESS) {
            result |= vk1_0::BufferUsageFlags::SHADER_DEVICE_ADDRESS;
        }

        result
    }
}

impl FromErupt<PresentModeKHR> for Option<PresentMode> {
    fn from_erupt(mode: PresentModeKHR) -> Option<PresentMode> {
        match mode {
            PresentModeKHR::IMMEDIATE_KHR => Some(PresentMode::Immediate),
            PresentModeKHR::MAILBOX_KHR => Some(PresentMode::Mailbox),
            PresentModeKHR::FIFO_KHR => Some(PresentMode::Fifo),
            PresentModeKHR::FIFO_RELAXED_KHR => Some(PresentMode::FifoRelaxed),
            _ => None,
        }
    }
}

impl ToErupt<PresentModeKHR> for PresentMode {
    fn to_erupt(self) -> PresentModeKHR {
        match self {
            PresentMode::Immediate => PresentModeKHR::IMMEDIATE_KHR,
            PresentMode::Mailbox => PresentModeKHR::MAILBOX_KHR,
            PresentMode::Fifo => PresentModeKHR::FIFO_KHR,
            PresentMode::FifoRelaxed => PresentModeKHR::FIFO_RELAXED_KHR,
        }
    }
}

#[track_caller]
pub(crate) fn oom_error_from_erupt(err: vk1_0::Result) -> OutOfMemory {
    match err {
        vk1_0::Result::ERROR_OUT_OF_HOST_MEMORY => out_of_host_memory(),
        vk1_0::Result::ERROR_OUT_OF_DEVICE_MEMORY => OutOfMemory,
        _ => unreachable!("Error {} is unexpected", err),
    }
}

impl ToErupt<vk1_0::AttachmentLoadOp> for LoadOp {
    fn to_erupt(self) -> vk1_0::AttachmentLoadOp {
        match self {
            LoadOp::Load => vk1_0::AttachmentLoadOp::LOAD,
            LoadOp::Clear => vk1_0::AttachmentLoadOp::CLEAR,
            LoadOp::DontCare => vk1_0::AttachmentLoadOp::DONT_CARE,
        }
    }
}

impl ToErupt<vk1_0::AttachmentStoreOp> for StoreOp {
    fn to_erupt(self) -> vk1_0::AttachmentStoreOp {
        match self {
            StoreOp::Store => vk1_0::AttachmentStoreOp::STORE,
            StoreOp::DontCare => vk1_0::AttachmentStoreOp::DONT_CARE,
        }
    }
}

impl FromErupt<vk1_0::QueueFlags> for QueueCapabilityFlags {
    fn from_erupt(flags: vk1_0::QueueFlags) -> QueueCapabilityFlags {
        let mut result = QueueCapabilityFlags::empty();

        if flags.contains(vk1_0::QueueFlags::TRANSFER) {
            result |= QueueCapabilityFlags::TRANSFER
        }

        if flags.contains(vk1_0::QueueFlags::COMPUTE) {
            result |= QueueCapabilityFlags::COMPUTE
        }

        if flags.contains(vk1_0::QueueFlags::GRAPHICS) {
            result |= QueueCapabilityFlags::GRAPHICS
        }

        result
    }
}

impl ToErupt<vk1_0::ImageAspectFlags> for AspectFlags {
    fn to_erupt(self) -> vk1_0::ImageAspectFlags {
        let mut result = vk1_0::ImageAspectFlags::empty();

        if self.contains(AspectFlags::COLOR) {
            result |= vk1_0::ImageAspectFlags::COLOR;
        }

        if self.contains(AspectFlags::DEPTH) {
            result |= vk1_0::ImageAspectFlags::DEPTH;
        }

        if self.contains(AspectFlags::STENCIL) {
            result |= vk1_0::ImageAspectFlags::STENCIL;
        }

        result
    }
}

impl ToErupt<vk1_0::PipelineStageFlags> for PipelineStageFlags {
    fn to_erupt(self) -> vk1_0::PipelineStageFlags {
        let mut result = vk1_0::PipelineStageFlags::empty();

        if self.contains(PipelineStageFlags::TOP_OF_PIPE) {
            result |= vk1_0::PipelineStageFlags::TOP_OF_PIPE
        }

        if self.contains(PipelineStageFlags::DRAW_INDIRECT) {
            result |= vk1_0::PipelineStageFlags::DRAW_INDIRECT
        }

        if self.contains(PipelineStageFlags::VERTEX_INPUT) {
            result |= vk1_0::PipelineStageFlags::VERTEX_INPUT
        }

        if self.contains(PipelineStageFlags::VERTEX_SHADER) {
            result |= vk1_0::PipelineStageFlags::VERTEX_SHADER
        }

        if self.contains(PipelineStageFlags::EARLY_FRAGMENT_TESTS) {
            result |= vk1_0::PipelineStageFlags::EARLY_FRAGMENT_TESTS
        }

        if self.contains(PipelineStageFlags::FRAGMENT_SHADER) {
            result |= vk1_0::PipelineStageFlags::FRAGMENT_SHADER
        }

        if self.contains(PipelineStageFlags::LATE_FRAGMENT_TESTS) {
            result |= vk1_0::PipelineStageFlags::LATE_FRAGMENT_TESTS
        }

        if self.contains(PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT) {
            result |= vk1_0::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
        }

        if self.contains(PipelineStageFlags::COMPUTE_SHADER) {
            result |= vk1_0::PipelineStageFlags::COMPUTE_SHADER
        }

        if self.contains(PipelineStageFlags::TRANSFER) {
            result |= vk1_0::PipelineStageFlags::TRANSFER
        }

        if self.contains(PipelineStageFlags::BOTTOM_OF_PIPE) {
            result |= vk1_0::PipelineStageFlags::BOTTOM_OF_PIPE
        }

        if self.contains(PipelineStageFlags::HOST) {
            result |= vk1_0::PipelineStageFlags::HOST
        }

        if self.contains(PipelineStageFlags::ALL_GRAPHICS) {
            result |= vk1_0::PipelineStageFlags::ALL_GRAPHICS
        }

        if self.contains(PipelineStageFlags::ALL_COMMANDS) {
            result |= vk1_0::PipelineStageFlags::ALL_COMMANDS
        }

        if self.contains(PipelineStageFlags::RAY_TRACING_SHADER) {
            result |= vk1_0::PipelineStageFlags::RAY_TRACING_SHADER_KHR
        }

        if self.contains(PipelineStageFlags::ACCELERATION_STRUCTURE_BUILD) {
            result |= vk1_0::PipelineStageFlags::ACCELERATION_STRUCTURE_BUILD_KHR
        }

        result
    }
}

impl ToErupt<vk1_0::ShaderStageFlags> for ShaderStageFlags {
    fn to_erupt(self) -> vk1_0::ShaderStageFlags {
        if self == ShaderStageFlags::ALL {
            return vk1_0::ShaderStageFlags::ALL;
        }

        let mut result = vk1_0::ShaderStageFlags::empty();

        if self.contains(ShaderStageFlags::VERTEX) {
            result |= vk1_0::ShaderStageFlags::VERTEX;
        }

        if self.contains(ShaderStageFlags::TESSELLATION_CONTROL) {
            result |= vk1_0::ShaderStageFlags::TESSELLATION_CONTROL;
        }

        if self.contains(ShaderStageFlags::TESSELLATION_EVALUATION) {
            result |= vk1_0::ShaderStageFlags::TESSELLATION_EVALUATION;
        }

        if self.contains(ShaderStageFlags::GEOMETRY) {
            result |= vk1_0::ShaderStageFlags::GEOMETRY;
        }

        if self.contains(ShaderStageFlags::FRAGMENT) {
            result |= vk1_0::ShaderStageFlags::FRAGMENT;
        }

        if self.contains(ShaderStageFlags::COMPUTE) {
            result |= vk1_0::ShaderStageFlags::COMPUTE;
        }

        if self.contains(ShaderStageFlags::RAYGEN) {
            result |= vk1_0::ShaderStageFlags::RAYGEN_KHR;
        }

        if self.contains(ShaderStageFlags::ANY_HIT) {
            result |= vk1_0::ShaderStageFlags::ANY_HIT_KHR;
        }

        if self.contains(ShaderStageFlags::CLOSEST_HIT) {
            result |= vk1_0::ShaderStageFlags::CLOSEST_HIT_KHR;
        }

        if self.contains(ShaderStageFlags::MISS) {
            result |= vk1_0::ShaderStageFlags::MISS_KHR;
        }

        if self.contains(ShaderStageFlags::INTERSECTION) {
            result |= vk1_0::ShaderStageFlags::INTERSECTION_KHR;
        }

        if self.contains(ShaderStageFlags::ALL_GRAPHICS) {
            result |= vk1_0::ShaderStageFlags::ALL_GRAPHICS;
        }

        result
    }
}

impl ToErupt<vk1_0::ShaderStageFlagBits> for ShaderStage {
    fn to_erupt(self) -> vk1_0::ShaderStageFlagBits {
        match self {
            ShaderStage::Vertex => vk1_0::ShaderStageFlagBits::VERTEX,
            ShaderStage::TessellationControl => vk1_0::ShaderStageFlagBits::TESSELLATION_CONTROL,
            ShaderStage::TessellationEvaluation => {
                vk1_0::ShaderStageFlagBits::TESSELLATION_EVALUATION
            }
            ShaderStage::Geometry => vk1_0::ShaderStageFlagBits::GEOMETRY,
            ShaderStage::Fragment => vk1_0::ShaderStageFlagBits::FRAGMENT,
            ShaderStage::Compute => vk1_0::ShaderStageFlagBits::COMPUTE,
            ShaderStage::Raygen => vk1_0::ShaderStageFlagBits::RAYGEN_KHR,
            ShaderStage::AnyHit => vk1_0::ShaderStageFlagBits::ANY_HIT_KHR,
            ShaderStage::ClosestHit => vk1_0::ShaderStageFlagBits::CLOSEST_HIT_KHR,
            ShaderStage::Miss => vk1_0::ShaderStageFlagBits::MISS_KHR,
            ShaderStage::Intersection => vk1_0::ShaderStageFlagBits::INTERSECTION_KHR,
        }
    }
}

impl ToErupt<vk1_0::VertexInputRate> for VertexInputRate {
    fn to_erupt(self) -> vk1_0::VertexInputRate {
        match self {
            VertexInputRate::Vertex => vk1_0::VertexInputRate::VERTEX,
            VertexInputRate::Instance => vk1_0::VertexInputRate::INSTANCE,
        }
    }
}

impl ToErupt<vk1_0::PrimitiveTopology> for PrimitiveTopology {
    fn to_erupt(self) -> vk1_0::PrimitiveTopology {
        match self {
            PrimitiveTopology::PointList => vk1_0::PrimitiveTopology::POINT_LIST,
            PrimitiveTopology::LineList => vk1_0::PrimitiveTopology::LINE_LIST,
            PrimitiveTopology::LineStrip => vk1_0::PrimitiveTopology::LINE_STRIP,
            PrimitiveTopology::TriangleList => vk1_0::PrimitiveTopology::TRIANGLE_LIST,
            PrimitiveTopology::TriangleStrip => vk1_0::PrimitiveTopology::TRIANGLE_STRIP,
            PrimitiveTopology::TriangleFan => vk1_0::PrimitiveTopology::TRIANGLE_FAN,
        }
    }
}

impl ToErupt<vk1_0::Viewport> for Viewport {
    fn to_erupt(self) -> vk1_0::Viewport {
        vk1_0::Viewport {
            x: self.x.offset.into(),
            y: self.y.offset.into(),
            width: self.x.size.into(),
            height: self.y.size.into(),
            min_depth: self.z.offset.into(),
            max_depth: self.z.size.into_inner() + self.z.offset.into_inner(),
        }
    }
}

impl ToErupt<vk1_0::PolygonMode> for PolygonMode {
    fn to_erupt(self) -> vk1_0::PolygonMode {
        match self {
            PolygonMode::Point => vk1_0::PolygonMode::POINT,
            PolygonMode::Line => vk1_0::PolygonMode::LINE,
            PolygonMode::Fill => vk1_0::PolygonMode::FILL,
        }
    }
}

impl ToErupt<vk1_0::CullModeFlags> for Option<Culling> {
    fn to_erupt(self) -> vk1_0::CullModeFlags {
        match self {
            None => vk1_0::CullModeFlags::NONE,
            Some(Culling::Front) => vk1_0::CullModeFlags::FRONT,
            Some(Culling::Back) => vk1_0::CullModeFlags::BACK,
            Some(Culling::FrontAndBack) => vk1_0::CullModeFlags::FRONT_AND_BACK,
        }
    }
}

impl ToErupt<vk1_0::FrontFace> for FrontFace {
    fn to_erupt(self) -> vk1_0::FrontFace {
        match self {
            FrontFace::Clockwise => vk1_0::FrontFace::CLOCKWISE,
            FrontFace::CounterClockwise => vk1_0::FrontFace::COUNTER_CLOCKWISE,
        }
    }
}

impl ToErupt<vk1_0::CompareOp> for CompareOp {
    fn to_erupt(self) -> vk1_0::CompareOp {
        match self {
            CompareOp::Never => vk1_0::CompareOp::NEVER,
            CompareOp::Less => vk1_0::CompareOp::LESS,
            CompareOp::Equal => vk1_0::CompareOp::EQUAL,
            CompareOp::LessOrEqual => vk1_0::CompareOp::LESS_OR_EQUAL,
            CompareOp::Greater => vk1_0::CompareOp::GREATER,
            CompareOp::NotEqual => vk1_0::CompareOp::NOT_EQUAL,
            CompareOp::GreaterOrEqual => vk1_0::CompareOp::GREATER_OR_EQUAL,
            CompareOp::Always => vk1_0::CompareOp::ALWAYS,
        }
    }
}

impl ToErupt<vk1_0::StencilOp> for StencilOp {
    fn to_erupt(self) -> vk1_0::StencilOp {
        match self {
            StencilOp::Keep => vk1_0::StencilOp::KEEP,
            StencilOp::Zero => vk1_0::StencilOp::ZERO,
            StencilOp::Replace => vk1_0::StencilOp::REPLACE,
            StencilOp::IncrementAndClamp => vk1_0::StencilOp::INCREMENT_AND_CLAMP,
            StencilOp::DecrementAndClamp => vk1_0::StencilOp::DECREMENT_AND_CLAMP,
            StencilOp::Invert => vk1_0::StencilOp::INVERT,
            StencilOp::IncrementAndWrap => vk1_0::StencilOp::INCREMENT_AND_WRAP,
            StencilOp::DecrementAndWrap => vk1_0::StencilOp::DECREMENT_AND_WRAP,
        }
    }
}

impl ToErupt<vk1_0::LogicOp> for LogicOp {
    fn to_erupt(self) -> vk1_0::LogicOp {
        match self {
            LogicOp::Clear => vk1_0::LogicOp::CLEAR,
            LogicOp::And => vk1_0::LogicOp::AND,
            LogicOp::AndReverse => vk1_0::LogicOp::AND_REVERSE,
            LogicOp::Copy => vk1_0::LogicOp::COPY,
            LogicOp::AndInverted => vk1_0::LogicOp::AND_INVERTED,
            LogicOp::NoOp => vk1_0::LogicOp::NO_OP,
            LogicOp::Xor => vk1_0::LogicOp::XOR,
            LogicOp::Or => vk1_0::LogicOp::OR,
            LogicOp::Nor => vk1_0::LogicOp::NOR,
            LogicOp::Equivalent => vk1_0::LogicOp::EQUIVALENT,
            LogicOp::Invert => vk1_0::LogicOp::INVERT,
            LogicOp::OrReverse => vk1_0::LogicOp::OR_REVERSE,
            LogicOp::CopyInverted => vk1_0::LogicOp::COPY_INVERTED,
            LogicOp::OrInverted => vk1_0::LogicOp::OR_INVERTED,
            LogicOp::Nand => vk1_0::LogicOp::NAND,
            LogicOp::Set => vk1_0::LogicOp::SET,
        }
    }
}

impl ToErupt<vk1_0::BlendFactor> for BlendFactor {
    fn to_erupt(self) -> vk1_0::BlendFactor {
        match self {
            BlendFactor::Zero => vk1_0::BlendFactor::ZERO,
            BlendFactor::One => vk1_0::BlendFactor::ONE,
            BlendFactor::SrcColor => vk1_0::BlendFactor::SRC_COLOR,
            BlendFactor::OneMinusSrcColor => vk1_0::BlendFactor::ONE_MINUS_SRC_COLOR,
            BlendFactor::DstColor => vk1_0::BlendFactor::DST_COLOR,
            BlendFactor::OneMinusDstColor => vk1_0::BlendFactor::ONE_MINUS_DST_COLOR,
            BlendFactor::SrcAlpha => vk1_0::BlendFactor::SRC_ALPHA,
            BlendFactor::OneMinusSrcAlpha => vk1_0::BlendFactor::ONE_MINUS_SRC_ALPHA,
            BlendFactor::DstAlpha => vk1_0::BlendFactor::DST_ALPHA,
            BlendFactor::OneMinusDstAlpha => vk1_0::BlendFactor::ONE_MINUS_DST_ALPHA,
            BlendFactor::ConstantColor => vk1_0::BlendFactor::CONSTANT_COLOR,
            BlendFactor::OneMinusConstantColor => vk1_0::BlendFactor::ONE_MINUS_CONSTANT_COLOR,
            BlendFactor::ConstantAlpha => vk1_0::BlendFactor::CONSTANT_ALPHA,
            BlendFactor::OneMinusConstantAlpha => vk1_0::BlendFactor::ONE_MINUS_CONSTANT_ALPHA,
            BlendFactor::SrcAlphaSaturate => vk1_0::BlendFactor::SRC_ALPHA_SATURATE,
        }
    }
}

impl ToErupt<vk1_0::BlendOp> for BlendOp {
    fn to_erupt(self) -> vk1_0::BlendOp {
        match self {
            BlendOp::Add => vk1_0::BlendOp::ADD,
            BlendOp::Subtract => vk1_0::BlendOp::SUBTRACT,
            BlendOp::ReverseSubtract => vk1_0::BlendOp::REVERSE_SUBTRACT,
            BlendOp::Min => vk1_0::BlendOp::MIN,
            BlendOp::Max => vk1_0::BlendOp::MAX,
        }
    }
}

impl ToErupt<vk1_0::ColorComponentFlags> for ComponentMask {
    fn to_erupt(self) -> vk1_0::ColorComponentFlags {
        let mut result = vk1_0::ColorComponentFlags::empty();

        if self.contains(ComponentMask::R) {
            result |= vk1_0::ColorComponentFlags::R
        }

        if self.contains(ComponentMask::G) {
            result |= vk1_0::ColorComponentFlags::G
        }

        if self.contains(ComponentMask::B) {
            result |= vk1_0::ColorComponentFlags::B
        }

        if self.contains(ComponentMask::A) {
            result |= vk1_0::ColorComponentFlags::A
        }

        result
    }
}

impl ToErupt<vk1_0::ImageLayout> for Layout {
    fn to_erupt(self) -> vk1_0::ImageLayout {
        match self {
            Layout::General => vk1_0::ImageLayout::GENERAL,
            Layout::ColorAttachmentOptimal => vk1_0::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            Layout::DepthStencilAttachmentOptimal => {
                vk1_0::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL
            }
            Layout::DepthStencilReadOnlyOptimal => {
                vk1_0::ImageLayout::DEPTH_STENCIL_READ_ONLY_OPTIMAL
            }
            Layout::ShaderReadOnlyOptimal => vk1_0::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            Layout::TransferSrcOptimal => vk1_0::ImageLayout::TRANSFER_SRC_OPTIMAL,
            Layout::TransferDstOptimal => vk1_0::ImageLayout::TRANSFER_DST_OPTIMAL,
            Layout::Present => vk1_0::ImageLayout::PRESENT_SRC_KHR,
        }
    }
}

impl ToErupt<vk1_0::ImageLayout> for Option<Layout> {
    fn to_erupt(self) -> vk1_0::ImageLayout {
        match self {
            None => vk1_0::ImageLayout::UNDEFINED,
            Some(layout) => layout.to_erupt(),
        }
    }
}

impl ToErupt<vk1_0::ImageViewType> for ImageViewKind {
    fn to_erupt(self) -> vk1_0::ImageViewType {
        match self {
            ImageViewKind::D1 => vk1_0::ImageViewType::_1D,
            ImageViewKind::D2 => vk1_0::ImageViewType::_2D,
            ImageViewKind::D3 => vk1_0::ImageViewType::_3D,
            ImageViewKind::Cube => vk1_0::ImageViewType::CUBE,
        }
    }
}

impl ToErupt<vk1_0::ImageType> for ImageExtent {
    fn to_erupt(self) -> vk1_0::ImageType {
        match self {
            ImageExtent::D1 { .. } => vk1_0::ImageType::_1D,
            ImageExtent::D2 { .. } => vk1_0::ImageType::_2D,
            ImageExtent::D3 { .. } => vk1_0::ImageType::_3D,
        }
    }
}

impl ToErupt<vk1_0::SampleCountFlagBits> for Samples {
    fn to_erupt(self) -> vk1_0::SampleCountFlagBits {
        match self {
            Samples::Samples1 => vk1_0::SampleCountFlagBits::_1,
            Samples::Samples2 => vk1_0::SampleCountFlagBits::_2,
            Samples::Samples4 => vk1_0::SampleCountFlagBits::_4,
            Samples::Samples8 => vk1_0::SampleCountFlagBits::_8,
            Samples::Samples16 => vk1_0::SampleCountFlagBits::_16,
            Samples::Samples32 => vk1_0::SampleCountFlagBits::_32,
            Samples::Samples64 => vk1_0::SampleCountFlagBits::_64,
        }
    }
}

// pub(crate) fn memory_usage_to_tvma(
//     usage: MemoryUsage,
// ) -> tvma::UsageFlags {
//     tvma::UsageFlags::from_bits_truncate(usage.bits())
// }

pub(crate) fn buffer_memory_usage_to_gpu_alloc(
    buffer_usage: BufferUsage,
    memory_usage: Option<MemoryUsage>,
) -> gpu_alloc::UsageFlags {
    use gpu_alloc::UsageFlags;

    let mut result = gpu_alloc::UsageFlags::empty();

    if buffer_usage.contains(BufferUsage::TRANSIENT) {
        result |= UsageFlags::TRANSIENT;
    }
    if buffer_usage.contains(BufferUsage::DEVICE_ADDRESS) {
        result |= UsageFlags::DEVICE_ADDRESS;
    }
    if let Some(memory_usage) = memory_usage {
        result |= UsageFlags::HOST_ACCESS;
        if memory_usage.contains(MemoryUsage::UPLOAD) {
            result |= UsageFlags::UPLOAD;
        }
        if memory_usage.contains(MemoryUsage::DOWNLOAD) {
            result |= UsageFlags::DOWNLOAD;
        }
        if memory_usage.contains(MemoryUsage::FAST_DEVICE_ACCESS) {
            result |= UsageFlags::FAST_DEVICE_ACCESS;
        }
    }
    result
}

pub(crate) fn image_memory_usage_to_gpu_alloc(image_usage: ImageUsage) -> gpu_alloc::UsageFlags {
    use gpu_alloc::UsageFlags;

    let mut result = gpu_alloc::UsageFlags::empty();

    if image_usage.contains(ImageUsage::TRANSIENT) {
        result |= UsageFlags::TRANSIENT;
    }
    result
}

impl ToErupt<vkacc::AccelerationStructureTypeKHR> for AccelerationStructureLevel {
    fn to_erupt(self) -> vkacc::AccelerationStructureTypeKHR {
        match self {
            AccelerationStructureLevel::Bottom => {
                vkacc::AccelerationStructureTypeKHR::BOTTOM_LEVEL_KHR
            }
            AccelerationStructureLevel::Top => vkacc::AccelerationStructureTypeKHR::TOP_LEVEL_KHR,
        }
    }
}

impl ToErupt<vkacc::BuildAccelerationStructureFlagsKHR> for AccelerationStructureBuildFlags {
    fn to_erupt(self) -> vkacc::BuildAccelerationStructureFlagsKHR {
        vkacc::BuildAccelerationStructureFlagsKHR::from_bits(self.bits()).unwrap()
    }
}

impl ToErupt<vk1_0::IndexType> for IndexType {
    fn to_erupt(self) -> vk1_0::IndexType {
        match self {
            IndexType::U16 => vk1_0::IndexType::UINT16,
            IndexType::U32 => vk1_0::IndexType::UINT32,
        }
    }
}

impl ToErupt<vkacc::DeviceOrHostAddressConstKHR> for DeviceAddress {
    fn to_erupt(self) -> vkacc::DeviceOrHostAddressConstKHR {
        vkacc::DeviceOrHostAddressConstKHR {
            device_address: self.0.get(),
        }
    }
}

impl ToErupt<vkacc::DeviceOrHostAddressKHR> for DeviceAddress {
    fn to_erupt(self) -> vkacc::DeviceOrHostAddressKHR {
        vkacc::DeviceOrHostAddressKHR {
            device_address: self.0.get(),
        }
    }
}

impl ToErupt<vkacc::GeometryFlagsKHR> for GeometryFlags {
    fn to_erupt(self) -> vkacc::GeometryFlagsKHR {
        let mut result = vkacc::GeometryFlagsKHR::empty();

        if self.contains(GeometryFlags::OPAQUE) {
            result |= vkacc::GeometryFlagsKHR::OPAQUE_KHR
        }

        if self.contains(GeometryFlags::NO_DUPLICATE_ANY_HIT_INVOCATION) {
            result |= vkacc::GeometryFlagsKHR::NO_DUPLICATE_ANY_HIT_INVOCATION_KHR
        }

        result
    }
}

impl FromErupt<u64> for Option<DeviceAddress> {
    fn from_erupt(value: u64) -> Self {
        NonZeroU64::new(value).map(DeviceAddress)
    }
}

impl ToErupt<vk1_0::DescriptorType> for DescriptorType {
    fn to_erupt(self) -> vk1_0::DescriptorType {
        match self {
            Self::Sampler => vk1_0::DescriptorType::SAMPLER,
            Self::CombinedImageSampler => vk1_0::DescriptorType::COMBINED_IMAGE_SAMPLER,
            Self::SampledImage => vk1_0::DescriptorType::SAMPLED_IMAGE,
            Self::StorageImage => vk1_0::DescriptorType::STORAGE_IMAGE,
            // Self::UniformTexelBuffer => vk1_0::DescriptorType::UNIFORM_TEXEL_BUFFER,
            // Self::StorageTexelBuffer => vk1_0::DescriptorType::STORAGE_TEXEL_BUFFER,
            Self::UniformBuffer => vk1_0::DescriptorType::UNIFORM_BUFFER,
            Self::StorageBuffer => vk1_0::DescriptorType::STORAGE_BUFFER,
            Self::UniformBufferDynamic => vk1_0::DescriptorType::UNIFORM_BUFFER_DYNAMIC,
            Self::StorageBufferDynamic => vk1_0::DescriptorType::STORAGE_BUFFER_DYNAMIC,
            Self::InputAttachment => vk1_0::DescriptorType::INPUT_ATTACHMENT,
            Self::AccelerationStructure => vk1_0::DescriptorType::ACCELERATION_STRUCTURE_KHR,
        }
    }
}

impl ToErupt<vk1_0::BorderColor> for BorderColor {
    fn to_erupt(self) -> vk1_0::BorderColor {
        match self {
            Self::FloatTransparentBlack => vk1_0::BorderColor::FLOAT_TRANSPARENT_BLACK,
            Self::IntTransparentBlack => vk1_0::BorderColor::INT_TRANSPARENT_BLACK,
            Self::FloatOpaqueBlack => vk1_0::BorderColor::FLOAT_OPAQUE_BLACK,
            Self::IntOpaqueBlack => vk1_0::BorderColor::INT_OPAQUE_BLACK,
            Self::FloatOpaqueWhite => vk1_0::BorderColor::FLOAT_OPAQUE_WHITE,
            Self::IntOpaqueWhite => vk1_0::BorderColor::INT_OPAQUE_WHITE,
        }
    }
}

impl ToErupt<vk1_0::Filter> for Filter {
    fn to_erupt(self) -> vk1_0::Filter {
        match self {
            Self::Nearest => vk1_0::Filter::NEAREST,
            Self::Linear => vk1_0::Filter::LINEAR,
            // Self::Cubic => vk1_0::Filter::CUBIC_EXT,
        }
    }
}

impl ToErupt<vk1_0::SamplerMipmapMode> for MipmapMode {
    fn to_erupt(self) -> vk1_0::SamplerMipmapMode {
        match self {
            Self::Nearest => vk1_0::SamplerMipmapMode::NEAREST,
            Self::Linear => vk1_0::SamplerMipmapMode::LINEAR,
        }
    }
}

impl ToErupt<vk1_0::SamplerAddressMode> for SamplerAddressMode {
    fn to_erupt(self) -> vk1_0::SamplerAddressMode {
        match self {
            Self::Repeat => vk1_0::SamplerAddressMode::REPEAT,
            Self::MirroredRepeat => vk1_0::SamplerAddressMode::MIRRORED_REPEAT,
            Self::ClampToEdge => vk1_0::SamplerAddressMode::CLAMP_TO_EDGE,
            Self::ClampToBorder => vk1_0::SamplerAddressMode::CLAMP_TO_BORDER,
            Self::MirrorClampToEdge => vk1_0::SamplerAddressMode::MIRROR_CLAMP_TO_EDGE,
        }
    }
}

impl ToErupt<vk1_0::ImageSubresource> for Subresource {
    fn to_erupt(self) -> vk1_0::ImageSubresource {
        vk1_0::ImageSubresource {
            aspect_mask: self.aspect.to_erupt(),
            mip_level: self.level,
            array_layer: self.layer,
        }
    }
}

impl ToErupt<vk1_0::ImageSubresourceLayers> for SubresourceLayers {
    fn to_erupt(self) -> vk1_0::ImageSubresourceLayers {
        vk1_0::ImageSubresourceLayers {
            aspect_mask: self.aspect.to_erupt(),
            mip_level: self.level,
            base_array_layer: self.first_layer,
            layer_count: self.layer_count,
        }
    }
}

impl ToErupt<vk1_0::ImageSubresourceRange> for SubresourceRange {
    fn to_erupt(self) -> vk1_0::ImageSubresourceRange {
        vk1_0::ImageSubresourceRange {
            aspect_mask: self.aspect.to_erupt(),
            base_mip_level: self.first_level,
            level_count: self.level_count,
            base_array_layer: self.first_layer,
            layer_count: self.layer_count,
        }
    }
}

impl ToErupt<vk1_0::ImageCopy> for ImageCopy {
    fn to_erupt(self) -> vk1_0::ImageCopy {
        vk1_0::ImageCopy {
            src_subresource: self.src_subresource.to_erupt(),
            src_offset: self.src_offset.to_erupt(),
            dst_subresource: self.dst_subresource.to_erupt(),
            dst_offset: self.dst_offset.to_erupt(),
            extent: self.extent.to_erupt(),
        }
    }
}

impl ToErupt<vk1_0::BufferCopy> for BufferCopy {
    fn to_erupt(self) -> vk1_0::BufferCopy {
        vk1_0::BufferCopy {
            src_offset: self.src_offset,
            dst_offset: self.dst_offset,
            size: self.size,
        }
    }
}

impl ToErupt<vk1_0::BufferImageCopy> for BufferImageCopy {
    fn to_erupt(self) -> vk1_0::BufferImageCopy {
        vk1_0::BufferImageCopy {
            buffer_offset: self.buffer_offset,
            buffer_row_length: self.buffer_row_length,
            buffer_image_height: self.buffer_image_height,
            image_subresource: self.image_subresource.to_erupt(),
            image_offset: self.image_offset.to_erupt(),
            image_extent: self.image_extent.to_erupt(),
        }
    }
}

impl ToErupt<vk1_0::ImageBlit> for ImageBlit {
    fn to_erupt(self) -> vk1_0::ImageBlit {
        vk1_0::ImageBlit {
            src_subresource: self.src_subresource.to_erupt(),
            src_offsets: [
                self.src_offsets[0].to_erupt(),
                self.src_offsets[1].to_erupt(),
            ],
            dst_subresource: self.dst_subresource.to_erupt(),
            dst_offsets: [
                self.dst_offsets[0].to_erupt(),
                self.dst_offsets[1].to_erupt(),
            ],
        }
    }
}

impl ToErupt<vk1_2::DescriptorBindingFlags> for DescriptorBindingFlags {
    fn to_erupt(self) -> vk1_2::DescriptorBindingFlags {
        let mut result = vk1_2::DescriptorBindingFlags::empty();

        if self.contains(DescriptorBindingFlags::UPDATE_AFTER_BIND) {
            result |= vk1_2::DescriptorBindingFlags::UPDATE_AFTER_BIND
        }

        if self.contains(DescriptorBindingFlags::UPDATE_UNUSED_WHILE_PENDING) {
            result |= vk1_2::DescriptorBindingFlags::UPDATE_UNUSED_WHILE_PENDING
        }

        if self.contains(DescriptorBindingFlags::PARTIALLY_BOUND) {
            result |= vk1_2::DescriptorBindingFlags::PARTIALLY_BOUND
        }

        if self.contains(DescriptorBindingFlags::VARIABLE_DESCRIPTOR_COUNT) {
            result |= vk1_2::DescriptorBindingFlags::VARIABLE_DESCRIPTOR_COUNT
        }

        result
    }
}

impl ToErupt<vk1_0::DescriptorSetLayoutCreateFlags> for DescriptorSetLayoutFlags {
    fn to_erupt(self) -> vk1_0::DescriptorSetLayoutCreateFlags {
        let mut result = vk1_0::DescriptorSetLayoutCreateFlags::empty();

        if self.contains(DescriptorSetLayoutFlags::UPDATE_AFTER_BIND_POOL) {
            result |= vk1_0::DescriptorSetLayoutCreateFlags::UPDATE_AFTER_BIND_POOL
        }

        if self.contains(DescriptorSetLayoutFlags::PUSH_DESCRIPTOR) {
            result |= vk1_0::DescriptorSetLayoutCreateFlags::PUSH_DESCRIPTOR_KHR
        }

        result
    }
}

impl ToErupt<vk1_0::AccessFlags> for AccessFlags {
    fn to_erupt(self) -> vk1_0::AccessFlags {
        let mut result = vk1_0::AccessFlags::empty();

        if self.contains(Self::INDIRECT_COMMAND_READ) {
            result |= vk1_0::AccessFlags::INDIRECT_COMMAND_READ;
        }
        if self.contains(Self::INDEX_READ) {
            result |= vk1_0::AccessFlags::INDEX_READ;
        }
        if self.contains(Self::VERTEX_ATTRIBUTE_READ) {
            result |= vk1_0::AccessFlags::VERTEX_ATTRIBUTE_READ;
        }
        if self.contains(Self::UNIFORM_READ) {
            result |= vk1_0::AccessFlags::UNIFORM_READ;
        }
        if self.contains(Self::INPUT_ATTACHMENT_READ) {
            result |= vk1_0::AccessFlags::INPUT_ATTACHMENT_READ;
        }
        if self.contains(Self::SHADER_READ) {
            result |= vk1_0::AccessFlags::SHADER_READ;
        }
        if self.contains(Self::SHADER_WRITE) {
            result |= vk1_0::AccessFlags::SHADER_WRITE;
        }
        if self.contains(Self::COLOR_ATTACHMENT_READ) {
            result |= vk1_0::AccessFlags::COLOR_ATTACHMENT_READ;
        }
        if self.contains(Self::COLOR_ATTACHMENT_WRITE) {
            result |= vk1_0::AccessFlags::COLOR_ATTACHMENT_WRITE;
        }
        if self.contains(Self::DEPTH_STENCIL_ATTACHMENT_READ) {
            result |= vk1_0::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ;
        }
        if self.contains(Self::DEPTH_STENCIL_ATTACHMENT_WRITE) {
            result |= vk1_0::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE;
        }
        if self.contains(Self::TRANSFER_READ) {
            result |= vk1_0::AccessFlags::TRANSFER_READ;
        }
        if self.contains(Self::TRANSFER_WRITE) {
            result |= vk1_0::AccessFlags::TRANSFER_WRITE;
        }
        if self.contains(Self::HOST_READ) {
            result |= vk1_0::AccessFlags::HOST_READ;
        }
        if self.contains(Self::HOST_WRITE) {
            result |= vk1_0::AccessFlags::HOST_WRITE;
        }
        if self.contains(Self::MEMORY_READ) {
            result |= vk1_0::AccessFlags::MEMORY_READ;
        }
        if self.contains(Self::MEMORY_WRITE) {
            result |= vk1_0::AccessFlags::MEMORY_WRITE;
        }
        // if self.contains(Self::TRANSFORM_FEEDBACK_WRITE) {
        //     result |= vk1_0::AccessFlags::TRANSFORM_FEEDBACK_WRITE_EXT;
        // }
        // if self.contains(Self::TRANSFORM_FEEDBACK_COUNTER_READ) {
        //     result |= vk1_0::AccessFlags::TRANSFORM_FEEDBACK_COUNTER_READ_EXT;
        // }
        // if self.contains(Self::TRANSFORM_FEEDBACK_COUNTER_WRITE) {
        //     result |= vk1_0::AccessFlags::TRANSFORM_FEEDBACK_COUNTER_WRITE_EXT;
        // }
        if self.contains(Self::CONDITIONAL_RENDERING_READ) {
            result |= vk1_0::AccessFlags::CONDITIONAL_RENDERING_READ_EXT;
        }
        if self.contains(Self::COLOR_ATTACHMENT_READ_NONCOHERENT) {
            result |= vk1_0::AccessFlags::COLOR_ATTACHMENT_READ_NONCOHERENT_EXT;
        }
        if self.contains(Self::ACCELERATION_STRUCTURE_READ) {
            result |= vk1_0::AccessFlags::ACCELERATION_STRUCTURE_READ_KHR;
        }
        if self.contains(Self::ACCELERATION_STRUCTURE_WRITE) {
            result |= vk1_0::AccessFlags::ACCELERATION_STRUCTURE_WRITE_KHR;
        }
        if self.contains(Self::FRAGMENT_DENSITY_MAP_READ) {
            result |= vk1_0::AccessFlags::FRAGMENT_DENSITY_MAP_READ_EXT;
        }
        if self.contains(Self::FRAGMENT_SHADING_RATE_ATTACHMENT_READ) {
            result |= vk1_0::AccessFlags::FRAGMENT_SHADING_RATE_ATTACHMENT_READ_KHR;
        }

        result
    }
}

impl FromErupt<CompositeAlphaFlagsKHR> for CompositeAlphaFlags {
    fn from_erupt(value: CompositeAlphaFlagsKHR) -> Self {
        let mut result = CompositeAlphaFlags::empty();
        if value.contains(CompositeAlphaFlagsKHR::OPAQUE_KHR) {
            result |= CompositeAlphaFlags::OPAQUE;
        }
        if value.contains(CompositeAlphaFlagsKHR::PRE_MULTIPLIED_KHR) {
            result |= CompositeAlphaFlags::PRE_MULTIPLIED;
        }
        if value.contains(CompositeAlphaFlagsKHR::POST_MULTIPLIED_KHR) {
            result |= CompositeAlphaFlags::POST_MULTIPLIED;
        }
        if value.contains(CompositeAlphaFlagsKHR::INHERIT_KHR) {
            result |= CompositeAlphaFlags::INHERIT;
        }
        result
    }
}

impl ToErupt<CompositeAlphaFlagsKHR> for CompositeAlphaFlags {
    fn to_erupt(self) -> CompositeAlphaFlagsKHR {
        let mut result = CompositeAlphaFlagsKHR::empty();
        if self.contains(CompositeAlphaFlags::OPAQUE) {
            result |= CompositeAlphaFlagsKHR::OPAQUE_KHR;
        }
        if self.contains(CompositeAlphaFlags::PRE_MULTIPLIED) {
            result |= CompositeAlphaFlagsKHR::PRE_MULTIPLIED_KHR;
        }
        if self.contains(CompositeAlphaFlags::POST_MULTIPLIED) {
            result |= CompositeAlphaFlagsKHR::POST_MULTIPLIED_KHR;
        }
        if self.contains(CompositeAlphaFlags::INHERIT) {
            result |= CompositeAlphaFlagsKHR::INHERIT_KHR;
        }
        result
    }
}

impl FromErupt<SurfaceTransformFlagsKHR> for SurfaceTransformFlags {
    fn from_erupt(value: SurfaceTransformFlagsKHR) -> Self {
        let mut result = SurfaceTransformFlags::empty();
        if value.contains(SurfaceTransformFlagsKHR::IDENTITY_KHR) {
            result |= SurfaceTransformFlags::IDENTITY;
        }
        if value.contains(SurfaceTransformFlagsKHR::ROTATE_90_KHR) {
            result |= SurfaceTransformFlags::ROTATE_90;
        }
        if value.contains(SurfaceTransformFlagsKHR::ROTATE_180_KHR) {
            result |= SurfaceTransformFlags::ROTATE_180;
        }
        if value.contains(SurfaceTransformFlagsKHR::ROTATE_270_KHR) {
            result |= SurfaceTransformFlags::ROTATE_270;
        }
        if value.contains(SurfaceTransformFlagsKHR::HORIZONTAL_MIRROR_KHR) {
            result |= SurfaceTransformFlags::HORIZONTAL_MIRROR;
        }
        if value.contains(SurfaceTransformFlagsKHR::HORIZONTAL_MIRROR_ROTATE_90_KHR) {
            result |= SurfaceTransformFlags::HORIZONTAL_MIRROR_ROTATE_90;
        }
        if value.contains(SurfaceTransformFlagsKHR::HORIZONTAL_MIRROR_ROTATE_180_KHR) {
            result |= SurfaceTransformFlags::HORIZONTAL_MIRROR_ROTATE_180;
        }
        if value.contains(SurfaceTransformFlagsKHR::HORIZONTAL_MIRROR_ROTATE_270_KHR) {
            result |= SurfaceTransformFlags::HORIZONTAL_MIRROR_ROTATE_270;
        }
        if value.contains(SurfaceTransformFlagsKHR::INHERIT_KHR) {
            result |= SurfaceTransformFlags::INHERIT;
        }
        result
    }
}

impl ToErupt<SurfaceTransformFlagsKHR> for SurfaceTransformFlags {
    fn to_erupt(self) -> SurfaceTransformFlagsKHR {
        let mut result = SurfaceTransformFlagsKHR::empty();
        if self.contains(SurfaceTransformFlags::IDENTITY) {
            result |= SurfaceTransformFlagsKHR::IDENTITY_KHR;
        }
        if self.contains(SurfaceTransformFlags::ROTATE_90) {
            result |= SurfaceTransformFlagsKHR::ROTATE_90_KHR;
        }
        if self.contains(SurfaceTransformFlags::ROTATE_180) {
            result |= SurfaceTransformFlagsKHR::ROTATE_180_KHR;
        }
        if self.contains(SurfaceTransformFlags::ROTATE_270) {
            result |= SurfaceTransformFlagsKHR::ROTATE_270_KHR;
        }
        if self.contains(SurfaceTransformFlags::HORIZONTAL_MIRROR) {
            result |= SurfaceTransformFlagsKHR::HORIZONTAL_MIRROR_KHR;
        }
        if self.contains(SurfaceTransformFlags::HORIZONTAL_MIRROR_ROTATE_90) {
            result |= SurfaceTransformFlagsKHR::HORIZONTAL_MIRROR_ROTATE_90_KHR;
        }
        if self.contains(SurfaceTransformFlags::HORIZONTAL_MIRROR_ROTATE_180) {
            result |= SurfaceTransformFlagsKHR::HORIZONTAL_MIRROR_ROTATE_180_KHR;
        }
        if self.contains(SurfaceTransformFlags::HORIZONTAL_MIRROR_ROTATE_270) {
            result |= SurfaceTransformFlagsKHR::HORIZONTAL_MIRROR_ROTATE_270_KHR;
        }
        if self.contains(SurfaceTransformFlags::INHERIT) {
            result |= SurfaceTransformFlagsKHR::INHERIT_KHR;
        }
        result
    }
}
