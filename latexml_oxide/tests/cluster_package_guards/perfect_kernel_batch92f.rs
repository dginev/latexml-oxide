//! Red/green guards for batch 92f: the nature-pre / nature_mod binding (`latexml_contrib/src/nature_pre_cls.rs`, bound
//! for both names) defines the class's own list environments, verbatim from the class (TL nature.cls:67-81 and
//! :110-123, byte-identical in the copies papers ship), where it emptied them to nothing. `{affiliations}`, set after
//! `\maketitle`, is a `list` numbering each institution `\upshape$^{\arabic{enumi}}$` in a `\slshape` body, so a
//! late abstract follows it as the class prints them (the cover rule, `cover_order.rs` `is_body_prose`: a
//! `\noindent`ed list is a cover line, the emptied items were body prose); `{addendum}` labels its items through
//! `\addendumlabel`, the bold `\item[Competing Interests]`. Witnesses 2608.12563 (nature-pre), 1702.00936, 1408.2522,
//! 2412.11966 (`{affiliations}` under nature_mod), 2402.00473 (`{addendum}`). Each repro under
//! `tools/perfect_kernel/repros/class-bindings/` carries a cut of the class (written by `{filecontents*}`, the paper's
//! copy the binding stands in for) and compiles clean with pdflatex (TL 2025). One guard per environment: the
//! affiliations guard also converts the repro without its copy (the binding alone, the same XML).
use latexml::util::test::assert_element;

use super::{
  perfect_kernel_batch46::{convert_with, error_count, warning_count},
  perfect_kernel_batch84b::{creators, text},
  perfect_kernel_batch89a::cli_conversion,
};

const PRELOAD: &str = "ar5iv.sty";

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/class-bindings/",
      $path
    ))
  };
}

/// The names of the root's element children, in order (the output is indented two spaces a level).
fn root_children(xml: &str) -> Vec<String> {
  xml
    .lines()
    .filter_map(|line| line.strip_prefix("  <"))
    .filter(|rest| !rest.starts_with(['/', ' ', '?', '!']))
    .map(|rest| rest.split([' ', '>', '/']).next().unwrap_or("").to_string())
    .collect()
}

/// The document without its `{filecontents*}` copy of the class: the binding alone answers `\documentclass`.
fn without_copy(tex: &str) -> String {
  let start = tex.find("\\begin{filecontents*}").expect("a copy");
  let end = tex.find("\\end{filecontents*}\n").expect("its end") + "\\end{filecontents*}\n".len();
  format!("{}{}", &tex[..start], &tex[end..])
}

/// 2608.12563's shape: `{affiliations}` after `\maketitle`, a braced `\item{…}`, a plain `\item` and an `\item[*]`
/// e-mail line, then the abstract. The list is the class's — each item tagged by its raised number (the `*` its own
/// mark), its body slanted —, set as the cover a continued titlepage holds, and the abstract follows it, as pdflatex
/// prints them; the same without the paper's copy of the class.
#[test]
fn nature_affiliations_list_is_the_cover_before_the_abstract() {
  let tex = repro!("nature_affiliations_list_before_abstract.tex");
  let (log, xml) = convert_with(tex, Some(PRELOAD));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(
    root_children(&xml),
    ["title", "creator", "titlepage", "abstract", "section"],
    "{xml}"
  );
  assert_eq!(creators(&xml), [["Ann Author 1"]], "{xml}");
  assert_element(&xml, "titlepage", &[], TITLEPAGE);
  let at = xml.find("<abstract").expect("an abstract");
  assert_eq!(
    text(&xml[at..at + xml[at..].find("</abstract>").expect("closed")]),
    "Abstract text here.",
    "{xml}"
  );
  let (log, bare) = convert_with(&without_copy(tex), Some(PRELOAD));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(bare, xml);
  // (the binary, eager and `--streaming`: the same root order and titlepage — review rev92f S2)
  for streaming in [false, true] {
    let (log, xml) = cli_conversion(tex, streaming);
    assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
    assert_eq!(
      root_children(&xml),
      ["title", "creator", "titlepage", "abstract", "section"],
      "streaming={streaming}: {xml}"
    );
    assert_element(&xml, "titlepage", &[], TITLEPAGE);
  }
}

