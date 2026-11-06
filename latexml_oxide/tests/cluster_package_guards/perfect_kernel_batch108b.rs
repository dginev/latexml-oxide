//! Red/green guards for batch 108b: the title-page check's third source and two narrow items (FIDELITY_CHARTER §4, every
//! residual loss counted by a Warning; OXIDIZED_DESIGN_DIVERGENCES #953 "108b"). What the document's text gives a setter
//! a raw class or a raw file of its own directory made, before the title page, each argument the setter writes into a
//! store, is digested once and wanted in the output; a word of this source alone may never be printed, so it is named
//! apart, in a `Warning:frontmatter:unclaimed_store`. A check only: the XML is the one the base wrote. Each guard converts
//! its repro under `tools/perfect_kernel/repros/sectioning-frontmatter/` eager and `--streaming`
//! (`perfect_kernel_batch98c::converted`), and asserts the frontmatter whole and the check's lines.
use super::{
  perfect_kernel_batch89a::cli_conversion, perfect_kernel_batch98c::converted,
  perfect_kernel_batch106t::check_lines,
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

/// The log's Info lines of `tag`, whole.
fn info_lines(tex: &str, tag: &str) -> Vec<String> {
  let (log, _) = cli_conversion(tex, false);
  log
    .lines()
    .filter(|line| line.starts_with(tag))
    .map(ToString::to_string)
    .collect()
}

/// CUP-JNL-DTM.cls's `\auaddress`, printed only under a count the class's refused `\author` advances: the print run
/// reaches no address, where pdflatex prints it. Distinct path: a store the rehearsal's print skips, through
/// `\g@addto@macro`. Witness 2305.00867.
#[test]
fn a_store_the_print_gates_out_is_counted() {
  let tex = repro!("check_raw_setter_store_gated");
  assert_eq!(
    converted(tex, 1),
    r##"<title>A Title</title> <creator role="author"> <personname>Ann Able</personname> </creator> <para xml:id="p1"> <p xml:id="p1.1">Body text.</p> </para>"##
  );
  assert_eq!(check_lines(tex), [
    r##"Warning:frontmatter:unclaimed_store 4 word(s) of the document's stored by a raw class's command, which pdflatex may not print, are not in the output: lunar institute gravel road"##,
  ]);
}

/// naverlabseurope.cls's `\maketitle`, refused by the replay gate over `\@teaserfig`: the print falls back to LaTeX's
/// fields, which never print `\@authors`, `\@affiliations`. Distinct path: no class body runs at all. Witnesses 2408.05088,
/// 2605.02209.
#[test]
fn a_store_of_a_refused_replay_is_counted() {
  let tex = repro!("check_raw_setter_replay_refused");
  assert_eq!(
    converted(tex, 1),
    r##"<title>A Title</title> <para xml:id="p1"> <p xml:id="p1.1">Body text.</p> </para>"##
  );
  assert_eq!(check_lines(tex), [
    r##"Warning:frontmatter:unclaimed_store 4 word(s) of the document's stored by a raw class's command, which pdflatex may not print, are not in the output: ann able lunar institute"##,
  ]);
}

/// Every writer `Expandable::stored_parameters` reads, an own-directory style's setter, `\@ifnextchar`'s way in; a
/// replacing setter's earlier call, what a body only tests, a shorthand maker, a bracketed key and a call after the title
/// page are not wanted; a store only page furniture reads, or none, and a number alone go to the Info
/// (`StoreReaders`, the false-alarm rule: 2512.10209, 1202.2064, 1312.3323). Distinct path: the body test, the call
/// record and the reader graph. Witnesses 1401.7920, 1011.2770, 2605.02209.
#[test]
fn each_writer_of_a_store_counts_and_a_replaced_call_does_not() {
  let tex = repro!("check_store_replaced_and_siblings");
  assert_eq!(
    converted(tex, 1),
    r##"<title>A Title</title> <creator role="author"> <personname>Ann Able</personname> </creator> <para xml:id="p1"> <p xml:id="p1.1">Body text.</p> </para>"##
  );
  assert_eq!(check_lines(tex), [
    r##"Warning:frontmatter:unclaimed_store 11 word(s) of the document's stored by a raw class's command, which pdflatex may not print, are not in the output: gdefword xdefword tokword writeword renewword ifnextword styword kept firstapp secondapp keyedvalue"##,
  ]);
  let (log, _) = cli_conversion(tex, false);
  assert_eq!(
    log
      .lines()
      .filter(|line| line.starts_with("Info:frontmatter:unread_store"))
      .collect::<Vec<_>>(),
    [
      "Info:frontmatter:unread_store 4 word(s) of the document's stored by a raw class's command that only page furniture reads, or nothing, are not in the output: 12345 csnameword unreadword headword"
    ]
  );
}

/// A raw TeX Live package's setter is out, and with no title page nothing prints a store. Distinct path: the source's two
/// restrictions (negative).
#[test]
fn a_tex_live_packages_setter_or_no_title_page_wants_nothing() {
  for tex in [
    repro!("check_raw_tex_live_package_setter_unwanted"),
    repro!("check_raw_setter_no_title_page_unwanted"),
  ] {
    converted(tex, 0);
    assert!(check_lines(tex).is_empty());
  }
}

/// IEEEtran's `\IEEEmembership` drop reached through the document's own macro: the dropped grade is no token of `\author`'s
/// argument, only of the body `\SeMieee` holds. Distinct path: `title_page_check::writes` reading a document macro's body.
/// Witness 2503.14354; the drop is Perl's too, IEEEtran.cls.ltxml:143.
#[test]
fn a_bindings_drop_inside_a_documents_macro_is_counted() {
  let tex = repro!("check_binding_drops_document_macro_expansion");
  assert_eq!(
    converted(tex, 1),
    r##"<title>A Title</title> <creator role="author"> <personname>Ann Able</personname> </creator> <para xml:id="p1"> <p xml:id="p1.1">Body text.</p> </para>"##
  );
  assert_eq!(check_lines(tex), [
    r##"Warning:frontmatter:unclaimed The document's frontmatter commands were given 3 word(s) that its output does not hold: senior member ieee"##,
  ]);
}

/// `\@listi`, `\@listI` empty left `\let` with the next letter as its name when a class's non-robust `\normalsize` expands
/// in a `\protected@xdef` (esapub2005.cls through authblk's `\affil`): two Errors and "ampus". The XML this one item
/// changes. Witness 2310.07393.
#[test]
fn list_parameter_macros_keep_the_letter_after_them() {
  let tex = repro!("list_parameters_macro_in_an_expansion_keeps_the_next_letter");
  assert_eq!(
    converted(tex, 0),
    r##"<title>A Title</title> <creator role="author"> <personname>Ann Able</personname> </creator> <para xml:id="p1"> <p xml:id="p1.1">Space Robotics, Campus Kirchberg. Small Word.</p> </para> <para xml:id="p2"> <p xml:id="p2.1">Body text.</p> </para>"##
  );
}

/// `\end{document}`, `\enddocument` and `\@@end` in an argument a raw class only stores: pdflatex prints the body, and the
/// check's speculative run of the argument (`argument_print`) must not end the real document, as it did, silently. Distinct
/// path: `gullet::speculative_digestion`, shared with 106t's dropped-argument run and with the title page's print.
#[test]
fn a_document_end_in_a_stored_argument_ends_nothing() {
  let tex = repro!("check_raw_setter_end_document_in_store");
  assert_eq!(
    converted(tex, 0),
    r##"<title>A Title</title> <creator role="author"> <personname>Ann Able</personname> </creator> <section inlist="toc" xml:id="S1"> <tags> <tag>1</tag> <tag role="refnum">1</tag> <tag role="typerefnum">§1</tag> </tags> <title><tag close=" ">1</tag>One</title> <para xml:id="S1.p1"> <p xml:id="S1.p1.1">Body text one.</p> </para> <para xml:id="S1.p2"> <p xml:id="S1.p2.1">Body text two.</p> </para> </section>"##
  );
  assert!(check_lines(tex).is_empty());
}

/// A setter that appends through `\expandafter`, a scratch register or macro or a temporary name: each call is wanted, as
/// each prints; a later call supersedes one that only replaces known stores (1612.06202). Distinct path: `written_store_name`'s
/// `\expandafter`, `stored_parameters`' hand-on and `replaces`. Real shape: SCIS2026.cls's `\@authoremail`.
#[test]
fn a_replacing_guess_keeps_each_call_of_an_appender() {
  let tex = repro!("check_store_replacing_guess_drops_an_appender");
  converted(tex, 1);
  assert_eq!(check_lines(tex), [
    r##"Warning:frontmatter:unclaimed_store 12 word(s) of the document's stored by a raw class's command, which pdflatex may not print, are not in the output: firstex secondex firsttok secondtok firsttmp secondtmp letword keptword firstappend secondappend keptone kepttwo"##,
  ]);
}

/// A `\csname` store (`\expandafter\gdef\csname aff\theaffn\endcsname`, `\@namedef`, `\csgdef`) never replaces, and has no
/// name to find a reader by: every call reaches the Info. `\gappto` writes a known store. Distinct path: the etoolbox
/// writers and the name in a group of `written_store_name`.
#[test]
fn a_csname_store_keeps_each_call_in_the_info() {
  let tex = repro!("check_store_csname_counter_keeps_each_call");
  converted(tex, 1);
  assert_eq!(check_lines(tex), [
    r##"Warning:frontmatter:unclaimed_store 2 word(s) of the document's stored by a raw class's command, which pdflatex may not print, are not in the output: firstgap secondgap"##,
  ]);
  assert_eq!(info_lines(tex, "Info:frontmatter:unread_store"), [
    "Info:frontmatter:unread_store 6 word(s) of the document's stored by a raw class's command that only page furniture reads, or nothing, are not in the output: firstaff secondaff firstnamed secondnamed firstcs secondcs"
  ]);
}

/// An optional argument or one in brackets is a key far more often than text, so it is not counted, but its words have a
/// record. Distinct path: `StoreCall::keys`.
#[test]
fn an_optional_argument_stored_reaches_the_info() {
  let tex = repro!("check_store_optional_argument_printed");
  converted(tex, 1);
  assert_eq!(check_lines(tex), [
    r##"Warning:frontmatter:unclaimed_store 4 word(s) of the document's stored by a raw class's command, which pdflatex may not print, are not in the output: gravel road sandy lane"##,
  ]);
  assert_eq!(info_lines(tex, "Info:frontmatter:unread_store"), [
    "Info:frontmatter:unread_store 4 word(s) of the document's stored by a raw class's command that only page furniture reads, or nothing, are not in the output: lunar institute solar works"
  ]);
}

/// A postal code printed with its address is counted with it; a number alone stays in the Info. Distinct path: the per
/// argument word sets of `check_title_page_words`. Witnesses 1011.2770, 2305.00867, 2605.02209.
#[test]
fn digits_count_with_a_counted_word_of_their_argument() {
  let tex = repro!("check_store_digits_with_an_address");
  converted(tex, 1);
  assert_eq!(check_lines(tex), [
    r##"Warning:frontmatter:unclaimed_store 3 word(s) of the document's stored by a raw class's command, which pdflatex may not print, are not in the output: gravel road 01886"##,
  ]);
  assert_eq!(info_lines(tex, "Info:frontmatter:unread_store"), [
    "Info:frontmatter:unread_store 1 word(s) of the document's stored by a raw class's command that only page furniture reads, or nothing, are not in the output: 10027"
  ]);
}

/// The source's bounds say what they left out: the calls past the first 512, an argument no run could read. Distinct path:
/// `StoreCalls::dropped`, `argument_print`'s Info.
#[test]
fn the_bounds_of_the_third_source_say_what_they_dropped() {
  let tex = repro!("check_store_caps_say_what_they_dropped");
  converted(tex, 1);
  assert_eq!(check_lines(tex), [
    r##"Warning:frontmatter:unclaimed_store 1 word(s) of the document's stored by a raw class's command, which pdflatex may not print, are not in the output: filler"##,
  ]);
  assert_eq!(info_lines(tex, "Info:frontmatter:store_calls_dropped"), [
    r"Info:frontmatter:store_calls_dropped The title-page check read the first 512 calls of raw setters and left out 9 later one(s), of \setaddr"
  ]);
  let unread = info_lines(tex, "Info:frontmatter:unread_argument");
  assert_eq!(unread.len(), 1);
  assert!(
    unread[0].starts_with(
      r"Info:frontmatter:unread_argument The title-page check could not read \longaddr's argument of 81920 token(s), which it leaves out: Word Word"
    ),
    "{unread:?}"
  );
}
