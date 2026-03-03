use anyhow::Result;
use rusqlite::{Connection, params};
use std::sync::Mutex;

use crate::models::Thought;

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn open(path: &str) -> Result<Self> {
        let conn = if path == ":memory:" {
            Connection::open_in_memory()?
        } else {
            Connection::open(path)?
        };
        
        unsafe { sqlite_vec::sqlite3_vec_init(); }
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        
        let db = Self { conn: Mutex::new(conn) };
        db.run_migrations()?;
        Ok(db)
    }

    pub fn run_migrations(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS thoughts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                content TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                has_embedding BOOLEAN DEFAULT 0
            );
            CREATE TABLE IF NOT EXISTS tags (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT UNIQUE NOT NULL
            );
            CREATE TABLE IF NOT EXISTS thought_tags (
                thought_id INTEGER NOT NULL,
                tag_id INTEGER NOT NULL,
                PRIMARY KEY (thought_id, tag_id),
                FOREIGN KEY (thought_id) REFERENCES thoughts(id) ON DELETE CASCADE,
                FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
            );
            CREATE VIRTUAL TABLE IF NOT EXISTS vec_thoughts USING vec0(
                thought_id INTEGER PRIMARY KEY,
                embedding float[384]
            );"
        )?;
        Ok(())
    }

    pub fn is_healthy(&self) -> bool {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch("SELECT 1").is_ok()
    }

    pub fn insert_thought(&self, content: &str, tags: &[String]) -> Result<Thought> {
        let conn = self.conn.lock().unwrap();
        conn.execute("INSERT INTO thoughts (content) VALUES (?1)", params![content])?;
        let id = conn.last_insert_rowid();

        for tag_name in tags {
            conn.execute(
                "INSERT OR IGNORE INTO tags (name) VALUES (?1)",
                params![tag_name],
            )?;
            let tag_id: i64 = conn.query_row(
                "SELECT id FROM tags WHERE name = ?1",
                params![tag_name],
                |row| row.get(0),
            )?;
            conn.execute(
                "INSERT INTO thought_tags (thought_id, tag_id) VALUES (?1, ?2)",
                params![id, tag_id],
            )?;
        }

        // Read back the inserted thought (inline to avoid deadlock)
        Self::read_thought_inner(&conn, id)
    }

    pub fn get_thought(&self, id: i64) -> Result<Thought> {
        let conn = self.conn.lock().unwrap();
        Self::read_thought_inner(&conn, id)
    }

    pub fn list_thoughts(&self, limit: i64, offset: i64) -> Result<Vec<Thought>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, content, created_at, has_embedding FROM thoughts ORDER BY id DESC LIMIT ?1 OFFSET ?2"
        )?;
        let rows = stmt.query_map(params![limit, offset], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, bool>(3)?,
            ))
        })?;

        let mut thoughts = Vec::new();
        for row in rows {
            let (id, content, created_at, has_embedding) = row?;
            let tags = Self::get_tags_inner(&conn, id)?;
            thoughts.push(Thought { id, content, created_at, has_embedding, tags });
        }
        Ok(thoughts)
    }

    pub fn delete_thought(&self, id: i64) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let affected = conn.execute("DELETE FROM thoughts WHERE id = ?1", params![id])?;
        Ok(affected > 0)
    }

    /// Internal: read a single thought with tags, requires caller to hold the lock.
    fn read_thought_inner(conn: &Connection, id: i64) -> Result<Thought> {
        let (id, content, created_at, has_embedding) = conn.query_row(
            "SELECT id, content, created_at, has_embedding FROM thoughts WHERE id = ?1",
            params![id],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, bool>(3)?,
                ))
            },
        )?;
        let tags = Self::get_tags_inner(conn, id)?;
        Ok(Thought { id, content, created_at, has_embedding, tags })
    }

    /// Internal: get tags for a thought, requires caller to hold the lock.
    fn get_tags_inner(conn: &Connection, thought_id: i64) -> Result<Vec<String>> {
        let mut stmt = conn.prepare(
            "SELECT t.name FROM tags t JOIN thought_tags tt ON t.id = tt.tag_id WHERE tt.thought_id = ?1"
        )?;
        let tags = stmt.query_map(params![thought_id], |row| row.get::<_, String>(0))?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(tags)
    }

    pub fn set_embedding(&self, thought_id: i64, embedding: &[f32]) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        // Convert &[f32] to bytes for sqlite-vec
        let embedding_bytes: Vec<u8> = embedding
            .iter()
            .flat_map(|&f| f.to_le_bytes())
            .collect();

        conn.execute(
            "INSERT OR REPLACE INTO vec_thoughts (thought_id, embedding) VALUES (?1, ?2)",
            params![thought_id, embedding_bytes],
        )?;

        conn.execute(
            "UPDATE thoughts SET has_embedding = 1 WHERE id = ?1",
            params![thought_id],
        )?;

        Ok(())
    }

    pub fn search(&self, embedding: &[f32], limit: i64) -> Result<Vec<Thought>> {
        let conn = self.conn.lock().unwrap();
        let embedding_bytes: Vec<u8> = embedding
            .iter()
            .flat_map(|&f| f.to_le_bytes())
            .collect();

        // KNN search via vec_thoughts using the custom MATCH operator
        let mut stmt = conn.prepare(
            "SELECT thought_id 
             FROM vec_thoughts 
             WHERE embedding MATCH ?1 AND k = ?2
             ORDER BY distance"
        )?;

        let rows = stmt.query_map(params![embedding_bytes, limit], |row| {
            row.get::<_, i64>(0)
        })?;

        let mut results = Vec::new();
        for row in rows {
            let id = row?;
            if let Ok(thought) = Self::read_thought_inner(&conn, id) {
                results.push(thought);
            }
        }

        Ok(results)
    }
}
