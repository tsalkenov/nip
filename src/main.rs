use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use anyhow::Context;
use clap::Parser;
use sha2::{Digest, Sha256};

const STATE_DIR: &str = ".nip";
const HASH_FILE: &str = ".nip/lock.hash";

#[derive(Parser)]
struct Cli {
    #[arg(default_value = ".")]
    /// Directory with shell nix file
    dir: PathBuf,
}

fn build_and_save(nix_file: PathBuf) -> anyhow::Result<()> {
    let build_res = Command::new("nix-build")
        .args([
            nix_file.file_name().unwrap().to_str().unwrap(),
            "--no-out-link",
        ])
        .stderr(Stdio::inherit())
        .output()?;
    let store_path = PathBuf::from(String::from_utf8(build_res.stdout)?.trim());
    let shell_link = Path::new(STATE_DIR).join(
        store_path
            .file_name()
            .context("Could not get build path from build command")?,
    );

    if shell_link.exists() {
        fs::remove_file(&shell_link)?;
    }

    Command::new("nix-store").args([
        "--add-root",
        &shell_link.to_string_lossy(),
        "--realise",
        &store_path.to_string_lossy(),
    ]).spawn()?;

    fs::write(Path::new(HASH_FILE), Sha256::digest(fs::read(nix_file)?))?;

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let nix_file = cli
        .dir
        .read_dir()?
        .filter_map(Result::ok)
        .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
        .filter_map(|e| e.file_name().to_str().map(str::to_owned))
        .find(|file_name| file_name.ends_with("nix"))
        .context("Nix file not found")?;

    if nix_file.as_str() == "flake.nix" {
        anyhow::bail!("Flakes not yet supported");
    };

    if Path::new(STATE_DIR).exists() {
        let new_hash = Sha256::digest(fs::read(cli.dir.join(&nix_file))?);
        let old_hash = fs::read(Path::new(HASH_FILE))?;

        if new_hash.as_slice() != old_hash.as_slice() {
            build_and_save(cli.dir.join(nix_file))?;
        };
    } else {
        fs::create_dir(cli.dir.join(STATE_DIR))?;
        build_and_save(cli.dir.join(nix_file))?;
    }

    let exit_status = Command::new("nix-shell")
        .envs(env::vars())
        .spawn()?
        .wait()?;

    if !exit_status.success() {
        anyhow::bail!("Shell failed with {}", exit_status.code().unwrap_or(1));
    }

    Ok(())
}
