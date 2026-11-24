//! A small SQLite store for the post-processor's on-disk caches: a file, or an in-memory database when there is no
//! file to keep or it cannot be used.
//!
//! WAL journal (the file's own mode where WAL cannot be set: a network filesystem, or one without shared memory, FUSE,
//! CIFS, WSL's `/mnt`), `synchronous=NORMAL` (in WAL no sync per commit, and a crash never corrupts the file), a busy
//! timeout, and a `user_version` gate: a file made for another schema, or another program's database, is left alone;
//! a new file gets its schema under the write lock. Opening is retried while another process holds the file: turning
//! a new file to WAL meets a lock the busy timeout does not wait on (SQLite returns `SQLITE_BUSY` at once to avoid a
//! deadlock). Writers take one `IMMEDIATE` transaction per allocation ([`SqliteStore::immediate`]), so processes
//! sharing the file serialize on it. A store is opened lazily by its owner and never kept in a static (a forked process
//! must not inherit the connection). Failure policy, opening, reading or writing: a Warning, then the rest of the run in
//! memory ([`SqliteStore::fell_back`]), from a copy of what the file held when it can still be read.

use std::path::{Path, PathBuf};

use rusqlite::{Connection, Transaction, TransactionBehavior};

/// An open store: its connection and the file it reads, when not in memory.
pub struct SqliteStore {
  conn:      Connection,
  file:      Option<PathBuf>,
  schema:    &'static str,
  fell_back: bool,
}

/// How long a process waits for others holding the file, opening it or writing (as `object_db`): well inside a
/// conversion's default `--timeout=60`, after which the run goes on in memory.
const BUSY_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

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
        Err(OpenError::Foreign) => return Err("another program's database".to_string()),
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
    let mut conn = Connection::open(file)?;
    conn.busy_timeout(BUSY_TIMEOUT)?;
    // A file in WAL already is not asked again. Where WAL cannot be set SQLite grants another mode, or errs (no shared
    // memory): the file keeps its own; only a busy file is tried again.
    let mode: String = conn.query_row("PRAGMA journal_mode", [], |row| row.get(0))?;
    if !mode.eq_ignore_ascii_case("wal")
      && let Err(e) = conn.query_row("PRAGMA journal_mode=WAL", [], |row| row.get::<_, String>(0))
      && Self::busy(&e)
    {
      return Err(e.into());
    }
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    let found: i32 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
    if found == version {
      return Ok(conn);
    }
    if found != 0 {
      return Err(OpenError::Version(found));
    }
    // A new file gets the schema under the write lock, read again there: another process may have made it meanwhile.
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    match tx.pragma_query_value(None, "user_version", |row| row.get(0))? {
      0 => {
        let tables: i64 = tx.query_row(
          "SELECT count(*) FROM sqlite_master WHERE type = 'table'",
          [],
          |row| row.get(0),
        )?;
        if tables > 0 {
          return Err(OpenError::Foreign);
        }
        tx.execute_batch(&format!("{schema} PRAGMA user_version = {version};"))?;
      },
      found if found == version => {},
      found => return Err(OpenError::Version(found)),
    }
    tx.commit()?;
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

  /// Whether a file was asked for and could not be used, so the store may not know what earlier runs stored there.
  pub fn fell_back(&self) -> bool { self.fell_back }

  /// Run `f` in one `IMMEDIATE` transaction, committed when it succeeds. A file that fails is reported (Warning) and
  /// replaced by memory (`fall_back`), where `f` runs again: `f` is told whether the store runs in memory in
  /// place of its file ([`Self::fell_back`]), and defers its own side effects (a Warning) until the call returns. A
  /// transaction that fails in memory is reported (Error) and gives `None`.
  pub fn immediate<T>(
    &mut self,
    mut f: impl FnMut(&Transaction, bool) -> rusqlite::Result<T>,
  ) -> Option<T> {
    let mut attempt = |store: &mut Self| Self::run(&mut store.conn, store.fell_back, &mut f);
    match attempt(self) {
      Ok(value) => Some(value),
      Err(e) => {
        if self.fall_back(&e) {
          Self::ok(attempt(self))
        } else {
          Self::ok(Err(e))
        }
      },
    }
  }

  fn run<T>(
    conn: &mut Connection,
    fell_back: bool,
    f: &mut impl FnMut(&Transaction, bool) -> rusqlite::Result<T>,
  ) -> rusqlite::Result<T> {
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let value = f(&tx, fell_back)?;
    tx.commit()?;
    Ok(value)
  }

  /// A read outside any transaction, under [`Self::immediate`]'s failure policy.
  pub fn read<T>(&mut self, f: impl Fn(&Connection) -> rusqlite::Result<T>) -> Option<T> {
    match f(&self.conn) {
      Ok(value) => Some(value),
      Err(e) => {
        if self.fall_back(&e) {
          Self::ok(f(&self.conn))
        } else {
          Self::ok(Err(e))
        }
      },
    }
  }

  /// A store in memory reports its failure (Error), a bug in its caller's statements.
  fn ok<T>(result: rusqlite::Result<T>) -> Option<T> {
    result
      .map_err(|e| {
        Error!(
          "internal",
          "cache",
          "The cache in memory failed ({}); the entry is not kept",
          e
        );
      })
      .ok()
  }

  /// After `e` on the file: a Warning, then the store in memory, holding a copy of the file's entries (this run's
  /// among them) when it can still be read, else none. Whether there was a file to leave.
  fn fall_back(&mut self, e: &rusqlite::Error) -> bool {
    let Some(file) = self.file.take() else {
      return false;
    };
    let mut copy = Connection::open_in_memory().expect("an in-memory SQLite database");
    let copied = rusqlite::backup::Backup::new(&self.conn, &mut copy)
      .and_then(|backup| backup.run_to_completion(1024, std::time::Duration::ZERO, None));
    Warn!(
      "I/O",
      "cache",
      "The cache {} failed ({}); this run's further entries are kept in memory only{}",
      file.display(),
      e,
      if copied.is_ok() {
        ""
      } else {
        ", and its earlier ones are lost"
      }
    );
    self.conn = if copied.is_ok() {
      copy
    } else {
      Self::in_memory(self.schema)
    };
    self.fell_back = true;
    true
  }
}

