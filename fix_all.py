import re
import sys


def rewrite_export():
    with open("diagram_tool/src/models/export.rs", "r") as f:
        code = f.read()

    # 1. Imports
    code = code.replace("use rusqlite::Connection;", "")
    code = code.replace("conn: &Connection", "pool: &sqlx::SqlitePool")
    code = code.replace("conn: &mut Connection", "pool: &sqlx::SqlitePool")
    code = code.replace("conn: &rusqlite::Connection", "pool: &sqlx::SqlitePool")
    code = code.replace(
        "pub fn export_diagram_json", "pub async fn export_diagram_json"
    )
    code = code.replace(
        "pub fn import_diagram_json", "pub async fn import_diagram_json"
    )
    code = code.replace("fn fetch_all_events", "async fn fetch_all_events")
    code = code.replace(
        "pub fn export_while_recovering", "pub async fn export_while_recovering"
    )

    # 2. fetch_all_events body
    fetch_old = """    let mut stmt = conn
        .prepare("SELECT operation_id, revision, payload, timestamp FROM events ORDER BY revision ASC")
        .map_err(|e| ExportError::Sqlite(e.to_string()))?;

    let row_results: Vec<Result<(String, i64, String, String), rusqlite::Error>> = stmt
        .query_map([], |row| {
            let operation_id: String = row.get(0)?;
            let revision: i64 = row.get(1)?;
            let payload: String = row.get(2)?;
            let timestamp: String = row.get(3)?;
            Ok((operation_id, revision, payload, timestamp))
        })
        .map_err(|e| ExportError::Sqlite(e.to_string()))?
        .collect();

    let mut decode_errors = Vec::new();
    let events: Vec<EventRecord> = row_results
        .into_iter()
        .filter_map(|result| match result {
            Ok((operation_id, revision, payload, timestamp)) => {
                match parse_event_envelope(&payload) {
                    Ok(envelope) => match timestamp.parse::<i64>() {
                        Ok(timestamp) => Some(Ok(EventRecord {
                            op_id: envelope.op_id,
                            revision: revision as u64,
                            operation: envelope.operation,
                            author: envelope.author,
                            timestamp,
                        })),
                        Err(e) => {
                            decode_errors.push(format!(
                                "timestamp parse error for op {}: {}",
                                operation_id, e
                            ));
                            None
                        }
                    },
                    Err(e) => {
                        decode_errors.push(format!(
                            "envelope parse error for op {}: {}",
                            operation_id, e
                        ));
                        None
                    }
                }
            }
            Err(e) => {
                decode_errors.push(format!("row error: {}", e));
                None
            }
        })
        .collect::<Result<Vec<_>, ExportError>>()
        .map_err(|e| ExportError::Sqlite(e.to_string()))?;"""

    fetch_new = """    let row_results: Vec<(String, i64, String, i64)> = sqlx::query_as(
        "SELECT operation_id, revision, payload, timestamp FROM events ORDER BY revision ASC"
    )
    .fetch_all(pool)
    .await
    .map_err(|e| ExportError::Sqlite(e.to_string()))?;

    let mut decode_errors = Vec::new();
    let events: Vec<EventRecord> = row_results
        .into_iter()
        .filter_map(|row| {
            let operation_id = row.0;
            let revision = row.1;
            let payload = row.2;
            let timestamp = row.3;
            
            match parse_event_envelope(&payload) {
                Ok(envelope) => Some(EventRecord {
                    op_id: envelope.op_id,
                    revision: revision as u64,
                    operation: envelope.operation,
                    author: envelope.author,
                    timestamp,
                }),
                Err(e) => {
                    decode_errors.push(format!(
                        "envelope parse error for op {}: {}",
                        operation_id, e
                    ));
                    None
                }
            }
        })
        .collect();"""

    code = code.replace(fetch_old, fetch_new)

    # 3. await additions inside the main code (non-test)
    code = code.replace("fetch_all_events(pool)?", "fetch_all_events(pool).await?")
    code = code.replace(
        "store::append_idempotent(pool, envelope)",
        "store::append_idempotent(pool, envelope).await",
    )
    code = code.replace(
        "store::fetch_latest_revision(pool).map_err(|e| ExportError::Sqlite(e.to_string()))? as u64",
        "store::fetch_latest_revision(pool).await.map_err(|e| ExportError::Sqlite(e.to_string()))? as u64",
    )
    code = code.replace(
        "export_while_recovering(&handle.pool)",
        "export_while_recovering(&handle.pool).await",
    )
    code = code.replace("export_diagram_json(pool)", "export_diagram_json(pool).await")

    # Tests module
    code = code.replace("#[test]", "#[tokio::test]")
    code = re.sub(r"fn given_([a-zA-Z0-9_]+)\(\) \{", r"async fn given_\1() {", code)
    code = re.sub(
        r"bootstrap_store\((.*?)\)\.unwrap\(\)",
        r"bootstrap_store(\1).await.unwrap()",
        code,
    )
    code = re.sub(
        r"bootstrap_store\((.*?)\)\.expect\(",
        r"bootstrap_store(\1).await.expect(",
        code,
    )
    code = re.sub(r"bootstrap_store\((.*?)\)\?", r"bootstrap_store(\1).await?", code)

    code = re.sub(
        r"append_event\((.*?), (.*?), (.*?)\)\.unwrap\(\)",
        r"append_event(\1, \2, \3).await.unwrap()",
        code,
    )
    code = re.sub(
        r"append_event\((.*?), (.*?), (.*?)\)\?",
        r"append_event(\1, \2, \3).await?",
        code,
    )

    code = code.replace("&bootstrap.conn", "&bootstrap.pool")
    code = code.replace("&mut bootstrap.conn", "&bootstrap.pool")
    code = code.replace("&mut conn", "&conn")
    code = code.replace("let mut conn = bootstrap.conn;", "let conn = bootstrap.pool;")
    code = code.replace("let mut conn = bootstrap.pool;", "let conn = bootstrap.pool;")
    code = code.replace(
        "let mut conn2 = bootstrap2.conn;", "let conn2 = bootstrap2.pool;"
    )

    code = re.sub(
        r"export_diagram_json\((.*?)\)\.unwrap\(\)",
        r"export_diagram_json(\1).await.unwrap()",
        code,
    )
    code = re.sub(
        r"import_diagram_json\((.*?)\)\.unwrap\(\)",
        r"import_diagram_json(\1).await.unwrap()",
        code,
    )
    code = re.sub(
        r"export_diagram_json\((.*?)\);", r"export_diagram_json(\1).await;", code
    )
    code = re.sub(
        r"import_diagram_json\((.*?)\);", r"import_diagram_json(\1).await;", code
    )

    code = re.sub(
        r"open_recovery_mode\((.*?)\)\.unwrap\(\)",
        r"open_recovery_mode(\1).await.unwrap()",
        code,
    )
    code = code.replace("handle.conn", "handle.pool")

    # Fix test asserts that were failing due to missing .await on Futures
    code = re.sub(r"result\.is_ok\(\)", r"result.await.is_ok()", code)
    code = re.sub(r"result\.is_err\(\)", r"result.await.is_err()", code)
    code = re.sub(r"result\.err\(\)", r"result.await.err()", code)
    code = re.sub(r"result\.unwrap\(\)", r"result.await.unwrap()", code)
    code = re.sub(r"result\.unwrap_err\(\)", r"result.await.unwrap_err()", code)

    with open("diagram_tool/src/models/export.rs", "w") as f:
        f.write(code)


