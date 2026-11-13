//! Four older over-expansions that tex.web §1215's check (a definition's name
//! must be a control sequence, batch 56jp) exposed as errors in sweep #125, and
//! the loader's rollback misfire (batch 56jw): `\hyperref`'s link text read as
//! Semiverbatim, listings' `\label` reverted against cleveref's `\label`, the
//! case changer expanding a declared UTF-8 character and l3text's encoding
//! escape, and binding closures that define inside a number scan. Text and
//! counts are pdflatex's, except the residual `case_change_keeps_a_text_command`
//! pins ("A Α B", pdflatex "A ΄Α B"); repros in `tools/perfect_kernel/repros/`.
use latexml::util::test::assert_element;

use super::perfect_kernel_batch46::{convert, error_count, warning_count};

/// Convert a clean repro: pdflatex has no error and no warning on it.
fn convert_clean(tex: &str) -> String {
  let (log, xml) = convert(tex, true);
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  xml
}

/// `\hyperref{url}{category}{name}{text}` typesets its text as ordinary material
/// (hyperref.sty:4825-4832), so `\textbf` makes no definition named `m`; the
/// link is the URL with the `category.name` anchor (:4827). Witnesses:
/// univie-ling, fixdif-zh-cn (KNOWN_PERL_ERRORS #284).
#[test]
fn hyperref_four_argument_text_is_material() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/backend-persona/hyperref_four_argument_text.tex"
  ));
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r#"<para xml:id="p1">
    <p>A <ref font="bold" href="univie-ling-expose.pdf">Manual</ref> B</p>
  </para>"#,
  );
  assert_element(
    &xml,
    "ref",
    &[r##"href="doc.pdf#section.intro""##],
    r##"<ref href="doc.pdf#section.intro"><Math mode="inline" tex="x^{2}" text="x ^ 2" xml:id="p2.m1">
          <XMath>
            <XMApp>
              <XMTok role="SUPERSCRIPTOP" scriptpos="post1"/>
              <XMTok font="italic" role="UNKNOWN">x</XMTok>
              <XMTok fontsize="70%" meaning="2" role="NUMBER">2</XMTok>
            </XMApp>
          </XMath>
        </Math> text</ref>"##,
  );
}

/// listings writes a literal `\label{<label>}` (listings.sty:1641): under
/// cleveref the label is the float's, and the caption survives. Witnesses:
/// ualberta, unbtex-example (KNOWN_PERL_ERRORS #285).
#[test]
fn listings_label_under_cleveref() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/captions-floats/listings_label_under_cleveref.tex"
  ));
  assert_element(
    &xml,
    "float",
    &[r#"xml:id="LST1""#],
    // `~` is the tag's no-break space (`\lx@nobreakspace`).
    &r#"<float class="ltx_lstlisting" inlist="lol" labels="LABEL:lst:T" xml:id="LST1">
    <tags>
      <tag>Listing~1</tag>
      <tag role="creftype">listing</tag>
      <tag role="creftypecap">Listing</tag>
      <tag role="creftypeplural">listings</tag>
      <tag role="creftypepluralcap">Listings</tag>
      <tag role="refnum">1</tag>
      <tag role="typerefnum">Listing 1</tag>
    </tags>
    <toccaption><tag close=" ">1</tag>Example of How</toccaption>
    <caption><tag close=": ">Listing~1</tag>Example of How</caption>
    <listing class="ltx_lstlisting" data="eCA9IDE=" dataencoding="base64" datamimetype="text/plain">
      <listingline xml:id="lstnumberx1"><text class="ltx_lst_identifier">x</text><text class="ltx_lst_space"> </text>=<text class="ltx_lst_space"> </text>1</listingline>
    </listing>
  </float>"#
      .replace('~', "\u{a0}"),
  );
}

