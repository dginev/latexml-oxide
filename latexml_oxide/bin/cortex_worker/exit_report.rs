//! Exit reports: the result of a harness child that dies mid-conversion.
//!
//! A harness child that the watchdog ends (exit 124), whose allocation fails under its
//! `RLIMIT_AS` or whose RSS passes the soft guard (exit 137), that the memory-pressure governor
//! sheds (SIGTERM), or that a signal kills (SIGSEGV, SIGBUS, SIGABRT, …) used to return nothing
//! for its task: the dispatcher re-leased it, and after the retry budget recorded
//! `Fatal:cortex:never_completed_with_retries` instead of the cause. The harness now returns a
//! `Status:conversion:3` result for that task itself, carrying the cause and the conversion's
//! partial log, through the same sink path as any other result. A retryable cause (a timeout, a
//! shed, an unrecorded death — [`retryable`]) is tagged `Info:cortex:retryable`, so the dispatcher
//! retries the task first and records the last such cause only once its budget is spent.
//!
//! **Protocol.** The harness makes a private directory and passes it to each child
//! (`--exit-report-dir`). A child keeps one *claim* file there, `child-<pid>-<nonce>.claim`, under
//! an exclusive `flock` for its whole life, and beside it `child-<pid>-<nonce>.log`, the mirror of the
//! conversion log being written ([`latexml_core::util::logger::set_log_mirror`]). The claim
//! names the worker identity and, while a conversion runs, its task id; it is rewritten when a
//! conversion begins and when it ends. A child that knows why it is exiting appends a `cause`
//! line first — the watchdog's pre-exit hook, the allocation-failure hook, and a handler for the
//! fatal signals, all without allocating — and a clean exit removes both files.
//!
//! The kernel drops the lock when the process ends, however it ends. The harness's reporter thread
//! polls the directory; a claim it can lock belongs to a dead child, and if that claim still names
//! a task, the reporter sends `[identity, service, taskid, result.zip]` to the sink and removes the
//! files. A SIGTERM (or a death with no recorded cause) while the harness itself is shutting down
//! is the shutdown, not a verdict on the paper, and is not reported: the dispatcher re-leases it.
//!
//! The window between a conversion's end and the result reaching the sink is not covered (the
//! claim is idle by then); there the dispatcher's lease reaper stays the net, as before.

use std::{
  fs::{self, File, OpenOptions},
  io::{self, Read, Seek, SeekFrom, Write},
  os::unix::io::AsRawFd,
  path::{Path, PathBuf},
  sync::{
    OnceLock,
    atomic::{AtomicI32, Ordering},
  },
  thread,
  time::Duration,
};

/// The bound on the mirrored conversion log beside a claim: past it the mirror is emptied and
/// restarted, so it keeps the most recent text of a runaway log.
const LOG_MIRROR_CAP_BYTES: u64 = 8 << 20;
/// The most of a dead child's partial log a report carries (its tail).
const PARTIAL_LOG_REPORT_BYTES: u64 = 4 << 20;
/// How old an unheld staging claim of a live pid must be before the reporter removes it.
const STAGING_GRACE: Duration = Duration::from_secs(10);
/// How often the reporter looks for dead children.
const REPORTER_POLL: Duration = Duration::from_millis(250);
/// How long the reporter waits before reporting a SIGTERM or an unrecorded death, so a shutdown
/// that is signalling the whole fleet at once is seen as one.
const SHUTDOWN_GRACE: Duration = Duration::from_secs(2);
/// The sink's send timeout and linger for a report, in milliseconds.
const SINK_TIMEOUT_MS: i32 = 30_000;

/// The signals a child records before they end it. SIGKILL cannot be caught; SIGTERM is the
/// memory-pressure governor's shed (and the harness's shutdown).
const RECORDED_SIGNALS: [libc::c_int; 6] = [
  libc::SIGTERM,
  libc::SIGSEGV,
  libc::SIGBUS,
  libc::SIGILL,
  libc::SIGFPE,
  libc::SIGABRT,
];

// ---------------------------------------------------------------------------------------------
// Child side
// ---------------------------------------------------------------------------------------------

/// The claim's descriptor (`-1` outside a harness child), for the hooks and signal handlers that
/// may neither allocate nor lock.
static CLAIM_FD: AtomicI32 = AtomicI32::new(-1);
/// The pid that owns the claim: a process forked from it (a subprocess between `fork` and `exec`,
/// which inherits the descriptor and the signal handlers) must not write the owner's cause.
static CLAIM_OWNER: AtomicI32 = AtomicI32::new(0);

/// A harness child's claim: the locked claim file and the log mirror beside it.
struct Claim {
  file:     File,
  path:     PathBuf,
  log_file: File,
  log_path: PathBuf,
}

static CLAIM: OnceLock<Claim> = OnceLock::new();

/// The file stem of a child's claim and log mirror: `child-<pid>-<nonce>`. The nonce (the child's
/// start time) keeps a reused pid from ever opening — and truncating — a dead child's claim the
/// reporter has not read yet.
fn child_stem(pid: u32) -> String {
  let nonce = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .map(|d| d.as_nanos())
    .unwrap_or_default();
  format!("child-{pid}-{nonce:x}")
}

/// The claim file of the child whose stem is `stem`, in `dir`.
fn claim_path(dir: &Path, stem: &str) -> PathBuf { dir.join(format!("{stem}.claim")) }

