//! Ticket `IR-101`: Deterministic `infra-red.lock` serialization and manifest digest verification.

use ir_core::{ArtifactKind, IrResult};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lockfile {
    pub lockfile_version: u32,
    pub manifest_digest: String,
    #[serde(default, rename = "package")]
    pub packages: Vec<LockedPackage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockedPackage {
    pub id: String,
    pub kind: ArtifactKind,
    pub version: String,
    pub source: String,
    pub subdir: Option<String>,
    pub integrity: String,
    pub mcp_protocol: Option<String>,
    #[serde(default)]
    pub requires: Vec<String>,
}

impl Lockfile {
    pub fn from_file(_path: &Path) -> IrResult<Self> {
        todo!("IR-101: Load and validate infra-red.lock")
    }

    pub fn write_to_file(&self, _path: &Path) -> IrResult<()> {
        todo!("IR-101: Write deterministic sorted infra-red.lock")
    }
}
