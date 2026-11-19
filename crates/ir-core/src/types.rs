use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

/// The four canonical artifact classes managed uniformly by `infra-red`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ArtifactKind {
    McpServer,
    Skill,
    Plugin,
    Runtime,
}

/// An artifact that has been resolved, checksum-verified, and materialized in the global CAS.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedArtifact {
    pub id: String,
    pub kind: ArtifactKind,
    pub version: semver::Version,
    pub integrity_sha256: String,
    pub cas_path: PathBuf,
}

/// Filesystem and network confinement rules enforced by a [`crate::SandboxBackend`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SandboxPolicy {
    pub workspace_root: PathBuf,
    pub read_only_paths: Vec<PathBuf>,
    pub read_write_paths: Vec<PathBuf>,
    pub denied_paths: Vec<PathBuf>,
    pub network_mode: NetworkMode,
}

/// Network isolation level for a sandboxed process.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NetworkMode {
    #[default]
    Disabled,
    ProxyAllowlist {
        domains: Vec<String>,
        proxy_addr: String,
    },
    Unrestricted,
}

/// Complete specification for spawning a child process inside an isolated or sandboxed environment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpawnSpec {
    pub program: PathBuf,
    pub args: Vec<String>,
    pub cwd: PathBuf,
    pub env: BTreeMap<String, String>,
    pub sandbox: SandboxPolicy,
}

/// Report emitted by [`crate::TargetAdapter::reconcile`].
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReconcileReport {
    pub files_written: Vec<PathBuf>,
    pub files_skipped_due_to_drift: Vec<PathBuf>,
    pub capability_warnings: Vec<String>,
}

/// Result returned after executing and reaping a sandboxed process tree.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionOutcome {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}
