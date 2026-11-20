//! Ticket `IR-102`: Multi-component plugin manifest parser (`.claude-plugin/plugin.json`,
//! `.plugin/plugin.json`, and Antigravity `plugin.json`).

use ir_core::{IrError, IrResult};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginHook {
    pub event: String,
    pub command: String,
}

/// Discovers and normalizes Claude Code, vendor-neutral, or Antigravity plugin manifests
/// into a [`CanonicalPlugin`] intermediate representation.
pub fn parse_plugin_dir(plugin_dir: &Path) -> IrResult<CanonicalPlugin> {
    let candidates = [
        plugin_dir.join(".claude-plugin").join("plugin.json"),
        plugin_dir.join(".plugin").join("plugin.json"),
        plugin_dir.join("plugin.json"),
    ];

    let manifest_path = candidates
        .into_iter()
        .find(|p| p.exists())
        .ok_or_else(|| IrError::Manifest {
            path: plugin_dir.to_path_buf(),
            message: "no plugin.json found (.claude-plugin/plugin.json, .plugin/plugin.json, or plugin.json)".to_string(),
        })?;

    let raw = std::fs::read_to_string(&manifest_path)?;
    let value: serde_json::Value = serde_json::from_str(&raw).map_err(|err| IrError::Manifest {
        path: manifest_path.clone(),
        message: err.to_string(),
    })?;

    let name = value
        .get("name")
        .and_then(|v| v.as_str())
        .map(ToString::to_string)
        .or_else(|| {
            plugin_dir
                .file_name()
                .and_then(|s| s.to_str())
                .map(ToString::to_string)
        })
        .unwrap_or_else(|| "unnamed-plugin".to_string());

    let version = value
        .get("version")
        .and_then(|v| v.as_str())
        .map(ToString::to_string);

    let mut hooks = Vec::new();
    if let Some(hook_map) = value.get("hooks").and_then(|v| v.as_object()) {
        for (event, entries) in hook_map {
            if let Some(arr) = entries.as_array() {
                for item in arr {
                    if let Some(cmd) = item.get("command").and_then(|c| c.as_str()) {
                        hooks.push(PluginHook {
                            event: event.clone(),
                            command: cmd.to_string(),
                        });
                    }
                }
            } else if let Some(cmd) = entries.as_str() {
                hooks.push(PluginHook {
                    event: event.clone(),
                    command: cmd.to_string(),
                });
            }
        }
    }

    let skill_paths = value
        .get("skills")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(ToString::to_string))
                .collect()
        })
        .unwrap_or_default();

    let mcp_servers = value
        .get("mcpServers")
        .and_then(|v| v.as_object())
        .map(|obj| obj.keys().cloned().collect())
        .unwrap_or_default();

    Ok(CanonicalPlugin {
        name,
        version,
        hooks,
        skill_paths,
        mcp_servers,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn parses_claude_plugin_manifest() {
        let dir = tempdir().unwrap();
        let meta_dir = dir.path().join(".claude-plugin");
        std::fs::create_dir_all(&meta_dir).unwrap();
        std::fs::write(
            meta_dir.join("plugin.json"),
            r#"{
                "name": "vercel-plugin",
                "version": "1.2.0",
                "skills": ["skills/deploy"],
                "mcpServers": { "vercel": { "command": "npx" } },
                "hooks": {
                    "PreToolUse": [{ "command": "echo checking" }]
                }
            }"#,
        )
        .unwrap();

        let plugin = parse_plugin_dir(dir.path()).unwrap();
        assert_eq!(plugin.name, "vercel-plugin");
        assert_eq!(plugin.version.as_deref(), Some("1.2.0"));
        assert_eq!(plugin.skill_paths, vec!["skills/deploy"]);
        assert_eq!(plugin.mcp_servers, vec!["vercel"]);
        assert_eq!(plugin.hooks.len(), 1);
        assert_eq!(plugin.hooks[0].event, "PreToolUse");
    }
}
