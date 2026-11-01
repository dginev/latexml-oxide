//! Red/green guards for batch 97f: a giant author list wrapped in a size environment is still a list, and AASTeX 6.x's
//! consecutive authors share the affiliations after them (OXIDIZED_DESIGN_DIVERGENCES #947). The 2015 BESIII template
//! wraps its whole author block in `\begin{small}\begin{center}…\end{center}\vspace{0.4cm}\end{small}` (1507.02068,
//! 1512.08197, 2201.02056: 385, 377 and 472 names one creator each); `unwrap_alignment_environment`
//! (author_block/separators.rs) now peels the declarations around the alignment environment and the spacing after it.
//! AASTeX's `\affiliation` is REVTeX's (`group_or_marked_affiliation`, revtex4_support_sty.rs): the group rule
//! (`annotate=run`, 2403.06760), a line its marks own by its marks (astro-ph0611722). Each guard converts its repro
//! under `tools/perfect_kernel/repros/sectioning-frontmatter/` from the binary, eager and `--streaming`, asserts the same
//! XML both ways, no Error and no Warning, and every creator (name and contacts in order). One guard per path: a
//! declaration environment, a declaration group (and bare declarations, and what stays wrapped), spacing after the
//! close (both unwrapping sites), the AASTeX group rule, and its marked lines.
use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch84b::creators,
  perfect_kernel_batch89a::cli_conversion,
};

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $name,
      ".tex"
    ))
  };
}

/// Converts `tex` from the binary eager and `--streaming`, checks each has no Error and no Warning and both give the
/// same XML, and returns the XML.
#[track_caller]
fn both_ways(tex: &str) -> String {
  let (log, eager) = cli_conversion(tex, false);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let (streamed_log, streamed) = cli_conversion(tex, true);
  assert_eq!(
    (error_count(&streamed_log), warning_count(&streamed_log)),
    (0, 0),
    "{streamed_log}"
  );
  assert_eq!(eager, streamed);
  eager
}

/// The three creators of the BESIII-shaped repros: each name with the collaboration and the affiliations its marks name.
fn bes_creators() -> Vec<Vec<String>> {
  let collaboration = "collaboration=(XYZ Collaboration)";
  [
    vec!["A. Able", collaboration, "affiliation=Uni One, City"],
    vec![
      "B. Baker",
      collaboration,
      "affiliation=Uni Two, Town",
      "affiliation=Also at Lab Three",
    ],
    vec!["C. Cole", collaboration, "affiliation=Uni One, City"],
  ]
  .into_iter()
  .map(|c| c.into_iter().map(String::from).collect())
  .collect()
}

/// The witness shape, `\begin{small}\begin{center} names \\ (XYZ Collaboration)\\ {\it $^{1}$ …}\end{center}
/// \vspace{0.4cm}\end{small}` (1507.02068; 97d: one creator holding an inline-block): three creators.
#[test]
fn size_environment_around_center_splits_the_block() {
  let xml = both_ways(repro!("author_size_environment_around_center_splits"));
  assert_eq!(creators(&xml), bes_creators());
  assert!(!xml.contains("inline-block"), "{xml}");
}

