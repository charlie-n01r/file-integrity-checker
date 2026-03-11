use std::os::unix::fs::OpenOptionsExt;
use tracing_subscriber::fmt;
use tracing::{info, error};
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};
use sha2::{Sha256, Digest};
use std::fs::OpenOptions;
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

fn main() {
    // Create today's log file and enforce permissions
    let today = chrono::Local::now().format("%Y-%m-%d");
    let log_name = format!("logs/{}-log.json", today);
    let log_file = match open_log_file(&log_name){
        Some(file) => file,
        None => {
            std::process::exit(1);
        }
    };

    // Create log appender that writes or appends to daily logs
    let (non_blocking, _guard) = tracing_appender::non_blocking(log_file);

    fmt()
        .json()
        .with_writer(non_blocking)
        .with_max_level(tracing::Level::INFO)
        .init();

    // Parse CLI input
    let cli = CLI::parse();

    match cli.command {
        Commands::Init { path } => {
            info!("Calculating initial hash values.");
            let _ = for_each_file(&path, &mut init);
        },
        Commands::Update { path } => {
            info!("Updating existing hash values.");
            let _ = for_each_file(&path, &mut update);
        },
        Commands::Check { path } => {
            info!("Checking and verifying hash values.");
            let _ = for_each_file(&path, &mut check);
        }
    }
}

fn calculate_hash(path: &Path) -> [u8; 32] {
    let contents = fs::read(path).unwrap();
    Sha256::digest(contents).into()
}

fn open_log_file(path: &str) -> Option<std::fs::File> {
    match OpenOptions::new()
        .create(true)
        .append(true)
        .mode(0o640)
        .open(path)
        {
            Ok(file) => Some(file),
            Err(e) => {
                error!(file = path, error = %e, "Failed to open log file");
                None
            }
        }
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
                info!("New directory '{entry_path:?}' found in the scope. Added to processing queue.");
                for_each_file(&entry_path, fun)?;
            }
        }
    }

    Ok(())
}