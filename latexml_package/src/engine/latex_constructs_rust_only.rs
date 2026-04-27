//! latex_constructs_rust_only — Rust-side hotfix overrides for LaTeX-format CSes.
//!
//! Holds bindings present in the Rust port but **not** in any of Perl's three
//! `latex_{base,bootstrap,constructs}.pool.ltxml` files. Anything that lives
//! here is a hotfix tracked separately so that the corresponding Rust
//! "engine/latex_*.rs" siblings stay byte-for-byte parity with the Perl
//! source.
//!
//! Loaded LAST in `latex.rs`'s `LoadFormat('latex')` chain, after
//! `latex_constructs`, so every entry can rely on:
//! * The dump (or `latex_base.rs` under NODUMP) having installed raw LaTeX-kernel CSes.
//! * `latex_constructs.rs` having registered its own definitions (which some entries here `Let!`
//!   against — e.g. `\IfPackageLoadedTF ↦ \@ifpackageloaded`).
//!
//! Categories (in source order below):
//! 1. Modern LaTeX kernel CSes added post-2020 (the `\If…AtLeast/LoadedTF` family). LaTeX2e re-Lets
//!    these from the kernel; the Perl source predates them, so they need an explicit override here.
//! 2. LaTeXML-internal helpers that the engine code expects to exist (`\ltx@hard@MessageBreak`,
//!    `\ltx@ifclassloaded`, `\ltx@ifpackageloaded`).
//! 3. List internals not in Perl source (`\@bls`, `\@listi`-`\@listvi`, `\@maxlistdepth`). The dump
//!    captures the values raw `latex.ltx` installs; these are defensive overrides for the NODUMP
//!    path.
//! 4. Misc Rust-side stubs (`\@latexbug`, `\maybe@end@title`, `\thebibliography@ID` empty default).
//!
//! Future migrations (deferred — require helper-fn moves):
//! * `\filecontents`, `\lx@filecontents@star` (need `cache_filecontents` helper relocated
//!   alongside).
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  //======================================================================
  // 1. Modern LaTeX kernel — `\If…AtLeast/LoadedTF` family
  //
  // Latex.ltx L15252-15256: LaTeX3-style aliases for the file-load
  // tracking commands. The dump captures these as `Lt(...)` self-let
  // entries that don't actually replay because we filter same-target
  // aliases in `dump_writer`. Re-establish here post-dump.
  //======================================================================
  Let!("\\IfPackageLoadedTF",  r"\@ifpackageloaded");
  Let!("\\IfClassLoadedTF",    r"\@ifclassloaded");
  Let!("\\IfPackageAtLeastTF", r"\@ifpackagelater");
  Let!("\\IfClassAtLeastTF",   r"\@ifclasslater");
  Let!("\\IfFormatAtLeastTF",  r"\@ifl@t@r@released");
  Let!("\\IfFileAtLeastTF",    r"\@ifl@t@r");

  //======================================================================
  // 2. LaTeXML-internal helpers
  //======================================================================
  // \ltx@hard@MessageBreak — emit a hard newline in error/warning text.
  // Used by `\GenericError`/`\GenericWarning`; the dump-loader's
  // let-target safety filter can clobber it under certain orderings,
  // so define here post-dump as well.
  DefMacro!("\\ltx@hard@MessageBreak", None, "^^J");

  // LaTeXML aliases for the file-loaded predicates.
  Let!("\\ltx@ifpackageloaded", r"\@ifpackageloaded");
  Let!("\\ltx@ifclassloaded",   r"\@ifclassloaded");

  //======================================================================
  // 3. List internals — defensive NODUMP-path overrides
  //
  // Raw LaTeX classes (article.cls etc.) define these; the dump captures
  // the kernel's `\def`s. Under `LATEXML_NODUMP=1` (no dump load) the
  // bindings would be missing, so we install no-op fallbacks here.
  //======================================================================
  DefRegister!("\\@bls"          => Dimension!("12pt"));
  DefRegister!("\\@maxlistdepth" => Number::new(6));

  // List formatting macros from article.cls / report.cls / book.cls.
  // No-ops because LaTeXML handles list formatting via CSS.
  DefMacro!("\\@listi",   "");
  DefMacro!("\\@listii",  "");
  DefMacro!("\\@listiii", "");
  DefMacro!("\\@listiv",  "");
  DefMacro!("\\@listv",   "");
  DefMacro!("\\@listvi",  "");

  //======================================================================
  // 4. Misc Rust-side stubs
  //======================================================================
  // `\@latexbug` — kernel macro used to mark would-be bug reports.
  // No-op stub.
  DefMacro!("\\@latexbug", "");

  // `\maybe@end@title` — Constructor that closes ltx:titlepage if open.
  // Used by Rust's titling pipeline; not directly mirrored in Perl.
  DefConstructor!("\\maybe@end@title", sub[document, _args, _props] {
    if document.is_closeable("ltx:titlepage").is_some() {
      document.close_element("ltx:titlepage")?;
    }
  });

  // `\thebibliography@ID` — initial empty default. Per-bibliography
  // value is reassigned at \begin{thebibliography} time (see
  // latex_constructs.rs `\bibliography` constructor).
  DefMacro!("\\thebibliography@ID", "");
});
