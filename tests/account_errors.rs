//! Platform-neutral public CLI contract; only synthetic credentials and isolated homes.
use serde_json::{Value, json};
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

fn run(home: &Path, args: &[&str]) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_railway"));
    cmd.env_clear();
    // Windows needs its OS directory; never inherit authentication or user config.
    for key in ["SystemRoot", "WINDIR"] {
        if let Some(value) = std::env::var_os(key) {
            cmd.env(key, value);
        }
    }
    cmd.env("HOME", home)
        .env("USERPROFILE", home)
        .env("DO_NOT_TRACK", "1")
        .env("RAILWAY_NO_AUTO_UPDATE", "1")
        .env("RAILWAY_API_TOKEN", "synthetic-ambient")
        .env("HTTPS_PROXY", "http://127.0.0.1:9")
        .current_dir(home)
        .args(args)
        .output()
        .unwrap()
}
fn profile(home: &Path, name: &str) {
    let dir = home.join(".railway/accounts");
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join(format!("{name}.json")),
        serde_json::to_vec(&json!({"projects": {}, "user": {"token": "synthetic-stored"}}))
            .unwrap(),
    )
    .unwrap();
}
#[test]
fn json_account_errors_are_single_objects_before_side_effects() {
    let home = tempfile::tempdir().unwrap();
    profile(home.path(), "lpg");
    profile(home.path(), "brianle");
    for (args, code) in [
        (vec!["whoami", "--json"], "ACCOUNT_REQUIRED"),
        (
            vec!["--account", "absent", "whoami", "--json"],
            "ACCOUNT_UNKNOWN",
        ),
        (
            vec!["--account", "../escape", "whoami", "--json"],
            "ACCOUNT_INVALID",
        ),
        (
            vec!["--account", "absent", "account", "list", "--json"],
            "ACCOUNT_UNKNOWN",
        ),
    ] {
        let out = run(home.path(), &args);
        assert_eq!(out.status.code(), Some(2));
        let error: Value = serde_json::from_slice(&out.stdout).unwrap();
        assert_eq!(error["code"], code);
        assert!(out.stderr.is_empty());
    }
    assert!(!home.path().join(".railway/config.json").exists());
    assert!(!home.path().join(".railway/accounts/absent.json").exists());
}
#[test]
fn named_paths_are_isolated_and_list_order_is_alphabetical() {
    let home = tempfile::tempdir().unwrap();
    profile(home.path(), "lpg");
    profile(home.path(), "brianle");
    let out = run(home.path(), &["account", "list", "--json"]);
    assert!(out.status.success());
    let rows: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(rows[0]["name"], "brianle");
    assert_eq!(rows[1]["name"], "lpg");
    let before = fs::read(home.path().join(".railway/accounts/lpg.json")).unwrap();
    assert!(
        run(home.path(), &["--account", "brianle", "logout"])
            .status
            .success()
    );
    assert_eq!(
        fs::read(home.path().join(".railway/accounts/lpg.json")).unwrap(),
        before
    );
}
