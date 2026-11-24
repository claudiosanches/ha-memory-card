use axum::extract::State;
use axum::{Extension, Json};
use serde::Serialize;
use utoipa::ToSchema;

use crate::AppState;
use crate::access::{Access, Role};
use crate::barcodes::{self, Product};
use crate::db::games::{self, BarcodeCopy};
use crate::domain;
use crate::error::{ApiError, ApiPath, ApiResult, ErrorBody};

#[derive(Debug, Serialize, ToSchema)]
pub struct BarcodeLookup {
    /// The code as digits.
    pub barcode: String,
    /// Copies in the collection with this barcode.
    pub copies: Vec<BarcodeCopy>,
    pub product: Option<Product>,
    pub lookup: LookupOutcome,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum LookupOutcome {
    /// A copy has the code, so the database was not asked.
    InCollection,
    Found,
    NotFound,
    /// No copy has the code. Only managers, who add copies, look codes up,
    /// which keeps viewers from spending the database's daily limit.
    NotInCollection,
    /// `external_services` is off.
    Offline,
    /// The database could not be reached or refused, for example over its daily limit.
    Failed,
}

#[utoipa::path(
    get, path = "/barcodes/{code}", tag = "barcodes", operation_id = "lookUpBarcode",
    params(("code" = String, Path, description = "UPC or EAN")),
    responses(
        (status = OK, body = BarcodeLookup),
        (status = UNPROCESSABLE_ENTITY, description = "`invalid_barcode`", body = ErrorBody),
    )
)]
pub async fn look_up(
    State(state): State<AppState>,
    Extension(access): Extension<Access>,
    ApiPath(code): ApiPath<String>,
) -> ApiResult<Json<BarcodeLookup>> {
    let barcode = domain::barcode(&code).ok_or(ApiError::invalid("invalid_barcode", "barcode"))?;
    let copies = games::with_barcode(&state.db, &barcode).await?;

    let (product, lookup) = if !copies.is_empty() {
        (None, LookupOutcome::InCollection)
    } else if access.role != Role::Manager {
        (None, LookupOutcome::NotInCollection)
    } else if !state.config.external_services {
        (None, LookupOutcome::Offline)
    } else {
        match barcodes::lookup(&state.http, &state.config.barcode_url, &barcode).await {
            Ok(Some(product)) => (Some(product), LookupOutcome::Found),
            Ok(None) => (None, LookupOutcome::NotFound),
            Err(err) => {
                tracing::warn!(error = %err, "barcode lookup failed");
                (None, LookupOutcome::Failed)
            }
        }
    };
    Ok(Json(BarcodeLookup {
        barcode,
        copies,
        product,
        lookup,
    }))
}
