//! This module contains traits and types for working with slices of
//! uninitialized memory

mod cursor;

pub use self::cursor::Cursor;

use core::mem::MaybeUninit;
use core::ptr::NonNull;
use core::ops::{Deref, DerefMut, Index, IndexMut};
use core::slice::SliceIndex;
use crate::deref_markers::SameDataDeref;
use crate::zeroed::ZeroValid;

/// Abstraction allowing treating `T` and `&T where T: Copy` equally. 
///
/// This is mainly useful when working with iterators, so there don't have to be
/// two different methods consuming iterator and doing pretty much the same
/// thing with it.
pub trait TakeItem<T> {

    /// Moves or copies the value.
    fn take_item(self) -> T;
}

impl<T> TakeItem<T> for T {
    fn take_item(self) -> T {
        self
    }
}

impl<T: Copy> TakeItem<T> for &T {
    fn take_item(self) -> T {
        *self
    }
}

#[cfg(feature = "alloc")]
impl<T> TakeItem<T> for alloc::boxed::Box<T> {
    fn take_item(self) -> T {
        *self
    }
}

/// A slice of `T` that may be uninitialized, but might also be borrowed from
/// `&mut [T]`.
///
/// One would normally expect this to be `&mut [MaybeUninit<T>]`, however,
/// that wouldn't be sound. Consider this code:
///
/// ```should_panic
/// use core::mem::MaybeUninit;
/// 
/// fn cast_mut<T>(val: &mut [T]) -> &mut [MaybeUninit<T>] {
///     unsafe {
///         core::slice::from_raw_parts_mut(val.as_mut_ptr() as *mut MaybeUninit<T>, val.len())
///     }
/// }
/// 
/// // No unsafe code here
/// fn main() {
///     let mut message = ["Hello world!".to_string()];
///     core::mem::replace(&mut cast_mut(&mut message)[0], MaybeUninit::uninit());
///     println!("This is now garbage: {}", message[0]);
/// }
/// ```
///
/// The code above triggers UB. Thus the users of the slice must be prevented
/// from writing invalid values into the slice. That's only possible by
/// creating a newtype like this one.
///
/// While the newtype itself doesn't track initializedness, so its use may be
/// limited in safe code, it's a base building block allowing sound
/// implementations of wrappers tracking initializedness. See `Cursor` type in
/// this crate.
pub struct OutSlice<T>([MaybeUninit<T>]);

impl<T> OutSlice<T> {
    fn as_raw(&self) -> &[MaybeUninit<T>] {
        unsafe {
            &*(self as *const OutSlice<T> as *const [MaybeUninit<T>])
        }
    }

    unsafe fn as_raw_mut(&mut self) -> &mut [MaybeUninit<T>] {
        &mut *(self as *mut OutSlice<T> as *mut [MaybeUninit<T>])
    }

    /// Accesses the value at given index.
    ///
    /// Note that this is useless unless you know that the value is
    /// initialized. Declare so by `unsafe`ly calling `assume_init_ref`.
    pub fn at<I: SliceIndex<[MaybeUninit<T>], Output=MaybeUninit<T>>>(&self, index: I) -> &MaybeUninit<T> {
        &self.0[index]
    }

    /// Accesses value at given index.
    ///
    /// A special reference wrapper is returned that allows you to only write
    /// valid values.
    pub fn at_mut<I: SliceIndex<[MaybeUninit<T>], Output=MaybeUninit<T>>>(&mut self, index: I) -> super::Out<'_, T> {
        (&mut self.0[index]).into()
    }

    /// Returns the length of the slice.
    pub fn len(&self) -> usize {
        self.as_raw().len()
    }

    /// Returns `true` if the slice is empty.
    pub fn is_empty(&self) -> bool {
        self.as_raw().is_empty()
    }

    /// Returns an iterator that allows initializing the slice
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        self.into_iter()
    }

    /// Splits the slice at given index into two parts.
    ///
    /// # Panics
    ///
    /// Panics if `mid` is out of bounds.
    pub fn split_at_mut(&mut self, mid: usize) -> (&mut OutSlice<T>, &mut OutSlice<T>) {
        unsafe {
            let (first, second) = self.as_raw_mut().split_at_mut(mid);
            (first.into(), second.into())
        }
    }

    /// Transforms the slice into initialized version.
    ///
    /// # Safety
    ///
    /// This method is `unsafe` because it may only be called when the whole
    /// slice was initialized. Use indexing operator to get a subslice if only a
    /// part of the slice was initialized.
    pub unsafe fn assume_init(&self) -> &[T] {
        let slice = self.as_raw();
        let ptr = slice.as_ptr() as *const T;
        let len = slice.len();

        core::slice::from_raw_parts(ptr, len)
    }

    /// Transforms the slice into initialized mutable version.
    ///
    /// # Safety
    ///
    /// This method is `unsafe` because it may only be called when the whole
    /// slice was initialized. Use indexing operator to get a subslice if only a
    /// part of the slice was initialized.
    pub unsafe fn assume_init_mut(&mut self) -> &mut [T] {
        let slice = self.as_raw_mut();
        let ptr = slice.as_mut_ptr() as *mut T;
        let len = slice.len();

        core::slice::from_raw_parts_mut(ptr, len)
    }

    /// Returns non-null pointer to the first element of the slice.
    pub fn as_non_null(&mut self) -> NonNull<T> {
        unsafe {
            NonNull::new_unchecked(self.as_raw_mut().as_mut_ptr()).cast()
        }
    }

    /// Returns mutable raw pointer to the first element of the slice.
    ///
    /// Note that this is in fact non-null, it's just sometimes more useful than
    /// `as_non_null`.
    pub fn as_mut_ptr(&mut self) -> *mut T {
        unsafe {
            self.as_raw_mut().as_mut_ptr() as *mut T
        }
    }

    /// Initializes the whole slice by overwriting it with zeroes.
    ///
    /// This is only possible if `Item` allows safely initializing itself with
    /// zeroes. (e.g. in case of integer types)
    ///
    /// You may implement `ZeroValid` if zero bit pattern is valid for your
    /// type.
    pub fn write_zeroes(&mut self) -> &mut [T] where T: ZeroValid {
        unsafe {
            self.as_raw_mut().write_zeroes();
            self.assume_init_mut()
        }
    }

    /// Uses the iterator to initialize the slice
    pub fn init_from_iter<I>(&mut self, iter: I) -> &mut [T] where I: IntoIterator, I::Item: TakeItem<T> {
        unsafe {
            // Yes, we really want map and then count.
            // If you know some nice, functional way of implementing this
            // without clippy screaming, make a PR.
            #[allow(clippy::suspicious_map)]
            let len = self.iter_mut().zip(iter).map(|(dest, source)| dest.write(source.take_item())).count();
            self[..len].assume_init_mut()
        }
    }

    /// Initializes the slice by copying another slice into it.
    ///
    /// This is a specialized version of `init_from_iter`, which may be more
    /// performant.
    ///
    /// # Panics
    ///
    /// The method panics if the slices are of different lengths. Use indexing
    /// opertors to make the lengths same.
    pub fn copy_from_slice(&mut self, slice: &[T]) -> &mut [T] where T: Copy {
        unsafe {
            self.as_raw_mut().copy_from_slice(slice.borrow_uninit_slice());
            self.assume_init_mut()
        }
    }
}

