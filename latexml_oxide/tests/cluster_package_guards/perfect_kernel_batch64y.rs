//! Red/green guards for batch 64y: an environment's frame met by a plain group closer. LaTeX's `\begin` is
//! `\begingroup` + `\X` and a bare `\X` opens no group at all (latex.ltx:15346-15355), so `{\thm …}`, `{\quote …}`
//! and `\begin{flushleft} … \endgroup` compile cleanly; LaTeXML keeps a mode frame for the environment's body, which
//! the plain closer could not pop ("Attempt to close a group that switched to mode …", Perl too), and the body
//! swallowed the rest of the document. The closer now ends the environment first, in tex.web §1064 `off_save`'s
//! shape (stomach.rs `close_environment_frame`): a bare `\X` gets its `\endX` inserted before the re-read closer, a
//! `\begin{X}` frame its `\end{X}` in place of the `\endgroup`. What TeX reports decides the diagnostic, by the kind of
//! group the environment's begin code leaves open (`TexGroup`): none — an Info; a box group (a float, a minipage) —
//! TeX's Error for either closer; a `\begingroup` (semi_simple: setspace, lscape, multicols, beamer blocks) — an Error
//! for `}` (§1069) and for `\endgroup` against a `\begin{X}` frame, while an `\endgroup` against a bare `\X` only
//! unsaves that group (§1063): `\endX` takes its place, the enclosing group stays open, an Info. A `}` against a
//! `\begin{X}` frame stays TeX's error (§1068-1069 `extra_right_brace`), and end code TeX never runs (a theorem's end
//! mark, `\end`'s hooks) is skipped. Repros `tools/perfect_kernel/repros/boxes-groups/`.
use latexml::util::test::assert_element;

use super::perfect_kernel_batch46::{error_count, warning_count};

/// The XML of `tex` converted under ar5iv, which must give no error and no warning.
fn convert_clean(tex: &str) -> String {
  let (log, xml) = latexml::util::test::convert_with(tex, Some("ar5iv.sty"));
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  xml
}

/// `{\thm …}` (1210.5866, 1111.2539, 2405.03639, 1703.03321, 2202.11643, 0812.1147, 1009.2722, 1312.6689,
/// 1905.06220, 2207.12734, 2308.09714, 2503.00772, 1403.7312, 2104.03038): the bare theorem ends at the brace, as its
/// `\begin{thm}…\end{thm}` control does, and the text after the brace is a paragraph of the document, not of the
/// theorem.
#[test]
fn bare_theorem_closed_by_brace() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/boxes-groups/bare_theorem_closed_by_brace.tex"
  ));
  assert_eq!(xml.matches("<theorem ").count(), 2, "{xml}");
  assert_element(
    &xml,
    "theorem",
    &[r#"xml:id="Thmthm2""#],
    r#"<theorem class="ltx_theorem_thm" inlist="thm theorem:thm" xml:id="Thmthm2">
      <tags>
        <tag>Theorem 2</tag>
        <tag role="refnum">2</tag>
        <tag role="typerefnum">Theorem 2</tag>
      </tags>
      <title class="ltx_runin"><tag><text font="bold" xml:id="Thmthm2.1">Theorem 2</text></tag></title>
      <para xml:id="Thmthm2.p1">
        <p xml:id="Thmthm2.p1.1"><text font="italic" xml:id="Thmthm2.p1.1.1">Bare theorem.</text></p>
      </para>
    </theorem>"#,
  );
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p2""#],
    r#"<para xml:id="p2"><p xml:id="p2.1">After two.</p></para>"#,
  );
}

/// `{\quote …}` (2412.14057): the quotation ends at the brace and the paragraph goes on after it.
#[test]
fn bare_quote_closed_by_brace() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/boxes-groups/bare_quote_closed_by_brace.tex"
  ));
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p2""#],
    r#"<para xml:id="p2">
      <quote xml:id="p2.1"><p xml:id="p2.1.1">Quoted words.</p></quote>
      <p xml:id="p2.2">After the quote.</p>
    </para>"#,
  );
}

