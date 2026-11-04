//! NIC-82: real default binary, disposable state, hostile legacy environment.
//! Full runtime tests remain separate; this must never run with feature unification.
#![cfg(all(unix, not(feature = "legacy-runtime")))]

use serde_json::Value;
use std::fs;
use std::io::Write;
use std::os::unix::{fs::PermissionsExt, net::UnixListener};
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread::{self, JoinHandle};
use std::time::Duration;

struct Fixture {
    _temp: tempfile::TempDir,
    root: PathBuf,
    home: PathBuf,
    state: PathBuf,
    bin: PathBuf,
    effects: PathBuf,
    ipc: PathBuf,
    stop: Arc<AtomicBool>,
    watcher: Option<JoinHandle<()>>,
}

impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("project");
        let home = temp.path().join("home");
        let state = temp.path().join("state");
        let bin = temp.path().join("bin");
        let effects = temp.path().join("forbidden-processes");
        let ipc = temp.path().join("forbidden-ipc");
        for path in [
            &root,
            &home,
            &state.join("run"),
            &bin,
            &root.join("layer/core"),
        ] {
            fs::create_dir_all(path).unwrap();
        }
        let mut config = patina::project::ProjectConfig::with_name("nic82-fixture");
        config.interfaces.default = "gemini".into();
        config.interfaces.allowed = vec!["pi".into(), "gemini".into(), "claude".into()];
        patina::project::save(&root, &config).unwrap();
        fs::write(
            root.join(".gitignore"),
            ".patina/local/\nENVIRONMENT.toml\n",
        )
        .unwrap();
        fs::write(root.join("layer/core/nic82-sentinel.md"),
            "---\nid: nic82-sentinel\nlayer: core\nstatus: active\n---\n# NIC82sentinel\n\nNIC82sentinel knowledge stays local.\n").unwrap();
        for name in [
            "tmux",
            "launchctl",
            "systemctl",
            "claude",
            "opencode",
            "gemini",
            "pi",
            "ps",
            "pgrep",
            "pkill",
            "killall",
            "osascript",
        ] {
            let path = bin.join(name);
            fs::write(
                &path,
                "#!/bin/sh\nprintf '%s\\n' \"$0 $*\" >> \"$NIC82_EFFECTS\"\nexit 97\n",
            )
            .unwrap();
            fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
        }
        std::os::unix::fs::symlink(env!("CARGO_BIN_EXE_patina"), bin.join("patina")).unwrap();
        let listener = UnixListener::bind(state.join("run/serve.sock")).unwrap();
        listener.set_nonblocking(true).unwrap();
        let stop = Arc::new(AtomicBool::new(false));
        let stopped = stop.clone();
        let ipc_log = ipc.clone();
        let watcher = thread::spawn(move || {
            while !stopped.load(Ordering::Acquire) {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        fs::write(&ipc_log, "unexpected legacy IPC").unwrap();
                        let _ = stream
                            .write_all(b"HTTP/1.1 503 Forbidden\r\nContent-Length: 0\r\n\r\n");
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(5))
                    }
                    Err(error) => panic!("IPC sentinel failed: {error}"),
                }
            }
        });
        let fixture = Self {
            _temp: temp,
            root,
            home,
            state,
            bin,
            effects,
            ipc,
            stop,
            watcher: Some(watcher),
        };
        fixture.git(&["init", "-q", "-b", "fixture"]);
        fixture.git(&["add", "."]);
        fixture.git(&["commit", "-qm", "NIC82sentinel fixture"]);
        fixture
    }

    fn command(&self, program: &str) -> Command {
        let mut command = Command::new(program);
        let path = std::env::join_paths(std::iter::once(self.bin.clone()).chain(
            std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()),
        ))
        .unwrap();
        command
            .env_clear()
            .current_dir(&self.root)
            .env("PATH", path)
            .env("HOME", &self.home)
            .env("PATINA_HOME", &self.state)
            .env("GIT_AUTHOR_NAME", "NIC82 Fixture")
            .env("GIT_COMMITTER_NAME", "NIC82 Fixture")
            .env("GIT_AUTHOR_EMAIL", "fixture@example.invalid")
            .env("GIT_COMMITTER_EMAIL", "fixture@example.invalid")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_SYSTEM", "/dev/null")
            .env("GIT_CONFIG_COUNT", "1")
            .env("GIT_CONFIG_KEY_0", "core.hooksPath")
            .env("GIT_CONFIG_VALUE_0", "/dev/null")
            .env("PATINA_AI_INTERFACE", "pi")
            .env("PATINA_PREFLIGHT_CLEANUP", "1")
            .env("PATINA_MOTHER", self.state.join("run/serve.sock"))
            .env("PATINA_MOTHER_ADDR", self.state.join("run/serve.sock"))
            .env("TMUX", "fixture-owned-by-pimux")
            .env("NIC82_EFFECTS", &self.effects);
        if let Some(value) = std::env::var_os("LD_LIBRARY_PATH") {
            command.env("LD_LIBRARY_PATH", value);
        }
        command
    }

    fn git(&self, args: &[&str]) -> String {
        let output = self.command("git").args(args).output().unwrap();
        success(&output);
        String::from_utf8(output.stdout).unwrap()
    }

    fn run(&self, args: &[&str]) -> Output {
        self.command(env!("CARGO_BIN_EXE_patina"))
            .args(args)
            .output()
            .unwrap()
    }

    fn json(&self, args: &[&str]) -> Value {
        let output = self.run(args);
        success(&output);
        // Existing session-end output may prefix its JSON with an event-export status line.
        let stdout = String::from_utf8(output.stdout.clone()).unwrap();
        let payload = if stdout.trim_start().starts_with("Exported ") {
            stdout
                .split_once("\n{")
                .map(|(_, json)| format!("{{{json}"))
                .unwrap()
        } else {
            stdout
        };
        serde_json::from_str(&payload).unwrap_or_else(|error| {
            panic!(
                "invalid JSON: {error}\n{}",
                String::from_utf8_lossy(&output.stdout)
            )
        })
    }

    fn assert_no_runtime_effects(&self) {
        // The sentinel responds immediately, so any attempted request is recorded.
        thread::sleep(Duration::from_millis(15));
        assert!(
            !self.effects.exists(),
            "legacy subprocess invoked: {:?}",
            fs::read_to_string(&self.effects)
        );
        assert!(!self.ipc.exists(), "legacy IPC attempted");
        assert!(
            !self.state.join("interfaces/registry.toml").exists(),
            "interface registry seeded"
        );
        assert!(
            !self.root.join("AGENTS.md").exists(),
            "interface surface bootstrapped"
        );
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        self.watcher.take().unwrap().join().unwrap();
    }
}

