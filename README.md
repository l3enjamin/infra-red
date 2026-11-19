# infra-red (`ir` & `irx`)

**infra-red** is a cross-platform Rust package manager, environment isolator, and protocol-bridging orchestrator for AI coding tooling.

From a single declarative manifest (`infra-red.toml`) and deterministic lockfile (`infra-red.lock`), `infra-red` manages four artifact classes—**MCP servers**, **`SKILL.md` Agent Skills**, **multi-component Plugins**, and **Agent CLI Runtimes**—and projects capability-matched configs into **Claude Code**, **OpenAI Codex**, **Google Antigravity (`agy`)**, **VS Code / Copilot**, **Cursor**, **Zed**, and **JetBrains IDEs**.

## Quick-Win Value Proposition

1. **Zero Config Drift (`ir migrate` + `ir sync`)**: Import scattered `.mcp.json`, `.codex/config.toml`, `.agents/mcp_config.json`, `.vscode/mcp.json`, and `.cursor/mcp.json` files into one `infra-red.toml` and project them cleanly across every tool (automatically using CoW reflinks/hardlinks for VS Code instead of broken symlinks).
2. **Stop Context-Window Token Bloat (`ir doctor` + `--profile`)**: Measure upfront MCP token tax and scope tools per project (`.infrared/`) or per task (`--profile pr-review`) so global MCP servers never burn scarce quotas.
3. **Zero Plaintext Secrets on Disk (`ir secret` + `ir exec-mcp`)**: Store credentials in the OS keyring and inject them in memory at runtime via a lightweight stdio trampoline.
4. **OS-Sandboxed Sub-Agent Delegation (`irx` + `irx serve-mcp`)**: Run stateless, one-shot isolated agent tasks with atomic process-tree reaping (`process-wrap`) and expose them to parent agents as a stateless MCP `2026-07-28` server.

## Workspace Layout

| Layer | Crate | Purpose |
| :---: | :--- | :--- |
| **L0** | [`crates/ir-core`](crates/ir-core) | Domain types, typed errors (`IrError`), 6 extensibility traits, and in-memory test mocks (`ir_core::testing`). |
| **L1** | [`crates/ir-manifest`](crates/ir-manifest) | `infra-red.toml`, `infra-red.lock`, `SKILL.md`, and plugin manifest parsing + `toml_edit` edits. |
| **L1** | [`crates/ir-resolver`](crates/ir-resolver) | PubGrub SAT solver with host virtual packages (`__os`, `__agent_*`, `__mcp_spec`). |
| **L1** | [`crates/ir-store`](crates/ir-store) | Versioned global CAS (`v1`), CoW/hardlink materializer, pure-Rust `gix` & MCP Registry fetchers. |
| **L1** | [`crates/ir-secrets`](crates/ir-secrets) | OS keyring vault (`keyring-core` + `zeroize`) and Tier-2 loopback secret-swapping proxy. |
| **L1** | [`crates/ir-sandbox`](crates/ir-sandbox) | Linux `bwrap`+`seccomp`, macOS `/usr/bin/sandbox-exec`, Windows Job Objects, and `process-wrap` reaping. |
| **L2** | [`crates/ir-targets`](crates/ir-targets) | Declarative IDE & agent config projections, drift detection, and `ir migrate`. |
| **L2** | [`crates/ir-runtime`](crates/ir-runtime) | `.infrared/` isolated project environments (`CLAUDE_CONFIG_DIR`, `CODEX_HOME`, `agy`) and stream-JSON parser. |
| **L2** | [`crates/ir-acp`](crates/ir-acp) | ACP v1/v2 Conductor Proxy (`ir acp-serve`) and Client for Zed & JetBrains. |
| **L2** | [`crates/ir-mcp-server`](crates/ir-mcp-server) | Stateless MCP `2026-07-28` server (`irx serve-mcp`) with MCP Tasks support. |
| **L3** | [`crates/infra-red`](crates/infra-red) | Primary CLI binary (`ir`). |
| **L3** | [`crates/irx`](crates/irx) | Ephemeral one-shot runner & MCP server binary (`irx`). |

## Developer Onboarding & Documentation

- **[Architecture & Dependency Rules](docs/ARCHITECTURE.md)**: Layering invariants, trait contracts, target projection matrix, and security boundaries.
- **[Project Roadmap & Quick-Win Milestones](docs/ROADMAP.md)**: Phased delivery plan (`Phase 1A Quick-Win MVP` -> `Phase 4`) and exit verification gates.
- **[Junior Developer Task Board](docs/JUNIOR_DEV_TASKS.md)**: Self-contained tickets (`IR-101`–`IR-403`) with exact file paths, Rust signatures, edge-case checklists, and test commands.

## Build & Test

```bash
# Requires Rust 1.92+ (Edition 2024)
cargo check --workspace --all-targets
cargo test --workspace
```