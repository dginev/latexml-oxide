//! Red/green guards for batch 68h: caption's keys and `\DeclareCaptionSubType`, an ORCID given as its address,
//! cleveref's `\crefalias`, graphics' driver name, `\ProcessOptions`'s `\protected@edef` read, a `\protect`ed name noun.
use latexml::util::test::{convert_files_with, convert_with};

use super::perfect_kernel_batch46::{error_count, warning_count};

/// The XML with every whitespace run collapsed to one space.
fn squash(xml: &str) -> String { xml.split_whitespace().collect::<Vec<_>>().join(" ") }

/// A clean conversion of a repro, squashed.
macro_rules! clean {
  ($path:literal) => {{
    let (log, xml) = convert_with(
      include_str!(concat!("../../../tools/perfect_kernel/repros/", $path)),
      Some("ar5iv.sty"),
    );
    assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
    squash(&xml)
  }};
}

/// caption3.sty:718-858, :1347-1350, :1661-1715: the label format, separator, name, text format and fonts are the
/// caption's, as pdflatex sets them ("Figure 1. A figure", "TABLE 1: A table", "Fig. (2). Parens", "Figure 3").
#[test]
fn caption_keys_set_the_label_and_separator() {
  let xml = clean!("captions-floats/caption_keys_set_the_label_and_separator.tex");
  for caption in [
    r#"<caption><tag close=". "><text font="bold" xml:id="S0.F1.1">Figure 1</text></tag><text font="italic" xml:id="S0.F1.2">A figure</text></caption>"#,
    r#"<caption><tag close=": "><text font="bold" xml:id="S0.T1.1">TABLE 1</text></tag><text font="italic" xml:id="S0.T1.2">A table</text></caption>"#,
    r#"<caption><tag close=". "><text font="bold" xml:id="S0.F2.1">Fig. (2)</text></tag><text font="italic" xml:id="S0.F2.2">Parens</text></caption>"#,
    r#"<caption><tag><text font="bold" xml:id="S0.F3.1">Figure 3</text></tag></caption>"#,
    r#"<caption><tag close=" – "><text font="bold" xml:id="S0.F4.1">Fig. 4</text></tag><text font="italic" xml:id="S0.F4.2">Dash.</text></caption>"#,
    r#"<caption><tag><text font="bold" xml:id="S0.F5.1">Figure 5</text></tag><break/>Newline</caption>"#,
  ] {
    assert!(xml.contains(caption), "{caption}\n{xml}");
  }
}

