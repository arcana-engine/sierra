use {
    super::device::{Device, WeakDevice},
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
        queue::QueueId,
        render_pass::RenderPassInfo,
        sampler::SamplerInfo,
        shader::ShaderModuleInfo,
        view::ImageViewInfo,
        DeviceAddress,
    },
    erupt::{extensions::khr_acceleration_structure as vkacc, vk1_0},
    gpu_alloc::MemoryBlock,
    gpu_descriptor::DescriptorTotalCount,
    std::{
        cell::UnsafeCell,
        fmt::{self, Debug},
        hash::{Hash, Hasher},
        mem::ManuallyDrop,
        num::NonZeroU64,
        ops::Deref,
        sync::Arc,
    },
};

struct BufferInner {
    owner: WeakDevice,
    index: usize,
    memory_handle: vk1_0::DeviceMemory,
    memory_offset: u64,
    memory_size: u64,
    memory_block: UnsafeCell<ManuallyDrop<MemoryBlock<vk1_0::DeviceMemory>>>,
}

impl Drop for BufferInner {
    fn drop(&mut self) {
        resource_freed();

        if let Some(device) = self.owner.upgrade() {
            unsafe {
                let block = ManuallyDrop::take(self.memory_block.get_mut());
                device.destroy_buffer(self.index, block);
            }
        }
    }
}

/// Handle for GPU buffer object.
/// GPU buffer is an object representing contiguous array of bytes
/// accessible by GPU operations.
#[derive(Clone)]
pub struct Buffer {
    handle: vk1_0::Buffer,
    info: BufferInfo,
    address: Option<DeviceAddress>,
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
            #[allow(unused)]
            struct Memory {
                handle: vk1_0::DeviceMemory,
                offset: u64,
                size: u64,
            }

            fmt.debug_struct("Buffer")
                .field("info", &self.info)
                .field("owner", &self.inner.owner)
                .field("handle", &self.handle)
                .field("address", &self.address)
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
            write!(fmt, "Buffer({:p})", self.handle)
        }
    }
}

impl Buffer {
    pub fn info(&self) -> &BufferInfo {
        &self.info
    }

    pub fn address(&self) -> Option<DeviceAddress> {
        self.address
    }

    pub(super) fn is_owned_by(&self, owner: &impl PartialEq<WeakDevice>) -> bool {
        *owner == self.inner.owner
    }

    pub(super) fn handle(&self) -> vk1_0::Buffer {
        debug_assert!(!self.handle.is_null());
        self.handle
    }
}

/// Handle to GPU buffer object.
///
/// Variation of `Buffer` which is not shared
/// and thus can be mapped onto host memory.
///
/// Mapping of shared buffer is forbidden due to requirement
/// of GPU driver to map any memory object at most once.
/// This allows accessing mapped memory safely without causing data races.
///
/// If buffer sharing is required and mapping is not,
/// [`MappedBuffer`] can be converted into [`Buffer`] with no cost.
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
        self.buffer.handle.hash(hasher)
    }
}

impl Debug for MappableBuffer {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        if fmt.alternate() {
            #[derive(Debug)]
            #[allow(unused)]
            struct Memory {
                handle: vk1_0::DeviceMemory,
                offset: u64,
                size: u64,
                usage: MemoryUsage,
            }

