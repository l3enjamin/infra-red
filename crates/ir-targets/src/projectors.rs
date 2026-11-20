//! Tickets `IR-104` & `IR-105`: Declarative target adapters for Claude Code, OpenAI Codex,
//! Google Antigravity (`agy`), VS Code Copilot, and Cursor.

use crate::drift::TargetHashState;
use async_trait::async_trait;
use ir_core::{ArtifactKind, IrResult, ReconcileReport, ResolvedArtifact, TargetAdapter};
use ir_manifest::manifest::{McpServerSpec, ProjectManifest};
use ir_store::{LinkStrategy, materialize_tree};
use serde_json::{Map, Value, json};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, Default)]
pub struct ReconcileOptions {
    pub dry_run: bool,
    pub force: bool,
}

const ALL_KINDS: &[ArtifactKind] = &[
    ArtifactKind::McpServer,
    ArtifactKind::Skill,
    ArtifactKind::Plugin,
];

macro_rules! simple_target_adapter {
    ($name:ident, $id:expr, $symlinks:expr) => {
        #[derive(Debug, Default, Clone)]
        pub struct $name;

        #[async_trait]
        impl TargetAdapter for $name {
            fn target_id(&self) -> &'static str {
                $id
            }

            fn supports_symlinks(&self) -> bool {
                $symlinks
            }

            fn supported_artifact_kinds(&self) -> &'static [ArtifactKind] {
                ALL_KINDS
            }

            async fn reconcile(
                &self,
                project_root: &Path,
                artifacts: &[ResolvedArtifact],
                dry_run: bool,
            ) -> anyhow::Result<ReconcileReport> {
                let mut report = ReconcileReport::default();
                let strategy = if self.supports_symlinks() {
                    LinkStrategy::Symlink
                } else {
                    LinkStrategy::CowOrHardlink
                };
                for artifact in artifacts {
                    if artifact.kind == ArtifactKind::Skill && artifact.cas_path.exists() {
                        let dst = skill_destination(self.target_id(), project_root, &artifact.id);
                        if !dry_run {
                            materialize_tree(&artifact.cas_path, &dst, strategy)?;
                        }
                        report.files_written.push(dst);
                    }
                }
                Ok(report)
            }
        }
    };
}

simple_target_adapter!(ClaudeCodeAdapter, "claude-code", true);
simple_target_adapter!(CodexAdapter, "codex", true);
simple_target_adapter!(AntigravityAdapter, "antigravity", true);
// VS Code Copilot cannot traverse symlinked agent skills — always use CowOrHardlink.
simple_target_adapter!(VsCodeAdapter, "vscode", false);
simple_target_adapter!(CursorAdapter, "cursor", true);

/// Resolves enabled target names from `infra-red.toml` into concrete [`TargetAdapter`] instances.
pub fn resolve_adapters(enabled: &[String]) -> Vec<Arc<dyn TargetAdapter>> {
    let mut adapters: Vec<Arc<dyn TargetAdapter>> = Vec::new();
    for id in enabled {
        match id.as_str() {
            "claude-code" | "claude" => adapters.push(Arc::new(ClaudeCodeAdapter)),
            "codex" => adapters.push(Arc::new(CodexAdapter)),
            "antigravity" | "agy" => adapters.push(Arc::new(AntigravityAdapter)),
            "vscode" | "copilot" => adapters.push(Arc::new(VsCodeAdapter)),
            "cursor" => adapters.push(Arc::new(CursorAdapter)),
            _ => {}
        }
    }
    adapters
}

