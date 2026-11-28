//! Red/green guards for batch COUNTERCHAIN: a theorem sharing the counter of a type that shares one in turn steps the
//! end of the chain (`counter_for_type`), as LaTeX's aliased `\c@` does; thmtools' sibling (thm-autoref.sty:31-44)
//! aliases the counter and names the theorem for `\autoref`. Repros `tools/perfect_kernel/repros/package-bindings/`;
//! each pdflatex-checked.
use latexml::util::test::convert_files_with;

use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch78i::html_of,
};

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/package-bindings/",
      $path
    ))
  };
}

/// The theorem heads, the equation tags and the text of the `See …` paragraph of an HTML page, in order.
fn heads_tags_and_sentence(html: &str) -> (Vec<String>, Vec<String>, String) {
  let all = |re: &str| -> Vec<String> {
    regex::Regex::new(re)
      .unwrap()
      .captures_iter(html)
      .map(|c| c[1].to_string())
      .collect()
  };
  let heads = all(r#"ltx_tag_theorem"><span[^>]*>([^<]*)</span>"#);
  let tags = all(r#"ltx_tag_equation[^"]*">([^<]*)<"#);
  let sentence = regex::Regex::new(r"(?s)See .*?</p>")
    .unwrap()
    .find(html)
    .map(|m| {
      let text = regex::Regex::new("<[^>]+>")
        .unwrap()
        .replace_all(m.as_str(), "");
      text.split_whitespace().collect::<Vec<_>>().join(" ")
    })
    .unwrap_or_default();
  (heads, tags, sentence)
}

/// thmtools `\declaretheorem[sibling=definition]{remark}` over `[sibling=equation]{definition}`, and a `\newtheorem`
/// sharing the remark's counter: thm-autoref's `\@counteralias` makes `\c@remark` the equation counter, so every one
/// steps it and `\autoref` names each by its heading (2009.11843: 149+149 Warnings, "Remark 1" where pdflatex prints
/// "Remark 2"). pdflatex: 0 errors, the heads and sentence below.
#[test]
fn thmtools_sibling_of_a_sibling_shares_its_counter() {
  let html = html_of(
    repro!("thmtools_sibling_of_a_sibling_shares_its_counter.tex"),
    &[],
  );
  let (heads, tags, sentence) = heads_tags_and_sentence(&html);
  assert_eq!(heads, ["Definition 1", "Remark 2", "Note 3"], "{html}");
  assert_eq!(tags, ["(4)"], "{html}");
  assert_eq!(
    sentence, "See 1, 2, 3, 4; Definition 1, Remark 2, Note 3; 4, 4, 4.",
    "{html}"
  );
}

/// `\newtheorem{remark}[remcnt]` over aliascnt's `\newaliascnt{remcnt}{theoremlike}` (1809.01129's
/// `\NewLinkedRemark`): the remark steps the root counter and takes its ids, while hyperref names it by `remcnt`, the
/// counter it was declared with (`\remcntautorefname`). pdflatex: 0 errors, "Remark 2", "Remark 3", the sentence below.
#[test]
fn newtheorem_over_an_aliascnt_alias_steps_its_root() {
  let html = html_of(
    repro!("newtheorem_over_an_aliascnt_alias_steps_its_root.tex"),
    &[],
  );
  let (heads, _, sentence) = heads_tags_and_sentence(&html);
  assert_eq!(heads, ["Theoremlike 1", "Remark 2", "Remark 3"], "{html}");
  assert!(html.contains(r#"id="Thmtheoremlike3""#), "{html}");
  assert_eq!(sentence, "See 1, 2, 3; 1, Remark 2, Remark 3; 3.", "{html}");
}

/// The main `<tag>` of each object in `xml`, in order (not its title's).
fn object_tags(xml: &str) -> Vec<&str> {
  xml
    .split("<tag>")
    .skip(1)
    .filter_map(|rest| rest.split("</tag>").next())
    .filter(|tag| !tag.starts_with('<'))
    .collect()
}

/// A theorem sharing a counter no `\c@` names: pdflatex errs "No counter … defined" (latex.ltx/amsthm `\@othm`, and
/// aliasctr's `\@counteralias` under thmtools) and leaves the theorem undefined; here each is one Warning and the
/// theorem is declared, numbered by the counter that type shares where it shares one. Without thmtools nothing
/// aliases `\c@definition`, so `remark` follows `definition` to `equation`; under thmtools a sibling sharing a counter
/// (`lemma`) or none (`foo`, 2107.02579's `numberlike=theo`: four aliasctr Errors in round 1) is a Warning.
#[test]
fn theorem_sharing_a_counter_no_c_names_warns() {
  let (log, xml) = convert_files_with(
    repro!("newtheorem_over_a_shared_counter_type_warns.tex"),
    &[],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(
    log.contains(
      "Warning:undefined:definition No counter 'definition' defined; remark is numbered by its counter 'equation'"
    ),
    "{log}"
  );
  assert_eq!(
    object_tags(&xml),
    ["Definition 1", "Remark 2", "(3)"],
    "{xml}"
  );
  assert!(xml.contains(r#"labels="LABEL:r1" xml:id="S0.E2""#), "{xml}");

  let (log, xml) = convert_files_with(
    repro!("thmtools_sibling_with_no_counter_warns.tex"),
    &[],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 2), "{log}");
  for warning in [
    "Warning:undefined:lemma No counter 'lemma' defined; cor is numbered by its counter 'theorem'",
    "Warning:undefined:foo No counter 'foo' defined; baz shares no counter with it",
  ] {
    assert!(log.contains(warning), "{log}");
  }
  assert_eq!(object_tags(&xml), ["Theorem 1", "Corollary 2"], "{xml}");
}

/// thmtools `\declaretheorem[numbered=no,sibling=theorem]`, before and after `theorem`: thm-kv.sty:394-404 declares an
/// unnumbered theorem `*{name}{heading}`, so it takes no sibling, and nothing warns or aliases its counter (the
/// numbered `lemma` beside it is aliased). Distinct from the sibling guards: the `numbered=no` branch of
/// `\declaretheorem`. pdflatex: 0 errors, the heads and sentence below.
#[test]
fn thmtools_unnumbered_theorem_takes_no_sibling() {
  let html = html_of(
    repro!("thmtools_unnumbered_theorem_takes_no_sibling.tex"),
    &[],
  );
  let (heads, _, sentence) = heads_tags_and_sentence(&html);
  assert_eq!(
    heads,
    ["Theorem 1", "Remark", "Note", "Theorem 2"],
    "{html}"
  );
  assert_eq!(sentence, "See 1, 2; own, shared.", "{html}");
}

/// svjour3's `\spnewtheorem{conj}[theorem]` after thmtools and hyperref: thm-patch.sty patches `\newtheorem` alone, so
/// the conjecture gets no `\conjautorefname` and `\autoref` names it by `theorem`, the counter `\@spothm` steps, while
/// the `\newtheorem{prop}[theorem]` beside it is named by its heading (2302.09175's shape). Distinct: the definers
/// thmtools does not patch (llncs/sv `\spnewtheorem`, elsarticle's `\newproof`) run no thmtools hook. pdflatex (with
/// 2302.09175's svjour3.cls): 0 errors, the heads and sentence below.
#[test]
fn spnewtheorem_after_thmtools_is_not_patched() {
  let html = html_of(repro!("spnewtheorem_after_thmtools_is_not_patched.tex"), &[
  ]);
  let (heads, _, sentence) = heads_tags_and_sentence(&html);
  assert_eq!(
    heads,
    ["Theorem 1", "Conjecture 2", "Proposition 3"],
    "{html}"
  );
  assert_eq!(
    sentence, "See 1, 2, 3; Theorem 1, Theorem 2, Proposition 3.",
    "{html}"
  );
}
