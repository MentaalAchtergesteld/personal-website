use std::{path::Path};
use chrono::{DateTime, Utc};
use rusqlite::{Connection, Row};

use crate::models::Message;

pub struct MessageDb {
    connection: Connection
}

impl MessageDb {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, ()> {
        let connection = Connection::open(path)
            .map_err(|e| eprintln!("ERROR: couldn't connect to database: {e}"))?; 

        connection.execute("
            CREATE TABLE IF NOT EXISTS messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                author TEXT NOT NULL,
                content TEXT NOT NULL,
                timestamp TEXT NOT NULL
            )         
        ", []).map_err(|e| eprintln!("ERROR: Couldn't create table in database: {e}"))?;

        Ok(Self {
            connection
        })
    }

    fn parse_message(row: &Row<'_>) -> rusqlite::Result<Message> {
        let timestamp_str: String = row.get(3)?;
        let timestamp = DateTime::parse_from_rfc3339(&timestamp_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or(Utc::now());

        Ok(Message {
            id: row.get(0)?,
            author: row.get(1)?,
            content: row.get(2)?,
            timestamp
        })
    }

    pub fn create_message(&self, author: &str, content: &str) -> Result<Message, ()> {
        let timestamp_str = Utc::now().to_rfc3339();

        let message = self.connection.query_row("
            INSERT INTO messages (author, content, timestamp)
            VALUES (?1, ?2, ?3)
            RETURNING id, author, content, timestamp
        ", [author, content, &timestamp_str], Self::parse_message
        ).map_err(|e| eprintln!("ERROR: Couldn't create message: {e}"))?;

        Ok(message)
    }

    pub fn read_messages(&self, last_id: Option<i64>, limit: i64) -> Result<Vec<Message>, ()> {
        let cursor = last_id.unwrap_or(i64::MAX);

        let mut stmt = self.connection.prepare("
            SELECT id, author, content, timestamp
            FROM messages
            WHERE id < ?1
            ORDER BY id DESC
            LIMIT ?2
        ").map_err(|e| eprintln!("ERROR: Couldn't prepare read statement: {e}"))?;

        let messages_iter = stmt.query_map([cursor, limit], Self::parse_message)
            .map_err(|e| eprintln!("ERROR: Couldn't read messages: {e}"))?;

        messages_iter.collect::<Result<Vec<Message>, _>>()
            .map_err(|e| eprintln!("ERROR: Couldn't collect messages: {e}"))
    }
}
