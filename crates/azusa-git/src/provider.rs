// Copyright (c) 2026 The azusa Authors
// SPDX-License-Identifier: MIT

//! The Git state provider.
//!
//! Resolves [`GitSelector`]s against a repository and captures them as
//! [`azusa_model::RepositoryState`]s: worktree (with uncommitted summary),
//! index, HEAD, upstream, remote default branch, or any revision. Tree
//! states are read from the object database; the worktree reuses the
//! `azusa-fs` walker so ignore rules behave identically across providers.

use std::collections::BTreeMap;
use std::path::Path;

use azusa_fs::{HashCache, SnapshotOptions, snapshot as fs_snapshot};
use azusa_model::{
    Ancestry, ConfigurationState, DependencyState, FileEntry, FileKind, FilesystemState,
    ProviderKind, Relation, RepositoryState, StateIdentity, StructureState, UncommittedSummary,
    VcsIdentity, content_hash, count_lines, fileset_digest, looks_like_text,
};
use git2::{Repository, Tree};

use crate::GitError;
use crate::selector::GitSelector;

/// Name under which this provider identifies itself.
pub const PROVIDER_NAME: &str = "git";

/// The Git state provider over one repository.
pub struct GitProvider {
    repo: Repository,
    root: std::path::PathBuf,
    options: SnapshotOptions,
    cache: HashCache,
}

// The repository handle and cache are opaque log noise; only the root
// identifies a provider instance.
#[allow(clippy::missing_fields_in_debug)]
impl std::fmt::Debug for GitProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GitProvider")
            .field("root", &self.root)
            .finish()
    }
}

impl GitProvider {
    /// Open the repository containing `root` (searching parent directories).
    ///
    /// # Errors
    ///
    /// Returns [`GitError::NotARepository`] when the discovered repository is
    /// bare, and [`GitError::Git`] for libgit2 failures.
    pub fn discover(root: impl AsRef<Path>) -> Result<Self, GitError> {
        let repo = Repository::discover(root.as_ref())?;
        let workdir = repo
            .workdir()
            .ok_or_else(|| GitError::NotARepository("bare repository has no worktree".into()))?
            .to_path_buf();
        Ok(Self {
            repo,
            root: workdir,
            options: SnapshotOptions::default(),
            cache: HashCache::default(),
        })
    }

    /// Open the repository exactly at `root`.
    ///
    /// # Errors
    ///
    /// Returns [`GitError::NotARepository`] when the repository is bare, and
    /// [`GitError::Git`] for libgit2 failures.
    pub fn open(root: impl AsRef<Path>) -> Result<Self, GitError> {
        let repo = Repository::open(root.as_ref())?;
        let workdir = repo
            .workdir()
            .ok_or_else(|| GitError::NotARepository("bare repository has no worktree".into()))?
            .to_path_buf();
        Ok(Self {
            repo,
            root: workdir,
            options: SnapshotOptions::default(),
            cache: HashCache::default(),
        })
    }

    /// The repository worktree root.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Whether a Git repository exists at or below `root`.
    #[must_use]
    pub fn is_repository(root: impl AsRef<Path>) -> bool {
        Repository::discover(root).is_ok()
    }

