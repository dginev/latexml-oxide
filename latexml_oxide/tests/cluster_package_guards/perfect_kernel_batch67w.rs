//! Red/green guards for batch 67w: the Wiley class bindings lacked commands and environments their real classes
//! define (binding gaps G2: WileyNJD-v2, WileyNJDv5, WileyASNA-v1, WileyMSP-template), and read the optional heading
//! of `\abstract[Summary]{…}` as text of the abstract. Round 2: the theorem styles as each version defines them,
//! `\uns` without the formula that ends the paragraph's ligatures, WileyNJD-v1's reference-style loads.
use latexml::util::test::convert_with;

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

/// WileyNJD-v2.cls:1781 `\presentaddress` (1701.02349, 1705.09846, 1712.08341), :1857 `\abstract[2][]` (1701.02349,
/// 1712.08341), :2923-3030 the theorem set (1803.11411, 1806.09518, 1904.09923), :2816-2828 `{tablenotes}`/`\tnote`
/// (1808.05540, 1808.09739), :466 `{boxtext}` (1811.09505, 2012.13740), :3393 `{ack}` (2205.06045), :3445
/// `{biography}` (1705.00674, 1812.10266), :1820 `\doiheadtext` (1803.06134, 2306.05470).
#[test]
fn wileynjd_v2_class_commands() {
  let xml = clean!("class-bindings/wileynjd_v2_class_commands.tex");
  assert!(
    xml.contains(
      r##"<abstract inlist="toc" name="Summary" xml:id="abstract1"> <p xml:id="abstract1.1">The abstract.</p> </abstract>"##
    ),
    "{xml}"
  );
  assert!(!xml.contains("[Summary]"), "{xml}");
  assert!(
    xml.contains(
      r##"<note role="present-address" xml:id="id1">400 Main Street, Minneapolis</note>"##
    ),
    "{xml}"
  );
  assert!(
    xml.contains(r##"<note role="doi" xml:id="id2">DOI: 10.1002/xyz.123</note>"##),
    "{xml}"
  );
  for (env, name) in [
    ("assumption", "Assumption"),
    ("eexample", "Example"),
    ("property", "Property"),
  ] {
    assert!(
      xml.contains(&format!(
        r##"<title class="ltx_runin"><tag><text font="bold" xml:id="Thm{env}1.2">{name} 1</text></tag><text font="bold" xml:id="Thm{env}1.3">.</text></title>"##
      )),
      "{env}: {xml}"
    );
  }
  // one note item, the class's `\item\relax` before the first `\item` not an empty item
  assert!(
    xml.contains(r##"<td align="left" xml:id="S1.T1.2.1.2">B<Math mode="inline" tex="\dagger" text="dagger" xml:id="S1.T1.m1">"##),
    "{xml}"
  );
  assert_eq!(xml.matches("<item ").count(), 1, "{xml}");
  assert!(
    xml.contains(r##"<para xml:id="S1.I1.ix1.p1"> <p xml:id="S1.I1.ix1.p1.1">A dagger note.</p> </para> </item> </itemize> </table>"##),
    "{xml}"
  );
  assert!(
    xml.contains(
      r##"<logical-block cssstyle="padding:9.0pt" framecolor="#000000" framed="rectangle" xml:id="S1.2"> <para class="ltx_noindent" xml:id="S1.p1"> <p xml:id="S1.p1.1"><text font="bold" xml:id="S1.p1.1.1">Box title</text></p> </para> <para xml:id="S1.p2"> <p xml:id="S1.p2.1">Boxed text.</p> </para> </logical-block>"##
    ),
    "{xml}"
  );
  assert_eq!(xml.matches("<section ").count(), 1, "{xml}");
  assert!(
    xml.contains(r##"<acknowledgements inlist="toc" name="Acknowledgements" xml:id="S1.acknowledgements1">We thank everyone. </acknowledgements>"##),
    "{xml}"
  );
  assert!(
    xml.contains(
      r##"<float class="biography" xml:id="S1.tab1"> <tabular xml:id="S1.tab1.1"> <tr xml:id="S1.tab1.1.1"> <td xml:id="S1.tab1.1.1.1"><inline-block xml:id="S1.tab1.1.1.1.1"> <p xml:id="S1.tab1.1.1.1.1.1"><text font="bold" xml:id="S1.tab1.1.1.1.1.1.1">Ann Author</text> is a professor.</p> </inline-block></td> </tr> </tabular> </float>"##
    ),
    "{xml}"
  );
}

/// WileyNJDv5.cls:601-608 `{boxwithhead}[2]` (2309.00837, 2405.03603, 2604.17508), :5764 `\bmsubsubsection`
/// (2309.00837, 2311.07331, 2602.03894), :6188 `{tablenotes}` with text before any `\item` (1808.09739), :6869 `{ack}`,
/// :6924 `{biography}` with a photo.
#[test]
fn wileynjdv5_box_environments() {
  let xml = clean!("class-bindings/wileynjdv5_box_environments.tex");
  assert!(
    xml.contains(r##"<title><tag close=" ">1.0.1</tag>Deep</title>"##),
    "{xml}"
  );
  assert!(
    xml.contains(
      r##"<logical-block cssstyle="padding:9.0pt" framecolor="#000000" framed="rectangle" xml:id="S1.SS0.SSS1.2"> <para class="ltx_noindent" xml:id="S1.SS0.SSS1.p2"> <p xml:id="S1.SS0.SSS1.p2.1"><text font="bold" xml:id="S1.SS0.SSS1.p2.1.1">BOX 1 The head</text></p> </para> <para xml:id="S1.SS0.SSS1.p3"> <p xml:id="S1.SS0.SSS1.p3.1">The boxed text.</p> </para> </logical-block> <para xml:id="S1.SS0.SSS1.p4"> <p xml:id="S1.SS0.SSS1.p4.1">After box.</p> </para>"##
    ),
    "{xml}"
  );
  assert!(
    xml.contains(r##"<td align="left" xml:id="S1.SS0.SSS1.tab1.1.1.2">Ba</td>"##),
    "{xml}"
  );
  assert!(
    xml.contains(r##"<para xml:id="S1.I1.i1.p1"> <p xml:id="S1.I1.i1.p1.1">a) First note, b) second.</p> </para> </item> </itemize>"##),
    "{xml}"
  );
  assert_eq!(xml.matches("<item ").count(), 1, "{xml}");
  assert!(
    xml.contains(r##"<acknowledgements inlist="toc" xml:id="S1.SS0.SSS1.acknowledgements1">Thanks. </acknowledgements>"##),
    "{xml}"
  );
  assert!(
    xml.contains(
      r##"<td xml:id="S1.SS0.SSS1.tab2.1.1.1"><graphics graphic="photo" options="width=60.0pt,keepaspectratio=true" xml:id="S1.SS0.SSS1.g1"/></td> <td xml:id="S1.SS0.SSS1.tab2.1.1.2"><inline-block xml:id="S1.SS0.SSS1.tab2.1.1.2.1"> <p xml:id="S1.SS0.SSS1.tab2.1.1.2.1.1"><text font="bold" xml:id="S1.SS0.SSS1.tab2.1.1.2.1.1.1">Ann Author</text> is a professor.</p> </inline-block></td>"##
    ),
    "{xml}"
  );
  assert!(
    xml.contains(r##"<note role="present-address" xml:id="id1">Somewhere 1</note>"##),
    "{xml}"
  );
}

/// WileyASNA-v1.cls:1300-1301 `\fundingAgency`/`\fundingNumber` in running text (1806.06754, 1810.06326, 1812.03179),
/// :2722 `\aaa` (1909.01905, 2303.03859), :269 `{boxtext}`, :1195 `\presentaddress`.
#[test]
fn wileyasna_funding_in_text() {
  let xml = clean!("class-bindings/wileyasna_funding_in_text.tex");
  assert!(
    xml.contains(
      r##"<p xml:id="S1.p1.1">We acknowledge the <text font="bold" xml:id="S1.p1.1.1">Czech Science Foundation</text> Grant No. <text font="italic" xml:id="S1.p1.1.2">16-03564Y</text>. See A&amp;A and ApJ.</p>"##
    ),
    "{xml}"
  );
  assert!(
    xml.contains(r##"<note role="present-address" xml:id="id1">Elsewhere 2</note>"##),
    "{xml}"
  );
  assert!(
    xml.contains(
      r##"<p xml:id="S1.p1.p1.1"><text font="bold" xml:id="S1.p1.p1.1.1">Box title</text></p>"##
    ),
    "{xml}"
  );
  assert!(
    xml.contains(r##"<abstract inlist="toc" name="Abstract" xml:id="abstract1"> <p xml:id="abstract1.1">An abstract.</p> </abstract>"##),
    "{xml}"
  );
}

/// WileyMSP-template.cls:34 `\dedication` (the template's empty `\dedication{}`: 2003.09099, 2006.03434, 2007.06369),
/// :49 `\threesubsection` (2009.06328, 2102.13559, 2111.07484).
#[test]
fn wileymsp_dedication_threesubsection() {
  let xml = clean!("class-bindings/wileymsp_dedication_threesubsection.tex");
  assert_eq!(xml.matches("<pubnote ").count(), 1, "{xml}");
  assert!(
    xml.contains(r##"<pubnote role="dedication">To all of us.</pubnote>"##),
    "{xml}"
  );
  assert!(
    xml.contains(
      r##"<p xml:id="S1.p1.1"><text font="italic" xml:id="S1.p1.1.1">Sample preparation</text>:<break/>Samples were prepared.</p>"##
    ),
    "{xml}"
  );
}

/// WileyNJD-v1.cls's reference styles load amsrefs (1511.08547's copy :1106-:3685; 1511.08547, 1707.00398, 1709.02675),
/// :256 `[labelformat=empty]{subfig}` (1511.08547, 1707.00398), :4712 listings (`\lstset`; WileyNJD-v2 1802.08557).
#[test]
fn wileynjd_v1_loads_amsrefs() {
  let xml = clean!("class-bindings/wileynjd_v1_loads_amsrefs.tex");
  assert!(
    xml.contains(
      r##"<bibentry key="knuth" type="article" xml:id="bib.bib1"> <bib-name role="author"> <surname>Knuth</surname> <givenname>Donald</givenname> </bib-name> <bib-title>The art</bib-title>"##
    ),
    "{xml}"
  );
  assert!(
    xml.contains(r##"<rule height="28.5pt" width="28.5pt"/> <toccaption><tag close=" ">a</tag>One</toccaption> <caption>One</caption> </figure>"##),
    "{xml}"
  );
}

/// The theorem `{thm}` a paper declares, one of `\theoremstyle{plain}` and the class's `{lemma}`, set as the class
/// sets them: `bodies` the font of each body (`None` upright); the paper's and the class's theorems with bold run-in
/// heads, the `plain` one with the class's `plain` head.
fn assert_theorem_styles(xml: &str, bodies: [Option<&str>; 3]) {
  for ((env, text), font) in [
    ("thm", "A paper theorem."),
    ("pthm", "A plain theorem."),
    ("lemma", "A class lemma."),
  ]
  .into_iter()
  .zip(bodies)
  {
    let body = match font {
      Some(font) => format!(r##"<text font="{font}" xml:id="Thm{env}1.p1.1.1">{text}</text>"##),
      None => text.to_owned(),
    };
    assert!(
      xml.contains(&format!(
        r##"<para xml:id="Thm{env}1.p1"> <p xml:id="Thm{env}1.p1.1">{body}</p> </para> </theorem>"##
      )),
      "{env}: {xml}"
    );
  }
  for (env, name) in [("thm", "Thm"), ("lemma", "Lemma")] {
    assert!(
      xml.contains(&format!(
        r##"<title class="ltx_runin"><tag><text font="bold" xml:id="Thm{env}1.2">{name} 1</text></tag><text font="bold" xml:id="Thm{env}1.3">.</text></title>"##
      )),
      "{env}: {xml}"
    );
  }
  // the class's `plain`: an italic head without punctuation, on a line of its own
  assert!(
    xml.contains(
      r##"<title><tag><text font="italic" xml:id="Thmpthm1.2">PThm 1</text></tag></title>"##
    ),
    "{xml}"
  );
}

/// WileyNJD-v2.cls:2902-2963 the class's `plain` and WB styles, :3005 WBstylethree left current (1712.08341's copy;
/// pdflatex: all three bodies upright, t1-stixgeneral). The binding ended with an invented `\theoremstyle{plain}`.
#[test]
fn wileynjd_v2_theorem_styles() {
  let xml = clean!("class-bindings/wileynjd_theorem_styles.tex");
  assert_theorem_styles(&xml, [None, None, None]);
}

/// WileyNJDv5.cls:6274-6335: `plain` and WBstyleone with italic bodies (2302.02310's copy; pdflatex: ptmr8t, ptmri8t,
/// ptmri8t).
#[test]
fn wileynjdv5_theorem_styles() {
  let xml = clean!("class-bindings/wileynjdv5_theorem_styles.tex");
  assert_theorem_styles(&xml, [None, Some("italic"), Some("italic")]);
}

/// WileyNJD-v2.cls:1180 `\uns` out of math is a thin space that leaves the paragraph's `--` an en dash (2208.08472).
/// Since 78a the class's own `$\,$` (a formula holding only a space, written as the six-per-em space U+2006), the
/// kernel ligating the text after it (`perfect_kernel_batch78a::text_after_a_space_only_formula_is_ligatured`); before,
/// the binding's `\thinspace` (U+2009) stood in.
#[test]
fn wileynjd_uns_keeps_en_dashes() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/class-bindings/wileynjd_uns_keeps_en_dashes.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  // unsquashed: the space (U+2006) is whitespace to `squash`
  assert!(
    xml.contains(
      "<p xml:id=\"S1.p1.1\">The 500\u{2006}hPa level over 1989\u{2013}2008, and <Math mode=\"inline\" tex=\"x{\\,}y\""
    ),
    "{xml}"
  );
  assert!(!xml.contains("--"), "{xml}");
}

/// WileyNJD-v1.cls (1511.08547's copy :33-43, :1104-3685): amsrefs only under a reference-style option, and the APA
/// style is amsrefs' (no apacite in the copies of 2015-2018).
#[test]
fn wileynjd_v1_reference_style_loads() {
  let xml = clean!("class-bindings/wileynjd_v1_reference_styles.tex");
  assert!(
    xml.contains(r##"<p xml:id="S1.p1.1">amsrefs not loaded; apacite not loaded.</p>"##),
    "{xml}"
  );
  let xml = clean!("class-bindings/wileynjd_v1_apa_is_amsrefs.tex");
  assert!(
    xml.contains(r##"<p xml:id="S1.p1.1">amsrefs loaded; apacite not loaded.</p>"##),
    "{xml}"
  );
}
