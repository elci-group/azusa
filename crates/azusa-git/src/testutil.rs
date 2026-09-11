//! Test fixtures: minimal real Git repositories built with libgit2.
//!
//! Fixture methods return the commit id for convenience; callers routinely
//! ignore it, so `must_use_candidate` is silenced for this module.
#![allow(
    clippy::must_use_candidate,
    clippy::missing_panics_doc,
    clippy::missing_errors_doc
)]

use std::path::{Path, PathBuf};

use git2::{IndexAddOption, Repository, Signature};

fn walk_tree_paths(tree: &git2::Tree, out: &mut std::collections::BTreeSet<String>) {
    tree.walk(
        git2::TreeWalkMode::PreOrder,
        &mut |root: &str, entry: &git2::TreeEntry| {
            if let Some(name) = entry.name() {
                let path = if root.is_empty() {
                    name.to_owned()
                } else if root.ends_with('/') {
                    format!("{root}{name}")
                } else {
                    format!("{root}/{name}")
                };
                out.insert(path);
            }
            git2::TreeWalkResult::Ok
        },
    )
    .ok();
}

/// A temporary Git repository with helper methods for building history.
pub struct FixtureRepo {
    /// Repository worktree path.
    pub path: PathBuf,
    repo: Repository,
    guard: tempfile::TempDir,
}

impl FixtureRepo {
    /// The tempdir keeping this fixture alive.
    #[must_use]
    pub fn guard_path(&self) -> &Path {
        self.guard.path()
    }
}

impl Default for FixtureRepo {
    fn default() -> Self {
        Self::new()
    }
}

impl FixtureRepo {
    /// A repository with one commit containing `src/main.rs` and `README.md`.
    pub fn new() -> Self {
        let guard = tempfile::tempdir().expect("tempdir");
        let path = guard.path().join("repo");
        std::fs::create_dir_all(&path).expect("mkdir");
        let repo = Repository::init(&path).expect("init");
        // Guarantee the branch is `main` regardless of init.defaultBranch.
        repo.set_head("refs/heads/main").expect("set head");
        let mut config = repo.config().expect("config");
        config
            .set_str("user.name", "Azusa Test")
            .expect("user.name");
        config
            .set_str("user.email", "azusa@test.local")
            .expect("user.email");

        let fixture = Self { path, repo, guard };
        fixture.write("src/main.rs", "fn main() {}\n");
        fixture.write("README.md", "# fixture\n");
        fixture.commit_all("chore: initial state");
        fixture
    }

    /// A repository whose `main` tracks `origin/main` from a sibling bare
    /// remote holding the initial commit.
    pub fn with_remote() -> Self {
        let fixture = Self::new();
        let bare_path = fixture.guard_path().join("origin.git");
        let bare = Repository::init_bare(&bare_path).expect("init bare");

        let head = fixture.repo.head().expect("head").target().expect("oid");
        fixture
            .repo
            .remote("origin", bare.path().to_str().expect("utf8 path"))
            .expect("remote add");
        let mut remote = fixture.repo.find_remote("origin").expect("remote");
        remote
            .push(&["refs/heads/main:refs/heads/main"], None)
            .expect("push");

        // Materialise the remote-tracking ref locally.
        fixture
            .repo
            .reference("refs/remotes/origin/main", head, true, "fixture")
            .expect("remote-tracking ref");
        drop(remote);
        {
            let mut branch = fixture
                .repo
                .find_branch("main", git2::BranchType::Local)
                .expect("branch");
            branch.set_upstream(Some("origin/main")).expect("upstream");
        }
        drop(bare);
        fixture
    }

    /// Write a file (creating parent directories).
    pub fn write(&self, rel: &str, content: &str) {
        let target = self.path.join(rel);
        std::fs::create_dir_all(target.parent().expect("parent")).expect("mkdir");
        std::fs::write(target, content).expect("write");
    }

    /// Stage everything and commit.
    pub fn commit_all(&self, message: &str) -> git2::Oid {
        let mut index = self.repo.index().expect("index");
        index
            .add_all(["*"].iter(), IndexAddOption::DEFAULT, None)
            .expect("add all");
        index.write().expect("index write");
        let tree_oid = index.write_tree().expect("tree");
        let tree = self.repo.find_tree(tree_oid).expect("find tree");
        let sig = Signature::now("Azusa Test", "azusa@test.local").expect("signature");
        let parent = self.repo.head().ok().and_then(|h| h.peel_to_commit().ok());
        let parents: Vec<git2::Commit<'_>> = parent.into_iter().collect();
        let parent_refs: Vec<&git2::Commit<'_>> = parents.iter().collect();
        self.repo
            .commit(Some("HEAD"), &sig, &sig, message, &tree, &parent_refs)
            .expect("commit")
    }

