//! Ticket `IR-101`: `infra-red.toml` schema, local override merging, secret-leak guardrails,
//! and comment-preserving edits via `toml_edit`.

use ir_core::{IrError, IrResult};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::Path;

/// Literal token prefixes that strongly indicate a hardcoded secret rather than a logical keyring path.
const PLAINTEXT_SECRET_PREFIXES: &[&str] = &[
    "ghp_",
    "gho_",
    "ghu_",
    "ghs_",
    "github_pat_",
    "sk-",
    "sk_live_",
    "sk_test_",
    "AIza",
    "xoxb-",
    "xoxp-",
    "AKIA",
    "ASIA",
];

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectMeta {
    pub name: String,
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
}

fn default_schema_version() -> u32 {
    1
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnvConfig {
    pub isolation: Option<String>,
    pub default_runtime: Option<String>,
    pub sandbox_profile: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeSpec {
    pub mode: String,
    pub version: Option<String>,
    pub sandbox_override: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct McpServerSpec {
    pub source: String,
    pub version: Option<String>,
    pub transport: String,
    #[serde(default)]
    pub command: Option<String>,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    pub mcp_spec: Option<String>,
    #[serde(default)]
    pub secrets: BTreeMap<String, String>,
    #[serde(default)]
    pub network_allowlist: Vec<String>,
    pub sandbox: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillSpec {
    pub source: String,
    pub subdir: Option<String>,
    pub rev: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginSpec {
    pub source: String,
    pub rev: Option<String>,
    #[serde(default)]
    pub trust_hooks: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileSpec {
    #[serde(default)]
    pub mcp: Vec<String>,
    #[serde(default)]
    pub skills: Vec<String>,
    pub sandbox: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TargetsConfig {
    #[serde(default)]
    pub enabled: Vec<String>,
    pub secret_delivery: Option<String>,
}

/// Returns `true` if `value` matches a known plaintext credential prefix.
pub fn looks_like_plaintext_secret(value: &str) -> bool {
    let trimmed = value.trim();
    PLAINTEXT_SECRET_PREFIXES
        .iter()
        .any(|prefix| trimmed.starts_with(prefix))
}

impl ProjectManifest {
    /// Parses and validates `infra-red.toml` from disk.
    pub fn from_file(path: &Path) -> IrResult<Self> {
        let raw = std::fs::read_to_string(path)?;
        let manifest: Self = toml::from_str(&raw).map_err(|err| IrError::Manifest {
            path: path.to_path_buf(),
            message: err.to_string(),
        })?;
        manifest.validate(path)?;
        Ok(manifest)
    }

    /// Loads `infra-red.toml` and merges optional machine-local overrides from `infra-red.local.toml`.
    pub fn load_with_local_override(project_root: &Path) -> IrResult<Self> {
        let manifest_path = project_root.join("infra-red.toml");
        let mut base = Self::from_file(&manifest_path)?;

        let local_path = project_root.join("infra-red.local.toml");
        if local_path.exists() {
            let local_raw = std::fs::read_to_string(&local_path)?;
            let local: PartialLocalOverlay =
                toml::from_str(&local_raw).map_err(|err| IrError::Manifest {
                    path: local_path.clone(),
                    message: err.to_string(),
                })?;

            if let Some(env) = local.env {
                if env.isolation.is_some() {
                    base.env.isolation = env.isolation;
                }
                if env.default_runtime.is_some() {
                    base.env.default_runtime = env.default_runtime;
                }
                if env.sandbox_profile.is_some() {
                    base.env.sandbox_profile = env.sandbox_profile;
                }
            }
            base.runtimes.extend(local.runtimes);
            base.mcp.extend(local.mcp);
            base.skills.extend(local.skills);
            base.plugins.extend(local.plugins);
            base.profiles.extend(local.profiles);
            if let Some(targets) = local.targets {
                if !targets.enabled.is_empty() {
                    base.targets.enabled = targets.enabled;
                }
                if targets.secret_delivery.is_some() {
                    base.targets.secret_delivery = targets.secret_delivery;
                }
            }
            base.validate(&local_path)?;
        }

        Ok(base)
    }

    /// Validates schema version and rejects plaintext credentials inside `[mcp.<name>.secrets]` or `env`.
    pub fn validate(&self, path: &Path) -> IrResult<()> {
        if self.project.schema_version != 1 {
            return Err(IrError::Manifest {
                path: path.to_path_buf(),
                message: format!(
                    "unsupported schema_version {} (supported: 1)",
                    self.project.schema_version
                ),
            });
        }
        if self.project.name.trim().is_empty() {
            return Err(IrError::Manifest {
                path: path.to_path_buf(),
                message: "`project.name` must not be empty".to_string(),
            });
        }
        for (server_name, spec) in &self.mcp {
            for (env_key, logical_ref) in &spec.secrets {
                if looks_like_plaintext_secret(logical_ref) {
                    return Err(IrError::PlaintextSecretDetected {
                        key: format!("mcp.{server_name}.secrets.{env_key}"),
                        hint: format!(
                            "replace raw secret with a logical keyring key (e.g. \"{server_name}/{env_key}\") and run `ir secret set {server_name}/{env_key}`"
                        ),
                    });
                }
            }
            for (env_key, val) in &spec.env {
                if looks_like_plaintext_secret(val) {
                    return Err(IrError::PlaintextSecretDetected {
                        key: format!("mcp.{server_name}.env.{env_key}"),
                        hint: format!(
                            "move `{env_key}` into `[mcp.{server_name}.secrets]` using a logical keyring path instead of plaintext env"
                        ),
                    });
                }
            }
        }
        Ok(())
    }

    /// Computes a deterministic SHA-256 digest of the canonicalized manifest (CRLF-normalized).
    pub fn canonical_digest(&self) -> IrResult<String> {
        let serialized = toml::to_string(self).map_err(|e| IrError::Other(e.into()))?;
        let normalized = serialized.replace("\r\n", "\n");
        let digest = Sha256::digest(normalized.as_bytes());
        Ok(format!("sha256:{}", hex_encode(&digest)))
    }

    /// Returns the subset of MCP servers and skills active for an optional named profile.
    pub fn filter_for_profile(
        &self,
        profile_name: Option<&str>,
    ) -> IrResult<(BTreeMap<String, McpServerSpec>, BTreeMap<String, SkillSpec>)> {
        let Some(name) = profile_name else {
            return Ok((self.mcp.clone(), self.skills.clone()));
        };
        let profile = self.profiles.get(name).ok_or_else(|| IrError::Manifest {
            path: std::path::PathBuf::from("infra-red.toml"),
            message: format!("unknown profile `{name}`"),
        })?;
        let mcp = self
            .mcp
            .iter()
            .filter(|(k, _)| profile.mcp.iter().any(|p| p == *k))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        let skills = self
            .skills
            .iter()
            .filter(|(k, _)| profile.skills.iter().any(|p| p == *k))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        Ok((mcp, skills))
    }
}

#[derive(Debug, Default, Deserialize)]
struct PartialLocalOverlay {
    env: Option<EnvConfig>,
    #[serde(default)]
    runtimes: BTreeMap<String, RuntimeSpec>,
    #[serde(default)]
    mcp: BTreeMap<String, McpServerSpec>,
    #[serde(default)]
    skills: BTreeMap<String, SkillSpec>,
    #[serde(default)]
    plugins: BTreeMap<String, PluginSpec>,
    #[serde(default)]
    profiles: BTreeMap<String, ProfileSpec>,
    targets: Option<TargetsConfig>,
}

/// Inserts or updates `[mcp.<name>]` in `infra-red.toml` while preserving existing user comments.
pub fn upsert_mcp_entry(manifest_path: &Path, name: &str, spec: &McpServerSpec) -> IrResult<()> {
    let raw = std::fs::read_to_string(manifest_path)?;
    let mut doc = raw
        .parse::<toml_edit::DocumentMut>()
        .map_err(|err| IrError::Manifest {
            path: manifest_path.to_path_buf(),
            message: err.to_string(),
        })?;

    if !doc.contains_key("mcp") {
        let mut mcp_table = toml_edit::Table::new();
        mcp_table.set_implicit(true);
        doc["mcp"] = toml_edit::Item::Table(mcp_table);
    }

    let mut entry = toml_edit::Table::new();
    entry["source"] = toml_edit::value(spec.source.as_str());
    if let Some(version) = &spec.version {
        entry["version"] = toml_edit::value(version.as_str());
    }
    entry["transport"] = toml_edit::value(spec.transport.as_str());
    if let Some(cmd) = &spec.command {
        entry["command"] = toml_edit::value(cmd.as_str());
    }
    if !spec.args.is_empty() {
        let mut arr = toml_edit::Array::new();
        for arg in &spec.args {
            arr.push(arg.as_str());
        }
        entry["args"] = toml_edit::value(arr);
    }
    if let Some(url) = &spec.url {
        entry["url"] = toml_edit::value(url.as_str());
    }
    if !spec.secrets.is_empty() {
        let mut inline = toml_edit::InlineTable::new();
        for (k, v) in &spec.secrets {
            inline.insert(k, v.as_str().into());
        }
        entry["secrets"] = toml_edit::value(inline);
    }

    doc["mcp"][name] = toml_edit::Item::Table(entry);
    std::fs::write(manifest_path, doc.to_string())?;
    Ok(())
}

pub(crate) fn hex_encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        use std::fmt::Write;
        let _ = write!(&mut out, "{b:02x}");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn parses_manifest_and_merges_local_override() {
        let dir = tempdir().unwrap();
        std::fs::write(
            dir.path().join("infra-red.toml"),
            r#"
# Top comment must survive edits
[project]
name = "payments-api"
schema_version = 1

[env]
default_runtime = "claude"

[mcp.memory]
source = "npm:@modelcontextprotocol/server-memory"
transport = "stdio"
command = "npx"
args = ["-y", "@modelcontextprotocol/server-memory"]

[targets]
enabled = ["claude-code", "vscode"]
"#,
        )
        .unwrap();

        std::fs::write(
            dir.path().join("infra-red.local.toml"),
            r#"
[env]
default_runtime = "codex"
"#,
        )
        .unwrap();

        let merged = ProjectManifest::load_with_local_override(dir.path()).unwrap();
        assert_eq!(merged.project.name, "payments-api");
        assert_eq!(merged.env.default_runtime.as_deref(), Some("codex"));
        assert!(merged.mcp.contains_key("memory"));
    }

    #[test]
    fn rejects_plaintext_secret_in_manifest() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("infra-red.toml");
        std::fs::write(
            &path,
            r#"
[project]
name = "demo"
schema_version = 1

[mcp.github]
source = "registry:io.github.github/github-mcp-server"
transport = "streamable-http"
secrets = { GITHUB_TOKEN = "ghp_1234567890abcdef" }
"#,
        )
        .unwrap();

        let err = ProjectManifest::from_file(&path).unwrap_err();
        assert!(matches!(err, IrError::PlaintextSecretDetected { .. }));
    }

    #[test]
    fn upsert_mcp_entry_preserves_comments() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("infra-red.toml");
        std::fs::write(
            &path,
            "# Important team header comment\n[project]\nname = \"demo\"\nschema_version = 1\n",
        )
        .unwrap();

        let spec = McpServerSpec {
            source: "npm:@modelcontextprotocol/server-memory".to_string(),
            version: Some("1.0.0".to_string()),
            transport: "stdio".to_string(),
            command: Some("npx".to_string()),
            args: vec![
                "-y".to_string(),
                "@modelcontextprotocol/server-memory".to_string(),
            ],
            url: None,
            env: BTreeMap::new(),
            mcp_spec: None,
            secrets: BTreeMap::new(),
            network_allowlist: vec![],
            sandbox: None,
        };

        upsert_mcp_entry(&path, "memory", &spec).unwrap();
        let updated = std::fs::read_to_string(&path).unwrap();
        assert!(updated.contains("# Important team header comment"));
        assert!(updated.contains("[mcp.memory]"));

        let parsed = ProjectManifest::from_file(&path).unwrap();
        assert_eq!(parsed.mcp["memory"].command.as_deref(), Some("npx"));
    }
}
