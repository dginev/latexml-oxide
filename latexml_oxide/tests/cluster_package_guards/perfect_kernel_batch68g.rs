//! Red/green guards for batch 68g: llncs's theorem set numbers as llncs.cls numbers it under each numbering option
//! (default, envcountsame, envcountsect, both, envcountreset; llncs.cls:1142-1174, TL 2025 v2.26, alike in every
//! shipped copy v2.2-v2.26). The expected numbers are pdflatex's on each repro (KNOWN_PERL_ERRORS #743). Round 2: an
//! env `\@spothm` shares a counter with is an alias counter (v2.16+), which `\autoref` names by the env ("Lemma 2");
//! v2.24+'s section reset of the alias; the class's errors for an undefined counter and a redefinition; an aliascnt
//! alias is hyperref's type too. Round 3: the env's title is `\<env>name` (a renamed `\lemmaname`), and the alias is
//! aliascnt's for `\aliascntresetthe`.
use latexml::util::test::convert_with;

use super::perfect_kernel_batch46::{error_count, warning_count};

/// The XML with every whitespace run collapsed to one space.
fn squash(xml: &str) -> String { xml.split_whitespace().collect::<Vec<_>>().join(" ") }

/// A clean conversion of a class-bindings repro, squashed.
macro_rules! clean {
  ($path:literal) => {{
    let (log, xml) = convert_with(
      include_str!(concat!(
        "../../../tools/perfect_kernel/repros/class-bindings/",
        $path
      )),
      Some("ar5iv.sty"),
    );
    assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
    squash(&xml)
  }};
}

