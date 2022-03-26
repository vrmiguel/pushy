//! A pushable array type with fixed capacity.
#![no_std]

mod trait_impls;

use core::{mem::MaybeUninit, ptr::addr_of_mut};

#[derive(Debug)]
pub enum Error {
    NotEnoughCapacity,
}

pub type Result<T> = core::result::Result<T, Error>;

/// A pushable array with fixed capacity.
///
/// Stack-allocated drop-in replacement for `Vec`, panic on `.push()` if capacity is
/// exhausted, see `.push_checked()` if you want a checked alternative.
///
/// # Examples
///
/// ```
/// use pushy::PushArray;
///
/// let mut array: PushArray<i32, 5> = PushArray::new();
/// array.push(1);
/// array.push(2);
/// array.push(3);
///
/// assert_eq!(array.len(), 3);
/// assert_eq!(array[0], 1);
///
/// assert_eq!(array.pop(), Some(3));
/// assert_eq!(array.len(), 2);
///
/// array[0] = 7;
/// assert_eq!(array, [7, 2]);
/// ```
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
    /// assert_eq!(arr, []);
    /// ```
    pub const fn new() -> Self {
        let buf = Self::array_of_uninit();

        Self { buf, len: 0 }
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

    /// Removes the last element from the `PushArray`.
    pub fn pop(&mut self) -> Option<T> {
        self.len = self.len.checked_sub(1)?;

        let mut popped = MaybeUninit::uninit();
        unsafe {
            let ptr = self.as_ptr().add(self.len) as *const T;
            popped.write(ptr.read());
            // Safety: we've just written to `popped`, therefore we
            //         can assume it's uninitialized
            Some(popped.assume_init())
        }
    }

    /// Gets a pointer to the first element of the array.
    ///
    /// # Safety
    ///
    /// * There is no guarantee that the first element pointed to is initialized.
    ///
    /// * There is no guarantee that the first element exists (if the capacity allocated was zero).
    pub unsafe fn as_ptr(&self) -> *const T {
        &self.buf as *const [MaybeUninit<T>] as *const T
    }

    /// Gets a mutable pointer to the first element of the array.
    ///
    /// # Safety
    ///
    /// * There is no guarantee that the first element pointed to is initialized.
    ///
    /// * There is no guarantee that the first element exists (if the capacity allocated was zero).
    pub unsafe fn as_mut_ptr(&mut self) -> *mut T {
        &mut self.buf as *mut [MaybeUninit<T>] as *mut T
    }

    /// Extracts a slice containing the entire array.
    pub fn as_slice(&self) -> &[T] {
        self
    }

    /// Extracts a mutable slice containing the entire array.
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        self
    }

    /// Clear the [`PushArray`]. All initialized elements will be dropped.
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
            core::ptr::drop_in_place(self.as_mut_slice());
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
        core::str::from_utf8(self).ok()
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