fn success(output: &Output) {
    assert!(
        output.status.success(),
        "status={:?}\nstdout={}\nstderr={}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn bare_help_and_disabled_commands_never_start_runtime() {
    let fixture = Fixture::new();
    let bare = fixture.run(&[]);
    success(&bare);
    assert!(String::from_utf8_lossy(&bare.stdout).contains("PiMux"));
    assert!(
        !fixture.root.join(".patina/local").exists(),
        "bare help ran migrations"
    );
    assert_eq!(
        fs::read_dir(&fixture.state).unwrap().count(),
        1,
        "bare help wrote global state"
    );
    for command in [
        "mother",
        "serve",
        "child",
        "plugin",
        "slate",
        "pando",
        "connect",
        "secrets",
        "spec",
        "lake",
        "doctor",
        "interface",
    ] {
        let output = fixture.run(&[command, "--arbitrary-old-flag"]);
        assert!(!output.status.success(), "{command} unexpectedly succeeded");
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("--features full"),
            "{command}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    for guest in ["setup", "refresh", "claude", "opencode", "gemini", "pi"] {
        let output = fixture.run(&["ai", guest]);
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("--features full"));
    }
    success(&fixture.run(&["ai"]));
    std::os::unix::fs::symlink(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("grammars"),
        fixture.root.join("grammars"),
    )
    .unwrap();
    let grammars = fixture.run(&["setup", "grammars", "--list"]);
    success(&grammars);
    assert!(String::from_utf8_lossy(&grammars.stdout).contains("grammar-rust"));
    fixture.assert_no_runtime_effects();
}

