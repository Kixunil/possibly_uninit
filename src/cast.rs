//! Helpers for casting between smart pointer types.
//!
//! This module attempts to abstract casting operations between various smart
//! pointer types as well as non-pointer types. While it succeeds at its goal
//! in some ways, it fails in other ways.
//!
//! Unfortunately it's a bunch of `unsafe` code and it reached the point at
//! which I'm not as confident in its soundness as I would like to be.
//!
//! Currently it mainly attempts to enable `Cursor::try_cast_initialized` to
//! work with all kinds of sensible arrays: `[T; N]`, `UniqueRefType[T; N]`,
//! `UniqueRefType<[T]>` for `T` and `MaybeUninit<T>`. It does work for
//! `MaybeUninit<T>` arrays but not for `T` arrays. Which isn't too important
//! right now, but it'd be nice to resolve it.

use core::mem::MaybeUninit;
use core::ops::{Deref, DerefMut};
use crate::deref_markers::StableDeref;

// See rust-lang/rust/issues/61956
// Thanks to HadrienG2 for the workaround!
unsafe fn transmute_workaround_size_bug_super_dangerous_ignores_size<T, U>(mut x: T) -> U {
    assert_eq!(core::mem::size_of::<T>(), core::mem::size_of::<U>());

    let ptr = &mut x as *mut _ as *mut U;
    let res = ptr.read();
    core::mem::forget(x);
    res
}

/// Array type constructor
///
/// Constructs an array with same (possibly dynamic) size and different item.
pub trait ArrTc<Item> {
    /// Constructed array.
    type Arr: ?Sized;
    /// Dereferenced version of array.
    type Kind: ?Sized;
}

impl<'a, U: 'a, A: ArrTc<U> + ?Sized> ArrTc<U> for &'a mut A {
    type Arr = &'a mut <A as ArrTc<U>>::Arr;
    type Kind = <A as ArrTc<U>>::Kind;
}

#[cfg(feature = "alloc")]
impl<U, A: ArrTc<U> + ?Sized> ArrTc<U> for alloc::boxed::Box<A> {
    type Arr = alloc::boxed::Box<<A as ArrTc<U>>::Arr>;
    type Kind = <A as ArrTc<U>>::Kind;
}

impl<T, U> ArrTc<U> for [T] {
    type Arr = [U];
    type Kind = [U];
}

impl<T, U> ArrTc<U> for [T; 0] {
    type Arr = [U; 0];
    type Kind = [U; 0];
}

impl<T, U> ArrTc<U> for [T; 1] {
    type Arr = [U; 1];
    type Kind = [U; 1];
}

impl<T, U> ArrTc<U> for [T; 2] {
    type Arr = [U; 2];
    type Kind = [U; 2];
}

impl<T, U> ArrTc<U> for [T; 3] {
    type Arr = [U; 3];
    type Kind = [U; 3];
}

impl<T, U> ArrTc<U> for [T; 4] {
    type Arr = [U; 4];
    type Kind = [U; 4];
}

impl<T, U> ArrTc<U> for [T; 5] {
    type Arr = [U; 5];
    type Kind = [U; 5];
}

impl<T, U> ArrTc<U> for [T; 6] {
    type Arr = [U; 6];
    type Kind = [U; 6];
}

impl<T, U> ArrTc<U> for [T; 7] {
    type Arr = [U; 7];
    type Kind = [U; 7];
}

impl<T, U> ArrTc<U> for [T; 8] {
    type Arr = [U; 8];
    type Kind = [U; 8];
}

impl<T, U> ArrTc<U> for [T; 9] {
    type Arr = [U; 9];
    type Kind = [U; 9];
}

impl<T, U> ArrTc<U> for [T; 10] {
    type Arr = [U; 10];
    type Kind = [U; 10];
}

impl<T, U> ArrTc<U> for [T; 11] {
    type Arr = [U; 11];
    type Kind = [U; 11];
}

impl<T, U> ArrTc<U> for [T; 12] {
    type Arr = [U; 12];
    type Kind = [U; 12];
}

impl<T, U> ArrTc<U> for [T; 13] {
    type Arr = [U; 13];
    type Kind = [U; 13];
}

impl<T, U> ArrTc<U> for [T; 14] {
    type Arr = [U; 14];
    type Kind = [U; 14];
}

impl<T, U> ArrTc<U> for [T; 15] {
    type Arr = [U; 15];
    type Kind = [U; 15];
}

impl<T, U> ArrTc<U> for [T; 16] {
    type Arr = [U; 16];
    type Kind = [U; 16];
}

impl<T, U> ArrTc<U> for [T; 17] {
    type Arr = [U; 17];
    type Kind = [U; 17];
}

impl<T, U> ArrTc<U> for [T; 18] {
    type Arr = [U; 18];
    type Kind = [U; 18];
}

impl<T, U> ArrTc<U> for [T; 19] {
    type Arr = [U; 19];
    type Kind = [U; 19];
}

impl<T, U> ArrTc<U> for [T; 20] {
    type Arr = [U; 20];
    type Kind = [U; 20];
}

impl<T, U> ArrTc<U> for [T; 21] {
    type Arr = [U; 21];
    type Kind = [U; 21];
}

impl<T, U> ArrTc<U> for [T; 22] {
    type Arr = [U; 22];
    type Kind = [U; 22];
}

impl<T, U> ArrTc<U> for [T; 23] {
    type Arr = [U; 23];
    type Kind = [U; 23];
}

impl<T, U> ArrTc<U> for [T; 24] {
    type Arr = [U; 24];
    type Kind = [U; 24];
}

impl<T, U> ArrTc<U> for [T; 25] {
    type Arr = [U; 25];
    type Kind = [U; 25];
}

impl<T, U> ArrTc<U> for [T; 26] {
    type Arr = [U; 26];
    type Kind = [U; 26];
}

impl<T, U> ArrTc<U> for [T; 27] {
    type Arr = [U; 27];
    type Kind = [U; 27];
}

impl<T, U> ArrTc<U> for [T; 28] {
    type Arr = [U; 28];
    type Kind = [U; 28];
}

impl<T, U> ArrTc<U> for [T; 29] {
    type Arr = [U; 29];
    type Kind = [U; 29];
}

impl<T, U> ArrTc<U> for [T; 30] {
    type Arr = [U; 30];
    type Kind = [U; 30];
}

impl<T, U> ArrTc<U> for [T; 31] {
    type Arr = [U; 31];
    type Kind = [U; 31];
}

impl<T, U> ArrTc<U> for [T; 32] {
    type Arr = [U; 32];
    type Kind = [U; 32];
}

