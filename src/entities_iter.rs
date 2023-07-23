//! Module containing the implementation for an iterator over the entities within a [`QEntities`]
//! collection.

use super::{QEntities, QEntityInfo, QEntityRef};
use core::slice;

/// Iterator over the entities within a [`QEntities`] collection.
pub struct QEntitiesIter<'a> {
    /// The collection of Quake entities being iterated.
    entities: &'a QEntities,
    /// The inner iterator for entity infos describing the entities.
    inner_iter: slice::Iter<'a, QEntityInfo>,
}

impl<'a> QEntitiesIter<'a> {
    /// Creates a new iterator over the entities of a [`QEntities`] collection.
    #[inline]
    pub(super) fn new(entities: &'a QEntities) -> Self {
        Self {
            entities,
            inner_iter: entities.entities.iter(),
        }
    }
}

impl<'a> Iterator for QEntitiesIter<'a> {
    type Item = QEntityRef<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner_iter
            .next()
            .map(self.entities.entity_ref_inator())
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
        self.inner_iter
            .last()
            .map(self.entities.entity_ref_inator())
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.inner_iter
            .nth(n)
            .map(self.entities.entity_ref_inator())
    }
}

impl<'a> DoubleEndedIterator for QEntitiesIter<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner_iter
            .next_back()
            .map(self.entities.entity_ref_inator())
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.inner_iter
            .nth_back(n)
            .map(self.entities.entity_ref_inator())
    }
}

impl ExactSizeIterator for QEntitiesIter<'_> {
    #[inline(always)]
    fn len(&self) -> usize {
        self.inner_iter.len()
    }
}
