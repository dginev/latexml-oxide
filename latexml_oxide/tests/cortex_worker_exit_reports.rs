//! `cortex_worker --harness` returns a `Status:conversion:3` result, naming the cause, for the task
//! of a child that dies mid-conversion (`bin/cortex_worker/exit_report.rs`).
//!
//! Each case runs a one-worker harness against an in-test dispatcher (a ROUTER ventilator that
//! leases one task, a PULL sink) and makes the child die as production children do: the watchdog's
//! timeout, an allocation past `RLIMIT_AS`, a SIGSEGV, the governor's SIGTERM, a SIGKILL. The
//! faults come from `CORTEX_WORKER_TEST_FAULT`, compiled into debug-assertion builds only. The
//! report must arrive under the child's own worker identity (the dispatcher's attempt key), carry
//! the child's partial log, and end in the cause's Fatal line and `Status:conversion:3`. And a
//! child that returns its own result, then exits cleanly, must get no second, harness, result.
#![cfg(debug_assertions)]

use std::{
  fs,
  io::Read,
  path::{Path, PathBuf},
  process::{Child, Command, Stdio},
  sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
  },
  thread,
  time::{Duration, Instant},
};

const TASK_ID: &str = "4242";
const SERVICE: &str = "exit_report_test";

/// What the sink received for the leased task.
struct Report {
  identity: String,
  service:  String,
  taskid:   String,
  log:      String,
  status:   String,
}

/// An in-test dispatcher: leases [`TASK_ID`] to the first request, mock-replies to the rest, and
/// collects what the sink receives.
struct Dispatcher {
  source:      String,
  sink:        String,
  leased_to:   Arc<std::sync::Mutex<Option<String>>>,
  stop:        Arc<AtomicBool>,
  ventilator:  Option<thread::JoinHandle<()>>,
  sink_socket: zmq::Socket,
  _context:    zmq::Context,
}

impl Dispatcher {
  fn start() -> Self {
    let context = zmq::Context::new();
    let router = context.socket(zmq::ROUTER).unwrap();
    router.bind("tcp://127.0.0.1:*").unwrap();
    let source = router.get_last_endpoint().unwrap().unwrap();
    let sink_socket = context.socket(zmq::PULL).unwrap();
    sink_socket.bind("tcp://127.0.0.1:*").unwrap();
    let sink = sink_socket.get_last_endpoint().unwrap().unwrap();
    router.set_rcvtimeo(200).unwrap();
    let leased_to = Arc::new(std::sync::Mutex::new(None));
    let stop = Arc::new(AtomicBool::new(false));
    let (leased, stopping) = (Arc::clone(&leased_to), Arc::clone(&stop));
    let ventilator = thread::spawn(move || {
      while !stopping.load(Ordering::Acquire) {
        let Ok(request) = router.recv_multipart(0) else {
          continue;
        };
        let identity = request[0].clone();
        let mut leased = leased.lock().unwrap();
        if leased.is_none() {
          *leased = Some(String::from_utf8_lossy(&identity).into_owned());
          router
            .send_multipart([identity, TASK_ID.as_bytes().to_vec(), task_archive()], 0)
            .unwrap();
        } else {
          router
            .send_multipart([identity, b"0".to_vec(), Vec::new()], 0)
            .unwrap();
        }
      }
    });
    Dispatcher {
      source,
      sink,
      leased_to,
      stop,
      ventilator: Some(ventilator),
      sink_socket,
      _context: context,
    }
  }

  /// The next result the sink receives, within `timeout`.
  fn receive(&self, timeout: Duration) -> Option<Report> {
    self
      .sink_socket
      .set_rcvtimeo(timeout.as_millis() as i32)
      .unwrap();
    let frames = self.sink_socket.recv_multipart(0).ok()?;
    assert_eq!(frames.len(), 4, "one data frame");
    let text = |i: usize| String::from_utf8_lossy(&frames[i]).into_owned();
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(frames[3].clone())).unwrap();
    let mut member = |name: &str| {
      let mut text = String::new();
      archive
        .by_name(name)
        .unwrap()
        .read_to_string(&mut text)
        .unwrap();
      text
    };
    Some(Report {
      identity: text(0),
      service:  text(1),
      taskid:   text(2),
      log:      member("cortex.log"),
      status:   member("status"),
    })
  }
}

