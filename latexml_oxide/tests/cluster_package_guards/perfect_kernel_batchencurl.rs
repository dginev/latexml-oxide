//! Red/green guards for batch ENCURL: babel's safe actives in the URL readers (hyperref.sty:4627), utf8.def's error for
//! a control character (utf8.def:122-128, :164-170), inputenc's activated ranges (inputenc.sty:82-85) and xargs's
//! space-restoring peek (xargs.sty:64-87). Repros under `tools/perfect_kernel/repros/`; each pdflatex-checked.
use latexml::util::test::{assert_element, convert_bytes_files_then, convert_with, kpse_has};

use super::perfect_kernel_batch46::{error_count, warning_count};

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!("../../../tools/perfect_kernel/repros/", $path))
  };
}

/// The `Error:` lines of `log` that carry `needle`.
fn errors_with(log: &str, needle: &str) -> usize {
  log
    .lines()
    .filter(|line| line.starts_with("Error:") && line.contains(needle))
    .count()
}

/// ngerman's `"`, tokenized in a `\footnote`'s argument before `\href`'s reader, is babel-german's unexpandable
/// dispatcher: `HyperVerbatim` stringifies it, as `\url`'s reader does, so the URL keeps `"o` (pdflatex's URI). Red
/// (INT7b): href `http://x.org/ö`.
#[test]
fn href_german_shorthand_in_a_footnote_is_its_character() {
  if !kpse_has("ngermanb.ldf") {
    eprintln!(
      "SKIP href_german_shorthand_in_a_footnote_is_its_character: ngermanb.ldf (babel-german) is absent"
    );
    return;
  }
  let (log, xml) = convert_with(
    repro!("package-bindings/hyperref_url_babel_safe_actives_in_footnote.tex"),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    "<p xml:id=\"p1.1\">A<note mark=\"1\" role=\"footnote\" xml:id=\"footnote1\"><tags><tag>1</tag><tag role=\"autoref\">Fu\u{df}note\u{a0}1<text xml:id=\"footnote1.1\"/></tag><tag role=\"refnum\">1</tag><tag role=\"typerefnum\">footnote \
     1</tag></tags><ref class=\"ltx_href\" href=\"http://x.org/&quot;o\">s</ref> and <ref \
     class=\"ltx_url\" font=\"typewriter\" href=\"http://x.org/&quot;a\">http://x.org/\"a</ref>.</note>\nB <ref \
     class=\"ltx_href\" href=\"http://x.org/&quot;u\">t</ref>.</p>",
  );
}

/// A babel shorthand of babel's own `\active@char` machinery (raw spanish.ldf's `"`), tokenized before the reader:
/// under `\Hy@safe@activestrue` (hyperref.sty:4627, :362) babel.def:1394-1400 makes it `\normal@char"`, the character,
/// in `\url` and `\href`. Red (INT7b): 7 Errors, the footnote's href `…e.\setbox\z@\hbox8…`.
#[test]
fn url_readers_set_babel_safe_actives() {
  if !kpse_has("spanish.ldf") {
    eprintln!("SKIP url_readers_set_babel_safe_actives: spanish.ldf (babel-spanish) is absent");
    return;
  }
  let (log, xml) = convert_with(
    repro!("package-bindings/hyperref_url_babel_spanish_shorthands_in_footnote.tex"),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    "<p xml:id=\"p1.1\">A<note mark=\"1\" role=\"footnote\" xml:id=\"footnote1\"><tags><tag>1</tag><tag role=\"autoref\">Nota a pie de p\u{e1}gina\u{a0}1<text xml:id=\"footnote1.1\"/></tag><tag role=\"refnum\">1</tag><tag role=\"typerefnum\">footnote \
     1</tag></tags><ref class=\"ltx_href\" \
     href=\"http://x.org/a:b;c!d?e&quot;o-f'g\">s</ref> and <ref class=\"ltx_url\" font=\"typewriter\" \
     href=\"http://x.org/a:b;c!d?e&quot;o-f'g\">http://x.org/a:b;c!d?e\"o-f'g</ref>.</note>\nB <ref class=\"ltx_href\" \
     href=\"http://x.org/a:b;c!d?e&quot;o-f'g\">t</ref> <ref class=\"ltx_url\" font=\"typewriter\" \
     href=\"http://x.org/a:b;c!d?e&quot;o-f'g\">http://x.org/a:b;c!d?e\"o-f'g</ref>.</p>",
  );
}

