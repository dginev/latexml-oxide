//! MathML for a `\parbox` (text box with nested `$...$` math) placed INSIDE
//! display math — full pipeline, runs the MathML post-processor + XSLT.
//!
//! arXiv html_feedback #6847 (arXiv:2608.05024v1): three display formulas in
//! Sections 3-4 that wrap a two-line `\parbox` of inline-math conditions inside
//! a set-builder, e.g.
//! `\Delta_k(S):=\sup\{|\det(BSA)|:\parbox{..}{$A\in\mathcal L(..),\|A\|\le1$,\\ $B\in..$}\}`.
//! The parbox's inline math is a nested `ltx:Math` that the top-level MathML
//! pass skips (`//ltx:Math[not(ancestor::ltx:Math)]`). It was then cloned
//! VERBATIM into the output as raw `<ltx:XMath>` content-MathML, which the
//! browser renders in operator-first document order — the reporter's garbled
//! `∈AL(ℓ2k,X),≤‖A‖1`.
//!
//! SHARED FAILURE with Perl (`MathML.pm` L1063-1073 clones raw + warns
//! `unexpected:nested-math`). The fix converts the nested math in place in
//! `rebuild_text_subtree_with_doc` (and makes `convert_to_pmml` reentrancy-safe)
//! — a surpass-Perl divergence (OXIDIZED_DESIGN). The in-process `Converter`
//! (`06_cluster_regressions.rs`) stops at Core XML and the `convert_and_post`
//! helper runs post with `pmml:false`, so this can only be checked end-to-end
//! via the binary — like `07_xslt_seclev_levels.rs`.

use std::{path::Path, process::Command};

fn run(cwd: &Path, args: &[&str]) -> std::process::Output {
  Command::new(env!("CARGO_BIN_EXE_latexml_oxide"))
    .args(args)
    .current_dir(cwd)
    .output()
    .expect("spawn latexml_oxide")
}

const TEX: &str = "\\documentclass{article}\n\
   \\usepackage{amsmath,amssymb}\n\
   \\newcommand{\\norm}[1]{\\|#1\\|}\n\
   \\begin{document}\n\
   \\[\n\
     \\Delta_k(S) := \\sup\\Bigl\\{\\, |\\det(BSA)| :\n\
     \\parbox{42mm}{\n\
     $A \\in \\mathcal{L}(\\ell_2^{k}, X), \\norm{A}\\le1$,\\\\[1mm]\n\
     $B \\in \\mathcal{L}(Y, \\ell_2^{k}), \\norm{B} \\le 1$}\\,\n\
     \\Bigr\\},\n\
   \\]\n\
   \\end{document}\n";

#[test]
fn parbox_nested_math_converts_to_presentation_mathml() {
  let work = tempfile::tempdir().expect("tempdir");
  std::fs::write(work.path().join("pb.tex"), TEX).unwrap();
  let out = run(work.path(), &["pb.tex", "--dest", "pb.html"]);
  assert!(
    out.status.success(),
    "conversion failed (status {:?}):\n{}",
    out.status.code(),
    String::from_utf8_lossy(&out.stderr)
  );
  let html = std::fs::read_to_string(work.path().join("pb.html")).expect("read pb.html");

  // Primary canary: no content-MathML may survive into the HTML. Before the fix
  // the parbox's nested math leaked as raw `<XMTok>`/`<XMApp>` — the browser
  // then renders those tokens in document (operator-first) order.
  assert!(
    !html.contains("XMTok") && !html.contains("XMApp") && !html.contains("XMDual"),
    "raw content-MathML leaked into the HTML — the \\parbox's nested math was not \
     converted (renders operator-first, garbled):\n{html}"
  );

  // Reading order inside the parbox: the operand `A` must precede the relation
  // `∈` (U+2208). Content-MathML is operator-first (`∈ A`); presentation MathML
  // is `A ∈`. Anchor at the parbox so we do not match the outer formula.
  let pb = &html[html
    .find("ltx_parbox")
    .expect("no ltx_parbox in output — the parbox itself was lost")..];
  let a_pos = pb
    .find("<mi>A</mi>")
    .expect("no presentation <mi>A</mi> in the parbox");
  let in_pos = pb.find('\u{2208}').expect("no ∈ operator in the parbox");
  assert!(
    a_pos < in_pos,
    "the ∈ operator precedes its operand A in the parbox — content-MathML \
     document order leaked into presentation:\n{pb}",
  );

  // The nested math must be a real inline `<math>` element. The parbox becomes
  // an HTML `<span class="ltx_inline-block">` (an HTML5 MathML-breakout tag), so
  // a bare `<mrow>` there would parse as HTML and render as FLAT TEXT — the
  // `<math>` re-enters MathML context so subscripts/superscripts render. `pb`
  // begins after the outer `<math>`, so any `<math` here is a nested one.
  assert!(
    pb.contains("<math"),
    "the parbox's nested math is not wrapped in a <math> element — a bare <mrow> \
     inside the inline-block's HTML renders as flat text, not math:\n{pb}",
  );
}
