use rusqlite::{params, Connection, Result};
use crate::models::Note;
use crate::config::get_db_path;
use chrono::Utc;

pub struct Db {
    conn: Connection,
}

impl Db {
    /// Opens connection to SQLite DB and runs migrations.
    pub fn open() -> Result<Self> {
        let path = get_db_path();
        let conn = Connection::open(path)?;
        let db = Db { conn };
        db.migrate()?;
        Ok(db)
    }

    /// Creates table if it does not exist.
    fn migrate(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS notes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                text TEXT NOT NULL,
                color TEXT NOT NULL,
                pos_x INTEGER NOT NULL,
                pos_y INTEGER NOT NULL,
                width INTEGER NOT NULL,
                height INTEGER NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );",
            [],
        )?;
        Ok(())
    }

    /// Creates a new note in the DB.
    pub fn create_note(
        &mut self,
        text: &str,
        color: &str,
        pos_x: i32,
        pos_y: i32,
        width: i32,
        height: i32,
    ) -> Result<i64> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO notes (text, color, pos_x, pos_y, width, height, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![text, color, pos_x, pos_y, width, height, now, now],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Retrieves a single note.
    pub fn get_note(&self, id: i64) -> Result<Note> {
        self.conn.query_row(
            "SELECT id, text, color, pos_x, pos_y, width, height, created_at, updated_at 
             FROM notes WHERE id = ?1",
            params![id],
            |row| {
                Ok(Note {
                    id: row.get(0)?,
                    text: row.get(1)?,
                    color: row.get(2)?,
                    pos_x: row.get(3)?,
                    pos_y: row.get(4)?,
                    width: row.get(5)?,
                    height: row.get(6)?,
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                })
            },
        )
    }

    /// Updates an existing note.
    pub fn update_note(
        &mut self,
        id: i64,
        text: &str,
        color: &str,
        pos_x: i32,
        pos_y: i32,
        width: i32,
        height: i32,
    ) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE notes SET text = ?1, color = ?2, pos_x = ?3, pos_y = ?4, width = ?5, height = ?6, updated_at = ?7
             WHERE id = ?8",
            params![text, color, pos_x, pos_y, width, height, now, id],
        )?;
        Ok(())
    }

    /// Deletes a note.
    pub fn delete_note(&mut self, id: i64) -> Result<()> {
        self.conn.execute("DELETE FROM notes WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// Lists all notes in the database.
    pub fn list_notes(&self) -> Result<Vec<Note>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, text, color, pos_x, pos_y, width, height, created_at, updated_at FROM notes",
        )?;
        let note_iter = stmt.query_map([], |row| {
            Ok(Note {
                id: row.get(0)?,
                text: row.get(1)?,
                color: row.get(2)?,
                pos_x: row.get(3)?,
                pos_y: row.get(4)?,
                width: row.get(5)?,
                height: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })?;

        let mut notes = Vec::new();
        for note in note_iter {
            notes.push(note?);
        }
        Ok(notes)
    }
}
