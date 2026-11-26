use serde::Serialize;
use sqlx::types::Json;
use sqlx::{SqliteConnection, SqlitePool};
use utoipa::ToSchema;

use super::found;
use super::slugs::{self, Table};
use super::users::Person;
use crate::error::{ApiError, ApiResult};

/// A store account that owns digital copies, such as a Nintendo Account.
#[derive(Debug, Serialize, ToSchema)]
pub struct Account {
    pub id: i64,
    pub name: String,
    /// Names the account in library links.
    pub slug: String,
    /// The store it buys from.
    pub store_id: Option<i64>,
    /// Whose account it is.
    pub person: Option<Person>,
    pub notes: Option<String>,
    pub active: bool,
    /// Platforms whose games it holds, in platform list order.
    pub platform_ids: Vec<i64>,
    /// Copies the account owns, of any status.
    pub copy_count: i64,
}

pub struct AccountFields {
    pub name: String,
    /// `None` makes one from the name for a new account and keeps it on update.
    pub slug: Option<String>,
    pub store_id: Option<i64>,
    pub user_id: Option<String>,
    pub notes: Option<String>,
    pub active: bool,
    pub platform_ids: Vec<i64>,
}

struct AccountRow {
    id: i64,
    name: String,
    slug: String,
    store_id: Option<i64>,
    user_id: Option<String>,
    person_name: Option<String>,
    notes: Option<String>,
    active: bool,
    platform_ids: Json<Vec<i64>>,
    copy_count: i64,
}

impl From<AccountRow> for Account {
    fn from(row: AccountRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            slug: row.slug,
            store_id: row.store_id,
            person: Person::from_columns(row.user_id, row.person_name),
            notes: row.notes,
            active: row.active,
            platform_ids: row.platform_ids.0,
            copy_count: row.copy_count,
        }
    }
}

pub async fn list(pool: &SqlitePool) -> sqlx::Result<Vec<Account>> {
    let rows = sqlx::query_as!(
        AccountRow,
        r#"SELECT a.id AS "id!", a.name, a.slug, a.store_id, a.user_id,
                  COALESCE(u.display_name, u.name) AS person_name, a.notes,
                  a.active AS "active: bool",
                  (SELECT json_group_array(platform_id) FROM (
                       SELECT ap.platform_id FROM account_platforms ap
                       JOIN platforms p ON p.id = ap.platform_id
                       WHERE ap.account_id = a.id
                       ORDER BY p.sort_order, p.name)) AS "platform_ids!: Json<Vec<i64>>",
                  (SELECT COUNT(*) FROM copies c WHERE c.account_id = a.id) AS "copy_count!: i64"
           FROM accounts a LEFT JOIN users u ON u.id = a.user_id
           ORDER BY a.name COLLATE NOCASE"#
    )
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(Account::from).collect())
}

pub async fn get(pool: &SqlitePool, id: i64) -> ApiResult<Account> {
    sqlx::query_as!(
        AccountRow,
        r#"SELECT a.id AS "id!", a.name, a.slug, a.store_id, a.user_id,
                  COALESCE(u.display_name, u.name) AS person_name, a.notes,
                  a.active AS "active: bool",
                  (SELECT json_group_array(platform_id) FROM (
                       SELECT ap.platform_id FROM account_platforms ap
                       JOIN platforms p ON p.id = ap.platform_id
                       WHERE ap.account_id = a.id
                       ORDER BY p.sort_order, p.name)) AS "platform_ids!: Json<Vec<i64>>",
                  (SELECT COUNT(*) FROM copies c WHERE c.account_id = a.id) AS "copy_count!: i64"
           FROM accounts a LEFT JOIN users u ON u.id = a.user_id
           WHERE a.id = ?"#,
        id
    )
    .fetch_optional(pool)
    .await?
    .map(Account::from)
    .ok_or(ApiError::NotFound)
}

pub async fn create(pool: &SqlitePool, fields: &AccountFields) -> ApiResult<i64> {
    let mut tx = crate::db::write_tx(pool).await?;
    let platform_ids = checked(&mut tx, fields).await?;
    slugs::ensure_name_free(&mut tx, Table::Accounts, &fields.name, None).await?;
    let slug = slugs::for_new(
        &mut tx,
        Table::Accounts,
        fields.slug.as_deref(),
        &fields.name,
    )
    .await?;
    let id = sqlx::query_scalar!(
        r#"INSERT INTO accounts (name, slug, store_id, user_id, notes, active)
           VALUES (?, ?, ?, ?, ?, ?)
           RETURNING id AS "id!""#,
        fields.name,
        slug,
        fields.store_id,
        fields.user_id,
        fields.notes,
        fields.active
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(slugs::conflict)?;
    set_platforms(&mut tx, id, &platform_ids).await?;
    tx.commit().await?;
    Ok(id)
}

pub async fn update(pool: &SqlitePool, id: i64, fields: &AccountFields) -> ApiResult<()> {
    let mut tx = crate::db::write_tx(pool).await?;
    slugs::ensure_name_free(&mut tx, Table::Accounts, &fields.name, Some(id)).await?;
    let platform_ids = checked(&mut tx, fields).await?;
    let result = sqlx::query!(
        "UPDATE accounts
         SET name = ?, slug = COALESCE(?, slug), store_id = ?, user_id = ?, notes = ?,
             active = ?, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE id = ?",
        fields.name,
        fields.slug,
        fields.store_id,
        fields.user_id,
        fields.notes,
        fields.active,
        id
    )
    .execute(&mut *tx)
    .await
    .map_err(slugs::conflict)?;
    found(result.rows_affected())?;
    sqlx::query!("DELETE FROM account_platforms WHERE account_id = ?", id)
        .execute(&mut *tx)
        .await?;
    set_platforms(&mut tx, id, &platform_ids).await?;
    tx.commit().await?;
    Ok(())
}

/// Copies on the account keep existing with no account.
pub async fn delete(pool: &SqlitePool, id: i64) -> ApiResult<()> {
    let result = sqlx::query!("DELETE FROM accounts WHERE id = ?", id)
        .execute(pool)
        .await?;
    found(result.rows_affected())
}

/// Checks the store, the person and the platforms exist.
/// Returns the platform ids as a JSON array.
async fn checked(conn: &mut SqliteConnection, fields: &AccountFields) -> ApiResult<String> {
    super::stores::check(conn, fields.store_id).await?;
    if let Some(user_id) = &fields.user_id {
        let known: bool = sqlx::query_scalar!(
            r#"SELECT EXISTS (SELECT 1 FROM users WHERE id = ?) AS "known!: bool""#,
            user_id
        )
        .fetch_one(&mut *conn)
        .await?;
        if !known {
            return Err(ApiError::invalid("unknown_user", "user_id"));
        }
    }
    super::platforms::checked_ids(conn, &fields.platform_ids).await
}

async fn set_platforms(
    conn: &mut SqliteConnection,
    account_id: i64,
    json_ids: &str,
) -> ApiResult<()> {
    sqlx::query!(
        "INSERT INTO account_platforms (account_id, platform_id)
         SELECT ?, value FROM json_each(?)",
        account_id,
        json_ids
    )
    .execute(&mut *conn)
    .await?;
    Ok(())
}
