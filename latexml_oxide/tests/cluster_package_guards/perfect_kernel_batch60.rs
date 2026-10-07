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
      // The authors, in the class's centred `\trivlist` (resphilosophica.cls:318-328; KNOWN_PERL_ERRORS #456: an
      // empty `<itemize/>` before them).
      (
        "para",
        "p1",
        r##"<para xml:id="p1"><itemize class="ltx_trivlist" xml:id="S0.I1"><item class="ltx_centering" xml:id="S0.I1.ix1"><tags><tag/></tags><para xml:id="S0.I1.ix1.p1"><p><text fontsize="120%">A</text></p></para></item></itemize></para>"##,
      ),
      (
        "para",
        "p2",
        r##"<para xml:id="p2"><p>Body text.</p></para>"##,
      ),
      (
        "para",
        "p3",
        r##"<para class="ltx_noindent" xml:id="p3"><p><text fontsize="80%">Bibliography notes:</text></p></para>"##,
      ),
      (
        "para",
        "p4",
        r##"<para xml:id="p4"><p><text fontsize="80%">Collected sentence alpha.</text></p></para>"##,
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

/// 60o (ruling 7e): an `\autoref` name is built at its target (ruling 2026-10-02 (A)) but its diagnostics are raised
/// where an `\autoref` prints it, as TeX expands `\<type>autorefname` only there (hyperref.sty:8202-8278). hyperref's
/// Russian section name `\cyr\cyrr\cyra\cyrz\cyrd.` (hyperref.sty:2952) is undefined under lualatex's TU: no `\autoref`
/// (or only a `\ref`), no error (lualatex 0); `\autoref`s, its 4 undefined names once each — LaTeXML reports an undefined
/// command once per document, also across two targets (lualatex 8, once per use) — with the status summary's undefined
/// list. The label ties the target's held tag however it is set: a `\subsection*` prints its numbered section's name
/// (CrossRef's walk up), a longtable's label is its `labels=` ("Таблица": `\cyrt` `\cyra` `\cyrb` `\cyrl`). Witness
/// biblatex-gost-examples (4 → 0 errors). Repro singletons/autoref_name_evaluated_at_every_target.
#[test]
fn autoref_name_diagnostics_wait_for_an_autoref() {
  const LUATEX: &str = "[rawstyles,rawclasses,luatex]latexml.sty";
  let convert = |body: &str| {
    super::convert_with(
      &format!(
        r"\documentclass{{article}}
\usepackage{{fontspec}}
\usepackage[english,russian]{{babel}}
\usepackage{{longtable}}
\usepackage{{hyperref}}
\begin{{document}}
\section{{A}}\label{{a}}Text.
{body}
\end{{document}}
"
      ),
      Some(LUATEX),
    )
  };
  let undefined = |stderr: &str| -> Vec<String> {
    let mut names: Vec<String> = stderr
      .lines()
      .filter_map(|line| line.strip_prefix("Error:undefined:"))
      .map(|rest| rest.split_whitespace().next().unwrap_or("").to_string())
      .collect();
    names.sort();
    names
  };
  let section = ["\\cyra", "\\cyrd", "\\cyrr", "\\cyrz"];
  for body in ["", "See \\ref{a}."] {
    let (stderr, _) = convert(body);
    assert_eq!(
      (error_count(&stderr), warning_count(&stderr)),
      (0, 0),
      "{body}: {stderr}"
    );
  }
  for body in [
    "\\section{B}Text. \\autoref{a} and \\autoref{a}.",
    "\\section{B}\\label{b}Text. \\autoref{a} and \\autoref{b}.",
    "\\subsection*{S}\\label{s}Text. \\autoref{s}.",
  ] {
    let (stderr, _) = convert(body);
    assert_eq!(undefined(&stderr), section, "{body}: {stderr}");
    assert_eq!(error_count(&stderr), 4, "{body}: {stderr}");
  }
  // The replayed names reach the status summary's undefined list (the CLI's "Conversion complete" line).
  let (_, _, status, _) = super::perfect_kernel_batch46::convert_with_status(
    "\\documentclass{article}\\usepackage{fontspec}\\usepackage[english,russian]{babel}\\usepackage{hyperref}\n\\begin{document}\\section{A}\\label{a}Text. \\autoref{a}.\\end{document}\n",
    Some(LUATEX),
  );
  let mut names: Vec<&str> = status
    .strip_prefix("4 errors; 4 undefined macros[")
    .and_then(|rest| rest.strip_suffix(']'))
    .unwrap_or_else(|| panic!("{status}"))
    .split(", ")
    .collect();
  names.sort();
  assert_eq!(names, section, "{status}");
  let (stderr, xml) = convert("\\section{B}Text. \\autoref{a}.");
  assert_eq!(undefined(&stderr), section, "{stderr}");
  assert_element(
    &xml,
    "tags",
    &[],
    r##"<tags><tag>1</tag><tag role="autoref"><ERROR class="undefined">\cyrr</ERROR><ERROR class="undefined">\cyra</ERROR><ERROR class="undefined">\cyrz</ERROR><ERROR class="undefined">\cyrd</ERROR>. 1</tag><tag role="refnum">1</tag><tag role="typerefnum">§1</tag></tags>"##,
  );
  let (stderr, _) = convert(
    "\\begin{longtable}{l}\\caption{T}\\label{tab:lt}\\\\ x\\end{longtable} See \\autoref{tab:lt}.",
  );
  assert_eq!(
    undefined(&stderr),
    ["\\cyra", "\\cyrb", "\\cyrl", "\\cyrt"],
    "{stderr}"
  );
  assert_eq!(error_count(&stderr), 4, "{stderr}");
}

/// 60r: fontspec's text encoding is TU (fontspec-xetex.sty:431-441), so `<`, `>` and `|` print as typed and the TeX
/// ligatures give `"` → ” outside typewriter, as xelatex prints them; under the pdfTeX persona the engine probe
/// `\XeTeXrevision` stays undefined. Repro fonts-nfss/fontspec_text_is_tu_encoded.
#[test]
fn fontspec_text_is_tu_encoded() {
  let (stderr, xml) = latexml::util::test::convert_with(
    include_str!("../../../tools/perfect_kernel/repros/fonts-nfss/fontspec_text_is_tu_encoded.tex"),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "p",
    &[],
    "<p>Arrow &lt;— here and a&gt;b, x|y, ”q”. <text font=\"typewriter\">tt &lt;a&gt; \"q\" `x'</text>\nREV-UNDEFINED.</p>",
  );
}

/// 60r: a footnote's font reset keeps the document's text encoding (`\reset@font`, latex.ltx:17659, selects
/// `\encodingdefault`, :14113-14122), and an `\item[…]` label — LaTeXML's tag neutralization; in LaTeX it inherits the
/// body's font — too: a T1 document's `<b> x|y` is not set through OT1 (`¡b¿ x—y`). Repro
/// fonts-nfss/normalfont_note_keeps_the_text_encoding.
#[test]
fn normalfont_note_keeps_the_text_encoding() {
  let (stderr, xml) = latexml::util::test::convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/fonts-nfss/normalfont_note_keeps_the_text_encoding.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "note",
    &[],
    r#"<note mark="1" role="footnote" xml:id="footnote1"><tags><tag>1</tag><tag role="refnum">1</tag><tag role="typerefnum">footnote 1</tag></tags>Note &lt;b&gt; x|y.</note>"#,
  );
  assert_element(
    &xml,
    "item",
    &[r#"xml:id="S0.I1.ix1""#],
    r#"<item xml:id="S0.I1.ix1"><tags><tag>&lt;i&gt;</tag><tag role="typerefnum">item &lt;i&gt;</tag></tags><para xml:id="S0.I1.ix1.p1"><p>item</p></para></item>"#,
  );
}

/// 60r: fontspec selects TU whatever encoding the document chose before it (fontspec-xetex.sty:441
/// `\RequirePackage[TU]{fontenc}`), under the pdfTeX persona and the luatex profile alike: after `[T1]{fontenc}`, `"`
/// is TU's ” (T1's is straight) in the body and in a footnote.
#[test]
fn fontspec_selects_tu_after_a_t1_fontenc() {
  for preload in [None, Some("[rawstyles,rawclasses,luatex]latexml.sty")] {
    let (stderr, xml) = latexml::util::test::convert_with(
      "\\documentclass{article}\n\\usepackage[T1]{fontenc}\n\\usepackage{fontspec}\n\\begin{document}\nSay \"q\" <a>.\\footnote{Note \"n\" <b>.}\n\\end{document}\n",
      preload,
    );
    assert_eq!(error_count(&stderr), 0, "{preload:?}: {stderr}");
    assert_eq!(warning_count(&stderr), 0, "{preload:?}: {stderr}");
    assert_element(
      &xml,
      "p",
      &[],
      r#"<p>Say ”q” &lt;a&gt;.<note mark="1" role="footnote" xml:id="footnote1"><tags><tag>1</tag><tag role="refnum">1</tag><tag role="typerefnum">footnote 1</tag></tags>Note ”n” &lt;b&gt;.</note></p>"#,
    );
  }
}

/// 60r: the reset font's encoding is `\encodingdefault` EXPANDED, as `\fontencoding\encodingdefault` expands it: babel
/// defines it as `\latinencoding` (babel.sty:3968-3970), greek.ldf as `\greekfontencoding` (:151-153). A footnote under
/// an `\encodingdefault` that names LGR through a macro is Greek; the unexpanded name (no fontmap) and an OT1 fallback
/// both print `abg`. The note's tags are LaTeXML's Latin text, in babel's `\latinencoding` (T1).
#[test]
fn normalfont_note_expands_the_encoding_default() {
  let (stderr, xml) = latexml::util::test::convert_with(
    "\\documentclass{article}\n\\usepackage[LGR,T1]{fontenc}\n\\usepackage[greek,english]{babel}\n\\makeatletter\\def\\lx@enc{LGR}\\def\\encodingdefault{\\lx@enc}\\makeatother\n\\begin{document}\nBody.\\footnote{abg}\n\\end{document}\n",
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "note",
    &[],
    r#"<note mark="1" role="footnote" xml:id="footnote1"><tags><tag>1</tag><tag role="refnum">1</tag><tag role="typerefnum">footnote 1</tag></tags>αβγ</note>"#,
  );
}

/// 60r: identifiers are read from their source tokens, not typeset: inside `\selectlanguage{greek}` the text
/// encoding is LGR (greek.ldf:151-153 `\greekscript` sets `\encodingdefault`), and a tag's role, a declaration's
/// role and meaning, digested in it, read back in Greek letters (`refnum` → `ρεφνυμ`, `ADDOP` → `ΑΔΔΟΠ`). Witness
/// greek-fontenc alphabeta-doc (69 roles). Repro fonts-nfss/identifiers_are_not_font_decoded.
#[test]
fn identifiers_are_not_font_decoded() {
  let (stderr, xml) = latexml::util::test::convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/fonts-nfss/identifiers_are_not_font_decoded.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "tags",
    &[],
    r#"<tags><tag>1</tag><tag role="refnum">1</tag><tag role="typerefnum">§1</tag></tags>"#,
  );
  assert_element(
    &xml,
    "XMTok",
    &[r#"name="star""#],
    r#"<XMTok meaning="plus" name="star" role="ADDOP">⋆</XMTok>"#,
  );
}

/// 60s: the TeX ligatures follow the encoding's fonts (pdflatex: cmr, ec, LH): `` '' → “ ” in all; ,, << >> → „ « »
/// in T1, T2A and LY1, not OT1 (its `<` `>` slots are ¡ ¿); !` → ¡ in OT1, T1 and LY1, not T2A. Repro
/// fonts-nfss/text_ligatures_follow_the_encoding.
#[test]
fn text_ligatures_follow_the_encoding() {
  let (stderr, xml) = latexml::util::test::convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/fonts-nfss/text_ligatures_follow_the_encoding.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  for (id, expected) in [
    (
      "p1",
      r#"<para xml:id="p1"><p>“q” a–b ¡¡g¿¿ ,,l. ¡x</p></para>"#,
    ),
    (
      "p2",
      r#"<para xml:id="p2"><p>“q” a–b «g» „l. ¡x</p></para>"#,
    ),
    (
      "p3",
      r#"<para xml:id="p3"><p>“q” a–b «g» „l. !‘x</p></para>"#,
    ),
    (
      "p4",
      r#"<para xml:id="p4"><p>“q” a–b «g» „l. ¡x</p></para>"#,
    ),
  ] {
    assert_element(&xml, "para", &[&format!(r#"xml:id="{id}""#)], expected);
  }
}

/// 60s: a paragraph's text before a `\footnote` is ligatured: moving the insertion point closes the open text node,
/// so its ligatures run (Perl Document.pm:74-76 `setNode`); it kept `‘‘a’’ b--c` (Perl and pdflatex “a” b–c). Repro
/// fonts-nfss/text_before_a_footnote_is_ligatured.
#[test]
fn text_before_a_footnote_is_ligatured() {
  let (stderr, xml) = latexml::util::test::convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/fonts-nfss/text_before_a_footnote_is_ligatured.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "p",
    &[],
    r#"<p>Body “a” b–c.<note mark="1" role="footnote" xml:id="footnote1"><tags><tag>1</tag><tag role="refnum">1</tag><tag role="typerefnum">footnote 1</tag></tags>n “x” y–z.</note> After e–f.</p>"#,
  );
}

/// 60s: a listing's code forms no ligature (listings' fixed columns box each character): `cout << x`, `s -- t` print as
/// typed in the listing and in `\lstinline`, while prose keeps its « (pdflatex). Repro
/// fonts-nfss/listing_code_is_not_ligatured.
#[test]
fn listing_code_is_not_ligatured() {
  let (stderr, xml) = latexml::util::test::convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/fonts-nfss/listing_code_is_not_ligatured.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "listingline",
    &[],
    r#"<listingline xml:id="lstnumberx1"><text class="ltx_lst_identifier">cout</text><text class="ltx_lst_space"> </text>&lt;&lt;<text class="ltx_lst_space"> </text><text class="ltx_lst_identifier">x</text>;<text class="ltx_lst_space"> </text><text class="ltx_lst_identifier">a</text><text class="ltx_lst_space"> </text>&gt;&gt;<text class="ltx_lst_space"> </text><text class="ltx_lst_identifier">f</text>;<text class="ltx_lst_space"> </text><text class="ltx_lst_identifier">s</text><text class="ltx_lst_space"> </text>--<text class="ltx_lst_space"> </text><text class="ltx_lst_identifier">t</text><text class="ltx_lst_space"> </text>,,<text class="ltx_lst_identifier">u</text></listingline>"#,
  );
  assert_element(
    &xml,
    "p",
    &[],
    r#"<p>Inline <text class="ltx_lstlisting"><text class="ltx_lst_identifier">a</text>&lt;&lt;<text class="ltx_lst_identifier">b</text>--<text class="ltx_lst_identifier">c</text></text> and prose a«b.</p>"#,
  );
}

/// 60s: a `\label` that moves the insertion point away from the open text and back leaves the earlier run ligated by
/// its own font: T2A's `!`` stays `!‘` before a T1 `x` (KNOWN_PERL_ERRORS #438). Repro
/// fonts-nfss/ligatures_stay_in_their_run_across_a_label.
#[test]
fn ligatures_stay_in_their_run_across_a_label() {
  let (stderr, xml) = latexml::util::test::convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/fonts-nfss/ligatures_stay_in_their_run_across_a_label.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "para",
    &[],
    r#"<para xml:id="S1.p1"><p>a!‘x</p></para>"#,
  );
}

/// 60r: in a Greek-main document a caption's label keeps babel greek's name in the document's encoding (Πίνακας), while
/// LaTeXML's own English reference name is Latin (`footnote 1`, babel's `\latinencoding`). Repro
/// fonts-nfss/greek_tags_keep_localized_names.
#[test]
fn greek_tags_keep_localized_names() {
  let (stderr, xml) = latexml::util::test::convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/fonts-nfss/greek_tags_keep_localized_names.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  // `ί` is U+1F77 (with oxia) where pdflatex extracts the canonically equivalent U+03AF (with tonos): RED
  // babel-lang/greek_tonos_is_the_nfc_letter.
  assert_element(
    &xml,
    "caption",
    &[],
    "<caption><tag close=\": \">\u{03A0}\u{1F77}\u{03BD}\u{03B1}\u{03BA}\u{03B1}\u{03C2} 1</tag>Κατι</caption>",
  );
  assert_element(
    &xml,
    "note",
    &[],
    r#"<note mark="1" role="footnote" xml:id="footnote1"><tags><tag>1</tag><tag role="refnum">1</tag><tag role="typerefnum">footnote 1</tag></tags>σ</note>"#,
  );
}

/// 60t: babel greek's `~` is the perispomeni shorthand only in POLYTONIC Greek (greek.ldf:576-578); monotonic Greek
/// keeps the tie, and leaving Greek restores it (`\noextrasgreek`, greek.ldf:150-170) — babel's own machinery, with no
/// copy in the binding. pdflatex: monotonic "γ ς δ" / "a b", polytonic "γ ς῀δ" / "a b". Under babel a tie is its system
/// shorthand `\leavevmode\nobreak\ ` (babel.sty:1504), a space (Perl 0.8.8 alike). Repro
/// babel-lang/greek_tilde_restored_after_greek.
#[test]
fn greek_tilde_is_polytonic_and_restored() {
  for (option, greek) in [("greek", "γ ς δ"), ("greek.polutoniko", "γ ς\u{1FC0}δ")] {
    let (stderr, xml) = latexml::util::test::convert_with(
      &format!(
        "\\documentclass{{article}}\n\\usepackage[LGR,T1]{{fontenc}}\n\\usepackage[{option},english]{{babel}}\n\\begin{{document}}\ne1 a~b\n\\selectlanguage{{greek}}\ng c~d\n\\selectlanguage{{english}}\ne2 a~b\n\\end{{document}}\n"
      ),
      None,
    );
    assert_eq!(error_count(&stderr), 0, "{option}: {stderr}");
    assert_eq!(warning_count(&stderr), 0, "{option}: {stderr}");
    assert_element(
      &xml,
      "p",
      &[],
      &format!("<p>e1 a b<text xml:lang=\"el\">{greek}</text>e2 a b</p>"),
    );
  }
}
