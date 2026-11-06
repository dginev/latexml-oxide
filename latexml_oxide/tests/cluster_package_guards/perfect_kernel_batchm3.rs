//! Red/green guards for the TeX Live manuals' remaining regressions (docs/perfect_kernel/MANUAL_REGRESSIONS.md), one on
//! each mechanism, under the manuals' raw profile, against pdflatex:
//! `\@ensuremath` of no parameter (membranecomputing), comment.sty's internals (tikz-trackschematic), `\everypar{}` in a
//! box's restore (plweb/pl), `\lx@pdfstringdef` under a latin `inputenc` and the PDF escapes (bibleref-german, fnlineno),
//! xcolor's repeat options under names of its own (ndsu-thesis-2022; 2605.00310, 2605.28926), autonum's `\forlistloop`
//! (simpler-wick, tikz-feynman; 1905.00606, 2211.09537, 1405.2249), a tcolorbox keytheorem's head (tutodoc), a
//! `\protected` `\begin` (yquant-doc), pgfmath's arrays (tikzpingus-doc; 2002.08586, 2509.25473) and a `\NewColumnType`
//! body's commands (non-decimal-units).
//! The fix round's: `lstlisting` declared at `\begin{document}` (2405.15073), `\end`'s own test of `\protect`, the case
//! changer's `\begin`, `\end`, pgfmath's array text, tabularray's `preto`, `appto`.
use latexml::util::test::{assert_element, convert_with};

use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch84b::text,
};

const RAW: &str = "[rawstyles,rawclasses]latexml.sty";

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/",
      $path,
      ".tex"
    ))
  };
}

/// The value of every `\typeout{<name> = <value>}` of a log, in order: the lines holding ` = `.
fn typed_out(log: &str) -> Vec<String> {
  log
    .lines()
    .filter_map(|line| {
      // (a `\typeout` follows the loader's last message on its line)
      let at = line.find(['R', 'S', 'A']).unwrap_or(0);
      let line = &line[at..];
      let head = line.split(' ').next().unwrap_or("");
      let is_tag = head.trim_end_matches(':').len() >= 2
        && head[1..]
          .trim_end_matches(':')
          .bytes()
          .all(|b| b.is_ascii_digit());
      (is_tag && line.contains(" = ")).then(|| line.to_string())
    })
    .collect()
}

/// xstring's `\StrExpand[2]` of a lone `\ensuremath` expands it twice: `\@ensuremath` takes no parameter, as latex.ltx's
/// `\ensuremath ` (:15801-15806), where one read the cell's `}` ("Missing { inserted", 3 errors); `protected` stays, so
/// the case changer's no-case-change list (sect13.rs) still sees it. Witness membranecomputing:509.
#[test]
fn ensuremath_expanded_one_step_alone_in_a_macro() {
  let (log, xml) = convert_with(
    repro!("expansion-primitives/ensuremath_one_step_alone_in_a_macro"),
    Some(RAW),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "tabular",
    &[],
    r##"<tabular class="ltx_guessed_headers" vattach="middle"><tbody><tr><td align="left" thead="row">a</td><td align="left"><Math mode="inline" tex="{G}" text="G" xml:id="p1.m1"><XMath><XMTok font="italic" role="UNKNOWN">G</XMTok></XMath></Math></td></tr><tr><td align="left" thead="row">b</td><td align="left"><Math mode="inline" tex="b" text="b" xml:id="p1.m2"><XMath><XMTok font="italic" role="UNKNOWN">b</XMTok></XMath></Math></td></tr></tbody></tabular>"##,
  );
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p2""#],
    r##"<para xml:id="p2"><p>X <Math mode="inline" tex="a" text="a" xml:id="p2.m1"><XMath><XMTok font="italic" role="UNKNOWN">a</XMTok></XMath></Math> Y</p></para>"##,
  );
}