/// A bare `\itemize` in braces inside an outer list's item: the inner list ends at the brace, the outer item's tail
/// stays in that item and the next `\item` is the outer list's.
#[test]
fn bare_itemize_nested_in_list() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/boxes-groups/bare_itemize_nested_in_list.tex"
  ));
  assert_eq!(xml.matches("<item ").count(), 3, "{xml}");
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="S0.I1.i1.p1""#],
    r#"<para xml:id="S0.I1.i1.p1">
      <p xml:id="S0.I1.i1.p1.1">Outer item.</p>
      <itemize xml:id="S0.I1.i1.I1">
        <item xml:id="S0.I1.i1.I1.i1">
          <tags>
            <tag>•</tag>
            <tag role="typerefnum">1st item</tag>
          </tags>
          <para xml:id="S0.I1.i1.I1.i1.p1"><p xml:id="S0.I1.i1.I1.i1.p1.1">Inner item.</p></para>
        </item>
      </itemize>
      <p xml:id="S0.I1.i1.p1.2">Outer tail.</p>
    </para>"#,
  );
  assert_element(
    &xml,
    "item",
    &[r#"xml:id="S0.I1.i2""#],
    &r#"<item xml:id="S0.I1.i2">
      <tags>
        <tag>2.</tag>
        <tag role="refnum">2</tag>
        <tag role="typerefnum">item~2</tag>
      </tags>
      <para xml:id="S0.I1.i2.p1"><p xml:id="S0.I1.i2.p1.1">Second outer.</p></para>
    </item>"#
      .replace('~', "\u{a0}"),
  );
}

/// `\begingroup\thm …\endgroup` (the `\begin{samepage}\rem …\end{samepage}` shape): the theorem ends before the
/// `\endgroup` closes the `\begingroup`.
#[test]
fn bare_theorem_closed_by_endgroup() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/boxes-groups/bare_theorem_closed_by_endgroup.tex"
  ));
  assert_element(
    &xml,
    "theorem",
    &[r#"xml:id="Thmthm1""#],
    r#"<theorem class="ltx_theorem_thm" inlist="thm theorem:thm" xml:id="Thmthm1">
      <tags>
        <tag>Theorem 1</tag>
        <tag role="refnum">1</tag>
        <tag role="typerefnum">Theorem 1</tag>
      </tags>
      <title class="ltx_runin"><tag><text font="bold" xml:id="Thmthm1.1">Theorem 1</text></tag></title>
      <para xml:id="Thmthm1.p1">
        <p xml:id="Thmthm1.p1.1"><text font="italic" xml:id="Thmthm1.p1.1.1">Bare theorem.</text></p>
      </para>
    </theorem>"#,
  );
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r#"<para xml:id="p1"><p xml:id="p1.1">After words.</p></para>"#,
  );
}

/// `\begin{flushleft} … \endgroup` (the ALICE author block, 1702.01612 and 1909.03158): the `\endgroup` closes the
/// environment's group, so the environment ends there and the words after it are not left-aligned.
#[test]
fn begin_flushleft_closed_by_endgroup() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/boxes-groups/begin_flushleft_closed_by_endgroup.tex"
  ));
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r#"<para xml:id="p1"><p align="left" xml:id="p1.1">Left words.</p></para>"#,
  );
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p2""#],
    r#"<para xml:id="p2"><p xml:id="p2.1">Following words.</p></para>"#,
  );
}

