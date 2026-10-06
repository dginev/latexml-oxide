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
  assert_eq!(streamed, eager);
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
