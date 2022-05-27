use crate::{DescriptorSetWrite, UpdateDescriptorSet};

use {
    super::{
        Descriptor, DescriptorBindingFlags, DescriptorSet, DescriptorSetInfo, DescriptorSetLayout,
        DescriptorSetLayoutBinding, DescriptorSetLayoutFlags, DescriptorSetLayoutInfo, Descriptors,
        DescriptorsAllocationError, DescriptorsInstance, DescriptorsLayout, UpdatedDescriptors,
    },
    crate::{encode::Encoder, shader::ShaderStageFlags, Device, OutOfMemory},
    bitsetium::{BitEmpty, BitSearch, BitSet, BitSetLimit, BitTest, Bits4096},
    std::{
        collections::hash_map::{Entry, HashMap},
        hash::Hash,
        marker::PhantomData,
    },
};

/// Descriptors layout for `SparseDescriptors`.
#[derive(Debug)]
pub struct SparseDescriptorsLayout<T> {
    raw: DescriptorSetLayout,
    cap: u32,
    marker: PhantomData<fn() -> T>,
}

impl<T> DescriptorsLayout for SparseDescriptorsLayout<T>
where
    T: Descriptor,
{
    type Instance = SparseDescriptorsInstance<T>;

    fn raw(&self) -> &DescriptorSetLayout {
        &self.raw
    }

    fn instance(&self) -> SparseDescriptorsInstance<T> {
        SparseDescriptorsInstance::new(self.cap, self.raw.clone())
    }
}

/// Descriptors input to be used in proc-macro pipelines.
#[derive(Debug)]
pub struct SparseDescriptors<T, const CAP: u32, const STAGES: u32> {
    marker: PhantomData<fn() -> T>,
}

impl<T, const CAP: u32, const STAGES: u32> Descriptors for SparseDescriptors<T, CAP, STAGES>
where
    T: Descriptor,
{
    type Layout = SparseDescriptorsLayout<T>;
    type Instance = SparseDescriptorsInstance<T>;

    fn layout(device: &Device) -> Result<SparseDescriptorsLayout<T>, OutOfMemory> {
        let raw = device.create_descriptor_set_layout(DescriptorSetLayoutInfo {
            bindings: vec![DescriptorSetLayoutBinding {
                binding: 0,
                ty: T::TYPE,
                count: CAP,
                stages: ShaderStageFlags::from_bits_truncate(STAGES),
                flags: DescriptorBindingFlags::PARTIALLY_BOUND
                    | DescriptorBindingFlags::UPDATE_UNUSED_WHILE_PENDING,
            }],
            flags: DescriptorSetLayoutFlags::empty(),
        })?;

        Ok(SparseDescriptorsLayout {
            raw,
            cap: CAP,
            marker: PhantomData,
        })
    }
}

/// Descriptor instance with sparsely located resources.
#[derive(Debug)]
pub struct SparseDescriptorsInstance<T: Descriptor> {
    layout: DescriptorSetLayout,
    set: Option<SparseDescriptorSet>,
    indices: HashMap<T::RawDescriptor, u32>,

    upper_bounds: u32,
    unused: Bits4096,

    updates: Vec<T::RawDescriptor>,
}

#[derive(Debug)]
pub struct SparseDescriptorSet {
    raw: DescriptorSet,
}

impl UpdatedDescriptors for SparseDescriptorSet {
    fn raw(&self) -> &DescriptorSet {
        &self.raw
    }
}

impl<T, const CAP: u32, const STAGES: u32> DescriptorsInstance<SparseDescriptors<T, CAP, STAGES>>
    for SparseDescriptorsInstance<T>
where
    T: Descriptor,
    T::RawDescriptor: Hash + Eq,
{
    type Updated = SparseDescriptorSet;

    fn update<'a, 'b: 'a>(
        &'b mut self,
        _input: &SparseDescriptors<T, CAP, STAGES>,
        device: &Device,
        _encoder: &mut Encoder<'a>,
    ) -> Result<&'b SparseDescriptorSet, DescriptorsAllocationError> {
        if self.set.is_none() {
            self.set = Some(SparseDescriptorSet {
                raw: device
                    .create_descriptor_set(DescriptorSetInfo {
                        layout: self.layout.clone(),
                    })?
                    .share(),
            });
        }

        let set = self.set.as_mut().unwrap();
        let indices = &self.indices;

        let mut writes = smallvec::SmallVec::<[_; 32]>::new();

        writes.extend(self.updates.drain(..).filter_map(|descriptor| {
            let (descriptor, idx) = indices.get_key_value(&descriptor)?;

            Some(DescriptorSetWrite {
                binding: 0,
                element: *idx,
                descriptors: T::descriptors(std::slice::from_ref(descriptor)),
            })
        }));

        device.update_descriptor_sets(&mut [UpdateDescriptorSet {
            set: unsafe {
                // # Safety
                //
                // None
                set.raw.as_writable()
            },
            writes: &writes,
            copies: &[],
        }]);

        Ok(set)
    }

    fn raw_layout(&self) -> &DescriptorSetLayout {
        &self.layout
    }
}

impl<T> SparseDescriptorsInstance<T>
where
    T: Descriptor,
{
    /// Returns new empty instance of `SparseDescriptorsInstance`.
    pub fn new(cap: u32, layout: DescriptorSetLayout) -> Self {
        SparseDescriptorsInstance {
            layout,
            set: None,
            upper_bounds: 0,
            indices: HashMap::new(),
            unused: Bits4096::empty(),
            updates: Vec::with_capacity(cap as usize),
        }
    }

    /// Returns index for specified resource inside this array.
    /// Inserts resource if not in array yet.
    ///
    /// # Panics
    ///
    ///
    pub fn get_or_insert(&mut self, descriptor: T::RawDescriptor) -> u32
    where
        T::RawDescriptor: Hash + Clone + Eq,
    {
        match self.indices.entry(descriptor.clone()) {
            Entry::Occupied(entry) => *entry.get(),
            Entry::Vacant(entry) => match self.unused.find_first_set(0) {
                None => {
                    if self.upper_bounds == Bits4096::MAX_SET_INDEX as u32 {
                        panic!("Too many resources inserted");
                    }
                    self.updates.push(descriptor.clone());
                    entry.insert(self.upper_bounds);
                    self.upper_bounds += 1;
                    self.upper_bounds - 1
                }
                Some(idx) => *entry.insert(idx as u32),
            },
        }
    }

    pub fn remove(&mut self, descriptor: T::RawDescriptor) -> bool
    where
        T::RawDescriptor: Hash + Eq,
    {
        match self.indices.get(&descriptor) {
            None => false,
            Some(idx) => {
                if *idx == self.upper_bounds {
                    self.upper_bounds -= 1;

                    while self.upper_bounds > 0 && self.unused.test(self.upper_bounds - 1) {
                        self.unused.set(self.upper_bounds - 1, false);
                        debug_assert!(self.unused.find_set(self.upper_bounds - 1).is_none());
                        self.upper_bounds -= 1;
                    }
                }
                true
            }
        }
    }
}