/// A `}` against `\begin{center}`'s group is TeX's "Extra }, or forgotten \endgroup" (tex.web §1068-1069): one error, the
/// brace dropped, the environment still ended by its `\end{center}`.
#[test]
fn begin_center_closed_by_brace_stays_an_error() {
  let (log, xml) = latexml::util::test::convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/boxes-groups/begin_center_closed_by_brace.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!(error_count(&log), 1, "{log}");
  assert!(
    log.contains("Error:unexpected:} Attempt to close a group that switched to mode horizontal"),
    "{log}"
  );
  assert_eq!(warning_count(&log), 0, "{log}");
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r#"<para xml:id="p1"><p align="center" xml:id="p1.1">Centered  words.</p></para>"#,
  );
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p2""#],
    r#"<para xml:id="p2"><p xml:id="p2.1">After.</p></para>"#,
  );
}

/// `(log, xml)` of `tex` converted under ar5iv.
fn convert(tex: &str) -> (String, String) {
  latexml::util::test::convert_with(tex, Some("ar5iv.sty"))
}

/// `\begin{minipage}{3cm} … \endgroup`: the `\endgroup` meets the minipage's own `\vbox\bgroup` in TeX (latex.ltx:16310,
/// §1064 "Missing } inserted"; pdflatex 6 errors), so the recovery reports TeX's Error; the minipage still ends there.
#[test]
fn minipage_closed_by_endgroup_is_an_error() {
  let (log, xml) = convert(include_str!(
    "../../../tools/perfect_kernel/repros/boxes-groups/minipage_closed_by_endgroup.tex"
  ));
  assert_eq!(error_count(&log), 1, "{log}");
  assert!(
    log.contains(
      "Error:unexpected:\\endgroup Environment \\begin{minipage} opened a TeX group of its own"
    ),
    "{log}"
  );
  assert_eq!(warning_count(&log), 0, "{log}");
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p2""#],
    r#"<para xml:id="p2"><p xml:id="p2.1"><inline-block class="ltx_minipage" vattach="middle" width="85.4pt" xml:id="p2.1.1">
      <p xml:id="p2.1.1.1">Mini text.</p></inline-block>After mini.</p></para>"#,
  );
}

/// `{\minipage{3cm} …}`: TeX's `}` ends the minipage's box group, not the brace group (pdflatex 4 errors): an Error.
#[test]
fn bare_minipage_closed_by_brace_is_an_error() {
  let (log, xml) = convert(include_str!(
    "../../../tools/perfect_kernel/repros/boxes-groups/bare_minipage_closed_by_brace.tex"
  ));
  assert_eq!(error_count(&log), 1, "{log}");
  assert!(
    log.contains("Error:unexpected:} Environment \\minipage opened a TeX group of its own"),
    "{log}"
  );
  assert_eq!(warning_count(&log), 0, "{log}");
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p3""#],
    r#"<para xml:id="p3"><p xml:id="p3.1">Next para.</p></para>"#,
  );
}

/// `\begin{figure} … \endgroup`: `\@xfloat`'s `\color@vbox` is still
/// open in TeX (latex.ltx:17479; pdflatex 7 errors): an Error, and the figure ends before the following text.
#[test]
fn figure_closed_by_endgroup_is_an_error() {
  let (log, xml) = convert(include_str!(
    "../../../tools/perfect_kernel/repros/boxes-groups/figure_closed_by_endgroup.tex"
  ));
  assert_eq!(error_count(&log), 1, "{log}");
  assert!(
    log.contains(
      "Error:unexpected:\\endgroup Environment \\begin{figure} opened a TeX group of its own"
    ),
    "{log}"
  );
  assert_eq!(warning_count(&log), 0, "{log}");
  assert_eq!(xml.matches("<figure ").count(), 1, "{xml}");
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p2""#],
    r#"<para xml:id="p2"><p xml:id="p2.1">After fig.</p></para>"#,
  );
}

