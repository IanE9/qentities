//! Module containing the implementations for byte-chunk related types.
//!
//! The term _byte-chunk_ is used here to describe a slice of bytes within a larger _byte-chunks_
//! collection.
//!
//! The term _byte-chunks_ is used here to describe a collection of byte-sequences that co-exist
//! within a single allocation whose memory footprint is optimized by de-duplicating byte-sequences
//! that are already within the collection.

use core::fmt;
use core::hash::{BuildHasher, Hasher};
use core::ops;
use hashbrown::hash_map::{HashMap, RawEntryMut};

/// Information describing a chunk of bytes within a [`ByteChunks`] collection.
#[derive(Debug, Clone, Copy)]
struct ByteChunkInfo {
    /// Offset to the first byte of the chunk.
    offset: usize,
    /// The length of the byte-chunk.
    length: usize,
}

impl ByteChunkInfo {
    /// Uses the description provided by `self` to create a reference to a sub-slice of bytes within
    /// the provided slice of bytes.
    ///
    /// # Panics
    /// This function panics if `self` describes a sub-slice of bytes out of bounds for the provided
    /// slice of bytes.
    #[inline]
    fn slice_from<'a>(&self, bytes: &'a [u8]) -> &'a [u8] {
        let start = self.offset;
        let end = start + self.length;
        &bytes[start..end]
    }
}

/// Builder for a [`ByteChunks`] collection.
pub(crate) struct ByteChunksBuilder<S> {
    /// Buffer holding the full collection of bytes.
    bytes: Vec<u8>,
    /// Buffer holding the information for the individual chunks.
    chunks: Vec<ByteChunkInfo>,
    /// The [`BuildHasher`] that produces [`Hasher`]s for hashing byte-sequences.
    hash_builder: S,
    /// Hash map for mapping byte-sequences to the indices of byte-chunks.
    hashes: HashMap<usize, (), ()>,
}

impl<S: fmt::Debug> fmt::Debug for ByteChunksBuilder<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ByteChunksBuilder")
            .field(
                "byte_chunks",
                &ByteChunksDebugger::new(&self.bytes, &self.chunks),
            )
            .field("hash_builder", &self.hash_builder)
            .finish()
    }
}

impl<S> ByteChunksBuilder<S> {
    /// Creates a new builder using the given hasher.
    pub fn with_hasher(hash_builder: S) -> Self {
        Self {
            bytes: Vec::new(),
            chunks: Vec::new(),
            hash_builder,
            hashes: HashMap::with_hasher(()),
        }
    }

    /// Gets the index of the associated byte-chunk present in the builder. If there exists no
    /// associated byte-chunk, then a new one is inserted.
    pub fn get_or_insert(&mut self, bytes: &[u8]) -> usize
    where
        S: BuildHasher,
    {
        let hash_bytes = |bytes: &[u8]| -> u64 {
            let mut hasher = self.hash_builder.build_hasher();
            hasher.write(bytes);
            hasher.finish()
        };

        let bytes_hash = hash_bytes(bytes);
        match self
            .hashes
            .raw_entry_mut()
            .from_hash(bytes_hash, |existing_index| {
                self.chunks[*existing_index].slice_from(&self.bytes) == bytes
            }) {
            RawEntryMut::Occupied(occupied) => *occupied.key(),
            RawEntryMut::Vacant(vacant) => {
                let new_chunk_info = ByteChunkInfo {
                    offset: self.bytes.len(),
                    length: bytes.len(),
                };
                let new_chunk_index = self.chunks.len();
                self.bytes.extend_from_slice(bytes);
                self.chunks.push(new_chunk_info);

                vacant.insert_with_hasher(bytes_hash, new_chunk_index, (), |chunk_index| {
                    hash_bytes(self.chunks[*chunk_index].slice_from(&self.bytes))
                });

                new_chunk_index
            }
        }
    }
}

/// Collection of byte-chunks.
pub(crate) struct ByteChunks {
    /// The full collection of bytes.
    bytes: Box<[u8]>,
    /// The individual chunk infos.
    chunks: Box<[ByteChunkInfo]>,
}