#[test]
fn native_session_contract_preserves_ids_tags_notes_json_and_commits_without_tmux() {
    let fixture = Fixture::new();
    // Exact argument ordering used by MCT .pi/bin/session-new.sh, including global flag after subcommands.
    let started = fixture.json(&[
        "ai",
        "session",
        "new",
        "--json",
        "--interface",
        "pi",
        "NIC82 session",
    ]);
    let id = started["session_id"].as_str().unwrap();
    let runtime = started["runtime_id"].as_str().unwrap();
    assert!(uuid::Uuid::parse_str(runtime).is_ok());
    assert_eq!(started["interface"], "pi");
    assert!(!id.is_empty());
    let start_tag = started["start_tag"].as_str().unwrap();
    fixture.git(&["rev-parse", "--verify", start_tag]);
    let updated = fixture.json(&[
        "ai",
        "session",
        "update",
        "--json",
        "--session",
        runtime,
        "--title",
        "NIC82 renamed",
    ]);
    assert_eq!(updated["session_id"], id);
    assert_eq!(updated["runtime_id"], runtime);
    success(&fixture.run(&[
        "ai",
        "session",
        "note",
        "NIC82 durable note",
        "--session",
        id,
    ]));
    let list = fixture.json(&["ai", "list", "--json"]);
    assert_eq!(list[0]["file_id"], id);
    let previous_head = fixture.git(&["rev-parse", "HEAD"]);
    let ended = fixture.json(&[
        "ai",
        "session",
        "end",
        "--json",
        "--commit",
        "--session",
        runtime,
        "--note",
        "NIC82 closeout",
    ]);
    assert_eq!(ended["session_id"], id);
    assert_eq!(ended["runtime_id"], runtime);
    fixture.git(&["rev-parse", "--verify", ended["end_tag"].as_str().unwrap()]);
    assert_ne!(
        fixture.git(&["rev-parse", "HEAD"]),
        previous_head,
        "--commit did not commit artifacts"
    );
    let artifact = fs::read_to_string(started["artifact_path"].as_str().unwrap()).unwrap();
    assert!(artifact.contains("status: archived"));
    assert!(artifact.contains("NIC82 durable note"));
    assert!(artifact.contains("NIC82 closeout"));
    assert_eq!(
        fixture.json(&["ai", "session", "list", "--json"]),
        serde_json::json!([])
    );
    fixture.assert_no_runtime_effects();
}

#[test]
fn interface_identity_precedence_and_end_alias_are_passive() {
    let fixture = Fixture::new();
    let original = fs::read(fixture.root.join(".patina/config.toml")).unwrap();
    for (explicit, environment, expected) in [
        (Some("claude"), Some("pi"), "claude"),
        (None, Some("pi"), "pi"),
        (None, None, "gemini"),
    ] {
        let mut command = fixture.command(env!("CARGO_BIN_EXE_patina"));
        command.args(["ai", "session", "new", "identity", "--json"]);
        if let Some(name) = explicit {
            command.args(["--interface", name]);
        }
        if let Some(name) = environment {
            command.env("PATINA_AI_INTERFACE", name);
        } else {
            command.env_remove("PATINA_AI_INTERFACE");
        }
        let output = command.output().unwrap();
        success(&output);
        let started: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(started["interface"], expected);
        let ended = fixture.json(&[
            "ai",
            "end",
            "--json",
            "--session",
            started["runtime_id"].as_str().unwrap(),
        ]);
        assert_eq!(ended["file_id"], started["session_id"]);
    }
    assert_eq!(
        fs::read(fixture.root.join(".patina/config.toml")).unwrap(),
        original
    );
    fixture.assert_no_runtime_effects();
}

