//! Guards for ERR2 (SYNC row 0b item 39, OXIDIZED_DESIGN_DIVERGENCES #983): an author block's undefined commands, whose
//! Errors the observer's rehearsal holds and discards with their stubs, are raised again — once, as Perl's one
//! digestion of the block raises them (State.pm:547-551, Base_Utility.pool.ltxml:691-693). In place where a piece the
//! emitter digests holds the command: right after a name (`content::Field::past_held_undefined`), past its marks and
//! notes (`emit::block_names`), and where it glues a word's ink onto an address's local part — the same word has ink on
//! both sides of it before its `@`, "Bob|b@" —, which no reading of addresses takes (`events::Event::glued_undef`; a Name
//! the ink table cannot align fails closed on its span). Elsewhere the address is read as without the Error: an
//! undefined command never changes the segmentation. Else at the flush's end
//! (`reading::HeldUndefined::raise_unraised`), for a piece written as text. Each guard asserts every creator's name and
//! contacts, in order, and the exact Error and Warning counts; witnesses 1101.4098, 1402.2906, 1808.09392, 2205.12847,
//! 2309.09045, 2405.06973, 2405.13308, 2506.10896, 2508.08170, 2508.21091, 2601.17318, 2602.18351, 2604.26688,
//! 2607.29185, 2609.00161, hep-th9306034.

