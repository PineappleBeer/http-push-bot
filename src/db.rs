use rusqlite::{Connection, named_params, Row};
use crate::types::*;
use rusqlite_migration::{Migrations, M};
use anyhow::{Context, Result};

const MIGRATIONS: &[&str] = &[
    "
    CREATE TABLE IF NOT EXISTS channel (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        channel_id TEXT NOT NULL UNIQUE,
        channel_name TEXT NOT NULL,
        type_name TEXT NOT NULL, 
        date DATETIME DEFAULT CURRENT_TIMESTAMP
    );
    ",
    "
    CREATE TABLE IF NOT EXISTS message (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        chat_id INTERGER,
        text TEXT NOT NULL,
        type_name TEXT NOT NULL,
        date DATETIME DEFAULT CURRENT_TIMESTAMP
    );
",
];

pub struct Database {
    pub conn: Connection
}

impl Database {
    pub fn open() -> Result<Self> {
        let conn = Self::get_conn().context("error connection to database")?;
        println!("{:?}", conn);

        Ok(Database {conn})
    }

    fn get_conn() -> Result<Connection, rusqlite::Error> {
        Connection::open("http-push-bot.sqlite3")
    }

    pub fn migrate(&mut self) -> Result<(), rusqlite_migration::Error> {
        let migrations = MIGRATIONS.iter().map(|e| M::up(e)).collect();

        Migrations::new(migrations).to_latest(&mut self.conn)
    }

    pub fn mark_push_message(&self, chat_id:i64, text: String, type_name: String) -> Result<()> {
        let mut stmt = self.conn.prepare("
        insert into message (text, chat_id, type_name)
        values (:text, :chat_id, :type_name)
        ")?;

        stmt.execute(named_params! {
            ":text": text,
            ":chat_id": chat_id,
            ":type_name": type_name
        })
        .context("could not mark push message")
        .map(|_| ())
    }

    pub fn mark_auth_channel(&self, channel_id: i64, channel_name: &str, type_name: String) -> Result<()> {
        let mut stmt = self.conn.prepare("
        insert or ignore into channel (channel_id, channel_name, type_name)
        values (:channel_id, :channel_name, :type_name)
        ")?;

        stmt.execute(named_params! {
            ":channel_id": channel_id,
            ":channel_name": channel_name,
            ":type_name": type_name
        })
        .context("could not mark auth channel")
        .map(|_| ())
    }

    pub fn get_auth_channel(&self) -> Result<Vec<Channel>> {
        let mut stmt = self.conn.prepare("
        select * from channel
        ")?;

        let channels = stmt
        .query_map([], |row| Channel::try_from(row))?
        .collect::<Result<Vec<_>, rusqlite::Error>>()?;

        Ok(channels)
    }

}

impl TryFrom<&Row<'_>> for Channel {
    type Error = rusqlite::Error;

    fn try_from(row: &Row<'_>) -> Result<Self, Self::Error> {
        Ok(Self {
            channel_id: row.get_unwrap("channel_id"),
            channel_name: row.get_unwrap("channel_name"),
            type_name: row.get_unwrap("type_name"),
            date: row.get_unwrap("date"),
            id: row.get_unwrap("id"),
        })
    }
}