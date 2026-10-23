//! Red/green guards for batch 79d (equation numbers and reference text): an eqnarray continuation row that prints a
//! number of its own keeps it (OXIDIZED_DESIGN_DIVERGENCES #888); an IEEEeqnarray `\label` in a row left unnumbered
//! names the next number, as IEEEtrantools' `\@currentlabel` does (#847's rule), and its sub-equations follow the last
//! printed number; cleveref names a section of appendix.sty's `{appendices}` as a section unless cleveref's
//! `\appendix` has run, after which every sectioning unit is an appendix (KNOWN_PERL_ERRORS #1145); the Springer
//! `\vec` residuals of 79b (a text-mode `\vec` named in its warning, svmono's own class copy read). Round 3: a
//! `\nonumber` after `\IEEEyessubnumber` on a row wins, every retraction gives back the sub-equation number, and
//! IEEEojcsys's `\appendices` keeps the `section` type. Each guard converts
//! its pdflatex-checked repro under `tools/perfect_kernel/repros/` (its header holds the oracle) and asserts whole
//! elements and exact diagnostic counts.
use latexml::util::test::{assert_element, convert_files_with, convert_with};

use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch78r::{equation_heads, reference_tags, tags},
};

const PRELOAD: &str = "ar5iv.sty";

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!("../../../tools/perfect_kernel/repros/", $path))
  };
}

/// Convert `tex`: 0 errors, 0 warnings.
fn convert_clean(tex: &str) -> String {
  let (log, xml) = convert_with(tex, Some(PRELOAD));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  xml
}

/// latex.ltx:15795: `\@@eqncr` prints the number of every row it ends, so a continuation row with only its last column
/// (`&&+ 2,`) under a numbered row is an equation of its own, and a label names the row that states it (pdflatex (1)
/// (2), (3) (4) (5); "See 1, 3 and 4."). Red (int72a): the continuation joined the equation above with the last
/// number ("See 2, 3 and 5."). Witness 1310.2253 (`eqn:zg`, PDF 11, int72a 12). Repro
/// cross-references/eqnarray_numbered_continuation_row_keeps_its_number.
#[test]
fn eqnarray_numbered_continuation_row_keeps_its_number() {
  let xml = convert_clean(repro!(
    "cross-references/eqnarray_numbered_continuation_row_keeps_its_number.tex"
  ));
  assert_eq!(
    equation_heads(&xml),
    format!(
      "LABEL:zg@S0.E1:{}|@S0.E2:{}|LABEL:a1@S0.E3:{}|LABEL:a2@S0.E4:{}|@S0.E5:{}",
      tags(1),
      tags(2),
      tags(3),
      tags(4),
      tags(5)
    ),
    "{xml}"
  );
}

/// IEEEtrantools.sty:1986-1991, :2072-2117: the counter is pre-advanced for the next row and `\@currentlabel` points at
/// it, so a `\label` in a row `\nonumber`/`\IEEEnonumber` or the star leaves unnumbered names the next numbered row's
/// number, or the counter's next value when none follows (pdflatex "See 2, 3, 3 and 3."). Red (int72a): ie1 on its
/// unnumbered row ("1"), ie2 and ie3 raw ids. Repro cross-references/ieeeeqnarray_unnumbered_row_label_names_next_number.
#[test]
fn ieeeeqnarray_unnumbered_row_label_names_next_number() {
  let xml = convert_clean(repro!(
    "cross-references/ieeeeqnarray_unnumbered_row_label_names_next_number.tex"
  ));
  assert_eq!(
    equation_heads(&xml),
    format!(
      "@S0.E1:{}|@S0.Ex1:-|LABEL:ie1@S0.E2:{}|LABEL:ie2@S0.Ex2:{}|LABEL:ie3@S0.Ex3:{}|LABEL:ie4@S0.E3:{}",
      tags(1),
      tags(2),
      reference_tags(3),
      reference_tags(3),
      tags(3)
    ),
    "{xml}"
  );
}

/// An equation's tags printing `(number)`, for a sub-equation number ("1a").
fn printed(number: &str) -> String {
  format!("<tags><tag>({number})</tag><tag role=\"refnum\">{number}</tag></tags>")
}