use super::{
  perfect_kernel_batch46::{convert, error_count, warning_count},
  perfect_kernel_batch84b::creators,
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

fn row(items: &[&str]) -> Vec<String> { items.iter().map(ToString::to_string).collect() }

/// Each `<creator>` element of `xml`, whole, in order.
fn creator_elements(xml: &str) -> Vec<&str> {
  xml
    .match_indices("<creator ")
    .map(|(at, _)| {
      let end = at + xml[at..].find("</creator>").expect("a closed creator") + "</creator>".len();
      &xml[at..end]
    })
    .collect()
}

/// The undefined command names of the log's Errors, in order.
fn undefined_errors(log: &str) -> Vec<&str> {
  log
    .lines()
    .filter_map(|line| line.strip_prefix("Error:undefined:"))
    .filter_map(|rest| rest.split(' ').next())
    .collect()
}

/// A piece written as text (a name its events don't spell, counted by its Warning) never digests its tokens: the Error
/// its rehearsal held is raised by the flush's net, once (`reading::HeldUndefined`; 2609.00161's `\aaai@corrmultitrue`).
/// Distinct: the one path where no piece raises it in place.
#[test]
fn an_undefined_command_in_a_text_written_name_errs_once_at_the_flush() {
  let (log, xml) = convert(
    repro!("emitter_undefined_command_in_a_name_written_as_text_errs_once"),
    false,
  );
  assert_eq!(undefined_errors(&log), ["\\undefinedprobecs"], "{log}");
  assert_eq!((error_count(&log), warning_count(&log)), (1, 1), "{log}");
  assert!(
    log.contains("The frontmatter emitter writes \"Ann Able\" as its plain text"),
    "{log}"
  );
  assert_eq!(creators(&xml), [row(&["Ann Able"]), row(&["Bob Baker"])]);
}

/// ERR1's synthetic, Perl's two creators and two Errors: `\undefinedprobecs` right after Ann's name is the name's, its
/// ERROR in her personname; `\email` (undefined in article) glues "Bob" and "b@x.org", which stays Bob's name — never Ann's
/// e-mail "Bobb@x.org" (the base's one creator). And a command inside a name (`Bob Baker\undefinedprobecs\ Jr.`), raised in
/// place as before, whose ERROR text the content check reads as no word of the creator's (the base's Warning). Distinct:
/// the plain gap and the glued address, the segmentation unchanged.
#[test]
fn an_undefined_command_in_author_errs_in_place_and_never_resegments() {
  let (log, xml) = convert(
    repro!("emitter_undefined_command_in_author_errs_once"),
    false,
  );
  assert_eq!(
    undefined_errors(&log),
    ["\\undefinedprobecs", "\\email"],
    "{log}"
  );
  assert_eq!((error_count(&log), warning_count(&log)), (2, 0), "{log}");
  assert_eq!(creators(&xml), [
    row(&["Ann Able \\undefinedprobecs"]),
    row(&["Bob \\email b@x.org"]),
  ]);
  let bob = xml
    .find("<personname>Bob<ERROR class=\"undefined\"")
    .expect("Bob's ERROR in his personname");
  let close = bob + xml[bob..].find("</personname>").expect("closed personname");
  assert!(xml[bob..close].ends_with("\\email</ERROR>b@x.org"), "{xml}");
  let (log, xml) = convert(
    "\\documentclass{article}\n\\author{Ann Able \\and Bob Baker\\undefinedprobecs\\ Jr.}\n\\title{T}\n\
     \\begin{document}\n\\maketitle\n\\end{document}\n",
    false,
  );
  assert_eq!((error_count(&log), warning_count(&log)), (1, 0), "{log}");
  assert_eq!(creators(&xml), [
    row(&["Ann Able"]),
    row(&["Bob Baker \\undefinedprobecs Jr."]),
  ]);
}

/// An undefined command past a name's mark or its `\thanks`, before the next separator: no event covers it and the name's
/// run stops at the mark or the note; it is written after the name, its Error raised in the personname
/// (`emit::block_names`; 2607.29185's `\internship`, 2508.21091's `\equalcorres`, 1101.4098's `\ETC`). Distinct: the
/// scan past what the reading writes apart.
#[test]
fn an_undefined_command_past_a_mark_or_a_note_errs_in_the_name() {
  let (log, xml) = convert(
    repro!("emitter_undefined_command_after_a_mark_errs_in_the_name"),
    false,
  );
  assert_eq!(undefined_errors(&log), ["\\undefinedprobecs"], "{log}");
  assert_eq!((error_count(&log), warning_count(&log)), (1, 0), "{log}");
  assert_eq!(creators(&xml), [
    row(&["Ann Able \\undefinedprobecs", "affiliation=Univ A"]),
    row(&["Bob Baker", "affiliation=Univ B"]),
  ]);
  let (log, xml) = convert(
    repro!("emitter_undefined_command_after_a_thanks_errs_in_the_name"),
    false,
  );
  assert_eq!(undefined_errors(&log), ["\\undefinedprobecs"], "{log}");
  assert_eq!((error_count(&log), warning_count(&log)), (1, 0), "{log}");
  assert_eq!(creators(&xml), [
    row(&["Ann Able \\undefinedprobecs"]),
    row(&["Bob Baker"]),
  ]);
  assert!(xml.contains("Supported by X."), "{xml}");
}

/// An undefined command right after a `\footnotemark` the reading writes apart from the name: the mark's piece grows over
/// it (`content::Field::past_held_undefined`), so the name's scan past its marks (`emit::block_names`) counts the mark's
/// run to where its piece's tokens end and takes it no more — one Error, one ERROR stub (review revERR2c finding 1: ERR2
/// wrote two stubs for its one Error; the user's ruling "never twice"; Perl and pdflatex one). With the mark's option and
/// an address after it, the same. Distinct: the apart mark's grown piece, which the scan past it must see.
#[test]
fn an_undefined_command_after_a_footnotemark_writes_one_stub() {
  let (log, xml) = convert(
    repro!("emitter_undefined_command_after_a_footnotemark_errs_once"),
    false,
  );
  assert_eq!(undefined_errors(&log), ["\\foo"], "{log}");
  assert_eq!((error_count(&log), warning_count(&log)), (1, 0), "{log}");
  assert_eq!(creator_elements(&xml), [
    "<creator role=\"author\">\n    <personname>Ann Able<note mark=\"*\" role=\"footnotemark\" \
     xml:id=\"footnote1\"><tags>\n          <tag>*</tag>\n          <tag role=\"refnum\">*</tag>\n          <tag \
     role=\"typerefnum\">footnote *</tag>\n        </tags></note><ERROR \
     class=\"undefined\">\\foo</ERROR></personname>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Bob Baker</personname>\n  </creator>",
  ]);
  let (log, xml) = convert(
    "\\documentclass{article}\n\\author{Ann Able\\footnotemark[2]\\foo{} ann@x.org}\\title{T}\n\
     \\begin{document}\\maketitle\\end{document}\n",
    false,
  );
  assert_eq!(undefined_errors(&log), ["\\foo"], "{log}");
  assert_eq!((error_count(&log), warning_count(&log)), (1, 0), "{log}");
  assert_eq!(creator_elements(&xml), [
    "<creator role=\"author\">\n    <personname>Ann Able<note mark=\"\u{2020}\" role=\"footnotemark\" \
     xml:id=\"footnotex1\"><tags>\n          <tag>\u{2020}</tag>\n          <tag role=\"refnum\">\u{2020}</tag>\n          \
     <tag role=\"typerefnum\">footnote \u{2020}</tag>\n        </tags></note><ERROR \
     class=\"undefined\">\\foo</ERROR></personname>\n    <contact name=\"Email:\u{a0}\" role=\"email\">ann@x.org</contact>\n  \
     </creator>",
  ]);
}

/// The flip A/B's lost `Error:expected:\fi` chains (1603.08231: 18 off, 2112.06251: 12) are no Errors of the paper's: an
/// `\and` inside a conditional (sibarticle.sty's `\ifnum \c@numAuthors>1 \and …\fi`) is skipped with the name after it,
/// as TeX reads the block in one pass (pdflatex 0 errors, "Ann Able" alone); Perl's split at the `\and` tokens cuts each
/// conditional from its `\fi` (KNOWN_PERL_ERRORS #1218: 1 Error and a creator Bob Baker pdflatex never prints). Distinct:
/// the classification's evidence, a conditional across the block's separators.
#[test]
fn an_and_inside_a_conditional_is_read_as_tex_reads_it() {
  let (log, xml) = convert(
    repro!("author_and_inside_a_conditional_reads_as_tex"),
    false,
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(creators(&xml), [row(&["Ann Able"])]);
}

/// An undefined command stops an address being read only where the same word has ink on both sides of it before its `@`
/// (`events::glues_an_address`; the synthetic's "Bob|b@x.org" above). Past the `@`, after the word, inside its bracket,
/// or with the whole local part before it (`ann\foo@x.org`) it glues no name into an address: the reading is the one
/// without the Error, its 1 Error raised by the flush's net, its Warnings the base's (review revERR2 finding 1: ERR2 read
/// a third creator, or folded the e-mail into the name; revERR2b finding 3: ERR2b's third creator "ann \foo @x.org").
/// Distinct: the narrowed rule's negatives, one code path.
#[test]
fn an_undefined_command_outside_an_address_local_part_keeps_the_address() {
  let (log, xml) = convert(
    repro!("emitter_undefined_command_past_an_address_domain_keeps_the_address"),
    false,
  );
  assert_eq!(undefined_errors(&log), ["\\IEEEmembership"], "{log}");
  assert_eq!((error_count(&log), warning_count(&log)), (1, 1), "{log}");
  assert_eq!(creators(&xml), [
    row(&["Ann Able", "email=ann@x.orgMember"]),
    row(&["Bob Baker", "email=bob@y.org"]),
  ]);
  let (log, xml) = convert(
    repro!("emitter_undefined_command_after_an_address_word_keeps_the_address"),
    false,
  );
  assert_eq!(undefined_errors(&log), ["\\foo"], "{log}");
  assert_eq!((error_count(&log), warning_count(&log)), (1, 2), "{log}");
  assert_eq!(creators(&xml), [row(&["Ann Able", "email=ann@x.org"])]);
  assert_eq!(creator_elements(&xml), [
    "<creator role=\"author\">\n    <personname>Ann Able</personname>\n    <contact name=\"Email:\u{a0}\" \
     role=\"email\">ann@x.org</contact>\n    <note role=\"note\">Univ X</note>\n  </creator>"
  ]);
  let (log, xml) = convert(
    repro!("emitter_undefined_command_in_a_bracketed_address_keeps_the_address"),
    false,
  );
  assert_eq!(undefined_errors(&log), ["\\foo"], "{log}");
  assert_eq!((error_count(&log), warning_count(&log)), (1, 2), "{log}");
  assert_eq!(creators(&xml), [
    row(&["Ann Able", "email=ann@x.org"]),
    row(&["Bob Baker", "email=bob@y.org"]),
  ]);
  let (log, xml) = convert(
    repro!("emitter_undefined_command_before_an_address_at_keeps_the_address"),
    false,
  );
  assert_eq!(undefined_errors(&log), ["\\foo"], "{log}");
  assert_eq!((error_count(&log), warning_count(&log)), (1, 0), "{log}");
  assert_eq!(creators(&xml), [
    row(&["Ann Able", "email=ann@x.org"]),
    row(&["Bob Baker", "email=bob@y.org"]),
  ]);
}

/// A Name the ink table cannot align — a mark glued into it: "Bob Baker1b@x.org", a text no ink record has — fails closed
/// on its span (`events::glues_an_address`): the Error strictly inside it with an `@` token past it declines the address,
/// so the name keeps its word, the mark, the ERROR and the local part, Perl's reading (review revERR2b finding 2: ERR2b
/// made up Bob's e-mail "Baker1b@x.org", and Ann's "Bob1b@x.org" for a one-word name). Its 2 Warnings are the content
/// check's: the block's word "baker1b@x.org" holds the mark, the personname's words do not. Distinct: the fallback when
/// the ink is missing, both name lengths.
#[test]
fn an_undefined_command_after_a_glued_mark_declines_the_address() {
  let (log, xml) = convert(
    repro!("emitter_undefined_command_after_a_glued_mark_declines_the_address"),
    false,
  );
  assert_eq!(undefined_errors(&log), ["\\foo"], "{log}");
  assert_eq!((error_count(&log), warning_count(&log)), (1, 2), "{log}");
  assert_eq!(creators(&xml), [
    row(&["Ann Able"]),
    row(&["Bob Baker 1 \\foo b@x.org"]),
  ]);
  assert_eq!(
    creator_elements(&xml)[1],
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Bob Baker<sup>1</sup><ERROR \
     class=\"undefined\">\\foo</ERROR>b@x.org</personname>\n  </creator>"
  );
  let (log, xml) = convert(
    "\\documentclass{article}\n\\author{Ann Able \\and Bob\\textsuperscript{1}\\foo b@x.org}\n\\title{T}\n\
     \\begin{document}\n\\maketitle\n\\end{document}\n",
    false,
  );
  assert_eq!(undefined_errors(&log), ["\\foo"], "{log}");
  assert_eq!((error_count(&log), warning_count(&log)), (1, 2), "{log}");
  assert_eq!(creators(&xml), [
    row(&["Ann Able"]),
    row(&["Bob 1 \\foo b@x.org"])
  ]);
  assert_eq!(
    creator_elements(&xml)[1],
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Bob<sup>1</sup><ERROR \
     class=\"undefined\">\\foo</ERROR>b@x.org</personname>\n  </creator>"
  );
  // (an `@` token right at the Error's end is past it, the span end-exclusive as the ink path's: `…b\foo@x.org` declines
  // the address as `…b\foo{}@x.org` does — review revERR2c finding 2, ERR2c kept the made-up "Baker1b@x.org" there)
  let (log, xml) = convert(
    repro!("emitter_undefined_command_before_an_address_at_declines_the_glued_mark"),
    false,
  );
  assert_eq!(undefined_errors(&log), ["\\foo"], "{log}");
  assert_eq!((error_count(&log), warning_count(&log)), (1, 2), "{log}");
  assert_eq!(creators(&xml), [
    row(&["Ann Able"]),
    row(&["Bob Baker 1 b \\foo @x.org"])
  ]);
  assert_eq!(
    creator_elements(&xml)[1],
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Bob Baker<sup>1</sup>b<ERROR \
     class=\"undefined\">\\foo</ERROR>@x.org</personname>\n  </creator>"
  );
}

/// A command the class's `\maketitle` defines before `\@author` (kr.sty's `\affiliations`) is defined where TeX reads the
/// block: no Error (pdflatex 0; Perl's 1 is KNOWN_PERL_ERRORS #1219), the affiliation read; 2602.18351, 2604.26688,
/// 2405.06973. Distinct: the net's exclusion of what the flush's code defines.
#[test]
fn a_command_the_class_maketitle_defines_raises_no_error() {
  let (log, xml) = convert(
    repro!("author_command_defined_in_maketitle_raises_no_error"),
    false,
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(creators(&xml), [row(&["Ann Able", "affiliation=Univ X"])]);
}