impl<T, R: SliceIndex<[MaybeUninit<T>], Output=[MaybeUninit<T>]>> Index<R> for OutSlice<T> {
    type Output = Self;

    fn index(&self, range: R) -> &Self::Output {
        (&self.as_raw()[range]).into()
    }
}

impl<T, R: SliceIndex<[MaybeUninit<T>], Output=[MaybeUninit<T>]>> IndexMut<R> for OutSlice<T> {
    fn index_mut(&mut self, range: R) -> &mut Self::Output {
        unsafe {
            (&mut self.as_raw_mut()[range]).into()
        }
    }
}

impl<'a, T> From<&'a [T]> for &'a OutSlice<T> {
    fn from(value: &'a [T]) -> Self {
        unsafe {
            let ptr = value.as_ptr() as *const MaybeUninit<T>;
            let len = value.len();

            &*(core::slice::from_raw_parts(ptr, len) as *const [MaybeUninit<T>] as *const OutSlice<T>)
        }
    }
}

impl<'a, T> From<&'a mut [T]> for &'a mut OutSlice<T> {
    fn from(value: &'a mut [T]) -> Self {
        unsafe {
            let ptr = value.as_mut_ptr() as *mut MaybeUninit<T>;
            let len = value.len();

            &mut *(core::slice::from_raw_parts_mut(ptr, len) as *mut [MaybeUninit<T>] as *mut OutSlice<T>)
        }
    }
}

impl<'a, T> From<&'a [MaybeUninit<T>]> for &'a OutSlice<T> {
    fn from(value: &'a [MaybeUninit<T>]) -> Self {
        unsafe {
            &*(value as *const [MaybeUninit<T>] as *const OutSlice<T>)
        }
    }
}

impl<'a, T> From<&'a mut [MaybeUninit<T>]> for &'a mut OutSlice<T> {
    fn from(value: &'a mut [MaybeUninit<T>]) -> Self {
        unsafe {
            &mut *(value as *mut [MaybeUninit<T>] as *mut OutSlice<T>)
        }
    }
}

unsafe impl<T> BorrowUninitSlice<T> for OutSlice<T> {
    fn borrow_uninit_slice(&self) -> &[MaybeUninit<T>] {
        self.as_raw()
    }
}

unsafe impl<'a, T> BorrowOutSlice<T> for OutSlice<T> {
    fn borrow_out_slice(&mut self) -> &mut OutSlice<T> {
        self
    }
}

impl<'a, T> IntoIterator for &'a mut OutSlice<T> {
    type Item = super::Out<'a, T>;
    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        unsafe {
            IterMut(self.as_raw_mut().iter_mut())
        }
    }
}

/// Mutable `OutSlice` iterator.
///
/// This struct is created by `iter_mut` method on `OutSlice`.
pub struct IterMut<'a, T>(core::slice::IterMut<'a, MaybeUninit<T>>);

impl<'a, T> IterMut<'a, T> {
    /// Views the underlying data as a subslice of the original data.
    ///
    /// To avoid creating `&mut` references that alias, this is forced to
    /// consume the iterator.
    pub fn into_slice(self) -> &'a mut OutSlice<T> {
        self.0.into_slice().into()
    }
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = super::Out<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(Into::into)
    }
}

/// A trait somewhat similar to core::borrow::Borrow, but very specific about
/// being a slice containing `MaybeUninit<Item>`
///
/// The main reason for it is to allow use of array-like types in these
/// scenarios:
///
/// * `Wrapper<[Item]>`
/// * `Wrapper<MaybeUninit<Item>>`
/// * `Wrapper<ReferenceType<Item>>`
/// * `Wrapper<ReferenceType<MaybeUninit<Item>>>`
/// * `Wrapper<ReferenceTypeMut<Item>>`
/// * `Wrapper<ReferenceTypeMut<MaybeUninit<Item>>>`
pub unsafe trait BorrowUninitSlice<Item> {
    /// Borrows the value as a slice of `MaybeUninit<Item>`
    fn borrow_uninit_slice(&self) -> &[MaybeUninit<Item>];

    /// Treat the slice as initialized.
    ///
    /// This method is `unsafe` because calling it without **all** the slice
    /// being initialized is undefined behavior.
    unsafe fn assume_init(&self) -> &[Item] {
        let slice = self.borrow_uninit_slice();
        core::slice::from_raw_parts(slice.as_ptr() as *const Item, slice.len())
    }
}

/// A trait somewhat similar to core::borrow::BorrowMut, but very specific about
/// being a slice containing `MaybeUninit<Item>`
///
/// The main reason for it is to allow use of array-like types in these
/// scenarios:
///
/// * `Wrapper<[Item]>`
/// * `Wrapper<MaybeUninit<Item>>`
/// * `Wrapper<ReferenceTypeMut<Item>>`
/// * `Wrapper<ReferenceTypeMut<MaybeUninit<Item>>>`
pub unsafe trait BorrowOutSlice<Item>: BorrowUninitSlice<Item> {
    /// Borrows the value as a mutable slice of `MaybeUninit<Item>`
    fn borrow_out_slice(&mut self) -> &mut OutSlice<Item>;

    /// Zeroes the buffer if it's needed and returns it as initialized.
    ///
    /// This can be used when interfacing with an old code which doesn't
    /// support `MaybeUninit<Item>`. However, it may cause performance decrease,
    /// especially in loops. If you hit such issue, you can supply `&mut [Item]`
    /// instead of `&mut [MaybeUninit<Item>]`, which doesn't need to zero the
    /// buffer anymore.
    ///
    /// Note that while strictly safe, the returned slice may contain garbage!
    /// Reading from the returned slice is not UB but nearly certainly a logic
    /// bug.
    fn zero_if_needed(&mut self) -> &mut [Item] where Item: ZeroValid {
        unsafe {
            self.borrow_out_slice().write_zeroes();
            self.assume_init_mut()
        }
    }

