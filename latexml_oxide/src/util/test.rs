use std::{
  path::PathBuf,
  sync::{LazyLock, Once},
};

use glob::glob;
use latexml_core::{Core, CoreOptions, common::BindingDispatcher, document::Document, s};

use crate::core_interface::DigestionAPI;

mod requirements;
pub use requirements::{
  GoldenVariant, JING, RequirementTable, Requirements, check_all_requirements, check_requirements,
  check_table, current_test_name, file_available, fixtures, loads, skip, skip_log_path,
  texlive_version,
};

// Process-once cached env vars (see WISDOM #56 — getenv hot-path race).
// Sampled at static init; subsequent reads are atomic loads.
static TEST_LOG: LazyLock<bool> = LazyLock::new(|| std::env::var("LATEXML_TEST_LOG").is_ok());
static SIGSEGV_TRACE: LazyLock<bool> =
  LazyLock::new(|| std::env::var("LATEXML_SIGSEGV_TRACE").is_ok());
static SAVE_ACTUAL: LazyLock<bool> = LazyLock::new(|| std::env::var("LATEXML_SAVE_ACTUAL").is_ok());
/// "Bless" / regenerate mode — the Rust equivalent of Perl's `tools/maketests`.
/// When `LATEXML_BLESS=1`, a `…_ok` test writes the ACTUAL conversion output to
/// its golden `.xml` (overwriting it) instead of comparing+asserting. Run via
/// `tools/maketests.sh` (optionally with a test-name filter). Because it reuses
/// the exact harness conversion + serialization (`process_texfile`), the
/// regenerated golden is byte-identical to what the comparison expects.
static BLESS: LazyLock<bool> = LazyLock::new(|| std::env::var("LATEXML_BLESS").is_ok());

pub fn latexml_tests(
  dirpath: &str,
  requires: Option<&RequirementTable>,
  dispatcher_opt: Option<BindingDispatcher>,
) {
  latexml_tests_internal(dirpath, requires, dispatcher_opt)
}
pub fn latexml_tests_internal(
  dirpath: &str,
  requires: Option<&RequirementTable>,
  dispatcher_opt: Option<BindingDispatcher>,
) {
  for tex_file in glob(&s!("{}/*.tex", dirpath)).unwrap().flatten() {
    let name = tex_file.file_stem().unwrap().to_str().unwrap();
    let xml_file = tex_file.with_extension("xml");

    let tex_file_string = tex_file.to_str().unwrap();
    let xml_file_str = xml_file.to_str().unwrap();
    if xml_file.exists() && check_table(dirpath, name, requires) {
      latexml_ok_internal(tex_file_string, xml_file_str, name, dispatcher_opt.clone());
    } else {
      // Skip, these could be tex fragment files.
    }
  }
}

static INIT_LOGGER: Once = Once::new();
pub fn init_logger() {
  INIT_LOGGER.call_once(|| {
    // Use Off level for clean test output. Error/Warn counting still works
    // via note_status(); set LATEXML_TEST_LOG=1 to see warnings during debugging.
    let level = if *TEST_LOG {
      log::LevelFilter::Warn
    } else {
      log::LevelFilter::Off
    };
    latexml_core::util::logger::init(level).unwrap();
  });
}

static INIT_TEST_RSS_CAP: Once = Once::new();
/// Raise the per-process RSS fuse for the multi-conversion test harness.
///
/// `latexml_core::stomach`'s memory budget defaults to **4.5 GB**, sized to
/// bound a *single* conversion — that low default is load-bearing in production,
/// where a massively parallel fleet runs many one-paper processes at once and
/// the aggregate host RSS is `N × cap` (raising it would OOM the machine).
///
/// `cargo test` is the one place that runs many conversions in ONE process:
/// libtest spawns a thread per test, so at high parallelism (e.g. `-j128` on a
/// many-core box) the process-wide RSS is the *sum* of all in-flight
/// conversions and trips the single-conversion fuse on otherwise-fine
/// documents (a false `MemoryBudget` cascade on article/book/report …). So the
/// harness — not the production default — raises the cap, once, here. An
/// explicit `LATEXML_RSS_CAP_BYTES` (or `--test-threads=N`) still wins.
///
/// `pub` so hand-written integration suites (e.g. `06_cluster_regressions.rs`)
/// that drive `Converter` directly — bypassing `latexml_test_single` — can opt
/// into the same raised cap. Otherwise their conversions run under the low
/// production default and trip a false `MemoryBudget` cascade at
/// `--test-threads=2` once the file accumulates enough in-flight RSS.
pub fn init_test_rss_cap() {
  INIT_TEST_RSS_CAP.call_once(|| {
    if std::env::var_os("LATEXML_RSS_CAP_BYTES").is_none() {
      // SAFETY: set exactly once under `Once`, at the very top of every
      // generated test (before any conversion thread reads the env in
      // `stomach::check_timeout`). `Once`'s release/acquire ordering
      // happens-before all those reads, so there is no setenv/getenv race.
      let mib = latexml_core::watchdog::default_ceiling_mib().max(16_000);
      let bytes = mib.saturating_mul(1024 * 1024);
      unsafe {
        std::env::set_var("LATEXML_RSS_CAP_BYTES", bytes.to_string());
      }
    }
  });
}

// Linker-section trick: register a SIGSEGV handler via `.init_array` so
// it runs BEFORE `main()` (and therefore before any thread the test
// harness spawns can crash). `init_logger` was too late — by the time
// the first test thread called it, sibling threads had already crashed.
//
// Gated by `LATEXML_SIGSEGV_TRACE` so the handler is opt-in; signal()
// + std::backtrace inside a SIGSEGV handler is technically not async-
// signal-safe (uses heap), but for diagnostic purposes a best-effort
// stack dump is far more useful than the bare `signal: 11` line cargo
// reports today.
#[used]
#[cfg_attr(target_os = "linux", unsafe(link_section = ".init_array"))]
static SIGSEGV_INSTALLER: extern "C" fn() = sigsegv_installer;

extern "C" fn sigsegv_installer() {
  // Read env at process start. SAFETY: nothing has run yet, env is
  // populated by the kernel/loader.
  if *SIGSEGV_TRACE {
    eprintln!("[sigsegv_installer] installing SIGSEGV handler");
    install_sigsegv_handler();
  }
}

/// Install a SIGSEGV handler that prints the crashing thread's name and a
/// best-effort backtrace before the kernel terminates the process. This
/// is a diagnostic-only hook — the handler is not async-signal-safe (it
/// uses `std::backtrace` and `eprintln!`, which malloc), but for
/// post-mortem of the libxml2-suspected multi-thread crash that's
/// observed under `cargo test --release --tests`, even an unsafe
/// stack print is more useful than the bare `signal: 11` line cargo
/// reports today.
fn install_sigsegv_handler() {
  // SAFETY: declares the libc `signal(2)`/`raise(3)` FFI bindings for this
  // test-only crash-backtrace handler. The signatures match libc's
  // (`sighandler_t` is a `usize`-wide fn pointer here; `raise` returns int);
  // calls below uphold the platform contract.
  unsafe extern "C" {
    fn signal(sig: i32, handler: extern "C" fn(i32)) -> usize;
    fn raise(sig: i32) -> i32;
  }
  // Linux SIGSEGV = 11, SIGBUS = 7, SIGABRT = 6.
  const SIGSEGV: i32 = 11;
  const SIGBUS: i32 = 7;
  const SIGABRT: i32 = 6;
  const SIG_DFL: usize = 0;

  extern "C" fn handler(sig: i32) {
    // Capture context synchronously and persist to a per-pid file —
    // cargo test buffers/discards stderr from binaries that exit by
    // signal, so eprintln!() never reaches the user. Writing to
    // `<temp_dir>/latexml_sigsegv_<pid>.txt` survives the kill.
    let tid = std::thread::current().id();
    let name = std::thread::current()
      .name()
      .unwrap_or("<unnamed>")
      .to_string();
    let pid = std::process::id();
    let path = std::env::temp_dir()
      .join(format!("latexml_sigsegv_{pid}.txt"))
      .display()
      .to_string();
    let bt = std::backtrace::Backtrace::force_capture();
    let exe = std::env::current_exe()
      .map(|p| p.display().to_string())
      .unwrap_or_else(|_| "<unknown>".into());
    let body = format!(
      "=== SIGSEGV-handler ===\nsignal={sig}\nthread={name:?}\nid={tid:?}\nexe={exe}\npid={pid}\n\n{bt}\n"
    );
    let _ = std::fs::write(&path, &body);
    // Also try eprintln (best effort; usually lost by cargo on signal).
    eprintln!("{body}");
    eprintln!("[SIGSEGV-handler] full trace written to {path}");
    // Reset to default and re-raise so cargo still sees the original signal.
    // SAFETY: test-only crash-backtrace handler (not async-signal-safe by
    // design — see the fn docs). `transmute(SIG_DFL)` reinterprets the
    // null/0 SIG_DFL sentinel as the `sighandler_t`-shaped fn pointer libc
    // expects, restoring the default disposition; `signal`/`raise` then
    // re-raise `sig` on the current thread so cargo observes the original
    // fatal signal.
    unsafe {
      let raw_dfl: extern "C" fn(i32) = std::mem::transmute(SIG_DFL);
      signal(sig, raw_dfl);
      raise(sig);
    }
  }

  // SAFETY: installs the test-only `handler` as the disposition for SIGSEGV/
  // SIGBUS/SIGABRT via libc `signal(2)`. `handler` is a valid `extern "C"
  // fn(i32)` matching the expected `sighandler_t`; it is intentionally
  // limited to (mostly) async-signal-safe work — see the fn docs for the
  // accepted best-effort/heap caveat.
  unsafe {
    signal(SIGSEGV, handler);
    signal(SIGBUS, handler);
    signal(SIGABRT, handler);
  }
}

