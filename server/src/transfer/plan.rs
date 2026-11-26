//! Turns parsed rows into a plan of creates and updates, without touching the
//! database. The preview shows the plan; the import applies it.

use std::collections::{HashMap, HashSet};

use serde::Serialize;
use utoipa::ToSchema;

use super::parse::RawRow;
use crate::db::labels::{PER_GAME_MAX, SEPARATORS};
use crate::db::play_statuses;
use crate::db::transfer::snapshot::{CopyRow, GameRow, ReleaseRow, Snapshot};
use crate::domain::{self, Condition, CopyStatus, Format, Region};
use crate::money;

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
/// A device the import adds, always as a console.
pub struct NewDevice {
    pub name: String,
    pub platforms: Vec<Ref>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NewAccount {
    pub name: String,
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
    pub account: Option<Ref>,
    pub format: Format,
    pub store: Option<Ref>,
    pub condition: Option<Condition>,
    pub notes: Option<String>,
    pub edition: Option<String>,
    pub purchased_on: Option<String>,
    pub price: Option<i64>,
    pub currency: Option<String>,
    pub estimated_value: Option<i64>,
    pub estimated_currency: Option<String>,
    pub barcode: Option<String>,
    /// The existing bundle copy this copy comes in.
    pub bundle: Option<i64>,
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
    pub accounts: Vec<NewAccount>,
    pub games: Vec<NewGame>,
    pub releases: Vec<NewRelease>,
    /// Existing games whose play status changes.
    pub game_statuses: Vec<(i64, String)>,
    /// Existing games that get an IGDB id.
    pub game_igdb_links: Vec<(i64, i64)>,
    /// Existing releases whose title changes.
    pub release_titles: Vec<(i64, String)>,
    /// Label names to create.
    pub labels: Vec<String>,
    /// Games and the labels they gain.
    pub game_labels: Vec<(Ref, Ref)>,
    pub copies: Vec<CopyWrite>,
    pub rows: Vec<RowReport>,
}

impl Plan {
    pub fn has_errors(&self) -> bool {
        self.rows.iter().any(|row| row.action == RowAction::Error)
    }
}

/// Folds case as SQLite's `NOCASE` and `lower()` do, so names and product
/// codes the database keeps apart stay apart.
fn key(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

/// Game titles are not unique, so they match across any letter case.
fn title_key(value: &str) -> String {
    value.trim().to_lowercase()
}

pub fn plan(rows: &[RawRow], snapshot: &Snapshot) -> Plan {
    let mut planner = Planner::new(snapshot);
    for row in rows {
        planner.row(row);
    }
    planner.plan
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
    /// Controllers, accessories and toys, by name key; copies cannot be on them.
    hardware_only: HashSet<String>,
    device_platforms: HashMap<i64, HashSet<i64>>,
    /// The platforms each new device plays, by its index in the plan.
    new_device_platforms: HashSet<(usize, Ref)>,
    stores: HashMap<String, Ref>,
    accounts: HashMap<String, Ref>,
    /// The platforms each new account holds, by its index in the plan.
    new_account_platforms: HashSet<(usize, Ref)>,
    labels: HashMap<String, Ref>,
    game_labels: HashSet<(Ref, Ref)>,
    label_counts: HashMap<Ref, usize>,
    games_by_igdb: HashMap<i64, Ref>,
    games_by_title: HashMap<String, Vec<Ref>>,
    games_by_id: HashMap<i64, &'a GameRow>,
    releases_by_id: HashMap<i64, &'a ReleaseRow>,
    /// Games without an IGDB id, by title.
    unlinked_by_title: HashMap<String, Vec<Ref>>,
    /// Existing games this import sets a play status on.
    statuses_set: HashSet<i64>,
    releases: HashMap<(Ref, Ref, Region, String), Ref>,
    copies: HashMap<i64, &'a CopyRow>,
    seen_copies: HashMap<i64, u64>,
    /// Each copy inside a bundle, and how many copies each bundle holds, as
    /// the rows so far leave them.
    bundle_of: HashMap<i64, i64>,
    contents: HashMap<i64, usize>,
    game_statuses: HashMap<Ref, String>,
    release_titles: HashMap<Ref, String>,
    /// Where each existing release's new title sits in `plan.release_titles`.
    release_title_at: HashMap<i64, usize>,
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
                .entry(title_key(&g.title))
                .or_default()
                .push(Ref::Existing(g.id));
            if let Some(igdb_id) = g.igdb_id {
                games_by_igdb.insert(igdb_id, Ref::Existing(g.id));
            }
        }

        Self {
            snapshot,
            plan: Plan::default(),
            games_by_id: snapshot.games.iter().map(|g| (g.id, g)).collect(),
            releases_by_id: snapshot.releases.iter().map(|r| (r.id, r)).collect(),
            unlinked_by_title: snapshot.games.iter().filter(|g| g.igdb_id.is_none()).fold(
                HashMap::new(),
                |mut unlinked, g| {
                    unlinked
                        .entry(title_key(&g.title))
                        .or_insert_with(Vec::new)
                        .push(Ref::Existing(g.id));
                    unlinked
                },
            ),
            statuses_set: HashSet::new(),
            release_title_at: HashMap::new(),
            platforms,
            devices: snapshot
                .devices
                .iter()
                .map(|d| (key(&d.name), Ref::Existing(d.id)))
                .collect(),
            hardware_only: snapshot
                .devices
                .iter()
                .filter(|d| !d.holds_games)
                .map(|d| key(&d.name))
                .collect(),
            device_platforms: snapshot
                .devices
                .iter()
                .map(|d| (d.id, d.platform_ids.iter().copied().collect()))
                .collect(),
            new_device_platforms: HashSet::new(),
            stores: snapshot
                .stores
                .iter()
                .map(|s| (key(&s.name), Ref::Existing(s.id)))
                .collect(),
            accounts: snapshot
                .accounts
                .iter()
                .map(|a| (key(&a.name), Ref::Existing(a.id)))
                .collect(),
            new_account_platforms: HashSet::new(),
            labels: snapshot
                .labels
                .iter()
                .map(|l| (key(&l.name), Ref::Existing(l.id)))
                .collect(),
            game_labels: snapshot
                .game_labels
                .iter()
                .map(|&(game, label)| (Ref::Existing(game), Ref::Existing(label)))
                .collect(),
            label_counts: snapshot.game_labels.iter().fold(
                HashMap::new(),
                |mut counts, &(game, _)| {
                    *counts.entry(Ref::Existing(game)).or_default() += 1;
                    counts
                },
            ),
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
            bundle_of: snapshot
                .copies
                .iter()
                .filter_map(|c| c.bundle_id.map(|b| (c.id, b)))
                .collect(),
            contents: snapshot.copies.iter().filter_map(|c| c.bundle_id).fold(
                HashMap::new(),
                |mut counts, b| {
                    *counts.entry(b).or_default() += 1;
                    counts
                },
            ),
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
        }
    }