/// After `[utf8]{inputenc}`, a C1 control (U+0080-U+009F, no `\u8:` name) and a C0 control (utf8.def:164-170) err
/// "Unicode character … not set up for use with LaTeX" and are dropped, one Error each, as in pdflatex. Red (INT7b): 0
/// errors, the C1 controls in the text and the C0 ones as OT1 glyphs.
#[test]
fn utf8_control_character_errs_and_is_dropped() {
  let (log, xml) = convert_with(
    repro!("unicode-catcodes/utf8_inputenc_control_characters_err.tex"),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (9, 0), "{log}");
  for code in [
    "0092", "0085", "0093", "0094", "0080", "0001", "000B", "001A", "001F",
  ] {
    assert_eq!(
      errors_with(&log, &format!("(U+{code})")),
      1,
      "U+{code}: {log}"
    );
  }
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    "<p xml:id=\"p1.1\">Its a test and quoted.</p>",
  );
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p2.1\""],
    "<p xml:id=\"p2.1\"><text font=\"bold\" xml:id=\"p2.1.1\">AB</text></p>",
  );
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p3.1\""],
    "<p xml:id=\"p3.1\">CDEFG.</p>",
  );
}

/// `\DeclareUnicodeCharacter{0092}` replaces the error's meaning; a C1 byte of a line that is not UTF-8 is an invalid
/// byte, kept as its Latin-1 character with the Info (OXIDIZED_DESIGN_DIVERGENCES #481), not the code point's error;
/// a comment's C1 character is never read. Red (INT7b): 0 errors, U+0092-U+0094 in the text.
#[test]
fn utf8_c1_declared_and_byte_line_keep_their_text() {
  let (log, xml, ()) = convert_bytes_files_then(
    include_bytes!(
      "../../../tools/perfect_kernel/repros/unicode-catcodes/utf8_c1_declared_and_byte_line.tex"
    ),
    &[],
    Some("ar5iv.sty"),
    |_, _| (),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (2, 0), "{log}");
  assert_eq!(
    (errors_with(&log, "(U+0093)"), errors_with(&log, "(U+0094)")),
    (1, 1),
    "{log}"
  );
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    "<p xml:id=\"p1.1\">It\u{2019}s declared, not.</p>",
  );
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p2.1\""],
    "<p xml:id=\"p2.1\">Byte line: \u{93}q\u{94} caf\u{e9}.</p>",
  );
}

/// A declaration made before `[utf8]{inputenc}` — in the preamble or a package, C1 and C0 — keeps its meaning through
/// utf8.def's (re)load and a later `\inputencoding{utf8}`: the declaration records `\u8:<character>` (utf8.def:207-221),
/// which the activated character reads (utf8.def:93-108); in a `\csname` a C1 character is itself (utf8.def:55-60). Red
/// (ENCURL r1): 22 Errors, the declared characters dropped, the names' errors in the text; INT7b: 0 Errors, "A dont  Δ
/// (two) Λ.".
#[test]
fn utf8_declared_before_inputenc_keeps_its_meaning() {
  let (log, xml, ()) = convert_bytes_files_then(
    include_bytes!(
      "../../../tools/perfect_kernel/repros/unicode-catcodes/utf8_declared_before_inputenc_keeps_meaning.tex"
    ),
    &[(
      "declc.sty",
      include_bytes!("../../../tools/perfect_kernel/repros/unicode-catcodes/declc.sty"),
    )],
    Some("ar5iv.sty"),
    |_, _| (),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  for (id, lead) in [("p1.1", "A"), ("p2.1", "B")] {
    assert_element(
      &xml,
      "p",
      &[&format!("xml:id=\"{id}\"")],
      &format!("<p xml:id=\"{id}\">{lead} \u{2018}don\u{2019}t \u{a1} (one) (two) (three).</p>"),
    );
  }
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p3.1\""],
    "<p xml:id=\"p3.1\">P Y Z Q.</p>",
  );
}

