//! Lightweight SQLite helpers backed by `rusqlite` + `r2d2`.

use anyhow::Result;
use chrono::{DateTime, Utc};
use rusqlite::params;
use serde::{Deserialize, Serialize};

pub type DbPool = r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>;

/// JSON representation of a card passed to the UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardJson {
    pub id: i64,
    pub front: String,
    pub back: String,
    pub tags: Vec<String>,
}

/// Connect / create database pool.
pub fn new_pool(path: &str) -> Result<DbPool> {
    let mgr = r2d2_sqlite::SqliteConnectionManager::file(path);
    let pool = r2d2::Pool::new(mgr)?;
    // Basic migration – create tables if they do not exist.
    {
        let conn = pool.get()?;
        conn.execute_batch(
            r#"CREATE TABLE IF NOT EXISTS cards (
                    id           INTEGER PRIMARY KEY AUTOINCREMENT,
                    front_text   TEXT NOT NULL,
                    back_text    TEXT NOT NULL,
                    tags         TEXT,
                    created_at   INTEGER NOT NULL DEFAULT (strftime('%s','now')),
                    next_due     INTEGER NOT NULL DEFAULT (strftime('%s','now')),
                    efactor      REAL    NOT NULL DEFAULT 2.5,
                    interval     INTEGER NOT NULL DEFAULT 1,
                    source_image TEXT
                );
              CREATE TABLE IF NOT EXISTS reviews (
                    id         INTEGER PRIMARY KEY AUTOINCREMENT,
                    card_id    INTEGER NOT NULL REFERENCES cards(id),
                    reviewed_at INTEGER NOT NULL,
                    passed     INTEGER NOT NULL
              );
            "#,
        )?;
    }
    Ok(pool)
}

/// Insert new card and return its rowid.
pub fn insert_card(pool: &DbPool, c: &CardJson, img_path: Option<&str>) -> Result<i64> {
    let conn = pool.get()?;
    conn.execute(
        "INSERT INTO cards (front_text, back_text, tags, source_image) VALUES (?1, ?2, ?3, ?4)",
        params![c.front, c.back, c.tags.join(","), img_path],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Fetch cards due before given timestamp.
pub fn fetch_due_cards(pool: &DbPool, ts: DateTime<Utc>) -> Result<Vec<CardJson>> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare(
        "SELECT id, front_text, back_text, tags FROM cards WHERE next_due <= ?1 ORDER BY id",
    )?;
    let rows = stmt
        .query_map([ts.timestamp()], |row| {
            let tags: String = row.get(3)?;
            Ok(CardJson {
                id: row.get(0)?,
                front: row.get(1)?,
                back: row.get(2)?,
                tags: if tags.is_empty() {
                    Vec::new()
                } else {
                    tags.split(',').map(|s| s.trim().to_owned()).collect()
                },
            })
        })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

/// Fetch all cards (front/back/tags).
pub fn fetch_all_cards(pool: &DbPool) -> Result<Vec<CardJson>> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare("SELECT id, front_text, back_text, tags FROM cards ORDER BY id DESC")?;
    let rows = stmt.query_map([], |row| {
        let tags: String = row.get(3)?;
        Ok(CardJson {
            id: row.get(0)?,
            front: row.get(1)?,
            back: row.get(2)?,
            tags: if tags.is_empty() {
                Vec::new()
            } else {
                tags.split(',').map(|s| s.trim().to_owned()).collect()
            },
        })
    })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
} 