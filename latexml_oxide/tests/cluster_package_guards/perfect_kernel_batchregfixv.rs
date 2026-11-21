//! Red/green guards for batch REGFIXV: the URL readers (`\url`'s reader, `HyperVerbatim`) expand under the ambient
//! `\protect`, as hyperref's `\hyper@normalise` (hyperref.sty:4604-4650), and `HyperVerbatim` lets the active `~` to
//! `\hyper@tilde` (:4622). Repros `tools/perfect_kernel/repros/package-bindings/hyperref_*_in_footnote.tex`; each
//! pdflatex-checked.
use latexml::util::test::{assert_element, convert_with, kpse_has};

use super::perfect_kernel_batch46::{error_count, warning_count};

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!("../../../tools/perfect_kernel/repros/", $path))
  };
}

/// An `\href` URL with `~`, read in a `\footnote`'s argument under babel: the `~` was tokenized active before the
/// reader, so `HyperVerbatim` lets it to `\hyper@tilde` (hyperref.sty:4622). Red (INT6D): Fatal:Timeout:PushbackLimit,
/// babel's `\active@prefix~` re-expanding under the empty `\protect`; REGFIXN2 href
/// `http://x.org/\unhbox\voidb@x\protect\penalty\@M\ y`. Witnesses 2605.13298, 2605.19761, 2605.28243, 2605.29955.
#[test]
fn href_tilde_in_a_footnote_under_babel() {
  let (log, xml) = convert_with(
    repro!("package-bindings/hyperref_href_babel_tilde_in_footnote.tex"),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "note",
    &["xml:id=\"footnote1\""],
    "<note mark=\"1\" role=\"footnote\" xml:id=\"footnote1\"><tags><tag>1</tag><tag role=\"autoref\">footnote\u{a0}\
     1<text xml:id=\"footnote1.1\"/></tag><tag role=\"refnum\">1</tag><tag role=\"typerefnum\">footnote \
     1</tag></tags>See <ref class=\"ltx_href\" href=\"http://x.org/~y\">site</ref>.</note>",
  );
}

/// A babel shorthand (slovak `-`) tokenized active before the reader, in `\url` and `\href` in a `\footnote`: under the
/// ambient `\protect` babel's `\active@prefix` (babel.def:1418-1442) lets it expand to its character. Red (INT6D):
/// Fatal:Timeout:PushbackLimit, the empty `\protect` of `url_string_messages` making `\protect-` re-expand. Witness
/// 2605.16660. Skipped, visibly, where babel's slovak.ldf (babel-slovak) is not in the TeX Live: babel then errs on the
/// option itself (Ubuntu's texlive-lang-european ships only the `babel-slovak.tex` locale).
#[test]
fn babel_shorthand_in_a_footnote_url() {
  if !kpse_has("slovak.ldf") {
    eprintln!(
      "SKIP babel_shorthand_in_a_footnote_url: slovak.ldf (babel-slovak) is not in this TeX Live"
    );
    return;
  }
  let (log, xml) = convert_with(
    repro!("package-bindings/hyperref_url_babel_shorthand_in_footnote.tex"),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "note",
    &["xml:id=\"footnote1\""],
    "<note mark=\"1\" role=\"footnote\" xml:id=\"footnote1\"><tags><tag>1</tag><tag role=\"autoref\">footnote\u{a0}\
     1<text xml:id=\"footnote1.1\"/></tag><tag role=\"refnum\">1</tag><tag role=\"typerefnum\">footnote \
     1</tag></tags>See <ref class=\"ltx_url\" font=\"typewriter\" \
     href=\"http://x.org/a-b\">http://x.org/a-b</ref> and <ref class=\"ltx_href\" \
     href=\"http://x.org/c-d\">site</ref>.</note>",
  );
}
