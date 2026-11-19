//! Ticket `IR-102`: `SKILL.md` frontmatter parser and agentskills.io specification linter.

use ir_core::IrResult;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMetadata {
    pub name: String,
    pub description: String,
    pub license: Option<String>,
    pub compatibility: Option<String>,
    #[serde(default)]
    pub allowed_tools: Vec<String>,
    pub line_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkillWarning {
    ProgressiveDisclosureExceeded {
        lines: usize,
        recommended_max: usize,
    },
}

pub fn parse_and_validate_skill_dir(
    _skill_dir: &Path,
) -> IrResult<(SkillMetadata, Vec<SkillWarning>)> {
    todo!(
        "IR-102: Validate SKILL.md YAML frontmatter, directory name match, and 500-line lint rule"
    )
}
