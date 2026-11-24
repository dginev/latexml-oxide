//! A small SQLite store for the post-processor's on-disk caches: a file, or an in-memory database when there is no
//! file to keep or it cannot be used.
//!
//! WAL journal (whatever mode SQLite grants where WAL cannot be set, a network filesystem), `synchronous=OFF` (the
//! store is rebuildable: a lost write costs a name, not a document), a busy timeout, and a `user_version` gate: a file
//! made for another schema is left alone. Writers take one `IMMEDIATE` transaction per allocation
//! ([`SqliteStore::immediate`]), so processes sharing the file serialize on it. A store is opened lazily by its owner
//! and never kept in a static (a forked process must not inherit the connection). Failure policy: a Warning, then the
//! rest of the run in memory.

use std::path::{Path, PathBuf};

use rusqlite::{Connection, Transaction, TransactionBehavior};

/// An open store: its connection and the file it reads, when not in memory.
pub struct SqliteStore {
  conn:   Connection,
  file:   Option<PathBuf>,
  schema: &'static str,
}

impl SqliteStore {
  /// Open `file` (`None`: in memory) with `schema` (its statements idempotent) at schema `version`. A file that
  /// cannot be opened, or holds another version, is reported (Warning) and left alone; the store is then in memory.
  pub fn open(file: Option<&Path>, schema: &'static str, version: i32) -> Self {
    let conn = file.and_then(|file| match Self::open_file(file, schema, version) {
      Ok(conn) => Some(conn),
      Err(e) => {
        Warn!(
          "I/O",
          "cache",
          "Can't use the cache {} ({}); this run's entries are kept in memory only",
          file.display(),
          e
        );
        None
      },
    });
    let file = conn.as_ref().and(file.map(Path::to_path_buf));
    let conn = conn.unwrap_or_else(|| Self::in_memory(schema));
    SqliteStore { conn, file, schema }
  }

  fn open_file(file: &Path, schema: &str, version: i32) -> Result<Connection, String> {
    let conn = Connection::open(file).map_err(|e| e.to_string())?;
    conn
      .busy_timeout(std::time::Duration::from_secs(120))
      .map_err(|e| e.to_string())?;
    // The mode SQLite grants (`delete` where WAL cannot be set) is accepted.
    let _: String = conn
      .query_row("PRAGMA journal_mode=WAL", [], |row| row.get(0))
      .map_err(|e| e.to_string())?;
    conn
      .pragma_update(None, "synchronous", "OFF")
      .map_err(|e| e.to_string())?;
    let found: i32 = conn
      .pragma_query_value(None, "user_version", |row| row.get(0))
      .map_err(|e| e.to_string())?;
    match found {
      0 => {
        conn
          .execute_batch(&format!(
            "BEGIN IMMEDIATE; {schema} PRAGMA user_version = {version}; COMMIT;"
          ))
          .map_err(|e| e.to_string())?;
      },
      found if found == version => {},
      found => return Err(format!("schema version {found}, not {version}")),
    }
    Ok(conn)
  }

  fn in_memory(schema: &str) -> Connection {
    let conn = Connection::open_in_memory().expect("an in-memory SQLite database");
    conn
      .execute_batch(schema)
      .expect("the store's schema in memory");
    conn
  }

  /// The file the store reads, `None` in memory.
  pub fn file(&self) -> Option<&Path> { self.file.as_deref() }

  /// Run `f` in one `IMMEDIATE` transaction, committed when it succeeds. A store that fails is reported (Warning)
  /// and replaced by an empty in-memory one, where `f` runs again.
  pub fn immediate<T>(&mut self, mut f: impl FnMut(&Transaction) -> rusqlite::Result<T>) -> T {
    match Self::run(&mut self.conn, &mut f) {
      Ok(value) => value,
      Err(e) => {
        Warn!(
          "I/O",
          "cache",
          "The cache {} failed ({}); this run's further entries are kept in memory only",
          self
            .file
            .as_deref()
            .map_or("in memory".to_string(), |f| f.display().to_string()),
          e
        );
        self.conn = Self::in_memory(self.schema);
        self.file = None;
        Self::run(&mut self.conn, &mut f).expect("an in-memory SQLite transaction")
      },
    }
  }

  fn run<T>(
    conn: &mut Connection,
    f: &mut impl FnMut(&Transaction) -> rusqlite::Result<T>,
  ) -> rusqlite::Result<T> {
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let value = f(&tx)?;
    tx.commit()?;
    Ok(value)
  }

  /// A read outside any transaction.
  pub fn read<T>(&self, f: impl FnOnce(&Connection) -> rusqlite::Result<T>) -> Option<T> {
    f(&self.conn).ok()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  const SCHEMA: &str = "CREATE TABLE IF NOT EXISTS t(v INTEGER);";

  /// A file of another schema version is left alone and the store runs in memory; a fresh file gets the schema.
  #[test]
  fn another_version_is_left_alone() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("c.db");
    let store = SqliteStore::open(Some(&file), SCHEMA, 1);
    assert_eq!(store.file(), Some(file.as_path()));
    drop(store);
    let store = SqliteStore::open(Some(&file), SCHEMA, 2);
    assert_eq!(store.file(), None);
    let not_sqlite = dir.path().join("perl.cache");
    std::fs::write(
      &not_sqlite,
      b"a Berkeley DB file, not SQLite at all, padded past one header",
    )
    .unwrap();
    let mut store = SqliteStore::open(Some(&not_sqlite), SCHEMA, 1);
    assert_eq!(store.file(), None);
    assert_eq!(
      store.immediate(|tx| tx.execute("INSERT INTO t(v) VALUES (1)", [])),
      1
    );
    assert!(
      std::fs::read(&not_sqlite)
        .unwrap()
        .starts_with(b"a Berkeley DB file")
    );
  }
}
