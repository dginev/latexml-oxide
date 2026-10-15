//! Red/green guards for batch 67y: the class bindings lacked commands their real class defines (binding gaps G2:
//! amsart, revtex4-1, revtex4-2, kluwer, elsarticle, elsart, egpubl), and the loader kept a package loaded that the
//! document had unloaded.
use latexml::util::test::convert_with;

use super::perfect_kernel_batch46::{error_count, warning_count};

/// The XML with every whitespace run collapsed to one space.
fn squash(xml: &str) -> String { xml.split_whitespace().collect::<Vec<_>>().join(" ") }

macro_rules! repro {
  ($path:literal) => {
    convert_with(
      include_str!(concat!("../../../tools/perfect_kernel/repros/", $path)),
      Some("ar5iv.sty"),
    )
  };
}

/// A clean conversion of a repro, squashed.
macro_rules! clean {
  ($path:literal) => {{
    let (log, xml) = repro!($path);
    assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
    squash(&xml)
  }};
}

/// amsart.cls:75-96 `\for`/`\except` keep the long title in the body (1304.2934, 1601.05128); :837, :856-864 a
/// document's `\abstractbox` environment set at `\maketitle` is the frontmatter abstract, one after it is set where it
/// ends (1512.02361, 2602.22909; 2303.07306, 2201.00740); :554/:564 the stores, :687 `\copyrightyear`, :1262
/// `\bibliofont`, :1213-1241 the contents-line code, whose `\addcontentsline` does not list the section in the
/// contents twice (2105.14892, 2004.14791, 2503.14638).
#[test]
fn amsart_class_internals_are_defined() {
  let xml = clean!("class-bindings/amsart_class_internals.tex");
  assert!(xml.contains(r#"<p xml:id="p1.1">Stores [].</p>"#), "{xml}");
  assert!(
    xml.contains(
      r#"<abstract inlist="toc" name="Abstract." xml:id="abstract1"> <block vattach="top" xml:id="abstract1.1"> <itemize xml:id="abstract1.1.1"> <item xml:id="S0.I1.ix1"> <tags> <tag><text font="smallcaps" xml:id="S0.I1.ix1.1">Abstract.</text></tag> <tag role="typerefnum">item <text font="smallcaps" xml:id="S0.I1.ix1.2">Abstract.</text></tag> </tags> <para xml:id="S0.I1.ix1.p1"> <p xml:id="S0.I1.ix1.p1.1"><text fontsize="80%" xml:id="S0.I1.ix1.p1.1.1">Before the title.</text></p> </para> </item> </itemize> </block> </abstract>"#
    ),
    "{xml}"
  );
  assert_eq!(xml.matches("<abstract ").count(), 1, "{xml}");
  assert!(
    xml.contains(r#"<section inlist="toc" xml:id="S1">"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<title><tag close=". ">1</tag>The long title</title>"#),
    "{xml}"
  );
  assert!(!xml.contains("Short"), "{xml}");
  assert!(
    xml.contains(
      r#"<p xml:id="S1.p1.1">Body [0000] <text fontsize="80%" xml:id="S1.p1.1.1">small</text>.</p>"#
    ),
    "{xml}"
  );
  assert!(
    xml.contains(
      r#"<tag><text font="smallcaps" xml:id="S1.I1.ix1.1">Acknowledgements.</text></tag>"#
    ) && xml.contains(
      r#"<p xml:id="S1.I1.ix1.p1.1"><text fontsize="80%" xml:id="S1.I1.ix1.p1.1.1">After the title.</text></p>"#
    ),
    "{xml}"
  );
}

/// amsart.cls:41-42 marks amsthm loaded; a document that lets `\ver@amsthm.sty` to `\relax` loads amsthm.sty again,
/// as LaTeX's `\@ifl@aded` does (1510.08833, 1704.05671, 1707.00258, 1611.00843, 2106.07418, 2304.10644): the class
/// binding's mark only (`amsthm.sty:class_marked`, `state::unmarked_by_document`).
#[test]
fn amsart_reloads_amsthm_after_unmarking_it() {
  let xml = clean!("loader/amsart_reloads_amsthm_after_unmarking_it.tex");
  assert!(
    xml.contains(r#"<p xml:id="p1.1">amsthm loaded.</p>"#),
    "{xml}"
  );
  assert!(
    xml.contains(
      r#"<proof xml:id="id1"> <title class="ltx_runin" font="italic">Proof.</title> <para xml:id="p2"> <p xml:id="p2.1"><text xml:id="p2.1.1">A proof. ∎</text></p> </para> </proof>"#
    ),
    "{xml}"
  );
}

/// revtex4-2.cls:1986-1988 `\doi`/`\doibase` (2010.08759, 2301.10789); ltxgrid/ltxutil/ltxfront internals documents
/// call: `\onecolumngrid@push`, `\let@environment`, `\frontmatter@title@format`, `\widetext@top`, `\altaddress`,
/// `\REV@text`, `\clo@groupedaddress`, `\onecolumn@grid@setup` (2311.06373, 1101.2683, 2006.03287, 2101.04636).
#[test]
fn revtex4_2_class_internals_are_defined() {
  let xml = clean!("class-bindings/revtex4_2_class_internals.tex");
  assert!(
    xml.contains(
      r#"<contact name="Affiliation: " role="affiliation">Univ</contact> <contact name="Affiliation: " role="affiliation">Elsewhere</contact>"#
    ),
    "{xml}"
  );
  assert!(
    xml.contains(
      r#"<p xml:id="p1.1">See doi:<ref class="ltx_href" href="https://doi.org/10.1103/PhysRevLett.80.1373">10.1103/PhysRevLett.80.1373</ref> and <ref class="ltx_href" href="https://doi.org/10.1007/JHEP07(2018)156">JHEP 07 (2018) 156</ref>. <text align="center" font="bold" fontsize="120%" xml:id="p1.1.1">Second title</text></p>"#
    ),
    "{xml}"
  );
  assert!(xml.contains(r#"tex="t=R(t)\text{ mod }1""#), "{xml}");
  assert!(xml.contains(r#"</Math> Set: Kept.</p>"#), "{xml}");
}

/// revtex4-1.cls's internals as revtex4-2's, and :1990-2009 the `\@@startlink{<url>}`…`\@@endlink` link pair, live
/// under hyperref (1211.1037's own `\Doi`).
#[test]
fn revtex4_1_class_internals_are_defined() {
  let xml = clean!("class-bindings/revtex4_1_class_internals.tex");
  assert!(
    xml.contains(
      r#"10.1103/PhysRevLett.80.1373</ref>, <ref href="http://dx.doi.org/10.1147/rd.53.0183">IBM J.</ref> and <ref class="ltx_href""#
    ),
    "{xml}"
  );
  assert!(xml.contains(r#"</Math> Set: Kept.</p>"#), "{xml}");
}

/// kluwer.cls's named references (`\inlinecite`, `\opencite`, `\shortcite`: 1306.4881, 1408.6262), endnotes
/// (`\theendnotes`: 0910.2393), symbols (`\lsim`, `\dR`: astro-ph0309194, math0212055), `{subequation}`, `\qed`,
/// `{motto}` (0912.3846, math0308074, 0802.0687).
#[test]
fn kluwer_class_commands_are_defined() {
  let xml = clean!("class-bindings/kluwer_class_commands.tex");
  assert!(
    xml.contains(
      r#"<quote xml:id="p1.1"> <p xml:id="p1.1.1"><text fontsize="80%" xml:id="p1.1.1.1">A short motto.</text></p> </quote>"#
    ),
    "{xml}"
  );
  assert!(
    xml.contains(
      r#"<p xml:id="p2.1"><cite class="ltx_citemacro_citet"><bibref bibrefs="jones" separator=";" show="Authors Phrase1YearPhrase2" yyseparator=","> <bibrefphrase>(</bibrefphrase> <bibrefphrase>)</bibrefphrase> </bibref></cite> showed;"#
    ),
    "{xml}"
  );
  assert!(
    xml.contains(
      r#"and <cite class="ltx_citemacro_citeyearpar">(<bibref bibrefs="jones" separator=";" show="Year" yyseparator=","/>)</cite>."#
    ),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<note inlist="ent" mark="1" role="endnote" xml:id="endnote1">"#)
      && xml.contains(
        r#"<TOC lists="ent" scope="global" show="refnum &gt; note"> <title>Notes</title> </TOC>"#
      ),
    "{xml}"
  );
  assert!(
    xml.contains(
      r#"<Math mode="inline" tex="a\lsim b\gsim c" text="a less-than-or-similar-to b greater-than-or-similar-to c" xml:id="p2.m1">"#
    ),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<XMTok meaning="reals" name="dR" role="ID">ℝ</XMTok>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<XMTok meaning="integers" name="dZ" role="ID">ℤ</XMTok>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<tag>(1a)</tag> <tag role="refnum">1a</tag>"#)
      && xml.contains(r#"<tag>(1b)</tag> <tag role="refnum">1b</tag>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<p xml:id="p2.2">Text. <Math mode="inline" tex="\Box""#),
    "{xml}"
  );
}

/// elsarticle.cls (TL v3.4c) `\tnotemark`, `\myfooter`, `\abstracttitle`, `\resetTitleCounters`, `\absbox`,
/// `\elsRef`, `\@journal` (2203.02441, 2209.12222, 2204.07076, 1007.3192, 1212.6083).
#[test]
fn elsarticle_class_internals_are_defined() {
  let xml = clean!("class-bindings/elsarticle_class_internals.tex");
  assert!(
    xml.contains(
      r#"<title>A title<note class="ltx_note_frontmatter ltx_thanks_note" role="thanks" xml:id="id1">A title note.</note></title>"#
    ),
    "{xml}"
  );
  assert!(
    xml.contains(
      r#"<abstract inlist="toc" name="Summary" xml:id="abstract1"> <p xml:id="abstract1.1">The abstract.</p> </abstract> <pubnote name="Journal: " role="journal">Physics Letters B</pubnote>"#
    ),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<p xml:id="p1.1">Journal: Physics Letters B. Ref [0].</p>"#),
    "{xml}"
  );
}

/// elsarticle v1.21's `\email`, `\correspondingauthor`, `{nomenclature}`, `{deflist}` and `\normalMode` (1305.6441,
/// 2103.08145, 1303.2653, 2404.00783): the sections after `\normalMode` are sections again.
#[test]
fn elsarticle_v1_21_commands_are_defined() {
  let xml = clean!("class-bindings/elsarticle_v1_21_commands.tex");
  assert!(
    xml.contains(
      r#"<creator role="author"> <personname>A. Author</personname> <contact name="Email: " role="email">a@b.org</contact> </creator> <note role="corresponding" xml:id="id1">Corresponding author</note>"#
    ),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<section xml:id="Sx1"> <title>Nomenclature</title>"#)
      && xml.contains(r#"</Math> </text>the unknown</p>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"</appendix> <section inlist="toc" xml:id="S1"> <tags> <tag>1</tag>"#)
      && xml.contains(r#"<title><tag close=" ">1</tag>Back</title>"#),
    "{xml}"
  );
}

/// elsart.cls `{pf}`/`{pf*}` (0708.4409, 0709.3642, 0811.2675), a `$$` display kept in the body; `\cty`, `\cny`, `\@journal`,
/// `\@blstr`, `\output@glob@notes` (1202.0185, 0907.1629, astro-ph0103363).
#[test]
fn elsart_class_commands_are_defined() {
  let xml = clean!("class-bindings/elsart_class_commands.tex");
  assert!(
    xml.contains(r#"<contact name="Address: " role="address">Univ, Paris, France</contact>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<p xml:id="p1.1">In J. Test. [1]</p>"#),
    "{xml}"
  );
  assert!(
    xml.contains(
      r#"<proof xml:id="id1"> <title font="bold">Proof.</title> <para xml:id="p2"> <p xml:id="p2.1">Plain.</p> </para> </proof>"#
    ),
    "{xml}"
  );
  assert!(
    xml.contains(
      r#"<proof xml:id="id2"> <title font="bold">Proof of the claim</title> <para xml:id="p3"> <p xml:id="p3.1">A boxed proof</p> <equation xml:id="S0.Ex1"> <Math mode="display" tex="x^{2}""#
    ),
    "{xml}"
  );
}

/// The MACMILLAN elsart5.cls copies' `\KEY` opens the keywords after `\PACS` (0801.1765, 0706.1711).
#[test]
fn elsart5_macmillan_key_starts_the_keywords() {
  let xml = clean!("class-bindings/elsart5_macmillan_key.tex");
  assert!(
    xml.contains(
      r#"<keywords name="Keywords: ">quantum lattice model, DMRG </keywords> <classification name="PACS: " scheme="PACS">02.70.-c </classification>"#
    ),
    "{xml}"
  );
}

/// egpubl.cls's `\p@ConfYear`, `\shortcite`, `\CCScat`, `\EuroVisTutorialEditors`, `\copyrightTextTitPag`
/// (1409.5623, 1710.04867, 1102.4258, 2604.14791, 2404.02634): the copyright prints its © once, the frontmatter's.
#[test]
fn egpubl_class_commands_are_defined() {
  let xml = clean!("class-bindings/egpubl_class_commands.tex");
  assert!(
    xml.contains(
      r#"<date name="©" role="copyright">2025 The Authors.</date> <note role="year" xml:id="id1">2025</note> <note role="editors-tutorial" xml:id="id2">T. Editor</note>"#
    ),
    "{xml}"
  );
  assert!(
    xml.contains(
      r#"<p xml:id="p1.1">Year (2025); see <cite class="ltx_citemacro_cite">[<bibref bibrefs="jon" separator="," yyseparator=","/>]</cite>. I.3.3 [Computer Graphics]: Picture/Image Generation—Line and curve generation</p>"#
    ),
    "{xml}"
  );
}

/// A document's `\NewDocumentEnvironment`/`\NewDocumentCommand` replaces a binding's guess (`Fallback`) and a kernel
/// stub (`Stub`: `\shorttitle`), as its `\newenvironment` and `\newcommand` do (2509.10728).
#[test]
fn ltcmd_declarators_replace_a_fallback_guess() {
  let xml = clean!("loader/ltcmd_declarators_replace_a_fallback_guess.tex");
  assert!(
    xml.contains(r#"<p xml:id="p1.1">MINE(2.5em)</p> <itemize xml:id="S0.I1">"#),
    "{xml}"
  );
  assert!(xml.contains(r#"<p xml:id="p1.2">Mail: a@b</p>"#), "{xml}");
  assert!(
    xml.contains(r#"<p xml:id="p2.1">Running: Short</p>"#),
    "{xml}"
  );
}

/// amsart's mark again, by `\let\ver@amsthm.sty\undefined`, which `\@ifl@aded` takes for not loaded as it takes
/// `\relax` (latex.ltx `\ifx\csname ver@…\endcsname\relax`).
#[test]
fn amsart_reloads_amsthm_after_undefining_its_version() {
  let xml = clean!("loader/amsart_reloads_amsthm_after_undefining_its_version.tex");
  assert!(
    xml.contains(r#"<p xml:id="p1.1">amsthm loaded.</p>"#),
    "{xml}"
  );
  assert!(
    xml.contains(
      r#"<proof xml:id="id1"> <title class="ltx_runin" font="italic">Proof.</title> <para xml:id="p2"> <p xml:id="p2.1"><text xml:id="p2.1.1">A proof. ∎</text></p> </para> </proof>"#
    ),
    "{xml}"
  );
}

/// The loader takes back only a class binding's mark (user ruling 2026-10-14: no general loader `\ver@` rule): a
/// package the document loaded itself stays loaded through a later `\let\ver@<file>\relax` (in LaTeX it would load
/// again; floatrow_sty.rs is the same rule's other case), and its binding is not run twice.
#[test]
fn an_unmarked_document_package_stays_loaded() {
  let (log, xml) = convert_with(
    r"\documentclass{article}
\usepackage{amsthm}
\expandafter\let\csname ver@amsthm.sty\endcsname\relax
\makeatletter
\newcommand\amsthmstate{\@ifpackageloaded{amsthm}{loaded}{unloaded}}
\makeatother
\usepackage{amsthm}
\begin{document}
amsthm \amsthmstate.
\end{document}",
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let xml = squash(&xml);
  assert!(
    xml.contains(r#"<p xml:id="p1.1">amsthm loaded.</p>"#),
    "{xml}"
  );
}

/// kluwer.cls:2125 `\cite*` is `\citeyear`; :950-960 the abbreviations and nomenclature notes keep their bold
/// headings; natbib is borrowed, not loaded (kluwer.cls never loads it).
#[test]
fn kluwer_cite_star_and_title_notes() {
  let xml = clean!("class-bindings/kluwer_cite_star_and_title_notes.tex");
  assert!(
    xml.contains(
      r#"<note role="abbreviations" xml:id="id1"><text font="bold" xml:id="id1.1">Abbreviations: </text>KAP – Kluwer Academic Publishers</note>"#
    ),
    "{xml}"
  );
  assert!(
    xml.contains(
      r#"<note role="nomenclature" xml:id="id2"><text font="bold" xml:id="id2.1">Nomenclature:</text> <Math mode="inline" tex="x" text="x" xml:id="m1">"#
    ) && xml.contains(r#"</Math> – a variable</note>"#),
    "{xml}"
  );
  assert!(
    xml.contains(
      r#"<p xml:id="p1.1">Since <cite class="ltx_citemacro_citeyear"><bibref bibrefs="jones" separator=";" show="Year" yyseparator=","/></cite>; natbib not loaded.</p>"#
    ),
    "{xml}"
  );
}

/// elsarticle.cls:672 `\abstracttitle` names the abstract only (its `{abstract}` prints `\@elsarticleabstitle`, :676),
/// not `\abstractname`; :1022 `\myfooter[#1]#2␣`; `\@journal` empty until `\journal` (:1302).
#[test]
fn elsarticle_class_details() {
  let xml = clean!("class-bindings/elsarticle_class_details.tex");
  assert!(
    xml.contains(
      r#"<abstract inlist="toc" name="Summary" xml:id="abstract1"> <p xml:id="abstract1.1">The abstract.</p> </abstract>"#
    ),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<p xml:id="p1.1">Name: Abstract. Footer: Footer text. Journal: .</p>"#),
    "{xml}"
  );
}

/// A paper's copy of elsarticle.cls with aas_macros.sty pasted in (0812.4232, 1504.02326, 1807.02064, 2003.05953,
/// 2401.03908): its journal abbreviations, provided by aas_macros' binding.
#[test]
fn elsarticle_copy_aas_journal_macros() {
  let xml = clean!("class-bindings/elsarticle_copy_aas_journal_macros.tex");
  assert!(
    xml.contains(r#"<p xml:id="p1.1">In ApJ and MNRAS.</p>"#),
    "{xml}"
  );
}

/// elsart.cls's `\if@symbold\else\def\d{\,\mathrm{d}}\fi` (v2.18:1382; 0708.4004, 0712.3870): the binding's
/// differential `\d` (Perl elsart_support.sty.ltxml:48) where the paper's copy redefines it; the kernel's dot-under accent
/// where the copy does not (elsart5.cls: cond-mat0610502, 0706.1711), here a minimal stand-in.
#[test]
fn elsart_d_follows_the_shipped_copy() {
  let xml = clean!("class-bindings/elsart_d_follows_the_shipped_copy.tex");
  assert!(
    xml.contains(r#"<p xml:id="p1.1">Text ọ here, and"#),
    "{xml}"
  );
  assert!(xml.contains(r#"tex="\int f(x)\d{x}""#), "{xml}");
}
