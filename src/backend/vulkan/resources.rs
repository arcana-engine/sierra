use {
    super::{descriptor::DescriptorSizes, device::WeakDevice},
    crate::{
        accel::AccelerationStructureInfo,
        buffer::BufferInfo,
        descriptor::{DescriptorSetInfo, DescriptorSetLayoutInfo},
        framebuffer::FramebufferInfo,
        image::ImageInfo,
        memory::MemoryUsage,
        pipeline::{
            ComputePipelineInfo, GraphicsPipelineInfo, PipelineLayoutInfo, RayTracingPipelineInfo,
        },
        render_pass::RenderPassInfo,
        sampler::SamplerInfo,
        shader::ShaderModuleInfo,
        view::ImageViewInfo,
        DeviceAddress,
    },
    erupt::{extensions::khr_acceleration_structure as vkacc, vk1_0},
    gpu_alloc::MemoryBlock,
    std::{
        cell::UnsafeCell,
        fmt::{self, Debug},
        hash::{Hash, Hasher},
        num::NonZeroU64,
        ops::Deref,
        sync::Arc,
    },
};

struct BufferInner {
    info: BufferInfo,
    handle: vk1_0::Buffer,
    owner: WeakDevice,
    address: Option<DeviceAddress>,
    index: usize,
    memory_handle: vk1_0::DeviceMemory,
    memory_offset: u64,
    memory_size: u64,
    memory_block: UnsafeCell<MemoryBlock<vk1_0::DeviceMemory>>,
}

#[derive(Clone)]
#[repr(transparent)]
pub struct Buffer {
    inner: Arc<BufferInner>,
}

unsafe impl Send for Buffer {}
unsafe impl Sync for Buffer {}

impl PartialEq for Buffer {
    fn eq(&self, rhs: &Self) -> bool {
        std::ptr::eq(&*self.inner, &*rhs.inner)
    }
}

impl Eq for Buffer {}

impl Hash for Buffer {
    fn hash<H>(&self, hasher: &mut H)
    where
        H: Hasher,
    {
        std::ptr::hash(&*self.inner, hasher)
    }
}

impl Debug for Buffer {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        if fmt.alternate() {
            #[derive(Debug)]
            struct Memory {
                handle: vk1_0::DeviceMemory,
                offset: u64,
                size: u64,
            }

            fmt.debug_struct("Buffer")
                .field("info", &self.inner.info)
                .field("owner", &self.inner.owner)
                .field("handle", &self.inner.handle)
                .field("address", &self.inner.address)
                .field("index", &self.inner.index)
                .field(
                    "memory",
                    &Memory {
                        handle: self.inner.memory_handle,
                        offset: self.inner.memory_offset,
                        size: self.inner.memory_size,
                    },
                )
                .finish()
        } else {
            write!(fmt, "Buffer({:p})", self.inner.handle)
        }
    }
}

impl Buffer {
    pub fn info(&self) -> &BufferInfo {
        &self.inner.info
    }

    pub fn address(&self) -> Option<DeviceAddress> {
        self.inner.address
    }

    pub(super) fn is_owned_by(&self, owner: &impl PartialEq<WeakDevice>) -> bool {
        *owner == self.inner.owner
    }

    pub(super) fn handle(&self) -> vk1_0::Buffer {
        debug_assert!(!self.inner.handle.is_null());
        self.inner.handle
    }
}

pub struct MappableBuffer {
    buffer: Buffer,
    memory_usage: MemoryUsage,
}

impl From<MappableBuffer> for Buffer {
    fn from(buffer: MappableBuffer) -> Self {
        buffer.buffer
    }
}

impl PartialEq for MappableBuffer {
    fn eq(&self, rhs: &Self) -> bool {
        std::ptr::eq(self, rhs)
    }
}

impl Eq for MappableBuffer {}

impl Hash for MappableBuffer {
    fn hash<H>(&self, hasher: &mut H)
    where
        H: Hasher,
    {
        self.buffer.inner.handle.hash(hasher)
    }
}