/// **Intentionally-failing tests** — a *permanent contract* that this input
/// SHOULD produce errors. The TeX is genuinely ill-formed / pathological, so
/// erroring is the CORRECT, desired outcome forever — NOT something to "fix".
/// (The discriminator is the input's validity, NOT whether Perl also errors:
/// see `ERROR_DEBT` for valid inputs that merely error today.)
///
/// The contract is a SOFT, RECOVERABLE error: the harness asserts the **exact**
/// `Error:` count AND that there was **no `Fatal:`** — the whole point is that
/// the engine recovers and completes the conversion (graceful degradation),
/// never crashes. Drift fails BOTH ways: *more*/fatal = a handling regression;
/// *zero* = we silently STOPPED detecting the bad input. Logged
/// `[intentional-fail]`.
const INTENTIONALLY_FAILING: &[(&str, usize, &str)] = &[
  (
    "protect_self_ref",
    1,
    "intrinsic self-recursion (\\def\\cs{\\protect\\cs} typeset): pdflatex HANGS, \
     Perl+Rust both emit 1 SOFT error — the recursion guard prevents the hang. \
     Verified 2026-06-10 (latexml --verbose=1 error, pdflatex=timeout).",
  ),
  (
    "io",
    2,
    "deliberate malformed read content: `exists.data` line 21 has an unbalanced \
     brace (`line { with extra } }`) to verify the engine emits a SOFT, \
     recoverable error (not Fatal) and completes. pdflatex also errors+recovers \
     (`! Too many }'s, silently discards }`); Perl+Rust both emit 2 soft errors. \
     Verified 2026-06-10.",
  ),
  (
    "undefined_env",
    1,
    "undefined environment `\\begin{undefinedenv}`: genuinely erroneous (missing \
     defining package). Both Perl+Rust emit 1 SOFT error AND a visible \
     `<ltx:ERROR class='undefined'>{undefinedenv}</ltx:ERROR>` marker (Perl \
     `makeError`, latex_constructs.pool.ltxml:207-208). Guards the fix where Rust \
     formerly dropped the ERROR element (no-op trigger). Verified vs \
     /usr/local/bin/latexml.",
  ),
];

/// **Error debt** — valid input we INTEND to convert cleanly (a desired,
/// surpass-Perl success), but which errors today. TEMPORARY: each MUST be
/// driven to zero by improving the Rust core, then removed. The harness
/// tolerates ANY count (logged `[error-debt]`) and does NOT fail at zero,
/// because the count is **environment-dependent** for some entries (e.g.
/// `glossary` errors on one host's datatool/expl3 but converts clean in CI) —
/// failing at zero would break whichever environment is already clean. When an
/// entry's `[error-debt] … 0 errors` shows up EVERYWHERE (in captured output: run
/// with `--nocapture` to see it on a green run), remove it by review.
/// Each note records Perl's current behavior (verify with `latexml --verbose`
/// — `--quiet` HIDES Perl errors). Tracked in `docs/SYNC_STATUS.md`.
///
/// Currently EMPTY: the last entry (`figure_mixed_content` —
/// `ltx:theorem`/`ltx:proof` not allowed in `ltx:figure`/`ltx:table`/`ltx:float`)
/// was drained 2026-06-27 by the schema expansion in
/// `resources/RelaxNG/LaTeXML.model` + `LaTeXML-para.{rng,rnc}` (a boxed theorem/
/// proof inside a float is valid LaTeX; both engines previously rejected it). The
/// fix is output-neutral — the builder already placed the theorem inside the
/// figure, so only the spurious malformed-error is gone (XML byte-identical).
const ERROR_DEBT: &[(&str, &str)] = &[];

/// The `[error-debt]` and `[intentional-fail]` notices: in the test's captured
/// output, shown with it when it fails (`--nocapture` shows them on a green run).
/// They once went to fd 2 directly to be seen on every green run (review m2); a
/// green run now prints nothing (user directive 2026-09-28).
fn note(line: &str) {
  eprintln!("{line}");
}

pub fn latexml_test_single(
  tex_file_str: &str,
  name: &str,
  dirpath: &str,
  requires: Option<&RequirementTable>,
  dispatcher_opt: Option<BindingDispatcher>,
) {
  init_logger();
  init_test_rss_cap();
  if !check_table(dirpath, name, requires) {
    return;
  }
  // Suppress log output for any test expected to emit errors (both categories)
  // so single-test runs stay readable; the gate still counts + classifies them.
  let suppress = INTENTIONALLY_FAILING.iter().any(|(n, ..)| *n == name)
    || ERROR_DEBT.iter().any(|(n, _)| *n == name);
  if suppress {
    latexml_core::common::error::set_suppress_log_output(true);
  }
  let tex_file = PathBuf::from(tex_file_str);
  let xml_file = tex_file.with_extension("xml");
  // (a newer package's golden, where the installed file is that release: `Requirements::golden_variant`)
  let variant = requirements::golden_variant(name, requires);
  let xml_file = match variant {
    Some(variant) => {
      let variant_file = tex_file.with_extension(format!("{variant}.xml"));
      assert!(
        *BLESS || variant_file.is_file(),
        "{name}: no golden {} for the installed package",
        variant_file.display()
      );
      variant_file
    },
    None => xml_file,
  };
  if variant.is_some() || matches!(xml_file.try_exists(), Ok(true)) {
    latexml_ok_internal(
      tex_file_str,
      &xml_file.to_string_lossy(),
      name,
      dispatcher_opt,
    );
  } else {
    // Skip, these could be tex fragment files.
  }
  if suppress {
    latexml_core::common::error::set_suppress_log_output(false);
  }
}

// fn latexml_ok(tex_path: &str, xml_path: &str, name: &str) { latexml_ok_internal(tex_path,
// xml_path, name, None) }

fn latexml_ok_internal(
  tex_path: &str,
  xml_path: &str,
  name: &str,
  extra_bindings_dispatcher: Option<BindingDispatcher>,
) {
  let tex_strings = process_texfile(tex_path, name, extra_bindings_dispatcher);
  // Bless / regenerate mode (Perl `tools/maketests` equivalent): overwrite the
  // golden with the actual output instead of comparing. Git is the backup.
  if *BLESS {
    if tex_strings.is_empty() {
      eprintln!("BLESS skip {name:?}: conversion produced no output (not overwriting {xml_path})");
      return;
    }
    let body = format!("{}\n", tex_strings.join("\n"));
    match std::fs::write(xml_path, &body) {
      Ok(()) => eprintln!("BLESS wrote {xml_path} ({} lines)", tex_strings.len()),
      Err(e) => eprintln!("BLESS FAILED to write {xml_path}: {e}"),
    }
    return;
  }
  if !tex_strings.is_empty() {
    let xml_strings = process_xmlfile(xml_path, name);
    if !xml_strings.is_empty() {
      let mut found_diff = false;
      for (lineno, (tex_line, xml_line)) in tex_strings.iter().zip(xml_strings.iter()).enumerate() {
        if tex_line != xml_line {
          found_diff = true;
          eprintln!(
            "DIFF line {lineno} in {xml_path}:\n  ACTUAL:   {tex_line}\n  EXPECTED: {xml_line}"
          );
        }
      }
      if tex_strings.len() != xml_strings.len() {
        found_diff = true;
        eprintln!(
          "DIFF length mismatch for {name:?}: actual {} lines, expected {} lines",
          tex_strings.len(),
          xml_strings.len()
        );
        // Print extra lines
        let min_len = tex_strings.len().min(xml_strings.len());
        if tex_strings.len() > min_len {
          for (i, line) in tex_strings[min_len..].iter().enumerate() {
            eprintln!("  ACTUAL extra line {}: {line}", min_len + i);
          }
        }
        if xml_strings.len() > min_len {
          for (i, line) in xml_strings[min_len..].iter().enumerate() {
            eprintln!("  EXPECTED extra line {}: {line}", min_len + i);
          }
        }
      }
      if found_diff {
        panic!("Differences found in {xml_path} — see DIFF lines above");
      }
    }
  }
}

/// Returns the list-of-strings form of whatever was requested, if successful,
/// otherwise empty; and they will have reported the failure
fn process_texfile(
  tex_path: &str,
  name: &str,
  extra_bindings_dispatcher: Option<BindingDispatcher>,
) -> Vec<String> {
  let mut latexml = Core::new(CoreOptions {
    verbosity: Some(-2),
    search_paths: None,
    preload: None,
    include_comments: Some(false),
    ..CoreOptions::default()
  });
  // Install the SAME binding-resolution priority chain a real conversion uses
  // (rhai > extra/contrib > package), via the shared helper. This is what lets a
  // test resolve a local `<pkg>.<ext>.rhai` fixture sitting next to its `.tex`
  // (found through the source-dir search path) — exactly as the Perl suite
  // resolves a local `<pkg>.<ext>.ltxml`. The `extra` dispatcher (passed by test
  // groups that still rely on compiled `latexml_contrib` fixtures) is folded in
  // as tier 2.
  crate::converter::install_binding_dispatch(extra_bindings_dispatcher);
  let r = match latexml.convert_file(tex_path.to_owned()) {
    Err(e) => panic!("{:?}: Couldn't convert {:?}; {:?}", name, tex_path, e),
    Ok(doc) => process_ltx_doc(doc, name),
  };
  // Drop the engine, then free this thread's accumulated thread-local
  // state. libtest spawns a fresh thread per test, and the engine's
  // roots are `#[thread_local]` *attribute* statics, which do NOT run
  // destructors on thread exit — so without this each test would leak
  // its ~110 MB engine, accumulating to ~4.9 GB across the suite. The
  // output is already owned `String`s by now, so no live `SymStr`
  // survives the reset. See `latexml_core::reset_thread_engine`.
  // Error gate: every `.tex`/`.xml` regression test is an error-regression
  // sentinel, not just an XML-shape check. `note_status` counts `Error:`/
  // `Fatal:` even when log output is off, so this is the canonical signal.
  // Three contracts:
  //   • normal test            → MUST be error-clean (n_err == 0).
  //   • INTENTIONALLY_FAILING  → MUST emit its exact SOFT count, NEVER fatal
  //                              (graceful recovery is the contract; permanent).
  //   • ERROR_DEBT             → tolerated (count is env-dependent for some);
  //                              logged, never fails; manual review for removal.
  //
  // Perf (runs on every test): the hot path is two thread-local integer reads
  // via `get_status` (no allocation, no log-string scan) plus tiny slice
  // lookups; messages build only on the cold panic path. `convert_file` reset
  // the report at conversion start, so this count is exactly this conversion's.
  // Read BEFORE `reset_thread_engine`.
  use latexml_core::common::error::{LogStatus, get_status};
  let n_soft = get_status(LogStatus::Error);
  let n_fatal = get_status(LogStatus::Fatal);
  let n_err = n_soft + n_fatal;
  let intentional = INTENTIONALLY_FAILING
    .iter()
    .find(|(n, ..)| *n == name)
    .copied();
  let debt = ERROR_DEBT.iter().find(|(n, _)| *n == name).copied();
  // Decide the verdict (and any cold-path message) before tearing down.
  let verdict: Result<(), String> = match (intentional, debt) {
    // Permanent contract: exact SOFT-error count, and NEVER fatal — the point is
    // graceful recovery. Drift fails both ways; a Fatal is always a regression.
    (Some((_, expect, reason)), _) => {
      note(&format!(
        "[intentional-fail] {name}: {n_soft} soft errors, {n_fatal} fatal (expect {expect}, 0) — {reason}"
      ));
      if n_fatal > 0 {
        Err(format!(
          "{name}: INTENTIONALLY_FAILING must degrade to a SOFT error, but got a Fatal \
           ({}) — graceful recovery regressed. Reason: {reason}",
          latexml_core::common::error::get_status_message()
        ))
      } else if n_soft == expect {
        Ok(())
      } else if n_soft == 0 {
        Err(format!(
          "{name}: INTENTIONALLY_FAILING expects {expect} error(s) but got 0 — error \
           detection regressed (this input must still error). Reason: {reason}"
        ))
      } else {
        Err(format!(
          "{name}: INTENTIONALLY_FAILING expects exactly {expect} soft error(s), got {n_soft} \
           ({}) — handling drifted. Reason: {reason}",
          latexml_core::common::error::get_status_message()
        ))
      }
    },
    // Temporary debt: tolerate whatever it does today (the count is
    // environment-dependent for some entries — e.g. `glossary` errors on one
    // box but converts clean in CI, per the host's datatool/expl3 version), so
    // the gate does NOT fail at zero. Removal is a manual review step when an
    // entry is clean EVERYWHERE (the `[error-debt] … 0 errors` log flags it).
    (None, Some((_, reason))) => {
      if n_err == 0 {
        note(&format!(
          "[error-debt] {name}: 0 errors HERE — clean in this \
          environment; review for removal once clean everywhere — {reason}"
        ));
      } else {
        note(&format!("[error-debt] {name}: {n_err} errors — {reason}"));
      }
      Ok(())
    },
    // Normal test: must be error-clean.
    (None, None) => {
      if n_err == 0 {
        Ok(())
      } else {
        Err(format!(
          "{name}: conversion logged errors ({}) — a normal-TeX test must be error-clean. \
           Fix the engine/binding/specimen. If the input SHOULD error (verify with \
           bin/latexml --verbose), add to INTENTIONALLY_FAILING; if it should convert \
           clean but doesn't yet, add to ERROR_DEBT with a SYNC_STATUS entry. See \
           docs/archive/MALFORMED_CLOSE_NUMBERED_2026-06-10.md.",
          latexml_core::common::error::get_status_message()
        ))
      }
    },
  };
  drop(latexml);
  latexml_core::reset_thread_engine();
  if let Err(msg) = verdict {
    panic!("{msg}");
  }
  r
}

