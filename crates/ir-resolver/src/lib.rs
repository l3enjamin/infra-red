//! `ir-resolver` implements conflict-driven SAT solving via `pubgrub` with
//! synthetic host virtual packages (`__os`, `__arch`, `__agent_*`, `__mcp_spec`, `__sandbox`).
//!
//! Primary ticket: `IR-109`.

use ir_core::IrResult;
use std::collections::BTreeMap;

/// Detected or injected host capabilities exposed to PubGrub as virtual packages.
#[derive(Debug, Clone, Default)]
pub struct HostCapabilities {
    pub virtual_packages: BTreeMap<String, semver::Version>,
}

impl HostCapabilities {
    pub fn detect_host() -> Self {
        let mut virtual_packages = BTreeMap::new();
        virtual_packages.insert("__mcp_spec".to_string(), semver::Version::new(2026, 7, 28));
        Self { virtual_packages }
    }
}

pub fn solve_graph(_host: &HostCapabilities) -> IrResult<BTreeMap<String, semver::Version>> {
    todo!(
        "IR-109: Implement pubgrub DependencyProvider with virtual capability packages and causal conflict formatting"
    )
}
