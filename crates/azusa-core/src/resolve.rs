// Copyright (c) 2026 The azusa Authors
// SPDX-License-Identifier: MIT

//! State resolution: selectors to captured states.
//!
//! A selector names one repository state: a Git keyword (`worktree`,
//! `index`, `head`, `upstream`, `remote`), a Git revision (`main`,
//! `origin/main`, `HEAD~3`), or a filesystem path. The resolver picks the
//! right provider and returns the state plus a compact
//! [`StateRef`] for provenance.

use std::path::{Path, PathBuf};

use azusa_fs::FsProvider;
use azusa_git::{GitError, GitProvider, GitSelector};
use azusa_model::{RepositoryState, StateRef};

use crate::CoreError;

/// Resolves state selectors against a working root.
pub struct StateResolver {
    root: PathBuf,
}

impl StateResolver {
    /// Create a resolver rooted at `root`.
    #[must_use]
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
        }
    }

    /// The resolver's working root.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Whether a Git repository is discoverable from the root.
    #[must_use]
    pub fn has_git(&self) -> bool {
        GitProvider::is_repository(&self.root)
    }

    /// Resolve one selector into a captured state.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::UnresolvedState`] when no provider can resolve
    /// the selector, [`CoreError::NoProvider`] when no provider applies at
    /// the root, and [`CoreError::Provider`] on provider failure.
    pub fn resolve(&self, spec: &str) -> Result<RepositoryState, CoreError> {
        self.resolve_ref(spec).map(|(state, _reference)| state)
    }

    /// Resolve one selector into a captured state plus its provenance ref.
    ///
    /// # Errors
    ///
    /// As [`Self::resolve`].
    pub fn resolve_ref(&self, spec: &str) -> Result<(RepositoryState, StateRef), CoreError> {
        // 1. A directory path resolves through the filesystem provider.
        let path_candidate = PathBuf::from(spec);
        if path_candidate.is_dir() {
            let mut provider = FsProvider::new(&path_candidate).with_persistent_cache();
            let state = provider
                .snapshot()
                .map_err(|e| CoreError::Provider(format!("fs provider: {e}")))?;
            let state = azusa_graph::enrich(state);
            return Ok((state.clone(), state_ref(spec, &state)));
        }

        // 2. Git keywords and revisions resolve through the Git provider.
        if self.has_git() {
            let mut provider = GitProvider::discover(&self.root)
                .map_err(|e| CoreError::Provider(format!("git provider: {e}")))?;
            let selector = GitSelector::parse(spec);
            return match provider.snapshot(&selector) {
                Ok(state) => {
                    let state = azusa_graph::enrich(state);
                    Ok((state.clone(), state_ref(spec, &state)))
                }
                Err(GitError::UnresolvedSelector { selector, reason }) => {
                    Err(CoreError::UnresolvedState {
                        spec: selector,
                        reason,
                    })
                }
                Err(e) => Err(CoreError::Provider(format!("git provider: {e}"))),
            };
        }

        Err(CoreError::NoProvider(self.root.clone()))
    }

    /// Resolve the default comparison pair for this root.
    ///
    /// The default compares the local worktree against the upstream branch;
    /// when no upstream is configured it falls back to the remote default
    /// branch, then to HEAD. The returned notes explain any fallback so the
    /// report can carry the caveat as evidence.
    ///
    /// # Errors
    ///
    /// As [`Self::resolve_ref`] for each side; fails when no comparison
    /// target exists at all.
    pub fn resolve_default_pair(
        &self,
    ) -> Result<
        (
            RepositoryState,
            StateRef,
            RepositoryState,
            StateRef,
            Vec<String>,
        ),
        CoreError,
    > {
        let mut notes = Vec::new();
        let (a, a_ref) = self.resolve_ref("worktree")?;

        let b_spec_priority = ["upstream", "remote", "head"];
        let mut resolved = None;
        for (index, spec) in b_spec_priority.iter().enumerate() {
            match self.resolve_ref(spec) {
                Ok(pair) => {
                    resolved = Some(pair);
                    break;
                }
                Err(CoreError::UnresolvedState {
                    spec: wanted,
                    reason,
                }) if index + 1 < b_spec_priority.len() => {
                    notes.push(format!(
                        "state `{wanted}` unavailable ({reason}); falling back"
                    ));
                }
                Err(e) => return Err(e),
            }
        }
        let (b, b_ref) = resolved.ok_or_else(|| CoreError::UnresolvedState {
            spec: "upstream".into(),
            reason: "no upstream, remote branch or HEAD to compare against".into(),
        })?;
        Ok((a, a_ref, b, b_ref, notes))
    }
}

fn state_ref(spec: &str, state: &RepositoryState) -> StateRef {
    StateRef {
        spec: spec.to_owned(),
        provider: state.identity.provider,
        label: state.identity.label.clone(),
        digest: state.identity.digest.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use azusa_git::testutil::FixtureRepo;
    use azusa_model::ProviderKind;

    #[test]
    fn resolves_git_keywords_in_a_repository() {
        let fixture = FixtureRepo::with_remote();
        let resolver = StateResolver::new(&fixture.path);
        let (state, reference) = resolver.resolve_ref("head").unwrap();
        assert_eq!(reference.provider, ProviderKind::Git);
        assert_eq!(reference.label, "head");
        assert_eq!(state.filesystem.files.len(), 2);

        let (upstream, upstream_ref) = resolver.resolve_ref("upstream").unwrap();
        assert_eq!(upstream_ref.spec, "upstream");
        assert_eq!(upstream.filesystem.files.len(), 2);
    }

    #[test]
    fn resolves_directory_through_fs_provider() {
        let fixture = FixtureRepo::new();
        let resolver = StateResolver::new(fixture.path.parent().unwrap());
        let (state, reference) = resolver
            .resolve_ref(fixture.path.to_str().unwrap())
            .unwrap();
        assert_eq!(reference.provider, ProviderKind::Filesystem);
        assert!(state.filesystem.files.contains_key("README.md"));
    }

    #[test]
    fn default_pair_uses_upstream_then_falls_back() {
        let with_remote = FixtureRepo::with_remote();
        let resolver = StateResolver::new(&with_remote.path);
        let (_, a_ref, _, b_ref, notes) = resolver.resolve_default_pair().unwrap();
        assert_eq!(a_ref.spec, "worktree");
        assert_eq!(b_ref.spec, "upstream");
        assert!(notes.is_empty(), "expected no notes, got {notes:?}");

        let plain = FixtureRepo::new();
        let resolver = StateResolver::new(&plain.path);
        let (_, _, _, b_ref, notes) = resolver.resolve_default_pair().unwrap();
        assert_eq!(b_ref.spec, "head");
        assert!(!notes.is_empty(), "fallback must be explained: {notes:?}");
    }

    #[test]
    fn unknown_revision_is_an_unresolved_state() {
        let fixture = FixtureRepo::new();
        let resolver = StateResolver::new(&fixture.path);
        let err = resolver.resolve("no-such-branch").unwrap_err();
        assert!(matches!(err, CoreError::UnresolvedState { .. }));
    }
}