impl fmt::Debug for ByteChunks {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        ByteChunksDebugger::new(&self.bytes, &self.chunks).fmt(f)
    }
}

impl<S> From<ByteChunksBuilder<S>> for ByteChunks {
    /// Consume a [`ByteChunksBuilder`] and construct a new [`ByteChunks`] collection from it.
    fn from(value: ByteChunksBuilder<S>) -> Self {
        Self {
            bytes: value.bytes.into_boxed_slice(),
            chunks: value.chunks.into_boxed_slice(),
        }
    }
}

impl ops::Index<usize> for ByteChunks {
    type Output = [u8];

    /// Retrieves a byte-chunk from `self` by index.
    ///
    /// # Panics
    /// This function panics if the index does not correspond to any byte-chunk in `self`.
    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        self.chunks[index].slice_from(&self.bytes)
    }
}

/// Type implementing [`fmt::Debug`] with the purpose of debugging a [`ByteChunksBuilder`] or a
/// [`ByteChunks`] collection.
struct ByteChunksDebugger<'a> {
    bytes: &'a [u8],
    chunks: &'a [ByteChunkInfo],
}

impl<'a> ByteChunksDebugger<'a> {
    fn new(bytes: &'a [u8], chunks: &'a [ByteChunkInfo]) -> Self {
        Self { bytes, chunks }
    }
}

