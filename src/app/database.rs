use std::io::{BufReader, BufWriter};
use std::fs::File;
use bincode::{serialize_into, deserialize_from};
use std::path::{Path, PathBuf};
use serde::{Serialize, Deserialize};
use std::ffi::OsString;
use crate::app::utils::working_dir;


pub const DB_FILE: &str = "bad-reader.db";

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct DatabaseRow {
    pub file: PathBuf,
    pub filename: OsString,
    pub current_page: usize
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Default, Clone)]
#[serde(rename = "database", default)]
pub struct Database {
    #[serde(rename = "row", default)]
    pub rows: Option<Vec<DatabaseRow>>
}

impl Database {
    pub fn new(rows: Option<Vec<DatabaseRow>>) -> Database {
        Database {
            rows
        }
    }
    
    pub fn write_database(&mut self) {
        let file_result = File::create(&working_dir(DB_FILE));
        match file_result {
            Ok(file) => {
                let mut f = BufWriter::new(file);
                serialize_into(&mut f, self).expect("Cannot serialize into DB_FILE");
                debug!("Wrote to DB!");
            },
            Err(e) => {
                error!("Cannot write DB file. {}", e);
            }
        }
    }
    
    pub fn get_by_row_file(&self, file: &Path) -> Option<DatabaseRow> {
        let filename = file.file_name().unwrap();
        
        if let Some(rows) = &self.rows {
            for row in rows {
                if row.filename == filename {
                    return Some(row.clone());
                }
            }
        }
        
        None
    }
}

pub(crate) fn read_database() -> Database {
    debug!("db file exists: {:?}", Path::new(&working_dir(DB_FILE)).exists());
    
    // Create the db file if it doesn't exist
    if !Path::new(&working_dir(DB_FILE)).exists() {
        let mut db = Database::new(Some(vec![]));
        
        db.write_database();
        
        return db
    }
    // Open the db file and try to deserialize its contents into `Database`
    let db_file = File::open(&working_dir(DB_FILE))
        .expect("DB_FILE does not exist");
    deserialize_from(BufReader::new(db_file))
        .expect("Could not deserialize DB_FILE")
}