impl Drop for Dispatcher {
  fn drop(&mut self) {
    self.stop.store(true, Ordering::Release);
    if let Some(handle) = self.ventilator.take() {
      let _ = handle.join();
    }
  }
}

/// A small task archive (the faults fire before it is unpacked).
fn task_archive() -> Vec<u8> {
  let mut bytes = std::io::Cursor::new(Vec::new());
  let mut zip = zip::ZipWriter::new(&mut bytes);
  zip
    .start_file("main.tex", zip::write::SimpleFileOptions::default())
    .unwrap();
  std::io::Write::write_all(
    &mut zip,
    b"\\documentclass{article}\\begin{document}x\\end{document}",
  )
  .unwrap();
  zip.finish().unwrap();
  bytes.into_inner()
}

/// A running harness, terminated (SIGTERM, then reaped) when dropped.
struct Harness {
  child: Child,
  tmp:   tempfile::TempDir,
  log:   PathBuf,
}

impl Harness {
  fn start(
    dispatcher: &Dispatcher,
    fault: &str,
    timeout_secs: u64,
    mem_limit_mb: u64,
    extra: &[&str],
  ) -> Self {
    let tmp = tempfile::tempdir().unwrap();
    let log = tmp.path().join("harness.log");
    let child = Command::new(env!("CARGO_BIN_EXE_cortex_worker"))
      // The governor on (a SIGTERM is its shed), at a floor no host reaches.
      .args([
        "--harness",
        "--workers",
        "1",
        "--mem-pressure-floor-mb",
        "1",
      ])
      .args(["--service", SERVICE])
      .args(["--source-address", &dispatcher.source])
      .args(["--sink-address", &dispatcher.sink])
      .args(["--timeout", &timeout_secs.to_string()])
      .args(["--child-mem-limit-mb", &mem_limit_mb.to_string()])
      .args(extra)
      .env("CORTEX_WORKER_TEST_FAULT", fault)
      .env("TMPDIR", tmp.path())
      .stdin(Stdio::null())
      .stdout(Stdio::null())
      .stderr(fs::File::create(&log).unwrap())
      .spawn()
      .unwrap();
    Harness { child, tmp, log }
  }

  /// The pid of the child converting [`TASK_ID`], once its claim names the task and its log mirror
  /// has the fault's warning (so a signal now ends a conversion that has a partial log).
  fn converting_child(&self, within: Duration) -> Option<i32> {
    let deadline = Instant::now() + within;
    while Instant::now() < deadline {
      if let Some(dir) = self.exit_report_dir() {
        for entry in fs::read_dir(&dir).into_iter().flatten().flatten() {
          let name = entry.file_name().to_string_lossy().into_owned();
          if let Some(pid) = name
            .strip_prefix("child-")
            .and_then(|n| n.strip_suffix(".claim"))
            && fs::read_to_string(entry.path())
              .unwrap_or_default()
              .contains(&format!("task {TASK_ID}\n"))
            && fs::read_to_string(dir.join(format!("child-{pid}.log")))
              .unwrap_or_default()
              .contains("test_fault")
          {
            // `child-<pid>-<nonce>`
            return pid.split('-').next().and_then(|pid| pid.parse().ok());
          }
        }
      }
      thread::sleep(Duration::from_millis(50));
    }
    None
  }

  fn output(&self) -> String { fs::read_to_string(&self.log).unwrap_or_default() }

  /// The harness's exit-report directory (the only `cortex_harness_*` in its private TMPDIR).
  fn exit_report_dir(&self) -> Option<PathBuf> {
    fs::read_dir(self.tmp.path())
      .ok()?
      .flatten()
      .find(|e| {
        e.file_name()
          .to_string_lossy()
          .starts_with("cortex_harness_")
      })
      .map(|e| e.path())
  }
}

impl Drop for Harness {
  fn drop(&mut self) {
    // SAFETY: signalling the harness this test spawned.
    unsafe {
      libc::kill(self.child.id() as i32, libc::SIGTERM);
    }
    let deadline = Instant::now() + Duration::from_secs(30);
    while Instant::now() < deadline {
      if let Ok(Some(_)) = self.child.try_wait() {
        return;
      }
      thread::sleep(Duration::from_millis(50));
    }
    let _ = self.child.kill();
    let _ = self.child.wait();
  }
}