impl Debug for MappableBuffer {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        if fmt.alternate() {
            #[derive(Debug)]
            struct Memory {
                handle: vk1_0::DeviceMemory,
                offset: u64,
                size: u64,
                usage: MemoryUsage,
            }

            fmt.debug_struct("Buffer")
                .field("info", &self.inner.info)
                .field("owner", &self.inner.owner)
                .field("handle", &self.inner.handle)
                .field("address", &self.inner.address)
                .field("index", &self.inner.index)
                .field(
                    "memory",
                    &Memory {
                        handle: self.inner.memory_handle,
                        offset: self.inner.memory_offset,
                        size: self.inner.memory_size,
                        usage: self.memory_usage,
                    },
                )
                .finish()
        } else {
            write!(fmt, "MappableBuffer({:p})", self.inner.handle)
        }
    }
}

impl Deref for MappableBuffer {
    type Target = Buffer;

    fn deref(&self) -> &Buffer {
        &self.buffer
    }
}

impl MappableBuffer {
    pub fn share(&self) -> Buffer {
        Buffer {
            inner: self.inner.clone(),
        }
    }

    pub(super) fn new(
        info: BufferInfo,
        owner: WeakDevice,
        handle: vk1_0::Buffer,
        address: Option<DeviceAddress>,
        index: usize,
        memory_block: MemoryBlock<vk1_0::DeviceMemory>,
        memory_usage: MemoryUsage,
    ) -> Self {
        MappableBuffer {
            buffer: Buffer {
                inner: Arc::new(BufferInner {
                    info,
                    owner,
                    handle,
                    address,
                    memory_handle: *memory_block.memory(),
                    memory_offset: memory_block.offset(),
                    memory_size: memory_block.size(),
                    memory_block: UnsafeCell::new(memory_block),
                    index,
                }),
            },
            memory_usage,
        }
    }

    /// # Safety
    ///
    /// MemoryBlock must not be replaced
    pub(super) unsafe fn memory_block(&mut self) -> &mut MemoryBlock<vk1_0::DeviceMemory> {
        // exclusive access
        &mut *self.inner.memory_block.get()
    }
}

#[derive(Clone)]
enum ImageFlavor {
    DeviceImage {
        memory_block: Arc<MemoryBlock<vk1_0::DeviceMemory>>,
        index: usize,
    },
    SwapchainImage {
        uid: NonZeroU64,
    },
}

impl ImageFlavor {
    fn uid(&self) -> u64 {
        match *self {
            ImageFlavor::SwapchainImage { uid } => uid.get(),
            _ => 0,
        }
    }
}

#[derive(Clone)]
pub struct Image {
    info: ImageInfo,
    handle: vk1_0::Image,
    owner: WeakDevice,
    flavor: ImageFlavor,
}

impl PartialEq for Image {
    fn eq(&self, rhs: &Self) -> bool {
        (self.handle, self.flavor.uid()) == (rhs.handle, rhs.flavor.uid())
    }
}

impl Eq for Image {}

impl Hash for Image {
    fn hash<H>(&self, hasher: &mut H)
    where
        H: Hasher,
    {
        (self.handle, self.flavor.uid()).hash(hasher)
    }
}

impl Debug for Image {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        if fmt.alternate() {
            let mut fmt = fmt.debug_struct("Image");
            fmt.field("info", &self.info)
                .field("owner", &self.owner)
                .field("handle", &self.handle);

            match &self.flavor {
                ImageFlavor::DeviceImage {
                    memory_block,
                    index,
                } => {
                    fmt.field("memory_block", &**memory_block)
                        .field("index", index);
                }
                _ => {}
            }

            fmt.finish()
        } else {
            write!(fmt, "Image({:p})", self.handle)
        }
    }
}

impl Image {
    pub fn info(&self) -> &ImageInfo {
        &self.info
    }