/// Initialization type constructor
///
/// For a possibly uninitialized type it constructs an initialied version of the type.
pub trait InitTc<Item>: ArrTc<Item> {
    /// Resulting type
    type Output: Sized;

    /// Casts the types.
    unsafe fn cast(self) -> Self::Output;
}

impl<Item, T> InitTc<Item> for T where T: ArrTc<Item>, (Self, <Self as ArrTc<Item>>::Kind): CastArrHelper<From=Self> {
    type Output = <(Self, <Self as ArrTc<Item>>::Kind) as CastArrHelper>::To;

    unsafe fn cast(self) -> Self::Output {
        <(Self, <Self as ArrTc<Item>>::Kind)>::cast(self)
    }
}

/// A trick to work arround the limitation of Rust where the compiler can't understand that two
/// impls with different associated types aren't conflicting.
pub unsafe trait CastArrHelper {
    /// Type from which we are casting
    type From: Sized;

    /// Type into which we are casting
    type To: Sized;

    /// Cast those two things
    unsafe fn cast(from: Self::From) -> Self::To;
}

unsafe impl<P, T> CastArrHelper for (P, [T]) where P: SlicePointerMut<Item=MaybeUninit<T>> + Deref<Target=[MaybeUninit<T>]> + PtrTc<[T]>, <P as PtrTc<[T]>>::Pointer: SlicePointerMut<Item=T> + Deref<Target=[T]> {
    type From = P;
    type To = <P as PtrTc<[T]>>::Pointer;

    unsafe fn cast(from: Self::From) -> Self::To {
        let (ptr, len) = from.into_raw_parts_mut();
        Self::To::from_raw_parts_mut(ptr as *mut <Self::To as SlicePointerMut>::Item, len)
    }
}

// Dear reader, I really, really recommend learning Vim and its record tool (`q` key).
// It's extremely helpful at doing stuff like this.
unsafe impl<T> CastArrHelper for ([T; 0], [T; 0]) {
    type From = [T; 0];
    type To = [T; 0];

    unsafe fn cast(from: Self::From) -> Self::To {
        transmute_workaround_size_bug_super_dangerous_ignores_size(from)
    }
}

unsafe impl<T> CastArrHelper for ([MaybeUninit<T>; 0], [T; 0]) {
    type From = [MaybeUninit<T>; 0];
    type To = [T; 0];

    unsafe fn cast(from: Self::From) -> Self::To {
        transmute_workaround_size_bug_super_dangerous_ignores_size(from)
    }
}

unsafe impl<P, T> CastArrHelper for (P, [T; 0]) where P: PointerMut + Deref<Target=[MaybeUninit<T>; 0]> + PtrTc<[T; 0]>, <P as PtrTc<[T; 0]>>::Pointer: PointerMut + Deref<Target=[T; 0]> {
    type From = P;
    type To = <P as PtrTc<[T; 0]>>::Pointer;

    unsafe fn cast(from: Self::From) -> Self::To {
        Self::To::from_raw_mut(from.into_raw_mut() as *mut <Self::To as Deref>::Target)
    }
}

unsafe impl<T> CastArrHelper for ([T; 1], [T; 1]) {
    type From = [T; 1];
    type To = [T; 1];

    unsafe fn cast(from: Self::From) -> Self::To {
        transmute_workaround_size_bug_super_dangerous_ignores_size(from)
    }
}

unsafe impl<T> CastArrHelper for ([MaybeUninit<T>; 1], [T; 1]) {
    type From = [MaybeUninit<T>; 1];
    type To = [T; 1];

    unsafe fn cast(from: Self::From) -> Self::To {
        transmute_workaround_size_bug_super_dangerous_ignores_size(from)
    }
}

unsafe impl<P, T> CastArrHelper for (P, [T; 1]) where P: PointerMut + Deref<Target=[MaybeUninit<T>; 1]> + PtrTc<[T; 1]>, <P as PtrTc<[T; 1]>>::Pointer: PointerMut + Deref<Target=[T; 1]> {
    type From = P;
    type To = <P as PtrTc<[T; 1]>>::Pointer;

    unsafe fn cast(from: Self::From) -> Self::To {
        Self::To::from_raw_mut(from.into_raw_mut() as *mut <Self::To as Deref>::Target)
    }
}

unsafe impl<T> CastArrHelper for ([T; 2], [T; 2]) {
    type From = [T; 2];
    type To = [T; 2];

    unsafe fn cast(from: Self::From) -> Self::To {
        transmute_workaround_size_bug_super_dangerous_ignores_size(from)
    }
}

unsafe impl<T> CastArrHelper for ([MaybeUninit<T>; 2], [T; 2]) {
    type From = [MaybeUninit<T>; 2];
    type To = [T; 2];

    unsafe fn cast(from: Self::From) -> Self::To {
        transmute_workaround_size_bug_super_dangerous_ignores_size(from)
    }
}

unsafe impl<P, T> CastArrHelper for (P, [T; 2]) where P: PointerMut + Deref<Target=[MaybeUninit<T>; 2]> + PtrTc<[T; 2]>, <P as PtrTc<[T; 2]>>::Pointer: PointerMut + Deref<Target=[T; 2]> {
    type From = P;
    type To = <P as PtrTc<[T; 2]>>::Pointer;

    unsafe fn cast(from: Self::From) -> Self::To {
        Self::To::from_raw_mut(from.into_raw_mut() as *mut <Self::To as Deref>::Target)
    }
}

unsafe impl<T> CastArrHelper for ([T; 3], [T; 3]) {
    type From = [T; 3];
    type To = [T; 3];

    unsafe fn cast(from: Self::From) -> Self::To {
        transmute_workaround_size_bug_super_dangerous_ignores_size(from)
    }
}

unsafe impl<T> CastArrHelper for ([MaybeUninit<T>; 3], [T; 3]) {
    type From = [MaybeUninit<T>; 3];
    type To = [T; 3];

    unsafe fn cast(from: Self::From) -> Self::To {
        transmute_workaround_size_bug_super_dangerous_ignores_size(from)
    }
}

unsafe impl<P, T> CastArrHelper for (P, [T; 3]) where P: PointerMut + Deref<Target=[MaybeUninit<T>; 3]> + PtrTc<[T; 3]>, <P as PtrTc<[T; 3]>>::Pointer: PointerMut + Deref<Target=[T; 3]> {
    type From = P;
    type To = <P as PtrTc<[T; 3]>>::Pointer;

    unsafe fn cast(from: Self::From) -> Self::To {
        Self::To::from_raw_mut(from.into_raw_mut() as *mut <Self::To as Deref>::Target)
    }
}

