//! Unix server: warm-fork pipeline + single-threaded poll loop.
//! (Split out of the old monolithic `lsp_server.rs`; see `mod.rs`
//! for the architecture overview.)

use std::collections::VecDeque;
use std::io::{Read, Write};
use std::os::unix::io::FromRawFd;

use serde_json::Value;

use crate::converter::Converter;

use super::*;

use crate::core_interface::DigestionAPI;
use latexml_core::BoxOps;

/// Outcome of one warm-fork conversion attempt.
enum WarmResult {
  /// Conversion finished (possibly via in-process fallback).
  Done(ConvertOutput),
  /// A newer same-document request superseded this one; the preempting
  /// message has already been pushed onto the pending queue.
  Cancelled,
}

/// Buffered, `poll`-aware reader over a raw fd (stdin = 0). We read with
/// `libc::read` directly rather than `std::io::Stdin` so that `poll(2)`
/// readiness and our own user-space buffer stay consistent.
struct FdReader {
  fd:  i32,
  buf: Vec<u8>,
}

impl FdReader {
  fn new() -> Self { Self::from_fd(0) }

  /// Reader over an arbitrary fd — the framing logic is fd-agnostic; tests
  /// drive it over a `pipe(2)` instead of stdin.
  fn from_fd(fd: i32) -> Self { FdReader { fd, buf: Vec::new() } }

  /// One blocking `read` into the buffer; `Ok(0)` is EOF.
  fn fill(&mut self) -> std::io::Result<usize> {
    let mut tmp = [0u8; 8192];
    loop {
      let n = unsafe {
        libc::read(self.fd, tmp.as_mut_ptr() as *mut libc::c_void, tmp.len())
      };
      if n < 0 {
        let err = std::io::Error::last_os_error();
        if err.raw_os_error() == Some(libc::EINTR) {
          continue;
        }
        return Err(err);
      }
      if n == 0 {
        return Ok(0);
      }
      self.buf.extend_from_slice(&tmp[..n as usize]);
      return Ok(n as usize);
    }
  }

  /// Pull one complete frame out of the buffer, if present. A loop (not
  /// recursion) so a flood of malformed headers can't grow the stack.
  fn take_frame(&mut self) -> Option<String> {
    loop {
      let he = find_subseq(&self.buf, b"\r\n\r\n")?;
      let cl = parse_content_length(&self.buf[..he]);
      let body_start = he + 4;
      match cl {
        Some(cl) if self.buf.len() >= body_start + cl => {
          let body: Vec<u8> = self.buf[body_start..body_start + cl].to_vec();
          self.buf.drain(..body_start + cl);
          return Some(String::from_utf8_lossy(&body).into_owned());
        },
        // Malformed header with no parseable Content-Length: drop it and retry.
        None => {
          self.buf.drain(..body_start);
        },
        // Header present but body not fully arrived yet.
        Some(_) => return None,
      }
    }
  }

  /// Blocking read of the next complete message; `None` on EOF. Skips empty
  /// bodies.
  fn next_message(&mut self) -> Option<String> {
    loop {
      if let Some(frame) = self.take_frame() {
        if frame.is_empty() {
          continue;
        }
        return Some(frame);
      }
      match self.fill() {
        Ok(0) => return None,
        Ok(_) => continue,
        Err(_) => return None,
      }
    }
  }
}

fn find_subseq(haystack: &[u8], needle: &[u8]) -> Option<usize> {
  haystack.windows(needle.len()).position(|w| w == needle)
}

/// Number of live threads in this process (Linux: `/proc/self/task`
/// entries; elsewhere `0` = unknown, which disables the advisory check).
fn task_count() -> usize {
  std::fs::read_dir("/proc/self/task")
    .map(|entries| entries.count())
    .unwrap_or(0)
}

/// Thread count observed when the server started. The `latexml_oxide`
/// binary runs everything on a dedicated big-stack worker thread while
/// `main` parks in `join()` — so the baseline is normally 2, and that
/// parked `main` is fork-benign (a futex wait holds no allocator/logger
/// lock). What MUST hold is that serving never *grows* the thread count:
/// any thread spawned during a conversion (a future engine/post
/// `thread::spawn` that outlives its scope) could hold the malloc lock at
/// fork time and deadlock the child before its Watchdog arms.
static BASELINE_THREADS: std::sync::atomic::AtomicUsize =
  std::sync::atomic::AtomicUsize::new(0);