/// comment.sty's internals (:170-233, :283-289) for a comment type of a package's own, as lwarp.sty:1349-1363 makes, and
/// the two types left to them, `\generalcomment`, `\processcomment`; the binding's own stays for the others.
/// Witnesses tikz-trackschematic, tikz-trackschematic-snippets (2 errors, 101 and a Fatal on the manuals).
#[test]
fn comment_internals_make_a_type_of_its_own() {
  let (log, xml) = convert_with(
    repro!("package-bindings/comment_internals_for_a_type_of_its_own"),
    Some(RAW),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r##"<para xml:id="p1"><p>Before. Middle. Shown <text font="bold">bold</text>. Then. General <emph font="italic">text</emph>. Next. Processed text. Also. After.</p></para>"##,
  );
}

/// latex.ltx:16279: `\parbox`, `{minipage}` (:16322), a `p` cell (:16755) and a float (:17482) clear `\everypar`, so the
/// box's paragraphs do not run the paragraph's, which would recurse where it holds the box (plweb's pl.sty:121, a Fatal).
#[test]
fn a_box_restore_clears_everypar() {
  let (log, xml) = convert_with(
    repro!("boxes-groups/box_restore_clears_everypar"),
    Some(RAW),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r##"<para xml:id="p1"><p>First line.</p></para>"##,
  );
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p2""#],
    r##"<para xml:id="p2"><p>Second line.</p></para>"##,
  );
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p3""#],
    r##"<para xml:id="p3"><p>[E]Text.<inline-block class="ltx_parbox" vattach="middle" width="56.9pt"><p>Parbox</p></inline-block><inline-block class="ltx_minipage" vattach="middle" width="56.9pt"><p>Minipage</p></inline-block><note mark="1" role="footnote" xml:id="footnote1"><tags><tag>1</tag><tag role="refnum">1</tag><tag role="typerefnum">footnote 1</tag></tags>Note</note><note class="ltx_marginpar" role="margin"><inline-logical-block><para xml:id="p3.p1"><p>Margin</p></para></inline-logical-block></note><tabular vattach="middle"><tbody><tr><td align="left" vattach="top"><inline-block vattach="top" width="56.9pt"><p>Pcell</p></inline-block></td><td align="left">Lcell</td></tr></tbody></tabular></p></para>"##,
  );
  assert_element(
    &xml,
    "figure",
    &[],
    r##"<figure inlist="lof" xml:id="S0.F1"><tags><tag>Figure 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Figure 1</tag></tags><p>Float</p><toccaption><tag close=" ">1</tag>Cap</toccaption><caption><tag close=": ">Figure 1</tag>Cap</caption></figure>"##,
  );
}

/// `\lx@pdfstringdef`'s letters are `\string`'d: a latin `inputenc` makes ß active with the meaning `\ss`, a loop as the
/// binding's own `\def\ss{ß}` (bibleref-german, a Fatal); the digit commands are hyperref.sty:388-396's and the escapes
/// are the characters they name (fnlineno:44's `\374`, "\3 undefined").
#[test]
fn a_pdf_string_under_a_latin_inputenc_and_with_escapes() {
  let (log, xml) = convert_with(
    repro!("sectioning-frontmatter/hyperref_pdfinfo_under_latin_inputenc"),
    Some(RAW),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let rdf: Vec<&str> = xml
    .match_indices("<rdf ")
    .map(|(i, _)| &xml[i..i + xml[i..].find("/>").unwrap_or(0) + 2])
    .collect();
  assert_eq!(rdf, [
    "<rdf about=\"\" content=\"Uwe Lück\" property=\"dcterms:creator\"/>",
    "<rdf about=\"\" content=\"Waßenhoven, øre, ä\" property=\"dcterms:subject\"/>",
    "<rdf about=\"\" content=\"• and ß\" property=\"dcterms:subject\"/>",
    "<rdf about=\"\" content=\"Waßenhoven ø Æ ł œ ö\" property=\"dcterms:title\"/>",
  ]);
}

/// xcolor's options as keys outlive its load (OD #43, witnesses 2605.00310, 2605.28926) under names of their own: another
/// package's option `table` is that package's (`\usepackage[figure,table]{totalcount}`, ndsu-thesis-2022), while the
/// repeat load of xcolor still loads colortbl and the dvips names.
#[test]
fn a_repeat_option_of_xcolor_is_xcolors_alone() {
  let (log, xml) = convert_with(
    repro!("package-bindings/xcolor_repeat_option_is_xcolors_alone"),
    Some(RAW),
  );
  // (totalcount's "Total counter(s) have changed", pdflatex's on the first run too)
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r##"<para xml:id="p1"><p>Before. no tables after.<tabular vattach="middle"><tbody><tr><td align="left" backgroundcolor="#AD0000"><text backgroundcolor="#AD0000">Cell</text></td></tr></tbody></tabular></p></para>"##,
  );
}