/// The case changer keeps a text command unexpanded (l3text's encoding escape):
/// textalpha's `ά` (`\ensuregreek{\acctonos\textalpha}`) makes no definition
/// named `T`, and the explicit forms are pdflatex's "[Α] [Α]". Residual:
/// pdflatex's literal `\MakeUppercase{ά}` is "΄Α" (a code point case-mapped to
/// U+0386, see the repro). Witnesses: textalpha-doc, char-list,
/// hyperref-with-greek.
#[test]
fn case_change_keeps_a_text_command() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/unicode-catcodes/case_change_text_command_escape.tex"
  ));
  assert_element(&xml, "p", &[], "<p>A \u{391} B [\u{391}] [\u{391}]</p>");
}

/// l3text keeps the command after `\@changed@cmd`/`\@current@cmd` and drops the
/// encoding-specific name (`\__text_expand_encoding_escape:NN`): pdflatex
/// "A É Ç B".
#[test]
fn case_change_keeps_encoding_dispatched_command() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/unicode-catcodes/case_change_encoding_escape.tex"
  ));
  assert_element(&xml, "p", &[], "<p>A \u{c9} \u{c7} B</p>");
}

/// `\newrobustcmd` in a number scan's look-ahead defines nothing until the
/// `\ifnum` has chosen its branch (pdflatex "xBy"), and an `\edef` keeps it
/// (pdflatex "EC"). Witness: synthslant-gauge (KNOWN_PERL_ERRORS #286).
#[test]
fn newrobustcmd_waits_for_a_number_scan() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/newrobustcmd_number_scan.tex"
  ));
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r#"<para xml:id="p1"><p>xBy</p></para>"#,
  );
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p2""#],
    r#"<para xml:id="p2"><p>EC</p></para>"#,
  );
}

/// `\patchcmd`, `\AfterEndPreamble`, `\pushQED`, `\nocite` and `\restartlist`
/// in a false `\ifnum` branch's number-scan look-ahead change nothing
/// (pdflatex "ORIG. .", no citation, item d numbered 4).
#[test]
fn binding_assignments_wait_for_a_number_scan() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/binding_assignments_number_scan.tex"
  ));
  assert_element(&xml, "p", &[], "<p>ORIG.\n.</p>");
  assert!(!xml.contains("<cite"), "{xml}");
  assert_element(
    &xml,
    "enumerate",
    &[r#"xml:id="S0.I3""#],
    &r#"<enumerate xml:id="S0.I3">
      <item xml:id="S0.I3.i4">
        <tags>
          <tag>4.</tag>
          <tag role="refnum">4</tag>
          <tag role="typerefnum">item~4</tag>
        </tags>
        <para xml:id="S0.I3.i4.p1">
          <p>d</p>
        </para>
      </item>
    </enumerate>"#
      .replace('~', "\u{a0}"),
  );
}

/// e-TeX's `\readline` is an assignment: `\endlinechar=-1%` + a newline +
/// `\readline` reads the line after the number scan has set `\endlinechar`
/// (pdflatex "RLSAME"; latexgit.sty:53-54, shdoc.sty:210-211).
#[test]
fn readline_is_an_assignment() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/readline_in_number_scan.tex"
  ));
  assert_element(&xml, "p", &[], "<p>RLSAME</p>");
}

/// `\usepackage{parskip}[=v1]` reads `parskip-2001-04-09.sty` (parskip.sty:41,
/// latex.ltx:19216-19222) instead of version-stripping it back to the parskip
/// binding: no recursion warnings, and the release's `\parskip` and
/// `\parindent` (pdflatex "6.0pt plus 2.0pt", unindented). Witnesses:
/// biblatex-sbl-ibid, biblatex-sbl-examples.
#[test]
fn rollback_release_file_is_read() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/loader/parskip_rollback_release.tex"
  ));
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p2""#],
    r#"<para class="ltx_noindent" xml:id="p2"><p>6.0pt plus 2.0pt</p></para>"#,
  );
}
