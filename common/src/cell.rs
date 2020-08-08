#[cfg(feature = "threading")]
use parking_lot::{Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};
#[cfg(not(feature = "threading"))]
use std::cell::{Ref, RefCell, RefMut};
use std::ops::{Deref, DerefMut};

cfg_if::cfg_if! {
    if #[cfg(feature = "threading")] {
        pub use once_cell::sync::OnceCell;
    } else {
        pub use once_cell::unsync::OnceCell;
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "threading")] {
        type MutexInner<T> = Mutex<T>;
        type MutexGuardInner<'a, T> = MutexGuard<'a, T>;
        const fn new_mutex<T>(value: T) -> MutexInner<T> {
            parking_lot::const_mutex(value)
        }
        fn lock_mutex<T: ?Sized>(m: &MutexInner<T>) -> MutexGuardInner<T> {
            m.lock()
        }
    } else {
        type MutexInner<T> = RefCell<T>;
        type MutexGuardInner<'a, T> = RefMut<'a, T>;
        const fn new_mutex<T>(value: T) -> MutexInner<T> {
            RefCell::new(value)
        }
        fn lock_mutex<T: ?Sized>(m: &MutexInner<T>) -> MutexGuardInner<T> {
            m.borrow_mut()
        }
    }
}

#[derive(Debug, Default)]
#[repr(transparent)]
pub struct PyMutex<T: ?Sized>(MutexInner<T>);

impl<T> PyMutex<T> {
    pub const fn new(value: T) -> Self {
        Self(new_mutex(value))
    }
}

impl<T: ?Sized> PyMutex<T> {
    pub fn lock(&self) -> PyMutexGuard<T> {
        PyMutexGuard(lock_mutex(&self.0))
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct PyMutexGuard<'a, T: ?Sized>(MutexGuardInner<'a, T>);
impl<T: ?Sized> Deref for PyMutexGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.0.deref()
    }
}
impl<T: ?Sized> DerefMut for PyMutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.0.deref_mut()
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "threading")] {
        type RwLockInner<T> = RwLock<T>;
        type RwLockReadInner<'a, T> = RwLockReadGuard<'a, T>;
        type RwLockWriteInner<'a, T> = RwLockWriteGuard<'a, T>;
        const fn new_rwlock<T>(value: T) -> RwLockInner<T> {
            parking_lot::const_rwlock(value)
        }
        fn read_rwlock<T: ?Sized>(m: &RwLockInner<T>) -> RwLockReadInner<T> {
            m.read()
        }
        fn write_rwlock<T: ?Sized>(m: &RwLockInner<T>) -> RwLockWriteInner<T> {
            m.write()
        }
    } else {
        type RwLockInner<T> = RefCell<T>;
        type RwLockReadInner<'a, T> = Ref<'a, T>;
        type RwLockWriteInner<'a, T> = RefMut<'a, T>;
        const fn new_rwlock<T>(value: T) -> RwLockInner<T> {
            RefCell::new(value)
        }
        fn read_rwlock<T: ?Sized>(m: &RwLockInner<T>) -> RwLockReadInner<T> {
            m.borrow()
        }
        fn write_rwlock<T: ?Sized>(m: &RwLockInner<T>) -> RwLockWriteInner<T> {
            m.borrow_mut()
        }
    }
}

#[derive(Debug, Default)]
#[repr(transparent)]
pub struct PyRwLock<T: ?Sized>(RwLockInner<T>);

impl<T> PyRwLock<T> {
    pub const fn new(value: T) -> Self {
        Self(new_rwlock(value))
    }
}

impl<T: ?Sized> PyRwLock<T> {
    pub fn read(&self) -> PyRwLockReadGuard<T> {
        PyRwLockReadGuard(read_rwlock(&self.0))
    }
    pub fn write(&self) -> PyRwLockWriteGuard<T> {
        PyRwLockWriteGuard(write_rwlock(&self.0))
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct PyRwLockReadGuard<'a, T: ?Sized>(RwLockReadInner<'a, T>);
impl<T: ?Sized> Deref for PyRwLockReadGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.0.deref()
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct PyRwLockWriteGuard<'a, T: ?Sized>(RwLockWriteInner<'a, T>);
impl<T: ?Sized> Deref for PyRwLockWriteGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.0.deref()
    }
}
impl<T: ?Sized> DerefMut for PyRwLockWriteGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.0.deref_mut()
    }
}
