//! W14 package leads: listing files that are missing or not UTF-8
//! (`listings_read_raw_file`), nomencl's `nomentbl` unit column, and siunitx
//! unit input that is not all unit macros.
//! Repros under `tools/perfect_kernel/repros/singletons/`.
use std::{path::Path, process::Command};

use super::perfect_kernel_batch46::convert;

/// Every `Error:`/`Fatal:` line, whatever its category (the shared
/// `error_count` reads `[A-Za-z_]+` and so misses `Error:I/O:`).
fn diagnostic_count(log: &str, kind: &str) -> usize {
  let re = regex::Regex::new(&format!(r"{kind}:[^:\s]+:")).unwrap();
  log.lines().filter(|l| re.is_match(l)).count()
}

/// Convert `tex` with the raw preload after writing `files` as BYTES beside it
/// (a latin1 data file is not a `&str`). Returns (ANSI-stripped stderr, XML).
fn convert_with_byte_files(tex: &str, files: &[(&str, &[u8])]) -> (String, String) {
  let bin = env!("CARGO_BIN_EXE_latexml_oxide");
  assert!(Path::new(bin).is_file(), "binary not staged at {bin}");
  let workdir = tempfile::tempdir().expect("create tempdir");
  std::fs::write(workdir.path().join("t.tex"), tex).expect("write t.tex");
  for (name, bytes) in files {
    std::fs::write(workdir.path().join(name), bytes).expect("write data file");
  }
  let output = Command::new(bin)
    .args([
      "t.tex",
      "--dest",
      "t.xml",
      "--nocomments",
      "--timeout=110",
      "--preload=[rawstyles,rawclasses]latexml.sty",
    ])
    .current_dir(workdir.path())
    .output()
    .expect("spawn latexml_oxide");
  let stderr = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
  let xml = std::fs::read_to_string(workdir.path().join("t.xml")).unwrap_or_default();
  (stderr, xml)
}

