//! Reusable in-memory mock implementations of `ir-core` traits.
//!
//! Junior developers working on Layer 1–3 crates (`ir-targets`, `ir-runtime`,
//! `ir-mcp-server`, `ir-acp`, `infra-red`, `irx`) should use these test doubles
//! so unit tests run fast, deterministically, and without requiring network access,
//! OS keyrings, or kernel sandbox privileges.

use crate::error::IrError;
use crate::traits::{AgentRuntime, ArtifactSource, SandboxBackend, SecretProvider};
use crate::types::{ArtifactKind, ExecutionOutcome, ResolvedArtifact, SandboxPolicy, SpawnSpec};
use async_trait::async_trait;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use zeroize::Zeroizing;

/// Thread-safe in-memory [`SecretProvider`] for unit testing secret injection and trampolines.
#[derive(Default, Clone)]
pub struct InMemorySecretProvider {
    secrets: Arc<Mutex<BTreeMap<String, String>>>,
}

impl std::fmt::Debug for InMemorySecretProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InMemorySecretProvider")
            .field("secrets", &"[REDACTED]")
            .finish()
    }
}

impl InMemorySecretProvider {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_secret(self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.secrets
            .lock()
            .expect("secret lock poisoned")
            .insert(key.into(), value.into());
        self
    }
}

#[async_trait]
impl SecretProvider for InMemorySecretProvider {
    fn provider_name(&self) -> &'static str {
        "in-memory-mock"
    }

    async fn get_secret(&self, key: &str) -> anyhow::Result<Zeroizing<String>> {
        let guard = self.secrets.lock().expect("secret lock poisoned");
        guard.get(key).cloned().map(Zeroizing::new).ok_or_else(|| {
            IrError::SecretNotFound {
                key: key.to_string(),
                provider: self.provider_name().to_string(),
            }
            .into()
        })
    }

    async fn set_secret(&self, key: &str, value: &str) -> anyhow::Result<()> {
        let mut guard = self.secrets.lock().expect("secret lock poisoned");
        guard.insert(key.to_string(), value.to_string());
        Ok(())
    }
}

/// Recording [`SandboxBackend`] that captures [`SpawnSpec`] invocations without spawning OS processes.
#[derive(Debug, Default, Clone)]
pub struct NoopSandboxBackend {
    recorded: Arc<Mutex<Vec<SpawnSpec>>>,
    canned_stdout: String,
}

impl NoopSandboxBackend {
    pub fn new() -> Self {
        Self {
            recorded: Arc::new(Mutex::new(Vec::new())),
            canned_stdout: "noop-sandbox-ok".to_string(),
        }
    }

    pub fn with_stdout(mut self, stdout: impl Into<String>) -> Self {
        self.canned_stdout = stdout.into();
        self
    }

    pub fn recorded_specs(&self) -> Vec<SpawnSpec> {
        self.recorded.lock().expect("sandbox lock poisoned").clone()
    }
}

#[async_trait]
impl SandboxBackend for NoopSandboxBackend {
    fn backend_name(&self) -> &'static str {
        "noop-mock"
    }

    fn is_available(&self) -> anyhow::Result<()> {
        Ok(())
    }

    async fn execute_and_reap(&self, spec: SpawnSpec) -> anyhow::Result<ExecutionOutcome> {
        self.recorded
            .lock()
            .expect("sandbox lock poisoned")
            .push(spec);
        Ok(ExecutionOutcome {
            exit_code: 0,
            stdout: self.canned_stdout.clone(),
            stderr: String::new(),
        })
    }
}

/// Deterministic [`ArtifactSource`] stub for testing manifest resolution and target projections.
#[derive(Debug, Clone)]
pub struct StubArtifactSource {
    scheme: &'static str,
    kind: ArtifactKind,
}

impl StubArtifactSource {
    pub fn new(scheme: &'static str, kind: ArtifactKind) -> Self {
        Self { scheme, kind }
    }
}

#[async_trait]
impl ArtifactSource for StubArtifactSource {
    fn scheme(&self) -> &'static str {
        self.scheme
    }

    async fn resolve_metadata(&self, spec: &str) -> anyhow::Result<serde_json::Value> {
        Ok(serde_json::json!({
            "spec": spec,
            "scheme": self.scheme,
            "version": "1.0.0"
        }))
    }

    async fn fetch_to_cas(&self, spec: &str, cas_root: &Path) -> anyhow::Result<ResolvedArtifact> {
        let sanitized = spec.replace([':', '/', '@'], "_");
        Ok(ResolvedArtifact {
            id: spec.to_string(),
            kind: self.kind,
            version: semver::Version::new(1, 0, 0),
            integrity_sha256:
                "sha256:0000000000000000000000000000000000000000000000000000000000000000"
                    .to_string(),
            cas_path: cas_root.join(sanitized),
        })
    }
}

/// Minimal [`AgentRuntime`] mock for testing `ir-acp` and `ir-mcp-server`.
#[derive(Debug, Clone)]
pub struct MockAgentRuntime {
    id: &'static str,
}

impl MockAgentRuntime {
    pub fn new(id: &'static str) -> Self {
        Self { id }
    }
}

#[async_trait]
impl AgentRuntime for MockAgentRuntime {
    fn runtime_id(&self) -> &'static str {
        self.id
    }

    fn supports_acp(&self) -> bool {
        true
    }

    fn build_isolated_env(
        &self,
        env_root: &Path,
        _artifacts: &[ResolvedArtifact],
    ) -> anyhow::Result<BTreeMap<String, String>> {
        let mut env = BTreeMap::new();
        env.insert("IR_MOCK_HOME".to_string(), env_root.display().to_string());
        Ok(env)
    }

    fn build_headless_command(
        &self,
        binary: &Path,
        prompt: &str,
        env_root: &Path,
    ) -> anyhow::Result<SpawnSpec> {
        Ok(SpawnSpec {
            program: binary.to_path_buf(),
            args: vec!["--prompt".to_string(), prompt.to_string()],
            cwd: env_root.to_path_buf(),
            env: self.build_isolated_env(env_root, &[])?,
            sandbox: SandboxPolicy {
                workspace_root: PathBuf::from(env_root),
                ..Default::default()
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn in_memory_secret_provider_roundtrip_and_redaction() {
        let provider = InMemorySecretProvider::new().with_secret("github/pat", "secret-token");
        let debug_str = format!("{provider:?}");
        assert!(debug_str.contains("[REDACTED]"));
        assert!(!debug_str.contains("secret-token"));

        let retrieved = provider.get_secret("github/pat").await.unwrap();
        assert_eq!(retrieved.as_str(), "secret-token");

        let missing = provider.get_secret("missing/key").await;
        assert!(missing.is_err());
    }

    #[tokio::test]
    async fn noop_sandbox_records_spawn_specs() {
        let sandbox = NoopSandboxBackend::new().with_stdout("audit-clean");
        let spec = SpawnSpec {
            program: PathBuf::from("echo"),
            args: vec!["hello".into()],
            cwd: PathBuf::from("/tmp"),
            env: BTreeMap::new(),
            sandbox: SandboxPolicy::default(),
        };

        let outcome = sandbox.execute_and_reap(spec.clone()).await.unwrap();
        assert_eq!(outcome.exit_code, 0);
        assert_eq!(outcome.stdout, "audit-clean");
        assert_eq!(sandbox.recorded_specs(), vec![spec]);
    }
}
