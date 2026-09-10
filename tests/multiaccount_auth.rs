//! Real subprocess + HTTP boundary: no provider credentials or live endpoints.
#![cfg(unix)]
use serde_json::json;
use std::{
    fs,
    io::{Read, Write},
    net::TcpListener,
    path::Path,
    process::Command,
    time::{Duration, Instant},
};

fn command(home: &Path) -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_railway"));
    cmd.env_clear()
        .env("HOME", home)
        .env("PATH", "/usr/bin:/bin")
        .env("DO_NOT_TRACK", "1")
        .env("RAILWAY_NO_AUTO_UPDATE", "1")
        .env("RAILWAY_TOKEN", "ambient-project")
        .env("RAILWAY_API_TOKEN", "ambient-api")
        .current_dir(home);
    cmd
}
fn account(home: &Path, name: &str, token: Option<&str>) {
    let directory = home.join(".railway/accounts");
    fs::create_dir_all(&directory).unwrap();
    fs::write(
        directory.join(format!("{name}.json")),
        serde_json::to_vec(&json!({"projects":{},"user":{"token":token}})).unwrap(),
    )
    .unwrap();
}
#[test]
fn actual_http_request_uses_selected_profile_not_ambient_tokens() {
    for explicit in [false, true] {
        let home = tempfile::tempdir().unwrap();
        account(home.path(), "work", Some("selected-fixture"));
        if explicit {
            account(home.path(), "other", Some("other-fixture"));
        }
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let addr = listener.local_addr().unwrap();
        let server = std::thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(10);
            let mut socket = loop {
                match listener.accept() {
                    Ok((stream, _)) => break stream,
                    Err(e)
                        if e.kind() == std::io::ErrorKind::WouldBlock
                            && Instant::now() < deadline =>
                    {
                        std::thread::sleep(Duration::from_millis(10))
                    }
                    Err(e) => panic!("mock received no request: {e}"),
                }
            };
            socket.set_nonblocking(false).unwrap();
            socket
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            let mut request = Vec::new();
            let mut buf = [0; 4096];
            while !request.windows(4).any(|w| w == b"\r\n\r\n") {
                let n = socket.read(&mut buf).unwrap();
                assert_ne!(n, 0);
                request.extend_from_slice(&buf[..n]);
            }
            let body = r#"{"data":{"me":{"id":"test-user","name":"Fixture","email":"fixture@example.invalid"}}}"#;
            write!(socket, "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", body.len(), body).unwrap();
            String::from_utf8(request).unwrap().to_lowercase()
        });
        let mut cmd = command(home.path());
        cmd.env("RAILWAY_BACKBOARD_URL", format!("http://{addr}"));
        if explicit {
            cmd.args(["--account", "work"]);
        }
        let out = cmd.arg("whoami").output().unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        let request = server.join().unwrap();
        assert!(request.contains("authorization: bearer selected-fixture"));
        assert!(!request.contains("ambient-"));
        assert!(!request.contains("other-fixture"));
        assert!(String::from_utf8_lossy(&out.stdout).contains("fixture@example.invalid"));
    }
}
#[test]
fn logged_out_named_account_never_falls_back_and_zero_credentials_get_guidance() {
    let home = tempfile::tempdir().unwrap();
    account(home.path(), "work", None);
    let out = command(home.path()).arg("whoami").output().unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("login"));
    fs::remove_file(home.path().join(".railway/accounts/work.json")).unwrap();
    let out = command(home.path())
        .env_remove("RAILWAY_TOKEN")
        .env_remove("RAILWAY_API_TOKEN")
        .arg("whoami")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("login --account"));
}
#[test]
fn named_logout_preserves_legacy_and_other_account_ancillary_secrets() {
    let home = tempfile::tempdir().unwrap();
    account(home.path(), "a", Some("a"));
    account(home.path(), "b", Some("b"));
    for dir in [".railway", ".railway/accounts/a", ".railway/accounts/b"] {
        fs::create_dir_all(home.path().join(dir)).unwrap();
        for file in ["claude-code-token", "last-code-config.json"] {
            fs::write(home.path().join(dir).join(file), b"fixture").unwrap();
        }
    }
    let out = command(home.path())
        .args(["--account", "a", "logout"])
        .output()
        .unwrap();
    assert!(out.status.success());
    for file in ["claude-code-token", "last-code-config.json"] {
        assert!(!home.path().join(".railway/accounts/a").join(file).exists());
        assert!(home.path().join(".railway/accounts/b").join(file).exists());
        assert!(home.path().join(".railway").join(file).exists());
    }
    let out = command(home.path())
        .args(["--account", "a", "code", "get-config"])
        .output()
        .unwrap();
    assert!(String::from_utf8_lossy(&out.stderr).contains("No saved connection"));
}
#[test]
fn import_rejects_symlink_destination_and_invalid_source() {
    let home = tempfile::tempdir().unwrap();
    account(home.path(), "a", Some("a"));
    let legacy = home.path().join(".railway/config.json");
    fs::write(&legacy, b"invalid").unwrap();
    assert!(
        !command(home.path())
            .args(["account", "import-legacy", "new"])
            .output()
            .unwrap()
            .status
            .success()
    );
    fs::write(&legacy, br#"{"projects":{},"user":{"token":"fixture"}}"#).unwrap();
    std::os::unix::fs::symlink(&legacy, home.path().join(".railway/accounts/new.json")).unwrap();
    assert!(
        !command(home.path())
            .args(["account", "import-legacy", "new"])
            .output()
            .unwrap()
            .status
            .success()
    );
    assert_eq!(
        fs::read(&legacy).unwrap(),
        br#"{"projects":{},"user":{"token":"fixture"}}"#
    );
    for name in ["..", ".", "../outside"] {
        assert!(
            !command(home.path())
                .args(["account", "import-legacy", name])
                .output()
                .unwrap()
                .status
                .success()
        );
    }
}