/// `\begin{foo}\begin{center} … \end{foo}` with a macro-defined `foo`: LaTeX's `\@checkend` (latex.ltx:15389, :15394)
/// reports "\begin{center} … ended by \end{foo}" (pdflatex 2 errors); `foo`'s group stays open, as in TeX (the
/// Warning at `\end{document}`).
#[test]
fn raw_end_against_inner_environment_is_an_error() {
  let (log, xml) = convert(include_str!(
    "../../../tools/perfect_kernel/repros/boxes-groups/raw_end_against_inner_environment.tex"
  ));
  assert_eq!(error_count(&log), 1, "{log}");
  assert!(
    log
      .contains("Error:unexpected:\\end{foo} \\begin{center} on input line 11 ended by \\end{foo}"),
    "{log}"
  );
  assert_eq!(warning_count(&log), 1, "{log}");
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p3""#],
    r#"<para xml:id="p3"><p xml:id="p3.1">After.</p></para>"#,
  );
}

/// ntheorem `[thmmarks]`: `{\Proof …}` never reaches `\endProof` in TeX, so no end mark is set (pdflatex); the
/// delimited proof keeps its ■.
#[test]
fn ntheorem_bare_proof_has_no_endmark() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/boxes-groups/ntheorem_bare_proof_has_no_endmark.tex"
  ));
  assert_element(
    &xml,
    "theorem",
    &[r#"xml:id="ThmProofx1""#],
    r#"<theorem class="ltx_theorem_Proof" inlist="thm theorem:Proof" xml:id="ThmProofx1">
      <tags><tag>Proof</tag><tag role="typerefnum">Proof</tag></tags>
      <title class="ltx_runin"><tag><text font="bold" xml:id="ThmProofx1.1">Proof</text></tag><text font="bold" xml:id="ThmProofx1.2">.</text></title>
      <para xml:id="ThmProofx1.p1"><p xml:id="ThmProofx1.p1.1"><text font="italic" xml:id="ThmProofx1.p1.1.1">Bare proof text.</text></p></para>
    </theorem>"#,
  );
  assert_eq!(xml.matches("ltx_align_floatright").count(), 1, "{xml}");
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p2""#],
    r#"<para xml:id="p2"><p xml:id="p2.1">After bare.</p></para>"#,
  );
}

/// `{\thm …\let\endthm\relax}`: the recovery inserts `\endthm` once; it no longer closes the frame, so the closer meets
/// the frame again and gets the Error — exactly one, no loop.
#[test]
fn redefined_end_recovers_once() {
  let (log, _xml) = convert(include_str!(
    "../../../tools/perfect_kernel/repros/boxes-groups/bare_theorem_end_redefined_recovers_once.tex"
  ));
  assert_eq!(error_count(&log), 1, "{log}");
  assert!(
    log.contains("Error:unexpected:} Attempt to close a group that switched to mode horizontal"),
    "{log}"
  );
}

/// `{\equation x=1}`: the closer meets a display-math frame (TeX: `$$` opened a `math_shift_group`, §1069; pdflatex 3
/// errors), which is not an environment recovery: the math off_save path and its two Errors stay.
#[test]
fn bare_equation_closed_by_brace_stays_an_error() {
  let (log, _xml) = convert(include_str!(
    "../../../tools/perfect_kernel/repros/boxes-groups/bare_equation_closed_by_brace.tex"
  ));
  assert_eq!(error_count(&log), 2, "{log}");
  assert!(
    log.contains("Error:unexpected:} Extra }, or forgotten $"),
    "{log}"
  );
  assert!(!log.contains("inserting \\endequation"), "{log}");
}

/// A bare `\quote` in a `p` cell, plain and with array's `>{\bgroup}…<{\egroup}` template closer: each quote ends in
/// its cell and the text after the tabular stays outside it.
#[test]
fn bare_quote_in_template_closed_cell() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/boxes-groups/bare_quote_in_template_closed_cell.tex"
  ));
  assert_element(
    &xml,
    "td",
    &[r#"xml:id="p2.1.1.1""#],
    r#"<td align="left" vattach="top" xml:id="p2.1.1.1"><inline-block vattach="top" width="85.4pt" xml:id="p2.1.1.1.1">
      <quote xml:id="p2.1.1.1.1.1"><p xml:id="p2.1.1.1.1.1.1">c</p></quote></inline-block></td>"#,
  );
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="p2.2""#],
    r#"<p xml:id="p2.2">After.</p>"#,
  );
}

