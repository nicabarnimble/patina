use anyhow::{Context, Result};
use std::fs::{File, OpenOptions, TryLockError};
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

static SHUTDOWN_REQUESTED: AtomicBool = AtomicBool::new(false);

pub fn write_pid_file(pid_path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    if let Some(parent) = pid_path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("creating runtime directory {}", parent.display()))?;
            std::fs::set_permissions(parent, std::fs::Permissions::from_mode(0o700))
                .with_context(|| format!("setting permissions on {}", parent.display()))?;
        }
    }

    let pid = std::process::id();

    std::fs::write(pid_path, pid.to_string())
        .with_context(|| format!("writing PID file {}", pid_path.display()))?;

    std::fs::set_permissions(pid_path, std::fs::Permissions::from_mode(0o600))
        .with_context(|| format!("setting permissions on {}", pid_path.display()))?;

    Ok(())
}

#[derive(Debug)]
pub struct UdsOwnership {
    _lock: File,
    pid_path: PathBuf,
    socket_path: PathBuf,
    published: bool,
}

impl UdsOwnership {
    pub fn socket_path(&self) -> &Path {
        &self.socket_path
    }

    pub fn bind_listener(&mut self) -> Result<std::os::unix::net::UnixListener> {
        let listener = std::os::unix::net::UnixListener::bind(&self.socket_path)
            .with_context(|| format!("binding socket {}", self.socket_path.display()))?;
        self.published = true;
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&self.socket_path, std::fs::Permissions::from_mode(0o600))?;
        write_pid_file(&self.pid_path)?;
        Ok(listener)
    }
}

impl Drop for UdsOwnership {
    fn drop(&mut self) {
        if self.published {
            for path in [&self.pid_path, &self.socket_path] {
                if let Err(error) = remove_file_if_exists(path) {
                    tracing::warn!(path = %path.display(), %error, "failed to remove runtime file");
                }
            }
        }
        // Keep the lock inode. Unlinking it would let starters lock different files.
    }
}

fn remove_file_if_exists(path: &Path) -> Result<()> {
    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

pub fn reconcile_pid_state(pid_path: &Path, socket_path: &Path) -> Result<Option<UdsOwnership>> {
    let run_dir = pid_path
        .parent()
        .context("PID path has no runtime directory")?;
    crate::socket::ensure_run_dir(run_dir)?;
    let Some(lock) = try_runtime_lock(&pid_path.with_extension("lock"))? else {
        return Ok(None);
    };

    if legacy_mother_is_running(socket_path)? {
        return Ok(None);
    }
    crate::socket::cleanup_stale_socket(socket_path)?;
    remove_file_if_exists(pid_path)
        .with_context(|| format!("removing stale PID file {}", pid_path.display()))?;
    Ok(Some(UdsOwnership {
        _lock: lock,
        pid_path: pid_path.to_path_buf(),
        socket_path: socket_path.to_path_buf(),
        published: false,
    }))
}

fn try_runtime_lock(lock_path: &Path) -> Result<Option<File>> {
    let lock = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(lock_path)
        .with_context(|| format!("opening Mother lock {}", lock_path.display()))?;
    anyhow::ensure!(
        lock.metadata()?.is_file(),
        "Mother lock is not a regular file"
    );
    match lock.try_lock() {
        Ok(()) => Ok(Some(lock)),
        Err(TryLockError::WouldBlock) => Ok(None),
        Err(TryLockError::Error(error)) => Err(error).context("locking Mother runtime"),
    }
}

fn legacy_mother_is_running(socket_path: &Path) -> Result<bool> {
    // Older Mothers have no lock. A connected but unresponsive socket is not stale.
    match crate::lifecycle::query_health(socket_path) {
        Ok(_) => Ok(true),
        Err(error) => match error
            .downcast_ref::<std::io::Error>()
            .map(|error| error.kind())
        {
            Some(std::io::ErrorKind::NotFound | std::io::ErrorKind::ConnectionRefused) => Ok(false),
            _ => Err(error).context("existing Mother socket could not be verified"),
        },
    }
}

pub fn register_signal_handlers() {
    SHUTDOWN_REQUESTED.store(false, Ordering::SeqCst);

    unsafe {
        libc::signal(
            libc::SIGINT,
            sigint_handler as *const () as libc::sighandler_t,
        );
        libc::signal(
            libc::SIGTERM,
            sigint_handler as *const () as libc::sighandler_t,
        );
    }
}

pub fn shutdown_flag() -> &'static AtomicBool {
    &SHUTDOWN_REQUESTED
}

pub fn shutdown_requested() -> bool {
    SHUTDOWN_REQUESTED.load(Ordering::Relaxed)
}

