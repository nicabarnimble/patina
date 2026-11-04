#!/usr/bin/env python3
"""Audit active rustc dep-info, not stale artifacts or runtime cfg! branches.

Usage: cargo build --locked --message-format=json > default-build.json
       python3 resources/scripts/check-default-build.py default-build.json
Run only against the default build's separate target directory.
"""
import json
import pathlib
import sys

root = pathlib.Path(__file__).resolve().parents[2]
expected_manifests = {root / "Cargo.toml", root / "mother/Cargo.toml"}
artifacts = []
for line in pathlib.Path(sys.argv[1]).read_text().splitlines():
    message = json.loads(line)
    if message.get("reason") != "compiler-artifact":
        continue
    if pathlib.Path(message["manifest_path"]).resolve() not in expected_manifests:
        continue
    if not set(message["target"]["kind"]) & {"lib", "bin"}:
        continue
    assert not set(message["features"]) & {"legacy-runtime", "full", "bundled-doctor"}, message
    artifacts.append(message)
assert len(artifacts) == 3, "expected root lib, root bin and shared Mother lib"

sources = set()
for artifact in artifacts:
    depfiles = set()
    for name in artifact["filenames"]:
        path = pathlib.Path(name)
        if path.suffix in {".rlib", ".rmeta"}:
            path = path.with_name(path.stem.removeprefix("lib") + ".d")
        else:
            path = path.with_suffix(".d")
        if path.exists():
            depfiles.add(path)
    assert depfiles, f"missing dep-info for {artifact['target']['name']}"
    for path in depfiles:
        for token in path.read_text().replace("\\\n", " ").split():
            candidate = pathlib.Path(token.rstrip(":"))
            if candidate.suffix != ".rs":
                continue
            # rustc can emit either absolute or package-relative source paths.
            if not candidate.is_absolute():
                candidate = pathlib.Path(artifact["manifest_path"]).parent / candidate
            try:
                sources.add(candidate.resolve().relative_to(root).as_posix())
            except ValueError:
                pass
    print(f"{artifact['target']['kind'][0]} {artifact['target']['name']}: features={artifact['features']}")

required = {
    "src/session/internal/live.rs", "src/commands/session/internal.rs",
    "src/child/internal/pipeline.rs", "src/child/internal/passive.rs",
    "mother/src/state/mod.rs", "mother/src/eventlog_schema.rs",
    "src/commands/scry/internal/routing.rs", "src/interface/core.rs",
    "src/commands/scrape/code/mod.rs", "src/commands/oxidize/mod.rs",
    "src/commands/assay/mod.rs", "src/commands/setup/grammars.rs",
    "src/beliefs/mod.rs", "src/models/mod.rs", "src/embeddings/mod.rs",
}
assert required <= sources, f"missing shared sources: {sorted(required - sources)}"
forbidden = (
    "src/preflight.rs", "src/mother/client.rs", "src/mother/internal.rs",
    "src/main_dispatch/mother.rs", "src/main_dispatch/child.rs", "src/main_dispatch/spec.rs",
    "src/commands/doctor/",
    "src/mother/broker/", "src/mother/doctor_runtime.rs", "src/mother/skills/",
    "src/child/internal/child.rs", "src/child/internal/host_support.rs",
    "src/child/internal/typed_conversion.rs", "src/child/toy_host/", "src/child/scaffold/",
    "src/interface/internal/launcher.rs", "src/interface/internal/bootstrap.rs",
    "src/interface/internal/surface.rs", "src/interface/runtime/", "src/interface/launch.rs",
    "src/commands/launch/", "src/commands/interface/", "src/commands/mother/",
    "src/commands/child/", "src/commands/pando.rs", "src/commands/slate.rs",
    "src/commands/connect/", "src/commands/secrets/", "src/commands/lake/",
    "src/commands/ai/surface.rs", "src/commands/ai/legacy_session.rs",
    "src/commands/spec/legacy.rs", "src/connect/", "src/secrets/", "src/workspace/",
    "mother/src/bridge.rs", "mother/src/broker/", "mother/src/builtin_children.rs",
    "mother/src/child_registry/", "mother/src/daemon_", "mother/src/http_",
    "mother/src/lifecycle.rs", "mother/src/microserver.rs", "mother/src/pando.rs",
    "mother/src/registry.rs", "mother/src/secrets_authority_", "mother/src/services/",
    "mother/src/socket.rs", "mother/src/toys/git.rs",
)
leaked = sorted(path for path in sources if path.startswith(forbidden))
assert not leaked, f"legacy implementation compiled by default: {leaked}"
print(f"PASS: {len(sources)} active Rust source inputs; {len(forbidden)} legacy boundaries excluded")
print("PASS: local records, parser/pipeline, search and passive interface metadata remain compiled")