    /// Capture the state named by `selector`.
    ///
    /// # Errors
    ///
    /// Returns [`GitError::UnresolvedSelector`] when the selector cannot be
    /// resolved against this repository, and [`GitError::Git`] for libgit2 or
    /// worktree-walk failures.
    pub fn snapshot(&mut self, selector: &GitSelector) -> Result<RepositoryState, GitError> {
        let vcs = self.vcs_identity();
        let (filesystem, captured_text, uncommitted) = match selector {
            GitSelector::Worktree => {
                let outcome =
                    fs_snapshot(&self.root, &self.options, &mut self.cache).map_err(|e| {
                        GitError::Git(git2::Error::from_str(&format!("worktree walk failed: {e}")))
                    })?;
                (
                    outcome.filesystem,
                    outcome.captured_text,
                    Some(self.uncommitted_summary()),
                )
            }
            GitSelector::Index => {
                let (filesystem, captured) = self.index_filesystem()?;
                (filesystem, captured, None)
            }
            GitSelector::Head
            | GitSelector::Upstream
            | GitSelector::RemoteDefault
            | GitSelector::Rev(_) => {
                let commit = self.resolve_commit(selector)?;
                let (filesystem, captured) = self.tree_filesystem(&commit.tree()?)?;
                (filesystem, captured, None)
            }
        };

        let ancestry = self.ancestry(selector, uncommitted);
        let identity = StateIdentity {
            provider: ProviderKind::Git,
            label: selector.label(),
            root: Some(self.root.display().to_string()),
            digest: filesystem.digest.clone(),
            file_count: filesystem.files.len() as u64,
            total_bytes: filesystem.total_bytes,
            vcs: Some(vcs),
        };
        Ok(RepositoryState {
            identity,
            ancestry,
            filesystem,
            structure: StructureState::default(),
            dependencies: DependencyState::default(),
            configuration: ConfigurationState::default(),
            operations: azusa_model::OperationalState::default(),
            behaviour: None,
            captured_text,
        })
    }