/// Loads the reference XML file as raw text lines, avoiding libxml2
/// re-serialization which would normalize `<p></p>` to `<p/>`.
fn process_xmlfile<'a>(xml_path: &'a str, _name: &'a str) -> Vec<String> {
  match std::fs::read_to_string(xml_path) {
    Err(e) => panic!("Failed to read XML file {:?}: {:?}", xml_path, e),
    Ok(contents) => {
      let mut lines: Vec<String> = contents.split('\n').map(ToString::to_string).collect();
      // Remove trailing empty line from final newline
      if lines.last().is_some_and(|l| l.is_empty()) {
        lines.pop();
      }
      lines
    },
  }
}
fn process_ltx_doc(doc: Document, name: &str) -> Vec<String> {
  let doc_str = doc.serialize_to_string();
  if *SAVE_ACTUAL {
    let tmp = std::env::temp_dir();
    let path = tmp
      .join(format!("latexml_actual_{name}.xml"))
      .display()
      .to_string();
    std::fs::write(&path, &doc_str).ok();
    eprintln!("Saved actual XML to {path}");
    // Also save using libxml's built-in serializer for comparison
    let path2 = tmp
      .join(format!("latexml_actual_{name}_libxml.xml"))
      .display()
      .to_string();
    let libxml_str = doc
      .document
      .to_string_with_options(libxml::tree::SaveOptions {
        format: true,
        ..libxml::tree::SaveOptions::default()
      });
    std::fs::write(&path2, &libxml_str).ok();
    eprintln!("Saved libxml XML to {path2}");
  }
  let mut lines: Vec<String> = doc_str.split('\n').map(ToString::to_string).collect();
  // Remove trailing empty line from final newline
  if lines.last().is_some_and(|l| l.is_empty()) {
    lines.pop();
  }
  lines
}

// `new_test_engine` and `lex_single_tex_formula` moved to
// `crate::util::preset` (audit DEP-02, 2026-05-18). They have no
// dependency on `glob` and need to be callable from the
// `latexmlmath_oxide` production binary, which builds without the
// `test-utils` feature. Re-exported here so the dominant
// `use latexml::util::test::*;` pattern in integration tests
// continues to work unchanged.
pub use super::preset::{lex_single_tex_formula, new_test_engine};

/// Build a test function for each "*.tex" source found in a given directory path.
/// The path should be absolute, or relative to the root latexml-oxide checkout.
#[macro_export]
macro_rules! tex_tests {
  ($dir:literal) => {
    tex_tests!($dir, None, None);
  };
  ($dir:literal, $requires:expr_2021, $dispatch:expr_2021) => {
    macro_rules! this_test_requires {
      () => {
        $requires
      };
    }
    macro_rules! this_test_dispatch {
      () => {
        $dispatch
      };
    }
    use latexml_codegen::GlobTeXTests;
    #[derive(GlobTeXTests)]
    #[directory=$dir]
    struct _TestDirective;
  };
}

// ======================================================================
// Shared helpers for the standalone integration tests (PR #249 review P3-10).
// The lax error grep, the dump/kpsewhich gates, and the converter
// boilerplate previously lived as per-test-file copies — a drift hazard for
// the project's #1 signal-integrity rule (robust error-log counting).

/// Count inline `Error:<category>:` markers (parity_check.sh's lax pattern, see
/// feedback_strict_vs_lax_error_grep.md). Errors are emitted INLINE within
/// `(Building...Error:..)` envelopes, not at line starts. The category is anything up
/// to the next `:` — `missing_file`, `I/O`, `<char>` count too (WISDOM 85; a
/// lowercase-letters category missed them).
pub fn error_count(log: &str) -> usize {
  log
    .match_indices("Error:")
    .filter(|(i, _)| {
      let tail = &log.as_bytes()[*i + 6..];
      let n_class = tail
        .iter()
        .take_while(|b| **b != b':' && !b.is_ascii_whitespace())
        .count();
      n_class > 0 && tail.get(n_class) == Some(&b':')
    })
    .count()
}

/// The first `<tag …>…</tag>` (or self-closing `<tag …/>`) element of `xml`
/// whose start tag contains every fragment in `attrs` (e.g. `key="smith2020"`),
/// returned WHOLE — start tag, attributes, children, end tag — with the
/// inter-tag whitespace of pretty-printing collapsed (`>\s+<` → `><`, trimmed)
/// so the same element compares equal across serializers. Nesting of the same
/// tag is tracked, so the outermost match is returned complete. `None` when no
/// element matches.
///
/// This is the guard-strength primitive (PLANS.md item 5, user directive
/// 2026-09-17): a guard asserts the entire markup context of its feature, not a
/// substring that unrelated markup can satisfy. `tag` is matched with or
/// without the `ltx:` prefix.
///
/// Under `LATEXML_TEST_SHADOW=<dir>`, a call whose result differs from the first of [`xml_elements`] is appended to
/// `<dir>/<pid>.tsv` (caller, tag, attributes, this result, that one); this result stays the one returned. `<dir>` must be
/// absolute (a test's working directory is its crate's); a relative one, or a log it cannot write, panics the call.
#[track_caller]
pub fn xml_element(xml: &str, tag: &str, attrs: &[&str]) -> Option<String> {
  let found = first_element_by_substring(xml, tag, attrs);
  if let Some(dir) = TEST_SHADOW.as_deref() {
    shadow_compare(
      dir,
      std::panic::Location::caller(),
      xml,
      tag,
      attrs,
      found.as_deref(),
    );
  }
  found
}

/// `LATEXML_TEST_SHADOW`: the directory [`xml_element`] logs its disagreements with [`xml_elements`] to.
static TEST_SHADOW: LazyLock<Option<PathBuf>> =
  LazyLock::new(|| shadow_dir(std::env::var_os("LATEXML_TEST_SHADOW")));

/// The shadow directory of a `LATEXML_TEST_SHADOW` value: unset or empty is none; a relative path panics.
fn shadow_dir(value: Option<std::ffi::OsString>) -> Option<PathBuf> {
  let dir = PathBuf::from(value.filter(|v| !v.is_empty())?);
  assert!(
    dir.is_absolute(),
    "LATEXML_TEST_SHADOW must be an absolute path, not {}",
    dir.display()
  );
  Some(dir)
}

/// Log one [`xml_element`] call whose result `old` differs from the first of [`xml_elements`].
fn shadow_compare(
  dir: &std::path::Path,
  caller: &std::panic::Location,
  xml: &str,
  tag: &str,
  attrs: &[&str],
  old: Option<&str>,
) {
  use std::io::Write;
  let new = element_spans(xml, tag, attrs)
    .first()
    .map(|&(start, end)| normalize_markup(&xml[start..end]));
  if new.as_deref() == old {
    return;
  }
  let cell = |s: Option<&str>| {
    let s = s
      .unwrap_or("<None>")
      .replace('\t', "\\t")
      .replace('\n', "\\n");
    match s.char_indices().nth(160) {
      Some((cut, _)) => format!("{}…", &s[..cut]),
      None => s,
    }
  };
  let line = format!(
    "{}:{}\t{tag}\t{}\t{}\t{}\n",
    caller.file(),
    caller.line(),
    attrs.join(" "),
    cell(old),
    cell(new.as_deref())
  );
  let path = dir.join(format!("{}.tsv", std::process::id()));
  // A lost row would read as "no flips": a write failure fails the test instead.
  std::fs::create_dir_all(dir)
    .and_then(|()| {
      std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    })
    .and_then(|mut log| log.write_all(line.as_bytes()))
    .unwrap_or_else(|e| panic!("LATEXML_TEST_SHADOW: cannot write {}: {e}", path.display()));
}

/// [`xml_element`]'s own scan: the first start tag containing every `attrs` fragment, to its close.
fn first_element_by_substring(xml: &str, tag: &str, attrs: &[&str]) -> Option<String> {
  let bare = tag.strip_prefix("ltx:").unwrap_or(tag);
  let opens = [format!("<{bare}"), format!("<ltx:{bare}")];
  let mut from = 0;
  while let Some(rel) = xml[from..].find('<') {
    let start = from + rel;
    let rest = &xml[start..];
    let Some(open) = opens.iter().find(|o| {
      rest.starts_with(o.as_str())
        && rest[o.len()..]
          .chars()
          .next()
          .is_some_and(|c| c.is_whitespace() || c == '>' || c == '/')
    }) else {
      from = start + 1;
      continue;
    };
    let tag_end = start_tag_end(rest)?;
    let start_tag = &rest[..=tag_end];
    if !attrs.iter().all(|a| start_tag.contains(a)) {
      from = start + 1;
      continue;
    }
    if start_tag.ends_with("/>") {
      return Some(normalize_markup(start_tag));
    }
    // Depth-aware scan to the matching close tag.
    let close = format!("</{}>", &open[1..]);
    let mut depth = 1usize;
    let mut pos = tag_end + 1;
    while depth > 0 {
      let rel = rest[pos..].find('<')?;
      let at = pos + rel;
      if rest[at..].starts_with(&close) {
        depth -= 1;
        pos = at + close.len();
        if depth == 0 {
          return Some(normalize_markup(&rest[..pos]));
        }
      } else if rest[at..].starts_with(open.as_str())
        && rest[at + open.len()..]
          .chars()
          .next()
          .is_some_and(|c| c.is_whitespace() || c == '>' || c == '/')
      {
        let e = start_tag_end(&rest[at..])?;
        if !rest[at..at + e].ends_with('/') {
          depth += 1;
        }
        pos = at + e + 1;
      } else {
        pos = at + 1;
      }
    }
    return None;
  }
  None
}

