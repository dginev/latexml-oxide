//! Red/green guards for batch 99b: a raw class's affiliation or address that its own title page prints into the body
//! is the creators' contact (OXIDIZED_DESIGN_DIVERGENCES #955; FIDELITY_CHARTER §4, displaced text moves), and the
//! unclaimed-text net's review items (rev98c S1-S3, OD #953). Code paths: a table setter that stores its argument for
//! the title page with code of its own around it, recorded by a meaning recorder and routed by the title page's
//! speculative run, its body copy taken out by provenance (latexml_engine/src/frontmatter_stores.rs
//! `reroute_displaced_setters`, `route_and_hand_displaced`; a static store, 2607.20672, and a computed numbered one,
//! 2510.16758); an in-place setter with no line break of its own (`run_in_contact_setter`, iopjournal 2608.28486); the
//! net's begin-code notes, waiting for the environment (`hand_net_begin_code_stores`); a register a class's
//! begin-document hook fills (`note_document_hooks`); a `\label` in a note's store (`without_labels`). Round 2 (review
//! rev99b): each routed line placed by its leading label (`contact_line_call`), a store's `\and` list read by the
//! class's numbering or left the class's, the rerouted setters protected, the recorder nesting. Each guard
//! converts its repro under `tools/perfect_kernel/repros/sectioning-frontmatter/` from the binary, eager and
//! `--streaming`, and asserts the frontmatter it makes whole.
use super::{
  perfect_kernel_batch46::{error_count, warning_count},
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

/// Converts `tex` from the binary eager and `--streaming`, checks each has no Error and `warnings` Warnings, all of
/// them the net's `unclaimed`, and that both give the same XML; returns it, each whitespace run one space, from its
/// `<title>` to its end.
#[track_caller]
pub(super) fn converted(tex: &str, warnings: usize) -> String {
  let mut xmls = Vec::new();
  for streaming in [false, true] {
    let (log, xml) = cli_conversion(tex, streaming);
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, warnings),
      "{log}"
    );
    assert_eq!(
      log.matches("Warning:frontmatter:unclaimed").count(),
      warnings,
      "{log}"
    );
    xmls.push(xml);
  }
  assert_eq!(xmls[0], xmls[1]);
  let xml = xmls[0].split_whitespace().collect::<Vec<_>>().join(" ");
  let start = xml.find("<title>").unwrap_or(0);
  let end = xml[start..]
    .find("</document>")
    .map_or(xml.len(), |end| start + end);
  xml[start..end].trim_end().to_string()
}

/// A static store with code of the class's own around the argument (rstransa.cls:1154 `\gdef\@address{#1\ifaffbreak
/// …\fi}`), printed by the class's `\@maketitle`: each `\\` line goes by its leading mark to the author requesting it,
/// and the deposit no longer prints them below the names. A store the title page prints only in a running head moves
/// nowhere. Witnesses 2607.20672, 2609.39929, nlin0310001.
#[test]
fn a_marked_address_store_the_class_title_page_prints_is_each_authors_line() {
  assert_eq!(
    converted(
      repro!("displaced_marked_address_store_printed_by_the_class_title_page"),
      0
    ),
    r#"<title>A Title</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Address: " role="address"><sup xml:id="id1"><text font="italic" xml:id="id1.1">1</text></sup>Univ A, Town</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Address: " role="address"><sup xml:id="id2"><text font="italic" xml:id="id2.1">2</text></sup>Univ B, City</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#
  );
  assert_eq!(
    converted(
      repro!("displaced_store_printed_only_in_a_running_head_stays"),
      0
    ),
    r#"<title>A Title</title> <creator role="author"> <personname>Ann Able</personname> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#
  );
}

/// A store whose name the setter computes (iau.cls:1515 `\expandafter\protected@xdef\csname affil\the\affilcount
/// \endcsname{#1}`), found by the journal around the call: the unmarked affiliation is the sole author's. Witnesses
/// 2510.16758, 2607.25849.
#[test]
fn a_numbered_affiliation_store_is_its_sole_authors() {
  assert_eq!(
    converted(
      repro!("displaced_numbered_affiliation_store_under_one_author"),
      0
    ),
    r#"<title>A Title</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Department of Physics, University of Aberdeen, Scotland</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#
  );
}

