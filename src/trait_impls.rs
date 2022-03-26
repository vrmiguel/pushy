use core::{
    fmt::Debug,
    hash::Hash,
    ops::{Deref, DerefMut},
};

use crate::PushArray;

impl<T: Clone, const CAP: usize> Clone for PushArray<T, CAP> {
    fn clone(&self) -> Self {
        self.iter().cloned().collect()
    }
}

impl<T: Hash, const CAP: usize> Hash for PushArray<T, CAP> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.initialized().hash(state);
        self.len.hash(state);
    }
}

impl<T, const CAP: usize> AsRef<[T]> for PushArray<T, CAP> {
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T: PartialEq, const CAP: usize, U> PartialEq<U> for PushArray<T, CAP>
where
    U: AsRef<[T]>,
{
    fn eq(&self, other: &U) -> bool {
        self.as_ref() == other.as_ref()
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

impl<T, const CAP: usize> Deref for PushArray<T, CAP> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        self.initialized()
    }
}

impl<T, const CAP: usize> DerefMut for PushArray<T, CAP> {
    fn deref_mut(&mut self) -> &mut [T] {
        self.initialized_mut()
    }
}

impl<T, const CAP: usize> FromIterator<T> for PushArray<T, CAP> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut arr = Self::new();

        for item in iter {
            arr.push(item);
        }

        arr
    }
}
