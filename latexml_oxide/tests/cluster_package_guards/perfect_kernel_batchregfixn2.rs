//! Red/green guards for batch REGFIXN2: v0.8.0 regressions — the daj binding's lastpage beside its copy's totpages,
//! an undefined latin1 byte in a hyperref PDF key, and lipics' comment.sty before lipics-v2019.
//! Repros `tools/perfect_kernel/repros/{stand-in-copies/daj_tocbase,unicode-catcodes,class-bindings/lipics_v2018}/`;
//! each pdflatex-checked.
use latexml::util::test::{
  assert_element, convert_bytes_files_then, convert_files_with, convert_with,
};

use super::perfect_kernel_batch46::{error_count, warning_count};

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!("../../../tools/perfect_kernel/repros/", $path))
  };
}

/// The daj binding loads no lastpage: the paper's copies of daj.cls and tocbase.cls, read after it, load totpages
/// (tocbase.cls:162, 2020-2022) or lastpage (:162, 2016 and 2022-2023) themselves, and both is totpages.sty:148-153's
/// "Can't use both" Error. Red (REGFIXJ3): 1 Error:latex:(totpages) on the totpages copy. Witnesses 2003.11116,
/// 2008.04606, 2010.07854, 2203.04657, 2212.13728; the lastpage copy (2208.01327, 2305.10828) stays clean.
#[test]
fn daj_copy_loads_its_own_page_counter() {
  for (tocbase, label) in [
    (
      repro!("stand-in-copies/daj_tocbase/tocbase.cls"),
      "TotPages",
    ),
    (
      repro!("stand-in-copies/daj_tocbase/tocbase_lastpage.cls"),
      "LastPage",
    ),
  ] {
    let tex = repro!("stand-in-copies/daj_tocbase/daj_class_chain_loads_totpages.tex")
      .replace("\\ref{TotPages}", &format!("\\ref{{{label}}}"));
    let (log, xml) = convert_files_with(
      &tex,
      &[
        ("daj.cls", repro!("stand-in-copies/daj_tocbase/daj.cls")),
        ("tocbase.cls", tocbase),
      ],
      Some("ar5iv.sty"),
    );
    assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
    assert_element(
      &xml,
      "p",
      &["xml:id=\"p1.1\""],
      &format!("<p xml:id=\"p1.1\">Text on <ref labelref=\"LABEL:{label}\"/> pages.</p>"),
    );
  }
}

/// An undefined latin1 byte is inputenc's robust `\protect\@inpenc@undefined@{latin1}` (inputenc.sty:80), whose
/// `\PackageError` (:139) hyperref's PDF strings gobble: no Error in `pdfauthor`. In body text it is still TeX's Error
/// (pdflatex 1 `!`), the byte dropped. Red (REGFIXJ3): 1 Error:unexpected:<char> for the PDF key. Witnesses 1803.03083,
/// 1609.03819, 1903.03818, 2108.12063, 2408.03605. Repros unicode-catcodes/latin1_undefined_byte_in_{pdf_key,text_errs}.
#[test]
fn latin1_undefined_byte_errs_only_where_tex_does() {
  let pdf_key: &[u8] = include_bytes!(
    "../../../tools/perfect_kernel/repros/unicode-catcodes/latin1_undefined_byte_in_pdf_key.tex"
  );
  let (log, xml, ()) = convert_bytes_files_then(pdf_key, &[], Some("ar5iv.sty"), |_, _| ());
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    "<p xml:id=\"p1.1\">A word.</p>",
  );
  let text: &[u8] = include_bytes!(
    "../../../tools/perfect_kernel/repros/unicode-catcodes/latin1_undefined_byte_in_text_errs.tex"
  );
  let (log, xml, ()) = convert_bytes_files_then(text, &[], Some("ar5iv.sty"), |_, _| ());
  assert_eq!((error_count(&log), warning_count(&log)), (1, 0), "{log}");
  assert!(
    log.contains(
      "Error:latex:(inputenc) Package inputenc Error: Keyboard character used is undefined"
    ),
    "{log}"
  );
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    "<p xml:id=\"p1.1\">A word.</p>",
  );
  // The `{enc}` of `\@inpenc@undefined@{enc}` is the encoding in force: cp1252 leaves 0x81 undefined.
  let switched = [
    &text[..text.len() - b"A \x83word.\n\\end{document}\n".len()],
    b"\\inputencoding{cp1252}A \x81word.\n\\end{document}\n",
  ]
  .concat();
  let (log, xml, ()) = convert_bytes_files_then(&switched, &[], Some("ar5iv.sty"), |_, _| ());
  assert_eq!((error_count(&log), warning_count(&log)), (1, 0), "{log}");
  assert!(log.contains("in inputencoding `cp1252'"), "{log}");
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    "<p xml:id=\"p1.1\">A word.</p>",
  );
}

