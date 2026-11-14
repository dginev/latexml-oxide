//! Red/green guards for batch 78q (quality batch 14, class numbering): equations within sections (JHEP, JHEP3, PoS,
//! jheppub, jfm), the classes' `\appendix` counters (elsarticle, mnras/mn2e, aa, interact, elsart, emulateapj
//! `[appendixfloats]`, the copies' versions; l-aa's own), llncs's section-level appendices, svjour's theorem-numbering
//! options, the llncs/svjour theorem separator, jfm's `\section*` and aipproc's unnumbered headings. Each guard
//! converts its repro under `tools/perfect_kernel/repros/cross-references/` (the class binding alone, or a stub of the
//! paper's copy), asserts 0 errors and 0 warnings, every labelled element's id and refnum against pdflatex's `.aux`
//! (with the paper's class copy), and whole elements.
use latexml::util::test::{assert_element, convert_files_with, convert_with};

use super::perfect_kernel_batch46::{error_count, warning_count};

const PRELOAD: &str = "ar5iv.sty";

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/cross-references/",
      $path,
      ".tex"
    ))
  };
}

/// Every labelled element in document order as `label@xml:id=refnum` (`-` for none), space-separated.
fn label_refnums(xml: &str) -> String {
  let open = regex::Regex::new(r#"<\w+\b[^>]*\blabels="([^"]*)"[^>]*>"#).unwrap();
  let id = regex::Regex::new(r#"xml:id="([^"]*)""#).unwrap();
  let tags = regex::Regex::new(r#"(?s)^\s*<tags>(.*?)</tags>"#).unwrap();
  let refnum = regex::Regex::new(r#"(?s)<tag role="refnum">(.*?)</tag>"#).unwrap();
  let markup = regex::Regex::new(r"<[^>]*>").unwrap();
  open
    .captures_iter(xml)
    .map(|c| {
      let whole = c.get(0).unwrap();
      let id = id
        .captures(whole.as_str())
        .map_or("", |i| i.get(1).unwrap().as_str());
      let value = tags
        .captures(&xml[whole.end()..])
        .and_then(|t| {
          refnum
            .captures(t.get(1).unwrap().as_str())
            .map(|r| r[1].to_string())
        })
        .map_or("-".to_string(), |r| {
          markup.replace_all(&r, "").trim().to_string()
        });
      format!("{}@{id}={value}", c[1].replace("LABEL:", ""))
    })
    .collect::<Vec<_>>()
    .join(" ")
}

/// Convert `tex`: 0 errors, 0 warnings, the labels' ids and refnums as `labels`, each `(tag, xml:id, element)` as
/// given.
fn assert_numbers(tex: &str, labels: &str, expected: &[(&str, &str, &str)]) {
  let (log, xml) = convert_with(tex, Some(PRELOAD));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(label_refnums(&xml), labels, "{xml}");
  for (tag, id, element) in expected {
    assert_element(&xml, tag, &[&format!("xml:id=\"{id}\"")], element);
  }
}

/// JHEP.cls v2.0.3:675-676, :1451, JHEP3.cls:905-906, :1787 and PoS.cls:771-772, :1419 (`\theequation` with the
/// section number, `\@addtoreset{equation}{section}`), jheppub.sty:313-314 (`\numberwithin`): "(1.1) (2.1) (2.2)
/// (A.1) (B.1)", as pdflatex's `.aux`. Red (int53a; Perl alike): 1, 2, 3, 4, 5. Witnesses 2205.03535, 0708.0971,
/// 2112.14648. Repros cross-references/{jhep3,pos,jheppub}_equations_within_sections.
#[test]
fn jhep_family_equations_within_sections() {
  assert_numbers(
    repro!("jhep3_equations_within_sections"),
    "sA@S1=1 e1@S1.E1=1.1 f1@S1.F1=1 t1@S1.T1=1 sB@S2=2 sC@S2.SS1=2.1 e2@S2.E1=2.1 e3@S2.E2=2.2 sD@A1=A e4@A1.E1=A.1 sE@A1.SS1=A.1 f2@A1.F2=2 t2@A1.T2=2 sG@A2=B e5@A2.E1=B.1 f3@A2.F3=3",
    &[
      (
        "equation",
        "S2.E1",
        "<equation labels=\"LABEL:e2\" xml:id=\"S2.E1\"><tags><tag>(2.1)</tag>\
 <tag role=\"autoref\">Equation\u{a0}2.1<text xml:id=\"S2.E1.1\"/></tag><tag role=\"refnum\">2.1</tag>\
 </tags><Math mode=\"display\" tex=\"b\" text=\"b\" xml:id=\"S2.E1.m1\"><XMath xml:id=\"S2.E1.m1.1\">\
 <XMTok font=\"italic\" role=\"UNKNOWN\">b</XMTok></XMath></Math></equation>",
      ),
      (
        "equation",
        "A2.E1",
        "<equation labels=\"LABEL:e5\" xml:id=\"A2.E1\"><tags><tag>(B.1)</tag>\
 <tag role=\"autoref\">Equation\u{a0}B.1<text xml:id=\"A2.E1.1\"/></tag><tag role=\"refnum\">B.1</tag>\
 </tags><Math mode=\"display\" tex=\"y\" text=\"y\" xml:id=\"A2.E1.m1\"><XMath xml:id=\"A2.E1.m1.1\">\
 <XMTok font=\"italic\" role=\"UNKNOWN\">y</XMTok></XMath></Math></equation>",
      ),
    ],
  );
  assert_numbers(
    repro!("pos_equations_within_sections"),
    "sA@S1=1 e1@S1.E1=1.1 f1@S1.F1=1 t1@S1.T1=1 sB@S2=2 sC@S2.SS1=2.1 e2@S2.E1=2.1 e3@S2.E2=2.2 sD@A1=A e4@A1.E1=A.1 sE@A1.SS1=A.1 f2@A1.F2=2 t2@A1.T2=2 sG@A2=B e5@A2.E1=B.1 f3@A2.F3=3",
    &[(
      "equation",
      "A1.E1",
      "<equation labels=\"LABEL:e4\" xml:id=\"A1.E1\"><tags><tag>(A.1)</tag>\
 <tag role=\"autoref\">Equation\u{a0}A.1<text xml:id=\"A1.E1.1\"/></tag><tag role=\"refnum\">A.1</tag>\
 </tags><Math mode=\"display\" tex=\"x\" text=\"x\" xml:id=\"A1.E1.m1\"><XMath xml:id=\"A1.E1.m1.1\">\
 <XMTok font=\"italic\" role=\"UNKNOWN\">x</XMTok></XMath></Math></equation>",
    )],
  );
  assert_numbers(
    repro!("jheppub_equations_within_sections"),
    "sA@S1=1 e1@S1.E1=1.1 f1@S1.F1=1 t1@S1.T1=1 sB@S2=2 sC@S2.SS1=2.1 e2@S2.E1=2.1 e3@S2.E2=2.2 sD@A1=A e4@A1.E1=A.1 sE@A1.SS1=A.1 f2@A1.F2=2 t2@A1.T2=2 sG@A2=B e5@A2.E1=B.1 f3@A2.F3=3",
    &[(
      "equation",
      "A1.E1",
      "<equation labels=\"LABEL:e4\" xml:id=\"A1.E1\"><tags><tag>(A.1)</tag>\
 <tag role=\"autoref\">Equation\u{a0}A.1<text xml:id=\"A1.E1.1\"/></tag><tag role=\"refnum\">A.1</tag>\
 </tags><Math mode=\"display\" tex=\"x\" text=\"x\" xml:id=\"A1.E1.m1\"><XMath xml:id=\"A1.E1.m1.1\">\
 <XMTok font=\"italic\" role=\"UNKNOWN\">x</XMTok></XMath></Math></equation>",
    )],
  );
}

/// jfm.cls (every version; v1.27 :573, the appendix :361 `\thesection\,\@arabic\c@equation`): "(1.1) (2.1) (2.2)",
/// the appendices "(A 1)" with a thin space, as pdflatex's `.aux` (`A\,1`). Red (int53a): 1, 2, 3, 4, 5. Witness
/// 2608.04345; SYNC row 76. Repro cross-references/jfm_equations_within_sections.
#[test]
fn jfm_equations_within_sections() {
  assert_numbers(
    repro!("jfm_equations_within_sections"),
    "sA@S1=1 e1@S1.E1=1.1 f1@S1.F1=1 t1@S1.T1=1 sB@S2=2 sC@S2.SS1=2.1 e2@S2.E1=2.1 e3@S2.E2=2.2 sD@A1=A e4@A1.E1=A\u{2009}1 sE@A1.SS1=A.1 f2@A1.F2=2 t2@A1.T2=2 sG@A2=B e5@A2.E1=B\u{2009}1 f3@A2.F3=3",
    &[(
      "equation",
      "A1.E1",
      "<equation labels=\"LABEL:e4\" xml:id=\"A1.E1\"><tags><tag>(A\u{2009}1)</tag>\
 <tag role=\"autoref\">Equation\u{a0}A\u{2009}1<text xml:id=\"A1.E1.1\"/></tag>\
 <tag role=\"refnum\">A\u{2009}1</tag></tags>\
 <Math mode=\"display\" tex=\"x\" text=\"x\" xml:id=\"A1.E1.m1\"><XMath xml:id=\"A1.E1.m1.1\">\
 <XMTok font=\"italic\" role=\"UNKNOWN\">x</XMTok></XMath></Math></equation>",
    )],
  );
}

/// jfm.cls:299-306 (v1.23-v1.28): `\section` zeroes the equation counter, a starred one too: "(2.1)" after
/// `\section*`, as pdflatex with 2608.04345's v1.27 copy; in the appendices pdflatex zeroes it too ("A 1") where ours
/// keeps the count ("A 2", e6 pinned at the residual: the locked `\section` ignores the re-wrap; review rev78q2). An
/// FLM copy declaring itself `jfm` (JFM-FLM_Au.cls:501, :1318; 2507.06135) resets it by the section counter only, "(2.2)", and the paper's copy decides. Red (78q r1; review finding
/// 4): 2.2 (and int53a 4). Repros cross-references/jfm_section_star_equations,
/// class_copies_78q/jfm_flm_copy_section_star_keeps_count.
#[test]
fn jfm_section_star_zeroes_equations() {
  assert_numbers(
    repro!("jfm_section_star_equations"),
    "sA@S1=1 e1@S1.E1=1.1 e2@S1.E2=1.2 sB@S2=2 e3@S2.E1=2.1 e4@Sx1.E1=2.1 sC@A1=A e5@A1.E1=A\u{2009}1 \
     e6@Ax1.E2=A\u{2009}2",
    &[(
      "equation",
      "Sx1.E1",
      "<equation labels=\"LABEL:e4\" xml:id=\"Sx1.E1\"><tags><tag>(2.1)</tag>\
 <tag role=\"autoref\">Equation\u{a0}2.1<text xml:id=\"Sx1.E1.1\"/></tag><tag role=\"refnum\">2.1</tag>\
 </tags><Math mode=\"display\" tex=\"d\" text=\"d\" xml:id=\"Sx1.E1.m1\"><XMath xml:id=\"Sx1.E1.m1.1\">\
 <XMTok font=\"italic\" role=\"UNKNOWN\">d</XMTok></XMath></Math></equation>",
    )],
  );
  assert_copy_numbers(
    repro!("class_copies_78q/jfm_flm_copy_section_star_keeps_count"),
    &[(
      "jfm.cls",
      include_str!(
        "../../../tools/perfect_kernel/repros/cross-references/class_copies_78q/jfm_flm.cls"
      ),
    )],
    "sA@S1=1 e1@S1.E1=1.1 e2@S1.E2=1.2 sB@S2=2 e3@S2.E1=2.1 e4@Sx1.E2=2.2 sC@A1=A e5@A1.E1=A\u{2009}1",
  );
}

/// elsarticle.cls:1342-1353: the appendix sections "Appendix A" (subsections "Appendix A.1"), equations "A.1" reset
/// per appendix, floats "A.2" (the running number after the letter), the heading's tag the section number alone. Red
/// (int53a; Perl alike): A, 4, 2. Witnesses 2609.03753, 2608.23563. Repro
/// cross-references/elsarticle_appendix_counters.
#[test]
fn appendix_counters_elsarticle() {
  assert_numbers(
    repro!("elsarticle_appendix_counters"),
    "sA@S1=1 e1@S1.E1=1 f1@S1.F1=1 t1@S1.T1=1 sB@S2=2 sC@S2.SS1=2.1 e2@S2.E2=2 e3@S2.E3=3 sD@A1=Appendix\u{a0}A e4@A1.E1=A.1 sE@A1.SS1=Appendix\u{a0}A.1 f2@A1.F2=A.2 t2@A1.T2=A.2 sG@A2=Appendix\u{a0}B e5@A2.E1=B.1 f3@A2.F3=B.3",
    &[
      (
        "figure",
        "A1.F2",
        "<figure inlist=\"lof\" labels=\"LABEL:f2\" placement=\"t\" xml:id=\"A1.F2\"><tags><tag>Figure A.2</tag>\
 <tag role=\"autoref\">Figure\u{a0}A.2<text xml:id=\"A1.F2.1\"/></tag><tag role=\"refnum\">A.2</tag>\
 <tag role=\"typerefnum\">Figure A.2</tag></tags><toccaption><tag close=\" \">A.2</tag>F2</toccaption>\
 <caption><tag close=\": \">Figure A.2</tag>F2</caption></figure>",
      ),
      (
        "equation",
        "A2.E1",
        "<equation labels=\"LABEL:e5\" xml:id=\"A2.E1\"><tags><tag>(B.1)</tag>\
 <tag role=\"autoref\">Equation\u{a0}B.1<text xml:id=\"A2.E1.1\"/></tag><tag role=\"refnum\">B.1</tag>\
 </tags><Math mode=\"display\" tex=\"y\" text=\"y\" xml:id=\"A2.E1.m1\"><XMath xml:id=\"A2.E1.m1.1\">\
 <XMTok font=\"italic\" role=\"UNKNOWN\">y</XMTok></XMath></Math></equation>",
      ),
    ],
  );
}

/// mnras.cls:818-833 and mn2e.cls:690-704: subsections "A1", the equations, figures and tables "A1", "B1" (reset by
/// each appendix section). Red (int53a; Perl alike): A.1, 4, 2, 3. Witnesses 2608.17430, 0705.0429. Repros
/// cross-references/{mnras,mn2e}_appendix_counters.
#[test]
fn appendix_counters_mn_family() {
  assert_numbers(
    repro!("mnras_appendix_counters"),
    "sA@S1=1 e1@S1.E1=1 f1@S1.F1=1 t1@S1.T1=1 sB@S2=2 sC@S2.SS1=2.1 e2@S2.E2=2 e3@S2.E3=3 sD@A1=A e4@A1.E1=A1 sE@A1.SS1=A1 f2@A1.F1=A1 t2@A1.T1=A1 sG@A2=B e5@A2.E1=B1 f3@A2.F1=B1",
    &[
      (
        "figure",
        "A2.F1",
        "<figure inlist=\"lof\" labels=\"LABEL:f3\" placement=\"t\" xml:id=\"A2.F1\"><tags><tag>Figure B1</tag>\
 <tag role=\"autoref\">Figure\u{a0}B1<text xml:id=\"A2.F1.1\"/></tag><tag role=\"refnum\">B1</tag>\
 <tag role=\"typerefnum\">Figure B1</tag></tags><toccaption><tag close=\" \">B1</tag>F3</toccaption>\
 <caption><tag close=\": \">Figure B1</tag>F3</caption></figure>",
      ),
      (
        "table",
        "A1.T1",
        "<table inlist=\"lot\" labels=\"LABEL:t2\" placement=\"t\" xml:id=\"A1.T1\"><tags><tag>Table A1</tag>\
 <tag role=\"autoref\">Table\u{a0}A1<text xml:id=\"A1.T1.1\"/></tag><tag role=\"refnum\">A1</tag>\
 <tag role=\"typerefnum\">Table A1</tag></tags><toccaption><tag close=\" \">A1</tag>T2</toccaption>\
 <caption><tag close=\": \">Table A1</tag>T2</caption></table>",
      ),
    ],
  );
  assert_numbers(
    repro!("mn2e_appendix_counters"),
    "sA@S1=1 e1@S1.E1=1 f1@S1.F1=1 t1@S1.T1=1 sB@S2=2 sC@S2.SS1=2.1 e2@S2.E2=2 e3@S2.E3=3 sD@A1=A e4@A1.E1=A1 sE@A1.SS1=A1 f2@A1.F1=A1 t2@A1.T1=A1 sG@A2=B e5@A2.E1=B1 f3@A2.F1=B1",
    &[(
      "equation",
      "A1.E1",
      "<equation labels=\"LABEL:e4\" xml:id=\"A1.E1\"><tags><tag>(A1)</tag><tag role=\"refnum\">A1</tag></tags>\
 <Math mode=\"display\" tex=\"x\" text=\"x\" xml:id=\"A1.E1.m1\"><XMath xml:id=\"A1.E1.m1.1\">\
 <XMTok font=\"italic\" role=\"UNKNOWN\">x</XMTok></XMath></Math></equation>",
    )],
  );
}

/// aa.cls (v5.01-v9.4; the v6.1 `{appendix}` environment): the equations, figures and tables "A.1", "B.1" (reset per
/// appendix section). Red (int53a; Perl alike): 4, 2, 3. Witnesses astro-ph0609025, 0705.1240. Repro
/// cross-references/aa_appendix_counters.
#[test]
fn appendix_counters_aa() {
  assert_numbers(
    repro!("aa_appendix_counters"),
    "sA@S1=1 e1@S1.E1=1 f1@S1.F1=1 t1@S1.T1=1 sB@S2=2 sC@S2.SS1=2.1 e2@S2.E2=2 e3@S2.E3=3 sD@A1=A e4@A1.E1=A.1 sE@A1.SS1=A.1 f2@A1.F1=A.1 t2@A1.T1=A.1 sG@A2=B e5@A2.E1=B.1 f3@A2.F1=B.1",
    &[
      (
        "figure",
        "A2.F1",
        "<figure inlist=\"lof\" labels=\"LABEL:f3\" placement=\"t\" xml:id=\"A2.F1\"><tags><tag>Figure B.1</tag>\
 <tag role=\"refnum\">B.1</tag><tag role=\"typerefnum\">Figure B.1</tag></tags><toccaption>\
 <tag close=\" \">B.1</tag>F3</toccaption><caption><tag close=\": \">Figure B.1</tag>F3</caption></figure>",
      ),
      (
        "equation",
        "A1.E1",
        "<equation labels=\"LABEL:e4\" xml:id=\"A1.E1\"><tags><tag>(A.1)</tag><tag role=\"refnum\">A.1</tag>\
 </tags><Math mode=\"display\" tex=\"x\" text=\"x\" xml:id=\"A1.E1.m1\"><XMath xml:id=\"A1.E1.m1.1\">\
 <XMTok font=\"italic\" role=\"UNKNOWN\">x</XMTok></XMath></Math></equation>",
      ),
    ],
  );
}

/// l-aa.sty:208-214 (v3.0, astro-ph9604133's and astro-ph9701125's copies): the appendix equations "A1", "B1" (no dot),
/// the figures and tables in their running numbers, not the aa binding's "A.1" it loads. Red (78q r1, the aa numbering;
/// review finding 1): A.1, A.1, A.1, B.1, B.1. Witnesses astro-ph9705112 ("Fig. 12.", "(A1)"), astro-ph9607057. Repro
/// cross-references/l_aa_appendix_counters.
#[test]
fn appendix_counters_l_aa() {
  assert_numbers(
    repro!("l_aa_appendix_counters"),
    "sA@S1=1 e1@S1.E1=1 f1@S1.F1=1 t1@S1.T1=1 sB@S2=2 sC@S2.SS1=2.1 e2@S2.E2=2 e3@S2.E3=3 sD@A1=A e4@A1.E1=A1 sE@A1.SS1=A.1 f2@A1.F2=2 t2@A1.T2=2 sG@A2=B e5@A2.E1=B1 f3@A2.F3=3",
    &[
      (
        "figure",
        "A1.F2",
        "<figure inlist=\"lof\" labels=\"LABEL:f2\" placement=\"t\" xml:id=\"A1.F2\"><tags><tag>Fig. 2.</tag>\
 <tag role=\"refnum\">2</tag><tag role=\"typerefnum\">Figure 2</tag></tags><toccaption>\
 <tag close=\" \">2</tag>F2</toccaption><caption><tag close=\" \">Fig. 2.</tag>F2</caption></figure>",
      ),
      (
        "equation",
        "A1.E1",
        "<equation labels=\"LABEL:e4\" xml:id=\"A1.E1\"><tags><tag>(A1)</tag><tag role=\"refnum\">A1</tag>\
 </tags><Math mode=\"display\" tex=\"x\" text=\"x\" xml:id=\"A1.E1.m1\"><XMath xml:id=\"A1.E1.m1.1\">\
 <XMTok font=\"italic\" role=\"UNKNOWN\">x</XMTok></XMath></Math></equation>",
      ),
    ],
  );
}

/// interact.cls:562-573 ("A1") and elsart.cls:239-255 ("A.1"): the equations, figures and tables numbered within each
/// appendix. Red (int53a): 4, 2, 3. Witnesses 1603.08785, 0706.3521. Repros
/// cross-references/{interact,elsart}_appendix_counters.
#[test]
fn appendix_counters_interact_elsart() {
  assert_numbers(
    repro!("interact_appendix_counters"),
    "sA@S1=1 e1@S1.E1=1 f1@S1.F1=1 t1@S1.T1=1 sB@S2=2 sC@S2.SS1=2.1 e2@S2.E2=2 e3@S2.E3=3 sD@A1=A e4@A1.E1=A1 sE@A1.SS1=A.1 f2@A1.F1=A1 t2@A1.T1=A1 sG@A2=B e5@A2.E1=B1 f3@A2.F1=B1",
    &[(
      "table",
      "A1.T1",
      "<table inlist=\"lot\" labels=\"LABEL:t2\" placement=\"t\" xml:id=\"A1.T1\"><tags><tag>Table A1</tag>\
 <tag role=\"autoref\">Table\u{a0}A1<text xml:id=\"A1.T1.1\"/></tag><tag role=\"refnum\">A1</tag>\
 <tag role=\"typerefnum\">Table A1</tag></tags><toccaption><tag close=\" \">A1</tag>T2</toccaption>\
 <caption><tag close=\": \">Table A1</tag>T2</caption></table>",
    )],
  );
  assert_numbers(
    repro!("elsart_appendix_counters"),
    "sA@S1=1 e1@S1.E1=1 f1@S1.F1=1 t1@S1.T1=1 sB@S2=2 sC@S2.SS1=2.1 e2@S2.E2=2 e3@S2.E3=3 sD@A1=A e4@A1.E1=A.1 sE@A1.SS1=A.1 f2@A1.F1=A.1 t2@A1.T1=A.1 sG@A2=B e5@A2.E1=B.1 f3@A2.F1=B.1",
    &[(
      "figure",
      "A2.F1",
      "<figure inlist=\"lof\" labels=\"LABEL:f3\" placement=\"t\" xml:id=\"A2.F1\"><tags><tag>Figure B.1</tag>\
 <tag role=\"refnum\">B.1</tag><tag role=\"typerefnum\">Figure B.1</tag></tags><toccaption>\
 <tag close=\" \">B.1</tag>F3</toccaption><caption><tag close=\": \">Figure B.1</tag>F3</caption></figure>",
    )],
  );
}

/// emulateapj.cls:128-131, :744-797: with `[appendixfloats]` the figures and tables "A1", "B1" (zeroed by each
/// appendix section), the equations "A1". Red (int53a; Perl alike): figures 2, 3. Witness 2609.30392. Repro
/// cross-references/emulateapj_appendixfloats_counters.
#[test]
fn appendix_counters_emulateapj_appendixfloats() {
  assert_numbers(
    repro!("emulateapj_appendixfloats_counters"),
    "sA@S1=1 e1@S1.E1=1 f1@S1.F1=1 t1@S1.T1=1 sB@S2=2 sC@S2.SS1=2.1 e2@S2.E2=2 e3@S2.E3=3 sD@A1=A e4@A1.E1=A1 sE@A1.SS1=A.1 f2@A1.F1=A1 t2@A1.T1=A1 sG@A2=B e5@A2.E1=B1 f3@A2.F1=B1",
    &[(
      "figure",
      "A2.F1",
      "<figure inlist=\"lof\" labels=\"LABEL:f3\" placement=\"t\" xml:id=\"A2.F1\"><tags><tag>Figure B1</tag>\
 <tag role=\"autoref\">Figure\u{a0}B1<text xml:id=\"A2.F1.1\"/></tag><tag role=\"refnum\">B1</tag>\
 <tag role=\"typerefnum\">Figure B1</tag></tags><toccaption><tag close=\" \">B1</tag>F3</toccaption>\
 <caption><tag close=\".— \">Figure B1</tag>F3</caption></figure>",
    )],
  );
}

/// llncs.cls:51 loads article, whose `\appendix` numbers the appendix sections within the document: "A", "A.1", "B"
/// (ids A1, A2), not within the class's `\chapter` ("0.A", ids Pt0.A1). Red (int53a; Perl "Appendix 0.A"). Witnesses
/// 2609.39362, 2303.08726. Repro cross-references/llncs_appendices_are_sections.
#[test]
fn llncs_appendices_are_sections() {
  assert_numbers(
    repro!("llncs_appendices_are_sections"),
    "sA@S1=1 e1@S1.E1=1 f1@S1.F1=1 t1@S1.T1=1 sB@S2=2 sC@S2.SS1=2.1 e2@S2.E2=2 e3@S2.E3=3 sD@A1=A e4@A1.E4=4 sE@A1.SS1=A.1 f2@A1.F2=2 t2@A1.T2=2 sG@A2=B e5@A2.E5=5 f3@A2.F3=3",
    &[(
      "subsection",
      "A1.SS1",
      "<subsection inlist=\"toc\" labels=\"LABEL:sE\" xml:id=\"A1.SS1\"><tags><tag>A.1</tag>\
 <tag role=\"refnum\">A.1</tag><tag role=\"typerefnum\">§A.1</tag></tags><title><tag close=\" \">A.1\
 </tag>E</title><figure inlist=\"lof\" labels=\"LABEL:f2\" placement=\"t\" xml:id=\"A1.F2\"><tags>\
 <tag>Figure 2</tag><tag role=\"refnum\">2</tag><tag role=\"typerefnum\">Figure 2</tag></tags><toccaption>\
 <tag close=\" \">2</tag>F2</toccaption><caption><tag close=\": \">Figure 2</tag>F2</caption></figure>\
 <table inlist=\"lot\" labels=\"LABEL:t2\" placement=\"t\" xml:id=\"A1.T2\"><tags><tag>Table 2</tag>\
 <tag role=\"refnum\">2</tag><tag role=\"typerefnum\">Table 2</tag></tags><toccaption><tag close=\" \">2\
 </tag>T2</toccaption><caption><tag close=\": \">Table 2</tag>T2</caption></table></subsection>",
    )],
  );
}

/// svjour3.cls:885-933 (svjour.cls:756-792): `{theorem}` on its own counter by default ("Theorem 1", "Theorem 2"),
/// the other environments each on theirs; under envcountsame all on the theorem counter; under envcountsect all
/// within the section ("1.1"). Red (int53a; Perl alike): Theorem 1.1, 2.1 by default. Witnesses 1604.08616,
/// 1803.06644. Repros cross-references/svjour3{,_envcountsame,_envcountsect}_theorem_counters.
#[test]
fn svjour_theorem_counters() {
  assert_numbers(
    repro!("svjour3_theorem_counters"),
    "s1@S1=1 t1@Thmtheorem1=1 l1@Thmlemma1=1 s2@S2=2 t2@Thmtheorem2=2 d1@Thmdefinition1=1 l2@Thmlemma2=2",
    &[(
      "theorem",
      "Thmtheorem2",
      "<theorem class=\"ltx_theorem_theorem\" inlist=\"thm theorem:theorem\" labels=\"LABEL:t2\" xml:id=\"Thmtheorem2\">\
 <tags><tag>Theorem 2</tag><tag role=\"refnum\">2</tag><tag role=\"typerefnum\">Theorem 2</tag></tags>\
 <title class=\"ltx_runin\"><tag><text font=\"bold\" xml:id=\"Thmtheorem2.1\">Theorem 2</text></tag>\
 </title><para xml:id=\"Thmtheorem2.p1\"><p xml:id=\"Thmtheorem2.p1.1\">\
 <text font=\"italic\" xml:id=\"Thmtheorem2.p1.1.1\">C.</text></p></para></theorem>",
    )],
  );
  assert_numbers(
    repro!("svjour3_envcountsame_theorem_counters"),
    "s1@S1=1 t1@Thmtheorem1=1 l1@Thmtheorem2=2 s2@S2=2 t2@Thmtheorem3=3 d1@Thmtheorem4=4 l2@Thmtheorem5=5",
    &[(
      "theorem",
      "Thmtheorem4",
      "<theorem class=\"ltx_theorem_definition\" inlist=\"thm theorem:definition\" labels=\"LABEL:d1\" xml:id=\"Thmtheorem4\">\
 <tags><tag>Definition 4</tag><tag role=\"refnum\">4</tag><tag role=\"typerefnum\">Definition 4</tag>\
 </tags><title class=\"ltx_runin\"><tag><text font=\"bold\" xml:id=\"Thmtheorem4.1\">Definition 4</text>\
 </tag></title><para xml:id=\"Thmtheorem4.p1\"><p xml:id=\"Thmtheorem4.p1.1\">\
 <text font=\"italic\" xml:id=\"Thmtheorem4.p1.1.1\">D.</text></p></para></theorem>",
    )],
  );
  assert_numbers(
    repro!("svjour3_envcountsect_theorem_counters"),
    "s1@S1=1 t1@S1.Thmtheorem1=1.1 l1@S1.Thmlemma1=1.1 s2@S2=2 t2@S2.Thmtheorem1=2.1 d1@S2.Thmdefinition1=2.1 l2@S2.Thmlemma1=2.1",
    &[(
      "theorem",
      "S2.Thmlemma1",
      "<theorem class=\"ltx_theorem_lemma\" inlist=\"thm theorem:lemma\" labels=\"LABEL:l2\" xml:id=\"S2.Thmlemma1\">\
 <tags><tag>Lemma 2.1</tag><tag role=\"refnum\">2.1</tag><tag role=\"typerefnum\">Lemma 2.1</tag></tags>\
 <title class=\"ltx_runin\"><tag><text font=\"bold\" xml:id=\"S2.Thmlemma1.1\">Lemma 2.1</text></tag>\
 </title><para xml:id=\"S2.Thmlemma1.p1\"><p xml:id=\"S2.Thmlemma1.p1.1\">\
 <text font=\"italic\" xml:id=\"S2.Thmlemma1.p1.1.1\">E.</text></p></para></theorem>",
    )],
  );
  assert_numbers(
    repro!("svjour3_envcountreset_theorem_counters"),
    "s1@S1=1 t1@S1.Thmtheorem1=1 l1@S1.Thmlemma1=1 s2@S2=2 t2@S2.Thmtheorem1=1 t3@S2.Thmtheorem2=2 l2@S2.Thmlemma1=1 d1@S2.Thmdefinition1=1",
    &[(
      "theorem",
      "S2.Thmtheorem1",
      "<theorem class=\"ltx_theorem_theorem\" inlist=\"thm theorem:theorem\" labels=\"LABEL:t2\" xml:id=\"S2.Thmtheorem1\">\
 <tags><tag>Theorem 1</tag><tag role=\"refnum\">1</tag><tag role=\"typerefnum\">Theorem 1</tag></tags>\
 <title class=\"ltx_runin\"><tag><text font=\"bold\" xml:id=\"S2.Thmtheorem1.1\">Theorem 1</text></tag>\
 </title><para xml:id=\"S2.Thmtheorem1.p1\"><p xml:id=\"S2.Thmtheorem1.p1.1\">\
 <text font=\"italic\" xml:id=\"S2.Thmtheorem1.p1.1.1\">C.</text></p></para></theorem>",
    )],
  );
}

/// llncs.cls:1058, :1141 and svjour3.cls:804, :887: the separator between a within-counter's number and a theorem's
/// own, `\@thmcountersep`, is empty unless envcountsect, so a paper's own `\newtheorem{obs}{Observation}[section]`
/// prints "Observation 11", "A1" (pdflatex), "1.1" under envcountsect. Red (int53a, 78q r1; review finding 8): 1.1,
/// 1.2, 2.1, A.1. Repros cross-references/{llncs,svjour3}_own_theorem_separator{,_envcountsect}.
#[test]
fn theorem_counter_separator_is_the_class_s() {
  assert_numbers(
    repro!("llncs_own_theorem_separator"),
    "s1@S1=1 o1@S1.Thmobs1=11 o2@S1.Thmobs2=12 s2@S2=2 o3@S2.Thmobs1=21 s3@A1=A o4@A1.Thmobs1=A1",
    &[(
      "theorem",
      "S1.Thmobs1",
      "<theorem class=\"ltx_theorem_obs\" inlist=\"thm theorem:obs\" labels=\"LABEL:o1\" xml:id=\"S1.Thmobs1\">\
 <tags><tag>Observation 11</tag><tag role=\"refnum\">11</tag><tag role=\"typerefnum\">Observation 11</tag>\
 </tags><title class=\"ltx_runin\"><tag><text font=\"bold\" xml:id=\"S1.Thmobs1.1\">Observation 11</text>\
 </tag></title><para xml:id=\"S1.Thmobs1.p1\"><p xml:id=\"S1.Thmobs1.p1.1\">\
 <text font=\"italic\" xml:id=\"S1.Thmobs1.p1.1.1\">A.</text></p></para></theorem>",
    )],
  );
  assert_numbers(
    repro!("svjour3_own_theorem_separator"),
    "s1@S1=1 o1@S1.Thmobs1=11 o2@S1.Thmobs2=12 s2@S2=2 o3@S2.Thmobs1=21 s3@A1=A o4@A1.Thmobs1=A1",
    &[(
      "theorem",
      "A1.Thmobs1",
      "<theorem class=\"ltx_theorem_obs\" inlist=\"thm theorem:obs\" labels=\"LABEL:o4\" xml:id=\"A1.Thmobs1\">\
 <tags><tag>Observation A1</tag><tag role=\"refnum\">A1</tag><tag role=\"typerefnum\">Observation A1</tag>\
 </tags><title class=\"ltx_runin\"><tag><text font=\"bold\" xml:id=\"A1.Thmobs1.1\">Observation A1</text>\
 </tag></title><para xml:id=\"A1.Thmobs1.p1\"><p xml:id=\"A1.Thmobs1.p1.1\">\
 <text font=\"italic\" xml:id=\"A1.Thmobs1.p1.1.1\">D.</text></p></para></theorem>",
    )],
  );
  for repro in [
    repro!("llncs_own_theorem_separator_envcountsect"),
    repro!("svjour3_own_theorem_separator_envcountsect"),
  ] {
    assert_numbers(
      repro,
      "s1@S1=1 o1@S1.Thmobs1=1.1 o2@S1.Thmobs2=1.2 s2@S2=2 o3@S2.Thmobs1=2.1 s3@A1=A o4@A1.Thmobs1=A.1",
      &[],
    );
  }
}

/// aipproc.cls:252-255, :267: the headings are unnumbered by default (`unnumberedheadings`, secnumdepth
/// −`\maxdimen`): no section refnum, as pdflatex's `.aux` (`{}`). Red (int53a; Perl alike): 1, 2, 2.1, A. Repro
/// cross-references/aipproc_headings_unnumbered.
#[test]
fn aipproc_headings_unnumbered() {
  assert_numbers(
    repro!("aipproc_headings_unnumbered"),
    "sA@Sx1=- e1@Sx1.E1=1 f1@Sx1.F1=1 t1@Sx1.T1=1 sB@Sx2=- sC@Sx2.SSx1=- e2@Sx2.E2=2 e3@Sx2.E3=3 sD@Ax1=- e4@Ax1.E4=4 sE@Ax1.SSx1=- f2@Ax1.F2=2 t2@Ax1.T2=2 sG@Ax2=- e5@Ax2.E5=5 f3@Ax2.F3=3",
    &[(
      "equation",
      "Ax1.E4",
      "<equation labels=\"LABEL:e4\" xml:id=\"Ax1.E4\"><tags><tag>(4)</tag><tag role=\"refnum\">4</tag></tags>\
 <Math mode=\"display\" tex=\"x\" text=\"x\" xml:id=\"Ax1.E4.m1\"><XMath xml:id=\"Ax1.E4.m1.1\">\
 <XMTok font=\"italic\" role=\"UNKNOWN\">x</XMTok></XMath></Math></equation>",
    )],
  );
}
/// Convert `tex` beside the class copy `files`: 0 errors, 0 warnings, the labels' ids and refnums as `labels`.
fn assert_copy_numbers(tex: &str, files: &[(&str, &str)], labels: &str) {
  let (log, xml) = convert_files_with(tex, files, Some(PRELOAD));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(label_refnums(&xml), labels, "{xml}");
}

/// Where a class's versions conflict the paper's copy decides (OXIDIZED_DESIGN_DIVERGENCES #860): an elsarticle v1.0.1
/// copy (2008/05/22, 0711.2399) has no `\appendix`, so the article's numbers the appendix (pdflatex with that copy:
/// "A", 4, figures 2, 3); an emulateapj copy without `[appendixfloats]` whose appendix code zeroes the floats (11/26/04
/// to 08/13/06: 0704.2079, 0801.1670) numbers them "A1", "B1" always (pdflatex with the 11/26/04 copy), one whose code
/// does not (05/14/03 to 6/22/04: 0709.1471, 0803.1491, astro-ph0508218) puts the letter before the running number,
/// "A2", "B3" (pdflatex with 0709.1471's copy; red on 78q r1, review finding 2). The copies here are stubs of their
/// headers and appendix code, which is all the bindings read. Repros cross-references/class_copies_78q/.
#[test]
fn appendix_counters_follow_the_paper_s_copy() {
  assert_copy_numbers(
    repro!("class_copies_78q/elsarticle_v101_copy_keeps_article_appendix"),
    &[(
      "elsarticle.cls",
      include_str!(
        "../../../tools/perfect_kernel/repros/cross-references/class_copies_78q/elsarticle.cls"
      ),
    )],
    "sA@S1=1 e1@S1.E1=1 f1@S1.F1=1 t1@S1.T1=1 sB@S2=2 sC@S2.SS1=2.1 e2@S2.E2=2 e3@S2.E3=3 sD@A1=A e4@A1.E4=4 \
     sE@A1.SS1=A.1 f2@A1.F2=2 t2@A1.T2=2 sG@A2=B e5@A2.E5=5 f3@A2.F3=3",
  );
  assert_copy_numbers(
    repro!("class_copies_78q/emulateapj_old_copy_numbers_appendix_floats"),
    &[(
      "emulateapj.cls",
      include_str!(
        "../../../tools/perfect_kernel/repros/cross-references/class_copies_78q/emulateapj.cls"
      ),
    )],
    "sA@S1=1 e1@S1.E1=1 f1@S1.F1=1 t1@S1.T1=1 sB@S2=2 sC@S2.SS1=2.1 e2@S2.E2=2 e3@S2.E3=3 sD@A1=A e4@A1.E1=A1 \
     sE@A1.SS1=A.1 f2@A1.F1=A1 t2@A1.T1=A1 sG@A2=B e5@A2.E1=B1 f3@A2.F1=B1",
  );
  let (log, xml) = convert_files_with(
    repro!("class_copies_78q/emulateapj_letter_copy_keeps_running_float_numbers"),
    &[(
      "emulateapj.cls",
      include_str!(
        "../../../tools/perfect_kernel/repros/cross-references/class_copies_78q/emulateapj_letter.cls"
      ),
    )],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(
    label_refnums(&xml),
    "sA@S1=1 e1@S1.E1=1 f1@S1.F1=1 t1@S1.T1=1 sB@S2=2 sC@S2.SS1=2.1 e2@S2.E2=2 e3@S2.E3=3 sD@A1=A e4@A1.E1=A1 \
     sE@A1.SS1=A.1 f2@A1.F2=A2 t2@A1.T2=A2 sG@A2=B e5@A2.E1=B1 f3@A2.F3=B3",
    "{xml}"
  );
  assert_element(
    &xml,
    "figure",
    &["xml:id=\"A2.F3\""],
    "<figure inlist=\"lof\" labels=\"LABEL:f3\" placement=\"t\" xml:id=\"A2.F3\"><tags><tag>Figure B3</tag>\
 <tag role=\"autoref\">Figure\u{a0}B3<text xml:id=\"A2.F3.1\"/></tag><tag role=\"refnum\">B3</tag>\
 <tag role=\"typerefnum\">Figure B3</tag></tags><toccaption><tag close=\" \">B3</tag>F3</toccaption>\
 <caption><tag close=\".\u{2014} \">Figure B3</tag>F3</caption></figure>",
  );
}

/// A jfm.cls copy routed to the jfm binding (JFM-FLM_Au.cls:1317-1318, 2507.06135; `\documentclass{jfmcopy}`, the 77w
/// provides route) keeps the binding's numbering, which is now the class's: "(1.1)", "(2.1)", as its PDF (SYNC jfm
/// row,
/// closed). Red (int53a): (1), (2). Repro stand-in-copies/provides_route/copy_numbers_equations_by_section.
#[test]
fn jfm_copy_numbers_equations_by_section() {
  assert_copy_numbers(
    include_str!(
      "../../../tools/perfect_kernel/repros/stand-in-copies/provides_route/copy_numbers_equations_by_section.tex"
    ),
    &[(
      "jfmcopy.cls",
      include_str!(
        "../../../tools/perfect_kernel/repros/stand-in-copies/provides_route/jfmcopy.cls"
      ),
    )],
    "e@S2.E1=2.1",
  );
}
