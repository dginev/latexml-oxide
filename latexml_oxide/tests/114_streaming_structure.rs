//! Forced-streaming corpus sweep — the wide version of `113_streaming_core`.
//! Every fixture converts twice (eager, and streaming with an aggressive
//! 3-box budget); the XML must be byte-identical and both conversions error-free.
//! One suite per test binary — see `streaming_sweep/mod.rs` for why.

mod streaming_sweep;
use streaming_sweep::sweep_dir;

#[test]
fn streaming_matches_eager_on_structure() { sweep_dir("tests/structure"); }

/// 62zm: the unit right before a bibliography stays resident while the bibliography is open, so a streamed conversion
/// lets the bibliography take that unit's place as the eager one does (OXIDIZED_DESIGN_DIVERGENCES #456). Repro
/// sectioning-frontmatter/bibliography_section_titled_as_it_becomes_it.
#[test]
fn streaming_bibliography_takes_its_heading_unit() {
  let source = format!(
    "literal:{}",
    include_str!(
      "../../tools/perfect_kernel/repros/sectioning-frontmatter/bibliography_section_titled_as_it_becomes_it.tex"
    )
  );
  let (eager, eager_errors) = streaming_sweep::convert(&source, None);
  let (streamed, streamed_errors) = streaming_sweep::convert(&source, Some(3));
  assert_eq!((eager_errors, streamed_errors), (0, 0));
  assert!(!eager.contains("<section"), "{eager}");
  // The uncited ref-b's reference form (its autoref tag) is typeset only where a `\cite` prints it, and the streamed
  // run typesets it at the spill, as Perl does (the deferred-print fallback, OXIDIZED_DESIGN_DIVERGENCES #738, #750).
  let uncited_streamed =
    "<tag>[]</tag>\n          <tag role=\"autoref\"/>\n        </tags>\n        <bibblock>\nBaker";
  let uncited_eager = "<tag>[]</tag>\n        </tags>\n        <bibblock>\nBaker";
  assert!(eager.contains(uncited_eager), "{eager}");
  assert_eq!(streamed.replacen(uncited_streamed, uncited_eager, 1), eager);
}

/// 62zm: a unit's heading (and the bibliography's own title) stays resident while the unit is open — a blank line
/// after `\section{References}` is a spill seam, and the spilled heading had left the hook nothing to compare.
#[test]
fn streaming_keeps_an_open_units_heading() {
  let source = r"literal:\documentclass{article}
\usepackage{hyperref}
\begin{document}
Text \cite{a}.

\section{References}

\protect\phantomsection\label{refs}
\begin{list}{}{}
\bibitem{a} Able.
\end{list}
\end{document}";
  let (eager, eager_errors) = streaming_sweep::convert(source, None);
  let (streamed, streamed_errors) = streaming_sweep::convert(source, Some(1));
  assert_eq!((eager_errors, streamed_errors), (0, 0));
  assert!(!eager.contains("<section"), "{eager}");
  assert_eq!(eager.matches("<title").count(), 1, "{eager}");
  assert_eq!(streamed, eager);
}

/// 86g: a late abstract's marker waits for `insert_late_frontmatter` in a streamed build (the abstract is still queued
/// when its marker is built), which places the abstract behind the cover printed before it as the eager build does,
/// and removes the marker (OXIDIZED_DESIGN_DIVERGENCES #903). The hand-typeset cover, the class title page's deposit,
/// an abstract inside the deposit, the fallback without `\maketitle` (whose marker the flush reads first), and a raw
/// class's stores its title page prints before the cover (86g r2 review finding 3; their late run keeps its slot, the
/// fail-safe, as eager).
#[test]
fn streaming_late_abstract_follows_its_cover() {
  for repro in [
    include_str!(
      "../../tools/perfect_kernel/repros/sectioning-frontmatter/late_abstract_after_hand_typeset_cover.tex"
    ),
    include_str!(
      "../../tools/perfect_kernel/repros/sectioning-frontmatter/late_abstract_after_the_class_title_page.tex"
    ),
    include_str!(
      "../../tools/perfect_kernel/repros/sectioning-frontmatter/abstract_inside_the_class_title_page.tex"
    ),
    include_str!(
      "../../tools/perfect_kernel/repros/sectioning-frontmatter/abstract_after_in_place_setters_without_maketitle.tex"
    ),
  ] {
    let source = format!("literal:{repro}");
    let (eager, eager_errors) = streaming_sweep::convert(&source, None);
    let (streamed, streamed_errors) = streaming_sweep::convert(&source, Some(3));
    assert_eq!((eager_errors, streamed_errors), (0, 0));
    assert!(eager.contains("ltx_titlepage_continued"), "{eager}");
    assert!(!streamed.contains("_Capture_"), "{streamed}");
    assert_eq!(streamed, eager);
  }
  // A raw class's title page that prints frontmatter before the cover — an abstract store (the marker settles with
  // an abstract `\maketitle` placed already in the tree, which is no late one), its stored keywords and a
  // classification store (title-page provenance: the late run is not moved, the fail-safe) — under ar5iv.sty, as the
  // CLI and the batch guards convert: without it the class falls back to OmniBus, its title page and stores unread.
  let stores: [(&str, &[&str]); 2] = [
    (
      include_str!(
        "../../tools/perfect_kernel/repros/sectioning-frontmatter/late_abstract_after_a_class_store_abstract.tex"
      ),
      &[
        "title",
        "creator",
        "abstract",
        "titlepage",
        "abstract",
        "section",
      ],
    ),
    (
      include_str!(
        "../../tools/perfect_kernel/repros/sectioning-frontmatter/late_abstract_after_class_printed_frontmatter.tex"
      ),
      &[
        "title",
        "creator",
        "abstract",
        "keywords",
        "classification",
        "para",
        "section",
      ],
    ),
  ];
  for (repro, order) in stores {
    let source = format!("literal:{repro}");
    let convert = |budget| {
      streaming_sweep::convert_preloaded(
        &source,
        budget,
        latexml_contrib::dispatch,
        Some(vec!["ar5iv.sty".to_string()]),
      )
    };
    let (eager, eager_errors) = convert(None);
    let (streamed, streamed_errors) = convert(Some(3));
    assert_eq!((eager_errors, streamed_errors), (0, 0));
    assert!(!streamed.contains("_Capture_"), "{streamed}");
    assert_eq!(streamed, eager);
    let doc = libxml::parser::Parser::default()
      .parse_string(&eager)
      .expect("the output parses");
    let root = doc.get_root_element().expect("a root");
    let names: Vec<String> = root
      .get_child_elements()
      .into_iter()
      .map(|n| n.get_name())
      .filter(|n| n != "resource")
      .collect();
    assert_eq!(names, order, "{eager}");
  }
}
