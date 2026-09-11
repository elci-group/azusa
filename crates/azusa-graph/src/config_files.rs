//! Configuration-file classification.
//!
//! Maps repository paths onto [`ConfigurationKind`]s so layer L5 can
//! compare project configuration with severity that respects what each
//! kind controls (CI and deployment configs outweigh formatter tweaks).

use azusa_model::ConfigurationKind;

/// Classify a repository path as a configuration file of interest.
#[must_use]
pub fn classify(path: &str) -> Option<ConfigurationKind> {
    let name = path.rsplit('/').next().unwrap_or(path);

    if name == "Cargo.toml" {
        return Some(ConfigurationKind::CargoManifest);
    }
    if path.starts_with(".github/workflows/") && has_yaml_extension(name) {
        return Some(ConfigurationKind::CiWorkflow);
    }
    if name.starts_with("Dockerfile") || name == "Containerfile" {
        return Some(ConfigurationKind::Dockerfile);
    }
    if matches!(
        name,
        "docker-compose.yml" | "docker-compose.yaml" | "compose.yml" | "compose.yaml"
    ) {
        return Some(ConfigurationKind::Compose);
    }
    if name.ends_with(".service") {
        return Some(ConfigurationKind::SystemdUnit);
    }
    if matches!(name, "rustfmt.toml" | ".rustfmt.toml" | ".editorconfig") {
        return Some(ConfigurationKind::Formatter);
    }
    if matches!(name, "clippy.toml" | "deny.toml") {
        return Some(ConfigurationKind::Linter);
    }
    if matches!(
        name,
        "rust-toolchain" | "rust-toolchain.toml" | ".env.example"
    ) {
        return Some(ConfigurationKind::Environment);
    }
    let lower = path.to_lowercase();
    let in_k8s_dir = ["/k8s/", "/kube/", "/manifests/"]
        .iter()
        .any(|marker| lower.contains(marker))
        || lower.starts_with("k8s/")
        || lower.starts_with("kube/");
    let extension = std::path::Path::new(name)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(str::to_ascii_lowercase);
    if in_k8s_dir && matches!(extension.as_deref(), Some("yml" | "yaml" | "json")) {
        return Some(ConfigurationKind::Kubernetes);
    }
    None
}

/// Case-insensitive `.yml`/`.yaml` check.
fn has_yaml_extension(name: &str) -> bool {
    std::path::Path::new(name)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("yml") || ext.eq_ignore_ascii_case("yaml"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_each_kind() {
        assert_eq!(
            classify("Cargo.toml"),
            Some(ConfigurationKind::CargoManifest)
        );
        assert_eq!(
            classify(".github/workflows/ci.yml"),
            Some(ConfigurationKind::CiWorkflow)
        );
        assert_eq!(
            classify("deploy/Dockerfile"),
            Some(ConfigurationKind::Dockerfile)
        );
        assert_eq!(classify("compose.yaml"), Some(ConfigurationKind::Compose));
        assert_eq!(
            classify("system/api.service"),
            Some(ConfigurationKind::SystemdUnit)
        );
        assert_eq!(classify("rustfmt.toml"), Some(ConfigurationKind::Formatter));
        assert_eq!(classify("clippy.toml"), Some(ConfigurationKind::Linter));
        assert_eq!(
            classify("rust-toolchain.toml"),
            Some(ConfigurationKind::Environment)
        );
        assert_eq!(
            classify("k8s/deploy.yml"),
            Some(ConfigurationKind::Kubernetes)
        );
        assert_eq!(classify("src/main.rs"), None);
    }
}
