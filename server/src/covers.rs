//! Cover files: `game-<id>/` and `release-<id>/` under the covers directory.
//! Paths stored in the database are relative to it.

use std::io;
use std::path::Path;

#[derive(Debug, Clone, Copy)]
pub enum Owner {
    Game(i64),
    Release(i64),
}

impl Owner {
    fn dir(self) -> String {
        match self {
            Self::Game(id) => format!("game-{id}"),
            Self::Release(id) => format!("release-{id}"),
        }
    }

    /// The relative path of `file_name` for this owner.
    pub fn path(self, file_name: &str) -> String {
        format!("{}/{file_name}", self.dir())
    }
}

/// Writes through a temporary file so a reader never sees half an image.
pub async fn write(
    covers_dir: &Path,
    owner: Owner,
    file_name: &str,
    bytes: &[u8],
) -> io::Result<String> {
    let dir = covers_dir.join(owner.dir());
    tokio::fs::create_dir_all(&dir).await?;
    let partial = dir.join(format!("{file_name}.part"));
    tokio::fs::write(&partial, bytes).await?;
    tokio::fs::rename(&partial, dir.join(file_name)).await?;
    Ok(owner.path(file_name))
}

pub async fn exists(covers_dir: &Path, relative: &str) -> io::Result<bool> {
    tokio::fs::try_exists(covers_dir.join(relative)).await
}

/// Deletes the owner's files except `kept`, a relative path. Best effort.
pub async fn keep_only(covers_dir: &Path, owner: Owner, kept: Option<&str>) {
    let dir = covers_dir.join(owner.dir());
    let Some(kept) = kept else {
        if let Err(err) = tokio::fs::remove_dir_all(&dir).await
            && err.kind() != io::ErrorKind::NotFound
        {
            tracing::warn!(dir = %dir.display(), error = %err, "could not delete covers");
        }
        return;
    };
    let Ok(mut entries) = tokio::fs::read_dir(&dir).await else {
        return;
    };
    let kept = covers_dir.join(kept);
    while let Ok(Some(entry)) = entries.next_entry().await {
        if entry.path() != kept {
            let _ = tokio::fs::remove_file(entry.path()).await;
        }
    }
}
