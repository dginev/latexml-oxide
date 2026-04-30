use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: emulateapj.cls.ltxml — Seems to be equivalent to aastex.
  // Perl `LoadClass('aastex', withoptions => 1)` forwards the current class
  // options; Rust's `load_class_with_options` (latexml_core::binding::content)
  // reads `class_options` from state and passes them through. Previous
  // `load_class("aastex", Vec::new(), ...)` silently dropped the user's
  // `\documentclass[...]{emulateapj}` options before they reached aastex.
  load_class_with_options("aastex", Tokens!())?;
  RequireResource!("ltx-apj.css");
  RequirePackage!("emulateapj");
});
