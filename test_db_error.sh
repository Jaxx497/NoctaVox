#!/bin/bash
# Test script to reproduce database loading error

# Create a corrupted database file
TESTDIR=$(mktemp -d)
DB_FILE="$TESTDIR/noctavox.db"

# Write garbage data to simulate corruption
echo "corrupted database" > "$DB_FILE"

# Try to open the database with rusqlite
cat > test_db_open.rs << 'RUST_EOF'
use rusqlite::Connection;

fn main() {
    let db_path = std::env::args().nth(1).expect("Need db path");
    match Connection::open(&db_path) {
        Ok(_) => println!("Database opened successfully"),
        Err(e) => {
            eprintln!("Database error: {}", e);
            std::process::exit(1);
        }
    }
}
RUST_EOF

rustc test_db_open.rs -o test_db_open 2>/dev/null || {
    echo "Failed to compile test"
    rm -rf "$TESTDIR"
    exit 1
}

./test_db_open "$DB_FILE"
EXIT_CODE=$?

# Cleanup
rm -rf "$TESTDIR" test_db_open.rs test_db_open

exit $EXIT_CODE
