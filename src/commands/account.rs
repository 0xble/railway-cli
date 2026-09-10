use super::*;
use crate::config::{RailwayConfig, secure_config_dir};
use std::{fs, io::Write};

/// Inspect named Railway accounts or explicitly import the legacy login.
#[derive(Parser)]
pub struct Args {
    #[clap(subcommand)]
    command: Command,
}
#[derive(Parser)]
enum Command {
    /// List local credential metadata (never tokens; no network requests)
    List {
        #[clap(long)]
        json: bool,
    },
    /// Copy the legacy login without changing it or overwriting a named account
    ImportLegacy { name: String },
}

pub async fn command(args: Args) -> Result<()> {
    match args.command {
        Command::List { json } => {
            let mut rows = Vec::new();
            for name in Configs::account_names()? {
                let path = Configs::account_config_path(&name)?;
                let config: RailwayConfig = serde_json::from_slice(&fs::read(&path)?)
                    .with_context(|| {
                        format!("Unable to read account {name:?}; file left unchanged")
                    })?;
                let user = config.user;
                let authenticated = user
                    .access_token
                    .as_ref()
                    .or(user.token.as_ref())
                    .is_some_and(|token| !token.is_empty());
                rows.push(serde_json::json!({
                    "name": name, "userId": user.id, "authenticated": authenticated,
                    "authType": if user.access_token.is_some() { "oauth" } else if user.token.is_some() { "legacy" } else { "none" },
                    "tokenExpiresAt": user.token_expires_at,
                    "linkedProjects": config.projects.len(),
                }));
            }
            if json {
                println!("{}", serde_json::to_string_pretty(&rows)?);
            } else if rows.is_empty() {
                println!(
                    "No named accounts. Run `railway login --account <NAME>` or `railway account import-legacy <NAME>`."
                );
            } else {
                for row in rows {
                    println!(
                        "{}\t{}\tuser={}\tlinked_projects={}",
                        row["name"].as_str().unwrap(),
                        if row["authenticated"] == true {
                            "credentials saved"
                        } else {
                            "logged out"
                        },
                        row["userId"].as_str().unwrap_or("unknown"),
                        row["linkedProjects"]
                    );
                }
            }
            Ok(())
        }
        Command::ImportLegacy { name } => {
            let destination = Configs::account_config_path(&name)?;
            let source = Configs::legacy_config_path()?;
            let bytes = fs::read(&source).context(
                "No readable legacy login. Run `railway login --account <NAME>` instead.",
            )?;
            let config: RailwayConfig = serde_json::from_slice(&bytes)
                .context("Legacy config is invalid; refusing to import it")?;
            if !config
                .user
                .access_token
                .as_ref()
                .or(config.user.token.as_ref())
                .is_some_and(|s| !s.is_empty())
            {
                bail!(
                    "Legacy config has no login credentials. Run `railway login --account {name}`."
                );
            }
            let parent = destination
                .parent()
                .context("account config has no parent directory")?;
            fs::create_dir_all(parent)?;
            secure_config_dir(parent)?;
            let mut tmp = tempfile::NamedTempFile::new_in(parent)?;
            tmp.write_all(&bytes)?;
            tmp.as_file().sync_all()?;
            // Atomic no-clobber publication; even a concurrent importer or a
            // destination symlink cannot overwrite another account/legacy file.
            tmp.persist_noclobber(&destination).with_context(|| {
                format!(
                    "Account {name:?} already exists or cannot be created; refusing to overwrite it"
                )
            })?;
            println!(
                "Imported legacy login as account {name:?}. The legacy file was left unchanged."
            );
            Ok(())
        }
    }
}
