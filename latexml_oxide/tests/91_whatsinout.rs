//! End-to-end `--whatsin` / `--whatsout` CLI coverage (Perl
//! `LaTeXML::Util::Pack` + `LaTeXML.pm` driver logic).
//!
//! Exercises the binary the way a user invokes it, asserting the
//! shape of the output for each `whatsout` mode:
//!
//! * `document` (default) → full HTML page (has `<head>`).
//! * `fragment` → embeddable snippet (no page chrome).
//! * `archive` → a zip bundle (HTML + status), with a placeholder
//!   `<source>.zip` destination when `--dest` is omitted (Perl
//!   LaTeXML.pm:185-187).
//!
//! Run via the prebuilt-binary harness (`CARGO_BIN_EXE_latexml_oxide`),
//! like `001_single_binary_smoke.rs`.

use std::io::Read;
use std::path::Path;
use std::process::Command;

const HELLO_TEX: &str = "\\documentclass{article}\n\
                         \\begin{document}\n\
                         Hello World!\n\
                         \\end{document}\n";

/// Spawn the binary in `cwd` with `args`, returning the captured output.
fn run(cwd: &Path, args: &[&str]) -> std::process::Output {
  let bin = env!("CARGO_BIN_EXE_latexml_oxide");
  Command::new(bin)
    .args(args)
    .current_dir(cwd)
    .output()
    .expect("spawn latexml_oxide")
}

fn stderr_of(out: &std::process::Output) -> String {
  String::from_utf8_lossy(&out.stderr).to_string()
}

#[test]
fn whatsout_document_is_full_page() {
  let work = tempfile::tempdir().expect("tempdir");
  std::fs::write(work.path().join("hello.tex"), HELLO_TEX).unwrap();
  let out = run(work.path(), &["hello.tex", "--dest", "doc.html"]);
  assert!(
    out.status.success(),
    "status {:?}\nstderr:\n{}",
    out.status.code(),
    stderr_of(&out)
  );
  let html = std::fs::read_to_string(work.path().join("doc.html")).expect("read doc.html");
  assert!(html.contains("Hello World"), "missing body text:\n{html}");
  // A full document carries the page chrome (`<head>`…`</head>`).
  assert!(
    html.contains("<head") && html.contains("</head>"),
    "document output should be a full HTML page with a <head>:\n{html}"
  );
}

#[test]
fn whatsout_fragment_strips_page_chrome() {
  let work = tempfile::tempdir().expect("tempdir");
  std::fs::write(work.path().join("hello.tex"), HELLO_TEX).unwrap();
  let out = run(work.path(), &[
    "hello.tex",
    "--whatsout",
    "fragment",
    "--dest",
    "frag.html",
  ]);
  assert!(
    out.status.success(),
    "status {:?}\nstderr:\n{}",
    out.status.code(),
    stderr_of(&out)
  );
  let frag = std::fs::read_to_string(work.path().join("frag.html")).expect("read frag.html");
  assert!(frag.contains("Hello World"), "missing body text:\n{frag}");
  // An embeddable fragment must NOT carry the full-page `<head>`/`<html>`
  // wrapper — that is the whole point of `--whatsout=fragment`.
  assert!(
    !frag.contains("<head") && !frag.contains("<html"),
    "fragment output must not carry page chrome:\n{frag}"
  );
}

#[test]
fn whatsout_archive_writes_zip_bundle() {
  let work = tempfile::tempdir().expect("tempdir");
  std::fs::write(work.path().join("hello.tex"), HELLO_TEX).unwrap();
  let out = run(work.path(), &[
    "hello.tex",
    "--whatsout",
    "archive",
    "--dest",
    "bundle.zip",
  ]);
  assert!(
    out.status.success(),
    "status {:?}\nstderr:\n{}",
    out.status.code(),
    stderr_of(&out)
  );
  let zip_path = work.path().join("bundle.zip");
  assert!(zip_path.is_file(), "expected bundle.zip on disk");

  let f = std::fs::File::open(&zip_path).unwrap();
  let mut archive = zip::ZipArchive::new(f).expect("valid zip");
  let names: Vec<String> = (0..archive.len())
    .map(|i| archive.by_index(i).unwrap().name().to_string())
    .collect();
  assert!(
    names.iter().any(|n| n == "bundle.html"),
    "zip should contain bundle.html; names: {names:?}"
  );
  assert!(
    names.iter().any(|n| n == "status"),
    "zip should contain a status entry; names: {names:?}"
  );
  // The bundled HTML is the full document.
  let mut html = String::new();
  archive
    .by_name("bundle.html")
    .unwrap()
    .read_to_string(&mut html)
    .unwrap();
  assert!(html.contains("Hello World"), "bundled HTML missing body:\n{html}");
}

#[test]
fn whatsout_archive_defaults_destination_to_source_zip() {
  // Perl LaTeXML.pm:185-187: `--whatsout=archive` with no `--dest`
  // invents `<source-name>.zip`.
  let work = tempfile::tempdir().expect("tempdir");
  std::fs::write(work.path().join("paper.tex"), HELLO_TEX).unwrap();
  let out = run(work.path(), &["paper.tex", "--whatsout", "archive"]);
  assert!(
    out.status.success(),
    "status {:?}\nstderr:\n{}",
    out.status.code(),
    stderr_of(&out)
  );
  let zip_path = work.path().join("paper.zip");
  assert!(
    zip_path.is_file(),
    "expected placeholder paper.zip on disk; stderr:\n{}",
    stderr_of(&out)
  );
  // With no --dest/--format an archive still defaults to an html5 web
  // bundle: `paper.html` inside, carrying the body text.
  let f = std::fs::File::open(&zip_path).unwrap();
  let mut archive = zip::ZipArchive::new(f).expect("valid zip");
  let names: Vec<String> = (0..archive.len())
    .map(|i| archive.by_index(i).unwrap().name().to_string())
    .collect();
  assert!(
    names.iter().any(|n| n == "paper.html"),
    "placeholder zip should contain paper.html; names: {names:?}"
  );
  let mut html = String::new();
  archive
    .by_name("paper.html")
    .unwrap()
    .read_to_string(&mut html)
    .unwrap();
  assert!(html.contains("Hello World"), "bundled HTML missing body:\n{html}");
}
