//! Shared resource
//!
//! This is used when a resource must be shared between multiple other resources
//!
//! The implementation depends on the `threading` feature being enabled

use std::fmt;
use std::ops::Deref;

/// Trait that should be implemented for cyclic shared resources
pub trait CyclicSharedResource<T: Clone> {
    /// When this object is deep-cloned, this function gets called
    /// So that it can change out all self references to the new `SharedResourceWeakRef`
    fn on_cloned(&mut self, new_me: &SharedResourceWeakRef<T>);
}

/// Shared resource
pub struct SharedResource<T: ?Sized> {
    #[cfg(not(feature = "threading"))]
    resource: std::rc::Rc<std::cell::RefCell<T>>,
    #[cfg(feature = "threading")]
    resource: std::sync::Arc<std::sync::RwLock<T>>,
}

impl<T> SharedResource<T> {
    /// Create a new `SharedResource` instance
    #[cfg(not(feature = "threading"))]
    pub fn new(value: T) -> Self {
        SharedResource {
            resource: std::rc::Rc::new(std::cell::RefCell::new(value)),
        }
    }

    /// Create a new `SharedResource` instance
    #[cfg(feature = "threading")]
    pub fn new(value: T) -> Self {
        SharedResource {
            resource: std::sync::Arc::new(std::sync::RwLock::new(value)),
        }
    }

    /// Create a new cyclic `SharedResource` instance
    #[cfg(not(feature = "threading"))]
    pub fn new_cyclic<F: FnOnce(&SharedResourceWeakRef<T>) -> T>(data_fn: F) -> SharedResource<T> {
        let resource = std::rc::Rc::new_cyclic(|me| {
            std::cell::RefCell::new(data_fn(&SharedResourceWeakRef::new(me.clone())))
        });
        SharedResource { resource }
    }

    /// Create a new cyclic `SharedResource` instance
    #[cfg(feature = "threading")]
    pub fn new_cyclic<F: FnOnce(&SharedResourceWeakRef<T>) -> T>(data_fn: F) -> SharedResource<T> {
        let resource = std::sync::Arc::new_cyclic(|me| {
            std::sync::RwLock::new(data_fn(&SharedResourceWeakRef::new(me.clone())))
        });
        SharedResource { resource }
    }

    /// Get a reference to the value inside of this `SharedResource`
    ///
    /// # Panics
    ///
    /// In both scenarios panics if the value is already mutably borrowed
    ///
    /// In a multithreaded scenario panics if the lock was poisoned
    #[cfg(not(feature = "threading"))]
    pub fn get_ref(&self) -> std::cell::Ref<'_, T> {
        self.resource.borrow()
    }

    /// Get a reference to the value inside of this `SharedResource`
    ///
    /// # Panics
    ///
    /// In both scenarios panics if the value is already mutably borrowed
    ///
    /// In a multithreaded scenario panics if the lock was poisoned
    #[cfg(feature = "threading")]
    pub fn get_ref(&self) -> std::sync::RwLockReadGuard<'_, T> {
        self.resource.read().unwrap()
    }

    /// Get a mutable reference to the value inside of this `SharedResource`
    ///
    /// # Panics
    ///
    /// In a singlethreaded scenario panics if the value is already borrowed
    ///
    /// In a multithreaded scenario panics if the lock was poisoned
    #[cfg(not(feature = "threading"))]
    pub fn get_mut(&mut self) -> std::cell::RefMut<'_, T> {
        self.resource.borrow_mut()
    }

    /// Get a mutable reference to the value inside of this `SharedResource`
    ///
    /// # Panics
    ///
    /// In a singlethreaded scenario panics if the value is already borrowed
    ///
    /// In a multithreaded scenario panics if the lock was poisoned
    #[cfg(feature = "threading")]
    pub fn get_mut(&mut self) -> std::sync::RwLockWriteGuard<T> {
        self.resource.write().unwrap()
    }
}

impl<T: CyclicSharedResource<T> + Clone> SharedResource<T> {
    /// Clone this shared resource with the value inside of it
    pub fn clone_resource(&self) -> SharedResource<T> {
        let mut cloned_resource = self.get_ref().clone();
        SharedResource::new_cyclic(|me| {
            cloned_resource.on_cloned(me);
            cloned_resource
        })
    }
}

#[cfg(not(feature = "threading"))]
impl<T: ?Sized> Deref for SharedResource<T> {
    type Target = std::cell::RefCell<T>;

    fn deref(&self) -> &Self::Target {
        self.resource.deref()
    }
}

#[cfg(feature = "threading")]
impl<T: ?Sized> Deref for SharedResource<T> {
    type Target = std::sync::RwLock<T>;

    fn deref(&self) -> &Self::Target {
        self.resource.deref()
    }
}

impl<T: Default> Default for SharedResource<T> {
    fn default() -> Self {
        Self {
            resource: Default::default(),
        }
    }
}

#[cfg(not(feature = "threading"))]
impl<T: ?Sized + PartialEq> PartialEq for SharedResource<T> {
    fn eq(&self, other: &Self) -> bool {
        self.resource == other.resource
    }
}

#[cfg(not(feature = "threading"))]
impl<T: ?Sized + Eq> Eq for SharedResource<T> {}

#[cfg(not(feature = "threading"))]
impl<T: ?Sized + PartialOrd> PartialOrd for SharedResource<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.resource.partial_cmp(&other.resource)
    }
}

#[cfg(not(feature = "threading"))]
impl<T: ?Sized + Ord> Ord for SharedResource<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.resource.cmp(&other.resource)
    }
}

impl<T: ?Sized + fmt::Debug> fmt::Debug for SharedResource<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.resource, f)
    }
}

impl<T: ?Sized> Clone for SharedResource<T> {
    fn clone(&self) -> Self {
        Self {
            resource: self.resource.clone(),
        }
    }
}

/// Weak reference to a `SharedResource`
pub struct SharedResourceWeakRef<T: ?Sized> {
    #[cfg(not(feature = "threading"))]
    resource: std::rc::Weak<std::cell::RefCell<T>>,
    #[cfg(feature = "threading")]
    resource: std::sync::Weak<std::sync::RwLock<T>>,
}

impl<T> SharedResourceWeakRef<T> {
    /// Create a new `SharedResourceWeakRef`
    #[cfg(not(feature = "threading"))]
    pub fn new(resource: std::rc::Weak<std::cell::RefCell<T>>) -> Self {
        SharedResourceWeakRef { resource }
    }

    /// Create a new `SharedResourceWeakRef`
    #[cfg(feature = "threading")]
    pub fn new(resource: std::sync::Weak<std::sync::RwLock<T>>) -> Self {
        SharedResourceWeakRef { resource }
    }

    /// Attempts to upgrade a `SharedResourceWeakRef` to a `SharedResource`
    ///
    /// Returns [`None`] if the inner value has since been dropped
    pub fn upgrade(&self) -> Option<SharedResource<T>> {
        self.resource
            .upgrade()
            .map(|e| SharedResource { resource: e })
    }
}

impl<T: ?Sized> Clone for SharedResourceWeakRef<T> {
    fn clone(&self) -> Self {
        Self {
            resource: self.resource.clone(),
        }
    }
}

impl<T: ?Sized> fmt::Debug for SharedResourceWeakRef<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SharedResourceWeakRef")
            .field("resource", &self.resource)
            .finish()
    }
}
