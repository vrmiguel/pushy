#![no_std]
#![feature(maybe_uninit_slice)]

mod trait_impls;

use core::{mem::MaybeUninit, ptr::addr_of_mut};

#[derive(Debug)]
pub enum Error {
    NotEnoughCapacity,
}

pub type Result<T> = core::result::Result<T, Error>;

/// A Vec-like (but non-growing) stack-allocated array.
// #[derive(Hash)]
pub struct PushArray<T, const CAP: usize> {
    buf: [MaybeUninit<T>; CAP],
    len: usize,
}

impl<T, const CAP: usize> PushArray<T, CAP> {
    #[inline]
    const fn array_of_uninit() -> [MaybeUninit<T>; CAP] {
        // Safety: safe since this is an array of `MaybeUninit`s and they don't require initialization
        unsafe { MaybeUninit::uninit().assume_init() }
    }

    /// Create an empty [`PushArray`] with the given capacity.
    /// ```
    /// # use pushy::PushArray;
    /// let mut arr: PushArray<u8, 5> = PushArray::new();
    ///
    /// assert!(arr.is_empty());
    /// assert_eq!(arr.len(), 0);
    /// assert_eq!(arr.initialized(), &[]);
    /// ```
    pub const fn new() -> Self {
        let buf = Self::array_of_uninit();

        Self { buf, len: 0 }
    }

    /// Returns the amount of initialized elements in this [`PushArray`].
    /// ```
    /// # use pushy::PushArray;
    /// let mut arr: PushArray<u32, 5> = PushArray::new();
    /// assert_eq!(arr.len(), 0);
    ///
    /// arr.push(0);
    ///
    /// assert_eq!(arr.len(), 1);
    /// ```
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns true if this [`PushArray`] is empty.
    ///
    /// ```
    /// # use pushy::PushArray;
    /// let mut arr: PushArray<u32, 5> = PushArray::new();
    /// assert!(arr.is_empty());
    ///
    /// arr.push(0);
    ///
    /// assert_eq!(arr.is_empty(), false);
    /// ```
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns a reference to an initialized element of the array.
    ///
    /// Returns `None` if the given index is out-of-bounds or not initialized.
    ///
    /// ```
    /// # use pushy::PushArray;
    /// let mut arr: PushArray<u8, 3> = PushArray::new();
    ///
    /// arr.push_str("Hey").unwrap();
    ///
    /// assert_eq!(arr.get(0), Some(&b'H'));
    /// assert_eq!(arr.get(1), Some(&b'e'));
    /// assert_eq!(arr.get(2), Some(&b'y'));
    /// assert_eq!(arr.get(3), None);
    /// ```
    pub fn get(&self, index: usize) -> Option<&T> {
        // Safety: only called after we've made sure that the
        // element in the given index is in-bounds and initialized
        let get_elem = || unsafe { self.get_unchecked(index) };

        (self.len > index).then(get_elem)
    }

