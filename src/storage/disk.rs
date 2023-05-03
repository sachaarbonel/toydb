use bincode::{deserialize, deserialize_from, serialize, serialize_into};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

use crate::{ColumnDef, DataValue};

use super::Storage;

#[derive(Serialize, Deserialize, Debug)]
pub struct OnDiskStorage {
    file_path: String,
    tables: HashMap<String, (Vec<ColumnDef>, Vec<Vec<DataValue>>)>,
}

impl OnDiskStorage {
    pub fn new(file_path: String) -> Self {
        let path = Path::new(&file_path);

        let mut tables = HashMap::new();
        if path.exists() {
            let mut file = File::open(path).unwrap();
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer).unwrap();
            tables = deserialize(&buffer).unwrap();
            print!("tables: {:?}", tables);
        }
        OnDiskStorage { file_path, tables }
    }

    pub fn save(&self) {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(&self.file_path)
            .unwrap();
        let mut writer = BufWriter::new(file);
        let buffer = serialize(&self.tables).unwrap();
        writer.write_all(&buffer).unwrap();
    }
}

impl Storage for OnDiskStorage {
    type NewArgs = String;

    fn new(args: Self::NewArgs) -> Self {
        OnDiskStorage::new(args)
    }

    fn insert_table(
        &mut self,
        table_name: String,
        columns: Vec<ColumnDef>,
        row: Vec<Vec<DataValue>>,
    ) {
        self.tables.insert(table_name, (columns, row));
        self.save();
    }

    fn get_table(
        &mut self,
        table_name: &str,
    ) -> Option<&mut (Vec<ColumnDef>, Vec<Vec<DataValue>>)> {
        self.tables.get_mut(table_name)
    }

    fn table_exists(&self, table_name: &str) -> bool {
        self.tables.contains_key(table_name)
    }

    fn push_value(&mut self, table_name: &str, row: Vec<DataValue>) {
        let (_, rows) = self.tables.get_mut(table_name).unwrap();
        rows.push(row);
        self.save();
    }

    fn update_table(
        &mut self,
        table_name: &str,
        updates: Vec<(String, DataValue)>,
        where_clause: Option<(String, DataValue)>,
    ) -> usize {
        //TODO: Option<usize>
        // None if table doesn't exist or column not found
        let (schema, rows) = self.get_table(table_name).unwrap();
        let mut idx = 0;

        for row in rows.iter_mut() {
            if let Some((column, value)) = &where_clause {
                let column_idx = schema.iter().position(|c| c.name == *column).unwrap();
                if row[column_idx] != *value {
                    idx += 1;
                    continue;
                }
            }

            for (column, value) in &updates {
                let column_idx = schema.iter().position(|c| c.name == *column).unwrap();
                row[column_idx] = value.clone();
            }
            idx += 1;
        }
        self.save();
        idx
    }

    fn delete_table(
        &mut self,
        table_name: &str,
        where_clause: Option<(String, DataValue)>,
    ) -> usize {
        //TODO: Option<usize>
        // None if table doesn't exist or column not found
        let (schema, rows) = self.get_table(table_name).unwrap();
        let mut idx = 0;

        for row in rows.iter_mut() {
            if let Some((column, value)) = &where_clause {
                let column_idx = schema.iter().position(|c| c.name == *column).unwrap();
                if row[column_idx] != *value {
                    idx += 1;
                    continue;
                }
            }
            idx += 1;
        }
        self.save();
        idx
    }
}