/// The group form `{\small \begin{center}…\end{center}}` (97d: one creator, the merge Error once the `center` is gone),
/// bare declarations before the environment, and nested declaration environments around `flushleft`: the same three
/// creators. A declaration environment with no alignment environment inside keeps its size on each name, and a command
/// that prints before the `center` stays a name (neither is peeled).
#[test]
fn declaration_group_around_center_splits_the_block() {
  let tex = repro!("author_size_group_around_center_splits");
  assert_eq!(creators(&both_ways(tex)), bes_creators());
  let wrap = |open: &str, close: &str| {
    let out = tex
      .replace("  {\\small\n    \\begin{center}", open)
      .replace("    \\end{center}\n  }\n}", close);
    assert_ne!(out, tex);
    out
  };
  let bare = wrap(
    "\\small\\bfseries\n    \\begin{center}",
    "    \\end{center}\n}",
  );
  assert_eq!(creators(&both_ways(&bare)), bes_creators());
  let nested = wrap(
    "\\begin{footnotesize}\\begin{itshape}\n    \\begin{flushleft}",
    "    \\end{flushleft}\\end{itshape}\\end{footnotesize}\n}",
  );
  assert_eq!(creators(&both_ways(&nested)), bes_creators());
  let lead = wrap("\\lead\n    \\begin{center}", "    \\end{center}\n}").replace(
    "\\begin{document}",
    "\\newcommand\\lead{Zed Zorn, }\n\\begin{document}",
  );
  let mut with_lead = vec![vec![
    "Zed Zorn".to_string(),
    "collaboration=(XYZ Collaboration)".to_string(),
  ]];
  with_lead.extend(bes_creators());
  assert_eq!(creators(&both_ways(&lead)), with_lead);
  let sized = r"\documentclass[prd,twocolumn]{revtex4-1}
\begin{document}
\title{T}
\author{\begin{small}A.~Able$^{1}$, B.~Baker$^{2}$\\ $^{1}$ Uni One\\ $^{2}$ Uni Two\end{small}}
\affiliation{}
\maketitle
Text.
\end{document}
";
  let xml = both_ways(sized);
  assert_eq!(creators(&xml), [["A. Able", "affiliation=Uni One"], [
    "B. Baker",
    "affiliation=Uni Two"
  ]]);
  assert_eq!(
    xml.matches(r#"<personname><text fontsize="90%""#).count(),
    2,
    "{xml}"
  );
}

/// Spacing after the closing `\end{center}` (`\end{center}\vspace{0.4cm}`, 97d: one creator) is no part of it: the block
/// still splits. And after a `center` set under the names (`… Bob Baker \begin{center}Department of
/// Physics\end{center}\vspace{2mm}`, 97d: "Bob Baker Department of Physics" one name), the line is both names'.
#[test]
fn spacing_after_the_center_still_ends_the_block() {
  let tex = repro!("author_center_then_spacing_splits");
  assert_eq!(creators(&both_ways(tex)), bes_creators());
  // (the document's own `\author`, at a line start: the header names one too)
  let start = tex.find("\n\\author{").expect("author") + 1;
  let end = tex.find("\n\\affiliation{}").expect("affiliation") + 1;
  let under = format!(
    "{}\\author{{Ann Able and Bob Baker \\begin{{center}}Department of \
     Physics\\end{{center}}\\vspace{{2mm}}}}\n{}",
    &tex[..start],
    &tex[end..]
  );
  assert_eq!(creators(&both_ways(&under)), [
    ["Ann Able", "affiliation=Department of Physics"],
    ["Bob Baker", "affiliation=Department of Physics"],
  ]);
  // An `\end{small}` after the `\end{center}` closes what opened before the names, so it is no spacing: 2012.14750's
  // `\author{\author{Author list} \begin{small}\begin{center}…\end{center}\vspace{0.6cm}\end{small}}` stays as
  // before, no Error (read as spacing, the `\begin{small}` stayed open in a names line: two Errors).
  let nested = format!(
    "{}\\author{{\\author{{Author list}} \\begin{{small}}\\begin{{center}}A.~Able$^{{1}}$, B.~Baker$^{{2}}$ \\\\ \
     $^{{1}}$ Uni One\\\\ $^{{2}}$ Uni Two\\end{{center}}\\vspace{{0.6cm}}\\end{{small}}}}\n{}",
    &tex[..start],
    &tex[end..]
  );
  let xml = both_ways(&nested);
  assert!(
    xml.contains("Author list") && xml.contains("Uni Two"),
    "{xml}"
  );
}

/// aastex631's `\author{Ann Able}\author{Bob Baker}\affiliation{Uni One}\affiliation{Uni Two}\author{Cat Cole}
/// \affiliation{Uni One}` (2403.06760's "K. Ieki 1, 46"; 97d gave Ann Able Uni One only): both affiliations each of the
/// first two authors', the third only Uni One.
#[test]
fn aastex6_consecutive_authors_share_every_affiliation_after_them() {
  let xml = both_ways(repro!("aastex6_consecutive_authors_share_affiliations"));
  assert_eq!(creators(&xml), [
    vec!["Ann Able", "affiliation=Uni One", "affiliation=Uni Two"],
    vec!["Bob Baker", "affiliation=Uni One", "affiliation=Uni Two"],
    vec!["Cat Cole", "affiliation=Uni One"],
  ]);
}

/// aastex631's marked lines under marked names (`Ann Able$^{1,2}$`, `\affiliation{$^{1}$Uni One}`; astro-ph0611722's
/// shape, 97d giving every author "1Uni One"): each line the names showing its mark, the mark off the line.
#[test]
fn aastex6_marked_affiliations_go_to_the_names_showing_the_mark() {
  let xml = both_ways(repro!("aastex6_marked_affiliations_go_by_marks"));
  assert_eq!(creators(&xml), [
    vec!["Ann Able 1,2", "affiliation=Uni One", "affiliation=Uni Two"],
    vec!["Bob Baker 1", "affiliation=Uni One"],
    vec!["Cat Cole 2", "affiliation=Uni Two"],
  ]);
}
