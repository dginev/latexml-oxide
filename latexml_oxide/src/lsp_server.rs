//! Persistent server for editor/preview integration (`latexml_oxide --server`).
//!
//! This is a JSON-RPC-over-stdio server using LSP message framing. It speaks a
//! subset of LSP (`initialize`, `textDocument/did{Open,Change,Close}` →
//! `publishDiagnostics`, `shutdown`, `exit`) **plus** a custom
//! `latexml/convert` request that returns `{html, log, diagnostics, sources,
//! status, statusCode}` — the response shape the `ar5iv-editor` client
//! consumes for its live source↔preview loop (see `docs/SOURCE_PROVENANCE.md`).
//!
//! Performance model: the preamble (everything up to and including
//! `\begin{document}`) is digested once and cached in this (parent) process.
//! Each body conversion `fork()`s a child that inherits the warm post-preamble
//! state via copy-on-write, digests only the body, builds the DOM, and writes
//! the result back over a pipe before exiting. The child is a throwaway, so a
//! body conversion can never pollute the cache, and a panicking/looping body
//! can't take down the server.
//!
//! Concurrency model: a **single thread** drives everything. While a body
//! child runs, the parent `poll(2)`s `{stdin, child-pipe}`; a newer
//! `latexml/convert` for the same document `SIGKILL`s the in-flight child
//! (a pid we still own — reaped here, so no PID-recycle race) and supersedes
//! it. Keeping it single-threaded is also what makes the `fork()` safe: there
//! is no second thread that could hold the allocator lock at fork time.

use std::collections::{BTreeMap, VecDeque};
use std::rc::Rc;

use crate::converter::Converter;
use latexml_core::common::{Config, DataSize, OutputFormat};

// ======================================================================
// JSON — backed by serde_json. The server uses only `Value` parse/
// serialize (no derive), which costs ~16 KiB in the LTO'd binary — well
// within the distribution size budget. `Value::get`/`as_str` map 1:1 onto
// the call sites the previous hand-rolled `Value` exposed.
// ======================================================================

use serde_json::Value;

/// Parse a JSON document. Keeps the `Result<_, String>` signature so call
/// sites are unchanged from the previous hand-rolled parser.
pub fn parse_json(s: &str) -> Result<Value, String> {
  serde_json::from_str(s).map_err(|e| e.to_string())
}

/// Build a `Value::String`.
fn jstr(s: impl Into<String>) -> Value { Value::String(s.into()) }

/// Build a JSON number from an `f64` (non-finite → `null`).
fn jnum(n: f64) -> Value {
  serde_json::Number::from_f64(n).map(Value::Number).unwrap_or(Value::Null)
}

/// Build a JSON object from `(key, value)` pairs. `serde_json::Map` is a
/// `BTreeMap` by default, so serialized key order is deterministic.
fn jobj(pairs: Vec<(&str, Value)>) -> Value {
  let mut map = serde_json::Map::new();
  for (k, v) in pairs {
    map.insert(k.to_string(), v);
  }
  Value::Object(map)
}

// ======================================================================
// Diagnostics — one parser, two output shapes.
// ======================================================================

#[derive(Debug, Clone, Copy, PartialEq)]
enum Severity {
  Error,
  Warning,
  Info,
  Fatal,
}

impl Severity {
  /// LSP `DiagnosticSeverity` numeric code.
  fn lsp_code(self) -> f64 {
    match self {
      Severity::Error | Severity::Fatal => 1.0,
      Severity::Warning => 2.0,
      Severity::Info => 3.0,
    }
  }

  /// ar5iv-editor normalized severity string.
  fn normalized(self) -> &'static str {
    match self {
      Severity::Error => "error",
      Severity::Warning => "warning",
      Severity::Info => "info",
      Severity::Fatal => "fatal",
    }
  }
}

/// A single parsed engine diagnostic. `line`/`col` are 1-based, file-relative
/// (the warm-fork path offsets body lines so this holds — see `run_warm`).
#[derive(Debug, Clone)]
struct Diag {
  severity: Severity,
  line:     Option<usize>,
  col:      Option<usize>,
  message:  String,
}

impl Diag {
  /// LSP `Diagnostic` (0-based positions).
  fn to_lsp(&self) -> Value {
    let line0 = self.line.map(|l| l.saturating_sub(1)).unwrap_or(0);
    let col0 = self.col.map(|c| c.saturating_sub(1)).unwrap_or(0);
    let start = jobj(vec![
      ("line", jnum(line0 as f64)),
      ("character", jnum(col0 as f64)),
    ]);
    // One-character caret anchor; the message carries the detail.
    let end = jobj(vec![
      ("line", jnum(line0 as f64)),
      ("character", jnum((col0 + 1) as f64)),
    ]);
    jobj(vec![
      ("range", jobj(vec![("start", start), ("end", end)])),
      ("severity", jnum(self.severity.lsp_code())),
      ("source", jstr("latexml")),
      ("message", jstr(self.message.clone())),
    ])
  }

  /// ar5iv-editor normalized diagnostic (1-based `from.{line,column}`).
  fn to_normalized(&self) -> Value {
    let mut pairs = vec![
      ("severity", jstr(self.severity.normalized())),
      ("category", jstr("latexml")),
      ("message", jstr(self.message.clone())),
    ];
    if let Some(line) = self.line {
      let mut from = vec![("line", jnum(line as f64))];
      if let Some(col) = self.col {
        from.push(("column", jnum(col as f64)));
      }
      pairs.push(("from", jobj(from)));
    }
    jobj(pairs)
  }
}

