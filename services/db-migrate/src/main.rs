use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone)]
struct MigrationFile {
    version: u32,
    filename: String,
    path: PathBuf,
    checksum: String,
}

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    if let Err(err) = run().await {
        tracing::error!(error = %err, "migration failed");
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let database_url = match resolve_database_url() {
        Some(url) => url,
        None => {
            tracing::error!("DATABASE_URL or POSTGRES_USER/POSTGRES_PASSWORD must be set");
            std::process::exit(1);
        }
    };
    let core_dir =
        std::env::var("MIGRATIONS_CORE_DIR").unwrap_or_else(|_| "/migrations/core".to_string());
    let baseline_version: u32 = std::env::var("DB_MIGRATE_BASELINE_VERSION")
        .ok()
        .and_then(|value| value.trim().parse().ok())
        .unwrap_or(17);

    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await
        .context("connect to Postgres")?;

    migrate_postgres(&pool, Path::new(&core_dir), baseline_version).await?;

    let clickhouse_url = std::env::var("CLICKHOUSE_URL").unwrap_or_default();
    if !clickhouse_url.trim().is_empty() {
        let migrations_dir = std::env::var("MIGRATIONS_CLICKHOUSE_DIR")
            .unwrap_or_else(|_| "/migrations/clickhouse".to_string());
        let clickhouse_baseline: u32 = std::env::var("DB_MIGRATE_CLICKHOUSE_BASELINE_VERSION")
            .ok()
            .and_then(|value| value.trim().parse().ok())
            .unwrap_or(1);
        migrate_clickhouse(
            &clickhouse_url,
            Path::new(&migrations_dir),
            clickhouse_baseline,
        )
        .await?;
    } else {
        tracing::info!("CLICKHOUSE_URL empty; skipping ClickHouse migrations");
    }

    Ok(())
}

/// Builds postgres:// URL from POSTGRES_* vars (as provided by .env.db) when
/// DATABASE_URL is not set directly.
fn resolve_database_url() -> Option<String> {
    if let Ok(url) = std::env::var("DATABASE_URL") {
        if !url.trim().is_empty() {
            return Some(url);
        }
    }
    let user = std::env::var("POSTGRES_USER").ok()?;
    let password = std::env::var("POSTGRES_PASSWORD").ok()?;
    let db = std::env::var("POSTGRES_DB").unwrap_or_else(|_| "core".to_string());
    let host = std::env::var("POSTGRES_HOST").unwrap_or_else(|_| "postgres".to_string());
    let port = std::env::var("POSTGRES_PORT").unwrap_or_else(|_| "5432".to_string());
    Some(format!(
        "postgres://{user}:{}@{host}:{port}/{db}",
        percent_encode(&password)
    ))
}

fn percent_encode(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        match byte {
            b'%' | b'@' | b':' | b'/' | b'#' | b'?' => {
                encoded.push_str(&format!("%{byte:02X}"));
            }
            _ => encoded.push(byte as char),
        }
    }
    encoded
}

