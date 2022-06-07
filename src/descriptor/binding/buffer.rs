use std::{
    alloc::Layout,
    mem::size_of,
    ops::{Deref, DerefMut},
};

use arrayvec::ArrayVec;
use smallvec::SmallVec;

use crate::{
    buffer::{Buffer, BufferInfo, BufferRange, BufferUsage, BufferView, BufferViewInfo},
    descriptor::{
        BufferDescriptor, DescriptorBinding, DescriptorBindingFlags, DynamicFormat, Storage,
        TexelBufferDescriptor, Uniform,
    },
    format::StaticFormat,
    DescriptorKind, Device, Encoder, Format, OutOfMemory,
};

trait BufferDescriptorKind: DescriptorKind<Descriptor = BufferRange> {
    const USAGE: BufferUsage;
}

impl<O> BufferDescriptorKind for BufferDescriptor<Uniform, O>
where
    Self: DescriptorKind<Descriptor = BufferRange>,
{
    const USAGE: BufferUsage = BufferUsage::UNIFORM;
}

impl<O> BufferDescriptorKind for BufferDescriptor<Storage, O>
where
    Self: DescriptorKind<Descriptor = BufferRange>,
{
    const USAGE: BufferUsage = BufferUsage::STORAGE;
}

trait TexelBufferDescriptorKind: DescriptorKind<Descriptor = BufferView> {
    const USAGE: BufferUsage;
}

impl<F> TexelBufferDescriptorKind for TexelBufferDescriptor<Uniform, F>
where
    Self: DescriptorKind<Descriptor = BufferView>,
{
    const USAGE: BufferUsage = BufferUsage::UNIFORM_TEXEL;
}

impl<F> TexelBufferDescriptorKind for TexelBufferDescriptor<Storage, F>
where
    Self: DescriptorKind<Descriptor = BufferView>,
{
    const USAGE: BufferUsage = BufferUsage::STORAGE_TEXEL;
}

impl<S, O> DescriptorBinding<BufferDescriptor<S, O>> for Buffer
where
    BufferDescriptor<S, O>: BufferDescriptorKind,
{
    const FLAGS: DescriptorBindingFlags = DescriptorBindingFlags::empty();

    #[inline]
    fn is_compatible(&self, range: &BufferRange) -> bool {
        range.buffer == *self && range.offset == 0 && range.size == self.info().size
    }

    #[inline]
    fn get_descriptor(&self, _device: &Device) -> Result<BufferRange, OutOfMemory> {
        assert!(
            self.info().usage.contains(<BufferDescriptor<S, O>>::USAGE),
            "Missing usage flags {:?} for buffer descriptor",
            <BufferDescriptor<S, O>>::USAGE,
        );
        Ok(BufferRange::whole(self.clone()))
    }
}

impl<S, F> DescriptorBinding<TexelBufferDescriptor<S, F>> for Buffer
where
    TexelBufferDescriptor<S, F>: TexelBufferDescriptorKind,
    F: StaticFormat,
{
    const FLAGS: DescriptorBindingFlags = DescriptorBindingFlags::empty();

    #[inline]
    fn is_compatible(&self, view: &BufferView) -> bool {
        view.info().buffer == *self
    }

    #[inline]
    fn get_descriptor(&self, device: &Device) -> Result<BufferView, OutOfMemory> {
        assert!(
            self.info()
                .usage
                .contains(<TexelBufferDescriptor<S, F>>::USAGE),
            "Missing usage flags {:?} for buffer descriptor",
            <TexelBufferDescriptor<S, F>>::USAGE
        );

        device.create_buffer_view(BufferViewInfo {
            buffer: self.clone(),
            format: F::FORMAT,
            offset: 0,
            size: self.info().size,
        })
    }
}

impl<S, O> DescriptorBinding<BufferDescriptor<S, O>> for BufferRange
where
    BufferDescriptor<S, O>: BufferDescriptorKind,
{
    const FLAGS: DescriptorBindingFlags = DescriptorBindingFlags::empty();

    #[inline]
    fn is_compatible(&self, range: &BufferRange) -> bool {
        *self == *range
    }

    #[inline]
    fn get_descriptor(&self, _device: &Device) -> Result<BufferRange, OutOfMemory> {
        assert!(
            self.buffer
                .info()
                .usage
                .contains(<BufferDescriptor<S, O>>::USAGE),
            "Missing usage flags {:?} for buffer descriptor",
            <BufferDescriptor<S, O>>::USAGE
        );

        Ok(self.clone())
    }
}

