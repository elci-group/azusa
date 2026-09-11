//! Engine configuration.
//!
//! Weights, thresholds and rename options live in an optional `azusa.toml`
//! at the repository root (or any parent). Everything has documented
//! defaults, so an empty repository needs no configuration. The effective
//! configuration is serialised and digested into report provenance for
//! reproducibility.

use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use azusa_diff::RenameOptions;
use azusa_model::Layer;
use azusa_model::fileset_digest;

/// File name azusa looks for.
pub const CONFIG_FILE_NAME: &str = "azusa.toml";

/// Drift-index weights per layer. Higher weight pulls the composite score
/// toward that layer. Defaults follow the security-sensitive profile from
/// the technical directive.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DriftWeights {
    /// Ancestry weight (commits ahead/behind, uncommitted drift).
    #[serde(default = "default_ancestry")]
    pub ancestry: f32,
    /// Filesystem weight.
    #[serde(default = "default_one")]
    pub filesystem: f32,
    /// Structure weight.
    #[serde(default = "default_one")]
    pub structure: f32,
    /// Dependency weight.
    #[serde(default = "default_one")]
    pub dependency: f32,
    /// Configuration weight.
    #[serde(default = "default_one")]
    pub configuration: f32,
    /// Interface weight.
    #[serde(default = "default_one")]
    pub interface: f32,
    /// Behaviour weight.
    #[serde(default = "default_one_point_five")]
    pub behaviour: f32,
    /// Semantic weight.
    #[serde(default = "default_two")]
    pub semantic: f32,
    /// Operational weight.
    #[serde(default = "default_one_point_five")]
    pub operational: f32,
}

fn default_one() -> f32 {
    1.0
}

fn default_one_point_five() -> f32 {
    1.5
}

fn default_two() -> f32 {
    2.0
}

fn default_ancestry() -> f32 {
    0.5
}

impl Default for DriftWeights {
    fn default() -> Self {
        Self {
            ancestry: 0.5,
            filesystem: 1.0,
            structure: 1.0,
            dependency: 1.0,
            configuration: 1.0,
            interface: 1.0,
            behaviour: 1.5,
            semantic: 2.0,
            operational: 1.5,
        }
    }
}

impl DriftWeights {
    /// Weight for a layer (identity is never scored).
    #[must_use]
    pub fn get(&self, layer: Layer) -> f32 {
        match layer {
            Layer::Identity => 0.0,
            Layer::Ancestry => self.ancestry,
            Layer::Filesystem => self.filesystem,
            Layer::Structure => self.structure,
            Layer::Dependency => self.dependency,
            Layer::Configuration => self.configuration,
            Layer::Interface => self.interface,
            Layer::Behaviour => self.behaviour,
            Layer::Semantic => self.semantic,
            Layer::Operational => self.operational,
        }
    }
}

/// Exit-code thresholds: which risk levels turn into which exit codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExitThresholds {
    /// Exit code for medium risk.
    #[serde(default = "default_medium_exit")]
    pub medium: i32,
    /// Exit code for high risk.
    #[serde(default = "default_high_exit")]
    pub high: i32,
    /// Exit code for critical risk.
    #[serde(default = "default_critical_exit")]
    pub critical: i32,
}

fn default_medium_exit() -> i32 {
    1
}

fn default_high_exit() -> i32 {
    2
}

fn default_critical_exit() -> i32 {
    2
}

impl Default for ExitThresholds {
    fn default() -> Self {
        Self {
            medium: 1,
            high: 2,
            critical: 2,
        }
    }
}

/// Full engine configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[derive(Default)]
pub struct AzusaConfig {
    /// Drift-index layer weights.
    #[serde(default)]
    pub drift: DriftWeights,
    /// Rename-detection thresholds.
    #[serde(default)]
    pub rename: RenameThresholds,
    /// Exit-code mapping.
    #[serde(default)]
    pub exit: ExitThresholds,
    /// State relatedness thresholds.
    #[serde(default)]
    pub relatedness: Relatedness,
}

/// Rename-detection thresholds (mirrors `azusa-diff` options).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RenameThresholds {
    /// Minimum full-path similarity for a similarity-based pairing.
    #[serde(default = "default_path_threshold")]
    pub path_threshold: f64,
    /// Minimum size ratio for a similarity-based pairing.
    #[serde(default = "default_size_ratio")]
    pub size_ratio_threshold: f64,
    /// Files smaller than this never pair by path similarity alone.
    #[serde(default = "default_min_similarity_size")]
    pub min_similarity_size: u64,
}

