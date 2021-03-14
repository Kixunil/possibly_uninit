use super::{BorrowOutSlice, OutSlice, TakeItem};
use crate::cast::InitTc;

/// Wraps a (maybe uninitialized) slice as a `BorrowOutSlice`, tracking the
/// position. This allows you to work with readers and uninitialized memory
/// completely safely.
///
/// Note that this type doesn't implement traits like `Deref`, `Borrow`,
/// `AsRef`, `BorrowUninitSlice`. This isn't a gross negligence, it's done on
/// purpose. Basically, it acts as if it contained *two* or even *three*
/// buffers! Which buffer should be returned? One might kinda reasonably claim
/// it should be first part for [Item] slices, second part for [MaybeUninit<Item>].
/// But that would be extremely confusing, especially considering that [Item] can
/// be used where [MaybeUniniti<Item>] is expected.
///
/// This looks like a huge freaking footgun that I'm unwilling to create.
/// However I'd like to implement wrapper type in the future that will make it
/// explicit. E.g. something like `Cursed<Written, T>`, `Cursed<Uninit, T>`,
/// `Cursed<Entire, T>`. :) PRs welcome!
///
/// Important: This wrapper currently doesn't care about leaking! If you want
/// to avoid leaks, it's recommended to use it only with buffers containing
/// `MaybeUninit<T>` or with `Copy` types.
///
/// This may change in future versions.
pub struct Cursor<Item, Arr: BorrowOutSlice<Item> + ?Sized> {
    _phantom: core::marker::PhantomData<[Item]>,
    position: usize,
    data: Arr,
}

impl<Item, Arr: BorrowOutSlice<Item>> Cursor<Item, Arr> {
    /// Creates `Cursor` initialized with position 0
    pub fn new(buf: Arr) -> Self {
        Cursor {
            _phantom: Default::default(),
            data: buf,
            position: 0,
        }
    }
}

impl<Item, Arr: BorrowOutSlice<Item> + ?Sized> Cursor<Item, Arr> {
    /// Pushes an item at the end of the array
    pub fn push(&mut self, item: Item) -> Result<&mut Item, Item> {
        if self.position < self.data.borrow_uninit_slice().len() {
            let res = self.data.borrow_out_slice().at_mut(self.position).write(item);
            self.position += 1;
            Ok(res)
        } else {
            Err(item)
        }
    }

    /// Pushes up to `.remaining_count()` items from the iterator and returns a
    /// subslice corresponding to *just written* items.
    ///
    /// If you want to access everything that has been written so far, use the
    /// `written()` method after call to this one.
    pub fn push_iter<I>(&mut self, iter: I) -> &mut [Item] where I: IntoIterator, I::Item: TakeItem<Item> {
        let res = self.data.borrow_out_slice()[self.position..].init_from_iter(iter);
        self.position += res.len();
        res
    }

    /// Returns the number of items that can be written to this cursor.
    pub fn remaining_count(&self) -> usize {
        self.data.borrow_uninit_slice().len() - self.position
    }

    /// Resets the position to 0.
    pub fn reset(&mut self) {
        self.position = 0;
    }

    /// Access the written slice.
    pub fn written(&self) -> &[Item] {
        unsafe {
            let slice = &self.data.borrow_uninit_slice()[..self.position];
            core::slice::from_raw_parts(slice.as_ptr() as *const Item, slice.len())
        }
    }

    /// Mutably access the written slice.
    pub fn written_mut(&mut self) -> &mut [Item] {
        unsafe {
            let slice = &mut self.data.borrow_out_slice()[..self.position];
            core::slice::from_raw_parts_mut(slice.as_mut_ptr() as *mut Item, slice.len())
        }
    }

