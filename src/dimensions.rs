use std::{
    cmp::{Ord, Ordering},
    convert::TryInto,
    fmt::Debug,
    ops::{Deref, DerefMut},
};

use arrayvec::ArrayVec;
use num_traits::{One, ToPrimitive, Zero};

/// Image size is defined to `u32` which is standard for graphics API of today.
pub type ImageSize = u32;

/// Multidimensional size.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Extent<const D: usize, T = ImageSize> {
    values: [T; D],
}

pub type Extent1<T = ImageSize> = Extent<1, T>;
pub type Extent2<T = ImageSize> = Extent<2, T>;
pub type Extent3<T = ImageSize> = Extent<3, T>;

#[derive(Debug)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
#[repr(C)]
pub struct Width<T> {
    pub width: T,
}

impl<T> Deref for Extent1<T> {
    type Target = Width<T>;

    #[inline]
    fn deref(&self) -> &Width<T> {
        unsafe { &*(self as *const _ as *const _) }
    }
}

impl<T> DerefMut for Extent1<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Width<T> {
        unsafe { &mut *(self as *mut _ as *mut _) }
    }
}

#[cfg(feature = "serde-1")]
impl<T> serde::Serialize for Extent1<T>
where
    T: serde::Serialize,
{
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.deref().serialize(serializer)
    }
}

#[cfg(feature = "serde-1")]
impl<'de, T> serde::Deserialize<'de> for Extent1<T>
where
    T: serde::Deserialize<'de>,
{
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let repr = Width::deserialize(deserializer)?;
        Ok(Extent {
            values: [repr.width],
        })
    }
}

impl<T> Extent1<T> {
    #[inline]
    pub const fn new(width: T) -> Self {
        Extent { values: [width] }
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
#[repr(C)]
pub struct WidthHeight<T> {
    pub width: T,
    pub height: T,
}

impl<T> Deref for Extent2<T> {
    type Target = WidthHeight<T>;

    #[inline]
    fn deref(&self) -> &WidthHeight<T> {
        unsafe { &*(self as *const _ as *const _) }
    }
}

impl<T> DerefMut for Extent2<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut WidthHeight<T> {
        unsafe { &mut *(self as *mut _ as *mut _) }
    }
}

#[cfg(feature = "serde-1")]
impl<T> serde::Serialize for Extent2<T>
where
    T: serde::Serialize,
{
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.deref().serialize(serializer)
    }
}

#[cfg(feature = "serde-1")]
impl<'de, T> serde::Deserialize<'de> for Extent2<T>
where
    T: serde::Deserialize<'de>,
{
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let repr = WidthHeight::deserialize(deserializer)?;
        Ok(Extent {
            values: [repr.width, repr.height],
        })
    }
}

impl<T> Extent2<T> {
    #[inline]
    pub const fn new(width: T, height: T) -> Self {
        Extent {
            values: [width, height],
        }
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
#[repr(C)]
pub struct WidthHeightDepth<T> {
    pub width: T,
    pub height: T,
    pub depth: T,
}

impl<T> Deref for Extent3<T> {
    type Target = WidthHeightDepth<T>;

    #[inline]
    fn deref(&self) -> &WidthHeightDepth<T> {
        unsafe { &*(self as *const _ as *const _) }
    }
}

impl<T> DerefMut for Extent3<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut WidthHeightDepth<T> {
        unsafe { &mut *(self as *mut _ as *mut _) }
    }
}

#[cfg(feature = "serde-1")]
impl<T> serde::Serialize for Extent3<T>
where
    T: serde::Serialize,
{
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.deref().serialize(serializer)
    }
}

#[cfg(feature = "serde-1")]
impl<'de, T> serde::Deserialize<'de> for Extent3<T>
where
    T: serde::Deserialize<'de>,
{
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let repr = WidthHeightDepth::deserialize(deserializer)?;
        Ok(Extent {
            values: [repr.width, repr.height, repr.depth],
        })
    }
}

impl<T> Extent3<T> {
    #[inline]
    pub const fn new(width: T, height: T, depth: T) -> Self {
        Extent {
            values: [width, height, depth],
        }
    }
}

impl<T> Extent1<T>
where
    T: One,
{
    #[inline]
    pub fn into_1d(self) -> Extent1<T> {
        self
    }

    #[inline]
    pub fn into_2d(self) -> Extent2<T> {
        let [width] = self.values;

        Extent {
            values: [width, T::one()],
        }
    }

    #[inline]
    pub fn into_3d(self) -> Extent3<T> {
        let [width] = self.values;

        Extent {
            values: [width, T::one(), T::one()],
        }
    }
}