    /// Initializes the slice by copying from another slice.
    ///
    /// The returned slice will have the same length as self.borrow_uninit_slice()
    ///
    /// Panics if the lengths differ.
    fn init_with_copy_from_slice(&mut self, slice: &[Item]) -> &mut [Item] where Item: Copy {
        self.borrow_out_slice().copy_from_slice(slice)
    }

    /// Initializes a subslice by copying from another slice.
    ///
    /// This is similar to `init_with_copy_from_slice` except that instead of
    /// panicking, it copies minimum of the slice lengths.
    fn init_with_copy_from_slice_min(&mut self, slice: &[Item]) -> &mut [Item] where Item: Copy {
        let to_copy = self.borrow_out_slice().len().min(slice.len());
        let target = &mut self.borrow_out_slice()[..to_copy];

        target.init_with_copy_from_slice(&slice[..to_copy])
    }

    /// Treat the slice as initialized.
    ///
    /// This method is `unsafe` because calling it without **all** the slice
    /// being initialized is undefined behavior.
    unsafe fn assume_init_mut(&mut self) -> &mut [Item] {
        let slice = self.borrow_out_slice();
        core::slice::from_raw_parts_mut(slice.as_mut_ptr() as *mut Item, slice.len())
    }
}

unsafe impl<Item> BorrowUninitSlice<Item> for [MaybeUninit<Item>] {
    fn borrow_uninit_slice(&self) -> &[MaybeUninit<Item>] {
        self
    }
}

unsafe impl<Item> BorrowOutSlice<Item> for [MaybeUninit<Item>] {
    fn borrow_out_slice(&mut self) -> &mut OutSlice<Item> {
        self.into()
    }
}

unsafe impl<Item> BorrowUninitSlice<Item> for [Item] {
    fn borrow_uninit_slice(&self) -> &[MaybeUninit<Item>] {
        unsafe {
            core::slice::from_raw_parts(self.as_ptr() as *const MaybeUninit<Item>, self.len())
        }
    }
}

unsafe impl<Item> BorrowOutSlice<Item> for [Item] {
    fn borrow_out_slice(&mut self) -> &mut OutSlice<Item> {
        self.into()
    }

    fn zero_if_needed(&mut self) -> &mut [Item] {
        self
    }
}

/// Not very useful, but for the sake of completeness.
unsafe impl BorrowUninitSlice<u8> for str {
    fn borrow_uninit_slice(&self) -> &[MaybeUninit<u8>] {
        self.as_bytes().borrow_uninit_slice()
    }
}

unsafe impl<T, Item> BorrowUninitSlice<Item> for T where T: SameDataDeref + Deref + ?Sized,
                                                 T::Target: BorrowUninitSlice<Item> {

    fn borrow_uninit_slice(&self) -> &[MaybeUninit<Item>] {
        (**self).borrow_uninit_slice()
    }
}

unsafe impl<T, Item> BorrowOutSlice<Item> for T where T: SameDataDeref + Deref + DerefMut + ?Sized,
                                                    T::Target: BorrowOutSlice<Item> {
    fn borrow_out_slice(&mut self) -> &mut OutSlice<Item> {
        (**self).borrow_out_slice()
    }

    fn zero_if_needed(&mut self) -> &mut [Item] where Item: ZeroValid {
        (**self).zero_if_needed()
    }
}

unsafe impl<Item> BorrowUninitSlice<Item> for [MaybeUninit<Item>; 0] {
    fn borrow_uninit_slice(&self) -> &[MaybeUninit<Item>] {
        self
    }
}

unsafe impl<Item> BorrowOutSlice<Item> for [MaybeUninit<Item>; 0] {
    fn borrow_out_slice(&mut self) -> &mut OutSlice<Item> {
        (self as &mut [_]).into()
    }
}

unsafe impl<Item> BorrowUninitSlice<Item> for [Item; 0] {
    fn borrow_uninit_slice(&self) -> &[MaybeUninit<Item>] {
        unsafe {
            let slice = self as &[_];
            core::slice::from_raw_parts(slice.as_ptr() as *const MaybeUninit<Item>, slice.len())
        }
    }
}

unsafe impl<Item> BorrowOutSlice<Item> for [Item; 0] {
    fn borrow_out_slice(&mut self) -> &mut OutSlice<Item> {
        (self as &mut [_]).into()
    }

    fn zero_if_needed(&mut self) -> &mut [Item] {
        self
    }
}

unsafe impl<Item> BorrowUninitSlice<Item> for [MaybeUninit<Item>; 1] {
    fn borrow_uninit_slice(&self) -> &[MaybeUninit<Item>] {
        self
    }
}

unsafe impl<Item> BorrowOutSlice<Item> for [MaybeUninit<Item>; 1] {
    fn borrow_out_slice(&mut self) -> &mut OutSlice<Item> {
        (self as &mut [_]).into()
    }
}

unsafe impl<Item> BorrowUninitSlice<Item> for [Item; 1] {
    fn borrow_uninit_slice(&self) -> &[MaybeUninit<Item>] {
        unsafe {
            let slice = self as &[_];
            core::slice::from_raw_parts(slice.as_ptr() as *const MaybeUninit<Item>, slice.len())
        }
    }
}

unsafe impl<Item> BorrowOutSlice<Item> for [Item; 1] {
    fn borrow_out_slice(&mut self) -> &mut OutSlice<Item> {
        (self as &mut [_]).into()
    }

    fn zero_if_needed(&mut self) -> &mut [Item] {
        self
    }
}

unsafe impl<Item> BorrowUninitSlice<Item> for [MaybeUninit<Item>; 2] {
    fn borrow_uninit_slice(&self) -> &[MaybeUninit<Item>] {
        self
    }
}

unsafe impl<Item> BorrowOutSlice<Item> for [MaybeUninit<Item>; 2] {
    fn borrow_out_slice(&mut self) -> &mut OutSlice<Item> {
        (self as &mut [_]).into()
    }
}

unsafe impl<Item> BorrowUninitSlice<Item> for [Item; 2] {
    fn borrow_uninit_slice(&self) -> &[MaybeUninit<Item>] {
        unsafe {
            let slice = self as &[_];
            core::slice::from_raw_parts(slice.as_ptr() as *const MaybeUninit<Item>, slice.len())
        }
    }
}

unsafe impl<Item> BorrowOutSlice<Item> for [Item; 2] {
    fn borrow_out_slice(&mut self) -> &mut OutSlice<Item> {
        (self as &mut [_]).into()
    }