/// Byte offset of the `>` that closes the start tag beginning at `s[0]`,
/// skipping any `>` inside a quoted attribute value (libxml2 does not escape
/// `>` in attributes: `tex="a>b"`).
fn start_tag_end(s: &str) -> Option<usize> {
  let mut quote: Option<u8> = None;
  for (i, b) in s.bytes().enumerate() {
    match (quote, b) {
      (Some(q), c) if c == q => quote = None,
      (Some(_), _) => {},
      (None, b'"') | (None, b'\'') => quote = Some(b),
      (None, b'>') => return Some(i),
      _ => {},
    }
  }
  None
}

/// Collapse pretty-printing whitespace and trim, so an element's serialization
/// compares equal whatever the indentation: a whitespace run that contains a
/// newline is indentation — dropped when it touches a tag boundary (`>` before
/// it or `<` after it), one space otherwise; a run without a newline is real
/// text spacing and is kept verbatim.
pub fn normalize_markup(s: &str) -> String {
  let mut out = String::with_capacity(s.len());
  let chars: Vec<char> = s.chars().collect();
  let mut i = 0;
  while i < chars.len() {
    if chars[i].is_whitespace() {
      let start = i;
      while i < chars.len() && chars[i].is_whitespace() {
        i += 1;
      }
      let run: String = chars[start..i].iter().collect();
      if run.contains('\n') {
        let at_boundary =
          out.ends_with('>') || chars.get(i) == Some(&'<') || out.is_empty() || i >= chars.len();
        if !at_boundary {
          out.push(' ');
        }
      } else {
        out.push_str(&run);
      }
    } else {
      out.push(chars[i]);
      i += 1;
    }
  }
  sort_start_tag_attributes(out.trim())
}

/// Rewrite every start tag as `<name a="…" b="…">` with its attributes in
/// lexicographic order. Attribute order is not significant in XML, and one
/// emitter (the bibliography's `<ref>` for a `.bib` URL field) orders them by
/// hash — the test-side normalization keeps a pinned element stable; the
/// nondeterminism itself is tracked separately.
fn sort_start_tag_attributes(s: &str) -> String {
  use std::sync::LazyLock;
  static START_TAG: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r#"<([A-Za-z_:][\w:.-]*)((?:\s+[\w:.-]+="[^"]*")+)\s*(/?)>"#).unwrap()
  });
  static ATTR: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r#"([\w:.-]+)="([^"]*)""#).unwrap());
  START_TAG
    .replace_all(s, |c: &regex::Captures| {
      let mut attrs: Vec<(String, String)> = ATTR
        .captures_iter(&c[2])
        .map(|a| (a[1].to_string(), a[2].to_string()))
        .collect();
      attrs.sort();
      let attrs: Vec<String> = attrs
        .into_iter()
        .map(|(k, v)| format!("{k}=\"{v}\""))
        .collect();
      format!("<{} {}{}>", &c[1], attrs.join(" "), &c[3])
    })
    .to_string()
}

/// Assert that the first `<tag …>` element carrying every `attrs` fragment is
/// EXACTLY `expected` (whitespace-normalized on both sides). The failure
/// message prints the element that was found, so a guard can be pinned from a
/// real conversion in one round trip.
#[track_caller]
pub fn assert_element(xml: &str, tag: &str, attrs: &[&str], expected: &str) {
  match xml_element(xml, tag, attrs) {
    Some(found) => {
      let want = normalize_markup(expected);
      assert!(
        found == want,
        "element <{tag} {}> differs from the expected markup\n--- found:\n{found}\n--- expected:\n{want}\n--- xml:\n{xml}",
        attrs.join(" ")
      );
    },
    None => panic!(
      "no <{tag} {}> element in the output\n--- xml:\n{xml}",
      attrs.join(" ")
    ),
  }
}

/// Every element `<tag …>…</tag>` of `xml` whose start tag carries every `attrs` fragment, in document order, nested
/// ones included, each whole and normalized as [`xml_element`] returns it. Unlike [`xml_element`], markup inside a
/// comment, CDATA section or processing instruction is no element; a `name=…` fragment matches at an attribute's start
/// only (`id="p1"` is not `xml:id="p1"`), any other fragment anywhere in the start tag; an HTML void element (`<br>`)
/// is its start tag.
pub fn xml_elements(xml: &str, tag: &str, attrs: &[&str]) -> Vec<String> {
  element_spans(xml, tag, attrs)
    .into_iter()
    .map(|(start, end)| normalize_markup(&xml[start..end]))
    .collect()
}

/// The number of [`xml_elements`] of `xml`.
pub fn element_count(xml: &str, tag: &str, attrs: &[&str]) -> usize {
  element_spans(xml, tag, attrs).len()
}

/// Assert that `xml` holds none of the [`xml_elements`] `tag` with `attrs`, nor such a start tag left unclosed
/// (truncated markup); the failure prints the first one.
#[track_caller]
pub fn assert_no_element(xml: &str, tag: &str, attrs: &[&str]) {
  let scan = scan_elements(xml, tag, attrs);
  if let Some(&(start, end)) = scan.whole.first() {
    panic!(
      "unexpected <{tag} {}> element in the output\n--- found:\n{}\n--- xml:\n{xml}",
      attrs.join(" "),
      normalize_markup(&xml[start..end])
    );
  }
  if let Some(&(start, end)) = scan.unclosed.first() {
    panic!(
      "unexpected <{tag} {}> element in the output\n--- found unclosed {}\n--- xml:\n{xml}",
      attrs.join(" "),
      normalize_markup(&xml[start..end])
    );
  }
}

/// Assert that the [`xml_elements`] `tag` with `attrs` of `xml` are EXACTLY `expected`, in order, each
/// whitespace-normalized as [`assert_element`] normalizes.
#[track_caller]
pub fn assert_all_elements(xml: &str, tag: &str, attrs: &[&str], expected: &[impl AsRef<str>]) {
  let want: Vec<String> = expected
    .iter()
    .map(|e| normalize_markup(e.as_ref()))
    .collect();
  assert_eq!(
    xml_elements(xml, tag, attrs),
    want,
    "the <{tag} {}> elements differ from the expected markup\n--- xml:\n{xml}",
    attrs.join(" ")
  );
}

/// The character data of `markup`: a `<break/>` or `<br>` read as one space, every other tag, comment and processing
/// instruction dropped with no space in its place (so `2<text>021</text>` reads `2021`), CDATA kept, entities and
/// character references decoded, XML whitespace runs collapsed to one space and the ends trimmed. Never use it where
/// whitespace is the signal (listings, verbatim, `\verb`): pin those elements whole.
pub fn markup_text(markup: &str) -> String {
  let mut text = String::with_capacity(markup.len());
  let mut pos = 0;
  while let Some(rel) = markup[pos..].find('<') {
    let at = pos + rel;
    decode_entities(&markup[pos..at], &mut text);
    pos = match markup_item(markup, at) {
      Some((MarkupItem::Cdata, end)) => {
        text.push_str(&markup[at + "<![CDATA[".len()..end - "]]>".len()]);
        end
      },
      Some((MarkupItem::Start { name, .. }, end)) => {
        if matches!(name.strip_prefix("ltx:").unwrap_or(name), "break" | "br") {
          text.push(' ');
        }
        end
      },
      Some((_, end)) => end,
      None => {
        text.push('<');
        at + 1
      },
    };
  }
  decode_entities(&markup[pos..], &mut text);
  text
    .split([' ', '\t', '\r', '\n'])
    .filter(|word| !word.is_empty())
    .collect::<Vec<_>>()
    .join(" ")
}

/// Append `text` to `out` with its predefined and HTML `&nbsp;` entities and its character references decoded; an
/// unknown or unterminated one stays as written.
fn decode_entities(text: &str, out: &mut String) {
  let mut rest = text;
  while let Some(amp) = rest.find('&') {
    out.push_str(&rest[..amp]);
    rest = &rest[amp..];
    let decoded = rest[1..]
      .find(';')
      .filter(|&semi| semi <= 32)
      .and_then(|semi| {
        let name = &rest[1..=semi];
        let c = match name {
          "amp" => Some('&'),
          "lt" => Some('<'),
          "gt" => Some('>'),
          "quot" => Some('"'),
          "apos" => Some('\''),
          "nbsp" => Some('\u{a0}'),
          _ => name.strip_prefix('#').and_then(|num| {
            let code = match num.strip_prefix(['x', 'X']) {
              Some(hex) => u32::from_str_radix(hex, 16).ok(),
              None => num.parse().ok(),
            };
            code.and_then(char::from_u32)
          }),
        };
        c.map(|c| (c, semi + 2))
      });
    match decoded {
      Some((c, len)) => {
        out.push(c);
        rest = &rest[len..];
      },
      None => {
        out.push('&');
        rest = &rest[1..];
      },
    }
  }
  out.push_str(rest);
}

