//! Red/green guards for batch 78n (fidelity audit round 3, numbering and cross-reference text): eqnarray rows ended by
//! a raw `\cr`, REVTeX's Roman table numbers, iopart's `\appendix`, IEEEtran's theorems numbered within a section,
//! elsarticle's `\newproof` heads and REVTeX's merged citations; round 2: eqnarray's empty numbered rows and labels in
//! `\cr` rows, `\newproof`'s counters, iopart's `\numparts`, natbib's merge levels; round 3: eqnarray numbering as
//! `\@@eqncr`'s `\if@eqnsw` state (labels in `\nonumber` rows, `\nonumber` in `\cr` rows), `\newproof`'s `\@ifdefinable`. Each guard converts its repro under
//! `tools/perfect_kernel/repros/`, asserts the diagnostics by count and the numbers in whole elements, against pdflatex.
use latexml::util::test::{assert_element, convert_with};

use super::perfect_kernel_batch46::{error_count, warning_count};

const PRELOAD: &str = "ar5iv.sty";

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!("../../../tools/perfect_kernel/repros/", $path))
  };
}

/// Convert `tex`: 0 errors, 0 warnings, and each `(tag, xml:id, element)` as given.
fn assert_clean(tex: &str, expected: &[(&str, &str, &str)]) -> String {
  let (log, xml) = convert_with(tex, Some(PRELOAD));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  for (tag, id, element) in expected {
    assert_element(&xml, tag, &[&format!("xml:id=\"{id}\"")], element);
  }
  xml
}

