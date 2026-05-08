use chrono::Utc;
use duckdb::{Result, Row, params};
use machines::machine_identification::MachineIdentificationUnique;

use crate::graph::DataPointMarker;

impl super::DataPoint {
    pub fn new(machine_id: MachineIdentificationUnique, data_type: String, value: f64) -> Self {
        return Self {
            id: 0,
            vendor: machine_id.machine_identification.vendor,
            machine: machine_id.machine_identification.machine,
            serial: machine_id.serial,
            data_type: data_type,
            data_timestamp: Utc::now().timestamp_millis(),
            value: value,
        };
    }

    pub fn persist(&self) -> Result<()> {
        let conn = super::get_db_connection()?;
        conn.execute(
            "INSERT INTO data_points (vendor, machine, serial, data_type, data_timestamp, value)
            VALUES (?, ?, ?, ?, ?, ?)",
            params![
                self.vendor,
                self.machine,
                self.serial,
                self.data_type,
                self.data_timestamp,
                self.value
            ],
        )?;
        Ok(())
    }

    pub fn persist_many(pts: Vec<Self>) -> Result<()> {
        let conn = super::get_db_connection()?;
        let mut appender = conn.appender_with_columns(
            "data_points",
            &[
                "vendor",
                "machine",
                "serial",
                "data_type",
                "data_timestamp",
                "value",
            ],
        )?;
        for p in pts {
            appender.append_row(params![
                p.vendor,
                p.machine,
                p.serial,
                p.data_type,
                p.data_timestamp,
                p.value,
            ])?;
        }
        Ok(())
    }

    pub fn get_since_timestamp(
        machine_id: MachineIdentificationUnique,
        data_type: String,
        since_timestamp: i64,
    ) -> Result<Vec<Self>> {
        let conn = super::get_db_connection()?;
        let mut stmt = conn.prepare(
            "SELECT * FROM data_points
            WHERE vendor == ?
            AND machine == ?
            AND serial == ?
            AND data_type == ?
            AND data_timestamp > ?",
        )?;
        let query_iter = stmt.query_map(
            params![
                machine_id.machine_identification.vendor,
                machine_id.machine_identification.machine,
                machine_id.serial,
                data_type,
                since_timestamp
            ],
            Self::map_row,
        )?;

        let mut data = Vec::new();
        for i in query_iter {
            data.push(i?);
        }

        Ok(data)
    }

    pub fn get_after_id(
        machine_id: MachineIdentificationUnique,
        data_type: String,
        since_id: u64,
    ) -> Result<Vec<Self>> {
        let conn = super::get_db_connection()?;
        let mut stmt = conn.prepare(
            "SELECT * FROM data_points
            WHERE vendor == ?
            AND machine == ?
            AND serial == ?
            AND data_type == ?
            AND id > ?",
        )?;
        let query_iter = stmt.query_map(
            params![
                machine_id.machine_identification.vendor,
                machine_id.machine_identification.machine,
                machine_id.serial,
                data_type,
                since_id
            ],
            Self::map_row,
        )?;

        let mut data = Vec::new();
        for i in query_iter {
            data.push(i?);
        }

        Ok(data)
    }

    pub fn delete_before(
        machine_id: MachineIdentificationUnique,
        data_type: String,
        before_timestamp: i64,
    ) -> Result<()> {
        let conn = super::get_db_connection()?;
        conn.execute(
            "DELETE FROM data_points
            WHERE vendor == ?
            AND machine == ?
            AND serial == ?
            AND data_type == ?
            AND data_timestamp < ?",
            params![
                machine_id.machine_identification.vendor,
                machine_id.machine_identification.machine,
                machine_id.serial,
                data_type,
                before_timestamp
            ],
        )?;
        Ok(())
    }

    pub fn delete_all_before(before_timestamp: i64) -> Result<()> {
        let conn = super::get_db_connection()?;
        conn.execute(
            "DELETE FROM data_points WHERE data_timestamp < ?",
            params![before_timestamp],
        )?;
        Ok(())
    }

    fn map_row(row: &Row<'_>) -> Result<Self> {
        Ok(Self {
            id: row.get(0)?,
            vendor: row.get(1)?,
            machine: row.get(2)?,
            serial: row.get(3)?,
            data_type: row.get(4)?,
            data_timestamp: row.get(5)?,
            value: row.get(6)?,
        })
    }
}

impl super::DataPointMarker {
    pub fn new(data_point_id: u64, note: String, color: String) -> Self {
        Self {
            id: 0,
            data_point_id: data_point_id,
            note: note,
            color: color,
        }
    }

    pub fn persist_or_update(&self) -> Result<()> {
        if self.id == 0 {
            self.persist()?;
        } else {
            let conn = super::get_db_connection()?;
            conn.execute(
                "UPDATE data_point_markers SET data_point_id = ?, note = ?, color = ? WHERE id == ?",
                params![self.data_point_id, self.note, self.color, self.id],
            )?;
        }
        Ok(())
    }

    /// No-op if already persisted
    fn persist(&self) -> Result<()> {
        let conn = super::get_db_connection()?;
        if self.id != 0 {
            conn.execute(
                "INSERT INTO data_point_markers (data_point_id, note, color) VALUES (?, ?, ?)",
                params![self.data_point_id, self.note, self.color],
            )?;
        }
        Ok(())
    }

    pub fn get_for_machine_data_type(
        machine_id: MachineIdentificationUnique,
        data_type: String,
    ) -> Result<Vec<DataPointMarker>> {
        let conn = super::get_db_connection()?;
        let mut stmt = conn.prepare(
            "SELECT m.* FROM data_point_markers m
            JOIN data_points p ON m.data_point_id == p.id
            WHERE p.vendor == ?
            AND p.machine == ?
            AND p.serial == ?
            AND p.data_type == ?",
        )?;
        let query_iter = stmt.query_map(
            params![
                machine_id.machine_identification.vendor,
                machine_id.machine_identification.machine,
                machine_id.serial,
                data_type
            ],
            Self::map_row,
        )?;

        let mut data = Vec::new();
        for i in query_iter {
            data.push(i?);
        }

        Ok(data)
    }

    pub fn delete(&self) -> Result<()> {
        let conn = super::get_db_connection()?;
        conn.execute(
            "DELETE FROM data_point_markers WHERE id == ?",
            params![self.id],
        )?;
        Ok(())
    }

    pub fn delete_for_machine_data_type(
        machine_id: MachineIdentificationUnique,
        data_type: String,
    ) -> Result<()> {
        let conn = super::get_db_connection()?;
        conn.execute(
            "DELETE FROM data_point_markers m
            JOIN data_points p ON m.data_point_id == p.id
            WHERE p.vendor == ?
            AND p.machine == ?
            AND p.serial == ?
            AND p.data_type == ?",
            params![
                machine_id.machine_identification.vendor,
                machine_id.machine_identification.machine,
                machine_id.serial,
                data_type
            ],
        )?;
        Ok(())
    }

    fn map_row(row: &Row<'_>) -> Result<Self> {
        Ok(Self {
            id: row.get(0)?,
            data_point_id: row.get(1)?,
            note: row.get(2)?,
            color: row.get(3)?,
        })
    }
}