impl<S, F> DescriptorBinding<TexelBufferDescriptor<S, F>> for BufferRange
where
    TexelBufferDescriptor<S, F>: TexelBufferDescriptorKind,
    F: StaticFormat,
{
    const FLAGS: DescriptorBindingFlags = DescriptorBindingFlags::empty();

    #[inline]
    fn is_compatible(&self, view: &BufferView) -> bool {
        let info = view.info();
        info.buffer == self.buffer && info.offset == self.offset && info.size == self.size
    }

    #[inline]
    fn get_descriptor(&self, device: &Device) -> Result<BufferView, OutOfMemory> {
        assert!(
            self.buffer
                .info()
                .usage
                .contains(<TexelBufferDescriptor<S, F>>::USAGE),
            "Missing usage flags {:?} for buffer descriptor",
            <TexelBufferDescriptor<S, F>>::USAGE
        );

        device.create_buffer_view(BufferViewInfo {
            buffer: self.buffer.clone(),
            format: F::FORMAT,
            offset: self.offset,
            size: self.size,
        })
    }
}

impl<S> DescriptorBinding<TexelBufferDescriptor<S, DynamicFormat>> for BufferView
where
    TexelBufferDescriptor<S, DynamicFormat>: TexelBufferDescriptorKind,
{
    const FLAGS: DescriptorBindingFlags = DescriptorBindingFlags::empty();

    #[inline]
    fn is_compatible(&self, view: &BufferView) -> bool {
        *self == *view
    }

    #[inline]
    fn get_descriptor(&self, _device: &Device) -> Result<BufferView, OutOfMemory> {
        assert!(
            self.info()
                .buffer
                .info()
                .usage
                .contains(<TexelBufferDescriptor<S, DynamicFormat>>::USAGE),
            "Missing usage flags {:?} for buffer descriptor",
            <TexelBufferDescriptor<S, DynamicFormat>>::USAGE
        );

        Ok(self.clone())
    }
}

impl<S, F> DescriptorBinding<TexelBufferDescriptor<S, F>> for BufferView
where
    TexelBufferDescriptor<S, F>: TexelBufferDescriptorKind,
    F: StaticFormat,
{
    const FLAGS: DescriptorBindingFlags = DescriptorBindingFlags::empty();

    #[inline]
    fn is_compatible(&self, view: &BufferView) -> bool {
        *self == *view
    }

    #[inline]
    fn get_descriptor(&self, _device: &Device) -> Result<BufferView, OutOfMemory> {
        assert!(
            self.info()
                .buffer
                .info()
                .usage
                .contains(<TexelBufferDescriptor<S, F>>::USAGE),
            "Missing usage flags {:?} for buffer descriptor",
            <TexelBufferDescriptor<S, F>>::USAGE
        );

        assert_eq!(self.info().format, F::FORMAT, "Wrong view format");

        Ok(self.clone())
    }
}

#[derive(Clone, Copy, Debug)]
pub struct TypedBuffer<T> {
    data: T,
    is_dirty: bool,
}

impl<T> Default for TypedBuffer<T>
where
    T: Default,
{
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T> TypedBuffer<T> {
    pub const fn new(data: T) -> Self {
        TypedBuffer {
            data,
            is_dirty: true,
        }
    }

    #[inline]
    fn update_descriptor(
        &mut self,
        device: &Device,
        encoder: &mut Encoder,
        range: &BufferRange,
    ) -> Result<(), OutOfMemory>
    where
        T: TypedBufferData,
    {
        if self.is_dirty {
            let data = self.data.raw();
            let scope = encoder.scope();

            let slice = scope.alloc(Layout::array::<u8>(data.len()).unwrap());

            let slice: &[u8] = unsafe {
                std::ptr::copy_nonoverlapping(
                    data.as_ptr(),
                    slice.as_mut_ptr() as *mut u8,
                    data.len(),
                );
                std::slice::from_raw_parts(slice.as_ptr() as *mut u8, slice.len())
            };

            encoder.upload_buffer(
                scope.to_scope(range.buffer.clone()),
                range.offset,
                slice,
                device,
            )?;
            self.is_dirty = false;
        }
        Ok(())
    }
}

impl<T> Deref for TypedBuffer<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.data
    }
}