/// inputenc.sty:82-85 activates 1-8, 11, 14-31 and 127-255 as `\@inpenc@undefined`; `^^_` and `^^?` were missing
/// (Perl's inputenc.sty.ltxml:48-51, KNOWN_PERL_ERRORS #1309), printed as OT1's "Ø" and "¨". `^^@` keeps Perl's
/// activation, under a byte encoding an Error where latex.ltx:22189's invalid `^^@` errs. Red (INT7b): 1 Error, "AØB¨CD".
#[test]
fn inputenc_activates_unit_separator_and_delete() {
  let (log, xml) = convert_with(
    repro!("unicode-catcodes/inputenc_loop_covers_us_and_del.tex"),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (3, 0), "{log}");
  assert_eq!(
    errors_with(&log, "Keyboard character used is undefined"),
    3,
    "{log}"
  );
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    "<p xml:id=\"p1.1\">ABCD</p>",
  );
}

/// A later optional argument of an xargs command (`\xargs@read`'s, xargs.sty:205-212) peeks past spaces and puts them
/// back where no `[` follows (`\xargs@ifnextchar`, :64-87): `\ry{i} \ry{i}[j]` is "Sie Sij"; with `usedefault`, `[]`
/// is the default. Red (INT7b, and Perl): "SieSij", "Yqznext", "Zaecx", "Uadb" (KNOWN_PERL_ERRORS #1304).
#[test]
fn xargs_later_optional_puts_spaces_back() {
  let (log, xml) = convert_with(
    repro!("macro-state/xargs_later_optional_keeps_spaces.tex"),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    "<p xml:id=\"p1.1\">Sie Sij Sik end.</p>",
  );
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p2.1\""],
    "<p xml:id=\"p2.1\">Yqz next Yqr Zaec x Zmnc y Zap w</p>",
  );
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p3.1\""],
    "<p xml:id=\"p3.1\">Uad b Uad c Uav d.</p>",
  );
}

/// biblatex stores a `.bbl` `\field{abstract}` (biblatex.sty:7871 `\blx@bbl@fielddef`) and its styles never print it:
/// a C1 control there (cp1252 mojibake, arXiv 2302.06923, 2304.01940) raises no utf8.def error, and the bib-extract
/// keeps the stored text; the same kind of character in the text errs (U+0083), as in pdflatex. The `.bib` path and
/// `keywords`/`contents` share the handler (`\bib@field@extract`). Red (ENCURL f10be4db4c6): 3 Errors, 2 for U+009D.
#[test]
fn bbl_abstract_c1_is_stored_data() {
  let (log, xml, ()) = convert_bytes_files_then(
    include_bytes!(
      "../../../tools/perfect_kernel/repros/index-bib/biblatex_bbl_abstract_c1_is_stored_data.tex"
    ),
    &[(
      "t.bbl",
      include_bytes!(
        "../../../tools/perfect_kernel/repros/index-bib/biblatex_bbl_abstract_c1_is_stored_data.bbl"
      ),
    )],
    Some("ar5iv.sty"),
    |_, _| (),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (1, 0), "{log}");
  assert_eq!(errors_with(&log, "(U+0083)"), 1, "{log}");
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    "<p xml:id=\"p1.1\">Text.</p>",
  );
  assert_element(
    &xml,
    "bibentry",
    &["key=\"k\""],
    "<bibentry key=\"k\" type=\"article\" xml:id=\"bib.bib1\"><bib-name role=\"author\"><surname>Doe</surname><givenname>Jane</givenname></bib-name><bib-extract \
     role=\"abstract\">On class \u{e2}\u{20ac}\u{9d}cat\u{e2}\u{20ac}\u{9d} it fails.</bib-extract><bib-title>A \
     Title</bib-title><bib-date role=\"publication\">2021</bib-date><bib-data role=\"self\" type=\"BibTeX\">@article{k,\n    author = {Doe, \
     Jane},\n  abstract = {On class \u{e2}\u{20ac}\u{9d}cat\u{e2}\u{20ac}\u{9d} it fails.},\n     title = {A Title},\n      date = \
     {2021}}\n</bib-data></bibentry>",
  );
}
