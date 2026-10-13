//! Red/green guards for batch 65z: package bindings define the commands and internals their real `.sty` defines
//! and arXiv papers call (run 340's first-error "undefined" cluster, row 7: caption/subcaption, bigstrut, natbib,
//! titlesec/titling, array, authblk, IEEEtrantools, accents, acronym, crop, enumitem, ulem, footmisc, listings,
//! amsthm, feynmf). Repros `tools/perfect_kernel/repros/package-bindings/*`.
use super::perfect_kernel_batch46::{error_count, warning_count};

fn convert(tex: &str) -> String {
  let (log, xml) = latexml::util::test::convert_with(tex, Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  xml
}

fn assert_contains(xml: &str, texts: &[&str]) {
  for text in texts {
    assert!(xml.contains(text), "{text}\n{xml}");
  }
}

/// caption.sty:633-638 `\subcaptionsetup`, :396-413 `\captionlistentry` (the counter step a `\label` after it
/// refers to), :415-428 `\captiontext*` (the panel's label, "(a)") and `\captiontext` (label, separator, text). Witnesses 2508.10437, 2604.10403, 2407.10358.
#[test]
fn caption_subcaption_setup_list_entry_and_text() {
  let xml = convert(include_str!(
    "../../../tools/perfect_kernel/repros/package-bindings/caption_subcaptionsetup_listentry_text.tex"
  ));
  assert_contains(&xml, &[
    r#"<p xml:id="S0.F1.2"><text fontsize="90%" xml:id="S0.F1.2.1">(a)</text></p>"#,
    r#"<figure inlist="lof" labels="LABEL:fig:entry" xml:id="S0.F2">"#,
    r#"<tag role="refnum">2</tag>"#,
    ">Figure 2</text>: Text only</p>",
  ]);
}

/// bigstrut.sty:28 `\bigstrutjot`. Witnesses 2103.02346, 2103.03442.
#[test]
fn bigstrut_jot_is_a_dimen() {
  let xml = convert(include_str!(
    "../../../tools/perfect_kernel/repros/package-bindings/bigstrut_jot.tex"
  ));
  assert_contains(&xml, &[
    r#"<td align="center" border="b t" xml:id="p1.1.1.1">A</td>"#,
  ]);
}

/// natbib.sty:1111-1115 the hyperref hooks, which kotexutf.sty:233 extends. Witnesses 2508.13812, 2103.13674.
#[test]
fn natbib_provides_its_hyperref_hooks() {
  let xml = convert(include_str!(
    "../../../tools/perfect_kernel/repros/package-bindings/natbib_hyper_hooks.tex"
  ));
  assert_contains(&xml, &[
    r#"<tag role="refnum">Knuth (1984)</tag>"#,
    r#"<bibref bibrefs="k""#,
  ]);
}

/// titlesec.sty:1384-1414 `\newpagestyle` loads titleps; :103 `\ttl@straightclass{section}` sets a section.
/// Witnesses 2103.06559, 2411.16470.
#[test]
fn titlesec_pagestyle_and_straight_class() {
  let xml = convert(include_str!(
    "../../../tools/perfect_kernel/repros/package-bindings/titlesec_pagestyle_straightclass.tex"
  ));
  assert_contains(&xml, &[
    r#"<title><tag close=" ">1</tag>One</title>"#,
    r#"<title><tag close=" ">2</tag>Two</title>"#,
    r#"<title><tag close=" ">3</tag>Three</title>"#,
  ]);
  assert!(!xml.contains("BROKEN"), "{xml}");
}

/// titling.sty:245-252 `\thetitle` is the title, without its `\thanks`, before `\maketitle`. Witnesses 1904.02630,
/// 2008.03410.
#[test]
fn titling_thetitle_before_maketitle() {
  let xml = convert(include_str!(
    "../../../tools/perfect_kernel/repros/package-bindings/titling_thetitle.tex"
  ));
  assert_contains(&xml, &[r#"<p xml:id="p1.1">Title: The Title</p>"#]);
}

/// titling.sty:245-268 sets `\thetitle`, `\theauthor` and `\thedate` by `\protected@xdef` when `\title`, `\author`,
/// `\date` run, so `\newcommand{\thetitle}{…}\title{\thetitle}` takes that text once; `\maketitle`'s
/// `\global\let\thetitle\@title` (Perl titling.sty.ltxml:85, KNOWN_PERL_ERRORS #636) made it expand to itself, a Fatal
/// timeout.
#[test]
fn titling_own_thetitle_does_not_loop() {
  let xml = convert(include_str!(
    "../../../tools/perfect_kernel/repros/package-bindings/titling_own_thetitle.tex"
  ));
  assert_contains(&xml, &["T=Own Title; A=B. Writer, C. Writer; D=2020."]);
}

/// titling.sty:245-268's `\appendiargdef` wraps the setter in force when titling loads: a class's own `\title`/`\author`
/// (amsart, revtex4-1, IEEEtran) or authblk's `\author` sets `\thetitle`/`\theauthor` too — through the setter's slot
/// call (revtex4, authblk) or, at `\begin{document}` and `\maketitle`, from what the setter stored (amsart, also with
/// its `\title`/`\author` in the body; IEEEtran, over its placeholder `\theauthor`). RED residual:
/// `package-bindings/titling_setter_llncs` (llncs/acmart/elsarticle/svjour3/aastex setters store no `\@author`).
#[test]
fn titling_under_class_and_package_setters() {
  for tex in [
    include_str!("../../../tools/perfect_kernel/repros/package-bindings/titling_setter_amsart.tex"),
    include_str!("../../../tools/perfect_kernel/repros/package-bindings/titling_setter_revtex.tex"),
    include_str!(
      "../../../tools/perfect_kernel/repros/package-bindings/titling_setter_authblk.tex"
    ),
    include_str!(
      "../../../tools/perfect_kernel/repros/package-bindings/titling_setter_amsart_body.tex"
    ),
    include_str!(
      "../../../tools/perfect_kernel/repros/package-bindings/titling_setter_ieeetran.tex"
    ),
  ] {
    let xml = convert(tex);
    assert_contains(&xml, &[">TT[My Title] TA[A. Author]</p>"]);
  }
}

/// A paper's own `\thetitle` stands where titling would not set it: with no `\title` at all, and redefined after
/// `\begin{document}` (the `\maketitle` fallback acts only on a field changed since; amsart, whose setter reaches no
/// slot).
#[test]
fn titling_keeps_a_papers_own_thetitle() {
  for (tex, text) in [
    (
      include_str!(
        "../../../tools/perfect_kernel/repros/package-bindings/titling_own_thetitle_no_title.tex"
      ),
      ">TT[Own Title] TA[A. Author] TD[]</p>",
    ),
    (
      include_str!(
        "../../../tools/perfect_kernel/repros/package-bindings/titling_own_thetitle_renewed.tex"
      ),
      ">TT[Custom Title] TA[A. Author] TD[]</p>",
    ),
  ] {
    let xml = convert(tex);
    assert_contains(&xml, &[text]);
  }
}

/// array.sty:121-122, :370, :430-431 the registers raw alignment code uses. Witnesses 2508.08881, 2103.08263,
/// 2004.09405, 2006.15794.
#[test]
fn array_internal_registers() {
  let xml = convert(include_str!(
    "../../../tools/perfect_kernel/repros/package-bindings/array_internals.tex"
  ));
  assert_contains(&xml, &[
    r#"<td align="center" xml:id="p1.1.1.1">A</td>"#,
    "Sep 2.0pt, surround 4.0pt.",
  ]);
}

/// authblk.sty:98-167 the state an `\affil`-derived `\email` reads; the frontmatter as without it. Witnesses
/// 2103.17238, 1904.03969, 2406.11632.
#[test]
fn authblk_state_for_copied_affil_code() {
  let xml = convert(include_str!(
    "../../../tools/perfect_kernel/repros/package-bindings/authblk_internals.tex"
  ));
  assert_contains(&xml, &[
    "<personname>Ann Able</personname>",
    "<personname>Bob Baker</personname>",
  ]);
  assert_eq!(
    xml
      .matches(r#"role="affiliation">One University</contact>"#)
      .count(),
    2,
    "{xml}"
  );
}

/// IEEEtrantools.sty: `\bstctlcite`, `\IEEEsettextheight`, `\IEEEstrut`, and `\yesnumber` numbering a row of
/// `IEEEeqnarray*`. Witnesses 2103.12231, 2211.06691, 1310.1135, 2008.05838.
#[test]
fn ieeetrantools_commands_and_yesnumber() {
  let xml = convert(include_str!(
    "../../../tools/perfect_kernel/repros/package-bindings/ieeetrantools_commands.tex"
  ));
  assert_contains(&xml, &[r#"<tags>
          <tag>(1)</tag>
          <tag role="refnum">1</tag>
        </tags>"#]);
}

/// accents.sty:40-47 `\cc@accent{<mathchar>}{<nucleus>}` the math accent; :111 `\cc@style` in `\widebar`.
/// Witnesses 2508.19844, 2103.15271.
#[test]
fn accents_internals_are_the_accent() {
  let xml = convert(include_str!(
    "../../../tools/perfect_kernel/repros/package-bindings/accents_internals.tex"
  ));
  assert_contains(&xml, &[r#"<XMApp xml:id="p1.m2.1.1">
            <XMTok name="hat" role="OVERACCENT" stretchy="false">^</XMTok>
            <XMTok font="italic" role="UNKNOWN">p</XMTok>
          </XMApp>"#]);
}

/// acronym.sty:624-643 `\@iaci` (the defined article, short once used, "a" otherwise), :208 `\@firstupper`,
/// :127 `\texorpdfstring`. Witnesses 1904.05584, 2407.19212, 2508.18123.
#[test]
fn acronym_indefinite_articles() {
  let xml = convert(include_str!(
    "../../../tools/perfect_kernel/repros/package-bindings/acronym_indefinite.tex"
  ));
  assert_contains(&xml, &[
    r#"<p xml:id="p1.1">A  <glossaryref inlist="acronym" key="LSTM" show="long"/>"#,
    r#"Then an <glossaryref inlist="acronym" key="LSTM" show="short"/> again and a  <glossaryref inlist="acronym" key="SVM" show="long"/>"#,
    "and text.</p>",
  ]);
}

/// crop.sty:163 `\CROP@size`, :288-331 the corner marks a class renews. Witnesses 2508.14007, 2406.14864.
#[test]
fn crop_size_and_marks() {
  let xml = convert(include_str!(
    "../../../tools/perfect_kernel/repros/package-bindings/crop_size.tex"
  ));
  assert_contains(&xml, &["Text. Stock 100.0pt by 200.0pt."]);
}

/// enumitem.sty:139 `\labelindent`, :969 `\enit@format`, :944 `\enit@type` (0 in an enumerate: the inline item
/// steps the counter). Witnesses 2508.14310, 2406.17390, 2411.02902.
#[test]
fn enumitem_internals_and_list_type() {
  let xml = convert(include_str!(
    "../../../tools/perfect_kernel/repros/package-bindings/enumitem_internals.tex"
  ));
  assert_contains(&xml, &["<p xml:id=\"S0.I1.i1.p1.1\">One 2.\u{a0}Two</p>"]);
}

/// ulem.sty:313-316 `\ULforem`/`\normalem`, :63-73 `\ULset`, `\UL@setULdepth`. Witnesses 2103.16561, 0709.1684,
/// 2411.10835.
#[test]
fn ulem_forem_normalem_and_set() {
  let xml = convert(include_str!(
    "../../../tools/perfect_kernel/repros/package-bindings/ulem_internals.tex"
  ));
  assert_contains(&xml, &[
    r#"<text class="ltx_ulem_uline" xml:id="p1.1.1">under</text>"#,
    r#"<emph font="italic" xml:id="p1.1.2">back</emph>"#,
    r##"<text color="#FF0000" xml:id="p1.1.3">red</text> mine.</p>"##,
  ]);
}

/// footmisc.sty:496-512 `\DefineFNsymbolsTM`. Witnesses 1904.04281, 1107.5789.
#[test]
fn footmisc_define_fnsymbols_tm() {
  let xml = convert(include_str!(
    "../../../tools/perfect_kernel/repros/package-bindings/footmisc_fnsymbolstm.tex"
  ));
  assert_contains(&xml, &[r#"</tags>Note.</note>"#]);
}

/// listings.sty:24-26 `\lst@CheckVersion`, :405-459 `\lst@mode` and the modes, :481 `\lst@token`, :993
/// `\lst@SaveOutputDef`. Witnesses 2103.02487, 2508.14132, 2004.05517, 2508.15166.
#[test]
fn listings_internals_for_literates_and_styles() {
  let xml = convert(include_str!(
    "../../../tools/perfect_kernel/repros/package-bindings/listings_internals.tex"
  ));
  assert_contains(&xml, &[
    r#"<text class="ltx_lst_identifier" xml:id="lstnumberx1.1">x</text>"#,
    r#"<p xml:id="p1.2">Underscore defined.</p>"#,
  ]);
  assert!(xml.contains(":=") && xml.contains(">y</text>"), "{xml}");
}

/// amsthm.sty:177-180 `\thm@headsep`, `\dth@everypar`; :316 `\displaymath@qed` the QED in the display. Witnesses
/// 2103.16309, 2508.18305, 2406.02526.
#[test]
fn amsthm_internals_and_display_qed() {
  let xml = convert(include_str!(
    "../../../tools/perfect_kernel/repros/package-bindings/amsthm_internals.tex"
  ));
  assert_contains(&xml, &[
    r#"<Math mode="display" tex="x=y.\qed""#,
    r#"<XMTok font="italic" role="PUNCT">∎</XMTok>"#,
  ]);
}

/// feynmf.sty:182 `\fmfkeep` in a graph, :239 `\fmfreuse` sets it again. Witnesses hep-lat0210017, hep-th9805098.
#[test]
fn feynmf_keep_and_reuse() {
  let xml = convert(include_str!(
    "../../../tools/perfect_kernel/repros/package-bindings/feynmf_keep_reuse.tex"
  ));
  assert_eq!(xml.matches(r#"role="feynman-diagram""#).count(), 2, "{xml}");
  assert_eq!(
    xml.matches("(Feynman diagram, 40x25)</note>").count(),
    2,
    "{xml}"
  );
}
