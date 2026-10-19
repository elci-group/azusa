// Copyright (c) 2026 The azusa Authors
// SPDX-License-Identifier: MIT

//! Cargo manifest discovery and parsing.
//!
//! Parses every `Cargo.toml` retained in a state's `captured_text`,
//! classifies it (workspace root, library package, binary package) and
//! extracts declared dependencies with lockfile-resolved versions.

use std::collections::BTreeMap;

use azusa_model::{DependencyEntry, DependencyKind, LockfileInfo, ModuleInfo, ModuleKind};

/// One discovered manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveredManifest {
    /// Repository-relative path of the `Cargo.toml`.
    pub path: String,
    /// Package name (or workspace label).
    pub name: String,
    /// Whether this manifest declares a `[workspace]`.
    pub is_workspace: bool,
    /// Whether the package has an explicit `[lib]` target.
    pub has_lib: bool,
    /// Explicit `[[bin]]` target names.
    pub bins: Vec<String>,
    /// Declared dependencies.
    pub dependencies: Vec<DependencyEntry>,
}

/// Parse one manifest from its TOML text.
///
/// Workspace-root manifests without a `[package]` table are accepted; their
/// name falls back to the containing directory.
#[must_use]
pub fn parse_manifest(path: &str, text: &str) -> Option<DiscoveredManifest> {
    let value: toml::Value = toml::from_str(text).ok()?;
    let package = value.get("package");
    let name = package
        .and_then(|p| p.get("name"))
        .and_then(toml::Value::as_str)
        .map_or_else(|| fallback_name(path), std::borrow::ToOwned::to_owned);

    let is_workspace = value.get("workspace").is_some();
    let has_lib = value.get("lib").is_some();

    let mut bins = Vec::new();
    if let Some(bin_array) = value.get("bin").and_then(toml::Value::as_array) {
        for bin in bin_array {
            if let Some(bin_name) = bin.get("name").and_then(toml::Value::as_str) {
                bins.push(bin_name.to_owned());
            }
        }
    }

    let mut dependencies = Vec::new();
    for (table, kind) in [
        ("dependencies", DependencyKind::Normal),
        ("dev-dependencies", DependencyKind::Dev),
        ("build-dependencies", DependencyKind::Build),
    ] {
        let Some(deps) = value.get(table).and_then(toml::Value::as_table) else {
            continue;
        };
        for (dep_name, dep_value) in deps {
            dependencies.push(DependencyEntry {
                manifest: path.to_owned(),
                name: dep_name.clone(),
                requirement: requirement_of(dep_value),
                kind,
                resolved: None,
            });
        }
    }
    // Deterministic order.
    dependencies.sort_by(|a, b| (&a.name, a.kind.as_str()).cmp(&(&b.name, b.kind.as_str())));

    Some(DiscoveredManifest {
        path: path.to_owned(),
        name,
        is_workspace,
        has_lib,
        bins,
        dependencies,
    })
}

fn fallback_name(path: &str) -> String {
    std::path::Path::new(path)
        .parent()
        .and_then(|parent| parent.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("workspace")
        .to_owned()
}

fn requirement_of(value: &toml::Value) -> String {
    match value {
        toml::Value::String(s) => s.clone(),
        toml::Value::Table(table) => table
            .get("version")
            .and_then(toml::Value::as_str)
            .map(std::borrow::ToOwned::to_owned)
            .or_else(|| {
                table
                    .get("git")
                    .and_then(toml::Value::as_str)
                    .map(std::borrow::ToOwned::to_owned)
            })
            .or_else(|| {
                table
                    .get("path")
                    .and_then(toml::Value::as_str)
                    .map(std::borrow::ToOwned::to_owned)
            })
            .unwrap_or_default(),
        _ => String::new(),
    }
}

/// Summarise a `Cargo.lock` (v3 format) into package count and digest.
#[must_use]
pub fn parse_lockfile(text: &str, digest: &str) -> LockfileInfo {
    LockfileInfo {
        path: "Cargo.lock".to_owned(),
        digest: digest.to_owned(),
        package_count: text.lines().filter(|l| l.starts_with("name = ")).count() as u64,
    }
}

/// Parse `Cargo.lock` package names → resolved versions.
#[must_use]
pub fn lockfile_versions(text: &str) -> BTreeMap<String, String> {
    let mut versions = BTreeMap::new();
    let mut current_name: Option<String> = None;
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(name) = trimmed.strip_prefix("name = ") {
            current_name = Some(name.trim_matches('"').to_owned());
        } else if let Some(version) = trimmed.strip_prefix("version = ") {
            if let Some(name) = current_name.take() {
                versions.insert(name, version.trim_matches('"').to_owned());
            }
        }
    }
    versions
}

