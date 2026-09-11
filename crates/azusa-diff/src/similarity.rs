//! String similarity for rename detection.
//!
//! Jaro–Winkler similarity over repository-relative paths: deterministic,
//! cheap, and well-suited to spotting moves (`src/auth.rs` →
//! `src/session/auth.rs`).

/// Jaro similarity of two strings, 0.0–1.0.
#[must_use]
pub fn jaro(a: &str, b: &str) -> f64 {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    let max_distance = (a.len().max(b.len()) / 2).saturating_sub(1);
    let mut a_matched = vec![false; a.len()];
    let mut b_matched = vec![false; b.len()];
    let mut matches = 0_usize;

    for (i, ca) in a.iter().enumerate() {
        let start = i.saturating_sub(max_distance);
        let end = (i + max_distance + 1).min(b.len());
        for j in start..end {
            if !b_matched[j] && *ca == b[j] {
                a_matched[i] = true;
                b_matched[j] = true;
                matches += 1;
                break;
            }
        }
    }
    if matches == 0 {
        return 0.0;
    }
    let mut transpositions = 0_usize;
    let mut k = 0_usize;
    for (i, matched) in a_matched.iter().enumerate() {
        if !matched {
            continue;
        }
        while !b_matched[k] {
            k += 1;
        }
        if a[i] != b[k] {
            transpositions += 1;
        }
        k += 1;
    }
    let m = matches as f64;
    (m / a.len() as f64 + m / b.len() as f64 + (m - transpositions as f64 / 2.0) / m) / 3.0
}

/// Jaro–Winkler similarity (prefix-boosted), 0.0–1.0.
#[must_use]
pub fn jaro_winkler(a: &str, b: &str) -> f64 {
    let jaro = jaro(a, b);
    if jaro <= 0.7 {
        return jaro;
    }
    let prefix = a
        .chars()
        .zip(b.chars())
        .take(4)
        .take_while(|(x, y)| x == y)
        .count();
    jaro + 0.1 * prefix as f64 * (1.0 - jaro)
}

/// Combined rename plausibility for a path pair: Jaro–Winkler over the full
/// paths plus a filename-equality bonus, clamped to 0.0–1.0.
#[must_use]
pub fn path_rename_similarity(from: &str, to: &str) -> f64 {
    let base = jaro_winkler(from, to);
    let same_name = std::path::Path::new(from)
        .file_name()
        .and_then(|f| f.to_str())
        .zip(
            std::path::Path::new(to)
                .file_name()
                .and_then(|f| f.to_str()),
        )
        .is_some_and(|(f, t)| f == t);
    let boosted = if same_name {
        base + (1.0 - base) * 0.5
    } else {
        base
    };
    boosted.clamp(0.0, 1.0)
}

/// Gate for similarity-based rename pairing: the basenames must be equal or
/// near-identical. Similar extensions alone (`gone.txt` vs `new.txt`) must
/// never fabricate a rename.
#[must_use]
pub fn basename_rename_gate(from: &str, to: &str) -> bool {
    let f = basename(from);
    let t = basename(to);
    f == t || jaro_winkler(f, t) >= 0.9
}

/// The final path component, or the whole path when there is none.
fn basename(path: &str) -> &str {
    std::path::Path::new(path)
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_and_disjoint() {
        assert!((jaro_winkler("src/auth.rs", "src/auth.rs") - 1.0).abs() < 1e-9);
        assert!(jaro_winkler("zzz", "yyy") < 0.5);
    }

    #[test]
    fn moves_score_higher_than_unrelated() {
        let move_score = path_rename_similarity("src/auth.rs", "src/session/auth.rs");
        let unrelated = path_rename_similarity("src/auth.rs", "docs/billing.md");
        assert!(move_score > unrelated);
        assert!(move_score > 0.85);
    }
}
