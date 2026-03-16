pub mod row_mapping;

use std::sync::Arc;

use quiver_codegen::gen_sql::SqlGenerator;
use quiver_codegen::SqlDialect;
use quiver_driver_core::{Connection, DdlStatement, Pool, PoolConfig};
use quiver_driver_sqlite::SqlitePool;
use quiver_error::QuiverError;

/// Shared database connection pool type used by all services.
/// Uses a connection pool instead of a single Mutex-wrapped connection
/// to allow concurrent database access across service handlers.
pub type DbConn = Arc<SqlitePool>;

/// Initialize a quiver SQLite connection pool and run DDL to ensure tables exist.
pub async fn init_db(db_url: &str) -> Result<DbConn, QuiverError> {
    let pool = SqlitePool::new(PoolConfig::new(db_url, 4)).await?;

    let schema_src = include_str!("../../../schema.quiver");
    let schema = quiver_schema::parse(schema_src)
        .map_err(|e| QuiverError::Validation(format!("schema parse error: {e:?}")))?;
    quiver_schema::validate::validate(&schema)
        .map_err(|e| QuiverError::Validation(format!("schema validation error: {e:?}")))?;

    let ddl = SqlGenerator::generate(&schema, SqlDialect::Sqlite)
        .map_err(|e| QuiverError::Codegen(format!("DDL generation error: {e:?}")))?;

    // Run DDL on one connection from the pool to ensure tables exist.
    // For file-based SQLite, DDL is visible to all connections sharing the same file.
    let conn = pool.acquire().await?;
    conn.execute_ddl(&DdlStatement::new(ddl)).await?;
    drop(conn);

    Ok(Arc::new(pool))
}
