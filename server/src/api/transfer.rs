use axum::Json;
use axum::body::Bytes;
use axum::extract::State;
use axum::http::header;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use time::OffsetDateTime;
use time::macros::format_description;
use utoipa::ToSchema;

use crate::AppState;
use crate::db::devices::DeviceKind;
use crate::error::{ApiError, ApiResult, ErrorBody};
use crate::transfer::parse::{FileIssue, parse};
use crate::transfer::plan::{Plan, Ref, RowAction, RowReport, plan};
use crate::transfer::snapshot::{self, Snapshot};
use crate::transfer::{apply, export, template};

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
    /// Set when the file cannot be read at all; everything else is then empty.
    pub file_issue: Option<FileIssue>,
    pub summary: ImportSummary,
    pub new_platforms: Vec<String>,
    pub new_devices: Vec<NewDeviceReport>,
    pub new_stores: Vec<String>,
    pub rows: Vec<RowReport>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ImportResult {
    pub summary: ImportSummary,
}

fn summarize(plan: &Plan) -> ImportSummary {
    let count = |action| plan.rows.iter().filter(|row| row.action == action).count();
    ImportSummary {
        rows: plan.rows.len(),
        new_games: plan.games.len(),
        updated_games: plan.game_statuses.len() + plan.game_igdb_links.len(),
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
        rows: plan.rows,
    }
}

fn file_problem(issue: FileIssue) -> ImportPreview {
    ImportPreview {
        file_issue: Some(issue),
        summary: ImportSummary::default(),
        new_platforms: Vec::new(),
        new_devices: Vec::new(),
        new_stores: Vec::new(),
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

#[utoipa::path(
    get, path = "/export.csv", tag = "transfer", operation_id = "exportCsv",
    responses((status = OK, description = "One row per copy, plus games without copies", content_type = "text/csv", body = String))
)]
pub async fn export_csv(State(state): State<AppState>) -> ApiResult<Response> {
    let body = export::export(&state.db).await.map_err(|err| match err {
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
pub async fn preview(State(state): State<AppState>, body: Bytes) -> ApiResult<Json<ImportPreview>> {
    let rows = match parse(&body) {
        Ok(rows) => rows,
        Err(issue) => return Ok(Json(file_problem(issue))),
    };
    let mut conn = state.db.acquire().await?;
    let snapshot = snapshot::load(&mut conn).await?;
    let plan = plan(&rows, &snapshot);
    Ok(Json(preview_of(plan, &snapshot)))
}

#[utoipa::path(
    post, path = "/import", tag = "transfer", operation_id = "runImport",
    request_body(content = String, content_type = "text/csv"),
    responses(
        (status = OK, body = ImportResult),
        (status = UNPROCESSABLE_ENTITY, description = "`import_invalid`: preview the file to see why", body = ErrorBody),
    )
)]
pub async fn run(State(state): State<AppState>, body: Bytes) -> ApiResult<Json<ImportResult>> {
    let rows = parse(&body).map_err(|_| ApiError::Invalid {
        code: "import_invalid",
        field: None,
    })?;

    let mut tx = crate::db::write_tx(&state.db).await?;
    let snapshot = snapshot::load(&mut tx).await?;
    let plan = plan(&rows, &snapshot);
    if plan.has_errors() {
        return Err(ApiError::Invalid {
            code: "import_invalid",
            field: None,
        });
    }
    apply::apply(&mut tx, &plan).await?;
    tx.commit().await?;

    let summary = summarize(&plan);
    tracing::info!(
        rows = summary.rows,
        new_games = summary.new_games,
        new_copies = summary.new_copies,
        updated_copies = summary.updated_copies,
        "import applied"
    );
    Ok(Json(ImportResult { summary }))
}
