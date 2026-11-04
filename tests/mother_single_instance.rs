#![cfg(unix)]

use std::io::{Read, Write};
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

struct TestMother(Child);

impl Drop for TestMother {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn command(home: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_patina"));
    command
        .args(["mother", "start", "--profile", "core"])
        .current_dir(home)
        .env("HOME", home)
        .env("XDG_CONFIG_HOME", home.join(".config"))
        .env("PATINA_HOME", home.join(".patina"))
        .env("PATINA_SUPERVISED", "1")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    command
}

fn wait_for_health(mother: &mut TestMother, socket: &Path) {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        if let Some(status) = mother.0.try_wait().unwrap() {
            let mut stderr = String::new();
            mother
                .0
                .stderr
                .take()
                .unwrap()
                .read_to_string(&mut stderr)
                .unwrap();
            panic!("Mother exited before serving health: {status}\n{stderr}");
        }
        if let Ok(mut stream) = UnixStream::connect(socket) {
            stream
                .set_read_timeout(Some(Duration::from_millis(200)))
                .unwrap();
            stream
                .set_write_timeout(Some(Duration::from_millis(200)))
                .unwrap();
            if stream
                .write_all(b"GET /health HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
                .is_ok()
            {
                let mut response = String::new();
                if stream.take(64 * 1024).read_to_string(&mut response).is_ok()
                    && response.starts_with("HTTP/1.1 200")
                    && response.contains("\"uptime_secs\"")
                {
                    return;
                }
            }
        }
        assert!(
            Instant::now() < deadline,
            "Mother did not serve health within 30 seconds"
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}

#[test]
fn stale_live_non_mother_pid_starts_and_duplicate_start_exits_zero() {
    let home = tempfile::Builder::new()
        .prefix("mother-")
        .tempdir_in("/tmp")
        .unwrap();
    let run = home.path().join(".patina/run");
    std::fs::create_dir_all(&run).unwrap();
    std::fs::set_permissions(&run, std::fs::Permissions::from_mode(0o700)).unwrap();
    let pid = run.join("mother.pid");
    let socket = run.join("serve.sock");
    std::fs::write(&pid, std::process::id().to_string()).unwrap();
    drop(UnixListener::bind(&socket).unwrap());

    let mut mother = TestMother(command(home.path()).spawn().unwrap());
    wait_for_health(&mut mother, &socket);
    assert_eq!(
        std::fs::read_to_string(&pid).unwrap(),
        mother.0.id().to_string()
    );
    let socket_inode = std::fs::metadata(&socket).unwrap().ino();

    let plist = home
        .path()
        .join("Library/LaunchAgents/com.patina.mother.plist");
    let unit = home
        .path()
        .join(".config/systemd/user/patina-mother.service");
    for marker in [plist, unit] {
        std::fs::create_dir_all(marker.parent().unwrap()).unwrap();
        std::fs::write(marker, "installed supervisor marker").unwrap();
    }
    let mut duplicate = TestMother(
        command(home.path())
            .env_remove("PATINA_SUPERVISED")
            .spawn()
            .unwrap(),
    );
    let deadline = Instant::now() + Duration::from_secs(5);
    let status = loop {
        if let Some(status) = duplicate.0.try_wait().unwrap() {
            break status;
        }
        assert!(Instant::now() < deadline, "duplicate start did not exit");
        std::thread::sleep(Duration::from_millis(25));
    };
    assert!(status.success(), "duplicate start must exit zero: {status}");
    assert_eq!(std::fs::metadata(&socket).unwrap().ino(), socket_inode);
    assert_eq!(
        std::fs::read_to_string(&pid).unwrap(),
        mother.0.id().to_string()
    );
    wait_for_health(&mut mother, &socket);

    mother.0.kill().unwrap();
    mother.0.wait().unwrap();
    assert!(socket.exists(), "SIGKILL should leave stale runtime files");
    let mut replacement = TestMother(command(home.path()).spawn().unwrap());
    wait_for_health(&mut replacement, &socket);
    assert_eq!(
        std::fs::read_to_string(&pid).unwrap(),
        replacement.0.id().to_string()
    );
}