/// Convert discovered manifests into model [`ModuleInfo`]s.
#[must_use]
pub fn modules_of(manifests: &[DiscoveredManifest]) -> Vec<ModuleInfo> {
    manifests
        .iter()
        .map(|manifest| {
            let kind = if manifest.is_workspace {
                ModuleKind::RustWorkspace
            } else if manifest.has_lib {
                ModuleKind::RustLibrary
            } else if !manifest.bins.is_empty() {
                ModuleKind::RustBinary
            } else {
                ModuleKind::Other
            };
            let dir = manifest
                .path
                .rsplit_once('/')
                .map_or_else(String::new, |(dir, _)| dir.to_owned());
            ModuleInfo {
                name: manifest.name.clone(),
                path: dir,
                kind,
            }
        })
        .collect()
}

/// Resolve dependency entries against lockfile versions (name → version).
#[must_use]
pub fn resolve_dependencies(
    mut entries: Vec<DependencyEntry>,
    versions: &BTreeMap<String, String>,
) -> Vec<DependencyEntry> {
    for entry in &mut entries {
        if let Some(version) = versions.get(&entry.name) {
            entry.resolved = Some(version.clone());
        }
    }
    entries
}

#[cfg(test)]
mod tests {
    use super::*;

    const MANIFEST: &str = r#"
[package]
name = "azusa-demo"
version = "0.1.0"

[dependencies]
serde = { version = "1", features = ["derive"] }
toml = "0.9"

[dev-dependencies]
tempfile = "3"

[[bin]]
name = "azusa-demo"
path = "src/main.rs"
"#;

    #[test]
    fn parses_package_targets_and_dependencies() {
        let manifest = parse_manifest("crates/demo/Cargo.toml", MANIFEST).expect("parses");
        assert_eq!(manifest.name, "azusa-demo");
        assert!(!manifest.is_workspace);
        assert_eq!(manifest.bins, vec!["azusa-demo"]);
        assert_eq!(manifest.dependencies.len(), 3);
        let serde_dep = manifest
            .dependencies
            .iter()
            .find(|d| d.name == "serde")
            .expect("serde");
        assert_eq!(serde_dep.requirement, "1");
        assert_eq!(serde_dep.kind, DependencyKind::Normal);
        assert!(
            manifest
                .dependencies
                .iter()
                .any(|d| d.name == "tempfile" && d.kind == DependencyKind::Dev)
        );
    }

    #[test]
    fn workspace_manifest_is_classified() {
        let workspace = r#"
[workspace]
members = ["crates/a"]
"#;
        let manifest = parse_manifest("Cargo.toml", workspace).expect("parses");
        assert!(manifest.is_workspace);
        assert_eq!(manifest.name, "workspace");
        let modules = modules_of(std::slice::from_ref(&manifest));
        assert_eq!(modules[0].kind, ModuleKind::RustWorkspace);
        assert_eq!(modules[0].path, "");
    }

    #[test]
    fn lib_package_classification() {
        let lib_manifest = r#"
[package]
name = "azusa-lib"
version = "0.1.0"

[lib]
path = "src/lib.rs"
"#;
        let manifest = parse_manifest("crates/lib/Cargo.toml", lib_manifest).expect("parses");
        assert!(manifest.has_lib);
        let modules = modules_of(std::slice::from_ref(&manifest));
        assert_eq!(modules[0].kind, ModuleKind::RustLibrary);
        assert_eq!(modules[0].path, "crates/lib");
    }

    #[test]
    fn lockfile_versions_and_count() {
        let lock = r#"
[[package]]
name = "serde"
version = "1.0.210"

[[package]]
name = "toml"
version = "0.9.0"
"#;
        let info = parse_lockfile(lock, "digest");
        assert_eq!(info.package_count, 2);
        let versions = lockfile_versions(lock);
        assert_eq!(versions.get("serde").map(String::as_str), Some("1.0.210"));
    }

    #[test]
    fn resolve_dependencies_fills_versions() {
        let manifest = parse_manifest("Cargo.toml", MANIFEST).expect("parses");
        let versions = BTreeMap::from([("serde".to_owned(), "1.0.210".to_owned())]);
        let resolved = resolve_dependencies(manifest.dependencies, &versions);
        let serde = resolved.iter().find(|d| d.name == "serde").expect("serde");
        assert_eq!(serde.resolved.as_deref(), Some("1.0.210"));
        let toml_dep = resolved.iter().find(|d| d.name == "toml").expect("toml");
        assert!(toml_dep.resolved.is_none());
    }
}
