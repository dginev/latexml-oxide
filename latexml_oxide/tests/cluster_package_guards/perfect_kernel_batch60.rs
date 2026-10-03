//! Red/green guards for perfect-kernel phase-60 batches: G3 residual slice 3 (figbib's `@fig` entries, clefval's
//! values).
use std::process::Command;

use latexml::util::test::assert_element;

use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch57::{RAW, assert_elements_with},
};

/// Convert `tex` as `t.tex` to HTML in one process under the raw preload (a bibliography is built by post). Returns
/// (ANSI-stripped stderr, HTML).
fn convert_html(tex: &str) -> (String, String) {
  let bin = env!("CARGO_BIN_EXE_latexml_oxide");
  let workdir = tempfile::tempdir().expect("create tempdir");
  std::fs::write(workdir.path().join("t.tex"), tex).expect("write t.tex");
  let output = Command::new(bin)
    .args([
      "t.tex",
      "--dest",
      "t.html",
      "--nocomments",
      "--timeout=110",
      &format!("--preload={RAW}"),
    ])
    .current_dir(workdir.path())
    .output()
    .expect("spawn latexml_oxide");
  let stderr = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
  let html = std::fs::read_to_string(workdir.path().join("t.html")).unwrap_or_default();
  (stderr, html)
}

/// 60h: figbib's figure-source list is the figures used, in figure order (each figure command's `\citation`, figbib.bst
/// has no SORT), each `@fig` entry printing its fields — `main` as its title, `add` and `source` (after `\figbibFrom`)
/// as notes, an empty one not at all — under `\figbibListHeader` (figbib.bst:19-35, figbib.sty:365-381); no handler knew
/// them, so each entry was its number alone, and the list was alphabetical. The three warnings are figbib's own
/// "Figure … undefined", whose data comes from the next run's `.aux` (pdflatex's first pass too). Repro
/// index-bib/figbib_fig_entries_print_their_fields.
#[test]
fn figbib_fig_entries_print_their_fields() {
  let (stderr, html) = convert_html(include_str!(
    "../../../tools/perfect_kernel/repros/index-bib/figbib_fig_entries_print_their_fields.tex"
  ));
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 3, "{stderr}");
  for figure in ["alpha", "foxtrot", "kilo"] {
    assert!(
      stderr.contains(&format!("Figure `{figure}' on page 1 undefined")),
      "{stderr}"
    );
  }
  assert_element(
    &html,
    "h2",
    &[r#"class="ltx_title ltx_title_bibliography""#],
    r##"<h2 class="ltx_title ltx_title_bibliography">List of Figures</h2>"##,
  );
  assert_element(
    &html,
    "li",
    &[r#"id="bib.bib1""#],
    r##"<li class="ltx_bibitem ltx_bib_misc" id="bib.bib1"><span class="ltx_tag ltx_bib_key ltx_role_refnum ltx_tag_bibitem">[2]</span><span class="ltx_bibblock"><span class="ltx_text ltx_bib_title">Bravo charlie</span>.</span><span class="ltx_bibblock">Note: <span class="ltx_text ltx_bib_note">From: India juliet</span></span><span class="ltx_bibblock ltx_bib_cited">Cited by: <a class="ltx_ref" href="#p1" title="">p1</a>.</span></li>"##,
  );
  assert_element(
    &html,
    "li",
    &[r#"id="bib.bib2""#],
    r##"<li class="ltx_bibitem ltx_bib_misc" id="bib.bib2"><span class="ltx_tag ltx_bib_key ltx_role_refnum ltx_tag_bibitem">[1]</span><span class="ltx_bibblock"><span class="ltx_text ltx_bib_title">Golf hotel</span>.</span><span class="ltx_bibblock">Note: <span class="ltx_text ltx_bib_note">Delta echo</span></span><span class="ltx_bibblock ltx_bib_cited">Cited by: <a class="ltx_ref" href="#p1" title="">p1</a>.</span></li>"##,
  );
  assert_element(
    &html,
    "li",
    &[r#"id="bib.bib3""#],
    r##"<li class="ltx_bibitem ltx_bib_misc" id="bib.bib3"><span class="ltx_tag ltx_bib_key ltx_role_refnum ltx_tag_bibitem">[3]</span><span class="ltx_bibblock"><span class="ltx_text ltx_bib_title">Lima mike</span>.</span><span class="ltx_bibblock ltx_bib_cited">Cited by: <a class="ltx_ref" href="#p1" title="">p1</a>.</span></li>"##,
  );
  // Listed in figure order: Golf hotel (bib2), Bravo charlie (bib1), Lima mike (bib3).
  let at = |id: &str| {
    html
      .find(&format!("id=\"{id}\""))
      .unwrap_or_else(|| panic!("no {id}:\n{html}"))
  };
  assert!(
    at("bib.bib2") < at("bib.bib1") && at("bib.bib1") < at("bib.bib3"),
    "not in figure order:\n{html}"
  );
  assert!(
    !html.contains("November oscar"),
    "an entry no figure cites is listed:\n{html}"
  );
}

/// 60h: clefval's `\TheKey` defines its value at once, as the next run's `.aux` read-back would, so a value used after
/// its key prints; one used before it keeps pdflatex's first-pass "[?? key ??]" and its warning (DIVERGENCES #425).
/// Repro singletons/clefval_value_after_its_key.
#[test]
fn clefval_value_after_its_key() {
  assert_elements_with(
    include_str!("../../../tools/perfect_kernel/repros/singletons/clefval_value_after_its_key.tex"),
    RAW,
    (0, 1),
    &["Value of `b' on page 1 undefined"],
    &[(
      "para",
      "p1",
      r##"<para xml:id="p1"><p>Alpha <text font="bold">[?? b ??]</text> bravo. Delta Charlie echo. Foxtrot  hotel.</p></para>"##,
    )],
  );
}

/// 60h: a clefval key defined twice warns as the `.aux` read-back does (`\@newk@ey`: "Key … multiply defined"); its
/// values take effect in reading order (DIVERGENCES #425).
#[test]
fn clefval_key_defined_twice_warns() {
  assert_elements_with(
    "\\documentclass{article}\\usepackage{clefval}\\begin{document}\n\\TheKey{b}{Bravo}Alpha \\TheValue{b} \
     charlie.\\TheKey{b}{Other}\n\\end{document}\n",
    RAW,
    (0, 1),
    &["Key `b' multiply defined"],
    &[(
      "para",
      "p1",
      r##"<para xml:id="p1"><p>Alpha Bravo charlie.</p></para>"##,
    )],
  );
}
