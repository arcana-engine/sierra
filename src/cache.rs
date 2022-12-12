use std::{
    borrow::Borrow,
    convert::Infallible,
    hash::Hash,
    ops::{Deref, DerefMut},
};

use crate::{
    image::Image,
    view::{ImageView, ImageViewInfo},
    Device, OutOfMemory,
};

/// General purpose cache for sierra resources.
/// This cache evicts resources based on their last used epoch.
/// Each time a resource is used, its last used epoch is updated to the current.
/// The cache evicts all resources that have not been used for a certain amount of epochs.
///
/// This strategy works best for resources that may become obsolete and require substantial amount of memory
/// while only few resources are in use so cache size is always low.
#[derive(Debug)]
pub struct ResourceCache<T> {
    /// Resources in the cache.
    /// Key is a resource.
    ///
    /// Value is a last used time.
    resources: Vec<(T, u64)>,

    /// Current epoch of the cache.
    current_epoch: u64,
}

impl<T> ResourceCache<T> {
    /// Creates a new empty cache.
    pub const fn new() -> Self {
        Self {
            resources: Vec::new(),
            current_epoch: 0,
        }
    }

    /// Creates a new cache with preallocated resource capacity.
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            resources: Vec::with_capacity(cap),
            current_epoch: 0,
        }
    }

    /// Fetches resource from cache.
    /// Resource last used epoch is updated to the current epoch.
    /// Returns `None` if resource is not in cache.
    /// Returns `Some` if resource is in cache.
    pub fn fetch<F>(&mut self, eq: F) -> Option<&T>
    where
        F: Fn(&T) -> bool,
    {
        let (r, e) = self.resources.iter_mut().find(|(r, _)| eq(r))?;
        *e = self.current_epoch;
        Some(&*r)
    }

    /// Fetches resource from cache.
    /// Resource last used epoch is updated to the current epoch.
    /// Returns `None` if resource is not in cache.
    /// Returns `Some` if resource is in cache.
    pub fn get<Q>(&mut self, key: &Q) -> Option<&T>
    where
        T: Borrow<Q>,
        Q: ?Sized + Hash + Eq,
    {
        self.fetch(|r| r.borrow() == key)
    }

    /// Fetches resource from cache.
    /// Resource last used epoch is not updated.
    /// This is useful when only shared reference to cache is available.
    /// Returns `None` if resource is not in cache.
    /// Returns `Some` if resource is in cache.
    pub fn fetch_no_update<F>(&self, eq: F) -> Option<&T>
    where
        F: Fn(&T) -> bool,
    {
        self.resources.iter().find(|(r, _)| eq(r)).map(|(r, _)| r)
    }

    /// Fetches resource from cache.
    /// Resource last used epoch is not updated.
    /// This is useful when only shared reference to cache is available.
    /// Returns `None` if resource is not in cache.
    /// Returns `Some` if resource is in cache.
    pub fn get_no_update<Q>(&self, key: &Q) -> Option<&T>
    where
        T: Borrow<Q>,
        Q: ?Sized + Hash + Eq,
    {
        self.fetch_no_update(|r| r.borrow() == key)
    }

    /// Fetches resource from cache.
    /// Resource last used epoch is updated to the current epoch.
    /// If resource is not in cache, it is created and added to the cache.
    /// Returns a reference to the resource.
    /// Returns error if resource is not in cache and create function fails.
    pub fn try_fetch_or_create<K, F, E>(&mut self, eq: K, create: F) -> Result<&T, E>
    where
        K: Fn(&T) -> bool,
        F: FnOnce() -> Result<T, E>,
    {
        let idx = match self.resources.iter().position(|(r, _)| eq(r)) {
            None => {
                let r = create()?;
                self.resources.push((r, self.current_epoch));
                self.resources.len() - 1
            }
            Some(idx) => idx,
        };

        let (r, e) = &mut self.resources[idx];
        *e = self.current_epoch;
        Ok(r)
    }

    /// Fetches resource from cache.
    /// Resource last used epoch is updated to the current epoch.
    /// If resource is not in cache, it is created and added to the cache.
    /// Returns a reference to the resource.
    pub fn fetch_or_create<E, F>(&mut self, eq: E, create: F) -> &T
    where
        E: Fn(&T) -> bool,
        F: FnOnce() -> T,
    {
        match self.try_fetch_or_create(eq, || Ok::<_, Infallible>(create())) {
            Ok(r) => r,
            Err(infallible) => match infallible {},
        }
    }

    /// Fetches resource from cache.
    /// Resource last used epoch is updated to the current epoch.
    /// If resource is not in cache, it is created and added to the cache.
    /// Returns a reference to the resource.
    pub fn try_get_or_create<Q, F, E>(&mut self, key: &Q, create: F) -> Result<&T, E>
    where
        T: Borrow<Q>,
        Q: ?Sized + Hash + Eq,
        F: FnOnce() -> Result<T, E>,
    {
        self.try_fetch_or_create(|r| r.borrow() == key, create)
    }

    /// Fetches resource from cache.
    /// Resource last used epoch is updated to the current epoch.
    /// If resource is not in cache, it is created and added to the cache.
    /// Returns a reference to the resource.
    pub fn get_or_create<Q, F>(&mut self, key: &Q, create: F) -> &T
    where
        T: Borrow<Q>,
        Q: ?Sized + Hash + Eq,
        F: FnOnce() -> T,
    {
        self.fetch_or_create(|r| r.borrow() == key, create)
    }

    /// Moves to the next epoch.
    pub fn next_epoch(&mut self) {
        self.current_epoch += 1;
    }

    /// Evicts resources that have not been used since specified epoch.
    pub fn evict(&mut self, epoch: u64) {
        self.resources.retain(|(_, e)| *e >= epoch);
    }
}

