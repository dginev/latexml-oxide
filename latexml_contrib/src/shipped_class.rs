//! Shared helpers for a binding that reads a file raw ([`RawRead`] + [`read_raw`]: `pasa_cls`, `an_cls`, `cclayout_sty`,
//! the `compositio`/`jinst`/apa families) or inspects the copy a paper ships; they live in
//! `latexml_package::shipped_class`, where the package bindings use them too (`mn2e_cls`, `deepseek_cls`).
pub(crate) use latexml_package::shipped_class::{
  RawRead, SUBFIGURE_PACKAGES, class_code, class_code_argument, class_code_contains,
  class_option_given, class_text_loads, provide_shipped_aas_macros, read_raw,
  require_shipped_algorithm_packages, require_shipped_packages, shipped_class_loads_or,
  shipped_class_text,
};