fn default_path_threshold() -> f64 {
    0.85
}

fn default_size_ratio() -> f64 {
    0.5
}

fn default_min_similarity_size() -> u64 {
    16
}

impl Default for RenameThresholds {
    fn default() -> Self {
        Self {
            path_threshold: default_path_threshold(),
            size_ratio_threshold: default_size_ratio(),
            min_similarity_size: default_min_similarity_size(),
        }
    }
}

impl From<&RenameThresholds> for RenameOptions {
    fn from(value: &RenameThresholds) -> Self {
        Self {
            path_threshold: value.path_threshold,
            size_ratio_threshold: value.size_ratio_threshold,
            min_similarity_size: value.min_similarity_size,
        }
    }
}

/// State relatedness thresholds.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Relatedness {
    /// Minimum common-path overlap ratio before two states are declared
    /// unrelated.
    #[serde(default = "default_min_overlap")]
    pub min_overlap: f64,
}

fn default_min_overlap() -> f64 {
    0.15
}

impl Default for Relatedness {
    fn default() -> Self {
        Self {
            min_overlap: default_min_overlap(),
        }
    }
}

impl AzusaConfig {
    /// Load configuration for a repository root: the nearest `azusa.toml`
    /// at or above the root, or defaults when absent. Invalid files are
    /// reported as errors rather than silently ignored.
    ///
    /// # Errors
    ///
    /// Returns [`crate::CoreError::Config`] when an `azusa.toml` exists but
    /// cannot be read or parsed.
    pub fn load(root: &Path) -> Result<Self, crate::CoreError> {
        let mut current = Some(root);
        while let Some(dir) = current {
            let candidate = dir.join(CONFIG_FILE_NAME);
            if candidate.is_file() {
                let text = std::fs::read_to_string(&candidate).map_err(|e| {
                    crate::CoreError::Config(format!("{}: {e}", candidate.display()))
                })?;
                let config: Self = toml::from_str(&text).map_err(|e| {
                    crate::CoreError::Config(format!("{}: {e}", candidate.display()))
                })?;
                return Ok(config);
            }
            current = dir.parent();
        }
        Ok(Self::default())
    }

    /// A stable digest of the effective configuration (provenance input).
    #[must_use]
    pub fn digest(&self) -> String {
        let canonical = serde_json::to_string(self).unwrap_or_default();
        let lines = canonical
            .lines()
            .map(std::borrow::ToOwned::to_owned)
            .collect::<Vec<_>>();
        fileset_digest(
            lines
                .into_iter()
                .map(|line| ("config".to_owned(), line, String::new())),
        )
    }

    /// Rename options derived from thresholds.
    #[must_use]
    pub fn rename_options(&self) -> RenameOptions {
        RenameOptions::from(&self.rename)
    }
}

/// Layer weights as an ordered map (for score serialisation).
#[must_use]
pub fn weights_map(weights: &DriftWeights) -> BTreeMap<Layer, f32> {
    Layer::all()
        .iter()
        .copied()
        .filter(|layer| *layer != Layer::Identity)
        .map(|layer| (layer, weights.get(layer)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_the_directive_profile() {
        let weights = DriftWeights::default();
        assert_eq!(weights.semantic, 2.0);
        assert_eq!(weights.behaviour, 1.5);
        assert_eq!(weights.operational, 1.5);
        assert_eq!(weights.filesystem, 1.0);
    }

    #[test]
    fn config_digest_is_stable_and_input_sensitive() {
        let a = AzusaConfig::default();
        let b = AzusaConfig::default();
        assert_eq!(a.digest(), b.digest());
        let mut c = a.clone();
        c.drift.semantic = 3.0;
        assert_ne!(a.digest(), c.digest());
    }

    #[test]
    fn parses_minimal_toml() {
        let config: AzusaConfig = toml::from_str("[drift]\nsemantic = 3.0\n").unwrap();
        assert_eq!(config.drift.semantic, 3.0);
        assert_eq!(config.drift.filesystem, 1.0);
    }

    #[test]
    fn rejects_unknown_keys() {
        assert!(toml::from_str::<AzusaConfig>("[drift]\nbogus = 1.0\n").is_err());
    }
}
