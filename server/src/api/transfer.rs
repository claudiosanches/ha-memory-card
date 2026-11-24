use std::collections::HashSet;

use axum::Json;
use axum::extract::rejection::QueryRejection;
use axum::extract::{Query, State};
use axum::http::header;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use time::OffsetDateTime;
use time::macros::format_description;
use utoipa::ToSchema;

use super::body::CsvBody;
use crate::AppState;
use crate::access::Actor;
use crate::db::devices::DeviceKind;
use crate::db::library::{LibraryFilter, Ownership};
use crate::db::transfer::{self, snapshot::Snapshot};
use crate::error::{ApiError, ApiResult, ErrorBody};
use crate::transfer::parse::{FileIssue, HeaderMatch, headers, parse};
use crate::transfer::plan::{Plan, Ref, RowAction, RowReport, plan};
use crate::transfer::{export, template};

#[derive(Debug, Default, Serialize, ToSchema)]
pub struct ImportSummary {
    pub rows: usize,
    pub new_games: usize,
    pub updated_games: usize,
    pub new_releases: usize,
    pub updated_releases: usize,
    pub new_copies: usize,
    pub updated_copies: usize,
    pub unchanged_copies: usize,
    pub game_only_rows: usize,
    pub error_rows: usize,
    pub warning_rows: usize,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct NewDeviceReport {
    pub name: String,
    pub kind: DeviceKind,
    pub platforms: Vec<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ImportPreview {
    /// Set when the file cannot be read at all; everything else but the headers is then empty.
    pub file_issue: Option<FileIssue>,
    /// How each header of the file was read, in file order.
    pub headers: Vec<HeaderMatch>,
    /// The file's separator, `,` or `;`.
    pub delimiter: String,
    pub summary: ImportSummary,
    pub new_platforms: Vec<String>,
    pub new_devices: Vec<NewDeviceReport>,
    pub new_stores: Vec<String>,
    pub new_labels: Vec<String>,
    pub rows: Vec<RowReport>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ImportResult {
    pub summary: ImportSummary,
    /// Games in the whole collection, after the import, with no IGDB link.
    pub unlinked_games: i64,
}

fn summarize(plan: &Plan) -> ImportSummary {
    let count = |action| plan.rows.iter().filter(|row| row.action == action).count();
    ImportSummary {
        rows: plan.rows.len(),
        new_games: plan.games.len(),
        updated_games: plan
            .game_statuses
            .iter()
            .map(|(id, _)| *id)
            .chain(plan.game_igdb_links.iter().map(|(id, _)| *id))
            .chain(plan.game_labels.iter().filter_map(|(game, _)| match game {
                Ref::Existing(id) => Some(*id),
                Ref::New(_) => None,
            }))
            .collect::<HashSet<_>>()
            .len(),
        new_releases: plan.releases.len(),
        updated_releases: plan.release_titles.len(),
        new_copies: count(RowAction::NewCopy),
        updated_copies: count(RowAction::UpdateCopy),
        unchanged_copies: count(RowAction::Unchanged),
        game_only_rows: count(RowAction::GameOnly),
        error_rows: count(RowAction::Error),
        warning_rows: plan
            .rows
            .iter()
            .filter(|row| row.issues.iter().any(|issue| !issue.error))
            .count(),
    }
}

fn preview_of(plan: Plan, snapshot: &Snapshot) -> ImportPreview {
    let platform_name = |r: Ref| match r {
        Ref::Existing(id) => snapshot
            .platforms
            .iter()
            .find(|p| p.id == id)
            .map(|p| p.name.clone())
            .unwrap_or_default(),
        Ref::New(i) => plan.platforms[i].clone(),
    };
    ImportPreview {
        file_issue: None,
        headers: Vec::new(),
        delimiter: ",".to_owned(),
        summary: summarize(&plan),
        new_devices: plan
            .devices
            .iter()
            .map(|d| NewDeviceReport {
                name: d.name.clone(),
                kind: d.kind,
                platforms: d.platforms.iter().map(|p| platform_name(*p)).collect(),
            })
            .collect(),
        new_platforms: plan.platforms.clone(),
        new_stores: plan.stores.clone(),
        new_labels: plan.labels.clone(),
        rows: plan.rows,
    }
}

fn file_problem(issue: FileIssue) -> ImportPreview {
    ImportPreview {
        file_issue: Some(issue),
        headers: Vec::new(),
        delimiter: ",".to_owned(),
        summary: ImportSummary::default(),
        new_platforms: Vec::new(),
        new_devices: Vec::new(),
        new_stores: Vec::new(),
        new_labels: Vec::new(),
        rows: Vec::new(),
    }
}

fn csv_download(name: &str, body: Vec<u8>) -> Response {
    let date = OffsetDateTime::now_utc()
        .format(format_description!("[year][month][day]"))
        .expect("static format description");
    (
        [
            (header::CONTENT_TYPE, "text/csv; charset=utf-8".to_owned()),
            (
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"memory-card-{name}-{date}.csv\""),
            ),
            (header::CACHE_CONTROL, "no-store".to_owned()),
        ],
        body,
    )
        .into_response()
}

/// Takes the library filters, except that ownership defaults to `any`, so
/// the plain URL exports the whole collection.
#[utoipa::path(
    get, path = "/export.csv", tag = "transfer", operation_id = "exportCsv",
    params(LibraryFilter),
    responses(
        (status = OK, description = "One row per copy; without copy filters, also one per game without copies", content_type = "text/csv", body = String),
        (status = UNPROCESSABLE_ENTITY, description = "`invalid_filter`", body = ErrorBody),
    )
)]
pub async fn export_csv(
    State(state): State<AppState>,
    Actor(by): Actor,
    filter: Result<Query<LibraryFilter>, QueryRejection>,
) -> ApiResult<Response> {
    let Query(mut filter) = filter.map_err(|_| ApiError::Invalid {
        code: "invalid_filter",
        field: None,
    })?;
    filter.ownership.get_or_insert(Ownership::Any);
    filter.person = Some(by);
    let body = export::export(&state.db, &filter)
        .await
        .map_err(|err| match err {
            export::ExportError::Database(err) => ApiError::Database(err),
            other => ApiError::Internal(other.to_string()),
        })?;
    Ok(csv_download("export", body))
}

