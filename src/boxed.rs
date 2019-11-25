//! Helpers for using boxes.

use alloc::boxed::Box;
use core::mem::MaybeUninit;

/// Extension trait allowing placement new operation.
pub trait BoxExt {
    /// Type stored in the box.
    type Stored: ?Sized;

    /// Creates the `Box<Stored>` with the closure running after the box is
    /// allocated, so that the return value can be written into box diretly.
    fn new_with<F: FnOnce() -> Self::Stored>(f: F) -> Self where Self::Stored: Sized;
}

impl<T: ?Sized> BoxExt for Box<T> {
    type Stored = T;

    fn new_with<F: FnOnce() -> Self::Stored>(f: F) -> Self where Self::Stored: Sized {
        let mut boxed: Box<MaybeUninit<T>> = Box::new(MaybeUninit::uninit());
        unsafe {
            *boxed = MaybeUninit::new(f());
            Box::from_raw(Box::into_raw(boxed) as *mut T)
        }
    }
}

#[cfg(test)]
#[test]
fn test_box_new_with() {
    let b = Box::new_with(|| 42);
    assert_eq!(*b, 42);
}