/// The log mirror beside the claim of the child whose stem is `stem`.
fn log_path(dir: &Path, stem: &str) -> PathBuf { dir.join(format!("{stem}.log")) }

/// The pid in a claim's stem (`child-<pid>-<nonce>`).
fn stem_pid(stem: &str) -> Option<u32> {
  stem.strip_prefix("child-")?.split('-').next()?.parse().ok()
}

/// Make this process a reporting harness child: create and lock its claim in `dir`, mirror the
/// conversion log beside it, and record the fatal signals. The claim is locked before it is
/// renamed into place, so the reporter never sees an unlocked claim of a live child.
pub fn install_child(dir: &Path) -> io::Result<()> {
  let pid = std::process::id();
  let stem = child_stem(pid);
  let path = claim_path(dir, &stem);
  let staging = dir.join(format!("{stem}.claim.new"));
  let file = OpenOptions::new()
    .read(true)
    .append(true)
    .create(true)
    .open(&staging)?;
  file.set_len(0)?;
  lock(&file, true)?;
  (&file).write_all(b"idle\n")?;
  fs::rename(&staging, &path)?;
  let log_path = log_path(dir, &stem);
  let log_file = OpenOptions::new()
    .append(true)
    .create(true)
    .open(&log_path)?;
  log_file.set_len(0)?;
  latexml_core::util::logger::set_log_mirror(Some(log_file.as_raw_fd()), LOG_MIRROR_CAP_BYTES);
  CLAIM_OWNER.store(pid as i32, Ordering::Release);
  CLAIM_FD.store(file.as_raw_fd(), Ordering::Release);
  install_signal_recorders();
  let _ = CLAIM.set(Claim { file, path, log_file, log_path });
  Ok(())
}

/// Rewrite the claim to `body` (the claim is opened for appending, so after the truncation the
/// write lands at its start).
fn rewrite_claim(claim: &Claim, body: &str) {
  let _ = claim.file.set_len(0);
  let _ = (&claim.file).write_all(body.as_bytes());
}

/// A claimed task: the claim names it until this is dropped.
#[must_use = "the claim on the task ends when this is dropped"]
pub struct ClaimedTask(());

impl Drop for ClaimedTask {
  fn drop(&mut self) { end_task(); }
}

/// A conversion of `taskid` begins on the worker `identity`: claim it until the returned guard is
/// dropped, and empty the log mirror.
pub fn claim_task(identity: &str, taskid: &str) -> ClaimedTask {
  begin_task(identity, taskid);
  ClaimedTask(())
}

/// Name `taskid`, converting on the worker `identity`, in the claim, and empty the log mirror.
fn begin_task(identity: &str, taskid: &str) {
  if let Some(claim) = CLAIM.get() {
    let started = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .map(|d| d.as_secs())
      .unwrap_or(0);
    rewrite_claim(
      claim,
      &format!(
        "identity {}\ntask {}\nstarted {started}\n",
        one_line(identity),
        one_line(taskid)
      ),
    );
    latexml_core::util::logger::reset_log_mirror();
  }
}

/// The conversion has its result: the claim no longer holds a task.
fn end_task() {
  if let Some(claim) = CLAIM.get() {
    rewrite_claim(claim, "idle\n");
  }
}

/// A clean exit: remove the claim and the log mirror, so the reporter has nothing to report.
pub fn remove_child_claim() {
  if let Some(claim) = CLAIM.get() {
    latexml_core::util::logger::set_log_mirror(None, 0);
    CLAIM_FD.store(-1, Ordering::Release);
    let _ = fs::remove_file(&claim.path);
    let _ = fs::remove_file(&claim.log_path);
    let _ = claim.log_file.set_len(0);
  }
}

/// Append a `cause` line to the claim, if this is a harness child: `cause <kind> <number>`.
/// Allocation-free and async-signal-safe (`write(2)` only), for the allocation-failure hook and
/// the signal handlers; `number` is omitted when `None`.
pub fn record_cause(kind: &[u8], number: Option<u64>) {
  let fd = CLAIM_FD.load(Ordering::Acquire);
  // SAFETY: `getpid` is async-signal-safe and cannot fail.
  if fd < 0 || unsafe { libc::getpid() } != CLAIM_OWNER.load(Ordering::Acquire) {
    return;
  }
  let mut line = [0u8; 96];
  let mut len = 0;
  for part in [b"cause ".as_slice(), kind] {
    let take = part.len().min(line.len() - len - 24);
    line[len..len + take].copy_from_slice(&part[..take]);
    len += take;
  }
  if let Some(mut n) = number {
    line[len] = b' ';
    len += 1;
    let mut digits = [0u8; 20];
    let mut count = 0;
    loop {
      digits[count] = b'0' + (n % 10) as u8;
      count += 1;
      n /= 10;
      if n == 0 {
        break;
      }
    }
    for digit in digits[..count].iter().rev() {
      line[len] = *digit;
      len += 1;
    }
  }
  line[len] = b'\n';
  len += 1;
  // SAFETY: a write from a stack buffer to the claim descriptor, which stays open for the life of
  // the process.
  unsafe {
    libc::write(fd, line.as_ptr().cast(), len);
  }
}