fn load_migrations(dir: &Path) -> Result<Vec<MigrationFile>> {
    let mut files = Vec::new();
    for entry in
        std::fs::read_dir(dir).with_context(|| format!("read migration dir {}", dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("sql") {
            continue;
        }
        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .to_string();
        let Some((version, _)) = parse_version(&filename) else {
            bail!("migration file {filename} does not match {{version}}_{{name}}.sql");
        };
        let content = std::fs::read_to_string(&path)?;
        let checksum = hex::encode(Sha256::digest(content.as_bytes()));
        files.push(MigrationFile {
            version,
            filename,
            path,
            checksum,
        });
    }

    files.sort_by(|a, b| a.version.cmp(&b.version).then(a.filename.cmp(&b.filename)));

    let mut seen = std::collections::HashSet::new();
    for file in &files {
        if !seen.insert(file.version) {
            bail!(
                "duplicate migration version {} ({})",
                file.version,
                file.filename
            );
        }
    }
    Ok(files)
}

fn parse_version(filename: &str) -> Option<(u32, &str)> {
    let (version, rest) = filename.split_once('_')?;
    let version = version.parse().ok()?;
    Some((version, rest))
}

async fn migrate_postgres(pool: &sqlx::PgPool, dir: &Path, baseline_version: u32) -> Result<()> {
    let files = load_migrations(dir)?;
    sqlx::query("CREATE SCHEMA IF NOT EXISTS platform")
        .execute(pool)
        .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS platform.schema_history (
            filename TEXT PRIMARY KEY,
            version INTEGER NOT NULL,
            checksum TEXT NOT NULL,
            applied_at TIMESTAMPTZ NOT NULL DEFAULT now()
        )",
    )
    .execute(pool)
    .await?;

    let history: Vec<(String, String)> =
        sqlx::query_as("SELECT filename, checksum FROM platform.schema_history")
            .fetch_all(pool)
            .await?;
    let recorded: std::collections::HashMap<String, String> = history.into_iter().collect();

    if recorded.is_empty() && schema_already_deployed(pool).await? {
        tracing::info!(
            baseline_version,
            "existing schema detected; baselining applied migrations"
        );
        for file in files.iter().filter(|f| f.version <= baseline_version) {
            sqlx::query(
                "INSERT INTO platform.schema_history (filename, version, checksum) VALUES ($1, $2, $3) ON CONFLICT (filename) DO NOTHING",
            )
            .bind(&file.filename)
            .bind(file.version as i32)
            .bind(&file.checksum)
            .execute(pool)
            .await?;
            tracing::info!(file = %file.filename, "baselined");
        }
    }

    for file in &files {
        match recorded.get(&file.filename) {
            Some(checksum) => {
                if checksum != &file.checksum {
                    bail!(
                        "migration {} was already applied with a different checksum; refusing to continue",
                        file.filename
                    );
                }
                tracing::debug!(file = %file.filename, "already applied");
            }
            None => {
                let content = std::fs::read_to_string(&file.path)?;
                let mut tx = pool.begin().await?;
                // raw_sql uses the simple protocol and supports multi-statement files.
                sqlx::raw_sql(&content).execute(&mut *tx).await?;
                sqlx::query(
                    "INSERT INTO platform.schema_history (filename, version, checksum) VALUES ($1, $2, $3)",
                )
                .bind(&file.filename)
                .bind(file.version as i32)
                .bind(&file.checksum)
                .execute(&mut *tx)
                .await?;
                tx.commit().await?;
                tracing::info!(file = %file.filename, "applied");
            }
        }
    }
    tracing::info!(count = files.len(), "postgres migrations up to date");
    Ok(())
}

async fn schema_already_deployed(pool: &sqlx::PgPool) -> Result<bool> {
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT table_schema FROM information_schema.tables WHERE table_name = 'forex_news_sources' AND table_schema = 'news' LIMIT 1",
    )
    .fetch_optional(pool)
    .await?;
    Ok(row.is_some())
}

struct ClickHouseMigrator {
    http: reqwest::Client,
    url: String,
    database: String,
    user: String,
    password: String,
}

impl ClickHouseMigrator {
    async fn execute(&self, sql: &str) -> Result<String> {
        let response = self
            .http
            .post(&self.url)
            .basic_auth(&self.user, Some(&self.password))
            .query(&[("database", self.database.as_str()), ("query", sql)])
            .send()
            .await?;
        let status = response.status();
        let text = response.text().await?;
        if !status.is_success() {
            bail!("clickhouse query failed with {status}: {text}");
        }
        Ok(text)
    }
}

