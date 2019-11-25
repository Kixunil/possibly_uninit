//! Helpers for zeroing out the memory

use core::mem::MaybeUninit;

mod sealed {
    pub trait PtrCount {}

    impl<T> PtrCount for T {}
    impl<T> PtrCount for [T] {}
    impl PtrCount for str {}
}

/// Marker trait declaring that all values of `<Self as PtrCount>::Item` are
/// valid for `Self`.
pub unsafe trait AnyValid: PtrCount {}

unsafe impl<T> AnyValid for T {}
unsafe impl<T> AnyValid for [T] {}

/// Trait unifying `T` and `[T]`
///
/// This trait is used to make writing zeroes easier.
///
/// # Safety
///
/// The trait is `unsafe` because returning invalid values from `ptr_coun*`
/// methods is udefined behavior.
pub unsafe trait PtrCount: sealed::PtrCount {
    /// Either Self or type of item in the slice.
    type Item: Sized;

    /// Returns a pointer pointing to the first item and the count of items.
    ///
    /// This is only defined for sized types, in which case it must return
    /// pointer to the value and count `1` and for slices, when it must return
    /// pointer to the zeroth item of the slice and length of the slice.
    ///
    /// It's undefined for trait objects so, the trait is sealed.
    fn ptr_count(&self) -> (*const Self::Item, usize);

    /// Returns a mutable pointer pointing to the first item and the count of
    /// items.
    ///
    /// This is only defined for sized types, in which case it must return
    /// pointer to the value and count `1` and for slices, when it must return
    /// pointer to the zeroth item of the slice and length of the slice.
    ///
    /// It's undefined for trait objects so, the trait is sealed.
    fn ptr_count_mut(&mut self) -> (*mut Self::Item, usize);

    /// Helper that turns the value into slice based on `ptr` and `count`
    fn as_slice(&self) -> &[Self::Item] {
        unsafe {
            let (ptr, count) = self.ptr_count();
            core::slice::from_raw_parts(ptr, count)
        }
    }

    /// Helper that turns the value into mutable slice based on `ptr` and `count`
    fn as_slice_mut(&mut self) -> &mut [Self::Item] where Self: AnyValid {
        unsafe {
            let (ptr, count) = self.ptr_count_mut();
            core::slice::from_raw_parts_mut(ptr, count)
        }
    }
}

unsafe impl<T> PtrCount for T {
    type Item = T;

    fn ptr_count(&self) -> (*const Self::Item, usize) {
        (self, 1)
    }

    fn ptr_count_mut(&mut self) -> (*mut Self::Item, usize) {
        (self, 1)
    }
}

unsafe impl<T> PtrCount for [T] {
    type Item = T;

    fn ptr_count(&self) -> (*const Self::Item, usize) {
        (self.as_ptr(), self.len())
    }

    fn ptr_count_mut(&mut self) -> (*mut Self::Item, usize) {
        (self.as_mut_ptr(), self.len())
    }
}

unsafe impl PtrCount for str {
    type Item = u8;

    fn ptr_count(&self) -> (*const Self::Item, usize) {
        (self.as_ptr(), self.len())
    }

    fn ptr_count_mut(&mut self) -> (*mut Self::Item, usize) {
        (self.as_mut_ptr(), self.len())
    }
}

/// Trait declaring that overwriting the whole memory location of `Self` with
/// zeroes produces a valid value.
///
/// # Safety
///
/// The trait is unsafe because implementing it for a type that doesn't permit
/// all-zero bit pattern is undefined behavior.
pub unsafe trait ZeroValid {
    /// Creates value containing all zeroes in memory.
    fn zeroed() -> Self where Self: Sized {
        unsafe {
            <MaybeUninit<Self>>::zeroed().assume_init()
        }
    }

    /// Overwrites value with zeroes
    fn write_zeroes(&mut self) where Self: PtrCount {
        unsafe {
            let (ptr, count) = self.ptr_count_mut();
            core::ptr::write_bytes(ptr, 0, count);
        }
    }
}

/// Note that while `MaybeUninit<T>` may always contain zero bit pattern,
/// it doesn't mean `T` may contain zeroes!
///
/// In other words, this is just an assertion that
/// `<MaybeUninit<MaybeUninit<T>>>::zeroed().assume_init()` is valid, **not**
/// that `<MaybeUninit<T>>::zeroed().assume_init()` is valid.
unsafe impl<T> ZeroValid for MaybeUninit<T> {}

/// The pointer bit pattern zero is valid, but it doesn't mean that you can
/// dereference it!
unsafe impl<T> ZeroValid for *const T {}

/// The pointer bit pattern zero is valid, but it doesn't mean that you can
/// dereference it!
unsafe impl<T> ZeroValid for *mut T {}