    pub(super) fn new(
        info: ImageInfo,
        owner: WeakDevice,
        handle: vk1_0::Image,
        memory_block: MemoryBlock<vk1_0::DeviceMemory>,
        index: usize,
    ) -> Self {
        Image {
            info,
            owner,
            handle,
            flavor: ImageFlavor::DeviceImage {
                memory_block: Arc::new(memory_block),
                index,
            },
        }
    }

    pub(super) fn new_swapchain(
        info: ImageInfo,
        owner: WeakDevice,
        handle: vk1_0::Image,
        uid: NonZeroU64,
    ) -> Self {
        Image {
            info,
            owner,
            handle,
            flavor: ImageFlavor::SwapchainImage { uid },
        }
    }

    pub(super) fn is_owned_by(&self, owner: &impl PartialEq<WeakDevice>) -> bool {
        *owner == self.owner
    }

    pub(super) fn handle(&self) -> vk1_0::Image {
        debug_assert!(!self.handle.is_null());
        self.handle
    }

    // pub(super) fn swapchain_acq(&self) {
    //     match &self.flavor {
    //         ImageFlavor::SwapchainImage { state } => {
    //             state.fetch_add(1, Relaxed);
    //         }
    //         _ => unreachable!(),
    //     }
    // }

    // pub(super) fn swapchain_rel(&self) {
    //     match &self.flavor {
    //         ImageFlavor::SwapchainImage { state } => {
    //             state.fetch_sub(1, Relaxed);
    //         }
    //         _ => unreachable!(),
    //     }
    // }

    // fn retired(&self) -> bool {
    //     if let ImageFlavor::SwapchainImage { state } = &self.flavor {
    //         state.load(Relaxed) == 0
    //     } else {
    //         false
    //     }
    // }
}

#[derive(Clone)]
pub struct ImageView {
    info: ImageViewInfo,
    handle: vk1_0::ImageView,
    owner: WeakDevice,
    index: usize,
}

impl Debug for ImageView {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        if fmt.alternate() {
            fmt.debug_struct("ImageView")
                .field("info", &self.info)
                .field("handle", &self.handle)
                .field("owner", &self.owner)
                .finish()
        } else {
            write!(fmt, "ImageView({:p})", self.handle)
        }
    }
}

impl PartialEq for ImageView {
    fn eq(&self, rhs: &Self) -> bool {
        self.handle == rhs.handle
    }
}

impl Eq for ImageView {}

impl Hash for ImageView {
    fn hash<H>(&self, hasher: &mut H)
    where
        H: Hasher,
    {
        self.handle.hash(hasher)
    }
}

impl ImageView {
    pub fn info(&self) -> &ImageViewInfo {
        &self.info
    }

    pub(super) fn new(
        info: ImageViewInfo,
        owner: WeakDevice,
        handle: vk1_0::ImageView,
        index: usize,
    ) -> Self {
        ImageView {
            info,
            owner,
            handle,
            index,
        }
    }

    pub(super) fn is_owned_by(&self, owner: &impl PartialEq<WeakDevice>) -> bool {
        *owner == self.owner
    }

    pub(super) fn handle(&self) -> vk1_0::ImageView {
        debug_assert!(!self.handle.is_null());
        self.handle
    }
}

#[derive(Clone)]
pub struct Fence {
    handle: vk1_0::Fence,
    owner: WeakDevice,
    index: usize,
}

impl Debug for Fence {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        if fmt.alternate() {
            fmt.debug_struct("Fence")
                .field("handle", &self.handle)
                .field("owner", &self.owner)
                .finish()
        } else {
            write!(fmt, "Fence({:p})", self.handle)
        }
    }
}

impl PartialEq for Fence {
    fn eq(&self, rhs: &Self) -> bool {
        self.handle == rhs.handle
    }
}

impl Eq for Fence {}

impl Hash for Fence {
    fn hash<H>(&self, hasher: &mut H)
    where
        H: Hasher,
    {
        self.handle.hash(hasher)
    }
}

impl Fence {
    pub(super) fn new(owner: WeakDevice, handle: vk1_0::Fence, index: usize) -> Self {
        Fence {
            owner,
            handle,
            index,
        }
    }

