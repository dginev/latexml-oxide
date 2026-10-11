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
  // no AASTeX 6 `\fig` family or `\gridline` (which the aastex binding defines for a 6+ class only, aastex_cls.rs), so a
  // paper's own `\fig` is its own (astro-ph/0503342 `\newcommand{\fig}`; KNOWN_PERL_ERRORS #557). The 5.x
  // `\subsubsubsection`/`\supportfrom` stay, the family's union (user 2026-10-10).
  for command in ["\\fig", "\\leftfig", "\\rightfig", "\\boxedfig", "\\rotatefig", "\\gridline"] {
    Let!(T_CS!(command), "\\@undefined");
  }
  // …and unlocked: emulateapj_sty.rs locks the `\fig` it found defined, which a 6+ `aastex.cls` the paper ships leaves
  // (aastex_cls.rs), and a lock on an undefined `\fig` would refuse the paper's own (64f review r3).
  assign_value("\\fig:locked", Stored::None, Some(Scope::Global));
  // emulateapj loads REVTeX 4.1 when it is installed (emulateapj.cls:76-78, :151-167; copies since 11/10/2009, 1601.00007's
  // too), whose `\email`/`\homepage` set the address with `\href`, in the current font (revtex4-1.cls:1959-1961,
  // 2383-2391): roman, as pdflatex prints them (CMR8, also under `[revtex4]`), not AASTeX 5.x's `\tt` or REVTeX 4.0's
  // `\url` that the aastex binding it loads installs. The `_` is kept (user ruling 2026-10-11).
  DefMacro!("\\email[]{}", "\\lx@add@email{\\lx@address@textfont{#2}}");
  DefMacro!("\\homepage[] Semiverbatim", "\\lx@add@url[name={#1}]{\\lx@address@textfont{#2}}");
});
