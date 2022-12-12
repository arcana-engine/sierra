mod buffer;
mod image;
mod sampler;

pub use self::buffer::TypedBuffer;

use crate::{sealed::Sealed, DescriptorBindingFlags, DescriptorKind, Device, Encoder, OutOfMemory};

/// Trait for all types that can be used as a descriptor.
pub trait DescriptorBinding<K: DescriptorKind> {
    /// Flags necessary for this binding type.
    const FLAGS: DescriptorBindingFlags;

    /// Checks descriptor bound to a set.
    /// Returns `true` if bound descriptor is compatible with input.
    /// Returns `false` if new descriptor should be bound.
    fn is_compatible(&self, descriptor: &K::Descriptor) -> bool;

    /// Updates content of descriptor.
    #[inline]
    fn update_descriptor(
        &mut self,
        device: &Device,
        encoder: &mut Encoder,
        descriptor: &K::Descriptor,
    ) -> Result<(), OutOfMemory> {
        let _ = device;
        let _ = encoder;
        let _ = descriptor;
        Ok(())
    }

    /// Returns compatible descriptor to be bound to the set.
    fn get_descriptor(&self, device: &Device) -> Result<K::Descriptor, OutOfMemory>;
}

/// Trait for all types that can be used as a descriptor array.
pub trait DescriptorBindingArray<K> {
    /// Number of descriptors in the binding.
    const COUNT: u32;

    /// Flags necessary for this binding type.
    const FLAGS: DescriptorBindingFlags;

    /// Descriptors value.
    type DescriptorArray;

    /// Checks descriptors bound to a set.
    /// Returns `true` if bound descriptors are compatible with input.
    /// Returns `false` if new descriptors should be bound.
    fn is_compatible(&self, descriptors: &Self::DescriptorArray) -> bool;

    /// Updates content of descriptors.
    #[inline]
    fn update_descriptors(
        &mut self,
        device: &Device,
        encoder: &mut Encoder,
        descriptors: &Self::DescriptorArray,
    ) -> Result<(), OutOfMemory> {
        let _ = device;
        let _ = encoder;
        let _ = descriptors;
        Ok(())
    }

    /// Returns `Descriptors` equivalent to self.
    fn get_descriptors(&self, device: &Device) -> Result<Self::DescriptorArray, OutOfMemory>;
}

impl<K, T> DescriptorBindingArray<K> for T
where
    K: DescriptorKind,
    T: DescriptorBinding<K> + Sealed,
{
    const COUNT: u32 = 1;
    const FLAGS: DescriptorBindingFlags = <T as DescriptorBinding<K>>::FLAGS;
    type DescriptorArray = [K::Descriptor; 1];

    #[inline]
    fn is_compatible(&self, descriptors: &[K::Descriptor; 1]) -> bool {
        <T as DescriptorBinding<K>>::is_compatible(self, &descriptors[0])
    }

    #[inline]
    fn update_descriptors(
        &mut self,
        device: &Device,
        encoder: &mut Encoder,
        descriptors: &[K::Descriptor; 1],
    ) -> Result<(), OutOfMemory> {
        <T as DescriptorBinding<K>>::update_descriptor(self, device, encoder, &descriptors[0])
    }

    #[inline]
    fn get_descriptors(&self, device: &Device) -> Result<[K::Descriptor; 1], OutOfMemory> {
        Ok([<T as DescriptorBinding<K>>::get_descriptor(self, device)?])
    }
}