impl<T> Extent2<T>
where
    T: One,
{
    #[inline]
    pub fn into_1d(self) -> Extent1<T> {
        let [width, _] = self.values;

        Extent { values: [width] }
    }

    #[inline]
    pub fn into_2d(self) -> Extent2<T> {
        self
    }

    #[inline]
    pub fn into_3d(self) -> Extent3<T> {
        let [width, height] = self.values;

        Extent {
            values: [width, height, T::one()],
        }
    }
}

impl<T> Extent3<T>
where
    T: One,
{
    #[inline]
    pub fn into_1d(self) -> Extent1<T> {
        let [width, _, _] = self.values;

        Extent { values: [width] }
    }

    #[inline]
    pub fn into_2d(self) -> Extent2<T> {
        let [width, height, _] = self.values;

        Extent {
            values: [width, height],
        }
    }

    #[inline]
    pub fn into_3d(self) -> Extent3<T> {
        self
    }
}

impl<const D: usize, T> Extent<D, T> {
    #[inline]
    pub fn pick<F>(self, rhs: Self, mut picker: F) -> Self
    where
        F: FnMut(T, T) -> T,
    {
        let mut lhs = ArrayVec::<T, D>::from(self.values);
        let mut rhs = ArrayVec::<T, D>::from(rhs.values);

        let mut values = [(); D].map(|()| picker(lhs.pop().unwrap(), rhs.pop().unwrap()));
        values.reverse();

        Extent { values }
    }

    #[inline]
    pub fn min(self, rhs: Self) -> Self
    where
        T: Ord,
    {
        self.pick(rhs, std::cmp::min)
    }

    #[inline]
    pub fn max(self, rhs: Self) -> Self
    where
        T: Ord,
    {
        self.pick(rhs, std::cmp::max)
    }

    #[inline]
    pub fn ones() -> Self
    where
        T: One,
    {
        Extent {
            values: array_fu::array![T::one(); D],
        }
    }

    #[inline]
    pub fn zeros() -> Self
    where
        T: Zero,
    {
        Extent {
            values: array_fu::array![T::zero(); D],
        }
    }
}

impl<T> Extent2<T>
where
    T: ToPrimitive,
{
    #[inline]
    pub fn aspect_ratio(&self) -> f32 {
        self.width.to_f32().unwrap_or(0.0) / self.height.to_f32().unwrap_or(std::f32::INFINITY)
    }
}

impl<const D: usize, T> PartialOrd for Extent<D, T>
where
    T: PartialOrd,
{
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        merge_ordering_iter(
            self.values
                .iter()
                .zip(other.values.iter())
                .map(|(lhs, rhs)| lhs.partial_cmp(rhs)),
        )
    }
}

/// Image offset is defined to `i32` which is standard for graphics API today.
pub type ImageOffset = i32;

/// Multidimensional offset.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]

pub struct Offset<const D: usize, T = ImageOffset> {
    values: [T; D],
}

pub type Offset1<T = ImageOffset> = Offset<1, T>;
pub type Offset2<T = ImageOffset> = Offset<2, T>;
pub type Offset3<T = ImageOffset> = Offset<3, T>;

#[derive(Debug)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
#[repr(C)]
pub struct X<T> {
    /// Width offset.
    pub x: T,
}

impl<T> Deref for Offset1<T> {
    type Target = X<T>;

    #[inline]
    fn deref(&self) -> &X<T> {
        unsafe { &*(self as *const _ as *const _) }
    }
}

impl<T> DerefMut for Offset1<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut X<T> {
        unsafe { &mut *(self as *mut _ as *mut _) }
    }
}

#[cfg(feature = "serde-1")]
impl serde::Serialize for Offset1<ImageOffset> {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.deref().serialize(serializer)
    }
}

#[cfg(feature = "serde-1")]
impl<'de> serde::Deserialize<'de> for Offset1<ImageOffset> {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let repr = X::deserialize(deserializer)?;
        Ok(Offset { values: [repr.x] })
    }
}

impl<T> Offset1<T> {
    #[inline]
    pub const fn new(x: T) -> Self {
        Offset { values: [x] }
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
#[repr(C)]
pub struct XY<T> {
    /// Width offset.
    pub x: T,

    /// Height offset.
    pub y: T,
}

impl<T> Deref for Offset2<T> {
    type Target = XY<T>;

    #[inline]
    fn deref(&self) -> &XY<T> {
        unsafe { &*(self as *const _ as *const _) }
    }
}

impl<T> DerefMut for Offset2<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut XY<T> {
        unsafe { &mut *(self as *mut _ as *mut _) }
    }
}

#[cfg(feature = "serde-1")]
impl serde::Serialize for Offset2<ImageOffset> {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.deref().serialize(serializer)
    }
}