unsafe impl<T> CastArrHelper for ([T; 4], [T; 4]) {
    type From = [T; 4];
    type To = [T; 4];

    unsafe fn cast(from: Self::From) -> Self::To {
        transmute_workaround_size_bug_super_dangerous_ignores_size(from)
    }
}

unsafe impl<T> CastArrHelper for ([MaybeUninit<T>; 4], [T; 4]) {
    type From = [MaybeUninit<T>; 4];
    type To = [T; 4];

    unsafe fn cast(from: Self::From) -> Self::To {
        transmute_workaround_size_bug_super_dangerous_ignores_size(from)
    }
}

unsafe impl<P, T> CastArrHelper for (P, [T; 4]) where P: PointerMut + Deref<Target=[MaybeUninit<T>; 4]> + PtrTc<[T; 4]>, <P as PtrTc<[T; 4]>>::Pointer: PointerMut + Deref<Target=[T; 4]> {
    type From = P;
    type To = <P as PtrTc<[T; 4]>>::Pointer;

    unsafe fn cast(from: Self::From) -> Self::To {
        Self::To::from_raw_mut(from.into_raw_mut() as *mut <Self::To as Deref>::Target)
    }
}

unsafe impl<T> CastArrHelper for ([T; 5], [T; 5]) {
    type From = [T; 5];
    type To = [T; 5];

    unsafe fn cast(from: Self::From) -> Self::To {
        transmute_workaround_size_bug_super_dangerous_ignores_size(from)
    }
}

unsafe impl<T> CastArrHelper for ([MaybeUninit<T>; 5], [T; 5]) {
    type From = [MaybeUninit<T>; 5];
    type To = [T; 5];

    unsafe fn cast(from: Self::From) -> Self::To {
        transmute_workaround_size_bug_super_dangerous_ignores_size(from)
    }
}

unsafe impl<P, T> CastArrHelper for (P, [T; 5]) where P: PointerMut + Deref<Target=[MaybeUninit<T>; 5]> + PtrTc<[T; 5]>, <P as PtrTc<[T; 5]>>::Pointer: PointerMut + Deref<Target=[T; 5]> {
    type From = P;
    type To = <P as PtrTc<[T; 5]>>::Pointer;

    unsafe fn cast(from: Self::From) -> Self::To {
        Self::To::from_raw_mut(from.into_raw_mut() as *mut <Self::To as Deref>::Target)
    }
}

unsafe impl<T> CastArrHelper for ([T; 6], [T; 6]) {
    type From = [T; 6];
    type To = [T; 6];

    unsafe fn cast(from: Self::From) -> Self::To {
        transmute_workaround_size_bug_super_dangerous_ignores_size(from)
    }
}

unsafe impl<T> CastArrHelper for ([MaybeUninit<T>; 6], [T; 6]) {
    type From = [MaybeUninit<T>; 6];
    type To = [T; 6];

    unsafe fn cast(from: Self::From) -> Self::To {
        transmute_workaround_size_bug_super_dangerous_ignores_size(from)
    }
}

unsafe impl<P, T> CastArrHelper for (P, [T; 6]) where P: PointerMut + Deref<Target=[MaybeUninit<T>; 6]> + PtrTc<[T; 6]>, <P as PtrTc<[T; 6]>>::Pointer: PointerMut + Deref<Target=[T; 6]> {
    type From = P;
    type To = <P as PtrTc<[T; 6]>>::Pointer;

    unsafe fn cast(from: Self::From) -> Self::To {
        Self::To::from_raw_mut(from.into_raw_mut() as *mut <Self::To as Deref>::Target)
    }
}

unsafe impl<T> CastArrHelper for ([T; 7], [T; 7]) {
    type From = [T; 7];
    type To = [T; 7];

    unsafe fn cast(from: Self::From) -> Self::To {
        transmute_workaround_size_bug_super_dangerous_ignores_size(from)
    }
}

unsafe impl<T> CastArrHelper for ([MaybeUninit<T>; 7], [T; 7]) {
    type From = [MaybeUninit<T>; 7];
    type To = [T; 7];

    unsafe fn cast(from: Self::From) -> Self::To {
        transmute_workaround_size_bug_super_dangerous_ignores_size(from)
    }
}

unsafe impl<P, T> CastArrHelper for (P, [T; 7]) where P: PointerMut + Deref<Target=[MaybeUninit<T>; 7]> + PtrTc<[T; 7]>, <P as PtrTc<[T; 7]>>::Pointer: PointerMut + Deref<Target=[T; 7]> {
    type From = P;
    type To = <P as PtrTc<[T; 7]>>::Pointer;

    unsafe fn cast(from: Self::From) -> Self::To {
        Self::To::from_raw_mut(from.into_raw_mut() as *mut <Self::To as Deref>::Target)
    }
}

unsafe impl<T> CastArrHelper for ([T; 8], [T; 8]) {
    type From = [T; 8];
    type To = [T; 8];

    unsafe fn cast(from: Self::From) -> Self::To {
        transmute_workaround_size_bug_super_dangerous_ignores_size(from)
    }
}

unsafe impl<T> CastArrHelper for ([MaybeUninit<T>; 8], [T; 8]) {
    type From = [MaybeUninit<T>; 8];
    type To = [T; 8];

    unsafe fn cast(from: Self::From) -> Self::To {
        transmute_workaround_size_bug_super_dangerous_ignores_size(from)
    }
}

unsafe impl<P, T> CastArrHelper for (P, [T; 8]) where P: PointerMut + Deref<Target=[MaybeUninit<T>; 8]> + PtrTc<[T; 8]>, <P as PtrTc<[T; 8]>>::Pointer: PointerMut + Deref<Target=[T; 8]> {
    type From = P;
    type To = <P as PtrTc<[T; 8]>>::Pointer;

    unsafe fn cast(from: Self::From) -> Self::To {
        Self::To::from_raw_mut(from.into_raw_mut() as *mut <Self::To as Deref>::Target)
    }
}

unsafe impl<T> CastArrHelper for ([T; 9], [T; 9]) {
    type From = [T; 9];
    type To = [T; 9];

    unsafe fn cast(from: Self::From) -> Self::To {
        transmute_workaround_size_bug_super_dangerous_ignores_size(from)
    }
}

unsafe impl<T> CastArrHelper for ([MaybeUninit<T>; 9], [T; 9]) {
    type From = [MaybeUninit<T>; 9];
    type To = [T; 9];

