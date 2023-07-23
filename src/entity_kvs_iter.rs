//! Module containing the implementation for an iterator over the key-values of an entity within a
//! [`QEntities`] collection.

use super::{QEntities, QEntityInfo, QEntityKeyValueInfo, QEntityKeyValueRef};
use core::slice;

/// Iterator over the key-values of an entity within a [`QEntities`] collection.
pub struct QEntityKeyValuesIter<'a> {
    /// The collection of Quake entities that contains the entity whose key-values are iterated.
    entities: &'a QEntities,
    /// The inner iterator for entity key-value infos describing the entity key-values.
    inner_iter: slice::Iter<'a, QEntityKeyValueInfo>,
}

impl<'a> QEntityKeyValuesIter<'a> {
    /// Creates a new iterator over the key-values of an entity.
    ///
    /// # Panics
    /// This function will panic if the provided [`QEntityInfo`] describes an entity that is not
    /// valid for the provided [`QEntities`] collection.
    #[inline]
    pub(super) fn new(entities: &'a QEntities, entity_info: &'a QEntityInfo) -> Self {
        let first_kv = entity_info.first_kv;
        let last_kv = first_kv + entity_info.kvs_length;
        let kvs_slice = &entities.key_values[first_kv..last_kv];
        QEntityKeyValuesIter {
            entities,
            inner_iter: kvs_slice.iter(),
        }
    }
}

impl<'a> Iterator for QEntityKeyValuesIter<'a> {
    type Item = QEntityKeyValueRef<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner_iter.next().map(self.entities.kv_ref_inator())
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner_iter.size_hint()
    }

    #[inline(always)]
    fn count(self) -> usize {
        self.inner_iter.count()
    }

    #[inline]
    fn last(self) -> Option<Self::Item> {
        self.inner_iter.last().map(self.entities.kv_ref_inator())
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.inner_iter.nth(n).map(self.entities.kv_ref_inator())
    }
}

impl<'a> DoubleEndedIterator for QEntityKeyValuesIter<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner_iter
            .next_back()
            .map(self.entities.kv_ref_inator())
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.inner_iter
            .nth_back(n)
            .map(self.entities.kv_ref_inator())
    }
}

impl ExactSizeIterator for QEntityKeyValuesIter<'_> {
    #[inline(always)]
    fn len(&self) -> usize {
        self.inner_iter.len()
    }
}
