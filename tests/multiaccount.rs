//! Public CLI account routing and persistence; isolated HOME, synthetic credentials.
#![cfg(unix)]
use serde_json::{Value, json};
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

fn config(home: &Path, name: Option<&str>) -> std::path::PathBuf {
    let path = match name {
        Some(n) => home.join(format!(".railway/accounts/{n}.json")),
        None => home.join(".railway/config.json"),
    };
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        serde_json::to_vec(
            &json!({"projects":{},"user":{"id":"fixture-user","token":"fixture-secret"}}),
        )
        .unwrap(),
    )
    .unwrap();
    path
}
fn run(home: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_railway"))
        .env_clear()
        .env("HOME", home)
        .env("PATH", "/usr/bin:/bin")
        .env("DO_NOT_TRACK", "1")
        .env("RAILWAY_NO_AUTO_UPDATE", "1")
        .env("RAILWAY_API_TOKEN", "ambient-secret")
        .env("HTTPS_PROXY", "http://127.0.0.1:9")
        .env("HTTP_PROXY", "http://127.0.0.1:9")
        .current_dir(home)
        .args(args)
        .output()
        .unwrap()
}
#[test]
fn legacy_logout_remains_supported() {
    let home = tempfile::tempdir().unwrap();
    let path = config(home.path(), None);
    let out = run(home.path(), &["logout"]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(!fs::read_to_string(path).unwrap().contains("fixture-secret"));
}
#[test]
fn selection_is_required_only_for_multiple_and_logout_is_isolated() {
    let home = tempfile::tempdir().unwrap();
    let legacy = config(home.path(), None);
    let a = config(home.path(), Some("a"));
    let b = config(home.path(), Some("b"));
    let before = fs::read(&legacy).unwrap();
    let b_before = fs::read(&b).unwrap();
    for args in [
        &["logout"][..],
        &["whoami"],
        &["mcp", "local"],
        &["ssh"],
        &["setup", "agent", "-y"],
    ] {
        let out = run(home.path(), args);
        assert_eq!(out.status.code(), Some(2));
        assert!(String::from_utf8_lossy(&out.stderr).contains("Multiple Railway accounts"));
    }
    assert!(
        run(home.path(), &["logout", "--account", "a"])
            .status
            .success()
    );
    assert!(!fs::read_to_string(&a).unwrap().contains("fixture-secret"));
    assert_eq!(fs::read(&b).unwrap(), b_before);
    assert_eq!(fs::read(&legacy).unwrap(), before);
    fs::remove_file(a).unwrap();
    assert!(run(home.path(), &["logout"]).status.success());
    assert!(!fs::read_to_string(b).unwrap().contains("fixture-secret"));
}
#[test]
fn list_is_offline_metadata_and_import_is_private_no_clobber() {
    use std::os::unix::fs::PermissionsExt;
    let home = tempfile::tempdir().unwrap();
    let legacy = config(home.path(), None);
    let original = fs::read(&legacy).unwrap();
    assert!(
        run(home.path(), &["account", "import-legacy", "work"])
            .status
            .success()
    );
    let path = home.path().join(".railway/accounts/work.json");
    assert_eq!(
        fs::metadata(&path).unwrap().permissions().mode() & 0o777,
        0o600
    );
    assert_eq!(
        fs::metadata(path.parent().unwrap())
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o700
    );
    assert_eq!(fs::read(&legacy).unwrap(), original);
    assert!(
        !run(home.path(), &["account", "import-legacy", "work"])
            .status
            .success()
    );
    let out = run(home.path(), &["account", "list", "--json"]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let rows: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(rows[0]["name"], "work");
    assert_eq!(rows[0]["userId"], "fixture-user");
    assert_eq!(rows[0]["authenticated"], true);
    assert!(!String::from_utf8_lossy(&out.stdout).contains("secret"));
}
#[test]
fn explicit_selection_is_not_ignored_by_unscoped_commands_and_mcp_is_pinned() {
    let home = tempfile::tempdir().unwrap();
    config(home.path(), Some("work"));
    config(home.path(), Some("other"));
    let out = run(home.path(), &["--account", "missing", "account", "list"]);
    assert_eq!(out.status.code(), Some(2));
    let out = run(
        home.path(),
        &[
            "--account",
            "work",
            "mcp",
            "install",
            "--agent",
            "cursor",
            "--local",
        ],
    );
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let value: Value =
        serde_json::from_slice(&fs::read(home.path().join(".cursor/mcp.json")).unwrap()).unwrap();
    assert_eq!(
        value["mcpServers"]["railway"]["args"],
        json!(["--account", "work", "mcp", "local"])
    );
}