    /// A row that updates a copy, with the copy columns its file does not
    /// have taken from that copy, so the import changes only what it has.
    fn kept(&self, raw: &RawRow) -> Option<RawRow> {
        if raw.absent.is_empty() {
            return None;
        }
        let copy = *self
            .copies
            .get(&raw.copy_id.as_deref()?.parse::<i64>().ok()?)?;
        let release = *self.releases_by_id.get(&copy.release_id)?;
        let snapshot = self.snapshot;
        let money = |minor: Option<i64>, currency: Option<&str>| {
            minor
                .zip(currency)
                .map(|(minor, currency)| money::format(minor, currency))
        };
        let has = |column: &str| !raw.absent.contains(&column);
        let format = raw.format.as_deref().and_then(Format::parse_loose);
        let digital = format == Some(Format::Digital);
        let physical = format == Some(Format::Physical);
        let mut row = raw.clone();
        for &column in raw.absent.iter() {
            let value = match column {
                "status" => Some(copy.status.as_str().to_owned()),
                "platform" => snapshot
                    .platforms
                    .iter()
                    .find(|p| p.id == release.platform_id)
                    .map(|p| p.name.clone()),
                "region" => Some(release.region.as_str().to_owned()),
                "product_code" => release.product_code.clone(),
                "format" => Some(copy.format.as_str().to_owned()),
                "device" => copy
                    .device_id
                    .and_then(|id| snapshot.devices.iter().find(|d| d.id == id))
                    .map(|d| d.name.clone()),
                "account" if physical => continue,
                "account" => copy
                    .account_id
                    .and_then(|id| snapshot.accounts.iter().find(|a| a.id == id))
                    .map(|a| a.name.clone()),
                "store" => copy
                    .store_id
                    .and_then(|id| snapshot.stores.iter().find(|s| s.id == id))
                    .map(|s| s.name.clone()),
                "condition" if digital => continue,
                "condition" => copy.condition.map(|c| c.as_str().to_owned()),
                "notes" => copy.notes.clone(),
                "edition" => copy.edition.clone(),
                "purchased_on" => copy.purchased_on.clone(),
                "currency" => copy.currency.clone(),
                "barcode" => copy.barcode.clone(),
                "bundle_copy_id" => copy.bundle_id.map(|id| id.to_string()),
                _ => continue,
            };
            if let Some(cell) = row.copy_cell_mut(column) {
                *cell = value;
            }
        }
        // A kept amount keeps its own currency. A value the file has takes
        // the file's currency, else the value's own, never the price's;
        // `row` leaves no fallback to a kept price currency.
        if !has("price") {
            row.price = money(copy.price, copy.currency.as_deref());
            row.currency = copy.currency.clone();
        }
        if !has("estimated_value") {
            row.estimated_value = money(copy.estimated_value, copy.estimated_currency.as_deref());
            row.estimated_currency = copy.estimated_currency.clone();
        } else if raw.estimated_currency.is_none() {
            row.estimated_currency = raw
                .currency
                .clone()
                .or_else(|| copy.estimated_currency.clone());
        }
        Some(row)
    }

