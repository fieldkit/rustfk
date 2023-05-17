use rusqlite_migration::{Migrations, M};

pub(crate) fn get_migrations<'m>() -> Migrations<'m> {
    Migrations::new(vec![M::up(
        r#"
        CREATE TABLE station (
            id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
            device_id TEXT NOT NULL,
            generation_id TEXT NOT NULL,
            name TEXT NOT NULL,
            firmware TEXT NOT NULL,
            last_seen DATETIME NOT NULL,
            meta_size INTEGER NOT NULL,
            meta_records INTEGER NOT NULL,
            data_size INTEGER NOT NULL,
            data_records INTEGER NOT NULL,
            battery_percentage REAL NOT NULL,
            battery_voltage REAL NOT NULL,
            solar_voltage REAL NOT NULL,
            status BLOB
        );

        CREATE UNIQUE INDEX station_idx_device_id ON station (device_id);

        CREATE TABLE module (
            id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
            station_id INTEGER NOT NULL REFERENCES station(id),
            hardware_id TEXT NOT NULL,
            manufacturer INTEGER NOT NULL,
            kind INTEGER NOT NULL,
            version INTEGER NOT NULL,
            flags INTEGER NOT NULL,
            position INTEGER NOT NULL,
            key TEXT NOT NULL,
            path TEXT NOT NULL,
            configuration BLOB,
            removed BOOL NOT NULL
        );

        CREATE INDEX module_idx_station_id ON module (station_id);
        CREATE INDEX module_idx_hardware_id ON module (hardware_id);

        CREATE TABLE sensor (
            id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
            module_id INTEGER NOT NULL REFERENCES module(id),
            number INTEGER NOT NULL,
            flags INTEGER NOT NULL,
            key TEXT NOT NULL,
            calibrated_uom TEXT NOT NULL,
            uncalibrated_uom TEXT NOT NULL,
            calibrated_value REAL,
            uncalibrated_value REAL,
            removed BOOL NOT NULL
        );

        CREATE INDEX sensor_idx_module_id ON sensor (module_id);

        CREATE TABLE sync_download (
            id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
            station_id INTEGER NOT NULL REFERENCES station(id),
            generation_id TEXT NOT NULL,
            started DATETIME NOT NULL,
            begin INTEGER NOT NULL,
            end INTEGER NOT NULL,
            finished DATETIME,
            error TEXT
        );

        CREATE INDEX sync_download_idx_station_id ON sync_download (station_id);

        CREATE TABLE sync_upload (
            id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
            station_id INTEGER NOT NULL REFERENCES station(id),
            generation_id TEXT NOT NULL,
            started DATETIME NOT NULL,
            begin INTEGER NOT NULL,
            end INTEGER NOT NULL,
            finished DATETIME,
            error TEXT
        );

        CREATE INDEX sync_upload_idx_station_id ON sync_upload (station_id);
        "#,
    )])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migrations_validity() {
        assert!(get_migrations().validate().is_ok());
    }
}
