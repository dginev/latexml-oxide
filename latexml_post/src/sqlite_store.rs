//! A small SQLite store for the post-processor's on-disk caches: a file, or an in-memory database when there is no
//! file to keep or it cannot be used.
//!
//! WAL journal (whatever mode SQLite grants where WAL cannot be set, a network filesystem), `synchronous=OFF` (the
//! store is rebuildable: a lost write costs a name, not a document), a busy timeout, and a `user_version` gate: a file
//! made for another schema is left alone. Opening is retried while another process holds the file: turning a new file
//! to WAL meets a lock the busy timeout does not wait on (SQLite returns `SQLITE_BUSY` at once to avoid a deadlock).
//! Writers take one `IMMEDIATE` transaction per allocation ([`SqliteStore::immediate`]), so processes sharing the file
//! serialize on it. A store is opened lazily by its owner and never kept in a static (a forked process must not
//! inherit the connection). Failure policy: a Warning, then the rest of the run in memory ([`SqliteStore::fell_back`]).

use std::path::{Path, PathBuf};

use rusqlite::{Connection, Transaction, TransactionBehavior};

/// An open store: its connection and the file it reads, when not in memory.
pub struct SqliteStore {
  conn:      Connection,
  file:      Option<PathBuf>,
  schema:    &'static str,
  fell_back: bool,
}

/// How long a process waits for others holding the file, opening it or writing.
const BUSY_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(120);

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
          "Can't use the cache {} ({}); this run's entries are kept in memory only. Delete the file when no \
           conversion is using it, and the next run makes a new one",
          file.display(),
          e
        );
        None
      },
    });
    let fell_back = file.is_some() && conn.is_none();
    let file = conn.as_ref().and(file.map(Path::to_path_buf));
    let conn = conn.unwrap_or_else(|| Self::in_memory(schema));
    SqliteStore { conn, file, schema, fell_back }
  }

  /// [`Self::open_once`], again after a pause while another process holds the file, up to [`BUSY_TIMEOUT`].
  fn open_file(file: &Path, schema: &str, version: i32) -> Result<Connection, String> {
    let start = std::time::Instant::now();
    // A pause that grows, and differs between processes, so they do not meet again.
    let mut pause = std::time::Duration::from_micros(500 + u64::from(std::process::id() % 1000));
    loop {
      match Self::open_once(file, schema, version) {
        Err(OpenError::Sqlite(e)) if Self::busy(&e) && start.elapsed() < BUSY_TIMEOUT => {
          std::thread::sleep(pause);
          pause = (pause * 2).min(std::time::Duration::from_millis(100));
        },
        Err(OpenError::Sqlite(e)) => return Err(e.to_string()),
        Err(OpenError::Version(found)) => {
          return Err(format!("schema version {found}, not {version}"));
        },
        Ok(conn) => return Ok(conn),
      }
    }
  }

  fn busy(e: &rusqlite::Error) -> bool {
    matches!(
      e.sqlite_error_code(),
      Some(rusqlite::ErrorCode::DatabaseBusy | rusqlite::ErrorCode::DatabaseLocked)
    )
  }

  fn open_once(file: &Path, schema: &str, version: i32) -> Result<Connection, OpenError> {
    let conn = Connection::open(file)?;
    conn.busy_timeout(BUSY_TIMEOUT)?;
    // The mode SQLite grants (`delete` where WAL cannot be set) is accepted; a file in WAL already is not asked again.
    let mode: String = conn.query_row("PRAGMA journal_mode", [], |row| row.get(0))?;
    if !mode.eq_ignore_ascii_case("wal") {
      let _: String = conn.query_row("PRAGMA journal_mode=WAL", [], |row| row.get(0))?;
    }
    conn.pragma_update(None, "synchronous", "OFF")?;
    let found: i32 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
    match found {
      0 => conn.execute_batch(&format!(
        "BEGIN IMMEDIATE; {schema} PRAGMA user_version = {version}; COMMIT;"
      ))?,
      found if found == version => {},
      found => return Err(OpenError::Version(found)),
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

  /// Whether a file was asked for and could not be used, so the store forgets what earlier runs stored there.
  pub fn fell_back(&self) -> bool { self.fell_back }

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
        self.fell_back |= self.file.is_some();
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

enum OpenError {
  Sqlite(rusqlite::Error),
  /// The file holds this schema version, another than the store's.
  Version(i32),
}

impl From<rusqlite::Error> for OpenError {
  fn from(e: rusqlite::Error) -> Self { OpenError::Sqlite(e) }
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

  /// Sixteen processes opening one new file at the same moment all end on the file: turning it to WAL and writing its
  /// schema meet a lock the busy timeout does not wait on, and are retried (issue 995 review: 3 trials in 5 left a
  /// process in memory, `database is locked`). The test runs itself as each process, released by one start file.
  #[test]
  fn simultaneous_opens_of_a_new_file_all_use_it() {
    const NAME: &str = "sqlite_store::tests::simultaneous_opens_of_a_new_file_all_use_it";
    if let Ok(file) = std::env::var("SQLITE_STORE_TEST_OPEN") {
      let go = Path::new(&file).with_extension("go");
      while !go.exists() {
        std::thread::sleep(std::time::Duration::from_micros(200));
      }
      let store = SqliteStore::open(Some(Path::new(&file)), SCHEMA, 1);
      print!(
        "{}",
        if store.file().is_some() {
          "FILE"
        } else {
          "MEMORY"
        }
      );
      return;
    }
    for _ in 0..3 {
      let dir = tempfile::tempdir().unwrap();
      let file = dir.path().join("c.db");
      let children: Vec<_> = (0..16)
        .map(|_| {
          std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", NAME, "--nocapture", "--test-threads=1"])
            .env("SQLITE_STORE_TEST_OPEN", &file)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .spawn()
            .unwrap()
        })
        .collect();
      std::thread::sleep(std::time::Duration::from_millis(300));
      std::fs::write(file.with_extension("go"), b"").unwrap();
      let outcomes: Vec<String> = children
        .into_iter()
        .map(|child| {
          let out = String::from_utf8(child.wait_with_output().unwrap().stdout).unwrap();
          if out.contains("FILE") {
            "FILE"
          } else {
            "MEMORY"
          }
          .to_string()
        })
        .collect();
      assert_eq!(outcomes, vec!["FILE"; 16]);
    }
  }
}