    fn zero_if_needed(&mut self) -> &mut [Item] {
        self
    }
}

unsafe impl<Item> BorrowUninitSlice<Item> for [MaybeUninit<Item>; 3] {
    fn borrow_uninit_slice(&self) -> &[MaybeUninit<Item>] {
        self
    }
}

unsafe impl<Item> BorrowOutSlice<Item> for [MaybeUninit<Item>; 3] {
    fn borrow_out_slice(&mut self) -> &mut OutSlice<Item> {
        (self as &mut [_]).into()
    }
}

unsafe impl<Item> BorrowUninitSlice<Item> for [Item; 3] {
    fn borrow_uninit_slice(&self) -> &[MaybeUninit<Item>] {
        unsafe {
            let slice = self as &[_];
            core::slice::from_raw_parts(slice.as_ptr() as *const MaybeUninit<Item>, slice.len())
        }
    }
}

unsafe impl<Item> BorrowOutSlice<Item> for [Item; 3] {
    fn borrow_out_slice(&mut self) -> &mut OutSlice<Item> {
        (self as &mut [_]).into()
    }

    fn zero_if_needed(&mut self) -> &mut [Item] {
        self
    }
}

unsafe impl<Item> BorrowUninitSlice<Item> for [MaybeUninit<Item>; 4] {
    fn borrow_uninit_slice(&self) -> &[MaybeUninit<Item>] {
        self
    }
}

unsafe impl<Item> BorrowOutSlice<Item> for [MaybeUninit<Item>; 4] {
    fn borrow_out_slice(&mut self) -> &mut OutSlice<Item> {
        (self as &mut [_]).into()
    }
}

unsafe impl<Item> BorrowUninitSlice<Item> for [Item; 4] {
    fn borrow_uninit_slice(&self) -> &[MaybeUninit<Item>] {
        unsafe {
            let slice = self as &[_];
            core::slice::from_raw_parts(slice.as_ptr() as *const MaybeUninit<Item>, slice.len())
        }
    }
}

unsafe impl<Item> BorrowOutSlice<Item> for [Item; 4] {
    fn borrow_out_slice(&mut self) -> &mut OutSlice<Item> {
        (self as &mut [_]).into()
    }

    fn zero_if_needed(&mut self) -> &mut [Item] {
        self
    }
}

unsafe impl<Item> BorrowUninitSlice<Item> for [MaybeUninit<Item>; 5] {
    fn borrow_uninit_slice(&self) -> &[MaybeUninit<Item>] {
        self
    }
}

unsafe impl<Item> BorrowOutSlice<Item> for [MaybeUninit<Item>; 5] {
    fn borrow_out_slice(&mut self) -> &mut OutSlice<Item> {
        (self as &mut [_]).into()
    }
}

unsafe impl<Item> BorrowUninitSlice<Item> for [Item; 5] {
    fn borrow_uninit_slice(&self) -> &[MaybeUninit<Item>] {
        unsafe {
            let slice = self as &[_];
            core::slice::from_raw_parts(slice.as_ptr() as *const MaybeUninit<Item>, slice.len())
        }
    }
}

unsafe impl<Item> BorrowOutSlice<Item> for [Item; 5] {
    fn borrow_out_slice(&mut self) -> &mut OutSlice<Item> {
        (self as &mut [_]).into()
    }

    fn zero_if_needed(&mut self) -> &mut [Item] {
        self
    }
}

unsafe impl<Item> BorrowUninitSlice<Item> for [MaybeUninit<Item>; 6] {
    fn borrow_uninit_slice(&self) -> &[MaybeUninit<Item>] {
        self
    }
}

unsafe impl<Item> BorrowOutSlice<Item> for [MaybeUninit<Item>; 6] {
    fn borrow_out_slice(&mut self) -> &mut OutSlice<Item> {
        (self as &mut [_]).into()
    }
}

unsafe impl<Item> BorrowUninitSlice<Item> for [Item; 6] {
    fn borrow_uninit_slice(&self) -> &[MaybeUninit<Item>] {
        unsafe {
            let slice = self as &[_];
            core::slice::from_raw_parts(slice.as_ptr() as *const MaybeUninit<Item>, slice.len())
        }
    }
}

unsafe impl<Item> BorrowOutSlice<Item> for [Item; 6] {
    fn borrow_out_slice(&mut self) -> &mut OutSlice<Item> {
        (self as &mut [_]).into()
    }

    fn zero_if_needed(&mut self) -> &mut [Item] {
        self
    }
}

unsafe impl<Item> BorrowUninitSlice<Item> for [MaybeUninit<Item>; 7] {
    fn borrow_uninit_slice(&self) -> &[MaybeUninit<Item>] {
        self
    }
}

unsafe impl<Item> BorrowOutSlice<Item> for [MaybeUninit<Item>; 7] {
    fn borrow_out_slice(&mut self) -> &mut OutSlice<Item> {
        (self as &mut [_]).into()
    }
}

unsafe impl<Item> BorrowUninitSlice<Item> for [Item; 7] {
    fn borrow_uninit_slice(&self) -> &[MaybeUninit<Item>] {
        unsafe {
            let slice = self as &[_];
            core::slice::from_raw_parts(slice.as_ptr() as *const MaybeUninit<Item>, slice.len())
        }
    }
}

unsafe impl<Item> BorrowOutSlice<Item> for [Item; 7] {
    fn borrow_out_slice(&mut self) -> &mut OutSlice<Item> {
        (self as &mut [_]).into()
    }

    fn zero_if_needed(&mut self) -> &mut [Item] {
        self
    }
}

unsafe impl<Item> BorrowUninitSlice<Item> for [MaybeUninit<Item>; 8] {
    fn borrow_uninit_slice(&self) -> &[MaybeUninit<Item>] {
        self
    }
}

unsafe impl<Item> BorrowOutSlice<Item> for [MaybeUninit<Item>; 8] {
    fn borrow_out_slice(&mut self) -> &mut OutSlice<Item> {
        (self as &mut [_]).into()
    }
}

unsafe impl<Item> BorrowUninitSlice<Item> for [Item; 8] {
    fn borrow_uninit_slice(&self) -> &[MaybeUninit<Item>] {
        unsafe {
            let slice = self as &[_];
            core::slice::from_raw_parts(slice.as_ptr() as *const MaybeUninit<Item>, slice.len())
        }
    }
}

unsafe impl<Item> BorrowOutSlice<Item> for [Item; 8] {
    fn borrow_out_slice(&mut self) -> &mut OutSlice<Item> {
        (self as &mut [_]).into()
    }

