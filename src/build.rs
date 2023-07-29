//! Module containing builder types for constructing [`QEntities`] instances.

use crate::byte_chunk::ByteChunksBuilder;
use crate::{QEntities, QEntityInfo, QEntityKeyValueInfo};
use core::hash::BuildHasher;

/// Builder for iteratively constructing a [`QEntities`] collection.
pub(crate) struct QEntitiesBuilder<S> {
    entities: Vec<QEntityInfo>,
    key_values: Vec<QEntityKeyValueInfo>,
    byte_chunks: ByteChunksBuilder<S>,
}

impl<S> QEntitiesBuilder<S> {
    /// Creates a new builder using the given hasher.
    #[inline]
    pub fn with_hasher(hash_builder: S) -> Self {
        Self {
            entities: Vec::new(),
            key_values: Vec::new(),
            byte_chunks: ByteChunksBuilder::with_hasher(hash_builder),
        }
    }

    /// Creates a new [`QEntityBuilder`] that is used to construct an entity for `self`.
    #[inline(always)]
    pub fn entity(&mut self) -> QEntityBuilder<S> {
        QEntityBuilder::new(self)
    }

    /// Consume `self` and construct a new [`QEntities`] collection.
    #[inline]
    pub fn finish(self) -> QEntities {
        QEntities {
            entities: self.entities.into(),
            key_values: self.key_values.into(),
            byte_chunks: self.byte_chunks.into(),
        }
    }
}

impl<S: Default> Default for QEntitiesBuilder<S> {
    #[inline]
    fn default() -> Self {
        Self {
            entities: Vec::new(),
            key_values: Vec::new(),
            byte_chunks: ByteChunksBuilder::default(),
        }
    }
}

/// Builder for iteratively constructing an entity within a [`QEntitiesBuilder`].
pub(crate) struct QEntityBuilder<'a, S> {
    entities: &'a mut QEntitiesBuilder<S>,
}

impl<'a, S> QEntityBuilder<'a, S> {
    /// Creates a new builder for the given entities builder.
    #[inline]
    pub fn new(entities: &'a mut QEntitiesBuilder<S>) -> Self {
        entities.entities.push(QEntityInfo {
            first_kv: entities.key_values.len(),
            kvs_length: 0,
        });
        Self { entities }
    }

    /// Inserts a new key-value into the entity.
    pub fn key_value(&mut self, key: &[u8], value: &[u8]) -> &mut Self
    where
        S: BuildHasher,
    {
        self.entities.key_values.push(QEntityKeyValueInfo {
            key_chunk: self.entities.byte_chunks.chunk(key),
            value_chunk: self.entities.byte_chunks.chunk(value),
        });
        self.entities.entities.last_mut().unwrap().kvs_length += 1;
        self
    }

    /// Consume `self` committing the entity in the process and return the underlying
    /// [`QEntitiesBuilder`].
    #[inline(always)]
    pub fn finish(self) -> &'a mut QEntitiesBuilder<S> {
        self.entities
    }
}

impl<S> From<QEntitiesBuilder<S>> for QEntities {
    /// Consume a q-entities builder and construct a new [`QEntities`] collection from it.
    #[inline(always)]
    fn from(value: QEntitiesBuilder<S>) -> Self {
        value.finish()
    }
}
