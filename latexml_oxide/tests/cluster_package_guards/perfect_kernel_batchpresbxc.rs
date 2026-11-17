//! PRESBXc: what the frontmatter-mark fail-safes leave beside the title's marks, which the FMRG rows
//! (`perfect_kernel_batchfmmarks`) do not read: the lock's refusal record of a name other than `\maketitle` survives a
//! `\let` (review revPRESBXb S1), and the mark of a `\maketitle`'s own note outside the author block (P3, RED).
use latexml::util::test::{convert_with, error_count, warning_count};

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $name,
      ".tex"
    ))
  };
}

/// A `\let` at the document's origin carries the refusal records of `\maketitle` alone, their one reader: the raw
/// class's refused `\title` keeps `\lx@dropped@title` past the document's `\global\let\title\relax`, as the kernel's
/// `\maketitle` cleanup (`\global\let\thanks\relax`, `\let\@maketitle\relax`) keeps the records the title-page reading
/// and the class readers look up after it (`state::carry_document_refusal_records`).
#[test]
fn a_let_carries_only_the_maketitle_refusal_record() {
  let (log, xml) = convert_with(
    repro!("presbxc_refusal_record_follows_maketitle_only"),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  for line in ["Before: kept; kept.", "After: kept; kept."] {
    assert!(xml.contains(line), "no {line:?} in:\n{xml}");
  }
}

/// RED (row R PRESBX/ownnote): a `\maketitle` whose body makes its own footnote before the title prints it with the
/// first mark of its `\thefootnote`, ∗ (pdflatex "∗ Preprint under review."); we write ‡, the mark after the names'.
#[test]
#[ignore = "RED: row R PRESBX/ownnote"]
fn a_maketitle_s_own_note_carries_its_printed_mark_red() {
  let (log, xml) = convert_with(
    repro!("presbxc_maketitle_footnote_before_the_title"),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let at = xml
    .find("Preprint under review.")
    .expect("the preprint note");
  let start = xml[..at].rfind("<note ").expect("its note element");
  let tag = &xml[start..start + xml[start..].find('>').expect("start tag")];
  assert!(
    tag.contains("mark=\"\u{2217}\"") || tag.contains("mark=\"*\""),
    "the preprint note's mark: {tag}"
  );
}
