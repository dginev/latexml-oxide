//! Per-job telemetry: phase wall times, counts, and resource peaks.
//!
//! See `docs/TELEMETRY.md` for the design contract.
//!
//! Default-on instrumentation. Coarse phase wrappers cost ~20ns each
//! (one `Instant::now()` call); the math-parse histogram update is
//! the only per-formula instrumentation and is a single atomic
//! increment of one of 9 `u32` slots.
//!
//! Thread-local state. Aggregate at end-of-process via `take()`.
//! All times in microseconds; counts in their natural unit.

use std::cell::RefCell;
use std::time::Instant;

/// Coarse phase enum. 17 values; bumping requires updating
/// `Telemetry::write_json` and `tools/perf_phase_summary.py`.
///
/// Phase ordering reflects the conversion pipeline order
/// (Bootstrap → Digest → Build → Rewrite → MathParse →
/// PostXmlParse → PostScan → Bibliography → Crossref → Graphics →
/// MathImages → MathmlPres → MathmlCont → Split → Xslt →
/// Html5Fixups → Serialize) so flat dumps read in execution order.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Phase {
  Bootstrap = 0,
  Digest = 1,
  Build = 2,
  Rewrite = 3,
  MathParse = 4,
  /// Parses the XML emitted by core into a `PostDocument`.
  /// Size-proportional; material on large papers.
  PostXmlParse = 5,
  /// Scan phase of latexml_post: ID assignment, label resolution, etc.
  PostScan = 6,
  Bibliography = 7,
  Crossref = 8,
  /// External-tool dispatch for `\includegraphics` (convert / gs / inkscape).
  Graphics = 9,
  /// External-tool dispatch for picture/latex/math image rendering
  /// (`picture_images.rs` + `latex_images.rs` + `math_images.rs`).
  MathImages = 10,
  MathmlPres = 11,
  MathmlCont = 12,
  /// Document splitting (only when `--split` is on; otherwise 0).
  Split = 13,
  Xslt = 14,
  /// Final HTML tweaks after XSLT (asset paths, header/footer).
  Html5Fixups = 15,
  Serialize = 16,
}

impl Phase {
  pub const COUNT: usize = 17;

  pub fn as_str(self) -> &'static str {
    match self {
      Phase::Bootstrap => "bootstrap",
      Phase::Digest => "digest",
      Phase::Build => "build",
      Phase::Rewrite => "rewrite",
      Phase::MathParse => "math_parse",
      Phase::PostXmlParse => "post_xml_parse",
      Phase::PostScan => "post_scan",
      Phase::Bibliography => "bibliography",
      Phase::Crossref => "crossref",
      Phase::Graphics => "graphics",
      Phase::MathImages => "math_images",
      Phase::MathmlPres => "mathml_pres",
      Phase::MathmlCont => "mathml_cont",
      Phase::Split => "split",
      Phase::Xslt => "xslt",
      Phase::Html5Fixups => "html5_fixups",
      Phase::Serialize => "serialize",
    }
  }
}

/// Math-parse time bucket boundaries in microseconds.
/// `bucket(us)` returns 0..=8.
const BUCKET_BOUNDS_US: [u64; 8] = [500, 1_000, 2_000, 5_000, 10_000, 20_000, 50_000, 100_000];

fn bucket_for(us: u64) -> usize {
  for (i, b) in BUCKET_BOUNDS_US.iter().enumerate() {
    if us < *b {
      return i;
    }
  }
  8
}

/// Per-job telemetry record.
///
/// Identifier fields (`paper_id`, `cmdline`, `host`, `git_sha`) are
/// filled in by the binary entry point, not by the engine. Phase
/// wall times and counts are populated by the engine via this
/// module's API.
#[derive(Clone, Debug)]
pub struct Telemetry {
  // Identifiers (set by the binary)
  pub paper_id: String,
  pub git_sha: String,
  pub cmdline: String,
  pub host: String,
  pub timeout_s: u32,
  pub schema_version: u32,

  // Wall (microseconds)
  pub wall_us: u64,
  pub phase_us: [u64; Phase::COUNT],

  // Counts
  pub formulae: u32,
  pub math_parse_attempts: u32,
  pub math_parse_count: u64,
  pub math_parse_buckets: [u32; 9],
  pub graphics_assets: u32,
  pub graphics_subprocess_count: u32,
  pub db_objects: u32,
  pub output_bytes: u64,
  pub warnings: u32,
  pub errors: u32,
  pub fatal_errors: u32,
  pub external_tool_count: u32,

  // Resource
  pub max_rss_kb: u64,
  pub child_user_us: u64,
  pub child_sys_us: u64,

