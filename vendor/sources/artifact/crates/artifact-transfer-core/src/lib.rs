//! Pure chunk geometry and explicit digest identity for artifact transfer.
//!
//! This crate defines no serialized manifest and performs no hashing, I/O,
//! transport, storage, compression, encryption, or authorization. Consumers
//! adapt their own wire formats into explicit chunk geometry and digest tags.

#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

use alloc::vec::Vec;
use core::fmt;

/// Byte length of currently supported `BLAKE3` and `SHA-256` digests.
pub const DIGEST_BYTES: usize = 32;

/// Explicit digest algorithm identity.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DigestAlgorithm {
    /// `BLAKE3` with a 32-byte output.
    Blake3,
    /// `SHA-256`, retained for interoperable formats that require it.
    Sha256,
}

// r[impl artifact_transfer.digest.tagged]
/// A digest whose algorithm cannot be confused with equal bytes from another algorithm.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TaggedDigest {
    algorithm: DigestAlgorithm,
    bytes: [u8; DIGEST_BYTES],
}

impl TaggedDigest {
    /// Construct a digest from an exact-size byte array.
    #[must_use]
    pub const fn new(algorithm: DigestAlgorithm, bytes: [u8; DIGEST_BYTES]) -> Self {
        Self { algorithm, bytes }
    }

    /// Parse an exact-size digest byte slice.
    ///
    /// # Errors
    ///
    /// Returns [`DigestError::InvalidLength`] when the slice is not exactly
    /// [`DIGEST_BYTES`] long.
    pub fn from_slice(algorithm: DigestAlgorithm, bytes: &[u8]) -> Result<Self, DigestError> {
        let actual_bytes = bytes.len();
        let digest_bytes = bytes
            .try_into()
            .map_err(|_error| DigestError::InvalidLength {
                expected_bytes: DIGEST_BYTES,
                actual_bytes,
            })?;
        Ok(Self::new(algorithm, digest_bytes))
    }

    /// Return the explicit algorithm identity.
    #[must_use]
    pub const fn algorithm(self) -> DigestAlgorithm {
        self.algorithm
    }

    /// Return the raw digest bytes.
    #[must_use]
    pub const fn bytes(self) -> [u8; DIGEST_BYTES] {
        self.bytes
    }

    /// Borrow the raw digest bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; DIGEST_BYTES] {
        &self.bytes
    }
}

/// Digest construction failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DigestError {
    /// The supplied digest byte sequence has the wrong length.
    InvalidLength {
        /// Required byte length.
        expected_bytes: usize,
        /// Supplied byte length.
        actual_bytes: usize,
    },
}

impl fmt::Display for DigestError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLength {
                expected_bytes,
                actual_bytes,
            } => write!(
                formatter,
                "digest length mismatch: expected {expected_bytes} bytes, got {actual_bytes}"
            ),
        }
    }
}

/// Digest identities used by a consumer-owned artifact format.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IdentityProfile {
    /// Algorithm used for individual retained chunks.
    pub chunk_digest: DigestAlgorithm,
    /// Algorithm used for the reassembled artifact, when the format defines one.
    pub artifact_digest: Option<DigestAlgorithm>,
}

/// Explicit chunk-boundary mechanism.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChunkingAlgorithm {
    /// Fixed-size chunks with a possibly shorter final chunk.
    Fixed {
        /// Required size of every non-final chunk and maximum final size.
        chunk_bytes: u64,
    },
    /// `FastCDC` content-defined chunk bounds.
    ///
    /// This validates declared output geometry only. It does not implement or
    /// identify a specific `FastCDC` gear table or boundary algorithm version.
    FastCdc {
        /// Minimum non-final chunk size.
        min_chunk_bytes: u64,
        /// Declared target average chunk size.
        avg_chunk_bytes: u64,
        /// Maximum chunk size.
        max_chunk_bytes: u64,
    },
    /// Ordered parts segmented by an external, format-owned mechanism.
    ///
    /// Only positive lengths, checked summation, count, and total coverage are
    /// validated. No fixed or content-defined boundary claim is made.
    ExternalOrderedParts,
}

/// Whether a consumer format admits a zero-byte artifact.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EmptyArtifactPolicy {
    /// An empty artifact is represented by zero chunks.
    Allow,
    /// A zero-byte artifact is outside the consumer format.
    Reject,
}

