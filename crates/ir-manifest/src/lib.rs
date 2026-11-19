//! `ir-manifest` handles parsing, validation, and comment-preserving edits for:
//! - `infra-red.toml` & `infra-red.local.toml` (`IR-101`)
//! - `infra-red.lock` deterministic text lockfile (`IR-101`)
//! - `SKILL.md` frontmatter validation per agentskills.io (`IR-102`)
//! - Multi-component plugins (`.claude-plugin/plugin.json`, `.agents/plugins/`) (`IR-102`)

pub mod lockfile;
pub mod manifest;
pub mod plugin;
pub mod skill;
