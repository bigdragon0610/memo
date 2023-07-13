use chrono::{DateTime, Local, TimeZone};
use clap::{Parser, Subcommand};
use rusqlite::{Connection, Result};
use std::fs;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new memo
    Add { content: String },
    /// List all memos
    List,
    /// Delete a memo
    Delete { id: i32 },
    /// Find memos
    Find { keyword: String },
}

#[derive(Debug)]
struct Memo {
    id: i32,
    content: String,
    created_at: DateTime<Local>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let home = dirs::home_dir().unwrap();
    let db_path = home.join(".memo/memo.sqlite");

    if let Some(parent) = db_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).unwrap();
        }
    }

    let conn = Connection::open(&db_path)?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS memo (
            id          INTEGER PRIMARY KEY,
            content     TEXT,
            created_at  TEXT NOT NULL
        )",
        (),
    )?;

    match &cli.command {
        Some(Commands::Add { content }) => add_memo(&conn, content.to_string()),
        Some(Commands::List) => list_memo(&conn),
        Some(Commands::Delete { id }) => delete_memo(&conn, *id),
        Some(Commands::Find { keyword }) => find_memo(&conn, keyword),
        None => {
            println!("No subcommand was used");
        }
    }

    Ok(())
}

fn add_memo(conn: &Connection, content: String) {
    let memo = Memo {
        id: 0,
        content,
        created_at: Local::now(),
    };
    conn.execute(
        "INSERT INTO memo (content, created_at) VALUES (?1, ?2)",
        (&memo.content, &memo.created_at.to_rfc3339()),
    )
    .unwrap();
}

fn list_memo(conn: &Connection) {
    let mut stmt = conn
        .prepare("SELECT id, content, created_at FROM memo")
        .unwrap();
    let memo_iter = stmt
        .query_map([], |row| {
            Ok(Memo {
                id: row.get(0)?,
                content: row.get(1)?,
                created_at: Local
                    .datetime_from_str(&row.get::<_, String>(2)?, "%Y-%m-%dT%H:%M:%S%.f%Z")
                    .unwrap(),
            })
        })
        .unwrap();
    for memo in memo_iter.flatten() {
        println!("{}: {}", memo.id, memo.content);
    }
}

fn delete_memo(conn: &Connection, id: i32) {
    conn.execute("DELETE FROM memo WHERE id = ?1", [id])
        .unwrap();
}

fn find_memo(conn: &Connection, keyword: &str) {
    let mut stmt = conn
        .prepare("SELECT id, content, created_at FROM memo WHERE content LIKE ?1")
        .unwrap();
    let memo_iter = stmt
        .query_map([format!("%{}%", keyword)], |row| {
            Ok(Memo {
                id: row.get(0)?,
                content: row.get(1)?,
                created_at: Local
                    .datetime_from_str(&row.get::<_, String>(2)?, "%Y-%m-%dT%H:%M:%S%.f%Z")
                    .unwrap(),
            })
        })
        .unwrap();
    for memo in memo_iter.flatten() {
        println!("{}: {}", memo.id, memo.content);
    }
}
