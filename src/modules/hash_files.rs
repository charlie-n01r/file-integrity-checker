use tracing::{info, error};
use sha2::{Sha256, Digest};
use std::path::Path;
use std::io;
use std::fs;

fn calculate_hash(path: &Path) -> [u8; 32] {
    let contents = fs::read(path).unwrap();
    Sha256::digest(contents).into()
}

/// CLI Subcommands
fn init(path: &Path) -> io::Result<()> {
    info!("Hash calculated for {path:?}");
    let hash = calculate_hash(path);
    let hex: String = hash.iter()
        .map(|b| format!("{:02x}", b))
        .collect();
    info!("SHA256 value: {hex}");

    Ok(())
}

fn update(path: &Path) -> io::Result<()> {
    info!("Updating hashes for {path:?}");
    Ok(())
}

fn check(path: &Path) -> io::Result<()> {
    info!("Checking hashes for {path:?}");
    Ok(())
}


///Function that applies the CLI subcommand to a file or a directory
fn for_each_file<F>(path: &Path, fun: &mut F) -> io::Result<()>
where
    F: FnMut(&Path) -> io::Result<()>,
{
    if path.is_file() {
        fun(path)?;
    } else if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let entry_path = entry.path();

            // If entry is a file, apply the closure
            if entry_path.is_file() {
                fun(&entry_path)?;
            }
            // If it's a directory, recurse
            else if entry_path.is_dir() {
                info!("New directory '{entry_path:?}' found in the scope. Added to processing queue.");
                for_each_file(&entry_path, fun)?;
            }
        }
    }

    Ok(())
}


///CLI subcommand executer
pub fn execute_subcommand(path: &Path, subcommand: &str) {
    match subcommand {
        "init" => {let _ = for_each_file(path, &mut init);},
        "check" => {let _ = for_each_file(path, &mut check);},
        "update" => {let _ = for_each_file(path, &mut update);},
        sub => {error!("Error executing subcommand, received illegal subcommand {sub}.");}
    };
}