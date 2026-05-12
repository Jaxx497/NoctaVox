/// Test that database errors are properly surfaced to users
///
/// This test reproduces issue #3: When the database is corrupted or has errors,
/// the user should receive a clear error message rather than a generic panic.
#[cfg(test)]
mod database_error_tests {
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_corrupted_database_error_message() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("corrupted.db");

        // Create a corrupted database file
        fs::write(&db_path, b"this is not a valid sqlite database").unwrap();

        // Attempt to open the corrupted database
        let conn = rusqlite::Connection::open(&db_path);
        assert!(conn.is_ok(), "Connection::open should succeed even with corrupted file");

        // SQLite won't detect corruption until we try to use it
        let conn = conn.unwrap();
        let result = conn.pragma_update(None, "journal_mode", "WAL");

        // Should return an error (not panic) when trying to use corrupted database
        assert!(result.is_err(), "Using corrupted database should return an error");

        // Error should contain useful information
        let error = result.unwrap_err();
        let error_msg = error.to_string();
        assert!(
            error_msg.contains("not a database") || error_msg.contains("file is not a database"),
            "Error message should indicate database corruption: {}",
            error_msg
        );
    }

    #[test]
    fn test_database_open_with_good_file() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("good.db");

        // Create a valid database
        let conn = rusqlite::Connection::open(&db_path).unwrap();
        conn.execute("CREATE TABLE test (id INTEGER PRIMARY KEY)", []).unwrap();
        drop(conn);

        // Should open successfully
        let result = rusqlite::Connection::open(&db_path);
        assert!(result.is_ok());
    }
}
