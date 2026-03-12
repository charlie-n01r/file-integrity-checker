use rusqlite::{Connection, Result};
use tracing::{info, error};
use sha2::{Sha256, Digest};
use std::path::Path;
use std::io;
use std::fs;

use super::db;

fn calculate_hash(path: &Path) -> [u8; 32] {
    let contents = fs::read(path).unwrap();
    Sha256::digest(contents).into()
}

/// CLI Subcommands
fn init(path: &Path, conn: &mut Connection) -> io::Result<()> {
    info!("Hash calculated for {path:?}");
    let hash = calculate_hash(path);
    match path.to_str() {
        Some(file_key) => {let _ = db::create_hash_entry(conn, &file_key, &hash);},
        None => error!("Unable to create a new entry for file")
    };
    println!("Hashes stored successfully.");
    Ok(())
}

fn update(path: &Path, conn: &mut Connection) -> io::Result<()> {
    info!("Updating hashes for {path:?}");
    let hash = calculate_hash(path);
    match path.to_str() {
        Some(file_key) => {let _ = db::update_hash_value(conn, &file_key, &hash);},
        None => error!("Unable to update entry for file")
    };
    Ok(())
}

fn check(path: &Path, conn: &mut Connection) -> io::Result<()> {
    info!("Checking hashes for {path:?}");
    let hash = calculate_hash(path);
    match path.to_str() {
        Some(file_key) => {let _ = db::check_hash(conn, &file_key, &hash);},
        None => error!("Unable to create a new entry for file")
    };
    Ok(())
}


///Function that applies the CLI subcommand to a file or a directory
fn for_each_file<F>(path: &Path, conn: &mut Connection, fun: &mut F) -> io::Result<()>
where
    F: FnMut(&Path, &mut Connection) -> io::Result<()>,
{
    if path.is_file() {
        fun(path, conn)?;
    } else if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let entry_path = entry.path();

            // If entry is a file, apply the closure
            if entry_path.is_file() {
                fun(&entry_path, conn)?;
            }
            // If it's a directory, recurse
            else if entry_path.is_dir() {
                info!("New directory '{entry_path:?}' found in the scope. Added to processing queue.");
                for_each_file(&entry_path, conn, fun)?;
            }
        }
    }

    Ok(())
}


///CLI subcommand executer
pub fn execute_subcommand(path: &Path, subcommand: &str) -> Result<()> {
    let mut conn = db::create_connection()?;

    match subcommand {
        "init" => {let _ = for_each_file(path, &mut conn, &mut init);},
        "update" => {let _ = for_each_file(path, &mut conn, &mut update);},
        "check" => {let _ = for_each_file(path, &mut conn, &mut check);},
        sub => error!("Error executing subcommand, received illegal subcommand {sub}.")
    };
    Ok(())
}