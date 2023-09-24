use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

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
    #[serde(alias = "type")]
    data_type: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct QueryTable {
    cols: Vec<Column>,
    rows: Vec<Row>,
    #[serde(alias = "parsedNumHeaders")]
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

impl Timestamp {
    fn clone(&self) -> Timestamp {
        Timestamp {
            datetime: self.datetime.clone(),
            timestamp: self.timestamp,
            year: self.year,
            month: self.month,
            day: self.day,
            hour: self.hour,
            min: self.min,
            sec: self.sec,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TroutStocking {
    #[serde(default = "default_key")]
    pub key: String,
    pub version: String,
    #[serde(alias = "reqId")]
    pub req_id: String,
    pub status: String,
    pub sig: String,
    pub table: QueryTable,
    #[serde(default)]
    pub timestamp: Timestamp,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TroutStockingOrganized {
    pub key: String,
    pub version: String,
    pub req_id: String,
    pub status: String,
    pub sig: String,
    pub data: HashMap<String, Vec<Option<String>>>,
    pub timestamp: Timestamp,
}

impl TroutStocking {
    pub fn organize(&self) -> TroutStockingOrganized {
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
            key: self.key.clone(),
            version: self.version.clone(),
            req_id: self.req_id.clone(),
            status: self.status.clone(),
            sig: self.sig.clone(),
            data,
            timestamp: self.timestamp.clone(),
        }
    }
}

/// Default key from UUID.
fn default_key() -> String {
    Uuid::new_v4().to_string()
}