fn parse_content_length(header: &[u8]) -> Option<usize> {
  let s = std::str::from_utf8(header).ok()?;
  for line in s.split("\r\n") {
    let l = line.trim();
    if l.to_ascii_lowercase().starts_with("content-length:") {
      if let Some(v) = l.split(':').nth(1) {
        if let Ok(n) = v.trim().parse::<usize>() {
          return Some(n);
        }
      }
    }
  }
  None
}

/// Reap `pid`, returning its exit code; a signal death maps to the shell
/// convention `128 + signo` (so an OS OOM-killer SIGKILL reports as `137`,
/// same as the Watchdog's deliberate memory-ceiling exit — both mean "ran
/// out of memory" to `finish`). The shared `Watchdog` in the child exits
/// `124` on timeout / `137` on the memory ceiling.
fn reap(pid: i32) -> i32 {
  let mut status = 0i32;
  unsafe {
    libc::waitpid(pid, &mut status, 0);
    if libc::WIFEXITED(status) {
      libc::WEXITSTATUS(status)
    } else if libc::WIFSIGNALED(status) {
      128 + libc::WTERMSIG(status)
    } else {
      -1
    }
  }
}

/// Does this raw message supersede the in-flight conversion of the project
/// rooted at `current_root`? A newer `latexml/convert` OR
/// `didChange`/`didOpen` of ANY file in the same project preempts: every
/// trigger of a project converts the same root, so the in-flight result is
/// already stale and finishing it would only delay the newer request behind a
/// wasted compile.
fn preempts(body: &str, current_root: &std::path::Path) -> bool {
  match parse_json(body) {
    Ok(req) => super::message_doc_uri(&req)
      .map(|uri| {
        let path = get_file_path(&uri);
        same_project(current_root, std::path::Path::new(&path))
      })
      .unwrap_or(false),
    Err(_) => false,
  }
}

fn is_exit(body: &str) -> bool {
  parse_json(body)
    .ok()
    .and_then(|r| r.get("method").and_then(|m| m.as_str()).map(str::to_string))
    .as_deref()
    == Some("exit")
}

/// Block until the body child finishes or is preempted, multiplexing
/// `{stdin, child-pipe}` on a single thread.
///
/// The child self-guards its wall-clock and RAM budgets via the shared
/// [`latexml_core::watchdog::Watchdog`] (see `run_body_child`), exiting with a
/// distinct code on breach. So the parent doesn't poll resources — it waits
/// for the pipe to close and lets `finish` interpret the exit code. The only
/// active concern here is stdin: a newer same-document `latexml/convert`
/// preempts (SIGKILL) the in-flight child.
fn wait_for_child(
  pid: i32,
  read_fd: i32,
  current_root: &std::path::Path,
  reader: &mut FdReader,
  pending: &mut VecDeque<String>,
) -> WarmResult {
  // Owns `read_fd`; closes it on every return path.
  let mut pipe = unsafe { std::fs::File::from_raw_fd(read_fd) };

  loop {
    // 1. Handle every COMPLETE stdin frame already buffered. We never block
    //    waiting for the rest of a partial frame here — doing so (the old
    //    `next_message()` call) ignored the child pipe while blocked, and a
    //    client that stalls mid-frame while the child blocks writing a
    //    > pipe-capacity payload would deadlock all three parties.
    while let Some(body) = reader.take_frame() {
      if body.is_empty() {
        continue;
      }
      if is_exit(&body) {
        kill_and_reap(pid);
        std::process::exit(0);
      }
      if preempts(&body, current_root) {
        kill_and_reap(pid);
        pending.push_back(body);
        return WarmResult::Cancelled;
      }
      // Unrelated message (didClose, shutdown, a different document):
      // queue it for after this compile and keep waiting.
      pending.push_back(body);
    }

    // 2. Poll both fds.
    let mut fds = [
      libc::pollfd { fd: 0, events: libc::POLLIN, revents: 0 },
      libc::pollfd { fd: read_fd, events: libc::POLLIN, revents: 0 },
    ];
    let rc = unsafe { libc::poll(fds.as_mut_ptr(), fds.len() as libc::nfds_t, -1) };
    if rc < 0 {
      let err = std::io::Error::last_os_error();
      if err.raw_os_error() == Some(libc::EINTR) {
        continue;
      }
      // poll failed: fall back to a blocking drain of the child.
      let mut bytes = Vec::new();
      let _ = pipe.read_to_end(&mut bytes);
      let code = reap(pid);
      return finish(&bytes, code);
    }
    let pipe_ready = (fds[1].revents & (libc::POLLIN | libc::POLLHUP)) != 0;
    if pipe_ready {
      // The child writes its whole payload in one shot at the very end, so
      // pipe-readable means the compile is essentially done — drain & reap.
      let mut bytes = Vec::new();
      let _ = pipe.read_to_end(&mut bytes);
      let code = reap(pid);
      return finish(&bytes, code);
    }
    if (fds[0].revents & (libc::POLLIN | libc::POLLHUP)) != 0 {
      // 3. One non-blocking-equivalent fill; complete frames are handled at
      //    the top of the next iteration.
      match reader.fill() {
        // stdin EOF mid-compile: client gone — kill the child and stop.
        Ok(0) => {
          kill_and_reap(pid);
          return WarmResult::Cancelled;
        },
        Ok(_) => {},
        Err(_) => {
          kill_and_reap(pid);
          return WarmResult::Cancelled;
        },
      }
    }
  }
}