/// Consumer-owned bounds for one geometry validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GeometryLimits {
    /// Largest declared artifact byte length.
    pub max_total_bytes: u64,
    /// Largest admitted chunk count.
    pub max_chunks: usize,
    /// Empty-artifact policy.
    pub empty_artifact: EmptyArtifactPolicy,
}

/// One derived ordered chunk range.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ChunkRange {
    /// Zero-based position in consumer-supplied order.
    pub index: usize,
    /// Derived byte offset in the reassembled artifact.
    pub offset_bytes: u64,
    /// Declared chunk length.
    pub length_bytes: u64,
}

/// Validated coverage summary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedGeometry {
    /// Declared and reconstructed artifact length.
    pub total_bytes: u64,
    /// Derived ordered non-overlapping ranges.
    pub ranges: Vec<ChunkRange>,
}

// r[impl artifact_transfer.geometry.explicit]
// r[impl artifact_transfer.claims.bounded]
/// Validate ordered chunk lengths under an explicit mechanism and bounds.
///
/// # Errors
///
/// Returns a typed error for invalid profiles, exceeded bounds, empty-artifact
/// policy failures, zero or out-of-profile chunks, checked-sum overflow, or a
/// reconstructed total that differs from the declared artifact length.
pub fn validate_geometry(
    total_bytes: u64,
    chunking: ChunkingAlgorithm,
    chunk_lengths: &[u64],
    limits: GeometryLimits,
) -> Result<ValidatedGeometry, GeometryError> {
    validate_chunking_profile(chunking)?;
    if total_bytes > limits.max_total_bytes {
        return Err(GeometryError::TotalBytesLimitExceeded {
            maximum_bytes: limits.max_total_bytes,
            actual_bytes: total_bytes,
        });
    }
    if chunk_lengths.len() > limits.max_chunks {
        return Err(GeometryError::ChunkCountLimitExceeded {
            maximum_chunks: limits.max_chunks,
            actual_chunks: chunk_lengths.len(),
        });
    }
    if total_bytes == 0 {
        if !chunk_lengths.is_empty() {
            return Err(GeometryError::EmptyArtifactHasChunks);
        }
        if limits.empty_artifact == EmptyArtifactPolicy::Reject {
            return Err(GeometryError::EmptyArtifactRejected);
        }
        return Ok(ValidatedGeometry {
            total_bytes,
            ranges: Vec::new(),
        });
    }
    if chunk_lengths.is_empty() {
        return Err(GeometryError::NonEmptyArtifactHasNoChunks);
    }

    let mut ranges = Vec::with_capacity(chunk_lengths.len());
    let mut offset_bytes = 0_u64;
    for (index, length_bytes) in chunk_lengths.iter().copied().enumerate() {
        if length_bytes == 0 {
            return Err(GeometryError::ChunkLengthZero { index });
        }
        let is_final = index + 1 == chunk_lengths.len();
        validate_chunk_length(chunking, index, length_bytes, is_final)?;
        ranges.push(ChunkRange {
            index,
            offset_bytes,
            length_bytes,
        });
        offset_bytes = offset_bytes
            .checked_add(length_bytes)
            .ok_or(GeometryError::ChunkTotalOverflow { index })?;
    }

    if offset_bytes != total_bytes {
        return Err(GeometryError::TotalMismatch {
            declared_bytes: total_bytes,
            reconstructed_bytes: offset_bytes,
        });
    }
    Ok(ValidatedGeometry {
        total_bytes,
        ranges,
    })
}

fn validate_chunking_profile(chunking: ChunkingAlgorithm) -> Result<(), GeometryError> {
    match chunking {
        ChunkingAlgorithm::Fixed { chunk_bytes: 0 } => Err(GeometryError::FixedChunkSizeZero),
        ChunkingAlgorithm::FastCdc {
            min_chunk_bytes,
            avg_chunk_bytes,
            max_chunk_bytes,
        } if min_chunk_bytes == 0
            || min_chunk_bytes >= avg_chunk_bytes
            || avg_chunk_bytes >= max_chunk_bytes =>
        {
            Err(GeometryError::FastCdcBoundsInvalid {
                min_chunk_bytes,
                avg_chunk_bytes,
                max_chunk_bytes,
            })
        }
        ChunkingAlgorithm::Fixed { .. }
        | ChunkingAlgorithm::FastCdc { .. }
        | ChunkingAlgorithm::ExternalOrderedParts => Ok(()),
    }
}

