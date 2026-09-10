//! Azusa structural analysis.
//!
//! Derives structure, dependency, configuration and behaviour facets from a
//! captured [`azusa_model::FilesystemState`]: Cargo workspace topology,
//! dependency graphs, configuration files of interest, public-API surface
//! inventory and test/build inventory. Pure function of the state — no I/O —
//! which keeps the analysis deterministic and cacheable.

/// Name of this crate, used in provenance records.
pub const CRATE_NAME: &str = "azusa-graph";

/// Smoke test.
#[test]
fn graph_smoke() {
    assert_eq!(CRATE_NAME, "azusa-graph");
}