/// In a URL, read expanded and then stringified (`\url`'s reader, `HyperVerbatim` for `\href`), the byte's robust
/// `\PackageError` raises its one Error and leaves no text: `url_string_messages` makes `\GenericError` err and gobble
/// its arguments, and the reader drops the `\protect` (`is_url_protect`; REGFIXV, OD #1112). Red (REGFIXN2): 0 Errors, href and text
/// `http://x.org/a\protect\GenericError{(inputenc)}…b`. pdflatex errs too (6 `!`; `\href` Fatal). Synthetic (review of
/// REGFIXN2); the PDF-key witnesses above stay silent. Repros unicode-catcodes/latin1_undefined_byte_in_{url,href}_errs.
#[test]
fn latin1_undefined_byte_in_a_url_errs_once() {
  for (tex, element) in [
    (
      &include_bytes!(
        "../../../tools/perfect_kernel/repros/unicode-catcodes/latin1_undefined_byte_in_url_errs.tex"
      )[..],
      "<p xml:id=\"p1.1\">A <ref class=\"ltx_url\" font=\"typewriter\" href=\"http://x.org/ab\">http://x.org/ab</ref> word.</p>",
    ),
    (
      &include_bytes!(
        "../../../tools/perfect_kernel/repros/unicode-catcodes/latin1_undefined_byte_in_href_errs.tex"
      )[..],
      "<p xml:id=\"p1.1\">A <ref class=\"ltx_href\" href=\"http://x.org/ab\">link</ref> word.</p>",
    ),
  ] {
    let (log, xml, ()) = convert_bytes_files_then(tex, &[], Some("ar5iv.sty"), |_, _| ());
    assert_eq!((error_count(&log), warning_count(&log)), (1, 0), "{log}");
    assert!(
      log.contains(
        "Error:latex:(inputenc) Package inputenc Error: Keyboard character used is undefined"
      ),
      "{log}"
    );
    assert_element(&xml, "p", &["xml:id=\"p1.1\""], element);
  }
}

/// lipics.cls, lipics-v2016/v2018 and lipicsmio (bound by name prefix) load no comment.sty; lipics-v2019.cls:756-757 and
/// later do, and exclude `{CCSXML}`. Red (REGFIXJ3): comment.sty for every version refused the paper's own
/// `\newcommand{\comment}[1]{}`, and `\comment{…}` opened comment.sty's environment to the end of the file (1
/// Error:unexpected:EOF, the last paragraph lost). Witnesses 1511.06260, 1705.05828, 1711.02032, 1803.04513; the
/// v2019 copies 1610.07530, 1902.11263.
#[test]
fn lipics_comment_sty_from_v2019_on() {
  let (log, xml) = convert_files_with(
    repro!("class-bindings/lipics_v2018/lipics_v2018_own_comment_command.tex"),
    &[(
      "lipics-v2018.cls",
      repro!("class-bindings/lipics_v2018/lipics-v2018.cls"),
    )],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    "<p xml:id=\"p1.1\">Kept  text.</p>",
  );
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p2.1\""],
    "<p xml:id=\"p2.1\">Last paragraph.</p>",
  );
  let v2019 = repro!("class-bindings/lipics_v2018/lipics_v2019_excludes_ccsxml.tex");
  let (log, xml) = convert_with(v2019, Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    "<p xml:id=\"p1.1\">Text.</p>",
  );
  // lipics-v2021.cls:861-862 and v2024 alike, beside a shipped copy that loads nothing itself.
  for class in ["lipics-v2021", "lipics-v2024"] {
    let copy =
      format!("\\NeedsTeXFormat{{LaTeX2e}}\n\\ProvidesClass{{{class}}}[2024/01/01 test]\n");
    let (log, xml) = convert_files_with(
      &v2019.replace("{lipics-v2019}", &format!("{{{class}}}")),
      &[(format!("{class}.cls").as_str(), copy.as_str())],
      Some("ar5iv.sty"),
    );
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "{class}: {log}"
    );
    assert_element(
      &xml,
      "p",
      &["xml:id=\"p1.1\""],
      "<p xml:id=\"p1.1\">Text.</p>",
    );
  }
}
