use chrono::{DateTime, Local, TimeZone};
use clap::{Parser, Subcommand};
use rusqlite::{Connection, Result};
use std::{
    fs,
    io::{Read, Write},
};

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
}

#[derive(Debug)]
struct Memo {
    id: i32,
    title: String,
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
            title       TEXT NOT NULL,
            content     TEXT,
            created_at  TEXT NOT NULL
        )",
        (),
    )?;

    match &cli.command {
        Some(Commands::Add) => add_memo(&conn),
        Some(Commands::List) => list_memo(&conn),
        None => {
            println!("No subcommand was used");
        }
    }

    Ok(())
}

fn add_memo(conn: &Connection) {
    print!("Enter title: ");
    std::io::stdout().flush().unwrap();

    let mut title = String::new();
    std::io::stdin().read_line(&mut title).unwrap();
    title = title.trim().to_string();

    let mut content = String::new();
    print!("Enter content: ");
    std::io::stdout().flush().unwrap();
    std::io::stdin()
        .lock()
        .read_to_string(&mut content)
        .unwrap();

    let memo = Memo {
        id: 0,
        title,
        content,
        created_at: Local::now(),
    };
    conn.execute(
        "INSERT INTO memo (title, content, created_at) VALUES (?1, ?2, ?3)",
        (&memo.title, &memo.content, &memo.created_at.to_rfc3339()),
    )
    .unwrap();
}

fn list_memo(conn: &Connection) {
    let mut stmt = conn
        .prepare("SELECT id, title, content, created_at FROM memo")
        .unwrap();
    let person_iter = stmt
        .query_map([], |row| {
            Ok(Memo {
                id: row.get(0)?,
                title: row.get(1)?,
                content: row.get(2)?,
                created_at: Local
                    .datetime_from_str(&row.get::<_, String>(3)?, "%Y-%m-%dT%H:%M:%S%.f%Z")
                    .unwrap(),
            })
        })
        .unwrap();
    for person in person_iter {
        println!("Found person {:?}", person.unwrap());
    }
}
