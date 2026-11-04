# NIC-82: sever, do not delete

Refs NIC-82. Doug approved this boundary at 9:39 AM ET.

PR 1 separates shared parser diagnostics/validation, passive interface version facts, legacy session integration and control-plane client implementation. It introduces `legacy-runtime`/`full` feature names without disabling existing behavior. Local database/schema APIs remain in place: their crate name is not a hosting requirement.

PR 2 switches the default to knowledge + local session records, gating implementation modules (not just command arms). Full keeps the old runtime and tests. PiMux owns launching; default session end no longer closes tmux. Optional runtime-dependent commands fail explicitly, not by starting a daemon. Local grammar WASM and models remain intact. No data migration, SDK relocation or service cutover.

Acceptance: clean isolated default build dep-info excludes host/launcher sources; default knowledge and helper-script fixtures preserve results without legacy IPC/process/tmux effects; full builds and existing tests run; no production installation or state access. Use separate default/full targets, bounded commands, and disposable state. Runtime tests must not operate the Studio launchd domain. Evidence is recorded per PR, with incomplete gates left explicit.