/// Append a `cause panic <detail>` line (a panic that will abort the process). Allocates; for the
/// panic hook only.
pub fn record_panic(detail: &str) {
  if let Some(claim) = CLAIM.get()
    && std::process::id() as i32 == CLAIM_OWNER.load(Ordering::Acquire)
  {
    let _ = (&claim.file).write_all(format!("cause panic {}\n", one_line(detail)).as_bytes());
  }
}

/// The handlers the recorders replaced, by [`RECORDED_SIGNALS`] index (zeroed = the default).
static mut PREVIOUS_ACTIONS: [Option<libc::sigaction>; RECORDED_SIGNALS.len()] =
  [None; RECORDED_SIGNALS.len()];

/// Install [`record_signal`] for each of [`RECORDED_SIGNALS`], keeping the handler it replaces
/// (Rust's own SIGSEGV/SIGBUS guard-page handler, which reports a stack overflow) to chain to.
fn install_signal_recorders() {
  for (index, signal) in RECORDED_SIGNALS.iter().enumerate() {
    // SAFETY: a zeroed `sigaction` is a valid value; `sigaction(2)` installs the handler and
    // returns the old one. `PREVIOUS_ACTIONS` is written here once per signal, at startup.
    unsafe {
      let mut action: libc::sigaction = std::mem::zeroed();
      action.sa_sigaction = record_signal as *const () as libc::sighandler_t;
      action.sa_flags = libc::SA_SIGINFO | libc::SA_ONSTACK;
      libc::sigemptyset(&mut action.sa_mask);
      let mut previous: libc::sigaction = std::mem::zeroed();
      if libc::sigaction(*signal, &action, &mut previous) == 0 {
        (*std::ptr::addr_of_mut!(PREVIOUS_ACTIONS))[index] = Some(previous);
      }
    }
  }
}

/// Record a fatal signal in the claim, run the handler it replaced (if any), then die of the
/// signal with its default action. Async-signal-safe: `write`, `sigaction`, `raise`.
extern "C" fn record_signal(
  signal: libc::c_int,
  info: *mut libc::siginfo_t,
  context: *mut libc::c_void,
) {
  record_cause(b"signal", Some(signal as u64));
  let index = RECORDED_SIGNALS.iter().position(|s| *s == signal);
  // SAFETY: reads the table filled at startup; calls the replaced handler with the arguments the
  // kernel gave this one, as its flags declare it takes them.
  unsafe {
    if let Some(Some(previous)) = index.map(|i| (*std::ptr::addr_of!(PREVIOUS_ACTIONS))[i]) {
      let handler = previous.sa_sigaction;
      if handler != libc::SIG_DFL && handler != libc::SIG_IGN {
        if previous.sa_flags & libc::SA_SIGINFO != 0 {
          let handler: extern "C" fn(libc::c_int, *mut libc::siginfo_t, *mut libc::c_void) =
            std::mem::transmute(handler);
          handler(signal, info, context);
        } else {
          let handler: extern "C" fn(libc::c_int) = std::mem::transmute(handler);
          handler(signal);
        }
      }
    }
    libc::signal(signal, libc::SIG_DFL);
    libc::raise(signal);
  }
}

// ---------------------------------------------------------------------------------------------
// Harness side
// ---------------------------------------------------------------------------------------------

/// Why a child ended mid-conversion, from the first `cause` line of its claim.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExitCause {
  /// The watchdog's wall-clock timeout (exit 124).
  Timeout { secs: Option<u64> },
  /// The watchdog's resident-memory guard (exit 137).
  Rss { ceiling_mb: Option<u64> },
  /// An allocation failed — the `RLIMIT_AS` address-space cap (exit 137).
  AllocFailed { bytes: Option<u64> },
  /// A panic in a `panic = "abort"` build.
  Panic { detail: String },
  /// A fatal signal: the first recorded, then any that followed it.
  Signal { first: i32, then: Vec<i32> },
  /// No cause recorded: an uncatchable SIGKILL, or an exit from inside a library.
  Unrecorded,
}

/// A dead child's claim, parsed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeadClaim {
  /// The worker identity the conversion ran under (the dispatcher's attempt key).
  pub identity: Option<String>,
  /// The task the child was converting, if any.
  pub task:     Option<String>,
  /// Why it ended.
  pub cause:    ExitCause,
}

impl DeadClaim {
  /// Parse a claim file's text.
  pub fn parse(text: &str) -> Self {
    let mut identity = None;
    let mut task = None;
    let mut first: Option<ExitCause> = None;
    let mut later_signals = Vec::new();
    for line in text.lines() {
      let (key, rest) = line.split_once(' ').unwrap_or((line, ""));
      match key {
        "identity" => identity = Some(rest.to_string()),
        "task" => task = Some(rest.to_string()),
        "cause" => {
          let (kind, value) = rest.split_once(' ').unwrap_or((rest, ""));
          let number = value.trim().parse::<u64>().ok();
          let cause = match kind {
            "timeout" => ExitCause::Timeout { secs: number },
            "rss" => ExitCause::Rss { ceiling_mb: number },
            "alloc" => ExitCause::AllocFailed { bytes: number },
            "panic" => ExitCause::Panic { detail: value.to_string() },
            "signal" => match number {
              Some(n) => ExitCause::Signal {
                first: n as i32,
                then:  Vec::new(),
              },
              None => continue,
            },
            _ => continue,
          };
          match (&first, &cause) {
            (None, _) => first = Some(cause),
            (Some(_), ExitCause::Signal { first: n, .. }) => later_signals.push(*n),
            _ => {},
          }
        },
        _ => {},
      }
    }
    let cause = match first {
      Some(ExitCause::Signal { first, .. }) => ExitCause::Signal { first, then: later_signals },
      Some(cause) => cause,
      None => ExitCause::Unrecorded,
    };
    DeadClaim { identity, task, cause }
  }

