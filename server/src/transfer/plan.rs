//! Turns parsed rows into a plan of creates and updates, without touching the
//! database. The preview shows the plan; the import applies it.

use std::collections::HashMap;

use serde::Serialize;
use utoipa::ToSchema;

use super::parse::RawRow;
use super::snapshot::{CopyRow, Snapshot};
use crate::db::devices::DeviceKind;
use crate::db::play_statuses;
use crate::domain::{Condition, CopyStatus, Format, Region};

const TITLE_MAX: usize = 200;
const NAME_MAX: usize = 100;
const CODE_MAX: usize = 40;
const NOTES_MAX: usize = 2000;

/// A row that already exists, or the n-th row the plan creates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ref {
    Existing(i64),
    New(usize),
}

#[derive(Debug, Clone, PartialEq)]
pub struct NewDevice {
    pub name: String,
    pub kind: DeviceKind,
    pub platforms: Vec<Ref>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NewGame {
    pub igdb_id: Option<i64>,
    pub title: String,
    pub play_status: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NewRelease {
    pub game: Ref,
    pub platform: Ref,
    pub region: Region,
    pub product_code: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CopyWrite {
    /// Set when the row updates this copy.
    pub existing: Option<i64>,
    pub release: Ref,
    pub status: CopyStatus,
    pub device: Option<Ref>,
    pub format: Format,
    pub store: Option<Ref>,
    pub condition: Option<Condition>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum RowAction {
    NewCopy,
    UpdateCopy,
    Unchanged,
    /// Game columns only: creates or updates the game, no copy.
    GameOnly,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub struct RowIssue {
    pub column: &'static str,
    pub code: &'static str,
    /// Errors block the import; warnings do not.
    pub error: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub struct RowReport {
    pub line: u64,
    pub game_title: Option<String>,
    pub platform: Option<String>,
    pub action: RowAction,
    pub issues: Vec<RowIssue>,
}

#[derive(Debug, Default)]
pub struct Plan {
    pub platforms: Vec<String>,
    pub stores: Vec<String>,
    pub devices: Vec<NewDevice>,
    pub games: Vec<NewGame>,
    pub releases: Vec<NewRelease>,
    /// Existing games whose play status changes.
    pub game_statuses: Vec<(i64, String)>,
    /// Existing games that get an IGDB id.
    pub game_igdb_links: Vec<(i64, i64)>,
    /// Existing releases whose title changes.
    pub release_titles: Vec<(i64, String)>,
    pub copies: Vec<CopyWrite>,
    pub rows: Vec<RowReport>,
}

impl Plan {
    pub fn has_errors(&self) -> bool {
        self.rows.iter().any(|row| row.action == RowAction::Error)
    }
}

fn key(value: &str) -> String {
    value.trim().to_lowercase()
}

pub fn plan(rows: &[RawRow], snapshot: &Snapshot) -> Plan {
    let mut planner = Planner::new(snapshot);
    for row in rows {
        planner.row(row);
    }
    planner.finish()
}

#[derive(Default)]
struct PlatformIndex {
    by_name: HashMap<String, Ref>,
    by_slug: HashMap<String, Ref>,
    /// `None` when two platforms share the abbreviation.
    by_abbreviation: HashMap<String, Option<Ref>>,
}

impl PlatformIndex {
    fn find(&self, name: &str) -> Result<Option<Ref>, &'static str> {
        let k = key(name);
        if let Some(found) = self.by_name.get(&k).or_else(|| self.by_slug.get(&k)) {
            return Ok(Some(*found));
        }
        match self.by_abbreviation.get(&k) {
            Some(Some(found)) => Ok(Some(*found)),
            Some(None) => Err("ambiguous_platform"),
            None => Ok(None),
        }
    }
}

struct Planner<'a> {
    snapshot: &'a Snapshot,
    plan: Plan,
    platforms: PlatformIndex,
    devices: HashMap<String, Ref>,
    device_platforms: HashMap<i64, Vec<i64>>,
    stores: HashMap<String, Ref>,
    games_by_igdb: HashMap<i64, Ref>,
    games_by_title: HashMap<String, Vec<Ref>>,
    releases: HashMap<(Ref, Ref, Region, String), Ref>,
    copies: HashMap<i64, &'a CopyRow>,
    seen_copies: HashMap<i64, u64>,
    game_statuses: HashMap<Ref, String>,
    release_titles: HashMap<Ref, String>,
    /// Per new device: true while every row using it is digital.
    new_device_digital_only: Vec<bool>,
}

impl<'a> Planner<'a> {
    fn new(snapshot: &'a Snapshot) -> Self {
        let mut platforms = PlatformIndex::default();
        for p in &snapshot.platforms {
            let r = Ref::Existing(p.id);
            platforms.by_name.insert(key(&p.name), r);
            platforms.by_slug.insert(key(&p.slug), r);
            if let Some(abbreviation) = &p.abbreviation {
                platforms
                    .by_abbreviation
                    .entry(key(abbreviation))
                    .and_modify(|slot| {
                        if *slot != Some(r) {
                            *slot = None;
                        }
                    })
                    .or_insert(Some(r));
            }
        }

        let mut games_by_title: HashMap<String, Vec<Ref>> = HashMap::new();
        let mut games_by_igdb = HashMap::new();
        for g in &snapshot.games {
            games_by_title
                .entry(key(&g.title))
                .or_default()
                .push(Ref::Existing(g.id));
            if let Some(igdb_id) = g.igdb_id {
                games_by_igdb.insert(igdb_id, Ref::Existing(g.id));
            }
        }

        Self {
            snapshot,
            plan: Plan::default(),
            platforms,
            devices: snapshot
                .devices
                .iter()
                .map(|d| (key(&d.name), Ref::Existing(d.id)))
                .collect(),
            device_platforms: snapshot
                .devices
                .iter()
                .map(|d| (d.id, d.platform_ids.clone()))
                .collect(),
            stores: snapshot
                .stores
                .iter()
                .map(|s| (key(&s.name), Ref::Existing(s.id)))
                .collect(),
            games_by_igdb,
            games_by_title,
            releases: snapshot
                .releases
                .iter()
                .map(|r| {
                    let code = r.product_code.as_deref().map(key).unwrap_or_default();
                    (
                        (
                            Ref::Existing(r.game_id),
                            Ref::Existing(r.platform_id),
                            r.region,
                            code,
                        ),
                        Ref::Existing(r.id),
                    )
                })
                .collect(),
            copies: snapshot.copies.iter().map(|c| (c.id, c)).collect(),
            seen_copies: HashMap::new(),
            game_statuses: snapshot
                .games
                .iter()
                .filter_map(|g| g.play_status.clone().map(|s| (Ref::Existing(g.id), s)))
                .collect(),
            release_titles: snapshot
                .releases
                .iter()
                .filter_map(|r| r.title.clone().map(|t| (Ref::Existing(r.id), t)))
                .collect(),
            new_device_digital_only: Vec::new(),
        }
    }

    fn finish(mut self) -> Plan {
        for (device, digital_only) in self
            .plan
            .devices
            .iter_mut()
            .zip(&self.new_device_digital_only)
        {
            device.kind = if *digital_only {
                DeviceKind::Account
            } else {
                DeviceKind::Console
            };
        }
        self.plan
    }

    fn row(&mut self, raw: &RawRow) {
        let mut issues = Issues::default();
        let report = |action, issues: Issues| RowReport {
            line: raw.line,
            game_title: raw.game_title.clone(),
            platform: raw.platform.clone(),
            action,
            issues: issues.0,
        };

        // ---- read every cell ----
        let title = issues.text(&raw.game_title, "game_title", TITLE_MAX, true);
        let igdb_id = issues.id(&raw.igdb_id, "igdb_id");
        let play_status = raw.play_status.as_deref().and_then(|value| {
            let found = play_statuses::resolve_name(&self.snapshot.play_statuses, value);
            if found.is_none() {
                issues.error("play_status", "invalid_value");
            }
            found
        });

        let has_copy = raw.has_copy_columns();
        let copy_id = issues.id(&raw.copy_id, "copy_id");
        let status = issues
            .value(&raw.status, "status", CopyStatus::parse_loose, false)
            .unwrap_or(CopyStatus::Owned);
        if status == CopyStatus::Wanted {
            issues.error("status", "wanted_unsupported");
        }
        let platform_name = issues.text(&raw.platform, "platform", NAME_MAX, has_copy);
        let region = issues.value(&raw.region, "region", Region::parse_loose, has_copy);
        let release_title = issues.text(&raw.release_title, "release_title", TITLE_MAX, false);
        let product_code = issues.text(&raw.product_code, "product_code", CODE_MAX, false);
        let format = issues.value(&raw.format, "format", Format::parse_loose, has_copy);
        let device_name = issues.text(&raw.device, "device", NAME_MAX, false);
        let store_name = issues.text(&raw.store, "store", NAME_MAX, false);
        let condition = issues.value(&raw.condition, "condition", Condition::parse_loose, false);
        if condition.is_some() && format == Some(Format::Digital) {
            issues.error("condition", "condition_digital");
        }
        let notes = issues.text(&raw.notes, "notes", NOTES_MAX, false);

        let existing_copy = match copy_id {
            None => None,
            Some(id) => {
                if self.seen_copies.insert(id, raw.line).is_some() {
                    issues.error("copy_id", "duplicate_copy_id");
                }
                let found = self.copies.get(&id).copied();
                if found.is_none() {
                    issues.error("copy_id", "unknown_copy");
                }
                found
            }
        };

        // ---- look things up without changing the plan ----
        let Some(title) = title else {
            self.plan.rows.push(report(RowAction::Error, issues));
            return;
        };
        let game = match self.find_game(igdb_id, &title, existing_copy) {
            Ok(found) => found,
            Err(code) => {
                issues.error("game_title", code);
                None
            }
        };
        if let (Some(status), Some(game)) = (play_status.as_deref(), game.map(|g| g.0))
            && self
                .game_statuses
                .get(&game)
                .is_some_and(|planned| planned != status)
            && self.status_set_by_import(game)
        {
            issues.error("play_status", "conflicting_play_status");
        }

        let platform = match platform_name
            .as_deref()
            .map(|name| self.platforms.find(name))
        {
            Some(Err(code)) => {
                issues.error("platform", code);
                None
            }
            Some(Ok(found)) => found,
            None => None,
        };
        let device = device_name
            .as_deref()
            .and_then(|name| self.devices.get(&key(name)).copied());
        if let (Some(Ref::Existing(device_id)), Some(platform)) = (device, platform) {
            let plays = match platform {
                Ref::Existing(platform_id) => self
                    .device_platforms
                    .get(&device_id)
                    .is_some_and(|ids| ids.contains(&platform_id)),
                Ref::New(_) => false,
            };
            if !plays {
                issues.warning("device", "device_platform_mismatch");
            }
        }

        if issues.has_errors() {
            self.plan.rows.push(report(RowAction::Error, issues));
            return;
        }

        // ---- commit the row to the plan ----
        let game = self.ensure_game(game, igdb_id, &title, play_status);
        if !has_copy {
            self.plan.rows.push(report(RowAction::GameOnly, issues));
            return;
        }

        let (Some(platform_name), Some(region), Some(format)) = (platform_name, region, format)
        else {
            unreachable!("required when the row has copy columns");
        };
        let platform = platform.unwrap_or_else(|| self.new_platform(&platform_name));
        let release = self.ensure_release(game, platform, region, product_code, release_title);
        let device = device_name.map(|name| self.ensure_device(device, &name, platform, format));
        let store = store_name.map(|name| self.ensure_store(&name));

        let write = CopyWrite {
            existing: existing_copy.map(|c| c.id),
            release,
            status,
            device,
            format,
            store,
            condition,
            notes,
        };
        let action = match existing_copy {
            None => RowAction::NewCopy,
            Some(copy) if same_copy(copy, &write) => RowAction::Unchanged,
            Some(_) => RowAction::UpdateCopy,
        };
        if action != RowAction::Unchanged {
            self.plan.copies.push(write);
        }
        self.plan.rows.push(report(action, issues));
    }

    /// `Ok(None)` means a new game. The bool is true when an existing game
    /// gets linked to the row's IGDB id.
    fn find_game(
        &self,
        igdb_id: Option<i64>,
        title: &str,
        existing_copy: Option<&CopyRow>,
    ) -> Result<Option<(Ref, bool)>, &'static str> {
        if let Some(igdb_id) = igdb_id {
            if let Some(found) = self.games_by_igdb.get(&igdb_id) {
                return Ok(Some((*found, false)));
            }
            // A game imported earlier without an IGDB id gets linked.
            let unlinked: Vec<Ref> = self
                .games_by_title
                .get(&key(title))
                .into_iter()
                .flatten()
                .copied()
                .filter(|r| !self.has_igdb_id(*r))
                .collect();
            return Ok(match unlinked.as_slice() {
                [only] => Some((*only, true)),
                _ => None,
            });
        }

        // The copy's current game wins when the title still matches.
        if let Some(copy) = existing_copy
            && let Some(game) = self.copy_game(copy)
            && key(&game.title) == key(title)
        {
            return Ok(Some((Ref::Existing(game.id), false)));
        }

        match self.games_by_title.get(&key(title)).map(Vec::as_slice) {
            None | Some([]) => Ok(None),
            Some([only]) => Ok(Some((*only, false))),
            Some(_) => Err("ambiguous_game"),
        }
    }

    fn has_igdb_id(&self, game: Ref) -> bool {
        match game {
            Ref::Existing(id) => {
                self.snapshot
                    .games
                    .iter()
                    .any(|g| g.id == id && g.igdb_id.is_some())
                    || self.plan.game_igdb_links.iter().any(|(g, _)| *g == id)
            }
            Ref::New(i) => self.plan.games[i].igdb_id.is_some(),
        }
    }

    fn copy_game(&self, copy: &CopyRow) -> Option<&'a crate::transfer::snapshot::GameRow> {
        let release = self
            .snapshot
            .releases
            .iter()
            .find(|r| r.id == copy.release_id)?;
        self.snapshot.games.iter().find(|g| g.id == release.game_id)
    }

    /// True when an earlier row of this import set the game's status, as
    /// opposed to the status the game already had.
    fn status_set_by_import(&self, game: Ref) -> bool {
        match game {
            Ref::New(_) => true,
            Ref::Existing(id) => self.plan.game_statuses.iter().any(|(g, _)| *g == id),
        }
    }

    fn ensure_game(
        &mut self,
        found: Option<(Ref, bool)>,
        igdb_id: Option<i64>,
        title: &str,
        play_status: Option<String>,
    ) -> Ref {
        let game = match found {
            Some((game, link)) => {
                if let (true, Some(igdb_id), Ref::Existing(id)) = (link, igdb_id, game) {
                    self.plan.game_igdb_links.push((id, igdb_id));
                    self.games_by_igdb.insert(igdb_id, game);
                }
                if let (true, Some(igdb_id), Ref::New(i)) = (link, igdb_id, game) {
                    self.plan.games[i].igdb_id = Some(igdb_id);
                    self.games_by_igdb.insert(igdb_id, game);
                }
                game
            }
            None => {
                let game = Ref::New(self.plan.games.len());
                self.plan.games.push(NewGame {
                    igdb_id,
                    title: title.to_owned(),
                    play_status: None,
                });
                self.games_by_title
                    .entry(key(title))
                    .or_default()
                    .push(game);
                if let Some(igdb_id) = igdb_id {
                    self.games_by_igdb.insert(igdb_id, game);
                }
                game
            }
        };

        if let Some(status) = play_status
            && self.game_statuses.get(&game) != Some(&status)
        {
            self.game_statuses.insert(game, status.clone());
            match game {
                Ref::New(i) => self.plan.games[i].play_status = Some(status),
                Ref::Existing(id) => self.plan.game_statuses.push((id, status)),
            }
        }
        game
    }

    fn new_platform(&mut self, name: &str) -> Ref {
        let platform = Ref::New(self.plan.platforms.len());
        self.plan.platforms.push(name.to_owned());
        self.platforms.by_name.insert(key(name), platform);
        platform
    }

    fn ensure_release(
        &mut self,
        game: Ref,
        platform: Ref,
        region: Region,
        product_code: Option<String>,
        title: Option<String>,
    ) -> Ref {
        let code = product_code.as_deref().map(key).unwrap_or_default();
        let lookup = (game, platform, region, code);
        let release = match self.releases.get(&lookup) {
            Some(found) => *found,
            None => {
                let release = Ref::New(self.plan.releases.len());
                self.plan.releases.push(NewRelease {
                    game,
                    platform,
                    region,
                    product_code,
                    title: None,
                });
                self.releases.insert(lookup, release);
                release
            }
        };

        if let Some(title) = title
            && self.release_titles.get(&release) != Some(&title)
        {
            self.release_titles.insert(release, title.clone());
            match release {
                Ref::New(i) => self.plan.releases[i].title = Some(title),
                Ref::Existing(id) => {
                    self.plan.release_titles.retain(|(r, _)| *r != id);
                    self.plan.release_titles.push((id, title));
                }
            }
        }
        release
    }

    fn ensure_device(
        &mut self,
        found: Option<Ref>,
        name: &str,
        platform: Ref,
        format: Format,
    ) -> Ref {
        let device = found.unwrap_or_else(|| {
            let device = Ref::New(self.plan.devices.len());
            self.plan.devices.push(NewDevice {
                name: name.to_owned(),
                kind: DeviceKind::Console,
                platforms: Vec::new(),
            });
            self.new_device_digital_only.push(true);
            self.devices.insert(key(name), device);
            device
        });
        if let Ref::New(i) = device {
            if !self.plan.devices[i].platforms.contains(&platform) {
                self.plan.devices[i].platforms.push(platform);
            }
            if format == Format::Physical {
                self.new_device_digital_only[i] = false;
            }
        }
        device
    }

    fn ensure_store(&mut self, name: &str) -> Ref {
        if let Some(found) = self.stores.get(&key(name)) {
            return *found;
        }
        let store = Ref::New(self.plan.stores.len());
        self.plan.stores.push(name.to_owned());
        self.stores.insert(key(name), store);
        store
    }
}

fn same_copy(copy: &CopyRow, write: &CopyWrite) -> bool {
    write.release == Ref::Existing(copy.release_id)
        && write.status == copy.status
        && write.device == copy.device_id.map(Ref::Existing)
        && write.format == copy.format
        && write.store == copy.store_id.map(Ref::Existing)
        && write.condition == copy.condition
        && write.notes == copy.notes
}

#[derive(Default)]
struct Issues(Vec<RowIssue>);

impl Issues {
    fn error(&mut self, column: &'static str, code: &'static str) {
        self.0.push(RowIssue {
            column,
            code,
            error: true,
        });
    }

    fn warning(&mut self, column: &'static str, code: &'static str) {
        self.0.push(RowIssue {
            column,
            code,
            error: false,
        });
    }

    fn has_errors(&self) -> bool {
        self.0.iter().any(|issue| issue.error)
    }

    fn text(
        &mut self,
        cell: &Option<String>,
        column: &'static str,
        max: usize,
        required: bool,
    ) -> Option<String> {
        match cell {
            None if required => {
                self.error(column, "required");
                None
            }
            None => None,
            Some(value) if value.chars().count() > max => {
                self.error(column, "too_long");
                None
            }
            Some(value) => Some(value.clone()),
        }
    }

    fn id(&mut self, cell: &Option<String>, column: &'static str) -> Option<i64> {
        let value = cell.as_deref()?;
        match value.parse::<i64>() {
            Ok(id) if id > 0 => Some(id),
            _ => {
                self.error(column, "invalid_number");
                None
            }
        }
    }

    fn value<T>(
        &mut self,
        cell: &Option<String>,
        column: &'static str,
        parse: fn(&str) -> Option<T>,
        required: bool,
    ) -> Option<T> {
        match cell.as_deref() {
            None => {
                if required {
                    self.error(column, "required");
                }
                None
            }
            Some(value) => {
                let parsed = parse(value);
                if parsed.is_none() {
                    self.error(column, "invalid_value");
                }
                parsed
            }
        }
    }
}
