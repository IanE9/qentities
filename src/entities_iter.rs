//! Module containing the implementation for an iterator over the entities within a
//! [collection of Quake entities](QEntities).

use super::{QEntities, QEntityInfo, QEntityRef};
use core::slice;

/// Iterator over the entities within a [collection of Quake entities](QEntities).
pub struct QEntitiesIter<'a> {
    /// The collection of Quake entities being iterated.
    pub(super) entities: &'a QEntities,
    /// The inner iterator for entity infos describing the entities.
    pub(super) inner_iter: slice::Iter<'a, QEntityInfo>,
}

impl<'a> Iterator for QEntitiesIter<'a> {
    type Item = QEntityRef<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner_iter
            .next()
            .map(|entity_info| self.entities.entity_ref(entity_info))
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
            .map(|entity_info| self.entities.entity_ref(entity_info))
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.inner_iter
            .nth(n)
            .map(|entity_info| self.entities.entity_ref(entity_info))
    }
}

impl<'a> DoubleEndedIterator for QEntitiesIter<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner_iter
            .next_back()
            .map(|entity_info| self.entities.entity_ref(entity_info))
    }
}

impl ExactSizeIterator for QEntitiesIter<'_> {
    #[inline(always)]
    fn len(&self) -> usize {
        self.inner_iter.len()
    }
}
