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
fn init(path: &Path, conn: &mut Connection) -> io::Result<db::FlowControl> {
    info!("Hash calculated for {path:?}");
    let hash = calculate_hash(path);
    match path.to_str() {
        Some(file_key) => {let _ = db::create_hash_entry(conn, &file_key, &hash);},
        None => error!("Unable to create a new entry for file")
    };
    Ok(db::FlowControl::Continue)
}

fn update(path: &Path, conn: &mut Connection) -> io::Result<db::FlowControl> {
    info!("Updating hashes for {path:?}");
    let hash = calculate_hash(path);
    match path.to_str() {
        Some(file_key) => {let _ = db::update_hash_value(conn, &file_key, &hash);},
        None => error!("Unable to update entry for file")
    };
    Ok(db::FlowControl::Continue)
}

fn check(path: &Path, conn: &mut Connection) -> io::Result<db::FlowControl> {
    info!("Checking hashes for {path:?}");
    let hash = calculate_hash(path);
    match path.to_str() {
        Some(file_key) => {
            match db::check_hash(conn, &file_key, &hash) {
                Ok(flow) => return Ok(flow),
                Err(e) => return Err(io::Error::new(io::ErrorKind::Other, e))
            }
        },
        None => {
            error!("Unable to create a new entry for file");
            return Ok(db::FlowControl::Continue);
        }
    }
}


///Function that applies the CLI subcommand to a file or a directory
fn for_each_file<F>(path: &Path, conn: &mut Connection, fun: &mut F) -> io::Result<db::FlowControl>
where
    F: FnMut(&Path, &mut Connection) -> io::Result<db::FlowControl>,
{
    if path.is_file() {
        match fun(path, conn)? {
            db::FlowControl::Continue => {},
            db::FlowControl::Stop => return Ok(db::FlowControl::Stop),
        }
    } else if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let entry_path = entry.path();

            if entry_path.is_file() {
                match fun(&entry_path, conn)? {
                    db::FlowControl::Continue => {},
                    db::FlowControl::Stop => return Ok(db::FlowControl::Stop),
                }
            } else if entry_path.is_dir() {
                info!("New directory '{entry_path:?}' found in the scope. Added to processing queue.");
                match for_each_file(&entry_path, conn, fun)? {
                    db::FlowControl::Continue => {},
                    db::FlowControl::Stop => return Ok(db::FlowControl::Stop),
                }
            }
        }
    }

    Ok(db::FlowControl::Continue)
}


///CLI subcommand executer
pub fn execute_subcommand(path: &Path, subcommand: &str) -> Result<db::FlowControl> {
    let mut conn = db::create_connection()?;

    match subcommand {
        "init" => {
            match for_each_file(path, &mut conn, &mut init) {
                Err(e) => {
                    error!(?e, "Error occurred while initializing hashes.");
                    println!("Error occurred while initializing hashes.");
                },
                _ => println!("Hashes stored successfully.")
            };
        },
        "update" => {
            match for_each_file(path, &mut conn, &mut update) {
                Err(e) => {
                    error!(?e, "Error occurred while updating hashes.");
                    println!("Error occurred while updating hashes.");
                },
                _ => println!("Hashes updated successfully.")
            };
        },
        "check" => {
            match for_each_file(path, &mut conn, &mut check) {
                Ok(db::FlowControl::Continue) => {
                    println!("Status: Unmodified.");
                },
                Ok(db::FlowControl::Stop) => info!("Stopped checking files due to a detected modification."),
                Err(e) => {
                    error!(?e, "Error occurred while checking files.");
                }
            };
        },
        sub => error!("Error executing subcommand, received illegal subcommand {sub}.")
    };
    Ok(db::FlowControl::Continue)
}