async fn migrate_clickhouse(url: &str, dir: &Path, baseline_version: u32) -> Result<()> {
    let migrator = ClickHouseMigrator {
        http: reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()?,
        url: url.trim_end_matches('/').to_string() + "/",
        database: std::env::var("CLICKHOUSE_DATABASE").unwrap_or_else(|_| "market".to_string()),
        user: std::env::var("CLICKHOUSE_USER").unwrap_or_else(|_| "default".to_string()),
        password: std::env::var("CLICKHOUSE_PASSWORD").unwrap_or_default(),
    };

    let files = load_migrations(dir)?;
    migrator
        .execute(
            "CREATE TABLE IF NOT EXISTS schema_history (
                filename String,
                checksum String,
                applied_at DateTime DEFAULT now()
            ) ENGINE = MergeTree ORDER BY filename",
        )
        .await?;

    let rows = migrator
        .execute("SELECT filename, toString(checksum) AS checksum FROM schema_history FORMAT JSONEachRow")
        .await?;
    let mut recorded: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    for line in rows.lines().filter(|line| !line.trim().is_empty()) {
        let Ok(entry) = serde_json::from_str::<serde_json::Value>(line) else {
            bail!("invalid schema_history row from clickhouse: {line}");
        };
        let filename = entry
            .get("filename")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        let checksum = entry
            .get("checksum")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        recorded.insert(filename.to_string(), checksum.to_string());
    }

    if recorded.is_empty() {
        let ticks_exist = migrator.execute("EXISTS TABLE price_ticks").await?;
        if ticks_exist.trim() == "1" {
            tracing::info!(
                baseline_version,
                "existing clickhouse schema detected; baselining"
            );
            for file in files.iter().filter(|f| f.version <= baseline_version) {
                migrator
                    .execute(&format!(
                        "INSERT INTO schema_history (filename, checksum) VALUES ('{}', '{}')",
                        file.filename.replace('\'', "\\'"),
                        file.checksum
                    ))
                    .await?;
                tracing::info!(file = %file.filename, "baselined (clickhouse)");
            }
        }
    }

    for file in &files {
        if recorded.contains_key(&file.filename) {
            if recorded.get(&file.filename) != Some(&file.checksum) {
                bail!(
                    "clickhouse migration {} was applied with a different checksum",
                    file.filename
                );
            }
            continue;
        }
        let content = std::fs::read_to_string(&file.path)?;
        for statement in split_statements(&content) {
            migrator.execute(statement).await?;
        }
        migrator
            .execute(&format!(
                "INSERT INTO schema_history (filename, checksum) VALUES ('{}', '{}')",
                file.filename.replace('\'', "\\'"),
                file.checksum
            ))
            .await?;
        tracing::info!(file = %file.filename, "applied (clickhouse)");
    }
    tracing::info!(count = files.len(), "clickhouse migrations up to date");
    Ok(())
}

/// Splits a SQL file into statements on top-level semicolons, ignoring
/// semicolons inside single/double/backtick quotes and comments.
pub fn split_statements(sql: &str) -> Vec<&str> {
    let mut statements = Vec::new();
    let mut start = 0usize;
    let bytes = sql.as_bytes();
    let mut i = 0usize;

    while i < bytes.len() {
        let rest = &sql[i..];
        if rest.starts_with("--") {
            if let Some(newline) = rest.find('\n') {
                let only_ws_so_far = sql[start..i].trim().is_empty();
                i += newline + 1;
                if only_ws_so_far {
                    start = i;
                }
                continue;
            }
            break;
        }
        if rest.starts_with("/*") {
            if let Some(end) = rest.find("*/") {
                let only_ws_so_far = sql[start..i].trim().is_empty();
                i += end + 2;
                if only_ws_so_far {
                    start = i;
                }
                continue;
            }
            break;
        }
        let c = bytes[i];
        if c == b'\'' || c == b'"' || c == b'`' {
            let quote = c;
            i += 1;
            while i < bytes.len() {
                if bytes[i] == b'\\' && quote != b'`' {
                    i += 2;
                    continue;
                }
                if bytes[i] == quote {
                    // doubled quote = escaped, stay inside
                    if i + 1 < bytes.len() && bytes[i + 1] == quote {
                        i += 2;
                        continue;
                    }
                    break;
                }
                i += 1;
            }
            i += 1;
            continue;
        }
        if c == b';' {
            let statement = sql[start..i].trim();
            if !statement.is_empty() {
                statements.push(statement);
            }
            i += 1;
            start = i;
            continue;
        }
        i += 1;
    }

    let tail = sql[start..].trim();
    if !tail.is_empty() {
        statements.push(tail);
    }
    statements
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_statements_separates_top_level_semicolons() {
        let sql = "CREATE TABLE a (x Int32);\nCREATE TABLE b (y String);";
        assert_eq!(split_statements(sql).len(), 2);
    }

    #[test]
    fn split_statements_ignores_semicolons_in_strings_and_comments() {
        let sql = "-- a;b comment\nINSERT INTO t VALUES ('a;b'); /* c;d */ SELECT 1";
        let parts = split_statements(sql);
        assert_eq!(parts.len(), 2);
        assert!(parts[0].contains("'a;b'"));
        assert_eq!(parts[1], "SELECT 1");
    }

    #[test]
    fn split_statements_handles_datetime_defaults_with_quotes() {
        let sql =
            "CREATE TABLE t (ts DateTime DEFAULT now(), s String) ENGINE = MergeTree ORDER BY s;";
        let parts = split_statements(sql);
        assert_eq!(parts.len(), 1);
        assert!(parts[0].contains("ORDER BY s"));
    }

    #[test]
    fn split_statements_keeps_statement_without_trailing_semicolon() {
        assert_eq!(split_statements("SELECT 1").len(), 1);
    }
}
