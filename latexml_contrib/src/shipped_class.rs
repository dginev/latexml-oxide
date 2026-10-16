//! Shared helper: the shipped class file a class-family binding reads raw (`compositio_cls`, `jinst_cls`); it lives in
//! `latexml_package::shipped_class`, where the package bindings use it too (`mn2e_cls`).
pub(crate) use latexml_package::shipped_class::{
  SUBFIGURE_PACKAGES, class_code, class_code_argument, class_code_contains, class_option_given,
  class_text_loads, requested_shipped_class, require_shipped_algorithm_packages,
  require_shipped_packages, shipped_class_loads_or, shipped_class_text,
};
