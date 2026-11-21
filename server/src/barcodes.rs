//! Names the product behind a UPC or EAN through UPCitemdb.

use std::time::Duration;

use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

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
}

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
pub async fn lookup(url: &str, code: &str) -> Result<Option<Product>, LookupError> {
    // `code` is digits only, so it needs no escaping.
    let response = crate::http::client(Duration::from_secs(10))
        .get(format!("{url}?upc={code}"))
        .send()
        .await?;
    match response.status() {
        // UPCitemdb answers both for codes it has never seen.
        StatusCode::NOT_FOUND | StatusCode::BAD_REQUEST => return Ok(None),
        status if !status.is_success() => return Err(LookupError::Status(status)),
        _ => {}
    }
    let answer: Answer = response.json().await?;
    Ok(answer
        .items
        .into_iter()
        .find(|item| !item.title.trim().is_empty())
        .map(|item| Product {
            title: item.title.trim().to_owned(),
            brand: item
                .brand
                .map(|b| b.trim().to_owned())
                .filter(|b| !b.is_empty()),
        }))
}