    fn row(&mut self, raw: &RawRow) {
        let kept = self.kept(raw);
        let raw = kept.as_ref().unwrap_or(raw);
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
        let labels = label_names(raw.labels.as_deref(), &mut issues);

        let has_copy = raw.has_copy_columns();
        let copy_id = issues.id(&raw.copy_id, "copy_id");
        let status = issues
            .value(&raw.status, "status", CopyStatus::parse_loose, false)
            .unwrap_or(CopyStatus::Owned);
        let platform_name = issues.text(&raw.platform, "platform", NAME_MAX, has_copy);
        let region = issues.value(&raw.region, "region", Region::parse_loose, has_copy);
        let release_title = issues.text(&raw.release_title, "release_title", TITLE_MAX, false);
        let product_code = issues.text(&raw.product_code, "product_code", CODE_MAX, false);
        let format = issues.value(&raw.format, "format", Format::parse_loose, has_copy);
        let mut device_name = issues.text(&raw.device, "device", NAME_MAX, false);
        if device_name
            .as_deref()
            .is_some_and(|name| self.hardware_only.contains(&key(name)))
        {
            issues.error("device", "device_holds_no_games");
            device_name = None;
        }
        let mut account_name = issues.text(&raw.account, "account", NAME_MAX, false);
        if account_name.is_some() && format == Some(Format::Physical) {
            issues.warning("account", "account_physical");
            account_name = None;
        }
        let store_name = issues.text(&raw.store, "store", NAME_MAX, false);
        let condition = issues.value(&raw.condition, "condition", Condition::parse_loose, false);
        if condition.is_some() && format == Some(Format::Digital) {
            issues.error("condition", "condition_digital");
        }
        let notes = issues.text(&raw.notes, "notes", NOTES_MAX, false);
        let edition = issues.text(&raw.edition, "edition", NAME_MAX, false);
        let purchased_on = issues.value(&raw.purchased_on, "purchased_on", date, false);
        let currency = issues.value(&raw.currency, "currency", money::currency, false);
        let price = issues.amount(
            &raw.price,
            "price",
            currency.as_deref(),
            &raw.currency,
            "currency",
        );
        // A sheet with one currency column uses it for the value too. A
        // kept row's currency may be the copy's, so `kept` picked its value's.
        let (estimated_currency, value_currency_cell) = match &raw.estimated_currency {
            Some(_) => (
                issues.value(
                    &raw.estimated_currency,
                    "estimated_currency",
                    money::currency,
                    false,
                ),
                &raw.estimated_currency,
            ),
            None if kept.is_some() => (None, &raw.estimated_currency),
            None => (currency.clone(), &raw.currency),
        };
        let currency = currency.filter(|_| price.is_some());
        let estimated_value = issues.amount(
            &raw.estimated_value,
            "estimated_value",
            estimated_currency.as_deref(),
            value_currency_cell,
            "estimated_currency",
        );
        let estimated_currency = estimated_currency.filter(|_| estimated_value.is_some());
        let barcode = issues.value(&raw.barcode, "barcode", domain::barcode, false);
        let bundle_id = issues.id(&raw.bundle_copy_id, "bundle_copy_id");

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

        let bundle = bundle_id.and_then(|id| {
            let Some(bundle) = self.copies.get(&id).copied() else {
                issues.error("bundle_copy_id", "unknown_bundle");
                return None;
            };
            let holds_copies =
                copy_id.is_some_and(|c| self.contents.get(&c).is_some_and(|n| *n > 0));
            if Some(id) == copy_id || self.bundle_of.contains_key(&id) || holds_copies {
                issues.error("bundle_copy_id", "nested_bundle");
                return None;
            }
            Some(bundle)
        });
        // Price and purchase date stay on the bundle copy.
        let (purchased_on, price, currency, estimated_value, estimated_currency) =
            if bundle.is_some() {
                for (cell, column) in [
                    (&raw.purchased_on, "purchased_on"),
                    (&raw.price, "price"),
                    (&raw.estimated_value, "estimated_value"),
                ] {
                    if cell.is_some() {
                        issues.warning(column, "ignored_in_bundle");
                    }
                }
                (None, None, None, None, None)
            } else {
                (
                    purchased_on,
                    price,
                    currency,
                    estimated_value,
                    estimated_currency,
                )
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
        if let Some((game, _)) = game
            && self.label_count(game, &labels) > PER_GAME_MAX
        {
            issues.error("labels", "too_many_labels");
        }
        if let (Some(bundle), Some((Ref::Existing(game), _))) = (bundle, game)
            && self.copy_game(bundle).is_some_and(|g| g.id == game)
        {
            issues.error("bundle_copy_id", "same_game");
        }
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
        self.add_labels(game, &labels);
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
        let device = device_name.map(|name| self.ensure_device(device, &name, platform));
        let account = account_name.map(|name| self.ensure_account(&name, platform));
        let store = store_name.map(|name| self.ensure_store(&name));

        let write = CopyWrite {
            existing: existing_copy.map(|c| c.id),
            release,
            status,
            device,
            account,
            format,
            store,
            condition,
            notes,
            edition,
            purchased_on,
            price,
            currency,
            estimated_value,
            estimated_currency,
            barcode,
            bundle: bundle.map(|b| b.id),
        };
        if let Some(copy) = existing_copy
            && let Some(old) = self.bundle_of.remove(&copy.id)
        {
            *self.contents.entry(old).or_default() -= 1;
        }
        if let Some(bundle) = write.bundle {
            if let Some(copy) = existing_copy {
                self.bundle_of.insert(copy.id, bundle);
            }
            *self.contents.entry(bundle).or_default() += 1;
        }
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
            return Ok(
                match self
                    .unlinked_by_title
                    .get(&title_key(title))
                    .map(Vec::as_slice)
                {
                    Some([only]) => Some((*only, true)),
                    _ => None,
                },
            );
        }

        // The copy's current game wins when the title still matches.
        if let Some(copy) = existing_copy
            && let Some(game) = self.copy_game(copy)
            && title_key(&game.title) == title_key(title)
        {
            return Ok(Some((Ref::Existing(game.id), false)));
        }

        match self
            .games_by_title
            .get(&title_key(title))
            .map(Vec::as_slice)
        {
            None | Some([]) => Ok(None),
            Some([only]) => Ok(Some((*only, false))),
            Some(_) => Err("ambiguous_game"),
        }
    }

