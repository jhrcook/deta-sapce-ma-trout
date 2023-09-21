use std::{net::SocketAddr, str::FromStr};

use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use detalib::Deta;
use serde::{Deserialize, Serialize};
use std::env;
use trout_scraping::models::TroutStocking;
use trout_scraping::scraping::{
    get_spreadsheet_url, get_trout_stocking_page, parse_trout_stocking_spreadsheet_data,
};

#[macro_use]
extern crate log;

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

#[derive(Serialize, Deserialize, Debug)]
struct TroutStockingDataMetadata {
    key: String,
    timestamp: i64,
    num_stocked_locations: u32,
}

#[derive(Serialize, Deserialize, Debug)]
struct DetaEvent {
    id: String, // TODO: refactor this into an Enum
    trigger: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct DetaAction {
    event: DetaEvent,
}

async fn deta_action_trigger(Json(payload): Json<DetaAction>) {
    info!("Recieved payload: {:?}", payload);
    if payload.event.id == "scrape-new-data" {
        collect_new_data().await;
    } else if payload.event.id == "reindex-data" {
        reindex_trout_stocking_data().await;
    } else {
        error!("Unrecognized payload event ID: {}", payload.event.id);
    }
}

async fn collect_new_data() {
    info!("Collecting new trout stocking data.");
    let trout_data = get_data().await;

    info!("Creating Deta client.");
    let deta = Deta::new();

    // Push raw data.
    let db_name = "trout-stocking";
    info!("Connecting to Deta Base '{db_name}'.");
    let base = deta.base(db_name);
    let result = base.insert(&trout_data);
    let entry_key = match result {
        Ok(v) => {
            let key = String::from_str(v["key"].as_str().unwrap()).unwrap();
            info!("Successfully pushed  data: {}.", key);
            key
        }
        Err(e) => {
            error!("Error pushing data: {}", e);
            panic!("Could not push data.")
        }
    };
    info!("data key: {}", entry_key);
    index_trout_data(&deta, trout_data, entry_key);
}

async fn reindex_trout_stocking_data() {
    // TODO: update the "trout-stocking-index" table.
    info!("Re-indexing trout stocking data.");
    warn!("Not implemented yet.");
}

fn index_trout_data(deta: &Deta, trout_data: TroutStocking, key: String) {
    // Compute metadata values.
    let timestamp = trout_data.timestamp.timestamp;
    let num_stocked_locations: u32 = trout_data.organize().data["Species"]
        .iter()
        .map(|x| x.is_some() as u32)
        .sum();
    let metadata = TroutStockingDataMetadata {
        key,
        timestamp,
        num_stocked_locations,
    };
    info!("metadata: {:?}", metadata);

    // Push metadata to index database.
    let db_name = "trout-stocking-index";
    info!("Connecting to Deta Base '{db_name}'.");
    let base = deta.base(db_name);
    let result = base.insert(metadata);
    match result {
        Ok(v) => info!("Successfully pushed metadata: {}.", v),
        Err(e) => {
            error!("Error pushing metadata: {}", e);
            panic!("Could not push metadata.")
        }
    };
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
