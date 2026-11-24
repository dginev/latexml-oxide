//! `converter::forbid_runtime_bindings`, the switch `cortex_worker` flips as its first statement, through the library:
//! CI does not build the `cortex` feature, so `cortex_worker_untrusted_bindings` runs only where it is enabled, and
//! this binary keeps the switch itself under CI. It is its own binary because the switch is one-way and process-wide.
//! The same source converts twice: before the switch its `probe.sty.rhai` loads (so the probe is live), after it the
//! binding is not found, as in a build without `runtime-bindings`.
#![cfg(feature = "runtime-bindings")]

use latexml::converter::Converter;
use latexml_core::common::{Config, OutputFormat};

const MAIN_TEX: &str = "\\documentclass{article}\n\\usepackage{probe}\n\\begin{document}\nText \\probe.\n\\end{document}\n";
const PROBE_RHAI: &str = "DefMacro(\"\\\\probe\", \"PROBE-LOADED\");\n";

/// The log and the XML of converting `source`, on a fresh thread (the State is per thread).
fn convert(source: &std::path::Path) -> (String, String) {
  let source = source.display().to_string();
  std::thread::Builder::new()
    .stack_size(64 * 1024 * 1024)
    .spawn(move || {
      let _ = latexml_core::util::logger::init(log::LevelFilter::Warn);
      let cfg = Config {
        format: OutputFormat::XML,
        extra_bindings_dispatch: Some(std::rc::Rc::new(latexml_contrib::dispatch)),
        ..Config::default()
      };
      let mut converter = Converter::from_config(cfg);
      converter.initialize_session().expect("initialize");
      let response = converter.convert(source);
      (response.log, response.result.unwrap_or_default())
    })
    .unwrap()
    .join()
    .unwrap()
}

#[test]
fn a_forbidden_runtime_binding_beside_the_source_is_not_loaded() {
  let dir = tempfile::tempdir().unwrap();
  let main = dir.path().join("main.tex");
  std::fs::write(&main, MAIN_TEX).unwrap();
  std::fs::write(dir.path().join("probe.sty.rhai"), PROBE_RHAI).unwrap();

  let (log, xml) = convert(&main);
  assert!(
    xml.contains("PROBE-LOADED"),
    "before the switch the runtime binding loads:\n{log}\n{xml}"
  );

  latexml::converter::forbid_runtime_bindings();
  let (log, xml) = convert(&main);
  assert!(
    !xml.contains("PROBE-LOADED"),
    "after the switch the runtime binding still loaded:\n{xml}"
  );
  assert!(
    log.contains("Warning:missing_file:probe") && log.contains("Error:undefined:\\probe"),
    "after the switch `probe` resolves as in a build without runtime bindings:\n{log}"
  );
}
