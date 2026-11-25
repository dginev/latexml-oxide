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
//! memory ([`SqliteStore::fell_back`]), from a copy of what the file held when it gives it at once and the copy works.

use std::path::{Path, PathBuf};

use rusqlite::{Connection, Transaction, TransactionBehavior};

/// An open store: its connection and the file it reads, when not in memory.
pub struct SqliteStore {
  conn:      Connection,
  file:      Option<PathBuf>,
  schema:    &'static str,
  fell_back: bool,
  /// Declared after `conn`, so dropped once the connection is closed.
  wal_check: WalCheck,
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
    let wal_check = WalCheck(file.clone());
    SqliteStore {
      conn,
      file,
      schema,
      fell_back,
      wal_check,
    }
  }

  /// [`Self::open_once`], again after a pause while another process holds the file, up to [`BUSY_TIMEOUT`]. A link is
  /// never followed (SQLite would make or write the file it leads to, out of the folder), nor anything else not a
  /// regular file opened (a FIFO, a device, a directory). The side files cannot be redirected: SQLite names them after
  /// the file and opens them with `O_NOFOLLOW` (`unixOpen`), so a planted `-wal` or `-shm` link fails the open.
  fn open_file(file: &Path, schema: &str, version: i32) -> Result<Connection, String> {
    match std::fs::symlink_metadata(file).map(|meta| meta.file_type()) {
      Ok(kind) if kind.is_symlink() => return Err("a symbolic link, never followed".to_string()),
      Ok(kind) if !kind.is_file() => return Err("not a regular file".to_string()),
      _ => {},
    }
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

  /// Whether a file was asked for: the store runs on it, or in memory in its place.
  pub fn wants_file(&self) -> bool { self.file.is_some() || self.fell_back }

  /// Run `f` in one `IMMEDIATE` transaction, committed when it succeeds. A file that fails is reported (Warning) and
  /// replaced by memory (`attempt`, from `copy`), where `f` runs again: `f` is told whether the store runs in memory
  /// in place of its file ([`Self::fell_back`]), and defers its own side effects (a Warning) until the call returns. A
  /// transaction that fails in a fresh store in memory is reported (Error) and gives `None`.
  pub fn immediate<T>(
    &mut self,
    mut f: impl FnMut(&Transaction, bool) -> rusqlite::Result<T>,
  ) -> Option<T> {
    self.attempt(|conn, fell_back| Self::run(conn, fell_back, &mut f))
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
    self.attempt(|conn, _| f(conn))
  }

  /// `f` on the store; when it fails on the file, the store goes to memory (one Warning) and `f` runs there: on a copy
  /// of the file's entries (this run's among them) when the file gives them at once, else, or when `f` fails on the
  /// copy (a broken file copies as it is), on an empty store.
  fn attempt<T>(
    &mut self,
    mut f: impl FnMut(&mut Connection, bool) -> rusqlite::Result<T>,
  ) -> Option<T> {
    let e = match f(&mut self.conn, self.fell_back) {
      Ok(value) => return Some(value),
      Err(e) => e,
    };
    let Some(file) = self.file.take() else {
      return Self::ok(Err(e));
    };
    self.fell_back = true;
    let mut result = None;
    if let Some(copy) = self.copy() {
      self.conn = copy;
      result = f(&mut self.conn, true).ok();
    }
    let kept = result.is_some();
    Warn!(
      "I/O",
      "cache",
      "The cache {} failed ({}); this run's further entries are kept in memory only{}",
      file.display(),
      e,
      if kept {
        ""
      } else {
        ", and its earlier ones are lost"
      }
    );
    if kept {
      return result;
    }
    self.conn = Self::in_memory(self.schema);
    Self::ok(f(&mut self.conn, true))
  }

  /// A copy in memory of the file's entries, when the file gives them at once: the connection, left right after, waits
  /// on no lock (a busy source would be retried for ever by `Backup::run_to_completion`), and one step copies the
  /// whole file or nothing.
  fn copy(&self) -> Option<Connection> {
    self.conn.busy_timeout(std::time::Duration::ZERO).ok()?;
    let mut copy = Connection::open_in_memory().ok()?;
    let done = matches!(
      rusqlite::backup::Backup::new(&self.conn, &mut copy).map(|backup| backup.step(-1)),
      Ok(Ok(rusqlite::backup::StepResult::Done))
    );
    done.then_some(copy)
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
}

/// A store's file, looked at once its connection is closed. A close removes the `-wal` and `-shm` files only when its
/// one try for the file's exclusive lock succeeds (`sqlite3WalClose`), so processes closing together can all leave
/// them: while a `-wal` is there after a pause (another each time, so the processes part), the file is opened, read
/// (which opens its WAL) and closed again, twice at most. A process still using the file keeps them, as SQLite means.
struct WalCheck(Option<PathBuf>);

impl Drop for WalCheck {
  fn drop(&mut self) {
    use std::hash::BuildHasher;
    let Some(file) = &self.0 else { return };
    let mut wal = file.as_os_str().to_owned();
    wal.push("-wal");
    if !Path::new(&wal).exists() {
      return;
    }
    let random = std::collections::hash_map::RandomState::new();
    for attempt in 0..2 {
      std::thread::sleep(std::time::Duration::from_micros(
        2000 + random.hash_one(attempt) % 3000,
      ));
      // Looked at again after the pause: a read of the file with no `-wal` would make one.
      if !Path::new(&wal).exists() {
        return;
      }
      // No `SQLITE_OPEN_CREATE`: a file removed meanwhile is not made again; nor is a link followed; nor a lock awaited.
      let flags = rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE
        | rusqlite::OpenFlags::SQLITE_OPEN_NOFOLLOW
        | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX;
      if let Ok(conn) = Connection::open_with_flags(file, flags)
        && conn.busy_timeout(std::time::Duration::ZERO).is_ok()
      {
        let _ = conn.query_row("SELECT count(*) FROM sqlite_master", [], |row| {
          row.get::<_, i64>(0)
        });
      }
    }
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

  /// Eight processes that open a new file together, write, and close it at the same moment leave no `-wal` or `-shm`,
  /// none in memory, no Warning ([`WalCheck`]; base: 162-282 side files in 200 rounds on Linux; macOS CI run
  /// 45645721304 left them in `batchfix995_concurrent_documents_in_one_folder_never_share_a_name`). Eight processes in
  /// all, each running every round (`SQLITE_STORE_TEST_ROUNDS`, default 100), released by start files.
  #[test]
  fn simultaneous_closes_leave_no_side_files() {
    const NAME: &str = "sqlite_store::tests::simultaneous_closes_leave_no_side_files";
    const PROCESSES: usize = 8;
    let wait_for = |path: &Path| {
      let start = std::time::Instant::now();
      while !path.exists() {
        assert!(
          start.elapsed() < std::time::Duration::from_secs(30),
          "{}",
          path.display()
        );
        std::thread::sleep(std::time::Duration::from_micros(100));
      }
    };
    let rounds: usize =
      std::env::var("SQLITE_STORE_TEST_ROUNDS").map_or(100, |n| n.parse().unwrap());
    if let Ok(child) = std::env::var("SQLITE_STORE_TEST_CLOSE") {
      use latexml_core::common::error::{LogStatus, get_status};
      let (dir, k) = child.split_once('|').unwrap();
      let dir = Path::new(dir);
      for round in 0..rounds {
        wait_for(&dir.join(format!("open{round}")));
        let warnings = get_status(LogStatus::Warning);
        let mut store = SqliteStore::open(Some(&dir.join(format!("c{round}.db"))), SCHEMA, 1);
        store.immediate(|tx, _| tx.execute("INSERT INTO t(v) VALUES (1)", []));
        let on_file = store.file().is_some();
        std::fs::write(dir.join(format!("ready{round}.{k}")), b"").unwrap();
        wait_for(&dir.join(format!("close{round}")));
        drop(store);
        let warnings = get_status(LogStatus::Warning) - warnings;
        // Written whole before it is seen: the parent reads it once it exists.
        let part = dir.join(format!("part{round}.{k}"));
        std::fs::write(&part, format!("{} {warnings}", u8::from(!on_file))).unwrap();
        std::fs::rename(part, dir.join(format!("done{round}.{k}"))).unwrap();
      }
      return;
    }
    let dir = tempfile::tempdir().unwrap();
    let children: Vec<_> = (0..PROCESSES)
      .map(|k| {
        std::process::Command::new(std::env::current_exe().unwrap())
          .args(["--exact", NAME, "--nocapture", "--test-threads=1"])
          .env(
            "SQLITE_STORE_TEST_CLOSE",
            format!("{}|{k}", dir.path().display()),
          )
          .env("SQLITE_STORE_TEST_ROUNDS", rounds.to_string())
          .stdout(std::process::Stdio::null())
          .spawn()
          .unwrap()
      })
      .collect();
    let (mut left, mut memory, mut warnings) = (Vec::new(), 0, 0);
    for round in 0..rounds {
      std::fs::write(dir.path().join(format!("open{round}")), b"").unwrap();
      (0..PROCESSES).for_each(|k| wait_for(&dir.path().join(format!("ready{round}.{k}"))));
      std::fs::write(dir.path().join(format!("close{round}")), b"").unwrap();
      for k in 0..PROCESSES {
        let done = dir.path().join(format!("done{round}.{k}"));
        wait_for(&done);
        let counts: Vec<usize> = std::fs::read_to_string(&done)
          .unwrap()
          .split(' ')
          .map(|n| n.parse().unwrap())
          .collect();
        memory += counts[0];
        warnings += counts[1];
      }
      for side in ["-wal", "-shm"] {
        if dir.path().join(format!("c{round}.db{side}")).exists() {
          left.push(format!("c{round}.db{side}"));
        }
      }
    }
    children
      .into_iter()
      .for_each(|mut child| assert!(child.wait().unwrap().success()));
    eprintln!(
      "rounds {rounds}: side files left {}, in memory {memory}, warnings {warnings}",
      left.len()
    );
    assert_eq!((left, memory, warnings), (Vec::<String>::new(), 0, 0));
  }

  /// [`WalCheck`] makes nothing: a file removed while another process keeps its `-wal` is not made again (no
  /// `SQLITE_OPEN_CREATE`), a file it cannot write is left with no Warning, and a file in another journal mode, which
  /// has no `-wal` (WAL refused: a network filesystem, WSL's `/mnt`), is not opened.
  #[cfg(unix)]
  #[test]
  fn a_close_check_makes_no_file() {
    use std::os::unix::fs::PermissionsExt;

    use latexml_core::common::error::{LogStatus, get_status};
    let side = |file: &Path, name: &str| {
      let mut side = file.as_os_str().to_owned();
      side.push(name);
      PathBuf::from(side)
    };
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("c.db");
    let mut store = SqliteStore::open(Some(&file), SCHEMA, 1);
    store.immediate(|tx, _| tx.execute("INSERT INTO t(v) VALUES (1)", []));
    std::fs::remove_file(&file).unwrap();
    assert!(side(&file, "-wal").exists());
    drop(WalCheck(Some(file.clone())));
    assert!(!file.exists());
    drop(store);
    assert!(!file.exists());

    let warnings = get_status(LogStatus::Warning);
    let file = dir.path().join("r.db");
    let mut store = SqliteStore::open(Some(&file), SCHEMA, 1);
    store.immediate(|tx, _| tx.execute("INSERT INTO t(v) VALUES (1)", []));
    let holder = Connection::open(&file).unwrap();
    holder
      .query_row("SELECT count(*) FROM t", [], |row| row.get::<_, i64>(0))
      .unwrap();
    drop(store);
    std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o444)).unwrap();
    std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o555)).unwrap();
    drop(WalCheck(Some(file.clone())));
    std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o755)).unwrap();
    assert_eq!(get_status(LogStatus::Warning) - warnings, 0);
    drop(holder);

    let file = dir.path().join("d.db");
    let conn = Connection::open(&file).unwrap();
    conn.execute_batch(SCHEMA).unwrap();
    drop(conn);
    drop(WalCheck(Some(file.clone())));
    assert!(!side(&file, "-wal").exists() && !side(&file, "-shm").exists());
  }

  /// A file another process holds locked against readers (a `DELETE` journal under `BEGIN EXCLUSIVE` with a pending
  /// write) past the 10 s lock wait: the store's transaction falls back to memory once that wait is over, with no copy
  /// of the file's entries (round 7 copied with `run_to_completion`, which retries a busy source for ever), one
  /// Warning, and the earlier entries lost.
  #[test]
  fn a_file_locked_against_readers_falls_back_without_its_entries() {
    use latexml_core::common::error::{LogStatus, get_status};
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("c.db");
    let mut store = SqliteStore::open(Some(&file), SCHEMA, 1);
    store.immediate(|tx, _| tx.execute("INSERT INTO t(v) VALUES (7)", []));
    store
      .conn
      .pragma_update(None, "journal_mode", "DELETE")
      .unwrap();
    let holder = Connection::open(&file).unwrap();
    holder
      .execute_batch("BEGIN EXCLUSIVE; INSERT INTO t(v) VALUES (9);")
      .unwrap();
    let (send, receive) = std::sync::mpsc::channel();
    let start = std::time::Instant::now();
    let worker = std::thread::spawn(move || {
      let warnings = get_status(LogStatus::Warning);
      let inserted = store.immediate(|tx, _| tx.execute("INSERT INTO t(v) VALUES (1)", []));
      let kept = store.read(|conn| {
        conn
          .prepare("SELECT v FROM t ORDER BY v")?
          .query_map([], |row| row.get::<_, i64>(0))?
          .collect::<rusqlite::Result<Vec<_>>>()
      });
      let counts = (
        get_status(LogStatus::Warning) - warnings,
        get_status(LogStatus::Error),
      );
      send
        .send((inserted, kept, counts, store.file().is_none()))
        .unwrap();
    });
    let outcome = receive.recv_timeout(std::time::Duration::from_secs(25));
    let took = start.elapsed();
    holder.execute_batch("COMMIT").unwrap();
    worker.join().unwrap();
    assert_eq!(
      outcome.expect("the store fell back within 25 s"),
      (Some(1), Some(vec![1]), (1, 0), true)
    );
    // One busy wait, not two: 10 s on Linux. On macOS CI the same wait measured 18.2-18.3 s twice; the suspected
    // cause, not yet confirmed on a Mac, is that SQLite's busy handler counts its short sleeps by their nominal
    // lengths and macOS lengthens them (timer coalescing). So the bound there only catches a wait that never ends.
    let bound = if cfg!(target_os = "macos") { 24 } else { 16 };
    assert!(took < std::time::Duration::from_secs(bound), "{took:?}");
  }

  /// A cache file that is a link is never followed (issue 995 review, round 7: SQLite made the database the link led
  /// to, out of the folder, with no Warning), nor a directory opened: the store runs in memory. A planted `-wal` link
  /// beside a new file is not followed either (SQLite opens the side files `O_NOFOLLOW`): nothing is made out of the
  /// folder, and the store keeps its entries.
  #[cfg(unix)]
  #[test]
  fn a_link_is_never_followed() {
    let dir = tempfile::tempdir().unwrap();
    let outside = dir.path().join("outside");
    std::fs::create_dir(&outside).unwrap();
    let file = dir.path().join("c.db");
    std::os::unix::fs::symlink(outside.join("o.db"), &file).unwrap();
    let store = SqliteStore::open(Some(&file), SCHEMA, 1);
    assert_eq!((store.file(), store.fell_back()), (None, true));
    let folder = dir.path().join("d.db");
    std::fs::create_dir(&folder).unwrap();
    let store = SqliteStore::open(Some(&folder), SCHEMA, 1);
    assert_eq!((store.file(), store.fell_back()), (None, true));
    let file = dir.path().join("e.db");
    std::os::unix::fs::symlink(outside.join("w"), dir.path().join("e.db-wal")).unwrap();
    let mut store = SqliteStore::open(Some(&file), SCHEMA, 1);
    store.immediate(|tx, _| tx.execute("INSERT INTO t(v) VALUES (1)", []));
    assert_eq!(
      store.read(|conn| conn.query_row("SELECT count(*) FROM t", [], |row| row.get::<_, i64>(0))),
      Some(1)
    );
    drop(store);
    assert_eq!(std::fs::read_dir(&outside).unwrap().count(), 0);
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
