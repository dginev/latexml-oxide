//! `cortex_worker` converts untrusted arXiv sources, so it never loads a runtime `.rhai` binding, in any build: a
//! `<pkg>.sty.rhai` shipped inside a paper would otherwise auto-load (`converter.rs` `rhai_dispatch`) and could run any
//! program through the binding API's `Command(..).output()` (docs/release/SAFETY.md). The same source through the
//! `latexml_oxide` CLI, the user's own machine, still loads it. Nor does it take a `00README` main file from outside
//! the source.
//!
//! Also the standalone worker's one memory knob: its `--max-rss-mb`, not `LATEXML_RSS_CAP_BYTES`, sets the fuse.

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
  write_zip(zip_path, &[
    ("main.tex", MAIN_TEX),
    ("probe.sty.rhai", PROBE_RHAI),
  ]);
}

/// `files` (name, body), zipped at `zip_path`.
fn write_zip(zip_path: &Path, files: &[(&str, &str)]) {
  let mut zip = zip::ZipWriter::new(fs::File::create(zip_path).unwrap());
  let options = zip::write::SimpleFileOptions::default();
  for &(name, body) in files {
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

/// A standalone worker's conversion runs under the fuse its `--max-rss-mb` sets, whatever `LATEXML_RSS_CAP_BYTES`
/// says (row R CORTEXRSS): a one-byte env fuse stopped the paper for memory, as the worker ignored its flag.
#[test]
fn a_standalone_workers_max_rss_mb_sets_its_fuse() {
  let dir = tempfile::tempdir().unwrap();
  let input = dir.path().join("paper.zip");
  let output = dir.path().join("result.zip");
  write_zip(&input, &[(
    "main.tex",
    "\\documentclass{article}\n\\begin{document}\nText.\n\\end{document}\n",
  )]);
  let run = Command::new(env!("CARGO_BIN_EXE_cortex_worker"))
    .env("LATEXML_RSS_CAP_BYTES", "1")
    .args(["--standalone", "--max-rss-mb", "6144", "--input"])
    .arg(&input)
    .arg("--output")
    .arg(&output)
    .output()
    .unwrap();
  let stderr = String::from_utf8_lossy(&run.stderr);
  assert!(run.status.success(), "cortex_worker failed:\n{stderr}");
  let log = zip_member(&output, "cortex.log");
  assert!(
    !log.contains("MemoryBudget") && !log.contains("streaming:restart"),
    "the conversion stopped for memory under a 6144 MiB --max-rss-mb:\n{log}"
  );
  assert!(zip_member(&output, "main.html").contains("Text."));
  let status = zip_member(&output, "status");
  assert!(
    matches!(status.trim(), "Status:conversion:0" | "Status:conversion:1"),
    "{status}:\n{log}"
  );
}

/// A `00README.json` naming a main file outside the source (an absolute path) is ignored with one `Warning:archive:path`:
/// the worker converts the source's own `main.tex` and never reads the named file (OD #1166).
#[test]
fn cortex_worker_ignores_a_readme_main_outside_the_source() {
  let dir = tempfile::tempdir().unwrap();
  let canary = dir.path().join("canary.tex");
  fs::write(
    &canary,
    "\\documentclass{article}\\begin{document}CANARY-OUTSIDE\\end{document}\n",
  )
  .unwrap();
  let readme = format!(
    r#"{{"sources":[{{"filename":"{}","usage":"toplevel"}}]}}"#,
    canary.to_string_lossy().replace('\\', "/")
  );
  let input = dir.path().join("paper.zip");
  let output = dir.path().join("result.zip");
  write_zip(&input, &[
    ("00README.json", readme.as_str()),
    (
      "main.tex",
      "\\documentclass{article}\n\\begin{document}\nINSIDE-MAIN\n\\end{document}\n",
    ),
  ]);
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
  let archive = zip::ZipArchive::new(fs::File::open(&output).unwrap()).unwrap();
  let names: Vec<String> = archive
    .file_names()
    .map(|n| n.unwrap().into_owned())
    .collect();
  assert!(
    names.iter().any(|n| n == "main.html"),
    "the source's main.tex is converted: {names:?}"
  );
  let html = zip_member(&output, "main.html");
  assert!(
    html.contains("INSIDE-MAIN") && !html.contains("CANARY"),
    "{html}"
  );
  let log = zip_member(&output, "cortex.log");
  let warnings: Vec<&str> = log.lines().filter(|l| l.starts_with("Warning:")).collect();
  assert_eq!(warnings.len(), 1, "one Warning:\n{log}");
  assert!(
    warnings[0].starts_with("Warning:archive:path the 00README names '")
      && warnings[0]
        .ends_with("canary.tex' as a top-level file outside the source directory; ignored"),
    "{}",
    warnings[0]
  );
}
