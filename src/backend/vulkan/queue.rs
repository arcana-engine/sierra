use {
    super::{
        convert::{oom_error_from_erupt, ToErupt as _},
        device::Device,
        device_lost,
        swapchain::SwapchainImage,
        unexpected_result,
    },
    crate::{
        encode::{CommandBuffer, Encoder},
        fence::Fence,
        out_of_host_memory,
        queue::*,
        semaphore::Semaphore,
        stage::PipelineStageFlags,
        OutOfMemory,
    },
    erupt::{extensions::khr_swapchain::PresentInfoKHRBuilder, vk1_0},
    smallvec::SmallVec,
    std::fmt::{self, Debug},
};

pub struct Queue {
    handle: vk1_0::Queue,
    pool: vk1_0::CommandPool,
    device: Device,
    id: QueueId,
    capabilities: QueueCapabilityFlags,
}

impl Debug for Queue {
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
        }
    }
}

impl Queue {
    pub fn id(&self) -> QueueId {
        self.id
    }

    #[tracing::instrument]
    pub fn create_encoder(&mut self) -> Result<Encoder<'static>, OutOfMemory> {
        if self.pool == vk1_0::CommandPool::null() {
            self.pool = unsafe {
                self.device.logical().create_command_pool(
                    &vk1_0::CommandPoolCreateInfoBuilder::new()
                        .flags(vk1_0::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
                        .queue_family_index(self.id.family as u32),
                    None,
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

        let cbuf = CommandBuffer::new(buffers.remove(0), self.id, self.device.downgrade());

        Ok(Encoder::new(cbuf, self.capabilities))
    }

    #[tracing::instrument(skip(cbufs))]
    pub fn submit(
        &mut self,
        wait: &[(PipelineStageFlags, Semaphore)],
        cbufs: impl IntoIterator<Item = CommandBuffer>,
        signal: &[Semaphore],
        fence: Option<&Fence>,
    ) {
        let cbufs: SmallVec<[_; 8]> = cbufs
            .into_iter()
            .map(|cbuf| {
                assert_owner!(cbuf, self.device);
                assert_eq!(self.id, cbuf.queue());
                cbuf.handle()
            })
            .collect();

        for (_, semaphore) in wait {
            assert_owner!(semaphore, self.device);
        }

        for semaphore in signal {
            assert_owner!(semaphore, self.device);
        }

        if let Some(fence) = fence {
            assert_owner!(fence, self.device);
        }

        // FIXME: Check semaphore states.
        let (wait_stages, wait_semaphores): (SmallVec<[_; 8]>, SmallVec<[_; 8]>) = wait
            .iter()
            .map(|(ps, sem)| (ps.to_erupt(), sem.handle()))
            .unzip();

        let signal_semaphores: SmallVec<[_; 8]> = signal.iter().map(|sem| sem.handle()).collect();

        unsafe {
            self.device
                .logical()
                .queue_submit(
                    self.handle,
                    &[vk1_0::SubmitInfoBuilder::new()
                        .wait_semaphores(&wait_semaphores)
                        .wait_dst_stage_mask(&wait_stages)
                        .signal_semaphores(&signal_semaphores)
                        .command_buffers(&cbufs)],
                    fence.map(|f| f.handle()),
                )
                .expect("TODO: Handle queue submit error")
        };
    }

    #[tracing::instrument]
    pub fn submit_one(&mut self, buffer: CommandBuffer, fence: Option<&Fence>) {
        self.submit(&[], Some(buffer), &[], fence);
    }

    #[tracing::instrument]
    pub fn present(&mut self, image: SwapchainImage) -> Result<PresentOk, PresentError> {
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

        let result = unsafe {
            self.device.logical().queue_present_khr(
                self.handle,
                &PresentInfoKHRBuilder::new()
                    .wait_semaphores(&[image.info().signal.handle()])
                    .swapchains(&[image.handle()])
                    .image_indices(&[image.index()]),
            )
        };

        match result.raw {
            vk1_0::Result::SUCCESS => Ok(PresentOk::Success),
            vk1_0::Result::SUBOPTIMAL_KHR => Ok(PresentOk::Suboptimal),
            vk1_0::Result::ERROR_OUT_OF_DATE_KHR => Err(PresentError::OutOfDate),
            vk1_0::Result::ERROR_SURFACE_LOST_KHR => Err(PresentError::SurfaceLost),
            // vk1_0::Result::ERROR_FULL_SCREEN_EXCLUSIVE_MODE_LOST_EXT => {}
            result => Err(PresentError::OutOfMemory {
                source: queue_error(result),
            }),
        }
    }

    #[tracing::instrument]
    pub fn wait_for_idle(&self) -> Result<(), OutOfMemory> {
        unsafe { self.device.logical().queue_wait_idle(self.handle) }
            .result()
            .map_err(queue_error)
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
