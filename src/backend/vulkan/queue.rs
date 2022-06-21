use std::fmt;

use arrayvec::ArrayVec;
use erupt::{
    extensions::{
        google_display_timing::{PresentTimeGOOGLEBuilder, PresentTimesInfoGOOGLEBuilder},
        khr_swapchain::PresentInfoKHRBuilder,
    },
    vk1_0, ExtendableFrom,
};
use scoped_arena::Scope;

use crate::{
    encode::{CommandBuffer, Encoder},
    fence::Fence,
    out_of_host_memory,
    queue::*,
    semaphore::Semaphore,
    stage::PipelineStageFlags,
    OutOfMemory,
};

use super::{
    convert::{oom_error_from_erupt, ToErupt as _},
    device::Device,
    device_lost,
    swapchain::SwapchainImage,
    unexpected_result,
};

pub struct Queue {
    handle: vk1_0::Queue,
    pool: vk1_0::CommandPool,
    device: Device,
    id: QueueId,
    capabilities: QueueCapabilityFlags,
    cbufs: Vec<CommandBuffer>,
}

impl fmt::Debug for Queue {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        if fmt.alternate() {
            fmt.debug_struct("Queue")
                .field("handle", &self.handle)
                .field("id", &self.id)
                .field("capabilities", &self.capabilities)
                .field("device", &self.device)
                .finish()
        } else {
            write!(fmt, "{:p}", self.handle)
        }
    }
}

impl Queue {
    pub(crate) fn new(
        handle: vk1_0::Queue,
        pool: vk1_0::CommandPool,
        device: Device,
        id: QueueId,
        capabilities: QueueCapabilityFlags,
    ) -> Self {
        Queue {
            handle,
            device,
            pool,
            id,
            capabilities,
            cbufs: Vec::new(),
        }
    }
}

