//! Ticket `IR-102`: `SKILL.md` frontmatter parser and agentskills.io specification linter.

use ir_core::{IrError, IrResult};
use serde::{Deserialize, Serialize};
use std::path::Path;

const RECOMMENDED_MAX_SKILL_LINES: usize = 500;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

/// Validates a `SKILL.md` directory against the agentskills.io specification:
/// - `SKILL.md` must exist with YAML frontmatter delimited by `---`
/// - `name`: 1..=64 chars, lowercase ASCII alphanumeric + hyphens, must match parent directory name
/// - `description`: 1..=1024 chars
/// - `compatibility`: optional, <= 500 chars
/// - Emits a non-fatal warning if `SKILL.md` exceeds 500 lines
pub fn parse_and_validate_skill_dir(
    skill_dir: &Path,
) -> IrResult<(SkillMetadata, Vec<SkillWarning>)> {
    let skill_file = skill_dir.join("SKILL.md");
    let content = std::fs::read_to_string(&skill_file).map_err(|_| IrError::InvalidSkill {
        path: skill_file.clone(),
        reason: "missing SKILL.md".to_string(),
    })?;

    let normalized = content.replace("\r\n", "\n");
    let line_count = normalized.lines().count();

    let frontmatter = extract_frontmatter(&normalized).ok_or_else(|| IrError::InvalidSkill {
        path: skill_file.clone(),
        reason: "SKILL.md must begin with YAML frontmatter delimited by `---`".to_string(),
    })?;

    let mut name: Option<String> = None;
    let mut description: Option<String> = None;
    let mut license: Option<String> = None;
    let mut compatibility: Option<String> = None;
    let mut allowed_tools: Vec<String> = Vec::new();

    for line in frontmatter.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some((raw_key, raw_val)) = trimmed.split_once(':') {
            let key = raw_key.trim();
            let val = unquote(raw_val.trim());
            match key {
                "name" => name = Some(val),
                "description" => description = Some(val),
                "license" => license = Some(val),
                "compatibility" => compatibility = Some(val),
                "allowed-tools" | "allowed_tools" => {
                    allowed_tools = val
                        .trim_matches(|c| c == '[' || c == ']')
                        .split([',', ' '])
                        .map(|s| unquote(s.trim()))
                        .filter(|s| !s.is_empty())
                        .collect();
                }
                _ => {}
            }
        }
    }

    let name = name.ok_or_else(|| IrError::InvalidSkill {
        path: skill_file.clone(),
        reason: "frontmatter missing required `name` field".to_string(),
    })?;
    let description = description.ok_or_else(|| IrError::InvalidSkill {
        path: skill_file.clone(),
        reason: "frontmatter missing required `description` field".to_string(),
    })?;

    if name.is_empty()
        || name.len() > 64
        || !name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        return Err(IrError::InvalidSkill {
            path: skill_file.clone(),
            reason: format!(
                "invalid skill name `{name}`: must be 1..=64 lowercase alphanumeric or hyphen characters"
            ),
        });
    }

    if let Some(dir_name) = skill_dir.file_name().and_then(|s| s.to_str())
        && dir_name != name
    {
        return Err(IrError::InvalidSkill {
            path: skill_file.clone(),
            reason: format!("skill name `{name}` must match parent directory name `{dir_name}`"),
        });
    }

    if description.is_empty() || description.len() > 1024 {
        return Err(IrError::InvalidSkill {
            path: skill_file.clone(),
            reason: "skill `description` must be 1..=1024 characters".to_string(),
        });
    }

    if let Some(compat) = &compatibility
        && compat.len() > 500
    {
        return Err(IrError::InvalidSkill {
            path: skill_file,
            reason: "skill `compatibility` must be at most 500 characters".to_string(),
        });
    }

    let mut warnings = Vec::new();
    if line_count > RECOMMENDED_MAX_SKILL_LINES {
        warnings.push(SkillWarning::ProgressiveDisclosureExceeded {
            lines: line_count,
            recommended_max: RECOMMENDED_MAX_SKILL_LINES,
        });
    }

    Ok((
        SkillMetadata {
            name,
            description,
            license,
            compatibility,
            allowed_tools,
            line_count,
        },
        warnings,
    ))
}

fn extract_frontmatter(doc: &str) -> Option<&str> {
    let rest = doc.strip_prefix("---\n")?;
    let end = rest.find("\n---")?;
    Some(&rest[..end])
}

fn unquote(val: &str) -> String {
    val.trim_matches(|c| c == '"' || c == '\'').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn validates_compliant_skill_dir() {
        let root = tempdir().unwrap();
        let skill_dir = root.path().join("rust-security-audit");
        std::fs::create_dir_all(&skill_dir).unwrap();
        std::fs::write(
            skill_dir.join("SKILL.md"),
            "---\nname: rust-security-audit\ndescription: Audits Rust crates for CVEs.\nlicense: MIT\nallowed-tools: [bash, read_file]\n---\n# Instructions\nRun cargo audit.\n",
        )
        .unwrap();

        let (meta, warnings) = parse_and_validate_skill_dir(&skill_dir).unwrap();
        assert_eq!(meta.name, "rust-security-audit");
        assert_eq!(meta.allowed_tools, vec!["bash", "read_file"]);
        assert!(warnings.is_empty());
    }

    #[test]
    fn warns_when_skill_exceeds_500_lines() {
        let root = tempdir().unwrap();
        let skill_dir = root.path().join("big-skill");
        std::fs::create_dir_all(&skill_dir).unwrap();
        let body = "line\n".repeat(505);
        std::fs::write(
            skill_dir.join("SKILL.md"),
            format!("---\nname: big-skill\ndescription: Large skill.\n---\n{body}"),
        )
        .unwrap();

        let (_meta, warnings) = parse_and_validate_skill_dir(&skill_dir).unwrap();
        assert_eq!(warnings.len(), 1);
        assert!(matches!(
            warnings[0],
            SkillWarning::ProgressiveDisclosureExceeded { .. }
        ));
    }

    #[test]
    fn rejects_mismatched_directory_name() {
        let root = tempdir().unwrap();
        let skill_dir = root.path().join("wrong-folder");
        std::fs::create_dir_all(&skill_dir).unwrap();
        std::fs::write(
            skill_dir.join("SKILL.md"),
            "---\nname: right-name\ndescription: Desc.\n---\n",
        )
        .unwrap();

        let err = parse_and_validate_skill_dir(&skill_dir).unwrap_err();
        assert!(matches!(err, IrError::InvalidSkill { .. }));
    }
}
