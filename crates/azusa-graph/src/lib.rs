// Copyright (c) 2026 The azusa Authors
// SPDX-License-Identifier: MIT

//! Azusa structural analysis.
//!
//! Derives structure, dependency and configuration facets from a captured
//! [`azusa_model::RepositoryState`]: Cargo workspace topology, dependency
//! declarations with lockfile-resolved versions, and configuration files of
//! interest. A pure function of the state — no I/O — which keeps the
//! analysis deterministic and testable.

pub mod behaviour;
pub mod config_files;
pub mod manifest;

pub use behaviour::behaviour_of;
pub use config_files::classify;
pub use manifest::{parse_lockfile, parse_manifest};

use std::collections::BTreeSet;

use azusa_model::{
    ConfigurationFile, ConfigurationState, DependencyState, LockfileInfo, OperationalState,
    OperationalUnit, RepositoryState, StructureState,
};

use crate::manifest::{lockfile_versions, modules_of, resolve_dependencies};

/// Name of this crate, used in provenance records.
pub const CRATE_NAME: &str = "azusa-graph";

/// Join path segments into a canonical repo-relative path: forward-slash
/// separated with no leading separator, on every host OS. Filesystem-state
/// keys use this canonical form, so lookups must build paths the same way.
#[must_use]
fn canonical_repo_path(segments: &[&str]) -> String {
    segments
        .iter()
        .filter(|segment| !(*segment).is_empty())
        .copied()
        .collect::<Vec<&str>>()
        .join("/")
}

/// Fill the structure, dependency and configuration facets of a captured
/// state from its filesystem listing and retained analysis text.
#[must_use]
pub fn enrich(mut state: RepositoryState) -> RepositoryState {
    let manifests: Vec<_> = state
        .captured_text
        .iter()
        .filter(|(path, _)| path.ends_with("Cargo.toml"))
        .filter_map(|(path, text)| manifest::parse_manifest(path, text))
        .collect();

    // L3 structure: modules + directory topology. Library/binary kinds are
    // refined against the filesystem (Cargo autodetects src/lib.rs and
    // src/main.rs without requiring explicit target tables).
    let directories: BTreeSet<String> = state
        .filesystem
        .files
        .keys()
        .filter_map(|path| path.rsplit_once('/').map(|(dir, _)| dir.to_owned()))
        .collect();
    let mut modules = modules_of(&manifests);
    for module in &mut modules {
        if module.kind == azusa_model::ModuleKind::Other {
            // Filesystem keys are canonical repo-relative paths: always
            // forward-slash, on every host OS. Joining with the host
            // separator (Path::join) would break lookups on Windows.
            let lib = canonical_repo_path(&[&module.path, "src", "lib.rs"]);
            let bin = canonical_repo_path(&[&module.path, "src", "main.rs"]);
            if state.filesystem.files.contains_key(&lib) {
                module.kind = azusa_model::ModuleKind::RustLibrary;
            } else if state.filesystem.files.contains_key(&bin) {
                module.kind = azusa_model::ModuleKind::RustBinary;
            }
        }
    }
    state.structure = StructureState {
        modules,
        directories,
    };

    // L4 dependencies: declared entries + lockfile summary and resolution.
    let mut lockfiles: Vec<LockfileInfo> = Vec::new();
    let mut resolved = std::collections::BTreeMap::default();
    if let Some(lock_text) = state.captured_text.get("Cargo.lock") {
        let digest = state
            .filesystem
            .files
            .get("Cargo.lock")
            .map_or_else(String::new, |entry| entry.hash.clone());
        lockfiles.push(parse_lockfile(lock_text, &digest));
        resolved = lockfile_versions(lock_text);
    }
    let declared = manifests
        .iter()
        .flat_map(|manifest| manifest.dependencies.clone())
        .collect();
    state.dependencies = DependencyState {
        entries: resolve_dependencies(declared, &resolved),
        lockfiles,
    };

    // L5 configuration: classify configuration files of interest by path.
    let files: Vec<ConfigurationFile> = state
        .filesystem
        .files
        .iter()
        .filter_map(|(path, entry)| {
            config_files::classify(path).map(|kind| ConfigurationFile {
                path: path.clone(),
                kind,
                digest: entry.hash.clone(),
            })
        })
        .collect();
    state.configuration = ConfigurationState {
        files: files.clone(),
    };

    // L9 operations: the deployment/service subset of the configuration
    // facet, framed as operational units.
    state.operations = OperationalState {
        units: files
            .iter()
            .filter(|file| file.kind.is_operational())
            .map(|file| OperationalUnit {
                path: file.path.clone(),
                kind: file.kind,
                digest: file.digest.clone(),
            })
            .collect(),
    };

    // L6–L7 inputs: the behavioural surface (tests, entry points, build
    // scripts, public items).
    state.behaviour = Some(behaviour::behaviour_of(&state));

    state
}

#[cfg(test)]
mod tests {
    use super::*;
    use azusa_model::{
        ConfigurationKind, DependencyKind, FileEntry, FileKind, FilesystemState, ModuleKind,
        content_hash,
    };