/// Parse one line/column pair out of a LaTeXML log message. Handles both the
/// `…; line N col M` and the bare `… line N` shapes.
fn parse_line_col(line: &str) -> (Option<usize>, Option<usize>) {
  let take_num = |s: &str| -> Option<usize> {
    let n: String = s.chars().take_while(|c| c.is_ascii_digit()).collect();
    n.parse::<usize>().ok().filter(|&v| v > 0)
  };
  if let Some(idx) = line.find("; line ") {
    let rest = &line[idx + 7..];
    let l = take_num(rest);
    let c = rest.find(" col ").and_then(|ci| take_num(&rest[ci + 5..]));
    (l, c)
  } else if let Some(idx) = line.find("line ") {
    (take_num(&line[idx + 5..]), None)
  } else {
    (None, None)
  }
}

fn parse_log_diagnostics(log_str: &str) -> Vec<Diag> {
  let mut diagnostics = Vec::new();
  for line in log_str.lines() {
    let severity = if line.starts_with("Error:") {
      Severity::Error
    } else if line.starts_with("Warn:") {
      Severity::Warning
    } else if line.starts_with("Fatal:") {
      Severity::Fatal
    } else if line.starts_with("Info:") {
      Severity::Info
    } else {
      continue;
    };
    let (l, c) = parse_line_col(line);
    diagnostics.push(Diag {
      severity,
      line: l,
      col: c,
      message: line.to_string(),
    });
  }
  diagnostics
}

// ======================================================================
// Conversion output + result/notification builders.
// ======================================================================

/// The platform-independent result of one conversion, before it is shaped
/// into either a `latexml/convert` result object or a `publishDiagnostics`
/// notification.
struct ConvertOutput {
  html:    String,
  log:     String,
  diags:   Vec<Diag>,
  sources: Vec<String>,
  /// Human-facing status label (the engine's status message, or `"timeout"`).
  status:  String,
  /// Engine status code: 0 = no problem, 1 = warning, 2 = error, 3 = fatal.
  status_code: i64,
}

/// Default label for a status code, when the engine message isn't carried.
fn status_label(code: i64) -> &'static str {
  match code {
    0 => "ok",
    1 => "warning",
    2 => "error",
    _ => "fatal",
  }
}

impl ConvertOutput {
  /// A failed conversion carrying a `status` label, a `status_code`
  /// (0/1/2/3), and a single Fatal diagnostic with `message`.
  fn failed(status: &str, status_code: i64, message: String) -> Self {
    ConvertOutput {
      html: String::new(),
      log: message.clone(),
      diags: vec![Diag {
        severity: Severity::Fatal,
        line: None,
        col: None,
        message,
      }],
      sources: Vec::new(),
      status: status.to_string(),
      status_code,
    }
  }

  /// A hard/fatal failure (status code 3).
  fn error(message: String) -> Self { Self::failed("fatal", 3, message) }

  /// The `latexml/convert` result object the ar5iv-editor client consumes.
  fn to_result_object(&self) -> Value {
    jobj(vec![
      ("html", jstr(self.html.clone())),
      ("log", jstr(self.log.clone())),
      (
        "diagnostics",
        Value::Array(self.diags.iter().map(Diag::to_normalized).collect()),
      ),
      (
        "sources",
        Value::Array(self.sources.iter().map(|s| jstr(s.clone())).collect()),
      ),
      ("status", jstr(self.status.clone())),
      ("statusCode", Value::from(self.status_code)),
    ])
  }
}

fn response(id: Value, result: Value) -> Value {
  jobj(vec![
    ("jsonrpc", jstr("2.0")),
    ("id", id),
    ("result", result),
  ])
}

fn error_response(id: Value, code: f64, message: String) -> Value {
  jobj(vec![
    ("jsonrpc", jstr("2.0")),
    ("id", id),
    (
      "error",
      jobj(vec![("code", jnum(code)), ("message", jstr(message))]),
    ),
  ])
}

fn cancelled_result_object() -> Value {
  jobj(vec![
    ("html", jstr("")),
    ("log", jstr("Request cancelled")),
    ("diagnostics", Value::Array(Vec::new())),
    ("sources", Value::Array(Vec::new())),
    ("status", jstr("cancelled")),
    ("statusCode", jnum(0.0)),
  ])
}

fn publish_diagnostics_notification(uri: &str, diags: &[Diag]) -> Value {
  jobj(vec![
    ("jsonrpc", jstr("2.0")),
    ("method", jstr("textDocument/publishDiagnostics")),
    (
      "params",
      jobj(vec![
        ("uri", jstr(uri)),
        (
          "diagnostics",
          Value::Array(diags.iter().map(Diag::to_lsp).collect()),
        ),
      ]),
    ),
  ])
}

fn send_message(writer: &mut impl std::io::Write, val: &Value) -> std::io::Result<()> {
  let body = val.to_string();
  let msg = format!("Content-Length: {}\r\n\r\n{}", body.len(), body);
  writer.write_all(msg.as_bytes())?;
  writer.flush()?;
  Ok(())
}

// ======================================================================
// URI / config / dependency helpers.
// ======================================================================