/// Every `<equation>` in document order as `xml:id:<tags>…</tags>`, `-` for one without tags.
fn equation_tags(xml: &str) -> String {
  let re = regex::Regex::new(r#"(?s)<equation [^>]*xml:id="([^"]+)"[^>]*>\s*(<tags>.*?</tags>)?"#)
    .unwrap();
  let space = regex::Regex::new(r"\s*\n\s*").unwrap();
  re.captures_iter(xml)
    .map(|c| {
      let tags = c.get(2).map_or("-".to_string(), |t| {
        space.replace_all(t.as_str(), "").to_string()
      });
      format!("{}:{tags}", &c[1])
    })
    .collect::<Vec<_>>()
    .join("|")
}

/// Every `<equation>` in document order as `labels@xml:id:<tags>…</tags>`, `-` for one without tags.
fn equation_heads(xml: &str) -> String {
  let re = regex::Regex::new(r#"(?s)<equation ([^>]*)>\s*(<tags>.*?</tags>)?"#).unwrap();
  let labels = regex::Regex::new(r#"labels="([^"]*)""#).unwrap();
  let id = regex::Regex::new(r#"xml:id="([^"]*)""#).unwrap();
  let space = regex::Regex::new(r"\s*\n\s*").unwrap();
  re.captures_iter(xml)
    .map(|c| {
      let label = labels
        .captures(&c[1])
        .map_or("", |l| l.get(1).unwrap().as_str());
      let id = id
        .captures(&c[1])
        .map_or("", |i| i.get(1).unwrap().as_str());
      let tags = c.get(2).map_or("-".to_string(), |t| {
        space.replace_all(t.as_str(), "").to_string()
      });
      format!("{label}@{id}:{tags}")
    })
    .collect::<Vec<_>>()
    .join("|")
}

/// latex.ltx:15789-15797: the number `\@@eqncr` prints on an empty row of its own — after rows ended by `\cr`, by the
/// `&&\\` idiom or `\end{eqnarray}` after a trailing `\cr` — is the unnumbered equation's above it: the row joins it
/// with its tags and id (pdflatex "Refs 1, 2, 3, 4, 5."), in a single-column eqnarray too (`x = y, \nonumber\\`,
/// 1206.6051). Red (78n r1): the rows removed with their numbers, `\ref` "S0.Ex2" (0708.0012's section numbers "III A"
/// where the PDF prints 24, 26, 30, 40, 45). Witnesses 0708.0012, nucl-th9807024, 2112.13115, 1206.6051. Repro
/// cross-references/eqnarray_cr_then_empty_numbered_row.
#[test]
fn eqnarray_empty_numbered_row_numbers_the_equation_above() {
  let xml = assert_clean(
    repro!("cross-references/eqnarray_cr_then_empty_numbered_row.tex"),
    &[],
  );
  let tags = |n: u32| format!("<tags><tag>({n})</tag><tag role=\"refnum\">{n}</tag></tags>");
  assert_eq!(
    equation_heads(&xml),
    ["a", "b", "c", "d", "e"]
      .iter()
      .zip(1..)
      .map(|(label, n)| format!("LABEL:{label}@S0.E{n}:{}", tags(n)))
      .collect::<Vec<_>>()
      .join("|"),
    "{xml}"
  );
}

/// latex.ltx:15757-15797: eqnarray's `\@currentlabel` is `\theequation`, stepped by `\@@eqncr` only after it prints
/// the number, so a `\label` in a row ended by `\cr` names the next number (pdflatex "Refs 1, 2, 2."). Red (78n r1): l3
/// joined the equation (1) above it. Witness 0708.0012. Repro cross-references/eqnarray_cr_row_label_names_next_number.
#[test]
fn eqnarray_cr_row_label_names_next_number() {
  let xml = assert_clean(
    repro!("cross-references/eqnarray_cr_row_label_names_next_number.tex"),
    &[],
  );
  assert_eq!(
    equation_heads(&xml),
    "LABEL:l1@S0.Ex2:<tags><tag>(1)</tag><tag role=\"refnum\">1</tag></tags>|\
     LABEL:l3 LABEL:l2@S0.E2:<tags><tag>(2)</tag><tag role=\"refnum\">2</tag></tags>",
    "{xml}"
  );
}

/// latex.ltx:15750-15797: `\@currentlabel` is `\theequation`, which only a printing `\@@eqncr` steps, so a `\label` in a
/// `\nonumber` row names the next printed number (pdflatex "Refs 1, 2, 3, 3."). Red (78n r2; int52a, Perl alike): a and c
/// left on their unnumbered rows. Witness 1206.6051 (NA3). Repro
/// cross-references/eqnarray_nonumber_row_label_names_next_number.
#[test]
fn eqnarray_nonumber_row_label_names_next_number() {
  let xml = assert_clean(
    repro!("cross-references/eqnarray_nonumber_row_label_names_next_number.tex"),
    &[],
  );
  let tags = |n: u32| format!("<tags><tag>({n})</tag><tag role=\"refnum\">{n}</tag></tags>");
  assert_eq!(
    equation_heads(&xml),
    format!(
      "@S0.Ex1:-|LABEL:a@S0.E1:{}|LABEL:b@S0.E2:{}|@S0.Ex2:-|LABEL:c LABEL:d@S0.E3:{}",
      tags(1),
      tags(2),
      tags(3)
    ),
    "{xml}"
  );
}

/// latex.ltx:15775, 15795: `\nonumber` is `\global\@eqnswfalse`, and only `\@@eqncr` tests and resets `\if@eqnsw`; a row
/// ended by `\cr` leaves it, so a `\nonumber` in a `\cr` row unnumbers the next `\\` row too (pdflatex: only e=f (1),
/// m=n (2), the equation (3); "Refs 1, 1, 2, 2, 3."). Red (78n r2): c=d (1), e=f (2), m=n (3), the equation (4), d on an
/// unnumbered row. Witness 2112.13115. Repro cross-references/eqnarray_nonumber_holds_until_eqncr.
#[test]
fn eqnarray_nonumber_holds_until_eqncr() {
  let xml = assert_clean(
    repro!("cross-references/eqnarray_nonumber_holds_until_eqncr.tex"),
    &[],
  );
  let tags = |n: u32| format!("<tags><tag>({n})</tag><tag role=\"refnum\">{n}</tag></tags>");
  assert_eq!(
    equation_heads(&xml),
    format!(
      "@S0.Ex1:-|@S0.Ex2:-|LABEL:a LABEL:b@S0.E1:{}|@S0.Ex3:-|@S0.Ex4:-|@S0.Ex5:-|LABEL:c LABEL:d@S0.E2:{}|\
       LABEL:e@S0.E3:{}",
      tags(1),
      tags(2),
      tags(3)
    ),
    "{xml}"
  );
}

/// latex.ltx:15789-15795: only `\@@eqncr` (`\\`, `\end{eqnarray}`) prints a number and steps the counter, so an
/// eqnarray row ended by a raw `\cr` is unnumbered and the next row takes the number (pdflatex "(1)" on c=d, "(2)" on
/// k=l, the equation "(3)"). Red (int50a; Perl alike): rows 1-4, the equation 5. Witnesses 1011.0523, nucl-th9903062.
/// Repro cross-references/eqnarray_cr_row_is_unnumbered.
#[test]
fn eqnarray_row_ended_by_cr_is_unnumbered() {
  let xml = assert_clean(
    repro!("cross-references/eqnarray_cr_row_is_unnumbered.tex"),
    &[],
  );
  assert_eq!(
    equation_tags(&xml),
    "S0.Ex1:-|S0.E1:<tags><tag>(1)</tag><tag role=\"refnum\">1</tag></tags>|S0.Ex2:-|\
     S0.E2:<tags><tag>(2)</tag><tag role=\"refnum\">2</tag></tags>|\
     S0.E3:<tags><tag>(3)</tag><tag role=\"refnum\">3</tag></tags>",
    "{xml}"
  );
  assert!(xml.contains("labels=\"LABEL:r\" xml:id=\"S0.E1\""), "{xml}");
}

/// revtex4-1.cls:6759 (revtex4.cls:5367, revtex4-2.cls:6780, REVTeX 3 aps.sty:636) `\thetable{\@Roman\c@table}`:
/// "TABLE I", "TABLE II"; figures stay arabic and the names stay "Table"/"Figure" (user ruling 2026-10-21, 18). Red
/// (int50a; Perl alike): "Table 1", "Table 2". Witnesses 0708.2630, 0902.4794, 1911.02024, cond-mat9610218,
/// cond-mat0408054. Repro cross-references/revtex_table_number_is_roman.
#[test]
fn revtex_tables_are_roman() {
  assert_clean(
    repro!("cross-references/revtex_table_number_is_roman.tex"),
    &[
      (
        "table",
        "Sx1.T1",
        "<table inlist=\"lot\" labels=\"LABEL:t\" xml:id=\"Sx1.T1\"><tags><tag>Table I</tag>\
 <tag role=\"autoref\">Table\u{00A0}I<text xml:id=\"Sx1.T1.1\"/></tag><tag role=\"refnum\">I</tag>\
 <tag role=\"typerefnum\">Table I</tag></tags><toccaption><tag close=\" \">I</tag>Cap.</toccaption>\
 <caption><tag close=\": \">Table I</tag>Cap.</caption><tabular vattach=\"middle\" xml:id=\"Sx1.T1.2\"><tbody>\
 <tr xml:id=\"Sx1.T1.2.1\"><td align=\"center\" xml:id=\"Sx1.T1.2.1.1\">x</td></tr></tbody></tabular></table>",
      ),
      (
        "table",
        "Sx1.T2",
        "<table inlist=\"lot\" labels=\"LABEL:u\" xml:id=\"Sx1.T2\"><tags><tag>Table II</tag>\
 <tag role=\"autoref\">Table\u{00A0}II<text xml:id=\"Sx1.T2.1\"/></tag><tag role=\"refnum\">II</tag>\
 <tag role=\"typerefnum\">Table II</tag></tags><toccaption><tag close=\" \">II</tag>Two.</toccaption>\
 <caption><tag close=\": \">Table II</tag>Two.</caption><tabular vattach=\"middle\" xml:id=\"Sx1.T2.2\"><tbody>\
 <tr xml:id=\"Sx1.T2.2.1\"><td align=\"center\" xml:id=\"Sx1.T2.2.1.1\">y</td></tr></tbody></tabular></table>",
      ),
      (
        "figure",
        "Sx1.F1",
        "<figure inlist=\"lof\" labels=\"LABEL:f\" xml:id=\"Sx1.F1\"><tags><tag>Figure 1</tag>\
 <tag role=\"autoref\">Figure\u{00A0}1<text xml:id=\"Sx1.F1.1\"/></tag><tag role=\"refnum\">1</tag>\
 <tag role=\"typerefnum\">Figure 1</tag></tags><toccaption><tag close=\" \">1</tag>Fig.</toccaption>\
 <caption><tag close=\": \">Figure 1</tag>Fig.</caption></figure>",
      ),
    ],
  );
}

/// iopart.cls:489-510 (0712.1190's copy): `\appendix` numbers the section "Appendix A" (its `\ref` text), the
/// equations within it "A.1", the figures "B1" (pdflatex "See Appendix A, Appendix B, (1), (A.1), (B.1), figure B1.").
/// Red (int50a; Perl alike): "A", "B", equations 2, 3, figure 1. The heading's tag is the number alone, not the
/// kernel's "Appendix Appendix A". Witnesses 0712.1190, 1509.00381. Repro cross-references/iopart_appendix_numbers.
#[test]
fn iopart_appendix_numbers() {
  let xml = assert_clean(repro!("cross-references/iopart_appendix_numbers.tex"), &[(
    "figure",
    "A2.F1",
    "<figure inlist=\"lof\" labels=\"LABEL:f\" xml:id=\"A2.F1\"><tags><tag>Figure B1</tag>\
 <tag role=\"refnum\">B1</tag><tag role=\"typerefnum\">Figure B1</tag></tags><toccaption><tag close=\" \">B1</tag>F.\
 </toccaption><caption><tag close=\": \">Figure B1</tag>F.</caption></figure>",
  )]);
  for (id, label, letter, title) in [("A1", "app", "A", "Proof"), ("A2", "app2", "B", "More")] {
    let head = format!(
      "<appendix inlist=\"toc\" labels=\"LABEL:{label}\" xml:id=\"{id}\"><tags><tag>Appendix {letter}</tag>\
       <tag role=\"refnum\">Appendix {letter}</tag><tag role=\"typerefnum\">Appendix {letter}</tag></tags>\
       <title><tag close=\" \">Appendix {letter}</tag>{title}</title><toctitle><tag close=\" \">Appendix {letter}</tag>\
       {title}</toctitle>"
    );
    assert!(
      latexml::util::test::normalize_markup(&xml).contains(&head),
      "{head}\n{xml}"
    );
  }
  assert_eq!(
    equation_tags(&xml),
    "S1.E1:<tags><tag>(1)</tag><tag role=\"refnum\">1</tag></tags>|\
     A1.E1:<tags><tag>(A.1)</tag><tag role=\"refnum\">A.1</tag></tags>|\
     A2.E1:<tags><tag>(B.1)</tag><tag role=\"refnum\">B.1</tag></tags>",
    "{xml}"
  );
}

/// IEEEtran.cls:5607-5627: a theorem numbered within sections prints the section `\arabic` through
/// `\@IEEEthmcounterinsection`, "Example 2.1" where `\thesection` is "II", and `\Alph` after `\appendices` (:5773),
/// "Example A.1". Red (int50a; Perl alike): "Example II.1". Witness 1911.01952. Repro
/// cross-references/ieeetran_theorem_within_section_is_arabic.
#[test]
fn ieeetran_theorem_within_section() {
  assert_clean(
    repro!("cross-references/ieeetran_theorem_within_section_is_arabic.tex"),
    &[
      (
        "theorem",
        "S2.Thmexample1",
        "<theorem class=\"ltx_theorem_example\" inlist=\"thm theorem:example\" labels=\"LABEL:x\" \
 xml:id=\"S2.Thmexample1\"><tags><tag>Example 2.1</tag><tag role=\"refnum\">2.1</tag>\
 <tag role=\"typerefnum\">Example 2.1</tag></tags><title class=\"ltx_runin\"><tag>\
 <text font=\"bold\" xml:id=\"S2.Thmexample1.1\">Example 2.1</text></tag></title><para xml:id=\"S2.Thmexample1.p1\">\
 <p xml:id=\"S2.Thmexample1.p1.1\"><text font=\"italic\" xml:id=\"S2.Thmexample1.p1.1.1\">E.</text></p></para></theorem>",
      ),
      (
        "theorem",
        "A1.Thmexample1",
        "<theorem class=\"ltx_theorem_example\" inlist=\"thm theorem:example\" labels=\"LABEL:y\" \
 xml:id=\"A1.Thmexample1\"><tags><tag>Example A.1</tag><tag role=\"refnum\">A.1</tag>\
 <tag role=\"typerefnum\">Example A.1</tag></tags><title class=\"ltx_runin\"><tag>\
 <text font=\"bold\" xml:id=\"A1.Thmexample1.1\">Example A.1</text></tag></title><para xml:id=\"A1.Thmexample1.p1\">\
 <p xml:id=\"A1.Thmexample1.p1.1\"><text font=\"italic\" xml:id=\"A1.Thmexample1.p1.1.1\">F.</text></p></para></theorem>",
      ),
    ],
  );
}

/// The `<theorem>` of a `\newproof` environment: its `class` and `id`, its number `n` (the tags read `name n`, the
/// `\autoref` tag `autoref`), the small-caps head "`name``note`." and the upright body.
struct NewProof<'a> {
  class:   &'a str,
  id:      &'a str,
  labels:  &'a str,
  name:    &'a str,
  autoref: &'a str,
  n:       u32,
  note:    &'a str,
  body:    &'a str,
}

impl NewProof<'_> {
  fn theorem(&self) -> String {
    let NewProof {
      class,
      id,
      labels,
      name,
      autoref,
      n,
      note,
      body,
    } = self;
    format!(
      "<theorem class=\"ltx_theorem_{class}\" inlist=\"thm theorem:{class}\"{labels} xml:id=\"{id}\"><tags>\
       <tag>{name} {n}</tag><tag role=\"autoref\">{autoref}<text xml:id=\"{id}.1\"/></tag><tag role=\"refnum\">{n}</tag>\
       <tag role=\"typerefnum\">{name} {n}</tag></tags><title class=\"ltx_runin\"><tag><text font=\"smallcaps\" \
       xml:id=\"{id}.2\">{name}</text></tag><text font=\"smallcaps\" xml:id=\"{id}.3\">{note}.</text></title>\
       <para xml:id=\"{id}.p1\"><p xml:id=\"{id}.p1.1\">{body}</p></para></theorem>"
    )
  }
}

/// elsarticle.cls:1200-1236 `\newproof`: the head prints the name alone in small caps with a period, "Proof." and "Proof
/// (of the lemma).", the body upright (pdflatex); the proofs are numbered all the same (`\@prf`'s `\refstepcounter`).
/// Red (int50a; Perl alike): bold "Proof 1", "Proof 2 (of the lemma)", italic bodies; (78n r1) unnumbered, no period.
/// Witness 1304.5109. Repro cross-references/elsarticle_newproof_head_is_unnumbered.
#[test]
fn elsarticle_newproof_head_is_unnumbered() {
  let proof = |id, autoref, n, note, body| {
    NewProof {
      class: "proof",
      id,
      labels: "",
      name: "Proof",
      autoref,
      n,
      note,
      body,
    }
    .theorem()
  };
  assert_clean(
    repro!("cross-references/elsarticle_newproof_head_is_unnumbered.tex"),
    &[
      (
        "theorem",
        "Thmproof1",
        &proof("Thmproof1", "Proof\u{00A0}1", 1, "", "P."),
      ),
      (
        "theorem",
        "Thmproof2",
        &proof("Thmproof2", "Proof\u{00A0}2", 2, " (of the lemma)", "Q."),
      ),
    ],
  );
}

/// elsarticle.cls:1212 `\@ynprf` declares under `\expandafter\@ifdefinable\csname #1\endcsname`: after amsthm,
/// `\newproof{proof}{Proof}` is LaTeX's "Command \proof already defined" (an Info here, as every "already defined. Or
/// name" message) and amsthm's proof stays; `\newproof{pf}` declares as before (pdflatex 1 error, italic "Proof.", then
/// small-caps "Proof."). Red (78n r2): no message, `\proof` redefined as a numbered elsarticle proof. The review's probe
/// el2. Repro cross-references/elsarticle_newproof_keeps_a_defined_proof.
#[test]
fn elsarticle_newproof_keeps_a_defined_proof() {
  let tex = repro!("cross-references/elsarticle_newproof_keeps_a_defined_proof.tex");
  let (log, xml) = convert_with(tex, Some(PRELOAD));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(
    log
      .lines()
      .filter(|l| l.starts_with("Info:") && l.contains("Command \\proof already defined"))
      .count(),
    1,
    "{log}"
  );
  assert_element(
    &xml,
    "proof",
    &["xml:id=\"id1\""],
    "<proof xml:id=\"id1\"><title class=\"ltx_runin\" font=\"italic\">Proof.</title><para xml:id=\"p1\">\
     <p xml:id=\"p1.1\"><text xml:id=\"p1.1.1\">P one. \u{220E}</text></p></para></proof>",
  );
  let pf = NewProof {
    class:   "pf",
    id:      "Thmpf1",
    labels:  "",
    name:    "Proof",
    autoref: "1",
    n:       1,
    note:    "",
    body:    "P two.",
  };
  assert_element(&xml, "theorem", &["xml:id=\"Thmpf1\""], &pf.theorem());
  assert!(!xml.contains("ltx_theorem_proof"), "{xml}");
}

/// elsarticle.cls:1209-1226: `\newproof{x}{…}` steps x's counter (`\@prf`, so `\ref` to a label in a proof prints its
/// number) and `\newproof{x}[y]{…}` steps y's (`\@oprf`): pdflatex "Refs 1, 2.", the theorem after the claim "Theorem 3".
/// Red (78n r1): "Refs Proof, Claim.", "Theorem 2". Witnesses 1304.5109, 2606.18393. Repro
/// cross-references/elsarticle_newproof_steps_its_counter.
#[test]
fn elsarticle_newproof_steps_its_counter() {
  let pf = NewProof {
    class:   "pf",
    id:      "Thmpf1",
    labels:  " labels=\"LABEL:p1\"",
    name:    "Proof",
    autoref: "1",
    n:       1,
    note:    "",
    body:    "P one.",
  };
  let claim = NewProof {
    class:   "clm",
    id:      "Thmthm2",
    labels:  " labels=\"LABEL:c1\"",
    name:    "Claim",
    autoref: "2",
    n:       2,
    note:    "",
    body:    "C.",
  };
  assert_clean(
    repro!("cross-references/elsarticle_newproof_steps_its_counter.tex"),
    &[
      ("theorem", "Thmpf1", &pf.theorem()),
      ("theorem", "Thmthm2", &claim.theorem()),
      (
        "theorem",
        "Thmthm3",
        "<theorem class=\"ltx_theorem_thm\" inlist=\"thm theorem:thm\" xml:id=\"Thmthm3\"><tags><tag>Theorem 3</tag>\
 <tag role=\"autoref\">3<text xml:id=\"Thmthm3.1\"/></tag><tag role=\"refnum\">3</tag><tag role=\"typerefnum\">Theorem 3</tag>\
 </tags><title class=\"ltx_runin\"><tag><text font=\"bold\" xml:id=\"Thmthm3.2\">Theorem 3</text></tag></title>\
 <para xml:id=\"Thmthm3.p1\"><p xml:id=\"Thmthm3.p1.1\"><text font=\"italic\" xml:id=\"Thmthm3.p1.1.1\">U.</text></p></para>\
 </theorem>",
      ),
    ],
  );
}

/// iopart.cls:815-825: `\numparts` numbers its equations `\arabic{section}.\arabic{eqnval}\alph{equation}` whatever
/// `\theequation` was (an appendix's too), and `\endnumparts` sets the arabic form back, so the command form leaves no
/// letters behind (pdflatex (1.1a) (1.1b) (A.2) (1.3a) (1.3b) (1.4) (1.5a) (1.5b)). Red (78n r1): (A.1a) … (A.3d)
/// (A.3ea), 0910.0753's "4.3dabga" where the PDF prints 4.3a. Repro cross-references/iopart_numparts_number_arabic.
#[test]
fn iopart_numparts_number_arabic() {
  let xml = assert_clean(
    repro!("cross-references/iopart_numparts_number_arabic.tex"),
    &[],
  );
  let tags = |n: &str| format!("<tags><tag>({n})</tag><tag role=\"refnum\">{n}</tag></tags>");
  assert_eq!(
    equation_heads(&xml),
    [
      ("", "S1.E1", "1"),
      ("LABEL:e1", "A1.E1.1", "1.1a"),
      ("", "A1.E1.2", "1.1b"),
      ("LABEL:e0", "A1.E2", "A.2"),
      ("", "A1.E3.1", "1.3a"),
      ("", "A1.E3.2", "1.3b"),
      ("LABEL:e2", "A1.E4", "1.4"),
      ("LABEL:e3", "A1.E5.1", "1.5a"),
      ("", "A1.E5.2", "1.5b"),
    ]
    .iter()
    .map(|(label, id, n)| format!("{label}@{id}:{}", tags(n)))
    .collect::<Vec<_>>()
    .join("|"),
    "{xml}"
  );
}

/// natbib.sty:272-274: `mcite` sets `\NAT@merge` to 1, `merge` and `elide` raise it only (to 2, 3), in declaration
/// order: `[merge,mcite]` is 2 (pdflatex "Merge level 2."). Green on 78n r1 as well (LaTeX's option order never lowers
/// the level through these options). Repro index-bib/natbib_merge_options_raise_level.
#[test]
fn natbib_merge_options_raise_level() {
  let xml = assert_clean(repro!("index-bib/natbib_merge_options_raise_level.tex"), &[
  ]);
  assert!(
    xml.contains("<p xml:id=\"p1.1\">Merge level 2.</p>"),
    "{xml}"
  );
}

/// natbib.sty:1135-1160 with revtex4-1.cls:7046 `\let\NAT@merge\thr@@`: `\cite{a,*b}` cites a and b (pdflatex "[1]",
/// b's entry merged into a's), `\cite{[See][and Chap. 2]c}` cites c, its notes for the entry. Red (int50a; Perl alike
/// for `*b`): the keys "*b" and "[See][andChap.2]c", missing citations. The entry merge is a residual (b stays entry 2).
/// Witnesses 1808.00247, 1509.05768. (revtex's `[sort&compress]{natbib}` puts its ordering class on the bibrefs,
/// 78j.) Repro index-bib/revtex_merged_citation_key.
#[test]
fn revtex_merged_citation() {
  let xml = assert_clean(repro!("index-bib/revtex_merged_citation_key.tex"), &[]);
  let cites = regex::Regex::new(r"<cite[ >].*?</cite>").unwrap();
  let found: Vec<&str> = cites.find_iter(&xml).map(|m| m.as_str()).collect();
  assert_eq!(
    found.join("|"),
    "<cite class=\"ltx_citemacro_cite\">[<bibref bibrefs=\"a,b\" class=\"ltx_cite_sort ltx_cite_compress\" \
     separator=\",\" show=\"Number\" yyseparator=\",\"/>]</cite>|<cite class=\"ltx_citemacro_cite\">[<bibref \
     bibrefs=\"c\" class=\"ltx_cite_sort ltx_cite_compress\" separator=\",\" show=\"Number\" yyseparator=\",\"/>]</cite>",
    "{xml}"
  );
}