    fn entry(content: &str) -> FileEntry {
        FileEntry {
            hash: content_hash(content.as_bytes()),
            size: content.len() as u64,
            executable: false,
            kind: FileKind::File,
            lines: Some(1),
        }
    }

    fn workspace_state() -> RepositoryState {
        let files = [
            (
                "Cargo.toml",
                "[workspace]\nmembers = [\"crates/a\", \"crates/b\"]\n",
            ),
            (
                "Cargo.lock",
                "[[package]]\nname = \"serde\"\nversion = \"1.0.210\"\n",
            ),
            (
                "crates/a/Cargo.toml",
                "[package]\nname = \"a\"\nversion = \"0.1.0\"\n\n[dependencies]\nserde = \"1\"\n",
            ),
            ("crates/a/src/lib.rs", "pub fn a() {}\n"),
            (
                "crates/b/Cargo.toml",
                "[package]\nname = \"b\"\nversion = \"0.1.0\"\n\n[[bin]]\nname = \"b\"\n",
            ),
            ("crates/b/src/main.rs", "fn main() {}\n"),
            (".github/workflows/ci.yml", "on: [push]\n"),
        ];
        let mut filesystem = FilesystemState::default();
        let mut captured = std::collections::BTreeMap::new();
        for (path, text) in files {
            filesystem.files.insert(path.to_owned(), entry(text));
            captured.insert(path.to_owned(), text.to_owned());
        }
        let mut state = RepositoryState::empty(azusa_model::ProviderKind::Filesystem, "fixture");
        state.filesystem = filesystem;
        state.captured_text = captured;
        state
    }

    #[test]
    fn enrich_fills_structure_dependencies_configuration() {
        let state = enrich(workspace_state());

        // L3: three modules (workspace root + lib + bin) with correct kinds.
        let mut modules = state.structure.modules.clone();
        modules.sort_by(|a, b| a.name.cmp(&b.name));
        assert_eq!(modules.len(), 3);
        assert_eq!(modules[0].kind, ModuleKind::RustLibrary); // crate a
        assert_eq!(modules[1].kind, ModuleKind::RustBinary); // crate b
        assert_eq!(modules[2].kind, ModuleKind::RustWorkspace); // root
        assert!(state.structure.directories.contains("crates/a/src"));

        // L4: one declared dependency, resolved from the lockfile.
        assert_eq!(state.dependencies.entries.len(), 1);
        let dep = &state.dependencies.entries[0];
        assert_eq!(dep.name, "serde");
        assert_eq!(dep.kind, DependencyKind::Normal);
        assert_eq!(dep.resolved.as_deref(), Some("1.0.210"));
        assert_eq!(state.dependencies.lockfiles.len(), 1);
        assert_eq!(state.dependencies.lockfiles[0].package_count, 1);

        // L5: manifests + CI workflow classified.
        let kinds: Vec<(String, ConfigurationKind)> = state
            .configuration
            .files
            .iter()
            .map(|f| (f.path.clone(), f.kind))
            .collect();
        assert!(kinds.contains(&(
            ".github/workflows/ci.yml".to_owned(),
            ConfigurationKind::CiWorkflow
        )));
        assert!(kinds.contains(&("Cargo.toml".to_owned(), ConfigurationKind::CargoManifest)));
        assert_eq!(kinds.len(), 4); // 3 manifests + 1 workflow
    }

    #[test]
    fn enrich_is_idempotent() {
        let once = enrich(workspace_state());
        let twice = enrich(once.clone());
        assert_eq!(once.structure, twice.structure);
        assert_eq!(once.dependencies, twice.dependencies);
        assert_eq!(once.configuration, twice.configuration);
        assert_eq!(once.behaviour, twice.behaviour);
    }

    #[test]
    fn enrich_fills_behaviour_facet() {
        let state = enrich(workspace_state());
        let behaviour = state.behaviour.expect("behaviour facet");
        assert!(
            behaviour
                .entry_points
                .contains(&"crates/b/src/main.rs".to_owned())
        );
        assert_eq!(
            behaviour.public_items_per_file.get("crates/a/src/lib.rs"),
            Some(&1)
        );
    }

    #[test]
    fn enrich_fills_operations_facet_from_deploy_configs() {
        let mut state = workspace_state();
        let manifest = "apiVersion: v1\nkind: Service\n";
        state
            .filesystem
            .files
            .insert("k8s/api.yaml".to_owned(), entry(manifest));
        state
            .captured_text
            .insert("k8s/api.yaml".to_owned(), manifest.to_owned());
        state
            .filesystem
            .files
            .insert("README.md".to_owned(), entry("docs\n"));

        let state = enrich(state);
        let units = &state.operations.units;
        let paths: Vec<&str> = units.iter().map(|unit| unit.path.as_str()).collect();
        assert!(paths.contains(&".github/workflows/ci.yml"));
        assert!(paths.contains(&"k8s/api.yaml"));
        assert!(
            !paths.contains(&"Cargo.toml"),
            "manifests are not operational"
        );
        assert!(!paths.contains(&"README.md"));
        let k8s = units
            .iter()
            .find(|u| u.path == "k8s/api.yaml")
            .expect("unit");
        assert_eq!(k8s.kind, ConfigurationKind::Kubernetes);
    }
}
