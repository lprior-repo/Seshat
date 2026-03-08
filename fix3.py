import re
import sys


def rewrite_export():
    with open("diagram_tool/src/models/export.rs", "r") as f:
        code = f.read()

    # 1. Main function signatures
    code = code.replace("use rusqlite::Connection;", "use sqlx::SqlitePool;")
    code = code.replace(
        "pub fn export_diagram_json(conn: &Connection)",
        "pub async fn export_diagram_json(pool: &SqlitePool)",
    )
    code = code.replace(
        "pub fn import_diagram_json(\n    conn: &mut Connection,",
        "pub async fn import_diagram_json(\n    pool: &SqlitePool,",
    )
    code = code.replace(
        "fn fetch_all_events(conn: &Connection)",
        "async fn fetch_all_events(pool: &SqlitePool)",
    )
    code = code.replace(
        "pub fn export_while_recovering(conn: &rusqlite::Connection)",
        "pub async fn export_while_recovering(pool: &SqlitePool)",
    )

    # Replace usages of `conn` with `pool` in these functions
    code = code.replace("fetch_all_events(conn)?", "fetch_all_events(pool).await?")
    code = code.replace(
        "store::append_idempotent(conn, envelope)",
        "store::append_idempotent(pool, envelope).await",
    )
    code = code.replace(
        "store::fetch_latest_revision(conn).map_err",
        "store::fetch_latest_revision(pool).await.map_err",
    )

    # fix fetch_all_events implementation (same as before)
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

    # Tests module
    code = code.replace("#[test]", "#[tokio::test]")
    code = re.sub(r"fn given_([a-zA-Z0-9_]+)\(\) \{", r"async fn given_\1() {", code)

    code = code.replace(
        "bootstrap_store(&db_path).unwrap()", "bootstrap_store(&db_path).await.unwrap()"
    )
    code = code.replace(
        "bootstrap_store(&db_path2).unwrap()",
        "bootstrap_store(&db_path2).await.unwrap()",
    )
    code = code.replace(
        "open_recovery_mode(&db_path).unwrap()",
        "open_recovery_mode(&db_path).await.unwrap()",
    )

    code = code.replace(".conn", ".pool")

    code = re.sub(
        r"append_event\(&mut bootstrap\.pool, (.*?), None\)\.unwrap\(\);",
        r"append_event(&bootstrap.pool, \1, None).await.unwrap();",
        code,
    )
    code = re.sub(
        r"append_event\(&mut conn, (.*?), None\)\.unwrap\(\);",
        r"append_event(&conn, \1, None).await.unwrap();",
        code,
    )
    code = re.sub(
        r"append_event\(&mut conn2, (.*?), None\)\.unwrap\(\);",
        r"append_event(&conn2, \1, None).await.unwrap();",
        code,
    )

    code = code.replace("export_diagram_json(conn)", "export_diagram_json(conn).await")
    code = code.replace(
        "export_diagram_json(&bootstrap.pool)",
        "export_diagram_json(&bootstrap.pool).await",
    )
    code = code.replace(
        "export_while_recovering(&handle.pool)",
        "export_while_recovering(&handle.pool).await",
    )
    code = code.replace("import_diagram_json(&mut conn,", "import_diagram_json(&conn,")
    code = code.replace(
        "import_diagram_json(&mut conn2,", "import_diagram_json(&conn2,"
    )

    code = code.replace(
        "let mut conn = bootstrap.pool;", "let conn = bootstrap.pool.clone();"
    )
    code = code.replace(
        "let mut conn2 = bootstrap2.pool;", "let conn2 = bootstrap2.pool.clone();"
    )

    code = re.sub(
        r"let result = export_diagram_json\((.*?)\);",
        r"let result = export_diagram_json(\1).await;",
        code,
    )
    code = re.sub(
        r"let result = import_diagram_json\((.*?)\);",
        r"let result = import_diagram_json(\1).await;",
        code,
    )
    code = re.sub(
        r"let result = export_while_recovering\((.*?)\);",
        r"let result = export_while_recovering(\1).await;",
        code,
    )

    with open("diagram_tool/src/models/export.rs", "w") as f:
        f.write(code)


def rewrite_harness():
    with open("diagram_tool/src/models/harness.rs", "r") as f:
        code = f.read()

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

    code = re.sub(
        r"fn test_([a-zA-Z0-9_]+)\(_db_path: &Path\) -> Result<TestReport, VerifyError> \{",
        r"async fn test_\1(_db_path: &Path) -> Result<TestReport, VerifyError> {",
        code,
    )
    code = re.sub(
        r"fn test_([a-zA-Z0-9_]+)\(rng: &mut SeededRng\) -> Result<TestReport, VerifyError> \{",
        r"async fn test_\1(rng: &mut SeededRng) -> Result<TestReport, VerifyError> {",
        code,
    )
    code = re.sub(
        r"fn test_([a-zA-Z0-9_]+)\(\) -> Result<TestReport, VerifyError> \{",
        r"async fn test_\1() -> Result<TestReport, VerifyError> {",
        code,
    )

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
        r"fetch_latest_revision\(&(.*?)\)\?", r"fetch_latest_revision(&\1).await?", code
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

    code = code.replace(".conn", ".pool")

    code = code.replace(
        "startup_integrity_check(&test_db_path)",
        "startup_integrity_check(&test_db_path).await",
    )
    code = code.replace(
        "crate::models::snapshot::load_projection(&recovery_bootstrap.pool)",
        "crate::models::snapshot::load_projection(&recovery_bootstrap.pool).await",
    )

    code = code.replace(
        "fn verify_all_events_applied(conn: &sqlx::SqlitePool, expected_count: usize) -> Result<(), VerifyError> {",
        "async fn verify_all_events_applied(conn: &sqlx::SqlitePool, expected_count: usize) -> Result<(), VerifyError> {",
    )
    v_old = """    let count: i64 = conn.query_row("SELECT COUNT(*) FROM events", [], |row| row.get(0))
        .map_err(|e| VerifyError::Sqlite(e.to_string()))?;"""
    v_new = """    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM events")
        .fetch_one(conn)
        .await
        .map_err(|e| VerifyError::Sqlite(e.to_string()))?;"""
    code = code.replace(v_old, v_new)

    q_old1 = """    let op_id = conn
        .query_row(
            "SELECT operation_id FROM events ORDER BY revision DESC LIMIT 1",
            [],
            |row| row.get(0),
        )
        .map_err(|e| VerifyError::Sqlite(e.to_string()))?;"""
    q_new1 = """    let op_id: String = sqlx::query_scalar("SELECT operation_id FROM events ORDER BY revision DESC LIMIT 1")
        .fetch_one(conn)
        .await
        .map_err(|e| VerifyError::Sqlite(e.to_string()))?;"""
    code = code.replace(q_old1, q_new1)

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

    q_old4 = """    let events_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM events", [], |row| row.get(0))
        .map_err(|e| VerifyError::Sqlite(e.to_string()))?;"""
    q_new4 = """    let events_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM events")
        .fetch_one(conn)
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

    code = code.replace(
        "StoreError::Sqlite(e) => Self::Sqlite(e.to_string()),",
        "StoreError::Sqlite(e) => Self::Sqlite(e),",
    )

    with open("diagram_tool/src/models/harness.rs", "w") as f:
        f.write(code)


rewrite_export()
rewrite_harness()
