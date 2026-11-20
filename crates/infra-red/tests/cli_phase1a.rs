use std::process::Command;
use tempfile::tempdir;

fn ir_bin() -> &'static str {
    env!("CARGO_BIN_EXE_ir")
}

#[test]
fn end_to_end_init_sync_frozen_and_doctor() {
    let dir = tempdir().unwrap();

    // 1. `ir init`
    let init = Command::new(ir_bin())
        .args(["--project-dir", dir.path().to_str().unwrap(), "init"])
        .output()
        .unwrap();
    assert!(
        init.status.success(),
        "init stderr: {}",
        String::from_utf8_lossy(&init.stderr)
    );
    assert!(dir.path().join("infra-red.toml").exists());
    let gitignore = std::fs::read_to_string(dir.path().join(".gitignore")).unwrap();
    assert!(gitignore.contains(".infrared/"));

    // 2. `ir sync`
    let sync = Command::new(ir_bin())
        .args(["--project-dir", dir.path().to_str().unwrap(), "sync"])
        .output()
        .unwrap();
    assert!(
        sync.status.success(),
        "sync stderr: {}",
        String::from_utf8_lossy(&sync.stderr)
    );
    assert!(dir.path().join("infra-red.lock").exists());
    assert!(dir.path().join(".mcp.json").exists());
    assert!(dir.path().join(".codex/config.toml").exists());
    assert!(dir.path().join(".agents/mcp_config.json").exists());
    assert!(dir.path().join(".vscode/mcp.json").exists());
    assert!(dir.path().join(".cursor/mcp.json").exists());

    // 3. `ir sync --frozen` succeeds against matching lockfile
    let frozen = Command::new(ir_bin())
        .args([
            "--project-dir",
            dir.path().to_str().unwrap(),
            "sync",
            "--frozen",
        ])
        .output()
        .unwrap();
    assert!(
        frozen.status.success(),
        "frozen stderr: {}",
        String::from_utf8_lossy(&frozen.stderr)
    );

    // 4. `ir doctor` succeeds
    let doctor = Command::new(ir_bin())
        .args(["--project-dir", dir.path().to_str().unwrap(), "doctor"])
        .output()
        .unwrap();
    assert!(doctor.status.success());
    assert!(String::from_utf8_lossy(&doctor.stdout).contains("Infra-Red Doctor Audit"));
}

#[test]
fn end_to_end_migrate_scrubs_secrets_and_projects_all_targets() {
    let dir = tempdir().unwrap();
    std::fs::write(
        dir.path().join(".mcp.json"),
        r#"{
            "mcpServers": {
                "github": {
                    "command": "npx",
                    "args": ["-y", "@modelcontextprotocol/server-github"],
                    "env": {
                        "GITHUB_TOKEN": "ghp_supersecret12345"
                    }
                }
            }
        }"#,
    )
    .unwrap();

    let migrate = Command::new(ir_bin())
        .args(["--project-dir", dir.path().to_str().unwrap(), "migrate"])
        .output()
        .unwrap();
    assert!(
        migrate.status.success(),
        "migrate stderr: {}",
        String::from_utf8_lossy(&migrate.stderr)
    );
    let stdout = String::from_utf8_lossy(&migrate.stdout);
    assert!(stdout.contains("Migrated 1 server(s)"));
    assert!(stdout.contains("ir secret set github/github_token"));

    let manifest = std::fs::read_to_string(dir.path().join("infra-red.toml")).unwrap();
    assert!(!manifest.contains("ghp_supersecret12345"));
    assert!(manifest.contains("github/github_token"));

    // Syncing after migrate rewrites `.mcp.json` cleanly with `ir exec-mcp github` trampoline!
    let sync = Command::new(ir_bin())
        .args(["--project-dir", dir.path().to_str().unwrap(), "sync"])
        .output()
        .unwrap();
    assert!(
        sync.status.success(),
        "sync stderr: {}",
        String::from_utf8_lossy(&sync.stderr)
    );
    let mcp_json = std::fs::read_to_string(dir.path().join(".mcp.json")).unwrap();
    assert!(!mcp_json.contains("ghp_supersecret12345"));
    assert!(mcp_json.contains("exec-mcp"));
}