  // Outcome (set by the binary at end)
  pub category: String,
  pub exit_code: i32,
}

impl Default for Telemetry {
  fn default() -> Self {
    Telemetry {
      paper_id: String::new(),
      git_sha: option_env!("LATEXML_GIT_SHA").unwrap_or("").to_string(),
      cmdline: String::new(),
      host: String::new(),
      timeout_s: 0,
      schema_version: 1,
      wall_us: 0,
      phase_us: [0; Phase::COUNT],
      formulae: 0,
      math_parse_attempts: 0,
      math_parse_count: 0,
      math_parse_buckets: [0; 9],
      graphics_assets: 0,
      graphics_subprocess_count: 0,
      db_objects: 0,
      output_bytes: 0,
      warnings: 0,
      errors: 0,
      fatal_errors: 0,
      external_tool_count: 0,
      max_rss_kb: 0,
      child_user_us: 0,
      child_sys_us: 0,
      category: String::new(),
      exit_code: 0,
    }
  }
}

thread_local! {
  static STATE: RefCell<Telemetry> = RefCell::new(Telemetry::default());
  // Phase stack: each entry is (phase, started_at). Time accrues only
  // to the innermost (top-of-stack) phase.
  static STACK: RefCell<Vec<(Phase, Instant)>> = const { RefCell::new(Vec::new()) };
}

/// Begin a phase. Subsequent time accrues to this phase until the
/// matching `phase_exit()` (or the `PhaseGuard` returned by
/// [`phase`]) drops.
pub fn phase_enter(p: Phase) {
  let now = Instant::now();
  STACK.with(|s| {
    let mut stack = s.borrow_mut();
    // If there's a parent phase, charge accumulated wall to it
    // before we steal the clock.
    if let Some((parent, started)) = stack.last_mut() {
      let dt = now.saturating_duration_since(*started).as_micros() as u64;
      let parent = *parent;
      STATE.with(|st| st.borrow_mut().phase_us[parent as usize] += dt);
      *started = now;
    }
    stack.push((p, now));
  });
}

/// End the innermost phase.
pub fn phase_exit() {
  let now = Instant::now();
  STACK.with(|s| {
    let mut stack = s.borrow_mut();
    let (p, started) =
      stack.pop().expect("telemetry::phase_exit called without matching phase_enter");
    let dt = now.saturating_duration_since(started).as_micros() as u64;
    STATE.with(|st| st.borrow_mut().phase_us[p as usize] += dt);
    // Reset parent's start so it doesn't double-count our time.
    if let Some((_, started_parent)) = stack.last_mut() {
      *started_parent = now;
    }
  });
}

/// RAII guard returned by [`phase`]. Calls `phase_exit` on drop.
pub struct PhaseGuard {
  _private: (),
}

impl Drop for PhaseGuard {
  fn drop(&mut self) {
    phase_exit();
  }
}

/// Convenience: `let _g = telemetry::phase(Phase::Digest);`
pub fn phase(p: Phase) -> PhaseGuard {
  phase_enter(p);
  PhaseGuard { _private: () }
}

// ─── counters ───────────────────────────────────────────────────────────────

pub fn incr_formulae() {
  STATE.with(|s| s.borrow_mut().formulae += 1);
}

/// Set the formulae count directly. Use when the document-wide count
/// is known up front (e.g., right before `MathParser::parse_math` is
/// invoked over all `<XMath>` nodes).
pub fn set_formulae(n: u32) {
  STATE.with(|s| s.borrow_mut().formulae = n);
}

/// Record one math parse: total time and number of successful parses
/// returned (the Marpa parser may produce multiple ASF derivations
/// for one input). Updates the histogram bucket for the elapsed time.
pub fn record_math_parse(us: u64, parses: u32) {
  STATE.with(|s| {
    let mut t = s.borrow_mut();
    t.math_parse_attempts += 1;
    t.math_parse_count += parses as u64;
    t.math_parse_buckets[bucket_for(us)] += 1;
  });
}

pub fn incr_graphics_asset() {
  STATE.with(|s| s.borrow_mut().graphics_assets += 1);
}
pub fn incr_graphics_subprocess() {
  STATE.with(|s| s.borrow_mut().graphics_subprocess_count += 1);
}
pub fn incr_external_tool() {
  STATE.with(|s| s.borrow_mut().external_tool_count += 1);
}
pub fn set_db_objects(n: u32) {
  STATE.with(|s| s.borrow_mut().db_objects = n);
}
pub fn set_output_bytes(n: u64) {
  STATE.with(|s| s.borrow_mut().output_bytes = n);
}
pub fn incr_warning() {
  STATE.with(|s| s.borrow_mut().warnings += 1);
}
pub fn incr_error() {
  STATE.with(|s| s.borrow_mut().errors += 1);
}
pub fn incr_fatal_error() {
  STATE.with(|s| s.borrow_mut().fatal_errors += 1);
}