/// Terminate the in-flight body child: SIGTERM first (default disposition
/// kills it, but gives any future cleanup handler a chance), a short grace
/// for it to disappear, then SIGKILL, then reap. The child's graphics
/// subprocesses are protected separately via `PR_SET_PDEATHSIG` (see
/// `graphics::run_with_timeout`): when the child dies, the kernel reaps the
/// `setsid`-detached converter grandchildren that would otherwise be
/// orphaned with no watcher.
fn kill_and_reap(pid: i32) {
  unsafe {
    libc::kill(pid, libc::SIGTERM);
  }
  // Brief grace: poll for exit up to ~50 ms before escalating.
  for _ in 0..10 {
    let mut status = 0i32;
    let r = unsafe { libc::waitpid(pid, &mut status, libc::WNOHANG) };
    if r == pid {
      return; // already reaped
    }
    std::thread::sleep(std::time::Duration::from_millis(5));
  }
  unsafe {
    libc::kill(pid, libc::SIGKILL);
  }
  reap(pid);
}

/// Parse the child's pipe payload into a `ConvertOutput`. The parent owns the
/// preamble (warmup) log and the source-map context, so it merges them here.
/// `exit_code` is the child's exit status: a child hard-terminated by its
/// `Watchdog` writes no payload and exits [`watchdog::EXIT_TIMEOUT`] /
/// [`watchdog::EXIT_OOM`], which we map to a fatal result.
fn finish(bytes: &[u8], exit_code: i32) -> WarmResult {
  use latexml_core::watchdog::{EXIT_OOM, EXIT_TIMEOUT};
  // Threaded back through a thread-local set just before the fork; see
  // `run_warm`. We stash the preamble log in a cell to avoid widening this
  // function's signature through the poll machinery.
  let body_str = String::from_utf8_lossy(bytes).into_owned();
  match parse_json(&body_str) {
    Ok(payload) => {
      if let Some(err) = payload.get("error").and_then(|e| e.as_str()) {
        WarmResult::Done(ConvertOutput::error(format!("child error: {err}")))
      } else {
        let html = payload
          .get("html")
          .and_then(|h| h.as_str())
          .unwrap_or("")
          .to_string();
        let body_log = payload
          .get("log")
          .and_then(|l| l.as_str())
          .unwrap_or("")
          .to_string();
        let sources = match payload.get("sources") {
          Some(Value::Array(arr)) => arr
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect(),
          _ => Vec::new(),
        };
        let combined_log = format!("{}{}", PREAMBLE_LOG.with(|c| c.borrow().clone()), body_log);
        let diags = parse_log_diagnostics(&combined_log);
        // Engine status/code reported by the child (0/1/2/3).
        let status_code = payload.get("statusCode").and_then(|c| c.as_i64()).unwrap_or(0);
        let status = payload
          .get("status")
          .and_then(|s| s.as_str())
          .map(String::from)
          .unwrap_or_else(|| status_label(status_code).to_string());
        WarmResult::Done(ConvertOutput {
          html,
          log: combined_log,
          diags,
          sources,
          root: None, // attributed by convert_trigger
          status,
          status_code,
        })
      }
    },
    // No usable payload — the child was hard-terminated (its Watchdog exited
    // it for a resource breach the cooperative guards didn't catch) or
    // crashed. Map the exit code to a fatal result.
    Err(e) => {
      let out = match exit_code {
        EXIT_TIMEOUT => {
          ConvertOutput::failed("timeout", 3, "conversion timed out (hard wall-clock limit)".to_string())
        },
        EXIT_OOM => ConvertOutput::failed(
          "fatal",
          3,
          "conversion exceeded the memory ceiling".to_string(),
        ),
        0 => ConvertOutput::error(format!("child payload parse error: {e}")),
        c => ConvertOutput::error(format!("child exited unexpectedly (code {c})")),
      };
      WarmResult::Done(out)
    },
  }
}