/// A displaced store whose one line holds several marked entries the names request (`\institute{$^{1}$Univ A, $^{2}$Univ
/// B}` run in) lands once at the block, `_unsplit`, which the frontmatter observer reads by its marks: never the whole
/// list copied onto each author (1812.07638's 64 institutes under 200 names).
#[test]
fn a_run_in_institute_store_holding_several_marks_is_cut_at_the_block() {
  assert_eq!(
    converted(
      repro!("displaced_run_in_institutes_are_cut_at_the_block"),
      0
    ),
    r#"<title>A Title</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation"><sup xml:id="id1"><text font="italic" xml:id="id1.1">1</text></sup>Univ A, Town</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation"><sup xml:id="id2"><text font="italic" xml:id="id2.1">2</text></sup>Univ B, City</contact> </creator> <creator before=" " role="author"> <personname>Cy Cole</personname> <contact name="Affiliation: " role="affiliation"><sup xml:id="id3"><text font="italic" xml:id="id3.1">1</text></sup>Univ A, Town</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#
  );
}

/// An in-place `\affil` with no line break of its own (iopjournal.cls:108), a paragraph by the document's blank lines:
/// each line its marked author's. Witnesses 2608.28486, 2512.15449, 2602.03759, 2606.14342, 2609.32125.
#[test]
fn a_run_in_affiliation_setter_line_is_its_marked_authors() {
  assert_eq!(
    converted(repro!("run_in_affiliation_setter_without_a_line_break"), 0),
    r#"<title>A Title</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation"><sup xml:id="id1">1</sup>Univ A, Town</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation"><sup xml:id="id2">2</sup>Univ B, City</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#
  );
}

/// Review rev98c S1: a store only the class's refused `{abstract}` begin code prints is no note when the document never
/// opens the environment, nor when it opens it before `\maketitle` set the store (the opened-after case is
/// `perfect_kernel_batch98c::a_title_note_register_only_the_class_abstract_prints_is_a_note`).
#[test]
fn a_begin_code_note_waits_for_the_document_to_open_the_environment() {
  assert_eq!(
    converted(
      repro!("net_title_note_register_without_the_abstract_is_no_note"),
      0
    ),
    r#"<title>A Title<sup xml:id="id1"><text font="italic" xml:id="id1.1">∗</text></sup></title> <creator role="author"> <personname>Ann Able</personname> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text without any abstract.</p> </para>"#
  );
  assert_eq!(
    converted(
      repro!("net_title_note_register_abstract_before_maketitle_is_no_note"),
      0
    ),
    r#"<title>A Title<sup xml:id="id1"><text font="italic" xml:id="id1.1">∗</text></sup></title> <creator role="author"> <personname>Ann Able</personname> </creator> <abstract inlist="toc" name="Abstract" xml:id="abstract1"> <p xml:id="abstract1.1">An abstract.</p> </abstract> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#
  );
}

/// Review rev98c S2: a token register the class's own `\AtBeginDocument` hook fills holds the class's text, a default:
/// no note.
#[test]
fn a_register_a_class_begin_document_hook_fills_is_no_note() {
  assert_eq!(
    converted(
      repro!("net_register_a_class_begin_document_hook_fills_is_no_note"),
      0
    ),
    r#"<title>A Title</title> <creator role="author"> <personname>Ann Able</personname> </creator> <abstract inlist="toc" name="Abstract" xml:id="abstract1"> <p xml:id="abstract1.1">An abstract.</p> </abstract> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#
  );
}

/// Review rev98c S3: a `\label` in a store the net hands on asks for no annotation: one Warning, the net's, and the
/// block's note holds both affiliations.
#[test]
fn a_label_in_a_net_store_is_no_orphaned_annotation() {
  assert_eq!(
    converted(repro!("net_store_label_is_no_orphaned_annotation"), 1),
    r#"<title>A Title</title> <creator role="author"> <personname>Ann Able</personname> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> </creator> <creator role="author"> <note role="note" xml:id="id1"><sup xml:id="id1.1"><text font="italic" xml:id="id1.1.1">1</text></sup>Institute of Risk and Disaster Reduction, University College London<break/><sup xml:id="id1.2"><text font="italic" xml:id="id1.2.1">2</text></sup>Department of Statistical Science, Imperial College</note> </creator> <pubnote role="thanks">Visiting the Lunar Geology Institute in Tycho.</pubnote> <pubnote>Formerly at Oxford Brookes Polytechnic.</pubnote> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#
  );
}

