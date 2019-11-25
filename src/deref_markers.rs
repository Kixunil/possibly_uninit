//! Traits used for marking safety properties of `Deref` and `DerefMut`

use core::ops::Deref;

/// An unsafe marker trait for types that deref to a stable address, even when
/// moved.
///
/// For example, this is implemented by Box, Vec, Rc, Arc and String, among
/// others. Even when a Box is moved, the underlying storage remains at a
/// fixed location.
///
/// It implies `SameDataRef`
pub unsafe trait StableDeref: SameDataDeref {}

unsafe impl<T: StableDeref> SameDataDeref for T {}

/// An unsafe marker trait for types that deref to the same data. (Not
/// necessarily stored at the same memory address.)
///
/// This is important because it's possible to implement `Deref` in such way to
/// return a different refernce each time it's called. It'd be very surprising
/// if anyone actually did that, but nevertheless, `unsafe` code must *not* rely
/// on it. Thus this `unsafe` marker ensuring it's not happening.
pub unsafe trait SameDataDeref: Deref {}

unsafe impl<T: ?Sized> StableDeref for &T {}
unsafe impl<T: ?Sized> StableDeref for &mut T {}
unsafe impl<T: ?Sized> StableDeref for core::cell::Ref<'_, T> {}
unsafe impl<T: ?Sized> StableDeref for core::cell::RefMut<'_, T> {}
unsafe impl<T: ?Sized> SameDataDeref for core::mem::ManuallyDrop<T> {}

#[cfg(feature = "alloc")]
mod alloc_impls {
    use super::*;

    unsafe impl<T: ?Sized> StableDeref for alloc::boxed::Box<T>  {}

    unsafe impl<T> StableDeref for alloc::vec::Vec<T>  {}

    unsafe impl StableDeref for alloc::string::String {}

    unsafe impl<T: ?Sized> StableDeref for alloc::rc::Rc<T> {}

    unsafe impl<T: ?Sized> StableDeref for alloc::sync::Arc<T> {}

    unsafe impl<T: core::cmp::Ord> StableDeref for alloc::collections::binary_heap::PeekMut<'_, T> {}

    unsafe impl<T: ?Sized + alloc::borrow::ToOwned> SameDataDeref for alloc::borrow::Cow<'_, T> {}
}

#[cfg(feature = "std")]
mod std_impls {
    use super::*;

    unsafe impl StableDeref for std::path::PathBuf {}

    unsafe impl StableDeref for std::ffi::CString {}

    unsafe impl StableDeref for std::ffi::OsString {}

    unsafe impl<T: ?Sized> StableDeref for std::sync::MutexGuard<'_, T> {}

    unsafe impl<T: ?Sized> StableDeref for std::sync::RwLockReadGuard<'_, T> {}

    unsafe impl<T: ?Sized> StableDeref for std::sync::RwLockWriteGuard<'_, T> {}

    unsafe impl StableDeref for std::io::IoSlice<'_> {}

    unsafe impl StableDeref for std::io::IoSliceMut<'_> {}

    unsafe impl<T> SameDataDeref for std::panic::AssertUnwindSafe<T> {}
}
