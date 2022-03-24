use core::{
    fmt::Debug,
    hash::Hash,
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
    ptr::addr_of_mut,
};

use crate::PushArray;

impl<T: Clone, const CAP: usize> Clone for PushArray<T, CAP> {
    fn clone(&self) -> Self {
        let mut cloned = Self::array_of_uninit();

        for ((idx, uninit), elem) in cloned.iter_mut().enumerate().zip(self.iter()) {
            unsafe {
                addr_of_mut!(*uninit)
                    .add(idx)
                    .write(MaybeUninit::new(elem.clone()));
            }
        }

        Self {
            buf: cloned,
            len: self.len,
        }
    }
}

impl<T: Hash, const CAP: usize> Hash for PushArray<T, CAP> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.initialized().hash(state);
        self.len.hash(state);
    }
}

impl<T: PartialEq, const CAP: usize> PartialEq for PushArray<T, CAP> {
    fn eq(&self, other: &Self) -> bool {
        self.len == other.len && self.as_slice() == other.as_slice()
    }
}

impl<T: Eq, const CAP: usize> Eq for PushArray<T, CAP> {}

impl<T: PartialOrd, const CAP: usize> PartialOrd for PushArray<T, CAP> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.initialized().partial_cmp(other.initialized())
    }
}

impl<T: Ord, const CAP: usize> Ord for PushArray<T, CAP> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.initialized().cmp(other.initialized())
    }
}

impl<T: Debug, const CAP: usize> Debug for PushArray<T, CAP> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PushArray")
            .field("initialized", &self.initialized())
            .finish()
    }
}

impl<T, const CAP: usize> Drop for PushArray<T, CAP> {
    fn drop(&mut self) {
        self.clear()
    }
}

impl<T, const N: usize> Deref for PushArray<T, N> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        self.initialized()
    }
}

impl<T, const N: usize> DerefMut for PushArray<T, N> {
    fn deref_mut(&mut self) -> &mut [T] {
        self.initialized_mut()
    }
}