#[utoipa::path(
    get, path = "/import/template.csv", tag = "transfer", operation_id = "importTemplate",
    responses((status = OK, description = "Header row only", content_type = "text/csv", body = String))
)]
pub async fn template_csv() -> Response {
    csv_download("template", template())
}

#[utoipa::path(
    post, path = "/import/preview", tag = "transfer", operation_id = "previewImport",
    request_body(content = String, content_type = "text/csv"),
    responses((status = OK, body = ImportPreview))
)]
pub async fn preview(
    State(state): State<AppState>,
    Actor(by): Actor,
    CsvBody(body): CsvBody,
) -> ApiResult<Json<ImportPreview>> {
    let mut preview = match parse(&body) {
        Ok(rows) => {
            let snapshot = transfer::snapshot(&state.db, &by).await?;
            let plan = plan(&rows, &snapshot);
            preview_of(plan, &snapshot)
        }
        Err(issue) => file_problem(issue),
    };
    if let Some((headers, delimiter)) = headers(&body) {
        preview.headers = headers;
        preview.delimiter = delimiter.to_string();
    }
    Ok(Json(preview))
}

#[utoipa::path(
    post, path = "/import", tag = "transfer", operation_id = "runImport",
    request_body(content = String, content_type = "text/csv"),
    responses(
        (status = OK, body = ImportResult),
        (status = UNPROCESSABLE_ENTITY, description = "`import_invalid`: preview the file to see why", body = ErrorBody),
    )
)]
pub async fn run(
    State(state): State<AppState>,
    Actor(by): Actor,
    CsvBody(body): CsvBody,
) -> ApiResult<Json<ImportResult>> {
    let rows = parse(&body).map_err(|_| ApiError::Invalid {
        code: "import_invalid",
        field: None,
    })?;

    let plan = transfer::import(&state.db, &by, |snapshot| plan(&rows, snapshot)).await?;

    let summary = summarize(&plan);
    tracing::info!(
        rows = summary.rows,
        new_games = summary.new_games,
        new_copies = summary.new_copies,
        updated_copies = summary.updated_copies,
        "import applied"
    );
    let unlinked_games = crate::db::igdb::unlinked_count(&state.db).await?;
    Ok(Json(ImportResult {
        summary,
        unlinked_games,
    }))
}
