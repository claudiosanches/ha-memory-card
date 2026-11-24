//! Photos of copies, stored next to the covers.

use std::sync::atomic::{AtomicU64, Ordering};

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use time::OffsetDateTime;

use super::body::ImageBody;
use super::covers::Image;
use crate::AppState;
use crate::access::Actor;
use crate::covers;
use crate::db::games::{self, GameDetail};
use crate::db::photos;
use crate::error::{ApiError, ApiPath, ApiResult, ErrorBody};
use crate::images;

const MAX_BYTES: usize = 10 * 1024 * 1024;

/// Keeps file names apart when uploads land in the same millisecond.
static UPLOADS: AtomicU64 = AtomicU64::new(0);

#[utoipa::path(
    post, path = "/copies/{id}/photos", tag = "photos", operation_id = "addCopyPhoto",
    params(("id" = i64, Path)),
    request_body(content = Image, content_type = "application/octet-stream"),
    responses(
        (status = CREATED, description = "The copy's game", body = GameDetail),
        (status = NOT_FOUND, body = ErrorBody),
        (status = CONFLICT, description = "`too_many_photos`: 20 per copy", body = ErrorBody),
        (status = UNPROCESSABLE_ENTITY, description = "`invalid_image` or `image_too_large`", body = ErrorBody),
    )
)]
pub async fn add(
    State(state): State<AppState>,
    Actor(by): Actor,
    ApiPath(id): ApiPath<i64>,
    ImageBody(body): ImageBody,
) -> ApiResult<(StatusCode, Json<GameDetail>)> {
    if body.len() > MAX_BYTES {
        return Err(ApiError::Invalid {
            code: "image_too_large",
            field: None,
        });
    }
    let image = images::clean(&body).map_err(|rejected| ApiError::Invalid {
        code: match rejected {
            images::Rejected::Invalid => "invalid_image",
            images::Rejected::TooLarge => "image_too_large",
        },
        field: None,
    })?;
    // Checked again when the row is added; this spares writing a file.
    photos::room_for(&state.db, id).await?;

    let dir = state.config.covers_dir();
    let stamp = OffsetDateTime::now_utc().unix_timestamp_nanos() / 1_000_000;
    let n = UPLOADS.fetch_add(1, Ordering::Relaxed);
    let name = format!("photo-{stamp}-{n}.{}", image.extension);
    let path = covers::write_in(&dir, &covers::copy_dir(id), &name, &image.bytes)
        .await
        .map_err(|err| ApiError::Internal(format!("saving photo: {err}")))?;
    let game_id = match photos::add(&state.db, id, &path, &by).await {
        Ok(game_id) => game_id,
        Err(err) => {
            covers::remove_file(&dir, &path).await;
            return Err(err);
        }
    };
    Ok((
        StatusCode::CREATED,
        Json(games::get(&state.db, game_id, &by).await?),
    ))
}

#[utoipa::path(
    delete, path = "/photos/{id}", tag = "photos", operation_id = "deleteCopyPhoto",
    params(("id" = i64, Path)),
    responses(
        (status = OK, description = "The photo's game", body = GameDetail),
        (status = NOT_FOUND, body = ErrorBody),
    )
)]
pub async fn delete(
    State(state): State<AppState>,
    Actor(by): Actor,
    ApiPath(id): ApiPath<i64>,
) -> ApiResult<Json<GameDetail>> {
    let (game_id, path) = photos::delete(&state.db, id).await?;
    covers::remove_file(&state.config.covers_dir(), &path).await;
    Ok(Json(games::get(&state.db, game_id, &by).await?))
}

/// Deletes photo directories of copies that are gone. Best effort.
pub async fn prune(state: &AppState) {
    let dir = state.config.covers_dir();
    let ids = covers::copy_dirs(&dir).await;
    if ids.is_empty() {
        return;
    }
    match photos::gone(&state.db, &ids).await {
        Ok(gone) => {
            for id in gone {
                let _ = tokio::fs::remove_dir_all(dir.join(covers::copy_dir(id))).await;
            }
        }
        Err(err) => tracing::warn!(error = %err, "cannot check photo directories"),
    }
}