// ─── identifiers (binary-set) ───────────────────────────────────────────────

pub fn set_paper_id(id: &str) {
  STATE.with(|s| s.borrow_mut().paper_id = id.to_string());
}
pub fn set_cmdline(s: &str) {
  STATE.with(|st| st.borrow_mut().cmdline = s.to_string());
}
pub fn set_host(h: &str) {
  STATE.with(|s| s.borrow_mut().host = h.to_string());
}
pub fn set_timeout_s(t: u32) {
  STATE.with(|s| s.borrow_mut().timeout_s = t);
}
pub fn set_category(c: &str) {
  STATE.with(|s| s.borrow_mut().category = c.to_string());
}
pub fn set_exit_code(e: i32) {
  STATE.with(|s| s.borrow_mut().exit_code = e);
}
pub fn set_wall_us(w: u64) {
  STATE.with(|s| s.borrow_mut().wall_us = w);
}
pub fn set_max_rss_kb(r: u64) {
  STATE.with(|s| s.borrow_mut().max_rss_kb = r);
}
pub fn set_child_rusage_us(user: u64, sys: u64) {
  STATE.with(|s| {
    let mut t = s.borrow_mut();
    t.child_user_us = user;
    t.child_sys_us = sys;
  });
}

/// Take the current telemetry record, replacing it with a fresh
/// default. Use at end-of-process to serialize the result.
pub fn take() -> Telemetry {
  STATE.with(|s| std::mem::take(&mut *s.borrow_mut()))
}

/// Read-only view for tests / instrumented assertions.
pub fn with<R>(f: impl FnOnce(&Telemetry) -> R) -> R {
  STATE.with(|s| f(&s.borrow()))
}

// ─── JSON serialization ─────────────────────────────────────────────────────

fn write_json_string(out: &mut String, s: &str) {
  out.push('"');
  for c in s.chars() {
    match c {
      '"' => out.push_str("\\\""),
      '\\' => out.push_str("\\\\"),
      '\n' => out.push_str("\\n"),
      '\r' => out.push_str("\\r"),
      '\t' => out.push_str("\\t"),
      c if (c as u32) < 0x20 => {
        use std::fmt::Write;
        write!(out, "\\u{:04x}", c as u32).unwrap();
      },
      c => out.push(c),
    }
  }
  out.push('"');
}

impl Telemetry {
  /// Serialize as a single-line JSON object (suitable for JSONL).
  /// Hand-written to avoid pulling serde into latexml_core.
  // The `field!` macro always writes `first = false` after emitting; on the
  // very last field that final write is naturally dead. Silence the warning.
  #[allow(unused_assignments)]
  pub fn to_json_line(&self) -> String {
    use std::fmt::Write;
    let mut s = String::with_capacity(1024);
    s.push('{');

    let mut first = true;
    macro_rules! field {
      ($name:literal, $val:expr) => {{
        if !first {
          s.push(',');
        }
        first = false;
        s.push('"');
        s.push_str($name);
        s.push_str("\":");
        write!(s, "{}", $val).unwrap();
      }};
    }
    macro_rules! field_str {
      ($name:literal, $val:expr) => {{
        if !first {
          s.push(',');
        }
        first = false;
        s.push('"');
        s.push_str($name);
        s.push_str("\":");
        write_json_string(&mut s, $val);
      }};
    }
    macro_rules! field_array_u64 {
      ($name:literal, $arr:expr) => {{
        if !first {
          s.push(',');
        }
        first = false;
        s.push('"');
        s.push_str($name);
        s.push_str("\":[");
        for (i, v) in $arr.iter().enumerate() {
          if i > 0 {
            s.push(',');
          }
          write!(s, "{}", v).unwrap();
        }
        s.push(']');
      }};
    }
    macro_rules! field_array_u32 {
      ($name:literal, $arr:expr) => {{
        field_array_u64!($name, $arr);
      }};
    }

    field_str!("paper_id", &self.paper_id);
    field_str!("git_sha", &self.git_sha);
    field_str!("cmdline", &self.cmdline);
    field_str!("host", &self.host);
    field!("timeout_s", self.timeout_s);
    field!("schema_version", self.schema_version);
    field!("wall_us", self.wall_us);
    field_array_u64!("phase_us", &self.phase_us);
    // Per-phase aliases for grep convenience
    for (i, val) in self.phase_us.iter().enumerate() {
      let phase = match i {
        0 => Phase::Bootstrap,
        1 => Phase::Digest,
        2 => Phase::Build,
        3 => Phase::Rewrite,
        4 => Phase::MathParse,
        5 => Phase::PostXmlParse,
        6 => Phase::PostScan,
        7 => Phase::Bibliography,
        8 => Phase::Crossref,
        9 => Phase::Graphics,
        10 => Phase::MathImages,
        11 => Phase::MathmlPres,
        12 => Phase::MathmlCont,
        13 => Phase::Split,
        14 => Phase::Xslt,
        15 => Phase::Html5Fixups,
        16 => Phase::Serialize,
        _ => unreachable!(),
      };
      s.push_str(",\"phase_");
      s.push_str(phase.as_str());
      s.push_str("_us\":");
      write!(s, "{}", val).unwrap();
    }
    field!("formulae", self.formulae);
    field!("math_parse_attempts", self.math_parse_attempts);
    field!("math_parse_count", self.math_parse_count);
    field_array_u32!("math_parse_buckets", &self.math_parse_buckets);
    field!("graphics_assets", self.graphics_assets);
    field!("graphics_subprocess_count", self.graphics_subprocess_count);
    field!("db_objects", self.db_objects);
    field!("output_bytes", self.output_bytes);
    field!("warnings", self.warnings);
    field!("errors", self.errors);
    field!("fatal_errors", self.fatal_errors);
    field!("external_tool_count", self.external_tool_count);
    field!("max_rss_kb", self.max_rss_kb);
    field!("child_user_us", self.child_user_us);
    field!("child_sys_us", self.child_sys_us);
    field_str!("category", &self.category);
    field!("exit_code", self.exit_code);

    s.push('}');
    s
  }
}

