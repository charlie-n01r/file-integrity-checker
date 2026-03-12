use rusqlite::{Connection, Result};
use std::{env, path::PathBuf};
use tracing::{info, error};

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
        Ok(_) => info!("Successfully inserted record into database"),
        Err(e) => {
            error!(?e, "Failed to commit transaction");
            return Err(e);
        }
    }
    Ok(())
}

///Check
pub fn check_hash(conn: &Connection, file_key: &str) -> Result<()> {
    let mut statement = match conn.prepare("SELECT path, hash FROM files WHERE path = (?1)") {
        Ok(stmt) => stmt,
        Err(e) => {
            error!(?e, "Failed to prepare insert statement");
            return Err(e);
        }
    };
    let files = statement.query_map([file_key], |row| {
        let path: String = row.get(0)?;
        let hash: Vec<u8> = row.get(1)?;
        Ok((path, hash))
    })?;

    for row in files {
        let (path, hash) = row?;

        // Print hash as hex
        let hex_hash: String = hash.iter()
            .map(|b| format!("{:02x}", b))
            .collect();

        println!("Path: {}", path);
        println!("Hash: {}", hex_hash);
        println!();
    };
    Ok(())
}