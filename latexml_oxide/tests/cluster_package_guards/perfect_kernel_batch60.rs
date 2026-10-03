//! Red/green guards for perfect-kernel phase-60 batches: G3 residual slice 3 (figbib's `@fig` entries, clefval's
//! values) and the §403 `scan_left_brace` of the alignment and box primitives (60j).
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

/// 60j: `\halign` and `\noalign` find their `{` as tex.web §403 `scan_left_brace` does (§774, §785) — expanded, a
/// `\protected` macro too, past spaces and `\relax` — so `\halign\relax{`, `\noalign\relax{\hrule}`, `\noalign\pb`,
/// `\halign\expandafter{\iffalse}\fi`, mdwtools' `\halign\expandafter\bgroup` and a nested `\halign` in a cell align as
/// in pdflatex. Repro kernel-alignment/alignment_brace_is_scanned.
#[test]
fn alignment_brace_is_scanned() {
  super::perfect_kernel_batch57::assert_elements(
    include_str!(
      "../../../tools/perfect_kernel/repros/kernel-alignment/alignment_brace_is_scanned.tex"
    ),
    RAW,
    (0, 0),
    &[
      (
        "para",
        "p1",
        r##"<para class="ltx_noindent" xml:id="p1"><tabular><tr><td align="left" class="ltx_nopad_l ltx_nopad_r">juliet</td></tr></tabular><tabular vattach="middle"><tbody><tr><td align="left">x</td></tr><tr><td align="left" border="t">y</td></tr></tbody></tabular></para>"##,
      ),
      (
        "para",
        "p2",
        r##"<para xml:id="p2"><tabular vattach="middle"><tbody><tr><td align="left">kilo</td></tr><tr><td align="left" border="t">lima</td></tr></tbody></tabular></para>"##,
      ),
      (
        "para",
        "p3",
        r##"<para xml:id="p3"><tabular><tr><td align="left" class="ltx_nopad_l ltx_nopad_r">mike</td></tr></tabular><tabular><tr><td align="left" class="ltx_nopad_l ltx_nopad_r">november</td></tr></tabular><tabular vattach="middle"><tbody><tr><td align="left"><tabular vattach="bottom"><tr><td align="left" class="ltx_nopad_l ltx_nopad_r">oscar</td></tr></tabular></td></tr><tr><td align="left">papa</td></tr></tbody></tabular></para>"##,
      ),
      (
        "para",
        "p4",
        r##"<para xml:id="p4"><p>Delta echo.</p></para>"##,
      ),
    ],
  );
}

/// 60j: the box primitives find their `{` after the box specification as tex.web §645 `scan_spec`'s §403
/// `scan_left_brace` — `\hbox\relax\mac` with `\def\mac{{bravo}}` is a box of "bravo"; skipping to the next `{`
/// unexpanded dropped the rest of the document. `\valign` swallows its alignment after the same scan. Repro
/// boxes-groups/box_brace_is_scanned.
#[test]
fn box_brace_is_scanned() {
  super::perfect_kernel_batch57::assert_elements(
    include_str!("../../../tools/perfect_kernel/repros/boxes-groups/box_brace_is_scanned.tex"),
    RAW,
    (0, 0),
    &[
      (
        "para",
        "p1",
        r##"<para class="ltx_noindent" xml:id="p1"><p>Alpha bravo charlie.</p></para>"##,
      ),
      (
        "para",
        "p2",
        r##"<para class="ltx_noindent" xml:id="p2"><p>Delta <text width="50.0pt">bravo</text> echo.</p></para>"##,
      ),
      (
        "para",
        "p3",
        r##"<para class="ltx_noindent" xml:id="p3"><p>Foxtrot bravo golf.</p></para>"##,
      ),
      (
        "para",
        "p4",
        r##"<para class="ltx_noindent" xml:id="p4"><p>Kilo <inline-block vattach="top"><p>bravo</p></inline-block> lima.</p></para>"##,
      ),
      (
        "para",
        "p5",
        r##"<para class="ltx_noindent" xml:id="p5"><p>Mike <inline-block class="ltx_markedasmath" vattach="bottom"><p>november</p></inline-block> oscar.</p></para>"##,
      ),
      (
        "para",
        "p6",
        r##"<para class="ltx_noindent" xml:id="p6"><p>Hotel  juliet.</p></para>"##,
      ),
    ],
  );
}

