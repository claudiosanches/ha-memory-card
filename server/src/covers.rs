//! Image files under the covers directory: covers in `game-<id>/` and
//! `release-<id>/`, copy photos in `copy-<id>/`. Paths stored in the database
//! are relative to it.

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

/// Waits for the game's other cover changes. Holding it from writing a file to
/// clearing out the old ones keeps one change from deleting what another is
/// storing. Games share 64 locks.
pub async fn lock(game_id: i64) -> tokio::sync::MutexGuard<'static, ()> {
    static LOCKS: [tokio::sync::Mutex<()>; 64] = [const { tokio::sync::Mutex::const_new(()) }; 64];
    LOCKS[game_id.rem_euclid(64) as usize].lock().await
}

/// Writes through a temporary file so a reader never sees half an image.
pub async fn write(
    covers_dir: &Path,
    owner: Owner,
    file_name: &str,
    bytes: &[u8],
) -> io::Result<String> {
    write_in(covers_dir, &owner.dir(), file_name, bytes).await
}

/// The directory holding a copy's photos.
pub fn copy_dir(copy_id: i64) -> String {
    format!("copy-{copy_id}")
}

/// Writes `file_name` into `dir` and returns its relative path.
pub async fn write_in(
    covers_dir: &Path,
    dir: &str,
    file_name: &str,
    bytes: &[u8],
) -> io::Result<String> {
    let full = covers_dir.join(dir);
    tokio::fs::create_dir_all(&full).await?;
    let partial = full.join(format!("{file_name}.part"));
    tokio::fs::write(&partial, bytes).await?;
    tokio::fs::rename(&partial, full.join(file_name)).await?;
    Ok(format!("{dir}/{file_name}"))
}

pub async fn exists(covers_dir: &Path, relative: &str) -> io::Result<bool> {
    tokio::fs::try_exists(covers_dir.join(relative)).await
}

/// Copy ids that have a photo directory.
pub async fn copy_dirs(covers_dir: &Path) -> Vec<i64> {
    let Ok(mut entries) = tokio::fs::read_dir(covers_dir).await else {
        return Vec::new();
    };
    let mut ids = Vec::new();
    while let Ok(Some(entry)) = entries.next_entry().await {
        if let Some(id) = entry
            .file_name()
            .to_str()
            .and_then(|name| name.strip_prefix("copy-"))
            .and_then(|id| id.parse().ok())
        {
            ids.push(id);
        }
    }
    ids
}

pub async fn remove_file(covers_dir: &Path, relative: &str) {
    if let Err(err) = tokio::fs::remove_file(covers_dir.join(relative)).await
        && err.kind() != io::ErrorKind::NotFound
    {
        tracing::warn!(path = relative, error = %err, "could not delete an image");
    }
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