def rewrite_harness():
    with open("diagram_tool/src/models/harness.rs", "r") as f:
        code = f.read()

    # Imports and traits
    # Find `impl From<rusqlite::Error> for VerifyError`
    from_old = """impl From<rusqlite::Error> for VerifyError {
    fn from(err: rusqlite::Error) -> Self {
        VerifyError::Sqlite(err.to_string())
    }
}"""
    code = code.replace(from_old, "")

    code = code.replace(
        "rusqlite::Error::InvalidQuery",
        'StoreError::Sqlite("InvalidQuery".to_string())',
    )
    code = code.replace("rusqlite::Connection", "sqlx::SqlitePool")
    code = code.replace("conn: &sqlx::SqlitePool", "pool: &sqlx::SqlitePool")

    # fetch_latest_revision inside harness.rs if it exists? Wait, it imports it.
    code = code.replace(
        "fetch_latest_revision(&bootstrap.conn)",
        "fetch_latest_revision(&bootstrap.pool).await",
    )
    code = code.replace(
        "fetch_latest_revision(&recovery_bootstrap.conn)",
        "fetch_latest_revision(&recovery_bootstrap.pool).await",
    )
    code = code.replace(
        "fetch_latest_revision(&recovery_bootstrap.pool)",
        "fetch_latest_revision(&recovery_bootstrap.pool).await",
    )

    # Make run_ functions async
    code = code.replace(
        "pub fn run_replay_determinism_suite",
        "pub async fn run_replay_determinism_suite",
    )
    code = code.replace(
        "pub fn run_human_ai_conflict_e2e", "pub async fn run_human_ai_conflict_e2e"
    )
    code = code.replace(
        "pub fn run_crash_recovery_suite", "pub async fn run_crash_recovery_suite"
    )

    # Internal test helper functions need async
    code = re.sub(
        r"fn test_([a-zA-Z0-9_]+)\(_db_path", r"async fn test_\1(_db_path", code
    )
    code = re.sub(
        r"fn test_([a-zA-Z0-9_]+)\(\&mut rng", r"async fn test_\1(&mut rng", code
    )
    code = re.sub(
        r"fn test_([a-zA-Z0-9_]+)\(\) -> Result", r"async fn test_\1() -> Result", code
    )

    # In the helper functions, `.await` all store calls
    code = re.sub(r"bootstrap_store\(&(.*?)\)\?", r"bootstrap_store(&\1).await?", code)
    code = re.sub(
        r"append_event\(&mut bootstrap\.conn, (.*?)\)\?",
        r"append_event(&bootstrap.pool, \1).await?",
        code,
    )
    code = re.sub(
        r"append_event\(&bootstrap\.pool, (.*?)\)\?",
        r"append_event(&bootstrap.pool, \1).await?",
        code,
    )

    # In run_ suites, `.await` the helper functions
    code = re.sub(
        r"let case_report = test_(.*?)\(\&mut rng\)\?;",
        r"let case_report = test_\1(&mut rng).await?;",
        code,
    )
    code = re.sub(
        r"let case_report = test_(.*?)\(\)\?;",
        r"let case_report = test_\1().await?;",
        code,
    )

    # Now fix tests module
    code = code.replace("#[test]", "#[tokio::test]")
    code = re.sub(r"fn test_([a-zA-Z0-9_]+)\(\) \{", r"async fn test_\1() {", code)

    code = re.sub(
        r"bootstrap_store\((.*?)\)\.expect\(",
        r"bootstrap_store(\1).await.expect(",
        code,
    )
    code = re.sub(
        r"append_event\(&mut bootstrap\.conn, (.*?)\)\.expect\(",
        r"append_event(&bootstrap.pool, \1).await.expect(",
        code,
    )
    code = re.sub(
        r"append_event\(&bootstrap\.pool, (.*?)\)\.expect\(",
        r"append_event(&bootstrap.pool, \1).await.expect(",
        code,
    )

    code = re.sub(
        r"fetch_latest_revision\(&(.*?)\)\.expect\(",
        r"fetch_latest_revision(&\1).await.expect(",
        code,
    )

    code = re.sub(
        r"run_replay_determinism_suite\((.*?)\)",
        r"run_replay_determinism_suite(\1).await",
        code,
    )
    code = re.sub(
        r"run_human_ai_conflict_e2e\(\)", r"run_human_ai_conflict_e2e().await", code
    )
    code = re.sub(
        r"run_crash_recovery_suite\(\)", r"run_crash_recovery_suite().await", code
    )
    code = re.sub(r"super::test_(.*?)\(\)", r"super::test_\1().await", code)

    code = code.replace("&bootstrap.conn", "&bootstrap.pool")
    code = code.replace("&mut bootstrap.conn", "&bootstrap.pool")
    code = code.replace(
        "let mut bootstrap = bootstrap_store", "let bootstrap = bootstrap_store"
    )

    code = code.replace(
        "startup_integrity_check(&test_db_path)",
        "startup_integrity_check(&test_db_path).await",
    )
    code = code.replace(
        "crate::models::snapshot::load_projection(&recovery_bootstrap.pool)",
        "crate::models::snapshot::load_projection(&recovery_bootstrap.pool).await",
    )

    # verify_all_events_applied
    code = code.replace(
        "fn verify_all_events_applied(pool: &sqlx::SqlitePool, expected_count: usize) -> Result<(), VerifyError> {",
        "async fn verify_all_events_applied(pool: &sqlx::SqlitePool, expected_count: usize) -> Result<(), VerifyError> {",
    )
    v_old = """    let count: i64 = conn.query_row("SELECT COUNT(*) FROM events", [], |row| row.get(0))
        .map_err(|e| VerifyError::Sqlite(e.to_string()))?;"""
    v_new = """    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM events")
        .fetch_one(pool)
        .await
        .map_err(|e| VerifyError::Sqlite(e.to_string()))?;"""
    code = code.replace(v_old, v_new)

    v_old2 = """    let count: i64 = pool.query_row("SELECT COUNT(*) FROM events", [], |row| row.get(0))
        .map_err(|e| VerifyError::Sqlite(e.to_string()))?;"""
    code = code.replace(v_old2, v_new)

    q_old1 = """    let op_id = pool
        .query_row(
            "SELECT operation_id FROM events ORDER BY revision DESC LIMIT 1",
            [],
            |row| row.get(0),
        )
        .map_err(|e| VerifyError::Sqlite(e.to_string()))?;"""
    q_new1 = """    let op_id: String = sqlx::query_scalar("SELECT operation_id FROM events ORDER BY revision DESC LIMIT 1")
        .fetch_one(pool)
        .await
        .map_err(|e| VerifyError::Sqlite(e.to_string()))?;"""
    code = code.replace(q_old1, q_new1)

    q_old2 = """    let op_id: String = pool
        .query_row(
            "SELECT operation_id FROM events ORDER BY revision DESC LIMIT 1",
            [],
            |row| row.get(0),
        )
        .map_err(|e| VerifyError::Sqlite(e.to_string()))?;"""
    code = code.replace(q_old2, q_new1)

    q_old3 = """        let count: i64 = bootstrap.pool.query_row(
            "SELECT COUNT(*) FROM events WHERE operation_id = ?1",
            rusqlite::params![envelope2.op_id],
            |row| row.get(0),
        ).expect("Failed to query");"""
    q_new3 = """        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM events WHERE operation_id = ?")
            .bind(&envelope2.op_id)
            .fetch_one(&bootstrap.pool)
            .await
            .expect("Failed to query");"""
    code = code.replace(q_old3, q_new3)

    q_old4 = """    let events_count: i64 = pool
        .query_row("SELECT COUNT(*) FROM events", [], |row| row.get(0))
        .map_err(|e| VerifyError::Sqlite(e.to_string()))?;"""
    q_new4 = """    let events_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM events")
        .fetch_one(pool)
        .await
        .map_err(|e| VerifyError::Sqlite(e.to_string()))?;"""
    code = code.replace(q_old4, q_new4)

    code = code.replace(
        "verify_all_events_applied(&bootstrap.pool, expected_count)?",
        "verify_all_events_applied(&bootstrap.pool, expected_count).await?",
    )
    code = code.replace(
        "verify_all_events_applied(&recovery_bootstrap.pool, expected_count)?",
        "verify_all_events_applied(&recovery_bootstrap.pool, expected_count).await?",
    )

    # Also fix some missed expected rusqlite::Error
    code = code.replace(
        "StoreError::Sqlite(e) => Self::Sqlite(e.to_string()),",
        "VerifyError::Sqlite(e) => Self::Sqlite(e),",
    )

    with open("diagram_tool/src/models/harness.rs", "w") as f:
        f.write(code)


rewrite_export()
rewrite_harness()