unsafe impl ZeroValid for bool {}
unsafe impl ZeroValid for char {}
unsafe impl ZeroValid for usize {}
unsafe impl ZeroValid for isize {}
unsafe impl ZeroValid for u8 {}
unsafe impl ZeroValid for i8 {}
unsafe impl ZeroValid for u16 {}
unsafe impl ZeroValid for i16 {}
unsafe impl ZeroValid for u32 {}
unsafe impl ZeroValid for i32 {}
unsafe impl ZeroValid for u64 {}
unsafe impl ZeroValid for i64 {}
unsafe impl ZeroValid for u128 {}
unsafe impl ZeroValid for i128 {}
unsafe impl ZeroValid for f32 {}
unsafe impl ZeroValid for f64 {}

unsafe impl<T: ZeroValid> ZeroValid for [T] {}
unsafe impl<T: ZeroValid> ZeroValid for [T; 1] {}
unsafe impl<T: ZeroValid> ZeroValid for [T; 2] {}
unsafe impl<T: ZeroValid> ZeroValid for [T; 3] {}
unsafe impl<T: ZeroValid> ZeroValid for [T; 4] {}
unsafe impl<T: ZeroValid> ZeroValid for [T; 5] {}
unsafe impl<T: ZeroValid> ZeroValid for [T; 6] {}
unsafe impl<T: ZeroValid> ZeroValid for [T; 7] {}
unsafe impl<T: ZeroValid> ZeroValid for [T; 8] {}
unsafe impl<T: ZeroValid> ZeroValid for [T; 9] {}
unsafe impl<T: ZeroValid> ZeroValid for [T; 10] {}
unsafe impl<T: ZeroValid> ZeroValid for [T; 11] {}
unsafe impl<T: ZeroValid> ZeroValid for [T; 12] {}
unsafe impl<T: ZeroValid> ZeroValid for [T; 13] {}
unsafe impl<T: ZeroValid> ZeroValid for [T; 14] {}
unsafe impl<T: ZeroValid> ZeroValid for [T; 15] {}
unsafe impl<T: ZeroValid> ZeroValid for [T; 16] {}
unsafe impl<T: ZeroValid> ZeroValid for [T; 17] {}
unsafe impl<T: ZeroValid> ZeroValid for [T; 18] {}
unsafe impl<T: ZeroValid> ZeroValid for [T; 19] {}
unsafe impl<T: ZeroValid> ZeroValid for [T; 20] {}
unsafe impl<T: ZeroValid> ZeroValid for [T; 21] {}
unsafe impl<T: ZeroValid> ZeroValid for [T; 22] {}
unsafe impl<T: ZeroValid> ZeroValid for [T; 23] {}
unsafe impl<T: ZeroValid> ZeroValid for [T; 24] {}
unsafe impl<T: ZeroValid> ZeroValid for [T; 25] {}
unsafe impl<T: ZeroValid> ZeroValid for [T; 26] {}
unsafe impl<T: ZeroValid> ZeroValid for [T; 27] {}
unsafe impl<T: ZeroValid> ZeroValid for [T; 28] {}
unsafe impl<T: ZeroValid> ZeroValid for [T; 29] {}
unsafe impl<T: ZeroValid> ZeroValid for [T; 30] {}
unsafe impl<T: ZeroValid> ZeroValid for [T; 31] {}
unsafe impl<T: ZeroValid> ZeroValid for [T; 32] {}

unsafe impl<A: ZeroValid, B: ZeroValid, C: ZeroValid, D: ZeroValid, E: ZeroValid, F: ZeroValid, G: ZeroValid, H: ZeroValid> ZeroValid for (A, B, C, D, E, F, G, H) {}
unsafe impl<A: ZeroValid, B: ZeroValid, C: ZeroValid, D: ZeroValid, E: ZeroValid, F: ZeroValid, G: ZeroValid> ZeroValid for (A, B, C, D, E, F, G) {}
unsafe impl<A: ZeroValid, B: ZeroValid, C: ZeroValid, D: ZeroValid, E: ZeroValid, F: ZeroValid> ZeroValid for (A, B, C, D, E, F) {}
unsafe impl<A: ZeroValid, B: ZeroValid, C: ZeroValid, D: ZeroValid, E: ZeroValid> ZeroValid for (A, B, C, D, E) {}
unsafe impl<A: ZeroValid, B: ZeroValid, C: ZeroValid, D: ZeroValid> ZeroValid for (A, B, C, D) {}
unsafe impl<A: ZeroValid, B: ZeroValid, C: ZeroValid> ZeroValid for (A, B, C) {}
unsafe impl<A: ZeroValid, B: ZeroValid> ZeroValid for (A, B) {}
unsafe impl<A: ZeroValid> ZeroValid for (A,) {}
unsafe impl ZeroValid for () {}