/// autonum.sty:20-22 puts etoolbox's `\forlistloop` back after loading etextools: translations.sty's begin-document hook
/// stays whole (simpler-wick, tikz-feynman: an Extra `\fi`, the next chunk eaten).
#[test]
fn autonum_keeps_the_begin_document_hook_whole() {
  let (log, xml) = convert_with(
    repro!("package-bindings/autonum_keeps_etoolbox_forlistloop"),
    Some(RAW),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    log.contains("NEXT CHUNK RUNS") && log.contains("THIRD CHUNK RUNS"),
    "{log}"
  );
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r##"<para xml:id="p1"><p>Word.</p></para>"##,
  );
}

/// keytheorems.sty's `title=\__keythms_thm_tcbtemphead:` for a head the amsthm binding's theorem has made its `ltx:title`:
/// no Error, the head and the body kept (tutodoc-en, tutodoc-fr).
#[test]
fn a_tcolorbox_keytheorem_keeps_its_head_and_body() {
  let (log, xml) = convert_with(
    repro!("package-bindings/keytheorem_tcolorbox_head"),
    Some(RAW),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "title",
    &[],
    r##"<title class="ltx_runin"><tag><text font="bold">Warning</text></tag><text font="bold">.</text></title>"##,
  );
  assert_element(
    &xml,
    "p",
    &[],
    r##"<p><text color="#000000" font="italic">Body words.</text></p>"##,
  );
  assert!(!xml.contains("<ERROR"), "{xml}");
}

/// latex.ltx:15346 `\protected\def\begin` and :15365-15380's `\protect\end `: a `\protected@edef` keeps both (yquant-doc:
/// `\matrix@check` at edef time, "Old form"), and a cell's look-ahead for `\omit` stops at `\begin`.
#[test]
fn begin_is_protected_in_an_edef() {
  let (log, xml) = convert_with(
    repro!("expansion-primitives/begin_protected_in_an_edef"),
    Some(RAW),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  for meaning in [
    r"X=macro:->\begin {myenv}a\protect \end  {myenv}",
    r"Y=macro:->\begin {center}b\protect \end  {center}",
    r"Z=macro:->$\begin {pmatrix} a & b\protect \end  {pmatrix}$",
  ] {
    assert!(log.contains(meaning), "{meaning} {log}");
  }
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r##"<para xml:id="p1"><p>[open:a:close]</p><p align="center">b</p><p><Math mode="inline" tex="\begin{pmatrix}a&amp;b\end{pmatrix}" text="matrix@(Array[[a, b]])" xml:id="p1.m1"><XMath><XMDual><XMApp><XMTok meaning="matrix"/><XMRef idref="p1.m1.1"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="true">(</XMTok><XMArray xml:id="p1.m1.1"><XMRow><XMCell align="center"><XMTok font="italic" role="UNKNOWN">a</XMTok></XMCell><XMCell align="center"><XMTok font="italic" role="UNKNOWN">b</XMTok></XMCell></XMRow></XMArray><XMTok role="CLOSE" stretchy="true">)</XMTok></XMWrap></XMDual></XMath></Math><tabular vattach="middle"><tbody><tr><td align="left">[open:c:close]</td></tr><tr><td align="left"><tabular vattach="middle"><tr><td align="left">d</td></tr></tabular></td></tr></tbody></tabular></p></para>"##,
  );
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p2""#],
    r##"<para xml:id="p2"><tabular vattach="middle"><tbody><tr><td align="center" colspan="2">[open:e:close]</td></tr><tr><td align="left"><inline-block class="ltx_minipage" vattach="middle" width="28.5pt"><p>f</p></inline-block></td><td align="left" vattach="top"><inline-block vattach="top" width="56.9pt"><p>[open:g:close]</p></inline-block></td></tr></tbody></tabular></para>"##,
  );
  assert_element(
    &xml,
    "section",
    &[],
    r##"<section inlist="toc" xml:id="S1"><tags><tag>1</tag><tag role="refnum">1</tag><tag role="typerefnum">§1</tag></tags><title><tag close=" ">1</tag>Head [open:h:close]</title><figure inlist="lof" xml:id="S1.F1"><tags><tag>Figure 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Figure 1</tag></tags><toccaption><tag close=" ">1</tag>Cap [open:i:close]</toccaption><caption><tag close=": ">Figure 1</tag>Cap [open:i:close]</caption></figure></section>"##,
  );
}

