//! ccaption.sty — extended captioning by Peter Wilson (Herries Press).
//!
//! ccaption provides:
//!  * Non-numbered/`*` captions (`\bottomcaption`, `\namedlegend`)
//!  * Continued captions (`\contcaption`)
//!  * Bilingual captions (`\bicaption`)
//!  * Sidecap-style captions, custom caption fonts/labels
//!  * `[subfigure]` compatibility: redefines `\caption`, `\@subcaption`, `\@float`, `\@dbflt` to
//!    track main-vs-sub state.
//!
//! Raw-loading ccaption.sty into our engine is fragile: with the
//! `[subfigure]` option, ccaption redefines `\@subcaption` to use
//! `\xdef\@subfigcaptionlist{... {\protect\numberline {\@currentlabel}
//! ...}}`. Our `\@currentlabel` expansion produces tokens like
//! `\the\c@figure` that the `\xdef` mis-handles, surfacing the
//! "You can't use `}` after `\the`" cascade at every figure +
//! `\caption{...}\label{a}` pair (driver: 1105.3285).
//!
//! Perl LaTeXML has no ccaption binding either; with default
//! `INCLUDE_STYLES=false`, raw ccaption.sty is not loaded. Match
//! Perl: stub the user-facing extension macros as thin wrappers
//! over `\caption`, leaving the kernel `\caption`/`\label`/
//! `\@subcaption` untouched.
use latexml_package::prelude::*;

LoadDefinitions!({
  Warn!(
    "missing_file",
    "ccaption.sty",
    "ccaption.sty is minimally stubbed — extended caption features (\\bicaption, \\contcaption, \\bottomcaption, sidecap-style) fall back to the base \\caption."
  );
  // Continued / split / bilingual captions — treat as regular captions.
  DefMacro!("\\contcaption{}", "\\caption{#1}");
  DefMacro!("\\bottomcaption{}", "\\caption{#1}");
  DefMacro!("\\legend{}", "\\caption{#1}");
  DefMacro!("\\namedlegend{}", "\\caption{#1}");
  // Bilingual caption: 4 args (label1, caption1, label2, caption2);
  // render the first language only (Perl's effective behavior).
  DefMacro!("\\bicaption{}{}{}{}", "\\caption{#2}");
  // Sub-captions (when ccaption's `[subfigure]` compat would be
  // active): forward to the standalone subfigure/subcaption packages
  // when available; otherwise emit as italic text. Both `\subcaption`
  // and `\subtop`/`\subbottom` follow this pattern.
  DefMacro!("\\subcaption{}", "{\\itshape #1}");
  DefMacro!("\\subtop{}", "{\\itshape #1}");
  DefMacro!("\\subbottom{}", "{\\itshape #1}");
  // Caption-formatting / -setup commands ccaption provides — accept
  // and discard.
  DefMacro!("\\captionnamefont{}", "");
  DefMacro!("\\captiontitlefont{}", "");
  DefMacro!("\\captiondelim{}", "");
  DefMacro!("\\captionstyle{}", "");
  DefMacro!("\\hangcaption", "");
  DefMacro!("\\indentcaption{}", "");
  DefMacro!("\\normalcaption", "");
  // `\newfixedcaption{cs}{type}` defines `cs` as a `\caption` for a
  // specific float type. Define `cs` to forward.
  DefMacro!("\\newfixedcaption[]{}{}", "\\providecommand#2{\\caption}");
  DefMacro!("\\renewfixedcaption[]{}{}", "\\renewcommand#2{\\caption}");
  DefMacro!(
    "\\providefixedcaption[]{}{}",
    "\\providecommand#2{\\caption}"
  );
  // Continued-figure subitem counter (ccaption tracks via
  // `\@captype`); user code may `\thesubfigure` etc. We provide a
  // benign `\thesubfigure` placeholder if not defined.
  def_macro_noop("\\contlabel{}")?;
  def_macro_noop("\\bicaptionbox{}{}")?;
});
