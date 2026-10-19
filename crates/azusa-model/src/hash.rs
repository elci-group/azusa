// Copyright (c) 2026 The azusa Authors
// SPDX-License-Identifier: MIT

//! Content hashing primitives.
//!
//! Azusa hashes file content with BLAKE3 and derives deterministic digests
//! over canonical fileset listings. Hashes are the currency of cheap drift
//! detection: two states with equal per-file hashes are content-identical
//! regardless of timestamps.

use blake3::Hasher;
use std::io::Read;

/// A BLAKE3 content hash, hex-encoded (64 characters).
pub type ContentHash = String;

/// Hash a single blob of content.
#[must_use]
pub fn content_hash(bytes: &[u8]) -> ContentHash {
    let mut hasher = Hasher::new();
    hasher.update(bytes);
    hasher.finalize().to_hex().to_string()
}

/// Hash a reader's content incrementally, so large files never need to fit in
/// memory.
///
/// # Errors
///
/// Propagates I/O failures from the reader.
pub fn content_hash_reader(reader: impl Read) -> std::io::Result<ContentHash> {
    let mut hasher = Hasher::new();
    let mut buffer = [0_u8; 16 * 1024];
    let mut reader = reader;
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hasher.finalize().to_hex().to_string())
}

/// Deterministic digest over a canonical fileset listing.
///
/// Each entry contributes one line `"<mode> <hash> <path>\n"`. Because the
/// listing is ordered (callers use ordered maps) the digest is stable for
/// identical states and changes whenever any path, mode or hash changes.
#[must_use]
pub fn fileset_digest(entries: impl Iterator<Item = (String, String, String)>) -> ContentHash {
    let mut hasher = Hasher::new();
    for (mode, hash, path) in entries {
        hasher.update(mode.as_bytes());
        hasher.update(b" ");
        hasher.update(hash.as_bytes());
        hasher.update(b" ");
        hasher.update(path.as_bytes());
        hasher.update(b"\n");
    }
    hasher.finalize().to_hex().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_hash_is_stable_and_sensitive() {
        assert_eq!(content_hash(b"hello"), content_hash(b"hello"));
        assert_ne!(content_hash(b"hello"), content_hash(b"hellp"));
    }

    #[test]
    fn reader_hash_matches_byte_hash() {
        let bytes = vec![7_u8; 200_000];
        assert_eq!(
            content_hash_reader(&bytes[..]).unwrap(),
            content_hash(&bytes)
        );
    }

    #[test]
    fn fileset_digest_is_sensitive_to_hash_change() {
        let a = fileset_digest(
            [
                ("100644".into(), "h1".into(), "a.txt".into()),
                ("100644".into(), "h2".into(), "b.txt".into()),
            ]
            .into_iter(),
        );
        let b = fileset_digest(
            [
                ("100644".into(), "h1".into(), "a.txt".into()),
                ("100644".into(), "h9".into(), "b.txt".into()),
            ]
            .into_iter(),
        );
        assert_ne!(a, b);
    }
}