  /// Whether the death may be the harness's own shutdown (which signals every child with SIGTERM),
  /// rather than a verdict on the paper.
  fn may_be_shutdown(&self) -> bool {
    matches!(
      self.cause,
      ExitCause::Unrecorded | ExitCause::Signal { first: libc::SIGTERM, .. }
    )
  }
}

/// The name of a signal number, `SIG<NAME>` (or `signal<N>`).
fn signal_name(signal: i32) -> String {
  match signal {
    libc::SIGTERM => "SIGTERM".into(),
    libc::SIGSEGV => "SIGSEGV".into(),
    libc::SIGBUS => "SIGBUS".into(),
    libc::SIGILL => "SIGILL".into(),
    libc::SIGFPE => "SIGFPE".into(),
    libc::SIGABRT => "SIGABRT".into(),
    libc::SIGKILL => "SIGKILL".into(),
    n => format!("signal{n}"),
  }
}

/// The `Fatal:<category>:<what> <message>` line for a cause, in the worker's existing vocabulary
/// (`Fatal:timeout:wallclock`, `Fatal:oom:rss`, `Fatal:oom:alloc_failed`, as the child itself
/// prints them). A SIGTERM is the memory-pressure governor's shed only when `governor` is on.
pub fn fatal_line(cause: &ExitCause, pid: u32, governor: bool) -> String {
  let reported = format!("(worker pid {pid}; reported by the harness)");
  match cause {
    ExitCause::Timeout { secs } => format!(
      "Fatal:timeout:wallclock latexml-oxide hit the {} main-level wall-clock timeout; the \
       watchdog ended the worker process (exit 124) {reported}",
      secs.map_or("per-document".to_string(), |s| format!("{s}s"))
    ),
    ExitCause::Rss { ceiling_mb } => format!(
      "Fatal:oom:rss latexml-oxide: resident memory exceeded the {} ceiling; the watchdog ended \
       the worker process (exit 137) {reported}",
      ceiling_mb.map_or("worker's".to_string(), |mb| format!("{mb}MB"))
    ),
    ExitCause::AllocFailed { bytes } => format!(
      "Fatal:oom:alloc_failed an allocation{} failed under the worker's address-space limit \
       (RLIMIT_AS); the worker process exited (137) {reported}",
      bytes.map_or(String::new(), |b| format!(" of {b} bytes"))
    ),
    ExitCause::Panic { detail } => format!(
      "Fatal:panic:abort the worker panicked in a panic=abort build: {} {reported}",
      one_line(detail)
    ),
    ExitCause::Signal { first: libc::SIGTERM, .. } if governor => format!(
      "Fatal:oom:shed the worker process was terminated (SIGTERM) while the harness was running: \
       the harness's memory-pressure governor sheds the largest-RSS worker this way when free \
       memory falls below its floor {reported}"
    ),
    ExitCause::Signal { first, then } => {
      let followed = if then.is_empty() {
        String::new()
      } else {
        let names: Vec<String> = then.iter().map(|s| signal_name(*s)).collect();
        let overflow = if *first == libc::SIGSEGV && then.first() == Some(&libc::SIGABRT) {
          " — the signature of a stack overflow, which Rust's guard-page handler aborts"
        } else {
          ""
        };
        format!(", then {}{overflow}", names.join(", "))
      };
      format!(
        "Fatal:crash:{} the worker process was killed by signal {first} ({}){followed} \
         {reported}",
        signal_name(*first),
        signal_name(*first)
      )
    },
    ExitCause::Unrecorded => format!(
      "Fatal:crash:unrecorded the worker process ended mid-conversion without recording a cause: \
       an uncatchable SIGKILL (the kernel OOM killer, the harness's unresponsive-worker \
       watchdog, an operator) or an exit from inside a library; the harness log has its exit \
       status {reported}"
    ),
  }
}

/// Whether a cause is **retryable** (user ruling 2026-10-12, "retry, then cause"): a watchdog
/// timeout, a governor shed and an unrecorded death may not repeat on another try, so the
/// dispatcher returns the task to its retry budget, reporting the last such cause only when the
/// budget is spent. A crash by a signal and the memory limits (RSS, `RLIMIT_AS`) are final at once.
pub fn retryable(cause: &ExitCause, governor: bool) -> bool {
  match cause {
    ExitCause::Timeout { .. } | ExitCause::Unrecorded => true,
    ExitCause::Signal { first: libc::SIGTERM, .. } => governor,
    _ => false,
  }
}

