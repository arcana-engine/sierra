use std::{
    collections::{HashMap, VecDeque},
    convert::TryFrom as _,
};

use parking_lot::Mutex;
use smallvec::SmallVec;

use crate::QueueId;

use super::{
    encode::CommandBuffer,
    resources::{
        AccelerationStructure, Buffer, ComputePipeline, DescriptorSet, Framebuffer,
        GraphicsPipeline, Image, ImageView, PipelineLayout, RayTracingPipeline, Sampler,
    },
};

pub(super) struct Epochs {
    queues: HashMap<QueueId, Mutex<QueueEpochs>>,
}

impl Epochs {
    pub fn new(queues: impl Iterator<Item = QueueId>) -> Self {
        Epochs {
            queues: queues
                .map(|q| (q, Mutex::new(QueueEpochs::new())))
                .collect(),
        }
    }

    pub fn next_epoch(&self, queue: QueueId) -> u64 {
        let mut queue = self.queues[&queue].lock();
        queue.current += 1;

        let epoch = queue.cache.pop_front().unwrap_or_else(Epoch::new);
        queue.epochs.push_front(epoch);

        #[cfg(feature = "leak-detection")]
        if queue.epochs.len() > 32 {
            warn!(
                "Too many active epochs ({}) accumulated",
                queue.epochs.len()
            );
        }

        queue.current - 1
    }

    pub fn close_epoch(&self, queue: QueueId, epoch: u64) {
        let mut queue = self.queues[&queue].lock();
        debug_assert!(queue.current > epoch);
        if let Ok(len) = usize::try_from(queue.current - epoch) {
            if len < queue.epochs.len() {
                let epochs = queue.epochs.drain(len..).collect::<SmallVec<[_; 16]>>();

                for mut epoch in epochs {
                    for mut cbuf in epoch.cbufs.drain(..) {
                        cbuf.references().clear();
                        queue.cbufs.push(cbuf);
                    }
                    queue.cache.push_back(epoch);
                }
            }
        }

        if queue.cache.len() > 64 {
            warn!("Too large epochs cache accumulated");
        }

        if queue.cbufs.len() > 1024 {
            warn!("Too large cbuf cache accumulated");
        }
    }

    pub fn next_epoch_all_queues(&self) -> Vec<(QueueId, u64)> {
        let mut result = Vec::new();
        for (id, queue) in &self.queues {
            let mut queue = queue.lock();
            queue.current += 1;

            let epoch = queue.cache.pop_front().unwrap_or_else(Epoch::new);
            queue.epochs.push_front(epoch);
            result.push((*id, queue.current - 1))
        }
        result
    }

    pub fn drain_cbuf(&self, queue: QueueId, cbufs: &mut Vec<CommandBuffer>) {
        let mut queue = self.queues[&queue].lock();
        cbufs.append(&mut queue.cbufs);
    }

    pub fn submit(&self, queue: QueueId, cbufs: impl Iterator<Item = CommandBuffer>) {
        let mut queue = self.queues[&queue].lock();
        let front = queue
            .epochs
            .front_mut()
            .unwrap_or_else(|| unsafe { std::hint::unreachable_unchecked() });
        front.cbufs.extend(cbufs);
    }
}

struct QueueEpochs {
    current: u64,
    cbufs: Vec<CommandBuffer>,
    cache: VecDeque<Epoch>,
    epochs: VecDeque<Epoch>,
}

impl Drop for QueueEpochs {
    fn drop(&mut self) {
        if !std::thread::panicking() {
            assert!(
                self.cbufs
                    .iter_mut()
                    .all(|cbuf| cbuf.references().is_empty()),
                "All cbufs must be flushed"
            );
            assert!(
                self.epochs.iter().all(|e| e.cbufs.is_empty()),
                "All epochs must be flushed"
            );
        }

        self.cbufs.clear();
        self.epochs.clear();
        self.cache.clear();
    }
}

struct Epoch {
    cbufs: Vec<CommandBuffer>,
}