fn get_file_path(uri: &str) -> String {
  let s = uri.strip_prefix("file://").unwrap_or(uri);
  let mut decoded = String::new();
  let mut chars = s.chars();
  while let Some(c) = chars.next() {
    if c == '%' {
      let mut hex = String::new();
      if let Some(h1) = chars.next() {
        hex.push(h1);
      }
      if let Some(h2) = chars.next() {
        hex.push(h2);
      }
      if hex.len() == 2 {
        if let Ok(byte) = u8::from_str_radix(&hex, 16) {
          decoded.push(byte as char);
          continue;
        }
      }
      decoded.push('%');
      decoded.push_str(&hex);
    } else {
      decoded.push(c);
    }
  }
  decoded
}

/// Final path component (e.g. `main.tex`). The client lowercases for matching.
fn basename(path: &str) -> String {
  std::path::Path::new(path)
    .file_name()
    .and_then(|s| s.to_str())
    .map(String::from)
    .unwrap_or_else(|| path.to_string())
}

/// Resolve the source-map decoder ring: `sources[tag]` is the basename of the
/// file the integer `tag` (in each `data-sourcepos`) refers to. The main
/// buffer is digested as a `literal:` source named "Anonymous String"; map
/// that to the document's own basename so the client can resolve tag 0 back
/// to the active file. Other tags are real `\input`-ed files. Must be called
/// while the post-conversion thread-local state is still live.
fn collect_sources(uri: &str) -> Vec<String> {
  let self_base = basename(&get_file_path(uri));
  latexml_core::state::source_table_snapshot()
    .iter()
    .map(|sym| {
      let name = latexml_core::common::arena::with(*sym, |s| s.to_string());
      if name == "Anonymous String" {
        self_base.clone()
      } else {
        basename(&name)
      }
    })
    .collect()
}

/// Post-process the core ltx XML into HTML5 — the form the editor renders.
/// This runs the same pipeline the CLI and the ar5iv-editor server use
/// (`run_post_processing` with the embedded `LaTeXML-html5.xsl`), which turns
/// presentation MathML on and rewrites the source-map `data:sourcepos`
/// (foreign-namespaced, colon) attributes into the HTML `data-sourcepos`
/// (dash) the client decodes. Without this the server returned raw core XML.
fn post_process_html(core_xml: &str, uri: &str) -> String {
  let file_path = get_file_path(uri);
  let source_dir = std::path::Path::new(&file_path)
    .parent()
    .and_then(|p| p.to_str())
    .map(String::from);
  crate::post::run_post_processing(core_xml, &crate::post::PostOptions {
    pmml: true,
    cmml: false,
    keep_xmath: false,
    stylesheet: Some("resources/XSLT/LaTeXML-html5.xsl"),
    destination: None,
    source_directory: source_dir.as_deref(),
    // The server returns HTML as a string with no destination — it must never
    // write CSS/JS resource files to disk (would pollute the cwd). The client
    // supplies its own preview styling.
    nodefaultresources: true,
    css_files: &[],
    js_files: &[],
    noinvisibletimes: false,
    mathtex: false,
    navigationtoc: None,
    schemadocs: false,
    split: false,
    split_xpath: None,
    split_naming: None,
    xslt_parameters: &[],
    graphics_svg_threshold_kb: 0,
    whatsout: latexml_post::extract::Whatsout::Document,
  })
}

fn make_config(uri: &str) -> Config {
  let file_path = get_file_path(uri);
  let dir_path = std::path::Path::new(&file_path).parent();
  let mut search_paths = Vec::new();
  if let Some(parent) = dir_path {
    if let Some(p_str) = parent.to_str() {
      if !p_str.is_empty() {
        search_paths.push(p_str.to_string());
      }
    }
  }

  Config {
    verbosity: 0,
    format: OutputFormat::HTML5,
    whatsin: DataSize::Document,
    whatsout: DataSize::Document,
    preamble: None,
    postamble: None,
    mode: None,
    bindings_dispatch: Some(Rc::new(latexml_package::dispatch)),
    extra_bindings_dispatch: Some(Rc::new(latexml_contrib::dispatch)),
    // Preload ar5iv.sty: this server backs the ar5iv-editor, and ar5iv.sty
    // enables raw `.sty` handling so a paper's *local, binding-less* packages
    // (e.g. a bundled `mystyle.sty`) load instead of being skipped with a
    // missing-file warning. Mirrors the sandbox/ar5iv conversion workflow.
    preload: Some(vec!["ar5iv.sty".to_string()]),
    search_paths: if search_paths.is_empty() {
      None
    } else {
      Some(search_paths)
    },
    include_comments: None,
    nomathparse: if std::env::var("LATEXML_NOMATHPARSE").is_ok() {
      Some(true)
    } else {
      None
    },
    source_map: Some(true),
  }
}

fn get_directory_dependencies(uri: &str) -> BTreeMap<String, std::time::SystemTime> {
  let mut deps = BTreeMap::new();
  let file_path = get_file_path(uri);
  if let Some(parent) = std::path::Path::new(&file_path).parent() {
    if let Ok(entries) = std::fs::read_dir(parent) {
      for entry in entries.flatten() {
        let path = entry.path();
        if !entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
          continue;
        }
        let is_dep = path
          .extension()
          .and_then(|e| e.to_str())
          .map(|e| {
            matches!(
              e.to_lowercase().as_str(),
              "sty" | "cls" | "tex" | "cfg" | "def" | "bib" | "clo"
            )
          })
          .unwrap_or(false);
        if !is_dep {
          continue;
        }
        if let (Ok(metadata), Some(path_str)) = (entry.metadata(), path.to_str()) {
          if let Ok(mtime) = metadata.modified() {
            deps.insert(path_str.to_string(), mtime);
          }
        }
      }
    }
  }
  deps
}