/// caption3.sty:1774-1806: `\DeclareCaptionSubType*[arabic]{figure}` numbers the sub-figures 1.1, 1.2, and a starred
/// sub-type of algorithm gets its counter (1202.5818, 1604.05412, 2002.03129, 2106.01798, 1706.00437).
#[test]
fn declarecaptionsubtype_numbers_the_subtype() {
  let xml = clean!("captions-floats/declarecaptionsubtype_numbers_the_subtype.tex");
  assert!(
    xml.contains(
      r#"<caption><tag close=" "><text fontsize="90%" xml:id="S0.F1.sf1.3">1.1</text></tag><text fontsize="90%" xml:id="S0.F1.sf1.4">First</text></caption>"#
    ) && xml.contains(
      r#"<caption><tag close=" "><text fontsize="90%" xml:id="S0.F1.sf2.3">1.2</text></tag><text fontsize="90%" xml:id="S0.F1.sf2.4">Second</text></caption>"#
    ),
    "{xml}"
  );
  assert!(xml.contains(r#"<tag role="refnum">1.2</tag>"#), "{xml}");
  assert!(
    xml.contains(
      r#"<p xml:id="p1.1">See <ref labelref="LABEL:a"/>, <ref labelref="LABEL:b"/>, <ref labelref="LABEL:m"/> and 2.a.</p>"#
    ),
    "{xml}"
  );
}

/// An ORCID iD written as its orcid.org address is the iD (2104.12510, 2304.13458, 2407.18715).
#[test]
fn orcid_given_as_url_is_the_id() {
  let xml = clean!("sectioning-frontmatter/orcid_given_as_url_is_the_id.tex");
  for id in [
    "0000-0002-1825-0097",
    "0000-0001-5109-3700",
    "0000-0003-1415-9269",
  ] {
    let link =
      format!(r#"<ref class="ltx_orcid" href="https://orcid.org/{id}" title="ORCID {id}">"#);
    assert!(xml.contains(&link), "{link}\n{xml}");
  }
  assert!(!xml.contains("orcid.org/http"), "{xml}");
}

/// cleveref.sty:1277-1278 `\crefalias{<counter>}{<type>}`: "section 1, supplement A, step 1, item 1" (1810.01223,
/// 1706.03576, 2210.04778).
#[test]
fn cleveref_crefalias_names_the_type() {
  let xml = clean!("package-bindings/cleveref_crefalias_names_the_type.tex");
  let types: Vec<&str> = xml
    .split(r#"<tag role="creftype">"#)
    .skip(1)
    .filter_map(|rest| rest.split("</tag>").next())
    .collect();
  assert_eq!(types, ["section", "step", "item", "supplement"], "{xml}");
}

/// graphics.sty:66-107: `\Gin@driver` is pdftex.def for PDF output, so trimclip reads tc-pdftex.def and no pgf
/// (2202.10115, 2306.09847, 2507.09618).
#[test]
fn adjustbox_clipping_driver_is_pdftex() {
  let xml = clean!("package-bindings/adjustbox_clipping_driver_is_pdftex.tex");
  assert!(xml.contains("no pgf; driver pdftex.def.</p>"), "{xml}");
  assert!(
    xml.contains(
      r#"<p xml:id="p1.2"><text xml:id="p1.2.1"><text width="0.0pt" yoffset="-1.0pt" xml:id="p1.2.1.1">Box</text> </text></p>"#
    ),
    "{xml}"
  );
  assert!(!xml.contains("<picture"), "{xml}");
}

/// A document converted clean (0 errors, 0 warnings), squashed.
fn clean_doc(tex: &str) -> String {
  let (log, xml) = convert_with(tex, Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  squash(&xml)
}

/// bmvc2k.cls:277-278 defines the colours papers name in caption's options (2210.10486).
#[test]
fn bmvc2k_caption_colour_is_defined() {
  let xml = clean_doc(
    "\\documentclass{bmvc2k}\n\\usepackage[labelfont={color=bmv@captioncolor,bf}]{caption}\n\\begin{document}\n\
     \\begin{figure}\\caption{Colored}\\end{figure}\n\\end{document}\n",
  );
  assert!(
    xml.contains(
      r##"<caption><tag close=": "><text color="#000066" font="bold" xml:id="S0.F1.2">Figure 1</text></tag>Colored</caption>"##
    ),
    "{xml}"
  );
}

/// cas-sc.cls:49, :67: `[review]` loads setspace, whose `\setstretch` a declared caption font calls (2512.10464).
#[test]
fn cas_review_option_loads_setspace() {
  let xml = clean_doc(
    "\\documentclass[review]{cas-sc}\n\\usepackage{caption}\n\\DeclareCaptionFont{mysingle}{\\setstretch{1}}\n\
     \\captionsetup[figure]{font={small,mysingle}}\n\\begin{document}\n\\begin{figure}\\caption{Single}\\end{figure}\n\
     \\end{document}\n",
  );
  assert!(
    xml.contains(
      r#"<caption><tag close=": "><text fontsize="90%" xml:id="S0.F1.2">Figure 1</text></tag><text fontsize="90%" xml:id="S0.F1.3">Single</text></caption>"#
    ),
    "{xml}"
  );
}

/// caption.sty:1167: longtable's caption is formatted by caption too (pdflatex "Table 1. Long").
#[test]
fn longtable_caption_takes_the_caption_separator() {
  let xml = clean_doc(
    "\\documentclass{article}\n\\usepackage{longtable}\n\\usepackage[labelsep=period]{caption}\n\\begin{document}\n\
     \\begin{longtable}{l}\\caption{Long}\\\\ a\\\\ \\end{longtable}\n\\end{document}\n",
  );
  assert!(
    xml.contains(r#"<caption><tag close=". ">Table 1</tag>Long</caption>"#),
    "{xml}"
  );
}

/// caption.sty:171-172, latex.ltx:18557 `\protected@edef\@curroptions`: an option value is passed as written, so
/// `figurename=\textsc{Fig}` names the figures "Fig" in small caps (pdflatex "Fig 1 Small caps name."), and
/// subcaption's options still reach its sub-captions ("a").
#[test]
fn caption_option_values_unexpanded() {
  let xml = clean!("captions-floats/caption_option_values_unexpanded.tex");
  for caption in [
    r#"<caption><tag close=" "><text font="sansserif bold smallcaps" fontsize="90%" xml:id="S0.F1.3">Fig<text font="italic" xml:id="S0.F1.3.1"> 1</text></text></tag><text font="italic" fontsize="90%" xml:id="S0.F1.4">Small caps name.</text></caption>"#,
    r#"<caption><tag close=" "><text font="sansserif bold smallcaps" fontsize="90%" xml:id="S0.F2.3">Fig<text font="italic" xml:id="S0.F2.3.1"> 2</text></text></tag><text font="italic" fontsize="90%" xml:id="S0.F2.4">Parent.</text></caption>"#,
    r#"<text fontsize="90%" xml:id="S0.F2.sf1.2">a</text>"#,
  ] {
    assert!(xml.contains(caption), "{caption}\n{xml}");
  }
}

/// caption3.sty:1336-1345, newfloat.sty:179-188: `figurename=`/`tablename=` are added to babel's `\captionsenglish`,
/// so `\begin{document}` keeps them (pdflatex "Fig. 1: Fig text.", "Tab. 1: Tab text.").
#[test]
fn caption_names_survive_babel() {
  let xml = clean!("captions-floats/caption_names_survive_babel.tex");
  for caption in [
    r#"<caption><tag close=": "><text font="bold" xml:id="S0.F1.1">Fig. 1</text></tag>Fig text.</caption>"#,
    r#"<caption><tag close=": "><text font="bold" xml:id="S0.T1.1">Tab. 1</text></tag>Tab text.</caption>"#,
  ] {
    assert!(xml.contains(caption), "{caption}\n{xml}");
  }
}

/// caption3.sty:1680-1683, :1713-1748 `\caption@ifempty`: a caption of only a `\label`, an `\index` or spaces has no
/// separator and no text format (pdflatex "Figure 1" … "Figure 5", "Figure 6: Six."), and its label still names the
/// figure.
#[test]
fn caption_empty_text_ignores_label() {
  let xml = clean!("captions-floats/caption_empty_text_ignores_label.tex");
  for caption in [
    r#"<caption><tag>Figure 1</tag></caption>"#,
    r#"<caption class="ltx_centering"><tag>Figure 2</tag></caption>"#,
    r#"<caption><tag>Figure 3</tag></caption>"#,
    r#"<caption><tag>Figure 4</tag> <indexmark> <indexphrase key="x">x</indexphrase> </indexmark></caption>"#,
    r#"<caption><tag>Figure 5</tag></caption>"#,
    r#"<caption><tag close=": ">Figure 6</tag>Six.</caption>"#,
    r#"<p xml:id="p1.1">See <ref labelref="LABEL:f2"/> and <ref labelref="LABEL:f3"/>.</p>"#,
  ] {
    assert!(xml.contains(caption), "{caption}\n{xml}");
  }
  assert_eq!(xml.matches(r#"close=": ""#).count(), 1, "{xml}");
}

/// The inline ORCID links take an orcid.org address's iD as the frontmatter setter does (OXIDIZED_DESIGN_DIVERGENCES
/// #616): apa7's `\addORCIDlink` (apa7.cls:322-326 builds `https://orcid.org/#2`, doubled in pdflatex as in
/// orcidlink.sty) and mrm's `\orcid` and three-argument `\author`.
#[test]
fn orcid_inline_link_takes_the_id() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/orcid_inline_link_takes_the_id.tex"
    ),
    Some("ar5iv.sty"),
  );
  // The one Warning is pgf's svg.path library, which apa7 loads for its own logo (`\pgf@lib@svg@relative`), as
  // `perfect_kernel_batch61::apa7_author_block_maps_onto_the_frontmatter` pins it.
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(
    log.contains(
      "The conditional \\pgf@lib@svg@relative is being defined but doesn't start with \\if"
    ),
    "{log}"
  );
  let xml = squash(&xml);
  assert!(
    xml.contains(
      r#"<personname>Ann One <ref class="ltx_orcid" href="https://orcid.org/0000-0002-1825-0097" title="ORCID 0000-0002-1825-0097">"#
    ),
    "{xml}"
  );
  let xml = clean_doc(
    "\\documentclass{mrm}\n\\begin{document}\n\\title{T}\n\\author[1]{Ann One}{https://orcid.org/0000-0002-1825-0097}\n\
     \\maketitle\nText by Bob\\orcid{http://orcid.org/0000-0001-5109-3700}.\n\\end{document}\n",
  );
  for link in [
    r#"<contact role="orcid"><ref class="ltx_orcid" href="https://orcid.org/0000-0002-1825-0097" title="ORCID 0000-0002-1825-0097">"#,
    r#"<p xml:id="p1.1">Text by Bob<ref class="ltx_orcid" href="https://orcid.org/0000-0001-5109-3700" title="ORCID 0000-0001-5109-3700">"#,
  ] {
    assert!(xml.contains(link), "{link}\n{xml}");
  }
  assert!(!xml.contains("orcid.org/http"), "{xml}");
}

/// caption.sty:124-129: caption v1's size options are the whole caption font (`\def\captionfont{\small}`), so
/// `[font=it,small,bf]` sets the caption small and upright with a bold label (pdflatex: CMBX9 label, CMR9 text).
#[test]
fn caption_v1_size_option_is_the_caption_font() {
  let xml = clean_doc(
    "\\documentclass{article}\n\\usepackage[font=it,small,bf]{caption}\n\\begin{document}\n\
     \\begin{figure}\\caption{Text}\\end{figure}\n\\end{document}\n",
  );
  assert!(
    xml.contains(
      r#"<caption><tag close=": "><text font="bold" fontsize="90%" xml:id="S0.F1.1">Figure 1</text></tag><text fontsize="90%" xml:id="S0.F1.2">Text</text></caption>"#
    ),
    "{xml}"
  );
}

/// caption.sty:114-147's v1 keywords are compared with the option as written: `\ProcessOptions` read the list
/// (latex.ltx:18557 `\protected@edef`) and caption's `\@expandtwoargs` (caption3.sty:374-399) is another, so a robust
/// command's body is not run at package load (an undefined `\laterdefined` there) and names the figures as it reads
/// when they are typeset (pdflatex 0 errors, "Fx 1: One", "Gx 2: Two").
#[test]
fn caption_option_key_is_read_unexpanded() {
  let xml = clean!("captions-floats/caption_option_key_is_read_unexpanded.tex");
  for caption in [
    r#"<caption><tag close=": ">Fx 1</tag>One</caption>"#,
    r#"<caption><tag close=": ">Gx 2</tag>Two</caption>"#,
  ] {
    assert!(xml.contains(caption), "{caption}\n{xml}");
  }
}

/// A `\<type>name` of `\protect` and a robust, `\newcommand` or xparse command is a name noun (`\protect` is `\relax`
/// where the tag is typeset, latex.ltx `\@typeset@protect`): the tags print it (pdflatex "Fx 1: One", "Table 1: Gx",
/// "Hx 2: Two", "Ix 3: Three").
#[test]
fn protected_robust_name_is_a_name_noun() {
  let xml = clean!("captions-floats/protected_robust_name_is_a_name_noun.tex");
  for figure in [
    r#"<figure inlist="lof" xml:id="S0.F1"> <tags> <tag>Fx 1</tag> <tag role="refnum">1</tag> <tag role="typerefnum">Fx 1</tag> </tags> <toccaption><tag close=" ">1</tag>One</toccaption> <caption><tag close=": ">Fx 1</tag>One</caption> </figure>"#,
    r#"<figure inlist="lof" xml:id="S0.F2"> <tags> <tag>Hx 2</tag> <tag role="refnum">2</tag> <tag role="typerefnum">Hx 2</tag> </tags> <toccaption><tag close=" ">2</tag>Two</toccaption> <caption><tag close=": ">Hx 2</tag>Two</caption> </figure>"#,
    r#"<figure inlist="lof" xml:id="S0.F3"> <tags> <tag>Ix 3</tag> <tag role="refnum">3</tag> <tag role="typerefnum">Ix 3</tag> </tags> <toccaption><tag close=" ">3</tag>Three</toccaption> <caption><tag close=": ">Ix 3</tag>Three</caption> </figure>"#,
    r#"<figure inlist="lof" xml:id="S0.F4"> <tags> <tag>Ex 4</tag> <tag role="refnum">4</tag> <tag role="typerefnum">Ex 4</tag> </tags> <toccaption><tag close=" ">4</tag>Four</toccaption> <caption><tag close=": ">Ex 4</tag>Four</caption> </figure>"#,
    r#"<caption><tag close=": ">Table 1</tag>Gx</caption>"#,
  ] {
    assert!(xml.contains(figure), "{figure}\n{xml}");
  }
}

/// latex.ltx:18557 `\protected@edef\@curroptions`, a raw package's `\DeclareOption*`: a macro option runs its declared
/// option, passed and `\edef`-built options arrive expanded, `\the\textwidth` is its value, and a protected macro, a
/// robust command, `\protect\foo` and an optional-argument `\newcommand` stand (pdflatex 0 errors, "<ALPHA>[gamma]
/// [delta=D][beta=\protect \textbf {x}][w=345.0pt][\po ][o=\protect \oa ][r=\protect \rb ][p=\protect \foo ]"; the
/// kernel's `\textbf` and the `\newcommand` have no `\protect` here, not being `\protect`-wrapped macros).
#[test]
fn processoptions_reads_as_protected_edef() {
  let (log, xml) = convert_files_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/loader/processoptions_reads_as_protected_edef.tex"
    ),
    &[(
      "poq68h.sty",
      include_str!("../../../tools/perfect_kernel/repros/loader/poq68h.sty"),
    )],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let xml = squash(&xml);
  let shown = r#"<p xml:id="p1.1"><text font="typewriter" xml:id="p1.1.1">macro:-&gt;&lt;ALPHA&gt;[gamma][delta=D][beta=\textbf {x}][w=345.0pt][\po ][o=\oa ][r=\protect \rb ][p=\protect \foo ]</text></p>"#;
  assert!(xml.contains(shown), "{shown}\n{xml}");
}

/// ieeeaccess.cls:121-133 and :75-79: the class's `accessblue` (here the PANTONE 3015 C spot colour's CMYK alternate,
/// 1 0.3 0 0.2) and caption fonts, which papers name in their caption setup (2405.14505, 2205.15416); since caption's
/// keys take effect (68h) a missing colour or `\figcapfont` is an Error per caption (pdflatex 0 on the witness). The
/// label's name is the class's `\figurename` "FIGURE" (ieeeaccess.cls:261; "Fig." before 78y r2).
#[test]
fn ieeeaccess_caption_class_colour() {
  let xml = clean!("class-bindings/ieeeaccess_caption_class_colour.tex");
  let caption = r##"<caption><tag close=": "><text color="#0080CC" font="sansserif bold" fontsize="70%" xml:id="S0.F1.1">FIGURE1</text></tag>One</caption>"##;
  assert!(xml.contains(caption), "{caption}\n{xml}");
}

/// IEEEojcsys.cls:3346-3348 `\AtBeginDocument{\definecolor{ojcolor}{rgb}{0,0.40,0.80}}` (after loading color), named by
/// a paper's `\captionsetup{labelfont={color=ojcolor}}` (2206.03535; pdflatex 0 on the witness).
#[test]
fn ieeeojcsys_caption_class_colour() {
  let xml = clean!("class-bindings/ieeeojcsys_caption_class_colour.tex");
  let caption = r##"<caption><tag close=": "><text color="#0066CC" xml:id="S0.F1.2">Fig. 1</text></tag>One</caption>"##;
  assert!(xml.contains(caption), "{caption}\n{xml}");
}
