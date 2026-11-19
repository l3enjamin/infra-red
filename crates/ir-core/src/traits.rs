use crate::types::{ArtifactKind, ExecutionOutcome, ReconcileReport, ResolvedArtifact, SpawnSpec};
use async_trait::async_trait;
use std::collections::BTreeMap;
use std::path::Path;

/// Seam 1: Pluggable registries (Official MCP Registry, Git/gix, npm, PyPI, OCI, local).
#[async_trait]
pub trait ArtifactSource: Send + Sync {
    fn scheme(&self) -> &'static str;
    async fn resolve_metadata(&self, spec: &str) -> anyhow::Result<serde_json::Value>;
    async fn fetch_to_cas(&self, spec: &str, cas_root: &Path) -> anyhow::Result<ResolvedArtifact>;
}

/// Seam 2: Declarative IDE / Coding Agent config projection & drift reconciliation.
#[async_trait]
pub trait TargetAdapter: Send + Sync {
    fn target_id(&self) -> &'static str;
    fn supports_symlinks(&self) -> bool;
    fn supported_artifact_kinds(&self) -> &'static [ArtifactKind];
    async fn reconcile(
        &self,
        project_root: &Path,
        artifacts: &[ResolvedArtifact],
        dry_run: bool,
    ) -> anyhow::Result<ReconcileReport>;
}

/// Seam 3: Coding Agent CLI runtimes (`claude`, `codex`, `agy`, ACP-native agents).
#[async_trait]
pub trait AgentRuntime: Send + Sync {
    fn runtime_id(&self) -> &'static str;
    fn supports_acp(&self) -> bool;
    fn build_isolated_env(
        &self,
        env_root: &Path,
        artifacts: &[ResolvedArtifact],
    ) -> anyhow::Result<BTreeMap<String, String>>;
    fn build_headless_command(
        &self,
        binary: &Path,
        prompt: &str,
        env_root: &Path,
    ) -> anyhow::Result<SpawnSpec>;
}

/// Seam 4: OS-native isolation & process-tree confinement (bwrap, Seatbelt, Job/MXC, OCI).
#[async_trait]
pub trait SandboxBackend: Send + Sync {
    fn backend_name(&self) -> &'static str;
    fn is_available(&self) -> anyhow::Result<()>;
    async fn execute_and_reap(&self, spec: SpawnSpec) -> anyhow::Result<ExecutionOutcome>;
}

/// Seam 5: Zero-disk-leak credential resolution (OS keyring, CI env, Vault/1Password).
#[async_trait]
pub trait SecretProvider: Send + Sync {
    fn provider_name(&self) -> &'static str;
    async fn get_secret(&self, key: &str) -> anyhow::Result<zeroize::Zeroizing<String>>;
    async fn set_secret(&self, key: &str, value: &str) -> anyhow::Result<()>;
}

/// Seam 6: Protocol bridges (MCP 2026-07-28 Server, ACP Proxy, future A2A v1 / AG-UI).
#[async_trait]
pub trait ProtocolBridge: Send + Sync {
    fn protocol_name(&self) -> &'static str;
    async fn serve(&self) -> anyhow::Result<()>;
}