    pub(super) fn is_owned_by(&self, owner: &impl PartialEq<WeakDevice>) -> bool {
        *owner == self.owner
    }

    pub(super) fn handle(&self) -> vk1_0::Fence {
        debug_assert!(!self.handle.is_null());
        self.handle
    }
}

#[derive(Clone)]
pub struct Semaphore {
    handle: vk1_0::Semaphore,
    owner: WeakDevice,
    index: usize,
}

impl Debug for Semaphore {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        if fmt.alternate() {
            fmt.debug_struct("Semaphore")
                .field("handle", &self.handle)
                .field("owner", &self.owner)
                .finish()
        } else {
            write!(fmt, "Semaphore({:p})", self.handle)
        }
    }
}

impl PartialEq for Semaphore {
    fn eq(&self, rhs: &Self) -> bool {
        self.handle == rhs.handle
    }
}

impl Eq for Semaphore {}

impl Hash for Semaphore {
    fn hash<H>(&self, hasher: &mut H)
    where
        H: Hasher,
    {
        self.handle.hash(hasher)
    }
}

impl Semaphore {
    pub(super) fn new(owner: WeakDevice, handle: vk1_0::Semaphore, index: usize) -> Self {
        Semaphore {
            owner,
            handle,
            index,
        }
    }

    pub(super) fn is_owned_by(&self, owner: &impl PartialEq<WeakDevice>) -> bool {
        *owner == self.owner
    }

    pub(super) fn handle(&self) -> vk1_0::Semaphore {
        debug_assert!(!self.handle.is_null());
        self.handle
    }
}

/// Render pass represents collection of attachments,
/// subpasses, and dependencies between subpasses,
/// and describes how they are used over the course of the subpasses.
///
/// This value is handle to a render pass resource.
#[derive(Clone)]
pub struct RenderPass {
    info: RenderPassInfo,
    handle: vk1_0::RenderPass,
    owner: WeakDevice,
    index: usize,
}

impl Debug for RenderPass {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        if fmt.alternate() {
            fmt.debug_struct("RenderPass")
                .field("handle", &self.handle)
                .field("owner", &self.owner)
                .finish()
        } else {
            write!(fmt, "RenderPass({:p})", self.handle)
        }
    }
}

impl PartialEq for RenderPass {
    fn eq(&self, rhs: &Self) -> bool {
        self.handle == rhs.handle
    }
}

impl Eq for RenderPass {}

impl Hash for RenderPass {
    fn hash<H>(&self, hasher: &mut H)
    where
        H: Hasher,
    {
        self.handle.hash(hasher)
    }
}

impl RenderPass {
    pub fn info(&self) -> &RenderPassInfo {
        &self.info
    }

    pub(super) fn new(
        info: RenderPassInfo,
        owner: WeakDevice,
        handle: vk1_0::RenderPass,
        index: usize,
    ) -> Self {
        RenderPass {
            info,
            owner,
            handle,
            index,
        }
    }

    pub(super) fn is_owned_by(&self, owner: &impl PartialEq<WeakDevice>) -> bool {
        *owner == self.owner
    }

    pub(super) fn handle(&self) -> vk1_0::RenderPass {
        debug_assert!(!self.handle.is_null());
        self.handle
    }
}

#[derive(Clone)]
pub struct Sampler {
    info: SamplerInfo,
    handle: vk1_0::Sampler,
    owner: WeakDevice,
    index: usize,
}

impl Debug for Sampler {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        if fmt.alternate() {
            fmt.debug_struct("Sampler")
                .field("handle", &self.handle)
                .field("owner", &self.owner)
                .finish()
        } else {
            write!(fmt, "Sampler({:p})", self.handle)
        }
    }
}

impl PartialEq for Sampler {
    fn eq(&self, rhs: &Self) -> bool {
        self.handle == rhs.handle
    }
}

impl Eq for Sampler {}

impl Hash for Sampler {
    fn hash<H>(&self, hasher: &mut H)
    where
        H: Hasher,
    {
        self.handle.hash(hasher)
    }
}

