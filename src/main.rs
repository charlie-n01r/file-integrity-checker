use std::os::unix::fs::OpenOptionsExt;
use tracing_subscriber::fmt;
use tracing::{info, error};
use clap::{Parser, Subcommand};
use std::fs::OpenOptions;
use std::path::PathBuf;

mod modules;
use crate::modules::hash_files;

/// CLI parser
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
            hash_files::execute_subcommand(&path, "init");
        },
        Commands::Check { path } => {
            info!("Checking and verifying hash values.");
            hash_files::execute_subcommand(&path, "check");
        },
        Commands::Update { path } => {
            info!("Updating existing hash values.");
            hash_files::execute_subcommand(&path, "update");
        }
    }
}


// Logging functionality
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