/// IEEEtrantools.sty:1408-1424: `\IEEEyessubnumber` on a row whose number is a pre-advance backs it out, so the row is
/// the last printed number's next sub-equation; `\IEEEnosubnumber` (:1426-1437) makes a sub-equation row the next
/// equation; a `\label` in an `\IEEEnonumber` row names the pre-advanced number (:2086-2112), though the next row then
/// prints a sub-equation's (pdflatex "See 1, 1a, 1b, 2; 3a, 3b, 4, 4a; 5, 6, 5a, 6."). Red (79d r1): "1, 2a, 3a, 4; 5a,
/// 5b, 5, 5d; 6, 7a, 7a, 8". Witness 2012.02697 (`eq:XHp`, `eq:Q2`, PDF 11a, 11b). Repro
/// cross-references/ieeeeqnarray_sub_numbers_follow_the_last_number.
#[test]
fn ieeeeqnarray_sub_numbers_follow_the_last_number() {
  let xml = convert_clean(repro!(
    "cross-references/ieeeeqnarray_sub_numbers_follow_the_last_number.tex"
  ));
  assert_eq!(
    equation_heads(&xml),
    [
      ("s1a", "S0.E1", printed("1")),
      ("s1b", "S0.E1.E1", printed("1a")),
      ("s1c", "S0.E1.E2", printed("1b")),
      ("s1d", "S0.E2", printed("2")),
      ("s2a", "S0.E3", printed("3a")),
      ("s2b", "S0.E3.E2", printed("3b")),
      ("s2c", "S0.E3.E3", printed("4")),
      ("s2d", "S0.E4.E1", printed("4a")),
      ("s3a", "S0.E5", printed("5")),
      ("z1", "S0.Ex1", reference_tags(6)),
      ("z2", "S0.E5.E1", printed("5a")),
      ("s3d", "S0.E6", printed("6")),
    ]
    .map(|(label, id, tags)| format!("LABEL:{label}@{id}:{tags}"))
    .join("|"),
    "{xml}"
  );
}

/// `\nonumber` (latex.ltx) and `\IEEEnonumber` (IEEEtrantools.sty:1405) are `\global\@eqnswfalse`, so after an
/// `\IEEEyesnumber` or `\IEEEyessubnumber` on the row (:1393-1424, which set the counters and clear the pre-advance) the
/// last one wins: the row prints nothing, its counters stay, and the next row prints them (:2086-2112), or none follows
/// and they stay (:2022-2023, so "7" is skipped after the `p` block). pdflatex prints (1) (1a) (1b) (1c) (2) (3) (4) (5)
/// (5a) (6) (8) (8a) (8b) (9) (10). Red (79d r2): (1) (1a) (1b) (1c) (1d) (2) … (13), `\IEEEyessubnumber\IEEEnonumber`
/// after "(5)" printing "(5)" again (review rev79d2 finding 1, probes `eq/n.tex`, `eq/h.tex` block k). Witness 2012.02697
/// (`lcmpc.tex:123-136`, `eq:fpar`/`eq:par` 14a/14b as the `.aux`). Repro
/// cross-references/ieeeeqnarray_nonumber_after_yessubnumber_wins.
#[test]
fn ieeeeqnarray_nonumber_after_yessubnumber_wins() {
  let xml = convert_clean(repro!(
    "cross-references/ieeeeqnarray_nonumber_after_yessubnumber_wins.tex"
  ));
  assert_eq!(
    equation_heads(&xml),
    [
      ("LABEL:m0", "S0.E1", printed("1")),
      ("LABEL:m1", "S0.E1.E1", printed("1a")),
      ("LABEL:m2", "S0.E1.E2", printed("1b")),
      ("", "S0.E1.Ex1", "-".to_string()),
      ("LABEL:n2", "S0.E1.E3", printed("1c")),
      ("LABEL:n3", "S0.E2", printed("2")),
      ("LABEL:o1", "S0.E3", printed("3")),
      ("", "S0.Ex1", "-".to_string()),
      ("LABEL:o3", "S0.E4", printed("4")),
      ("LABEL:k1", "S0.E5", printed("5")),
      ("", "S0.Ex2", "-".to_string()),
      ("LABEL:k2 LABEL:k3", "S0.E5.E1", printed("5a")),
      ("LABEL:p1", "S0.E6", printed("6")),
      ("", "S0.Ex3", "-".to_string()),
      ("LABEL:q1", "S0.E8", printed("8")),
      ("", "S0.E8.Ex1", "-".to_string()),
      ("", "S0.E8.Ex2", "-".to_string()),
      ("LABEL:q3 LABEL:q4", "S0.E8.E1", printed("8a")),
      ("", "S0.E8.Ex1a", "-".to_string()),
      ("LABEL:r2", "S0.E8.E2", printed("8b")),
      ("LABEL:r3", "S0.E9", printed("9")),
      ("LABEL:t1", "S0.E10", printed("10")),
    ]
    .map(|(labels, id, tags)| format!("{labels}@{id}:{tags}"))
    .join("|"),
    "{xml}"
  );
}