/// Run one case: start a harness whose child meets `fault`, optionally signal the converting child,
/// and return the report the sink receives with the harness's output (for failure messages).
fn report_for(
  fault: &str,
  timeout_secs: u64,
  mem_limit_mb: u64,
  signal: Option<libc::c_int>,
) -> (Report, String, String, Harness) {
  let dispatcher = Dispatcher::start();
  let harness = Harness::start(&dispatcher, fault, timeout_secs, mem_limit_mb, &[]);
  if let Some(signal) = signal {
    let pid = harness
      .converting_child(Duration::from_secs(120))
      .unwrap_or_else(|| panic!("no child claimed the task:\n{}", harness.output()));
    // SAFETY: signalling the harness's child, as the governor or the kernel would.
    unsafe {
      libc::kill(pid, signal);
    }
  }
  let report = dispatcher
    .receive(Duration::from_secs(180))
    .unwrap_or_else(|| panic!("no result reached the sink:\n{}", harness.output()));
  let leased_to = dispatcher.leased_to.lock().unwrap().clone().unwrap();
  let output = harness.output();
  (report, leased_to, output, harness)
}

/// Whether a report carries CorTeX's retryable tag (user ruling 2026-10-12: a timeout, a shed
/// and an unrecorded death go back to the dispatcher's retry budget).
fn retry_tagged(report: &Report) -> bool {
  report
    .log
    .lines()
    .any(|line| line.starts_with("Info:cortex:retryable "))
}

/// The checks every report must pass, then its Fatal line.
fn fatal_of(report: &Report, leased_to: &str, output: &str) -> String {
  assert_eq!(report.taskid, TASK_ID, "{output}");
  assert_eq!(report.service, SERVICE, "{output}");
  assert_eq!(
    report.identity, leased_to,
    "the report must come under the identity the task was leased to (the dispatcher's attempt \
     key)\n{output}"
  );
  assert_eq!(report.status, "Status:conversion:3", "{output}");
  assert!(
    report.log.ends_with("\nStatus:conversion:3\n"),
    "the Fatal status is the last line:\n{}",
    report.log
  );
  assert!(
    report
      .log
      .contains("Warning:cortex_worker:test_fault injecting the test fault"),
    "the child's partial log rides along:\n{}",
    report.log
  );
  report
    .log
    .lines()
    .find(|line| line.starts_with("Fatal:"))
    .unwrap_or_else(|| panic!("no Fatal line:\n{}", report.log))
    .to_string()
}

#[test]
fn a_watchdog_timeout_is_reported_as_fatal_timeout() {
  let (report, leased_to, output, harness) = report_for("sleep", 3, 0, None);
  let fatal = fatal_of(&report, &leased_to, &output);
  assert!(
    retry_tagged(&report),
    "retry, then cause: a retryable cause is tagged\n{}",
    report.log
  );
  assert!(
    fatal.starts_with("Fatal:timeout:wallclock ") && fatal.contains("3s"),
    "{fatal}\n{output}"
  );
  // The reporter consumed the dead child's claim: none left names the task.
  thread::sleep(Duration::from_millis(500));
  let naming_the_task: Vec<_> = fs::read_dir(harness.exit_report_dir().expect("the directory"))
    .unwrap()
    .flatten()
    .filter(|e| {
      fs::read_to_string(e.path()).is_ok_and(|t| t.contains(&format!("task {TASK_ID}\n")))
    })
    .map(|e| e.file_name())
    .collect();
  assert!(
    naming_the_task.is_empty(),
    "claims left naming the task: {naming_the_task:?}"
  );
}

#[test]
fn an_allocation_past_rlimit_as_is_reported_as_fatal_oom() {
  let (report, leased_to, output, _harness) = report_for("alloc", 60, 4096, None);
  let fatal = fatal_of(&report, &leased_to, &output);
  assert!(
    !retry_tagged(&report),
    "retry, then cause: a final cause is not tagged\n{}",
    report.log
  );
  assert!(
    fatal.starts_with("Fatal:oom:alloc_failed ") && fatal.contains("1099511627776 bytes"),
    "{fatal}\n{output}"
  );
}

