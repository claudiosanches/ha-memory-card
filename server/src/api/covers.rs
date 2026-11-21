//! Uploaded covers. An upload wins over IGDB's cover until it is removed.

use axum::Json;
use axum::body::Bytes;
use axum::extract::State;
use time::OffsetDateTime;
use utoipa::ToSchema;

use crate::AppState;
use crate::covers::{self, Owner};
use crate::db::games::{self, GameDetail};
use crate::error::{ApiError, ApiPath, ApiResult, ErrorBody};
use crate::igdb::store;

const MAX_BYTES: usize = 10 * 1024 * 1024;

/// A JPEG, PNG or WebP image, at most 10 MB.
#[derive(ToSchema)]
#[schema(value_type = String, format = Binary)]
pub struct Image(#[allow(dead_code)] Vec<u8>);

#[utoipa::path(
    put, path = "/games/{id}/cover", tag = "covers", operation_id = "uploadGameCover",
    params(("id" = i64, Path)),
    request_body(content = Image, content_type = "application/octet-stream"),
    responses(
        (status = OK, body = GameDetail),
        (status = NOT_FOUND, body = ErrorBody),
        (status = UNPROCESSABLE_ENTITY, description = "`invalid_image` or `image_too_large`", body = ErrorBody),
    )
)]
pub async fn upload_game(
    State(state): State<AppState>,
    ApiPath(id): ApiPath<i64>,
    body: Bytes,
) -> ApiResult<Json<GameDetail>> {
    upload(&state, Owner::Game(id), &body).await
}

#[utoipa::path(
    delete, path = "/games/{id}/cover", tag = "covers", operation_id = "removeGameCover",
    params(("id" = i64, Path)),
    responses(
        (status = OK, description = "The upload is gone; a linked game gets its IGDB cover back", body = GameDetail),
        (status = NOT_FOUND, body = ErrorBody),
    )
)]
pub async fn remove_game(
    State(state): State<AppState>,
    ApiPath(id): ApiPath<i64>,
) -> ApiResult<Json<GameDetail>> {
    remove(&state, Owner::Game(id)).await
}

#[utoipa::path(
    put, path = "/releases/{id}/cover", tag = "covers", operation_id = "uploadReleaseCover",
    params(("id" = i64, Path)),
    request_body(content = Image, content_type = "application/octet-stream"),
    responses(
        (status = OK, description = "The release's game", body = GameDetail),
        (status = NOT_FOUND, body = ErrorBody),
        (status = UNPROCESSABLE_ENTITY, description = "`invalid_image` or `image_too_large`", body = ErrorBody),
    )
)]
pub async fn upload_release(
    State(state): State<AppState>,
    ApiPath(id): ApiPath<i64>,
    body: Bytes,
) -> ApiResult<Json<GameDetail>> {
    upload(&state, Owner::Release(id), &body).await
}

#[utoipa::path(
    delete, path = "/releases/{id}/cover", tag = "covers", operation_id = "removeReleaseCover",
    params(("id" = i64, Path)),
    responses(
        (status = OK, description = "The release's game; a linked game gets its IGDB cover back", body = GameDetail),
        (status = NOT_FOUND, body = ErrorBody),
    )
)]
pub async fn remove_release(
    State(state): State<AppState>,
    ApiPath(id): ApiPath<i64>,
) -> ApiResult<Json<GameDetail>> {
    remove(&state, Owner::Release(id)).await
}

async fn upload(state: &AppState, owner: Owner, bytes: &[u8]) -> ApiResult<Json<GameDetail>> {
    if bytes.len() > MAX_BYTES {
        return Err(ApiError::Invalid {
            code: "image_too_large",
            field: None,
        });
    }
    let extension = image_extension(bytes).ok_or(ApiError::Invalid {
        code: "invalid_image",
        field: None,
    })?;
    let game_id = games::cover_game(&state.db, owner).await?;

    let dir = state.config.covers_dir();
    // A new name per upload, so browsers never show a cached older one.
    let stamp = OffsetDateTime::now_utc().unix_timestamp_nanos() / 1_000_000;
    let path = covers::write(&dir, owner, &format!("upload-{stamp}.{extension}"), bytes)
        .await
        .map_err(|err| ApiError::Internal(format!("saving cover: {err}")))?;
    if !games::set_uploaded_cover(&state.db, owner, Some(&path)).await? {
        covers::keep_only(&dir, owner, None).await;
        return Err(ApiError::NotFound);
    }
    covers::keep_only(&dir, owner, Some(&path)).await;
    Ok(Json(games::get(&state.db, game_id).await?))
}

async fn remove(state: &AppState, owner: Owner) -> ApiResult<Json<GameDetail>> {
    let game_id = games::cover_game(&state.db, owner).await?;
    if games::set_uploaded_cover(&state.db, owner, None).await? {
        covers::keep_only(&state.config.covers_dir(), owner, None).await;
        restore_igdb_covers(state, game_id).await;
    }
    Ok(Json(games::get(&state.db, game_id).await?))
}

/// Best effort: without IGDB the cover stays empty until the next refresh.
async fn restore_igdb_covers(state: &AppState, game_id: i64) {
    let Some(igdb) = state.igdb.clone() else {
        return;
    };
    let igdb_id = match sqlx::query_scalar!("SELECT igdb_id FROM games WHERE id = ?", game_id)
        .fetch_one(&state.db)
        .await
    {
        Ok(Some(id)) => id,
        Ok(None) => return,
        Err(err) => {
            tracing::warn!(game_id, error = %err, "could not read the IGDB id");
            return;
        }
    };
    let result = match igdb.games(&[igdb_id]).await {
        Ok(found) => match found.first() {
            Some(data) => {
                store::refresh_covers(&state.db, &state.config.covers_dir(), &igdb, game_id, data)
                    .await
            }
            None => return,
        },
        Err(err) => Err(err.into()),
    };
    if let Err(err) = result {
        tracing::warn!(game_id, error = %err, "could not restore the IGDB cover");
    }
}

pub(crate) fn image_extension(bytes: &[u8]) -> Option<&'static str> {
    match bytes {
        [0xFF, 0xD8, 0xFF, ..] => Some("jpg"),
        [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, ..] => Some("png"),
        [
            b'R',
            b'I',
            b'F',
            b'F',
            _,
            _,
            _,
            _,
            b'W',
            b'E',
            b'B',
            b'P',
            ..,
        ] => Some("webp"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::image_extension;

    #[test]
    fn recognizes_images_by_their_bytes() {
        assert_eq!(image_extension(b"\xFF\xD8\xFF\xE0rest"), Some("jpg"));
        assert_eq!(image_extension(b"\x89PNG\r\n\x1a\nrest"), Some("png"));
        assert_eq!(image_extension(b"RIFF\0\0\0\0WEBPVP8 "), Some("webp"));
        assert_eq!(image_extension(b"<svg></svg>"), None);
        assert_eq!(image_extension(b""), None);
    }
}
