//! Red/green guards for batch 68v: undefined-CS binding gaps — package internals our bindings lacked (`\symAMSb`
//! through `\try@load@fontshape`, empheq's `overload` and `\empheqlbrace`, …) and class macros of shipped classes.
//! Every repro's pdflatex reference is in its header.
use latexml::util::test::{convert_files_with, convert_with};

use super::perfect_kernel_batch46::{error_count, warning_count};

/// The XML with every whitespace run collapsed to one space.
fn squash(xml: &str) -> String { xml.split_whitespace().collect::<Vec<_>>().join(" ") }

/// A repro under `tools/perfect_kernel/repros/`, converted with the ar5iv preload: its (errors, warnings) and the
/// squashed XML.
macro_rules! repro {
  ($path:literal) => {{
    let (log, xml) = convert_with(
      include_str!(concat!("../../../tools/perfect_kernel/repros/", $path)),
      Some("ar5iv.sty"),
    );
    ((error_count(&log), warning_count(&log)), log, squash(&xml))
  }};
}

/// A repro beside the files it ships (`(name, path under repros/)`), converted with the ar5iv preload.
#[allow(unused_macros)]
macro_rules! repro_with {
  ($path:literal, $( ($name:literal, $file:literal) ),+ ) => {{
    let (log, xml) = convert_files_with(
      include_str!(concat!("../../../tools/perfect_kernel/repros/", $path)),
      &[ $( ($name, include_str!(concat!("../../../tools/perfect_kernel/repros/", $file))) ),+ ],
      Some("ar5iv.sty"),
    );
    ((error_count(&log), warning_count(&log)), log, squash(&xml))
  }};
}

fn assert_all(xml: &str, parts: &[&str]) {
  for part in parts {
    assert!(xml.contains(part), "{part}\n{xml}");
  }
}

/// fonts-nfss/amsfonts_try_load_fontshape: amsart's amsfonts test (amsart.cls:338-350, copied by nyjm.cls 2401.06615,
/// cedram.cls 2105.03188, myart.cls 1506.02470) finds umsa.fd through `\try@load@fontshape`, whose
/// `\DeclareFontShape` defines `\U/msa/m/n`; amsfonts loads, so `\symAMSb` is the AMSb symbol font.
#[test]
fn batch68v_try_load_fontshape_finds_amsfonts() {
  let (counts, log, xml) = repro!("fonts-nfss/amsfonts_try_load_fontshape.tex");
  assert_eq!(counts, (0, 0), "{log}");
  assert!(!log.contains("amsfonts NOT loaded"), "{log}");
  assert_all(&xml, &[
    r#"<XMTok name="varnothingx" role="ID">∅</XMTok>"#,
    r#"<Math mode="inline" tex="\mathbb{R}" text="R" xml:id="p1.m2">"#,
  ]);
}

/// package-bindings/empheq_overload_left: `\usepackage[overload]{empheq}` gives the AMS environments empheq's options
/// (empheq.sty:961-1066); `left` opens the first cell, `right` closes the last, `\empheqlbrace` is the brace grown to
/// the display (1410.0933, 1906.02657, 2210.09120, 2201.03303, 2207.01386). The option list is no longer typeset.
/// The 7 warnings are the math parser's on the empty fence `\left\{\right.` (math parsing is a separate stream).
#[test]
fn batch68v_empheq_overload_left_and_right() {
  let (counts, log, xml) = repro!("package-bindings/empheq_overload_left.tex");
  assert_eq!(counts, (0, 7), "{log}");
  assert!(!xml.contains("left="), "{xml}");
  assert_all(&xml, &[
    r#"tex="\displaystyle F(x)=\left\{\right.x""#,
    r#"tex="\displaystyle y=2""#,
    r#"tex="\displaystyle=3\left.\right\}""#,
    r#"tex="\displaystyle G=\left\{\right.a""#,
    r#"tex="\displaystyle b=2""#,
    r#"tex="\left\{\right.c=d""#,
    r#"<tags> <tag>(4)</tag> <tag role="refnum">4</tag> </tags>"#,
  ]);
}

/// package-bindings/lastpage_class_putlabel: lastpage's `\lastpage@putlabel` is `\relax` in the document
/// (lastpagemodern.sty:84-89), as the classes' `\AtEndDocument{\clearpage\lastpage@putlabel}` expect (cta-author.cls
/// 2209.11927, 1710.08633; IET-Conf-Paper.cls 2609.02892).
#[test]
fn batch68v_lastpage_putlabel_is_relax() {
  let (counts, log, xml) = repro!("package-bindings/lastpage_class_putlabel.tex");
  assert_eq!(counts, (0, 0), "{log}");
  assert_all(&xml, &[
    r#"<p xml:id="p1.1">Page 1 of <ref labelref="LABEL:LastPage"/>.</p>"#,
  ]);
}

/// package-bindings/caption_atbegindocument: caption3.sty:178-191 `\caption@AtBeginDocument[*]` queues code for
/// `\begin{document}` (the starred after the rest) and runs it at once inside the document (oasics-v2021.cls 2608.11802,
/// 2607.26704; no-lipics-v2022.cls 2403.18812).
#[test]
fn batch68v_caption_at_begin_document_hooks() {
  let (counts, log, xml) = repro!("package-bindings/caption_atbegindocument.tex");
  assert_eq!(counts, (0, 0), "{log}");
  assert_all(&xml, &[
    r#"<p xml:id="p1.1">Hooks: checked first, starred after; inside the document: <text font="bold" xml:id="p1.1.1">at once</text>.</p>"#,
  ]);
}