enum OpenError {
  Sqlite(rusqlite::Error),
  /// The file holds this schema version, another than the store's.
  Version(i32),
  /// The file is another program's SQLite database: tables, and no schema version.
  Foreign,
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
      store.immediate(|tx, _| tx.execute("INSERT INTO t(v) VALUES (1)", [])),
      Some(1)
    );
    assert!(
      std::fs::read(&not_sqlite)
        .unwrap()
        .starts_with(b"a Berkeley DB file")
    );
    // Another program's database, its `user_version` 0, keeps its tables and its version.
    let foreign = dir.path().join("other.db");
    Connection::open(&foreign)
      .unwrap()
      .execute_batch("CREATE TABLE notes(text TEXT);")
      .unwrap();
    let store = SqliteStore::open(Some(&foreign), SCHEMA, 1);
    assert_eq!((store.file(), store.fell_back()), (None, true));
    let conn = Connection::open(&foreign).unwrap();
    let tables: Vec<String> = conn
      .prepare("SELECT name FROM sqlite_master WHERE type = 'table'")
      .unwrap()
      .query_map([], |row| row.get(0))
      .unwrap()
      .collect::<Result<_, _>>()
      .unwrap();
    let version: i32 = conn
      .pragma_query_value(None, "user_version", |row| row.get(0))
      .unwrap();
    assert_eq!((tables, version), (vec!["notes".to_string()], 0));
  }

  /// A statement that fails in memory is reported and gives nothing, without a panic, and the store keeps its entries.
  #[test]
  fn a_failure_in_memory_gives_nothing() {
    let mut store = SqliteStore::open(None, SCHEMA, 1);
    assert_eq!(
      store.immediate(|tx, _| tx.execute("INSERT INTO t(v) VALUES (1)", [])),
      Some(1)
    );
    assert_eq!(
      store.immediate(|tx, _| tx.execute("INSERT INTO nowhere(v) VALUES (1)", [])),
      None
    );
    assert_eq!(
      store.read(|conn| conn.query_row("SELECT nothing FROM t", [], |row| row.get::<_, i64>(0))),
      None
    );
    assert_eq!(
      store.read(|conn| conn.query_row("SELECT count(*) FROM t", [], |row| row.get::<_, i64>(0))),
      Some(1)
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

  /// A transaction that fails on the file (here a read-only file) runs again in memory, and is told so: a caller
  /// deciding by the store's state before the call missed the change (issue 995 review: a read-only cache, and a
  /// second document's image written over the first's `x1.png`).
  #[cfg(unix)]
  #[test]
  fn a_failed_transaction_runs_again_told_it_is_in_memory() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("c.db");
    let mut store = SqliteStore::open(Some(&file), SCHEMA, 1);
    store.immediate(|tx, _| tx.execute("INSERT INTO t(v) VALUES (7)", []));
    drop(store);
    std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o444)).unwrap();
    if std::fs::OpenOptions::new().write(true).open(&file).is_ok() {
      eprintln!("SKIP: a read-only file is writable here (root)");
      return;
    }
    let mut store = SqliteStore::open(Some(&file), SCHEMA, 1);
    assert_eq!(store.file(), Some(file.as_path()));
    let mut told = Vec::new();
    store.immediate(|tx, fell_back| {
      told.push(fell_back);
      tx.execute("INSERT INTO t(v) VALUES (1)", [])
    });
    // The entries the file held are kept in memory, the transaction's own added there once.
    let kept = store.read(|conn| {
      conn
        .prepare("SELECT v FROM t ORDER BY v")?
        .query_map([], |row| row.get::<_, i64>(0))?
        .collect::<rusqlite::Result<Vec<_>>>()
    });
    assert_eq!(
      (told, store.file(), store.fell_back(), kept),
      (vec![false, true], None, true, Some(vec![1, 7]))
    );
  }
}
