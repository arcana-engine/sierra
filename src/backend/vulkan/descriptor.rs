use {
    crate::descriptor::*,
    erupt::vk1_0,
    std::{
        hash::{Hash, Hasher},
        ops::Deref,
    },
};

const DESCRIPTOR_TYPES_COUNT: usize = 12;

fn descriptor_type_from_index(index: usize) -> vk1_0::DescriptorType {
    debug_assert!(index < DESCRIPTOR_TYPES_COUNT);

    match index {
        0 => {
            debug_assert_eq!(DescriptorType::Sampler as usize, index);

            vk1_0::DescriptorType::SAMPLER
        }
        1 => {
            debug_assert_eq!(
                DescriptorType::CombinedImageSampler as usize,
                index
            );

            vk1_0::DescriptorType::COMBINED_IMAGE_SAMPLER
        }
        2 => {
            debug_assert_eq!(DescriptorType::SampledImage as usize, index);

            vk1_0::DescriptorType::SAMPLED_IMAGE
        }
        3 => {
            debug_assert_eq!(DescriptorType::StorageImage as usize, index);

            vk1_0::DescriptorType::STORAGE_IMAGE
        }
        4 => {
            debug_assert_eq!(
                DescriptorType::UniformTexelBuffer as usize,
                index
            );

            vk1_0::DescriptorType::UNIFORM_TEXEL_BUFFER
        }
        5 => {
            debug_assert_eq!(
                DescriptorType::StorageTexelBuffer as usize,
                index
            );

            vk1_0::DescriptorType::STORAGE_TEXEL_BUFFER
        }
        6 => {
            debug_assert_eq!(DescriptorType::UniformBuffer as usize, index);

            vk1_0::DescriptorType::UNIFORM_BUFFER
        }
        7 => {
            debug_assert_eq!(DescriptorType::StorageBuffer as usize, index);

            vk1_0::DescriptorType::STORAGE_BUFFER
        }
        8 => {
            debug_assert_eq!(
                DescriptorType::UniformBufferDynamic as usize,
                index
            );

            vk1_0::DescriptorType::UNIFORM_BUFFER_DYNAMIC
        }
        9 => {
            debug_assert_eq!(
                DescriptorType::StorageBufferDynamic as usize,
                index
            );

            vk1_0::DescriptorType::STORAGE_BUFFER_DYNAMIC
        }
        10 => {
            debug_assert_eq!(DescriptorType::InputAttachment as usize, index);

            vk1_0::DescriptorType::INPUT_ATTACHMENT
        }
        11 => {
            debug_assert_eq!(
                DescriptorType::AccelerationStructure as usize,
                index
            );

            vk1_0::DescriptorType::ACCELERATION_STRUCTURE_KHR
        }
        _ => unreachable!(),
    }
}

#[derive(Clone, Debug)]
pub struct DescriptorSizesBuilder {
    sizes: [u32; DESCRIPTOR_TYPES_COUNT],
}

impl DescriptorSizesBuilder {
    /// Create new instance without descriptors.
    pub fn zero() -> Self {
        DescriptorSizesBuilder {
            sizes: [0; DESCRIPTOR_TYPES_COUNT],
        }
    }

    /// Add a single layout binding.
    /// Useful when created with `DescriptorSizes::zero()`.
    pub fn add_binding(&mut self, binding: &DescriptorSetLayoutBinding) {
        self.sizes[binding.ty as usize] += binding.count;
    }

    /// Calculate ranges from bindings.
    pub fn from_bindings(bindings: &[DescriptorSetLayoutBinding]) -> Self {
        let mut ranges = Self::zero();

        for binding in bindings {
            ranges.add_binding(binding);
        }

        ranges
    }

    pub fn build(self) -> DescriptorSizes {
        let mut sizes = [vk1_0::DescriptorPoolSizeBuilder::new()
            ._type(vk1_0::DescriptorType::SAMPLER)
            .descriptor_count(0);
            DESCRIPTOR_TYPES_COUNT];

        let mut count = 0u8;

        for (i, size) in self.sizes.iter().copied().enumerate() {
            if size > 0 {
                sizes[count as usize]._type = descriptor_type_from_index(i);

                sizes[count as usize].descriptor_count = size;

                count += 1;
            }
        }

        DescriptorSizes { sizes, count }
    }
}

/// Number of descriptors per type.
#[derive(Clone, Debug)]
pub struct DescriptorSizes {
    sizes: [vk1_0::DescriptorPoolSizeBuilder<'static>; DESCRIPTOR_TYPES_COUNT],
    count: u8,
}

impl DescriptorSizes {
    pub fn as_slice(&self) -> &[vk1_0::DescriptorPoolSizeBuilder<'static>] {
        &self.sizes[..self.count.into()]
    }

    /// Calculate ranges from bindings.
    pub fn from_bindings(bindings: &[DescriptorSetLayoutBinding]) -> Self {
        DescriptorSizesBuilder::from_bindings(bindings).build()
    }
}

impl Deref for DescriptorSizes {
    type Target = [vk1_0::DescriptorPoolSizeBuilder<'static>];

    fn deref(&self) -> &[vk1_0::DescriptorPoolSizeBuilder<'static>] {
        self.as_slice()
    }
}

impl Hash for DescriptorSizes {
    fn hash<H>(&self, hasher: &mut H)
    where
        H: Hasher,
    {
        for size in self.as_slice() {
            hasher.write_u32(size.descriptor_count);
        }
    }
}

impl PartialEq for DescriptorSizes {
    fn eq(&self, rhs: &Self) -> bool {
        self.as_slice().iter().zip(rhs.as_slice()).all(|(l, r)| {
            l._type == r._type && l.descriptor_count == r.descriptor_count
        })
    }
}

impl Eq for DescriptorSizes {}
