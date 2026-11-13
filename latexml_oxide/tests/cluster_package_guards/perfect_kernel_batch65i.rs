//! Red/green guards for perfect-kernel batch 65i, two regressions of run 340 and an older loader defect:
//! - a construct the document ended inside of (its group abandoned by `\end{document}`, 65c) lets its elements
//!   auto-close, so a section or a bibliography after the defect that left it open closes it instead of being held as
//!   `malformed:*` (OXIDIZED_DESIGN #542);
//! - the template reader starts a column-type rewrite with `\@temptokena` empty, as array.sty's `\NC@do` does, and
//!   rccol's `R` column has a binding (natbib + rccol: `Fatal:Timeout:PushbackLimit` since 64ze);
//! - `AddToMacro` in a package a standalone sub-file's preamble loads is local to that preamble's bracket (OXIDIZED_DESIGN
//!   #543).
//!
//! The 65c-class fixtures are ill-formed on purpose (pdflatex reports the open group), so each pins its exact
//! diagnostic counts.
use latexml::util::test::{assert_element, convert_with, rng_error_count, warning_count};

use super::perfect_kernel_batch46::error_count;

/// The (log, XML) of a repro converted under ar5iv, with exactly `errors` errors and `warnings` warnings, no Fatal, no
/// `malformed` placement, and schema-valid (when jing is installed).
fn convert_counted(tex: &str, errors: usize, warnings: usize) -> (String, String) {
  let (log, xml) = convert_with(tex, Some("ar5iv.sty"));
  assert_eq!(error_count(&log), errors, "{log}");
  assert_eq!(warning_count(&log), warnings, "{log}");
  assert!(!log.contains("Fatal:"), "{log}");
  assert!(!log.contains("Error:malformed"), "{log}");
  if let Some(invalid) = rng_error_count(&xml) {
    assert_eq!(invalid, 0, "schema errors:\n{xml}");
  }
  (log, xml)
}

/// 65i: two itemizes left open by a `\description` used as a command (1611.05714, revtex4-1): the document ended inside
/// the outer one, whose elements then auto-close when the bibliography arrives — a sibling of the lists, not
/// `malformed:ltx:bibliography` in the list's paragraph (65c kept the lists' content; Perl errs the same way, its
/// `ltx:itemize` being no autoClose element). Repro list-structure/open_description_holds_the_bibliography.
#[test]
fn open_list_gives_way_to_the_bibliography() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/list-structure/open_description_holds_the_bibliography.tex"
  );
  let (_log, xml) = convert_counted(tex, 0, 1);
  assert!(
    xml.contains("    </itemize>\n  </para>\n  <bibliography inlist=\"toc\" xml:id=\"bib\">"),
    "{xml}"
  );
  assert_element(
    &xml,
    "description",
    &["xml:id=\"S0.I1.i1.I1.i0.I1\""],
    "<description xml:id=\"S0.I1.i1.I1.i0.I1\"><item xml:id=\"S0.I1.i1.I1.i0.I1.1\"><para xml:id=\"S0.I1.i1.I1.i0.I1.p1\"><p xml:id=\"S0.I1.i1.I1.i0.I1.p1.1\">Returns a value.</p></para></item></description>",
  );
  assert_eq!(xml.matches("<bibitem ").count(), 3, "{xml}");
}

