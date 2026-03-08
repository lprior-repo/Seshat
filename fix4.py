import re


def fix():
    with open("diagram_tool/src/models/harness.rs", "r") as f:
        code = f.read()

    # 1. Missing `.await?` in run_crash_recovery_scenario
    code = code.replace(
        "let case_report = test_integrity_check(db_path)?;",
        "let case_report = test_integrity_check(db_path).await?;",
    )
    code = code.replace(
        "let case_report = test_fresh_database_recovery(db_path)?;",
        "let case_report = test_fresh_database_recovery(db_path).await?;",
    )
    code = code.replace(
        "let case_report = test_append_only_invariant(db_path)?;",
        "let case_report = test_append_only_invariant(db_path).await?;",
    )

    # 2. Missing `.await` in snapshot result checks
    code = code.replace(
        "if snapshot_result.is_err() {", "if snapshot_result.await.is_err() {"
    )

    # 3. .execute -> sqlx::query().execute
    old_exec1 = """    bootstrap
        .pool
        .execute(
            "DELETE FROM snapshots WHERE revision = 2",
            [],
        )
        .map_err(|e| VerifyError::Sqlite(e.to_string()))?;"""
    new_exec1 = """    sqlx::query("DELETE FROM snapshots WHERE revision = 2")
        .execute(&bootstrap.pool)
        .await
        .map_err(|e| VerifyError::Sqlite(e.to_string()))?;"""
    code = code.replace(old_exec1, new_exec1)

    old_exec1b = """            bootstrap
                .pool
                .execute("DELETE FROM snapshots WHERE revision = 2", [])
                .map_err(|e| VerifyError::Sqlite(e.to_string()))?;"""
    new_exec1b = """            sqlx::query("DELETE FROM snapshots WHERE revision = 2")
                .execute(&bootstrap.pool)
                .await
                .map_err(|e| VerifyError::Sqlite(e.to_string()))?;"""
    code = code.replace(old_exec1b, new_exec1b)

    # 4. Result `.await`
    code = code.replace(
        'assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());',
        'assert!(result.await.is_ok(), "Expected Ok");',
    )
    code = code.replace(
        'let outcome = result.expect("Checked is_ok");',
        'let outcome = result.await.expect("Checked is_ok");',
    )

    code = code.replace(
        'assert!(result.is_err(), "Expected error for stale revision");\n        match result {',
        'assert!(result.is_err(), "Expected error for stale revision");\n        match result.await {',
    )

    code = code.replace(
        'assert!(result1.is_ok(), "First append should succeed");\n        let outcome1 = result1.expect("Checked is_ok");',
        'assert!(result1.await.is_ok(), "First append should succeed");\n        let outcome1 = result1.await.expect("Checked is_ok");',
    )

    code = code.replace(
        'assert!(result2.is_err(), "Duplicate op_id should be rejected");',
        'assert!(result2.await.is_err(), "Duplicate op_id should be rejected");',
    )

    # 5. `bootstrap.pool.query_row` at end
    old_query = """        let count: i64 = bootstrap
            .pool
            .query_row(
                "SELECT COUNT(*) FROM events WHERE operation_id = ?1",
                [op_id],
                |row| row.get(0),
            )
            .expect("Failed to query");"""
    new_query = """        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM events WHERE operation_id = ?")
            .bind(op_id)
            .fetch_one(&bootstrap.pool)
            .await
            .expect("Failed to query");"""
    code = code.replace(old_query, new_query)

    # Missing matches
    code = code.replace("match result {", "match result.await {")
    code = code.replace("match retry_result {", "match retry_result.await {")

    with open("diagram_tool/src/models/harness.rs", "w") as f:
        f.write(code)


fix()
