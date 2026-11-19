# Junior Developer Task Board (`IR-101` – `IR-403`)

Every ticket below maps directly to `todo!("IR-XXX: ...")` anchors in the workspace.
Use the in-memory test doubles in `crates/ir-core/src/testing.rs` (`InMemorySecretProvider`, `NoopSandboxBackend`, `StubArtifactSource`, `MockAgentRuntime`) so your unit tests run without network access, OS keyrings, or kernel sandbox privileges.

---

## Phase 1A: Quick-Win MVP (Start Here — Parallelizable)

### `IR-101`: `infra-red.toml` & `infra-red.lock` Parser + `toml_edit` Editor
- **Crate**: `crates/ir-manifest` (`src/manifest.rs`, `src/lockfile.rs`)
- **Depends On**: `ir-core` (ready)
- **Tasks**:
  1. Implement `ProjectManifest::from_file` and `load_with_local_override` (merging optional `infra-red.local.toml`).
  2. Reject literal secret patterns (`ghp_`, `sk-`, `AIza`, `xoxb-`) in `[mcp.<name>.secrets]` with `IrError::PlaintextSecretDetected`.
  3. Implement `Lockfile::from_file`, `write_to_file`, and canonical sorted manifest SHA-256 digesting (normalizing CRLF -> LF).
  4. Add comment-preserving `upsert_mcp_entry` using `toml_edit`.
- **Verify**: `cargo test -p ir-manifest`