impl<K, T, const N: usize> DescriptorBindingArray<K> for [T; N]
where
    K: DescriptorKind,
    T: DescriptorBinding<K> + Sealed,
{
    const COUNT: u32 = N as u32;
    const FLAGS: DescriptorBindingFlags = <T as DescriptorBinding<K>>::FLAGS;
    type DescriptorArray = [K::Descriptor; N];

    #[inline]
    fn is_compatible(&self, descriptors: &[K::Descriptor; N]) -> bool {
        for i in 0..N {
            if !<T as DescriptorBinding<K>>::is_compatible(&self[i], &descriptors[i]) {
                return false;
            }
        }
        true
    }

    #[inline]
    fn update_descriptors(
        &mut self,
        device: &Device,
        encoder: &mut Encoder,
        descriptors: &[K::Descriptor; N],
    ) -> Result<(), OutOfMemory> {
        for i in 0..N {
            <T as DescriptorBinding<K>>::update_descriptor(
                &mut self[i],
                device,
                encoder,
                &descriptors[i],
            )?;
        }
        Ok(())
    }

    #[inline]
    fn get_descriptors(&self, device: &Device) -> Result<[K::Descriptor; N], OutOfMemory> {
        Ok(array_fu::array![i => <T as DescriptorBinding<K>>::get_descriptor(&self[i], device)?; N])
    }
}

impl<T, K, const N: usize> DescriptorBindingArray<K> for arrayvec::ArrayVec<T, N>
where
    K: DescriptorKind,
    T: DescriptorBinding<K>,
{
    const COUNT: u32 = N as u32;
    const FLAGS: DescriptorBindingFlags = with_partially_bound(<T as DescriptorBinding<K>>::FLAGS);
    type DescriptorArray = arrayvec::ArrayVec<K::Descriptor, N>;

    #[inline]
    fn is_compatible(&self, descriptors: &arrayvec::ArrayVec<K::Descriptor, N>) -> bool {
        if self.len() != descriptors.len() {
            return false;
        }

        for i in 0..self.len() {
            if !<T as DescriptorBinding<K>>::is_compatible(&self[i], &descriptors[i]) {
                return false;
            }
        }
        true
    }

    #[inline]
    fn update_descriptors(
        &mut self,
        device: &Device,
        encoder: &mut Encoder,
        descriptors: &arrayvec::ArrayVec<K::Descriptor, N>,
    ) -> Result<(), OutOfMemory> {
        assert_eq!(self.len(), descriptors.len());
        for (i, elem) in self.iter_mut().enumerate() {
            <T as DescriptorBinding<K>>::update_descriptor(elem, device, encoder, &descriptors[i])?;
        }
        Ok(())
    }

    #[inline]
    fn get_descriptors(
        &self,
        device: &Device,
    ) -> Result<arrayvec::ArrayVec<K::Descriptor, N>, OutOfMemory> {
        let mut result = arrayvec::ArrayVec::new_const();

        for elem in self {
            result.push(elem.get_descriptor(device)?);
        }

        Ok(result)
    }
}

macro_rules! impl_for_refs {
    (impl[$($bounds:tt)+] for $t:ty) => {
        impl_for_refs!(exclusive impl[$($bounds)+] for &mut $t);
        impl_for_refs!(exclusive impl[$($bounds)+] for Box<$t>);
        // impl_for_refs!(shared impl[$($bounds)+] for &$t);
        // impl_for_refs!(shared impl[$($bounds)+] for Rc<$t>);
        // impl_for_refs!(shared impl[$($bounds)+] for Arc<$t>);
    };

    (exclusive impl[$($bounds:tt)+] for $ref_ty:ty) => {
        impl< $($bounds)+ > DescriptorBinding<K> for $ref_ty
        where
            K: DescriptorKind,
            T: DescriptorBinding<K> + Sealed,
        {
            const FLAGS: DescriptorBindingFlags = T::FLAGS;

            #[inline]
            fn is_compatible(&self, descriptor: &K::Descriptor) -> bool {
                T::is_compatible(self, descriptor)
            }

            #[inline]
            fn update_descriptor(
                &mut self,
                device: &Device,
                encoder: &mut Encoder,
                descriptor: &K::Descriptor,
            ) -> Result<(), OutOfMemory> {
                T::update_descriptor(self, device, encoder, descriptor)
            }

            #[inline]
            fn get_descriptor(
                &self,
                device: &Device,
            ) -> Result<K::Descriptor, OutOfMemory> {
                T::get_descriptor(&self, device)
            }
        }
    };

    (shared impl[$($bounds:tt)+] for $ref_ty:ty) => {
        impl< $($bounds)+ > DescriptorBinding<K> for $ref_ty
        where
            K: DescriptorKind,
            T: ShareableDescriptorBinding<K> + DescriptorBinding<K> + Sealed,

        {
            const FLAGS: DescriptorBindingFlags = T::FLAGS;

            #[inline]
            fn is_compatible(&self, descriptor: &K::Descriptor) -> bool {
                T::is_compatible(self, descriptor)
            }

            #[inline]
            fn get_descriptor(
                &self,
                device: &Device,
            ) -> Result<K::Descriptor, OutOfMemory> {
                T::get_descriptor(&self, device)
            }
        }
    };
}

