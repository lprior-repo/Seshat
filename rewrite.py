import re

def rewrite_export():
    with open("diagram_tool/src/models/export.rs", "r") as f:
        code = f.read()

    # 1. Imports
    code = code.replace("use rusqlite::Connection;", "")
    code = code.replace("conn: &Connection", "pool: &sqlx::SqlitePool")
    code = code.replace("conn: &mut Connection", "pool: &sqlx::SqlitePool")
    code = code.replace("conn: &rusqlite::Connection", "pool: &sqlx::SqlitePool")
    code = code.replace("pub fn export_diagram_json", "pub async fn export_diagram_json")
    code = code.replace("pub fn import_diagram_json", "pub async fn import_diagram_json")
    code = code.replace("fn fetch_all_events", "async fn fetch_all_events")
    code = code.replace("pub fn export_while_recovering", "pub async fn export_while_recovering")

    # 2. fetch_all_events
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

    # 3. export_diagram_json
    code = code.replace("fetch_all_events(pool)?", "fetch_all_events(pool).await?")

    # 4. import_diagram_json calls
    code = code.replace("store::append_idempotent(pool, envelope)", "store::append_idempotent(pool, envelope).await")
    code = code.replace("store::fetch_latest_revision(pool).map_err(|e| ExportError::Sqlite(e.to_string()))? as u64", "store::fetch_latest_revision(pool).await.map_err(|e| ExportError::Sqlite(e.to_string()))? as u64")
    code = code.replace("export_while_recovering(&handle.pool)", "export_while_recovering(&handle.pool).await")
    code = code.replace("export_diagram_json(pool)", "export_diagram_json(pool).await")

    # Tests module
    code = code.replace("#[test]", "#[tokio::test]")
    code = re.sub(r'fn given_([a-zA-Z0-9_]+)\(\) \{', r'async fn given_\1() {', code)
    code = re.sub(r'bootstrap_store\((.*?)\)\.unwrap\(\)', r'bootstrap_store(\1).await.unwrap()', code)
    code = re.sub(r'bootstrap_store\((.*?)\)\.expect\(', r'bootstrap_store(\1).await.expect(', code)
    code = re.sub(r'bootstrap_store\((.*?)\)\?', r'bootstrap_store(\1).await?', code)

    code = re.sub(r'append_event\((.*?), (.*?), (.*?)\)\.unwrap\(\)', r'append_event(\1, \2, \3).await.unwrap()', code)
    code = re.sub(r'append_event\((.*?), (.*?), (.*?)\)\?', r'append_event(\1, \2, \3).await?', code)

    code = code.replace('&bootstrap.conn', '&bootstrap.pool')
    code = code.replace('&mut bootstrap.conn', '&bootstrap.pool')
    code = code.replace('&mut conn', '&conn')
    code = code.replace('let mut conn = bootstrap.conn;', 'let conn = bootstrap.pool;')
    code = code.replace('let mut conn2 = bootstrap2.conn;', 'let conn2 = bootstrap2.pool;')

    code = re.sub(r'export_diagram_json\((.*?)\)\.unwrap\(\)', r'export_diagram_json(\1).await.unwrap()', code)
    code = re.sub(r'import_diagram_json\((.*?)\)\.unwrap\(\)', r'import_diagram_json(\1).await.unwrap()', code)
    code = re.sub(r'export_diagram_json\((.*?)\);', r'export_diagram_json(\1).await;', code)
    code = re.sub(r'import_diagram_json\((.*?)\);', r'import_diagram_json(\1).await;', code)

    code = re.sub(r'open_recovery_mode\((.*?)\)\.unwrap\(\)', r'open_recovery_mode(\1).await.unwrap()', code)
    code = code.replace('handle.conn', 'handle.pool')

    code = re.sub(r'result\.is_ok\(\)', r'result.await.is_ok()', code)
    code = re.sub(r'result\.is_err\(\)', r'result.await.is_err()', code)
    code = re.sub(r'result\.err\(\)', r'result.await.err()', code)
    code = re.sub(r'result\.unwrap\(\)', r'result.await.unwrap()', code)
    code = re.sub(r'result\.unwrap_err\(\)', r'result.await.unwrap_err()', code)

    # Some variables like `export` from export_diagram_json might be `.unwrap().await` if not matched.
    with open("diagram_tool/src/models/export.rs", "w") as f:
        f.write(code)

rewrite_export()
