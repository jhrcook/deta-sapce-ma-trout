use anyhow::{Context, Result};

use crate::models::TroutStocking;

/// Collects the URL for the trout stocking spreadsheet.
pub async fn get_spreadsheet_url() -> Result<String> {
    let result = reqwest::get(
        "https://raw.githubusercontent.com/massgov/FWE/master/DFW/Trout/troutstocking-table.html",
    )
    .await?;

    match result.status() {
        reqwest::StatusCode::OK => {
            info!("Successfully downloaded trout stocking table HTML.");
        }
        _ => {
            panic!("Unable to get trout stocking table HTML.");
        }
    }

    let text = result.text().await?;
    let url = text
        .split("google.visualization.Query('")
        .nth(1)
        .context("Could not find first expected pattern.")?
        .split('\'')
        .next()
        .context("Did not find spreadsheet URL.")?
        .to_string();
    Ok(url)
}

/// Downloads the trout stocking data from the spreadsheet.
pub async fn get_trout_stocking_page(spreadsheet_url: &str) -> Result<String> {
    let res = reqwest::get(spreadsheet_url).await?;

    match res.status() {
        reqwest::StatusCode::OK => info!("Successfully downloaded spreadsheet data."),
        _ => panic!("Unable to get trout stocking table HTML."),
    };
    Ok(res.text().await?.to_string())
}

/// Parse the trout stocking data from the raw content of the spreadsheet request.
pub fn parse_trout_stocking_spreadsheet_data(raw_data: &str) -> Result<TroutStocking> {
    let parsed_data = raw_data
        .split("google.visualization.Query.setResponse(")
        .nth(1)
        .context("Could not clean raw spreadsheet data.")?
        .strip_suffix(");")
        .context("Could not clean end of raw spreadsheet data")?;
    Ok(serde_json::from_str::<TroutStocking>(parsed_data)?)
}
