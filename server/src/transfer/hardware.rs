//! Hardware CSV: one row per device, console, controller, accessory or toy.
//! Rows match a device by `device_id`, then by name. Columns the file does
//! not have keep the device's values, while an empty cell clears one.

use std::collections::{HashMap, HashSet};

use serde::Serialize;
use utoipa::ToSchema;

use super::UTF8_BOM;
use super::export::{plain, unplain};
use super::parse::{detect_delimiter, too_wide};
use crate::api::devices::DeviceInput;
use crate::db::Pool;
use crate::db::devices::{self, Device, DeviceFields, DeviceKind};
use crate::db::platforms::{self, Platform};
use crate::db::stores::{self, Store, StoreFields};
use crate::domain::{Condition, CopyStatus, Region};
use crate::error::{ApiError, ApiResult};

pub const COLUMNS: [&str; 19] = [
    "device_id",
    "status",
    "name",
    "kind",
    "model",
    "platform",
    "plays",
    "region",
    "condition",
    "store",
    "purchased_on",
    "price",
    "currency",
    "estimated_value",
    "estimated_currency",
    "serial_number",
    "barcode",
    "notes",
    "active",
];

/// Far more than a household owns.
const MAX_ROWS: usize = 5_000;

#[derive(Debug, Serialize, ToSchema)]
pub struct HardwarePreview {
    /// Why the file cannot be read at all, such as `missing_column`.
    pub file_issue: Option<String>,
    pub rows: Vec<HardwareRow>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct HardwareRow {
    /// The file's line, counting the header as 1.
    pub line: usize,
    pub name: String,
    /// The device it updates, or none for a new one.
    pub device_id: Option<i64>,
    pub issues: Vec<HardwareIssue>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct HardwareIssue {
    pub column: String,
    pub code: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct HardwareResult {
    pub created: usize,
    pub updated: usize,
    /// Stores named in the file that were added.
    pub new_stores: usize,
}

/// The whole hardware list, with a byte order mark so spreadsheets read UTF-8.
pub async fn export(pool: &Pool) -> ApiResult<Vec<u8>> {
    let devices = devices::list(pool).await?;
    let platforms = platforms::list(pool).await?;
    let stores = stores::list(pool).await?;
    let platform = |id: i64| {
        platforms
            .iter()
            .find(|p| p.id == id)
            .map(|p| p.name.clone())
    };
    let mut out = UTF8_BOM.to_vec();
    {
        let mut writer = csv::Writer::from_writer(&mut out);
        writer
            .write_record(COLUMNS)
            .map_err(|err| ApiError::Internal(err.to_string()))?;
        for d in devices {
            let text = |value: Option<String>| plain(value.unwrap_or_default());
            let record = [
                d.id.to_string(),
                d.status.as_str().to_owned(),
                plain(d.name),
                kind_text(d.kind).to_owned(),
                text(d.model),
                d.platform_id.and_then(platform).unwrap_or_default(),
                d.platform_ids
                    .iter()
                    .filter_map(|id| platform(*id))
                    .collect::<Vec<_>>()
                    .join("; "),
                d.region.map(|r| r.as_str().to_owned()).unwrap_or_default(),
                d.condition
                    .map(|c| c.as_str().to_owned())
                    .unwrap_or_default(),
                d.store_id
                    .and_then(|id| stores.iter().find(|s| s.id == id))
                    .map(|s| plain(s.name.clone()))
                    .unwrap_or_default(),
                d.purchased_on.unwrap_or_default(),
                d.price.unwrap_or_default(),
                d.currency.unwrap_or_default(),
                d.estimated_value.unwrap_or_default(),
                d.estimated_currency.unwrap_or_default(),
                text(d.serial_number),
                d.barcode.unwrap_or_default(),
                text(d.notes),
                if d.active { "yes" } else { "no" }.to_owned(),
            ];
            writer
                .write_record(&record)
                .map_err(|err| ApiError::Internal(err.to_string()))?;
        }
        writer
            .flush()
            .map_err(|err| ApiError::Internal(err.to_string()))?;
    }
    Ok(out)
}

/// The header row only.
pub fn template() -> Vec<u8> {
    let mut out = UTF8_BOM.to_vec();
    out.extend_from_slice(COLUMNS.join(",").as_bytes());
    out.extend_from_slice(b"\r\n");
    out
}

fn kind_text(kind: DeviceKind) -> &'static str {
    match kind {
        DeviceKind::Console => "console",
        DeviceKind::Handheld => "handheld",
        DeviceKind::Pc => "pc",
        DeviceKind::Account => "account",
        DeviceKind::Controller => "controller",
        DeviceKind::Accessory => "accessory",
        DeviceKind::Toy => "toy",
    }
}

fn parse_kind(text: &str) -> Option<DeviceKind> {
    let key: String = text
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .collect::<String>()
        .to_lowercase();
    Some(match key.as_str() {
        "console" => DeviceKind::Console,
        "handheld" | "portable" => DeviceKind::Handheld,
        "pc" | "computer" => DeviceKind::Pc,
        "account" => DeviceKind::Account,
        "controller" | "gamepad" => DeviceKind::Controller,
        "accessory" => DeviceKind::Accessory,
        "toy" | "amiibo" | "figure" => DeviceKind::Toy,
        _ => return None,
    })
}

fn key(text: &str) -> String {
    text.trim().to_lowercase()
}

/// A planned write for one row, once it has no issues.
pub struct Planned {
    pub device_id: Option<i64>,
    pub input: DeviceInput,
    /// A store to add first, by name, when the row names one that is new.
    pub new_store: Option<String>,
}

pub struct Plan {
    pub file_issue: Option<&'static str>,
    pub rows: Vec<(HardwareRow, Option<Planned>)>,
}

/// Reads the file and checks every row against what is stored.
pub async fn plan(pool: &Pool, bytes: &[u8]) -> ApiResult<Plan> {
    let records = match read(bytes) {
        Ok(records) => records,
        Err(code) => {
            return Ok(Plan {
                file_issue: Some(code),
                rows: Vec::new(),
            });
        }
    };
    let devices = devices::list(pool).await?;
    let platforms = platforms::list(pool).await?;
    let stores = stores::list(pool).await?;
    let mut seen: HashSet<String> = HashSet::new();
    let mut rows = Vec::new();
    for (line, cells) in records {
        rows.push(plan_row(
            line, &cells, &devices, &platforms, &stores, &mut seen,
        ));
    }
    Ok(Plan {
        file_issue: None,
        rows,
    })
}

type Cells = HashMap<&'static str, String>;

/// The rows by column, with their line numbers, or why the file cannot be read.
fn read(bytes: &[u8]) -> Result<Vec<(usize, Cells)>, &'static str> {
    let bytes = bytes.strip_prefix(UTF8_BOM).unwrap_or(bytes);
    let text = std::str::from_utf8(bytes).map_err(|_| "not_utf8")?;
    if text.trim().is_empty() {
        return Err("empty_file");
    }
    let delimiter = detect_delimiter(text);
    if too_wide(text.as_bytes(), delimiter) {
        return Err("too_many_columns");
    }
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(delimiter)
        .flexible(true)
        .trim(csv::Trim::All)
        .from_reader(text.as_bytes());
    let headers = reader.headers().map_err(|_| "malformed")?.clone();
    let index: Vec<(usize, &'static str)> = headers
        .iter()
        .enumerate()
        .filter_map(|(at, header)| {
            let name = header.trim().to_lowercase().replace([' ', '-'], "_");
            COLUMNS.iter().find(|c| **c == name).map(|c| (at, *c))
        })
        .collect();
    if !index.iter().any(|(_, c)| *c == "name") {
        return Err("missing_column");
    }
    let mut rows = Vec::new();
    for (read, record) in reader.records().enumerate() {
        if read == MAX_ROWS {
            return Err("too_many_rows");
        }
        let record = record.map_err(|_| "malformed")?;
        if record.iter().all(|cell| cell.trim().is_empty()) {
            continue;
        }
        let cells = index
            .iter()
            .map(|(at, column)| (*column, record.get(*at).unwrap_or("").to_owned()))
            .collect();
        rows.push((read + 2, cells));
    }
    Ok(rows)
}

fn plan_row(
    line: usize,
    cells: &Cells,
    devices: &[Device],
    platforms: &[Platform],
    stores: &[Store],
    seen: &mut HashSet<String>,
) -> (HardwareRow, Option<Planned>) {
    let mut issues: Vec<HardwareIssue> = Vec::new();
    let mut issue = |column: &str, code: &str| {
        issues.push(HardwareIssue {
            column: column.to_owned(),
            code: code.to_owned(),
        });
    };
    let cell = |column: &str| cells.get(column).map(|v| unplain(v.trim()));
    let filled = |column: &str| cell(column).filter(|v| !v.is_empty());

    let name = filled("name").unwrap_or_default();
    let by_id = match filled("device_id") {
        None => None,
        Some(text) => match text
            .parse::<i64>()
            .ok()
            .and_then(|id| devices.iter().find(|d| d.id == id))
        {
            Some(device) => Some(device),
            None => {
                issue("device_id", "unknown_device");
                None
            }
        },
    };
    let existing = by_id.or_else(|| devices.iter().find(|d| key(&d.name) == key(&name)));
    if name.is_empty() {
        issue("name", "required");
    } else {
        if !seen.insert(key(&name)) {
            issue("name", "duplicate_row");
        }
        if devices
            .iter()
            .any(|d| key(&d.name) == key(&name) && Some(d.id) != existing.map(|e| e.id))
        {
            issue("name", "name_taken");
        }
    }

    let mut input = existing.map_or_else(DeviceInput::blank, DeviceInput::from_device);
    input.name = name.clone();
    if let Some(text) = cell("kind") {
        match parse_kind(&text) {
            Some(kind) => input.kind = kind,
            None if text.is_empty() && existing.is_some() => {}
            None => issue(
                "kind",
                if text.is_empty() {
                    "required"
                } else {
                    "invalid_value"
                },
            ),
        }
    } else if existing.is_none() {
        issue("kind", "required");
    }
    if let Some(text) = cell("status") {
        input.status = if text.is_empty() {
            Some(CopyStatus::Owned)
        } else {
            match CopyStatus::parse_loose(&text) {
                Some(status) => Some(status),
                None => {
                    issue("status", "invalid_value");
                    None
                }
            }
        };
    }
    let platform_id = |text: &str| {
        platforms
            .iter()
            .find(|p| {
                key(&p.name) == key(text)
                    || p.abbreviation
                        .as_deref()
                        .is_some_and(|a| key(a) == key(text))
                    || key(&p.slug) == key(text)
            })
            .map(|p| p.id)
    };
    if let Some(text) = cell("platform") {
        input.platform_id = if text.is_empty() {
            None
        } else {
            let found = platform_id(&text);
            if found.is_none() {
                issue("platform", "unknown_platform");
            }
            found
        };
    }
    if let Some(text) = cell("plays") {
        let mut ids = Vec::new();
        for part in text
            .split([';', ','])
            .map(str::trim)
            .filter(|p| !p.is_empty())
        {
            match platform_id(part) {
                Some(id) => ids.push(id),
                None => issue("plays", "unknown_platform"),
            }
        }
        input.platform_ids = ids;
    }
    if let Some(text) = cell("region") {
        input.region = None;
        if !text.is_empty() {
            match Region::parse_loose(&text) {
                Some(region) => input.region = Some(region),
                None => issue("region", "invalid_value"),
            }
        }
    }
    if let Some(text) = cell("condition") {
        input.condition = None;
        if !text.is_empty() {
            match Condition::parse_loose(&text) {
                Some(condition) => input.condition = Some(condition),
                None => issue("condition", "invalid_value"),
            }
        }
    }
    let mut new_store = None;
    if let Some(text) = cell("store") {
        input.store_id = None;
        if !text.is_empty() {
            match stores.iter().find(|s| key(&s.name) == key(&text)) {
                Some(store) => input.store_id = Some(store.id),
                None => new_store = Some(text),
            }
        }
    }
    if let Some(text) = cell("active") {
        match key(&text).as_str() {
            "" | "yes" | "true" | "1" => input.active = true,
            "no" | "false" | "0" => input.active = false,
            _ => issue("active", "invalid_value"),
        }
    }
    for (column, field) in [
        ("model", &mut input.model),
        ("purchased_on", &mut input.purchased_on),
        ("price", &mut input.price),
        ("currency", &mut input.currency),
        ("estimated_value", &mut input.estimated_value),
        ("estimated_currency", &mut input.estimated_currency),
        ("serial_number", &mut input.serial_number),
        ("barcode", &mut input.barcode),
        ("notes", &mut input.notes),
    ] {
        if let Some(text) = cell(column) {
            *field = Some(text).filter(|t| !t.is_empty());
        }
    }

    let checked: Result<DeviceFields, ApiError> = input.validate();
    if let Err(ApiError::Invalid { code, field }) = &checked {
        issue(field.unwrap_or("name"), code);
    }
    if let Some(device) = existing
        && device.copy_count > 0
        && !input.kind.holds_games()
    {
        issue("kind", "device_has_copies");
    }

    let row = HardwareRow {
        line,
        name,
        device_id: existing.map(|d| d.id),
        issues,
    };
    let planned = (row.issues.is_empty() && checked.is_ok()).then(|| Planned {
        device_id: existing.map(|d| d.id),
        input,
        new_store,
    });
    (row, planned)
}

pub fn preview(plan: Plan) -> HardwarePreview {
    HardwarePreview {
        file_issue: plan.file_issue.map(str::to_owned),
        rows: plan.rows.into_iter().map(|(row, _)| row).collect(),
    }
}

/// Writes every row, once none has an issue.
pub async fn apply(pool: &Pool, plan: Plan) -> ApiResult<HardwareResult> {
    if plan.file_issue.is_some() || plan.rows.iter().any(|(_, planned)| planned.is_none()) {
        return Err(ApiError::Invalid {
            code: "import_invalid",
            field: None,
        });
    }
    let mut result = HardwareResult {
        created: 0,
        updated: 0,
        new_stores: 0,
    };
    let mut added_stores: HashMap<String, i64> = HashMap::new();
    for (_, planned) in plan.rows {
        let Some(mut planned) = planned else { continue };
        if let Some(name) = planned.new_store.take() {
            let id = match added_stores.get(&key(&name)) {
                Some(id) => *id,
                None => {
                    let id = stores::create(
                        pool,
                        &StoreFields {
                            name: name.clone(),
                            slug: None,
                            notes: None,
                        },
                    )
                    .await?;
                    result.new_stores += 1;
                    added_stores.insert(key(&name), id);
                    id
                }
            };
            planned.input.store_id = Some(id);
        }
        let fields = planned.input.validate()?;
        match planned.device_id {
            Some(id) => {
                devices::update(pool, id, &fields).await?;
                result.updated += 1;
            }
            None => {
                devices::create(pool, &fields).await?;
                result.created += 1;
            }
        }
    }
    Ok(result)
}