    fn zero_if_needed(&mut self) -> &mut [Item] {
        self
    }
}

unsafe impl<Item> BorrowUninitSlice<Item> for [MaybeUninit<Item>; 9] {
    fn borrow_uninit_slice(&self) -> &[MaybeUninit<Item>] {
        self
    }
}

unsafe impl<Item> BorrowOutSlice<Item> for [MaybeUninit<Item>; 9] {
    fn borrow_out_slice(&mut self) -> &mut OutSlice<Item> {
        (self as &mut [_]).into()
    }
}

unsafe impl<Item> BorrowUninitSlice<Item> for [Item; 9] {
    fn borrow_uninit_slice(&self) -> &[MaybeUninit<Item>] {
        unsafe {
            let slice = self as &[_];
            core::slice::from_raw_parts(slice.as_ptr() as *const MaybeUninit<Item>, slice.len())
        }
    }
}

unsafe impl<Item> BorrowOutSlice<Item> for [Item; 9] {
    fn borrow_out_slice(&mut self) -> &mut OutSlice<Item> {
        (self as &mut [_]).into()
    }

    fn zero_if_needed(&mut self) -> &mut [Item] {
        self
    }
}

unsafe impl<Item> BorrowUninitSlice<Item> for [MaybeUninit<Item>; 10] {
    fn borrow_uninit_slice(&self) -> &[MaybeUninit<Item>] {
        self
    }
}

unsafe impl<Item> BorrowOutSlice<Item> for [MaybeUninit<Item>; 10] {
    fn borrow_out_slice(&mut self) -> &mut OutSlice<Item> {
        (self as &mut [_]).into()
    }
}

unsafe impl<Item> BorrowUninitSlice<Item> for [Item; 10] {
    fn borrow_uninit_slice(&self) -> &[MaybeUninit<Item>] {
        unsafe {
            let slice = self as &[_];
            core::slice::from_raw_parts(slice.as_ptr() as *const MaybeUninit<Item>, slice.len())
        }
    }
}

unsafe impl<Item> BorrowOutSlice<Item> for [Item; 10] {
    fn borrow_out_slice(&mut self) -> &mut OutSlice<Item> {
        (self as &mut [_]).into()
    }

    fn zero_if_needed(&mut self) -> &mut [Item] {
        self
    }
}

unsafe impl<Item> BorrowUninitSlice<Item> for [MaybeUninit<Item>; 11] {
    fn borrow_uninit_slice(&self) -> &[MaybeUninit<Item>] {
        self
    }
}

unsafe impl<Item> BorrowOutSlice<Item> for [MaybeUninit<Item>; 11] {
    fn borrow_out_slice(&mut self) -> &mut OutSlice<Item> {
        (self as &mut [_]).into()
    }
}

unsafe impl<Item> BorrowUninitSlice<Item> for [Item; 11] {
    fn borrow_uninit_slice(&self) -> &[MaybeUninit<Item>] {
        unsafe {
            let slice = self as &[_];
            core::slice::from_raw_parts(slice.as_ptr() as *const MaybeUninit<Item>, slice.len())
        }
    }
}

unsafe impl<Item> BorrowOutSlice<Item> for [Item; 11] {
    fn borrow_out_slice(&mut self) -> &mut OutSlice<Item> {
        (self as &mut [_]).into()
    }

    fn zero_if_needed(&mut self) -> &mut [Item] {
        self
    }
}

unsafe impl<Item> BorrowUninitSlice<Item> for [MaybeUninit<Item>; 12] {
    fn borrow_uninit_slice(&self) -> &[MaybeUninit<Item>] {
        self
    }
}

unsafe impl<Item> BorrowOutSlice<Item> for [MaybeUninit<Item>; 12] {
    fn borrow_out_slice(&mut self) -> &mut OutSlice<Item> {
        (self as &mut [_]).into()
    }
}

unsafe impl<Item> BorrowUninitSlice<Item> for [Item; 12] {
    fn borrow_uninit_slice(&self) -> &[MaybeUninit<Item>] {
        unsafe {
            let slice = self as &[_];
            core::slice::from_raw_parts(slice.as_ptr() as *const MaybeUninit<Item>, slice.len())
        }
    }
}

unsafe impl<Item> BorrowOutSlice<Item> for [Item; 12] {
    fn borrow_out_slice(&mut self) -> &mut OutSlice<Item> {
        (self as &mut [_]).into()
    }

    fn zero_if_needed(&mut self) -> &mut [Item] {
        self
    }
}

unsafe impl<Item> BorrowUninitSlice<Item> for [MaybeUninit<Item>; 13] {
    fn borrow_uninit_slice(&self) -> &[MaybeUninit<Item>] {
        self
    }
}

unsafe impl<Item> BorrowOutSlice<Item> for [MaybeUninit<Item>; 13] {
    fn borrow_out_slice(&mut self) -> &mut OutSlice<Item> {
        (self as &mut [_]).into()
    }
}

unsafe impl<Item> BorrowUninitSlice<Item> for [Item; 13] {
    fn borrow_uninit_slice(&self) -> &[MaybeUninit<Item>] {
        unsafe {
            let slice = self as &[_];
            core::slice::from_raw_parts(slice.as_ptr() as *const MaybeUninit<Item>, slice.len())
        }
    }
}

unsafe impl<Item> BorrowOutSlice<Item> for [Item; 13] {
    fn borrow_out_slice(&mut self) -> &mut OutSlice<Item> {
        (self as &mut [_]).into()
    }

    fn zero_if_needed(&mut self) -> &mut [Item] {
        self
    }
}

unsafe impl<Item> BorrowUninitSlice<Item> for [MaybeUninit<Item>; 14] {
    fn borrow_uninit_slice(&self) -> &[MaybeUninit<Item>] {
        self
    }
}

unsafe impl<Item> BorrowOutSlice<Item> for [MaybeUninit<Item>; 14] {
    fn borrow_out_slice(&mut self) -> &mut OutSlice<Item> {
        (self as &mut [_]).into()
    }
}

unsafe impl<Item> BorrowUninitSlice<Item> for [Item; 14] {
    fn borrow_uninit_slice(&self) -> &[MaybeUninit<Item>] {
        unsafe {
            let slice = self as &[_];
            core::slice::from_raw_parts(slice.as_ptr() as *const MaybeUninit<Item>, slice.len())
        }
    }
}

unsafe impl<Item> BorrowOutSlice<Item> for [Item; 14] {
    fn borrow_out_slice(&mut self) -> &mut OutSlice<Item> {
        (self as &mut [_]).into()
    }

