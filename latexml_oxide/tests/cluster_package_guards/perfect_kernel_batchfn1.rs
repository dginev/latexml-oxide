//! Guards for FN1: a note an author's name or contact holds is the creator's, beside it (OXIDIZED_DESIGN_DIVERGENCES
//! #1001; queue_digest.rs `lift_notes_beside`, called by the `ltx:personname` and `ltx:contact` closes). The
//! `ltx:note` a `\footnotemark[<n>]` makes where it stands, filled by its `\footnotetext[<n>]` (relocate_footnote),
//! sat inside `<personname>` on every class route, as in Perl (RT1's F17; witness 2104.12292). Each guard converts its
//! repros and asserts the exact Error and Warning counts and every creator whole (ids masked, whitespace between
//! elements dropped). Distinct paths: the name's close, after the text fills the note or before (the name guard: the
//! article and OmniBus routes, a text before `\maketitle`, an unpaired mark); the contact's close (the affiliation
//! guard); a wrapper of the mark left empty, and the whole-name bold rule after the lift (the wrapper guard); and
//! `\footnote` in `\author`, already the creator's `role="note"` contact (the already-right sibling); the text a
//! lifted note or emptied wrapper stood between joined, so the name's end is trimmed (the trim guard).

use latexml::util::test::{error_count, warning_count};
use regex::Regex;

use super::perfect_kernel_batchz4::{creators_xml, run};

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $path
    ))
  };
}

/// Every creator of `xml`, whole, ids masked and the whitespace between elements dropped.
fn creators(xml: &str) -> Vec<String> {
  let between = Regex::new(r">\s+<").expect("regex");
  creators_xml(xml)
    .iter()
    .map(|c| between.replace_all(c, "><").into_owned())
    .collect()
}

/// A `role="footnote"` note with the mark `mark` and the content `body`, as the creator's.
fn note(mark: &str, body: &str) -> String {
  format!(
    "<note mark=\"{mark}\" role=\"footnote\"><tags><tag>{mark}</tag><tag role=\"refnum\">{mark}</tag><tag \
     role=\"typerefnum\">footnote {mark}</tag></tags>{body}</note>"
  )
}

/// Asserts the counts and the creators.
#[track_caller]
fn assert_creators(tex: &str, warnings: usize, want: &[String]) {
  let (log, xml) = run(tex);
  assert_eq!(
    (error_count(&log), warning_count(&log)),
    (0, warnings),
    "{log}"
  );
  assert_eq!(creators(&xml), want, "{xml}");
}

/// `Ann Able\footnotemark[1] \and Bob Baker\footnotemark[2]` with the `\footnotetext`s after `\maketitle` (article and
/// the OmniBus route) or before it: `<personname>Ann Able</personname>` then the note "Note A.", mark kept. Red
/// (int159a2): each note inside its name. A bare mark whose text prints apart stays a mark, beside the name.
#[test]
fn footnotemark_note_is_the_creators_beside_the_name() {
  let pair = |a: &str, b: &str| {
    vec![
      format!(
        "<creator role=\"author\"><personname>Ann Able</personname>{}</creator>",
        note(a, "Note A.")
      ),
      format!(
        "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname>{}</creator>",
        note(b, "Note B.")
      ),
    ]
  };
  assert_creators(
    repro!("author_footnotemark_note_is_the_creators_beside_the_name.tex"),
    0,
    &pair("*", "†"),
  );
  assert_creators(
    repro!("author_footnotemark_text_before_maketitle_beside_the_name.tex"),
    0,
    &pair("*", "†"),
  );
  assert_creators(
    repro!("author_footnotemark_note_beside_the_name_omnibus.tex"),
    1,
    &pair("1", "2"),
  );
  let (log, xml) = run(repro!("author_bare_footnotemark_beside_the_name.tex"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(
    creators(&xml),
    [
      "<creator role=\"author\"><personname>Ann Able</personname><note mark=\"*\" role=\"footnotemark\"><tags><tag>*\
       </tag><tag role=\"refnum\">*</tag><tag role=\"typerefnum\">footnote *</tag></tags></note></creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname></creator>",
    ],
    "{xml}"
  );
  assert_eq!(xml.matches(">Note A.</note>").count(), 1, "{xml}");
}

/// authblk's `\affil{Univ X\protect\footnotemark[1]}`: the affiliation "Univ X" alone, the note after it. Red
/// (int159a2): the note inside the contact.
#[test]
fn footnotemark_in_an_affiliation_is_the_creators_note_beside_it() {
  assert_creators(
    repro!("affiliation_footnotemark_note_beside_the_contact.tex"),
    0,
    &[format!(
      "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Affiliation:\u{a0}\" \
       role=\"affiliation\">Univ X</contact>{}</creator>",
      note("*", "Note A.")
    )],
  );
}

/// `\textbf{Ann Able\footnotemark[1]}` and `Bob Baker{\small\footnotemark[2]}`: plain names (the bold whole-name rule
/// after the lift; the emptied font wrapper gone), each note beside its name with its text as set. Red (int159a2):
/// the notes and wrappers inside the names.
#[test]
fn footnotemark_in_a_wrapper_leaves_the_name_alone() {
  assert_creators(
    repro!("author_footnotemark_in_a_wrapper_beside_the_name.tex"),
    0,
    &[
      format!(
        "<creator role=\"author\"><personname>Ann Able</personname>{}</creator>",
        note("*", "<text font=\"bold\">Note A.</text>")
      ),
      format!(
        "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname>{}</creator>",
        note("†", "<text fontsize=\"90%\">Note B.</text>")
      ),
    ],
  );
}

/// `\footnote{Note A.}` in `\author`: the creator's `role="note"` contact, never in the name (green on int159a2; the
/// already-right sibling).
#[test]
fn footnote_in_the_author_is_the_creators_note_contact() {
  assert_creators(
    repro!("author_footnote_in_the_name_is_the_creators_note.tex"),
    0,
    &[
      "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Note:\u{a0}\" \
       role=\"note\">Note A.</contact></creator>"
        .to_string(),
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname></creator>".to_string(),
    ],
  );
}

/// `Ann Able \footnotemark[1] \footnotemark[2]` and `Bob Baker {\small\footnotemark[3]} \footnotemark[4]` (1612.03574's
/// `Kent-Andre Mardal \footnotemark[2] \footnotemark[3]`): the text the lifted notes, or the emptied wrapper, stood
/// between is one text again, so the name's end is trimmed whole. Red (FN1 round 0, no join): "Ann Able " and
/// "Bob Baker ", a space left in each name.
#[test]
fn spaced_marks_leave_the_name_end_trimmed() {
  assert_creators(
    repro!("author_two_footnotemarks_name_end_trimmed.tex"),
    0,
    &[
      format!(
        "<creator role=\"author\"><personname>Ann Able</personname>{}{}</creator>",
        note("*", "Note A."),
        note("†", "Note B.")
      ),
      format!(
        "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname>{}{}</creator>",
        note("‡", "<text fontsize=\"90%\">Note C.</text>"),
        // (the 90 % of "Note D." is the base's too, not the lift's: RED catalog
        // `author_footnotemark_texts_take_a_sized_marks_size_red.tex`)
        note("§", "<text fontsize=\"90%\">Note D.</text>")
      ),
    ],
  );
}
