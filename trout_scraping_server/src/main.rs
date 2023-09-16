use std::net::SocketAddr;

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
        .route("/__space/v0/actions", post(collect_new_data));
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
    let trout_data_organized = get_data().await.organize();
    (StatusCode::OK, Json(trout_data_organized))
}

#[derive(Serialize, Deserialize, Debug)]
struct DetaEvent {
    id: String,
    trigger: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct DetaAction {
    event: DetaEvent,
}

async fn collect_new_data(Json(payload): Json<DetaAction>) {
    info!("Recieved payload: {:?}", payload);
    let trout_data = get_data().await;

    info!("Creating Deta client.");
    let deta = Deta::new();

    // Push raw data.
    info!("Connecting to Deta Base 'trout-stocking-raw'.");
    let base = deta.base("trout-stocking-raw");
    let result = base.insert(&trout_data);
    match result {
        Ok(_) => info!("Successfully pushed raw data."),
        Err(e) => error!("Error pushing raw data: {}", e),
    }

    // Organize data and push if successful.
    let trout_data_df = trout_data.organize();
    info!("Connecting to Deta Base 'trout-stocking'.");
    let base = deta.base("trout-stocking");
    let result = base.insert(trout_data_df);
    match result {
        Ok(_) => info!("Successfully pushed organized data."),
        Err(e) => error!("Error pushing organized data: {}", e),
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
