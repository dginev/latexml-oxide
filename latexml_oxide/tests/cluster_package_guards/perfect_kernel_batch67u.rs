//! Red/green guards for batch 67u: a binding loads the packages the file it stands in for loads (run 353's
//! undefined-only papers whose missing commands come from a package their class, or a package it loads, requires).
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

/// A binding whose base is scrartcl, as its class loads it (ceurart, atlasdoc: OXIDIZED_DESIGN_DIVERGENCES #736): the
/// raw KOMA load is quiet, as `\documentclass{scrartcl}` is under pdflatex (OXIDIZED_DESIGN_DIVERGENCES #737).
macro_rules! clean_koma {
  ($path:literal) => {{
    let (log, xml) = repro!($path);
    assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
    assert!(
      !["scrartcl", "scrreprt"]
        .iter()
        .any(|class| super::perfect_kernel_batch72a::koma_raw_warnings(&log, class)),
      "{log}"
    );
    squash(&xml)
  }};
}

/// lscape.sty:35 and psfrag.sty:35 `\RequirePackage{graphics}` (0710.2354, 0802.0639, 1007.2821; 0801.1195,
/// 0802.1229, 0704.2560).
#[test]
fn lscape_and_psfrag_load_graphics() {
  let xml = clean!("stub-dependencies/lscape_psfrag_load_graphics.tex");
  assert!(
    xml.contains(
      r#"<inline-block depth="0.0pt" height="3.5pt" width="13.1pt" xscale="0.5" xtranslate="-6.5pt" yscale="0.5" ytranslate="1.7pt" xml:id="p1.1.1"> <p xml:id="p1.1.1.1">scaled</p> </inline-block>"#
    ),
    "{xml}"
  );
  assert!(xml.contains(r#"<p xml:id="p1.1.2.1">resized</p>"#), "{xml}");
}

/// showkeys.sty:51-56: the `color` option loads color (0712.3213, 1010.3777, 1103.2047).
#[test]
fn showkeys_color_option_loads_color() {
  let xml = clean!("stub-dependencies/showkeys_color_option_loads_color.tex");
  assert!(
    xml.contains(
      r##"<p xml:id="p1.1">Some <text color="#E60000" xml:id="p1.1.1"> red</text> text.</p>"##
    ),
    "{xml}"
  );
}

/// mathtools.sty:37 `\RequirePackage{mhsetup}`; `_` is a subscript again after `\MHInternalSyntaxOff` (1202.0045,
/// 1611.02845, 1903.08038).
#[test]
fn mathtools_loads_mhsetup() {
  let xml = clean!("stub-dependencies/mathtools_loads_mhsetup.tex");
  assert!(
    xml.contains(r#"<Math mode="inline" tex="x_{1}=1" text="x _ 1 = 1" xml:id="p1.m1">"#),
    "{xml}"
  );
}

/// newtxmath.sty:36 `\RequirePackage{centernot}`; `\centernot` negates the relation as `\not` does (2002.09451,
/// 2103.01489, 2104.13315).
#[test]
fn newtxmath_loads_centernot() {
  let xml = clean!("stub-dependencies/newtxmath_loads_centernot.tex");
  assert!(
    xml.contains(
      r#"<XMApp xml:id="p1.m1.1.1"> <XMTok meaning="not-implies" name="not-⟹" role="ARROW">⟹̸</XMTok> <XMTok font="italic" role="UNKNOWN">a</XMTok> <XMTok font="italic" role="UNKNOWN">b</XMTok> </XMApp>"#
    ),
    "{xml}"
  );
}

/// thm-patch.sty:164-176 (loaded by thmtools.sty:47): `\addtotheorempostheadhook[thm]{…}` is defined, and its code
/// runs at the theorem's begin, inside it (1710.06939, 1712.09887, 1801.10129).
#[test]
fn thmtools_theorem_hooks_run_in_the_theorem() {
  let xml = clean!("stub-dependencies/thmtools_theorem_hooks_run.tex");
  assert!(
    xml.contains(r#"<theorem class="ltx_theorem_thm" inlist="thm theorem:thm" labels="LABEL:t" xml:id="S1.Thmthm1">"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<item labels="LABEL:t2" xml:id="S1.I1.i2">"#),
    "{xml}"
  );
  let (log, xml) = convert_with(
    "\\documentclass{article}\n\\usepackage{amsthm,thmtools}\n\\newtheorem{thm}{Theorem}\n\
     \\addtotheorempostheadhook[thm]{HEAD}\n\\addtotheoremprefoothook[thm]{FOOT}\n\\begin{document}\n\
     \\begin{thm}Body.\\end{thm}\nAfter.\n\\end{document}\n",
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let xml = squash(&xml);
  let theorem = &xml[xml.find("<theorem ").expect("theorem")..xml.find("</theorem>").expect("end")];
  assert!(
    theorem.contains("HEAD") && theorem.contains("FOOT") && theorem.contains("Body."),
    "{xml}"
  );
  assert_eq!(xml.matches("HEAD").count(), 1, "{xml}");
}

/// amsopn.sty:92-97 `\varlim@`, which papers call for their own limits; renders as `\varprojlim`/`\varinjlim`
/// (1608.08273, 1807.11932, 1909.11399).
#[test]
fn amsopn_varlim_builder_renders_the_limit() {
  let xml = clean!("stub-dependencies/amsopn_varlim_builder.tex");
  assert!(
    xml.contains(
      r#"<XMApp role="BIGOP" scriptpos="post1" xml:id="p1.m1.1.1.1.2"> <XMTok name="underleftarrow" role="UNDERACCENT" stretchy="true">←</XMTok> <XMTok role="OPFUNCTION" scriptpos="post">lim</XMTok> </XMApp>"#
    ),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<XMTok name="underrightarrow" role="UNDERACCENT" stretchy="true">→</XMTok>"#),
    "{xml}"
  );
}

/// sn-jnl.cls:311 `\usepackage{program}` in the older copies: program's `\gt`, `\union` (2010.12415, 2102.03176,
/// 2104.02781), with the paper's adjustbox and `\index` intact.
#[test]
fn snjnl_class_copy_loads_program() {
  let xml = clean!("class-bindings/snjnl_class_copy_loads_program.tex");
  assert!(
    xml.contains(r#"<XMTok meaning="greater-than" role="RELOP">&gt;</XMTok>"#)
      && xml.contains(r#"<XMTok meaning="union" name="cup" role="ADDOP">∪</XMTok>"#),
    "{xml}"
  );
  // program.sty:175-176's programbox `\(` stays the kernel's inline maths under the class (OXIDIZED_DESIGN #597).
  assert!(
    xml.contains(r#"<Math mode="inline" tex="a\semicolon\;b""#),
    "{xml}"
  );
  assert!(!xml.contains("ltx_minipage"), "{xml}");
  // adjustbox reads pgf with `|` other (2202.10115); program.sty's `\index` does not replace the binding's (2306.09847).
  assert_eq!(xml.matches("<indexmark>").count(), 1, "{xml}");
  assert!(
    xml.contains(r#"<Math mode="inline" tex="z" text="z""#),
    "{xml}"
  );
}

/// bmvc2k.cls:121 `\RequirePackage{graphicx,subfigure}` (1507.05726, 1510.01553, 1605.07146); a copy that loads
/// subcaption instead (2110.13859) gets subcaption, whose `{subfigure}` environment then works.
#[test]
fn bmvc2k_class_copy_loads_its_subfigure_package() {
  let xml = clean!("class-bindings/bmvc2k_class_copy_loads_subfigure.tex");
  assert_eq!(
    xml.matches(r#"<figure class="ltx_figure_panel" "#).count(),
    2,
    "{xml}"
  );
  assert!(
    xml.contains(
      r#"<caption><tag close=" "><text fontsize="80%" xml:id="S0.F1.sf2.3">(b)</text></tag><text fontsize="80%" xml:id="S0.F1.sf2.4">Right</text></caption>"#
    ),
    "{xml}"
  );
  let (log, xml) = convert_with(
    "\\begin{filecontents*}[overwrite]{bmvc2k.cls}\n\\ProvidesClass{bmvc2k}\n\\LoadClass{article}\n\
     \\RequirePackage{graphicx,subcaption}\n\\end{filecontents*}\n\\documentclass{bmvc2k}\n\\begin{document}\n\
     \\begin{figure}\n\\begin{subfigure}{0.4\\linewidth}\\rule{1cm}{1cm}\\caption{Left}\\end{subfigure}\n\
     \\caption{Both}\n\\end{figure}\n\\end{document}\n",
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    squash(&xml).contains(r#"<figure inlist="lof" xml:id="S0.F1.sf1">"#),
    "{xml}"
  );
}

/// ieeeaccess.cls:121-131 color, then spotcolor (1901.05553, 1907.00310, 1712.00216).
#[test]
fn ieeeaccess_loads_color_and_spotcolor() {
  let xml = clean!("class-bindings/ieeeaccess_loads_color_and_spotcolor.tex");
  assert!(
    xml.contains(r##"<p xml:id="p1.1">Received today, <text color="#FF0000" xml:id="p1.1.1">revised</text>.</p>"##),
    "{xml}"
  );
}

/// agujournal2019.cls:91-112 xcolor, url, lineno, indentfirst, ragged2e (1803.00457, 1902.01137, 1812.02692).
#[test]
fn agujournal2019_loads_lineno_ragged2e_xcolor() {
  let xml = clean!("class-bindings/agujournal2019_loads_lineno_ragged2e_xcolor.tex");
  assert!(
    xml.contains(r#"<Math mode="display" tex="x=1" text="x = 1" xml:id="S0.E1.m1">"#),
    "{xml}"
  );
  assert!(
    xml.contains(r##"<p xml:id="p1.2"><text color="#0000FF" xml:id="p1.2.1">Blue.</text></p>"##),
    "{xml}"
  );
}

/// WileyMSP-template.cls:10 `\RequirePackage{gensymb}` (2003.01904, 2007.06369, 2103.00309).
#[test]
fn wileymsp_loads_gensymb() {
  let xml = clean!("class-bindings/wileymsp_loads_gensymb.tex");
  assert!(
    xml.contains(r#"<p xml:id="p1.1">At 25 ℃ and 90°.</p>"#),
    "{xml}"
  );
}

/// oup-authoring-template.cls `\RequirePackage{multirow}`, TL 2025's copy :2426, 2102.02669's :2391 (2102.02669,
/// 2104.08961, 2206.08262).
#[test]
fn oup_authoring_template_loads_multirow() {
  let xml = clean!("class-bindings/oup_loads_multirow.tex");
  assert!(
    xml.contains(
      r#"<td align="left" rowspan="2" xml:id="p1.1.1.1"><text xml:id="p1.1.1.1.1">a</text></td>"#
    ),
    "{xml}"
  );
}

/// cas-common.sty:1599-1610 `\MSC` (1909.01564, 2012.11920, 2407.03849), :415-423 `\orcidauthor` (2104.12510,
/// 2304.13458, 2407.18715).
#[test]
fn cas_sc_classification_lines_and_orcid() {
  let xml = clean!("class-bindings/cas_sc_classification_lines.tex");
  assert!(
    xml.contains(
      r#"<classification scheme="keywords"> graphs ,trees <text font="italic" xml:id="id1">2010 MSC: </text>05C75 </classification>"#
    ),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<creator role="author"> <personname>Jane Doe</personname> <contact role="orcid"><ref class="ltx_orcid" href="https://orcid.org/0000-0002-1825-0097""#),
    "{xml}"
  );
}

/// ws-rv9x6.cls:1527 `\usepackage{makeidx}` (0706.1148, 0810.2192, 0901.1361).
#[test]
fn ws_rv9x6_class_copy_loads_makeidx() {
  let xml = clean!("class-bindings/ws_rv9x6_class_copy_loads_makeidx.tex");
  assert!(
    xml.contains(r#"<index xml:id="idx"> <title>Index</title> </index>"#),
    "{xml}"
  );
}

/// revtex4-2.cls:6207-6214, revtex4-1.cls:6186-6193: the `linenumbers` option loads lineno (2003.07846, 2010.12920,
/// 1010.3732); without the option the class loads none, and `\nolinenumbers` errs as in pdflatex.
#[test]
fn revtex_linenumbers_option_loads_lineno() {
  let xml = clean!("class-bindings/revtex_linenumbers_option_loads_lineno.tex");
  assert!(xml.contains(r#"<p xml:id="p1.1">Text. More.</p>"#), "{xml}");
  let body = "\\begin{document}\nText.\n\\nolinenumbers\nMore.\n\\end{document}\n";
  let (log, _) = convert_with(
    &format!("\\documentclass[aps,linenumbers]{{revtex4-1}}\n{body}"),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let (log, _) = convert_with(
    &format!("\\documentclass[aps]{{revtex4-2}}\n{body}"),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (1, 0), "{log}");
  assert!(log.contains("Error:undefined:\\nolinenumbers"), "{log}");
}

/// quantumarticle.cls:155-168 the AMS options, :282-284 geometry, :1508-1512 caption (1802.10344, 1612.06828,
/// 1910.03969).
#[test]
fn quantumarticle_options_load_ams_geometry_caption() {
  let xml = clean!("class-bindings/quantumarticle_options_load_ams_geometry_caption.tex");
  assert!(
    xml.contains(
      r#"<XMTok meaning="less-than-or-similar-to" name="lesssim" role="RELOP">≲</XMTok>"#
    ),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<caption><tag close=": ">Figure 1</tag>Boxed</caption>"#),
    "{xml}"
  );
}

/// aa.cls:249 `\RequirePackage{answers}` in the v8-era copies, which loads verbatim's `{comment}` (1501.03426,
/// 1504.01878, 1507.00595); a paper without a copy (today's aa.cls loads none) gets none.
#[test]
fn aa_class_copy_loads_answers() {
  let xml = clean!("class-bindings/aa_class_copy_loads_answers.tex");
  assert!(
    xml.contains(r#"<p xml:id="p1.1">Shown.</p>"#) && !xml.contains("Hidden"),
    "{xml}"
  );
  let (log, _) = convert_with(
    "\\documentclass{aa}\n\\begin{document}\nShown.\n\\begin{comment}\nHidden.\n\\end{comment}\n\\end{document}\n",
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (1, 0), "{log}");
  assert!(log.contains("Error:undefined:{comment}"), "{log}");
}

/// atlasdoc.cls:52-54, 542-545: the default `newtx` fonts load newtxmath (2207.12246, 2210.05415, 2304.03053).
#[test]
fn atlasdoc_default_fonts_load_newtxmath() {
  let xml = clean_koma!("class-bindings/atlasdoc_default_fonts_load_newtxmath.tex");
  assert!(
    xml.contains(r#"<XMTok name="upmu" role="UNKNOWN">μ</XMTok>"#),
    "{xml}"
  );
  assert!(xml.contains(r#"<XMTok role="UNKNOWN">✓</XMTok>"#), "{xml}");
}

/// ceurart.cls:88 colortbl (2212.08130, 2212.10039, 2303.14461).
#[test]
fn ceurart_loads_colortbl() {
  let xml = clean_koma!("class-bindings/ceurart_loads_colortbl.tex");
  assert!(
    xml.contains(
      r##"<td align="left" backgroundcolor="#FFFF00" xml:id="p1.1.1.1"><text backgroundcolor="#FFFF00" xml:id="p1.1.1.1.1">a</text></td>"##
    ),
    "{xml}"
  );
}

/// ejpecp.cls:88 mathtools, :94-96 dsfont (1607.07914, 1611.05220, 1904.07135).
#[test]
fn ejpecp_loads_mathtools_and_dsfont() {
  let xml = clean!("class-bindings/ejpecp_loads_mathtools_dsfont.tex");
  assert!(
    xml.contains(r#"<XMTok name="coloneqq" role="RELOP">≔</XMTok>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<XMTok font="blackboard" meaning="1" role="NUMBER">1</XMTok>"#),
    "{xml}"
  );
}

/// WileyNJD-v2.cls:63, 2722-2723: the `APA` option loads NJDapacite, apacite under another name (1811.00964,
/// 1902.09230, 1904.10792).
#[test]
fn wileynjd_apa_option_loads_apacite() {
  let xml = clean!("class-bindings/wileynjd_apa_option_loads_apacite.tex");
  assert!(
    xml.contains(
      r#"<bibitem key="doe" xml:id="bib.bib1"> <tags> <tag role="number">1</tag> <tag role="year">2020</tag> <tag role="authors">Doe</tag> <tag role="fullauthors">Doe</tag> <tag role="refnum">Doe (2020)</tag> <tag role="key">doe</tag> </tags> <bibblock> Doe, J. </bibblock> <bibblock>(2020). </bibblock> <bibblock>A title. </bibblock> </bibitem>"#
    ),
    "{xml}"
  );
}

/// jmlr.cls:153-154 xcolor (1902.03035, 2010.02010, 2107.03212).
#[test]
fn jmlr_loads_xcolor() {
  let xml = clean!("class-bindings/jmlr_loads_xcolor.tex");
  assert!(
    xml.contains(
      r##"<p xml:id="p1.1">A <text color="#FF0000" xml:id="p1.1.1">red</text> word.</p>"##
    ),
    "{xml}"
  );
}

/// sigma.cls:24 amscd (0902.2977, 0907.2394, 0910.5929).
#[test]
fn sigma_loads_amscd() {
  let xml = clean!("class-bindings/sigma_loads_amscd.tex");
  assert!(
    xml.contains(r#"<XMTok meaning="commutative-diagram"/>"#),
    "{xml}"
  );
}

// Round 2 (review of 67u).

/// sn-jnl.cls:1571-1575 (1907.11468's copy, all 138 copies of the run-353 sn-jnl papers): the class's begin-document
/// code makes `\opensquare` (msa "03) the `\qedsymbol`, over program.sty:971-973's `\blacksquare`, so the proof mark is □
/// (1907.11468, 2210.16458); a copy without those lines keeps program's ■.
#[test]
fn snjnl_class_open_square_is_the_proof_mark() {
  let xml = clean!("class-bindings/snjnl_class_open_square_qed.tex");
  assert!(
    xml.contains(
      r#"<p xml:id="p1.1"><text xml:id="p1.1.1">Trivial. <text xml:id="p1.1.1.1"/> <Math mode="inline" tex="\opensquare" text="opensquare" xml:id="p1.m1"> <XMath xml:id="p1.m1.1"> <XMTok name="opensquare" role="ID">□</XMTok> </XMath> </Math></text></p>"#
    ),
    "{xml}"
  );
  assert!(!xml.contains("blacksquare"), "{xml}");
  let source = include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/snjnl_class_open_square_qed.tex"
  )
  .lines()
  .filter(|line| {
    !line.starts_with("\\DeclareSymbolFont")
      && !line.starts_with("\\DeclareMathSymbol")
      && !line.starts_with("\\def\\qedsymbol")
  })
  .collect::<Vec<_>>()
  .join("\n");
  let (log, xml) = convert_with(&source, Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let xml = squash(&xml);
  assert!(
    xml.contains(
      r#"<Math mode="inline" tex="\blacksquare" text="blacksquare" xml:id="p1.m1"> <XMath xml:id="p1.m1.1"> <XMTok name="blacksquare" role="UNKNOWN">■</XMTok> </XMath> </Math>"#
    ),
    "{xml}"
  );
}

/// thmtools.sty:47 loads thm-patch, so the binding marks it loaded and a later `\usepackage{thm-patch}` reads nothing
/// over it; thm-patch.sty:178-217 wraps amsthm's `proof` to run the `[proof]` hooks — the head hooks after the head, the
/// pre-foot hook before the QED mark, the post-foot hook after the proof — and not the generic ones (synthetic; the hook
/// commands of 1710.06939, 1712.09887, 1801.10129).
#[test]
fn thmtools_then_thm_patch_runs_the_proof_hooks() {
  let xml = clean!("stub-dependencies/thmtools_then_thm_patch_proof_hooks.tex");
  assert!(
    xml.contains(r#"<p xml:id="Thmthm1.p1.1"><text font="italic" xml:id="Thmthm1.p1.1.1">HEADGENERICBody.</text></p>"#),
    "{xml}"
  );
  assert!(
    xml.contains(
      r#"<proof xml:id="id1"> <title class="ltx_runin" font="italic">Proof.</title> <para xml:id="p1"> <p xml:id="p1.1"><text xml:id="p1.1.1">PHEADTrivial.PFOOT∎</text></p> </para> </proof> <para xml:id="p2"> <p xml:id="p2.1">PAFTER After.</p> </para>"#
    ),
    "{xml}"
  );
  // amsthm loaded after thmtools: there was no proof for thm-patch to wrap (`\ifx\proof\endproof`), so a `[proof]`
  // hook is gathered and never run (pdflatex: "Proof. Trivial.").
  let (log, xml) = convert_with(
    "\\documentclass{article}\n\\usepackage{thmtools}\n\\usepackage{amsthm}\n\\addtotheorempostheadhook[proof]{PHEAD}\n\
     \\begin{document}\n\\begin{proof}Trivial.\\end{proof}\n\\end{document}\n",
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    squash(&xml).contains(r#"<p xml:id="p1.1"><text xml:id="p1.1.1">Trivial.∎</text></p>"#),
    "{xml}"
  );
}

/// quantumarticle.cls:1508-1512 loads caption `\AtEndPreamble` (etoolbox, :27), so a document's own `\AtEndPreamble`
/// code, queued after the class's, finds it (1910.03969, 1711.07279, 1812.02144); with the document's own etoolbox too.
#[test]
fn quantumarticle_loads_caption_at_the_end_of_the_preamble() {
  let expected = r#"<p xml:id="p1.1">At the end of the preamble: caption loaded.</p>"#;
  let xml = clean!("class-bindings/quantumarticle_caption_at_end_preamble.tex");
  assert!(xml.contains(expected), "{xml}");
  let source = include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/quantumarticle_caption_at_end_preamble.tex"
  )
  .replace("\\pdfoutput=1\n", "\\pdfoutput=1\n\\usepackage{etoolbox}\n");
  let (log, xml) = convert_with(&source, Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(squash(&xml).contains(expected), "{xml}");
}
