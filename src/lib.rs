//! Traits and types helping with using uninitialized memory safely.
//!
//! This crate provides several traits and types that make working with
//! uninitialized memory safer. They avoid memory bugs like accidentally
//! writing uninitialized value into initialized memory, reading uninitialized
//! memory, etc. They also provide strong guarantees for other safe code, which
//! is expressed as `unsafe` traits.
//!
//! Since uninitialized values make most sense when it comes to large objects,
//! the main focus is on slices and arrays. For instance, you can initialize
//! `Box<[T]>` or `Box<[T; N]>` after it was allocated, avoiding copying.
//! Unfortunately that part isn't quite perfect right now, but it does seem to
//! work correctly.
//!
//! The crate is `no_std`-compatible and `alloc`-compatible, of course.

#![deny(missing_docs)]
#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

pub mod deref_markers;
mod borrow;
pub mod zeroed;
pub mod slice;
#[cfg(feature = "alloc")]
pub mod boxed;
pub mod cast;

use core::mem::MaybeUninit;
use core::ptr::NonNull;

pub use borrow::{BorrowUninit, BorrowOut};

/// Mutable reference wrapper that only allows writing valid values to the
/// memory location.
///
/// This is used for values that might be borowed from either `T` or
/// `MaybeUninit<T>`.
///
/// One would normally expect this to be `&mut MaybeUninit<T>`, however,
/// that wouldn't be sound. Consider this code:
///
/// ```should_panic
/// use core::mem::MaybeUninit;
/// 
/// fn cast_mut<T>(val: &mut T) -> &mut MaybeUninit<T> {
///     unsafe {
///         core::mem::transmute(val)
///     }
/// }
/// 
/// // No unsafe code here
/// fn main() {
///     let mut message = "Hello world!".to_string();
///     core::mem::replace(cast_mut(&mut message), MaybeUninit::uninit());
///     println!("This is now garbage: {}", message);
/// }
/// ```
///
/// The code above triggers UB. Thus the users of the reference must be
/// prevented from writing invalid values into the memory location. That's only
/// possible by creating a newtype like this one.
///
/// While the newtype itself doesn't track initializedness, so its use may be
/// limited in safe code, it's a base building block allowing sound
/// implementations of wrappers tracking initializedness.
pub struct Out<'a, T>(&'a mut MaybeUninit<T>);

impl<'a, T> Out<'a, T> {
    /// Writes a valid value to given memory location, initializing it.
    pub fn write(self, value: T) -> &'a mut T {
        unsafe {
            *self.0 = MaybeUninit::new(value);
            self.into_assume_init()
        }
    }

    /// Writes a valid value to given memory location, initializing it.
    ///
    /// This method is same as `write`, it just reborows the reference instead
    /// of consuming it.
    pub fn write_mut(&mut self, value: T) -> &mut T {
        unsafe {
            *self.0 = MaybeUninit::new(value);
            self.assume_init_mut()
        }
    }

    /// Turns the wrapper into reference assuming the value was initialized.
    ///
    /// # Safety
    ///
    /// Calling this function if no value was written is UB.
    pub unsafe fn into_assume_init(self) -> &'a mut T {
        &mut *(self.0 as *mut MaybeUninit<T> as *mut T)
    }

    /// Returns mutable non-null pointer to the value.
    pub fn as_non_null(&mut self) -> NonNull<T> {
        <NonNull<MaybeUninit<T>>>::from(&mut *self.0).cast()
    }

    /// Returns mutable raw pointer to the value.
    ///
    /// Note that this is in fact non-null, it's just sometimes more useful than
    /// `as_non_null`.
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.0 as *mut MaybeUninit<T> as *mut T
    }

    /// Overwrites the value with all zeroes.
    pub fn write_zeroes(&mut self) -> &mut T where T: zeroed::ZeroValid {
        use crate::zeroed::ZeroValid;

        unsafe {
            self.0.write_zeroes();
            self.0.assume_init_mut()
        }
    }

    /// Overwrites the value with all zeroes.
    /// 
    /// Consumes the referenceto to preserve the lifetime
    pub fn into_zeroed(self) -> &'a mut T where T: zeroed::ZeroValid {
        use crate::zeroed::ZeroValid;

        unsafe {
            self.0.write_zeroes();
            self.0.assume_init_mut()
        }
    }
}

impl<'a, T> From<&'a mut T> for Out<'a, T> {
    fn from(value: &'a mut T) -> Self {
        unsafe {
            Out(&mut *(value as *mut T as *mut MaybeUninit<T>))
        }
    }
}

impl<'a, T> From<&'a mut MaybeUninit<T>> for Out<'a, T> {
    fn from(value: &'a mut MaybeUninit<T>) -> Self {
        Out(value)
    }
}

unsafe impl<'a, T> BorrowUninit<T> for Out<'a, T> {
    fn borrow_uninit(&self) -> &MaybeUninit<T> {
        self.0
    }
}

unsafe impl<'a, T> BorrowOut<T> for Out<'a, T> {
    fn borrow_out(&mut self) -> Out<'_, T> {
        self.0.into()
    }
}
