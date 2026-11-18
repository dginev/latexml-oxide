//! Red/green guards for batch 67z: journal class bindings define the commands their classes define (binding gaps G2:
//! sn-jnl, sigma, arxbj, informs, autart, asme2ej, aipproc, sagej, oup-authoring-template, ejpecp; the run-347
//! inventory bindgaps/g23_macros.tsv). Each repro is checked against pdflatex with a witness's copy of the class.
use latexml::util::test::{convert_bytes_files_then, convert_files_with, convert_with};

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

/// sigma.cls:77-83, 161-191 (0705.3628, 1404.0720): `\EmailDD`/`\URLaddressDD` frontmatter notes (an empty one
/// prints nothing), the starred theorems unnumbered, the class's own counters, and `\tsep`/`\bsep` struts.
#[test]
fn sigma_class_commands() {
  let xml = clean!("sigma_class_commands.tex");
  assert!(
    xml.contains(r#"<note role="email" xml:id="id2">ann@example.org</note>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<note role="url" xml:id="id3">https://example.org</note>"#),
    "{xml}"
  );
  assert_eq!(xml.matches("<note ").count(), 3, "{xml}");
  assert!(
    xml.contains(r#"<tag role="refnum">1</tag> <tag role="typerefnum">Theorem 1</tag>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<theorem class="ltx_theorem_theorem" inlist="thm theorem:theorem*" xml:id="Thmtheoremx1"> <tags> <tag>Theorem</tag>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<theorem class="ltx_theorem_remark" inlist="thm theorem:remark*" xml:id="Thmremarkx1"> <tags> <tag>Remark</tag>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<td align="center" border="b t" xml:id="p1.1.1.1">A</td>"#),
    "{xml}"
  );
}

/// sn-jnl.cls (1802.08831's copy): `\nomail`, `\artnote`/`\miscnote` frontmatter notes (after the starred author's
/// correspondent note, `id1`), the title and table fonts,
/// `{tablenotes}` and `\tnote`, booktabs's rules, `\sidewaystablefn`, `{unenumerate}`, `{biography}` with its bold
/// `\author`, and the begin-document `\let\proof\spiproof` after a preamble `\let\proof\relax` (2211.03599).
#[test]
fn sn_jnl_class_commands() {
  let xml = clean!("snjnl_class_commands.tex");
  assert!(
    xml.contains(r#"<note role="artnote" xml:id="id2">An article note.</note>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<note role="miscnote" xml:id="id3">A misc note.</note>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<text fontsize="171%" xml:id="S1.p1.1.1">Big</text>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<td align="center" border="tt" xml:id="S1.T1.2.1.2">B<sup xml:id="S1.T1.2.1.2.1"><text font="italic" xml:id="S1.T1.2.1.2.1.1">a</text></sup></td>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<p xml:id="S1.I1.ix1.p1.1">The note.</p>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<p xml:id="S1.I2.i2.p1.1">Second</p>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<proof xml:id="S1.2"> <title class="ltx_runin" font="italic">Proof.</title>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<p xml:id="S1.p4.1"><text font="bold" xml:id="S1.p4.1.1">Ann Author</text> is a researcher.</p>"#),
    "{xml}"
  );
  assert_eq!(xml.matches("<creator ").count(), 2, "{xml}");
}

/// informs3.cls (1501.01126's copy): the bold symbols, the four subject classifications, `\proof{head}` titled by its
/// argument (1301.2609), `\halmos`, `\ECRepeatTheorems`, the table struts, `\NatBibNumeric`.
#[test]
fn informs_class_commands() {
  let xml = clean!("informs_class_commands.tex");
  assert!(
    xml.contains(r#"<classification name="MSC2000 subject classification: " scheme="MSC">90C15</classification>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<classification name="OR/MS subject classification: " scheme="OR/MS">Programming: stochastic</classification>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<classification name="Subject classifications: " scheme="subject">Inventory</classification>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<classification name="Area of review: " scheme="area of review">Optimization</classification>"#),
    "{xml}"
  );
  assert!(xml.contains(r#"<XMTok font="bold" role="UNKNOWN">x</XMTok> <XMTok font="caligraphic bold" role="UNKNOWN">X</XMTok>"#), "{xml}");
  assert!(
    xml.contains(r#"<XMTok font="bold italic" name="beta" role="UNKNOWN">β</XMTok>"#),
    "{xml}"
  );
  // pdflatex: "Theorem 1 (Name)." — the class's TH style ends the head with a period (informs3.cls:1818).
  assert!(xml.contains(r#"<title class="ltx_runin"><tag><text font="bold" xml:id="Thmtheorem1.1">Theorem 1</text></tag><text font="bold" xml:id="Thmtheorem1.2"> </text>(Name)<text font="bold" xml:id="Thmtheorem1.3">.</text></title>"#), "{xml}");
  assert!(xml.contains(r#"<proof xml:id="S1.1"> <title font="italic">Proof.</title> <para xml:id="S1.p2"> <p xml:id="S1.p2.1">Trivial."#), "{xml}");
  assert!(xml.contains(r#"<proof xml:id="S1.2"> <title font="italic">Proof of Theorem 1.</title> <para xml:id="S1.p3"> <p xml:id="S1.p3.1">Also trivial.</p>"#), "{xml}");
  assert_eq!(xml.matches("Proof.").count(), 1, "{xml}");
  assert!(xml.contains(r#"<theorem class="ltx_theorem_repeattheorem" inlist="thm theorem:repeattheorem" xml:id="Thmrepeattheorem1">"#), "{xml}");
}

/// autart.cls (1112.0879's copy): elsart's `{keyword}` with `\sep` and `\PACS`, the class's math shorthands and its
/// theorem set on `{thm}`'s counter, `{case}` on its own.
#[test]
fn autart_class_commands() {
  let xml = clean!("autart_class_commands.tex");
  assert!(
    xml.contains(r#"<keywords name="Keywords: "> Control, Estimation, Filtering </keywords>"#),
    "{xml}"
  );
  assert!(
    xml.contains(
      r#"<classification name="PACS: " scheme="PACS">13.85.Hd, 12.40.Ee </classification>"#
    ),
    "{xml}"
  );
  assert!(
    xml.contains(r#"tex="\mathop{\mathrm{e}}\nolimits^{x}""#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<XMTok font="blackboard" role="UNKNOWN">R</XMTok>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<tag>Theorem 1</tag>"#) && xml.contains(r#"<tag>Lemma 2</tag>"#),
    "{xml}"
  );
  assert!(xml.contains(r#"<tag>Case 1</tag>"#), "{xml}");
}

/// asme2ej.cls:1209-1246 (1212.4243's copy): `{nomenclature}` with `\entry`, `{acknowledgment}`, `\qed`, the fixed fonts.
#[test]
fn asme2ej_class_commands() {
  let xml = clean!("asme2ej_class_commands.tex");
  assert!(
    xml.contains(r#"<section xml:id="Sx1"> <title>Nomenclature</title>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"</Math> Position</p>"#) && xml.contains(r#"</Math> Time</p>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<acknowledgements inlist="toc" name="Acknowledgements" xml:id="S1.acknowledgements1">We thank everyone. </acknowledgements>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<XMTok name="Box" role="UNKNOWN">□</XMTok>"#),
    "{xml}"
  );
}

/// aipproc.cls:405 loads aipxfm: a paper's own `\usepackage{aipxfm}` is a no-op, and the class's designer commands
/// are defined (1010.4521).
#[test]
fn aipproc_paper_loads_aipxfm() {
  let xml = clean!("aipproc_paper_loads_aipxfm.tex");
  assert!(
    xml.contains(r#"<creator role="author"> <personname>Ann Author</personname> <contact name="Address: " role="address">Some University</contact> </creator>"#),
    "{xml}"
  );
  // (the class's headings are unnumbered, `unnumberedheadings`: the section's id `Sx1`; 78q, KNOWN_PERL_ERRORS #1101)
  assert!(xml.contains(r#"<p xml:id="Sx1.p1.1">Text.</p>"#), "{xml}");
}

/// sagej.cls (2101.10072's, 2505.13309's, 2406.06038's, 2204.02460's copies): `\sagesf`, `{refsize}`, `{biogs}`, and
/// each end-matter statement named by the heading its class prints, which the XHTML shows as the statement's heading
/// (`@name`; "sm." before).
#[test]
fn sagej_class_commands() {
  let xml = clean!("sagej_class_commands.tex");
  assert!(
    xml.contains(r#"<text font="sansserif" xml:id="S1.p1.1.1">sans</text>"#),
    "{xml}"
  );
  for (n, name, text) in [
    (2, "Funding", "Some grant."),
    (3, "Declaration of conflicting interests", "None declared."),
    (4, "Author contributions", "Ann wrote it."),
    (5, "Declaration of conflicting interests", "No conflicts."),
    (6, "Data Accessibility Statement", "The data is online."),
    (
      7,
      "Supplemental material",
      "Supplementary material is online.",
    ),
  ] {
    let element = format!(
      r#"<acknowledgements inlist="toc" name="{name}" xml:id="S1.acknowledgements{n}">{text}</acknowledgements>"#
    );
    assert!(xml.contains(&element), "{element}: {xml}");
  }
  assert!(
    xml.contains(r#"<subsection xml:id="S1.SSx1"> <title>Author Biographies</title> <para class="ltx_noindent" xml:id="S1.SSx1.p1"> <p xml:id="S1.SSx1.p1.1"><text fontsize="90%" xml:id="S1.SSx1.p1.1.1">Ann Author is a researcher.</text>"#),
    "{xml}"
  );
}

/// 2503.14633's sagej.cls:447-449 heads `{sm}` "Supplemental Material", which the binding reads from the paper's copy
/// (the other 78 sampled copies: "Supplemental material").
#[test]
fn sagej_sm_heading_follows_the_copy() {
  let (log, xml) = convert_files_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/class-bindings/sagej_v2503/sagej_sm_heading_follows_the_copy.tex"
    ),
    &[(
      "sagej.cls",
      include_str!("../../../tools/perfect_kernel/repros/class-bindings/sagej_v2503/sagej.cls"),
    )],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let xml = squash(&xml);
  assert!(
    xml.contains(r#"<acknowledgements inlist="toc" name="Supplemental Material" xml:id="S1.acknowledgements1">Supplementary material is online.</acknowledgements>"#),
    "{xml}"
  );
}

/// oup-authoring-template.cls (TL 2025): `\vol` and `\boxedtext` frontmatter notes (the box's heading leading its
/// text, TL 2025's "Key Messages"), `\secsize`. The tree's v1.2 (2026-01-02, cls:1263-1264) takes the heading as a first
/// argument (the one-argument call swallows `\keywords`, pdflatex prints "-8pt"): the repro says it there.
#[test]
fn oup_authoring_template_class_commands() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/oup_authoring_template_class_commands.tex"
  );
  let two_args = latexml::util::test::kpse_text("oup-authoring-template.cls")
    .is_some_and(|cls| cls.contains(r"\newcommand{\@@boxedtext}[2]"));
  let tex = if two_args {
    tex.replace(r"\boxedtext{", r"\boxedtext{Key Messages}{")
  } else {
    tex.to_string()
  };
  let (log, xml) = convert_with(&tex, Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let xml = squash(&xml);
  assert!(
    // (`id6`: the address note, which the class prints under the names, comes first, before the abstract: BLOCKPLACE,
    // OD #1074)
    xml.contains(r#"<note role="volume" xml:id="id6">12</note>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<note role="boxedtext" xml:id="id10"><text font="bold" xml:id="id10.1">Key Messages</text> <itemize xml:id="S0.I1">"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<p xml:id="S0.I1.i1.p1.1">One point.</p>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<text font="bold" xml:id="S1.p1.1.1">Heading font</text> text."#),
    "{xml}"
  );
}

/// ejpecp.cls (TL 2025): `\ARXIVID`/`\HALID` notes, `\ACKNO`, and the class's theorem set numbered within sections.
#[test]
fn ejpecp_class_commands() {
  let xml = clean!("ejpecp_class_commands.tex");
  assert!(
    xml.contains(r#"<note role="supersedes" xml:id="id3">Supersedes <ref class="ltx_href" font="typewriter" href="https://arXiv.org/abs/1234.5678">arXiv:1234.5678</ref>.</note>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<ref class="ltx_href" font="typewriter" href="https://hal.archives-ouvertes.fr/hal-00000001">HAL:hal-00000001</ref>"#),
    "{xml}"
  );
  for tag in [
    "Theorem 1.1",
    "Notation 1.2",
    "Hypothesis 1.3",
    "Condition 1.4",
  ] {
    assert!(xml.contains(&format!("<tag>{tag}</tag>")), "{tag}: {xml}");
  }
  assert!(xml.contains(r#"<acknowledgements inlist="toc" xml:id="S1.acknowledgements1">We thank everyone.</acknowledgements>"#), "{xml}");
}

/// arxbj.cls (1106.2467's copy): the VTeX URL helpers, `\docsubty`, `{keyword}` with `\kwd`, `\eqntext`, the
/// starred twin of every `\newtheorem`/`\newproclaim`, and `{supplement}`.
#[test]
fn arxbj_class_commands() {
  let xml = clean!("arxbj_class_commands.tex");
  // Both blocks kept: the AMS one the subject classification (arxbj.cls:2495), its scheme the printed heading the HTML
  // shows (86a: was the key, "AMS"), the other the keywords.
  assert!(
    xml.contains(r#"<classification name="AMS 2000 subject classifications: " scheme="AMS 2000 subject classifications">Primary 60F05; secondary 62G20 </classification>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<keywords name="Keywords: ">one; two </keywords>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"See <ref class="ltx_href" href="http://arxiv.org/abs/1234.5678">1234.5678</ref> and <ref class="ltx_href" href="http://dx.doi.org/10.1000/x">10.1000/x</ref>."#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<theorem class="ltx_theorem_cnd" inlist="thm theorem:cnd*" xml:id="Thmcndx1"> <tags> <tag>Condition</tag>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<XMText xml:id="S1.E1.m4.1.1.2">for all </XMText>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<p xml:id="Sx1.p1.1"><text font="bold" xml:id="Sx1.p1.1.1">Supplement A: Proofs </text>(DOI: <ref class="ltx_href" href="http://dx.doi.org/10.3150/00-BEJ000SUPP">10.3150/00-BEJ000SUPP</ref>; .pdf). All the proofs.</p>"#),
    "{xml}"
  );
}

/// arximspdf.cls (1107.3284's copy :2404-2450, 1287-1337): both `{keyword}` blocks kept — the AMS one, which the
/// keywords block after it replaced (1107.3284, 1309.4864), the subject classification — their entries joined by
/// ", " with no trailing separator, under the class's headings; and vtexthm's starred twin of a `\newproclaim`
/// (`{hy*}`, 1107.3284).
#[test]
fn arximspdf_keyword_blocks_and_starred_theorems() {
  let xml = clean!("arximspdf_keyword_blocks_and_starred_theorems.tex");
  assert!(
    xml.contains(r#"<classification name="AMS 2000 subject classifications. " scheme="AMS 2000 subject classifications">Primary 65C05, 91G60 </classification>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<keywords name="Key words and phrases. ">Monte Carlo methods, importance sampling </keywords>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<theorem class="ltx_theorem_hy" inlist="thm theorem:hy*" xml:id="Thmhyx1"> <tags> <tag>Hypotheses</tag>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<tag role="typerefnum">Theorem 1.1</tag>"#),
    "{xml}"
  );
}

/// sn-jnl's further author notes (2607.26008's copy :591-600, 2307.08214's :591-595): `\equalcont`, `\intern` and
/// `\equalsup` kept with the author's text, the starred author's correspondent note beside them (SNJNL): with no copy,
/// the class as distributed, `\equalcont` marks its author † and the others, which that class lacks and whose glyph no
/// copy gives, are the document's notes.
#[test]
fn snjnl_author_notes() {
  let xml = clean!("snjnl_author_notes.tex");
  for element in [
    r#"<personname>Ann Author</personname> <contact name="Email: " role="email">ann@example.org</contact> <contact name="Affiliation: " role="affiliation">Some University</contact> <note mark="*" role="correspondent" xml:id="id1">Corresponding author(s).</note> <note mark="†" role="equal-contributors" xml:id="id2">These authors contributed equally.</note> </creator>"#,
    r#"<personname>Bob Writer</personname> <contact name="Affiliation: " role="affiliation">Some University</contact> </creator>"#,
    r#"</abstract> <note role="equal-supervision" xml:id="id3">Equal supervision.</note> <note role="intern" xml:id="id4">Work done as an intern.</note>"#,
  ] {
    assert!(xml.contains(element), "{element}: {xml}");
  }
  assert_eq!(xml.matches("<note ").count(), 4, "{xml}");
}

/// A copy whose author notes take their text as an optional argument, the class's default in its place (2606.24824's
/// `\intern`, 2504.21788's `\equalcont`): a bare `\equalcont` or an `\intern{}` is the copy's default text, an
/// `\intern[text]` its text, and the class prints the last text once under the glyph its copy appends (†, ‡), so the
/// two authors share one note of each (SNJNL): `\intern{}`'s default, which the second `\intern[…]` replaces before
/// `\maketitle`, prints nowhere. The binding's one-argument forms read the next command as the text
/// (`\equalcont\intern`: every note empty).
#[test]
fn snjnl_notes_take_the_copys_default() {
  let (log, xml) = convert_files_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/class-bindings/snjnl_v2606/snjnl_notes_take_the_copys_default.tex"
    ),
    &[(
      "sn-jnl.cls",
      include_str!("../../../tools/perfect_kernel/repros/class-bindings/snjnl_v2606/sn-jnl.cls"),
    )],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let xml = squash(&xml);
  for element in [
    r#"<personname>Ann Author</personname> <contact name="Email: " role="email">ann@example.org</contact> <contact name="Affiliation: " role="affiliation">Some University</contact> <note mark="*" role="correspondent" xml:id="id3">Corresponding author(s).</note> <note mark="†" role="equal-contributors" xml:id="id1">These authors contributed equally to this work.</note> <note mark="‡" role="intern" xml:id="id2">Work done as a visitor.</note> </creator>"#,
    r#"<personname>Bob Writer</personname> <contact name="Affiliation: " role="affiliation">Some University</contact> <note idref="id1" mark="†" role="equal-contributors" xml:id="id4">These authors contributed equally to this work.</note> <note idref="id2" mark="‡" role="intern" xml:id="id5">Work done as a visitor.</note> </creator>"#,
  ] {
    assert!(xml.contains(element), "{element}: {xml}");
  }
}

/// sn-jnl's author note that marks no name is the document's note: one given before any `\author`, whose glyph prints
/// before the first name, one given inside a name, which runs as the names print (2412.01641's sn-jnl.cls:586, :754;
/// SNJNL, OD #1066), and those of a document with no `\maketitle`, which prints no title block: the last text once
/// (SNJNLb, review revSNJNL S1: the text had been dropped with no Warning).
#[test]
fn snjnl_notes_marking_no_name_are_the_documents() {
  let unmailed = r#"<creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Univ A</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Univ A</contact> </creator>"#;
  let mailed = r#"<creator role="author"> <personname>Ann Able</personname> <contact name="Email: " role="email">ann@a.org</contact> <contact name="Affiliation: " role="affiliation">Univ A</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Email: " role="email">bob@b.org</contact> <contact name="Affiliation: " role="affiliation">Univ A</contact> </creator>"#;
  for (repro, creators, text) in [
    (
      include_str!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/snjnl_note_before_any_name.tex"
      ),
      unmailed,
      "Equal before.",
    ),
    (
      include_str!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/snjnl_note_inside_a_name.tex"
      ),
      unmailed,
      "Equal inside.",
    ),
    (
      include_str!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/snjnl_no_maketitle_notes_are_the_documents.tex"
      ),
      mailed,
      "Equal work.",
    ),
  ] {
    let (log, xml) = convert_with(repro, Some("ar5iv.sty"));
    assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
    let xml = squash(&xml);
    for element in [
      creators.to_string(),
      format!(r#"<note role="equal-contributors" xml:id="id1">{text}</note>"#),
    ] {
      assert!(xml.contains(&element), "{element}: {xml}");
    }
    assert_eq!(xml.matches("<note ").count(), 1, "{xml}");
    assert_eq!(xml.matches(text).count(), 1, "{text}: {xml}");
  }
}

/// sn-jnl's notes with a section before `\maketitle`, whose fallback flush digests the names: the class prints the notes
/// once, at `\maketitle` (2412.01641's sn-jnl.cls:749-756), so the binding queues them there alone, never at the section
/// (revSNJNLb B2: the section's copy printed each twice; 2510.03878's `\subsection*` in its `\abstract`). A note given
/// after the section marks no name the section wrote: the document's, unmarked (row R SNJNLAFTERSEC).
#[test]
fn snjnl_notes_before_a_section_and_maketitle_print_once() {
  let ann_star = r#"<creator role="author"> <personname>Ann Able</personname> <contact name="Email: " role="email">ann@a.org</contact> <contact name="Affiliation: " role="affiliation">Univ A</contact> <note mark="*" role="correspondent" xml:id="id1">Corresponding author(s).</note>"#;
  let bob = r#"<creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Univ A</contact> </creator>"#;
  for (repro, elements, text, notes) in [
    (
      include_str!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/snjnl_note_before_section_and_maketitle_prints_once.tex"
      ),
      vec![format!(
        r#"{ann_star} <note mark="†" role="equal-contributors" xml:id="id2">Outer.</note> </creator> {bob}"#
      )],
      "Outer.",
      2,
    ),
    (
      include_str!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/snjnl_note_inside_a_name_before_section_and_maketitle_prints_once.tex"
      ),
      vec![
        format!(
          r#"<creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Univ A</contact> </creator> {bob}"#
        ),
        r#"</abstract> <note role="equal-contributors" xml:id="id1">Inside only.</note> <section"#
          .to_string(),
      ],
      "Inside only.",
      1,
    ),
    (
      include_str!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/snjnl_note_between_section_and_maketitle_red.tex"
      ),
      vec![
        format!("{ann_star} </creator> {bob}"),
        r#"</abstract> <note role="equal-contributors" xml:id="id2">Between.</note> <section"#
          .to_string(),
      ],
      "Between.",
      2,
    ),
  ] {
    let (log, xml) = convert_with(repro, Some("ar5iv.sty"));
    assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
    let xml = squash(&xml);
    for element in elements {
      assert!(xml.contains(&element), "{element}: {xml}");
    }
    assert_eq!(xml.matches(text).count(), 1, "{text}: {xml}");
    assert_eq!(xml.matches("<note ").count(), notes, "{xml}");
  }
}

/// 2501.01157's copy appends a math `$^{*}$` for `\equalcont` (:587), printing ∗, beside the starred names' text `*`
/// (:513): two marks of two notes, which never answer each other (revSNJNLb S3). With a corresponding line Ann carries
/// both notes; with none her `*` shows (written ∗: row R SNJNLSTARGLYPH), her note's ∗ answering it no more (a declared
/// label, `ltx_note_label`, which the page's fold never links: OD #1068).
#[test]
fn snjnl_math_star_note_is_not_the_class_star() {
  let copy = (
    "sn-jnl.cls",
    include_str!("../../../tools/perfect_kernel/repros/class-bindings/snjnl_v2501/sn-jnl.cls"),
  );
  for (repro, creators) in [
    (
      include_str!(
        "../../../tools/perfect_kernel/repros/class-bindings/snjnl_v2501/snjnl_math_star_note_beside_the_class_star.tex"
      ),
      r#"<creator role="author"> <personname>Ann Able</personname> <contact name="Email: " role="email">ann@a.org</contact> <contact name="Affiliation: " role="affiliation">Univ A</contact> <note mark="*" role="correspondent" xml:id="id1">Corresponding author(s).</note> <note mark="∗" role="equal-contributors" xml:id="id2">Equal work.</note> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Email: " role="email">bob@b.org</contact> <contact name="Affiliation: " role="affiliation">Univ A</contact> <note idref="id2" mark="∗" role="equal-contributors" xml:id="id3">Equal work.</note> </creator> <creator before=" " role="author"> <personname>Cat Cole</personname> <contact name="Email: " role="email">cat@c.org</contact> <contact name="Affiliation: " role="affiliation">Univ A</contact> <note idref="id1" mark="*" role="correspondent" xml:id="id4">Corresponding author(s).</note> </creator>"#,
    ),
    (
      include_str!(
        "../../../tools/perfect_kernel/repros/class-bindings/snjnl_v2501/snjnl_math_star_note_beside_a_shown_star.tex"
      ),
      r#"<creator role="author"> <personname>Ann Able</personname> <note class="ltx_note_label" mark="∗" role="footnotemark" xml:id="id2"/> <contact name="Affiliation: " role="affiliation">Univ A</contact> <note mark="∗" role="equal-contributors" xml:id="id1">Equal work.</note> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Univ A</contact> <note idref="id1" mark="∗" role="equal-contributors" xml:id="id3">Equal work.</note> </creator>"#,
    ),
  ] {
    let (log, xml) = convert_files_with(repro, &[copy], Some("ar5iv.sty"));
    assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
    let xml = squash(&xml);
    assert!(xml.contains(creators), "{creators}: {xml}");
  }
}

/// A copy that prints no `*` for a starred name, no corresponding line and no glyph for `\equalcont` (2209.11450's
/// sn-jnl.cls :506-516, :587, :763): the binding reads it, so no name is marked and no correspondent note is invented;
/// the `\equalcont` text, printed under the addresses alone, is the document's note (SNJNL, OD #1066).
#[test]
fn snjnl_copy_that_marks_no_name() {
  let (log, xml) = convert_files_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/class-bindings/snjnl_v2209/snjnl_copy_marks_no_name.tex"
    ),
    &[(
      "sn-jnl.cls",
      include_str!("../../../tools/perfect_kernel/repros/class-bindings/snjnl_v2209/sn-jnl.cls"),
    )],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let xml = squash(&xml);
  let creators = r#"<creator role="author"> <personname>Ann Able</personname> <contact name="Email: " role="email">ann@a.org</contact> <contact name="Affiliation: " role="affiliation">Univ A</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Email: " role="email">bob@b.org</contact> <contact name="Affiliation: " role="affiliation">Univ B</contact> </creator>"#;
  assert!(xml.contains(creators), "{creators}: {xml}");
  let note = r#"<note role="equal-contributors" xml:id="id1">Journal note.</note>"#;
  assert!(xml.contains(note), "{note}: {xml}");
  assert_eq!(xml.matches("<note ").count(), 1, "{xml}");
}

/// oup-authoring-template's `\figalttext[short]{long}` (2602.07066's copy :1935, which drops it; 2608.22901,
/// 2608.14294, 2609.06413) is the image's description, as acmart's `\Description` (float_description.rs;
/// OXIDIZED_DESIGN_DIVERGENCES #604): the short text the image's `description` and the long one, with maths, its
/// hidden note; a plain long text alone the `description`.
#[test]
fn oup_figalttext_is_the_images_description() {
  let (log, xml, ()) = convert_bytes_files_then(
    include_bytes!(
      "../../../tools/perfect_kernel/repros/class-bindings/oup_v2602/oup_figalttext_is_the_images_description.tex"
    ),
    &[
      (
        "oup-authoring-template.cls",
        include_bytes!(
          "../../../tools/perfect_kernel/repros/class-bindings/oup_v2602/oup-authoring-template.cls"
        ),
      ),
      (
        "none.png",
        include_bytes!("../../../tools/perfect_kernel/repros/class-bindings/oup_v2602/none.png"),
      ),
    ],
    Some("ar5iv.sty"),
    |_, _| (),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let xml = squash(&xml);
  assert!(
    xml.contains(r#"<graphics aria:describedby="acmlabel1" candidates="none.png" description="A short alt text." graphic="none" xml:id="S0.F1.g1"/>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<caption><tag close=": ">Figure 1</tag>A caption.<note class="ltx_nodisplay ltx_acm_description_short" xml:id="acmlabel1-short">A short alt text.</note><note class="ltx_nodisplay ltx_acm_description" xml:id="acmlabel1">A long description with <Math mode="inline" tex="n_{1}\times p""#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<graphics candidates="none.png" description="A plain long alt text." graphic="none" xml:id="S0.F2.g1"/>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<figure inlist="lof" labels="LABEL:fig:one" xml:id="S0.F1">"#),
    "{xml}"
  );
}
