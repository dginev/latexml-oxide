//! Red/green guards for SETCOPY: a `\let` copy of the kernel's `\author`/`\title` taken before a class's (or the
//! document's) redefinition, which the lock refuses and the kernel's setter replays (`\lx@replay@class@setter`,
//! OXIDIZED_DESIGN #482), is the setter as it stood then, as TeX's `\let` freezes it (tex.web §1221): its call neither
//! replays the refused body nor appends (OXIDIZED_DESIGN_DIVERGENCES #1078). tocbase.cls:647-677 `\let\orig@author
//! \author` … `\orig@author{\toc@authors}` replayed the class's `\g@addto@macro\toc@authors{\and \toc@authors}` through
//! the copy, a self-containing list and a Fatal box runaway (1811.06821, 1811.11810, 1904.05269, 2011.14174 aic.cls;
//! 1905.07135, 2311.03341 toc). Restoring the setter from its copy (`\let\author\orig@author`) leaves the kernel's own
//! setter, the refusal's records cleared (review SETCOPY B1). Repros under `tools/perfect_kernel/repros/sectioning-frontmatter/setter_copy/`.
use super::{
  convert_with,
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch84b::creators,
};

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/setter_copy/",
      $path
    ))
  };
}

fn names(xml: &str) -> Vec<String> { creators(xml).into_iter().map(|c| c.join(" | ")).collect() }

/// The investigation's minimal repro in tocbase's shape (the first author `\gdef`ed; red: `Fatal:Stomach:MemoryBudget`,
/// Status 3): one creator, the store's one author. And the `\title` sibling (red: no `<title>` and an Error): the copy's
/// call sets the title from the class's store, once.
#[test]
fn a_copy_taken_before_the_redefinition_is_the_kernels_setter() {
  let (log, xml) = convert_with(repro!("copy_before_redefinition.tex"), Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(!log.contains("Fatal:"), "{log}");
  assert_eq!(names(&xml), ["Christian Elbracht"]);
  let (log, xml) = convert_with(repro!("copy_of_title.tex"), Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(xml.matches("<title>").count(), 1, "{xml}");
  assert!(xml.contains("<title>My Title</title>"), "{xml}");
  assert_eq!(names(&xml), ["A. Author"]);
}

/// tocbase's shape: three `\author[x]{…}` calls the class appends to its list, handed whole to the copy in the
/// environment's end code — three creators, in order, each once (with the replay alone disarmed: six, each twice, as
/// the kernel's call appended; red: a Fatal box runaway); `\ifx` reads the copy as the setter it copies ("same").
#[test]
fn a_copy_in_an_end_code_takes_the_class_list_once() {
  let (log, xml) = convert_with(repro!("tocbase_shape.tex"), Some("ar5iv.sty"));
  // (one plain-text Warning per name: row R SETCOPY-FLAG, as in the guard above; the names are the PDF's)
  assert_eq!((error_count(&log), warning_count(&log)), (0, 3), "{log}");
  assert!(!log.contains("Fatal:"), "{log}");
  assert_eq!(names(&xml), ["Ann Able", "Bob Baker", "Cy Cole"]);
  assert!(xml.contains("Body same."), "{xml}");
}

/// A copy taken after the redefinition holds the class's definition in TeX, for which the kernel's replay stands: its
/// call replays the class's body (the counter counts both calls), as before this fix.
#[test]
fn a_copy_taken_after_the_redefinition_still_replays() {
  let (log, xml) = convert_with(repro!("copy_after_redefinition.tex"), Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(names(&xml), ["Ann Able", "Bob Baker"]);
  assert!(xml.contains("Authors: 2."), "{xml}");
}

/// Restoring the setter from its frozen copy (`\let\author\orig@author`, `\global\let\title\orig@title`) leaves the
/// kernel's own setter, the refusal's records cleared, as in TeX, where the class's definition is gone: a second
/// redefinition is refused afresh and replays (red on SETCOPY round 0: the copy's wrapper on `\author` itself hid it,
/// "Second: ." with no diagnostic). Distinct from the guard below: the replay after the restore, not its absence.
#[test]
fn a_restored_setter_takes_a_later_redefinition_afresh() {
  let (log, xml) = convert_with(repro!("restore_then_redefine.tex"), Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(names(&xml), ["Ann Able"]);
  assert!(xml.contains("Second: Ann Able."), "{xml}");
  let (log, xml) = convert_with(repro!("restore_then_redefine_title.tex"), Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(xml.matches("<title>").count(), 1, "{xml}");
  assert!(xml.contains("<title>My Title</title>"), "{xml}");
  assert!(xml.contains("Second: My Title."), "{xml}");
}

/// A restore with no later redefinition drops the class's definition: the document's `\author` runs the kernel's setter
/// alone (red on base: "Calls: 1.", the refused wrapper replayed); restored in an `\AtBeginDocument` hook, the preamble
/// call ran the wrapper and the document's call replaces the author.
#[test]
fn a_restored_setter_drops_the_class_definition() {
  let (log, xml) = convert_with(repro!("restore_only.tex"), Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(names(&xml), ["Ann Able"]);
  assert!(xml.contains("Calls: 0."), "{xml}");
  let (log, xml) = convert_with(repro!("copy_restore_in_class.tex"), Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(names(&xml), ["Bob Baker"]);
  assert!(xml.contains("Short: Ann Able."), "{xml}");
}
