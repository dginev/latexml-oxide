//! Red/green guards for batch CLUSTFIX, four Error roots of run 397 where TeX (or the class TeX would read) is clean: a
//! pasj00 paper without its class, the 1998 aipproc's `\author{name}` + `\affiliation`, a class's starred `\cite`
//! refused by the locked kernel `\cite`, and elsart's option switches. Repros under `tools/perfect_kernel/repros/`.
use latexml::util::test::{assert_element, convert_files_with, convert_with};

use super::perfect_kernel_batch46::{error_count, warning_count};

/// The counts `(errors, warnings)` of `log`, which must hold no Fatal.
fn counts(log: &str) -> (usize, usize) {
  assert!(!log.contains("Fatal:"), "{log}");
  (error_count(log), warning_count(log))
}

/// A pasj00 paper whose source lacks the class (OmniBus): the class's own `\SetRunningHead`, `\FigureFile`,
/// `\lesssim`, solar symbols and author/year citations (pasj00.cls:52-54, 4122-4164, 3445-3446, 3433-3444, 1636-1637;
/// astro-ph0108490, astro-ph0201077, astro-ph0204110: an Error each and the figures ERROR spans). The running head is
/// no content; the figure's file is at the class's width, 75 mm. Distinct from
/// `perfect_kernel_batch65h::pasj00_without_its_class_keeps_longtable` (the same branch, the class's tables). Repro
/// class-bindings/pasj00_unshipped_figurefile.
#[test]
fn pasj00_without_its_class_keeps_its_figures() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/class-bindings/pasj00_unshipped_figurefile.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!(counts(&log), (0, 0), "{log}");
  assert_element(
    &xml,
    "figure",
    &[],
    r#"<figure inlist="lof" xml:id="S0.F1"><tags><tag>Figure 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Figure 1</tag></tags><graphics graphic="./fig1.eps" options="width=213.39314pt,keepaspectratio=true" xml:id="S0.F1.g1"/><toccaption><tag close=" ">1</tag>Spectra.</toccaption><caption><tag close=": ">Figure 1</tag>Spectra.</caption></figure>"#,
  );
  assert!(
    !xml.contains("SiO Masers") && !xml.contains("Deguchi"),
    "{xml}"
  );
  assert!(
    xml.contains(r#"tex="\lesssim 5\,{{M}_{\mathbin{\odot}}}""#),
    "{xml}"
  );
  // `\authorcite`/`\yearcite` (pasj00.cls:1636-1637): natbib's author-only and year-only citations
  assert!(
    xml.contains(
      r#"accuracy, as <cite class="ltx_citemacro_citeauthor"><bibref bibrefs="m_89" separator=";" show="Authors" yyseparator=","/></cite> found (<cite class="ltx_citemacro_citeyear"><bibref bibrefs="m_89" separator=";" show="Year" yyseparator=","/></cite>).</p>"#
    ),
    "{xml}"
  );
}

/// The 1998 aipproc.cls (v1.0a, astro-ph0003214's copy; the run-397 `expected {` cluster) takes `\author{name}` and
/// `\affiliation{text}` (aipproc.cls:85-89), its own placeholders included: the binding's `\author` reads keyvals only
/// before a `{`, and the copy's `\affiliation` is the author's affiliation. The class's placeholders stay its defaults,
/// replaced by the document's calls. Repro stand-in-copies/aipproc_v1.
#[test]
fn aipproc_old_author_affiliation() {
  let (log, xml) = convert_files_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/stand-in-copies/aipproc_v1/aipproc_old_author_affiliation.tex"
    ),
    &[(
      "aipproc.cls",
      include_str!("../../../tools/perfect_kernel/repros/stand-in-copies/aipproc_v1/aipproc.cls"),
    )],
    Some("ar5iv.sty"),
  );
  assert_eq!(counts(&log), (0, 0), "{log}");
  assert_element(
    &xml,
    "creator",
    &[],
    "<creator role=\"author\"><personname>Donald C.\u{a0}Ellison</personname><contact name=\"Affiliation:\u{a0}\" \
     role=\"affiliation\">Department of Physics, NCSU</contact></creator>",
  );
  assert_eq!(xml.matches("<creator").count(), 1, "{xml}");
  assert!(!xml.contains(" here"), "{xml}");
  // The later classes' `\author{name}{keys}` keeps its keyvals.
  let modern = r"\documentclass{aipproc}
\begin{document}
\title{T}
\author{A. Author}{address={Dept, Univ}, email={a@b.c}}
\maketitle
Text.
\end{document}";
  let (log, xml) = convert_with(modern, Some("ar5iv.sty"));
  assert_eq!(counts(&log), (0, 0), "{log}");
  assert_element(
    &xml,
    "creator",
    &[],
    "<creator role=\"author\"><personname>A. Author</personname><contact name=\"Address:\u{a0}\" \
     role=\"address\">Dept, Univ</contact><contact name=\"Email:\u{a0}\" role=\"email\">a@b.c</contact></creator>",
  );
}

/// A class's `\def\cite{\@ifstar{\citeyear}{\citefull}}` (EslabStyle.cls:1443, ISOStyle.cls:1447; astro-ph0001290,
/// astro-ph0008362, astro-ph0005563), refused by the locked kernel `\cite` (OXIDIZED_DESIGN_DIVERGENCES #88): the
/// kernel `\cite` reads the star and drops it, so `\cite*{c_89}` cites c_89, not a key `*`. Repro
/// index-bib/starred_cite.
#[test]
fn starred_cite_under_the_lock() {
  let (log, xml) = convert_files_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/index-bib/starred_cite/starred_cite_under_the_lock.tex"
    ),
    &[(
      "starcite.sty",
      include_str!("../../../tools/perfect_kernel/repros/index-bib/starred_cite/starcite.sty"),
    )],
    Some("ar5iv.sty"),
  );
  assert_eq!(counts(&log), (0, 0), "{log}");
  assert_element(
    &xml,
    "p",
    &[],
    r#"<p xml:id="p1.1">As shown by <cite class="ltx_citemacro_cite">[<bibref bibrefs="c_89" separator="," yyseparator=","/>]</cite> and <cite class="ltx_citemacro_cite">[<bibref bibrefs="c_89" separator="," yyseparator=","/>]</cite>.</p>"#,
  );
}

/// elsart's options are the class's switches (elsart.cls:34-35, 62-65, 73-74, v2.20), which a journal style built on
/// it tests: yjsco.sty:98 `\if@amsthm` (0708.3431; the run-397 `(yjsco)` cluster). Repro
/// class-bindings/elsart_amsthm_option_switch.
#[test]
fn elsart_amsthm_option_is_the_class_switch() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/class-bindings/elsart_amsthm_option_switch.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!(counts(&log), (0, 0), "{log}");
  assert_element(&xml, "p", &[], r#"<p xml:id="p1.1">amsthm seceqn</p>"#);
}