    /// Point the remote-tracking ref at the current HEAD (simulating a
    /// pushed, up-to-date upstream).
    pub fn sync_remote(&self) {
        let head = self.repo.head().expect("head").target().expect("oid");
        self.repo
            .reference("refs/remotes/origin/main", head, true, "fixture")
            .expect("sync remote");
    }

    /// Advance the remote-tracking ref by committing several files on top of
    /// the current remote state.
    pub fn advance_remote_files(&self, files: &[(&str, &str)], message: &str) -> git2::Oid {
        let remote_ref = self
            .repo
            .find_reference("refs/remotes/origin/main")
            .expect("remote ref");
        let base = remote_ref.peel_to_commit().expect("remote commit");
        for (rel, content) in files {
            self.write(rel, content);
        }
        let mut index = self.repo.index().expect("index");
        for (rel, _) in files {
            index.add_path(Path::new(rel)).expect("add");
        }
        index.write().expect("index write");
        let tree_oid = index.write_tree().expect("tree");
        let tree = self.repo.find_tree(tree_oid).expect("find tree");
        let sig = Signature::now("Azusa Test", "azusa@test.local").expect("signature");
        let oid = self
            .repo
            .commit(None, &sig, &sig, message, &tree, &[&base])
            .expect("commit");
        self.repo
            .reference("refs/remotes/origin/main", oid, true, "fixture")
            .expect("advance remote");
        // Restore the worktree to HEAD (local state), removing leftovers.
        let mut index = self.repo.index().expect("index");
        let head_tree = self
            .repo
            .head()
            .expect("head")
            .peel_to_tree()
            .expect("tree");
        index.read_tree(&head_tree).expect("read tree");
        index.write().expect("index write");
        self.repo
            .checkout_head(Some(git2::build::CheckoutBuilder::new().force()))
            .ok();
        // Remove only files that exist solely in the remote commit; files
        // tracked by HEAD were already restored by the checkout.
        let head_tree = self
            .repo
            .head()
            .expect("head")
            .peel_to_tree()
            .expect("tree");
        let mut head_paths = std::collections::BTreeSet::new();
        walk_tree_paths(&head_tree, &mut head_paths);
        for (rel, _) in files {
            if !head_paths.contains(*rel) {
                std::fs::remove_file(self.path.join(rel)).ok();
            }
        }
        oid
    }

    /// Stage a single path without committing.
    pub fn stage(&self, rel: &str) {
        let mut index = self.repo.index().expect("index");
        index.add_path(Path::new(rel)).expect("add");
        index.write().expect("index write");
    }

    /// Write one file, stage it, and commit it.
    pub fn commit_file(&self, rel: &str, content: &str, message: &str) -> git2::Oid {
        self.write(rel, content);
        self.commit_all(message)
    }

    /// Advance the remote-tracking ref to a new commit, simulating fetched
    /// upstream evolution.
    pub fn advance_remote(&self, rel: &str, content: &str, message: &str) -> git2::Oid {
        // Commit on a temp branch from the current remote-tracking commit.
        let remote_ref = self
            .repo
            .find_reference("refs/remotes/origin/main")
            .expect("remote ref");
        let base = remote_ref.peel_to_commit().expect("remote commit");
        self.write(rel, content);

        let mut index = self.repo.index().expect("index");
        index.add_path(Path::new(rel)).expect("add");
        index.write().expect("index write");
        let tree_oid = index.write_tree().expect("tree");
        let tree = self.repo.find_tree(tree_oid).expect("find tree");
        let sig = Signature::now("Azusa Test", "azusa@test.local").expect("signature");
        let oid = self
            .repo
            .commit(None, &sig, &sig, message, &tree, &[&base])
            .expect("commit");
        self.repo
            .reference("refs/remotes/origin/main", oid, true, "fixture")
            .expect("advance remote");
        // Restore the index to HEAD so the worktree stays clean for HEAD-based
        // checks (the commit was built on the remote base).
        let mut index = self.repo.index().expect("index");
        let head_tree = self
            .repo
            .head()
            .expect("head")
            .peel_to_tree()
            .expect("tree");
        index.read_tree(&head_tree).expect("read tree");
        index.write().expect("index write");
        self.repo
            .checkout_head(Some(git2::build::CheckoutBuilder::new().force()))
            .ok();
        // The file we wrote for the remote commit is untracked from HEAD's
        // perspective; remove it so the worktree reflects only local state.
        std::fs::remove_file(self.path.join(rel)).ok();
        oid
    }
}
