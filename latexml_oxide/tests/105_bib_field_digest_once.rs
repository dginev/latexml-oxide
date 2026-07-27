//! Regression test: a bibliography field value is digested EXACTLY ONCE.
//!
//! The original defect was two digesting paths in the since-deleted
//! `convert_bib_file_to_xml` string route — `interpret_tex_markup` (XML
//! fragment, so `\url`/`\href`/font switches survive) and `interpret_tex_text`
//! (plain string) — both run over the SAME value, so every error that field
//! raised was reported twice and every macro side effect ran twice. That route
//! is gone (`BIBLIOGRAPHY_WORKLIST.md` re-port item 1: the recursive `.bib`
//! session replaced it), but the PROPERTY it violated is a standing one for
//! whatever route is current, and it is cheap to keep pinned.
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
/// * The probe must raise its error on EVERY digest. An undefined macro will
///   NOT do: it is defined as `<ltx:ERROR/>` on first sight and is therefore
///   silently self-healing on a second pass, so an undefined-macro fixture
///   passes even with the bug present. `\hline` in a `note` is the probe — it
///   expands to `\noalign`, which is a CONTEXT error (`\noalign cannot be used
///   here`) with nothing to memoize, so a second digest counts a second time.
///   Verified: two entries each carrying one `\hline` produce exactly 2.
/// * The value must contain a BACKSLASH. The interpretation paths short-circuit
///   on a value with no `\`, `~` or `$`, so a punctuation-only probe never
///   digests at all and the test goes vacuously green (observed: "digested 0
///   times"). `\textbf` is the carrier in the second entry because it needs no
///   package.
///
/// `_` and `^` were the two earlier probes and neither can be one any more:
/// OXIDIZED_DESIGN #74 escapes `_ & # %` and `^` in a `.bib` field as DATA, so
/// `note={a _ … ^ …}` now renders the literal characters and raises nothing.
/// The `a2` entry keeps both of them as the standing check that the escaping did
/// not disturb the once-only property — it must contribute ZERO errors — while
/// `a1`'s `\hline` is the live probe.
const BIB: &str = r"@article{a1, author={Doe, J.}, title={T}, year={2020},
  note={a \hline \textbf{b}} }
@article{a2, author={Roe, R.}, title={T2}, year={2021},
  note={x _ y ^ z \textbf{w}} }
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

  let needle = "\\noalign cannot be used here";
  let n = stderr.matches(needle).count();
  assert_eq!(
    n, 1,
    "the field was digested {n} times, not once — bibliography errors are \
     being multiplied into the document's error count.\nstderr:\n{stderr}"
  );
  // `_` and `^` are DATA in a `.bib` field (OXIDIZED_DESIGN #74), so `a2` must
  // raise nothing. Asserted rather than dropped: if the escaping ever regresses
  // this catches it here too, and once-per-digest would show up as a count of 2.
  for script in ['_', '^'] {
    let n = stderr
      .matches(&format!("Script {script} can only appear in math mode"))
      .count();
    assert_eq!(
      n, 0,
      "a `{script}` in a bib field is data and must raise nothing, got {n}.\n\
       stderr:\n{stderr}"
    );
  }
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