/// The addendum's items (2402.00473's `\item[Competing Interests]`, 1408.2522's bare `\item`): an itemize, each body
/// `\small`, the given label set by `\addendumlabel` (bold, its `\hspace{1em}` a space) and a bare `\item` the list's
/// "Acknowledgements" (not bold: the kernel's `\list` sets its default label without `\makelabel`, as Perl's,
/// latex_constructs.pool.ltxml:1642-1643, where LaTeX's `\@item` sets it through `\makelabel`; a label's styling is the
/// CSS theme's, user ruling 2026-10-30).
#[test]
fn nature_addendum_items_carry_the_class_labels() {
  let (log, xml) = convert_with(repro!("nature_addendum_labels.tex"), Some(PRELOAD));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(&xml, "itemize", &[], ADDENDUM);
  // (the binary, eager and `--streaming` — review rev92f S2)
  for streaming in [false, true] {
    let (log, xml) = cli_conversion(repro!("nature_addendum_labels.tex"), streaming);
    assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
    assert_element(&xml, "itemize", &[], ADDENDUM);
  }
}

// (the label digit's `font="italic"` is Perl's reading too, but TeX sets it upright — `\upshape$^{…}$`, CMR8 in the
// repro's PDF: a later fix of the math font under `\slshape` changes this row and is no regression — review rev92f N4)
const TITLEPAGE: &str = r#"<titlepage class="ltx_titlepage_continued" xml:id="id2">
  <block class="ltx_noindent" xml:id="p1">
    <itemize xml:id="p1.1">
      <item xml:id="S0.I1.i1">
        <tags>
          <tag><sup xml:id="S0.I1.i1.1"><text font="italic" xml:id="S0.I1.i1.1.1">1</text></sup></tag>
          <tag role="autoref">item 1<text xml:id="S0.I1.i1.2"/></tag>
          <tag role="refnum">1</tag>
          <tag role="typerefnum">item 1</tag>
        </tags>
        <para xml:id="S0.I1.i1.p1">
          <p xml:id="S0.I1.i1.p1.1"><text font="slanted" xml:id="S0.I1.i1.p1.1.1">Department of Physics, Budapest</text></p>
        </para>
      </item>
      <item xml:id="S0.I1.i2">
        <tags>
          <tag><sup xml:id="S0.I1.i2.1"><text font="italic" xml:id="S0.I1.i2.1.1">2</text></sup></tag>
          <tag role="autoref">item 2<text xml:id="S0.I1.i2.2"/></tag>
          <tag role="refnum">2</tag>
          <tag role="typerefnum">item 2</tag>
        </tags>
        <para xml:id="S0.I1.i2.p1">
          <p xml:id="S0.I1.i2.p1.1"><text font="slanted" xml:id="S0.I1.i2.p1.1.1">Institute of Chemistry, Szeged</text></p>
        </para>
      </item>
      <item xml:id="S0.I1.ix1">
        <tags>
          <tag>*</tag>
          <tag role="autoref">item <text xml:id="S0.I1.ix1.1"/></tag>
          <tag role="typerefnum">item *</tag>
        </tags>
        <para xml:id="S0.I1.ix1.p1">
          <p xml:id="S0.I1.ix1.p1.1"><text font="slanted" xml:id="S0.I1.ix1.p1.1.1">email: ann@example.org</text></p>
        </para>
      </item>
    </itemize>
  </block>
</titlepage>"#;

const ADDENDUM: &str = r#"<itemize xml:id="S1.p1.2">
  <item xml:id="S1.I1.i1">
    <tags>
      <tag>Acknowledgements</tag>
      <tag role="autoref">item <text xml:id="S1.I1.i1.1"/></tag>
      <tag role="typerefnum">1st item</tag>
    </tags>
    <para xml:id="S1.I1.i1.p1">
      <p xml:id="S1.I1.i1.p1.1"><text fontsize="90%" xml:id="S1.I1.i1.p1.1.1">We thank the referees.</text></p>
    </para>
  </item>
  <item xml:id="S1.I1.ix1">
    <tags>
      <tag><text font="bold" xml:id="S1.I1.ix1.1">Competing Interests</text> </tag>
      <tag role="autoref">item <text xml:id="S1.I1.ix1.2"/></tag>
      <tag role="typerefnum">item Competing Interests</tag>
    </tags>
    <para xml:id="S1.I1.ix1.p1">
      <p xml:id="S1.I1.ix1.p1.1"><text fontsize="90%" xml:id="S1.I1.ix1.p1.1.1">The authors declare none.</text></p>
    </para>
  </item>
</itemize>"#;