/// IEEEtrantools.sty:1985-1987: each pre-advance of `equation` keeps the sub-equation number it resets
/// (`\@IEEEsubeqnnumrollback`), and a row left unnumbered steps nothing, so an `\IEEEyessubnumber` row after an
/// `IEEEeqnarray*` `\nonumber` row backs out to it (pdflatex "See s 0a, 0b, 0c; t 0d, 0e, 0f; u 0g, 0h, 0i."). Red (79d
/// r2): "t 0d, 0e, 0a; u 0b, 0c, 0d" — the starred environment's `\nonumber` retracted the row at once, dropping the
/// number to give back (review rev79d2 finding 2, probe `eq/s.tex`). Witness 1610.06458 (`paper.tex:1260-1266`,
/// `eq:inq3` 36c). Repro cross-references/ieeeeqnarray_starred_nonumber_gives_back_the_sub_number.
#[test]
fn ieeeeqnarray_starred_nonumber_gives_back_the_sub_number() {
  let xml = convert_clean(repro!(
    "cross-references/ieeeeqnarray_starred_nonumber_gives_back_the_sub_number.tex"
  ));
  assert_eq!(
    equation_heads(&xml),
    [
      ("LABEL:s1", "S0.E0.E1", printed("0a")),
      ("LABEL:s2", "S0.E0.E2", printed("0b")),
      ("LABEL:s3", "S0.E0.E3", printed("0c")),
      ("LABEL:t1", "S0.E0.E4", printed("0d")),
      ("LABEL:t2", "S0.E0.E5", printed("0e")),
      ("", "S0.Ex1", "-".to_string()),
      ("LABEL:t3", "S0.E0.E6", printed("0f")),
      ("LABEL:u1", "S0.E0.E7", printed("0g")),
      ("LABEL:u2", "S0.E0.E8", printed("0h")),
      ("LABEL:u3", "S0.E0.E9", printed("0i")),
    ]
    .map(|(labels, id, tags)| format!("{labels}@{id}:{tags}"))
    .join("|"),
    "{xml}"
  );
}

/// A sectioning unit's cleveref type forms under `\crefname{section}{Sec.}{Secs.}`.
fn sec_forms() -> &'static str {
  r#"<tag role="creftype">Sec.</tag><tag role="creftypecap">Sec.</tag><tag role="creftypeplural">Secs.</tag><tag role="creftypepluralcap">Secs.</tag>"#
}