/// Review rev99b B1 (round 2): a run-in or displaced line led by a printed label a name requests is that label's
/// (`label={affiliation:<key>}`, frontmatter_stores.rs `contact_line_call`), so the kernel pairs it with the names
/// requesting it even when the frontmatter observer declines to re-part them: lettered lines under comma-list marks, the
/// observer declining over the ORCID links (2512.20289), and numbered lines under one creator holding two names and a
/// collaboration, whose own mark's line stays the class's text (2609.15125); and the in-place route's same lines
/// (`by_its_label`). Witnesses 1603.07668, 1704.03333 too.
#[test]
fn a_run_in_line_led_by_a_requested_mark_is_its_names_when_the_observer_declines() {
  assert_eq!(
    converted(
      repro!("run_in_marked_lines_go_by_their_marks_when_the_observer_declines"),
      0
    ),
    r#"<title>A Title</title> <creator role="author"> <personname>Ann Able<ref class="ltx_href" href="https://orcid.org/https://orcid.org/0009-0009-1414-288X">(iD)</ref><sup xml:id="id1">1</sup></personname> <contact name="Affiliation: " role="affiliation"><sup xml:id="id2"><text font="italic" xml:id="id2.1">a</text></sup> Univ A, Town</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker<ref class="ltx_href" href="https://orcid.org/https://orcid.org/0000-0003-1431-2884">(iD)</ref><sup xml:id="id3">2</sup></personname> <contact name="Affiliation: " role="affiliation"><sup xml:id="id4"><text font="italic" xml:id="id4.1">b</text></sup> Univ B, City</contact> </creator> <creator before=" " role="author"> <personname>Cy Cole<ref class="ltx_href" href="https://orcid.org/https://orcid.org/0000-0002-6069-9031">(iD)</ref><sup xml:id="id5">3</sup></personname> <contact name="Affiliation: " role="affiliation"><sup xml:id="id6"><text font="italic" xml:id="id6.1">a</text></sup> Univ A, Town</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#
  );
  assert_eq!(
    converted(
      repro!("run_in_numbered_lines_under_a_collaboration_go_by_their_marks"),
      0
    ),
    r#"<title>A Title</title> <creator role="author"> <personname>Ann Able<ref class="ltx_href" href="https://orcid.org/0000-0000-0000-0000">(iD)</ref><sup xml:id="id1">∗</sup></personname> <contact role="collaboration">the XY team</contact> <contact name="Affiliation: " role="affiliation"><sup xml:id="id2">1</sup>Univ A, Town</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker<ref class="ltx_href" href="https://orcid.org/0000-0000-0000-0000">(iD)</ref> Dan Dole<ref class="ltx_href" href="https://orcid.org/0000-0000-0000-0000">(iD)</ref></personname> <contact role="collaboration">the XY team</contact> <contact name="Affiliation: " role="affiliation"><sup xml:id="id3">2</sup>Univ B, City</contact> </creator> <creator before=" " role="author"> <personname>Cy Cole</personname> <contact role="collaboration">the XY team</contact> <contact name="Affiliation: " role="affiliation"><sup xml:id="id4">3</sup>Univ C, Place</contact> </creator> <para class="ltx_align_left" xml:id="p1"> <p xml:id="p1.1"><sup xml:id="p1.1.1"><text fontsize="90%" xml:id="p1.1.1.1">4</text></sup><text fontsize="90%" xml:id="p1.1.2">See the author list of the XY team</text></p> </para> <para xml:id="p2"> <p xml:id="p2.1">Text.</p> </para>"#
  );
  // (the in-place route's sibling, `\lx@class@contact`: the same lines under a setter that breaks its own line, every
  // author's in the 99b0 base too)
  assert_eq!(
    converted(
      repro!("in_place_marked_lines_go_by_their_marks_when_the_observer_declines"),
      0
    ),
    r#"<title>A Title</title> <creator role="author"> <personname>Ann Able<ref class="ltx_href" href="https://orcid.org/https://orcid.org/0009-0009-1414-288X">(iD)</ref><sup xml:id="id1">1</sup></personname> <contact name="Affiliation: " role="affiliation"><sup xml:id="id2"><text font="italic" xml:id="id2.1">a</text></sup> Univ A, Town</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker<ref class="ltx_href" href="https://orcid.org/https://orcid.org/0000-0003-1431-2884">(iD)</ref><sup xml:id="id3">2</sup></personname> <contact name="Affiliation: " role="affiliation"><sup xml:id="id4"><text font="italic" xml:id="id4.1">b</text></sup> Univ B, City</contact> </creator> <creator before=" " role="author"> <personname>Cy Cole<ref class="ltx_href" href="https://orcid.org/https://orcid.org/0000-0002-6069-9031">(iD)</ref><sup xml:id="id5">3</sup></personname> <contact name="Affiliation: " role="affiliation"><sup xml:id="id6"><text font="italic" xml:id="id6.1">a</text></sup> Univ A, Town</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#
  );
  // (a legend its symbol labels is its carrier's note, not an affiliation: `legend_note_kind`; 2609.15125, 2602.03759,
  // 2604.08878, 2607.12158)
  assert_eq!(
    converted(repro!("run_in_legend_line_is_a_note_of_its_carrier"), 0),
    r#"<title>A Title</title> <creator role="author"> <personname>Ann Able<ref class="ltx_href" href="https://orcid.org/0000-0000-0000-0000">(iD)</ref></personname> <contact role="collaboration">the XY team</contact> <contact name="Affiliation: " role="affiliation"><sup xml:id="id1">1</sup>Univ A, Town</contact> <note class="ltx_note_frontmatter ltx_thanks_correspondence" mark="*" role="thanks" xml:id="id2"><sup xml:id="id2.1">∗</sup>Author to whom any correspondence should be addressed.</note> </creator> <creator before=" " role="author"> <personname>Bob Baker<ref class="ltx_href" href="https://orcid.org/0000-0000-0000-0000">(iD)</ref> Dan Dole<ref class="ltx_href" href="https://orcid.org/0000-0000-0000-0000">(iD)</ref></personname> <contact role="collaboration">the XY team</contact> <contact name="Affiliation: " role="affiliation"><sup xml:id="id3">2</sup>Univ B, City</contact> </creator> <creator before=" " role="author"> <personname>Cy Cole</personname> <contact role="collaboration">the XY team</contact> <contact name="Affiliation: " role="affiliation"><sup xml:id="id4">3</sup>Univ C, Place</contact> </creator> <para class="ltx_align_left" xml:id="p1"> <p xml:id="p1.1"><sup xml:id="p1.1.1"><text fontsize="90%" xml:id="p1.1.1.1">4</text></sup><text fontsize="90%" xml:id="p1.1.2">See the author list of the XY team</text></p> </para> <para xml:id="p2"> <p xml:id="p2.1">Text.</p> </para>"#
  );
}