/// The `cortex.log` of a report: the child's partial log (without any `Status:` line, so the
/// verdict below is the only one), the CorTeX retryable tag for a retryable cause
/// (`Info:cortex:retryable <category>:<what>`), the cause, and the Fatal status.
pub fn report_log(partial_log: &str, fatal: &str, retryable: bool) -> String {
  let mut log = String::with_capacity(partial_log.len() + fatal.len() + 80);
  // The partial log is the document's: its `<severity>:cortex:` lines are defused (only the tag below
  // is the harness's), and its `Status:` lines dropped.
  for line in super::defuse_cortex_tags(partial_log).lines() {
    if !line.starts_with("Status:") {
      log.push_str(line);
      log.push('\n');
    }
  }
  if retryable {
    log.push_str("Info:cortex:retryable ");
    log.push_str(cause_what(fatal));
    log.push('\n');
  }
  log.push_str(fatal);
  log.push_str("\nStatus:conversion:3\n");
  log
}

/// The result archive of a report, in memory: the members of any failure archive.
pub fn report_archive(taskid: &str, cause_what: &str, log: &str) -> io::Result<Vec<u8>> {
  let telemetry = serde_json::json!({
    "paper_id": taskid,
    "category": "conversion_fatal",
    "exit_code": 3,
    "exit_cause": cause_what,
  });
  let mut bytes = io::Cursor::new(Vec::new());
  super::write_failure_archive(&mut bytes, log, &telemetry)?;
  Ok(bytes.into_inner())
}

/// The `<category>:<what>` of a Fatal line (`Fatal:oom:shed …` → `oom:shed`).
fn cause_what(fatal: &str) -> &str {
  fatal
    .strip_prefix("Fatal:")
    .and_then(|rest| rest.split(' ').next())
    .unwrap_or("unknown")
}

/// Take an exclusive `flock` on `file`, blocking or not. `Ok(false)`: another process holds it.
fn lock(file: &File, block: bool) -> io::Result<bool> {
  let operation = libc::LOCK_EX | if block { 0 } else { libc::LOCK_NB };
  // SAFETY: `flock(2)` on an open descriptor.
  if unsafe { libc::flock(file.as_raw_fd(), operation) } == 0 {
    return Ok(true);
  }
  let error = io::Error::last_os_error();
  if error.raw_os_error() == Some(libc::EWOULDBLOCK) {
    Ok(false)
  } else {
    Err(error)
  }
}

/// The last `max` bytes of the file at `path` (lossily decoded), from a line start.
fn read_tail(path: &Path, max: u64) -> String {
  let Ok(mut file) = File::open(path) else {
    return String::new();
  };
  let len = file.metadata().map(|m| m.len()).unwrap_or(0);
  let skip = len.saturating_sub(max);
  if file.seek(SeekFrom::Start(skip)).is_err() {
    return String::new();
  }
  let mut bytes = Vec::new();
  let _ = file.take(max).read_to_end(&mut bytes);
  let text = String::from_utf8_lossy(&bytes).into_owned();
  match (skip > 0, text.find('\n')) {
    (true, Some(newline)) => text[newline + 1..].to_string(),
    _ => text,
  }
}

/// A single line: newlines folded to spaces.
fn one_line(text: &str) -> String { text.replace(['\n', '\r'], " ") }

/// The prefix of a harness's exit-report directory in `$TMPDIR`.
const HARNESS_DIR_PREFIX: &str = "cortex_harness_";
/// The lock file a live harness holds in its directory.
const HARNESS_LOCK: &str = "harness.lock";
/// How old an unheld exit-report directory must be before it is removed: one just created by a
/// starting harness has no lock file yet, or one not locked yet.
const UNLOCKED_DIR_GRACE: Duration = Duration::from_secs(60);

/// A harness's exit-report directory: its own (a fresh, randomly named directory — never one
/// left by an earlier harness, whose stale claims it would report), held by a lock for the
/// harness's life so another harness's [`remove_stale_dirs`] leaves it alone.
pub struct HarnessDir {
  path:  PathBuf,
  _lock: File,
}

impl HarnessDir {
  /// Create and lock a new exit-report directory in `tmp`.
  pub fn create(tmp: &Path) -> io::Result<Self> {
    let path = tempfile::Builder::new()
      .prefix(HARNESS_DIR_PREFIX)
      .tempdir_in(tmp)?
      .keep();
    let lock_file = OpenOptions::new()
      .write(true)
      .create(true)
      .truncate(true)
      .open(path.join(HARNESS_LOCK))?;
    lock(&lock_file, true)?;
    Ok(HarnessDir { path, _lock: lock_file })
  }

  /// The directory.
  pub fn path(&self) -> &Path { &self.path }

  /// Remove the directory (the harness is stopping).
  pub fn remove(self) { let _ = fs::remove_dir_all(&self.path); }
}

/// Remove the exit-report directories in `tmp` that no live harness holds (a harness that was
/// killed, rather than stopped, leaves its directory, its children's claims and partial logs
/// behind). Nothing in them is reported: their tasks belong to a fleet that is gone, and the
/// dispatcher re-leases them.
pub fn remove_stale_dirs(tmp: &Path) {
  let Ok(entries) = fs::read_dir(tmp) else {
    return;
  };
  for entry in entries.flatten() {
    let is_harness_dir = entry
      .file_name()
      .to_str()
      .is_some_and(|n| n.starts_with(HARNESS_DIR_PREFIX));
    if !is_harness_dir || !entry.path().is_dir() {
      continue;
    }
    // Old enough either way: a starting harness creates its lock file a moment after its
    // directory, and locks it a moment after that.
    let old = entry
      .metadata()
      .and_then(|m| m.modified())
      .ok()
      .and_then(|modified| modified.elapsed().ok())
      .is_some_and(|age| age > UNLOCKED_DIR_GRACE);
    let stale = old
      && match File::open(entry.path().join(HARNESS_LOCK)) {
        Ok(lock_file) => lock(&lock_file, false).unwrap_or(false),
        Err(_) => true,
      };
    if stale {
      let _ = fs::remove_dir_all(entry.path());
    }
  }
}