impl Sampler {
    pub fn info(&self) -> &SamplerInfo {
        &self.info
    }

    pub(super) fn new(
        info: SamplerInfo,
        owner: WeakDevice,
        handle: vk1_0::Sampler,
        index: usize,
    ) -> Self {
        Sampler {
            info,
            owner,
            handle,
            index,
        }
    }

    pub(super) fn is_owned_by(&self, owner: &impl PartialEq<WeakDevice>) -> bool {
        *owner == self.owner
    }

    pub(super) fn handle(&self) -> vk1_0::Sampler {
        debug_assert!(!self.handle.is_null());
        self.handle
    }
}

/// Framebuffer is a collection of attachments for render pass.
/// Images format and sample count should match attachment definitions.
/// All image views must be 2D with 1 mip level and 1 array level.
#[derive(Clone)]
pub struct Framebuffer {
    info: FramebufferInfo,
    handle: vk1_0::Framebuffer,
    owner: WeakDevice,
    index: usize,
}

impl Debug for Framebuffer {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        if fmt.alternate() {
            fmt.debug_struct("Framebuffer")
                .field("handle", &self.handle)
                .field("owner", &self.owner)
                .finish()
        } else {
            write!(fmt, "Framebuffer({:p})", self.handle)
        }
    }
}

impl PartialEq for Framebuffer {
    fn eq(&self, rhs: &Self) -> bool {
        self.handle == rhs.handle
    }
}

impl Eq for Framebuffer {}

impl Hash for Framebuffer {
    fn hash<H>(&self, hasher: &mut H)
    where
        H: Hasher,
    {
        self.handle.hash(hasher)
    }
}

impl Framebuffer {
    pub fn info(&self) -> &FramebufferInfo {
        &self.info
    }

    pub(super) fn new(
        info: FramebufferInfo,
        owner: WeakDevice,
        handle: vk1_0::Framebuffer,
        index: usize,
    ) -> Self {
        Framebuffer {
            info,
            owner,
            handle,
            index,
        }
    }

    pub(super) fn is_owned_by(&self, owner: &impl PartialEq<WeakDevice>) -> bool {
        *owner == self.owner
    }

    pub(super) fn handle(&self) -> vk1_0::Framebuffer {
        debug_assert!(!self.handle.is_null());
        self.handle
    }
}

/// Resource that describes layout for descriptor sets.
#[derive(Clone)]
pub struct ShaderModule {
    info: ShaderModuleInfo,
    handle: vk1_0::ShaderModule,
    owner: WeakDevice,
    index: usize,
}

impl Debug for ShaderModule {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        if fmt.alternate() {
            fmt.debug_struct("ShaderModule")
                .field("handle", &self.handle)
                .field("owner", &self.owner)
                .finish()
        } else {
            write!(fmt, "ShaderModule({:p})", self.handle)
        }
    }
}

impl PartialEq for ShaderModule {
    fn eq(&self, rhs: &Self) -> bool {
        self.handle == rhs.handle
    }
}

impl Eq for ShaderModule {}

impl Hash for ShaderModule {
    fn hash<H>(&self, hasher: &mut H)
    where
        H: Hasher,
    {
        self.handle.hash(hasher)
    }
}

impl ShaderModule {
    pub fn info(&self) -> &ShaderModuleInfo {
        &self.info
    }

    pub(super) fn new(
        info: ShaderModuleInfo,
        owner: WeakDevice,
        handle: vk1_0::ShaderModule,
        index: usize,
    ) -> Self {
        ShaderModule {
            info,
            owner,
            handle,
            index,
        }
    }

    pub(super) fn is_owned_by(&self, owner: &impl PartialEq<WeakDevice>) -> bool {
        *owner == self.owner
    }

    pub(super) fn handle(&self) -> vk1_0::ShaderModule {
        debug_assert!(!self.handle.is_null());
        self.handle
    }
}