/// A binding's macro that assigns as it expands (`\ket`, a math dual's counter) in an `\edef` leaves a pending
/// `\afterassignment` pending: `\protected@edef`'s `\restore@protect` runs after the assignment, not inside its body, so
/// `\protect` is back and `\end{yquant*}` ends its environment (yquant-doc: 501 errors and a Fatal once `\end` read it).
#[test]
fn a_binding_macro_in_an_edef_leaves_afterassignment_pending() {
  let (log, xml) = convert_with(
    repro!("expansion-primitives/afterassignment_pending_through_an_edef"),
    Some(RAW),
  );
  // (yquant's Warning for the `compat` key's newest)
  assert_eq!(error_count(&log), 0, "{log}");
  assert!(log.contains("P=\\relax") && log.contains("AFTER"), "{log}");
  assert_element(
    &xml,
    "Math",
    &[],
    r##"<Math mode="inline" tex="\ket{x}" text="ket@(x)" xml:id="p1.pic1.m1"><XMath><XMDual><XMApp><XMTok meaning="ket" role="UNKNOWN"/><XMRef idref="p1.pic1.m1.1"/></XMApp><XMWrap><XMTok role="VERTBAR" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="p1.pic1.m1.1">x</XMTok><XMTok name="rangle" role="CLOSE" stretchy="false">⟩</XMTok></XMWrap></XMDual></XMath></Math>"##,
  );
  assert_element(&xml, "p", &[], r##"<p>Word.</p>"##);
}

/// The native parser reads pgf's arrays, each value pdflatex's: a `{…}[i]` primary, text elements, `array`, `dim`, nesting
/// and a list as `\pgfmathresult` holds it. Each was a wrong number with no Error (tikzpingus-doc's `0.0` wing name;
/// 2002.08586, 2509.25473).
#[test]
fn pgfmath_arrays_are_read_as_pgf_reads_them() {
  let (log, _) = convert_with(
    repro!("graphics-tikz/pgfmath_array_primary_and_strings"),
    Some(RAW),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(typed_out(&log), [
    r#"R0: {1,2,3}[1] = 2"#,
    r#"R1: {1,2,3}[0] = 1"#,
    r#"R2: {1.5,2}[0] = 1.5"#,
    r#"R3: {1+1,2}[0] = 2.0"#,
    r#"R4: {"none","up","down"}[1] = up"#,
    r#"R5: {"a","b"}[1] = b"#,
    r#"R6: array({1,2,3},1) = 2"#,
    r#"R7: array({"a","b"},1) = b"#,
    r#"R8: dim({"a","b"}) = 2"#,
    r#"R9: dim({1,2}) = 2"#,
    r#"R10: dim({{1,2,3},{4,5,6}}) = 2"#,
    r#"R11: dim({}) = 0"#,
    r#"R12: {{1,2},{3,4}}[1][0] = 3"#,
    r#"R13: {{1,2},{3,4}}[1] = {{3}{4}}"#,
    r#"R14: array({{1,2},{3,4}},1) = {3}{4}"#,
    r#"R15: 2*{1,2,3}[1]+1 = 5.0"#,
    r#"R16: {1,2,3}[1]*{1,2,3}[2] = 6.0"#,
    r#"R17: {2} = 2"#,
    r#"R19: {1,2,3} = {1}{2}{3}"#,
    r#"R20: {"a","b"} = {a}{b}"#,
    r#"R21: sin({30,60}[0]) = 0.5"#,
    r#"R22: {"a","b"}[1+0] = b"#,
    r#"R23: {"a b","c,d"}[1] = c,d"#,
    r#"R24: 1 ? {"a","b"}[0] : "c" = a"#,
    r#"R25: 0 ? {"a","b"}[0] : {"c","d"}[1] = d"#,
    r#"R26: {1pt,2pt}[1] = 2.0"#,
    r#"R27: {1cm,2cm}[0]+1pt = 29.45276"#,
    r#"R28: max({1,2,3}[1],0) = 2.0"#,
    r#"R29: ifthenelse(1,{1,2}[0],0) = 1"#,
    r#"R30: {1,2,3}[1.7] = 2"#,
    r#"R31: {1,2,3}[int(1.7)] = 2"#,
    r#"R32: {7,8,9}[dim({1,2})] = 9"#,
    r#"R33: array({5,6},0)+1 = 6.0"#,
    r#"R34: "up" = up"#,
    r#"S1 = up"#,
    r#"S2 = down"#,
    r#"S3 = {1}{2}{3}"#,
  ]);
}

/// What pgf reports stays reported: an index out of bounds (pgfmathfunctions.misc.code.tex:252) and text in arithmetic
/// ("Missing number"), the values as pgf's.
#[test]
fn pgfmath_array_misuse_is_an_error() {
  let (log, _) = convert_with(repro!("graphics-tikz/pgfmath_array_errors"), Some(RAW));
  assert_eq!((error_count(&log), warning_count(&log)), (5, 0), "{log}");
  assert_eq!(typed_out(&log), [
    "A9 = 0",
    "A10 = 0",
    "A11 = 0",
    "A12 = 1.0",
    "A13 = 2.0",
  ]);
}

/// A `\NewColumnType`'s body keeps its commands, of any name and defined before or after it (non-decimal-units.sty:778,
/// `\ndu_tblr_cell:nnn` read back as `\ndu` and `_`), with its arguments and a default.
#[test]
fn a_column_type_body_keeps_its_commands() {
  let (log, xml) = convert_with(
    repro!("alignment-bindings/tblr_column_type_body_keeps_its_commands"),
    Some(RAW),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "tabular",
    &[],
    r##"<tabular vattach="middle"><tbody><tr><td align="left">(One)</td><td align="left">¡Two¿</td><td align="right">[a—b—Three]</td><td align="center">[x—d—Four]</td><td align="center">[y—d—Five]</td></tr></tbody></tabular>"##,
  );
}

/// listings.sty:1604-1609 declares `lstlisting` in its `\AtBeginDocument`, in the hook's place: a preamble's tocloft
/// `\newlistentry{lstlisting}` finds it undefined, a document's own hook finds it defined, whichever side of the package it
/// was added (2405.15073, whose `\globaldefs\@ne` kept that spurious error's display `\protect` for good; KPE #1217).
#[test]
fn the_lstlisting_counter_waits_for_begin_document() {
  let (log, xml) = convert_with(
    repro!("listings-algorithms/listings_counter_at_begin_document"),
    None,
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(typed_out(&log), [
    "R1 = undefined",
    "R2 = undefined",
    "R0 = defined",
    "R3 = defined",
  ]);
  assert_element(
    &xml,
    "float",
    &[],
    r##"<float class="ltx_lstlisting" inlist="lol" labels="LABEL:l1" xml:id="S1.LST1"><tags><tag>Listing 1.1</tag><tag role="refnum">1.1</tag><tag role="typerefnum">Listing 1.1</tag></tags><toccaption><tag close=" ">1.1</tag>Code</toccaption><caption><tag close=": ">Listing 1.1</tag>Code</caption><listing class="ltx_lstlisting" data="eA==" dataencoding="base64" datamimetype="text/plain"><listingline xml:id="lstnumberx1"><text class="ltx_lst_identifier">x</text></listingline></listing></float>"##,
  );
  assert_element(
    &xml,
    "p",
    &[r#"align="center""#],
    r##"<p align="center">Centered <text font="bold">bold</text>.</p>"##,
  );
}

/// `\end` is latex.ltx:15365-15380's front, `\end ` its body: any `\protect` but the typesetting one puts `\protect` before
/// `\end ` (a `\typeout` prints it, a `\protected@edef` keeps it), and one that expands away reaches the body, no loop. A
/// siunitx `S` cell's read, in that regime, still ends at a last row's `\end{tabular}` (2505.21533: E 1 → 113 otherwise;
/// 2105.06936).
#[test]
fn end_tests_the_typeset_protect() {
  let (log, xml) = convert_with(
    repro!("expansion-primitives/end_tests_the_typeset_protect"),
    Some(RAW),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(typed_out(&log), [
    r"R1 = \end {myenv}",
    r"R3 = macro:->\protect \end  {myenv}",
    r"R4 = macro:->\end {myenv}",
  ]);
  assert_element(&xml, "p", &[], "<p>[open:A:close]</p>");
  assert_element(
    &xml,
    "p",
    &[r#"align="center""#],
    r#"<p align="center">B</p>"#,
  );
  assert_element(
    &xml,
    "tabular",
    &[],
    r##"<tabular vattach="middle"><tbody><tr><td align="left">a</td><td align="left"><Math mode="inline" tex="1.5" text="1.5" xml:id="p3.m1"><XMath><XMTok meaning="1.5" role="NUMBER">1.5</XMTok></XMath></Math></td></tr><tr><td align="left">b</td><td align="left"><text font="bold">2.5</text></td></tr></tbody></tabular>"##,
  );
}

/// The case changer takes the name after `\begin` and `\end` as it is (expl3-code.tex:36005), for all three changes.
#[test]
fn a_case_change_keeps_an_environment_name() {
  let (log, xml) = convert_with(
    repro!("expansion-primitives/case_change_keeps_an_environment_name"),
    Some(RAW),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(
    text(&xml),
    "[open:ABC:close] [open:abc:close] [open:Abc def:close] X IN Y 1 1 §1 1 [open:HEAD:close]"
  );
}

/// A pgfmath value that is text, each pdflatex's: through `ifthenelse`'s branches, a numeric string in arithmetic, a whole
/// formula's braces, a list's strip of a string's spaces (`{"a b","c"}[0]` is `ab`, `{"a ? b","c"}[0]` is `a? b`), and a
/// list of one, whose index reads its value's tokens, and the sizers' braced `height{"Q"}` (circuitikz's `raised voltages`,
/// circuitikzmanual: 11 false "Missing number" with M3's arrays). No Error here, where pdflatex has none.
#[test]
fn pgfmath_array_text_is_read_as_pgf_reads_it() {
  let (log, _) = convert_with(
    repro!("graphics-tikz/pgfmath_array_text_as_pgf_reads_it"),
    Some(RAW),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(typed_out(&log), [
    r#"R0: ifthenelse(0,{"a","b"}[0],{"c","d"}[1]) = d"#,
    r#"R1: ifthenelse(1,{1,2}[1],{"c","d"}[1]) = 2"#,
    r#"R2: ifthenelse(0,{1,2}[1],{"c","d"}[1]) = d"#,
    r#"R3: ifthenelse(1,{"a b","c"}[0],2) = ab"#,
    r#"R4: ifthenelse(1,ifthenelse(0,1,{"x","y"}[1]),2) = y"#,
    r#"R5: ifthenelse({"1","0"}[1],"s","t") = t"#,
    r#"R6: {"1","2"}[0]+1 = 2.0"#,
    r#"R7: 1+{"1","2"}[1] = 3.0"#,
    r#"R8: {"1","2"}[0]*{"3","4"}[0] = 3.0"#,
    r#"R9: {"1.5","2"}[0]*2 = 3.0"#,
    r#"R10: {"-1","2"}[0]+1 = 0.0"#,
    r#"R11: {"1","0"}[0]?5:6 = 5"#,
    r#"R12: {5,6}[{"1","0"}[0]] = 6"#,
    r#"R13: sin({"30","0"}[0]) = 0.5"#,
    r#"R14: {"1pt","2"}[0]+1 = 2.0"#,
    r#"R15: {"1cm","2"}[0]+1 = 29.45276"#,
    r#"R16: {".5","2"}[0]+1 = 1.5"#,
    r#"R17: {"--1","2"}[0]+1 = 2.0"#,
    r#"R18: {"a"} = a"#,
    r#"R19: {"a b"} = a b"#,
    r#"R20: { "a" } = a"#,
    r#"R21: {" a"} = a"#,
    r#"R22: {{"a"}} = a"#,
    r#"R23: {{"a","b"}} = {a}{b}"#,
    r#"R24: {{{1,2}}} = {1}{2}"#,
    r#"R25: {"a b","c"}[0] = ab"#,
    r#"R26: {"c","a b"}[1] = ab"#,
    r#"R27: {" a","c"}[0] = a"#,
    r#"R28: {"a b c","d"}[0] = ab c"#,
    r#"R29: {"ab c","d"}[0] = ab c"#,
    r#"R30: {"a ? b","c"}[0] = a? b"#,
    r#"R31: {" a b c","d"}[0] = ab c"#,
    r#"R32: {"\foo b","c"}[0] = FOOb"#,
    r#"R33: {{"a b","c"},"d"}[0][0] = ab"#,
    r#"R34: {{"a b"},"d"}[0] = ab"#,
    r#"R35: {"a b","c d"} = {ab}{cd}"#,
    r#"R36: array({"a b","c"},0) = ab"#,
    r#"R37: \strs [1] = cd"#,
    r#"R38: "a b" = a b"#,
    r#"R39: " a b" = a b"#,
    r#"R40: {"a b"}[0] = a"#,
    r#"R41: {"a b"}[1] = b"#,
    r#"R42: {"abc"}[1] = b"#,
    r#"R43: {12}[1] = 2"#,
    r#"R44: {1cm}[0] = 2"#,
    r#"R45: {1.5}[0] = 1"#,
    r#"R46: {12}[1]+1 = 3.0"#,
    r#"R47: {"12"}[1]+1 = 3.0"#,
    r#"R48: array({"ab"},1) = b"#,
    r#"R49: array({"ab"},0) = a"#,
    r#"R50: array({12},1) = 2"#,
    r#"R51: {{1}}[0] = 1"#,
    r#"R52: {{"ab"}}[0] = ab"#,
    r#"R53: height{"Q"} = 6.83331"#,
    r#"R54: width{"M"} = 9.16669"#,
    r#"R55: depth{"Q"} = 1.94444"#,
    r#"R56: height("Q") = 6.83331"#,
    r#"R57: 1pt+2*height{"Q"} = 14.66663"#,
  ]);
}

/// `preto` and `appto` of every key list join the cell's text as tabularray's do, inside its math and its `cmd`; those of
/// `\SetCell`, `\SetRow`, `\SetColumn`, which the binding does not apply, are each counted by a Warning naming the key.
#[test]
fn tblr_preto_and_appto_join_the_cell_text() {
  let (log, xml) = convert_with(
    repro!("alignment-bindings/tblr_preto_appto_join_the_cell_text"),
    Some(RAW),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 3), "{log}");
  for (key, command, lost) in [
    ("preto", "SetCell", "S"),
    ("preto", "SetRow", "T"),
    ("appto", "SetColumn", "U"),
  ] {
    let line = format!(
      "Warning:tabularray_key:{key} The key {key} of \\{command} is not applied: its text '{lost}' is lost"
    );
    assert!(log.contains(&line), "{line}\n{log}");
  }
  let cells: Vec<String> = xml
    .split("<td ")
    .skip(1)
    .map(|cell| {
      text(&format!(
        "<td {}",
        &cell[..cell.find("</td>").unwrap_or(cell.len())]
      ))
    })
    .collect();
  assert_eq!(cells, [
    "xf",
    "gy",
    "phq",
    "i",
    "j",
    "ck",
    "l",
    "mr",
    "anb",
    "Ao",
    "As",
    "CtB",
    "CuB",
    "[dve]",
    "\u{2062} α w β",
    "21z34",
    "E",
    "kK",
    "aa",
    "bb",
    "cc",
    "dd",
    "ee",
    "ff",
  ]);
}