/// cleveref.sty:174-287 aliases the sectioning counters to `appendix` only in its patched `\appendix`, which
/// appendix.sty's `{appendices}` never calls: a `\cref` to its sections prints the section's name (pdflatex "See Sec. A,
/// Sec. B and Sec. 1; Sec. A.1."). Red (int72a): "appendix A". An `\appendix`-opened appendix keeps the `appendix`
/// type. Witnesses 2507.16108, 2511.19260. Repro cross-references/cleveref_appendices_environment_keeps_section_type.
#[test]
fn cleveref_appendices_environment_keeps_section_type() {
  let tex = repro!("cross-references/cleveref_appendices_environment_keeps_section_type.tex");
  let xml = convert_clean(tex);
  assert_element(
    &xml,
    "appendix",
    &[r#"xml:id="A2""#],
    &format!(
      r#"<appendix inlist="toc" labels="LABEL:a:two" xml:id="A2"><tags><tag>Appendix B</tag>{}<tag role="refnum">B</tag><tag role="typerefnum">Appendix B</tag></tags><title><tag close=" ">Appendix B</tag>Two</title><toctitle><tag close=" ">B</tag>Two</toctitle></appendix>"#,
      sec_forms()
    ),
  );
  assert!(
    !xml.contains(r#"<tag role="creftype">appendix</tag>"#),
    "{xml}"
  );
  // A subsection inside it is a section to cleveref, as everywhere.
  assert_element(
    &xml,
    "subsection",
    &[r#"xml:id="A1.SS1""#],
    &format!(
      r#"<subsection inlist="toc" labels="LABEL:a:sub" xml:id="A1.SS1"><tags><tag>A.1</tag>{}<tag role="refnum">A.1</tag><tag role="typerefnum">§A.1</tag></tags><title><tag close=" ">A.1</tag>Sub</title></subsection>"#,
      sec_forms()
    ),
  );
  assert_eq!(
    unit_cref_types(&xml),
    "s:intro=Sec.|a:one=Sec.|a:sub=Sec.|a:two=Sec.",
    "{xml}"
  );
  // `\appendix` (cleveref's patched one) gives the `appendix` type, a subsection after it `subappendix` ("appendix").
  let with_appendix = convert_clean(
    &tex
      .replace("\\begin{appendices}\n", "\\appendix\n")
      .replace("\\end{appendices}\n", ""),
  );
  assert_element(
    &with_appendix,
    "appendix",
    &[r#"xml:id="A2""#],
    r#"<appendix inlist="toc" labels="LABEL:a:two" xml:id="A2"><tags><tag>Appendix B</tag><tag role="creftype">appendix</tag><tag role="creftypecap">Appendix</tag><tag role="creftypeplural">appendices</tag><tag role="creftypepluralcap">Appendices</tag><tag role="refnum">B</tag><tag role="typerefnum">Appendix B</tag></tags><title><tag close=" ">Appendix B</tag>Two</title><toctitle><tag close=" ">B</tag>Two</toctitle></appendix>"#,
  );
  assert_eq!(
    unit_cref_types(&with_appendix),
    "s:intro=Sec.|a:one=appendix|a:sub=appendix|a:two=appendix",
    "{with_appendix}"
  );
}

/// Each labelled sectioning unit in document order as `label=creftype`, the `creftype` tag of its own `<tags>`.
fn unit_cref_types(xml: &str) -> String {
  let re = regex::Regex::new(
    r#"(?s)<(?:chapter|section|subsection|subsubsection|appendix) [^>]*labels="LABEL:([^"]*)"[^>]*>\s*<tags>(.*?)</tags>"#,
  )
  .unwrap();
  let creftype = regex::Regex::new(r#"<tag role="creftype">([^<]*)</tag>"#).unwrap();
  re.captures_iter(xml)
    .map(|c| {
      let ty = creftype
        .captures(&c[2])
        .map_or("-", |t| t.get(1).unwrap().as_str());
      format!("{}={ty}", &c[1])
    })
    .collect::<Vec<_>>()
    .join("|")
}

/// The type forms cleveref gives an `appendix` (and a `subappendix`, … — cleveref.sty:4055-4057 name them alike).
fn appendix_forms() -> &'static str {
  r#"<tag role="creftype">appendix</tag><tag role="creftypecap">Appendix</tag><tag role="creftypeplural">appendices</tag><tag role="creftypepluralcap">Appendices</tag>"#
}

/// cleveref.sty:174-287: the patched `\appendix` aliases section, subsection, subsubsection to appendix, subappendix,
/// subsubappendix for the rest of the document (`\gdef\refstepcounter@noarg`, by counter name), so appendix.sty's
/// `{appendices}` opened after it inherits the alias (pdflatex with the first aid removed, as the arXiv PDFs' cleveref:
/// "See appendix A, appendix A.1, appendix A.1.1 and section 1."). Red (79d r1): "section" for all three — the
/// environment's units kept their counter's type though `\appendix` had run. Witnesses 1810.06583, 1904.07153,
/// 2208.03406. Repro cross-references/cleveref_appendix_before_appendices_names_appendix.
#[test]
fn cleveref_appendix_before_appendices_names_appendix() {
  let xml = convert_clean(repro!(
    "cross-references/cleveref_appendix_before_appendices_names_appendix.tex"
  ));
  assert_eq!(
    unit_cref_types(&xml),
    "s:i=section|a:one=appendix|a:sub=appendix|a:subsub=appendix",
    "{xml}"
  );
  assert_element(
    &xml,
    "subsubsection",
    &[r#"xml:id="A1.SS1.SSS1""#],
    &format!(
      r#"<subsubsection inlist="toc" labels="LABEL:a:subsub" xml:id="A1.SS1.SSS1"><tags><tag>A.1.1</tag>{}<tag role="refnum">A.1.1</tag><tag role="typerefnum">§A.1.1</tag></tags><title><tag close=" ">A.1.1</tag>SubSub</title></subsubsection>"#,
      appendix_forms()
    ),
  );
}

/// A section of `{appendices}` stepped before an `\appendix` inside the environment keeps its counter's type, one
/// stepped after it is an `appendix` (pdflatex with the first aid removed: "See section A and appendix A."). Red (79d
/// r1): "section" for both — the inner `\appendix` (the kernel's `\@appendix` already `\relax`) never cleared the
/// environment's flag. Witness 2112.09636. Repro cross-references/cleveref_appendix_inside_appendices_names_appendix.
/// The "Appendix B" asserted below is a known residual, not pdflatex's: latex.ltx's `\appendix` resets the section
/// counter, so pdflatex letters the second "A" again (SYNC_STATUS row "appendix.sty: `\appendix` inside
/// `{appendices}` does not restart the letters", OPEN; review rev79d2 finding 8).
#[test]
fn cleveref_appendix_inside_appendices_names_appendix() {
  let xml = convert_clean(repro!(
    "cross-references/cleveref_appendix_inside_appendices_names_appendix.tex"
  ));
  assert_eq!(
    unit_cref_types(&xml),
    "a:one=section|a:two=appendix",
    "{xml}"
  );
  assert_element(
    &xml,
    "appendix",
    &[r#"xml:id="A2""#],
    &format!(
      r#"<appendix inlist="toc" labels="LABEL:a:two" xml:id="A2"><tags><tag>Appendix B</tag>{}<tag role="refnum">B</tag><tag role="typerefnum">Appendix B</tag></tags><title><tag close=" ">Appendix B</tag>Two</title><toctitle><tag close=" ">B</tag>Two</toctitle></appendix>"#,
      appendix_forms()
    ),
  );
}

/// cleveref.sty:232-284: with chapters the patched `\appendix` aliases chapter, section, subsection to appendix,
/// subappendix, subsubappendix (pdflatex with the first aid removed: "See appendix A, appendix A.1, appendix A.1.1 and
/// chapter 1."). Red (int72a, 79d r1): "section" for the appendix chapter's section and subsection. Repro
/// cross-references/cleveref_appendix_in_a_book_aliases_its_sections.
#[test]
fn cleveref_appendix_in_a_book_aliases_its_sections() {
  let xml = convert_clean(repro!(
    "cross-references/cleveref_appendix_in_a_book_aliases_its_sections.tex"
  ));
  assert_eq!(
    unit_cref_types(&xml),
    "c:i=chapter|a:one=appendix|a:sec=appendix|a:sub=appendix",
    "{xml}"
  );
  assert_element(
    &xml,
    "subsection",
    &[r#"xml:id="A1.S1.SS1""#],
    &format!(
      r#"<subsection inlist="toc" labels="LABEL:a:sub" xml:id="A1.S1.SS1"><tags><tag>A.1.1</tag>{}<tag role="refnum">A.1.1</tag><tag role="typerefnum">§A.1.1</tag></tags><title><tag close=" ">A.1.1</tag>Sub</title></subsection>"#,
      appendix_forms()
    ),
  );
}

/// llncs.cls:393-397: a text-mode `\vec` is the author's error (pdflatex "! Missing $ inserted." at `\vec{w}`); the
/// warning names `\vec`, not the binding's `\lx@sv@vec@boldmath`. Red (79b): the internal name. Repro
/// class-bindings/llncs_text_mode_vec_warns_as_vec (probe rev79b/ltext).
#[test]
fn llncs_text_mode_vec_warns_as_vec() {
  let (log, xml) = convert_with(
    repro!("class-bindings/llncs_text_mode_vec_warns_as_vec.tex"),
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(
    log.contains("unexpected:\\vec \\vec should only appear in math mode"),
    "{log}"
  );
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r#"<para xml:id="p1"><p xml:id="p1.1">Text <text font="bold" xml:id="p1.1.1">w</text> here.</p></para>"#,
  );
}

const SVMONO_COPY: &str = repro!("stand-in-copies/svmono_vec/svmono.cls");

/// An svmono paper's `\vec` is its own class copy's (the svmult binding read `svmult.cls`, which such a paper lacks):
/// 0910.4696's copy runs its test in the class body, so `vecarrow` keeps the kernel's arrow (pdflatex CMMI10 and the
/// accent), and without it the upright bold (CMBX10, ω CMMIB10). Red (79b): the bold italic under `vecarrow`.
/// Witnesses 0910.4696, 1001.2906, 1408.4098. Repro stand-in-copies/svmono_vec.
#[test]
fn svmono_vec_reads_its_own_copy() {
  let tex = repro!("stand-in-copies/svmono_vec/svmono_vec_reads_its_own_copy.tex");
  let convert = |tex: &str| {
    let (log, xml) = convert_files_with(tex, &[("svmono.cls", SVMONO_COPY)], Some(PRELOAD));
    assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
    xml
  };
  let arrow = convert(tex);
  assert_element(
    &arrow,
    "XMath",
    &[r#"xml:id="p1.m1.1""#],
    r#"<XMath xml:id="p1.m1.1"><XMApp xml:id="p1.m1.1.1"><XMTok meaning="equals" role="RELOP">=</XMTok><XMApp xml:id="p1.m1.1.1.2"><XMTok meaning="plus" role="ADDOP">+</XMTok><XMApp xml:id="p1.m1.1.1.2.2"><XMTok name="vec" role="OVERACCENT" stretchy="false">→</XMTok><XMTok font="italic" role="UNKNOWN">v</XMTok></XMApp><XMApp xml:id="p1.m1.1.1.2.3"><XMTok name="vec" role="OVERACCENT" stretchy="false">→</XMTok><XMTok font="italic" name="omega" role="UNKNOWN">ω</XMTok></XMApp></XMApp><XMTok font="italic" role="UNKNOWN">x</XMTok></XMApp></XMath>"#,
  );
  let bold = convert(&tex.replace("[vecarrow]", ""));
  assert_element(
    &bold,
    "XMath",
    &[r#"xml:id="p1.m1.1""#],
    r#"<XMath xml:id="p1.m1.1"><XMApp xml:id="p1.m1.1.1"><XMTok meaning="equals" role="RELOP">=</XMTok><XMApp xml:id="p1.m1.1.1.2"><XMTok meaning="plus" role="ADDOP">+</XMTok><XMTok font="bold" role="UNKNOWN">v</XMTok><XMTok font="bold italic" name="omega" role="UNKNOWN">ω</XMTok></XMApp><XMTok font="italic" role="UNKNOWN">x</XMTok></XMApp></XMath>"#,
  );
}

/// IEEEojcsys.cls:4058-4080 (2206.03535's copy; IEEEtran.cls:5755's code) letters the appendices itself and never calls
/// `\appendix`, so cleveref's patched `\appendix` does not run and an appendix keeps the `section` type (pdflatex with
/// the class copy and the first aid removed: "See section I and section A.", the heading "Appendix A One"). Red (79d
/// r2): "appendix A" and the heading "Appendix" — the binding let `\appendices` be `\appendix` (review rev79d2
/// finding 9). Repro cross-references/ieeeojcsys_appendices_keep_the_section_type.
#[test]
fn ieeeojcsys_appendices_keep_the_section_type() {
  let xml = convert_clean(repro!(
    "cross-references/ieeeojcsys_appendices_keep_the_section_type.tex"
  ));
  assert_eq!(
    unit_cref_types(&xml),
    "s:intro=section|a:one=section",
    "{xml}"
  );
  assert_element(
    &xml,
    "appendix",
    &[r#"xml:id="A1""#],
    &format!(
      r#"<appendix inlist="toc" labels="LABEL:a:one" xml:id="A1"><tags><tag>Appendix A</tag><tag role="autoref">Appendix{nbsp}A<text xml:id="A1.1"/></tag><tag role="creftype">section</tag><tag role="creftypecap">Section</tag><tag role="creftypeplural">sections</tag><tag role="creftypepluralcap">Sections</tag><tag role="refnum">A</tag><tag role="typerefnum">Appendix A</tag></tags><title><tag close=" ">Appendix A</tag>One</title><toctitle><tag close=" ">A</tag>One</toctitle><para xml:id="A1.p1"><p xml:id="A1.p1.1">Text.</p></para></appendix>"#,
      nbsp = '\u{a0}'
    ),
  );
}