    /// Attempts to "remove" the last item and return it.
    ///
    /// This can be implemented for `Copy` types safely, as swapping the place with
    /// uninitialized memory would cause corruption if the slice was actually
    /// not MaybeUninit.
    pub fn pop(&mut self) -> Option<Item> where Item: Copy {
        use crate::borrow::BorrowUninit;

        unsafe {
            if self.position > 0 && self.position - 1 < self.data.borrow_uninit_slice().len() {
                let res = *self.data.borrow_out_slice().at_mut(self.position - 1).assume_init_ref();
                self.position -= 1;
                Some(res)
            } else {
                None
            }
        }
    }

    /// "Removes" up to `max` slice from the buffer and returns them as slice.
    ///
    /// Empty slice is returned if the position is zero, of course.
    pub fn pop_slice(&mut self, max: usize) -> &mut [Item] {
        unsafe {
            let to_remove = self.position.min(max);
            let res = &mut self.data.borrow_out_slice()[(self.position - to_remove)..self.position];
            self.position -= to_remove;
            core::slice::from_raw_parts_mut(res.as_mut_ptr() as *mut Item, to_remove)
        }
    }

    /// Attempts to "remove" `required` number of slice from the buffer and
    /// return them as slice. Keeps the cursor unchanged in case of failure.
    ///
    /// Empty slice is returned if the position is zero, of course.
    pub fn try_pop_slice(&mut self, required: usize) -> Option<&mut [Item]> {
        if self.position >= required {
            Some(self.pop_slice(required))
        } else {
            None
        }
    }

    /// Splits the internal buffer at current position and returns both
    /// initialized and uninitialized part.
    pub fn split_mut(&mut self) -> (&mut [Item], &mut OutSlice<Item>) {
        unsafe {
            let (first, second) = self.data.borrow_out_slice().split_at_mut(self.position);
            (first.assume_init_mut(), second)
        }
    }

    /// Tries to cast the underlying data into initialized version of the
    /// container, if the entire slice was filled.
    pub fn try_cast_initialized(self) -> Result<<Arr as InitTc<Item>>::Output, Self> where Arr: Sized + InitTc<Item> {
        unsafe {
            if self.position == self.data.borrow_uninit_slice().len() {
                Ok(self.data.cast())
            } else {
                Err(self)
            }
        }
    }
}

impl<Item, Arr: BorrowOutSlice<Item>> From<Arr> for Cursor<Item, Arr> {
    fn from(value: Arr) -> Self {
        Self::new(value)
    }
}

#[cfg(feature = "alloc")]
mod alloc_impls {
    use super::Cursor;
    use core::mem::MaybeUninit;
    use super::super::BorrowOutSlice;

    use alloc::boxed::Box;
    use alloc::vec::Vec;

    impl<Item> Cursor<Item, Box<[MaybeUninit<Item>]>> {
        /// Constructs the `Cursor` from `Vec` while preserving lenght of the `Vec`
        /// as position of the cursor.
        pub fn from_vec_preserving_len(vec: Vec<Item>) -> Self {
            let len = vec.len();
            let mut cursor = Cursor::from_vec_entire_capaity(vec);
            cursor.position = len;
            cursor
        }

        /// Constructs the `Cursor` from `Vec` using whole capacity of the vec.
        ///
        /// This method currently leaks all present items
        pub fn from_vec_entire_capaity<T>(mut vec: Vec<T>) -> Self where Box<[MaybeUninit<T>]>: BorrowOutSlice<Item> {
            unsafe {
                let ptr = vec.as_mut_ptr();
                let capacity = vec.capacity();

                core::mem::forget(vec);

                // We set len to capacity to avoid reallocations.
                // This is safe because the item type is MaybeUninit.
                let boxed = Vec::from_raw_parts(ptr as *mut MaybeUninit<Item>, capacity, capacity)
                    .into_boxed_slice();
                Cursor {
                    _phantom: Default::default(),
                    data: boxed,
                    position: 0,
                }
            }
        }
    }

