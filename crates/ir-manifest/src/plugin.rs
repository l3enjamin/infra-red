//! Ticket `IR-102`: Multi-component plugin manifest parser (`.claude-plugin/plugin.json`, `.agents/plugins/`).

use ir_core::IrResult;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CanonicalPlugin {
    pub name: String,
    pub version: Option<String>,
    #[serde(default)]
    pub hooks: Vec<PluginHook>,
    #[serde(default)]
    pub skill_paths: Vec<String>,
    #[serde(default)]
    pub mcp_servers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginHook {
    pub event: String,
    pub command: String,
}

pub fn parse_plugin_dir(_plugin_dir: &Path) -> IrResult<CanonicalPlugin> {
    todo!(
        "IR-102: Parse Claude, vendor-neutral, and Antigravity plugin manifests into CanonicalPlugin"
    )
}