### `IR-102`: `SKILL.md` Frontmatter Validator & Plugin Parser
- **Crate**: `crates/ir-manifest` (`src/skill.rs`, `src/plugin.rs`)
- **Depends On**: `ir-core` (ready)
- **Tasks**:
  1. Parse `SKILL.md` YAML frontmatter per [agentskills.io](https://agentskills.io/specification): `name` (`^[a-z0-9-]+$`, 1–64 chars, must match parent dir), `description` (1–1024 chars), optional `compatibility` (<= 500 chars).
  2. Emit `SkillWarning::ProgressiveDisclosureExceeded` when `SKILL.md` > 500 lines.
  3. Parse `.claude-plugin/plugin.json`, `.plugin/plugin.json`, and `.agents/plugins/<name>/plugin.json` into `CanonicalPlugin`.
- **Verify**: `cargo test -p ir-manifest`

### `IR-103`: CoW Reflink / Hardlink Materializer & Tree Digest
- **Crate**: `crates/ir-store` (`src/lib.rs` -> split into `src/linker.rs`, `src/digest.rs`)
- **Depends On**: `ir-core` (ready)
- **Tasks**:
  1. Implement `materialize_tree(src, dst, strategy)` with ladder: `reflink_copy::reflink` -> `std::fs::hard_link` -> `std::fs::copy` when `LinkStrategy::CowOrHardlink`.
  2. Guarantee **zero symlinks** are created under `CowOrHardlink` (required for VS Code Copilot).
  3. Implement deterministic sorted directory tree SHA-256 (`sha256_path`).
- **Verify**: `cargo test -p ir-store`

### `IR-104`: CLI Agent Target Adapters (`claude-code`, `codex`, `antigravity`) + Drift Tracker
- **Crate**: `crates/ir-targets` (`src/{claude.rs, codex.rs, antigravity.rs, drift.rs}`)
- **Depends On**: `IR-101`, `IR-103`
- **Tasks**:
  1. Implement `TargetAdapter` for Claude Code (`.mcp.json`, `.claude/skills/`), Codex (`.codex/config.toml`, `AGENTS.md`), and Antigravity (`.agents/mcp_config.json`, `.agents/plugins/`, `.agents/hooks.json` only if trusted).
  2. Track SHA-256 hashes of emitted files in `.infrared/state/target-hashes.json` and refuse to overwrite hand-edited files unless `--force` is set.
- **Verify**: `cargo test -p ir-targets`

### `IR-105`: IDE Target Adapters (`vscode`, `cursor`)
- **Crate**: `crates/ir-targets` (`src/{vscode.rs, cursor.rs}`)
- **Depends On**: `IR-101`, `IR-103`
- **Tasks**:
  1. `VsCodeAdapter`: `supports_symlinks() -> false`, emit `.vscode/mcp.json` with top-level `"servers"` key and `${workspaceFolder}`, materialize `.github/skills/<name>/SKILL.md` via `CowOrHardlink`, rewrite secret-bearing servers to `ir exec-mcp <name>`.
  2. `CursorAdapter`: emit `.cursor/mcp.json` (`mcpServers`), link `.cursor/skills/<name>/SKILL.md`, emit non-fatal warnings for Claude-only hooks.
- **Verify**: `cargo test -p ir-targets`

### `IR-106`: `ir migrate` Importer & `ir doctor` Token-Bloat / Drift Auditor
- **Crates**: `crates/ir-targets`, `crates/infra-red`
- **Depends On**: `IR-101`, `IR-104`, `IR-105`
- **Tasks**:
  1. Scan existing `.mcp.json`, `.codex/config.toml`, `.agents/mcp_config.json`, `.vscode/mcp.json`, `.cursor/mcp.json` and merge into `infra-red.toml`, extracting plaintext secret values into logical keyring references.
  2. Implement `ir doctor` to estimate upfront MCP token overhead, detect target config drift, flag plaintext secrets, and warn on deprecated MCP `2025-11-25` transports.
- **Verify**: `cargo test -p infra-red`

### `IR-107`: Primary CLI (`ir init`, `ir sync`)
- **Crate**: `crates/infra-red` (`src/main.rs` + `src/commands/`)
- **Depends On**: `IR-104`, `IR-105`
- **Tasks**:
  1. Implement `ir init` (creates starter `infra-red.toml`, updates `.gitignore`).
  2. Implement `ir sync [--frozen] [--dry-run] [--force] [--profile <name>]`.
- **Verify**: `cargo test -p infra-red`

---

## Phase 1B: CAS, Remote Fetchers & PubGrub SAT Solver

- **`IR-108` (`crates/ir-store`)**: Implement versioned `v1` CAS under `etcetera` cache dirs, pure-Rust `gix` Git fetcher with 40-char SHA pinning + `subdir` extraction, and Official MCP Registry HTTP client (`server.json`).
- **`IR-109` (`crates/ir-resolver`)**: Implement `pubgrub::solver::DependencyProvider` with host virtual packages (`__os`, `__arch`, `__agent_*`, `__mcp_spec`, `__sandbox`) and human-readable conflict diagnostics.
- **`IR-110` (`crates/infra-red`)**: Wire `ir add`, `ir remove`, and `ir trust` quarantine approval workflow.

---

## Phase 2: Secrets & Stateful `.infrared/` Environments

- **`IR-201` (`crates/ir-secrets`)**: Implement OS-native `keyring-core` provider + `CiEnvSecretProvider` with `Zeroizing<String>` and `[REDACTED]` debug formatting.
- **`IR-202` (`crates/infra-red`)**: Implement `ir secret set|list|delete` and `ir exec-mcp <server>` stdio trampoline.
- **`IR-203` (`crates/ir-runtime`)**: Implement `.infrared/` layout, `CLAUDE_CONFIG_DIR` / `CODEX_HOME` / `agy` isolation, `global` vs `venv` runtime modes, and stream-JSON parser.
- **`IR-204` (`crates/infra-red`)**: Wire `ir venv` and `ir run [agent] --profile <name>`.

---

## Phase 3: Sandboxing & Ephemeral `irx`

- **`IR-301` (`crates/ir-sandbox`)**: Implement `process-wrap` process-group / Job Object reaping (`SIGTERM` -> grace wait -> `SIGKILL`) with orphan-process stress tests.
- **`IR-302` (`crates/ir-sandbox`)**: Implement Linux `bwrap`+`seccomp` (with `.git`/`.infrared` read-only re-binds and Ubuntu AppArmor userns diagnostic) and macOS `/usr/bin/sandbox-exec` SBPL generator.
- **`IR-303` (`crates/ir-sandbox`)**: Implement Windows Restricted Token + Job Object backend (with MXC trait seam) and opt-in `--sandbox=oci` backend.
- **`IR-304` (`crates/ir-mcp-server`)**: Wire `IrxMcpServer` (`irx_list_catalog`, `irx_run_isolated`) over `rmcp` stdio + Streamable HTTP with MCP Tasks support.
- **`IR-305` (`crates/irx`)**: Wire `irx --agent ... --with ...` one-shot CLI and `irx serve-mcp`.

---

## Phase 4: ACP Conductor & Tier-2 Egress Proxy

- **`IR-401` (`crates/ir-acp`)**: Implement ACP v1 Conductor (`ir acp-serve`) dynamically injecting locked MCP servers into `session/new` / `session/load` and handling `agy acp` auth handshake.
- **`IR-402` (`crates/ir-targets`)**: Implement Zed and JetBrains adapters (enforcing absolute binary paths for JetBrains).
- **`IR-403` (`crates/ir-secrets`)**: Implement loopback HTTP/SOCKS5 secret-swapping proxy with post-policy DNS resolution.
