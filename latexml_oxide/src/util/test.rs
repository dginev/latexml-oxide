use std::{path::PathBuf, sync::Once};

use glob::glob;
use latexml_core::{Core, CoreOptions, common::BindingDispatcher, document::Document, s};
use once_cell::sync::Lazy;

use crate::core_interface::DigestionAPI;

// Process-once cached env vars (see WISDOM #56 — getenv hot-path race).
// Sampled at static init; subsequent reads are atomic loads.
static TEST_LOG: Lazy<bool> = Lazy::new(|| std::env::var("LATEXML_TEST_LOG").is_ok());
static SIGSEGV_TRACE: Lazy<bool> = Lazy::new(|| std::env::var("LATEXML_SIGSEGV_TRACE").is_ok());
static SAVE_ACTUAL: Lazy<bool> = Lazy::new(|| std::env::var("LATEXML_SAVE_ACTUAL").is_ok());
/// "Bless" / regenerate mode — the Rust equivalent of Perl's `tools/maketests`.
/// When `LATEXML_BLESS=1`, a `…_ok` test writes the ACTUAL conversion output to
/// its golden `.xml` (overwriting it) instead of comparing+asserting. Run via
/// `tools/maketests.sh` (optionally with a test-name filter). Because it reuses
/// the exact harness conversion + serialization (`process_texfile`), the
/// regenerated golden is byte-identical to what the comparison expects.
static BLESS: Lazy<bool> = Lazy::new(|| std::env::var("LATEXML_BLESS").is_ok());