/// The harness's reporter: polls the claims of `dir` and returns the result of every child that
/// died mid-conversion to the sink at `sink_address`, as `service`.
pub struct Reporter {
  dir:          PathBuf,
  service:      String,
  sink_address: String,
  governor:     bool,
  context:      zmq::Context,
  sink:         Option<zmq::Socket>,
}

impl Reporter {
  /// A reporter for the claims in `dir`; `governor` says whether the harness's memory-pressure
  /// governor is on (a SIGTERM is then its shed).
  pub fn new(dir: PathBuf, service: String, sink_address: String, governor: bool) -> Self {
    Reporter {
      dir,
      service,
      sink_address,
      governor,
      context: zmq::Context::new(),
      sink: None,
    }
  }

  /// Poll until `stopping()` turns true, then return.
  pub fn run(mut self, stopping: impl Fn() -> bool) {
    while !stopping() {
      self.scan_once(&stopping);
      thread::sleep(REPORTER_POLL);
    }
  }

  /// One pass over the claims: report and remove each dead child's.
  pub fn scan_once(&mut self, stopping: &impl Fn() -> bool) {
    let Ok(entries) = fs::read_dir(&self.dir) else {
      return;
    };
    for entry in entries.flatten() {
      let name = entry.file_name();
      // A staging claim left by a child that died before renaming it into place: remove it once
      // nobody holds it and its child is gone (or, a pid reused, it is old) — a live child creates
      // it a moment before it locks it.
      if let Some(stem) = name.to_str().and_then(|n| n.strip_suffix(".claim.new")) {
        let gone = stem_pid(stem).is_some_and(|pid| {
          // SAFETY: `kill(pid, 0)` only checks that the process exists.
          let alive = unsafe { libc::kill(pid as libc::pid_t, 0) } == 0;
          !alive && io::Error::last_os_error().raw_os_error() == Some(libc::ESRCH)
        });
        let old = entry
          .metadata()
          .and_then(|m| m.modified())
          .ok()
          .and_then(|modified| modified.elapsed().ok())
          .is_some_and(|age| age > STAGING_GRACE);
        if (gone || old)
          && let Ok(file) = File::open(entry.path())
          && matches!(lock(&file, false), Ok(true))
        {
          let _ = fs::remove_file(entry.path());
        }
        continue;
      }
      let Some((stem, pid)) = name
        .to_str()
        .and_then(|n| n.strip_suffix(".claim"))
        .and_then(|stem| Some((stem.to_string(), stem_pid(stem)?)))
      else {
        continue;
      };
      let path = entry.path();
      let Ok(file) = File::open(&path) else {
        continue;
      };
      match lock(&file, false) {
        Ok(true) => {},
        // Alive (or unreadable this pass): look again next time.
        _ => continue,
      }
      let mut text = String::new();
      let _ = (&file).read_to_string(&mut text);
      let claim = DeadClaim::parse(&text);
      if let Some(taskid) = claim.task.as_deref() {
        let shutdown = claim.may_be_shutdown() && {
          thread::sleep(SHUTDOWN_GRACE);
          stopping()
        };
        if shutdown {
          eprintln!(
            "exit report: worker pid {pid} ended during the harness shutdown with task {taskid} \
             in flight; not reported (the dispatcher re-leases it)"
          );
        } else {
          self.report(pid, &stem, &claim, taskid);
        }
      }
      let _ = fs::remove_file(log_path(&self.dir, &stem));
      let _ = fs::remove_file(&path);
    }
  }

  /// Send one dead child's result to the sink.
  fn report(&mut self, pid: u32, stem: &str, claim: &DeadClaim, taskid: &str) {
    let fatal = fatal_line(&claim.cause, pid, self.governor);
    let partial = read_tail(&log_path(&self.dir, stem), PARTIAL_LOG_REPORT_BYTES);
    let log = report_log(&partial, &fatal, retryable(&claim.cause, self.governor));
    let identity = claim.identity.clone().unwrap_or_default();
    let outcome = report_archive(taskid, cause_what(&fatal), &log)
      .map_err(|e| e.to_string())
      .and_then(|archive| self.send(&identity, taskid, &archive));
    match outcome {
      Ok(()) => eprintln!(
        "exit report: task {taskid} (worker pid {pid}, {identity}) returned as {}",
        cause_what(&fatal)
      ),
      Err(e) => eprintln!(
        "exit report: could not return task {taskid} (worker pid {pid}) to the sink ({e}); the \
         dispatcher's lease reaper recovers it"
      ),
    }
  }