#[test]
fn a_segfault_is_reported_with_its_signal() {
  let (report, leased_to, output, _harness) = report_for("segv", 60, 0, None);
  let fatal = fatal_of(&report, &leased_to, &output);
  assert!(
    !retry_tagged(&report),
    "retry, then cause: a final cause is not tagged\n{}",
    report.log
  );
  assert!(
    fatal.starts_with("Fatal:crash:SIGSEGV "),
    "{fatal}\n{output}"
  );
}

#[test]
fn a_governor_shed_is_reported_as_fatal_oom_shed() {
  let (report, leased_to, output, _harness) = report_for("sleep", 120, 0, Some(libc::SIGTERM));
  let fatal = fatal_of(&report, &leased_to, &output);
  assert!(
    retry_tagged(&report),
    "retry, then cause: a retryable cause is tagged\n{}",
    report.log
  );
  assert!(fatal.starts_with("Fatal:oom:shed "), "{fatal}\n{output}");
}

#[test]
fn a_stack_overflow_is_reported_as_a_segfault() {
  let (report, leased_to, output, _harness) = report_for("stack", 120, 0, None);
  let fatal = fatal_of(&report, &leased_to, &output);
  assert!(
    !retry_tagged(&report),
    "retry, then cause: a final cause is not tagged\n{}",
    report.log
  );
  assert!(
    fatal.starts_with("Fatal:crash:SIGSEGV "),
    "{fatal}\n{output}"
  );
}

/// A child that returns its own result (here the worker's Fatal for a conversion that failed
/// cleanly) and then exits cleanly (`--limit 1`) gets no second result from the harness.
#[test]
fn a_returned_result_and_a_clean_exit_report_nothing_more() {
  let dispatcher = Dispatcher::start();
  let harness = Harness::start(&dispatcher, "fail", 60, 0, &["--limit", "1"]);
  let report = dispatcher
    .receive(Duration::from_secs(180))
    .unwrap_or_else(|| panic!("no result reached the sink:\n{}", harness.output()));
  let output = harness.output();
  assert_eq!(report.taskid, TASK_ID, "{output}");
  assert!(
    report.log.contains("Fatal:conversion:caught")
      && !report.log.contains("reported by the harness"),
    "the worker's own result:\n{}",
    report.log
  );
  assert!(
    dispatcher.receive(Duration::from_secs(5)).is_none(),
    "a second result was sent for the task:\n{}",
    harness.output()
  );
  let naming_the_task: Vec<_> = harness
    .exit_report_dir()
    .and_then(|dir| fs::read_dir(dir).ok())
    .into_iter()
    .flatten()
    .flatten()
    .filter(|e| {
      fs::read_to_string(e.path()).is_ok_and(|t| t.contains(&format!("task {TASK_ID}\n")))
    })
    .map(|e| e.file_name())
    .collect();
  assert!(
    naming_the_task.is_empty(),
    "claims left naming the task: {naming_the_task:?}"
  );
}

#[test]
fn a_sigkill_is_reported_as_an_unrecorded_crash() {
  let (report, leased_to, output, _harness) = report_for("sleep", 120, 0, Some(libc::SIGKILL));
  let fatal = fatal_of(&report, &leased_to, &output);
  assert!(
    retry_tagged(&report),
    "retry, then cause: a retryable cause is tagged\n{}",
    report.log
  );
  assert!(
    fatal.starts_with("Fatal:crash:unrecorded "),
    "{fatal}\n{output}"
  );
}

/// A harness shutdown that ends a conversion is no verdict on the paper: nothing is reported (the
/// dispatcher re-leases the task), and the exit-report directory goes with the harness.
#[test]
fn a_shutdown_mid_conversion_reports_nothing() {
  let dispatcher = Dispatcher::start();
  let harness = Harness::start(&dispatcher, "sleep", 120, 0, &[]);
  assert!(
    harness.converting_child(Duration::from_secs(120)).is_some(),
    "{}",
    harness.output()
  );
  let dir = harness.exit_report_dir().expect("the directory");
  let output_path = harness.log.clone();
  drop(harness);
  let output = fs::read_to_string(output_path).unwrap_or_default();
  assert!(
    dispatcher.receive(Duration::from_secs(5)).is_none(),
    "a shutdown was reported as a result:\n{output}"
  );
  assert!(
    !Path::new(&dir).exists(),
    "the directory outlived the harness"
  );
}