/// 65i: a bare `\itemize` inside an itemize, ended by the `\end{itemize}` meant for the outer list (1805.06896,
/// 2304.12197, 2307.13461, 2511.04367, 0705.0894): the outer list never ends, so the unnumbered section closes it and
/// the section, the bibliography (its blank lines included) and the appendix follow the list as the document's
/// children — 65e's `ltx:appendix isn't allowed in ltx:inline-sectional-block` gone. 65e's `\par` rule for a
/// bibliography in a list that ends is `perfect_kernel_batch65e::par_in_item_bibliography_keeps_the_block`. Repros
/// list-structure/bare_itemize_end_leaves_appendix_in_block, list-structure/bib_blank_lines_in_open_list.
#[test]
fn open_list_gives_way_to_the_section() {
  let after_list =
    "        </item>\n      </itemize>\n    </para>\n  </section>\n  <section xml:id=\"Sx1\">";
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/list-structure/bare_itemize_end_leaves_appendix_in_block.tex"
  );
  let (_log, xml) = convert_counted(tex, 0, 1);
  assert!(xml.contains(after_list), "{xml}");
  assert!(
    xml.contains("  </bibliography>\n  <appendix inlist=\"toc\" xml:id=\"A1\">"),
    "{xml}"
  );
  assert!(!xml.contains("<inline-sectional-block"), "{xml}");
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/list-structure/bib_blank_lines_in_open_list.tex"
  );
  let (_log, xml) = convert_counted(tex, 0, 1);
  assert!(xml.contains(after_list), "{xml}");
  assert!(
    xml.contains("  </section>\n  <bibliography inlist=\"toc\" xml:id=\"bib\">"),
    "{xml}"
  );
  assert_eq!(xml.matches("<biblist").count(), 1, "{xml}");
  assert_eq!(xml.matches("<bibitem ").count(), 4, "{xml}");
}

/// 65i: a superscript group left open in a formula in an enumerate (2007.11971: pst-all's `\si` eats the `}`): the
/// section and the bibliography after it are document children, outside the list and the formula. Since 65n the `$`
/// inserts the `}` and ends the formula there (tex.web §1193 + §1064-1065 `off_save`, OXIDIZED_DESIGN #553) — before,
/// it deferred the math end to the script group's end, the document ended inside the script, the formula and the list,
/// and OXIDIZED_DESIGN #542's auto-close let the section close them. One error, = pdflatex ("Missing } inserted"), and
/// "hold." and "Text after." spaced text as in the PDF. Repro list-structure/open_math_script_holds_the_section.
#[test]
fn open_formula_gives_way_to_the_section() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/list-structure/open_math_script_holds_the_section.tex"
  );
  let (_log, xml) = convert_counted(tex, 1, 0);
  assert!(xml.contains("</Math> hold.</p>"), "{xml}");
  assert!(xml.contains(">Text after.</p>"), "{xml}");
  assert!(
    xml.contains("    </enumerate>\n  </para>\n  <section inlist=\"toc\" xml:id=\"S1\">"),
    "{xml}"
  );
  assert!(
    xml.contains("  </section>\n  <bibliography inlist=\"toc\" xml:id=\"bib\">"),
    "{xml}"
  );
  assert!(xml.contains(">After</title>"), "{xml}");
}

/// 65i: an `\fbox` left open around a figure, a second figure inside it (cs0009011, `Fatal:Document:Generic("add_child
/// encountered NULL pointer")` on 65c+): the framed `ltx:text`, `_noautoclose` in a box that ends, auto-closes when
/// the construct never ended, and its capture block closes with it; every word is kept. pdflatex stops at the open
/// `\fbox` ("File ended while scanning use of \fbox"). Pinned: two figures, each caption once, Figure 1 whole, the
/// text after the figures kept. Not pinned (the current shape; TeX gives no nesting to follow): Figure 1 nested inside
/// the float that becomes Figure 2, which takes the second figure's number and placement. Repro
/// boxes-groups/unended_fbox_holding_figures.
#[test]
fn unended_fbox_holding_figures_builds() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/boxes-groups/unended_fbox_holding_figures.tex"
  );
  let (log, xml) = convert_with(tex, Some("ar5iv.sty"));
  assert_eq!(error_count(&log), 5, "{log}");
  assert_eq!(warning_count(&log), 1, "{log}");
  assert!(!log.contains("Fatal:"), "{log}");
  assert!(!log.contains("Error:malformed"), "{log}");
  assert_eq!(xml.matches("<figure ").count(), 2, "{xml}");
  assert_eq!(xml.matches("<caption>").count(), 2, "{xml}");
  assert_element(
    &xml,
    "figure",
    &["xml:id=\"S0.F1\""],
    "<figure class=\"ltx_minipage\" inlist=\"lof\" vattach=\"middle\" width=\"369.9pt\" xml:id=\"S0.F1\"><tags><tag>Figure 1</tag><tag role=\"refnum\">1</tag><tag role=\"typerefnum\">Figure 1</tag></tags><toccaption><tag close=\" \">1</tag>First</toccaption><caption><tag close=\": \">Figure 1</tag>First</caption></figure>",
  );
  assert!(
    xml.contains("<caption><tag close=\": \">Figure 2</tag>Second</caption>"),
    "{xml}"
  );
  assert!(xml.contains(">Heuristic rules.</p>"), "{xml}");
}

