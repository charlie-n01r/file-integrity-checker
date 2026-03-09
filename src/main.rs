use clap::{Parser, Subcommand};
use std::path::PathBuf;

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
    let cli = CLI::parse();

    match cli.command {
        Commands::Init { path } => init(path),
        Commands::Update { path } => update(path),
        Commands::Check { path } => check(path),
    }
}

fn init(path: PathBuf) {
    println!("Initializing hashes for {:?}", path);
}

fn update(path: PathBuf) {
    println!("Updating hashes for {:?}", path);
}

fn check(path: PathBuf) {
    println!("Checking hashes for {:?}", path);
}