thread_local! {
  /// Preamble (warmup) log for the conversion currently being assembled.
  /// Set in `run_warm` immediately before forking, read back in `finish`.
  static PREAMBLE_LOG: std::cell::RefCell<String> = const { std::cell::RefCell::new(String::new()) };
}

/// Fork a child that digests `body`, prepends the warm preamble's digested
/// boxes, builds the DOM and writes `{html, log, sources}` (or `{error}`)
/// back over a pipe. Returns `(pid, read_fd)` to the parent.
fn spawn_body_child(
  uri: &str,
  offset_lines: usize,
  body: &str,
  warmed: &latexml_core::digested::Digested,
  timeout_secs: u64,
  max_rss_kb: u64,
) -> Result<(i32, i32), String> {
  let mut fds = [0i32; 2];
  unsafe {
    if libc::pipe(fds.as_mut_ptr()) < 0 {
      return Err("pipe() failed".to_string());
    }
  }
  let (read_fd, write_fd) = (fds[0], fds[1]);

  // INVARIANT: no thread beyond the server-start baseline may be live at
  // fork time — an extra thread could hold the allocator (or logger) lock
  // mid-operation, and the forked child would deadlock on its first
  // allocation, BEFORE its Watchdog spawns, wedging the server on the pipe
  // forever. Today this holds: graphics workers are `thread::scope`-joined,
  // `run_with_timeout` is poll-based, and `Watchdog`s are created
  // child-side only. (The baseline itself is fork-benign: the binary's
  // `main` parks in `join()` on the worker thread that runs this server —
  // see BASELINE_THREADS.) The debug assertion catches any future
  // `thread::spawn` that silently breaks the invariant.
  #[cfg(debug_assertions)]
  {
    let baseline = BASELINE_THREADS.load(std::sync::atomic::Ordering::Relaxed);
    let now = task_count();
    debug_assert!(
      baseline == 0 || now == 0 || now <= baseline,
      "forking the body child with {now} live threads (server started with {baseline}) — fork-unsafe"
    );
  }

  let pid = unsafe { libc::fork() };
  if pid < 0 {
    unsafe {
      libc::close(read_fd);
      libc::close(write_fd);
    }
    return Err("fork() failed".to_string());
  }

  if pid == 0 {
    // ---- child ----
    unsafe {
      libc::close(read_fd);
      // Insurance: the child inherits the parent's stdout (fd 1) — the LSP
      // protocol stream. No engine path writes to stdout today, but a
      // single stray `println!` would corrupt the framing for every client.
      // Point the child's fd 1 at /dev/null; its payload goes over the
      // pipe and its logging over stderr.
      let devnull = libc::open(c"/dev/null".as_ptr(), libc::O_WRONLY);
      if devnull >= 0 {
        libc::dup2(devnull, 1);
        libc::close(devnull);
      }
    }
    let payload = run_body_child(uri, offset_lines, body, warmed, timeout_secs, max_rss_kb);
    let bytes = payload.to_string().into_bytes();
    let mut file = unsafe { std::fs::File::from_raw_fd(write_fd) };
    let _ = file.write_all(&bytes);
    let _ = file.flush();
    drop(file); // close write end → parent sees EOF
    std::process::exit(0);
  }

  // ---- parent ----
  unsafe {
    libc::close(write_fd);
  }
  Ok((pid, read_fd))
}

