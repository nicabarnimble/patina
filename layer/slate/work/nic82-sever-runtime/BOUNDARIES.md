# NIC-82 — compile-time sever boundary

Refs NIC-82. Doug approved two stacked PRs: shared boundaries, then default-off runtime.

## /why

MCT owns orchestration; PiMux owns guest launching. Patina's capture, index, search,
beliefs, models, grammars and durable session records remain standalone. No legacy
implementation is deleted, no SDK move or MCT adapter is required, and no service,
installation, plist or real user data is changed by this source change.

## /architect

- Root `default = ["duckdb-bundled"]`; `legacy-runtime` forwards to Mother and
  enables legacy-only dependencies and the bundled doctor. `full` is the supported
  compatibility alias. `dev` explicitly opts into legacy runtime.
- Mother itself defaults to shared support only. Plain Cargo commands select the
  root package, not all workspace child/application products. Explicit workspace
  builds remain available; full CI uses `--workspace --features full`.
- `#[cfg]` is on module declarations, exports, CLI variants and dispatch: this is
  source exclusion, not `cfg!` runtime branching. Binary and library are covered.
- Retain Mother SQLite/session/project/event records, local belief graph and pure
  protocol/manifest types. The `mother` directory is not synonymous with hosting.
- Retain `PipelineEngine`, parser manifests, passive logging/domain validation,
  grammar installation and metadata-only interface lookup. The `child` directory
  is not synonymous with application hosting.
- Exclude daemon bootstrap/lifecycle/HTTP/service/registry/broker/secrets hosts,
  application `ChildEngine` and toy-host bindings, client/remote routing, Pando
  startup, preflight process cleanup, interface launching/projection and tmux.
- The local spec model and direct record helpers remain; the daemon-backed spec
  CLI implementation is opt-in. Future MCT integration can replace the optional
  adapter without moving local knowledge records behind a daemon.

## /how — user-visible behavior

Default bare `patina` prints help successfully before migrations. Optional legacy
command names are recognized and return `requires --features full`; `ai` without
arguments explains local sessions and PiMux. Full builds retain the old startup
sequence and optional behavior.

Local scrape implementations are unchanged. The optional post-scrape connector
callback is excluded, with an explicit notice when `sources.toml` exists. Scry's
legacy remote branch is excluded; local search and graph routing remain, with a
notice if legacy remote environment variables are set. There is no implicit daemon
fallback, even when remote configuration is present.

`patina ai session new/update/note/end/list` and `ai list/end` retain their flags,
JSON, file/runtime IDs, durable artifacts, notes, tags and optional commits.
Default identity selection is explicit global `--interface`, then
`PATINA_AI_INTERFACE`, then project config. No executable probe or interface
bootstrap occurs. Default end archives records without tearing down tmux; full
compatibility retains the legacy adapter.

### Caller disposition

MCT's `.pi/bin` wrappers remain valid without modification:

- `PATINA_AI_INTERFACE=pi patina ai session new --json --interface pi "$@"`
- `PATINA_AI_INTERFACE=pi patina ai session update --json "$@"`
- `PATINA_AI_INTERFACE=pi patina ai session note "$@"`
- `PATINA_AI_INTERFACE=pi patina ai session end --json --commit "$@"`

Claude/OpenCode/Gemini helpers using the same session verbs remain valid. They must
select a known interface by flag/environment/project config; they cannot depend on
Patina launching or discovering a guest in the default build. Full launcher callers
must explicitly select a full binary. No MCT SDK pin changes are part of this work.

## Acceptance proof

1. Build default in a separate target with `cargo build --message-format=json`;
   `resources/scripts/check-default-build.py` checks the active root library,
   binary and Mother artifacts' features and rustc dep-info, asserting exclusion
   of legacy implementation paths and inclusion of local records/parser/search.
   A feature tree is retained alongside the audit, not used as its substitute.
2. `tests/default_runtime_boundary.rs` executes the real default binary in
   disposable projects/HOME/state with forbidden-process sentinels and an IPC
   sentinel. It checks help/disabled surfaces, identity precedence, native session
   JSON/IDs/tags/notes/commits and end alias, capture/FTS search, identical fixed
   local/reference search results, explicit local related-repo graph routing,
   scry's no-index behavior, grammar discovery, and actual WASM pipeline execution.
   It does not claim a new semantic-ranking benchmark without model/index fixtures.
3. Full CI builds with `--features full`, enumerates named daemon/Child/launcher
   tests to prevent zero-test false greens, and runs existing workspace, lifecycle,
   broker, parity and WASM/composition lanes with the feature explicitly enabled.
4. Default CI is a separate job/target. Default/full checks do not share a Cargo
   invocation that could mask an accidentally enabled feature through unification.
5. Local evidence is collected under a sandbox denying production Patina state,
   main-checkout writes and supervisor execution. Existing tests that tag their
   working repository run in a disposable independent clone, not through shared
   main-checkout Git metadata. No operational continuity claim depends on probing
   or restarting the live service.

## Rollback and limits

Rebuild with `--features full` for explicit compatibility, or revert the sever PR.
Neither action requires data migration. Do not replace or restart an installed
service as part of validation. This is source-level severing, not service retirement,
proof of complete MCT application parity, a new secrets backend, or a model/grammar
format change. Existing external installed releases are unaffected until an operator
separately chooses to deploy a new binary.
