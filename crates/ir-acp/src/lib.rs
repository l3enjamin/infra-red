//! `ir-acp` encapsulates Agent Client Protocol (ACP v1 stable + feature-gated v2)
//! Conductor/Proxy mode (`ir acp serve`) and ACP Client mode (`ir run --via-acp`).
//!
//! Primary ticket: `IR-401`.

use async_trait::async_trait;
use ir_core::ProtocolBridge;
use std::path::PathBuf;

pub struct AcpConductorBridge {
    pub project_root: PathBuf,
}

#[async_trait]
impl ProtocolBridge for AcpConductorBridge {
    fn protocol_name(&self) -> &'static str {
        "acp-v1"
    }

    async fn serve(&self) -> anyhow::Result<()> {
        todo!(
            "IR-401: Implement ACP v1 conductor injecting locked MCP servers into session/new and session/load"
        )
    }
}