#[test]
fn local_capture_and_search_ignore_remote_environment() {
    let fixture = Fixture::new();
    success(&fixture.run(&["scrape", "git"]));
    success(&fixture.run(&["scrape", "layer"]));
    let found = fixture.json(&["assay", "search", "NIC82sentinel", "--json"]);
    assert!(
        found.to_string().contains("NIC82sentinel"),
        "missing local search result: {found}"
    );
    let scry = fixture.run(&["scry", "NIC82sentinel"]);
    // No semantic index was installed: preserve local empty-oracle behavior,
    // never use the configured remote daemon as an implicit fallback.
    success(&scry);
    assert!(String::from_utf8_lossy(&scry.stdout).contains("No results found."));
    let file_search = fixture.run(&["scry", "--file", "src/fixture.rs"]);
    assert!(!file_search.status.success());
    assert!(String::from_utf8_lossy(&file_search.stderr).contains("oxidize"));
    assert!(!String::from_utf8_lossy(&scry.stdout).contains("Querying mother"));
    let uid =
        patina::project::get_uid(&fixture.root).expect("local capture registered project UID");
    // Storage remains the existing local Mother project store, not a daemon API.
    let db = rusqlite::Connection::open(
        fixture
            .state
            .join("mother/projects")
            .join(uid)
            .join("patina.db"),
    )
    .unwrap();
    let count: i64 = db
        .query_row("SELECT COUNT(*) FROM commits", [], |row| row.get(0))
        .unwrap();
    assert_eq!(count, 1);

    // Positive local cross-repository retrieval uses the existing cache/registry
    // contract, not a Mother endpoint. The reference fixture has the same fixed DB.
    let reference = fixture.state.join("cache/repos/reference");
    fs::create_dir_all(reference.join(".patina/local/data")).unwrap();
    fs::copy(
        db.path().unwrap(),
        reference.join(".patina/local/data/patina.db"),
    )
    .unwrap();
    fs::write(
        fixture.state.join("registry.yaml"),
        serde_json::to_vec(&serde_json::json!({
            "version": 1,
            "repos": {"reference": {
                "path": reference, "github": "https://example.invalid/reference",
                "registered": "2026-01-01T00:00:00Z"
            }}
        }))
        .unwrap(),
    )
    .unwrap();
    let reference_results = fixture.json(&[
        "assay",
        "--repo",
        "reference",
        "search",
        "NIC82sentinel",
        "--json",
    ]);
    assert_eq!(
        reference_results, found,
        "fixed local and reference search fixtures diverged"
    );

    // Initialize the ordinary local graph, then seed an explicit relationship.
    success(&fixture.run(&["scry", "NIC82sentinel", "--all-repos"]));
    let graph = rusqlite::Connection::open(fixture.state.join("mother/graph.db")).unwrap();
    for (id, kind, path) in [
        ("project", patina::mother::NodeType::Project, &fixture.root),
        ("reference", patina::mother::NodeType::Reference, &reference),
    ] {
        graph
            .execute(
                "INSERT INTO nodes (id, node_type, path, domains) VALUES (?1, ?2, ?3, '[]')",
                rusqlite::params![
                    id,
                    kind.as_str(),
                    path.canonicalize().unwrap().to_string_lossy()
                ],
            )
            .unwrap();
    }
    graph.execute("INSERT INTO edges (from_node, to_node, edge_type, created) VALUES ('project', 'reference', ?1, '2026-01-01T00:00:00Z')",
        [patina::mother::EdgeType::Uses.as_str()]).unwrap();
    let routed = fixture.run(&["scry", "NIC82sentinel", "--all-repos"]);
    success(&routed);
    let routed_text = String::from_utf8_lossy(&routed.stdout);
    assert!(routed_text.contains("Related repos: reference"));
    assert!(routed_text.contains("Searching reference"));
    fixture.assert_no_runtime_effects();
}

#[test]
fn local_pipeline_engine_still_executes_without_application_host() {
    let engine = patina::child::engine::PipelineEngine::new().unwrap();
    let component = engine
        .load_component(include_bytes!("fixtures/echo_pipeline.wasm"))
        .unwrap();
    assert_eq!(engine.get_name(&component).unwrap(), "echo-pipeline");
    let result = engine
        .handle(
            &component,
            "echo-pipeline",
            r#"{"op":"echo","version":"1","payload":{"knowledge":"local","count":42}}"#,
        )
        .unwrap();
    let result: Value = serde_json::from_str(&result).unwrap();
    assert_eq!(result, serde_json::json!({"knowledge":"local", "count":42}));
}
