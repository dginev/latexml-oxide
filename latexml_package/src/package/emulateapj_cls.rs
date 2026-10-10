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
  // emulateapj.cls is a revtex4 class of its own (:165-167 `\LoadClass…{\@revtex@cls}`), not an AASTeX version: it has
  // neither AASTeX 5.x's `\subsubsubsection`/`\supportfrom` nor AASTeX 6's `\fig` family and `\gridline` (the aastex
  // binding defines them by version, aastex_cls.rs), so a paper's own are its own (astro-ph/0503342 `\newcommand{\fig}`;
  // KNOWN_PERL_ERRORS #557).
  for command in [
    "\\subsubsubsection", "\\supportfrom",
    "\\fig", "\\leftfig", "\\rightfig", "\\boxedfig", "\\rotatefig", "\\gridline",
  ] {
    Let!(T_CS!(command), "\\@undefined");
  }
  // …and unlocked: emulateapj_sty.rs locks the `\fig` it found defined, which a 6+ `aastex.cls` the paper ships leaves
  // (aastex_cls.rs), and a lock on an undefined `\fig` would refuse the paper's own (64f review r3).
  assign_value("\\fig:locked", Stored::None, Some(Scope::Global));
});