// ======================================================================
// Server — warm preamble cache + in-process fallback conversion.
// ======================================================================

struct Server {
  begin_doc_regex:           regex::Regex,
  /// Per-conversion wall-clock budget in seconds (`--timeout`; 0 disables).
  timeout_secs:              u64,
  warmed_uri:                Option<String>,
  warmed_preamble:           Option<String>,
  warmed_preamble_digested:  Option<latexml_core::digested::Digested>,
  /// Log captured while digesting the preamble, so preamble diagnostics
  /// survive across body-only fork conversions (the warmup only re-runs on a
  /// cache miss).
  warmed_preamble_log:       String,
  warmed_dependencies:       BTreeMap<String, std::time::SystemTime>,
}

impl Server {
  fn new(timeout_secs: u64) -> Self {
    Server {
      begin_doc_regex: regex::Regex::new(r"\\begin\s*\{\s*document\s*\}").unwrap(),
      timeout_secs,
      warmed_uri: None,
      warmed_preamble: None,
      warmed_preamble_digested: None,
      warmed_preamble_log: String::new(),
      warmed_dependencies: BTreeMap::new(),
    }
  }

  /// Drop the warm preamble cache. MUST be called whenever the thread-local
  /// engine state is reset by an in-process conversion — otherwise a later
  /// cache-hit fork would inherit a state that no longer matches the cached
  /// preamble boxes (the bug that made `didChange` corrupt the next
  /// `latexml/convert`).
  fn invalidate_cache(&mut self) {
    self.warmed_uri = None;
    self.warmed_preamble = None;
    self.warmed_preamble_digested = None;
    self.warmed_preamble_log.clear();
    self.warmed_dependencies.clear();
  }

  /// Full, in-process conversion of `text`. Resets and rebuilds engine state,
  /// so it invalidates the warm cache. Used as the fallback whenever the
  /// warm-fork path is unavailable (no `\begin{document}`, fork failure,
  /// non-Unix). Line numbers are naturally file-relative (the whole document
  /// is one source), and the source-map table is read back after conversion.
  fn convert_in_process(&mut self, uri: &str, text: &str) -> ConvertOutput {
    self.invalidate_cache();
    latexml_core::state::reset_thread_state();
    // Cooperative wall-clock guard for the in-process path. (There is no child
    // to reap here — this path runs on the server's own thread — so the hard
    // RAM/time backstops don't apply; the cooperative deadline + the engine's
    // RSS fuse are what bound a runaway fallback conversion.)
    latexml_core::stomach::set_timeout(self.timeout_secs);

    let opts = make_config(uri);
    let mut converter = Converter::from_config(opts.clone());
    if let Err(e) = converter.prepare_session(&opts) {
      return ConvertOutput::error(format!("Fatal: prepare_session failed: {e}"));
    }
    converter.bind_log();
    // Fallback path (no `\begin{document}`, fork failure, or non-Unix). Use a
    // *named* in-memory source (the document path) so `--source-map` stamps
    // locators here too, matching the warm-fork path.
    let resp = converter.convert_content_with_provenance(&get_file_path(uri), text.to_string());
    let sources = collect_sources(uri);
    let status_code = resp.status_code as i64;
    let status = if resp.status.is_empty() {
      status_label(status_code).to_string()
    } else {
      resp.status
    };
    let log = resp.log;
    let diags = parse_log_diagnostics(&log);
    let html = post_process_html(&resp.result.unwrap_or_default(), uri);
    ConvertOutput {
      html,
      log,
      diags,
      sources,
      status,
      status_code,
    }
  }
}

// ======================================================================
// Unix: warm-fork pipeline + single-threaded poll loop.
// ======================================================================

#[cfg(unix)]
mod unix_server {
  use super::*;
  use std::io::{Read, Write};
  use std::os::unix::io::FromRawFd;

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
    fn new() -> Self { FdReader { fd: 0, buf: Vec::new() } }

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

    /// Is a *complete* LSP frame already buffered (no syscall)? Used to decide
    /// stdin-readiness without consuming, so `poll` and the buffer agree.
    fn has_complete_frame(&self) -> bool {
      if let Some(he) = find_subseq(&self.buf, b"\r\n\r\n") {
        let cl = parse_content_length(&self.buf[..he]).unwrap_or(0);
        self.buf.len() >= he + 4 + cl
      } else {
        false
      }
    }