/// Cache for image views
/// This cache uses eviction strategy of [`ResourceCache`].
/// But has convenience methods for fetching image views.
#[derive(Debug)]
pub struct ImageViewCache {
    cache: ResourceCache<ImageView>,
}

impl Deref for ImageViewCache {
    type Target = ResourceCache<ImageView>;

    fn deref(&self) -> &Self::Target {
        &self.cache
    }
}

impl DerefMut for ImageViewCache {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.cache
    }
}

impl ImageViewCache {
    /// Creates a new empty cache.
    pub const fn new() -> Self {
        Self {
            cache: ResourceCache::new(),
        }
    }

    /// Creates a new cache with preallocated resource capacity.
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            cache: ResourceCache::with_capacity(cap),
        }
    }

    /// Fetches image view for specified image.
    /// Returns `None` if image is not in cache.
    /// Returns `Some` if image is in cache.
    pub fn fetch_image(&mut self, image: &Image) -> Option<&ImageView> {
        self.cache.fetch(|r| r.info().is_whole_image(image))
    }

    /// Fetches image view for specified image.
    /// Returns `None` if image is not in cache.
    /// Returns `Some` if image is in cache.
    pub fn fetch_image_view(&mut self, info: &ImageViewInfo) -> Option<&ImageView> {
        self.cache.fetch(|r| r.info() == info)
    }

    /// Fetches image view for specified image.
    /// Returns `None` if image is not in cache.
    /// Returns `Some` if image is in cache.
    pub fn make_image(
        &mut self,
        image: &Image,
        device: &Device,
    ) -> Result<&ImageView, OutOfMemory> {
        self.cache.try_fetch_or_create(
            |view| view.info().is_whole_image(image),
            || {
                let info = ImageViewInfo::new(image.clone());
                device.create_image_view(info)
            },
        )
    }

    /// Fetches image view for specified image view info.
    /// Returns `None` if image is not in cache.
    /// Returns `Some` if image is in cache.
    pub fn make_image_view(
        &mut self,
        info: &ImageViewInfo,
        device: &Device,
    ) -> Result<&ImageView, OutOfMemory> {
        self.cache.try_fetch_or_create(
            |view| view.info() == info,
            || device.create_image_view(info.clone()),
        )
    }
}
