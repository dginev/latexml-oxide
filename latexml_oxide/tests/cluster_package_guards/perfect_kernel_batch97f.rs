//! Red/green guards for batch 97f: a giant author list wrapped in a size environment is still a list, and AASTeX 6.x's
//! consecutive authors share the affiliations after them (OXIDIZED_DESIGN_DIVERGENCES #947). The 2015 BESIII template
//! wraps its whole author block in `\begin{small}\begin{center}…\end{center}\vspace{0.4cm}\end{small}` (1507.02068,
//! 1512.08197, 2201.02056: 385, 377 and 472 names one creator each); `unwrap_alignment_environment`
//! (author_block/separators.rs) now peels the declarations around the alignment environment and the spacing after it.
//! AASTeX's `\affiliation` is REVTeX's (`group_or_marked_affiliation`, revtex4_support_sty.rs): the group rule
//! (`annotate=run`, 2403.06760) for names requesting no affiliation by a mark, `annotate=new` under `$^{N}$` names
//! (1202.0629), a line its marks own by its marks (astro-ph0611722), a line's leading label (a mark or a hand-printed
//! number) to the names requesting it, `\altaffilmark{N}` ones too (1810.01941, 1808.01089), a line holding only an
//! e-mail an author's e-mail (astro-ph0408475). Each guard converts its repro
//! under `tools/perfect_kernel/repros/sectioning-frontmatter/` from the binary, eager and `--streaming`, asserts the same
//! XML both ways, no Error and no Warning, and every creator (name and contacts in order). One guard per path: a
//! declaration environment, a declaration group (and bare declarations, and what stays wrapped), spacing after the
//! close (both unwrapping sites), the AASTeX group rule, its marked lines, line labels, unlabelled lines under marked
//! names, e-mail lines, and REVTeX address lines led by a macro mark (hep-ph0301204).
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
  both_ways(&nested);
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

/// A line's leading label goes to the names requesting it (review rev97cd F1; user rulings R1/R2, 2026-11-01): a
/// hand-printed number under `\altaffilmark{N}` names (`\affil{1 Uni One}`, 1810.01941; 97f r1 gave every author every
/// line, 39 -> 364 affiliations there), a `$^{N}$` mark under them (`\affil{$^{1}$Uni One}`, 1808.01089 and eleven
/// more), each entry of one `\affil` block of such lines, its `\email` the spelled author's (1610.06611, 1503.02933),
/// and a hand-printed number under `$^{N}$` names (`\affil{2) Uni Two}`, 0909.0058) — `leading_line_label`,
/// `queued_name_marks`, aas_support_sty.rs `\affiliation`.
#[test]
fn aastex_affil_line_labels_go_to_the_names_requesting_them() {
  let xml = both_ways(repro!("aastex_typed_affil_numbers_are_labels"));
  assert_eq!(creators(&xml), [
    vec!["Ann Able", "affiliation=Uni One"],
    vec!["Bob Baker", "affiliation=Uni One"],
    vec!["Cat Cole", "affiliation=Uni One", "affiliation=Uni Two"],
  ]);
  let xml = both_ways(repro!("aastex_altaffilmark_names_marked_affil_lines"));
  assert_eq!(creators(&xml), [
    vec!["Ann Able", "affiliation=Uni One"],
    vec!["Bob Baker", "affiliation=Uni Two", "affiliation=Uni Three"],
    vec!["Cat Cole", "affiliation=Uni Two"],
  ]);
  let xml = both_ways(repro!(
    "aastex_altaffilmark_names_one_affil_block_of_labelled_lines"
  ));
  assert_eq!(creators(&xml), [
    vec!["Ann Able", "affiliation=Uni One Dept X"],
    vec!["Bob Baker", "affiliation=Uni Two", "email=bbaker@uni.edu"],
    vec![
      "Cat Cole",
      "affiliation=Uni One Dept X",
      "affiliation=Uni Two"
    ],
  ]);
  let xml = both_ways(repro!("aastex_sup_names_typed_affil_numbers_are_labels"));
  assert_eq!(creators(&xml), [
    vec!["Ann Able 1", "affiliation=1 Uni One"],
    vec!["Bob Baker 1", "affiliation=1 Uni One"],
    vec![
      "Cat Cole 1,2",
      "affiliation=1 Uni One",
      "affiliation=Uni Two"
    ],
  ]);
}

