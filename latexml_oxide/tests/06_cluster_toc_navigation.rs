//! Cluster regressions in post-processing TOC generation and split-document
//! navigation.
//!
//! Split out of `06_cluster_regressions`; shares its helpers via
//! [`mod cluster`](cluster).

mod cluster;
use cluster::{convert_and_post, convert_and_post_navtoc};

/// Issue #291: `\setcounter{tocdepth}{0}` in a `book` must restrict the
/// `\tableofcontents` to chapters only. The `\tableofcontents` constructor
/// already computes the correct `select="ltx:part | ltx:chapter | ..."`
/// attribute from `tocdepth`; the defect was purely in POST — CrossRef's
/// `gen_toc` ignored `select` (and the TOC's `lists`), hardcoding
/// `NORMAL_TOC_TYPES` + `inlist=="toc"`, so every level leaked into the ToC.
/// Faithful Perl: `CrossRef.pm::gentoc` L246-261 filters by the `select`-derived
/// `$types` and `inlist_match($lists, ...)`. Witness = the issue's MWE.
#[test]
fn tocdepth_select_restricts_the_toc() {
  // tocdepth=0 ⇒ chapters only; sections/subsubsections must be filtered out.
  let x = convert_and_post("tests/cluster_regressions/tocdepth0.tex");
  assert!(
    x.contains("ltx_tocentry_chapter"),
    "#291: chapters must appear in the ToC:\n{x}"
  );
  assert!(
    !x.contains("ltx_tocentry_section"),
    "#291: \\setcounter{{tocdepth}}{{0}} must drop sections from the ToC, but a \
     section tocentry is present (CrossRef gen_toc ignored the `select` attr):\n{x}"
  );
  assert!(
    !x.contains("ltx_tocentry_subsubsection"),
    "#291: subsubsections must be dropped from a tocdepth=0 ToC:\n{x}"
  );

  // Guard against over-filtering: with the book default (tocdepth=2), sections
  // MUST still appear.
  let y = convert_and_post("tests/cluster_regressions/tocdepth_default.tex");
  assert!(
    y.contains("ltx_tocentry_chapter") && y.contains("ltx_tocentry_section"),
    "#291 guard: default tocdepth must still list chapters AND sections:\n{y}"
  );

  // Connected behavior — upstream LaTeXML#2316 / arXiv-fork: the abstract carries
  // inlist="toc" (so it shows in the navigation TOC), but the user
  // `\tableofcontents` emits a `select` that omits ltx:abstract, so the abstract
  // must be EXEMPT here. Honoring `select` (the #291 fix) is exactly what keeps
  // it out; before the fix `gen_toc` ignored `select`, so the abstract LEAKED
  // into `\tableofcontents` (`ltx_tocentry_abstract` present). `convert_and_post`
  // runs no navigation TOC, so the only TOC here is the user's.
  let z = convert_and_post("tests/cluster_regressions/toc_abstract_exempt.tex");
  assert!(
    z.contains("ltx_tocentry_section"),
    "#2316 guard: sections must appear in \\tableofcontents:\n{z}"
  );
  assert!(
    !z.contains("ltx_tocentry_abstract"),
    "#2316/#291 guard: the abstract (inlist=toc, for the nav TOC) must stay \
     EXEMPT from the user \\tableofcontents, whose `select` omits ltx:abstract:\n{z}"
  );
}
/// Upstream LaTeXML#2316 / arXiv-fork, the *inclusion* half: with the `context`
/// navigation TOC enabled, the abstract MUST appear in the nav TOC (screenreader
/// accessibility) — and, because its `select`-less nav TOC accepts all types
/// while the user `\tableofcontents` omits `ltx:abstract`, it must appear
/// **exactly once** (nav TOC only, not the inline one). This is the companion of
/// `tocdepth_select_restricts_the_toc`'s exempt half: both rely on `gen_toc`
/// honoring `select` (the #291 fix). Before that fix the abstract appeared twice
/// (leaked into `\tableofcontents`). Witness = the issue's frontmatter shape.
///
/// The navigation TOC now runs Perl's `format="context"` path
/// (`gen_toc_context`); on a single page that reduces to the same downward tree
/// as a normal TOC, so the count == 1 invariant holds. The multi-page breadcrumb
/// shape is covered by `context_toc_breadcrumb_across_split_pages`.
#[test]
fn nav_toc_includes_abstract_issue_2316() {
  let x = convert_and_post_navtoc("tests/cluster_regressions/toc_abstract_exempt.tex");
  let n = x.matches("ltx_tocentry_abstract").count();
  assert_eq!(
    n, 1,
    "#2316: the abstract must appear in the navigation TOC exactly once (nav \
     only — present for accessibility, exempt from \\tableofcontents), got {n}:\n{x}"
  );
  // Sections still populate both TOCs — sanity that the nav TOC was built at all.
  assert!(
    x.contains("ltx_tocentry_section"),
    "#2316: sections missing — navigation TOC was not generated:\n{x}"
  );
}
/// Issue #291 hardening: a *negative* `\setcounter{tocdepth}` must not panic.
/// `\tableofcontents` builds `select` from `tocdepth` by taking the first
/// `tocdepth + 1` section types; the old code cast that through `as usize`, so
/// `tocdepth = -1` (parts only) overflowed — a debug panic, and in release a
/// silently over-full ToC (`{-2}` listed everything). Faithful Perl
/// (`latex_constructs.pool.ltxml` L727-733) computes `0 .. $td` in signed space,
/// an empty range for negative `$td`. `tocdepth = -1` ⇒ the part stays, chapters
/// and sections are dropped. The conversion completing at all is the no-panic
/// guard (the fixture converts under the debug profile's overflow-checks).
#[test]
fn tocdepth_negative_is_parts_only_no_panic() {
  let x = convert_and_post("tests/cluster_regressions/tocdepth_negative.tex");
  assert!(
    x.contains("ltx_tocentry_part"),
    "#291: tocdepth=-1 must still list the part:\n{x}"
  );
  assert!(
    !x.contains("ltx_tocentry_chapter"),
    "#291: tocdepth=-1 (parts only) must drop chapters from the ToC:\n{x}"
  );
  assert!(
    !x.contains("ltx_tocentry_section"),
    "#291: tocdepth=-1 must drop sections from the ToC:\n{x}"
  );
}
/// Issue #291 latent fix: honoring the `<ltx:TOC>` `lists` attribute (`lof`)
/// also repairs `\listoffigures`/`\listoftables`, which the old hardcoded `"toc"`
/// bucket broke outright — `\listoffigures` listed a document *section* (an
/// `inlist="toc"` entry) instead of the figures (`inlist="lof"`). It must now
/// list exactly the figures and no section. Faithful Perl: `\listoffigures`
/// emits `<ltx:TOC lists='lof'>` and CrossRef draws only from that `inlist`
/// bucket. Guards a fix the #291 change delivered but did not otherwise cover.
#[test]
fn listoffigures_lists_figures_not_toc_sections() {
  let x = convert_and_post("tests/cluster_regressions/listoffigures.tex");
  let n = x.matches("ltx_tocentry_figure").count();
  assert_eq!(
    n, 2,
    "#291: \\listoffigures must list both figures (inlist=lof), got {n}:\n{x}"
  );
  assert!(
    !x.contains("ltx_tocentry_section"),
    "#291: \\listoffigures must NOT list document sections (inlist=toc) — the old \
     hardcoded `toc` bucket did exactly that:\n{x}"
  );
}
