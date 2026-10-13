//! Shared helper: the shipped class file a class-family binding reads raw (`compositio_cls`, `jinst_cls`); it lives in
//! `latexml_package::shipped_class`, where the package bindings use it too (`mn2e_cls`).
pub(crate) use latexml_package::shipped_class::{
  class_text_loads, requested_shipped_class, shipped_class_text,
};
