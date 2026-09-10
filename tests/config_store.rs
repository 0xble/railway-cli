//! Credential-store corruption is handled at the public CLI boundary.
// Windows release binaries ignore HOME and have no test-home override; keep
// these fixtures away from a developer's actual known-folder credential store.
#![cfg(any(unix, debug_assertions))]

use std::{fs, path::Path, process::Command};

fn command(home: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_railway"));
    command.env_clear();
    for key in ["SystemRoot", "WINDIR"] {
        if let Some(value) = std::env::var_os(key) {
            command.env(key, value);
        }
    }
    command
        .env("HOME", home)
        .env("USERPROFILE", home)
        .env("RAILWAY_TEST_HOME", home)
        .env("PATH", "/usr/bin:/bin")
        .env("DO_NOT_TRACK", "1")
        .env("RAILWAY_NO_AUTO_UPDATE", "1")
        .current_dir(home);
    command
}

#[test]
fn malformed_credential_store_is_preserved_and_its_contents_are_not_reported() {
    let home = tempfile::tempdir().unwrap();
    let config = home.path().join(".railway/config.json");
    fs::create_dir_all(config.parent().unwrap()).unwrap();
    let corrupted = br#"{"user":{"token":"fixture-secret"},"projects": "truncated""#;
    fs::write(&config, corrupted).unwrap();

    let output = command(home.path()).arg("logout").output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !output.status.success(),
        "logout unexpectedly succeeded: {stderr}"
    );
    assert!(
        stderr.contains("credential store") && stderr.contains("refusing to overwrite"),
        "expected actionable corruption diagnostic, got: {stderr}"
    );
    assert!(
        !stderr.contains("fixture-secret"),
        "credential value leaked in diagnostic: {stderr}"
    );
    assert_eq!(fs::read(&config).unwrap(), corrupted);
}
