//! Ticket `IR-101`: `infra-red.toml` schema, local override merging, and comment-preserving edits.

use ir_core::IrResult;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectManifest {
    pub project: ProjectMeta,
    #[serde(default)]
    pub env: EnvConfig,
    #[serde(default)]
    pub runtimes: BTreeMap<String, RuntimeSpec>,
    #[serde(default)]
    pub mcp: BTreeMap<String, McpServerSpec>,
    #[serde(default)]
    pub skills: BTreeMap<String, SkillSpec>,
    #[serde(default)]
    pub plugins: BTreeMap<String, PluginSpec>,
    #[serde(default)]
    pub profiles: BTreeMap<String, ProfileSpec>,
    #[serde(default)]
    pub targets: TargetsConfig,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectMeta {
    pub name: String,
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
}

fn default_schema_version() -> u32 {
    1
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EnvConfig {
    pub isolation: Option<String>,
    pub default_runtime: Option<String>,
    pub sandbox_profile: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeSpec {
    pub mode: String,
    pub version: Option<String>,
    pub sandbox_override: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerSpec {
    pub source: String,
    pub version: Option<String>,
    pub transport: String,
    pub mcp_spec: Option<String>,
    #[serde(default)]
    pub secrets: BTreeMap<String, String>,
    #[serde(default)]
    pub network_allowlist: Vec<String>,
    pub sandbox: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillSpec {
    pub source: String,
    pub subdir: Option<String>,
    pub rev: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSpec {
    pub source: String,
    pub rev: Option<String>,
    #[serde(default)]
    pub trust_hooks: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProfileSpec {
    #[serde(default)]
    pub mcp: Vec<String>,
    #[serde(default)]
    pub skills: Vec<String>,
    pub sandbox: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TargetsConfig {
    #[serde(default)]
    pub enabled: Vec<String>,
    pub secret_delivery: Option<String>,
}

impl ProjectManifest {
    pub fn from_file(_path: &Path) -> IrResult<Self> {
        todo!("IR-101: Parse infra-red.toml, validate schema_version, and reject plaintext secrets")
    }

    pub fn load_with_local_override(_project_root: &Path) -> IrResult<Self> {
        todo!("IR-101: Merge optional infra-red.local.toml overlay atop infra-red.toml")
    }
}