/// What the markup scanner of [`xml_elements`] and [`markup_text`] reads at a `<`.
enum MarkupItem<'a> {
  /// A start tag `<name …>`; `empty` when self-closing (`/>`).
  Start { name: &'a str, empty: bool },
  /// An end tag `</name>`.
  End { name: &'a str },
  /// `<![CDATA[…]]>`.
  Cdata,
  /// A comment, processing instruction or `<!DOCTYPE …>`.
  Other,
}

/// The HTML elements with no end tag (HTML Living Standard §13.1.2 "void elements").
const HTML_VOID_ELEMENTS: &[&str] = &[
  "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
  "track", "wbr",
];

/// The markup item starting at `xml[at] == '<'` and the byte offset just past it; `None` for a `<` that opens no
/// markup (a stray `<` in text) or an item left unterminated.
fn markup_item(xml: &str, at: usize) -> Option<(MarkupItem<'_>, usize)> {
  let rest = &xml[at..];
  let past = |open: usize, close: &str| {
    rest[open..]
      .find(close)
      .map(|e| at + open + e + close.len())
  };
  if rest.starts_with("<!--") {
    return Some((MarkupItem::Other, past(4, "-->")?));
  }
  if rest.starts_with("<![CDATA[") {
    return Some((MarkupItem::Cdata, past(9, "]]>")?));
  }
  if rest.starts_with("<?") {
    return Some((MarkupItem::Other, past(2, "?>")?));
  }
  if rest.starts_with("<!") {
    let end = start_tag_end(rest)?;
    // A `<!DOCTYPE d [ … ]>` internal subset holds `>`s of its own.
    if rest[..end].contains('[') {
      return Some((MarkupItem::Other, past(2, "]>")?));
    }
    return Some((MarkupItem::Other, at + end + 1));
  }
  let (closing, name_at) = if rest.starts_with("</") {
    (true, 2)
  } else {
    (false, 1)
  };
  if !rest[name_at..]
    .chars()
    .next()
    .is_some_and(|c| c.is_alphabetic() || c == '_' || c == ':')
  {
    return None;
  }
  let name_len = rest[name_at..]
    .find(|c: char| c.is_whitespace() || c == '>' || c == '/')
    .unwrap_or(rest.len() - name_at);
  let name = &rest[name_at..name_at + name_len];
  let end = start_tag_end(rest)?;
  let item = if closing {
    MarkupItem::End { name }
  } else {
    let empty = rest[..end].ends_with('/');
    MarkupItem::Start { name, empty }
  };
  Some((item, at + end + 1))
}

/// Whether `start_tag` carries the fragment `attr`: a `name=…` fragment starting one of its attributes, any other
/// fragment as a substring (the fragment contract of [`xml_element`]).
fn start_tag_has(start_tag: &str, attr: &str) -> bool {
  let attr_name = attr.split_once('=').map(|(name, _)| name);
  let is_attribute = attr_name.is_some_and(|name| {
    name.starts_with(|c: char| c.is_alphabetic() || c == '_' || c == ':')
      && name
        .chars()
        .all(|c| c.is_alphanumeric() || matches!(c, '_' | ':' | '.' | '-'))
  });
  if !is_attribute {
    return start_tag.contains(attr);
  }
  let bytes = start_tag.as_bytes();
  let mut i = start_tag
    .find(|c: char| c.is_whitespace() || c == '>' || c == '/')
    .unwrap_or(bytes.len());
  loop {
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
      i += 1;
    }
    if i >= bytes.len() || matches!(bytes[i], b'>' | b'/') {
      return false;
    }
    if start_tag[i..].starts_with(attr) {
      return true;
    }
    // Past this attribute: its name, then an optional `= value`, quoted or bare.
    while i < bytes.len()
      && !matches!(bytes[i], b'=' | b'>' | b'/')
      && !bytes[i].is_ascii_whitespace()
    {
      i += 1;
    }
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
      i += 1;
    }
    if bytes.get(i) != Some(&b'=') {
      continue;
    }
    i += 1;
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
      i += 1;
    }
    match bytes.get(i) {
      Some(&q @ (b'"' | b'\'')) => {
        i += 1
          + start_tag[i + 1..]
            .find(q as char)
            .map_or(bytes.len(), |e| e + 1)
      },
      _ => {
        while i < bytes.len() && !bytes[i].is_ascii_whitespace() && bytes[i] != b'>' {
          i += 1;
        }
      },
    }
  }
}

/// The byte ranges of [`xml_elements`], in document order.
fn element_spans(xml: &str, tag: &str, attrs: &[&str]) -> Vec<(usize, usize)> {
  scan_elements(xml, tag, attrs).whole
}

/// What [`scan_elements`] finds: the whole elements, and the start tags never closed.
struct ElementScan {
  whole:    Vec<(usize, usize)>,
  unclosed: Vec<(usize, usize)>,
}

/// One scan for the elements `tag` with `attrs`, each open element of the name (with or without `ltx:`) on its own
/// stack, so an element closes at the end tag of its own name, as [`xml_element`]'s depth count. An HTML void element
/// left open is its start tag, whole; closed, as XML may write it, it is the element. Any other start tag left open
/// has no extent, so it is no whole element (only [`assert_no_element`] reads it).
fn scan_elements(xml: &str, tag: &str, attrs: &[&str]) -> ElementScan {
  let bare = tag.strip_prefix("ltx:").unwrap_or(tag);
  let prefixed = format!("ltx:{bare}");
  let names = [bare, prefixed.as_str()];
  // Per name, its open elements: `Some((start, start tag's end))` for one that matches `attrs`.
  let mut open: [Vec<Option<(usize, usize)>>; 2] = [Vec::new(), Vec::new()];
  let mut spans = Vec::new();
  let mut pos = 0;
  while let Some(rel) = xml[pos..].find('<') {
    let at = pos + rel;
    let Some((item, end)) = markup_item(xml, at) else {
      pos = at + 1;
      continue;
    };
    pos = end;
    match item {
      MarkupItem::Start { name, empty } => {
        let Some(which) = names.iter().position(|n| *n == name) else {
          continue;
        };
        let matches = attrs.iter().all(|a| start_tag_has(&xml[at..end], a));
        if empty {
          if matches {
            spans.push((at, end));
          }
        } else {
          open[which].push(matches.then_some((at, end)));
        }
      },
      MarkupItem::End { name } => {
        if let Some(which) = names.iter().position(|n| *n == name)
          && let Some(Some((start, _))) = open[which].pop()
        {
          spans.push((start, end));
        }
      },
      MarkupItem::Cdata | MarkupItem::Other => {},
    }
  }
  let mut unclosed: Vec<(usize, usize)> = open.into_iter().flatten().flatten().collect();
  if HTML_VOID_ELEMENTS.contains(&bare) {
    spans.append(&mut unclosed);
  }
  spans.sort_unstable();
  unclosed.sort_unstable();
  ElementScan { whole: spans, unclosed }
}

/// RelaxNG-validate a core-XML string against the repo's authoritative
/// `LaTeXML.rng` with `jing` (the S2 bar of the perfect-kernel program): the
/// number of jing error lines, `Some(0)` = schema-valid; `None` when `jing` is
/// not installed (CI installs it), logged as a skip of the check ([`skip`]); a guard
/// whose point is the schema declares [`JING`] instead.
/// The schema directory is prepared the way `tools/perfect_kernel/validate.sh`
/// does — the `.rng` files copied to a temp dir with the `urn:x-LaTeXML:RelaxNG:`
/// includes rewritten to relative paths — once for every process and checkout of
/// the same schema: the directory is named by a hash of the files' names and
/// contents.
pub fn rng_error_count(xml: &str) -> Option<usize> {
  use std::{path::PathBuf, process::Command, sync::OnceLock};
  static SCHEMA: OnceLock<Option<PathBuf>> = OnceLock::new();
  let dir = SCHEMA
    .get_or_init(|| {
      if !requirements::program_available("jing") {
        return None;
      }
      let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../latexml_core/resources/RelaxNG");
      // Shared by every test process (a per-process directory left ~15,000 in `/tmp`), named by a
      // hash of the schema sources so a changed `.rng` or model gets a directory of its own; written
      // aside and renamed into place, so a concurrent reader never sees a partial one.
      let mut sources: Vec<PathBuf> = [src.clone(), src.join("svg")]
        .iter()
        .filter_map(|dir| std::fs::read_dir(dir).ok())
        .flatten()
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|e| e == "rng"))
        .collect();
      sources.sort();
      let hash = sources.iter().fold(0xcbf2_9ce4_8422_2325u64, |hash, path| {
        // The path below the schema root (`svg/…`), so every checkout of the same schema shares it.
        let name = path
          .strip_prefix(&src)
          .unwrap_or(path)
          .to_string_lossy()
          .into_owned()
          .into_bytes();
        let bytes = std::fs::read(path).unwrap_or_default();
        name.into_iter().chain(bytes).fold(hash, |h, b| {
          (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3)
        })
      });
      let dir = std::env::temp_dir().join(format!("latexml-rng-schema-{hash:016x}"));
      if dir.join("LaTeXML.rng").exists() {
        return Some(dir);
      }
      let dir_partial = std::env::temp_dir().join(format!(
        "latexml-rng-schema-{hash:016x}.{}",
        std::process::id()
      ));
      let dir = {
        let target = dir;
        let dir = dir_partial;
        std::fs::create_dir_all(dir.join("svg")).ok()?;
        for (sub, from) in [("", src.clone()), ("svg", src.join("svg"))] {
          for entry in std::fs::read_dir(&from).ok()? {
            let path = entry.ok()?.path();
            if path.extension().is_some_and(|e| e == "rng") {
              let text = std::fs::read_to_string(&path).ok()?;
              let text = if sub.is_empty() {
                text
                  .replace("urn:x-LaTeXML:RelaxNG:svg:", "svg/")
                  .replace("urn:x-LaTeXML:RelaxNG:", "")
              } else {
                text
                  .replace("urn:x-LaTeXML:RelaxNG:svg:", "")
                  .replace("urn:x-LaTeXML:RelaxNG:", "")
              };
              std::fs::write(dir.join(sub).join(path.file_name()?), text).ok()?;
            }
          }
        }
        if std::fs::rename(&dir, &target).is_err() {
          let _ = std::fs::remove_dir_all(&dir);
        }
        target
      };
      Some(dir)
    })
    .as_ref();
  let Some(dir) = dir else {
    // The caller's other assertions still run; the skipped schema check is logged like a skipped test.
    let test = current_test_name();
    skip(
      &test,
      &format!("Missing program jing for {test} (its schema check)"),
    );
    return None;
  };
  let file = dir.join(format!(
    "doc-{}-{:?}.xml",
    std::process::id(),
    std::thread::current().id()
  ));
  std::fs::write(&file, xml).ok()?;
  let out = Command::new("jing")
    .arg(dir.join("LaTeXML.rng"))
    .arg(&file)
    .output()
    .ok()?;
  std::fs::remove_file(&file).ok();
  let text = format!(
    "{}{}",
    String::from_utf8_lossy(&out.stdout),
    String::from_utf8_lossy(&out.stderr)
  );
  Some(
    text
      .lines()
      .filter(|l| l.contains(": error:") || l.contains(": fatal:"))
      .count(),
  )
}

/// True iff a latex kernel dump built from this TeX tree is on disk where the format loads one from
/// (`LATEXML_DUMP_PATH`/`LATEXML_DUMP_DIR`, installed, dev tree). Without it the engine raw-loads `expl3-code.tex`
/// (degraded mode) and the error landscape is dominated by unrelated raw-load cascades — dump-gated tests declare
/// it ([`Requirements::format_dump`]) and SKIP visibly. The engine's own lookup, so a test cannot disagree with the
/// conversion it gates.
pub fn dump_available() -> bool { latexml_engine::dump_paths::format_dump_on_disk("latex") }

/// The path the engine's kpathsea lookup resolves `file` to in the host TeX tree, if any (the lookup the conversion
/// itself uses, so a test and the engine agree).
pub fn kpse_path(file: &str) -> Option<String> { latexml_core::util::pathname::kpsewhich(&[file]) }