    fn zero_if_needed(&mut self) -> &mut [Item] {
        self
    }
}

unsafe impl<Item> BorrowUninitSlice<Item> for [MaybeUninit<Item>; 15] {
    fn borrow_uninit_slice(&self) -> &[MaybeUninit<Item>] {
        self
    }
}

unsafe impl<Item> BorrowOutSlice<Item> for [MaybeUninit<Item>; 15] {
    fn borrow_out_slice(&mut self) -> &mut OutSlice<Item> {
        (self as &mut [_]).into()
    }
}

unsafe impl<Item> BorrowUninitSlice<Item> for [Item; 15] {
    fn borrow_uninit_slice(&self) -> &[MaybeUninit<Item>] {
        unsafe {
            let slice = self as &[_];
            core::slice::from_raw_parts(slice.as_ptr() as *const MaybeUninit<Item>, slice.len())
        }
    }
}

unsafe impl<Item> BorrowOutSlice<Item> for [Item; 15] {
    fn borrow_out_slice(&mut self) -> &mut OutSlice<Item> {
        (self as &mut [_]).into()
    }

    fn zero_if_needed(&mut self) -> &mut [Item] {
        self
    }
}

unsafe impl<Item> BorrowUninitSlice<Item> for [MaybeUninit<Item>; 16] {
    fn borrow_uninit_slice(&self) -> &[MaybeUninit<Item>] {
        self
    }
}

unsafe impl<Item> BorrowOutSlice<Item> for [MaybeUninit<Item>; 16] {
    fn borrow_out_slice(&mut self) -> &mut OutSlice<Item> {
        (self as &mut [_]).into()
    }
}

unsafe impl<Item> BorrowUninitSlice<Item> for [Item; 16] {
    fn borrow_uninit_slice(&self) -> &[MaybeUninit<Item>] {
        unsafe {
            let slice = self as &[_];
            core::slice::from_raw_parts(slice.as_ptr() as *const MaybeUninit<Item>, slice.len())
        }
    }
}

unsafe impl<Item> BorrowOutSlice<Item> for [Item; 16] {
    fn borrow_out_slice(&mut self) -> &mut OutSlice<Item> {
        (self as &mut [_]).into()
    }

    fn zero_if_needed(&mut self) -> &mut [Item] {
        self
    }
}

unsafe impl<Item> BorrowUninitSlice<Item> for [MaybeUninit<Item>; 17] {
    fn borrow_uninit_slice(&self) -> &[MaybeUninit<Item>] {
        self
    }
}

unsafe impl<Item> BorrowOutSlice<Item> for [MaybeUninit<Item>; 17] {
    fn borrow_out_slice(&mut self) -> &mut OutSlice<Item> {
        (self as &mut [_]).into()
    }
}

unsafe impl<Item> BorrowUninitSlice<Item> for [Item; 17] {
    fn borrow_uninit_slice(&self) -> &[MaybeUninit<Item>] {
        unsafe {
            let slice = self as &[_];
            core::slice::from_raw_parts(slice.as_ptr() as *const MaybeUninit<Item>, slice.len())
        }
    }
}

unsafe impl<Item> BorrowOutSlice<Item> for [Item; 17] {
    fn borrow_out_slice(&mut self) -> &mut OutSlice<Item> {
        (self as &mut [_]).into()
    }

    fn zero_if_needed(&mut self) -> &mut [Item] {
        self
    }
}

unsafe impl<Item> BorrowUninitSlice<Item> for [MaybeUninit<Item>; 18] {
    fn borrow_uninit_slice(&self) -> &[MaybeUninit<Item>] {
        self
    }
}

unsafe impl<Item> BorrowOutSlice<Item> for [MaybeUninit<Item>; 18] {
    fn borrow_out_slice(&mut self) -> &mut OutSlice<Item> {
        (self as &mut [_]).into()
    }
}

unsafe impl<Item> BorrowUninitSlice<Item> for [Item; 18] {
    fn borrow_uninit_slice(&self) -> &[MaybeUninit<Item>] {
        unsafe {
            let slice = self as &[_];
            core::slice::from_raw_parts(slice.as_ptr() as *const MaybeUninit<Item>, slice.len())
        }
    }
}

unsafe impl<Item> BorrowOutSlice<Item> for [Item; 18] {
    fn borrow_out_slice(&mut self) -> &mut OutSlice<Item> {
        (self as &mut [_]).into()
    }

    fn zero_if_needed(&mut self) -> &mut [Item] {
        self
    }
}

unsafe impl<Item> BorrowUninitSlice<Item> for [MaybeUninit<Item>; 19] {
    fn borrow_uninit_slice(&self) -> &[MaybeUninit<Item>] {
        self
    }
}

unsafe impl<Item> BorrowOutSlice<Item> for [MaybeUninit<Item>; 19] {
    fn borrow_out_slice(&mut self) -> &mut OutSlice<Item> {
        (self as &mut [_]).into()
    }
}

unsafe impl<Item> BorrowUninitSlice<Item> for [Item; 19] {
    fn borrow_uninit_slice(&self) -> &[MaybeUninit<Item>] {
        unsafe {
            let slice = self as &[_];
            core::slice::from_raw_parts(slice.as_ptr() as *const MaybeUninit<Item>, slice.len())
        }
    }
}

unsafe impl<Item> BorrowOutSlice<Item> for [Item; 19] {
    fn borrow_out_slice(&mut self) -> &mut OutSlice<Item> {
        (self as &mut [_]).into()
    }

    fn zero_if_needed(&mut self) -> &mut [Item] {
        self
    }
}

unsafe impl<Item> BorrowUninitSlice<Item> for [MaybeUninit<Item>; 20] {
    fn borrow_uninit_slice(&self) -> &[MaybeUninit<Item>] {
        self
    }
}

unsafe impl<Item> BorrowOutSlice<Item> for [MaybeUninit<Item>; 20] {
    fn borrow_out_slice(&mut self) -> &mut OutSlice<Item> {
        (self as &mut [_]).into()
    }
}

unsafe impl<Item> BorrowUninitSlice<Item> for [Item; 20] {
    fn borrow_uninit_slice(&self) -> &[MaybeUninit<Item>] {
        unsafe {
            let slice = self as &[_];
            core::slice::from_raw_parts(slice.as_ptr() as *const MaybeUninit<Item>, slice.len())
        }
    }
}

unsafe impl<Item> BorrowOutSlice<Item> for [Item; 20] {
    fn borrow_out_slice(&mut self) -> &mut OutSlice<Item> {
        (self as &mut [_]).into()
    }

    fn zero_if_needed(&mut self) -> &mut [Item] {
        self
    }
}

