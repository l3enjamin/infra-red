//! `ir-mcp-server` implements the stateless MCP 2026-07-28 server (`irx serve-mcp`)
//! exposing `irx_list_catalog` and `irx_run_isolated` with progressive disclosure and MCP Tasks.
//!
//! Primary ticket: `IR-304`.

use ir_core::{SandboxBackend, SandboxPolicy};
use schemars::JsonSchema;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct RunIsolatedRequest {
    /// Target agent runtime id (e.g., "claude", "codex", "agy").
    pub agent: String,
    /// Prompt to execute inside the clean, ephemeral environment.
    pub prompt: String,
    /// Explicit minimal set of MCP servers to attach (prevents context bloat).
    #[serde(default)]
    pub with_mcp: Vec<String>,
    /// Explicit minimal set of Agent Skills to attach.
    #[serde(default)]
    pub with_skills: Vec<String>,
    /// Whether the workspace is mounted read-only (default: true).
    #[serde(default = "default_true")]
    pub read_only: bool,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListCatalogRequest {
    /// Optional search query to filter locked MCP servers and skills.
    #[serde(default)]
    pub query: Option<String>,
}

fn default_true() -> bool {
    true
}

#[derive(Clone)]
pub struct IrxMcpServer {
    sandbox: Arc<dyn SandboxBackend>,
}

impl IrxMcpServer {
    pub fn new(sandbox: Arc<dyn SandboxBackend>) -> Self {
        Self { sandbox }
    }

    pub fn sandbox_backend_name(&self) -> &'static str {
        self.sandbox.backend_name()
    }

    pub fn default_ephemeral_policy(ephemeral_root: std::path::PathBuf) -> SandboxPolicy {
        SandboxPolicy {
            workspace_root: ephemeral_root,
            ..Default::default()
        }
    }
}

/// Entrypoint for `irx serve-mcp` over stdio (MCP 2026-07-28 via `rmcp`).
pub async fn serve_stdio(_sandbox: Arc<dyn SandboxBackend>) -> anyhow::Result<()> {
    todo!(
        "IR-304: Wire IrxMcpServer with rmcp #[tool_router(server_handler)] over stdio & Streamable HTTP"
    )
}