/// Child-side body compilation. State (definitions, mode, fonts, source-map
/// table) is inherited from the warm parent via copy-on-write, so we work
/// against a bare `Core` and the thread-local logger — **never**
/// `Converter::from_config`, whose `Core::new` calls `set_state` and would
/// wipe the inherited state (undefined `\par`, `lookup_font()` → None).
/// `offset_lines` blank lines are prepended to the body literal so the child
/// mouth's line counter is file-relative (fixes diagnostics/`data-sourcepos`
/// being off by the preamble length).
fn run_body_child(
  uri: &str,
  offset_lines: usize,
  body: &str,
  warmed: &latexml_core::digested::Digested,
  timeout_secs: u64,
  max_rss_kb: u64,
) -> Value {
  // Hard guard for this child, identical to cortex_worker's: a background
  // thread that exits the child (124 timeout / 137 OOM) if it blows through
  // its wall-clock or RAM budget — including native hangs (libxslt, Marpa)
  // the cooperative deadline can't see. The child self-terminates; the parent
  // reaps it (`finish` maps the exit code). Held for the whole conversion.
  let _watchdog = latexml_core::watchdog::Watchdog::with_limits(timeout_secs, max_rss_kb);
  let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
    // Re-arm the cooperative deadline FRESH from this child's clock. The child
    // inherits the parent's thread-local CONVERSION_DEADLINE via COW (set
    // during warm-up); without this the body would run against a stale (often
    // already-expired) deadline. Digest loops raise Fatal:Timeout (caught
    // below) for the graceful common case; the Watchdog above is the backstop.
    latexml_core::stomach::set_timeout(timeout_secs);
    // Bare Core over the inherited state — does NOT reset thread-local state.
    let mut core = latexml_core::Core {
      preload: make_config(uri).preload.unwrap_or_default(),
    };
    latexml_core::util::logger::bind_log();

    // Open the body as a *named* in-memory mouth (same path as the preamble)
    // so its locators are stampable user sources sharing tag 0. The
    // `offset_lines` blank lines make the body mouth's line counter
    // file-relative.
    let body_content = format!("{}{}", "\n".repeat(offset_lines), body);
    crate::converter::open_named_in_memory_mouth(&get_file_path(uri), body_content)
      .map_err(|e| format!("input error: {e}"))?;

    let body_digested = core
      .digest_internal()
      .map_err(|e| format!("digest error: {e}"))?;

    let mut combined = warmed.unlist();
    combined.extend(body_digested.unlist());
    let full = latexml_core::digested::Digested::from(latexml_core::list::List {
      boxes: combined,
      ..Default::default()
    });

    let dom = core
      .convert_document(full)
      .map_err(|e| format!("dom conversion error: {e}"))?;
    let core_xml = dom.serialize_to_string();
    let sources = collect_sources(uri);
    // Post-process to HTML5 in the child (the heavy XSLT stays inside the
    // cancellable/throwaway process).
    let html = post_process_html(&core_xml, uri);
    // Engine status (0 ok / 1 warning / 2 error / 3 fatal) — cumulative over
    // the inherited preamble report plus this body.
    let status = latexml_core::common::error::get_status_message();
    let status_code = latexml_core::common::error::get_status_code() as i64;
    let log = latexml_core::util::logger::flush_log();
    Ok::<(String, String, Vec<String>, String, i64), String>((
      html,
      log,
      sources,
      status,
      status_code,
    ))
  }));

  match result {
    Ok(Ok((html, log, sources, status, status_code))) => jobj(vec![
      ("html", jstr(html)),
      ("log", jstr(log)),
      (
        "sources",
        Value::Array(sources.into_iter().map(jstr).collect()),
      ),
      ("status", jstr(status)),
      // Integer (not jnum's float) so the parent's `as_i64()` round-trips.
      ("statusCode", Value::from(status_code)),
    ]),
    Ok(Err(msg)) => jobj(vec![("error", jstr(msg))]),
    Err(_) => jobj(vec![("error", jstr("child panicked"))]),
  }
}