/// 65i: a column type whose rewrite reads `\@temptokena` back (array.sty's `*` does, rccol.sty:63-68) finds it empty,
/// as array.sty's `\NC@do` leaves it (array.sty:361), not holding natbib's `\appdef` leftover (natbib.sty:163-166),
/// whose `\@bibibid@` ate the cell. The three warnings name what the reader does not run — a known gap named in
/// SYNC_STATUS's 65i bullet (the reader expands a column type, it does not run its assignments; no pdflatex-valid
/// rewrite reading `\@temptokena` converts without them). Repro kernel-alignment/column_rewrite_reads_temptokena_empty.
#[test]
fn column_rewrite_reads_temptokena_empty() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/kernel-alignment/column_rewrite_reads_temptokena_empty.tex"
  );
  let (log, xml) = convert_counted(tex, 0, 3);
  assert!(
    log.contains("Unrecognized tabular template \"\\\\NC@find\""),
    "{log}"
  );
  assert_element(
    &xml,
    "tr",
    &["xml:id=\"p1.1.1\""],
    "<tr xml:id=\"p1.1.1\"><td align=\"left\" xml:id=\"p1.1.1.1\">a</td><td align=\"center\" xml:id=\"p1.1.1.2\">b</td></tr>",
  );
}

/// 65i: rccol's `R{1}{2}` with natbib loaded (the 18 run-340 `Fatal:Timeout:PushbackLimit` papers, 1101.1265): the
/// binding's `\NC@rewrite@R` is the result of rccol's `\edef` rewrite, and the raw cell code formats the number as
/// pdflatex does, "15,00." (the default decimal comma). Repro alignment-bindings/rccol_with_natbib_pushback_loop.
#[test]
fn rccol_column_formats_its_number() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/alignment-bindings/rccol_with_natbib_pushback_loop.tex"
  );
  let (_log, xml) = convert_counted(tex, 0, 0);
  assert_element(
    &xml,
    "tr",
    &["xml:id=\"p1.1.1\""],
    "<tr xml:id=\"p1.1.1\"><td align=\"left\" xml:id=\"p1.1.1.1\">a</td><td align=\"center\" xml:id=\"p1.1.1.2\">15,00.</td></tr>",
  );
}

/// 65i: a standalone sub-file's preamble loads xcolor (2512.09430): its tabular hooks are added in the preamble's
/// bracket, as its definitions are, so the main document's table after the `\input` calls no undefined
/// `\@xcolor@tabular@before` and is an alignment with its booktabs rules. Repro
/// loader/standalone_subfile_xcolor_tabular_hooks.
#[test]
fn standalone_subfile_package_hooks_stay_in_its_preamble() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/loader/standalone_subfile_xcolor_tabular_hooks.tex"
  );
  let (_log, xml) = convert_counted(tex, 0, 0);
  assert_element(
    &xml,
    "tr",
    &["xml:id=\"p1.1.1.1\""],
    "<tr xml:id=\"p1.1.1.1\"><td align=\"left\" border=\"bb tt\" xml:id=\"p1.1.1.1.1\">a</td><td align=\"left\" border=\"bb tt\" xml:id=\"p1.1.1.1.2\">b</td></tr>",
  );
}
