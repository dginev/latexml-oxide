//! Red/green guards for batch 67z: journal class bindings define the commands their classes define (binding gaps G2:
//! sn-jnl, sigma, arxbj, informs, autart, asme2ej, aipproc, sagej, oup-authoring-template, ejpecp; the run-347
//! inventory bindgaps/g23_macros.tsv). Each repro is checked against pdflatex with a witness's copy of the class.
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

/// sn-jnl.cls (1802.08831's copy): `\nomail`, `\artnote`/`\miscnote` frontmatter notes, the title and table fonts,
/// `{tablenotes}` and `\tnote`, booktabs's rules, `\sidewaystablefn`, `{unenumerate}`, `{biography}` with its bold
/// `\author`, and the begin-document `\let\proof\spiproof` after a preamble `\let\proof\relax` (2211.03599).
#[test]
fn sn_jnl_class_commands() {
  let xml = clean!("snjnl_class_commands.tex");
  assert!(
    xml.contains(r#"<note role="artnote" xml:id="id1">An article note.</note>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<note role="miscnote" xml:id="id2">A misc note.</note>"#),
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
    xml.contains(r#"<text font="bold""#) && xml.contains(">Ann Author</text> is a researcher."),
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
  assert!(xml.contains(r#"<p xml:id="S1.p1.1">Text.</p>"#), "{xml}");
}

/// sagej.cls (2101.10072's copy): `\sagesf`, `{refsize}`, `{sm}`, `{biogs}`.
#[test]
fn sagej_class_commands() {
  let xml = clean!("sagej_class_commands.tex");
  assert!(
    xml.contains(r#"<text font="sansserif" xml:id="S1.p1.1.1">sans</text>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<acknowledgements inlist="toc" name="sm" xml:id="S1.acknowledgements2">Supplementary material is online.</acknowledgements>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<title>Author Biographies</title>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"Ann Author is a researcher.</text>"#),
    "{xml}"
  );
}

/// oup-authoring-template.cls (TL 2025): `\vol` and `\boxedtext` frontmatter notes (the box's heading leading its
/// text, TL 2025's "Key Messages"), `\secsize`.
#[test]
fn oup_authoring_template_class_commands() {
  let xml = clean!("oup_authoring_template_class_commands.tex");
  assert!(
    xml.contains(r#"<note role="volume" xml:id="id5">12</note>"#),
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
