use core::mem::MaybeUninit;
use super::RefMut;
use crate::zeroed::ZeroValid;

/// Trait allowing borrowing of `MaybeUninit<T>` values.
///
/// It's analogous to `core::borrow::Borrow`, expect it works with
/// `MaybeUninit` values.
pub unsafe trait BorrowUninit<Borrowed> {
    /// Borrows maybe uninitialized value.
    ///
    /// This method is only useful for implementing `assume_init_ref`.
    fn borrow_uninit(&self) -> &MaybeUninit<Borrowed>;
    
    /// Converts the reference assuming it's initialized.
    ///
    /// # Safety
    ///
    /// Calling this method on uninitialized value is undefined behavior.
    unsafe fn assume_init_ref(&self) -> &Borrowed {
        &*(self.borrow_uninit() as *const MaybeUninit<Borrowed> as *const Borrowed)
    }
}

/// Trait allowing mutably borrowing of `MaybeUninit<T>` values.
///
/// It's analogous to `core::borrow::Borrow`, expect it works with
/// `MaybeUninit` values.
pub unsafe trait BorrowUninitMut<Borrowed>: BorrowUninit<Borrowed> {
    /// Mutably borrows maybe uninitialized value.
    ///
    /// This method must return `RefMut` instead of normal `&mut T` because of
    /// safety reasons described in the documentation of `RefMut` type.
    fn borrow_uninit_mut(&mut self) -> RefMut<'_, Borrowed>;

    /// Converts the reference assuming it's initialized.
    ///
    /// # Safety
    ///
    /// Calling this method on uninitialized value is undefined behavior.
    unsafe fn assume_init_mut(&mut self) -> &mut Borrowed {
        self.borrow_uninit_mut().into_assume_init()
    }

    /// Initializes the memory location with valid value.
    fn init(&mut self, item: Borrowed) -> &mut Borrowed {
        self.borrow_uninit_mut().write(item)
    }

    /// Overwrites the memory with `Default::default()` if necessary.
    ///
    /// This method is noop on `&mut Borrowed` types. It may be used when
    /// interfacing with legacy code which can't work with
    /// `MaybeUninit<Borrowed>`.
    fn default_if_needed(&mut self) -> &mut Borrowed where Borrowed: Default {
        self.init(Default::default())
    }

    /// Overwrites the memory with `Default::default()` if necessary.
    ///
    /// This method is noop on `&mut Borrowed` types. It may be used when
    /// interfacing with legacy code which can't work with
    /// `MaybeUninit<Borrowed>`.
    fn zeroed_if_needed(&mut self) -> &mut Borrowed where Borrowed: ZeroValid {
        self.borrow_uninit_mut().into_zeroed()
    }
}

unsafe impl<T> BorrowUninit<T> for T {
    fn borrow_uninit(&self) -> &MaybeUninit<T> {
        unsafe {
            &*(self as *const T as *const MaybeUninit<T>)
        }
    }
}

unsafe impl<T> BorrowUninitMut<T> for T {
    fn borrow_uninit_mut(&mut self) -> RefMut<'_, T> {
        self.into()
    }

    fn default_if_needed(&mut self) -> &mut T where T: Default {
        self
    }

    fn zeroed_if_needed(&mut self) -> &mut T where T: ZeroValid {
        self
    }
}

unsafe impl<T> BorrowUninit<T> for MaybeUninit<T> {
    fn borrow_uninit(&self) -> &MaybeUninit<T> {
        self
    }
}

unsafe impl<T> BorrowUninitMut<T> for MaybeUninit<T> {
    fn borrow_uninit_mut(&mut self) -> RefMut<'_, T> {
        self.into()
    }
}

unsafe impl<T> BorrowUninit<T> for &T {
    fn borrow_uninit(&self) -> &MaybeUninit<T> {
        (**self).borrow_uninit()
    }
}

unsafe impl<T> BorrowUninit<T> for &mut T {
    fn borrow_uninit(&self) -> &MaybeUninit<T> {
        (**self).borrow_uninit()
    }
}

unsafe impl<T> BorrowUninitMut<T> for &mut T {
    fn borrow_uninit_mut(&mut self) -> RefMut<'_, T> {
        (*self).into()
    }

    fn default_if_needed(&mut self) -> &mut T where T: Default {
        *self
    }

    fn zeroed_if_needed(&mut self) -> &mut T where T: ZeroValid {
        *self
    }
}

unsafe impl<T> BorrowUninit<T> for &MaybeUninit<T> {
    fn borrow_uninit(&self) -> &MaybeUninit<T> {
        (**self).borrow_uninit()
    }
}

unsafe impl<T> BorrowUninit<T> for &mut MaybeUninit<T> {
    fn borrow_uninit(&self) -> &MaybeUninit<T> {
        (**self).borrow_uninit()
    }
}

unsafe impl<T> BorrowUninitMut<T> for &mut MaybeUninit<T> {
    fn borrow_uninit_mut(&mut self) -> RefMut<'_, T> {
        (*self).borrow_uninit_mut()
    }
}

unsafe impl<T> BorrowUninit<T> for core::cell::Ref<'_, T> {
    fn borrow_uninit(&self) -> &MaybeUninit<T> {
        (**self).borrow_uninit()
    }
}

unsafe impl<T> BorrowUninit<T> for core::cell::RefMut<'_, T> {
    fn borrow_uninit(&self) -> &MaybeUninit<T> {
        (**self).borrow_uninit()
    }
}

unsafe impl<T> BorrowUninitMut<T> for core::cell::RefMut<'_, T> {
    fn borrow_uninit_mut(&mut self) -> RefMut<'_, T> {
        (**self).borrow_uninit_mut()
    }

    fn default_if_needed(&mut self) -> &mut T where T: Default {
        &mut **self
    }

    fn zeroed_if_needed(&mut self) -> &mut T where T: ZeroValid {
        &mut **self
    }
}

unsafe impl<T> BorrowUninit<T> for core::cell::Ref<'_, MaybeUninit<T>> {
    fn borrow_uninit(&self) -> &MaybeUninit<T> {
        (**self).borrow_uninit()
    }
}

unsafe impl<T> BorrowUninit<T> for core::cell::RefMut<'_, MaybeUninit<T>> {
    fn borrow_uninit(&self) -> &MaybeUninit<T> {
        (**self).borrow_uninit()
    }
}

unsafe impl<T> BorrowUninitMut<T> for core::cell::RefMut<'_, MaybeUninit<T>> {
    fn borrow_uninit_mut(&mut self) -> RefMut<'_, T> {
        (**self).borrow_uninit_mut()
    }
}

/*
// This would be a great shortcut, but unfortunately T can impl Deref with Target=T
// Type inequality bounds would help.
use core::ops::{Deref,DerefMut};
use crate::deref_markers::SameDataDeref;

unsafe impl<T, U> BorrowUninit<U> for T where T: SameDataDeref + Deref + ?Sized,
                                              T::Target: BorrowUninit<U> {

    fn borrow_uninit(&self) -> &MaybeUninit<U> {
        self.borrow_uninit()
    }
}

unsafe impl<T, U> BorrowUninitMut<U> for T where T: SameDataDeref + Deref + DerefMut + ?Sized,
                                                 T::Target: BorrowUninitMut<U> {

    fn borrow_uninit_mut(&self) -> RefMut<'_, U> {
        self.borrow_uninit_mut()
    }
}
*/
