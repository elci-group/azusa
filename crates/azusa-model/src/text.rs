// Copyright (c) 2026 The azusa Authors
// SPDX-License-Identifier: MIT

//! Text heuristics shared by state providers.
//!
//! Both the filesystem and Git providers classify blobs as text and count
//! lines; one implementation keeps the classification identical across
//! providers.

/// Whether a byte slice is plausibly text: no NUL byte in the first 8 KiB.
#[must_use]
pub fn looks_like_text(bytes: &[u8]) -> bool {
    !bytes[..bytes.len().min(8192)].contains(&0)
}

/// Count lines by terminator; a final unterminated line counts too.
#[must_use]
pub fn count_lines(bytes: &[u8]) -> u64 {
    let mut lines = bytes.iter().filter(|&&b| b == b'\n').count() as u64;
    if !bytes.is_empty() && !bytes.ends_with(b"\n") {
        lines += 1;
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_terminated_and_partial_lines() {
        assert_eq!(count_lines(b"a\nb\n"), 2);
        assert_eq!(count_lines(b"a\nb"), 2);
        assert_eq!(count_lines(b""), 0);
    }

    #[test]
    fn text_detection_rejects_binary() {
        assert!(looks_like_text(b"plain text"));
        assert!(!looks_like_text(b"plain\x00binary"));
    }
}