    unsafe fn cast(from: Self::From) -> Self::To {
        transmute_workaround_size_bug_super_dangerous_ignores_size(from)
    }
}

unsafe impl<P, T> CastArrHelper for (P, [T; 9]) where P: PointerMut + Deref<Target=[MaybeUninit<T>; 9]> + PtrTc<[T; 9]>, <P as PtrTc<[T; 9]>>::Pointer: PointerMut + Deref<Target=[T; 9]> {
    type From = P;
    type To = <P as PtrTc<[T; 9]>>::Pointer;

    unsafe fn cast(from: Self::From) -> Self::To {
        Self::To::from_raw_mut(from.into_raw_mut() as *mut <Self::To as Deref>::Target)
    }
}

unsafe impl<T> CastArrHelper for ([T; 10], [T; 10]) {
    type From = [T; 10];
    type To = [T; 10];

    unsafe fn cast(from: Self::From) -> Self::To {
        transmute_workaround_size_bug_super_dangerous_ignores_size(from)
    }
}

unsafe impl<T> CastArrHelper for ([MaybeUninit<T>; 10], [T; 10]) {
    type From = [MaybeUninit<T>; 10];
    type To = [T; 10];

    unsafe fn cast(from: Self::From) -> Self::To {
        transmute_workaround_size_bug_super_dangerous_ignores_size(from)
    }
}

unsafe impl<P, T> CastArrHelper for (P, [T; 10]) where P: PointerMut + Deref<Target=[MaybeUninit<T>; 10]> + PtrTc<[T; 10]>, <P as PtrTc<[T; 10]>>::Pointer: PointerMut + Deref<Target=[T; 10]> {
    type From = P;
    type To = <P as PtrTc<[T; 10]>>::Pointer;

    unsafe fn cast(from: Self::From) -> Self::To {
        Self::To::from_raw_mut(from.into_raw_mut() as *mut <Self::To as Deref>::Target)
    }
}

unsafe impl<T> CastArrHelper for ([T; 11], [T; 11]) {
    type From = [T; 11];
    type To = [T; 11];

    unsafe fn cast(from: Self::From) -> Self::To {
        transmute_workaround_size_bug_super_dangerous_ignores_size(from)
    }
}

unsafe impl<T> CastArrHelper for ([MaybeUninit<T>; 11], [T; 11]) {
    type From = [MaybeUninit<T>; 11];
    type To = [T; 11];

    unsafe fn cast(from: Self::From) -> Self::To {
        transmute_workaround_size_bug_super_dangerous_ignores_size(from)
    }
}

unsafe impl<P, T> CastArrHelper for (P, [T; 11]) where P: PointerMut + Deref<Target=[MaybeUninit<T>; 11]> + PtrTc<[T; 11]>, <P as PtrTc<[T; 11]>>::Pointer: PointerMut + Deref<Target=[T; 11]> {
    type From = P;
    type To = <P as PtrTc<[T; 11]>>::Pointer;

    unsafe fn cast(from: Self::From) -> Self::To {
        Self::To::from_raw_mut(from.into_raw_mut() as *mut <Self::To as Deref>::Target)
    }
}

unsafe impl<T> CastArrHelper for ([T; 12], [T; 12]) {
    type From = [T; 12];
    type To = [T; 12];

    unsafe fn cast(from: Self::From) -> Self::To {
        transmute_workaround_size_bug_super_dangerous_ignores_size(from)
    }
}

unsafe impl<T> CastArrHelper for ([MaybeUninit<T>; 12], [T; 12]) {
    type From = [MaybeUninit<T>; 12];
    type To = [T; 12];

    unsafe fn cast(from: Self::From) -> Self::To {
        transmute_workaround_size_bug_super_dangerous_ignores_size(from)
    }
}

unsafe impl<P, T> CastArrHelper for (P, [T; 12]) where P: PointerMut + Deref<Target=[MaybeUninit<T>; 12]> + PtrTc<[T; 12]>, <P as PtrTc<[T; 12]>>::Pointer: PointerMut + Deref<Target=[T; 12]> {
    type From = P;
    type To = <P as PtrTc<[T; 12]>>::Pointer;

    unsafe fn cast(from: Self::From) -> Self::To {
        Self::To::from_raw_mut(from.into_raw_mut() as *mut <Self::To as Deref>::Target)
    }
}

unsafe impl<T> CastArrHelper for ([T; 13], [T; 13]) {
    type From = [T; 13];
    type To = [T; 13];

    unsafe fn cast(from: Self::From) -> Self::To {
        transmute_workaround_size_bug_super_dangerous_ignores_size(from)
    }
}

unsafe impl<T> CastArrHelper for ([MaybeUninit<T>; 13], [T; 13]) {
    type From = [MaybeUninit<T>; 13];
    type To = [T; 13];

    unsafe fn cast(from: Self::From) -> Self::To {
        transmute_workaround_size_bug_super_dangerous_ignores_size(from)
    }
}

unsafe impl<P, T> CastArrHelper for (P, [T; 13]) where P: PointerMut + Deref<Target=[MaybeUninit<T>; 13]> + PtrTc<[T; 13]>, <P as PtrTc<[T; 13]>>::Pointer: PointerMut + Deref<Target=[T; 13]> {
    type From = P;
    type To = <P as PtrTc<[T; 13]>>::Pointer;

    unsafe fn cast(from: Self::From) -> Self::To {
        Self::To::from_raw_mut(from.into_raw_mut() as *mut <Self::To as Deref>::Target)
    }
}

unsafe impl<T> CastArrHelper for ([T; 14], [T; 14]) {
    type From = [T; 14];
    type To = [T; 14];

    unsafe fn cast(from: Self::From) -> Self::To {
        transmute_workaround_size_bug_super_dangerous_ignores_size(from)
    }
}

unsafe impl<T> CastArrHelper for ([MaybeUninit<T>; 14], [T; 14]) {
    type From = [MaybeUninit<T>; 14];
    type To = [T; 14];

    unsafe fn cast(from: Self::From) -> Self::To {
        transmute_workaround_size_bug_super_dangerous_ignores_size(from)
    }
}

unsafe impl<P, T> CastArrHelper for (P, [T; 14]) where P: PointerMut + Deref<Target=[MaybeUninit<T>; 14]> + PtrTc<[T; 14]>, <P as PtrTc<[T; 14]>>::Pointer: PointerMut + Deref<Target=[T; 14]> {
    type From = P;
    type To = <P as PtrTc<[T; 14]>>::Pointer;

    unsafe fn cast(from: Self::From) -> Self::To {
        Self::To::from_raw_mut(from.into_raw_mut() as *mut <Self::To as Deref>::Target)
    }
}

