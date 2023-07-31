//! Implementation of the q-entities format.

#![warn(missing_docs)]

mod build;
mod byte_chunk;
pub mod entities_iter;
pub mod entity_kvs_iter;
pub mod parse;

use byte_chunk::ByteChunks;
use core::fmt;
use entities_iter::QEntitiesIter;
use entity_kvs_iter::QEntityKeyValuesIter;

/// Information describing an entity instance within a [`QEntities`] collection.
#[derive(Debug, Clone, Copy)]
struct QEntityInfo {
    /// Index of the entity's first key-value.
    first_kv: usize,
    /// The number of key-values the entity has.
    kvs_length: usize,
}

/// Information describing a key-value instance within a [`QEntities`] collection.
#[derive(Debug, Clone, Copy)]
struct QEntityKeyValueInfo {
    /// Index of the byte-chunk for the key.
    key_chunk: usize,
    /// Index of the byte-chunk for the value.
    value_chunk: usize,
}

/// Collection of q-entities.
pub struct QEntities {
    entities: Box<[QEntityInfo]>,
    key_values: Box<[QEntityKeyValueInfo]>,
    byte_chunks: ByteChunks,
}

impl fmt::Debug for QEntities {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl QEntities {
    /// Creates a new reference to an entity within the collection.
    ///
    /// # Panics
    /// The correct operation of this function is dependent upon the passed in entity info
    /// describing an entity that is valid for `self`. As such this function expects that the passed
    /// in entity info reference be a child of `self`.
    ///
    /// In debug builds this function explicitly panics when this condition is violated. In release
    /// builds this function on its own is incapable of panicking, but if the aforementioned
    /// condition has been violated, then it is possible for the returned reference to be used in
    /// such a manner that the program will subsequently panic.
    #[inline]
    fn entity_ref<'a>(&'a self, entity_info: &'a QEntityInfo) -> QEntityRef<'a> {
        debug_assert!(
            self.entities
                .as_ptr_range()
                .contains(&(entity_info as *const _)),
            "entity references must be constructed from entity infos contained within self",
        );

        QEntityRef {
            entities: self,
            entity_info,
        }
    }

    /// Returns a closure that can be used to invoke [`entity_ref()`](Self::entity_ref) on `self`.
    ///
    /// # Panics
    /// Invocation of the returned closure may lead to panicking under all the same circumstances
    /// that [`entity_ref()`](Self::entity_ref) may.
    #[inline]
    fn entity_ref_inator<'a>(&'a self) -> impl Fn(&'a QEntityInfo) -> QEntityRef<'a> {
        #[inline]
        |entity_info| self.entity_ref(entity_info)
    }

    /// Creates a new reference to a key-value within the collection.
    ///
    /// # Panics
    /// The correct operation of this function is dependent upon the passed in key-value info
    /// describing a key-value that is valid for `self`. As such this function expects that the
    /// passed in key-value info reference be a child of `self`.
    ///
    /// In debug builds this function explicitly panics when this condition is violated. In release
    /// builds this function on its own is incapable of panicking, but if the aforementioned
    /// condition has been violated, then it is possible for the returned reference to be used in
    /// such a manner that the program will subsequently panic.
    #[inline]
    fn kv_ref<'a>(&'a self, kv_info: &'a QEntityKeyValueInfo) -> QEntityKeyValueRef<'a> {
        debug_assert!(
            self.key_values
                .as_ptr_range()
                .contains(&(kv_info as *const _)),
            "key-value references must be constructed from key-value infos contained within self",
        );

        QEntityKeyValueRef {
            entities: self,
            kv_info,
        }
    }

    /// Returns a closure that can be used to invoke [`kv_ref()`](Self::kv_ref) on `self`.
    ///
    /// # Panics
    /// Invocation of the returned closure may lead to panicking under all the same circumstances
    /// that [`kv_ref()`](Self::kv_ref) may.
    #[inline]
    fn kv_ref_inator<'a>(&'a self) -> impl Fn(&'a QEntityKeyValueInfo) -> QEntityKeyValueRef<'a> {
        #[inline]
        |kv_info| self.kv_ref(kv_info)
    }

    /// Gets the number of entities within the collection.
    #[inline]
    pub fn len(&self) -> usize {
        self.entities.len()
    }

    /// Gets a [`QEntityRef`] by index.
    #[inline]
    pub fn get(&self, index: usize) -> Option<QEntityRef> {
        self.entities.get(index).map(self.entity_ref_inator())
    }

    /// Creates an iterator that yields [`QEntityRef`]s for the entities of the collection.
    #[inline]
    pub fn iter(&self) -> QEntitiesIter {
        QEntitiesIter::new(self)
    }
}

impl<'a> IntoIterator for &'a QEntities {
    type IntoIter = QEntitiesIter<'a>;
    type Item = QEntityRef<'a>;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// Reference to an entity within a [`QEntities`] collection.
#[derive(Clone, Copy)]
pub struct QEntityRef<'a> {
    /// The collection of q-entities in which the entity resides.
    entities: &'a QEntities,
    /// Information about the referenced entity.
    entity_info: &'a QEntityInfo,
}

impl fmt::Debug for QEntityRef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<'a> QEntityRef<'a> {
    /// Gets the number of key-values that compose the entity.
    #[inline]
    pub fn len(&self) -> usize {
        self.entity_info.kvs_length
    }

    /// Gets a [`QEntityKeyValueRef`] by index.
    #[inline]
    pub fn get(&self, index: usize) -> Option<QEntityKeyValueRef> {
        self.entities
            .key_values
            .get(self.entity_info.first_kv + index)
            .map(self.entities.kv_ref_inator())
    }

    /// Creates an iterator that yields [`QEntityKeyValueRef`]s for the key-values of the entity.
    #[inline]
    pub fn iter(&self) -> QEntityKeyValuesIter<'a> {
        QEntityKeyValuesIter::new(self.entities, self.entity_info)
    }
}

impl<'a> IntoIterator for QEntityRef<'a> {
    type IntoIter = QEntityKeyValuesIter<'a>;
    type Item = QEntityKeyValueRef<'a>;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// Reference to a key-value within a [`QEntities`] collection.
#[derive(Clone, Copy)]
pub struct QEntityKeyValueRef<'a> {
    /// The collection of q-entities in which the key-value resides.
    entities: &'a QEntities,
    /// Information about the referenced key-value.
    kv_info: &'a QEntityKeyValueInfo,
}

impl fmt::Debug for QEntityKeyValueRef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use bstr::BStr;
        write!(
            f,
            "{:?}: {:?}",
            BStr::new(self.key()),
            BStr::new(self.value()),
        )
    }
}

impl<'a> QEntityKeyValueRef<'a> {
    /// Gets a reference to the bytes of the key.
    #[inline]
    pub fn key(&self) -> &'a [u8] {
        &self.entities.byte_chunks[self.kv_info.key_chunk]
    }

    /// Gets a reference to the bytes of the value.
    #[inline]
    pub fn value(&self) -> &'a [u8] {
        &self.entities.byte_chunks[self.kv_info.value_chunk]
    }
}