// ─── tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
  use super::*;
  use std::thread::sleep;
  use std::time::Duration;

  #[test]
  fn bucket_boundaries() {
    assert_eq!(bucket_for(0), 0);
    assert_eq!(bucket_for(499), 0);
    assert_eq!(bucket_for(500), 1);
    assert_eq!(bucket_for(999), 1);
    assert_eq!(bucket_for(1_000), 2);
    assert_eq!(bucket_for(99_999), 7);
    assert_eq!(bucket_for(100_000), 8);
    assert_eq!(bucket_for(1_000_000), 8);
  }

  #[test]
  fn nested_phases_charge_innermost_only() {
    let _g_outer = phase(Phase::Bootstrap);
    sleep(Duration::from_micros(200));
    {
      let _g_inner = phase(Phase::Digest);
      sleep(Duration::from_micros(500));
    }
    sleep(Duration::from_micros(200));
    drop(_g_outer);

    let t = take();
    // Bootstrap should have accumulated time outside the Digest scope.
    assert!(t.phase_us[Phase::Bootstrap as usize] >= 300, "bootstrap got {}", t.phase_us[0]);
    // Digest should have ~500us (with slack for sleep imprecision).
    assert!(
      t.phase_us[Phase::Digest as usize] >= 400,
      "digest got {}",
      t.phase_us[Phase::Digest as usize]
    );
  }

  #[test]
  fn json_round_trip_basic_fields() {
    set_paper_id("0901.0001");
    set_host("test-host");
    set_timeout_s(120);
    set_category("ok");
    set_exit_code(0);
    incr_formulae();
    incr_formulae();
    record_math_parse(750, 3);
    record_math_parse(50_000, 1);
    let t = take();
    let json = t.to_json_line();
    assert!(json.starts_with('{'));
    assert!(json.ends_with('}'));
    assert!(json.contains("\"paper_id\":\"0901.0001\""));
    assert!(json.contains("\"formulae\":2"));
    assert!(json.contains("\"math_parse_attempts\":2"));
    assert!(json.contains("\"math_parse_count\":4"));
    // 750us → bucket 1 (>= 500, < 1000)
    // 50_000us → bucket 7 (>= 20_000, < 100_000)
    assert!(
      json.contains("\"math_parse_buckets\":[0,1,0,0,0,0,0,1,0]"),
      "buckets in json: {}",
      json
    );
  }

  #[test]
  fn json_escapes_string_fields() {
    set_paper_id("a \"weird\" id\nwith newline");
    set_cmdline("cmd\twith\ttabs");
    let t = take();
    let json = t.to_json_line();
    assert!(json.contains("\\\"weird\\\""));
    assert!(json.contains("\\n"));
    assert!(json.contains("\\t"));
  }
}