unsafe impl<T> CastArrHelper for ([T; 15], [T; 15]) {
    type From = [T; 15];
    type To = [T; 15];

    unsafe fn cast(from: Self::From) -> Self::To {
        transmute_workaround_size_bug_super_dangerous_ignores_size(from)
    }
}

unsafe impl<T> CastArrHelper for ([MaybeUninit<T>; 15], [T; 15]) {
    type From = [MaybeUninit<T>; 15];
    type To = [T; 15];

    unsafe fn cast(from: Self::From) -> Self::To {
        transmute_workaround_size_bug_super_dangerous_ignores_size(from)
    }
}

unsafe impl<P, T> CastArrHelper for (P, [T; 15]) where P: PointerMut + Deref<Target=[MaybeUninit<T>; 15]> + PtrTc<[T; 15]>, <P as PtrTc<[T; 15]>>::Pointer: PointerMut + Deref<Target=[T; 15]> {
    type From = P;
    type To = <P as PtrTc<[T; 15]>>::Pointer;

    unsafe fn cast(from: Self::From) -> Self::To {
        Self::To::from_raw_mut(from.into_raw_mut() as *mut <Self::To as Deref>::Target)
    }
}

unsafe impl<T> CastArrHelper for ([T; 16], [T; 16]) {
    type From = [T; 16];
    type To = [T; 16];

    unsafe fn cast(from: Self::From) -> Self::To {
        transmute_workaround_size_bug_super_dangerous_ignores_size(from)
    }
}

unsafe impl<T> CastArrHelper for ([MaybeUninit<T>; 16], [T; 16]) {
    type From = [MaybeUninit<T>; 16];
    type To = [T; 16];

    unsafe fn cast(from: Self::From) -> Self::To {
        transmute_workaround_size_bug_super_dangerous_ignores_size(from)
    }
}

unsafe impl<P, T> CastArrHelper for (P, [T; 16]) where P: PointerMut + Deref<Target=[MaybeUninit<T>; 16]> + PtrTc<[T; 16]>, <P as PtrTc<[T; 16]>>::Pointer: PointerMut + Deref<Target=[T; 16]> {
    type From = P;
    type To = <P as PtrTc<[T; 16]>>::Pointer;

    unsafe fn cast(from: Self::From) -> Self::To {
        Self::To::from_raw_mut(from.into_raw_mut() as *mut <Self::To as Deref>::Target)
    }
}

unsafe impl<T> CastArrHelper for ([T; 17], [T; 17]) {
    type From = [T; 17];
    type To = [T; 17];

    unsafe fn cast(from: Self::From) -> Self::To {
        transmute_workaround_size_bug_super_dangerous_ignores_size(from)
    }
}

unsafe impl<T> CastArrHelper for ([MaybeUninit<T>; 17], [T; 17]) {
    type From = [MaybeUninit<T>; 17];
    type To = [T; 17];

    unsafe fn cast(from: Self::From) -> Self::To {
        transmute_workaround_size_bug_super_dangerous_ignores_size(from)
    }
}

unsafe impl<P, T> CastArrHelper for (P, [T; 17]) where P: PointerMut + Deref<Target=[MaybeUninit<T>; 17]> + PtrTc<[T; 17]>, <P as PtrTc<[T; 17]>>::Pointer: PointerMut + Deref<Target=[T; 17]> {
    type From = P;
    type To = <P as PtrTc<[T; 17]>>::Pointer;

    unsafe fn cast(from: Self::From) -> Self::To {
        Self::To::from_raw_mut(from.into_raw_mut() as *mut <Self::To as Deref>::Target)
    }
}

unsafe impl<T> CastArrHelper for ([T; 18], [T; 18]) {
    type From = [T; 18];
    type To = [T; 18];

    unsafe fn cast(from: Self::From) -> Self::To {
        transmute_workaround_size_bug_super_dangerous_ignores_size(from)
    }
}

unsafe impl<T> CastArrHelper for ([MaybeUninit<T>; 18], [T; 18]) {
    type From = [MaybeUninit<T>; 18];
    type To = [T; 18];

    unsafe fn cast(from: Self::From) -> Self::To {
        transmute_workaround_size_bug_super_dangerous_ignores_size(from)
    }
}

unsafe impl<P, T> CastArrHelper for (P, [T; 18]) where P: PointerMut + Deref<Target=[MaybeUninit<T>; 18]> + PtrTc<[T; 18]>, <P as PtrTc<[T; 18]>>::Pointer: PointerMut + Deref<Target=[T; 18]> {
    type From = P;
    type To = <P as PtrTc<[T; 18]>>::Pointer;

    unsafe fn cast(from: Self::From) -> Self::To {
        Self::To::from_raw_mut(from.into_raw_mut() as *mut <Self::To as Deref>::Target)
    }
}

unsafe impl<T> CastArrHelper for ([T; 19], [T; 19]) {
    type From = [T; 19];
    type To = [T; 19];

    unsafe fn cast(from: Self::From) -> Self::To {
        transmute_workaround_size_bug_super_dangerous_ignores_size(from)
    }
}

unsafe impl<T> CastArrHelper for ([MaybeUninit<T>; 19], [T; 19]) {
    type From = [MaybeUninit<T>; 19];
    type To = [T; 19];

    unsafe fn cast(from: Self::From) -> Self::To {
        transmute_workaround_size_bug_super_dangerous_ignores_size(from)
    }
}

unsafe impl<P, T> CastArrHelper for (P, [T; 19]) where P: PointerMut + Deref<Target=[MaybeUninit<T>; 19]> + PtrTc<[T; 19]>, <P as PtrTc<[T; 19]>>::Pointer: PointerMut + Deref<Target=[T; 19]> {
    type From = P;
    type To = <P as PtrTc<[T; 19]>>::Pointer;

    unsafe fn cast(from: Self::From) -> Self::To {
        Self::To::from_raw_mut(from.into_raw_mut() as *mut <Self::To as Deref>::Target)
    }
}

unsafe impl<T> CastArrHelper for ([T; 20], [T; 20]) {
    type From = [T; 20];
    type To = [T; 20];

    unsafe fn cast(from: Self::From) -> Self::To {
        transmute_workaround_size_bug_super_dangerous_ignores_size(from)
    }
}

unsafe impl<T> CastArrHelper for ([MaybeUninit<T>; 20], [T; 20]) {
    type From = [MaybeUninit<T>; 20];
    type To = [T; 20];

    unsafe fn cast(from: Self::From) -> Self::To {
        transmute_workaround_size_bug_super_dangerous_ignores_size(from)
    }
}

