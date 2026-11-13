//! Red/green guards for batch CR10b: three silent content losses of review CR9. A continuation line inside a
//! mark-split address keeps its line break (`emitter::emit::call_pieces`, `kept_breaks`); an `\fcolorbox` line inside
//! `\author` is read (its text a property, `util_text::printed_properties`); frontmatter setters in a nameless `\author`
//! run as a read block's do (`emitter::emit::recover_queued`). OXIDIZED_DESIGN_DIVERGENCES #985, #989.
use latexml::util::test::convert_with;

use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batchcr2::{convert_clean, elements},
};

/// elsarticle `\address{$^{1}$ … School of Yak\\ Owl University, Gnu City\\ $^{2}$ Emu Institute\\ E-mail: bob@emu.edu\\
/// Kiwi Town}`: each mark's entry runs on to the next mark with its line breaks kept, as pdflatex and Perl print it — the
/// base printed "School of YakOwl University", 0 Warnings (2212.03638, 2211.03175, cond-mat0204589, 0904.3899,
/// 1311.0012) — and a break whose line was cut out (the e-mail, its own contact) not doubled. Repro
/// sectioning-frontmatter/address_continuation_line_keeps_its_break.
#[test]
fn address_continuation_line_keeps_its_break() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/address_continuation_line_keeps_its_break.tex"
  ));
  assert_eq!(elements(&xml, "creator"), vec![
    "<creator role=\"author\">\n    <personname>Ann Able<sup xml:id=\"id1\"><text font=\"italic\" xml:id=\"id1.1\">1</text></sup></personname>\n    <contact name=\"Address:\u{a0}\" role=\"address\"><sup xml:id=\"id2\"><text font=\"italic\" xml:id=\"id2.1\">1</text></sup> Department of Zebra, School of Yak<break/>Owl University, Gnu City</contact>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Bob Baker<sup xml:id=\"id3\"><text font=\"italic\" xml:id=\"id3.1\">2</text></sup></personname>\n    <contact name=\"Address:\u{a0}\" role=\"address\"><sup xml:id=\"id4\"><text font=\"italic\" xml:id=\"id4.1\">2</text></sup> Emu Institute<break/>Kiwi Town</contact>\n    <contact name=\"E-mail:\u{a0}\" role=\"email\">bob@emu.edu</contact>\n  </creator>",
  ]);
}

/// acl's e-mail line boxed in `\author` (`\fcolorbox{white}{white}{$\{$\texttt{\href{mailto:ann@…}{ann}, }…$\}$\texttt{@zebra.edu}}`,
/// 2211.07344, 2305.15057): the box keeps its text in its `text` property, which no reader walked, so the line was lost
/// with 0 Warnings. Now each author's e-mail, and the boxed line read as the same line unboxed is — the same creators
/// and the same Warning count (the content check's one Warning on the brace-group spelling is the unboxed line's too).
/// Repro sectioning-frontmatter/fcolorbox_email_line_in_author.
#[test]
fn fcolorbox_email_line_in_author() {
  let boxed = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/fcolorbox_email_line_in_author.tex"
  );
  let unboxed = boxed.replace("\\fcolorbox{white}{white}{$", "{$");
  let (log, xml) = convert_with(boxed, Some("ar5iv.sty"));
  let (unboxed_log, unboxed_xml) = convert_with(&unboxed, Some("ar5iv.sty"));
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), warning_count(&unboxed_log), "{log}");
  assert_eq!(elements(&xml, "creator"), vec![
    "<creator role=\"author\">\n    <personname>Ann Able</personname>\n    <contact name=\"Email:\u{a0}\" role=\"email\"><text backgroundcolor=\"#FFFFFF\" xml:id=\"id1\">ann@zebra.edu</text></contact>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Bob Baker</personname>\n    <contact name=\"Email:\u{a0}\" role=\"email\"><text backgroundcolor=\"#FFFFFF\" xml:id=\"id2\">bob@zebra.edu</text></contact>\n  </creator>",
  ]);
  let plain = |xml: &str| {
    elements(xml, "creator")
      .iter()
      .map(|c| {
        c.replace("<text backgroundcolor=\"#FFFFFF\" xml:id=\"id1\">", "")
          .replace("<text backgroundcolor=\"#FFFFFF\" xml:id=\"id2\">", "")
          .replace("</text>", "")
      })
      .collect::<Vec<_>>()
  };
  assert_eq!(plain(&xml), elements(&unboxed_xml, "creator"));
}

/// jheppub's `\author{\abstract{We define …} \keywords{defect operator, …}}` after a named `\author[a]{…}` (1211.2861):
/// a block of no name whose setters pdflatex runs as `\maketitle` prints the names — the abstract and the keywords,
/// which were lost with 0 Warnings, its block of no creator skipped by the recovery a read block's setters get. Repro
/// sectioning-frontmatter/jheppub_setters_in_a_nameless_author.
#[test]
fn jheppub_setters_in_a_nameless_author() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/jheppub_setters_in_a_nameless_author.tex"
  ));
  assert_eq!(elements(&xml, "creator"), vec![
    "<creator role=\"author\">\n    <personname>Ann Able</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Zebra University</contact>\n  </creator>",
  ]);
  assert_eq!(elements(&xml, "abstract"), vec![
    "<abstract inlist=\"toc\" name=\"Abstract\" xml:id=\"abstract1\">\n    <p xml:id=\"abstract1.1\">We define a class of loops.</p>\n  </abstract>",
  ]);
  assert_eq!(elements(&xml, "keywords"), vec![
    "<keywords name=\"Keywords\">defect operator, supersymmetry</keywords>",
  ]);
}