    /// Returns a mutable reference to an initialized element of the array.
    ///
    /// Returns `None` if the given index is out-of-bounds or not initialized.
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        // Safety: only called after we've made sure that the
        // element in the given index is in-bounds and initialized
        (self.len > index).then(|| unsafe { self.get_unchecked_mut(index) })
    }

    /// Returns a reference to an element without doing bounds
    /// checking.
    ///
    /// For a safe alternative see [`get`].
    ///
    /// # Safety
    ///
    /// Calling this method with an out-of-bounds index is *[undefined behavior]*
    /// even if the resulting reference is not used.
    ///
    /// This method does not guarantee that the element returned is properly initialized.
    ///
    /// [`get`]: PushArray::get
    /// [undefined behavior]: https://doc.rust-lang.org/reference/behavior-considered-undefined.html
    pub unsafe fn get_unchecked(&self, index: usize) -> &T {
        self.buf.get_unchecked(index).assume_init_ref()
    }

    /// Returns a mutable reference to an element without doing bounds
    /// checking.
    ///
    /// For a safe alternative see [`get_mut`].
    ///
    /// # Safety
    ///
    /// Calling this method with an out-of-bounds index is *[undefined behavior]*
    /// even if the resulting reference is not used.
    ///
    /// This method does not guarantee that the element returned is properly initialized.
    ///
    /// [`get_mut`]: PushArray::get_mut
    /// [undefined behavior]: https://doc.rust-lang.org/reference/behavior-considered-undefined.html
    pub unsafe fn get_unchecked_mut(&mut self, index: usize) -> &mut T {
        self.buf.get_unchecked_mut(index).assume_init_mut()
    }

    /// Pushes an element to the back of the [`PushArray`] without
    /// checking the boundaries of the array first.
    ///
    /// # Safety
    ///
    /// The programmer must ensure this function does not push data after the end of the buffer, which would cause undefined behavior.
    pub unsafe fn push_unchecked(&mut self, value: T) {
        let ptr = self.buf.as_mut_ptr();
        addr_of_mut!(*ptr)
            .add(self.len)
            .write(MaybeUninit::new(value));

        self.len += 1;
    }

    /// Push an element to the end of this array after making sure
    /// that the array has enough space to hold it.
    ///
    /// ```
    /// # use pushy::PushArray;
    /// let mut arr: PushArray<u32, 2> = PushArray::new();
    ///
    /// assert!(arr.push_checked(5).is_ok());
    /// assert!(arr.push_checked(20).is_ok());
    ///
    /// // Not enough capacity!
    /// assert!(arr.push_checked(9).is_err());
    /// ```
    pub fn push_checked(&mut self, value: T) -> Result<()> {
        (self.len < CAP)
            .then(|| unsafe { self.push_unchecked(value) })
            .ok_or(Error::NotEnoughCapacity)
    }

    /// Push an element to the back of this [`PushArray`].
    ///
    /// # Panics
    ///
    /// Panics if the capacity of this array is overrun.
    ///
    /// ```
    /// # use pushy::PushArray;
    /// let mut bytes: PushArray<u8, 2> = PushArray::new();
    /// bytes.push(b'H');
    /// bytes.push(b'i');
    ///
    /// assert_eq!(bytes.as_str().unwrap(), "Hi");
    /// ```
    pub fn push(&mut self, value: T) {
        self.push_checked(value).expect("overflow in PushArray!")
    }

    /// Push all elements of the given array at the end of the [`PushArray`].
    pub fn push_array<const M: usize>(&mut self, array: [T; M]) -> Result<()> {
        if self.len + M > CAP {
            return Err(Error::NotEnoughCapacity);
        }

        unsafe {
            // Safety: we've just checked that there is enough capacity to
            // push these elements into our array.
            (self.as_mut_ptr().add(self.len) as *mut [T; M]).write(array);
        }

        self.len += M;

        Ok(())
    }

    /// Gets a pointer to the first element of the array.
    ///
    /// # Safety
    ///
    /// * There is no guarantee that the first element pointed to is initialized.
    ///
    /// * There is no guarantee that the first element exists (if the capacity allocated was zero).
    pub unsafe fn as_ptr(&self) -> *const T {
        MaybeUninit::slice_as_ptr(&self.buf)
    }

    /// Gets a mutable pointer to the first element of the array.
    ///
    /// # Safety
    ///
    /// * There is no guarantee that the first element pointed to is initialized.
    ///
    /// * There is no guarantee that the first element exists (if the capacity allocated was zero).
    pub unsafe fn as_mut_ptr(&mut self) -> *mut T {
        MaybeUninit::slice_as_mut_ptr(&mut self.buf)
    }

    /// Returns the initialized elements of this [`PushArray`].
    ///
    /// Alias to [`PushArray::initialized`].
    pub fn as_slice(&self) -> &[T] {
        self.initialized()
    }

    /// Returns the initialized elements of this [`PushArray`].
    pub fn initialized(&self) -> &[T] {
        // Safety:
        //
        // * The elements given by `self.as_ptr()` are properly aligned since they come from
        //   an array (and the memory layout of MaybeUninit<T> is the same as the memory layout of T)
        //
        // * The slice will be created only with initialized values since we know that `self.len` is
        //   the amount of properly initialized elements in our array.
        unsafe { core::slice::from_raw_parts(self.as_ptr(), self.len) }
    }

    /// Returns the initialized elements of this [`PushArray`].
    pub fn initialized_mut(&mut self) -> &mut [T] {
        // Safety:
        //
        // * The elements given by `self.as_mut_ptr()` are properly aligned since they come from
        //   an array (and the memory layout of MaybeUninit<T> is the same as the memory layout of T)
        //
        // * The slice will be created only with initialized values since we know that `self.len` is
        //   the amount of properly initialized elements in our array.
        unsafe { core::slice::from_raw_parts_mut(self.as_mut_ptr(), self.len) }
    }

    /// "Clears" the [`PushArray`]. The stored memory is not cleared or immediately
    /// dropped, but will be overwritten whenever new information is
    /// pushed into the array.
    ///
    /// ```
    /// # use pushy::PushArray;
    /// let mut bytes: PushArray<u8, 5> = PushArray::new();
    /// bytes.push_str("Hello").unwrap();
    ///
    /// assert_eq!(
    ///     bytes.as_str().unwrap(),
    ///     "Hello"
    /// );
    ///
    /// // Logically clear this array
    /// bytes.clear();
    ///
    /// assert_eq!(
    ///     bytes.as_str().unwrap(),
    ///     ""
    /// );
    /// ```
    pub fn clear(&mut self) {
        unsafe {
            core::ptr::drop_in_place(self.initialized_mut());
        }
        self.len = 0;
    }
}