/// True iff the engine's kpathsea lookup resolves the named file in the host TeX tree. For an expectation that
/// depends on the tree; a test that cannot run without the file declares it with [`check_requirements`] instead.
pub fn kpse_has(file: &str) -> bool { kpse_path(file).is_some() }

/// The text of the host TeX tree's copy of `file` (a class's version decides a guard's expectation), if any.
pub fn kpse_text(file: &str) -> Option<String> {
  kpse_path(file)
    .and_then(|path| std::fs::read(path).ok())
    .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
}

//======================================================================
// In-process conversion for guards (user directive 2026-10-03): the cargo-built library, never a spawned binary.
// Lifted from cluster_package_guards/perfect_kernel_batch46.rs so every test crate shares one copy.

/// The jobname of a `convert_with*` conversion: its source is `<GUARD_JOBNAME>.tex` in a temporary directory, so a side
/// file the document reads by `\jobname` is `t.bbl`, `t.aux`. A fixture names no file of its own `t`: the source is
/// a real file, among `\includegraphics{t}`'s candidates and what `\input{t}` reads.
pub const GUARD_JOBNAME: &str = "t";

/// The deadline of a `convert_with*` conversion, in seconds: the probe ceiling — a bound, so a runaway regression fails
/// as a `Fatal:Timeout` rather than hanging the test, with headroom for a heavy raw-preload guard on a loaded CI box
/// (the CLI's own default is 60 s).
pub const GUARD_TIMEOUT_SECS: u64 = 180;

/// The HTML page of `tex` through [`crate::api::convert_to_html_with`], without the bundled CSS/JS. With no
/// destination those files would otherwise be copied into the test's working directory, the crate root.
pub fn convert_to_html_page(tex: &str) -> Result<String, String> {
  crate::api::convert_to_html_with(
    tex,
    &crate::api::HtmlOptions::default().with_default_resources(false),
  )
}

/// Convert `tex` in-process (`convert_in_process`) — the package and contrib bindings, `preload` when given, comments
/// off, XML. Returns (log, XML).
pub fn convert_with(tex: &str, preload: Option<&str>) -> (String, String) {
  let (log, xml, ()) = convert_with_then(tex, preload, |_| ());
  (log, xml)
}

/// [`convert_with`], also running `inspect` on the XML in the conversion's thread (while its arena and state live).
pub fn convert_with_then<R: Send + 'static>(
  tex: &str,
  preload: Option<&str>,
  inspect: impl FnOnce(&str) -> R + Send + 'static,
) -> (String, String, R) {
  convert_with_setup_then(tex, preload, || {}, inspect)
}

/// [`convert_with_then`], also running `setup` in the conversion's thread before it starts: where a test sets a
/// thread-local override (never the process env, which parallel tests share).
pub fn convert_with_setup_then<R: Send + 'static>(
  tex: &str,
  preload: Option<&str>,
  setup: impl FnOnce() + Send + 'static,
  inspect: impl FnOnce(&str) -> R + Send + 'static,
) -> (String, String, R) {
  let run = convert_in_process(
    tex.as_bytes(),
    &[],
    preload,
    None,
    GUARD_TIMEOUT_SECS,
    setup,
    move |xml, _| inspect(xml),
  );
  (run.log, run.xml, run.inspected)
}

/// [`convert_with`], also returning the conversion's verdict: (log, XML, status line, status code) —
/// `ConversionResponse::status` ("4 errors; 4 undefined macros[…]", the CLI's "Conversion complete" summary) and
/// `status_code` (3 = fatal, 2 = error, lower = OK/warnings).
pub fn convert_with_status(tex: &str, preload: Option<&str>) -> (String, String, String, usize) {
  let run = convert_in_process(
    tex.as_bytes(),
    &[],
    preload,
    None,
    GUARD_TIMEOUT_SECS,
    || {},
    |_, _| (),
  );
  (run.log, run.xml, run.status, run.status_code)
}

/// [`convert_with`] after writing `files` (`(name, content)`, subdirectories created) beside the document — for a
/// guard needing a package, class, binding (`.rhai`) or data file next to its source.
pub fn convert_files_with(
  tex: &str,
  files: &[(&str, &str)],
  preload: Option<&str>,
) -> (String, String) {
  let files: Vec<(&str, &[u8])> = files
    .iter()
    .map(|(name, content)| (*name, content.as_bytes()))
    .collect();
  let run = convert_in_process(
    tex.as_bytes(),
    &files,
    preload,
    None,
    GUARD_TIMEOUT_SECS,
    || {},
    |_, _| (),
  );
  (run.log, run.xml)
}

/// [`convert_files_with`], also returning the conversion's verdict as [`convert_with_status`] does: (log, XML, status
/// line, status code).
pub fn convert_files_with_status(
  tex: &str,
  files: &[(&str, &str)],
  preload: Option<&str>,
) -> (String, String, String, usize) {
  let files: Vec<(&str, &[u8])> = files
    .iter()
    .map(|(name, content)| (*name, content.as_bytes()))
    .collect();
  let run = convert_in_process(
    tex.as_bytes(),
    &files,
    preload,
    None,
    GUARD_TIMEOUT_SECS,
    || {},
    |_, _| (),
  );
  (run.log, run.xml, run.status, run.status_code)
}

/// [`convert_files_with`] for sources that are not UTF-8 text — a legacy-encoded document or `.bib`, written byte for
/// byte — also running `inspect` on the XML and the source directory in the conversion's thread, while its state and
/// that directory live (the post stage's recursive `.bib` session digests in the document's own state, bib_session.rs,
/// and finds the `.bib` beside the source). Returns (log, XML, `inspect`'s result).
pub fn convert_bytes_files_then<R: Send + 'static>(
  tex: &[u8],
  files: &[(&str, &[u8])],
  preload: Option<&str>,
  inspect: impl FnOnce(&str, &std::path::Path) -> R + Send + 'static,
) -> (String, String, R) {
  let run = convert_in_process(
    tex,
    files,
    preload,
    None,
    GUARD_TIMEOUT_SECS,
    || {},
    inspect,
  );
  (run.log, run.xml, run.inspected)
}

/// [`convert_with_setup_then`] after writing `files` (`(name, content)`) beside the document: `setup` runs in the
/// conversion's thread before it (a thread-local override), `inspect` on the XML after it, in the same thread. Returns
/// (log, XML, `inspect`'s result).
pub fn convert_files_with_setup_then<R: Send + 'static>(
  tex: &str,
  files: &[(&str, &str)],
  preload: Option<&str>,
  setup: impl FnOnce() + Send + 'static,
  inspect: impl FnOnce(&str) -> R + Send + 'static,
) -> (String, String, R) {
  let files: Vec<(&str, &[u8])> = files
    .iter()
    .map(|(name, content)| (*name, content.as_bytes()))
    .collect();
  let run = convert_in_process(
    tex.as_bytes(),
    &files,
    preload,
    None,
    GUARD_TIMEOUT_SECS,
    setup,
    move |xml, _| inspect(xml),
  );
  (run.log, run.xml, run.inspected)
}

/// [`convert_files_with`] of a source dated `source_date_epoch` (Unix seconds, UTC) — what the CLI and cortex read
/// from a paper's bundle (`latexml::source_date`) — for a guard of the job's clock under ar5iv's archival preload.
pub fn convert_dated_files_with(
  tex: &str,
  files: &[(&str, &str)],
  preload: Option<&str>,
  source_date_epoch: i64,
) -> (String, String) {
  let files: Vec<(&str, &[u8])> = files
    .iter()
    .map(|(name, content)| (*name, content.as_bytes()))
    .collect();
  let run = convert_in_process(
    tex.as_bytes(),
    &files,
    preload,
    Some(source_date_epoch),
    GUARD_TIMEOUT_SECS,
    || {},
    |_, _| (),
  );
  (run.log, run.xml)
}

/// [`convert_with`] under a `secs` deadline instead of [`GUARD_TIMEOUT_SECS`] — for a guard of a heavy document.
pub fn convert_with_timeout(tex: &str, preload: Option<&str>, secs: u64) -> (String, String) {
  let run = convert_in_process(tex.as_bytes(), &[], preload, None, secs, || {}, |_, _| ());
  (run.log, run.xml)
}

/// What [`convert_in_process`] returns: the response's log, XML, verdict, and the `inspect` result.
struct InProcessRun<R> {
  log:         String,
  xml:         String,
  status:      String,
  status_code: usize,
  inspected:   R,
}

/// The conversion behind the `convert_with*` family — the CLI's production path, in-process: `tex` written as
/// `<GUARD_JOBNAME>.tex` with `files` beside it in a temporary directory, `prepare_session`, then [`Converter::convert`] on the
/// source's path (the digest, salvage and verdict arms the CLI and cortex run), on a fresh 256 MiB-stack thread so
/// each conversion starts from a clean thread-local engine. The thread carries a `timeout_secs` deadline (a runaway
/// regression fails as a `Fatal:Timeout`, not a hung test) and the harness's raised memory fuse ([`init_test_rss_cap`]'s
/// figure, thread-local: `cargo test` runs sibling conversions in one process, whose RSS they share; an explicit
/// `LATEXML_RSS_CAP_BYTES` still wins). `setup` runs after the fuse is set, so it may override it.
///
/// [`Converter::convert`]: crate::converter::Converter::convert
fn convert_in_process<R: Send + 'static>(
  tex: &[u8],
  files: &[(&str, &[u8])],
  preload: Option<&str>,
  source_date_epoch: Option<i64>,
  timeout_secs: u64,
  setup: impl FnOnce() + Send + 'static,
  inspect: impl FnOnce(&str, &std::path::Path) -> R + Send + 'static,
) -> InProcessRun<R> {
  let workdir = tempfile::tempdir().expect("create tempdir");
  for (name, content) in files {
    let path = workdir.path().join(name);
    if let Some(parent) = path.parent() {
      let _ = std::fs::create_dir_all(parent);
    }
    std::fs::write(path, content).expect("write file");
  }
  let source = workdir.path().join(format!("{GUARD_JOBNAME}.tex"));
  std::fs::write(&source, tex).expect("write the guard source");
  let source = source.to_string_lossy().into_owned();
  let source_dir = workdir.path().to_path_buf();
  let preload = preload.map(String::from);
  let fixtures: Vec<String> = files.iter().map(|(name, _)| (*name).to_string()).collect();
  std::thread::Builder::new()
    .stack_size(256 * 1024 * 1024)
    .spawn(move || {
      if std::env::var_os("LATEXML_RSS_CAP_BYTES").is_none() {
        let mib = latexml_core::watchdog::default_ceiling_mib().max(16_000);
        latexml_core::stomach::set_memory_cap(Some(mib.saturating_mul(1024 * 1024)));
      }
      setup();
      let _ = latexml_core::util::logger::init(log::LevelFilter::Info);
      let opts = latexml_core::common::Config {
        format: latexml_core::common::OutputFormat::XML,
        include_comments: Some(false),
        preload: preload.map(|p| vec![p]),
        bindings_dispatch: Some(std::rc::Rc::new(latexml_package::dispatch)),
        extra_bindings_dispatch: Some(std::rc::Rc::new(latexml_contrib::dispatch)),
        source_date_epoch,
        ..latexml_core::common::Config::default()
      };
      let mut converter = crate::converter::Converter::from_config(opts.clone());
      // As cortex_worker and the CLI do before preparing a session (plain TeX vs LaTeX, DVI class option).
      converter.note_main_source(&source);
      let resp = match converter.prepare_session(&opts) {
        Ok(()) => {
          latexml_core::stomach::set_timeout(timeout_secs);
          converter.convert(source)
        },
        Err(e) => crate::converter::ConversionResponse {
          result:       None,
          log:          format!("Error:prepare_session:{e}"),
          status:       String::from("Initialization failed."),
          status_code:  3,
          handoff_file: None,
        },
      };
      let xml = resp.result.unwrap_or_default();
      let inspected = inspect(&xml, &source_dir);
      latexml_core::stomach::set_timeout(0);
      latexml_core::reset_thread_engine();
      assert_fixtures_not_shadowed(&fixtures, &resp.log);
      InProcessRun {
        log: resp.log,
        xml,
        status: resp.status,
        status_code: resp.status_code,
        inspected,
      }
    })
    .expect("spawn test worker")
    .join()
    // the worker's own panic (an `inspect` assertion) as the test's, message intact
    .unwrap_or_else(|panic| std::panic::resume_unwind(panic))
}