fn validate_chunk_length(
    chunking: ChunkingAlgorithm,
    index: usize,
    length_bytes: u64,
    is_final: bool,
) -> Result<(), GeometryError> {
    match chunking {
        ChunkingAlgorithm::Fixed { chunk_bytes } if !is_final && length_bytes != chunk_bytes => {
            Err(GeometryError::FixedNonFinalLength {
                index,
                expected_bytes: chunk_bytes,
                actual_bytes: length_bytes,
            })
        }
        ChunkingAlgorithm::Fixed { chunk_bytes } if is_final && length_bytes > chunk_bytes => {
            Err(GeometryError::FixedFinalLengthExceeded {
                index,
                maximum_bytes: chunk_bytes,
                actual_bytes: length_bytes,
            })
        }
        ChunkingAlgorithm::FastCdc {
            min_chunk_bytes, ..
        } if !is_final && length_bytes < min_chunk_bytes => {
            Err(GeometryError::FastCdcNonFinalBelowMinimum {
                index,
                minimum_bytes: min_chunk_bytes,
                actual_bytes: length_bytes,
            })
        }
        ChunkingAlgorithm::FastCdc {
            max_chunk_bytes, ..
        } if length_bytes > max_chunk_bytes => Err(GeometryError::FastCdcMaximumExceeded {
            index,
            maximum_bytes: max_chunk_bytes,
            actual_bytes: length_bytes,
        }),
        ChunkingAlgorithm::Fixed { .. }
        | ChunkingAlgorithm::FastCdc { .. }
        | ChunkingAlgorithm::ExternalOrderedParts => Ok(()),
    }
}

/// Pure chunk-geometry validation failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GeometryError {
    /// Fixed-size profile declares a zero chunk size.
    FixedChunkSizeZero,
    /// `FastCDC` bounds are not strictly increasing from a positive minimum.
    FastCdcBoundsInvalid {
        /// Declared minimum.
        min_chunk_bytes: u64,
        /// Declared average.
        avg_chunk_bytes: u64,
        /// Declared maximum.
        max_chunk_bytes: u64,
    },
    /// Declared artifact length exceeds consumer bounds.
    TotalBytesLimitExceeded {
        /// Consumer maximum.
        maximum_bytes: u64,
        /// Declared artifact length.
        actual_bytes: u64,
    },
    /// Chunk count exceeds consumer bounds.
    ChunkCountLimitExceeded {
        /// Consumer maximum.
        maximum_chunks: usize,
        /// Supplied count.
        actual_chunks: usize,
    },
    /// Consumer policy rejects zero-byte artifacts.
    EmptyArtifactRejected,
    /// A zero-byte artifact declared one or more chunks.
    EmptyArtifactHasChunks,
    /// A non-empty artifact declared no chunks.
    NonEmptyArtifactHasNoChunks,
    /// A chunk has zero length.
    ChunkLengthZero {
        /// Offending ordered index.
        index: usize,
    },
    /// A non-final fixed chunk differs from the exact configured size.
    FixedNonFinalLength {
        /// Offending ordered index.
        index: usize,
        /// Required length.
        expected_bytes: u64,
        /// Declared length.
        actual_bytes: u64,
    },
    /// The final fixed chunk exceeds the configured size.
    FixedFinalLengthExceeded {
        /// Offending ordered index.
        index: usize,
        /// Maximum length.
        maximum_bytes: u64,
        /// Declared length.
        actual_bytes: u64,
    },
    /// A non-final `FastCDC` chunk is below the declared minimum.
    FastCdcNonFinalBelowMinimum {
        /// Offending ordered index.
        index: usize,
        /// Declared minimum.
        minimum_bytes: u64,
        /// Declared length.
        actual_bytes: u64,
    },
    /// A `FastCDC` chunk exceeds the declared maximum.
    FastCdcMaximumExceeded {
        /// Offending ordered index.
        index: usize,
        /// Declared maximum.
        maximum_bytes: u64,
        /// Declared length.
        actual_bytes: u64,
    },
    /// Checked summation overflowed while adding one chunk.
    ChunkTotalOverflow {
        /// Offending ordered index.
        index: usize,
    },
    /// Reconstructed size differs from the declared artifact length.
    TotalMismatch {
        /// Declared artifact length.
        declared_bytes: u64,
        /// Checked reconstructed length.
        reconstructed_bytes: u64,
    },
}

