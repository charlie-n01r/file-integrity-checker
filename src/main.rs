use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};
use sha2::{Sha256, Digest};
use env_logger;

use std::io;
use std::fs;

#[derive(Parser)]
#[command(name = "integrity-check", version = None)]
#[command(about = "A tool that verifies the integrity of \
application log files to detect changes and tampering.")]
struct CLI {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize and store the hash value of a file
    /// or all the hash valus for all files in a directory
    Init {
        path: PathBuf,
    },
    /// Verify the integrity of a file or all files in a directory
    Check {
        path: PathBuf,
    },
    /// Update the current hash value of a file
    /// or all the hash values for all files in a directory
    Update {
        path: PathBuf,
    },
}

fn main() -> io::Result<()> {
    let cli = CLI::parse();

    match cli.command {
        Commands::Init { path } => for_each_file(&path, &mut init),
        Commands::Update { path } => for_each_file(&path, &mut update),
        Commands::Check { path } => for_each_file(&path, &mut check),
    }
}

fn init(path: &Path) -> io::Result<()> {
    println!("Initializing hashes for {:?}", path);
    let hash = calculate_hash(path);
    let hex: String = hash.iter()
        .map(|b| format!("{:02x}", b))
        .collect();
    println!("{hex}");

    Ok(())
}

fn update(path: &Path) -> io::Result<()> {
    println!("Updating hashes for {:?}", path);
    Ok(())
}

fn check(path: &Path) -> io::Result<()> {
    println!("Checking hashes for {:?}", path);
    Ok(())
}

fn calculate_hash(path: &Path) -> [u8; 32] {
    let contents = fs::read(path).unwrap();
    Sha256::digest(contents).into()
}


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
                for_each_file(&entry_path, fun)?;
            }
        }
    }

    Ok(())
}