impl Server {
  /// Convert `text` via the warm-preamble + fork-body pipeline, falling back
  /// to in-process conversion when that path is unavailable.
  fn run_warm(
    &mut self,
    uri: &str,
    text: &str,
    reader: &mut FdReader,
    pending: &mut VecDeque<String>,
  ) -> WarmResult {
    let Some((_, mat_end)) = find_begin_document(&self.begin_doc_regex, text) else {
      // No (un-commented) document body boundary — convert the whole thing
      // in-process.
      return WarmResult::Done(self.convert_in_process(uri, text));
    };
    let preamble = &text[..mat_end];
    let body = &text[mat_end..];
    let offset_lines = preamble.matches('\n').count();
    let deps = get_directory_dependencies(uri);
    let root_path = std::path::PathBuf::from(get_file_path(uri));
    let timeout = self.timeout_secs;
    // Cooperative deadline for the (parent-side) warm-up digest. The forked
    // body child re-arms its own fresh deadline; see `run_body_child`.
    latexml_core::stomach::set_timeout(timeout);

    let cache_hit = self.warmed_uri.as_deref() == Some(uri)
      && self.warmed_preamble.as_deref() == Some(preamble)
      && self.warmed_preamble_digested.is_some()
      // Same-dir file SET unchanged (no file appeared/disappeared that
      // could change find_file resolution — mtimes deliberately not
      // compared; see `get_directory_dependencies`).
      && self.warmed_dependencies == deps
      // Read-log snapshot: every source the warm-up opened must still be at
      // its pinned Overlay(version)/Disk(mtime) state (multi-file model).
      && deps_still_current(&self.warmed_source_deps, &self.open_buffers);

    if !cache_hit {
      log::info!("Warming preamble cache for {uri}");
      self.invalidate_cache();
      latexml_core::state::reset_thread_state();

      let opts = make_config(uri);
      let mut converter = Converter::from_config(opts.clone());
      if converter.prepare_session(&opts).is_err() {
        return WarmResult::Done(self.convert_in_process(uri, text));
      }
      // The reset wiped the engine state: re-apply the unsaved-buffer overlay
      // BEFORE the preamble digest, so preamble-time \input / \usepackage of
      // open buffers see editor state.
      self.overlay_keys.clear();
      self.apply_overlay(&root_path);
      // Name the preamble source after the document path so its locators are
      // stampable user sources (and share tag 0 with the body).
      match converter.digest_content_with_provenance(&get_file_path(uri), preamble.to_string()) {
        Ok(pre) => {
          self.warmed_preamble_log = converter.flush_log();
          self.warmed_uri = Some(uri.to_string());
          self.warmed_preamble = Some(preamble.to_string());
          self.warmed_preamble_digested = Some(pre);
          self.warmed_dependencies = deps;
          // Pin the warm-up read-log (which sources, at which state).
          self.warmed_source_deps = warmup_dep_snapshot(&self.open_buffers, &root_path);
        },
        Err(e) => {
          log::error!("Preamble warmup failed ({e}); falling back to in-process");
          return WarmResult::Done(self.convert_in_process(uri, text));
        },
      }
    } else {
      // Cache hit: refresh the overlay so the BODY fork inherits current
      // buffer state (body-time \input of an edited chapter). The preamble
      // deps were just verified unchanged, so the warm state stays valid.
      self.apply_overlay(&root_path);
    }

    let warmed = self.warmed_preamble_digested.as_ref().unwrap();
    // The child self-guards its time + RAM budget via the shared Watchdog.
    let (pid, read_fd) =
      match spawn_body_child(uri, offset_lines, body, warmed, timeout, self.max_rss_kb) {
        Ok(v) => v,
        Err(e) => {
          log::error!("{e}; falling back to in-process");
          return WarmResult::Done(self.convert_in_process(uri, text));
        },
      };

    // Hand the preamble log to `finish` (which runs deep inside the poll
    // loop) without threading it through every signature.
    PREAMBLE_LOG.with(|c| *c.borrow_mut() = self.warmed_preamble_log.clone());

    match wait_for_child(pid, read_fd, &root_path, reader, pending) {
      WarmResult::Cancelled => WarmResult::Cancelled,
      // A child that reported an internal error is reported as-is rather than
      // silently re-run in-process: re-running would reset the warm cache and
      // mask the failure. The fallback is reserved for *spawn/transport*
      // failures (handled above), not engine errors inside the child.
      done => done,
    }
  }
}

pub fn run(timeout_secs: u64, max_rss_kb: u64) -> Result<(), Box<dyn std::error::Error>> {
  // Snapshot the thread-count baseline for the fork-safety debug assertion.
  BASELINE_THREADS.store(task_count(), std::sync::atomic::Ordering::Relaxed);
  let mut stdout = std::io::stdout();
  let mut server = Server::new(timeout_secs, max_rss_kb);
  let mut reader = FdReader::new();
  let mut pending: VecDeque<String> = VecDeque::new();

  loop {
    let body = match pending.pop_front() {
      Some(b) => b,
      None => match reader.next_message() {
        Some(b) => b,
        None => break, // stdin EOF
      },
    };
    if !dispatch(&mut server, &mut reader, &mut pending, &body, &mut stdout)? {
      break;
    }
  }
  Ok(())
}