/// A listing file that is nowhere is Perl's `Error('I/O', …)`
/// (listings.sty.ltxml:333) and pdflatex's "Package Listings Error: File …
/// not found" (listings.sty:2074-2086), for `\lstinputlisting` and for
/// tcolorbox's direct `\tcbinputlisting` alike; it was a warning. Witness:
/// crossreftools_doc.tex:587/:899 (files the package does not ship).
#[test]
fn missing_listing_file_is_an_io_error() {
  let (log, xml) = convert(
    include_str!(
      "../../../tools/perfect_kernel/repros/singletons/listings_missing_file_io_error.tex"
    ),
    true,
  );
  let io: Vec<&str> = log
    .lines()
    .filter(|l| l.starts_with("Error:I/O:"))
    .collect();
  assert_eq!(
    io,
    [
      "Error:I/O:listings-nonexistent-a.txt Can't read listings file 'listings-nonexistent-a.txt'",
      "Error:I/O:listings-nonexistent-b.txt Can't read listings file 'listings-nonexistent-b.txt'",
    ],
    "{log}"
  );
  assert_eq!(diagnostic_count(&log, "(Error|Fatal)"), 2, "{log}");
  assert_eq!(diagnostic_count(&log, "Warning"), 0, "{log}");
  latexml::util::test::assert_element(
    &xml,
    "listing",
    &[r#"dataname="listings-nonexistent-a.txt""#],
    r#"<listing class="ltx_lstlisting" dataencoding="base64" datamimetype="text/plain" dataname="listings-nonexistent-a.txt"/>"#,
  );
  latexml::util::test::assert_element(
    &xml,
    "listing",
    &[r#"dataname="listings-nonexistent-b.txt""#],
    r#"<listing class="ltx_lstlisting" dataencoding="base64" datamimetype="text/plain" dataname="listings-nonexistent-b.txt"/>"#,
  );
}

/// A latin1 listing file is read as the Mouth reads a file (UTF-8, else each
/// line's Latin-1 image); `read_to_string` refused it and the listing came out
/// empty with no diagnostic. pdflatex under `inputenc[latin1]`: "café = 1".
#[test]
fn latin1_listing_file_is_read() {
  let (log, xml) = convert_with_byte_files(
    include_str!("../../../tools/perfect_kernel/repros/singletons/listings_latin1_file.tex"),
    &[(
      "lstlatin1.txt",
      include_bytes!("../../../tools/perfect_kernel/repros/singletons/lstlatin1.txt"),
    )],
  );
  assert_eq!(diagnostic_count(&log, "(Error|Fatal)"), 0, "{log}");
  assert_eq!(diagnostic_count(&log, "Warning"), 0, "{log}");
  latexml::util::test::assert_element(
    &xml,
    "listing",
    &[r#"dataname="lstlatin1.txt""#],
    r#"<listing class="ltx_lstlisting" data="Y2Fmw6kgPSAxCnNlY29uZCBsaW5lCg==" dataencoding="base64" datamimetype="text/plain" dataname="lstlatin1.txt"><listingline xml:id="lstnumberx1"><text class="ltx_lst_identifier">caf</text>é<text class="ltx_lst_space"> </text>=<text class="ltx_lst_space"> </text>1</listingline><listingline xml:id="lstnumberx2"><text class="ltx_lst_identifier">second</text><text class="ltx_lst_space"> </text><text class="ltx_lst_identifier">line</text></listingline></listing>"#,
  );
}

/// `nomentbl`'s unit column is `\unit{#4}` (nomencl.sty:232), so `\meter` is
/// the siunitx unit "m" as pdflatex prints it; bare it was siunitx's `\relax`
/// and the unit vanished at 0 errors. Witness: nomencl sample03.tex:15-30.
#[test]
fn nomentbl_unit_column_is_a_siunitx_unit() {
  let (log, xml) = convert(
    include_str!(
      "../../../tools/perfect_kernel/repros/singletons/nomencl_nomentbl_unit_siunitx.tex"
    ),
    true,
  );
  assert_eq!(diagnostic_count(&log, "(Error|Fatal)"), 0, "{log}");
  assert_eq!(diagnostic_count(&log, "Warning"), 0, "{log}");
  latexml::util::test::assert_element(
    &xml,
    "glossaryphrase",
    &[r#"role="unit""#],
    r#"<glossaryphrase key="nomencl.1" role="unit"><Math mode="inline" tex="\mathrm{m}" text="meter" xml:id="m3"><XMath><XMTok class="ltx_unit" meaning="meter" role="ID">m</XMTok></XMath></Math></glossaryphrase>"#,
  );
  // A blank unit (`{ }`) keeps no unit phrase, as `\unit{ }` prints nothing.
  latexml::util::test::assert_element(
    &xml,
    "glossarydefinition",
    &[r#"key="nomencl.2""#],
    r#"<glossarydefinition inlist="nomenclature" key="nomencl.2"><glossaryphrase key="nomencl.2" role="sort">zii</glossaryphrase><glossaryphrase key="nomencl.2" role="name">i</glossaryphrase><glossaryphrase key="nomencl.2" role="description">in</glossaryphrase></glossarydefinition>"#,
  );
}

/// Unit input holding anything but unit macros is LITERAL, in siunitx
/// (`\__siunitx_unit_if_symbolic:n`, siunitx.sty:6596-6613) and in Perl
/// (six_convertUnits, siunitx.sty.ltxml:903-927): the units read before the
/// `(` were kept and the rest dropped ("W^{-1}"). A literal pre-power raises
/// the item after it, a braced group or an upright letter (siunitx.sty
/// :6633-6641), and a style prints nothing (:6698-6700). pdflatex: W/(m² K),
/// 1 meV/c², km², 1 c³m, m³/(s). The all-unit inputs stay symbolic.
/// Witnesses: nomencl sample03.tex:13; arXiv 2602.18218 (main1.tex:408).
#[test]
fn siunitx_mixed_unit_input_is_literal() {
  let (log, xml) = convert(
    include_str!("../../../tools/perfect_kernel/repros/singletons/siunitx_mixed_unit_literal.tex"),
    true,
  );
  assert_eq!(diagnostic_count(&log, "(Error|Fatal)"), 0, "{log}");
  assert_eq!(diagnostic_count(&log, "Warning"), 0, "{log}");
  latexml::util::test::assert_element(
    &xml,
    "Math",
    &[r#"xml:id="p1.m1""#],
    r#"<Math mode="inline" tex="\mathrm{W}\mathrm{/}\mathrm{(}{\mathrm{m}}^{2}\mathrm{K}\mathrm{)}" text="W / (m ^ 2 * K)" xml:id="p1.m1"><XMath><XMApp><XMTok meaning="divide" role="MULOP">/</XMTok><XMTok role="UNKNOWN">W</XMTok><XMDual><XMRef idref="p1.m1.1"/><XMWrap><XMTok role="OPEN" stretchy="false">(</XMTok><XMApp xml:id="p1.m1.1"><XMTok meaning="times" role="MULOP">⁢</XMTok><XMApp><XMTok role="SUPERSCRIPTOP" scriptpos="post1"/><XMTok role="UNKNOWN">m</XMTok><XMTok fontsize="70%" meaning="2" role="NUMBER">2</XMTok></XMApp><XMTok role="UNKNOWN">K</XMTok></XMApp><XMTok role="CLOSE" stretchy="false">)</XMTok></XMWrap></XMDual></XMApp></XMath></Math>"#,
  );
  latexml::util::test::assert_element(
    &xml,
    "Math",
    &[r#"xml:id="p1.m2""#],
    r#"<Math mode="inline" tex="\mathrm{m}\text{\,}{\mathrm{s}}^{-1}" text="meter * power@(second, - 1)" xml:id="p1.m2"><XMath><XMApp><XMText meaning="times" role="MULOP" xml:id="p1.m2.1"> </XMText><XMTok class="ltx_unit" meaning="meter" role="ID">m</XMTok><XMApp xml:id="p1.m2.3"><XMTok meaning="power" role="SUPERSCRIPTOP" scriptpos="post1"/><XMTok class="ltx_unit" meaning="second" role="ID">s</XMTok><XMApp><XMTok fontsize="70%" meaning="minus" role="ADDOP">-</XMTok><XMTok fontsize="70%" meaning="1" role="NUMBER">1</XMTok></XMApp></XMApp></XMApp></XMath></Math>"#,
  );
  latexml::util::test::assert_element(
    &xml,
    "Math",
    &[r#"xml:id="p1.m3""#],
    r#"<Math mode="inline" tex="3\text{\,}\mathrm{kg}" text="3 * kilogram" xml:id="p1.m3"><XMath><XMApp><XMText meaning="times" role="MULOP" xml:id="p1.m3.1"> </XMText><XMTok meaning="3" role="NUMBER">3</XMTok><XMTok class="ltx_unit" meaning="kilogram" role="ID">kg</XMTok></XMApp></XMath></Math>"#,
  );
  for (id, expected) in [
    (
      "p1.m4",
      r#"<Math mode="inline" tex="1\text{\,}\mathrm{m}\mathrm{eV}\mathrm{/}{c^{2}}" text="1 * (meV / c ^ 2)" xml:id="p1.m4"><XMath><XMApp><XMText meaning="times" role="MULOP" xml:id="p1.m4.1"> </XMText><XMTok meaning="1" role="NUMBER">1</XMTok><XMApp xml:id="p1.m4.3"><XMTok meaning="divide" role="MULOP">/</XMTok><XMTok role="UNKNOWN">meV</XMTok><XMApp><XMTok role="SUPERSCRIPTOP" scriptpos="post2"/><XMTok font="italic" role="UNKNOWN">c</XMTok><XMTok fontsize="70%" meaning="2" role="NUMBER">2</XMTok></XMApp></XMApp></XMApp></XMath></Math>"#,
    ),
    (
      "p1.m5",
      r#"<Math mode="inline" tex="{{\mathrm{k}\mathrm{m}}}^{2}" text="km ^ 2" xml:id="p1.m5"><XMath><XMApp><XMTok role="SUPERSCRIPTOP" scriptpos="post1"/><XMTok role="UNKNOWN">km</XMTok><XMTok fontsize="70%" meaning="2" role="NUMBER">2</XMTok></XMApp></XMath></Math>"#,
    ),
    (
      "p1.m6",
      r#"<Math mode="inline" tex="1\text{\,}{\mathrm{c}}^{3}\mathrm{m}" text="1 * (c ^ 3 * m)" xml:id="p1.m6"><XMath><XMApp><XMText meaning="times" role="MULOP" xml:id="p1.m6.1"> </XMText><XMTok meaning="1" role="NUMBER">1</XMTok><XMApp xml:id="p1.m6.3"><XMTok meaning="times" role="MULOP">⁢</XMTok><XMApp><XMTok role="SUPERSCRIPTOP" scriptpos="post1"/><XMTok role="UNKNOWN">c</XMTok><XMTok fontsize="70%" meaning="3" role="NUMBER">3</XMTok></XMApp><XMTok role="UNKNOWN">m</XMTok></XMApp></XMApp></XMath></Math>"#,
    ),
    (
      "p1.m7",
      r#"<Math mode="inline" tex="{\mathrm{m}}^{3}\mathrm{/}\mathrm{(}\mathrm{s}\mathrm{)}" text="m ^ 3 / s" xml:id="p1.m7"><XMath><XMApp><XMTok meaning="divide" role="MULOP">/</XMTok><XMApp><XMTok role="SUPERSCRIPTOP" scriptpos="post1"/><XMTok role="UNKNOWN">m</XMTok><XMTok fontsize="70%" meaning="3" role="NUMBER">3</XMTok></XMApp><XMDual><XMRef idref="p1.m7.1"/><XMWrap><XMTok role="OPEN" stretchy="false">(</XMTok><XMTok role="UNKNOWN" xml:id="p1.m7.1">s</XMTok><XMTok role="CLOSE" stretchy="false">)</XMTok></XMWrap></XMDual></XMApp></XMath></Math>"#,
    ),
  ] {
    latexml::util::test::assert_element(&xml, "Math", &[&format!(r#"xml:id="{id}""#)], expected);
  }
}