impl_for_refs!(impl[K, T] for T);

const fn with_partially_bound(flags: DescriptorBindingFlags) -> DescriptorBindingFlags {
    DescriptorBindingFlags::from_bits_truncate(
        flags.bits() | DescriptorBindingFlags::PARTIALLY_BOUND.bits(),
    )
}

macro_rules! impl_array_for_refs {
    (impl<K, T, const N: usize> for $t:ty where $flags:expr; $descriptors:ty) => {
        impl_array_for_refs!(exclusive impl<K, T, const N: usize> for &mut $t | $t where $flags; $descriptors);
        impl_array_for_refs!(exclusive impl<K, T, const N: usize> for Box<$t> | $t where $flags; $descriptors);
        // impl_array_for_refs!(shared impl<K, T, const N: usize> for &$t | $t where $flags; $descriptors);
        // impl_array_for_refs!(shared impl<K, T, const N: usize> for Rc<$t> | $t where $flags; $descriptors);
        // impl_array_for_refs!(shared impl<K, T, const N: usize> for Arc<$t> | $t where $flags; $descriptors);
    };

    (exclusive impl<K, T, const N: usize> for $ref_ty:ty | $t:ty where $flags:expr; $descriptors:ty) => {
        impl<K, T, const N: usize> DescriptorBindingArray<K> for $ref_ty
        where
            K: DescriptorKind,
            T: DescriptorBinding<K> + Sealed,
        {
            const COUNT: u32 = N as u32;
            const FLAGS: DescriptorBindingFlags = $flags;
            type DescriptorArray = $descriptors;

            #[inline]
            fn is_compatible(
                &self,
                descriptors: &$descriptors,
            ) -> bool {
                <$t>::is_compatible(self, descriptors)
            }

            #[inline]
            fn update_descriptors(
                &mut self,
                device: &Device,
                encoder: &mut Encoder,
                descriptors: &$descriptors
            ) -> Result<(), OutOfMemory> {
                <$t>::update_descriptors(self, device, encoder, descriptors)
            }

            #[inline]
            fn get_descriptors(
                &self,
                device: &Device,
            ) -> Result<$descriptors, OutOfMemory> {
                <$t>::get_descriptors(self, device)
            }
        }
    };

    (shared impl<K, T, const N: usize> for $ref_ty:ty | $t:ty where $flags:expr; $descriptors:ty) => {
        impl<K, T, const N: usize> DescriptorBindingArray<K> for $ref_ty
        where
            K: DescriptorKind,
            T: ShareableDescriptorBinding<K> + Sealed,
        {
            const COUNT: u32 = N as u32;
            const FLAGS: DescriptorBindingFlags = with_partially_bound(<T as DescriptorBinding<K>>::FLAGS);
            type DescriptorArray = $descriptors;

            #[inline]
            fn is_compatible(
                &self,
                descriptors: &$descriptors,
            ) -> bool {
                <$t>::is_compatible(self, descriptors)
            }

            #[inline]
            fn get_descriptors(
                &self,
                device: &Device,
            ) -> Result<$descriptors, OutOfMemory> {
                <$t>::get_descriptors(self, device)
            }
        }
    };
}

impl_array_for_refs!(impl<K, T, const N: usize> for [T; N] where T::FLAGS; [K::Descriptor; N]);
impl_array_for_refs!(impl<K, T, const N: usize> for arrayvec::ArrayVec<T, N> where with_partially_bound(T::FLAGS); arrayvec::ArrayVec<K::Descriptor, N>);