extern "C" fn sigint_handler(_: libc::c_int) {
    SHUTDOWN_REQUESTED.store(true, Ordering::Relaxed);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn private_runtime_dir() -> tempfile::TempDir {
        use std::os::unix::fs::PermissionsExt;
        let temp = tempfile::tempdir().unwrap();
        std::fs::set_permissions(temp.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        temp
    }

    #[test]
    fn ownership_is_retained_until_drop_and_lock_file_is_not_unlinked() {
        let temp = private_runtime_dir();
        let pid_path = temp.path().join("mother.pid");
        let socket_path = temp.path().join("serve.sock");
        let mut owner = reconcile_pid_state(&pid_path, &socket_path)
            .unwrap()
            .unwrap();
        let listener = owner.bind_listener().unwrap();
        let pid = std::fs::read_to_string(&pid_path).unwrap();

        assert!(reconcile_pid_state(&pid_path, &socket_path)
            .unwrap()
            .is_none());
        assert_eq!(std::fs::read_to_string(&pid_path).unwrap(), pid);
        assert!(socket_path.exists());
        std::fs::remove_file(&pid_path).unwrap();
        assert!(reconcile_pid_state(&pid_path, &socket_path)
            .unwrap()
            .is_none());
        drop(listener);
        drop(owner);

        assert!(!socket_path.exists());
        assert!(pid_path.with_extension("lock").exists());
        assert!(reconcile_pid_state(&pid_path, &socket_path)
            .unwrap()
            .is_some());
    }

    #[test]
    fn malformed_pid_does_not_block_start() {
        let temp = private_runtime_dir();
        let pid_path = temp.path().join("mother.pid");
        std::fs::write(&pid_path, "not a PID").unwrap();
        assert!(
            reconcile_pid_state(&pid_path, &temp.path().join("serve.sock"))
                .unwrap()
                .is_some()
        );
    }

    #[test]
    fn insecure_runtime_directory_is_rejected_before_writes() {
        use std::os::unix::fs::PermissionsExt;
        let temp = tempfile::tempdir().unwrap();
        std::fs::set_permissions(temp.path(), std::fs::Permissions::from_mode(0o755)).unwrap();
        let pid_path = temp.path().join("mother.pid");
        let error = reconcile_pid_state(&pid_path, &temp.path().join("serve.sock")).unwrap_err();
        assert!(error.to_string().contains("group/world accessible"));
        assert!(!pid_path.with_extension("lock").exists());
    }

    #[test]
    fn non_socket_file_and_symlink_lock_are_preserved() {
        let temp = private_runtime_dir();
        let pid_path = temp.path().join("mother.pid");
        let socket_path = temp.path().join("serve.sock");
        std::fs::write(&socket_path, "not a socket").unwrap();
        assert!(reconcile_pid_state(&pid_path, &socket_path).is_err());
        assert_eq!(
            std::fs::read_to_string(&socket_path).unwrap(),
            "not a socket"
        );

        let other_pid = temp.path().join("other.pid");
        std::os::unix::fs::symlink(&socket_path, other_pid.with_extension("lock")).unwrap();
        assert!(reconcile_pid_state(&other_pid, &socket_path).is_err());
        assert_eq!(
            std::fs::read_to_string(&socket_path).unwrap(),
            "not a socket"
        );
    }

    #[test]
    fn failed_pid_publication_cleans_socket_and_releases_ownership() {
        let temp = private_runtime_dir();
        let pid_path = temp.path().join("mother.pid");
        let socket_path = temp.path().join("serve.sock");
        let mut owner = reconcile_pid_state(&pid_path, &socket_path)
            .unwrap()
            .unwrap();
        std::fs::create_dir(&pid_path).unwrap();
        assert!(owner.bind_listener().is_err());
        drop(owner);
        assert!(!socket_path.exists());
        std::fs::remove_dir(&pid_path).unwrap();
        assert!(reconcile_pid_state(&pid_path, &socket_path)
            .unwrap()
            .is_some());
    }

    #[test]
    fn legacy_socket_handshake_preserves_runtime_files() {
        use std::io::{Read, Write};
        for (response, valid) in [
            (
                "HTTP/1.1 200 OK\r\n\r\n{\"version\":\"0.71.0\",\"uptime_secs\":1}",
                true,
            ),
            (
                "HTTP/1.1 500 Error\r\n\r\n{\"version\":\"0.71.0\",\"uptime_secs\":1}",
                false,
            ),
            ("HTTP/1.1 200 OK\r\n\r\n{}", false),
        ] {
            let temp = private_runtime_dir();
            let pid_path = temp.path().join("mother.pid");
            let socket_path = temp.path().join("serve.sock");
            let listener = std::os::unix::net::UnixListener::bind(&socket_path).unwrap();
            let server = std::thread::spawn(move || {
                let (mut stream, _) = listener.accept().unwrap();
                let mut request = [0; 1024];
                stream.read(&mut request).unwrap();
                stream.write_all(response.as_bytes()).unwrap();
            });
            let result = reconcile_pid_state(&pid_path, &socket_path);
            if valid {
                assert!(result.unwrap().is_none());
            } else {
                assert!(result.is_err());
            }
            assert!(socket_path.exists());
            assert!(!pid_path.exists());
            server.join().unwrap();
        }
    }

    #[test]
    fn stale_pid_cleans_pid_and_socket_files() {
        let temp = private_runtime_dir();
        let pid_path = temp.path().join("mother.pid");
        let socket_path = temp.path().join("serve.sock");
        std::fs::write(&pid_path, "999999").unwrap();
        let stale_socket = std::os::unix::net::UnixListener::bind(&socket_path).unwrap();
        drop(stale_socket);

        reconcile_pid_state(&pid_path, &socket_path).unwrap();

        assert!(!pid_path.exists());
        assert!(!socket_path.exists());
    }

    #[test]
    fn stale_live_non_mother_pid_does_not_block_start() {
        let temp = private_runtime_dir();
        let pid_path = temp.path().join("mother.pid");
        let socket_path = temp.path().join("serve.sock");
        std::fs::write(&pid_path, std::process::id().to_string()).unwrap();

        let stale_socket = std::os::unix::net::UnixListener::bind(&socket_path).unwrap();
        drop(stale_socket);

        let _ownership = reconcile_pid_state(&pid_path, &socket_path)
            .expect("a live test process is not Mother and must not block startup");
        assert!(!pid_path.exists());
        assert!(!socket_path.exists());
    }
}
