//! Ticket `IR-106`: `ir migrate` importer discovering scattered `.mcp.json`,
//! `.codex/config.toml`, `.agents/mcp_config.json`, `.vscode/mcp.json`, and `.cursor/mcp.json`
//! files and unifying them into `infra-red.toml` while scrubbing plaintext secrets.

use ir_core::IrResult;
use ir_manifest::manifest::{
    McpServerSpec, ProjectManifest, ProjectMeta, TargetsConfig, looks_like_plaintext_secret,
};
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Default, Clone)]
pub struct MigrateReport {
    pub sources_imported: Vec<PathBuf>,
    pub servers_imported: usize,
    pub extracted_secrets: Vec<(String, String)>,
    pub manifest: ProjectManifest,
}

/// Scans `project_root` for existing agent/IDE MCP configs and merges them into a unified
/// [`ProjectManifest`], replacing any detected plaintext secrets with logical keyring keys.
pub fn migrate_repository(project_root: &Path) -> IrResult<MigrateReport> {
    let project_name = project_root
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("infra-red-project")
        .to_string();

    let mut report = MigrateReport {
        manifest: ProjectManifest {
            project: ProjectMeta {
                name: project_name,
                schema_version: 1,
            },
            targets: TargetsConfig {
                enabled: Vec::new(),
                secret_delivery: Some("trampoline".to_string()),
            },
            ..Default::default()
        },
        ..Default::default()
    };

    let json_sources = [
        ("claude-code", project_root.join(".mcp.json"), "mcpServers"),
        (
            "antigravity",
            project_root.join(".agents").join("mcp_config.json"),
            "mcpServers",
        ),
        (
            "vscode",
            project_root.join(".vscode").join("mcp.json"),
            "servers",
        ),
        (
            "cursor",
            project_root.join(".cursor").join("mcp.json"),
            "mcpServers",
        ),
    ];

    for (target_id, path, top_key) in json_sources {
        if !path.exists() {
            continue;
        }
        let raw = std::fs::read_to_string(&path)?;
        if let Ok(val) = serde_json::from_str::<Value>(&raw)
            && let Some(servers) = val.get(top_key).and_then(|v| v.as_object())
        {
            report.sources_imported.push(path);
            if !report
                .manifest
                .targets
                .enabled
                .iter()
                .any(|t| t == target_id)
            {
                report.manifest.targets.enabled.push(target_id.to_string());
            }
            for (name, cfg) in servers {
                let spec = json_server_to_spec(name, cfg, &mut report.extracted_secrets);
                report.manifest.mcp.entry(name.clone()).or_insert(spec);
            }
        }
    }

    let codex_path = project_root.join(".codex").join("config.toml");
    if codex_path.exists() {
        let raw = std::fs::read_to_string(&codex_path)?;
        if let Ok(val) = toml::from_str::<toml::Value>(&raw)
            && let Some(servers) = val.get("mcp_servers").and_then(|v| v.as_table())
        {
            report.sources_imported.push(codex_path);
            if !report.manifest.targets.enabled.iter().any(|t| t == "codex") {
                report.manifest.targets.enabled.push("codex".to_string());
            }
            for (name, cfg) in servers {
                let command = cfg
                    .get("command")
                    .and_then(|v| v.as_str())
                    .map(ToString::to_string);
                let args = cfg
                    .get("args")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(ToString::to_string))
                            .collect()
                    })
                    .unwrap_or_default();
                let url = cfg
                    .get("url")
                    .and_then(|v| v.as_str())
                    .map(ToString::to_string);
                let transport = if url.is_some() {
                    "streamable-http".to_string()
                } else {
                    "stdio".to_string()
                };
                report
                    .manifest
                    .mcp
                    .entry(name.clone())
                    .or_insert(McpServerSpec {
                        source: format!("migrated:{name}"),
                        version: None,
                        transport,
                        command,
                        args,
                        url,
                        env: BTreeMap::new(),
                        mcp_spec: None,
                        secrets: BTreeMap::new(),
                        network_allowlist: Vec::new(),
                        sandbox: None,
                    });
            }
        }
    }

    report.servers_imported = report.manifest.mcp.len();
    Ok(report)
}

fn json_server_to_spec(
    server_name: &str,
    cfg: &Value,
    extracted_secrets: &mut Vec<(String, String)>,
) -> McpServerSpec {
    let command = cfg
        .get("command")
        .and_then(|v| v.as_str())
        .map(ToString::to_string);
    let args = cfg
        .get("args")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(ToString::to_string))
                .collect()
        })
        .unwrap_or_default();
    let url = cfg
        .get("url")
        .and_then(|v| v.as_str())
        .map(ToString::to_string);
    let transport = cfg
        .get("type")
        .and_then(|v| v.as_str())
        .map(ToString::to_string)
        .unwrap_or_else(|| {
            if url.is_some() {
                "streamable-http".to_string()
            } else {
                "stdio".to_string()
            }
        });

    let mut env = BTreeMap::new();
    let mut secrets = BTreeMap::new();
    if let Some(env_obj) = cfg.get("env").and_then(|v| v.as_object()) {
        for (k, v) in env_obj {
            if let Some(s) = v.as_str() {
                if looks_like_plaintext_secret(s) || is_secret_env_name(k) {
                    let logical_key = format!("{server_name}/{}", k.to_lowercase());
                    secrets.insert(k.clone(), logical_key.clone());
                    extracted_secrets.push((format!("{server_name}.{k}"), logical_key));
                } else {
                    env.insert(k.clone(), s.to_string());
                }
            }
        }
    }

    McpServerSpec {
        source: format!("migrated:{server_name}"),
        version: None,
        transport,
        command,
        args,
        url,
        env,
        mcp_spec: None,
        secrets,
        network_allowlist: Vec::new(),
        sandbox: None,
    }
}

fn is_secret_env_name(key: &str) -> bool {
    let upper = key.to_ascii_uppercase();
    upper.contains("TOKEN") || upper.contains("SECRET") || upper.contains("API_KEY")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn migrates_multiple_configs_and_scrubs_plaintext_tokens() {
        let dir = tempdir().unwrap();
        std::fs::write(
            dir.path().join(".mcp.json"),
            r#"{
                "mcpServers": {
                    "github": {
                        "command": "npx",
                        "args": ["-y", "@modelcontextprotocol/server-github"],
                        "env": {
                            "GITHUB_TOKEN": "ghp_secretvalue123",
                            "LOG_LEVEL": "info"
                        }
                    }
                }
            }"#,
        )
        .unwrap();

        std::fs::create_dir_all(dir.path().join(".vscode")).unwrap();
        std::fs::write(
            dir.path().join(".vscode/mcp.json"),
            r#"{
                "servers": {
                    "memory": {
                        "command": "npx",
                        "args": ["-y", "@modelcontextprotocol/server-memory"]
                    }
                }
            }"#,
        )
        .unwrap();

        let report = migrate_repository(dir.path()).unwrap();
        assert_eq!(report.sources_imported.len(), 2);
        assert_eq!(report.servers_imported, 2);
        assert_eq!(report.extracted_secrets.len(), 1);
        assert_eq!(
            report.manifest.mcp["github"].secrets["GITHUB_TOKEN"],
            "github/github_token"
        );
        assert_eq!(report.manifest.mcp["github"].env["LOG_LEVEL"], "info");
        // Verify resulting manifest passes strict plaintext secret validation!
        report
            .manifest
            .validate(&dir.path().join("infra-red.toml"))
            .unwrap();
    }
}
