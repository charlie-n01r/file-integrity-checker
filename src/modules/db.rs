use rusqlite::{Connection, Result};
use std::{env, path::PathBuf};
use tracing::{info, error};

pub enum FlowControl {
    Continue,
    Stop
}

///Create connection to the hash database
pub fn create_connection() -> Result<Connection> {
    let mut db_path = PathBuf::new();
    match env::home_dir() {
        Some(home_path) => {
            db_path.push(home_path);
            db_path.push(".local/share/hashcheck/hashes.db");
        },
        None => {
            error!("Could not find home directory for database path");
            return Err(rusqlite::Error::InvalidPath(PathBuf::new()))
        }
    };
    match Connection::open(&db_path) {
        Ok(conn) => {
            info!("Successfully connected to database at {:?}", db_path);
            Ok(conn)
        }
        Err(e) => {
            error!(?e, "Failed to connect to database at {:?}", db_path);
            Err(e)
        }
    }
}

///Init
pub fn create_hash_entry(conn: &mut Connection, file_key: &str, hash_value: &[u8; 32]) -> Result<()> {
    let transaction = match conn.transaction() {
        Ok(tx) => tx,
        Err(e) => {
            error!(?e, "Failed to start database transaction!");
            return Err(e);
        }
    };

    {
        let mut statement = match transaction.prepare("INSERT OR REPLACE INTO files (path, hash) VALUES (?1, ?2)") {
            Ok(stmt) => stmt,
            Err(e) => {
                error!(?e, "Failed to prepare insert statement");
                return Err(e);
            }
        };
        if let Err(e) = statement.execute((file_key, &hash_value[..])) {
            error!(?e, %file_key, "Failed to execute insert for a file");
            return Err(e);
        }
        
    }
    match transaction.commit() {
        Ok(_) => info!("Successfully inserted record into database"),
        Err(e) => {
            error!(?e, "Failed to commit transaction");
            return Err(e);
        }
    }

    Ok(())
}

///Update
pub fn update_hash_value(conn: &mut Connection, file_key: &str, hash_value: &[u8; 32]) -> Result<()> {
    let transaction = match conn.transaction() {
        Ok(tx) => tx,
        Err(e) => {
            error!(?e, "Failed to start database transaction!");
            return Err(e);
        }
    };

    {
        let mut statement = match transaction.prepare("UPDATE files SET hash = (?1) WHERE path = (?2)") {
            Ok(stmt) => stmt,
            Err(e) => {
                error!(?e, "Failed to prepare update statement");
                return Err(e);
            }
        };
        if let Err(e) = statement.execute((&hash_value[..], file_key)) {
            error!(?e, %file_key, "Failed to execute insert for a file");
            return Err(e);
        }
    }

    match transaction.commit() {
        Ok(_) => info!("Hash updated successfully for file '{file_key}'"),
        Err(e) => {
            error!(?e, "Failed to commit transaction");
            return Err(e);
        }
    }
    Ok(())
}

///Check
pub fn check_hash(conn: &Connection, file_key: &str, hash_value: &[u8; 32]) -> Result<FlowControl> {
    let mut statement = match conn.prepare("SELECT hash FROM files WHERE path = (?1)") {
        Ok(stmt) => stmt,
        Err(e) => {
            error!(?e, "Failed to prepare insert statement");
            return Err(e);
        }
    };
    let stored_hash: Vec<u8> = match statement.query_row([file_key], |row| row.get(0)) {
        Ok(hash) => hash,
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            error!("No entry found for path: {}", file_key);
            return Ok(FlowControl::Stop); // or return Err(...) if you want
        }
        Err(e) => {
            error!(?e, "Error querying hash for path: {}", file_key);
            return Err(e);
        }
    };

    if stored_hash.as_slice() == hash_value {
        info!("File '{file_key}' remains unmodified");
        Ok(FlowControl::Continue)
    } else {
        error!("File '{file_key}' has been modified");
        println!("Status: File '{file_key}' was modified (Hash mismatch).");
        Ok(FlowControl::Stop)
    }
}