use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct DataValue {
    v: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Row {
    c: Vec<Option<DataValue>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Column {
    id: String,
    label: String,
    #[serde(rename(deserialize = "type"))]
    data_type: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct QueryTable {
    cols: Vec<Column>,
    rows: Vec<Row>,
    #[serde(rename(deserialize = "parsedNumHeaders"))]
    parsed_num_headers: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Timestamp {
    pub datetime: String,
    pub timestamp: i64,
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub hour: u32,
    pub min: u32,
    pub sec: u32,
}

impl Default for Timestamp {
    fn default() -> Self {
        let utc: DateTime<Utc> = Utc::now();
        Timestamp {
            datetime: utc.to_string(),
            timestamp: utc.timestamp(),
            year: utc.year(),
            month: utc.month(),
            day: utc.day(),
            hour: utc.hour(),
            min: utc.minute(),
            sec: utc.second(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TroutStocking {
    pub version: String,
    #[serde(rename(deserialize = "reqId"))]
    pub req_id: String,
    pub status: String,
    pub sig: String,
    pub table: QueryTable,
    #[serde(default)]
    pub timestamp: Timestamp,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TroutStockingOrganized {
    pub version: String,
    pub req_id: String,
    pub status: String,
    pub sig: String,
    pub data: HashMap<String, Vec<Option<String>>>,
    pub timestamp: Timestamp,
}

impl TroutStocking {
    pub fn organize(self) -> TroutStockingOrganized {
        let mut data = HashMap::new();
        for (c, col) in self.table.cols.iter().enumerate() {
            if !col.label.is_empty() {
                let values = self
                    .table
                    .rows
                    .iter()
                    .map(|r| match &r.c[c] {
                        Some(x) => x.v.clone(),
                        None => None,
                    })
                    .collect();
                data.insert(col.label.clone(), values);
            }
        }
        TroutStockingOrganized {
            version: self.version,
            req_id: self.req_id,
            status: self.status,
            sig: self.sig,
            data,
            timestamp: self.timestamp,
        }
    }
}