/// Each of `sources` (paths of `.tex` files) converted in turn on ONE thread, as `cortex_worker` converts its papers: a
/// `Converter` and `prepare_session` per document, no engine reset between them (the [`convert_with`] options). Returns
/// (log, XML) per source; a guard compares a document's pair with the one it gets converted alone.
pub fn convert_on_one_thread(sources: Vec<String>, preload: Option<&str>) -> Vec<(String, String)> {
  let preload = preload.map(String::from);
  std::thread::Builder::new()
    .stack_size(256 * 1024 * 1024)
    .spawn(move || {
      let _ = latexml_core::util::logger::init(log::LevelFilter::Info);
      let opts = latexml_core::common::Config {
        format: latexml_core::common::OutputFormat::XML,
        include_comments: Some(false),
        preload: preload.map(|p| vec![p]),
        bindings_dispatch: Some(std::rc::Rc::new(latexml_package::dispatch)),
        extra_bindings_dispatch: Some(std::rc::Rc::new(latexml_contrib::dispatch)),
        ..latexml_core::common::Config::default()
      };
      let runs = sources
        .into_iter()
        .map(|source| {
          let mut converter = crate::converter::Converter::from_config(opts.clone());
          converter.note_main_source(&source);
          converter.prepare_session(&opts).expect("a session");
          latexml_core::stomach::set_timeout(GUARD_TIMEOUT_SECS);
          let resp = converter.convert(source);
          latexml_core::stomach::set_timeout(0);
          (resp.log, resp.result.unwrap_or_default())
        })
        .collect();
      latexml_core::reset_thread_engine();
      runs
    })
    .expect("spawn test worker")
    .join()
    .unwrap_or_else(|panic| std::panic::resume_unwind(panic))
}

/// Fails the guard when the conversion took a binding in place of one of its fixture packages or classes and never
/// read the fixture: Perl's FindFile_fallback (Package.pm:2186-2189 and the suffix lists after it;
/// `binding::content::find_file_fallback`) reads `myproof.sty`, `foo_v2.sty` or `bar-arxiv.cls` as a version of
/// `proof.sty`, `foo.sty`, `bar.cls`, and a binding of that name wins over the file beside the source
/// (`Info:fallback:<name> Interpreted as versioned package, falling back to …`), so the guard tests something else
/// (`perfect_kernel_batch67s::package_redefined_proof_keeps_the_binding` once did). A binding that follows the copy
/// it answers for reads the fixture (its path in the log: `(Processing definitions …/JINST_mod.cls`, `Loading
/// dependencies for …/icml2026.sty`), which is the fallback under test, not a shadowed fixture.
fn assert_fixtures_not_shadowed(fixtures: &[String], log: &str) {
  for fixture in fixtures {
    let base = fixture.rsplit('/').next().unwrap_or(fixture);
    let Some(stem) = base
      .strip_suffix(".sty")
      .or_else(|| base.strip_suffix(".cls"))
    else {
      continue;
    };
    let fell_back = log.lines().any(|line| {
      line
        .split("Info:fallback:")
        .nth(1)
        .and_then(|rest| rest.split_whitespace().next())
        .is_some_and(|name| name == stem || name == base)
    });
    let read = log.contains(&format!("/{base}"));
    assert!(
      !fell_back || read,
      "fixture {fixture} is shadowed by a binding through the versioned-name fallback (Package.pm \
       FindFile_fallback) and never read: rename it so the conversion reads it\n{log}"
    );
  }
}

/// Count of lines carrying a `Warning:<category>:` diagnostic anywhere in the line (WISDOM 85: a diagnostic can
/// follow other output on the same line), whatever its category spells. A package's own notice that it is obsolete
/// (aliascnt v1.7 on a 2026-06+ kernel, as pdflatex prints it) or replaced (sourceserifpro v2.0, 2025/04/05, a
/// compatibility stub that loads `sourceserif`: "This package has been replaced by `sourceserif'") follows the installed
/// TeX Live, not the document, so it is not counted: a guard's count then holds on every TeX Live (portable tests,
/// DG 2026-11-13; the sourceserifpro notice failed `frontmatter_ragslab_marks_link_affiliations` on Windows CI's
/// latest TeX Live, 2026-11-18).
pub fn warning_count(log: &str) -> usize {
  log
    .lines()
    .filter(|line| {
      !line.contains("Warning: The package is obsolete.")
        && !line.contains("Warning: This package has been replaced by `")
    })
    .filter(|line| {
      line.match_indices("Warning:").any(|(i, _)| {
        let tail = &line.as_bytes()[i + 8..];
        let n_class = tail
          .iter()
          .take_while(|b| **b != b':' && !b.is_ascii_whitespace())
          .count();
        n_class > 0 && tail.get(n_class) == Some(&b':')
      })
    })
    .count()
}

/// Convert a test fixture with the standard HTML5 config and return the full
/// response (result/log/status). The shared boilerplate for the standalone
/// regression tests.
pub fn convert_fixture(source: &str) -> crate::converter::ConversionResponse {
  init_test_rss_cap();
  let _ = latexml_core::util::logger::init(log::LevelFilter::Warn);
  let cfg = latexml_core::common::Config {
    format: latexml_core::common::OutputFormat::HTML5,
    ..latexml_core::common::Config::default()
  };
  let mut c = crate::converter::Converter::from_config(cfg);
  c.note_main_source(source);
  c.initialize_session().expect("initialize");
  c.convert(source.to_string())
}

#[cfg(test)]
mod exemption_audit {
  use std::path::{Path, PathBuf};

  use super::{ERROR_DEBT, INTENTIONALLY_FAILING};

  /// Recursively collect `(file_stem, path)` for every `.tex` fixture under `dir`.
  /// A missing/unreadable `dir` yields nothing (not an error): the caller
  /// interprets "no fixtures" as "no corpus to audit".
  fn collect_tex_stems(dir: &Path, out: &mut Vec<(String, PathBuf)>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
      return;
    };
    for ent in rd.flatten() {
      let p = ent.path();
      if p.is_dir() {
        collect_tex_stems(&p, out);
      } else if p.extension().and_then(|e| e.to_str()) == Some("tex")
        && let Some(stem) = p.file_stem().and_then(|s| s.to_str())
      {
        out.push((stem.to_string(), p.clone()));
      }
    }
  }

  /// Return every exemption key that matches MORE THAN ONE of the collected
  /// `(file_stem, path)` fixtures — the collision the audit exists to catch.
  /// Pure (no I/O), so it can be exercised on a synthetic in-memory corpus.
  fn find_stem_collisions(all: &[(String, PathBuf)]) -> Vec<(String, Vec<PathBuf>)> {
    let keys = INTENTIONALLY_FAILING
      .iter()
      .map(|(n, ..)| *n)
      .chain(ERROR_DEBT.iter().map(|(n, _)| *n));
    let mut collisions = Vec::new();
    for key in keys {
      let hits: Vec<PathBuf> = all
        .iter()
        .filter(|(s, _)| s == key)
        .map(|(_, p)| p.clone())
        .collect();
      if hits.len() > 1 {
        collisions.push((key.to_string(), hits));
      }
    }
    collisions
  }

  /// Audit the exemption tables against the `.tex` corpus rooted at `tests_dir`,
  /// returning every exemption key that matches MORE THAN ONE fixture.
  ///
  /// Returns `None` when `tests_dir` holds no corpus at all — kept distinct from
  /// `Some(vec![])` (corpus present, no collisions). The published `latexml`
  /// crate EXCLUDES `tests/` (Cargo.toml `exclude`, to fit crates.io's 10 MiB
  /// cap), so a downstream `cargo test` on the packaged crate legitimately has
  /// nothing to audit: that is a skip, not a failure (issue #301).
  fn exemption_stem_collisions(tests_dir: &Path) -> Option<Vec<(String, Vec<PathBuf>)>> {
    let mut all = Vec::new();
    collect_tex_stems(tests_dir, &mut all);
    if all.is_empty() {
      return None;
    }
    Some(find_stem_collisions(&all))
  }

  /// Review m1: the exemption tables match on the bare `file_stem`, which is
  /// NOT unique across the suite's globbed test dirs. A future `glossary.tex`
  /// (etc.) under a different directory would silently inherit the
  /// ERROR_DEBT / INTENTIONALLY_FAILING exemption — an ERROR_DEBT collision
  /// could then MASK a real regression. Guard the invariant the match relies
  /// on: every exemption key resolves to AT MOST ONE `.tex` across `tests/`.
  /// (Cheaper and lower-churn than dir-qualifying the keys; if this ever fails,
  /// rename the fixture or dir-qualify that entry.)
  #[test]
  fn exemption_keys_have_unique_stems() {
    // Locate the corpus via CARGO_MANIFEST_DIR, NOT a CWD-relative path: the scan
    // must not depend on the test binary's working directory, and the packaged
    // crate (which excludes `tests/`) must skip cleanly instead of tripping a
    // "wrong CWD" assert (issue #301).
    let tests_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let Some(collisions) = exemption_stem_collisions(&tests_dir) else {
      // No corpus on disk — packaged crate. Nothing to audit.
      return;
    };
    assert!(
      collisions.is_empty(),
      "exemption keys match multiple .tex fixtures {collisions:?} — the bare-stem \
       match would apply the exemption to ALL of them, potentially masking a \
       regression. Dir-qualify the entry or rename the fixture."
    );
  }

  /// Regression guard for issue #301: the published crate EXCLUDES `tests/`, so
  /// the audit must treat a missing corpus as "nothing to check" (`None`) and the
  /// caller must skip — never panic. Before the fix the audit asserted on the
  /// empty scan, blowing up a downstream `cargo test` on the packaged crate.
  #[test]
  fn audit_skips_when_corpus_absent() {
    let absent = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests-this-dir-does-not-exist");
    assert!(
      exemption_stem_collisions(&absent).is_none(),
      "an absent corpus must yield None (skip), not a spurious audit result"
    );
  }

  /// The audit still FIRES on a real duplicate-stem collision, exercised on an
  /// IN-MEMORY corpus so the check is deterministic, independent of the live
  /// exemption tables staying collision-free, and needs no filesystem at all —
  /// not even a writable temp dir, so it survives the same hostile
  /// packaged/sandboxed environments as the rest of the suite (issue #301). Two
  /// fixtures under different dirs share a real exemption key; the bare-stem
  /// match must flag it.
  #[test]
  fn audit_detects_duplicate_stems() {
    let key = INTENTIONALLY_FAILING
      .iter()
      .map(|(n, ..)| *n)
      .chain(ERROR_DEBT.iter().map(|(n, _)| *n))
      .next()
      .expect("at least one exemption key");
    let all = vec![
      (
        key.to_string(),
        PathBuf::from(format!("tests/here/{key}.tex")),
      ),
      (
        key.to_string(),
        PathBuf::from(format!("tests/nested/{key}.tex")),
      ),
    ];
    let collisions = find_stem_collisions(&all);
    assert!(
      collisions
        .iter()
        .any(|(k, hits)| k == key && hits.len() == 2),
      "audit should flag the duplicate {key:?}: {collisions:?}"
    );
  }
}