/// package-bindings/textpos_block_colour: color.sty:155-156 `\color@block`, the colour block textpos lays under a
/// coloured `{textblock}` (textpos.sty:328-331; 2305.15117, 2206.12186, 2407.11064).
#[test]
fn batch68v_color_block_under_a_textblock() {
  let (counts, log, xml) = repro!("package-bindings/textpos_block_colour.tex");
  assert_eq!(counts, (0, 0), "{log}");
  assert_all(&xml, &[
    r#"<p xml:id="p1.1">Before.<inline-block vattach="bottom" xml:id="p1.1.1"> <p xml:id="p1.1.1.1">Sidebar text.</p> </inline-block> After.</p>"#,
  ]);
}

/// class-bindings/memo_l/memo_l_amsbook_internals: amsbook's binding has amsbook.cls's internals (:113-114
/// `\@dateposted`, :616 `\abstractbox`, :638-649 `\@setabstract`), which memo-l.cls's `\tableofcontents` and abstract
/// notes read (2412.15928, 2511.16188, 1512.07128).
#[test]
fn batch68v_amsbook_internals_for_memo_l() {
  let (counts, log, xml) = repro_with!(
    "class-bindings/memo_l/memo_l_amsbook_internals.tex",
    ("memomini.cls", "class-bindings/memo_l/memomini.cls")
  );
  assert_eq!(counts, (0, 0), "{log}");
  assert_all(&xml, &[
    r#"<abstract inlist="toc" name="Abstract." xml:id="abstract1"> <p xml:id="abstract1.1">The abstract.</p> </abstract>"#,
    r#"<title><tag close=" ">Chapter 1</tag>One</title>"#,
  ]);
  assert!(!xml.contains("ERROR"), "{xml}");
}

/// class-bindings/bmcart/bmcart_environment_body_commands: the commands a class's own `\figure` and `\abstract`
/// define for their bodies (bmcart.cls:712-717 `\csentence`, :1505-1510 `\parttitle`), which the kernel's
/// environments never run, are made where they are called (frontmatter_stores.rs `refused_environment_definition`;
/// 2305.08284, 1611.01327, 1909.07026, 2007.07886, 1907.07303, 2106.01700).
#[test]
fn batch68v_bmcart_environment_body_commands() {
  let (counts, log, xml) = repro_with!(
    "class-bindings/bmcart/bmcart_environment_body_commands.tex",
    ("bmcmini.cls", "class-bindings/bmcart/bmcmini.cls")
  );
  assert_eq!(counts, (0, 0), "{log}");
  assert_all(&xml, &[
    r#"<abstract inlist="toc" name="Abstract" xml:id="abstract1"> <p xml:id="abstract1.1"><text font="bold" xml:id="abstract1.1.1">Background:</text> Some background.</p> <p xml:id="abstract1.2"><text font="bold" xml:id="abstract1.2.1">Methods:</text> Some methods.</p> </abstract>"#,
    r#"<caption><tag close=": ">Figure 1</tag><text font="bold" xml:id="S0.F1.2">Lead sentence.</text> The rest of the caption.</caption>"#,
  ]);
}

/// class-bindings/mastersdoctoralthesis/mdt_authorname: a refused class setter's replay keeps what it defines locally,
/// as the setter would at the call (sect05.rs `replay_class_setter`, `\lx@replay@capture`): MastersDoctoralThesis.cls:218
/// `\newcommand{\authorname}{#1}` (2603.21941, 2312.06150, 2011.02318).
#[test]
fn batch68v_replayed_author_keeps_authorname() {
  let (counts, log, xml) = repro_with!(
    "class-bindings/mastersdoctoralthesis/mdt_authorname.tex",
    (
      "mdtmini.cls",
      "class-bindings/mastersdoctoralthesis/mdtmini.cls"
    )
  );
  assert_eq!(counts, (0, 0), "{log}");
  assert_all(&xml, &[
    r#"<creator role="author"> <personname>Ludovic <text font="smallcaps" xml:id="id1">Varrin</text></personname> </creator>"#,
    r#"<p xml:id="p1.1">I, Ludovic <text font="smallcaps" xml:id="p1.1.1">Varrin</text>, declare that this thesis is my own.</p>"#,
  ]);
}

/// class-bindings/revtex4_copy/rtcopy_collaboration_switch: the replayed revtex4 `\author`'s `\@booleanfalse
/// \collaboration@sw` (`\let`) stays for the class's `\affiliation` (qm2008_abs.cls 0805.2253, snp.cls 0806.0481,
/// jasatex.cls 0805.4759).
#[test]
fn batch68v_replayed_author_keeps_collaboration_switch() {
  let (counts, log, xml) = repro_with!(
    "class-bindings/revtex4_copy/rtcopy_collaboration_switch.tex",
    ("rtcopy.cls", "class-bindings/revtex4_copy/rtcopy.cls")
  );
  assert_eq!(counts, (0, 0), "{log}");
  assert_all(&xml, &[
    r#"<creator role="author"> <personname>Takeshi Osada</personname> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#,
  ]);
}

/// class-bindings/vmsta/vmsta_name_parts: a command the class's own (refused) `\author` defines for the author's text
/// is made while a frontmatter field is digested (`lx_frontmatter_item`, frontmatter_stores.rs
/// `refused_environment_definition`): vmsta.cls:278-283 `\fnm` (1508.02842, 1411.4499, 1601.01109).
#[test]
fn batch68v_vmsta_author_name_parts() {
  let (counts, log, xml) = repro_with!(
    "class-bindings/vmsta/vmsta_name_parts.tex",
    ("vmstamini.cls", "class-bindings/vmsta/vmstamini.cls")
  );
  assert_eq!(counts, (0, 0), "{log}");
  assert_all(&xml, &[
    r#"<creator role="author"> <personname>Yu.YuliyaMishura</personname> </creator> <creator before=" " role="author"> <personname>I.Ivan Voronov</personname> </creator>"#,
  ]);
}
