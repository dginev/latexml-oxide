use glob::glob;
use libxml::tree::Node;
use once_cell::sync::Lazy;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Once;

use crate::core_interface::DigestionAPI;
use latexml_codegen::LoadModel;
use latexml_core::common::BindingDispatcher;
use latexml_core::document::Document;
use latexml_core::{Core, CoreOptions, s, state};
use latexml_math_parser::node_to_grammar_lexemes;

// Process-once cached env vars (see WISDOM #56 — getenv hot-path race).
// Sampled at static init; subsequent reads are atomic loads.
static TEST_LOG: Lazy<bool> = Lazy::new(|| std::env::var("LATEXML_TEST_LOG").is_ok());
static SIGSEGV_TRACE: Lazy<bool> = Lazy::new(|| std::env::var("LATEXML_SIGSEGV_TRACE").is_ok());
static SAVE_ACTUAL: Lazy<bool> = Lazy::new(|| std::env::var("LATEXML_SAVE_ACTUAL").is_ok());

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
#[cfg_attr(target_os = "linux", link_section = ".init_array")]
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
  extern "C" {
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
    // `/tmp/latexml_sigsegv_<pid>.txt` survives the kill.
    let tid = std::thread::current().id();
    let name = std::thread::current()
      .name()
      .unwrap_or("<unnamed>")
      .to_string();
    let pid = std::process::id();
    let path = format!("/tmp/latexml_sigsegv_{pid}.txt");
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
    unsafe {
      let raw_dfl: extern "C" fn(i32) = std::mem::transmute(SIG_DFL);
      signal(sig, raw_dfl);
      raise(sig);
    }
  }

  unsafe {
    signal(SIGSEGV, handler);
    signal(SIGBUS, handler);
    signal(SIGABRT, handler);
  }
}

/// Tests whose TeX input is *known* to produce Error/Warn messages in both
/// the Perl reference implementation and the Rust port.  We suppress log
/// output for these so that `cargo test` runs cleanly, while still counting
/// errors internally (MAX_ERRORS check still fires).
/// Tests where Perl LaTeXML also produces Error/Warn messages.
/// ONLY add tests here if verified that Perl `bin/latexml` emits errors on the same input.
const KNOWN_ERROR_TESTS: &[&str] = &[
  "io",                   // Perl: 2 errors (mode-switch egroup from \readnext)
  "figure_mixed_content", // Perl: 1 error (ltx:theorem not allowed in ltx:figure)
];

pub fn latexml_test_single(
  tex_file_str: &str,
  name: &str,
  dirpath: &str,
  requires: Option<&phf::Map<&str, &str>>,
  dispatcher_opt: Option<BindingDispatcher>,
) {
  init_logger();
  if !validate_requirements(dirpath, requires) {
    return; // test group only if required files are found.
  }
  let suppress = KNOWN_ERROR_TESTS.contains(&name);
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
  // Add the package bindings
  state::set_bindings_dispatch(Rc::new(latexml_package::dispatch));
  state::add_binding_names(latexml_package::binding_names());
  // If we want to test the latexml_contrib bindings, we need to pass in the additional binding
  // dispatcher, which makes the contrib bindings visible
  if let Some(dispatcher) = extra_bindings_dispatcher {
    state::set_extra_bindings_dispatch(dispatcher);
  }
  match latexml.convert_file(tex_path.to_owned()) {
    Err(e) => panic!("{:?}: Couldn't convert {:?}; {:?}", name, tex_path, e),
    Ok(doc) => process_ltx_doc(doc, name),
  }
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
    let path = format!("/tmp/latexml_actual_{name}.xml");
    std::fs::write(&path, &doc_str).ok();
    eprintln!("Saved actual XML to {path}");
    // Also save using libxml's built-in serializer for comparison
    let path2 = format!("/tmp/latexml_actual_{name}_libxml.xml");
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

/// Provide a default test `Core` engine for simple operations
pub fn new_test_engine() -> Core {
  let core_engine = Core::new(CoreOptions {
    preload: Some(
      ["article.cls", "amsmath.sty"]
        .map(|x| x.to_string())
        .to_vec(),
    ),
    verbosity: Some(-2),
    search_paths: None,
    nomathparse: Some(true),
    include_comments: Some(false),
    ..CoreOptions::default()
  });
  load_model!("LaTeXML");
  state::set_bindings_dispatch(Rc::new(latexml_package::dispatch));
  state::add_binding_names(latexml_package::binding_names());
  core_engine
}

/// Simple tokenization of a single formula, without any custom preloads
/// beyond latex and amsmath
pub fn lex_single_tex_formula(
  tex: &str,
  latexml: &mut Core,
) -> (Vec<String>, Vec<Node>, Option<Node>, Document) {
  let xml_result = latexml.convert_file(format!("literal:\\[ {tex} \\]"));
  assert!(xml_result.is_ok(), "{:?}", xml_result.err());
  let mut doc = xml_result.unwrap();

  // grab the first formula
  match doc.findnode("//*[local-name()='XMath']", None) {
    Some(math) => {
      let mut idx = 0;
      let (lexemes, nodes) = node_to_grammar_lexemes(&math, &mut idx);
      (lexemes, nodes, Some(math), doc)
    },
    None => (Vec::new(), Vec::new(), None, doc),
  }
}

/// Build a test function for each "*.tex" source found in a given directory path.
/// The path should be absolute, or relative to the root latexml-oxide checkout.
#[macro_export]
macro_rules! tex_tests {
  ($dir:literal) => {
    tex_tests!($dir, None, None);
  };
  ($dir:literal, $requires:expr, $dispatch:expr) => {
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
