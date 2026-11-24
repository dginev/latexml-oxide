//! `cortex_worker` converts untrusted arXiv sources, so it never loads a runtime `.rhai` binding, in any build: a
//! `<pkg>.sty.rhai` shipped inside a paper would otherwise auto-load (`converter.rs` `rhai_dispatch`) and could run any
//! program through the binding API's `Command(..).output()` (docs/release/SAFETY.md). The same source through the
//! `latexml_oxide` CLI, the user's own machine, still loads it.

use std::{
  fs,
  io::{Read, Write},
  path::Path,
  process::Command,
};

const MAIN_TEX: &str = "\\documentclass{article}\n\\usepackage{probe}\n\\begin{document}\nText \\probe.\n\\end{document}\n";
const PROBE_RHAI: &str =
  "let r = Command(\"echo\").output();\nDefMacro(\"\\\\probe\", \"PROBE-LOADED\");\n";

/// `main.tex` and `probe.sty.rhai`, zipped as an arXiv source bundle at `zip_path`.
fn write_source_zip(zip_path: &Path) {
  let mut zip = zip::ZipWriter::new(fs::File::create(zip_path).unwrap());
  let options = zip::write::SimpleFileOptions::default();
  for (name, body) in [("main.tex", MAIN_TEX), ("probe.sty.rhai", PROBE_RHAI)] {
    zip.start_file(name, options).unwrap();
    zip.write_all(body.as_bytes()).unwrap();
  }
  zip.finish().unwrap();
}

/// The text of `name` inside the result zip at `zip_path`.
fn zip_member(zip_path: &Path, name: &str) -> String {
  let mut archive = zip::ZipArchive::new(fs::File::open(zip_path).unwrap()).unwrap();
  let mut text = String::new();
  archive
    .by_name(name)
    .unwrap()
    .read_to_string(&mut text)
    .unwrap();
  text
}

#[test]
fn cortex_worker_never_loads_a_runtime_binding_shipped_in_the_source() {
  let dir = tempfile::tempdir().unwrap();
  let input = dir.path().join("paper.zip");
  let output = dir.path().join("result.zip");
  write_source_zip(&input);
  let run = Command::new(env!("CARGO_BIN_EXE_cortex_worker"))
    .arg("--standalone")
    .arg("--input")
    .arg(&input)
    .arg("--output")
    .arg(&output)
    .output()
    .unwrap();
  let stderr = String::from_utf8_lossy(&run.stderr);
  assert!(run.status.success(), "cortex_worker failed:\n{stderr}");
  let log = zip_member(&output, "cortex.log");
  let html = zip_member(&output, "main.html");
  for text in [&*stderr, &*log] {
    assert!(
      !text.contains("probe.sty.rhai") && !text.contains("external program"),
      "cortex_worker loaded the source's runtime binding:\n{text}"
    );
  }
  assert!(
    !html.contains("PROBE-LOADED"),
    "the binding's macro reached the HTML:\n{html}"
  );
  // Positively: `probe` resolves as in a build without runtime bindings (the documented outcome).
  assert!(
    log.contains("Warning:missing_file:probe") && log.contains("Error:undefined:\\probe"),
    "cortex_worker did not reach `\\usepackage{{probe}}` as a build without runtime bindings would:\n{log}"
  );
}

/// The CLI keeps runtime bindings: the same source, unpacked, loads `probe.sty.rhai` and runs its command.
#[cfg(feature = "runtime-bindings")]
#[test]
fn the_cli_still_loads_the_runtime_binding() {
  let dir = tempfile::tempdir().unwrap();
  fs::write(dir.path().join("main.tex"), MAIN_TEX).unwrap();
  fs::write(dir.path().join("probe.sty.rhai"), PROBE_RHAI).unwrap();
  let html_path = dir.path().join("main.html");
  let run = Command::new(env!("CARGO_BIN_EXE_latexml_oxide"))
    .arg(format!("--dest={}", html_path.display()))
    .arg(dir.path().join("main.tex"))
    .output()
    .unwrap();
  let stderr = String::from_utf8_lossy(&run.stderr);
  assert!(run.status.success(), "latexml_oxide failed:\n{stderr}");
  assert!(
    stderr.contains("probe.sty.rhai") && stderr.contains("running the external program 'echo'"),
    "the CLI did not load the runtime binding:\n{stderr}"
  );
  assert!(
    fs::read_to_string(&html_path)
      .unwrap()
      .contains("PROBE-LOADED"),
    "the binding's macro is missing from the CLI's HTML"
  );
}
