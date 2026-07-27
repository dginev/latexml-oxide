//! Regression test: a bibliography field value is digested EXACTLY ONCE.
//!
//! `convert_bib_file_to_xml` has two digesting paths — `interpret_tex_markup`
//! (XML fragment, so `\url`/`\href`/font switches survive) and
//! `interpret_tex_text` (plain string). They digest the SAME value, so running
//! both on one field re-reports every error that field raises and re-runs any
//! side effect its macros have. The markup path therefore runs first and
//! suppresses the text path on success.
//!
//! This is guarded by counting `Error:` lines rather than by inspecting the XML,
//! because the duplicate is invisible in the output — the rendered entry looked
//! perfectly fine while the document's error count silently doubled. Error
//! counts are the canvas pass/fail signal, so inflating them is a real defect.
//!
//! Binary-driven: the count has to come from the conversion log.

use std::{path::Path, process::Command};

/// Two properties this fixture must have, both learned the hard way:
///
/// * `^` outside math raises `unexpected:^` on EVERY digest — unlike an
///   undefined macro, which is defined as `<ltx:ERROR/>` on first sight and is
///   therefore silently self-healing on a second pass. An undefined-macro
///   fixture would pass even with the bug present.
/// * The value must contain a BACKSLASH. Both interpret paths short-circuit on
///   a value with no `\`, `~` or `$`, so `note={a ^ b}` never digests at all and
///   the test goes vacuously green (observed: "digested 0 times"). `\textbf` is
///   the carrier here because it needs no package.
///
/// `_` used to be the other probe, and no longer can be: OXIDIZED_DESIGN #74
/// escapes `_ & # %` in a `.bib` field as DATA, so `note={a _ …}` now renders a
/// literal underscore and raises nothing at all. The `a1` entry stays in the
/// fixture as the standing check that escaping did not disturb the once-only
/// property — it must contribute ZERO errors — while `a2`'s `^` (outside the
/// escaped set) remains the live probe.
const BIB: &str = r"@article{a1, author={Doe, J.}, title={T}, year={2020},
  note={a _ \textbf{b}} }
@article{a2, author={Roe, R.}, title={T2}, year={2021},
  note={x ^ \textbf{y}} }
";

const TEX: &str = r"\documentclass{article}
\begin{document}
See \cite{a1,a2}.
\bibliographystyle{plain}
\bibliography{refs}
\end{document}
";

#[test]
fn bib_field_errors_are_reported_once_not_once_per_digest() {
  let bin = env!("CARGO_BIN_EXE_latexml_oxide");
  assert!(Path::new(bin).is_file(), "binary not staged at {bin}");

  let workdir = tempfile::tempdir().expect("create tempdir");
  std::fs::write(workdir.path().join("refs.bib"), BIB).expect("write refs.bib");
  std::fs::write(workdir.path().join("t.tex"), TEX).expect("write t.tex");

  let output = Command::new(bin)
    .args([
      "t.tex",
      "--dest",
      "t.html",
      "--format=html5",
      "--nocomments",
    ])
    .current_dir(workdir.path())
    .output()
    .expect("spawn latexml_oxide");
  // ANSI-strip before counting: a naive grep over coloured output matches zero
  // and would make this test vacuously green.
  let stderr = strip_ansi(&String::from_utf8_lossy(&output.stderr));

  let needle = "Script ^ can only appear in math mode";
  let n = stderr.matches(needle).count();
  assert_eq!(
    n, 1,
    "the field was digested {n} times, not once — bibliography errors are \
     being multiplied into the document's error count.\nstderr:\n{stderr}"
  );
  // `_` is DATA in a `.bib` field (OXIDIZED_DESIGN #74), so `a1` must raise
  // nothing. Asserted rather than dropped: if the escaping ever regresses, this
  // catches it here too, and once-per-digest would show up as a count of 2.
  let n = stderr
    .matches("Script _ can only appear in math mode")
    .count();
  assert_eq!(
    n, 0,
    "a `_` in a bib field is data and must raise nothing, got {n}.\nstderr:\n{stderr}"
  );
}

fn strip_ansi(s: &str) -> String {
  let mut out = String::with_capacity(s.len());
  let mut chars = s.chars().peekable();
  while let Some(c) = chars.next() {
    if c == '\u{1b}' && chars.peek() == Some(&'[') {
      for c in chars.by_ref() {
        if c.is_ascii_alphabetic() {
          break;
        }
      }
    } else {
      out.push(c);
    }
  }
  out
}
