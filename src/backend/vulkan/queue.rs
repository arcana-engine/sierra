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
    bumpalo::{collections::Vec as BVec, Bump},
    erupt::{extensions::khr_swapchain::PresentInfoKHRBuilder, vk1_0},
    std::fmt::{self, Debug},
};

pub struct Queue {
    handle: vk1_0::Queue,
    pool: vk1_0::CommandPool,
    device: Device,
    id: QueueId,
    capabilities: QueueCapabilityFlags,
    cbufs: Vec<CommandBuffer>,
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
            cbufs: Vec::new(),
        }
    }
}

impl Queue {
    pub fn id(&self) -> QueueId {
        self.id
    }

    #[tracing::instrument]
    pub fn create_encoder<'a>(&mut self, bump: &'a Bump) -> Result<Encoder<'a>, OutOfMemory> {
        match self.cbufs.pop() {
            Some(cbuf) => Ok(Encoder::new(cbuf, self.capabilities, bump)),
            None => {
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

                Ok(Encoder::new(cbuf, self.capabilities, bump))
            }
        }
    }

    #[tracing::instrument(skip(cbufs))]
    pub fn submit(
        &mut self,
        wait: &mut [(PipelineStageFlags, &mut Semaphore)],
        cbufs: impl IntoIterator<IntoIter = impl ExactSizeIterator<Item = CommandBuffer>>,
        signal: &mut [&mut Semaphore],
        mut fence: Option<&mut Fence>,
        bump: &Bump,
    ) {
        let cbufs = cbufs.into_iter();
        let mut handles = BVec::with_capacity_in(cbufs.len(), bump);
        let mut array = BVec::with_capacity_in(cbufs.len(), bump);

        cbufs.for_each(|cbuf| {
            assert_owner!(cbuf, self.device);
            assert_eq!(self.id, cbuf.queue());
            handles.push(cbuf.handle());
            array.push(cbuf);
        });

        for (_, semaphore) in wait.iter_mut() {
            assert_owner!(semaphore, self.device);
        }

        for semaphore in signal.iter_mut() {
            assert_owner!(semaphore, self.device);
        }

        if let Some(fence) = fence.as_deref_mut() {
            assert_owner!(fence, self.device);
            let epoch = self.device.epochs().next_epoch(self.id);
            fence.arm(self.id, epoch, &self.device);
        }

        // FIXME: Check semaphore states.
        let wait_stages = bump.alloc_slice_fill_iter(wait.iter().map(|(ps, _)| ps.to_erupt()));
        let wait_semaphores = bump.alloc_slice_fill_iter(wait.iter().map(|(_, sem)| sem.handle()));
        let signal_semaphores = bump.alloc_slice_fill_iter(signal.iter().map(|sem| sem.handle()));

        unsafe {
            self.device
                .logical()
                .queue_submit(
                    self.handle,
                    &[vk1_0::SubmitInfoBuilder::new()
                        .wait_semaphores(&wait_semaphores)
                        .wait_dst_stage_mask(&wait_stages)
                        .signal_semaphores(&signal_semaphores)
                        .command_buffers(&handles)],
                    fence.map(|f| f.handle()),
                )
                .expect("TODO: Handle queue submit error")
        };

        self.device.epochs().submit(self.id, array.into_iter());
    }

    #[tracing::instrument]
    pub fn submit_one(&mut self, cbuf: CommandBuffer, fence: Option<&Fence>) {
        assert_owner!(cbuf, self.device);
        assert_eq!(self.id, cbuf.queue());
        let cbuf = cbuf.handle();

        unsafe {
            self.device
                .logical()
                .queue_submit(
                    self.handle,
                    &[vk1_0::SubmitInfoBuilder::new()
                        .wait_semaphores(&[])
                        .wait_dst_stage_mask(&[])
                        .signal_semaphores(&[])
                        .command_buffers(std::slice::from_ref(&cbuf))],
                    fence.map(|f| f.handle()),
                )
                .expect("TODO: Handle queue submit error")
        };
    }

    #[tracing::instrument]
    pub fn present(&mut self, mut image: SwapchainImage<'_>) -> Result<PresentOk, OutOfMemory> {
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

        let result = unsafe {
            self.device.logical().queue_present_khr(
                self.handle,
                &PresentInfoKHRBuilder::new()
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

    #[tracing::instrument]
    pub fn wait_for_idle(&self) -> Result<(), OutOfMemory> {
        unsafe { self.device.logical().queue_wait_idle(self.handle) }
            .result()
            .map_err(queue_error)
    }

    fn drain_ready_cbufs(&mut self) -> Result<(), OutOfMemory> {
        // self.device.epochs().drain_cbuf(self.id, &mut self.cbufs);
        // for cbuf in &self.cbufs {
        //     unsafe {
        //         self.device.logical().reset_command_buffer(
        //             cbuf.handle(),
        //             Some(vk1_0::CommandBufferResetFlags::RELEASE_RESOURCES),
        //         )
        //     }
        //     .result()
        //     .map_err(queue_error)?;
        // }

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
