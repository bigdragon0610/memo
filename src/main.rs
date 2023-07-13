use chrono::{DateTime, Local, TimeZone};
use clap::{Parser, Subcommand};
use rusqlite::{Connection, Result};
use std::{fs, io::Write};

/// Simple program to greet a person
#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new memo
    Add,
    /// List all memos
    List,
    /// Delete a memo
    Delete { id: i32 },
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
    let conn = Connection::open(&db_path)?;

    if let Some(parent) = db_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).unwrap();
        }
    }

    conn.execute(
        "CREATE TABLE IF NOT EXISTS memo (
            id          INTEGER PRIMARY KEY,
            content     TEXT,
            created_at  TEXT NOT NULL
        )",
        (),
    )?;

    match &cli.command {
        Some(Commands::Add) => add_memo(&conn),
        Some(Commands::List) => list_memo(&conn),
        Some(Commands::Delete { id }) => delete_memo(&conn, *id),
        None => {
            println!("No subcommand was used");
        }
    }

    Ok(())
}

fn add_memo(conn: &Connection) {
    print!("Enter memo: ");
    std::io::stdout().flush().unwrap();

    let mut content = String::new();
    std::io::stdin().read_line(&mut content).unwrap();
    content = content.trim().to_string();
    if content.is_empty() {
        println!("Memo is empty");
        return;
    }

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
    let person_iter = stmt
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
    for person in person_iter {
        if let Ok(person) = person {
            println!("{}: {}", person.id, person.content);
        }
    }
}

fn delete_memo(conn: &Connection, id: i32) {
    conn.execute("DELETE FROM memo WHERE id = ?1", [id])
        .unwrap();
}
