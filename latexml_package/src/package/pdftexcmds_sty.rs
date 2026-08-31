//! pdftexcmds.sty — pdfTeX utility commands
//! Perl: pdftexcmds.sty.ltxml
//! Everything is in pdfTeX.pool already; just require iftex.
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  RequirePackage!("iftex");
  // Real pdftexcmds.sty on a pdfTeX engine simply `\let`s each `\pdf@…`
  // wrapper to the engine primitive (pdftexcmds.sty: "\let\pdf@strcmp
  // \pdfstrcmp" etc.). Delegate the same way — the primitives in
  // `latexml_engine/src/pdftex.rs` are REAL implementations (strcmp,
  // md5, file size/date, PDF escapes), format-verified against live
  // pdfTeX. The former hardcoded stubs here were actively wrong:
  // `\pdf@strcmp{}{}` → "0" made EVERY string comparison "equal", so
  // e.g. hep-paper.sty's `\ifnum\pdf@strcmp{\hep@bibliography}{false}=0`
  // skipped loading its whole bibliography layer (`undefined
  // \printbibliography` across the hep-* manuals, 2026-08-31 corpus).
  DefMacro!("\\pdf@strcmp{}{}", "\\pdfstrcmp{#1}{#2}");
  DefMacro!("\\pdf@mdfivesum{}", "\\pdfmdfivesum{#1}");
  DefMacro!("\\pdf@filemdfivesum{}", "\\pdfmdfivesum file {#1}");
  DefMacro!("\\pdf@filesize{}", "\\pdffilesize{#1}");
  DefMacro!("\\pdf@filemoddate{}", "\\pdffilemoddate{#1}");
  DefMacro!("\\pdf@escapehex{}", "\\pdfescapehex{#1}");
  DefMacro!("\\pdf@unescapehex{}", "\\pdfunescapehex{#1}");
  DefMacro!("\\pdf@escapestring{}", "\\pdfescapestring{#1}");
  DefMacro!("\\pdf@escapename{}", "\\pdfescapename{#1}");
  // \pdf@shellescape reports the shell-escape state; 0 (disabled) is the
  // CORRECT value here — matches pdflatex run without -shell-escape, and we
  // never execute \write18. Justified stub, not an approximation.
  DefMacro!("\\pdf@shellescape", "0");
  // \pdf@filedump{offset}{len}{file}: raw byte-dump probe (bmpsize sniffs
  // image headers with it — TL bmpsize.sty L51-53, witnesses 2406.02536,
  // 2406.03347). Graphics sizing has its own pipeline here; a hex dump of
  // image bytes carries no document content. Justified noop.
  def_macro_noop("\\pdf@filedump{}{}{}")?;
  def_macro_noop("\\pdf@primitive{}")?;
});