/// `\let\x=}` and `\egroup` close a bare theorem like `}` does.
#[test]
fn bare_theorem_closed_by_let_brace() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/boxes-groups/bare_theorem_closed_by_let_brace.tex"
  ));
  assert_eq!(xml.matches("<theorem ").count(), 2, "{xml}");
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p2""#],
    r#"<para xml:id="p2"><p xml:id="p2.1">After one.</p></para>"#,
  );
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p3""#],
    r#"<para xml:id="p3"><p xml:id="p3.1">After two.</p></para>"#,
  );
}

/// `\expandafter\endgroup\relax` ends a bare theorem in a `\begingroup` and a `\begin{center}`.
#[test]
fn endgroup_after_expandafter() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/boxes-groups/endgroup_after_expandafter.tex"
  ));
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p2""#],
    r#"<para xml:id="p2"><p xml:id="p2.1">After thm.</p></para>"#,
  );
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p3""#],
    r#"<para xml:id="p3"><p align="center" xml:id="p3.1">Cent</p></para>"#,
  );
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p4""#],
    r#"<para xml:id="p4"><p xml:id="p4.1">After cent.</p></para>"#,
  );
}

/// `\begin{foo}\thm …\end{foo}` with a macro-defined `foo`: `\@checkend{foo}` passes over the bare theorem (it opens
/// no group) and `\end{foo}`'s `\endgroup` ends the theorem, then foo's group.
#[test]
fn raw_end_around_bare_theorem() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/boxes-groups/raw_end_around_bare_theorem.tex"
  ));
  assert_eq!(xml.matches("<theorem ").count(), 1, "{xml}");
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p2""#],
    r#"<para xml:id="p2"><p xml:id="p2.1">After.</p></para>"#,
  );
}

/// A macro-defined float whose own end code closes `\@xfloat`'s box (`\@endfloatbox`, as floatrow's `\float@end`
/// does) but not the bound `\@float`'s frame: `\lx@endfloatbox@close` clears the frame's box-group marker where TeX
/// closes the box, so `\end`'s `\endgroup` that then meets the frame is clean in TeX: an Info, not the Error.
#[test]
fn raw_float_end_code_closes_its_box() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/boxes-groups/raw_float_end_code_closes_its_box.tex"
  ));
  assert_eq!(xml.matches("<float ").count(), 1, "{xml}");
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p2""#],
    r#"<para xml:id="p2"><p xml:id="p2.1">After.</p></para>"#,
  );
}

/// Two bare `\figure`s inside a macro-defined `refs`, met by `\end{refs}`'s `\endgroup`: each figure's box group is
/// still open in TeX (pdflatex 8 errors), so each recovery is an Error — exactly two, the same for both frames.
#[test]
fn raw_end_after_two_bare_figures() {
  let (log, xml) = convert(include_str!(
    "../../../tools/perfect_kernel/repros/boxes-groups/raw_end_after_two_bare_figures.tex"
  ));
  assert_eq!(error_count(&log), 2, "{log}");
  assert_eq!(
    log
      .matches("Environment \\figure opened a TeX group of its own")
      .count(),
    2,
    "{log}"
  );
  assert_eq!(warning_count(&log), 0, "{log}");
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p3""#],
    r#"<para xml:id="p3"><p xml:id="p3.1">After.</p></para>"#,
  );
}