    /// Resolve a commit-based selector to its commit id.
    fn resolve_commit(&self, selector: &GitSelector) -> Result<git2::Commit<'_>, GitError> {
        let spec = match selector {
            GitSelector::Head => "HEAD".to_owned(),
            GitSelector::Upstream => {
                self.upstream_ref()
                    .ok_or_else(|| GitError::UnresolvedSelector {
                        selector: selector.label(),
                        reason: "HEAD has no upstream tracking branch".into(),
                    })?
            }
            GitSelector::RemoteDefault => self.remote_default_ref()?,
            GitSelector::Rev(rev) => rev.clone(),
            other => {
                return Err(GitError::UnresolvedSelector {
                    selector: other.label(),
                    reason: "not a commit-based selector".into(),
                });
            }
        };
        let object =
            self.repo
                .revparse_single(&spec)
                .map_err(|e| GitError::UnresolvedSelector {
                    selector: selector.label(),
                    reason: e.message().to_owned(),
                })?;
        let commit = object
            .peel_to_commit()
            .map_err(|_| GitError::UnresolvedSelector {
                selector: selector.label(),
                reason: format!("`{spec}` does not point at a commit"),
            })?;
        Ok(commit)
    }

    /// The upstream tracking ref of HEAD, if configured and resolvable.
    #[must_use]
    pub fn upstream_ref(&self) -> Option<String> {
        let branch = self.current_branch()?;
        let local = self
            .repo
            .find_branch(&branch, git2::BranchType::Local)
            .ok()?;
        let upstream = local.upstream().ok()?;
        upstream.name().ok()?.map(std::borrow::ToOwned::to_owned)
    }

    /// The current branch short name, if HEAD is attached.
    #[must_use]
    pub fn current_branch(&self) -> Option<String> {
        self.repo
            .head()
            .ok()
            .and_then(|h| h.shorthand().map(std::borrow::ToOwned::to_owned))
            .or_else(|| {
                self.repo.find_reference("HEAD").ok().and_then(|r| {
                    r.symbolic_target()
                        .and_then(|t| t.strip_prefix("refs/heads/"))
                        .map(std::borrow::ToOwned::to_owned)
                })
            })
    }

    /// The remote default branch ref, preferring `origin/HEAD` then
    /// `origin/main` then `origin/master`.
    fn remote_default_ref(&self) -> Result<String, GitError> {
        let remote_name = self.remote_name();
        let candidates = [
            format!("{remote_name}/HEAD"),
            format!("{remote_name}/main"),
            format!("{remote_name}/master"),
        ];
        for candidate in &candidates {
            if self.repo.revparse_single(candidate).is_ok() {
                return Ok(candidate.clone());
            }
        }
        Err(GitError::UnresolvedSelector {
            selector: "remote".into(),
            reason: format!("no remote-tracking branch among {}", candidates.join(", ")),
        })
    }

    /// The primary remote's name (usually `origin`).
    #[must_use]
    pub fn remote_name(&self) -> String {
        self.repo
            .remotes()
            .ok()
            .and_then(|names| {
                names
                    .iter()
                    .flatten()
                    .next()
                    .map(std::borrow::ToOwned::to_owned)
            })
            .unwrap_or_else(|| "origin".to_owned())
    }

    fn vcs_identity(&self) -> VcsIdentity {
        let head_oid = self
            .repo
            .head()
            .ok()
            .and_then(|h| h.target())
            .map(|o| o.to_string());
        let detached = self.repo.head_detached().unwrap_or(false);
        let branch = if detached {
            None
        } else {
            self.current_branch()
        };
        let remote_urls: Vec<String> = self
            .repo
            .remotes()
            .map(|remotes| {
                remotes
                    .iter()
                    .flatten()
                    .filter_map(|name| self.repo.find_remote(name).ok())
                    .filter_map(|remote| remote.url().map(std::borrow::ToOwned::to_owned))
                    .collect()
            })
            .unwrap_or_default();
        VcsIdentity {
            head_oid,
            branch,
            detached,
            upstream: self.upstream_ref(),
            remote_urls,
            shallow: self.root.join(".git/shallow").exists()
                || self.root.join(".git").join("shallow").exists(),
        }
    }

    /// Uncommitted summary of the worktree, from libgit2 status flags.
    #[must_use]
    pub fn uncommitted_summary(&self) -> UncommittedSummary {
        let mut stats = UncommittedSummary::default();
        let Ok(statuses) = self.repo.statuses(None) else {
            return stats;
        };
        for entry in statuses.iter() {
            match entry.status() {
                s if s.is_index_new()
                    || s.is_index_deleted()
                    || s.is_index_modified()
                    || s.is_index_renamed()
                    || s.is_index_typechange() =>
                {
                    stats.staged_files += 1;
                }
                _ => {}
            }
            match entry.status() {
                s if s.is_wt_new() => stats.untracked_files += 1,
                s if s.is_wt_deleted()
                    || s.is_wt_modified()
                    || s.is_wt_renamed()
                    || s.is_wt_typechange() =>
                {
                    stats.unstaged_files += 1;
                }
                _ => {}
            }
        }
        stats
    }

    fn ancestry(
        &self,
        selector: &GitSelector,
        uncommitted: Option<UncommittedSummary>,
    ) -> Ancestry {
        let mut ancestry = Ancestry {
            uncommitted,
            ..Ancestry::default()
        };
        // Ancestry is computed relative to the upstream for local states.
        let local = matches!(
            selector,
            GitSelector::Worktree | GitSelector::Head | GitSelector::Index
        );
        if !local {
            return ancestry;
        }
        let Some(upstream_ref) = self.upstream_ref() else {
            ancestry.relation = Relation::Unknown;
            return ancestry;
        };
        let Ok(upstream_target) = self.repo.revparse_single(&upstream_ref) else {
            ancestry.relation = Relation::Unknown;
            return ancestry;
        };
        let Ok(upstream_commit) = upstream_target.peel_to_commit() else {
            ancestry.relation = Relation::Unknown;
            return ancestry;
        };
        let Ok(local_commit) = self.resolve_commit(&GitSelector::Head) else {
            ancestry.relation = Relation::Unknown;
            return ancestry;
        };
        if let Ok(base) = self
            .repo
            .merge_base(local_commit.id(), upstream_commit.id())
        {
            ancestry.merge_base = Some(base.to_string());
            ancestry.relation = Relation::Related;
        } else {
            ancestry.relation = Relation::Unrelated;
            return ancestry;
        }
        if let Ok((ahead, behind)) = self
            .repo
            .graph_ahead_behind(local_commit.id(), upstream_commit.id())
        {
            ancestry.ahead = Some(ahead as u64);
            ancestry.behind = Some(behind as u64);
        }
        ancestry
    }

    fn tree_filesystem(
        &self,
        tree: &Tree<'_>,
    ) -> Result<(FilesystemState, BTreeMap<String, String>), GitError> {
        let mut files: BTreeMap<String, FileEntry> = BTreeMap::new();
        let mut captured_text: BTreeMap<String, String> = BTreeMap::new();
        self.walk_tree(tree, "", &mut files, &mut captured_text)?;
        let total_bytes = files.values().map(|e| e.size).sum();
        let digest = fileset_digest(files.iter().map(|(path, entry)| {
            (
                entry.kind.mode_string(entry.executable).to_owned(),
                entry.hash.clone(),
                path.clone(),
            )
        }));
        Ok((
            FilesystemState {
                files,
                total_bytes,
                digest,
            },
            captured_text,
        ))
    }

    fn walk_tree(
        &self,
        tree: &Tree<'_>,
        prefix: &str,
        files: &mut BTreeMap<String, FileEntry>,
        captured_text: &mut BTreeMap<String, String>,
    ) -> Result<(), GitError> {
        for entry in tree {
            let name = String::from_utf8_lossy(entry.name_bytes()).into_owned();
            let path = if prefix.is_empty() {
                name
            } else {
                format!("{prefix}/{name}")
            };
            match entry.kind() {
                Some(git2::ObjectType::Tree) => {
                    let subtree = entry.to_object(&self.repo)?.peel_to_tree()?;
                    self.walk_tree(&subtree, &path, files, captured_text)?;
                }
                Some(git2::ObjectType::Blob) => {
                    let blob = entry.to_object(&self.repo)?.peel_to_blob()?;
                    let content = blob.content();
                    let filemode = entry.filemode();
                    let kind = if filemode == 0o120_000 {
                        FileKind::Symlink
                    } else {
                        FileKind::File
                    };
                    let lines = looks_like_text(content).then(|| count_lines(content));
                    if azusa_model::is_analysis_file(&path)
                        && content.len() <= azusa_model::CAPTURED_TEXT_MAX_BYTES
                    {
                        if let Ok(text) = std::str::from_utf8(content) {
                            captured_text.insert(path.clone(), text.to_owned());
                        }
                    }
                    files.insert(
                        path,
                        FileEntry {
                            hash: content_hash(content),
                            size: blob.size() as u64,
                            executable: filemode == 0o100_755,
                            kind,
                            lines,
                        },
                    );
                }
                Some(git2::ObjectType::Commit) => {
                    // Submodule pointer: identity is the commit id.
                    files.insert(
                        path,
                        FileEntry {
                            hash: content_hash(entry.id().as_bytes()),
                            size: 0,
                            executable: false,
                            kind: FileKind::Submodule,
                            lines: None,
                        },
                    );
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn index_filesystem(&self) -> Result<(FilesystemState, BTreeMap<String, String>), GitError> {
        let index = self.repo.index()?;
        let mut files: BTreeMap<String, FileEntry> = BTreeMap::new();
        let mut captured_text: BTreeMap<String, String> = BTreeMap::new();
        for entry in index.iter() {
            let path = String::from_utf8_lossy(&entry.path).into_owned();
            let kind = if entry.mode == 0o120_000 {
                FileKind::Symlink
            } else {
                FileKind::File
            };
            let (hash, size, lines) = match self.repo.find_blob(entry.id) {
                Ok(blob) => {
                    let content = blob.content();
                    let lines = looks_like_text(content).then(|| count_lines(content));
                    if azusa_model::is_analysis_file(&path)
                        && content.len() <= azusa_model::CAPTURED_TEXT_MAX_BYTES
                    {
                        if let Ok(text) = std::str::from_utf8(content) {
                            captured_text.insert(path.clone(), text.to_owned());
                        }
                    }
                    (content_hash(content), blob.size() as u64, lines)
                }
                Err(_) => (content_hash(entry.id.as_bytes()), 0, None),
            };
            files.insert(
                path,
                FileEntry {
                    hash,
                    size,
                    executable: entry.mode == 0o100_755,
                    kind,
                    lines,
                },
            );
        }
        let total_bytes = files.values().map(|e| e.size).sum();
        let digest = fileset_digest(files.iter().map(|(path, entry)| {
            (
                entry.kind.mode_string(entry.executable).to_owned(),
                entry.hash.clone(),
                path.clone(),
            )
        }));
        Ok((
            FilesystemState {
                files,
                total_bytes,
                digest,
            },
            captured_text,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::FixtureRepo;

    #[test]
    fn snapshots_head_worktree_and_detects_uncommitted() {
        let fixture = FixtureRepo::new();
        let mut provider = GitProvider::open(&fixture.path).unwrap();

        let head = provider.snapshot(&GitSelector::Head).unwrap();
        assert_eq!(head.filesystem.files.len(), 2);
        assert!(head.ancestry.uncommitted.is_none());
        assert_eq!(
            head.identity.vcs.as_ref().unwrap().branch.as_deref(),
            Some("main")
        );

        // Modify one tracked file and add two untracked ones.
        fixture.write("README.md", "# fixture (edited)\n");
        fixture.write("src/lib.rs", "pub fn f() {}\n");
        fixture.write("notes.txt", "uncommitted\n");

        let worktree = provider.snapshot(&GitSelector::Worktree).unwrap();
        assert_eq!(worktree.filesystem.files.len(), 4);
        let uncommitted = worktree.ancestry.uncommitted.unwrap();
        assert_eq!(uncommitted.unstaged_files, 1, "got {uncommitted:?}");
        assert_eq!(uncommitted.untracked_files, 2, "got {uncommitted:?}");
        assert!(worktree.filesystem.files.contains_key("notes.txt"));
        assert_ne!(worktree.filesystem.digest, head.filesystem.digest);
    }

    #[test]
    fn index_snapshot_reflects_staging() {
        let fixture = FixtureRepo::new();
        fixture.write("staged.txt", "staged content\n");
        fixture.stage("staged.txt");

        let mut provider = GitProvider::open(&fixture.path).unwrap();
        let index_state = provider.snapshot(&GitSelector::Index).unwrap();
        assert!(index_state.filesystem.files.contains_key("staged.txt"));
        let head_state = provider.snapshot(&GitSelector::Head).unwrap();
        assert!(!head_state.filesystem.files.contains_key("staged.txt"));
    }

    #[test]
    fn ahead_behind_against_remote_tracking_ref() {
        let fixture = FixtureRepo::with_remote();
        // Local gains a commit the remote does not have.
        fixture.commit_file("extra.rs", "pub fn extra() {}\n", "feat: extra");

        let mut provider = GitProvider::open(&fixture.path).unwrap();
        let head = provider.snapshot(&GitSelector::Head).unwrap();
        assert_eq!(head.ancestry.relation, Relation::Related);
        assert_eq!(head.ancestry.ahead, Some(1));
        assert_eq!(head.ancestry.behind, Some(0));
        assert!(head.ancestry.merge_base.is_some());

        let upstream = provider.snapshot(&GitSelector::Upstream).unwrap();
        assert_eq!(upstream.filesystem.files.len(), 2);
    }

    #[test]
    fn tree_snapshot_matches_fs_snapshot_of_clean_worktree() {
        let fixture = FixtureRepo::new();
        let mut provider = GitProvider::open(&fixture.path).unwrap();
        let head = provider.snapshot(&GitSelector::Head).unwrap();
        let worktree = provider.snapshot(&GitSelector::Worktree).unwrap();
        assert_eq!(head.filesystem.digest, worktree.filesystem.digest);
        assert_eq!(
            head.filesystem.files.keys().collect::<Vec<_>>(),
            worktree.filesystem.files.keys().collect::<Vec<_>>()
        );
    }
}
