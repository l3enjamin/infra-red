//! Ticket `IR-106`: `ir doctor` context-window token-bloat estimator, target drift checker,
//! plaintext secret scanner, and MCP 2025-11-25 deprecation auditor.

use crate::drift::TargetHashState;
use ir_core::IrResult;
use ir_manifest::manifest::{ProjectManifest, looks_like_plaintext_secret};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Conservative mid-range upfront tool-definition token estimate per attached MCP server.
const ESTIMATED_TOKENS_PER_MCP_SERVER: usize = 12_000;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct DoctorReport {
    pub mcp_server_count: usize,
    pub estimated_upfront_tokens: usize,
    pub drifted_files: Vec<String>,
    pub plaintext_secret_findings: Vec<String>,
    pub deprecation_warnings: Vec<String>,
    pub recommendations: Vec<String>,
}

/// Runs all workspace health checks for `ir doctor`.
pub fn audit_repository(project_root: &Path) -> IrResult<DoctorReport> {
    let mut report = DoctorReport::default();

    let manifest_path = project_root.join("infra-red.toml");
    if manifest_path.exists() {
        let manifest = ProjectManifest::load_with_local_override(project_root)?;
        report.mcp_server_count = manifest.mcp.len();
        report.estimated_upfront_tokens = manifest.mcp.len() * ESTIMATED_TOKENS_PER_MCP_SERVER;

        if manifest.mcp.len() >= 3 && manifest.profiles.is_empty() {
            report.recommendations.push(format!(
                "{} MCP servers are attached globally (~{} upfront tokens/turn). Define `[profiles.<name>]` in infra-red.toml to scope tools per task.",
                manifest.mcp.len(),
                report.estimated_upfront_tokens
            ));
        }

        for (name, spec) in &manifest.mcp {
            if spec.transport == "sse" || spec.mcp_spec.as_deref() == Some("2025-11-25") {
                report.deprecation_warnings.push(format!(
                    "MCP server `{name}` uses legacy transport/spec (`{}`) eligible for removal after July 28, 2027; upgrade to `streamable-http` / `2026-07-28`.",
                    spec.mcp_spec.as_deref().unwrap_or(&spec.transport)
                ));
            }
        }
    } else {
        report.recommendations.push(
            "No infra-red.toml found. Run `ir migrate` to unify existing agent configs or `ir init` to start fresh."
                .to_string(),
        );
    }

    // Check target file drift
    let state = TargetHashState::load(project_root)?;
    for rel_path in state.files.keys() {
        let full = project_root.join(rel_path);
        if state.has_manual_drift(project_root, &full)? {
            report.drifted_files.push(rel_path.clone());
        }
    }

    // Scan common repo config files for raw plaintext secrets
    let scan_paths = [
        ".mcp.json",
        ".codex/config.toml",
        ".agents/mcp_config.json",
        ".vscode/mcp.json",
        ".cursor/mcp.json",
    ];
    for rel in scan_paths {
        let full = project_root.join(rel);
        if let Ok(content) = std::fs::read_to_string(&full) {
            for token in content.split(|c: char| c.is_whitespace() || c == '"' || c == '\'') {
                if looks_like_plaintext_secret(token) {
                    report.plaintext_secret_findings.push(format!(
                        "Possible plaintext credential detected in `{rel}` (run `ir migrate` + `ir secret set`)"
                    ));
                    break;
                }
            }
        }
    }

    Ok(report)
}