/// Review rev99b B2 (round 2): a displaced store's `\and` list is the class's entry markup. Numbered by the class's
/// title page (caosp.sty:519-527 `\institutename`, `class_numbered_institutes`), entry n is labelled n and goes to the
/// names requesting it, with no " and " written in (astro-ph9805069); not numbered — or its lines numbered by a separator
/// the title page defines anew where it prints the store (`ClassReaders::separators_redefined`) — the call is in doubt
/// and stays the class's text (1704.03333's sifrev).
#[test]
fn an_and_list_in_a_displaced_store_goes_by_the_class_numbers_or_stays_the_classs() {
  assert_eq!(
    converted(
      repro!("displaced_class_numbered_and_institutes_go_by_their_numbers"),
      0
    ),
    r#"<title>A Title</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation"><sup xml:id="id1">1</sup>Univ A, Town</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation"><sup xml:id="id2">2</sup>Univ B, City</contact> </creator> <creator before=" " role="author"> <personname>Cy Cole</personname> <contact name="Affiliation: " role="affiliation"><sup xml:id="id3">1</sup>Univ A, Town</contact> <contact name="Affiliation: " role="affiliation"><sup xml:id="id4">2</sup>Univ B, City</contact> </creator> <para class="ltx_noindent" xml:id="p1"> <p xml:id="p1.1"><sup xml:id="p1.1.1"><text font="italic" fontsize="90%" xml:id="p1.1.1.1">1</text></sup></p> </para> <para xml:id="p2"> <p xml:id="p2.1">Text.</p> </para>"#
  );
  assert_eq!(
    converted(
      repro!("displaced_and_institutes_the_class_does_not_number_stay_the_classs"),
      0
    ),
    r#"<title>A Title</title> <creator role="author"> <personname>Ann Able</personname> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> </creator> <para class="ltx_noindent" xml:id="p1"> <p xml:id="p1.1"><text font="italic" fontsize="90%" xml:id="p1.1.1">Univ A, Town</text></p> </para> <para class="ltx_noindent" xml:id="p2"> <p xml:id="p2.1"><text font="italic" fontsize="90%" xml:id="p2.1.1">Univ B, City</text></p> </para> <para xml:id="p3"> <p xml:id="p3.1">Text.</p> </para>"#
  );
  assert_eq!(
    converted(
      repro!("displaced_lines_a_title_page_separator_numbers_stay_the_classs"),
      0
    ),
    r#"<title>A Title</title> <creator role="author"> <personname>Ann Able<sup xml:id="id1">1</sup></personname> </creator> <creator before=" " role="author"> <personname>Bob Baker<sup xml:id="id2">2</sup></personname> </creator> <para class="ltx_noindent" xml:id="p1"> <p xml:id="p1.1"><sup xml:id="p1.1.1"><text font="italic" fontsize="90%" xml:id="p1.1.1.1">1</text></sup><text fontsize="90%" xml:id="p1.1.2"> </text><text font="italic" fontsize="90%" xml:id="p1.1.3">Univ A, Town</text></p> </para> <para class="ltx_noindent" xml:id="p2"> <p xml:id="p2.1"><sup xml:id="p2.1.1"><text font="italic" fontsize="90%" xml:id="p2.1.1.1">2</text></sup><text font="italic" fontsize="90%" xml:id="p2.1.2"> Univ B, City</text></p> </para> <para xml:id="p3"> <p xml:id="p3.1">Text.</p> </para>"#
  );
}