unsafe impl<Item> BorrowUninitSlice<Item> for [MaybeUninit<Item>; 21] {
    fn borrow_uninit_slice(&self) -> &[MaybeUninit<Item>] {
        self
    }
}

unsafe impl<Item> BorrowOutSlice<Item> for [MaybeUninit<Item>; 21] {
    fn borrow_out_slice(&mut self) -> &mut OutSlice<Item> {
        (self as &mut [_]).into()
    }
}

unsafe impl<Item> BorrowUninitSlice<Item> for [Item; 21] {
    fn borrow_uninit_slice(&self) -> &[MaybeUninit<Item>] {
        unsafe {
            let slice = self as &[_];
            core::slice::from_raw_parts(slice.as_ptr() as *const MaybeUninit<Item>, slice.len())
        }
    }
}

unsafe impl<Item> BorrowOutSlice<Item> for [Item; 21] {
    fn borrow_out_slice(&mut self) -> &mut OutSlice<Item> {
        (self as &mut [_]).into()
    }

    fn zero_if_needed(&mut self) -> &mut [Item] {
        self
    }
}

unsafe impl<Item> BorrowUninitSlice<Item> for [MaybeUninit<Item>; 22] {
    fn borrow_uninit_slice(&self) -> &[MaybeUninit<Item>] {
        self
    }
}

unsafe impl<Item> BorrowOutSlice<Item> for [MaybeUninit<Item>; 22] {
    fn borrow_out_slice(&mut self) -> &mut OutSlice<Item> {
        (self as &mut [_]).into()
    }
}

unsafe impl<Item> BorrowUninitSlice<Item> for [Item; 22] {
    fn borrow_uninit_slice(&self) -> &[MaybeUninit<Item>] {
        unsafe {
            let slice = self as &[_];
            core::slice::from_raw_parts(slice.as_ptr() as *const MaybeUninit<Item>, slice.len())
        }
    }
}

unsafe impl<Item> BorrowOutSlice<Item> for [Item; 22] {
    fn borrow_out_slice(&mut self) -> &mut OutSlice<Item> {
        (self as &mut [_]).into()
    }

    fn zero_if_needed(&mut self) -> &mut [Item] {
        self
    }
}

unsafe impl<Item> BorrowUninitSlice<Item> for [MaybeUninit<Item>; 23] {
    fn borrow_uninit_slice(&self) -> &[MaybeUninit<Item>] {
        self
    }
}

unsafe impl<Item> BorrowOutSlice<Item> for [MaybeUninit<Item>; 23] {
    fn borrow_out_slice(&mut self) -> &mut OutSlice<Item> {
        (self as &mut [_]).into()
    }
}

unsafe impl<Item> BorrowUninitSlice<Item> for [Item; 23] {
    fn borrow_uninit_slice(&self) -> &[MaybeUninit<Item>] {
        unsafe {
            let slice = self as &[_];
            core::slice::from_raw_parts(slice.as_ptr() as *const MaybeUninit<Item>, slice.len())
        }
    }
}

unsafe impl<Item> BorrowOutSlice<Item> for [Item; 23] {
    fn borrow_out_slice(&mut self) -> &mut OutSlice<Item> {
        (self as &mut [_]).into()
    }

    fn zero_if_needed(&mut self) -> &mut [Item] {
        self
    }
}

unsafe impl<Item> BorrowUninitSlice<Item> for [MaybeUninit<Item>; 24] {
    fn borrow_uninit_slice(&self) -> &[MaybeUninit<Item>] {
        self
    }
}

unsafe impl<Item> BorrowOutSlice<Item> for [MaybeUninit<Item>; 24] {
    fn borrow_out_slice(&mut self) -> &mut OutSlice<Item> {
        (self as &mut [_]).into()
    }
}

unsafe impl<Item> BorrowUninitSlice<Item> for [Item; 24] {
    fn borrow_uninit_slice(&self) -> &[MaybeUninit<Item>] {
        unsafe {
            let slice = self as &[_];
            core::slice::from_raw_parts(slice.as_ptr() as *const MaybeUninit<Item>, slice.len())
        }
    }
}

unsafe impl<Item> BorrowOutSlice<Item> for [Item; 24] {
    fn borrow_out_slice(&mut self) -> &mut OutSlice<Item> {
        (self as &mut [_]).into()
    }

    fn zero_if_needed(&mut self) -> &mut [Item] {
        self
    }
}

unsafe impl<Item> BorrowUninitSlice<Item> for [MaybeUninit<Item>; 25] {
    fn borrow_uninit_slice(&self) -> &[MaybeUninit<Item>] {
        self
    }
}

unsafe impl<Item> BorrowOutSlice<Item> for [MaybeUninit<Item>; 25] {
    fn borrow_out_slice(&mut self) -> &mut OutSlice<Item> {
        (self as &mut [_]).into()
    }
}

unsafe impl<Item> BorrowUninitSlice<Item> for [Item; 25] {
    fn borrow_uninit_slice(&self) -> &[MaybeUninit<Item>] {
        unsafe {
            let slice = self as &[_];
            core::slice::from_raw_parts(slice.as_ptr() as *const MaybeUninit<Item>, slice.len())
        }
    }
}

unsafe impl<Item> BorrowOutSlice<Item> for [Item; 25] {
    fn borrow_out_slice(&mut self) -> &mut OutSlice<Item> {
        (self as &mut [_]).into()
    }

    fn zero_if_needed(&mut self) -> &mut [Item] {
        self
    }
}

unsafe impl<Item> BorrowUninitSlice<Item> for [MaybeUninit<Item>; 26] {
    fn borrow_uninit_slice(&self) -> &[MaybeUninit<Item>] {
        self
    }
}

unsafe impl<Item> BorrowOutSlice<Item> for [MaybeUninit<Item>; 26] {
    fn borrow_out_slice(&mut self) -> &mut OutSlice<Item> {
        (self as &mut [_]).into()
    }
}

unsafe impl<Item> BorrowUninitSlice<Item> for [Item; 26] {
    fn borrow_uninit_slice(&self) -> &[MaybeUninit<Item>] {
        unsafe {
            let slice = self as &[_];
            core::slice::from_raw_parts(slice.as_ptr() as *const MaybeUninit<Item>, slice.len())
        }
    }
}

unsafe impl<Item> BorrowOutSlice<Item> for [Item; 26] {
    fn borrow_out_slice(&mut self) -> &mut OutSlice<Item> {
        (self as &mut [_]).into()
    }

    fn zero_if_needed(&mut self) -> &mut [Item] {
        self
    }
}