    fn copy_game(&self, copy: &CopyRow) -> Option<&'a GameRow> {
        let release = self.releases_by_id.get(&copy.release_id)?;
        self.games_by_id.get(&release.game_id).copied()
    }

    /// True when an earlier row of this import set the game's status, as
    /// opposed to the status the game already had.
    fn status_set_by_import(&self, game: Ref) -> bool {
        match game {
            Ref::New(_) => true,
            Ref::Existing(id) => self.statuses_set.contains(&id),
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
                if link && let Some(unlinked) = self.unlinked_by_title.get_mut(&title_key(title)) {
                    unlinked.retain(|g| *g != game);
                }
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
                    .entry(title_key(title))
                    .or_default()
                    .push(game);
                if let Some(igdb_id) = igdb_id {
                    self.games_by_igdb.insert(igdb_id, game);
                } else {
                    self.unlinked_by_title
                        .entry(title_key(title))
                        .or_default()
                        .push(game);
                }
                game
            }
        };

        // A status the file names counts even when the game already has it,
        // so a different one in a later row conflicts.
        if let (Some(_), Ref::Existing(id)) = (&play_status, game) {
            self.statuses_set.insert(id);
        }
        if let Some(status) = play_status
            && self.game_statuses.get(&game) != Some(&status)
        {
            self.game_statuses.insert(game, status.clone());
            match game {
                Ref::New(i) => self.plan.games[i].play_status = Some(status),
                Ref::Existing(id) => {
                    self.statuses_set.insert(id);
                    self.plan.game_statuses.push((id, status));
                }
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
                Ref::Existing(id) => match self.release_title_at.get(&id) {
                    Some(&at) => self.plan.release_titles[at].1 = title,
                    None => {
                        self.release_title_at
                            .insert(id, self.plan.release_titles.len());
                        self.plan.release_titles.push((id, title));
                    }
                },
            }
        }
        release
    }

    fn ensure_device(&mut self, found: Option<Ref>, name: &str, platform: Ref) -> Ref {
        let device = found.unwrap_or_else(|| {
            let device = Ref::New(self.plan.devices.len());
            self.plan.devices.push(NewDevice {
                name: name.to_owned(),
                platforms: Vec::new(),
            });
            self.devices.insert(key(name), device);
            device
        });
        if let Ref::New(i) = device
            && self.new_device_platforms.insert((i, platform))
        {
            self.plan.devices[i].platforms.push(platform);
        }
        device
    }

    /// How many labels the game would have with `names` added.
    fn label_count(&self, game: Ref, names: &[String]) -> usize {
        let new = names
            .iter()
            .filter(|name| {
                self.labels
                    .get(&key(name))
                    .is_none_or(|label| !self.game_labels.contains(&(game, *label)))
            })
            .count();
        self.label_counts.get(&game).copied().unwrap_or(0) + new
    }

    /// Adds labels to a game, creating the ones that do not exist. Labels
    /// the game already has are left alone, and none are removed.
    fn add_labels(&mut self, game: Ref, names: &[String]) {
        for name in names {
            let label = match self.labels.get(&key(name)) {
                Some(found) => *found,
                None => {
                    let label = Ref::New(self.plan.labels.len());
                    self.plan.labels.push(name.clone());
                    self.labels.insert(key(name), label);
                    label
                }
            };
            if self.game_labels.insert((game, label)) {
                self.plan.game_labels.push((game, label));
                *self.label_counts.entry(game).or_default() += 1;
            }
        }
    }

    fn ensure_account(&mut self, name: &str, platform: Ref) -> Ref {
        let account = match self.accounts.get(&key(name)) {
            Some(found) => *found,
            None => {
                let account = Ref::New(self.plan.accounts.len());
                self.plan.accounts.push(NewAccount {
                    name: name.to_owned(),
                    platforms: Vec::new(),
                });
                self.accounts.insert(key(name), account);
                account
            }
        };
        if let Ref::New(i) = account
            && self.new_account_platforms.insert((i, platform))
        {
            self.plan.accounts[i].platforms.push(platform);
        }
        account
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

/// The names in a labels cell, once each.
fn label_names(cell: Option<&str>, issues: &mut Issues) -> Vec<String> {
    let mut names = Vec::new();
    let mut seen = HashSet::new();
    for name in cell
        .unwrap_or_default()
        .split(SEPARATORS)
        .map(str::trim)
        .filter(|name| !name.is_empty())
    {
        if name.chars().count() > NAME_MAX {
            issues.error("labels", "too_long");
            break;
        }
        if name.contains('\0') {
            issues.error("labels", "invalid_value");
            break;
        }
        if seen.insert(key(name)) {
            if names.len() == PER_GAME_MAX {
                issues.error("labels", "too_many_labels");
                break;
            }
            names.push(name.to_owned());
        }
    }
    names
}

fn same_copy(copy: &CopyRow, write: &CopyWrite) -> bool {
    write.release == Ref::Existing(copy.release_id)
        && write.status == copy.status
        && write.device == copy.device_id.map(Ref::Existing)
        && write.account == copy.account_id.map(Ref::Existing)
        && write.format == copy.format
        && write.store == copy.store_id.map(Ref::Existing)
        && write.condition == copy.condition
        && write.notes == copy.notes
        && write.edition == copy.edition
        && write.purchased_on == copy.purchased_on
        && write.price == copy.price
        && write.currency == copy.currency
        && write.estimated_value == copy.estimated_value
        && write.estimated_currency == copy.estimated_currency
        && write.barcode == copy.barcode
        && write.bundle == copy.bundle_id
}

/// A `YYYY-MM-DD` date.
fn date(value: &str) -> Option<String> {
    time::Date::parse(
        value,
        time::macros::format_description!("[year]-[month]-[day]"),
    )
    .ok()
    .map(|_| value.to_owned())
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
            // SQLite compares text only up to a NUL, so two names that
            // differ after one would collide only when the import runs.
            Some(value) if value.contains('\0') => {
                self.error(column, "invalid_value");
                None
            }
            Some(value) => Some(value.clone()),
        }
    }

    /// An amount in `currency`'s minor unit. `currency_cell` is the raw
    /// currency cell, so a blank one is reported as missing under `currency_column`.
    fn amount(
        &mut self,
        cell: &Option<String>,
        column: &'static str,
        currency: Option<&str>,
        currency_cell: &Option<String>,
        currency_column: &'static str,
    ) -> Option<i64> {
        let text = cell.as_deref()?;
        let Some(currency) = currency else {
            if currency_cell.is_none() {
                self.error(currency_column, "required");
            }
            return None;
        };
        let minor = money::parse(text, currency);
        if minor.is_none() {
            self.error(column, "invalid_value");
        }
        minor
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