#[cfg(feature = "serde-1")]
impl<'de> serde::Deserialize<'de> for Offset2<ImageOffset> {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let repr = XY::deserialize(deserializer)?;
        Ok(Offset {
            values: [repr.x, repr.y],
        })
    }
}

impl<T> Offset2<T> {
    #[inline]
    pub const fn new(x: T, y: T) -> Self {
        Offset { values: [x, y] }
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
#[repr(C)]
pub struct XYZ<T> {
    /// Width offset.
    pub x: T,

    /// Height offset.
    pub y: T,

    /// Depth offset.
    pub z: T,
}

impl<T> Deref for Offset3<T> {
    type Target = XYZ<T>;

    #[inline]
    fn deref(&self) -> &XYZ<T> {
        unsafe { &*(self as *const _ as *const _) }
    }
}

impl<T> DerefMut for Offset3<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut XYZ<T> {
        unsafe { &mut *(self as *mut _ as *mut _) }
    }
}

#[cfg(feature = "serde-1")]
impl serde::Serialize for Offset3<ImageOffset> {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.deref().serialize(serializer)
    }
}

#[cfg(feature = "serde-1")]
impl<'de> serde::Deserialize<'de> for Offset3<ImageOffset> {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let repr = XYZ::deserialize(deserializer)?;
        Ok(Offset {
            values: [repr.x, repr.y, repr.z],
        })
    }
}

impl<T> Offset3<T> {
    #[inline]
    pub const fn new(x: T, y: T, z: T) -> Self {
        Offset { values: [x, y, z] }
    }
}

macro_rules! zero_offset {
    ($($t:ty = $zero:expr),* $(,)?) => {
        $(
            impl<const D: usize> Offset<D, $t> {
                pub const ZERO: Self = Offset { values: [$zero; D] };
            }
        )*
    };
}

zero_offset! {
    i8 = 0,
    i16 = 0,
    i32 = 0,
    i64 = 0,
    isize = 0,
    u8 = 0,
    u16 = 0,
    u32 = 0,
    u64 = 0,
    usize = 0,
    f32 = 0.0,
    f64 = 0.0,
}

impl<const D: usize, T> Offset<D, T>
where
    T: Zero,
{
    #[inline]
    pub fn zeros() -> Self {
        Offset {
            values: array_fu::array![T::zero(); D],
        }
    }
}

impl<const D: usize, T> Offset<D, T> {
    #[inline]
    pub fn from_extent<U>(extent: Extent<D, U>) -> Result<Self, U::Error>
    where
        U: TryInto<T>,
    {
        let mut result = ArrayVec::<T, D>::new();

        for value in extent.values {
            result.push(value.try_into()?);
        }

        Ok(Offset {
            values: match result.into_inner() {
                Ok(values) => values,
                Err(_) => unreachable!(),
            },
        })
    }
}

#[inline]
fn merge_ordering(left: Ordering, right: Ordering) -> Option<Ordering> {
    match (left, right) {
        (Ordering::Equal, right) => Some(right),
        (left, Ordering::Equal) => Some(left),
        (Ordering::Less, Ordering::Less) => Some(Ordering::Less),
        (Ordering::Greater, Ordering::Greater) => Some(Ordering::Greater),
        _ => None,
    }
}

#[inline]
fn merge_ordering_iter(mut iter: impl Iterator<Item = Option<Ordering>>) -> Option<Ordering> {
    iter.try_fold(Ordering::Equal, |acc, item| merge_ordering(acc, item?))
}

#[allow(missing_debug_implementations)]
pub struct MinimalExtent<const D: usize, T> {
    values: Option<[T; D]>,
}

impl<const D: usize, T> MinimalExtent<D, T> {
    #[inline]
    pub const fn new() -> Self {
        MinimalExtent { values: None }
    }

    #[inline]
    pub fn add(&mut self, extent: Extent<D, T>)
    where
        T: Ord,
    {
        match &mut self.values {
            None => self.values = Some(extent.values),
            Some(values) => {
                for (value, other) in values.iter_mut().zip(extent.values) {
                    match Ord::cmp(&*value, &other) {
                        Ordering::Greater => *value = other,
                        _ => {}
                    }
                }
            }
        }
    }

    #[inline]
    pub fn get(self) -> Extent<D, T>
    where
        T: One,
    {
        self.get_or(Extent {
            values: array_fu::array![T::one(); D],
        })
    }

    #[inline]
    pub fn get_or(self, default: Extent<D, T>) -> Extent<D, T>
    where
        T: One,
    {
        match self.values {
            None => default,
            Some(values) => Extent { values },
        }
    }
}

pub fn minimal_extent<const D: usize, T>() -> MinimalExtent<D, T> {
    MinimalExtent::new()
}