impl Epoch {
    fn new() -> Self {
        Epoch { cbufs: Vec::new() }
    }
}

pub(super) struct References {
    buffers: Vec<Buffer>,
    images: Vec<Image>,
    image_views: Vec<ImageView>,
    graphics_pipelines: Vec<GraphicsPipeline>,
    compute_pipelines: Vec<ComputePipeline>,
    ray_tracing_pipelines: Vec<RayTracingPipeline>,
    pipeline_layouts: Vec<PipelineLayout>,
    framebuffers: Vec<Framebuffer>,
    acceleration_strucutres: Vec<AccelerationStructure>,
    samplers: Vec<Sampler>,
    descriptor_sets: Vec<DescriptorSet>,
}

impl References {
    pub const fn new() -> Self {
        References {
            buffers: Vec::new(),
            images: Vec::new(),
            image_views: Vec::new(),
            graphics_pipelines: Vec::new(),
            compute_pipelines: Vec::new(),
            ray_tracing_pipelines: Vec::new(),
            pipeline_layouts: Vec::new(),
            framebuffers: Vec::new(),
            acceleration_strucutres: Vec::new(),
            samplers: Vec::new(),
            descriptor_sets: Vec::new(),
        }
    }

    pub fn add_buffer(&mut self, buffer: Buffer) {
        self.buffers.push(buffer);
    }

    pub fn add_image(&mut self, image: Image) {
        self.images.push(image);
    }

    // pub fn add_image_view(&mut self, image_view: ImageView) {
    //     self.image_views.push(image_view);
    // }

    pub fn add_graphics_pipeline(&mut self, graphics_pipeline: GraphicsPipeline) {
        self.graphics_pipelines.push(graphics_pipeline);
    }

    pub fn add_compute_pipeline(&mut self, compute_pipeline: ComputePipeline) {
        self.compute_pipelines.push(compute_pipeline);
    }

    pub fn add_ray_tracing_pipeline(&mut self, ray_tracing_pipeline: RayTracingPipeline) {
        self.ray_tracing_pipelines.push(ray_tracing_pipeline);
    }

    pub fn add_pipeline_layout(&mut self, pipeline_layout: PipelineLayout) {
        self.pipeline_layouts.push(pipeline_layout);
    }

    pub fn add_framebuffer(&mut self, framebuffer: Framebuffer) {
        self.framebuffers.push(framebuffer);
    }

    pub fn add_acceleration_strucutre(&mut self, acceleration_strucutre: AccelerationStructure) {
        self.acceleration_strucutres.push(acceleration_strucutre);
    }

    // pub fn add_sampler(&mut self, sampler: Sampler) {
    //     self.samplers.push(sampler);
    // }

    pub fn add_descriptor_set(&mut self, descriptor_set: DescriptorSet) {
        self.descriptor_sets.push(descriptor_set);
    }

    pub fn is_empty(&self) -> bool {
        self.buffers.is_empty()
            && self.images.is_empty()
            && self.image_views.is_empty()
            && self.graphics_pipelines.is_empty()
            && self.compute_pipelines.is_empty()
            && self.ray_tracing_pipelines.is_empty()
            && self.pipeline_layouts.is_empty()
            && self.framebuffers.is_empty()
            && self.acceleration_strucutres.is_empty()
            && self.samplers.is_empty()
            && self.descriptor_sets.is_empty()
    }

    pub fn clear(&mut self) {
        self.buffers.clear();
        self.images.clear();
        self.image_views.clear();
        self.graphics_pipelines.clear();
        self.compute_pipelines.clear();
        self.ray_tracing_pipelines.clear();
        self.pipeline_layouts.clear();
        self.framebuffers.clear();
        self.acceleration_strucutres.clear();
        self.samplers.clear();
        self.descriptor_sets.clear();
    }
}

impl QueueEpochs {
    fn new() -> Self {
        QueueEpochs {
            current: 0,
            cbufs: Vec::new(),
            cache: VecDeque::new(),
            epochs: std::iter::once(Epoch::new()).collect(),
        }
    }
}