    /// Pull one complete frame out of the buffer, if present.
    fn take_frame(&mut self) -> Option<String> {
      let he = find_subseq(&self.buf, b"\r\n\r\n")?;
      let cl = parse_content_length(&self.buf[..he]);
      let body_start = he + 4;
      match cl {
        Some(cl) if self.buf.len() >= body_start + cl => {
          let body: Vec<u8> = self.buf[body_start..body_start + cl].to_vec();
          self.buf.drain(..body_start + cl);
          Some(String::from_utf8_lossy(&body).into_owned())
        },
        // Malformed header with no parseable Content-Length: drop it and retry.
        None => {
          self.buf.drain(..body_start);
          self.take_frame()
        },
        // Header present but body not fully arrived yet.
        Some(_) => None,
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

  fn reap(pid: i32) {
    let mut status = 0i32;
    unsafe {
      libc::waitpid(pid, &mut status, 0);
    }
  }

  /// Does this raw message supersede the in-flight `latexml/convert` for
  /// `current_uri`? Only a newer convert of the *same* document preempts.
  fn preempts(body: &str, current_uri: &str) -> bool {
    if let Ok(req) = parse_json(body) {
      if req.get("method").and_then(|m| m.as_str()) == Some("latexml/convert") {
        return req
          .get("params")
          .and_then(|p| p.get("uri"))
          .and_then(|u| u.as_str())
          == Some(current_uri);
      }
    }
    false
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
  /// How long the parent's hard wall-clock backstop waits beyond the child's
  /// own cooperative `set_timeout`, so the child times out gracefully first.
  const TIMEOUT_GRACE_SECS: u64 = 5;
  /// Parent poll cadence — wake this often to re-check the child's resource use.
  const RESOURCE_TICK_MS: libc::c_int = 200;

  /// Read a child's resident set size (KiB) from `/proc/<pid>/status`.
  fn child_rss_kb(pid: i32) -> Option<u64> {
    let s = std::fs::read_to_string(format!("/proc/{pid}/status")).ok()?;
    for line in s.lines() {
      if let Some(rest) = line.strip_prefix("VmRSS:") {
        return rest.split_whitespace().next()?.parse::<u64>().ok();
      }
    }
    None
  }

  fn wait_for_child(
    pid: i32,
    read_fd: i32,
    current_uri: &str,
    reader: &mut FdReader,
    pending: &mut VecDeque<String>,
    timeout_secs: u64,
    ram_cap_kb: u64,
  ) -> WarmResult {
    use std::time::{Duration, Instant};
    // Owns `read_fd`; closes it on every return path.
    let mut pipe = unsafe { std::fs::File::from_raw_fd(read_fd) };

    let start = Instant::now();
    // Hard wall-clock limit (the child's cooperative deadline fires first).
    let time_limit = (timeout_secs > 0).then(|| Duration::from_secs(timeout_secs + TIMEOUT_GRACE_SECS));

    let kill_reap = |what: &str, msg: String| -> WarmResult {
      log::warn!("Reaping body child {pid}: {what}");
      unsafe {
        libc::kill(pid, libc::SIGKILL);
      }
      reap(pid);
      // Reaped for a resource breach → fatal (status code 3).
      let label = if what == "timeout" { "timeout" } else { "fatal" };
      WarmResult::Done(ConvertOutput::failed(label, 3, msg))
    };

    loop {
      // Parent-enforced hard backstops — the child is reaped here if it blows
      // through the cooperative guards (e.g. a runaway in libxslt that never
      // reaches a `check_timeout`, or a sudden allocation spike).
      if let Some(lim) = time_limit {
        if start.elapsed() > lim {
          return kill_reap("timeout", format!("conversion exceeded {timeout_secs}s wall-clock budget"));
        }
      }
      if ram_cap_kb > 0 {
        if let Some(rss) = child_rss_kb(pid) {
          if rss > ram_cap_kb {
            return kill_reap(
              "oom",
              format!("conversion exceeded {} MB RAM budget", ram_cap_kb / 1024),
            );
          }
        }
      }

      // Already-buffered stdin frame takes priority (poll wouldn't re-report
      // bytes we've pulled into user space).
      let stdin_ready = if reader.has_complete_frame() {
        true
      } else {
        let mut fds = [
          libc::pollfd { fd: 0, events: libc::POLLIN, revents: 0 },
          libc::pollfd { fd: read_fd, events: libc::POLLIN, revents: 0 },
        ];
        // Finite tick so the resource backstops above run even while neither fd
        // is ready (a CPU/RAM-bound child produces nothing until it finishes).
        let rc = unsafe { libc::poll(fds.as_mut_ptr(), fds.len() as libc::nfds_t, RESOURCE_TICK_MS) };
        if rc < 0 {
          let err = std::io::Error::last_os_error();
          if err.raw_os_error() == Some(libc::EINTR) {
            continue;
          }
          // poll failed: fall back to a blocking drain of the child.
          let mut bytes = Vec::new();
          let _ = pipe.read_to_end(&mut bytes);
          reap(pid);
          return finish(current_uri, &bytes);
        }
        if rc == 0 {
          // Tick elapsed, nothing ready — loop back to the resource checks.
          continue;
        }
        let pipe_ready = (fds[1].revents & (libc::POLLIN | libc::POLLHUP)) != 0;
        if pipe_ready {
          // The child writes its whole payload in one shot at the very end, so
          // pipe-readable means the compile is essentially done — drain & reap.
          let mut bytes = Vec::new();
          let _ = pipe.read_to_end(&mut bytes);
          reap(pid);
          return finish(current_uri, &bytes);
        }
        (fds[0].revents & libc::POLLIN) != 0
      };

      if stdin_ready {
        match reader.next_message() {
          // stdin EOF mid-compile: client gone — kill the child and stop.
          None => {
            unsafe {
              libc::kill(pid, libc::SIGKILL);
            }
            reap(pid);
            return WarmResult::Cancelled;
          },
          Some(body) => {
            if is_exit(&body) {
              unsafe {
                libc::kill(pid, libc::SIGKILL);
              }
              reap(pid);
              std::process::exit(0);
            }
            if preempts(&body, current_uri) {
              unsafe {
                libc::kill(pid, libc::SIGKILL);
              }
              reap(pid);
              pending.push_back(body);
              return WarmResult::Cancelled;
            }
            // Unrelated message (didClose, shutdown, a different document):
            // queue it for after this compile and keep waiting.
            pending.push_back(body);
          },
        }
      }
    }
  }

  /// Parse the child's pipe payload into a `ConvertOutput`. The parent owns the
  /// preamble (warmup) log and the source-map context, so it merges them here.
  fn finish(current_uri: &str, bytes: &[u8]) -> WarmResult {
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
          let _ = current_uri;
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
            status,
            status_code,
          })
        }
      },
      Err(e) => WarmResult::Done(ConvertOutput::error(format!(
        "child payload parse error: {e}"
      ))),
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
  ) -> Result<(i32, i32), String> {
    let mut fds = [0i32; 2];
    unsafe {
      if libc::pipe(fds.as_mut_ptr()) < 0 {
        return Err("pipe() failed".to_string());
      }
    }
    let (read_fd, write_fd) = (fds[0], fds[1]);

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
      }
      let payload = run_body_child(uri, offset_lines, body, warmed, timeout_secs);
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
  ) -> Value {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
      // Re-arm the conversion deadline FRESH from this child's clock. The child
      // inherits the parent's thread-local CONVERSION_DEADLINE via COW (set
      // during warm-up); without this the body would run against a stale (often
      // already-expired) deadline. This is the cooperative guard — digest loops
      // raise Fatal:Timeout, caught below; the parent enforces the hard backstop.
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
      let Some(mat) = self.begin_doc_regex.find(text) else {
        // No document body boundary — just convert the whole thing in-process.
        return WarmResult::Done(self.convert_in_process(uri, text));
      };
      let preamble = &text[..mat.end()];
      let body = &text[mat.end()..];
      let offset_lines = preamble.matches('\n').count();
      let deps = get_directory_dependencies(uri);
      let timeout = self.timeout_secs;
      // Cooperative deadline for the (parent-side) warm-up digest. The forked
      // body child re-arms its own fresh deadline; see `run_body_child`.
      latexml_core::stomach::set_timeout(timeout);

      let cache_hit = self.warmed_uri.as_deref() == Some(uri)
        && self.warmed_preamble.as_deref() == Some(preamble)
        && self.warmed_preamble_digested.is_some()
        && self.warmed_dependencies == deps;

      if !cache_hit {
        log::info!("Warming preamble cache for {uri}");
        self.invalidate_cache();
        latexml_core::state::reset_thread_state();

        let opts = make_config(uri);
        let mut converter = Converter::from_config(opts.clone());
        if converter.prepare_session(&opts).is_err() {
          return WarmResult::Done(self.convert_in_process(uri, text));
        }
        // Name the preamble source after the document path so its locators are
        // stampable user sources (and share tag 0 with the body).
        match converter.digest_content_with_provenance(&get_file_path(uri), preamble.to_string()) {
          Ok(pre) => {
            self.warmed_preamble_log = converter.flush_log();
            self.warmed_uri = Some(uri.to_string());
            self.warmed_preamble = Some(preamble.to_string());
            self.warmed_preamble_digested = Some(pre);
            self.warmed_dependencies = deps;
          },
          Err(e) => {
            log::error!("Preamble warmup failed ({e}); falling back to in-process");
            return WarmResult::Done(self.convert_in_process(uri, text));
          },
        }
      }

      let warmed = self.warmed_preamble_digested.as_ref().unwrap();
      let (pid, read_fd) = match spawn_body_child(uri, offset_lines, body, warmed, timeout) {
        Ok(v) => v,
        Err(e) => {
          log::error!("{e}; falling back to in-process");
          return WarmResult::Done(self.convert_in_process(uri, text));
        },
      };

      // Hand the preamble log to `finish` (which runs deep inside the poll
      // loop) without threading it through every signature.
      PREAMBLE_LOG.with(|c| *c.borrow_mut() = self.warmed_preamble_log.clone());

      match wait_for_child(pid, read_fd, uri, reader, pending, timeout, max_rss_kb()) {
        WarmResult::Cancelled => WarmResult::Cancelled,
        // A child that reported an internal error is reported as-is rather than
        // silently re-run in-process: re-running would reset the warm cache and
        // mask the failure. The fallback is reserved for *spawn/transport*
        // failures (handled above), not engine errors inside the child.
        done => done,
      }
    }
  }

  pub fn run(timeout_secs: u64) -> Result<(), Box<dyn std::error::Error>> {
    let mut stdout = std::io::stdout();
    let mut server = Server::new(timeout_secs);
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
        let caps = jobj(vec![(
          "capabilities",
          jobj(vec![("textDocumentSync", jnum(1.0))]),
        )]);
        send_message(stdout, &response(id, caps))?;
      },
      "initialized" => {},
      "textDocument/didOpen" => {
        if let Some((uri, text)) = did_open_params(&request) {
          run_diagnostics(server, reader, pending, &uri, &text, stdout)?;
        }
      },
      "textDocument/didChange" => {
        if let Some((uri, text)) = did_change_params(&request) {
          run_diagnostics(server, reader, pending, &uri, &text, stdout)?;
        }
      },
      "textDocument/didClose" => {
        if let Some(uri) = request
          .get("params")
          .and_then(|p| p.get("textDocument"))
          .and_then(|d| d.get("uri"))
          .and_then(|u| u.as_str())
        {
          // Clear diagnostics for the closed document.
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
          match server.run_warm(&uri, &text, reader, pending) {
            WarmResult::Done(out) => {
              send_message(stdout, &response(id, out.to_result_object()))?;
            },
            WarmResult::Cancelled => {
              send_message(stdout, &response(id, cancelled_result_object()))?;
            },
          }
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

  /// Run a conversion for its diagnostics (didOpen/didChange) and publish them.
  /// Uses the same warm-fork pipeline as `latexml/convert` so the cache is
  /// shared and stays coherent; a superseded run publishes nothing (the newer
  /// run will).
  fn run_diagnostics(
    server: &mut Server,
    reader: &mut FdReader,
    pending: &mut VecDeque<String>,
    uri: &str,
    text: &str,
    stdout: &mut std::io::Stdout,
  ) -> Result<(), Box<dyn std::error::Error>> {
    if let WarmResult::Done(out) = server.run_warm(uri, text, reader, pending) {
      send_message(stdout, &publish_diagnostics_notification(uri, &out.diags))?;
    }
    Ok(())
  }
}

// ======================================================================
// Non-Unix: simple blocking in-process server (no fork, no preemption).
// ======================================================================

#[cfg(not(unix))]
mod generic_server {
  use super::*;
  use std::io::{BufRead, Read};

  fn read_message(reader: &mut impl BufRead) -> Option<String> {
    let mut content_length = 0usize;
    let mut header = String::new();
    loop {
      header.clear();
      match reader.read_line(&mut header) {
        Ok(0) => return None,
        Ok(_) => {},
        Err(_) => return None,
      }
      let trimmed = header.trim_end();
      if trimmed.is_empty() {
        break;
      }
      if trimmed.to_lowercase().starts_with("content-length:") {
        if let Some(v) = trimmed.split(':').nth(1) {
          if let Ok(n) = v.trim().parse::<usize>() {
            content_length = n;
          }
        }
      }
    }
    if content_length == 0 {
      return Some(String::new());
    }
    let mut body = vec![0u8; content_length];
    if reader.read_exact(&mut body).is_err() {
      return None;
    }
    Some(String::from_utf8_lossy(&body).into_owned())
  }

  pub fn run(timeout_secs: u64) -> Result<(), Box<dyn std::error::Error>> {
    let mut stdout = std::io::stdout();
    let mut server = Server::new(timeout_secs);
    let stdin = std::io::stdin();
    let mut reader = std::io::BufReader::new(stdin.lock());

    while let Some(body_str) = read_message(&mut reader) {
      if body_str.is_empty() {
        continue;
      }
      let request = match parse_json(&body_str) {
        Ok(v) => v,
        Err(e) => {
          log::error!("Failed to parse incoming JSON: {e}");
          continue;
        },
      };
      let id = request.get("id").cloned().unwrap_or(Value::Null);
      let method = request.get("method").and_then(|m| m.as_str()).unwrap_or("");

      match method {
        "initialize" => {
          let caps = jobj(vec![(
            "capabilities",
            jobj(vec![("textDocumentSync", jnum(1.0))]),
          )]);
          send_message(&mut stdout, &response(id, caps))?;
        },
        "initialized" => {},
        "textDocument/didOpen" => {
          if let Some((uri, text)) = did_open_params(&request) {
            let out = server.convert_in_process(&uri, &text);
            send_message(&mut stdout, &publish_diagnostics_notification(&uri, &out.diags))?;
          }
        },
        "textDocument/didChange" => {
          if let Some((uri, text)) = did_change_params(&request) {
            let out = server.convert_in_process(&uri, &text);
            send_message(&mut stdout, &publish_diagnostics_notification(&uri, &out.diags))?;
          }
        },
        "textDocument/didClose" => {
          if let Some(uri) = request
            .get("params")
            .and_then(|p| p.get("textDocument"))
            .and_then(|d| d.get("uri"))
            .and_then(|u| u.as_str())
          {
            send_message(&mut stdout, &publish_diagnostics_notification(uri, &[]))?;
          }
        },
        "shutdown" => {
          send_message(&mut stdout, &response(id, Value::Null))?;
        },
        "latexml/convert" => {
          if let (Some(uri), Some(text)) = (
            request.get("params").and_then(|p| p.get("uri")).and_then(|u| u.as_str()),
            request.get("params").and_then(|p| p.get("text")).and_then(|t| t.as_str()),
          ) {
            let out = server.convert_in_process(uri, text);
            send_message(&mut stdout, &response(id, out.to_result_object()))?;
          }
        },
        "exit" => return Ok(()),
        other => {
          if id != Value::Null {
            send_message(
              &mut stdout,
              &error_response(id, -32601.0, format!("Method '{other}' not found")),
            )?;
          }
        },
      }
    }
    Ok(())
  }
}

// Shared param extraction (used by both server flavors).

fn did_open_params(request: &Value) -> Option<(String, String)> {
  let td = request.get("params")?.get("textDocument")?;
  let uri = td.get("uri")?.as_str()?.to_string();
  let text = td.get("text")?.as_str()?.to_string();
  Some((uri, text))
}

fn did_change_params(request: &Value) -> Option<(String, String)> {
  let params = request.get("params")?;
  let uri = params
    .get("textDocument")?
    .get("uri")?
    .as_str()?
    .to_string();
  let changes = match params.get("contentChanges")? {
    Value::Array(c) => c,
    _ => return None,
  };
  let text = changes.first()?.get("text")?.as_str()?.to_string();
  Some((uri, text))
}

/// `timeout_secs` is the per-conversion wall-clock budget (the `--timeout`
/// flag; 0 disables). It is applied **fresh per conversion** — in particular,
/// re-armed inside each forked child so a child never runs against the parent's
/// stale warm-up deadline — and backstopped by the parent (which also reaps a
/// child that exceeds the RAM cap; see [`max_rss_kb`]).
pub fn run_lsp_server(timeout_secs: u64) -> Result<(), Box<dyn std::error::Error>> {
  #[cfg(unix)]
  {
    unix_server::run(timeout_secs)
  }
  #[cfg(not(unix))]
  {
    generic_server::run(timeout_secs)
  }
}

/// Hard RSS ceiling for a forked body child (parent reaps it past this).
/// Default 6 GiB; override with `LATEXML_LSP_MAX_RSS_MB`. Returns KiB (to
/// compare directly against `/proc/<pid>/status` `VmRSS`). 0 disables.
fn max_rss_kb() -> u64 {
  std::env::var("LATEXML_LSP_MAX_RSS_MB")
    .ok()
    .and_then(|v| v.parse::<u64>().ok())
    .map(|mb| mb * 1024)
    .unwrap_or(6 * 1024 * 1024)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn json_roundtrip_object() {
    let v = parse_json(r#"{"a":1,"b":[true,null,"x"],"c":{"d":-2.5}}"#).unwrap();
    // serde_json parses `1` as an integer Number (distinct repr from a float),
    // so compare via the accessor rather than against `jnum`.
    assert_eq!(v.get("a").and_then(Value::as_i64), Some(1));
    assert_eq!(
      v.get("b"),
      Some(&Value::Array(vec![
        Value::Bool(true),
        Value::Null,
        Value::String("x".to_string())
      ]))
    );
    // BTreeMap → deterministic, sorted key order on serialization.
    assert_eq!(v.to_string(), r#"{"a":1,"b":[true,null,"x"],"c":{"d":-2.5}}"#);
  }

  #[test]
  fn json_escapes_all_control_chars() {
    // The old serializer only escaped \n \r \t; a form-feed / NUL produced
    // invalid JSON. Verify every control char is \u00xx-escaped and the
    // result re-parses (round-trip).
    let s = "x\u{0}\u{1}\u{8}\u{b}\u{c}\u{1f}y\t\n\r\"\\";
    let serialized = jstr(s).to_string();
    assert!(serialized.contains("\\u0000"));
    assert!(serialized.contains("\\u0001"));
    assert!(serialized.contains("\\u000b")); // vertical tab
    assert!(serialized.contains("\\b"));
    assert!(serialized.contains("\\f"));
    assert!(serialized.contains("\\u001f"));
    assert!(serialized.contains("\\t") && serialized.contains("\\n") && serialized.contains("\\r"));
    assert!(serialized.contains("\\\"") && serialized.contains("\\\\"));
    let reparsed = parse_json(&serialized).unwrap();
    assert_eq!(reparsed, Value::String(s.to_string()));
  }

  #[test]
  fn parse_line_col_variants() {
    assert_eq!(parse_line_col("Error:foo:bar baz; line 12 col 7"), (Some(12), Some(7)));
    assert_eq!(parse_line_col("Warn:foo bar at line 3"), (Some(3), None));
    assert_eq!(parse_line_col("Info:foo no position here"), (None, None));
  }

  #[test]
  fn diagnostics_severity_and_zero_basing() {
    let log = "Error:x:y problem; line 5 col 2\nWarn:p:q issue at line 9\nrandom line\nInfo:i:j note line 1";
    let diags = parse_log_diagnostics(log);
    assert_eq!(diags.len(), 3);
    assert_eq!(diags[0].severity, Severity::Error);
    // LSP is 0-based: line 5 → 4, col 2 → 1.
    let lsp = diags[0].to_lsp();
    let range = lsp.get("range").unwrap();
    let start = range.get("start").unwrap();
    assert_eq!(start.get("line"), Some(&jnum(4.0)));
    assert_eq!(start.get("character"), Some(&jnum(1.0)));
    assert_eq!(lsp.get("severity"), Some(&jnum(1.0)));
    // Normalized keeps 1-based.
    let norm = diags[0].to_normalized();
    let from = norm.get("from").unwrap();
    assert_eq!(from.get("line"), Some(&jnum(5.0)));
    assert_eq!(from.get("column"), Some(&jnum(2.0)));
    assert_eq!(norm.get("severity"), Some(&Value::String("error".to_string())));
  }

  #[test]
  fn basename_extraction() {
    assert_eq!(basename("/home/u/proj/main.tex"), "main.tex");
    assert_eq!(basename("main.tex"), "main.tex");
  }

  #[test]
  fn cancelled_object_shape() {
    let v = cancelled_result_object();
    assert_eq!(v.get("status"), Some(&Value::String("cancelled".to_string())));
    assert_eq!(v.get("html"), Some(&Value::String(String::new())));
  }
}
