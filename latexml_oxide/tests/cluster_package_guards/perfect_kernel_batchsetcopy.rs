//! Red/green guards for SETCOPY: a `\let` copy of the kernel's `\author`/`\title` taken before a class's (or the
//! document's) redefinition, which the lock refuses and the kernel's setter replays (`\lx@replay@class@setter`,
//! OXIDIZED_DESIGN #482), is the setter as it stood then, as TeX's `\let` freezes it (tex.web §1221): its call neither
//! replays the refused body nor appends (OXIDIZED_DESIGN_DIVERGENCES #1078). tocbase.cls:647-677 `\let\orig@author
//! \author` … `\orig@author{\toc@authors}` replayed the class's `\g@addto@macro\toc@authors{\and \toc@authors}` through
//! the copy, a self-containing list and a Fatal box runaway (1811.06821, 1811.11810, 1904.05269, 2011.14174 aic.cls;
//! 1905.07135 toc). Repros under `tools/perfect_kernel/repros/sectioning-frontmatter/setter_copy/`.
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

/// The investigation's minimal repro (red: `Fatal:Stomach:MemoryBudget`, Status 3): one creator, the store's one
/// author. Residual (row R SETCOPY-FLAG): the store's leading `\and` stays in the name the emitter writes as plain text,
/// "and Christian Elbracht", with the content check's Warning counting the extra word — a store the class's replayed
/// setter changed reads as no events of the copy's block. And the `\title` sibling (red: no `<title>` and an Error):
/// the copy's call sets the title from the class's store, once.
#[test]
fn a_copy_taken_before_the_redefinition_is_the_kernels_setter() {
  let (log, xml) = convert_with(repro!("copy_before_redefinition.tex"), Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(!log.contains("Fatal:"), "{log}");
  let creators = names(&xml);
  assert_eq!(creators.len(), 1, "{creators:?}");
  assert!(creators[0].ends_with("Christian Elbracht"), "{creators:?}");
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