/// Review rev99b B3 (round 2): a rerouted setter called inside a class's `\protected@xdef` store (iau.cls:1516's
/// numbered `\affiliation`, its inline `\email`, 2607.25849) stays in the store as written — the setters and their
/// deciders are protected —, and the displaced call holding it is in doubt, so the e-mail is the class's text where
/// its title page prints it, never an API call baked into the store that gave the address to the last creator. The
/// other call's line goes by its mark (B1): Bob Baker, mark 1 only, holds none of it.
#[test]
fn a_rerouted_setter_inside_a_class_xdef_store_decides_where_it_is_digested() {
  assert_eq!(
    converted(
      repro!("run_in_email_inside_a_class_xdef_store_stays_the_classs"),
      0
    ),
    r#"<title>A Title</title> <creator role="author"> <personname>Ann Able<sup xml:id="id1">1</sup></personname> <contact name="Affiliation: " role="affiliation"><sup xml:id="id2">2</sup>Univ B, City</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker<sup xml:id="id3">1</sup></personname> </creator> <para xml:id="p1"> <p align="center" xml:id="p1.1"><text fontsize="90%" xml:id="p1.1.1">email: <ref class="ltx_href" href="mailto:info@univ-a.edu">info@univ-a.edu</ref><break/><sup xml:id="p1.1.1.1">1</sup>Univ A, Town</text></p> </para> <para xml:id="p2"> <p xml:id="p2.1">Text.</p> </para>"#
  );
}

/// Review rev99b B4 (round 2): a displaced setter whose own code prints the name's mark (utarticle.cls:125-137
/// `\address` → `\@makeadmark{\@tempb}`, 1209.6020) is recorded by a meaning recorder that nests, not an assignment
/// journal, so the mark's speculative read still opens one: the names keep their printed marks "a" and "b".
#[test]
fn a_displaced_setter_printing_a_mark_keeps_the_names_marks() {
  assert_eq!(
    converted(
      repro!("displaced_setter_printing_a_mark_keeps_the_names_marks"),
      0
    ),
    r#"<title>A Title</title> <creator role="author"> <personname>Ann Able <text xml:id="id1"/><sup xml:id="id2">a</sup></personname> </creator> <creator before=" " role="author"> <personname>Bob Baker <text xml:id="id3"/><sup xml:id="id4">b</sup></personname> </creator> <creator before=" " role="author"> <personname>Cy Cole<sup xml:id="id5">a</sup></personname> </creator> <logical-block xml:id="id6"> <para xml:id="p1"> <p align="center" vattach="top" xml:id="p1.1"><text font="italic" fontsize="90%" xml:id="p1.1.1"><sup xml:id="p1.1.1.1">a</sup></text><text font="italic" fontsize="90%" xml:id="p1.1.2">Univ A, Town</text></p> <p align="center" vattach="top" xml:id="p1.2"><text font="italic" fontsize="90%" xml:id="p1.2.1"><sup xml:id="p1.2.1.1">b</sup></text><text font="italic" fontsize="90%" xml:id="p1.2.2">Univ B, City</text></p> </para> </logical-block> <para xml:id="p2"> <p xml:id="p2.1">Text.</p> </para>"#
  );
}