/// 60j: an undefined control sequence before a box's `{` is reported and discarded (tex.web §370 `expand`), and the
/// §404 scan goes on: `\hbox\undefinedzz{bravo}` is the box of "bravo", as pdflatex after the same error; the scan
/// stopping at it inserted a `{` and the box ran to the end of the document.
#[test]
fn box_brace_skips_an_undefined_macro() {
  assert_elements_with(
    "\\documentclass{article}\\begin{document}\n\\noindent Alpha \\hbox\\undefinedzz{bravo} charlie.\\par\n\\noindent \
     Delta.\\par\n\\end{document}\n",
    RAW,
    (1, 0),
    &["\\undefinedzz"],
    &[
      (
        "para",
        "p1",
        r##"<para class="ltx_noindent" xml:id="p1"><p>Alpha bravo charlie.</p></para>"##,
      ),
      (
        "para",
        "p2",
        r##"<para class="ltx_noindent" xml:id="p2"><p>Delta.</p></para>"##,
      ),
    ],
  );
}

/// 60m: amsart's `\enddoc@text` runs `\AtEndDocument` (amsart.cls:518-520), so the end matter a derived class queues
/// there is set after the body: resphilosophica's `{notes}` collection (`\AddtoEndMatter`, resphilosophica.cls:94,
/// :432), lost before. pdflatex: "Bibliography notes:" / "Collected sentence alpha." after "Body text.". Repro
/// sectioning-frontmatter/amsart_end_matter_is_set.
#[test]
fn amsart_end_matter_is_set() {
  assert_elements_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/amsart_end_matter_is_set.tex"
    ),
    RAW,
    (0, 0),
    &[],
    &[
      (
        "para",
        "p3",
        r##"<para xml:id="p3"><p>Body text.</p></para>"##,
      ),
      (
        "para",
        "p4",
        r##"<para class="ltx_noindent" xml:id="p4"><p><text fontsize="80%">Bibliography notes:</text></p></para>"##,
      ),
      (
        "para",
        "p5",
        r##"<para xml:id="p5"><p><text fontsize="80%">Collected sentence alpha.</text></p></para>"##,
      ),
    ],
  );
}

/// 60m: ltxdockit's `\rcsid` sets the revision's date (`\ltd@setdate`, ltxdockit.cls:129-137, the aux read-back the
/// binding replays in the same run), which `\rcstoday` prints: pdflatex "January 3, 2011", where the title page had
/// the conversion day. Repro sectioning-frontmatter/ltxdockit_rcsid_sets_the_date. (The two KOMA warnings are every
/// scrartcl document's: KOMA's check of the kernel sectioning macros, which are LaTeXML's; CONTROL
/// sectioning-frontmatter/koma_sectioning_check_warns.)
#[test]
fn ltxdockit_rcsid_sets_the_date() {
  let xml = assert_elements_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/ltxdockit_rcsid_sets_the_date.tex"
    ),
    RAW,
    (0, 2),
    &[
      "`\\@startsection' has been changed",
      "Unexpected definition of \\@sect",
    ],
    &[],
  );
  assert_element(
    &xml,
    "date",
    &["role=\"creation\""],
    r##"<date role="creation">January 3, 2011</date>"##,
  );
}