/// One bare `\figure` left open inside a macro-defined `refs` (pdflatex 7 errors): an Error, not hidden because the
/// closer is `\end{refs}`'s own `\endgroup`.
#[test]
fn raw_end_after_bare_figure() {
  let (log, xml) = convert(include_str!(
    "../../../tools/perfect_kernel/repros/boxes-groups/raw_end_after_bare_figure.tex"
  ));
  assert_eq!(error_count(&log), 1, "{log}");
  assert!(
    log.contains("Environment \\figure opened a TeX group of its own"),
    "{log}"
  );
  assert_eq!(warning_count(&log), 0, "{log}");
  assert_element(
    &xml,
    "figure",
    &[r#"xml:id="fig1""#],
    r#"<figure xml:id="fig1"><p xml:id="fig1.1">One caption text.</p></figure>"#,
  );
}

/// Eighteen valid macro environments wrapping bound ones (`\begin`/bare forms, nested, `\NewEnviron`, `\let` copies,
/// etoolbox hooks): `\lx@checkend` reports nothing (pdflatex 0).
#[test]
fn checkend_silent_on_valid_wrappers() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/boxes-groups/checkend_silent_on_valid_wrappers.tex"
  ));
  assert_element(
    &xml,
    "quote",
    &[r#"xml:id="p1.4""#],
    r#"<quote xml:id="p1.4"><p xml:id="p1.4.1"><text font="italic" xml:id="p1.4.1.1">quoted</text></p></quote>"#,
  );
}

/// `\end`'s hooks (etoolbox `\AtEndEnvironment`/`\AfterEndEnvironment`, lthooks `env/X/end`/`env/X/after`) run only in
/// `\end`: an environment ended by `\endgroup` or `}` runs none of them; the delimited control runs both.
#[test]
fn end_hooks_skipped_when_a_group_ends_the_environment() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/boxes-groups/end_hooks_skipped_when_a_group_ends_the_environment.tex"
  ));
  assert_eq!(xml.matches("[ATEND-").count(), 1, "{xml}");
  assert_eq!(xml.matches("[AFTEREND-").count(), 1, "{xml}");
  assert!(
    !xml.contains("[HOOKEND-FR]") && !xml.contains("[HOOKAFTER-FR]"),
    "{xml}"
  );
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p3""#],
    r#"<para xml:id="p3"><p xml:id="p3.1">Following one.</p></para>"#,
  );
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p7""#],
    r#"<para xml:id="p7"><p align="left" xml:id="p7.1">Control. [ATEND-FL]</p></para>"#,
  );
}

/// rotating's `\sideways` boxes its body (rotating.sty:69-72 `\setbox\z@\color@hbox`): ended by `\endgroup`, an Error
/// (pdflatex 1).
#[test]
fn sideways_closed_by_endgroup_is_an_error() {
  let (log, xml) = convert(include_str!(
    "../../../tools/perfect_kernel/repros/boxes-groups/sideways_closed_by_endgroup.tex"
  ));
  assert_eq!(error_count(&log), 1, "{log}");
  assert!(
    log.contains("Environment \\sideways opened a TeX group of its own"),
    "{log}"
  );
  assert_eq!(warning_count(&log), 0, "{log}");
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="p1.1.1.1""#],
    r#"<p xml:id="p1.1.1.1">rotated text</p>"#,
  );
  assert!(xml.contains("</inline-block>After.</p>"), "{xml}");
}

/// framed's `\MakeFramed` opens a group (framed.sty:289-326): `\begin{framed} … \endgroup` is an Error (pdflatex 1).
#[test]
fn framed_closed_by_endgroup_is_an_error() {
  let (log, xml) = convert(include_str!(
    "../../../tools/perfect_kernel/repros/boxes-groups/framed_closed_by_endgroup.tex"
  ));
  assert_eq!(error_count(&log), 1, "{log}");
  assert!(
    log.contains("Environment \\begin{framed} opened a TeX group of its own"),
    "{log}"
  );
  assert_eq!(warning_count(&log), 0, "{log}");
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r##"<para xml:id="p1"><p xml:id="p1.1">Intro.</p>
      <p cssstyle="padding:9.0pt" framecolor="#000000" framed="rectangle" xml:id="p1.2">Framed text</p>
      <p xml:id="p1.3">After.</p></para>"##,
  );
}

