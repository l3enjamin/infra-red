#!/usr/bin/env bash
set -euo pipefail

# Verifies the strict 4-layer acyclic workspace dependency rules defined in docs/ARCHITECTURE.md.
# Runs locally for junior developers and in GitHub Actions CI.

cargo metadata --no-deps --format-version 1 | python3 -c '
import json, sys

meta = json.load(sys.stdin)
workspace_members = set(meta["workspace_members"])
packages = {p["id"]: p for p in meta["packages"] if p["id"] in workspace_members}
name_by_id = {pid: p["name"] for pid, p in packages.items()}
workspace_names = set(name_by_id.values())

LAYERS = {
    # Layer 0: Foundation contracts & test doubles (0 internal deps)
    "ir-core": 0,
    # Layer 1: Independent domain engines (may only depend on L0)
    "ir-manifest": 1,
    "ir-resolver": 1,
    "ir-store": 1,
    "ir-secrets": 1,
    "ir-sandbox": 1,
    # Layer 2: Orchestration, projections & protocol bridges (may depend on L0-L1)
    "ir-targets": 2,
    "ir-runtime": 2,
    "ir-mcp-server": 2,
    "ir-acp": 2,
    # Layer 3: CLI binaries (may depend on L0-L2)
    "infra-red": 3,
    "irx": 3,
}

# Explicit allowed L2 -> L2 dependencies if ever needed (currently strictly L2 -> L0/L1 only, except ir-mcp-server/ir-acp -> ir-runtime)
ALLOWED_SAME_LAYER = {
    ("ir-mcp-server", "ir-runtime"),
    ("ir-acp", "ir-runtime"),
}

errors = []
for pkg in packages.values():
    name = pkg["name"]
    if name not in LAYERS:
        errors.append(f"Unclassified workspace crate `{name}`: assign it a layer in scripts/verify-architecture.sh and docs/ARCHITECTURE.md")
        continue
    src_layer = LAYERS[name]

    for dep in pkg["dependencies"]:
        dep_name = dep["name"]
        # Only enforce on normal (non-dev) internal workspace dependencies
        if dep_name not in workspace_names or dep.get("kind") == "dev":
            continue
        dst_layer = LAYERS.get(dep_name)
        if dst_layer is None:
            continue
        if dst_layer > src_layer:
            errors.append(
                f"Layer violation: `{name}` (L{src_layer}) cannot depend on higher-layer crate `{dep_name}` (L{dst_layer})"
            )
        elif dst_layer == src_layer and (name, dep_name) not in ALLOWED_SAME_LAYER:
            errors.append(
                f"Same-layer violation: `{name}` (L{src_layer}) cannot depend on peer crate `{dep_name}` (L{dst_layer})"
            )

if errors:
    print("ARCHITECTURE VERIFICATION FAILED:", file=sys.stderr)
    for err in errors:
        print(f"  - {err}", file=sys.stderr)
    sys.exit(1)

print(f"Architecture check passed for {len(packages)} workspace crates (L0 -> L1 -> L2 -> L3).")
'