/// Every theorem's visible tag and refnum, in document order: `"Theorem 1/1"`, …
fn theorem_tags(xml: &str) -> Vec<String> {
  let mut tags = Vec::new();
  for chunk in xml.split("<theorem ").skip(1) {
    let chunk = chunk.split_once("</tags>").map_or(chunk, |(tags, _)| tags);
    let tag = |open: &str| {
      chunk
        .split_once(open)
        .and_then(|(_, rest)| rest.split_once("</tag>"))
        .map(|(t, _)| t.to_string())
    };
    let shown = tag("<tags> <tag>").unwrap_or_default();
    let refnum = tag(r#"<tag role="refnum">"#).unwrap_or_default();
    tags.push(format!("{shown}/{refnum}"));
  }
  tags
}

/// The theorems' tags, `"Theorem 1/1, Lemma 1/1, …"`, are `expected`.
fn expect_tags(xml: &str, expected: &str) {
  assert_eq!(theorem_tags(xml).join(", "), expected, "{xml}");
}

/// The tags of the theorem labelled `label` (what `\ref` and `\autoref` to it show): the run of `<tag>`s in its
/// `<tags>`, squashed.
fn labelled_tags(xml: &str, label: &str) -> String {
  let at = format!(r#"labels="LABEL:{label}""#);
  xml
    .split_once(at.as_str())
    .and_then(|(_, rest)| rest.split_once("<tags> "))
    .and_then(|(_, rest)| rest.split_once(" </tags>"))
    .map(|(tags, _)| tags.to_string())
    .unwrap_or_default()
}

/// The refs `\ref{t1}` and `\ref{t2}` show (the labelled targets' refnum, and typerefnum), the repro's "Ref …".
fn expect_refs(xml: &str, expected: [&str; 2]) {
  for (label, number) in ["t1", "t2"].into_iter().zip(expected) {
    assert_eq!(
      labelled_tags(xml, label),
      format!(
        r#"<tag>Theorem {number}</tag> <tag role="refnum">{number}</tag> <tag role="typerefnum">Theorem {number}</tag>"#
      ),
      "{label}: {xml}"
    );
  }
}

/// The default: `{theorem}` and every other env on its own counter, none within the section ("Theorem 3" in
/// section 2), the unnumbered `{claim}` and the paper's own `\spnewtheorem` alike.
#[test]
fn llncs_numbering_default() {
  let xml = clean!("llncs_theorem_numbering_default.tex");
  expect_tags(
    &xml,
    "Theorem 1/1, Lemma 1/1, Theorem 2/2, Lemma 2/2, Definition 1/1, Fact 1/1, MyY 1/1, Claim/, Theorem 3/3, Corollary 1/1, Remark 1/1, Fact 2/2",
  );
  expect_refs(&xml, ["1", "3"]);
  assert!(
    xml.contains(r#"<theorem class="ltx_theorem_theorem" inlist="thm theorem:theorem" labels="LABEL:t2" xml:id="Thmtheorem3"> <tags> <tag>Theorem 3</tag> <tag role="refnum">3</tag> <tag role="typerefnum">Theorem 3</tag> </tags>"#),
    "{xml}"
  );
  // The class's `{definition}` has an italic body (llncs.cls:1160 `{\bfseries}{\itshape}`; Perl's binding upright).
  assert!(
    xml.contains(r#"<title class="ltx_runin"><tag><text font="bold" xml:id="Thmdefinition1.1">Definition 1</text></tag></title> <para xml:id="Thmdefinition1.p1"> <p xml:id="Thmdefinition1.p1.1"><text font="italic" xml:id="Thmdefinition1.p1.1.1">e</text></p> </para> </theorem>"#),
    "{xml}"
  );
}

/// envcountsame: every env of the set, and a paper's `\spn@wtheorem{fact}`, numbers with the theorem counter.
#[test]
fn llncs_numbering_envcountsame() {
  let xml = clean!("llncs_theorem_numbering_envcountsame.tex");
  expect_tags(
    &xml,
    "Theorem 1/1, Lemma 2/2, Theorem 3/3, Lemma 4/4, Definition 5/5, Fact 6/6, MyY 1/1, Claim/, Theorem 7/7, Corollary 8/8, Remark 9/9, Fact 10/10",
  );
  expect_refs(&xml, ["1", "7"]);
  assert!(
    xml.contains(r#"<theorem class="ltx_theorem_fact" inlist="thm theorem:fact" xml:id="Thmtheorem10"> <tags> <tag>Fact 10</tag> <tag role="refnum">10</tag> <tag role="typerefnum">Fact 10</tag> </tags>"#),
    "{xml}"
  );
}

/// envcountsect: each env on its own counter within the section ("Lemma 0.1" before the first section; 0906.4121's
/// "Fact 3.1").
#[test]
fn llncs_numbering_envcountsect() {
  let xml = clean!("llncs_theorem_numbering_envcountsect.tex");
  expect_tags(
    &xml,
    "Theorem 0.1/0.1, Lemma 0.1/0.1, Theorem 1.1/1.1, Lemma 1.1/1.1, Definition 1.1/1.1, Fact 1.1/1.1, MyY 1/1, Claim/, Theorem 2.1/2.1, Corollary 2.1/2.1, Remark 2.1/2.1, Fact 2.1/2.1",
  );
  expect_refs(&xml, ["0.1", "2.1"]);
  assert!(
    xml.contains(r#"<theorem class="ltx_theorem_fact" inlist="thm theorem:fact" xml:id="S2.Thmfact1"> <tags> <tag>Fact 2.1</tag> <tag role="refnum">2.1</tag> <tag role="typerefnum">Fact 2.1</tag> </tags>"#),
    "{xml}"
  );
}

/// envcountsame + envcountsect: one theorem counter, within the section.
#[test]
fn llncs_numbering_envcountsame_envcountsect() {
  let xml = clean!("llncs_theorem_numbering_envcountsame_envcountsect.tex");
  expect_tags(
    &xml,
    "Theorem 0.1/0.1, Lemma 0.2/0.2, Theorem 1.1/1.1, Lemma 1.2/1.2, Definition 1.3/1.3, Fact 1.4/1.4, MyY 1/1, Claim/, Theorem 2.1/2.1, Corollary 2.2/2.2, Remark 2.3/2.3, Fact 2.4/2.4",
  );
  expect_refs(&xml, ["0.1", "2.1"]);
  assert!(
    xml.contains(r#"<theorem class="ltx_theorem_fact" inlist="thm theorem:fact" xml:id="S2.Thmtheorem4"> <tags> <tag>Fact 2.4</tag> <tag role="refnum">2.4</tag> <tag role="typerefnum">Fact 2.4</tag> </tags>"#),
    "{xml}"
  );
}

/// envcountreset: each env on its own counter, reset by the section but printed without it.
#[test]
fn llncs_numbering_envcountreset() {
  let xml = clean!("llncs_theorem_numbering_envcountreset.tex");
  expect_tags(
    &xml,
    "Theorem 1/1, Lemma 1/1, Theorem 1/1, Lemma 1/1, Definition 1/1, Fact 1/1, MyY 1/1, Claim/, Theorem 1/1, Corollary 1/1, Remark 1/1, Fact 1/1",
  );
  expect_refs(&xml, ["1", "1"]);
}

/// envcountsame with hyperref (2202.10090, 1612.00484): each env of the set, and a paper's
/// `\spnewtheorem{obs}[theorem]`, steps an alias of the theorem counter (llncs.cls v2.16+ `\@spothm`
/// `\newaliascnt{#1}{#2}`), so it shares the number and `\autoref` names it by the env: pdflatex "Theorem 1, Lemma 2,
/// Definition 3, Observation 4" (round 1: "Theorem 2, Theorem 3, Theorem 4").
#[test]
fn llncs_autoref_envcountsame() {
  let xml = clean!("llncs_autoref_envcountsame.tex");
  for (label, autoref) in [
    ("t1", "Theorem 1"),
    ("l1", "Lemma 2"),
    ("d1", "Definition 3"),
    ("o1", "Observation 4"),
  ] {
    let (name, number) = autoref.split_once(' ').unwrap();
    let id = format!("Thmtheorem{number}");
    assert_eq!(
      labelled_tags(&xml, label),
      format!(
        r#"<tag>{autoref}</tag> <tag role="autoref">{autoref}<text xml:id="{id}.1"/></tag> <tag role="refnum">{number}</tag> <tag role="typerefnum">{name} {number}</tag>"#
      ),
      "{label}: {xml}"
    );
  }
  assert!(
    xml.contains(r#"<theorem class="ltx_theorem_lemma" inlist="thm theorem:lemma" labels="LABEL:l1" xml:id="Thmtheorem2"> <tags> <tag>Lemma 2</tag> <tag role="autoref">Lemma 2<text xml:id="Thmtheorem2.1"/></tag> <tag role="refnum">2</tag> <tag role="typerefnum">Lemma 2</tag> </tags>"#),
    "{xml}"
  );
}

/// A shipped llncs.cls v2.2-v2.14 (`\@spothm` steps `#2` itself, no alias): hyperref's type is `theorem`, pdflatex
/// "Theorem 1, Theorem 2, Theorem 3, Theorem 4" with the copy (the copy decides).
#[test]
fn llncs_autoref_envcountsame_v214_copy() {
  let xml = clean!("llncs_autoref_envcountsame_v214_copy.tex");
  assert!(
    xml.contains(r#"<theorem class="ltx_theorem_lemma" inlist="thm theorem:lemma" labels="LABEL:l1" xml:id="Thmtheorem2"> <tags> <tag>Lemma 2</tag> <tag role="autoref">Theorem 2<text xml:id="Thmtheorem2.1"/></tag> <tag role="refnum">2</tag> <tag role="typerefnum">Lemma 2</tag> </tags>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<theorem class="ltx_theorem_obs" inlist="thm theorem:obs" labels="LABEL:o1" xml:id="Thmtheorem4"> <tags> <tag>Observation 4</tag> <tag role="autoref">Theorem 4<text xml:id="Thmtheorem4.1"/></tag> <tag role="refnum">4</tag> <tag role="typerefnum">Observation 4</tag> </tags>"#),
    "{xml}"
  );
}

/// envcountsect + envcountsame, `\@spothm{conj2}[hyp]` over a paper's own `{hyp}` (the review's sect2 probe): v2.24+
/// resets the section through the alias, so section 2 starts over, pdflatex "Hypothesis 1, Conj 2" in both sections
/// (round 1: "Hypothesis 3, Conj 4"; OXIDIZED_DESIGN_DIVERGENCES #614).
#[test]
fn llncs_spothm_section_reset() {
  let xml = clean!("llncs_spothm_section_reset.tex");
  expect_tags(
    &xml,
    "Lemma 0.1/0.1, Hypothesis 1/1, Conj 2/2, Lemma 1.1/1.1, Hypothesis 1/1, Conj 2/2, Theorem 2.1/2.1",
  );
  assert_eq!(
    labelled_tags(&xml, "l"),
    r#"<tag>Lemma 1.1</tag> <tag role="refnum">1.1</tag> <tag role="typerefnum">Lemma 1.1</tag>"#,
    "{xml}"
  );
  assert!(
    xml.contains(r#"<theorem class="ltx_theorem_conj2" inlist="thm theorem:conj2" labels="LABEL:c" xml:id="Thmhyp2a"> <tags> <tag>Conj 2</tag> <tag role="refnum">2</tag> <tag role="typerefnum">Conj 2</tag> </tags>"#),
    "{xml}"
  );
}

/// A shipped llncs.cls v2.16-v2.23 (v2.20's `\@spothm`: the alias without the section reset): pdflatex with the copy
/// numbers on, "Hypothesis 3, Conj 4" in section 2.
#[test]
fn llncs_spothm_section_reset_v220_copy() {
  let xml = clean!("llncs_spothm_section_reset_v220_copy.tex");
  expect_tags(
    &xml,
    "Lemma 0.1/0.1, Hypothesis 1/1, Conj 2/2, Lemma 1.1/1.1, Hypothesis 3/3, Conj 4/4, Theorem 2.1/2.1",
  );
  assert!(
    xml.contains(r#"<theorem class="ltx_theorem_conj2" inlist="thm theorem:conj2" labels="LABEL:c" xml:id="Thmhyp4"> <tags> <tag>Conj 4</tag> <tag role="refnum">4</tag> <tag role="typerefnum">Conj 4</tag> </tags>"#),
    "{xml}"
  );
}

/// The class's errors: `\@spothm`'s "No theorem environment `nosuch' defined" (an Error, no `{foo}`), and
/// `\@ifdefinable`'s "Command \lemma already defined" for an env redeclared without `\let\lemma\relax` (an Info,
/// as every kernel "already defined"; the first definition stands): pdflatex "Lemma 1", "Lemma2 1", "Theorem 2".
#[test]
fn llncs_spnewtheorem_errors() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/class-bindings/llncs_spnewtheorem_errors.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (1, 0), "{log}");
  assert!(
    log.contains("No theorem environment `nosuch' defined"),
    "{log}"
  );
  assert_eq!(log.matches("already defined").count(), 2, "{log}");
  let xml = squash(&xml);
  expect_tags(&xml, "Lemma 1/1, Lemma2 1/1, Theorem 2/2");
  assert!(!xml.contains("ltx_theorem_foo"), "{xml}");
}

/// aliascnt's `\newaliascnt{lemma}{theorem}` (2605.20695): hyperref's type is the alias, named by its own
/// `\<alias>autorefname`/`\<alias>name` or not at all — pdflatex "See 8, Theorem 1, 2, Proposition 3, Theorem 4." (the
/// plain `[theorem]` corollary is the counter's "Theorem"); 67int9 named the alias by its root, "Theorem 2".
#[test]
fn aliascnt_autoref_names_the_alias() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/package-bindings/aliascnt_autoref_names_the_alias.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let xml = squash(&xml);
  for (label, tags) in [
    (
      "t1",
      r#"<tag>Theorem 1</tag> <tag role="autoref">Theorem 1<text xml:id="Thmtheorem1.1"/></tag> <tag role="refnum">1</tag> <tag role="typerefnum">Theorem 1</tag>"#,
    ),
    (
      "l1",
      r#"<tag>Lemma 2</tag> <tag role="autoref">2<text xml:id="Thmtheorem2.1"/></tag> <tag role="refnum">2</tag> <tag role="typerefnum">Lemma 2</tag>"#,
    ),
    (
      "p1",
      r#"<tag>Proposition 3</tag> <tag role="autoref">Proposition 3<text xml:id="Thmtheorem3.1"/></tag> <tag role="refnum">3</tag> <tag role="typerefnum">Proposition 3</tag>"#,
    ),
    (
      "c1",
      r#"<tag>Corollary 4</tag> <tag role="autoref">Theorem 4<text xml:id="Thmtheorem4.1"/></tag> <tag role="refnum">4</tag> <tag role="typerefnum">Corollary 4</tag>"#,
    ),
    (
      "l2",
      r#"<tag>Lemma 8</tag> <tag role="autoref">8<text xml:id="Thmtheorem8.1"/></tag> <tag role="refnum">8</tag> <tag role="typerefnum">Lemma 8</tag>"#,
    ),
  ] {
    assert_eq!(labelled_tags(&xml, label), tags, "{label}: {xml}");
  }
}

/// An alias given a register of its own (2303.08726: `\let\c@case\relax\newcounter{case}` under envcountsame) steps
/// that register, not `theorem`: the lemmas number as pdflatex's "Lemma 1, Lemma 2, Lemma 3" (round 1: "Lemma 4,
/// Lemma 6", every case stepping the theorem counter). The cases print the alias's `\thecase` = `\thetheorem`, which
/// latex.ltx's `\@definecounter` keeps with its warning ("not changed"): "Case 1, Case 1, Case 2", as pdflatex (77v,
/// KNOWN_PERL_ERRORS #1009; Perl's NewCounter redefines it).
#[test]
fn llncs_alias_newcounter() {
  let (log, xml) = convert_with(
    include_str!("../../../tools/perfect_kernel/repros/class-bindings/llncs_alias_newcounter.tex"),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(
    log.contains("Command `\\thecase' already defined -- not changed"),
    "{log}"
  );
  let xml = squash(&xml);
  expect_tags(
    &xml,
    "Lemma 1/1, Case 1/1, Case 1/1, Lemma 2/2, Case 2/2, Lemma 3/3",
  );
  assert_eq!(
    labelled_tags(&xml, "l3"),
    r#"<tag>Lemma 3</tag> <tag role="refnum">3</tag> <tag role="typerefnum">Lemma 3</tag>"#,
    "{xml}"
  );
}

/// The head of the theorem whose opening tag contains `marker`: from `<theorem ` through its `</title>`, squashed.
fn theorem_head(xml: &str, marker: &str) -> String {
  xml
    .split("<theorem ")
    .skip(1)
    .find(|chunk| {
      chunk
        .split_once('>')
        .is_some_and(|(open, _)| open.contains(marker))
    })
    .and_then(|chunk| chunk.split_once("</title>"))
    .map(|(head, _)| format!("<theorem {head}</title>"))
    .unwrap_or_default()
}

/// Round 3: a paper loading aliascnt itself calls `\aliascntresetthe{lemma}` on the class's alias (llncs.cls:53
/// loads aliascnt; aliascnt.sty v1.5 :54-57 records the alias as `\AC@cnt@lemma` and `\cl@lemma`): pdflatex "See
/// Theorem 1, Lemma 2, ClaimX 3." and no error (68g r2: "`lemma' is not an alias counter").
#[test]
fn llncs_aliascntresetthe() {
  let xml = clean!("llncs_aliascntresetthe.tex");
  for (label, env, autoref) in [
    ("t1", "theorem", "Theorem 1"),
    ("l1", "lemma", "Lemma 2"),
    ("c1", "claimx", "ClaimX 3"),
  ] {
    let number = autoref.split_once(' ').unwrap().1;
    let id = format!("Thmtheorem{number}");
    assert_eq!(
      theorem_head(&xml, &format!(r#"labels="LABEL:{label}""#)),
      format!(
        r#"<theorem class="ltx_theorem_{env}" inlist="thm theorem:{env}" labels="LABEL:{label}" xml:id="{id}"> <tags> <tag>{autoref}</tag> <tag role="autoref">{autoref}<text xml:id="{id}.1"/></tag> <tag role="refnum">{number}</tag> <tag role="typerefnum">{autoref}</tag> </tags> <title class="ltx_runin"><tag><text font="bold" xml:id="{id}.2">{autoref}</text></tag></title>"#
      ),
      "{label}: {xml}"
    );
  }
}

/// Round 3: the class titles an env by `\csname <env>name\endcsname` (llncs.cls:1073/1082/1096/1122), so a paper's
/// `\renewcommand\lemmaname{Lemme}` renames its title, tag and autoref alike: pdflatex "Lemme 2", "Remarque 3",
/// "Preuve" (68g r2: titles "Lemma 2", "Remark 3", "Proof" under autorefs "Lemme 2", "Remarque 3").
#[test]
fn llncs_renamed_theorem_title() {
  let xml = clean!("llncs_renamed_theorem_title.tex");
  for (label, env, autoref) in [
    ("t1", "theorem", "Theorem 1"),
    ("l1", "lemma", "Lemme 2"),
    ("r1", "remark", "Remarque 3"),
  ] {
    let number = autoref.split_once(' ').unwrap().1;
    let id = format!("Thmtheorem{number}");
    assert_eq!(
      theorem_head(&xml, &format!(r#"labels="LABEL:{label}""#)),
      format!(
        r#"<theorem class="ltx_theorem_{env}" inlist="thm theorem:{env}" labels="LABEL:{label}" xml:id="{id}"> <tags> <tag>{autoref}</tag> <tag role="autoref">{autoref}<text xml:id="{id}.1"/></tag> <tag role="refnum">{number}</tag> <tag role="typerefnum">{autoref}</tag> </tags> <title class="ltx_runin"><tag><text font="bold" xml:id="{id}.2">{autoref}</text></tag></title>"#
      ),
      "{label}: {xml}"
    );
  }
  assert!(
    theorem_head(&xml, r#"class="ltx_theorem_proof""#).ends_with(
      r#"<title class="ltx_runin"><tag><text font="bold" xml:id="Thmproofx1.2">Preuve</text></tag></title>"#
    ),
    "{xml}"
  );
}
