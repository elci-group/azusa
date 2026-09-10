//! Azusa ELCI integration.
//!
//! Emits the ELCI universal event contract
//! (`azusa.drift.detected`, `azusa.policy.failed`, `azusa.plan.generated`,
//! …) as NDJSON for Vamos-style transports, and writes versioned export
//! artefacts (canonical JSON drift reports with provenance) that other ELCI
//! tools — Amber, Fract, Traci, Isopod, Bart, Scrawny, Padagonia — can
//! consume as a shared data product.

/// Name of this crate, used in provenance records.
pub const CRATE_NAME: &str = "azusa-elci";

/// Smoke test.
#[test]
fn elci_smoke() {
    assert_eq!(CRATE_NAME, "azusa-elci");
}
