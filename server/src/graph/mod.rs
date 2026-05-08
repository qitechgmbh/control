use std::{env, fs};

use duckdb::{Connection, Result};
use serde::{Deserialize, Serialize};

pub mod api;
pub mod collector;
pub mod db_ops;

const DB_PATH_SYS: &str = "/var/qitech";
const DB_PATH_USR: &str = "/.local/share/qitech/qitech_control.duckdb";
const DB_FILENAME: &str = "/qitech_control.duckdb";

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct DataPoint {
    id: u64,
    vendor: u16,
    machine: u16,
    serial: u16,
    data_type: String,
    data_timestamp: i64,
    value: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct DataPointMarker {
    id: u32,
    data_point_id: u64,
    note: String,
    color: String,
}

pub fn db_init() -> Result<()> {
    let conn = get_db_connection()?;
    conn.execute_batch(
        r"CREATE SEQUENCE IF NOT EXISTS data_points_seq;
          CREATE TABLE IF NOT EXISTS data_points (
            id UINT64 PRIMARY KEY DEFAULT NEXTVAL('data_points_seq'),
            vendor UINT16 NOT NULL,
            machine UINT16 NOT NULL,
            serial UINT16 NOT NULL,
            data_type VARCHAR NOT NULL,
            data_timestamp TIMESTAMP_MS NOT NULL,
            value DOUBLE NOT NULL
          );
          CREATE SEQUENCE IF NOT EXISTS data_point_markers_seq;
          CREATE TABLE IF NOT EXISTS data_point_markers (
            id UINT32 PRIMARY KEY DEFAULT NEXTVAL('data_point_markers_seq'),
            data_point_id UINT64 NOT NULL,
            note VARCHAT NOT NULL,
            color VARCHAR NOT NULL DEFAULT '#000000',
            FOREIGN KEY (data_point_id) REFERENCES data_points(id)
          );
         ",
    )?;
    Ok(())
}

pub fn get_db_connection() -> Result<Connection> {
    return Connection::open(db_path().expect("I/O error while opening the database"));
}

fn db_path() -> std::io::Result<String> {
    let sys_path_md = fs::metadata("/var").expect("/var should exist");
    if sys_path_md.permissions().readonly() {
        let usr_path =
            env::var("HOME").expect("server needs root access or home dir") + DB_PATH_USR;
        fs::create_dir_all(&usr_path)?;
        return Ok(usr_path.to_owned() + DB_FILENAME);
    }
    fs::create_dir_all(DB_PATH_SYS)?;
    Ok(DB_PATH_SYS.to_owned() + DB_FILENAME)
}