/// Unlabelled lines under marked names (`relocate::run_names_show_a_mark`): `\altaffilmark` names request
/// `\altaffiltext`s, no affiliations, so the lines stay with the row (1210.4079), as under a footnote symbol alone
/// (`Ann Able$^{*}$`, 2403.06760's group); `$^{N}$` names request affiliations, so their run is no group —
/// `annotate=new`, 97d's result (1202.0629; 97f r1 31 -> 247 affiliations).
#[test]
fn aastex_unlabelled_lines_under_marked_names() {
  let xml = both_ways(repro!("aastex_altaffilmark_names_share_unlabelled_lines"));
  assert_eq!(creators(&xml), [
    vec![
      "Ann Able",
      "affiliation=Uni One",
      "affiliation=Main Street 5",
      "altaffiliation=Current address: Lab X",
    ],
    vec![
      "Bob Baker",
      "affiliation=Uni One",
      "affiliation=Main Street 5"
    ],
  ]);
  let xml = both_ways(repro!("aastex_symbol_marked_names_share_affil_lines"));
  assert_eq!(creators(&xml), [
    vec!["Ann Able *", "affiliation=Uni One", "affiliation=Uni Two"],
    vec!["Bob Baker", "affiliation=Uni One", "affiliation=Uni Two"],
  ]);
  let xml = both_ways(repro!(
    "aastex_sup_names_over_unlabelled_lines_are_no_group"
  ));
  assert_eq!(creators(&xml), [
    vec!["Ann Able 1", "affiliation=Uni One"],
    vec!["Bob Baker 1", "affiliation=Uni One"],
    vec!["Cat Cole 1,2", "affiliation=Uni One", "affiliation=Uni Two"],
  ]);
}

/// An `\affil` holding only an e-mail is an e-mail, not an affiliation (review rev97cd F3): unmarked, the author's it
/// spells (`\affil{e-mail: kangx@uni.edu}`, astro-ph0408475; 97f r1 an affiliation of every author), marked, the
/// author's requesting the mark (`\affil{$^*$ Email: bbaker@uni.edu}` under `Bob Baker\altaffilmark{2,*}`, 1808.01089);
/// the label is the contact's name, as the API's Kind step names an e-mail line's contacts (`named_email_calls`, the
/// label and a tie: Z18l's reroute, OD #992, the binding's own `name={label }` gone).
#[test]
fn aastex_affil_holding_only_an_email_is_an_authors_email() {
  let xml = both_ways(repro!("aastex_affil_email_line_is_its_authors_email"));
  assert_eq!(creators(&xml), [
    vec!["Xu Kang 1", "affiliation=Uni One", "email=kangx@uni.edu"],
    vec!["Yi Jing 1", "affiliation=Uni One"],
    vec!["Hu Mo 2", "affiliation=Uni Two"],
  ]);
  assert_eq!(
    xml
      .matches("<contact name=\"e-mail:\u{a0}\" role=\"email\">kangx@uni.edu</contact>")
      .count(),
    1,
    "{xml}"
  );
  let xml = both_ways(repro!(
    "aastex_affil_marked_email_line_is_the_marked_authors"
  ));
  assert_eq!(creators(&xml), [
    vec!["Ann Able", "affiliation=Uni One"],
    vec!["Bob Baker", "affiliation=Uni Two", "email=bbaker@uni.edu"],
  ]);
  assert_eq!(
    xml
      .matches("<contact name=\"Email:\u{a0}\" role=\"email\">bbaker@uni.edu</contact>")
      .count(),
    1,
    "{xml}"
  );
}

/// REVTeX `\address` lines led by a macro standing for a mark (`\edef\UA{$^{1}$ }`, `\address{\UA Uni One}`) under
/// names carrying such macros (`leading_macro_mark`, `run_names_macro_marks`; user ruling R1): written after the
/// commas, the marks land on the next names, so the lines are the block's, once (hep-ph0301204: 213 names had all 38
/// addresses each, 8,094 affiliations); written after the names, each line goes to the names its mark labels.
#[test]
fn revtex_macro_marked_address_lines_are_no_group() {
  let xml = both_ways(repro!(
    "revtex_macro_marked_addresses_after_commas_go_to_the_block"
  ));
  let all = creators(&xml);
  assert_eq!(all.last().unwrap(), &[
    "affiliation=Uni One",
    "affiliation=Uni Two"
  ]);
  assert_eq!(
    all
      .iter()
      .flatten()
      .filter(|c| c.starts_with("affiliation="))
      .count(),
    2,
    "{all:?}"
  );
  both_ways(repro!("revtex_macro_marked_addresses_go_by_their_marks"));
}
