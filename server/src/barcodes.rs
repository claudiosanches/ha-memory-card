//! Names the product behind a UPC or EAN through UPCitemdb.

use std::time::Duration;

use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::api::input::{NAME_MAX, TITLE_MAX, cut};

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct Product {
    /// As the database lists it, often with the platform, such as
    /// "Super Mario Odyssey - Nintendo Switch".
    pub title: String,
    pub brand: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum LookupError {
    #[error(transparent)]
    Http(#[from] reqwest::Error),
    #[error("the barcode database answered {0}")]
    Status(StatusCode),
    #[error("the barcode database's answer is larger than expected")]
    TooLarge,
    #[error("the barcode database's answer is not what was asked for: {0}")]
    Json(#[from] serde_json::Error),
}

/// An answer lists a handful of products.
const ANSWER_MAX: usize = 1024 * 1024;

#[derive(Deserialize)]
struct Answer {
    #[serde(default)]
    items: Vec<Item>,
}

#[derive(Deserialize)]
struct Item {
    #[serde(default)]
    title: String,
    brand: Option<String>,
}

/// `None` when the database does not know the code.
pub async fn lookup(
    http: &reqwest::Client,
    url: &str,
    code: &str,
) -> Result<Option<Product>, LookupError> {
    // `code` is digits only, so it needs no escaping.
    let response = http
        .get(format!("{url}?upc={code}"))
        .timeout(Duration::from_secs(10))
        .send()
        .await?;
    match response.status() {
        // UPCitemdb answers both for codes it has never seen.
        StatusCode::NOT_FOUND | StatusCode::BAD_REQUEST => return Ok(None),
        status if !status.is_success() => return Err(LookupError::Status(status)),
        _ => {}
    }
    let bytes = crate::http::body(response, ANSWER_MAX)
        .await?
        .ok_or(LookupError::TooLarge)?;
    let answer: Answer = serde_json::from_slice(&bytes)?;
    Ok(answer
        .items
        .into_iter()
        .find(|item| !item.title.trim().is_empty())
        .map(|item| Product {
            title: cut::<TITLE_MAX>(item.title.trim().to_owned()),
            brand: item
                .brand
                .map(|b| cut::<NAME_MAX>(b.trim().to_owned()))
                .filter(|b| !b.is_empty()),
        }))
}
