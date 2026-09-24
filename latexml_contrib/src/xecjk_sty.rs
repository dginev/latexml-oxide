//! xeCJK.sty — CJK font selection and spacing for XeLaTeX (no Perl binding).
//!
//! xeCJK's machinery (character classes via `\XeTeXinterchartoks`, CJK font
//! switching, punctuation kerning) is typesetting the Unicode-native engine
//! does not need: CJK text is already characters. The raw package checks the
//! engine through expl3 and stops outside XeTeX, so its user commands are bound
//! as argument-reading no-ops (arXiv 2605.30788: `\setCJKmainfont`,
//! `\newCJKfontfamily\japanesefont`). Guard:
//! `perfect_kernel_batch56::xetex_only_packages_load_under_the_default_persona`.
use latexml_package::prelude::*;

LoadDefinitions!({
  RequirePackage!("fontspec");
  // xeCJK.sty `\NewDocumentCommand` signatures: a font FAMILY command defines
  // its control sequence as a no-op font switch (fontspec's definer).
  DefMacro!(
    "\\newCJKfontfamily[] DefToken []{}",
    "\\lx@fontspec@definer#2[#3]{#4}[]"
  );
  for sig in [
    "\\setCJKmainfont[]{}",
    "\\setCJKsansfont[]{}",
    "\\setCJKmonofont[]{}",
    "\\setCJKmathfont[]{}",
    "\\setCJKfamilyfont{}[]{}",
    "\\setCJKfallbackfamilyfont{}[]{}",
    "\\CJKfontspec[]{}",
    "\\defaultCJKfontfeatures{}",
    "\\CJKsetecglue{}",
    "\\punctstyle{}",
    "\\normalspacedchars{}",
    "\\xeCJKsetup{}",
    "\\xeCJKsetwidth OptionalMatch:* {}{}",
    "\\xeCJKsetkern{}{}{}",
    "\\xeCJKsetcharclass{}{}{}",
    "\\xeCJKsetemboldenfactor{}",
    "\\xeCJKsetslantfactor{}",
    "\\xeCJKCancelSubCJKBlock OptionalMatch:* {}",
    "\\xeCJKRestoreSubCJKBlock OptionalMatch:* {}",
    "\\CJKspace",
    "\\CJKnospace",
    "\\makexeCJKactive",
    "\\makexeCJKinactive",
    "\\xeCJKallowbreakbetweenpuncts",
    "\\xeCJKnobreakbetweenpuncts",
    "\\xeCJKenablefallback",
    "\\xeCJKdisablefallback",
    "\\xeCJKnobreak",
    "\\xeCJKResetCharClass",
    "\\xeCJKResetPunctClass",
    "\\xeCJKVerbAddon",
    "\\xeCJKOffVerbAddon",
  ] {
    def_macro_noop(sig)?;
  }
  // `\CJKfamily{<family>}` (with `+`/`-` variants) and `\addCJKfontfeatures`
  // select a font: no-ops, reading their arguments.
  DefMacro!("\\CJKfamily OptionalMatch:+ OptionalMatch:- {}", "");
  DefMacro!("\\addCJKfontfeatures OptionalMatch:* []{}", "");
  // `\xeCJKDeclareCharClass*{<class>}{<chars>}` (xeCJK.sty:548) assigns characters
  // to a spacing class: no-op, reading its arguments.
  DefMacro!("\\xeCJKDeclareCharClass OptionalMatch:* {}{}", "");
  // The expl3 internals that classes and xeCJKfntef call (njuthesis, exam-zh,
  // xdupgthesis, xduugthesis, xduugtp, bitbeamer; class census 2026-09-24):
  // xeCJK.sty:87 `\__xeCJK_msg_new:nn` is `\msg_new:nnn{xeCJK}`; :139
  // `\xeCJK_add_to_shipout:n` and :977 `\xeCJK_declare_node:n` drive the page
  // builder and inter-character nodes (no-ops here); :159 `\xeCJK_cs_clear:N`
  // empties a command.
  RawTeX!(
    r"\expandafter\def\csname __xeCJK_msg_new:nn\endcsname{\csname msg_new:nnn\endcsname{xeCJK}}
\expandafter\def\csname __xeCJK_msg_new:nnn\endcsname{\csname msg_new:nnnn\endcsname{xeCJK}}
\expandafter\def\csname xeCJK_add_to_shipout:n\endcsname#1{}
\expandafter\def\csname xeCJK_declare_node:n\endcsname#1{}
\expandafter\def\csname xeCJK_cs_clear:N\endcsname#1{\def#1{}}
\expandafter\def\csname xeCJK_cs_gclear:N\endcsname#1{\gdef#1{}}"
  );
});
