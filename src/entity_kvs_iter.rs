//! Module containing the implementation for an iterator over the key-values of an entity within a
//! [collection of Quake entities](QEntities).

use super::{QEntities, QEntityKeyValueInfo, QEntityKeyValueRef};
use core::slice;

/// Iterator over the key-values of an entity within a [collection of Quake entities](QEntities).
pub struct QEntityKeyValuesIter<'a> {
    /// The collection of Quake entities that contains the entity whose key-values are iterated.
    pub(super) entities: &'a QEntities,
    /// The inner iterator for entity key-value infos describing the entity key-values that are
    /// iterated.
    pub(super) inner_iter: slice::Iter<'a, QEntityKeyValueInfo>,
}

impl<'a> Iterator for QEntityKeyValuesIter<'a> {
    type Item = QEntityKeyValueRef<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner_iter
            .next()
            .map(|kv_info| self.entities.kv_ref(kv_info))
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner_iter.size_hint()
    }

    #[inline(always)]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.inner_iter.count()
    }

    #[inline]
    fn last(self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        self.inner_iter
            .last()
            .map(|kv_info| self.entities.kv_ref(kv_info))
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.inner_iter
            .nth(n)
            .map(|kv_info| self.entities.kv_ref(kv_info))
    }
}

impl<'a> DoubleEndedIterator for QEntityKeyValuesIter<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner_iter
            .next_back()
            .map(|kv_info| self.entities.kv_ref(kv_info))
    }
}

impl ExactSizeIterator for QEntityKeyValuesIter<'_> {
    #[inline(always)]
    fn len(&self) -> usize {
        self.inner_iter.len()
    }
}
