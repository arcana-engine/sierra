// Modified from vk-sync-rs, originally Copyright 2019 Graham Wihlidal
// licensed under MIT license.
//
// https://github.com/gwihlidal/vk-sync-rs/blob/master/LICENSE-MIT

use crate::{BufferMemoryBarrier, GlobalMemoryBarrier, ImageMemoryBarrier, Layout};
use super::{access::GetAccessInfo, convert::ToErupt};

use erupt::vk1_0;

/// Mapping function that translates a global barrier into a set of source and
/// destination pipeline stages, and a memory barrier, that can be used with
/// Vulkan synchronization methods.
pub fn get_global_barrier<'a>(
	barrier: &GlobalMemoryBarrier,
) -> (
	vk1_0::PipelineStageFlags,
	vk1_0::PipelineStageFlags,
	vk1_0::MemoryBarrierBuilder<'a>,
) {
	let mut src_stages = vk1_0::PipelineStageFlags::empty();
	let mut dst_stages = vk1_0::PipelineStageFlags::empty();

	let mut src_access_mask = vk1_0::AccessFlags::empty();
	let mut dst_access_mask = vk1_0::AccessFlags::empty();

	for prev_access in barrier.prev_accesses {
		let previous_info = prev_access.access_info();

		src_stages |= previous_info.stage_mask;

		// Add appropriate availability operations - for writes only.
		if prev_access.is_write() {
			src_access_mask |= previous_info.access_mask;
		}
	}

	for next_access in barrier.next_accesses {
		let next_info = next_access.access_info();

		dst_stages |= next_info.stage_mask;

		// Add visibility operations as necessary.
		// If the src access mask, this is a WAR hazard (or for some reason a "RAR"),
		// so the dst access mask can be safely zeroed as these don't need visibility.
		if src_access_mask != vk1_0::AccessFlags::empty() {
			dst_access_mask |= next_info.access_mask;
		}
	}

	// Ensure that the stage masks are valid if no stages were determined
	if src_stages == vk1_0::PipelineStageFlags::empty() {
		src_stages = vk1_0::PipelineStageFlags::TOP_OF_PIPE;
	}

	if dst_stages == vk1_0::PipelineStageFlags::empty() {
		dst_stages = vk1_0::PipelineStageFlags::BOTTOM_OF_PIPE;
	}

	let memory_barrier = vk1_0::MemoryBarrierBuilder::new()
		.src_access_mask(src_access_mask)
		.dst_access_mask(dst_access_mask);

	(src_stages, dst_stages, memory_barrier)
}

/// Mapping function that translates a buffer barrier into a set of source and
/// destination pipeline stages, and a buffer memory barrier, that can be used
/// with Vulkan synchronization methods.
pub fn get_buffer_memory_barrier<'a>(
	barrier: &BufferMemoryBarrier,
) -> (
	vk1_0::PipelineStageFlags,
	vk1_0::PipelineStageFlags,
	vk1_0::BufferMemoryBarrierBuilder<'a>,
) {
	let mut src_stages = vk1_0::PipelineStageFlags::empty();
	let mut dst_stages = vk1_0::PipelineStageFlags::empty();

	let mut src_access_mask = vk1_0::AccessFlags::empty();
	let mut dst_access_mask = vk1_0::AccessFlags::empty();

	let (src_queue_family_index, dst_queue_family_index) = barrier
		.family_transfer
		.unwrap_or((vk1_0::QUEUE_FAMILY_IGNORED, vk1_0::QUEUE_FAMILY_IGNORED));


	let previous_info = barrier.prev_access.access_info();

	src_stages |= previous_info.stage_mask;

	// Add appropriate availability operations - for writes only.
	if barrier.prev_access.is_write() {
		src_access_mask |= previous_info.access_mask;
	}

	let next_info = barrier.next_access.access_info();

	dst_stages |= next_info.stage_mask;

	// Add visibility operations as necessary.
	// If the src access mask, this is a WAR hazard (or for some reason a "RAR"),
	// so the dst access mask can be safely zeroed as these don't need visibility.
	if src_access_mask != vk1_0::AccessFlags::empty() {
		dst_access_mask |= next_info.access_mask;
	}

	// Ensure that the stage masks are valid if no stages were determined
	if src_stages == vk1_0::PipelineStageFlags::empty() {
		src_stages = vk1_0::PipelineStageFlags::TOP_OF_PIPE;
	}

	if dst_stages == vk1_0::PipelineStageFlags::empty() {
		dst_stages = vk1_0::PipelineStageFlags::BOTTOM_OF_PIPE;
	}

	let buffer_barrier = vk1_0::BufferMemoryBarrierBuilder::new()
		.src_queue_family_index(src_queue_family_index)
		.dst_queue_family_index(dst_queue_family_index)
		.buffer(barrier.buffer.handle())
		.offset(barrier.offset as u64)
		.size(barrier.size as u64);

	(src_stages, dst_stages, buffer_barrier)
}

