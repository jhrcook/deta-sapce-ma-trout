use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::prelude::*;
use detalib::Deta;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, env};
use std::{
    fmt::{self, Display, Formatter},
    net::SocketAddr,
};
use trout_scraping::models::{TroutStocking, TroutStockingOrganized};
use trout_scraping::scraping::{
    get_spreadsheet_url, get_trout_stocking_page, parse_trout_stocking_spreadsheet_data,
};

#[macro_use]
extern crate log;

pub enum DetaBase {
    TroutStockingRaw,
    TroutStocking,
    TroutStockingIndex,
}

impl DetaBase {
    fn as_str(&self) -> &'static str {
        match self {
            DetaBase::TroutStockingRaw => "trout-stocking-raw",
            DetaBase::TroutStocking => "trout-stocking",
            DetaBase::TroutStockingIndex => "trout-stocking-index",
        }
    }
}

impl Display for DetaBase {
    // Required method
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

pub trait DetaPut: Serialize {
    const DETA_BASE: DetaBase;

    fn deta_put(&self, deta: &Deta) {
        debug!("Connecting to Deta Base: {}", Self::DETA_BASE);
        let base = deta.base(Self::DETA_BASE.as_str());
        match base.put(vec![self]) {
            Ok(_) => debug!("Successfully PUT data."),
            Err(e) => {
                error!("Error during PUT data: {}", e);
                panic!("Could not PUT data.")
            }
        }
    }
}

impl DetaPut for TroutStocking {
    const DETA_BASE: DetaBase = DetaBase::TroutStockingRaw;
}

impl DetaPut for Vec<TroutStocking> {
    const DETA_BASE: DetaBase = TroutStocking::DETA_BASE;
    fn deta_put(&self, deta: &Deta) {
        self.iter().for_each(|d| d.deta_put(deta));
    }
}

impl DetaPut for TroutStockingOrganized {
    const DETA_BASE: DetaBase = DetaBase::TroutStocking;
}

impl DetaPut for TroutStockingDataMetadata {
    const DETA_BASE: DetaBase = DetaBase::TroutStockingIndex;
}

#[derive(Serialize, Deserialize, Debug)]
struct TroutStockingDataMetadata {
    key: String,
    timestamp: i64,
    num_stocked_locations: u32,
}

#[derive(Serialize, Deserialize, Debug)]
enum DetaTriggerId {
    #[serde(alias = "scrape-new-data")]
    ScrapeNewData,
    #[serde(alias = "reindex-data")]
    ReindexData,
    #[serde(alias = "detabase-migration")]
    DetaBaseMigration,
}

#[derive(Serialize, Deserialize, Debug)]
struct DetaEvent {
    id: DetaTriggerId,
    trigger: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct DetaAction {
    event: DetaEvent,
}

#[tokio::main]
async fn main() {
    env_logger::init();
    info!("Starting app.");
    let app = Router::new()
        .route("/", get(root))
        .route("/demo", get(demo))
        .route("/__space/v0/actions", post(deta_action_trigger));
    info!("Build app routes.");

    let port = match env::var("PORT") {
        Ok(v) => v.parse::<u16>().unwrap(),
        Err(_) => {
            warn!("Could not get port from env var.");
            3000
        }
    };
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    info!("Listening on '{}'.", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn root() -> &'static str {
    info!("Requested root route.");
    "Trout web-scraping Micro"
}

async fn demo() -> impl IntoResponse {
    info!("Running demo data scraping.");
    let trout_data_organized = get_data().await;
    (StatusCode::OK, Json(trout_data_organized))
}

async fn deta_action_trigger(Json(payload): Json<DetaAction>) {
    info!("Recieved payload: {:?}", payload);
    match payload.event.id {
        DetaTriggerId::ScrapeNewData => collect_new_data().await,
        DetaTriggerId::ReindexData => reindex_trout_stocking_data().await,
        DetaTriggerId::DetaBaseMigration => migrate_trout_stocking_database(),
    };
}

async fn collect_new_data() {
    info!("Collecting new trout stocking data.");
    let trout_data = get_data().await;

    debug!("Creating Deta client.");
    let deta = Deta::new();

    // Collect new data and push to the three databases..
    debug!("Collecting new data.");
    trout_data.deta_put(&deta);
    debug!("Indexing trout data ({}).", trout_data.key);
    index_trout_data(&trout_data).deta_put(&deta);
    debug!("Organizing trout data.");
    trout_data.organize().deta_put(&deta);
    info!("Finished collecting new trout stocking data.");
}

async fn reindex_trout_stocking_data() {
    info!("Re-indexing trout stocking data.");

    debug!("Creating Deta client.");
    let deta = Deta::new();

    debug!("Connecting to deta base: {}", DetaBase::TroutStockingRaw);
    let trout_db = deta.base(DetaBase::TroutStockingRaw.as_str());

    let query_result = trout_db.query().limit(u16::MAX).run().unwrap();
    let trout_data: Vec<TroutStocking> =
        serde_json::from_value(query_result["items"].clone()).unwrap();

    if trout_data.len() == usize::from(u16::MAX) {
        warn!("Reached max query size, need to find a solution to getting all of the data.");
    } else {
        info!("Number of parsed results: {}", trout_data.len());
    }

    debug!("Indexing trout records and pushing data.");
    trout_data
        .iter()
        .map(index_trout_data)
        .for_each(|i| i.deta_put(&deta));

    info!("Finished re-indexing trout stocking data.");
}

fn index_trout_data(trout_data: &TroutStocking) -> TroutStockingDataMetadata {
    debug!("Indexing {}", trout_data.key);

    // Compute metadata values.
    let num_stocked_locations: u32 = trout_data.organize().data["Species"]
        .iter()
        .map(|x| x.is_some() as u32)
        .sum();

    // Build metadata object.
    let metadata = TroutStockingDataMetadata {
        key: trout_data.key.clone(),
        timestamp: trout_data.timestamp.timestamp,
        num_stocked_locations,
    };
    debug!("Metadata: {:?}", metadata);
    metadata
}

fn migrate_trout_stocking_database() {
    info!("Migrating trout stocking database.");

    debug!("Creating Deta client.");
    let deta = Deta::new();

    debug!("Connecting to Deta Base '{}'.", DetaBase::TroutStockingRaw);
    let trout_db = deta.base(DetaBase::TroutStockingRaw.as_str());

    debug!("Querying all trout stocking data.");
    let query_result = trout_db.query().limit(u16::MAX).run().unwrap();
    let data: Vec<Value> = serde_json::from_value(query_result["items"].clone()).unwrap();
    debug!("Number of entries: {}", data.len());

    for mut entry in data {
        let changed_data = execute_data_migration_steps(&mut entry);
        if changed_data {
            info!("Changes to data. Deserializing and pushing new entry.");
            let trout_data: TroutStocking = serde_json::from_value(entry).unwrap();
            trout_data.deta_put(&deta);
            trout_data.organize().deta_put(&deta);
            index_trout_data(&trout_data).deta_put(&deta);
        } else {
            debug!("No changes to data.")
        }
    }
    info!("Finished database migration.");
}

/// Execute data migration steps.
///
/// # Arguments
///
/// * `entry` - Trout stocking data entry to be updated.
///
/// # Returns
///
/// - `bool`: Whether the data has changed or not.
///
/// Currently, there is only a single migration step. If others are needed in the
/// future, add another function with the same signature. The returned boolean is to
/// indicate if the data has changed.
fn execute_data_migration_steps(entry: &mut Value) -> bool {
    add_timestamp_field_to_timestamp_data(entry)
}

fn add_timestamp_field_to_timestamp_data(data: &mut Value) -> bool {
    debug!("Adding `timestamp` field to `timestamp` data.");

    debug!("{}", data["timestamp"]);
    if data["timestamp"].get("timestamp").is_some() {
        debug!("Already has timestamp.timestamp field.");
        return false;
    }

    let ts = data["timestamp"]["datetime"]
        .as_str()
        .unwrap()
        .parse::<DateTime<Utc>>()
        .unwrap();
    debug!("timestamp: {}", ts);

    let new_ts_data: HashMap<String, Value> =
        HashMap::from([("timestamp".to_string(), Value::from(ts.timestamp()))]);

    data["timestamp"] = merge(&data["timestamp"], &new_ts_data);
    debug!("new timestamp data: {}", data["timestamp"]);
    true
}

pub fn merge(v: &Value, fields: &HashMap<String, Value>) -> Value {
    match v {
        Value::Object(m) => {
            let mut m = m.clone();
            for (k, v) in fields {
                m.insert(k.clone(), v.clone());
            }
            Value::Object(m)
        }
        v => v.clone(),
    }
}

async fn get_data() -> TroutStocking {
    let spreadsheet_url = match get_spreadsheet_url().await {
        Ok(url) => url,
        Err(e) => panic!("Could not find spreadsheet URL: {}", e),
    };
    let spreadsheet_data = match get_trout_stocking_page(&spreadsheet_url).await {
        Ok(data) => data,
        Err(e) => panic!("Could not aquire spreadsheet data: {}", e),
    };
    let trout_data = match parse_trout_stocking_spreadsheet_data(&spreadsheet_data) {
        Ok(d) => d,
        Err(e) => panic!("Failed to parse data: {}", e),
    };
    info!("Collected trout data at {}", trout_data.timestamp.datetime);
    trout_data
}