/// 60m: what a class or document appends to amsart's `\addresses` itself (smfart.cls:420-422) is set after the body by
/// amsart's `\@setaddresses` (amsart.cls:524-549, ported raw): pdflatex prints "Raw Appended Institute" after the
/// body; the hook without it was "Error:undefined:\@setaddresses". The binding's own `\address` stays in the
/// frontmatter. Repro sectioning-frontmatter/amsart_raw_addresses_are_set.
#[test]
fn amsart_raw_addresses_are_set() {
  let xml = assert_elements_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/amsart_raw_addresses_are_set.tex"
    ),
    RAW,
    (0, 0),
    &[],
    &[
      (
        "para",
        "p1",
        r##"<para xml:id="p1"><p>Body text.</p></para>"##,
      ),
      (
        "para",
        "p2",
        r##"<para class="ltx_indent" xml:id="p2"><p><text font="smallcaps" fontsize="80%">Raw Appended Institute</text></p></para>"##,
      ),
    ],
  );
  assert_element(
    &xml,
    "contact",
    &["role=\"address\""],
    "<contact name=\"Address:\u{a0}\" role=\"address\">First University</contact>",
  );
}

/// 60l: `\@array` zeroes `\lineskip` and `\baselineskip` for the alignment's body (latex.ltx:16580), so colortbl's
/// `\ifdim\baselineskip=\z@\noalign\fi{…}` (colortbl.sty:158, :163; tabu.sty:2159) is a `\noalign` between rows: the
/// table keeps its three rows and rules, as pdflatex, where the group became a cell ("\noalign cannot be used here",
/// two "Extra alignment tab"). Repro kernel-alignment/colortbl_noalign_idiom.
#[test]
fn colortbl_noalign_idiom() {
  assert_elements_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/kernel-alignment/colortbl_noalign_idiom.tex"
    ),
    RAW,
    (0, 0),
    &[],
    &[
      (
        "para",
        "p1",
        r##"<para xml:id="p1"><tabular class="ltx_guessed_headers" vattach="middle"><thead><tr><td align="left" border="l r t" thead="column">a</td><td align="left" border="r t" thead="column">b</td></tr></thead><tbody><tr><td align="left" border="l r t">c</td><td align="left" border="r t">d</td></tr><tr><td align="left" border="b l r t">e</td><td align="left" border="b r t">f</td></tr></tbody></tabular></para>"##,
      ),
      ("para", "p2", r##"<para xml:id="p2"><p>Final.</p></para>"##),
    ],
  );
}

/// 60l: the interline values in each context of a table, as pdflatex: zero in an l cell, between rows and in a nested
/// table (latex.ltx:16580 `\@array`), the document's in a p cell, `\parbox` and minipage (`\@arrayparboxrestore`,
/// :16272-16287). Repro kernel-alignment/array_zeroes_the_interline_values.
#[test]
fn array_zeroes_the_interline_values() {
  assert_elements_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/kernel-alignment/array_zeroes_the_interline_values.tex"
    ),
    RAW,
    (0, 0),
    &[],
    &[(
      "para",
      "p1",
      r##"<para xml:id="p1"><tabular class="ltx_guessed_headers" vattach="middle"><tbody><tr><td align="left" thead="row">0.0pt/0.0pt</td><td align="left" vattach="top"><inline-block vattach="top" width="85.4pt"><p>0.0pt/1.0pt</p></inline-block></td></tr><tr><td align="left" thead="row"><inline-block class="ltx_parbox" vattach="middle" width="56.9pt"><p>12.0pt</p></inline-block></td><td align="left" vattach="top"><tabular vattach="middle" width="85.4pt"><tr><td align="left">0.0pt</td></tr></tabular></td></tr></tbody></tabular></para>"##,
    )],
  );
}

/// 60l: a table nested in a cell, where `\@array` zeroed `\baselineskip`, keeps its rows' height: the strut is
/// `\strutbox`'s (latex.ltx:16567-16570, tex_tables.rs `array_strut`), as pdflatex's "nested 14.5pt/9.5pt". Repro
/// kernel-alignment/nested_table_keeps_its_strut.
#[test]
fn nested_table_keeps_its_strut() {
  assert_elements_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/kernel-alignment/nested_table_keeps_its_strut.tex"
    ),
    RAW,
    (0, 0),
    &[],
    &[(
      "para",
      "p1",
      r##"<para class="ltx_noindent" xml:id="p1"><p>nested 14.5pt/9.5pt; outer 14.5pt/9.5pt.</p></para>"##,
    )],
  );
}