/// Resource that describes layout for descriptor sets.
#[derive(Clone)]
pub struct DescriptorSetLayout {
    info: DescriptorSetLayoutInfo,
    handle: vk1_0::DescriptorSetLayout,
    owner: WeakDevice,
    sizes: DescriptorSizes,
    index: usize,
}

impl Debug for DescriptorSetLayout {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        if fmt.alternate() {
            fmt.debug_struct("DescriptorSetLayout")
                .field("handle", &self.handle)
                .field("owner", &self.owner)
                .finish()
        } else {
            write!(fmt, "DescriptorSetLayout({:p})", self.handle)
        }
    }
}

impl PartialEq for DescriptorSetLayout {
    fn eq(&self, rhs: &Self) -> bool {
        self.handle == rhs.handle
    }
}

impl Eq for DescriptorSetLayout {}

impl Hash for DescriptorSetLayout {
    fn hash<H>(&self, hasher: &mut H)
    where
        H: Hasher,
    {
        self.handle.hash(hasher)
    }
}

impl DescriptorSetLayout {
    pub fn info(&self) -> &DescriptorSetLayoutInfo {
        &self.info
    }

    pub(super) fn new(
        info: DescriptorSetLayoutInfo,
        owner: WeakDevice,
        handle: vk1_0::DescriptorSetLayout,
        sizes: DescriptorSizes,
        index: usize,
    ) -> Self {
        DescriptorSetLayout {
            info,
            owner,
            handle,
            sizes,
            index,
        }
    }

    pub(super) fn is_owned_by(&self, owner: &impl PartialEq<WeakDevice>) -> bool {
        *owner == self.owner
    }

    pub(super) fn handle(&self) -> vk1_0::DescriptorSetLayout {
        debug_assert!(!self.handle.is_null());
        self.handle
    }

    pub(super) fn sizes(&self) -> &DescriptorSizes {
        &self.sizes
    }
}

/// Set of descriptors with specific layout.
#[derive(Clone)]
pub struct DescriptorSet {
    info: DescriptorSetInfo,
    handle: vk1_0::DescriptorSet,
    owner: WeakDevice,
    pool: vk1_0::DescriptorPool,
    pool_index: usize,
}

impl Debug for DescriptorSet {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        if fmt.alternate() {
            fmt.debug_struct("DescriptorSet")
                .field("handle", &self.handle)
                .field("owner", &self.owner)
                .field("pool", &self.pool)
                .finish()
        } else {
            write!(fmt, "DescriptorSet({:p})", self.handle)
        }
    }
}

impl PartialEq for DescriptorSet {
    fn eq(&self, rhs: &Self) -> bool {
        self.handle == rhs.handle
    }
}

impl Eq for DescriptorSet {}

impl Hash for DescriptorSet {
    fn hash<H>(&self, hasher: &mut H)
    where
        H: Hasher,
    {
        self.handle.hash(hasher)
    }
}

impl DescriptorSet {
    pub fn info(&self) -> &DescriptorSetInfo {
        &self.info
    }

    pub(super) fn new(
        info: DescriptorSetInfo,
        owner: WeakDevice,
        handle: vk1_0::DescriptorSet,
        pool: vk1_0::DescriptorPool,
        pool_index: usize,
    ) -> Self {
        DescriptorSet {
            info,
            owner,
            handle,
            pool,
            pool_index,
        }
    }

    pub(super) fn is_owned_by(&self, owner: &impl PartialEq<WeakDevice>) -> bool {
        *owner == self.owner
    }

    pub(super) fn handle(&self) -> vk1_0::DescriptorSet {
        debug_assert!(!self.handle.is_null());
        self.handle
    }
}

/// Resource that describes layout of a pipeline.
#[derive(Clone)]
pub struct PipelineLayout {
    info: PipelineLayoutInfo,
    handle: vk1_0::PipelineLayout,
    owner: WeakDevice,
    index: usize,
}

impl Debug for PipelineLayout {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        if fmt.alternate() {
            fmt.debug_struct("PipelineLayout")
                .field("handle", &self.handle)
                .field("owner", &self.owner)
                .finish()
        } else {
            write!(fmt, "PipelineLayout({:p})", self.handle)
        }
    }
}