/// REVTeX 3 without `floats`: `\figure` is a caption-list entry that opens no group (aps.sty:614-630), so a figure line
/// ended by `\endgroup` is clean, as with the REVTeX 3.1 files arXiv ships (cond-mat9607130).
#[test]
fn revtex3_figure_list_entry_closed_by_endgroup() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/boxes-groups/revtex3_figure_list_entry_closed_by_endgroup.tex"
  ));
  assert_element(
    &xml,
    "figure",
    &[r#"xml:id="fig1""#],
    r#"<figure xml:id="fig1"><p xml:id="fig1.1">1 The first figure caption.</p></figure>"#,
  );
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p2""#],
    r#"<para xml:id="p2"><p xml:id="p2.1">After.</p></para>"#,
  );
}

/// REVTeX 3 with `floats`: `\figure` is `\@float{figure}` (aps.sty:614-616) and the same `\endgroup` is TeX's error.
#[test]
fn revtex3_floats_figure_closed_by_endgroup_is_an_error() {
  let (log, xml) = convert(include_str!(
    "../../../tools/perfect_kernel/repros/boxes-groups/revtex3_floats_figure_closed_by_endgroup.tex"
  ));
  assert_eq!(error_count(&log), 1, "{log}");
  assert!(
    log.contains("Environment \\figure opened a TeX group of its own"),
    "{log}"
  );
  assert_eq!(warning_count(&log), 0, "{log}");
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p2""#],
    r#"<para xml:id="p2"><p xml:id="p2.1">After.</p></para>"#,
  );
}

/// natbib's `\bibpostamble` closes what `\bibpreamble` opened (natbib.sty:1063-1086): a `\begin{multicols}{2}`
/// preamble ends with the list (2009.08029), not at `\end{thebibliography}`'s `\endgroup`; the bibliography is whole
/// (the column change is pagination: both marks before the list, as in 64w).
#[test]
fn bibpostamble_closes_the_preamble() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/boxes-groups/bibpostamble_closes_the_preamble.tex"
  ));
  assert_element(
    &xml,
    "bibliography",
    &[r#"xml:id="bib""#],
    r#"<bibliography inlist="toc" xml:id="bib"><title>References</title>
      <pagination role="start_2_columns"/><pagination role="end_2_columns"/>
      <biblist>
        <bibitem key="a" xml:id="bib.bib1"><tags><tag role="number">1</tag><tag role="year">2000</tag>
          <tag role="authors">A</tag><tag role="refnum">A (2000)</tag><tag role="key">a</tag></tags>
          <bibblock> A. Author, A paper.
</bibblock></bibitem>
        <bibitem key="b" xml:id="bib.bib2"><tags><tag role="number">2</tag><tag role="year">2001</tag>
          <tag role="authors">B</tag><tag role="refnum">B (2001)</tag><tag role="key">b</tag></tags>
          <bibblock> B. Author, Another paper.
</bibblock></bibitem>
      </biblist></bibliography>"#,
  );
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p2""#],
    r#"<para xml:id="p2"><p xml:id="p2.1">After.</p></para>"#,
  );
}

/// natbib's `\bibpostamble` content is typeset after the last entry (natbib.sty:1079-1086): it ends the last bibblock.
#[test]
fn bibpostamble_content_ends_the_list() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/boxes-groups/bibpostamble_content_ends_the_list.tex"
  ));
  assert_element(
    &xml,
    "bibblock",
    &[],
    "<bibblock> A. Author, A paper.\nPostamble note.</bibblock>",
  );
}

