//! tcolorbox.sty — colored and framed text boxes
//! Perl: tcolorbox.sty.ltxml
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // used in tcbbreakable.code.tex assuming it was defined
  DefRegister!("\\doublecol@number" => Number::new(0));
  // Ensure only unbreakable mode is possible.
  // Perl: locked => 1 prevents raw TeX tcbbreakable.code.tex from overriding
  // with the real breakable implementation (uses output routines → infinite loop).
  DefMacro!("\\tcb@init@breakable", "\\tcb@init@unbreakable", locked => true);

  // Perl 93f875a6: pre-define \tcb@use@autoparskip before raw TeX loading,
  // as pgfkeys initialization may not complete and the \AtBeginDocument hook
  // at tcolorbox.sty:1142 would call it undefined.
  DefMacro!("\\tcb@use@autoparskip", "\\relax");

  RequirePackage!("expl3");
  RequirePackage!("xparse");

  InputDefinitions!("tcolorbox", noltxml => true, extension => Some(Cow::Borrowed("sty")));

  // Suppress tcolorbox's library version check. The Rust kpathsea binding
  // may resolve a different tcolorbox.sty version than the library files,
  // causing spurious "tcolorbox is not installed correctly" errors.
  // Make the check a no-op — the versions are always compatible in practice.
  DefMacro!("\\tcb@check@library@version", "", locked => true);

  // \newtcblisting{name}[N][default]{tcb-options} (tcolorbox `listings`/`minted`
  // library) — a code-listing box. Its box styling is purely visual; what
  // matters for the logical output is the code BODY, which must be captured
  // verbatim and CLOSED at \end{name}. The raw library's body capture does not
  // integrate with LaTeXML's verbatim reader, so the listing runs past its
  // \end{name} and swallows following content (sections leak into
  // <ltx:verbatim>). Delegate to listings' \lstnewenvironment (same
  // name/[N][default] shape; the tcb options are dropped), whose verbatim reader
  // terminates correctly. `locked` so a later raw `\tcbuselibrary{listings}`
  // can't clobber it. Witness: 2507.00833 (ar5iv #569/#570), 2402.13846 (#504).
  DefMacro!(
    "\\newtcblisting{}[][]{}",
    "\\lstnewenvironment{#1}[#2][#3]{}{}",
    locked => true
  );
});