/// Projects a [`ProjectManifest`] (optionally filtered by `profile_name`) into all enabled
/// targets with drift protection and VS Code trampoline / hardlink guarantees.
pub fn project_manifest_to_targets(
    project_root: &Path,
    manifest: &ProjectManifest,
    profile_name: Option<&str>,
    options: ReconcileOptions,
) -> IrResult<ReconcileReport> {
    let (mcp_servers, skills) = manifest.filter_for_profile(profile_name)?;
    let use_trampoline = manifest
        .targets
        .secret_delivery
        .as_deref()
        .unwrap_or("trampoline")
        == "trampoline";

    let mut state = TargetHashState::load(project_root)?;
    let mut report = ReconcileReport::default();

    let enabled: Vec<String> = if manifest.targets.enabled.is_empty() {
        vec![
            "claude-code".to_string(),
            "codex".to_string(),
            "antigravity".to_string(),
            "vscode".to_string(),
            "cursor".to_string(),
        ]
    } else {
        manifest.targets.enabled.clone()
    };

    for target in &enabled {
        let (path, content) = match target.as_str() {
            "claude-code" | "claude" => (
                project_root.join(".mcp.json"),
                render_json_mcp("mcpServers", &mcp_servers, use_trampoline, false),
            ),
            "codex" => (
                project_root.join(".codex").join("config.toml"),
                render_codex_toml(&mcp_servers, use_trampoline),
            ),
            "antigravity" | "agy" => (
                project_root.join(".agents").join("mcp_config.json"),
                render_json_mcp("mcpServers", &mcp_servers, use_trampoline, false),
            ),
            "vscode" | "copilot" => (
                project_root.join(".vscode").join("mcp.json"),
                render_json_mcp("servers", &mcp_servers, use_trampoline, true),
            ),
            "cursor" => (
                project_root.join(".cursor").join("mcp.json"),
                render_json_mcp("mcpServers", &mcp_servers, use_trampoline, false),
            ),
            other => {
                report.capability_warnings.push(format!(
                    "target `{other}` is not yet supported in Phase 1A projection (skipped)"
                ));
                continue;
            }
        };

        if !options.force && state.has_manual_drift(project_root, &path)? {
            report.files_skipped_due_to_drift.push(path);
            continue;
        }

        if !options.dry_run {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&path, content)?;
            state.record(project_root, &path)?;
        }
        report.files_written.push(path);

        // Materialize any local `path:` skills into the target's skill directory.
        let strategy = if target == "vscode" || target == "copilot" {
            LinkStrategy::CowOrHardlink
        } else {
            LinkStrategy::Symlink
        };
        for (skill_name, spec) in &skills {
            if let Some(rel_src) = spec.source.strip_prefix("path:") {
                let src_dir = project_root.join(rel_src);
                if src_dir.exists() {
                    let dst_dir = skill_destination(target, project_root, skill_name);
                    if !options.dry_run {
                        materialize_tree(&src_dir, &dst_dir, strategy)?;
                    }
                    report.files_written.push(dst_dir);
                }
            }
        }
    }

    if !options.dry_run {
        state.save(project_root)?;
    }

    Ok(report)
}

fn skill_destination(target_id: &str, project_root: &Path, skill_name: &str) -> PathBuf {
    let clean_name = skill_name.strip_prefix("skill:").unwrap_or(skill_name);
    match target_id {
        "claude-code" | "claude" => project_root.join(".claude").join("skills").join(clean_name),
        "vscode" | "copilot" => project_root.join(".github").join("skills").join(clean_name),
        "cursor" => project_root.join(".cursor").join("skills").join(clean_name),
        "antigravity" | "agy" => project_root
            .join(".agents")
            .join("plugins")
            .join(clean_name),
        _ => project_root
            .join(".infrared")
            .join("skills")
            .join(clean_name),
    }
}

fn render_json_mcp(
    top_key: &str,
    servers: &BTreeMap<String, McpServerSpec>,
    use_trampoline: bool,
    is_vscode: bool,
) -> String {
    let mut map = Map::new();
    for (name, spec) in servers {
        let entry = if spec.transport == "streamable-http" || spec.transport == "http" {
            let mut obj = Map::new();
            if is_vscode {
                obj.insert("type".to_string(), json!("http"));
            } else {
                obj.insert("type".to_string(), json!("streamable-http"));
            }
            if let Some(url) = &spec.url {
                obj.insert("url".to_string(), json!(url));
            }
            Value::Object(obj)
        } else if use_trampoline && !spec.secrets.is_empty() {
            // Zero-disk-leak trampoline: invoke `ir exec-mcp <name>` so credentials stay in OS keyring.
            json!({
                "command": "ir",
                "args": ["exec-mcp", name]
            })
        } else {
            let mut obj = Map::new();
            let command = spec.command.clone().unwrap_or_else(|| "npx".to_string());
            obj.insert("command".to_string(), json!(command));
            if !spec.args.is_empty() {
                obj.insert("args".to_string(), json!(spec.args));
            }
            if !spec.env.is_empty() {
                obj.insert("env".to_string(), json!(spec.env));
            }
            Value::Object(obj)
        };
        map.insert(name.clone(), entry);
    }

    let mut root = Map::new();
    root.insert(top_key.to_string(), Value::Object(map));
    serde_json::to_string_pretty(&Value::Object(root)).unwrap_or_else(|_| "{}".to_string()) + "\n"
}

