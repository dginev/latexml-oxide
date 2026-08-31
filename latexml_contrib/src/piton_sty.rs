//! piton.sty — François Pantigny's LPEG-based code highlighter, a
//! LuaLaTeX-ONLY package (its .sty lives under `tex/lualatex/` and raises
//! "LuaLaTeX is mandatory" as a FATAL on every other engine — which killed
//! whole documents here, e.g. the nicematrix manual, whose `{Code}` example
//! environment is built with `\NewPitonEnvironment`).
//!
//! Binding scope (user directive 2026-08-31 on LuaTeX-only environments):
//! CONTENT-preserving verbatim treatment. A piton environment's body is
//! program text; it is captured raw and rendered through the listings
//! display engine, exactly as our minted/fancyvrb-class degradations do —
//! syntax COLORING is presentation (a future refinement could run piton.lua
//! itself through the texlua bridge — it is pure LPEG, which texlua ships —
//! and consume its `tex.sprint` stream; tracked in
//! docs/perfect_kernel/CLUSTERS.md).
use latexml_package::{
  package::listings_sty::{listings_read_raw_lines, lst_process_display},
  prelude::*,
};

#[rustfmt::skip]
LoadDefinitions!({
  RequirePackage!("listings");

  // Shared engine for every piton-style environment: absorb an optional
  // `[keys]`, capture the body raw to the matching \end, display it as a
  // listing, and re-inject the \end so the environment machinery balances.
  DefPrimitive!("\\lx@piton@env{}", sub[(name)] {
    let env_name = name.to_string();
    let _opt = read_optional(None)?;
    bgroup();
    let text = listings_read_raw_lines(&env_name);
    unread(Tokenize!(TeXString::assembled(s!("\\end{{{env_name}}}"))));
    unread(Tokens::new(lst_process_display(None, &text)));
  });

  // \NewPitonEnvironment{name}{argspec}{before}{after} (piton doc §"New
  // environments"): creates a verbatim environment whose body piton
  // highlights; before/after are layout wrappers (minipage, width setup) —
  // presentation, not absorbed into the captured code. The created env's
  // own args (argspec, typically `O{}`) are covered by the shared engine's
  // optional-absorb.
  DefPrimitive!("\\NewPitonEnvironment{}{}{}{}", sub[(name, _argspec, _before, _after)] {
    let env = name.to_string().trim().to_string();
    def_macro(
      T_CS!(s!("\\{env}")),
      None,
      Tokens!(T_CS!("\\lx@piton@env"), T_BEGIN!(), ExplodeText!(&env), T_END!()),
      None,
    )?;
    def_macro(T_CS!(s!("\\end{env}")), None, Tokens!(), None)?;
  });

  // {Piton} — the package's own default environment.
  RawTeX!(r"\NewPitonEnvironment{Piton}{}{}{}");

  // \piton{code} — inline code (braced form; verbatim-delimited forms
  // degrade through Semiverbatim).
  DefMacro!("\\piton Semiverbatim", "\\texttt{#1}");
  // \PitonInputFile[opt]{file} — display a source file.
  DefMacro!("\\PitonInputFile []{}", "\\lstinputlisting{#2}");
  // Configuration surface — styling keys (language=, width=, splittable=…).
  def_macro_noop("\\PitonOptions{}")?;
  def_macro_noop("\\SetPitonStyle{}")?;
  def_macro_noop("\\SetPitonIdentifier{}{}")?;
  def_macro_noop("\\PitonClearUserStyles")?;
  def_macro_noop("\\NewPitonLanguage{}{}")?;
});