/// `\begingroup\multicols{2} …\endgroup`: `\multicols` opens only a `\begingroup` (multicol.sty:199), which TeX's
/// `\endgroup` just unsaves (§1063): no error, the columns end before "After.", and the user's `\begingroup` stays
/// open (pdflatex 0 errors, "\end occurred inside a group at level 1").
#[test]
fn bare_multicols_closed_by_endgroup() {
  let (log, xml) = convert(include_str!(
    "../../../tools/perfect_kernel/repros/boxes-groups/bare_multicols_closed_by_endgroup.tex"
  ));
  assert_eq!(error_count(&log), 0, "{log}");
  // the enclosing `\begingroup` stays open to `\end{document}`, as in TeX ("\end occurred inside a group")
  assert_eq!(warning_count(&log), 1, "{log}");
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p2""#],
    r#"<para xml:id="p2"><p xml:id="p2.1">Text in columns.</p><pagination role="end_2_columns"/>
      <p xml:id="p2.2">After.</p></para>"#,
  );
}

/// `\begingroup\landscape …\endgroup`: `\landscape` opens only a `\begingroup` (lscape.sty:38-42), which the `\endgroup`
/// unsaves; the user's `\begingroup` stays open (pdflatex 0 errors).
#[test]
fn bare_landscape_closed_by_endgroup() {
  let (log, xml) = convert(include_str!(
    "../../../tools/perfect_kernel/repros/boxes-groups/bare_landscape_closed_by_endgroup.tex"
  ));
  assert_eq!(error_count(&log), 0, "{log}");
  // the enclosing `\begingroup` stays open to `\end{document}`, as in TeX
  assert_eq!(warning_count(&log), 1, "{log}");
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p2""#],
    r#"<para xml:id="p2"><p xml:id="p2.1">Landscape text.After.</p></para>"#,
  );
}

/// `\begin{doublespace} …\endgroup`: the `\endgroup` closes doublespace's `\begingroup` (setspace.sty:542-547) and
/// leaves `\begin`'s open, which TeX reports at the next `\end` (pdflatex 1): an Error.
#[test]
fn begin_doublespace_closed_by_endgroup_is_an_error() {
  let (log, xml) = convert(include_str!(
    "../../../tools/perfect_kernel/repros/boxes-groups/begin_doublespace_closed_by_endgroup.tex"
  ));
  assert_eq!(error_count(&log), 1, "{log}");
  // the user's `\begingroup` stays open to `\end{document}`, as in TeX
  assert_eq!(warning_count(&log), 1, "{log}");
  assert!(
    log.contains("Environment \\begin{doublespace} opened a TeX group of its own"),
    "{log}"
  );
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p2""#],
    r#"<para xml:id="p2"><p xml:id="p2.1">Text.</p><p xml:id="p2.2">After.</p></para>"#,
  );
}

/// `{\multicols{2} …\endgroup}`: the `\endgroup` unsaves `\multicols`'s `\begingroup` (§1063) and the `}` closes the
/// brace group: clean (pdflatex 0). Re-reading the `\endgroup` would have closed the brace group too.
#[test]
fn bare_multicols_closed_by_inner_endgroup() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/boxes-groups/bare_multicols_closed_by_inner_endgroup.tex"
  ));
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p2""#],
    r#"<para xml:id="p2"><p xml:id="p2.1">Text in columns.</p><pagination role="end_2_columns"/>
      <p xml:id="p2.2">After.</p></para>"#,
  );
}

/// `\begingroup\landscape …\endgroup\endgroup`: the first `\endgroup` unsaves `\landscape`'s group, the second the
/// `\begingroup` (pdflatex 0).
#[test]
fn bare_landscape_closed_by_two_endgroups() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/boxes-groups/bare_landscape_closed_by_two_endgroups.tex"
  ));
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p2""#],
    r#"<para xml:id="p2"><p xml:id="p2.1">Landscape text.After.</p></para>"#,
  );
}