impl PartialEq for PipelineLayout {
    fn eq(&self, rhs: &Self) -> bool {
        self.handle == rhs.handle
    }
}

impl Eq for PipelineLayout {}

impl Hash for PipelineLayout {
    fn hash<H>(&self, hasher: &mut H)
    where
        H: Hasher,
    {
        self.handle.hash(hasher)
    }
}

impl PipelineLayout {
    pub fn info(&self) -> &PipelineLayoutInfo {
        &self.info
    }

    pub(super) fn new(
        info: PipelineLayoutInfo,
        owner: WeakDevice,
        handle: vk1_0::PipelineLayout,
        index: usize,
    ) -> Self {
        PipelineLayout {
            info,
            owner,
            handle,
            index,
        }
    }

    pub(super) fn is_owned_by(&self, owner: &impl PartialEq<WeakDevice>) -> bool {
        *owner == self.owner
    }

    pub(super) fn handle(&self) -> vk1_0::PipelineLayout {
        debug_assert!(!self.handle.is_null());
        self.handle
    }
}

/// Resource that describes whole compute pipeline state.
#[derive(Clone)]
pub struct ComputePipeline {
    info: ComputePipelineInfo,
    handle: vk1_0::Pipeline,
    owner: WeakDevice,
    index: usize,
}

impl Debug for ComputePipeline {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        if fmt.alternate() {
            fmt.debug_struct("ComputePipeline")
                .field("handle", &self.handle)
                .field("owner", &self.owner)
                .finish()
        } else {
            write!(fmt, "ComputePipeline({:p})", self.handle)
        }
    }
}

impl PartialEq for ComputePipeline {
    fn eq(&self, rhs: &Self) -> bool {
        self.handle == rhs.handle
    }
}

impl Eq for ComputePipeline {}

impl Hash for ComputePipeline {
    fn hash<H>(&self, hasher: &mut H)
    where
        H: Hasher,
    {
        self.handle.hash(hasher)
    }
}

impl ComputePipeline {
    pub fn info(&self) -> &ComputePipelineInfo {
        &self.info
    }

    pub(super) fn new(
        info: ComputePipelineInfo,
        owner: WeakDevice,
        handle: vk1_0::Pipeline,
        index: usize,
    ) -> Self {
        ComputePipeline {
            info,
            owner,
            handle,
            index,
        }
    }

    pub(super) fn is_owned_by(&self, owner: &impl PartialEq<WeakDevice>) -> bool {
        *owner == self.owner
    }

    pub(super) fn handle(&self) -> vk1_0::Pipeline {
        debug_assert!(!self.handle.is_null());
        self.handle
    }
}

/// Resource that describes whole graphics pipeline state.
#[derive(Clone)]
pub struct GraphicsPipeline {
    info: GraphicsPipelineInfo,
    handle: vk1_0::Pipeline,
    owner: WeakDevice,
    index: usize,
}

impl Debug for GraphicsPipeline {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        if fmt.alternate() {
            fmt.debug_struct("GraphicsPipeline")
                .field("handle", &self.handle)
                .field("owner", &self.owner)
                .finish()
        } else {
            write!(fmt, "GraphicsPipeline({:p})", self.handle)
        }
    }
}

impl PartialEq for GraphicsPipeline {
    fn eq(&self, rhs: &Self) -> bool {
        self.handle == rhs.handle
    }
}

impl Eq for GraphicsPipeline {}

impl Hash for GraphicsPipeline {
    fn hash<H>(&self, hasher: &mut H)
    where
        H: Hasher,
    {
        self.handle.hash(hasher)
    }
}

impl GraphicsPipeline {
    pub fn info(&self) -> &GraphicsPipelineInfo {
        &self.info
    }

    pub(super) fn new(
        info: GraphicsPipelineInfo,
        owner: WeakDevice,
        handle: vk1_0::Pipeline,
        index: usize,
    ) -> Self {
        GraphicsPipeline {
            info,
            owner,
            handle,
            index,
        }
    }