unsafe impl<P, T> CastArrHelper for (P, [T; 20]) where P: PointerMut + Deref<Target=[MaybeUninit<T>; 20]> + PtrTc<[T; 20]>, <P as PtrTc<[T; 20]>>::Pointer: PointerMut + Deref<Target=[T; 20]> {
    type From = P;
    type To = <P as PtrTc<[T; 20]>>::Pointer;

    unsafe fn cast(from: Self::From) -> Self::To {
        Self::To::from_raw_mut(from.into_raw_mut() as *mut <Self::To as Deref>::Target)
    }
}

unsafe impl<T> CastArrHelper for ([T; 21], [T; 21]) {
    type From = [T; 21];
    type To = [T; 21];

    unsafe fn cast(from: Self::From) -> Self::To {
        transmute_workaround_size_bug_super_dangerous_ignores_size(from)
    }
}

unsafe impl<T> CastArrHelper for ([MaybeUninit<T>; 21], [T; 21]) {
    type From = [MaybeUninit<T>; 21];
    type To = [T; 21];

    unsafe fn cast(from: Self::From) -> Self::To {
        transmute_workaround_size_bug_super_dangerous_ignores_size(from)
    }
}

unsafe impl<P, T> CastArrHelper for (P, [T; 21]) where P: PointerMut + Deref<Target=[MaybeUninit<T>; 21]> + PtrTc<[T; 21]>, <P as PtrTc<[T; 21]>>::Pointer: PointerMut + Deref<Target=[T; 21]> {
    type From = P;
    type To = <P as PtrTc<[T; 21]>>::Pointer;

    unsafe fn cast(from: Self::From) -> Self::To {
        Self::To::from_raw_mut(from.into_raw_mut() as *mut <Self::To as Deref>::Target)
    }
}

unsafe impl<T> CastArrHelper for ([T; 22], [T; 22]) {
    type From = [T; 22];
    type To = [T; 22];

    unsafe fn cast(from: Self::From) -> Self::To {
        transmute_workaround_size_bug_super_dangerous_ignores_size(from)
    }
}

unsafe impl<T> CastArrHelper for ([MaybeUninit<T>; 22], [T; 22]) {
    type From = [MaybeUninit<T>; 22];
    type To = [T; 22];

    unsafe fn cast(from: Self::From) -> Self::To {
        transmute_workaround_size_bug_super_dangerous_ignores_size(from)
    }
}

unsafe impl<P, T> CastArrHelper for (P, [T; 22]) where P: PointerMut + Deref<Target=[MaybeUninit<T>; 22]> + PtrTc<[T; 22]>, <P as PtrTc<[T; 22]>>::Pointer: PointerMut + Deref<Target=[T; 22]> {
    type From = P;
    type To = <P as PtrTc<[T; 22]>>::Pointer;

    unsafe fn cast(from: Self::From) -> Self::To {
        Self::To::from_raw_mut(from.into_raw_mut() as *mut <Self::To as Deref>::Target)
    }
}

unsafe impl<T> CastArrHelper for ([T; 23], [T; 23]) {
    type From = [T; 23];
    type To = [T; 23];

    unsafe fn cast(from: Self::From) -> Self::To {
        transmute_workaround_size_bug_super_dangerous_ignores_size(from)
    }
}

unsafe impl<T> CastArrHelper for ([MaybeUninit<T>; 23], [T; 23]) {
    type From = [MaybeUninit<T>; 23];
    type To = [T; 23];

    unsafe fn cast(from: Self::From) -> Self::To {
        transmute_workaround_size_bug_super_dangerous_ignores_size(from)
    }
}

unsafe impl<P, T> CastArrHelper for (P, [T; 23]) where P: PointerMut + Deref<Target=[MaybeUninit<T>; 23]> + PtrTc<[T; 23]>, <P as PtrTc<[T; 23]>>::Pointer: PointerMut + Deref<Target=[T; 23]> {
    type From = P;
    type To = <P as PtrTc<[T; 23]>>::Pointer;

    unsafe fn cast(from: Self::From) -> Self::To {
        Self::To::from_raw_mut(from.into_raw_mut() as *mut <Self::To as Deref>::Target)
    }
}

unsafe impl<T> CastArrHelper for ([T; 24], [T; 24]) {
    type From = [T; 24];
    type To = [T; 24];

    unsafe fn cast(from: Self::From) -> Self::To {
        transmute_workaround_size_bug_super_dangerous_ignores_size(from)
    }
}

unsafe impl<T> CastArrHelper for ([MaybeUninit<T>; 24], [T; 24]) {
    type From = [MaybeUninit<T>; 24];
    type To = [T; 24];

    unsafe fn cast(from: Self::From) -> Self::To {
        transmute_workaround_size_bug_super_dangerous_ignores_size(from)
    }
}

unsafe impl<P, T> CastArrHelper for (P, [T; 24]) where P: PointerMut + Deref<Target=[MaybeUninit<T>; 24]> + PtrTc<[T; 24]>, <P as PtrTc<[T; 24]>>::Pointer: PointerMut + Deref<Target=[T; 24]> {
    type From = P;
    type To = <P as PtrTc<[T; 24]>>::Pointer;

    unsafe fn cast(from: Self::From) -> Self::To {
        Self::To::from_raw_mut(from.into_raw_mut() as *mut <Self::To as Deref>::Target)
    }
}

unsafe impl<T> CastArrHelper for ([T; 25], [T; 25]) {
    type From = [T; 25];
    type To = [T; 25];

    unsafe fn cast(from: Self::From) -> Self::To {
        transmute_workaround_size_bug_super_dangerous_ignores_size(from)
    }
}

unsafe impl<T> CastArrHelper for ([MaybeUninit<T>; 25], [T; 25]) {
    type From = [MaybeUninit<T>; 25];
    type To = [T; 25];

    unsafe fn cast(from: Self::From) -> Self::To {
        transmute_workaround_size_bug_super_dangerous_ignores_size(from)
    }
}

unsafe impl<P, T> CastArrHelper for (P, [T; 25]) where P: PointerMut + Deref<Target=[MaybeUninit<T>; 25]> + PtrTc<[T; 25]>, <P as PtrTc<[T; 25]>>::Pointer: PointerMut + Deref<Target=[T; 25]> {
    type From = P;
    type To = <P as PtrTc<[T; 25]>>::Pointer;

    unsafe fn cast(from: Self::From) -> Self::To {
        Self::To::from_raw_mut(from.into_raw_mut() as *mut <Self::To as Deref>::Target)
    }
}