impl fmt::Debug for ByteChunksDebugger<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        struct ByteChunkDebugger<'a>(&'a [u8]);
        impl fmt::Debug for ByteChunkDebugger<'_> {
            #[inline(always)]
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{:?}", bstr::BStr::new(self.0))
            }
        }

        f.debug_list()
            .entries(
                self.chunks
                    .iter()
                    .map(|chunk| ByteChunkDebugger(chunk.slice_from(&self.bytes))),
            )
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn byte_chunk_slicing() {
        let bytes = b"classnameworldspawn";

        let info_a = ByteChunkInfo {
            offset: 0,
            length: 9,
        };
        let info_b = ByteChunkInfo {
            offset: 9,
            length: 10,
        };
        let info_c = ByteChunkInfo {
            offset: bytes.len(),
            length: 0,
        };
        let info_d = ByteChunkInfo {
            offset: 0,
            length: bytes.len(),
        };

        let chunk_a = info_a.slice_from(bytes);
        let chunk_b = info_b.slice_from(bytes);
        let chunk_c = info_c.slice_from(bytes);
        let chunk_d = info_d.slice_from(bytes);

        assert_eq!(chunk_a, b"classname");
        assert_eq!(chunk_b, b"worldspawn");
        assert_eq!(chunk_c, b"");
        assert_eq!(chunk_d, bytes);
    }

    #[test]
    #[should_panic]
    fn byte_chunk_slicing_overflow() {
        let bytes = b"classnameworldspawn";
        ByteChunkInfo {
            offset: 0,
            length: bytes.len() + 1,
        }
        .slice_from(bytes);
    }

    #[test]
    #[should_panic]
    fn byte_chunk_slicing_out_of_bounds() {
        let bytes = b"classnameworldspawn";
        ByteChunkInfo {
            offset: bytes.len() + 1,
            length: 0,
        }
        .slice_from(bytes);
    }

    /// Helper type to for asserting that a [`ByteChunksBuilder`] is behaving correctly.
    ///
    /// To correctly use this type, the user need only to call [`insert_into()`] on `self` at least
    /// once with a [`ByteChunksBuilder`] instance, but multiple calls must be made to test for
    /// consistency.
    ///
    /// Making a call to [`insert_once()`] will insert the byte-sequence that was provided at
    /// construction into the [`ByteChunksBuilder`] and either assert that the returned chunk index
    /// matches that returned by all prior calls or store the returned chunk index if no prior calls
    /// had been made.
    ///
    /// All consistifiers must be finalized with a call to [`finalize()`] or as part of a call to
    /// [`finalize_many()`]. During finalization it will be asserted that the finally built
    /// [`ByteChunks`] collection contains the correct byte-sequence for the chunk index that had
    /// been returned during all insert operations.
    ///
    /// Finalization through [`finalize_many()`] will additionally ensure that the total number of
    /// bytes stored in the [`ByteChunks`] instances is less than or equal to the sum total of the
    /// number of bytes of all constifiers that were finalized by the call. The purpose of this is
    /// to ensure that the builder had successfully de-duplicated the inserted byte-sequences.
    struct ByteChunkConsistifier {
        bytes: &'static [u8],
        chunk: Option<usize>,
    }

    #[test]
    fn byte_chunk_building() {
        use core::hash::{BuildHasher, BuildHasherDefault};

        fn test_with_hasher<S: BuildHasher>(hasher: S) {
            let mut builder = ByteChunksBuilder::with_hasher(hasher);

            impl ByteChunkConsistifier {
                fn new(bytes: &'static [u8]) -> Self {
                    Self { bytes, chunk: None }
                }

                fn insert_into<S: BuildHasher>(&mut self, builder: &mut ByteChunksBuilder<S>) {
                    let chunk = builder.get_or_insert(self.bytes);
                    if let Some(expected_chunk) = self.chunk {
                        assert_eq!(expected_chunk, chunk);
                    } else {
                        self.chunk = Some(chunk);
                    }
                }

                fn finalize_many<const N: usize>(many: [Self; N], byte_chunks: &ByteChunks) {
                    let mut total_length = 0usize;
                    for one in many {
                        total_length += one.bytes.len();
                        one.finalize(byte_chunks);
                    }

                    // Assuming that the byte-chunks builder interned the bytes correctly, the total
                    // length of the byte-chunks should never exceed the total length of the inputs.
                    assert!(total_length <= byte_chunks.bytes.len());
                }

                fn finalize(self, byte_chunks: &ByteChunks) {
                    let this = core::mem::ManuallyDrop::new(self);
                    let chunk = this.chunk.expect("consistifier is used at least once");
                    assert_eq!(this.bytes, &byte_chunks[chunk]);
                }
            }

            impl Drop for ByteChunkConsistifier {
                fn drop(&mut self) {
                    panic!("consistifier must be consumed with finalize")
                }
            }

            let mut classname_consistifier = ByteChunkConsistifier::new(b"classname");
            let mut worldspawn_consistifier = ByteChunkConsistifier::new(b"worldspawn");
            let mut wad_consistifier = ByteChunkConsistifier::new(b"wad");
            let mut my_wad_consistifier = ByteChunkConsistifier::new(b"mywad.wad");
            let mut light_consistifier = ByteChunkConsistifier::new(b"light");
            let mut origin_consistifier = ByteChunkConsistifier::new(b"origin");
            let mut zero_zero_zero_consistifier = ByteChunkConsistifier::new(b"0 0 0");

            // Repeat simulation four times to ensure that all consistifiers perform multiple
            // insertions.
            for _ in 0..4 {
                // Simulate the worldspawn entity being interned.
                classname_consistifier.insert_into(&mut builder);
                worldspawn_consistifier.insert_into(&mut builder);
                wad_consistifier.insert_into(&mut builder);
                my_wad_consistifier.insert_into(&mut builder);

                // Simulate four lights being interned.
                for _ in 0..4 {
                    classname_consistifier.insert_into(&mut builder);
                    light_consistifier.insert_into(&mut builder);
                    origin_consistifier.insert_into(&mut builder);
                    zero_zero_zero_consistifier.insert_into(&mut builder);
                }
            }

            ByteChunkConsistifier::finalize_many(
                [
                    classname_consistifier,
                    worldspawn_consistifier,
                    wad_consistifier,
                    my_wad_consistifier,
                    light_consistifier,
                    origin_consistifier,
                    zero_zero_zero_consistifier,
                ],
                &ByteChunks::from(builder),
            );
        }

        test_with_hasher(hashbrown::hash_map::DefaultHashBuilder::default());
        test_with_hasher(BuildHasherDefault::<rustc_hash::FxHasher>::default());
    }
}