impl fmt::Display for GeometryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "invalid artifact chunk geometry: {self:?}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXED_CHUNK_BYTES: u64 = 4;
    const FIXED_TOTAL_BYTES: u64 = 10;
    const SHORT_FINAL_BYTES: u64 = 2;
    const TWO_FULL_FIXED_BYTES: u64 = FIXED_CHUNK_BYTES * 2;
    const TWO_CHUNK_LIMIT: usize = 2;
    const FINAL_CHUNK_INDEX: usize = 2;
    const FASTCDC_MIN_BYTES: u64 = 2;
    const FASTCDC_AVG_BYTES: u64 = 4;
    const FASTCDC_MAX_BYTES: u64 = 8;
    const GENEROUS_TOTAL_LIMIT_BYTES: u64 = 1_024;
    const GENEROUS_CHUNK_LIMIT: usize = 16;
    const SHORT_DIGEST_BYTES: usize = DIGEST_BYTES - 1;

    fn limits(empty_artifact: EmptyArtifactPolicy) -> GeometryLimits {
        GeometryLimits {
            max_total_bytes: GENEROUS_TOTAL_LIMIT_BYTES,
            max_chunks: GENEROUS_CHUNK_LIMIT,
            empty_artifact,
        }
    }

    // r[verify artifact_transfer.geometry.explicit]
    #[test]
    fn fixed_geometry_derives_contiguous_ranges_and_short_final_chunk() {
        let result = validate_geometry(
            FIXED_TOTAL_BYTES,
            ChunkingAlgorithm::Fixed {
                chunk_bytes: FIXED_CHUNK_BYTES,
            },
            &[FIXED_CHUNK_BYTES, FIXED_CHUNK_BYTES, SHORT_FINAL_BYTES],
            limits(EmptyArtifactPolicy::Reject),
        );
        let expected = ValidatedGeometry {
            total_bytes: FIXED_TOTAL_BYTES,
            ranges: Vec::from([
                ChunkRange {
                    index: 0,
                    offset_bytes: 0,
                    length_bytes: FIXED_CHUNK_BYTES,
                },
                ChunkRange {
                    index: 1,
                    offset_bytes: FIXED_CHUNK_BYTES,
                    length_bytes: FIXED_CHUNK_BYTES,
                },
                ChunkRange {
                    index: FINAL_CHUNK_INDEX,
                    offset_bytes: TWO_FULL_FIXED_BYTES,
                    length_bytes: SHORT_FINAL_BYTES,
                },
            ]),
        };

        assert_eq!(result, Ok(expected));
    }

    #[test]
    fn fastcdc_geometry_allows_only_the_final_chunk_below_minimum() {
        let result = validate_geometry(
            FASTCDC_MAX_BYTES + 1,
            ChunkingAlgorithm::FastCdc {
                min_chunk_bytes: FASTCDC_MIN_BYTES,
                avg_chunk_bytes: FASTCDC_AVG_BYTES,
                max_chunk_bytes: FASTCDC_MAX_BYTES,
            },
            &[FASTCDC_AVG_BYTES, FASTCDC_AVG_BYTES, 1],
            limits(EmptyArtifactPolicy::Reject),
        );
        let expected = ValidatedGeometry {
            total_bytes: FASTCDC_MAX_BYTES + 1,
            ranges: Vec::from([
                ChunkRange {
                    index: 0,
                    offset_bytes: 0,
                    length_bytes: FASTCDC_AVG_BYTES,
                },
                ChunkRange {
                    index: 1,
                    offset_bytes: FASTCDC_AVG_BYTES,
                    length_bytes: FASTCDC_AVG_BYTES,
                },
                ChunkRange {
                    index: FINAL_CHUNK_INDEX,
                    offset_bytes: FASTCDC_MAX_BYTES,
                    length_bytes: 1,
                },
            ]),
        };

        assert_eq!(result, Ok(expected));
    }

    #[test]
    fn external_ordered_parts_make_no_boundary_algorithm_claim() {
        let lengths = [1, FIXED_CHUNK_BYTES, FASTCDC_MAX_BYTES];
        let total = lengths.iter().sum();
        let result = validate_geometry(
            total,
            ChunkingAlgorithm::ExternalOrderedParts,
            &lengths,
            limits(EmptyArtifactPolicy::Reject),
        );
        let expected = ValidatedGeometry {
            total_bytes: total,
            ranges: Vec::from([
                ChunkRange {
                    index: 0,
                    offset_bytes: 0,
                    length_bytes: 1,
                },
                ChunkRange {
                    index: 1,
                    offset_bytes: 1,
                    length_bytes: FIXED_CHUNK_BYTES,
                },
                ChunkRange {
                    index: FINAL_CHUNK_INDEX,
                    offset_bytes: FIXED_CHUNK_BYTES + 1,
                    length_bytes: FASTCDC_MAX_BYTES,
                },
            ]),
        };

        assert_eq!(result, Ok(expected));
    }

    #[test]
    fn empty_artifact_policy_is_explicit() {
        let allowed = validate_geometry(
            0,
            ChunkingAlgorithm::ExternalOrderedParts,
            &[],
            limits(EmptyArtifactPolicy::Allow),
        );
        let rejected = validate_geometry(
            0,
            ChunkingAlgorithm::ExternalOrderedParts,
            &[],
            limits(EmptyArtifactPolicy::Reject),
        );
        let with_chunk = validate_geometry(
            0,
            ChunkingAlgorithm::ExternalOrderedParts,
            &[1],
            limits(EmptyArtifactPolicy::Allow),
        );

        assert_eq!(
            allowed,
            Ok(ValidatedGeometry {
                total_bytes: 0,
                ranges: Vec::new(),
            })
        );
        assert_eq!(rejected, Err(GeometryError::EmptyArtifactRejected));
        assert_eq!(with_chunk, Err(GeometryError::EmptyArtifactHasChunks));
    }

    #[test]
    fn fixed_geometry_rejects_wrong_nonfinal_final_and_total_shapes() {
        let algorithm = ChunkingAlgorithm::Fixed {
            chunk_bytes: FIXED_CHUNK_BYTES,
        };
        let wrong_nonfinal = validate_geometry(
            FIXED_TOTAL_BYTES,
            algorithm,
            &[SHORT_FINAL_BYTES, FIXED_CHUNK_BYTES, FIXED_CHUNK_BYTES],
            limits(EmptyArtifactPolicy::Reject),
        );
        let oversized_final = validate_geometry(
            FIXED_CHUNK_BYTES + 1,
            algorithm,
            &[FIXED_CHUNK_BYTES + 1],
            limits(EmptyArtifactPolicy::Reject),
        );
        let total_mismatch = validate_geometry(
            FIXED_TOTAL_BYTES,
            algorithm,
            &[FIXED_CHUNK_BYTES, FIXED_CHUNK_BYTES],
            limits(EmptyArtifactPolicy::Reject),
        );

        assert!(matches!(
            wrong_nonfinal,
            Err(GeometryError::FixedNonFinalLength { index: 0, .. })
        ));
        assert!(matches!(
            oversized_final,
            Err(GeometryError::FixedFinalLengthExceeded { index: 0, .. })
        ));
        assert_eq!(
            total_mismatch,
            Err(GeometryError::TotalMismatch {
                declared_bytes: FIXED_TOTAL_BYTES,
                reconstructed_bytes: TWO_FULL_FIXED_BYTES,
            })
        );
    }

    #[test]
    fn malformed_profiles_and_fastcdc_outputs_are_rejected() {
        let fixed_zero = validate_geometry(
            1,
            ChunkingAlgorithm::Fixed { chunk_bytes: 0 },
            &[1],
            limits(EmptyArtifactPolicy::Reject),
        );
        let invalid_fastcdc = validate_geometry(
            1,
            ChunkingAlgorithm::FastCdc {
                min_chunk_bytes: FASTCDC_AVG_BYTES,
                avg_chunk_bytes: FASTCDC_MIN_BYTES,
                max_chunk_bytes: FASTCDC_MAX_BYTES,
            },
            &[1],
            limits(EmptyArtifactPolicy::Reject),
        );
        let short_nonfinal = validate_geometry(
            FASTCDC_MIN_BYTES,
            ChunkingAlgorithm::FastCdc {
                min_chunk_bytes: FASTCDC_MIN_BYTES,
                avg_chunk_bytes: FASTCDC_AVG_BYTES,
                max_chunk_bytes: FASTCDC_MAX_BYTES,
            },
            &[1, 1],
            limits(EmptyArtifactPolicy::Reject),
        );
        let oversized = validate_geometry(
            FASTCDC_MAX_BYTES + 1,
            ChunkingAlgorithm::FastCdc {
                min_chunk_bytes: FASTCDC_MIN_BYTES,
                avg_chunk_bytes: FASTCDC_AVG_BYTES,
                max_chunk_bytes: FASTCDC_MAX_BYTES,
            },
            &[FASTCDC_MAX_BYTES + 1],
            limits(EmptyArtifactPolicy::Reject),
        );

        assert_eq!(fixed_zero, Err(GeometryError::FixedChunkSizeZero));
        assert!(matches!(
            invalid_fastcdc,
            Err(GeometryError::FastCdcBoundsInvalid { .. })
        ));
        assert!(matches!(
            short_nonfinal,
            Err(GeometryError::FastCdcNonFinalBelowMinimum { index: 0, .. })
        ));
        assert!(matches!(
            oversized,
            Err(GeometryError::FastCdcMaximumExceeded { index: 0, .. })
        ));
    }

    #[test]
    fn limits_zero_lengths_absent_chunks_and_sum_overflow_fail_closed() {
        let too_large = validate_geometry(
            FIXED_TOTAL_BYTES,
            ChunkingAlgorithm::ExternalOrderedParts,
            &[FIXED_TOTAL_BYTES],
            GeometryLimits {
                max_total_bytes: FIXED_TOTAL_BYTES - 1,
                ..limits(EmptyArtifactPolicy::Reject)
            },
        );
        let too_many = validate_geometry(
            FIXED_TOTAL_BYTES,
            ChunkingAlgorithm::ExternalOrderedParts,
            &[FIXED_CHUNK_BYTES, FIXED_CHUNK_BYTES, SHORT_FINAL_BYTES],
            GeometryLimits {
                max_chunks: TWO_CHUNK_LIMIT,
                ..limits(EmptyArtifactPolicy::Reject)
            },
        );
        let zero_chunk = validate_geometry(
            1,
            ChunkingAlgorithm::ExternalOrderedParts,
            &[0],
            limits(EmptyArtifactPolicy::Reject),
        );
        let no_chunks = validate_geometry(
            1,
            ChunkingAlgorithm::ExternalOrderedParts,
            &[],
            limits(EmptyArtifactPolicy::Reject),
        );
        let overflow = validate_geometry(
            u64::MAX,
            ChunkingAlgorithm::ExternalOrderedParts,
            &[u64::MAX, 1],
            GeometryLimits {
                max_total_bytes: u64::MAX,
                max_chunks: GENEROUS_CHUNK_LIMIT,
                empty_artifact: EmptyArtifactPolicy::Reject,
            },
        );

        assert!(matches!(
            too_large,
            Err(GeometryError::TotalBytesLimitExceeded { .. })
        ));
        assert!(matches!(
            too_many,
            Err(GeometryError::ChunkCountLimitExceeded { .. })
        ));
        assert_eq!(zero_chunk, Err(GeometryError::ChunkLengthZero { index: 0 }));
        assert_eq!(no_chunks, Err(GeometryError::NonEmptyArtifactHasNoChunks));
        assert_eq!(
            overflow,
            Err(GeometryError::ChunkTotalOverflow { index: 1 })
        );
    }

    // r[verify artifact_transfer.digest.tagged]
    // r[verify artifact_transfer.claims.bounded]
    #[test]
    fn equal_digest_bytes_remain_distinct_across_algorithms_and_bad_lengths_fail() {
        let bytes = [0_u8; DIGEST_BYTES];
        let blake3 = TaggedDigest::new(DigestAlgorithm::Blake3, bytes);
        let sha256 = TaggedDigest::new(DigestAlgorithm::Sha256, bytes);
        let invalid =
            TaggedDigest::from_slice(DigestAlgorithm::Blake3, &[0_u8; SHORT_DIGEST_BYTES]);

        assert_ne!(blake3, sha256);
        assert_eq!(blake3.algorithm(), DigestAlgorithm::Blake3);
        assert_eq!(sha256.algorithm(), DigestAlgorithm::Sha256);
        assert_eq!(blake3.as_bytes(), &bytes);
        assert_eq!(
            invalid,
            Err(DigestError::InvalidLength {
                expected_bytes: DIGEST_BYTES,
                actual_bytes: SHORT_DIGEST_BYTES,
            })
        );
    }
}