/// Mapping function that translates an image barrier into a set of source and
/// destination pipeline stages, and an image memory barrier, that can be used
/// with Vulkan synchronization methods.
pub fn get_image_memory_barrier<'a>(
	barrier: &ImageMemoryBarrier,
) -> (
	vk1_0::PipelineStageFlags,
	vk1_0::PipelineStageFlags,
	vk1_0::ImageMemoryBarrierBuilder<'a>,
) {
	let mut src_stages = vk1_0::PipelineStageFlags::empty();
	let mut dst_stages = vk1_0::PipelineStageFlags::empty();

	let mut src_access_mask = vk1_0::AccessFlags::empty();
	let mut dst_access_mask = vk1_0::AccessFlags::empty();

	let (src_queue_family_index, dst_queue_family_index) = barrier
		.family_transfer
		.unwrap_or((vk1_0::QUEUE_FAMILY_IGNORED, vk1_0::QUEUE_FAMILY_IGNORED));

	let previous_info = barrier.prev_access.access_info();

	src_stages |= previous_info.stage_mask;

	// Add appropriate availability operations - for writes only.
	if barrier.prev_access.is_write() {
		src_access_mask |= previous_info.access_mask;
	}

	let old_layout = if let Some(barrier_old_layout) = barrier.old_layout {
		match barrier_old_layout {
			Layout::Optimal => previous_info.image_layout,
			Layout::Manual(layout) => layout.to_erupt(),
		}
	} else {
		vk1_0::ImageLayout::UNDEFINED
	};

	let next_info = barrier.next_access.access_info();

	dst_stages |= next_info.stage_mask;

	// Add visibility operations as necessary.
	// If the src access mask, this is a WAR hazard (or for some reason a "RAR"),
	// so the dst access mask can be safely zeroed as these don't need visibility.
	if src_access_mask != vk1_0::AccessFlags::empty() {
		dst_access_mask |= next_info.access_mask;
	}

	let new_layout = match barrier.new_layout {
		Layout::Optimal => next_info.image_layout,
		Layout::Manual(layout) => layout.to_erupt(),
	};

	// Ensure that the stage masks are valid if no stages were determined
	if src_stages == vk1_0::PipelineStageFlags::empty() {
		src_stages = vk1_0::PipelineStageFlags::TOP_OF_PIPE;
	}

	if dst_stages == vk1_0::PipelineStageFlags::empty() {
		dst_stages = vk1_0::PipelineStageFlags::BOTTOM_OF_PIPE;
	}

	let image_barrier = vk1_0::ImageMemoryBarrierBuilder::new()
		.src_queue_family_index(src_queue_family_index)
		.dst_queue_family_index(dst_queue_family_index)
		.image(barrier.image.handle())
		.subresource_range(barrier.range.to_erupt())
		.src_access_mask(src_access_mask)
		.dst_access_mask(dst_access_mask)
		.old_layout(old_layout)
		.new_layout(new_layout);


	(src_stages, dst_stages, image_barrier)
}
