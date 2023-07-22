//! Implementation of the Quake entities format.

#![warn(missing_docs)]

mod entities_iter;
mod entity_kvs_iter;

pub use entities_iter::QEntitiesIter;
pub use entity_kvs_iter::QEntityKeyValuesIter;

/// Information describing an entity instance within a [collection of Quake entities](QEntities).
#[derive(Debug, Clone, Copy)]
struct QEntityInfo {
    /// Index of the entity's first key-value.
    first_kv: usize,
    /// Index of the entity's last key-value.
    last_kv: usize,
}

/// Information describing a key-value instance within a [collection of Quake entities](QEntities).
#[derive(Debug, Clone, Copy)]
struct QEntityKeyValueInfo {
    /// Index of the byte-chunk for the key.
    key_chunk: usize,
    /// Index of the byte-chunk for the value.
    value_chunk: usize,
}

/// Information describing a chunk of bytes within a [collection of Quake entities](QEntities).
#[derive(Debug, Clone, Copy)]
struct QEntitiesByteChunkInfo {
    /// Offset to the first byte of the chunk.
    offset: usize,
    /// The length of the byte-chunk.
    length: usize,
}

/// Collection of Quake entities.
#[derive(Debug)]
pub struct QEntities {
    entities: Box<[QEntityInfo]>,
    key_values: Box<[QEntityKeyValueInfo]>,
    byte_chunks: Box<[QEntitiesByteChunkInfo]>,
    bytes: Box<[u8]>,
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
    /// builds this function will only panic if the passed in entity info happens to be invalid for
    /// `self`.
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

    /// Creates a new reference to a key-value within the collection.
    ///
    /// # Panics
    /// The correct operation of this function is dependent upon the passed in key-value info
    /// describing a key-value that is valid for `self`. As such this function expects that the
    /// passed in entity info reference be a child of `self`.
    ///
    /// In debug builds this function explicitly panics when this condition is violated. In release
    /// builds this function will only panic if the passed in key-value info happens to be invalid
    /// for `self`.
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

    /// Creates a new reference to a byte-chunk within the collection.
    ///
    /// # Panics
    /// The correct operation of this function is dependent upon the passed in byte-chunk info
    /// describing a byte-chunk that is valid for `self`. As such this function expects that the
    /// passed in byte-chunk info reference be a child of `self`.
    ///
    /// In debug builds this function explicitly panics when this condition is violated. In release
    /// builds this function will only panic if the passed in byte-chunk info happens to be invalid
    /// for `self`.
    #[inline]
    fn byte_chunk_ref<'a>(&'a self, byte_chunk_info: &'a QEntitiesByteChunkInfo) -> &'a [u8] {
        debug_assert!(
            self.byte_chunks
                .as_ptr_range()
                .contains(&(byte_chunk_info as *const _)),
            "byte-chunk references must be constructed from byte-chunk infos contained within self",
        );

        let start = byte_chunk_info.offset;
        let end = start + byte_chunk_info.length;
        &self.bytes[start..end]
    }

    /// Creates an iterator that returns [references to the entities](QEntityRef) of the collection.
    #[inline]
    pub fn iter(&self) -> QEntitiesIter {
        QEntitiesIter {
            entities: self,
            inner_iter: self.entities.iter(),
        }
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

/// Reference to an entity within a [collection of Quake entities](QEntities).
#[derive(Debug, Clone, Copy)]
pub struct QEntityRef<'a> {
    /// The collection of Quake entities in which the entity resides.
    entities: &'a QEntities,
    /// Information about the referenced entity.
    entity_info: &'a QEntityInfo,
}

impl<'a> QEntityRef<'a> {
    /// Creates an iterator that returns [references to the key-values](QEntityKeyValueRef) of the
    /// entity.
    #[inline]
    pub fn iter(&self) -> QEntityKeyValuesIter<'a> {
        let kvs_slice =
            &self.entities.key_values[self.entity_info.first_kv..self.entity_info.last_kv];
        QEntityKeyValuesIter {
            entities: self.entities,
            inner_iter: kvs_slice.iter(),
        }
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

/// Reference to a key-value within a [collection of Quake entities](QEntities).
#[derive(Debug, Clone, Copy)]
pub struct QEntityKeyValueRef<'a> {
    /// The collection of Quake entities in which the key-value resides.
    entities: &'a QEntities,
    /// Information about the referenced key-value.
    kv_info: &'a QEntityKeyValueInfo,
}

impl<'a> QEntityKeyValueRef<'a> {
    /// Retrieves a reference to the bytes of the key.
    #[inline]
    pub fn key(&self) -> &'a [u8] {
        self.entities
            .byte_chunk_ref(&self.entities.byte_chunks[self.kv_info.key_chunk])
    }

    /// Retrieves a reference to the bytes of the value.
    #[inline]
    pub fn value(&self) -> &'a [u8] {
        self.entities
            .byte_chunk_ref(&self.entities.byte_chunks[self.kv_info.value_chunk])
    }
}
