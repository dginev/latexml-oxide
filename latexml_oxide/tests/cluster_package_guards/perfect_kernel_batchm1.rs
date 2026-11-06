//! Red/green guards for the TeX Live manuals' regressions (docs/perfect_kernel/MANUAL_REGRESSIONS.md).
//!
//! Cluster A: under raw classes a binding's name prefix took a published class that is a class of its own (`bookest` ->
//! `book`), and its commands were undefined. A prefix marks an author's renamed copy beside the document, or a sibling
//! shipped with the binding's class; `content.rs` `is_unrelated_distribution_class`, OD #444.
use super::{perfect_kernel_batch46::error_count, perfect_kernel_batch89a::cli_conversion};

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/class-bindings/",
      $name,
      ".tex"
    ))
  };
}

/// The arXiv profile reads classes raw (`rawclasses`), as the manuals' sweep does.
fn log_of(tex: &str) -> String { cli_conversion(tex, false).0 }

#[test]
fn a_published_class_named_after_a_binding_is_read_as_itself() {
  let log = log_of(repro!("distribution_class_named_after_a_binding"));
  assert!(!log.contains("Can't find binding"), "{log}");
  assert_eq!(error_count(&log), 0, "{log}");
}

#[test]
fn a_class_beside_the_bindings_class_keeps_the_binding() {
  let log = log_of(repro!(
    "distribution_class_beside_its_binding_is_its_family"
  ));
  assert!(
    log.contains("Can't find binding for class jmlrbook (using jmlr)"),
    "{log}"
  );
  assert_eq!(error_count(&log), 0, "{log}");
}

#[test]
fn a_renamed_copy_beside_the_document_keeps_the_binding() {
  let log = log_of(repro!(
    "renamed_class_copy_beside_the_document_keeps_the_binding"
  ));
  assert!(
    log.contains("Can't find binding for class bookofmine (using book)"),
    "{log}"
  );
}