unsafe impl<T> CastArrHelper for ([T; 26], [T; 26]) {
    type From = [T; 26];
    type To = [T; 26];

    unsafe fn cast(from: Self::From) -> Self::To {
        transmute_workaround_size_bug_super_dangerous_ignores_size(from)
    }
}

unsafe impl<T> CastArrHelper for ([MaybeUninit<T>; 26], [T; 26]) {
    type From = [MaybeUninit<T>; 26];
    type To = [T; 26];

    unsafe fn cast(from: Self::From) -> Self::To {
        transmute_workaround_size_bug_super_dangerous_ignores_size(from)
    }
}

unsafe impl<P, T> CastArrHelper for (P, [T; 26]) where P: PointerMut + Deref<Target=[MaybeUninit<T>; 26]> + PtrTc<[T; 26]>, <P as PtrTc<[T; 26]>>::Pointer: PointerMut + Deref<Target=[T; 26]> {
    type From = P;
    type To = <P as PtrTc<[T; 26]>>::Pointer;

    unsafe fn cast(from: Self::From) -> Self::To {
        Self::To::from_raw_mut(from.into_raw_mut() as *mut <Self::To as Deref>::Target)
    }
}

unsafe impl<T> CastArrHelper for ([T; 27], [T; 27]) {
    type From = [T; 27];
    type To = [T; 27];

    unsafe fn cast(from: Self::From) -> Self::To {
        transmute_workaround_size_bug_super_dangerous_ignores_size(from)
    }
}

unsafe impl<T> CastArrHelper for ([MaybeUninit<T>; 27], [T; 27]) {
    type From = [MaybeUninit<T>; 27];
    type To = [T; 27];

    unsafe fn cast(from: Self::From) -> Self::To {
        transmute_workaround_size_bug_super_dangerous_ignores_size(from)
    }
}

unsafe impl<P, T> CastArrHelper for (P, [T; 27]) where P: PointerMut + Deref<Target=[MaybeUninit<T>; 27]> + PtrTc<[T; 27]>, <P as PtrTc<[T; 27]>>::Pointer: PointerMut + Deref<Target=[T; 27]> {
    type From = P;
    type To = <P as PtrTc<[T; 27]>>::Pointer;

    unsafe fn cast(from: Self::From) -> Self::To {
        Self::To::from_raw_mut(from.into_raw_mut() as *mut <Self::To as Deref>::Target)
    }
}

unsafe impl<T> CastArrHelper for ([T; 28], [T; 28]) {
    type From = [T; 28];
    type To = [T; 28];

    unsafe fn cast(from: Self::From) -> Self::To {
        transmute_workaround_size_bug_super_dangerous_ignores_size(from)
    }
}

unsafe impl<T> CastArrHelper for ([MaybeUninit<T>; 28], [T; 28]) {
    type From = [MaybeUninit<T>; 28];
    type To = [T; 28];

    unsafe fn cast(from: Self::From) -> Self::To {
        transmute_workaround_size_bug_super_dangerous_ignores_size(from)
    }
}

unsafe impl<P, T> CastArrHelper for (P, [T; 28]) where P: PointerMut + Deref<Target=[MaybeUninit<T>; 28]> + PtrTc<[T; 28]>, <P as PtrTc<[T; 28]>>::Pointer: PointerMut + Deref<Target=[T; 28]> {
    type From = P;
    type To = <P as PtrTc<[T; 28]>>::Pointer;

    unsafe fn cast(from: Self::From) -> Self::To {
        Self::To::from_raw_mut(from.into_raw_mut() as *mut <Self::To as Deref>::Target)
    }
}

unsafe impl<T> CastArrHelper for ([T; 29], [T; 29]) {
    type From = [T; 29];
    type To = [T; 29];

    unsafe fn cast(from: Self::From) -> Self::To {
        transmute_workaround_size_bug_super_dangerous_ignores_size(from)
    }
}

unsafe impl<T> CastArrHelper for ([MaybeUninit<T>; 29], [T; 29]) {
    type From = [MaybeUninit<T>; 29];
    type To = [T; 29];

    unsafe fn cast(from: Self::From) -> Self::To {
        transmute_workaround_size_bug_super_dangerous_ignores_size(from)
    }
}

unsafe impl<P, T> CastArrHelper for (P, [T; 29]) where P: PointerMut + Deref<Target=[MaybeUninit<T>; 29]> + PtrTc<[T; 29]>, <P as PtrTc<[T; 29]>>::Pointer: PointerMut + Deref<Target=[T; 29]> {
    type From = P;
    type To = <P as PtrTc<[T; 29]>>::Pointer;

    unsafe fn cast(from: Self::From) -> Self::To {
        Self::To::from_raw_mut(from.into_raw_mut() as *mut <Self::To as Deref>::Target)
    }
}

unsafe impl<T> CastArrHelper for ([T; 30], [T; 30]) {
    type From = [T; 30];
    type To = [T; 30];

    unsafe fn cast(from: Self::From) -> Self::To {
        transmute_workaround_size_bug_super_dangerous_ignores_size(from)
    }
}

unsafe impl<T> CastArrHelper for ([MaybeUninit<T>; 30], [T; 30]) {
    type From = [MaybeUninit<T>; 30];
    type To = [T; 30];

    unsafe fn cast(from: Self::From) -> Self::To {
        transmute_workaround_size_bug_super_dangerous_ignores_size(from)
    }
}

unsafe impl<P, T> CastArrHelper for (P, [T; 30]) where P: PointerMut + Deref<Target=[MaybeUninit<T>; 30]> + PtrTc<[T; 30]>, <P as PtrTc<[T; 30]>>::Pointer: PointerMut + Deref<Target=[T; 30]> {
    type From = P;
    type To = <P as PtrTc<[T; 30]>>::Pointer;

    unsafe fn cast(from: Self::From) -> Self::To {
        Self::To::from_raw_mut(from.into_raw_mut() as *mut <Self::To as Deref>::Target)
    }
}

unsafe impl<T> CastArrHelper for ([T; 31], [T; 31]) {
    type From = [T; 31];
    type To = [T; 31];

    unsafe fn cast(from: Self::From) -> Self::To {
        transmute_workaround_size_bug_super_dangerous_ignores_size(from)
    }
}

unsafe impl<T> CastArrHelper for ([MaybeUninit<T>; 31], [T; 31]) {
    type From = [MaybeUninit<T>; 31];
    type To = [T; 31];

    unsafe fn cast(from: Self::From) -> Self::To {
        transmute_workaround_size_bug_super_dangerous_ignores_size(from)
    }
}