    impl<Item> Cursor<Item, Box<[Item]>> where Box<[Item]>: BorrowOutSlice<Item> {
        /// Constructs the `Cursor` from `Vec` by reallocating it in order to
        /// throw away the excess capacity.
        pub fn from_vec_resizing(vec: Vec<Item>) -> Self {
            Cursor::new(vec.into_boxed_slice())
        }
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use core::mem::MaybeUninit;

    #[test]
    fn slice() {
        let mut uninit = [MaybeUninit::uninit(); 4];
        let mut cursor = Cursor::new(&mut uninit as &mut [MaybeUninit<u8>]);
        cursor.push(0).expect("Array full");
        cursor.push(1).expect("Array full");
        cursor.push(2).expect("Array full");
        cursor.push(3).expect("Array full");

        assert_eq!(cursor.written(), &[0, 1, 2, 3]);
        assert_eq!(cursor.pop().expect("Empty array"), 3);
        assert_eq!(cursor.try_pop_slice(2).expect("Array too short"), &[1, 2]);

        cursor.push(24).expect("Array full");
        cursor.push(42).expect("Array full");
        cursor.push(47).expect("Array full");

        let arr = cursor.try_cast_initialized().unwrap_or_else(|_| panic!("Cursor not filled"));
        assert_eq!(arr, &[0, 24, 42, 47]);
    }

    #[test]
    fn arr() {
        let mut uninit = [MaybeUninit::uninit(); 4];
        let mut cursor = Cursor::new(&mut uninit);
        cursor.push(0).expect("Array full");
        cursor.push(1).expect("Array full");
        cursor.push(2).expect("Array full");
        cursor.push(3).expect("Array full");

        assert_eq!(cursor.written(), &[0, 1, 2, 3]);
        assert_eq!(cursor.pop().expect("Empty array"), 3);
        assert_eq!(cursor.try_pop_slice(2).expect("Array too short"), &[1, 2]);

        cursor.push(24).expect("Array full");
        cursor.push(42).expect("Array full");
        cursor.push(47).expect("Array full");

        let arr = cursor.try_cast_initialized().unwrap_or_else(|_| panic!("Cursor not filled"));
        assert_eq!(arr, &[0, 24, 42, 47]);
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn boxed_arr() {
        use alloc::boxed::Box;

        let uninit = Box::new([MaybeUninit::uninit(); 4]);
        let mut cursor = Cursor::new(uninit);
        cursor.push(0).expect("Array full");
        cursor.push(1).expect("Array full");
        cursor.push(2).expect("Array full");
        cursor.push(3).expect("Array full");

        assert_eq!(cursor.written(), &[0, 1, 2, 3]);
        assert_eq!(cursor.pop().expect("Empty array"), 3);
        assert_eq!(cursor.try_pop_slice(2).expect("Array too short"), &[1, 2]);

        cursor.push(24).expect("Array full");
        cursor.push(42).expect("Array full");
        cursor.push(47).expect("Array full");

        let arr = cursor.try_cast_initialized().unwrap_or_else(|_| panic!("Cursor not filled"));
        assert_eq!(&*arr, &[0, 24, 42, 47]);
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn boxed_slice() {
        use alloc::vec::Vec;

        let uninit = Vec::with_capacity(4);
        let mut cursor = Cursor::from_vec_preserving_len(uninit);
        cursor.push(0).expect("Array full");
        cursor.push(1).expect("Array full");
        cursor.push(2).expect("Array full");
        cursor.push(3).expect("Array full");

        assert_eq!(cursor.written(), &[0, 1, 2, 3]);
        assert_eq!(cursor.pop().expect("Empty array"), 3);
        assert_eq!(cursor.try_pop_slice(2).expect("Array too short"), &[1, 2]);

        cursor.push(24).expect("Array full");
        cursor.push(42).expect("Array full");
        cursor.push(47).expect("Array full");

        let arr = cursor.try_cast_initialized().unwrap_or_else(|_| panic!("Cursor not filled"));
        assert_eq!(&*arr, &[0, 24, 42, 47]);
    }
}