impl<T: Copy, const CAP: usize> PushArray<T, CAP> {
    /// Copy the elements from the given slice into the end of the [`PushArray`].
    ///
    // ```
    // # use pushy::PushArray;
    // let mut bytes: PushArray<u8, 5> = PushArray::new();
    // bytes.copy_from_slice(b"Hello").unwrap();
    //
    // assert_eq!(bytes.as_str(), Some("Hello"));
    // ```
    pub fn copy_from_slice(&mut self, slice: &[T]) -> Result<()> {
        if self.len + slice.len() > CAP {
            return Err(Error::NotEnoughCapacity);
        }

        // Safety: we've just checked that there is enough storage
        //         to hold the new elements.
        //
        //         We also know these elements are trivially copiable since they implement Copy.
        unsafe {
            core::ptr::copy_nonoverlapping(
                slice.as_ptr(),
                self.as_mut_ptr().add(self.len),
                slice.len(),
            );
        }

        self.len += slice.len();
        Ok(())
    }
}

impl<const CAP: usize> PushArray<u8, CAP> {
    /// Returns the bytes of this [`PushArray`] as a `&str` if they're valid UTF-8.
    /// ```
    /// # use pushy::PushArray;
    /// let mut bytes: PushArray<u8, 11> = PushArray::new();
    /// bytes.push_str("Hello").unwrap();
    /// assert_eq!(bytes.as_str(), Some("Hello"));
    ///
    /// bytes.push_str(" World").unwrap();
    /// assert_eq!(bytes.as_str(), Some("Hello World"));
    /// ```
    pub fn as_str(&self) -> Option<&str> {
        core::str::from_utf8(self.initialized()).ok()
    }

    /// Push a UTF-8 string to the back of this [`PushArray`].
    ///
    /// ```
    /// # use pushy::PushArray;
    /// let mut bytes: PushArray<u8, 11> = PushArray::new();
    ///
    /// assert_eq!(bytes.as_str(), Some(""));
    /// bytes.push_str("Hello").unwrap();
    /// assert_eq!(bytes.as_str(), Some("Hello"));
    /// ```
    pub fn push_str(&mut self, value: &str) -> Result<()> {
        let bytes = value.as_bytes();

        self.copy_from_slice(bytes)
    }
}
