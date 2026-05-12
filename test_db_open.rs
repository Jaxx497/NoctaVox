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
