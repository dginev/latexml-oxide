use latexml_package::prelude::*;

use crate::discard_env::discard_env_body;

#[rustfmt::skip]
LoadDefinitions!({
  RequirePackage!("tikz");
  RequirePackage!("etoolbox");
  // Register the load the way the real forest.sty:1 `\ProvidesPackage` does,
  // so `\@ifpackageloaded{forest}` / `\@ifpackagelater{forest}{…}` in
  // dependants (forest-doc's own preamble, neoschool.cls) take the loaded
  // branch instead of concluding forest is absent. Guard:
  // `perfect_kernel_batch54::forest_binding_registers_as_loaded`.
  RawTeX!(r"\ProvidesPackage{forest}[2017/07/14 v2.1.5 Drawing (linguistic) trees]");
  Warn!(
    "missing_file",
    "forest.sty",
    "forest.sty is not implemented and will not be interpreted raw."
  );
  // Perl ar5iv-bindings/forest.sty.ltxml L46-50: \begin{forest} emits
  // <ltx:ERROR> and discards the body via discard_env_body.
  DefConstructor!(
    T_CS!("\\begin{forest}"), None,
    "<ltx:ERROR>{forest}</ltx:ERROR>",
    bounded => true,
    mode    => "text",
    locked  => true,
    before_digest => { discard_env_body("forest", "forest.sty.ltxml")?; }
  );
  DefMacro!("\\endforest", "\\relax");
  DefMacro!("\\forestset{}", "\\relax");
  DefMacro!("\\forestoption{}", "\\relax");
  DefMacro!("\\foresteoption{}", "\\relax");
  DefMacro!("\\forestregister{}", "\\relax");
  DefMacro!("\\foresteregister{}", "\\relax");
  // `\useforestlibrary[opts]{name}` loads a forest-lib-<name>.code.tex
  // file (e.g. `edges`, `linguistics`). Since we already discard the
  // `forest` env body and stub the CSes, the library content has no
  // surface effect — no-op the call so papers that load libraries
  // before any `\begin{forest}` reach the env-discard path cleanly.
  // Witness: arXiv:2508.19011 — `\useforestlibrary{edges}` was the
  // only blocking error.
  DefMacro!("\\useforestlibrary[]{}", "\\relax");
});