impl<T> DerefMut for TypedBuffer<T> {
    fn deref_mut(&mut self) -> &mut T {
        self.is_dirty = true;
        &mut self.data
    }
}

pub trait TypedBufferData {
    fn raw(&self) -> &[u8];
}

impl<T> DescriptorBinding<BufferDescriptor<Storage>> for TypedBuffer<T>
where
    T: TypedBufferData,
{
    const FLAGS: DescriptorBindingFlags = DescriptorBindingFlags::empty();

    #[inline]
    fn is_compatible(&self, range: &BufferRange) -> bool {
        range.size >= self.data.raw().len() as u64
    }

    #[inline]
    fn update_descriptor(
        &mut self,
        device: &Device,
        encoder: &mut Encoder,
        range: &BufferRange,
    ) -> Result<(), OutOfMemory> {
        Self::update_descriptor(self, device, encoder, range)
    }

    #[inline]
    fn get_descriptor(&self, device: &Device) -> Result<BufferRange, OutOfMemory> {
        let size = self.data.raw().len() as u64;
        let buffer = device.create_buffer_static(
            BufferInfo {
                align: 255,
                size,
                usage: BufferUsage::STORAGE | BufferUsage::TRANSFER_DST,
            },
            self.data.raw(),
        )?;
        Ok(BufferRange {
            buffer,
            offset: 0,
            size,
        })
    }
}

pub trait UniformTypedBufferData: TypedBufferData {
    /// Static data size.
    fn static_size() -> usize;
}

impl<T> DescriptorBinding<BufferDescriptor<Uniform>> for TypedBuffer<T>
where
    T: UniformTypedBufferData,
{
    const FLAGS: DescriptorBindingFlags = DescriptorBindingFlags::empty();

    #[inline]
    fn is_compatible(&self, range: &BufferRange) -> bool {
        range.size >= T::static_size() as u64
    }

    #[inline]
    fn update_descriptor(
        &mut self,
        device: &Device,
        encoder: &mut Encoder,
        range: &BufferRange,
    ) -> Result<(), OutOfMemory> {
        Self::update_descriptor(self, device, encoder, range)
    }

    #[inline]
    fn get_descriptor(&self, device: &Device) -> Result<BufferRange, OutOfMemory> {
        let size = T::static_size() as u64;
        let buffer = device.create_buffer_static(
            BufferInfo {
                align: 255,
                size,
                usage: BufferUsage::UNIFORM | BufferUsage::TRANSFER_DST,
            },
            self.data.raw(),
        )?;
        Ok(BufferRange {
            buffer,
            offset: 0,
            size,
        })
    }
}

/// Trait implemented for valid texel types.
pub trait TexelBufferData: TypedBufferData {
    const FORMAT: Format;
}

pub trait UniformTexelBufferData: UniformTypedBufferData + TexelBufferData {}

impl<T> UniformTexelBufferData for T where T: UniformTypedBufferData + TexelBufferData {}

impl<T, const N: usize> TypedBufferData for [T; N]
where
    T: bytemuck::Pod,
{
    fn raw(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
}

impl<T, const N: usize> UniformTypedBufferData for [T; N]
where
    T: bytemuck::Pod,
{
    fn static_size() -> usize {
        size_of::<Self>()
    }
}

impl<T> TypedBufferData for Vec<T>
where
    T: bytemuck::Pod,
{
    fn raw(&self) -> &[u8] {
        bytemuck::cast_slice(&self[..])
    }
}

impl<T, const N: usize> TypedBufferData for ArrayVec<T, N>
where
    T: bytemuck::Pod,
{
    fn raw(&self) -> &[u8] {
        bytemuck::cast_slice(&self[..])
    }
}

impl<T, const N: usize> TypedBufferData for SmallVec<[T; N]>
where
    T: bytemuck::Pod,
{
    fn raw(&self) -> &[u8] {
        bytemuck::cast_slice(&self[..])
    }
}