impl Queue {
    pub fn id(&self) -> QueueId {
        self.id
    }

    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub fn create_encoder<'a>(&mut self, scope: &'a Scope<'a>) -> Result<Encoder<'a>, OutOfMemory> {
        let mut cbuf = match self.cbufs.pop() {
            Some(cbuf) => cbuf,
            None => {
                if self.pool == vk1_0::CommandPool::null() {
                    self.pool = unsafe {
                        self.device.logical().create_command_pool(
                            &vk1_0::CommandPoolCreateInfoBuilder::new()
                                .flags(vk1_0::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
                                .queue_family_index(self.id.family as u32),
                            None,
                        )
                    }
                    .result()
                    .map_err(oom_error_from_erupt)?;
                }

                assert_ne!(self.pool, vk1_0::CommandPool::null());

                let mut buffers = unsafe {
                    self.device.logical().allocate_command_buffers(
                        &vk1_0::CommandBufferAllocateInfoBuilder::new()
                            .command_pool(self.pool)
                            .level(vk1_0::CommandBufferLevel::PRIMARY)
                            .command_buffer_count(1),
                    )
                }
                .result()
                .map_err(oom_error_from_erupt)?;

                CommandBuffer::new(buffers.remove(0), self.id, self.device.clone())
            }
        };
        match cbuf.begin() {
            Err(err) => {
                self.cbufs.push(cbuf);
                Err(err)
            }
            Ok(()) => Ok(Encoder::new(cbuf, self.capabilities, scope)),
        }
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(skip(cbufs)))]
    pub fn submit(
        &mut self,
        wait: &mut [(PipelineStageFlags, &mut Semaphore)],
        cbufs: impl IntoIterator<Item = CommandBuffer>,
        signal: &mut [&mut Semaphore],
        mut fence: Option<&mut Fence>,
        scope: &Scope<'_>,
    ) {
        let array = scope.to_scope_with(ArrayVec::<_, 64>::new);

        let handles = scope.to_scope_from_iter(cbufs.into_iter().map(|cbuf| {
            assert_owner!(cbuf, self.device);
            assert_eq!(self.id, cbuf.queue());
            let handle = cbuf.handle();
            array.push(cbuf);
            handle
        }));

        for (_, semaphore) in wait.iter_mut() {
            assert_owner!(semaphore, self.device);
        }

        for semaphore in signal.iter_mut() {
            assert_owner!(semaphore, self.device);
        }

        if let Some(fence) = fence.as_mut() {
            assert_owner!(fence, self.device);
            let epoch = self.device.epochs().next_epoch(self.id);
            fence.arm(self.id, epoch, &self.device);
        }

        // FIXME: Check semaphore states.
        let wait_stages = scope.to_scope_from_iter(wait.iter().map(|(ps, _)| ps.to_erupt()));
        let wait_semaphores = scope.to_scope_from_iter(wait.iter().map(|(_, sem)| sem.handle()));
        let signal_semaphores = scope.to_scope_from_iter(signal.iter().map(|sem| sem.handle()));

        unsafe {
            self.device
                .logical()
                .queue_submit(
                    self.handle,
                    &[vk1_0::SubmitInfoBuilder::new()
                        .wait_semaphores(&*wait_semaphores)
                        .wait_dst_stage_mask(&*wait_stages)
                        .signal_semaphores(&*signal_semaphores)
                        .command_buffers(&*handles)],
                    fence.map_or(vk1_0::Fence::null(), |f| f.handle()),
                )
                .expect("TODO: Handle queue submit error")
        };

        self.device.epochs().submit(self.id, array.drain(..));
    }

    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub fn submit_one(&mut self, cbuf: CommandBuffer, fence: Option<&Fence>) {
        assert_owner!(cbuf, self.device);
        assert_eq!(self.id, cbuf.queue());
        let handle = cbuf.handle();

        unsafe {
            self.device
                .logical()
                .queue_submit(
                    self.handle,
                    &[vk1_0::SubmitInfoBuilder::new()
                        .wait_semaphores(&[])
                        .wait_dst_stage_mask(&[])
                        .signal_semaphores(&[])
                        .command_buffers(std::slice::from_ref(&handle))],
                    fence.map_or(vk1_0::Fence::null(), |f| f.handle()),
                )
                .expect("TODO: Handle queue submit error")
        };

        self.device.epochs().submit(self.id, std::iter::once(cbuf));
    }

    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub fn present(&mut self, image: SwapchainImage<'_>) -> Result<PresentOk, OutOfMemory> {
        self.present_impl(image, None)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub fn present_with_timing(
        &mut self,
        image: SwapchainImage<'_>,
        present_id: u32,
        desired_present_time: u64,
    ) -> Result<PresentOk, OutOfMemory> {
        assert!(
            self.device.logical().enabled().google_display_timing,
            "`DisplayTiming` feature is not enabled"
        );
        self.present_impl(image, Some((present_id, desired_present_time)))
    }

    pub fn present_impl(
        &mut self,
        mut image: SwapchainImage<'_>,
        timing: Option<(u32, u64)>,
    ) -> Result<PresentOk, OutOfMemory> {
        assert_owner!(image, self.device);

        // FIXME: Check semaphore states.
        assert!(
            self.device.logical().enabled().khr_swapchain,
            "Should be enabled given that there is a Swapchain"
        );

        assert!(
            image.supported_families()[self.id.family as usize],
            "Family `{}` does not support presentation to swapchain `{:?}`",
            self.id.family,
            image
        );

        let [_, signal] = image.wait_signal();

        let mut info = PresentInfoKHRBuilder::new();

        let mut present_times_info = PresentTimesInfoGOOGLEBuilder::new();
        let mut present_time = PresentTimeGOOGLEBuilder::new();

        if let Some((present_id, desired_present_time)) = timing {
            present_time = present_time
                .present_id(present_id)
                .desired_present_time(desired_present_time);

            present_times_info = present_times_info.times(std::slice::from_ref(&present_time));

            info = info.extend_from(&mut present_times_info);
        }

        let result = unsafe {
            self.device.logical().queue_present_khr(
                self.handle,
                &info
                    .wait_semaphores(&[signal.handle()])
                    .swapchains(&[image.handle()])
                    .image_indices(&[image.index()]),
            )
        };

        image.presented();

        self.drain_ready_cbufs()?;

        match result.raw {
            vk1_0::Result::SUCCESS => Ok(PresentOk::Success),
            vk1_0::Result::SUBOPTIMAL_KHR => Ok(PresentOk::Suboptimal),
            vk1_0::Result::ERROR_OUT_OF_DATE_KHR => Ok(PresentOk::OutOfDate),
            vk1_0::Result::ERROR_SURFACE_LOST_KHR => Ok(PresentOk::SurfaceLost),
            // vk1_0::Result::ERROR_FULL_SCREEN_EXCLUSIVE_MODE_LOST_EXT => {}
            result => Err(queue_error(result)),
        }
    }

    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub fn wait_idle(&self) -> Result<(), OutOfMemory> {
        unsafe { self.device.logical().queue_wait_idle(self.handle) }
            .result()
            .map_err(queue_error)
    }

    fn drain_ready_cbufs(&mut self) -> Result<(), OutOfMemory> {
        let offset = self.cbufs.len();
        self.device.epochs().drain_cbuf(self.id, &mut self.cbufs);
        for cbuf in &self.cbufs[offset..] {
            unsafe {
                self.device.logical().reset_command_buffer(
                    cbuf.handle(),
                    vk1_0::CommandBufferResetFlags::RELEASE_RESOURCES,
                )
            }
            .result()
            .map_err(queue_error)?;
        }

        #[cfg(feature = "leak-detection")]
        if self.cbufs.len() > 4096 {
            warn!("Too many cbufs accumulated");
        }

        Ok(())
    }
}

fn queue_error(result: vk1_0::Result) -> OutOfMemory {
    match result {
        vk1_0::Result::ERROR_OUT_OF_HOST_MEMORY => out_of_host_memory(),
        vk1_0::Result::ERROR_OUT_OF_DEVICE_MEMORY => OutOfMemory,
        vk1_0::Result::ERROR_DEVICE_LOST => device_lost(),
        result => unexpected_result(result),
    }
}
