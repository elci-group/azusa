// Copyright (c) 2026 The azusa Authors
// SPDX-License-Identifier: MIT

//! Git state selectors.
//!
//! Selectors name the state a caller wants captured:
//! `worktree`, `index`, `head`, `upstream`, `remote`, or any revision
//! libgit2 can resolve (`main`, `origin/main`, `HEAD~3`, an object id, …).

use serde::{Deserialize, Serialize};

/// A named Git state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GitSelector {
    /// The working directory as it exists on disk (including uncommitted and
    /// untracked files).
    Worktree,
    /// The staged index.
    Index,
    /// The commit `HEAD` points at.
    Head,
    /// The upstream tracking branch of `HEAD` (e.g. `origin/main`).
    Upstream,
    /// The remote repository's default branch (`origin/HEAD`, falling back
    /// to `origin/main`, then `origin/master`).
    RemoteDefault,
    /// An arbitrary revision, resolved through libgit2.
    Rev(String),
}

impl GitSelector {
    /// Parse a user selector string.
    #[must_use]
    pub fn parse(spec: &str) -> Self {
        match spec {
            "worktree" | "wt" => Self::Worktree,
            "index" => Self::Index,
            "head" | "HEAD" => Self::Head,
            "upstream" | "@{u}" => Self::Upstream,
            "remote" | "origin" => Self::RemoteDefault,
            other => Self::Rev(other.to_owned()),
        }
    }

    /// The canonical label for this selector.
    #[must_use]
    pub fn label(&self) -> String {
        match self {
            Self::Worktree => "worktree".into(),
            Self::Index => "index".into(),
            Self::Head => "head".into(),
            Self::Upstream => "upstream".into(),
            Self::RemoteDefault => "remote-default".into(),
            Self::Rev(rev) => rev.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_keywords_and_revisions() {
        assert_eq!(GitSelector::parse("worktree"), GitSelector::Worktree);
        assert_eq!(GitSelector::parse("wt"), GitSelector::Worktree);
        assert_eq!(GitSelector::parse("index"), GitSelector::Index);
        assert_eq!(GitSelector::parse("head"), GitSelector::Head);
        assert_eq!(GitSelector::parse("upstream"), GitSelector::Upstream);
        assert_eq!(GitSelector::parse("remote"), GitSelector::RemoteDefault);
        assert_eq!(
            GitSelector::parse("origin/main"),
            GitSelector::Rev("origin/main".into())
        );
        assert_eq!(
            GitSelector::parse("HEAD~3"),
            GitSelector::Rev("HEAD~3".into())
        );
    }

    #[test]
    fn labels_are_canonical() {
        assert_eq!(GitSelector::parse("wt").label(), "worktree");
        assert_eq!(GitSelector::Rev("v1.2".into()).label(), "v1.2");
    }
}