unsafe impl<P, T> CastArrHelper for (P, [T; 31]) where P: PointerMut + Deref<Target=[MaybeUninit<T>; 31]> + PtrTc<[T; 31]>, <P as PtrTc<[T; 31]>>::Pointer: PointerMut + Deref<Target=[T; 31]> {
    type From = P;
    type To = <P as PtrTc<[T; 31]>>::Pointer;

    unsafe fn cast(from: Self::From) -> Self::To {
        Self::To::from_raw_mut(from.into_raw_mut() as *mut <Self::To as Deref>::Target)
    }
}

unsafe impl<T> CastArrHelper for ([T; 32], [T; 32]) {
    type From = [T; 32];
    type To = [T; 32];

    unsafe fn cast(from: Self::From) -> Self::To {
        transmute_workaround_size_bug_super_dangerous_ignores_size(from)
    }
}

unsafe impl<T> CastArrHelper for ([MaybeUninit<T>; 32], [T; 32]) {
    type From = [MaybeUninit<T>; 32];
    type To = [T; 32];

    unsafe fn cast(from: Self::From) -> Self::To {
        transmute_workaround_size_bug_super_dangerous_ignores_size(from)
    }
}

unsafe impl<P, T> CastArrHelper for (P, [T; 32]) where P: PointerMut + Deref<Target=[MaybeUninit<T>; 32]> + PtrTc<[T; 32]>, <P as PtrTc<[T; 32]>>::Pointer: PointerMut + Deref<Target=[T; 32]> {
    type From = P;
    type To = <P as PtrTc<[T; 32]>>::Pointer;

    unsafe fn cast(from: Self::From) -> Self::To {
        Self::To::from_raw_mut(from.into_raw_mut() as *mut <Self::To as Deref>::Target)
    }
}

/*
unsafe impl<P, T> CastArrHelper for (P, [T]) where P: SlicePointerMut<Item=T> + Deref<Target=[T]> + PtrTc<[T]>, <P as PtrTc<[T]>>::Pointer: SlicePointerMut<Item=T> + Deref<Target=[T]> {
    type From = P;
    type To = <P as PtrTc<[T]>>::Pointer;

    unsafe fn cast(from: Self::From) -> Self::To {
        let (ptr, len) = from.into_raw_parts_mut();
        Self::To::from_raw_parts_mut(ptr as *mut <Self::To as SlicePointerMut>::Item, len)
    }
}
*/

/// Type constructor for smart pointer types
pub trait PtrTc<T: ?Sized> {
    /// The pointer type storing T
    type Pointer: Sized + Deref<Target=T>;
}

impl<'a, T: ?Sized, U: 'a + ?Sized> PtrTc<U> for &'a mut T {
    type Pointer = &'a mut U;
}

#[cfg(feature = "alloc")]
impl<T: ?Sized, U: ?Sized> PtrTc<U> for alloc::boxed::Box<T> {
    type Pointer = alloc::boxed::Box<U>;
}

/// Trait for unique smart pointers/references containing `Sized` types
/// allowing casting between them.
pub unsafe trait PointerMut: Sized + StableDeref + DerefMut where <Self as Deref>::Target: Sized {
    /// Converts the smart pointer into raw pointer.
    ///
    /// The original smart pointer must be forgotten.
    fn into_raw_mut(self) -> *mut Self::Target;

    /// Converts a raw pointer returned from `into_raw_mut` into smart pointer.
    ///
    /// # Safety
    ///
    /// These operations must be sound:
    ///
    /// * `P<T>::from_raw_mut(P<T>::into_raw_mut(x))
    /// * `P<T>::from_raw_mut(P<MaybeUninit<T>>::into_raw_mut(x) as *mut T)
    ///
    /// Where `P` is the pointer type corresponding to `Self` and `x` is a
    /// valid value for given smart pointer.
    unsafe fn from_raw_mut(ptr: *mut Self::Target) -> Self;
}

unsafe impl<T> PointerMut for &mut T {
    fn into_raw_mut(self) -> *mut Self::Target {
        self
    }

    unsafe fn from_raw_mut(ptr: *mut Self::Target) -> Self {
        &mut *ptr
    }
}

/// Trait for unique smart pointers/references containing slices, allowing
/// casting between them.
pub unsafe trait SlicePointerMut: Sized + StableDeref + Deref<Target=[<Self as SlicePointerMut>::Item]> + DerefMut {
    /// Type of contained item.
    type Item: Sized;

    /// Returns pointer and length of the slice, forgetting the original value.
    fn into_raw_parts_mut(self) -> (*mut Self::Item, usize);

    /// Converts a raw pointer and length returned from `into_raw_mut` into
    /// smart pointer.
    ///
    /// # Safety
    ///
    /// These operations must be sound (pseudo code ignoring len):
    ///
    /// * `P<T>::from_raw_parts_mut(P<T>::into_raw_parts_mut(x))
    /// * `P<T>::from_raw_parts_mut(P<MaybeUninit<T>>::into_raw_parts_mut(x) as *mut T)
    ///
    /// Where `P` is the pointer type corresponding to `Self` and `x` is a
    /// valid value for given smart pointer.
    unsafe fn from_raw_parts_mut(ptr: *mut Self::Item, len: usize) -> Self;
}

unsafe impl<T> SlicePointerMut for &mut [T] {
    type Item=T;

    fn into_raw_parts_mut(self) -> (*mut Self::Item, usize) {
        (self.as_mut_ptr(), self.len())
    }

    unsafe fn from_raw_parts_mut(ptr: *mut Self::Item, len: usize) -> Self {
        core::slice::from_raw_parts_mut(ptr, len)
    }
}

#[cfg(feature = "alloc")]
mod alloc_impls {
    use alloc::boxed::Box;
    use super::*;

    unsafe impl<T> PointerMut for Box<T> {
        fn into_raw_mut(self) -> *mut Self::Target {
            Box::into_raw(self)
        }

        unsafe fn from_raw_mut(ptr: *mut Self::Target) -> Self {
            Box::from_raw(ptr)
        }
    }

    unsafe impl<T> SlicePointerMut for Box<[T]> {
        type Item=T;

        fn into_raw_parts_mut(mut self) -> (*mut Self::Item, usize) {
            let ptr = self.as_mut_ptr();
            let len = self.len();

            core::mem::forget(self);

            (ptr, len)
        }

        unsafe fn from_raw_parts_mut(ptr: *mut Self::Item, len: usize) -> Self {
            Box::from_raw(core::slice::from_raw_parts_mut(ptr, len))
        }
    }
}