  /// `[identity, service, taskid, archive]` to the sink, on the reporter's PUSH socket.
  fn send(&mut self, identity: &str, taskid: &str, archive: &[u8]) -> Result<(), String> {
    if self.sink.is_none() {
      let socket = self.context.socket(zmq::PUSH).map_err(|e| e.to_string())?;
      socket
        .set_sndtimeo(SINK_TIMEOUT_MS)
        .and_then(|()| socket.set_linger(SINK_TIMEOUT_MS))
        .and_then(|()| socket.connect(&self.sink_address))
        .map_err(|e| e.to_string())?;
      self.sink = Some(socket);
    }
    let sink = self.sink.as_ref().ok_or("no sink socket")?;
    let frames: [&[u8]; 4] = [
      identity.as_bytes(),
      self.service.as_bytes(),
      taskid.as_bytes(),
      archive,
    ];
    sink.send_multipart(frames, 0).map_err(|e| {
      // A timed-out send leaves the socket's state unknown: start over with a fresh one.
      self.sink = None;
      e.to_string()
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn a_claim_names_its_task_and_first_cause() {
    let claim = DeadClaim::parse(
      "identity host:svc:03\ntask 4242\nstarted 17\ncause timeout 180\ncause signal 6\n",
    );
    assert_eq!(claim.identity.as_deref(), Some("host:svc:03"));
    assert_eq!(claim.task.as_deref(), Some("4242"));
    assert_eq!(claim.cause, ExitCause::Timeout { secs: Some(180) });
  }

  #[test]
  fn an_idle_claim_has_no_task_and_no_cause() {
    let claim = DeadClaim::parse("idle\n");
    assert_eq!(claim.task, None);
    assert_eq!(claim.cause, ExitCause::Unrecorded);
    assert!(claim.may_be_shutdown());
  }

  #[test]
  fn later_signals_follow_the_first() {
    let claim = DeadClaim::parse("task 1\ncause signal 11\ncause signal 6\n");
    assert_eq!(claim.cause, ExitCause::Signal {
      first: libc::SIGSEGV,
      then:  vec![libc::SIGABRT],
    });
    let line = fatal_line(&claim.cause, 7, true);
    assert!(line.starts_with("Fatal:crash:SIGSEGV "), "{line}");
    assert!(line.contains("stack overflow"), "{line}");
  }

  #[test]
  fn each_cause_has_its_fatal_name() {
    let name = |cause: ExitCause| cause_what(&fatal_line(&cause, 1, true)).to_string();
    assert_eq!(
      name(ExitCause::Timeout { secs: Some(180) }),
      "timeout:wallclock"
    );
    assert_eq!(name(ExitCause::Rss { ceiling_mb: None }), "oom:rss");
    assert_eq!(
      name(ExitCause::AllocFailed { bytes: Some(8) }),
      "oom:alloc_failed"
    );
    assert_eq!(
      name(ExitCause::Signal {
        first: libc::SIGTERM,
        then:  vec![],
      }),
      "oom:shed"
    );
    assert_eq!(
      name(ExitCause::Signal {
        first: libc::SIGBUS,
        then:  vec![],
      }),
      "crash:SIGBUS"
    );
    assert_eq!(name(ExitCause::Panic { detail: "x".into() }), "panic:abort");
    assert_eq!(name(ExitCause::Unrecorded), "crash:unrecorded");
    // Retry, then cause: timeout, shed and unrecorded are retryable; crashes and the memory
    // limits are final.
    let shed = ExitCause::Signal {
      first: libc::SIGTERM,
      then:  vec![],
    };
    assert!(retryable(&ExitCause::Timeout { secs: None }, true));
    assert!(retryable(&ExitCause::Unrecorded, true));
    assert!(retryable(&shed, true));
    assert!(
      !retryable(&shed, false),
      "a SIGTERM without a governor is no shed"
    );
    assert!(!retryable(&ExitCause::Rss { ceiling_mb: None }, true));
    assert!(!retryable(&ExitCause::AllocFailed { bytes: None }, true));
    assert!(!retryable(
      &ExitCause::Signal {
        first: libc::SIGSEGV,
        then:  vec![libc::SIGABRT],
      },
      true
    ));
    let sigterm = ExitCause::Signal {
      first: libc::SIGTERM,
      then:  vec![],
    };
    assert_eq!(
      cause_what(&fatal_line(&sigterm, 1, false)),
      "crash:SIGTERM",
      "without a governor a SIGTERM is no shed"
    );
  }

  #[test]
  fn a_report_log_ends_with_its_single_fatal_verdict() {
    let log = report_log(
      "(Loading a.sty)\nWarning:x:y z\nStatus:conversion:1\n",
      "Fatal:crash:SIGSEGV gone",
      false,
    );
    assert_eq!(
      log,
      "(Loading a.sty)\nWarning:x:y z\nFatal:crash:SIGSEGV gone\nStatus:conversion:3\n"
    );
    // A document's own tag line in the partial log is defused: only the harness's tag counts.
    let log = report_log(
      "Info:cortex:retryable timeout:wallclock\n",
      "Fatal:crash:SIGSEGV gone",
      false,
    );
    assert_eq!(
      log,
      " Info:cortex:retryable timeout:wallclock\nFatal:crash:SIGSEGV gone\nStatus:conversion:3\n"
    );
    // ...at any severity CorTeX would read as one: a real SIGSEGV stays final.
    let log = report_log(
      "Note:cortex:retryable crash:SIGSEGV\nINFO:cortex:retryable crash:SIGSEGV\n",
      "Fatal:crash:SIGSEGV killed",
      false,
    );
    assert!(
      !log
        .lines()
        .any(|line| line.contains(":cortex:retryable") && !line.starts_with(' ')),
      "{log}"
    );
    // A retryable cause carries CorTeX's tag just before its Fatal line.
    let log = report_log("", "Fatal:oom:shed gone", true);
    assert_eq!(
      log,
      "Info:cortex:retryable oom:shed\nFatal:oom:shed gone\nStatus:conversion:3\n"
    );
  }

  #[test]
  fn a_dead_childs_staging_claim_is_removed_and_a_live_ones_kept() {
    let dir = tempfile::tempdir().unwrap();
    // A pid no process has (past the kernel's `pid_max`, at most 2^22).
    let dead = dir.path().join("child-99999999-a.claim.new");
    fs::write(&dead, b"idle\n").unwrap();
    // Our own pid, unlocked and fresh: a child about to lock it.
    let fresh = dir
      .path()
      .join(format!("child-{}-c.claim.new", std::process::id()));
    fs::write(&fresh, b"idle\n").unwrap();
    let live = dir.path().join("child-2-b.claim.new");
    let holder = OpenOptions::new()
      .read(true)
      .append(true)
      .create(true)
      .open(&live)
      .unwrap();
    assert!(lock(&holder, true).unwrap());
    Reporter::new(
      dir.path().to_path_buf(),
      "svc".into(),
      "tcp://127.0.0.1:1".into(),
      false,
    )
    .scan_once(&|| false);
    assert!(!dead.exists() && live.exists() && fresh.exists());
  }

  #[test]
  fn the_tail_of_a_long_log_starts_at_a_line() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("log");
    fs::write(&path, "first line\nsecond line\nthird\n").unwrap();
    assert_eq!(read_tail(&path, 14), "third\n");
    assert_eq!(read_tail(&path, 1000), "first line\nsecond line\nthird\n");
    assert_eq!(read_tail(&dir.path().join("missing"), 10), "");
  }

  #[test]
  fn a_held_claim_is_alive_and_a_released_one_is_dead() {
    // `flock` locks belong to an open file description, so a second `open` in this process
    // contends exactly as the reporter in another process does.
    let dir = tempfile::tempdir().unwrap();
    let path = claim_path(dir.path(), "child-99-1");
    let holder = OpenOptions::new()
      .read(true)
      .append(true)
      .create(true)
      .open(&path)
      .unwrap();
    assert!(lock(&holder, true).unwrap());
    let probe = File::open(&path).unwrap();
    assert!(!lock(&probe, false).unwrap(), "a held claim is alive");
    drop(holder);
    assert!(lock(&probe, false).unwrap(), "a released claim is dead");
  }

  #[test]
  fn only_the_directories_of_gone_harnesses_are_removed() {
    let tmp = tempfile::tempdir().unwrap();
    let live = HarnessDir::create(tmp.path()).unwrap();
    // A harness killed a while ago: its lock file is there, nobody holds it, and it is old.
    let killed = HarnessDir::create(tmp.path()).unwrap();
    let killed_path = killed.path().to_path_buf();
    drop(killed);
    let age = |dir: &Path, secs: u64| {
      File::open(dir)
        .unwrap()
        .set_modified(std::time::SystemTime::now() - Duration::from_secs(secs))
        .unwrap();
    };
    age(&killed_path, 600);
    // One just killed (or a harness that has not locked its file yet): kept for now.
    let fresh_unheld = HarnessDir::create(tmp.path()).unwrap();
    let fresh_unheld_path = fresh_unheld.path().to_path_buf();
    drop(fresh_unheld);
    // One just created by a starting harness, not locked yet: kept (within the grace).
    let starting = tmp.path().join("cortex_harness_starting");
    let other = tmp.path().join("cortex_output_1");
    for dir in [&starting, &other] {
      fs::create_dir(dir).unwrap();
    }
    remove_stale_dirs(tmp.path());
    assert!(live.path().is_dir() && starting.is_dir() && other.is_dir());
    assert!(
      fresh_unheld_path.is_dir(),
      "an unheld directory within the grace is kept"
    );
    assert!(!killed_path.exists());
    // An old live harness's directory is kept: its lock is held.
    age(live.path(), 600);
    remove_stale_dirs(tmp.path());
    assert!(live.path().is_dir());
    // Two harnesses never share a directory.
    let second = HarnessDir::create(tmp.path()).unwrap();
    assert_ne!(second.path(), live.path());
  }

  #[test]
  fn a_report_is_a_failure_archive_with_the_cause() {
    let log = report_log(
      "",
      &fatal_line(&ExitCause::Timeout { secs: Some(2) }, 5, true),
      false,
    );
    let bytes = report_archive("4242", "timeout:wallclock", &log).unwrap();
    let mut archive = zip::ZipArchive::new(io::Cursor::new(bytes)).unwrap();
    let mut read = |name: &str| {
      let mut text = String::new();
      archive
        .by_name(name)
        .unwrap()
        .read_to_string(&mut text)
        .unwrap();
      text
    };
    assert_eq!(read("status"), "Status:conversion:3");
    let log = read("cortex.log");
    assert!(log.starts_with("Fatal:timeout:wallclock "), "{log}");
    assert!(log.ends_with("\nStatus:conversion:3\n"), "{log}");
    assert!(read("telemetry.json").contains("\"exit_cause\":\"timeout:wallclock\""));
  }
}
