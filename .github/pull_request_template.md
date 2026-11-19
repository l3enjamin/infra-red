## Summary

- **Ticket ID**: `IR-XXX` (see `docs/JUNIOR_DEV_TASKS.md`)
- **Architectural Layer(s) Touched**: `[ ] L0 (ir-core)` `[ ] L1` `[ ] L2` `[ ] L3 (CLI)` `[ ] Docs / CI`
- **What Changed**:
  - <!-- Concise bullet summary of changes -->

## Pre-Merge Checklist

- [ ] **Architecture Boundaries**: `./scripts/verify-architecture.sh` passes (no circular or cross-layer crate dependencies).
- [ ] **Formatting & Lints**: `cargo fmt --all -- --check` and `cargo clippy --workspace --all-targets --locked -- -D warnings` pass locally.
- [ ] **Unit Tests & Mocks**: Added unit tests using `ir_core::testing` mocks (`InMemorySecretProvider`, `NoopSandboxBackend`, `StubArtifactSource`, `MockAgentRuntime`) where applicable; `cargo test --workspace` passes.
- [ ] **Security & Cross-Platform Guardrails**:
  - [ ] Zero plaintext secrets written to repository files or printed in `Debug`/`Display` output.
  - [ ] No symlinks created for symlink-incompatible targets (e.g., VS Code Copilot uses `LinkStrategy::CowOrHardlink`).
  - [ ] No C-linked dependencies (`openssl-sys`, `libgit2-sys`) or `GPL`/`LGPL` crates added to default features.