/// Handle one message. Returns `false` to stop the server (on `exit`).
fn dispatch(
  server: &mut Server,
  reader: &mut FdReader,
  pending: &mut VecDeque<String>,
  body_str: &str,
  stdout: &mut std::io::Stdout,
) -> Result<bool, Box<dyn std::error::Error>> {
  let request = match parse_json(body_str) {
    Ok(v) => v,
    Err(e) => {
      log::error!("Failed to parse incoming JSON: {e}");
      return Ok(true);
    },
  };
  let id = request.get("id").cloned().unwrap_or(Value::Null);
  let method = request.get("method").and_then(|m| m.as_str()).unwrap_or("");
  log::debug!("LSP request: method='{method}', id={id:?}");

  match method {
    "initialize" => {
      // Multi-file model: a client-configured project root wins over all
      // detection (docs/archive/LSP_MULTIFILE_PLAN.md §3A).
      if let Some(root) = request
        .get("params")
        .and_then(|p| p.get("initializationOptions"))
        .and_then(|o| o.get("rootDocument"))
        .and_then(|r| r.as_str())
      {
        server.root_override = Some(std::path::PathBuf::from(root));
      }
      let caps = jobj(vec![(
        "capabilities",
        jobj(vec![("textDocumentSync", jnum(1.0))]),
      )]);
      send_message(stdout, &response(id, caps))?;
    },
    "initialized" => {},
    "textDocument/didOpen" => {
      if let Some((uri, text)) = did_open_params(&request) {
        convert_trigger(server, reader, pending, &uri, &text, doc_version(&request), stdout, None)?;
      }
    },
    "textDocument/didChange" => {
      if let Some((uri, text)) = did_change_params(&request) {
        convert_trigger(server, reader, pending, &uri, &text, doc_version(&request), stdout, None)?;
      }
    },
    "textDocument/didClose" => {
      if let Some(uri) = request
        .get("params")
        .and_then(|p| p.get("textDocument"))
        .and_then(|d| d.get("uri"))
        .and_then(|u| u.as_str())
      {
        // Forget the buffer (the overlay reverts to disk state on the next
        // apply) and clear its diagnostics.
        server.open_buffers.remove(&get_file_path(uri));
        send_message(stdout, &publish_diagnostics_notification(uri, &[]))?;
      }
    },
    "shutdown" => {
      send_message(stdout, &response(id, Value::Null))?;
    },
    "latexml/convert" => {
      if let (Some(uri), Some(text)) = (
        request
          .get("params")
          .and_then(|p| p.get("uri"))
          .and_then(|u| u.as_str()),
        request
          .get("params")
          .and_then(|p| p.get("text"))
          .and_then(|t| t.as_str()),
      ) {
        let (uri, text) = (uri.to_string(), text.to_string());
        convert_trigger(server, reader, pending, &uri, &text, None, stdout, Some(id))?;
      } else if id != Value::Null {
        // A request (has an id) MUST be answered — silently dropping it
        // leaves the client awaiting the response forever.
        send_message(
          stdout,
          &error_response(id, -32602.0, "latexml/convert: missing params.uri/params.text".to_string()),
        )?;
      }
    },
    "exit" => return Ok(false),
    other => {
      if id != Value::Null {
        send_message(
          stdout,
          &error_response(id, -32601.0, format!("Method '{other}' not found")),
        )?;
      } else {
        log::warn!("Unhandled LSP notification: {other}");
      }
    },
  }
  Ok(true)
}

/// LSP `textDocument.version` of a didOpen/didChange payload.
fn doc_version(request: &Value) -> Option<i64> {
  request
    .get("params")
    .and_then(|p| p.get("textDocument"))
    .and_then(|d| d.get("version"))
    .and_then(|v| v.as_i64())
}

/// One conversion trigger (didOpen / didChange / `latexml/convert`), through
/// the multi-file pipeline: record the buffer, resolve the project root,
/// coalesce against newer same-project triggers, convert the ROOT via the
/// shared warm-fork path, then answer (`request_id`-carrying convert gets the
/// result object; notifications publish diagnostics).
#[allow(clippy::too_many_arguments)]
fn convert_trigger(
  server: &mut Server,
  reader: &mut FdReader,
  pending: &mut VecDeque<String>,
  uri: &str,
  text: &str,
  version: Option<i64>,
  stdout: &mut std::io::Stdout,
  request_id: Option<Value>,
) -> Result<(), Box<dyn std::error::Error>> {
  let buffer_path = get_file_path(uri);
  server.upsert_buffer(buffer_path.clone(), text.to_string(), version);
  let override_root = server.root_override.clone();
  let root = resolve_root(
    &mut server.root_cache,
    override_root.as_deref(),
    std::path::Path::new(&buffer_path),
    Some(text),
  );

  // Coalesce: a newer trigger for this PROJECT already queued makes this
  // compile stale before it starts.
  if superseded_in_pending(pending, &root) {
    if let Some(id) = request_id {
      send_message(stdout, &response(id, cancelled_result_object()))?;
    }
    return Ok(());
  }

  // The root's text: the edited buffer itself, another open buffer, or disk.
  let root_str = root.to_string_lossy().into_owned();
  let root_text = if root_str == buffer_path {
    text.to_string()
  } else if let Some(buf) = server.open_buffers.get(&root_str) {
    buf.text.clone()
  } else {
    match std::fs::read_to_string(&root) {
      Ok(t) => t,
      Err(e) => {
        // Degrade to v1 behavior: convert the buffer standalone.
        log::warn!("cannot read project root {root_str} ({e}); converting the buffer standalone");
        text.to_string()
      },
    }
  };
  let root_uri = format!("file://{root_str}");

  match server.run_warm(&root_uri, &root_text, reader, pending) {
    WarmResult::Done(mut out) => {
      // Attribute diagnostics to project files (multi-file model).
      attribute_diag_files(&mut out.diags, &root, &server.open_buffers);
      out.root = Some(root_str);
      match request_id {
        Some(id) => send_message(stdout, &response(id, out.to_result_object()))?,
        None => publish_grouped_diagnostics(server, &root_uri, uri, &out, stdout)?,
      }
    },
    WarmResult::Cancelled => {
      if let Some(id) = request_id {
        send_message(stdout, &response(id, cancelled_result_object()))?;
      }
      // A superseded notification publishes nothing — the newer run will.
    },
  }
  Ok(())
}