#[cfg(test)]
mod element_helper_tests {
  use super::{
    assert_all_elements, assert_element, assert_no_element, element_count, markup_text,
    normalize_markup, xml_element, xml_elements,
  };

  const XML: &str = "<document>\n  <bibliography>\n    <biblist>\n      <bibitem key=\"a\" xml:id=\"bib.a\">\n        <bibtag role=\"refnum\">1</bibtag>\n        <bibblock>Author <emph>T</emph></bibblock>\n      </bibitem>\n      <bibitem key=\"b\"><bibtag>2</bibtag></bibitem>\n    </biblist>\n  </bibliography>\n  <graphics graphic=\"x.pdf\"/>\n</document>\n";

  #[test]
  fn first_element_is_returned_whole_and_normalized() {
    assert_eq!(
      xml_element(XML, "bibitem", &[]).as_deref(),
      Some(
        "<bibitem key=\"a\" xml:id=\"bib.a\"><bibtag role=\"refnum\">1</bibtag><bibblock>Author <emph>T</emph></bibblock></bibitem>"
      )
    );
  }

  #[test]
  fn attribute_fragments_select_a_later_element() {
    assert_eq!(
      xml_element(XML, "ltx:bibitem", &["key=\"b\""]).as_deref(),
      Some("<bibitem key=\"b\"><bibtag>2</bibtag></bibitem>")
    );
    assert_eq!(xml_element(XML, "bibitem", &["key=\"zz\""]), None);
  }

  #[test]
  fn nested_same_tag_and_self_closing_are_handled() {
    let nested = "<g id=\"o\"><g id=\"i\"><path/></g><text>L</text></g>";
    assert_eq!(xml_element(nested, "g", &[]).as_deref(), Some(nested));
    assert_eq!(
      xml_element(nested, "g", &["id=\"i\""]).as_deref(),
      Some("<g id=\"i\"><path/></g>")
    );
    assert_eq!(
      xml_element(XML, "graphics", &[]).as_deref(),
      Some("<graphics graphic=\"x.pdf\"/>")
    );
    // `<graphicsX>` is not `<graphics>`.
    assert_eq!(xml_element("<graphicsX/>", "graphics", &[]), None);
    // A `>` inside an attribute value does not end the start tag.
    assert_eq!(
      xml_element("<p><Math tex=\"a>b\"><XMath/></Math></p>", "Math", &[]).as_deref(),
      Some("<Math tex=\"a>b\"><XMath/></Math>")
    );
  }

  #[test]
  fn normalization_keeps_text_whitespace() {
    assert_eq!(
      normalize_markup("<p>\n  a <b> c </b>\n</p>\n"),
      "<p>a <b> c </b></p>"
    );
    assert_element(
      XML,
      "bibitem",
      &["key=\"b\""],
      "<bibitem key=\"b\">\n  <bibtag>2</bibtag>\n</bibitem>",
    );
  }

  #[test]
  fn markup_inside_a_comment_cdata_or_pi_is_no_element() {
    let xml = "<doc><!-- <note>old</note> --><note>a<!-- </note> -->b</note><![CDATA[<note>c</note>]]><?pi <note>p</note>?></doc>";
    assert_eq!(xml_elements(xml, "note", &[]), [
      "<note>a<!-- </note> -->b</note>"
    ]);
    assert_eq!(element_count(xml, "note", &[]), 1);
  }

  #[test]
  fn an_attribute_fragment_sits_at_an_attribute_boundary() {
    let xml = "<doc><p xml:id=\"p1\">a</p><p id=\"p1\" class=\"ltx_foo ltx_bar\">b</p></doc>";
    assert_eq!(xml_elements(xml, "p", &["id=\"p1\""]), [
      "<p class=\"ltx_foo ltx_bar\" id=\"p1\">b</p>"
    ]);
    assert_eq!(xml_elements(xml, "p", &["xml:id=\"p1\""]), [
      "<p xml:id=\"p1\">a</p>"
    ]);
    // Other fragments keep the substring contract: a class among several, a value's prefix.
    assert_eq!(element_count(xml, "p", &["ltx_bar"]), 1);
    assert_eq!(element_count(xml, "p", &["class=\"ltx_foo"]), 1);
  }

  #[test]
  fn every_element_is_returned_attribute_less_and_nested_ones_too() {
    let xml = "<doc><g><g id=\"i\"><path/></g><text>L</text></g><gx/><g/></doc>";
    assert_eq!(xml_elements(xml, "ltx:g", &[]), [
      "<g><g id=\"i\"><path/></g><text>L</text></g>",
      "<g id=\"i\"><path/></g>",
      "<g/>"
    ]);
    assert_eq!(element_count(xml, "g", &["id=\"i\""]), 1);
    assert_all_elements(xml, "g", &["id=\"i\""], &["<g id=\"i\">\n  <path/>\n</g>"]);
    assert_no_element(xml, "g", &["id=\"zz\""]);
    assert_no_element(xml, "graphics", &[]);
  }

  #[test]
  #[should_panic(expected = "<g id=\"i\"><path/></g>")]
  fn assert_no_element_names_the_element_it_found() {
    assert_no_element("<g id=\"i\"><path/></g>", "g", &[]);
  }

  #[test]
  #[should_panic(expected = "unclosed <note class=\"x\">")]
  fn assert_no_element_fails_on_a_start_tag_left_open() {
    // Truncated markup: no whole element, but the start tag is there.
    assert_eq!(
      xml_elements("<p><note class=\"x\">abc", "note", &[]),
      [""; 0]
    );
    assert_no_element("<p><note class=\"x\">abc", "note", &[]);
  }

  #[test]
  fn an_html_void_element_is_its_start_tag() {
    let html = "<p>a<br>b<br class=\"x\">c<img src=\"i.png\" alt=\"\"></p>";
    assert_eq!(xml_elements(html, "br", &[]), ["<br>", "<br class=\"x\">"]);
    assert_eq!(xml_elements(html, "img", &[]), [
      "<img alt=\"\" src=\"i.png\">"
    ]);
    assert_eq!(xml_elements(html, "p", &[]), [normalize_markup(html)]);
    // In XML a void-named element may have content (epub.rs writes `<meta …>…</meta>`).
    assert_eq!(
      xml_elements("<p><meta a=\"1\">v</meta></p>", "meta", &[]),
      ["<meta a=\"1\">v</meta>"]
    );
  }

  #[test]
  fn markup_text_reads_through_split_spans() {
    assert_eq!(markup_text("2<text>021</text>"), "2021");
    assert_eq!(
      markup_text(
        "<p>a &amp; b&#160;c&#x41;<!-- x --><?pi y?>&lt;d&gt;</p>\n  <p>e<![CDATA[ & f]]></p>"
      ),
      "a & b\u{a0}cA<d> e & f"
    );
    assert_eq!(markup_text("<!DOCTYPE d [<!ENTITY a \"b\">]><d>x</d>"), "x");
  }

  #[test]
  fn markup_text_reads_a_break_as_one_space() {
    assert_eq!(markup_text("Univ A<break/>Town"), "Univ A Town");
    assert_eq!(markup_text("<p><break/>a<break/></p>"), "a");
    assert_eq!(
      markup_text("Univ A <ltx:break/> Town<br>X"),
      "Univ A Town X"
    );
  }

  #[test]
  fn the_shadow_directory_is_absolute_and_nonempty() {
    use super::shadow_dir;
    assert_eq!(shadow_dir(None), None);
    assert_eq!(shadow_dir(Some("".into())), None);
    assert_eq!(
      shadow_dir(Some("/tmp/shadow".into())),
      Some("/tmp/shadow".into())
    );
    let relative = std::panic::catch_unwind(|| shadow_dir(Some("shadow".into())));
    assert!(relative.is_err(), "a relative directory must not be used");
  }

  #[test]
  #[should_panic(expected = "LATEXML_TEST_SHADOW: cannot write")]
  fn the_shadow_hook_fails_when_it_cannot_write() {
    let file = tempfile::NamedTempFile::new().unwrap();
    // A regular file where the directory should be.
    super::shadow_compare(
      file.path(),
      std::panic::Location::caller(),
      "<!-- <p>x</p> --><p>y</p>",
      "p",
      &[],
      Some("<p>x</p>"),
    );
  }

  #[test]
  fn the_shadow_hook_logs_a_disagreement_as_one_row() {
    let dir = tempfile::tempdir().unwrap();
    let here = std::panic::Location::caller();
    let xml = "<!-- <p>x</p> --><p>y</p>";
    let old = super::first_element_by_substring(xml, "p", &[]);
    super::shadow_compare(dir.path(), here, xml, "p", &[], old.as_deref());
    let log = dir.path().join(format!("{}.tsv", std::process::id()));
    assert_eq!(
      std::fs::read_to_string(log).unwrap(),
      format!("{}:{}\tp\t\t<p>x</p>\t<p>y</p>\n", here.file(), here.line())
    );
  }
}
