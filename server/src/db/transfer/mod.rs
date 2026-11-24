//! The CSV import's reads and writes, and the export's rows.

mod apply;
pub mod export;
pub mod snapshot;

use sqlx::SqlitePool;

use super::write_tx;
use crate::error::{ApiError, ApiResult};
use crate::transfer::plan::Plan;
use snapshot::Snapshot;

/// The collection as an import by `person` sees it.
pub async fn snapshot(pool: &SqlitePool, person: &str) -> sqlx::Result<Snapshot> {
    let mut conn = pool.acquire().await?;
    snapshot::load(&mut conn, person).await
}

/// Plans an import against the collection and applies the plan in one
/// transaction. `import_invalid` when the plan has errors.
pub async fn import(
    pool: &SqlitePool,
    by: &str,
    make_plan: impl FnOnce(&Snapshot) -> Plan,
) -> ApiResult<Plan> {
    let mut tx = write_tx(pool).await?;
    let snapshot = snapshot::load(&mut tx, by).await?;
    let plan = make_plan(&snapshot);
    if plan.has_errors() {
        return Err(ApiError::Invalid {
            code: "import_invalid",
            field: None,
        });
    }
    apply::apply(&mut tx, &plan, by).await?;
    tx.commit().await?;
    Ok(plan)
}
