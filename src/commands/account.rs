use std::fs;

use super::*;

/// Inspect named Railway accounts or explicitly import the legacy login.
#[derive(Parser)]
pub struct Args {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Parser)]
enum Command {
    /// List configured account names (never prints tokens)
    List,
    /// Copy the legacy ~/.railway/config.json login into a named account without changing the legacy file
    ImportLegacy { name: String },
}

pub async fn command(args: Args) -> Result<()> {
    match args.command {
        Command::List => {
            let accounts = Configs::account_names()?;
            if accounts.is_empty() {
                println!(
                    "No named accounts. Run `railway login --account <NAME>` or `railway account import-legacy <NAME>`."
                );
            } else {
                for account in accounts {
                    println!("{account}");
                }
            }
            Ok(())
        }
        Command::ImportLegacy { name } => {
            let source = Configs::legacy_config_path()?;
            if !source.is_file() {
                bail!(
                    "No legacy login found at {}. Run `railway login --account {name}` instead.",
                    source.display()
                );
            }
            let destination = Configs::account_config_path(&name)?;
            if destination.exists() {
                bail!("Account {name:?} already exists; refusing to overwrite it.");
            }
            let parent = destination
                .parent()
                .context("account config has no parent directory")?;
            fs::create_dir_all(parent)?;
            fs::copy(&source, &destination)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(&destination, fs::Permissions::from_mode(0o600))?;
            }
            println!(
                "Imported legacy login as account {name:?}. The legacy file was left unchanged."
            );
            Ok(())
        }
    }
}