fn render_codex_toml(servers: &BTreeMap<String, McpServerSpec>, use_trampoline: bool) -> String {
    let mut out = String::from("# Managed by infra-red (`ir sync`).\n");
    for (name, spec) in servers {
        out.push_str(&format!("\n[mcp_servers.{name}]\n"));
        if spec.transport == "streamable-http" || spec.transport == "http" {
            if let Some(url) = &spec.url {
                out.push_str(&format!("url = {url:?}\n"));
            }
        } else if use_trampoline && !spec.secrets.is_empty() {
            out.push_str("command = \"ir\"\n");
            out.push_str(&format!("args = [\"exec-mcp\", {name:?}]\n"));
        } else {
            let cmd = spec.command.as_deref().unwrap_or("npx");
            out.push_str(&format!("command = {cmd:?}\n"));
            if !spec.args.is_empty() {
                let args: Vec<String> = spec.args.iter().map(|a| format!("{a:?}")).collect();
                out.push_str(&format!("args = [{}]\n", args.join(", ")));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use ir_manifest::manifest::{ProjectMeta, SkillSpec, TargetsConfig};
    use tempfile::tempdir;

    #[test]
    fn projects_all_five_targets_with_vscode_servers_key_trampoline_and_hardlinks() {
        let dir = tempdir().unwrap();
        let local_skill = dir.path().join("local-skills").join("rust-audit");
        std::fs::create_dir_all(&local_skill).unwrap();
        std::fs::write(
            local_skill.join("SKILL.md"),
            "---\nname: rust-audit\ndescription: Audit crate.\n---\n",
        )
        .unwrap();

        let mut mcp = BTreeMap::new();
        mcp.insert(
            "github".to_string(),
            McpServerSpec {
                source: "npm:@modelcontextprotocol/server-github".to_string(),
                version: Some("1.0.0".to_string()),
                transport: "stdio".to_string(),
                command: Some("npx".to_string()),
                args: vec![
                    "-y".to_string(),
                    "@modelcontextprotocol/server-github".to_string(),
                ],
                url: None,
                env: BTreeMap::new(),
                mcp_spec: Some("2026-07-28".to_string()),
                secrets: BTreeMap::from([(
                    "GITHUB_TOKEN".to_string(),
                    "github/payments-pat".to_string(),
                )]),
                network_allowlist: vec!["api.github.com".to_string()],
                sandbox: None,
            },
        );

        let mut skills = BTreeMap::new();
        skills.insert(
            "rust-audit".to_string(),
            SkillSpec {
                source: "path:local-skills/rust-audit".to_string(),
                subdir: None,
                rev: None,
            },
        );

        let manifest = ProjectManifest {
            project: ProjectMeta {
                name: "demo".to_string(),
                schema_version: 1,
            },
            mcp,
            skills,
            targets: TargetsConfig {
                enabled: vec![
                    "claude-code".to_string(),
                    "codex".to_string(),
                    "antigravity".to_string(),
                    "vscode".to_string(),
                    "cursor".to_string(),
                ],
                secret_delivery: Some("trampoline".to_string()),
            },
            ..Default::default()
        };

        let report =
            project_manifest_to_targets(dir.path(), &manifest, None, ReconcileOptions::default())
                .unwrap();
        assert!(report.files_skipped_due_to_drift.is_empty());

        // VS Code uses top-level `servers` key + `ir exec-mcp` trampoline + non-symlink skill dir.
        let vscode_mcp: Value = serde_json::from_str(
            &std::fs::read_to_string(dir.path().join(".vscode/mcp.json")).unwrap(),
        )
        .unwrap();
        assert!(vscode_mcp.get("servers").is_some());
        assert!(vscode_mcp.get("mcpServers").is_none());
        assert_eq!(vscode_mcp["servers"]["github"]["command"], "ir");
        assert_eq!(
            vscode_mcp["servers"]["github"]["args"],
            json!(["exec-mcp", "github"])
        );

        let vscode_skill = dir.path().join(".github/skills/rust-audit/SKILL.md");
        assert!(vscode_skill.exists());
        assert!(
            !std::fs::symlink_metadata(&vscode_skill)
                .unwrap()
                .file_type()
                .is_symlink()
        );

        // Codex uses TOML `[mcp_servers.github]`.
        let codex_toml = std::fs::read_to_string(dir.path().join(".codex/config.toml")).unwrap();
        assert!(codex_toml.contains("[mcp_servers.github]"));

        // Manual edit drift protection: editing `.mcp.json` prevents clobbering without `--force`.
        std::fs::write(
            dir.path().join(".mcp.json"),
            "{\"mcpServers\": {\"hand-edited\": {}}}\n",
        )
        .unwrap();
        let second =
            project_manifest_to_targets(dir.path(), &manifest, None, ReconcileOptions::default())
                .unwrap();
        assert_eq!(second.files_skipped_due_to_drift.len(), 1);
        assert!(
            std::fs::read_to_string(dir.path().join(".mcp.json"))
                .unwrap()
                .contains("hand-edited")
        );
    }
}