unsafe impl<Item> BorrowUninitSlice<Item> for [MaybeUninit<Item>; 27] {
    fn borrow_uninit_slice(&self) -> &[MaybeUninit<Item>] {
        self
    }
}

unsafe impl<Item> BorrowOutSlice<Item> for [MaybeUninit<Item>; 27] {
    fn borrow_out_slice(&mut self) -> &mut OutSlice<Item> {
        (self as &mut [_]).into()
    }
}

unsafe impl<Item> BorrowUninitSlice<Item> for [Item; 27] {
    fn borrow_uninit_slice(&self) -> &[MaybeUninit<Item>] {
        unsafe {
            let slice = self as &[_];
            core::slice::from_raw_parts(slice.as_ptr() as *const MaybeUninit<Item>, slice.len())
        }
    }
}

unsafe impl<Item> BorrowOutSlice<Item> for [Item; 27] {
    fn borrow_out_slice(&mut self) -> &mut OutSlice<Item> {
        (self as &mut [_]).into()
    }

    fn zero_if_needed(&mut self) -> &mut [Item] {
        self
    }
}

unsafe impl<Item> BorrowUninitSlice<Item> for [MaybeUninit<Item>; 28] {
    fn borrow_uninit_slice(&self) -> &[MaybeUninit<Item>] {
        self
    }
}

unsafe impl<Item> BorrowOutSlice<Item> for [MaybeUninit<Item>; 28] {
    fn borrow_out_slice(&mut self) -> &mut OutSlice<Item> {
        (self as &mut [_]).into()
    }
}

unsafe impl<Item> BorrowUninitSlice<Item> for [Item; 28] {
    fn borrow_uninit_slice(&self) -> &[MaybeUninit<Item>] {
        unsafe {
            let slice = self as &[_];
            core::slice::from_raw_parts(slice.as_ptr() as *const MaybeUninit<Item>, slice.len())
        }
    }
}

unsafe impl<Item> BorrowOutSlice<Item> for [Item; 28] {
    fn borrow_out_slice(&mut self) -> &mut OutSlice<Item> {
        (self as &mut [_]).into()
    }

    fn zero_if_needed(&mut self) -> &mut [Item] {
        self
    }
}

unsafe impl<Item> BorrowUninitSlice<Item> for [MaybeUninit<Item>; 29] {
    fn borrow_uninit_slice(&self) -> &[MaybeUninit<Item>] {
        self
    }
}

unsafe impl<Item> BorrowOutSlice<Item> for [MaybeUninit<Item>; 29] {
    fn borrow_out_slice(&mut self) -> &mut OutSlice<Item> {
        (self as &mut [_]).into()
    }
}

unsafe impl<Item> BorrowUninitSlice<Item> for [Item; 29] {
    fn borrow_uninit_slice(&self) -> &[MaybeUninit<Item>] {
        unsafe {
            let slice = self as &[_];
            core::slice::from_raw_parts(slice.as_ptr() as *const MaybeUninit<Item>, slice.len())
        }
    }
}

unsafe impl<Item> BorrowOutSlice<Item> for [Item; 29] {
    fn borrow_out_slice(&mut self) -> &mut OutSlice<Item> {
        (self as &mut [_]).into()
    }

    fn zero_if_needed(&mut self) -> &mut [Item] {
        self
    }
}

unsafe impl<Item> BorrowUninitSlice<Item> for [MaybeUninit<Item>; 30] {
    fn borrow_uninit_slice(&self) -> &[MaybeUninit<Item>] {
        self
    }
}

unsafe impl<Item> BorrowOutSlice<Item> for [MaybeUninit<Item>; 30] {
    fn borrow_out_slice(&mut self) -> &mut OutSlice<Item> {
        (self as &mut [_]).into()
    }
}

unsafe impl<Item> BorrowUninitSlice<Item> for [Item; 30] {
    fn borrow_uninit_slice(&self) -> &[MaybeUninit<Item>] {
        unsafe {
            let slice = self as &[_];
            core::slice::from_raw_parts(slice.as_ptr() as *const MaybeUninit<Item>, slice.len())
        }
    }
}

unsafe impl<Item> BorrowOutSlice<Item> for [Item; 30] {
    fn borrow_out_slice(&mut self) -> &mut OutSlice<Item> {
        (self as &mut [_]).into()
    }

    fn zero_if_needed(&mut self) -> &mut [Item] {
        self
    }
}

unsafe impl<Item> BorrowUninitSlice<Item> for [MaybeUninit<Item>; 31] {
    fn borrow_uninit_slice(&self) -> &[MaybeUninit<Item>] {
        self
    }
}

unsafe impl<Item> BorrowOutSlice<Item> for [MaybeUninit<Item>; 31] {
    fn borrow_out_slice(&mut self) -> &mut OutSlice<Item> {
        (self as &mut [_]).into()
    }
}

unsafe impl<Item> BorrowUninitSlice<Item> for [Item; 31] {
    fn borrow_uninit_slice(&self) -> &[MaybeUninit<Item>] {
        unsafe {
            let slice = self as &[_];
            core::slice::from_raw_parts(slice.as_ptr() as *const MaybeUninit<Item>, slice.len())
        }
    }
}

unsafe impl<Item> BorrowOutSlice<Item> for [Item; 31] {
    fn borrow_out_slice(&mut self) -> &mut OutSlice<Item> {
        (self as &mut [_]).into()
    }

    fn zero_if_needed(&mut self) -> &mut [Item] {
        self
    }
}

unsafe impl<Item> BorrowUninitSlice<Item> for [MaybeUninit<Item>; 32] {
    fn borrow_uninit_slice(&self) -> &[MaybeUninit<Item>] {
        self
    }
}

unsafe impl<Item> BorrowOutSlice<Item> for [MaybeUninit<Item>; 32] {
    fn borrow_out_slice(&mut self) -> &mut OutSlice<Item> {
        (self as &mut [_]).into()
    }
}

unsafe impl<Item> BorrowUninitSlice<Item> for [Item; 32] {
    fn borrow_uninit_slice(&self) -> &[MaybeUninit<Item>] {
        unsafe {
            let slice = self as &[_];
            core::slice::from_raw_parts(slice.as_ptr() as *const MaybeUninit<Item>, slice.len())
        }
    }
}

unsafe impl<Item> BorrowOutSlice<Item> for [Item; 32] {
    fn borrow_out_slice(&mut self) -> &mut OutSlice<Item> {
        (self as &mut [_]).into()
    }

    fn zero_if_needed(&mut self) -> &mut [Item] {
        self
    }
}
