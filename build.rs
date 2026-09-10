fn fork_policy() -> Option<(String, bool)> {
    let manifest_path =
        std::path::Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("Cargo.toml");
    let manifest = std::fs::read_to_string(manifest_path).expect("failed to read Cargo.toml");
    let manifest: toml::Value = manifest.parse().expect("failed to parse Cargo.toml");
    let fork = manifest
        .get("package")?
        .get("metadata")?
        .get("railway")?
        .get("fork")?;
    let provenance = fork.get("provenance")?.as_str()?.trim().to_owned();
    let block_upstream_self_updates = fork.get("block_upstream_self_updates")?.as_bool()?;

    assert!(
        !provenance.is_empty() && !provenance.contains(['\n', '\r']),
        "package.metadata.railway.fork.provenance must be a non-empty single line"
    );
    Some((provenance, block_upstream_self_updates))
}

// Rebuild when the GraphQL queries and mutations change.
fn main() {
    println!("cargo:rerun-if-changed=src/gql/queries/strings");
    println!("cargo:rerun-if-changed=src/gql/mutations/strings");
    println!("cargo:rerun-if-changed=src/gql/subscriptions/strings");
    println!("cargo:rerun-if-changed=src/gql/schema.json");

    // Expose the compile-time target triple so the self-updater fetches the
    // correct release asset (respects ABI: gnu vs musl, msvc vs gnu, etc.).
    let target = std::env::var("TARGET").unwrap();
    println!("cargo:rustc-env=BUILD_TARGET={target}");

    // A maintained fork may declare a compile-time identity and opt out of
    // replacing itself with Railway's upstream releases. The absence of this
    // metadata is intentionally the upstream-compatible default.
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rustc-check-cfg=cfg(railway_fork_build)");
    println!("cargo:rustc-check-cfg=cfg(railway_upstream_self_update_blocked)");
    if let Some((provenance, block_upstream_self_updates)) = fork_policy() {
        let git = |args: &[&str]| {
            std::process::Command::new("git")
                .args(args)
                .output()
                .ok()
                .filter(|output| output.status.success())
                .and_then(|output| String::from_utf8(output.stdout).ok())
                .map(|value| value.trim().to_owned())
        };
        let commit = git(&["rev-parse", "HEAD"]).unwrap_or_else(|| "unavailable".into());
        println!("cargo:rustc-env=RAILWAY_SOURCE_COMMIT={commit}");
        for reference in [
            Some("HEAD".to_owned()),
            git(&["symbolic-ref", "-q", "HEAD"]),
        ]
        .into_iter()
        .flatten()
        {
            if let Some(path) = git(&["rev-parse", "--git-path", &reference]) {
                println!("cargo:rerun-if-changed={path}");
            }
        }
        println!("cargo:rustc-env=RAILWAY_FORK_PROVENANCE={provenance}");
        println!("cargo:rustc-cfg=railway_fork_build");
        if block_upstream_self_updates {
            println!("cargo:rustc-cfg=railway_upstream_self_update_blocked");
        }
    }
}
