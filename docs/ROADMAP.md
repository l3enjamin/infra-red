# Infra-Red Project Roadmap & Quick-Win Strategy

## Why Phase 1A Is the Top Quick Win

Developers rotating across `claude`, `codex`, `agy`, VS Code Copilot, and Cursor hit two immediate daily walls:
1. **Config Drift & Broken Symlinks**: Maintaining 5 incompatible MCP/skill config schemas in the same repo.
2. **Upfront Context-Window Token Bloat**: Globally attached MCP servers consume 55K–143K tokens before the first prompt.

By shipping **Phase 1A (`ir migrate`, `ir sync`, `ir doctor`)** first—before full SAT solving or kernel sandboxing—users get immediate value in under 60 seconds on any existing repository while establishing `infra-red.toml` as the single source of truth.

---

## Phased Milestones & Exit Verification Gates

| Milestone | Target Window | Tickets | User Value Delivered | Exit Verification Gate |
| :--- | :---: | :--- | :--- | :--- |
| **Phase 1A: Quick-Win MVP** | Weeks 1–2 | `IR-101` – `IR-107` | `ir migrate` imports existing `.mcp.json`, `.codex/config.toml`, `.agents/`, `.vscode/`, `.cursor/`; `ir sync` reconciles all 5 targets with drift detection and VS Code hardlink/reflink safety; `ir doctor` audits token bloat & plaintext secrets. | `ir migrate` + `ir sync --frozen` pass CI on Linux, macOS, and Windows; VS Code `.github/skills/` contains zero symlinks; hand-edited target files are not clobbered without `--force`. |
| **Phase 1B: CAS, Fetchers & SAT Solver** | Weeks 3–4 | `IR-108` – `IR-110` | `ir add` fetches MCP servers from the Official MCP Registry and skills/plugins via pure-Rust `gix` into the versioned `v1` CAS; `pubgrub` solves dependencies with `__os`/`__agent_*`/`__mcp_spec` virtual packages; `ir trust` quarantines repo hooks. | `cargo test -p ir-resolver -p ir-store` passes conflict proof-chain and CAS deduplication tests; untrusted hooks cannot project until `ir trust`. |
| **Phase 2: Secrets & Stateful Venvs** | Weeks 4–5 | `IR-201` – `IR-204` | `ir secret set` + `ir exec-mcp` remove all plaintext API keys from IDE configs; `ir venv` + `ir run --profile <name>` isolate `.infrared/` config roots (`CLAUDE_CONFIG_DIR`, `CODEX_HOME`, `agy`). | Zero plaintext secrets in repo configs; concurrent sessions in two repos show zero MCP or transcript cross-contamination. |
| **Phase 3: Sandboxing & Ephemeral `irx`** | Weeks 6–8 | `IR-301` – `IR-305` | OS-native confinement (`bwrap`+`seccomp`, `/usr/bin/sandbox-exec`, Windows Job Objects) with `process-wrap` reaping; `irx --with ...` one-shot CLI; `irx serve-mcp` stateless MCP `2026-07-28` server with MCP Tasks. | Stress test proves zero orphaned grandchild processes on SIGINT/timeout and zero leftover temp dirs; parent agent delegates a scoped sub-task to `irx serve-mcp`. |
| **Phase 4: ACP Conductor & Egress Proxy** | Weeks 9–10 | `IR-401` – `IR-403` | `ir acp-serve` dynamically injects locked MCP servers into Zed and JetBrains (with absolute binary paths); Tier-2 loopback HTTP/SOCKS5 proxy swaps placeholder secrets only on allowlisted domains post-policy DNS resolution. | Zed and IntelliJ drive `claude-agent-acp`, `codex-acp`, and `agy acp` via `ir acp-serve`; egress test verifies placeholder tokens cannot leak to non-allowlisted domains. |