    pub(super) fn is_owned_by(&self, owner: &impl PartialEq<WeakDevice>) -> bool {
        *owner == self.owner
    }

    pub(super) fn handle(&self) -> vk1_0::Pipeline {
        debug_assert!(!self.handle.is_null());
        self.handle
    }
}

/// Bottom-level acceleration structure.
#[derive(Clone)]
pub struct AccelerationStructure {
    info: AccelerationStructureInfo,
    handle: vkacc::AccelerationStructureKHR,
    owner: WeakDevice,
    address: DeviceAddress,
    index: usize,
}

impl Debug for AccelerationStructure {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        if fmt.alternate() {
            fmt.debug_struct("AccelerationStructure")
                .field("handle", &self.handle)
                .field("owner", &self.owner)
                .field("address", &self.address)
                .finish()
        } else {
            write!(fmt, "AccelerationStructure({:p})", self.handle)
        }
    }
}

impl PartialEq for AccelerationStructure {
    fn eq(&self, rhs: &Self) -> bool {
        self.handle == rhs.handle
    }
}

impl Eq for AccelerationStructure {}

impl Hash for AccelerationStructure {
    fn hash<H>(&self, hasher: &mut H)
    where
        H: Hasher,
    {
        self.handle.hash(hasher)
    }
}

impl AccelerationStructure {
    pub fn info(&self) -> &AccelerationStructureInfo {
        &self.info
    }

    pub fn address(&self) -> DeviceAddress {
        self.address
    }

    pub(super) fn new(
        info: AccelerationStructureInfo,
        owner: WeakDevice,
        handle: vkacc::AccelerationStructureKHR,
        address: DeviceAddress,
        index: usize,
    ) -> Self {
        AccelerationStructure {
            info,
            owner,
            handle,
            address,
            index,
        }
    }

    pub(super) fn is_owned_by(&self, owner: &impl PartialEq<WeakDevice>) -> bool {
        *owner == self.owner
    }

    pub(super) fn handle(&self) -> vkacc::AccelerationStructureKHR {
        debug_assert!(!self.handle.is_null());
        self.handle
    }
}

/// Resource that describes whole ray-tracing pipeline state.
#[derive(Clone)]
pub struct RayTracingPipeline {
    info: RayTracingPipelineInfo,
    handle: vk1_0::Pipeline,
    owner: WeakDevice,
    group_handlers: Arc<[u8]>,
    index: usize,
}

impl Debug for RayTracingPipeline {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        if fmt.alternate() {
            fmt.debug_struct("RayTracingPipeline")
                .field("handle", &self.handle)
                .field("owner", &self.owner)
                .finish()
        } else {
            write!(fmt, "RayTracingPipeline({:p})", self.handle)
        }
    }
}

impl PartialEq for RayTracingPipeline {
    fn eq(&self, rhs: &Self) -> bool {
        self.handle == rhs.handle
    }
}

impl Eq for RayTracingPipeline {}

impl Hash for RayTracingPipeline {
    fn hash<H>(&self, hasher: &mut H)
    where
        H: Hasher,
    {
        self.handle.hash(hasher)
    }
}

impl RayTracingPipeline {
    pub fn info(&self) -> &RayTracingPipelineInfo {
        &self.info
    }

    pub(super) fn new(
        info: RayTracingPipelineInfo,
        owner: WeakDevice,
        handle: vk1_0::Pipeline,
        group_handlers: Arc<[u8]>,
        index: usize,
    ) -> Self {
        RayTracingPipeline {
            info,
            owner,
            handle,
            group_handlers,
            index,
        }
    }

    pub(super) fn is_owned_by(&self, owner: &impl PartialEq<WeakDevice>) -> bool {
        *owner == self.owner
    }

    pub(super) fn handle(&self) -> vk1_0::Pipeline {
        debug_assert!(!self.handle.is_null());
        self.handle
    }

    pub(super) fn group_handlers(&self) -> &[u8] {
        &*self.group_handlers
    }
}