#[cfg(test)]
mod framing_tests {
  use super::*;

  /// A `pipe(2)` pair: write fragments into `w`, read frames via FdReader on
  /// the read end. Closed on drop.
  struct PipePair {
    r: i32,
    w: i32,
  }
  impl PipePair {
    fn new() -> Self {
      let mut fds = [0i32; 2];
      assert_eq!(unsafe { libc::pipe(fds.as_mut_ptr()) }, 0);
      PipePair { r: fds[0], w: fds[1] }
    }
    fn write(&self, bytes: &[u8]) {
      let n = unsafe { libc::write(self.w, bytes.as_ptr() as *const libc::c_void, bytes.len()) };
      assert_eq!(n, bytes.len() as isize);
    }
  }
  impl Drop for PipePair {
    fn drop(&mut self) {
      unsafe {
        libc::close(self.r);
        libc::close(self.w);
      }
    }
  }

  fn frame(body: &str) -> Vec<u8> {
    format!("Content-Length: {}\r\n\r\n{}", body.len(), body).into_bytes()
  }

  #[test]
  fn take_frame_handles_fragmented_input() {
    let pipe = PipePair::new();
    let mut reader = FdReader::from_fd(pipe.r);
    let msg = frame(r#"{"method":"initialized"}"#);
    // Header fragment only: no frame yet.
    pipe.write(&msg[..10]);
    reader.fill().unwrap();
    assert_eq!(reader.take_frame(), None);
    // Rest of header + partial body: still no frame.
    let header_len = msg.len() - 24;
    pipe.write(&msg[10..header_len + 5]);
    reader.fill().unwrap();
    assert_eq!(reader.take_frame(), None);
    // Remainder: one complete frame.
    pipe.write(&msg[header_len + 5..]);
    reader.fill().unwrap();
    assert_eq!(
      reader.take_frame().as_deref(),
      Some(r#"{"method":"initialized"}"#)
    );
    assert_eq!(reader.take_frame(), None);
  }

  #[test]
  fn take_frame_two_frames_one_fill() {
    let pipe = PipePair::new();
    let mut reader = FdReader::from_fd(pipe.r);
    let mut bytes = frame("{\"a\":1}");
    bytes.extend(frame("{\"b\":2}"));
    pipe.write(&bytes);
    reader.fill().unwrap();
    assert_eq!(reader.take_frame().as_deref(), Some("{\"a\":1}"));
    assert_eq!(reader.take_frame().as_deref(), Some("{\"b\":2}"));
    assert_eq!(reader.take_frame(), None);
  }

  #[test]
  fn take_frame_skips_malformed_header() {
    let pipe = PipePair::new();
    let mut reader = FdReader::from_fd(pipe.r);
    let mut bytes = b"Garbage-Header: x\r\n\r\n".to_vec();
    bytes.extend(frame("{\"ok\":true}"));
    pipe.write(&bytes);
    reader.fill().unwrap();
    // The malformed (no Content-Length) header is dropped; the good frame
    // behind it is returned — in a loop, not recursion.
    assert_eq!(reader.take_frame().as_deref(), Some("{\"ok\":true}"));
  }

  #[test]
  fn preempts_same_project_convert_and_didchange() {
    use std::path::Path;
    let conv = r#"{"method":"latexml/convert","params":{"uri":"file:///proj/main.tex","text":"x"}}"#;
    let chg = r#"{"method":"textDocument/didChange","params":{"textDocument":{"uri":"file:///proj/sections/ch2.tex"},"contentChanges":[{"text":"y"}]}}"#;
    let other = r#"{"method":"latexml/convert","params":{"uri":"file:///elsewhere/b.tex","text":"x"}}"#;
    let close = r#"{"method":"textDocument/didClose","params":{"textDocument":{"uri":"file:///proj/main.tex"}}}"#;
    let root = Path::new("/proj/main.tex");
    assert!(preempts(conv, root), "same-doc convert preempts");
    assert!(preempts(chg, root), "didChange anywhere in the project preempts");
    assert!(!preempts(other, root), "another project does not");
    assert!(!preempts(close, root), "didClose does not");
  }
}