pub fn latexml_tests(
  dirpath: &str,
  requires: Option<&phf::Map<&str, &str>>,
  dispatcher_opt: Option<BindingDispatcher>,
) {
  latexml_tests_internal(dirpath, requires, dispatcher_opt)
}
pub fn latexml_tests_internal(
  dirpath: &str,
  requires: Option<&phf::Map<&str, &str>>,
  dispatcher_opt: Option<BindingDispatcher>,
) {
  if !validate_requirements(dirpath, requires) {
    return; // test group only if required files are found.
  }
  for tex_file in glob(&s!("{}/*.tex", dirpath)).unwrap().flatten() {
    let name = tex_file.file_stem().unwrap().to_str().unwrap();
    let xml_file = tex_file.with_extension("xml");

    let tex_file_string = tex_file.to_str().unwrap();
    let xml_file_str = xml_file.to_str().unwrap();
    if xml_file.exists() {
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
fn init_test_rss_cap() {
  INIT_TEST_RSS_CAP.call_once(|| {
    if std::env::var_os("LATEXML_RSS_CAP_BYTES").is_none() {
      // SAFETY: set exactly once under `Once`, at the very top of every
      // generated test (before any conversion thread reads the env in
      // `stomach::check_timeout`). `Once`'s release/acquire ordering
      // happens-before all those reads, so there is no setenv/getenv race.
      unsafe {
        std::env::set_var("LATEXML_RSS_CAP_BYTES", "9000000000");
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
/// entry's `[error-debt] … 0 errors` shows up EVERYWHERE, remove it by review.
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

/// Emit a line to the process's REAL stderr, SURVIVING libtest's per-test
/// output capture. libtest only intercepts the `print!`/`eprint!` macros and
/// replays them solely on FAILURE, so a plain `eprintln!` from a PASSING test
/// is swallowed — which defeats the `[error-debt] … review for removal` and
/// `[intentional-fail]` notices, whose entire purpose is to be SEEN on a green
/// run (review m2). A direct `write(2)` bypasses the capture. One syscall per
/// line is atomic up to PIPE_BUF, so concurrent test threads don't interleave.
#[cfg(unix)]
fn note_uncaptured(line: &str) {
  use std::{io::Write, os::unix::io::FromRawFd};
  // SAFETY: fd 2 is the process stderr, valid for the whole run. `ManuallyDrop`
  // stops the `File`'s Drop from `close()`-ing the shared descriptor.
  let mut f = std::mem::ManuallyDrop::new(unsafe { std::fs::File::from_raw_fd(2) });
  let _ = f.write_all(format!("{line}\n").as_bytes());
}
#[cfg(not(unix))]
fn note_uncaptured(line: &str) {
  eprintln!("{line}");
}

pub fn latexml_test_single(
  tex_file_str: &str,
  name: &str,
  dirpath: &str,
  requires: Option<&phf::Map<&str, &str>>,
  dispatcher_opt: Option<BindingDispatcher>,
) {
  init_logger();
  init_test_rss_cap();
  if !validate_requirements(dirpath, requires) {
    return; // test group only if required files are found.
  }
  // Platform-skipped golden fixtures: kept live (and compared) on the
  // platforms whose TeX distribution matches the committed golden, but
  // skipped where an UNPINNABLE upstream package version differs. This is
  // a Linux↔Windows portability difference, not a code divergence — the
  // engine faithfully renders whatever the package emits. See SYNC_STATUS.md.
  #[cfg(windows)]
  {
    // circuitikz ≥ 1.8.0 lengthens drawn capacitor plates (12.4 → 12.68 in
    // our SVG space). The Windows TeX (setup-texlive net-install / a fresh
    // `install-tl`) ships the NEWEST circuitikz; Linux/macOS apt/brew ship
    // an older one that matches the golden. circuitikz can't be version-
    // pinned in the fixture (Perl/Rust both version-strip the request), so
    // skip on Windows only; Linux + macOS still run and compare it.
    const WINDOWS_GOLDEN_SKIP: &[&str] = &["ac-drive-components"];
    if WINDOWS_GOLDEN_SKIP.contains(&name) {
      eprintln!(
        "SKIP (Windows): {name} — circuitikz-version-nondeterministic golden; \
         kept live on Linux/macOS. See docs/SYNC_STATUS.md."
      );
      return;
    }
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
  if matches!(xml_file.try_exists(), Ok(true)) {
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

fn validate_requirements(_dirpath: &str, _requires: Option<&phf::Map<&str, &str>>) -> bool {
  // TODO
  true
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
      note_uncaptured(&format!(
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
        note_uncaptured(&format!(
          "[error-debt] {name}: 0 errors HERE — clean in this \
          environment; review for removal once clean everywhere — {reason}"
        ));
      } else {
        note_uncaptured(&format!("[error-debt] {name}: {n_err} errors — {reason}"));
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
           docs/reproducers/MALFORMED_CLOSE_NUMBERED_2026-06-10.md.",
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
// dependency on `glob`/`phf` and need to be callable from the
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

/// Count inline `Error:<class>:` markers (parity_check.sh's lax pattern, see
/// feedback_strict_vs_lax_error_grep.md). Errors are emitted INLINE within
/// `(Building...Error:..)` envelopes, not at line starts.
pub fn error_count(log: &str) -> usize {
  log
    .match_indices("Error:")
    .filter(|(i, _)| {
      let tail = &log.as_bytes()[*i + 6..];
      let n_class = tail.iter().take_while(|b| b.is_ascii_lowercase()).count();
      n_class > 0 && tail.get(n_class) == Some(&b':')
    })
    .count()
}

/// True iff a year-versioned latex kernel dump is present in the dev tree.
/// Without it the engine raw-loads `expl3-code.tex` (degraded mode) and the
/// error landscape is dominated by unrelated raw-load cascades — dump-gated
/// tests should SKIP rather than measure the wrong thing. Delegates to the
/// engine's own dump-name convention so a filename-scheme change cannot make
/// the tests silently self-skip forever.
pub fn dump_available() -> bool {
  let dir = std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../resources/dumps"));
  !latexml_engine::dump_paths::available_years_in_dir(dir, "latex").is_empty()
}

/// True iff `kpsewhich` resolves the named file in the host TeX tree.
pub fn kpse_has(file: &str) -> bool {
  std::process::Command::new("kpsewhich")
    .arg(file)
    .output()
    .map(|o| o.status.success() && !o.stdout.is_empty())
    .unwrap_or(false)
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
  c.initialize_session().expect("initialize");
  c.convert(source.to_string())
}

#[cfg(test)]
mod exemption_audit {
  use std::path::{Path, PathBuf};

  use super::{ERROR_DEBT, INTENTIONALLY_FAILING};

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
    fn collect(dir: &Path, out: &mut Vec<(String, PathBuf)>) {
      let Ok(rd) = std::fs::read_dir(dir) else {
        return;
      };
      for ent in rd.flatten() {
        let p = ent.path();
        if p.is_dir() {
          collect(&p, out);
        } else if p.extension().and_then(|e| e.to_str()) == Some("tex")
          && let Some(stem) = p.file_stem().and_then(|s| s.to_str())
        {
          out.push((stem.to_string(), p.clone()));
        }
      }
    }
    let mut all = Vec::new();
    collect(Path::new("tests"), &mut all);
    assert!(
      !all.is_empty(),
      "no .tex fixtures found under tests/ — wrong CWD?"
    );

    let keys = INTENTIONALLY_FAILING
      .iter()
      .map(|(n, ..)| *n)
      .chain(ERROR_DEBT.iter().map(|(n, _)| *n));
    for key in keys {
      let hits: Vec<&PathBuf> = all
        .iter()
        .filter(|(s, _)| s == key)
        .map(|(_, p)| p)
        .collect();
      assert!(
        hits.len() <= 1,
        "exemption key {key:?} matches {} .tex fixtures {:?} — the bare-stem \
         match would apply the exemption to ALL of them, potentially masking a \
         regression. Dir-qualify the entry or rename the fixture.",
        hits.len(),
        hits
      );
    }
  }
}
