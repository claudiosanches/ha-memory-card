//! Games with two or more owned copies, with labels saying how the copies differ.

use std::collections::BTreeSet;

use serde::Serialize;
use sqlx::SqlitePool;
use utoipa::ToSchema;

use crate::domain::{Condition, Format, Region};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum DuplicateLabel {
    /// Owned on more than one platform.
    CrossPlatform,
    /// More than one region.
    CrossRegion,
    /// More than one device, not counting copies without one.
    CrossDevice,
    /// Both physical and digital.
    PhysicalDigital,
    /// Two or more copies of one release, with or without a device.
    SameRelease,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DuplicateCopy {
    pub id: i64,
    pub release_id: i64,
    pub platform_id: i64,
    pub region: Region,
    pub release_title: Option<String>,
    pub format: Format,
    pub device_id: Option<i64>,
    pub account_id: Option<i64>,
    pub store_id: Option<i64>,
    pub condition: Option<Condition>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DuplicateGroup {
    pub game_id: i64,
    pub title: String,
    pub cover_path: Option<String>,
    pub labels: Vec<DuplicateLabel>,
    /// In platform list order, then region.
    pub copies: Vec<DuplicateCopy>,
}

#[derive(sqlx::FromRow)]
struct Row {
    game_id: i64,
    title: String,
    cover_path: Option<String>,
    id: i64,
    release_id: i64,
    platform_id: i64,
    region: Region,
    release_title: Option<String>,
    format: Format,
    device_id: Option<i64>,
    account_id: Option<i64>,
    store_id: Option<i64>,
    condition: Option<Condition>,
    notes: Option<String>,
}

pub async fn list(pool: &SqlitePool) -> sqlx::Result<Vec<DuplicateGroup>> {
    let rows: Vec<Row> = sqlx::query_as(
        "SELECT g.id AS game_id, g.title, g.cover_path, c.id, c.release_id, r.platform_id,
                r.region, r.title AS release_title, c.format, c.device_id, c.account_id, c.store_id,
                c.condition, c.notes
         FROM copies c
         JOIN releases r ON r.id = c.release_id
         JOIN games g ON g.id = r.game_id
         JOIN platforms p ON p.id = r.platform_id
         WHERE c.status = 'owned' AND r.game_id IN (
             SELECT r2.game_id FROM copies c2 JOIN releases r2 ON r2.id = c2.release_id
             WHERE c2.status = 'owned' GROUP BY r2.game_id HAVING COUNT(*) > 1)
         ORDER BY g.sort_title, g.id, p.sort_order, r.region, c.id",
    )
    .fetch_all(pool)
    .await?;

    let mut groups: Vec<DuplicateGroup> = Vec::new();
    for row in rows {
        if groups.last().is_none_or(|g| g.game_id != row.game_id) {
            groups.push(DuplicateGroup {
                game_id: row.game_id,
                title: row.title.clone(),
                cover_path: row.cover_path.clone(),
                labels: Vec::new(),
                copies: Vec::new(),
            });
        }
        let group = groups.last_mut().expect("pushed above");
        group.copies.push(DuplicateCopy {
            id: row.id,
            release_id: row.release_id,
            platform_id: row.platform_id,
            region: row.region,
            release_title: row.release_title,
            format: row.format,
            device_id: row.device_id,
            account_id: row.account_id,
            store_id: row.store_id,
            condition: row.condition,
            notes: row.notes,
        });
    }
    for group in &mut groups {
        group.labels = labels(&group.copies);
    }
    Ok(groups)
}

fn labels(copies: &[DuplicateCopy]) -> Vec<DuplicateLabel> {
    let checks = [
        (
            DuplicateLabel::CrossPlatform,
            distinct(copies.iter().map(|c| c.platform_id)) > 1,
        ),
        (
            DuplicateLabel::CrossRegion,
            distinct(copies.iter().map(|c| c.region.as_str())) > 1,
        ),
        (
            DuplicateLabel::CrossDevice,
            distinct(copies.iter().filter_map(|c| c.device_id)) > 1
                || distinct(copies.iter().filter_map(|c| c.account_id)) > 1,
        ),
        (
            DuplicateLabel::PhysicalDigital,
            distinct(copies.iter().map(|c| c.format.as_str())) > 1,
        ),
        (
            DuplicateLabel::SameRelease,
            distinct(copies.iter().map(|c| c.release_id)) < copies.len(),
        ),
    ];
    checks
        .into_iter()
        .filter(|(_, applies)| *applies)
        .map(|(label, _)| label)
        .collect()
}

fn distinct<T: Ord>(values: impl Iterator<Item = T>) -> usize {
    values.collect::<BTreeSet<_>>().len()
}