            fmt.debug_struct("Buffer")
                .field("info", &self.info)
                .field("owner", &self.inner.owner)
                .field("handle", &self.handle)
                .field("address", &self.address)
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
            write!(fmt, "MappableBuffer({:p})", self.handle)
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
    pub fn share(self) -> Buffer {
        self.buffer
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
        resource_allocated();

        MappableBuffer {
            buffer: Buffer {
                handle,
                info,
                address,
                inner: Arc::new(BufferInner {
                    owner,
                    memory_handle: *memory_block.memory(),
                    memory_offset: memory_block.offset(),
                    memory_size: memory_block.size(),
                    memory_block: UnsafeCell::new(ManuallyDrop::new(memory_block)),
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

enum ImageFlavor {
    DeviceImage {
        memory_block: ManuallyDrop<MemoryBlock<vk1_0::DeviceMemory>>,
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

/// Handle to GPU image object.
///
/// Images represent a one, two or three dimensional arrays of texel.
#[derive(Clone)]
pub struct Image {
    handle: vk1_0::Image,
    info: ImageInfo,
    inner: Arc<ImageInner>,
}

struct ImageInner {
    owner: WeakDevice,
    flavor: ImageFlavor,
}

impl Drop for ImageInner {
    fn drop(&mut self) {
        resource_freed();

        if let ImageFlavor::DeviceImage {
            memory_block,
            index,
        } = &mut self.flavor
        {
            if let Some(device) = self.owner.upgrade() {
                unsafe {
                    let block = ManuallyDrop::take(memory_block);
                    device.destroy_image(*index, block);
                }
            }
        }
    }
}

impl PartialEq for Image {
    fn eq(&self, rhs: &Self) -> bool {
        (self.handle, self.inner.flavor.uid()) == (rhs.handle, rhs.inner.flavor.uid())
    }
}

impl Eq for Image {}

impl Hash for Image {
    fn hash<H>(&self, hasher: &mut H)
    where
        H: Hasher,
    {
        (self.handle, self.inner.flavor.uid()).hash(hasher)
    }
}

impl Debug for Image {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        if fmt.alternate() {
            let mut fmt = fmt.debug_struct("Image");
            fmt.field("info", &self.info)
                .field("owner", &self.inner.owner)
                .field("handle", &self.handle);

            if let ImageFlavor::DeviceImage {
                memory_block,
                index,
            } = &self.inner.flavor
            {
                fmt.field("memory_block", &**memory_block)
                    .field("index", index);
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
        resource_allocated();

        Image {
            info,
            handle,
            inner: Arc::new(ImageInner {
                owner,
                flavor: ImageFlavor::DeviceImage {
                    memory_block: ManuallyDrop::new(memory_block),
                    index,
                },
            }),
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
            handle,
            inner: Arc::new(ImageInner {
                owner,
                flavor: ImageFlavor::SwapchainImage { uid },
            }),
        }
    }

    pub(super) fn is_owned_by(&self, owner: &impl PartialEq<WeakDevice>) -> bool {
        *owner == self.inner.owner
    }

    pub(super) fn handle(&self) -> vk1_0::Image {
        debug_assert!(!self.handle.is_null());
        self.handle
    }

    /// Must be called only for retired swapchain.
    pub(super) fn try_dispose(mut self) -> Result<(), Self> {
        assert!(matches!(
            self.inner.flavor,
            ImageFlavor::SwapchainImage { .. }
        ));
        match Arc::try_unwrap(self.inner) {
            Ok(_) => Ok(()),
            Err(inner) => {
                self.inner = inner;
                Err(self)
            }
        }
    }
}

/// Handle to GPU image view object.
///
/// A slice view into an [`Image`].
/// [`ImageView`] can encompass whole [`Image`]
/// or only part of [`Image`]s layers, levels and aspects.
#[derive(Clone)]
pub struct ImageView {
    handle: vk1_0::ImageView,
    inner: Arc<ImageViewInner>,
}

struct ImageViewInner {
    info: ImageViewInfo,
    owner: WeakDevice,
    index: usize,
}

impl Drop for ImageViewInner {
    fn drop(&mut self) {
        resource_freed();

        if let Some(device) = self.owner.upgrade() {
            unsafe { device.destroy_image_view(self.index) }
        }
    }
}

impl Debug for ImageView {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        if fmt.alternate() {
            fmt.debug_struct("ImageView")
                .field("info", &self.inner.info)
                .field("handle", &self.handle)
                .field("owner", &self.inner.owner)
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
        &self.inner.info
    }

    pub(super) fn new(
        info: ImageViewInfo,
        owner: WeakDevice,
        handle: vk1_0::ImageView,
        index: usize,
    ) -> Self {
        resource_allocated();

        ImageView {
            handle,
            inner: Arc::new(ImageViewInner { info, owner, index }),
        }
    }

    pub(super) fn is_owned_by(&self, owner: &impl PartialEq<WeakDevice>) -> bool {
        *owner == self.inner.owner
    }

    pub(super) fn handle(&self) -> vk1_0::ImageView {
        debug_assert!(!self.handle.is_null());
        self.handle
    }
}

/// Handle to GPU fence object.
///
/// Fence is object used for coarse grained GPU-CPU synchronization.
/// It should be used to wait for GPU operations to finish before
/// mutating or destroying resources on CPU.
///
/// This includes overwriting mappable buffer content and updating
/// descriptor sets.
/// (Mappable images can be added in future).
///
/// Prefer using semaphores and pipeline barriers to synchronize
/// operations on GPU with each other.
pub struct Fence {
    handle: vk1_0::Fence,
    owner: WeakDevice,
    index: usize,
    state: FenceState,
}

impl Drop for Fence {
    fn drop(&mut self) {
        resource_freed();

        if let Some(device) = self.owner.upgrade() {
            if let FenceState::Armed { .. } = self.state {
                device.wait_fences(&mut [self], true);
            }
            unsafe { device.destroy_fence(self.index) }
        }
    }
}

#[derive(Clone, Copy)]
enum FenceState {
    /// Fence is not signalled and won't be signalled by any pending submissions.
    /// It must not be used in `Device::wait_for_fences` without timeout.
    UnSignalled,

    /// Fence is currently unsignalled and will be signalled by pending submission.
    /// Pending submission may be already signalled the fence object
    /// but checking through device is required.
    Armed { queue: QueueId, epoch: u64 },

    /// Fence is in signalled state.
    Signalled,
}

impl Debug for Fence {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        if fmt.alternate() {
            fmt.debug_struct("Fence")
                .field("handle", &self.handle)
                .field("owner", &self.owner)
                .field(
                    "state",
                    &match &self.state {
                        FenceState::UnSignalled => "unsignalled",
                        FenceState::Signalled => "signalled",
                        FenceState::Armed { .. } => "armed",
                    },
                )
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
        resource_allocated();

        Fence {
            owner,
            handle,
            index,
            state: FenceState::UnSignalled,
        }
    }

    pub(super) fn is_owned_by(&self, owner: &impl PartialEq<WeakDevice>) -> bool {
        *owner == self.owner
    }

    pub(super) fn handle(&self) -> vk1_0::Fence {
        debug_assert!(!self.handle.is_null());
        self.handle
    }

    /// Called when submitted to a queue for signal.
    pub(super) fn arm(&mut self, queue: QueueId, epoch: u64, device: &Device) {
        debug_assert!(self.is_owned_by(device));
        match &self.state {
            FenceState::UnSignalled => {
                self.state = FenceState::Armed { queue, epoch };
            }
            FenceState::Armed { .. } => {
                // Could be come signalled already.
                // User may be sure because they called device or queue wait idle method.
                if device.is_fence_signalled(self) {
                    self.state = FenceState::Armed { queue, epoch };
                } else {
                    panic!("Fence must not be resubmitted while associated submission is pending")
                }
            }
            FenceState::Signalled => {
                panic!("Fence must not be resubmitted before resetting")
            }
        }
    }

    /// Called when device knows fence is signalled.
    pub(super) fn signalled(&mut self) -> Option<(QueueId, u64)> {
        match self.state {
            FenceState::Signalled => None,
            FenceState::Armed { queue, epoch } => {
                self.state = FenceState::Signalled;
                Some((queue, epoch))
            }
            FenceState::UnSignalled => {
                panic!("Fence cannot become signalled before being submitted")
            }
        }
    }

    /// Called when device resets the fence.
    pub(super) fn reset(&mut self, device: &Device) {
        match &self.state {
            FenceState::Signalled | FenceState::UnSignalled => {
                self.state = FenceState::UnSignalled;
            }
            FenceState::Armed { .. } => {
                // Could be come signalled already.
                // User may be sure because they called device or queue wait idle method.
                if device.is_fence_signalled(self) {
                    self.state = FenceState::UnSignalled;
                } else {
                    panic!("Fence must not be reset while associated submission is pending")
                }
            }
        }
    }
}

/// Handle for GPU semaphore object.
///
/// Semaphores are used to synchronize operations on different GPU queues
/// as well as operations on GPU queue with presentation engine.
///
/// Avoid using semaphores to synchronize operations on the same queue.
pub struct Semaphore {
    handle: vk1_0::Semaphore,
    owner: WeakDevice,
    index: usize,
}

impl Drop for Semaphore {
    fn drop(&mut self) {
        resource_freed();

        // TODO: Check there's no pending signal operations.
        if let Some(device) = self.owner.upgrade() {
            unsafe { device.destroy_semaphore(self.index) }
        }
    }
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
        resource_allocated();

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

/// Handle to GPU render pass object.
///
/// Render pass defines collection of abstract attachments,
/// subpasses, and dependencies between subpasses,
/// and describes how attachments are used over the course of subpasses.
#[derive(Clone)]
pub struct RenderPass {
    handle: vk1_0::RenderPass,
    inner: Arc<RenderPassInner>,
}

struct RenderPassInner {
    info: RenderPassInfo,
    owner: WeakDevice,
    index: usize,
}

impl Drop for RenderPassInner {
    fn drop(&mut self) {
        resource_freed();

        if let Some(device) = self.owner.upgrade() {
            unsafe { device.destroy_render_pass(self.index) }
        }
    }
}

impl Debug for RenderPass {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        if fmt.alternate() {
            fmt.debug_struct("RenderPass")
                .field("handle", &self.handle)
                .field("owner", &self.inner.owner)
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
        &self.inner.info
    }

    pub(super) fn new(
        info: RenderPassInfo,
        owner: WeakDevice,
        handle: vk1_0::RenderPass,
        index: usize,
    ) -> Self {
        resource_allocated();

        RenderPass {
            handle,
            inner: Arc::new(RenderPassInner { info, owner, index }),
        }
    }

    pub(super) fn is_owned_by(&self, owner: &impl PartialEq<WeakDevice>) -> bool {
        *owner == self.inner.owner
    }

    pub(super) fn handle(&self) -> vk1_0::RenderPass {
        debug_assert!(!self.handle.is_null());
        self.handle
    }
}

#[derive(Clone)]
pub struct Sampler {
    handle: vk1_0::Sampler,
    info: SamplerInfo,
    inner: Arc<SamplerInner>,
}

struct SamplerInner {
    owner: WeakDevice,
    index: usize,
}

impl Drop for SamplerInner {
    fn drop(&mut self) {
        resource_freed();

        if let Some(device) = self.owner.upgrade() {
            unsafe { device.destroy_sampler(self.index) }
        }
    }
}

impl Debug for Sampler {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        if fmt.alternate() {
            fmt.debug_struct("Sampler")
                .field("handle", &self.handle)
                .field("owner", &self.inner.owner)
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
        resource_allocated();

        Sampler {
            info,
            handle,
            inner: Arc::new(SamplerInner { owner, index }),
        }
    }

    pub(super) fn is_owned_by(&self, owner: &impl PartialEq<WeakDevice>) -> bool {
        *owner == self.inner.owner
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
    handle: vk1_0::Framebuffer,
    inner: Arc<FramebufferInner>,
}

struct FramebufferInner {
    info: FramebufferInfo,
    owner: WeakDevice,
    index: usize,
}

impl Drop for FramebufferInner {
    fn drop(&mut self) {
        resource_freed();

        if let Some(device) = self.owner.upgrade() {
            unsafe { device.destroy_framebuffer(self.index) }
        }
    }
}

impl Debug for Framebuffer {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        if fmt.alternate() {
            fmt.debug_struct("Framebuffer")
                .field("handle", &self.handle)
                .field("owner", &self.inner.owner)
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
        &self.inner.info
    }

    pub(super) fn new(
        info: FramebufferInfo,
        owner: WeakDevice,
        handle: vk1_0::Framebuffer,
        index: usize,
    ) -> Self {
        resource_allocated();

        Framebuffer {
            handle,
            inner: Arc::new(FramebufferInner { info, owner, index }),
        }
    }

    pub(super) fn is_owned_by(&self, owner: &impl PartialEq<WeakDevice>) -> bool {
        *owner == self.inner.owner
    }

    pub(super) fn handle(&self) -> vk1_0::Framebuffer {
        debug_assert!(!self.handle.is_null());
        self.handle
    }
}

/// Handle fot GPU shader module object.
///
/// Shader module is pre-compiled shader program,
/// optionally with multiple entry points for different shaders.
///
/// Used for pipelines creation:
/// [`GraphicsPipeline`], [`ComputePipeline`] and [`RayTracingPipeline`].
#[derive(Clone)]
pub struct ShaderModule {
    handle: vk1_0::ShaderModule,
    inner: Arc<ShaderModuleInner>,
}

struct ShaderModuleInner {
    info: ShaderModuleInfo,
    owner: WeakDevice,
    index: usize,
}

impl Drop for ShaderModuleInner {
    fn drop(&mut self) {
        resource_freed();

        if let Some(device) = self.owner.upgrade() {
            unsafe { device.destroy_shader_module(self.index) }
        }
    }
}

impl Debug for ShaderModule {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        if fmt.alternate() {
            fmt.debug_struct("ShaderModule")
                .field("handle", &self.handle)
                .field("owner", &self.inner.owner)
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
        &self.inner.info
    }

    pub(super) fn new(
        info: ShaderModuleInfo,
        owner: WeakDevice,
        handle: vk1_0::ShaderModule,
        index: usize,
    ) -> Self {
        resource_allocated();

        ShaderModule {
            handle,
            inner: Arc::new(ShaderModuleInner { info, owner, index }),
        }
    }

    pub(super) fn is_owned_by(&self, owner: &impl PartialEq<WeakDevice>) -> bool {
        *owner == self.inner.owner
    }

    pub(super) fn handle(&self) -> vk1_0::ShaderModule {
        debug_assert!(!self.handle.is_null());
        self.handle
    }
}

/// Handle for GPU descriptor set layout object.
///
/// Describes descriptor bindings and their types.
/// Used for [`PipelineLayout`] creation and [`DescriptorSet`] allocation.
///
/// Descriptor set bound at index N must be allocated with same
/// descriptor set layout that was specified at index N for bound [`PipelineLayout`].
#[derive(Clone)]
pub struct DescriptorSetLayout {
    handle: vk1_0::DescriptorSetLayout,
    inner: Arc<DescriptorSetLayoutInner>,
}

struct DescriptorSetLayoutInner {
    info: DescriptorSetLayoutInfo,
    owner: WeakDevice,
    total_count: DescriptorTotalCount,
    index: usize,
}

impl Drop for DescriptorSetLayoutInner {
    fn drop(&mut self) {
        resource_freed();

        if let Some(device) = self.owner.upgrade() {
            unsafe {
                device.destroy_descriptor_set_layout(self.index);
            }
        }
    }
}

impl Debug for DescriptorSetLayout {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        if fmt.alternate() {
            fmt.debug_struct("DescriptorSetLayout")
                .field("handle", &self.handle)
                .field("owner", &self.inner.owner)
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
        &self.inner.info
    }

    pub(super) fn new(
        info: DescriptorSetLayoutInfo,
        owner: WeakDevice,
        handle: vk1_0::DescriptorSetLayout,
        total_count: DescriptorTotalCount,
        index: usize,
    ) -> Self {
        resource_allocated();

        DescriptorSetLayout {
            handle,
            inner: Arc::new(DescriptorSetLayoutInner {
                info,
                owner,
                total_count,
                index,
            }),
        }
    }

    pub(super) fn is_owned_by(&self, owner: &impl PartialEq<WeakDevice>) -> bool {
        *owner == self.inner.owner
    }

    pub(super) fn handle(&self) -> vk1_0::DescriptorSetLayout {
        debug_assert!(!self.handle.is_null());
        self.handle
    }

    pub(super) fn total_count(&self) -> &DescriptorTotalCount {
        &self.inner.total_count
    }
}

/// Set of descriptors with specific layout.
#[derive(Clone)]
pub struct DescriptorSet {
    inner: Arc<DescriptorSetInner>,
}

struct DescriptorSetInner {
    info: DescriptorSetInfo,
    set: ManuallyDrop<gpu_descriptor::DescriptorSet<vk1_0::DescriptorSet>>,
    owner: WeakDevice,
}

impl Drop for DescriptorSetInner {
    fn drop(&mut self) {
        resource_freed();

        if let Some(device) = self.owner.upgrade() {
            unsafe { device.destroy_descriptor_set(ManuallyDrop::take(&mut self.set)) }
        } else if !std::thread::panicking() {
            panic!("Device already dropped")
        }
    }
}

impl Debug for DescriptorSet {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        if fmt.alternate() {
            fmt.debug_struct("DescriptorSet")
                .field("handle", &self.inner.set)
                .field("owner", &self.inner.owner)
                .finish()
        } else {
            write!(fmt, "DescriptorSet({:p})", self.inner.set.raw())
        }
    }
}

impl PartialEq for DescriptorSet {
    fn eq(&self, rhs: &Self) -> bool {
        self.inner.set.raw() == rhs.inner.set.raw()
    }
}

impl Eq for DescriptorSet {}

impl Hash for DescriptorSet {
    fn hash<H>(&self, hasher: &mut H)
    where
        H: Hasher,
    {
        self.inner.set.raw().hash(hasher)
    }
}

impl DescriptorSet {
    pub fn info(&self) -> &DescriptorSetInfo {
        &self.inner.info
    }

    pub(super) fn new(
        info: DescriptorSetInfo,
        owner: WeakDevice,
        set: gpu_descriptor::DescriptorSet<vk1_0::DescriptorSet>,
    ) -> Self {
        resource_allocated();

        DescriptorSet {
            inner: Arc::new(DescriptorSetInner {
                info,
                owner,
                set: ManuallyDrop::new(set),
            }),
        }
    }

    pub(super) fn is_owned_by(&self, owner: &impl PartialEq<WeakDevice>) -> bool {
        *owner == self.inner.owner
    }

    pub(super) fn handle(&self) -> vk1_0::DescriptorSet {
        debug_assert!(!self.inner.set.raw().is_null());
        *self.inner.set.raw()
    }
}

/// Handle for GPU pipeline layout object.
///
/// Pipeline layout defines all descriptor set layouts and push constants
/// used by a pipeline.
#[derive(Clone)]
pub struct PipelineLayout {
    handle: vk1_0::PipelineLayout,
    inner: Arc<PipelineLayoutInner>,
}

struct PipelineLayoutInner {
    info: PipelineLayoutInfo,
    owner: WeakDevice,
    index: usize,
}

impl Drop for PipelineLayoutInner {
    fn drop(&mut self) {
        resource_freed();

        if let Some(device) = self.owner.upgrade() {
            unsafe { device.destroy_pipeline_layout(self.index) }
        }
    }
}

impl Debug for PipelineLayout {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        if fmt.alternate() {
            fmt.debug_struct("PipelineLayout")
                .field("handle", &self.handle)
                .field("owner", &self.inner.owner)
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
        &self.inner.info
    }

    pub(super) fn new(
        info: PipelineLayoutInfo,
        owner: WeakDevice,
        handle: vk1_0::PipelineLayout,
        index: usize,
    ) -> Self {
        resource_allocated();

        PipelineLayout {
            handle,
            inner: Arc::new(PipelineLayoutInner { info, owner, index }),
        }
    }

    pub(super) fn is_owned_by(&self, owner: &impl PartialEq<WeakDevice>) -> bool {
        *owner == self.inner.owner
    }

    pub(super) fn handle(&self) -> vk1_0::PipelineLayout {
        debug_assert!(!self.handle.is_null());
        self.handle
    }
}

/// Resource that describes whole compute pipeline state.
#[derive(Clone)]
pub struct ComputePipeline {
    handle: vk1_0::Pipeline,
    inner: Arc<ComputePipelineInner>,
}

struct ComputePipelineInner {
    info: ComputePipelineInfo,
    owner: WeakDevice,
    index: usize,
}

impl Drop for ComputePipelineInner {
    fn drop(&mut self) {
        resource_freed();

        if let Some(device) = self.owner.upgrade() {
            unsafe { device.destroy_pipeline(self.index) }
        }
    }
}

impl Debug for ComputePipeline {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        if fmt.alternate() {
            fmt.debug_struct("ComputePipeline")
                .field("handle", &self.handle)
                .field("owner", &self.inner.owner)
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
        &self.inner.info
    }

    pub(super) fn new(
        info: ComputePipelineInfo,
        owner: WeakDevice,
        handle: vk1_0::Pipeline,
        index: usize,
    ) -> Self {
        resource_allocated();

        ComputePipeline {
            handle,
            inner: Arc::new(ComputePipelineInner { info, owner, index }),
        }
    }

    pub(super) fn is_owned_by(&self, owner: &impl PartialEq<WeakDevice>) -> bool {
        *owner == self.inner.owner
    }

    pub(super) fn handle(&self) -> vk1_0::Pipeline {
        debug_assert!(!self.handle.is_null());
        self.handle
    }
}

/// Resource that describes whole graphics pipeline state.
#[derive(Clone)]
pub struct GraphicsPipeline {
    handle: vk1_0::Pipeline,
    inner: Arc<GraphicsPipelineInner>,
}

struct GraphicsPipelineInner {
    info: GraphicsPipelineInfo,
    owner: WeakDevice,
    index: usize,
}

impl Drop for GraphicsPipelineInner {
    fn drop(&mut self) {
        resource_freed();

        if let Some(device) = self.owner.upgrade() {
            unsafe { device.destroy_pipeline(self.index) }
        }
    }
}

impl Debug for GraphicsPipeline {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        if fmt.alternate() {
            fmt.debug_struct("GraphicsPipeline")
                .field("handle", &self.handle)
                .field("owner", &self.inner.owner)
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
        &self.inner.info
    }

    pub(super) fn new(
        info: GraphicsPipelineInfo,
        owner: WeakDevice,
        handle: vk1_0::Pipeline,
        index: usize,
    ) -> Self {
        resource_allocated();

        GraphicsPipeline {
            handle,
            inner: Arc::new(GraphicsPipelineInner { info, owner, index }),
        }
    }

    pub(super) fn is_owned_by(&self, owner: &impl PartialEq<WeakDevice>) -> bool {
        *owner == self.inner.owner
    }

    pub(super) fn handle(&self) -> vk1_0::Pipeline {
        debug_assert!(!self.handle.is_null());
        self.handle
    }
}

/// Bottom-level acceleration structure.
#[derive(Clone)]
pub struct AccelerationStructure {
    address: DeviceAddress,
    handle: vkacc::AccelerationStructureKHR,
    inner: Arc<AccelerationStructureInner>,
}

struct AccelerationStructureInner {
    info: AccelerationStructureInfo,
    owner: WeakDevice,
    index: usize,
}

impl Drop for AccelerationStructureInner {
    fn drop(&mut self) {
        resource_freed();

        if let Some(device) = self.owner.upgrade() {
            unsafe { device.destroy_acceleration_structure(self.index) }
        }
    }
}

impl Debug for AccelerationStructure {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        if fmt.alternate() {
            fmt.debug_struct("AccelerationStructure")
                .field("handle", &self.handle)
                .field("owner", &self.inner.owner)
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
        &self.inner.info
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
        resource_allocated();

        AccelerationStructure {
            handle,
            address,
            inner: Arc::new(AccelerationStructureInner { info, owner, index }),
        }
    }

    pub(super) fn is_owned_by(&self, owner: &impl PartialEq<WeakDevice>) -> bool {
        *owner == self.inner.owner
    }

    pub(super) fn handle(&self) -> vkacc::AccelerationStructureKHR {
        debug_assert!(!self.handle.is_null());
        self.handle
    }
}

/// Resource that describes whole ray-tracing pipeline state.
#[derive(Clone)]
pub struct RayTracingPipeline {
    handle: vk1_0::Pipeline,
    inner: Arc<RayTracingPipelineInner>,
}

struct RayTracingPipelineInner {
    info: RayTracingPipelineInfo,
    owner: WeakDevice,
    group_handlers: Arc<[u8]>,
    index: usize,
}

impl Drop for RayTracingPipelineInner {
    fn drop(&mut self) {
        resource_freed();

        if let Some(device) = self.owner.upgrade() {
            unsafe { device.destroy_pipeline(self.index) }
        }
    }
}

impl Debug for RayTracingPipeline {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        if fmt.alternate() {
            fmt.debug_struct("RayTracingPipeline")
                .field("handle", &self.handle)
                .field("owner", &self.inner.owner)
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
        &self.inner.info
    }

    pub(super) fn new(
        info: RayTracingPipelineInfo,
        owner: WeakDevice,
        handle: vk1_0::Pipeline,
        group_handlers: Arc<[u8]>,
        index: usize,
    ) -> Self {
        resource_allocated();

        RayTracingPipeline {
            handle,
            inner: Arc::new(RayTracingPipelineInner {
                info,
                owner,
                group_handlers,
                index,
            }),
        }
    }

    pub(super) fn is_owned_by(&self, owner: &impl PartialEq<WeakDevice>) -> bool {
        *owner == self.inner.owner
    }

    pub(super) fn handle(&self) -> vk1_0::Pipeline {
        debug_assert!(!self.handle.is_null());
        self.handle
    }

    pub(super) fn group_handlers(&self) -> &[u8] {
        &*self.inner.group_handlers
    }
}

#[cfg(feature = "leak-detection")]
mod resource_counting {
    use std::sync::atomic::{AtomicU64, Ordering::Relaxed};

    static RESOURCE_ALLOCATED: AtomicU64 = AtomicU64::new(0);
    static RESOURCE_FREED: AtomicU64 = AtomicU64::new(0);

    #[track_caller]
    pub fn resource_allocated() {
        let allocated = 1 + RESOURCE_ALLOCATED.fetch_add(1, Relaxed);
        let freed = RESOURCE_FREED.load(Relaxed);

        assert!(
            allocated > freed,
            "More resources freed ({}) than allocated ({})",
            freed,
            allocated
        );

        if allocated - freed > 16536 {
            panic!("Too many resources allocated");
        }
    }

    #[track_caller]
    pub fn resource_freed() {
        RESOURCE_FREED.fetch_add(1, Relaxed);
    }
}

#[cfg(not(feature = "leak-detection"))]
mod resource_counting {
    #[inline(always)]
    pub fn resource_allocated() {}

    #[inline(always)]
    pub fn resource_freed() {}
}

use self::resource_counting::{resource_allocated, resource_freed};