/// 60l: every cell set as a paragraph box gives back the interline values `\@array` zeroed (`\@arrayparboxrestore`):
/// a p cell, tabularx's X, tabulary's L/C and a `\multirow` with a width other than `*` (multirow.sty:174-177); an l
/// cell and a `*` multirow keep the zero. pdflatex's markers, [Z] zero and [N] the document's. Repro
/// kernel-alignment/array_cells_restore_their_interline_values.
#[test]
fn array_cells_restore_their_interline_values() {
  let xml = assert_elements_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/kernel-alignment/array_cells_restore_their_interline_values.tex"
    ),
    RAW,
    (0, 0),
    &[],
    &[],
  );
  let text = regex::Regex::new(r"<[^>]+>")
    .unwrap()
    .replace_all(&xml, " ")
    .into_owned();
  let markers: Vec<String> = regex::Regex::new(r"([A-Za-z]+)\s*\[([ZN])\]")
    .unwrap()
    .captures_iter(&text)
    .map(|c| format!("{}[{}]", &c[1], &c[2]))
    .collect();
  assert_eq!(
    markers,
    [
      "s[Z]", "e[N]", "d[N]", "y[N]", "t[N]", "l[Z]", "p[N]", "After[N]", "bb[N]", "l[Z]", "bb[N]",
      "cc[N]"
    ],
    "{xml}"
  );
}

/// 60l: amsmath's matrices, `cases` and `aligned` in a table cell keep their rows' strut (`\strutbox`'s, tex_tables.rs
/// `array_strut`), not the cell's zero `\baselineskip` (pmatrix was 9.44pt/4.44pt). Repro
/// kernel-alignment/array_strut_in_a_cell.
#[test]
fn array_strut_in_a_cell() {
  assert_elements_with(
    include_str!("../../../tools/perfect_kernel/repros/kernel-alignment/array_strut_in_a_cell.tex"),
    RAW,
    (0, 0),
    &[],
    &[(
      "para",
      "p1",
      r##"<para class="ltx_noindent" xml:id="p1"><p>pmatrix 11.97221pt/6.97221pt; cases 12.69444pt/7.69444pt; aligned 11.97221pt/6.97221pt.</p></para>"##,
    )],
  );
}

/// 60l: a `tblr` keeps its interline values (tabularray's own alignment never zeroes them): `\\[\baselineskip]` is a
/// 12pt gap, as pdflatex, where 60l's `\@array` zero had removed it. Repro
/// kernel-alignment/tblr_keeps_its_interline_values.
#[test]
fn tblr_keeps_its_interline_values() {
  let xml = assert_elements_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/kernel-alignment/tblr_keeps_its_interline_values.tex"
    ),
    RAW,
    (0, 0),
    &[],
    &[],
  );
  assert_element(
    &xml,
    "tr",
    &[],
    r##"<tr><td align="left" border="l r t" cssstyle="padding-bottom: 12.0pt">a</td><td align="left" border="r t" cssstyle="padding-bottom: 12.0pt">b</td></tr>"##,
  );
}

/// 60l: a minipage's stacked lines keep the `\lineskip` in force inside it (tex.web §679): its captured body records the
/// interline values while its group is open, so the 1pt gap between two rules survives `\lineskip=0pt` around it and
/// a table cell's zero (arXiv 2605.19065's tcolorbox cells), as pdflatex's 58.0pt+53.0pt. Repro
/// kernel-alignment/minipage_in_a_cell_keeps_its_lineskip.
#[test]
fn minipage_in_a_cell_keeps_its_lineskip() {
  assert_elements_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/kernel-alignment/minipage_in_a_cell_keeps_its_lineskip.tex"
    ),
    RAW,
    (0, 0),
    &[],
    &[(
      "para",
      "p1",
      r##"<para class="ltx_noindent" xml:id="p1"><p>plain 58.0pt+53.0pt; lineskip0 58.0pt+53.0pt; cell 58.0pt+53.0pt.</p></para>"##,
    )],
  );
}
