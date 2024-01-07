use std::{
    env, fs,
    os::unix::{self, ffi::OsStringExt},
    path::{Path, PathBuf},
    process::Command,
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
        .output()?;
    let store_path = PathBuf::from(String::from_utf8(build_res.stdout)?.trim());
    let shell_link = Path::new(STATE_DIR).join(store_path.file_name().unwrap());

    if shell_link.exists() {
        fs::remove_file(&shell_link)?;
    }
    unix::fs::symlink(&store_path, &shell_link)?;

    fs::write(Path::new(HASH_FILE), Sha256::digest(fs::read(nix_file)?))?;

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let nix_file = cli
        .dir
        .read_dir()?
        .filter_map(Result::ok)
        .filter_map(|e| String::from_utf8(e.file_name().into_vec()).ok())
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
