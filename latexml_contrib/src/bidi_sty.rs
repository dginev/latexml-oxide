//! bidi.sty — bidirectional typesetting for XeLaTeX (no Perl binding).
//!
//! bidi drives XeTeX's direction primitives and inter-character token classes
//! (bidi.sty:348-357 `\XeTeXinterchartokenstate`, `\XeTeXcharclass` over the
//! digits); under the Unicode-native default persona those run on undefined
//! engine primitives (the sweep-57 error flood). The text is already Unicode
//! characters in logical order, so the direction commands keep their content
//! and drop the direction switch (arXiv 2605.19069, 2605.25928: `\RL{…}`,
//! `\LR{…}`). Guard:
//! `perfect_kernel_batch56::xetex_only_packages_load_under_the_default_persona`.
use latexml_package::prelude::*;

LoadDefinitions!({
  // Direction-wrapped text: the argument, as is.
  DefMacro!("\\RL{}", "#1");
  DefMacro!("\\LR{}", "#1");
  DefMacro!("\\RLE{}", "#1");
  DefMacro!("\\LRE{}", "#1");
  DefMacro!("\\LTRbox{}", "\\mbox{#1}");
  DefMacro!("\\RTLbox{}", "\\mbox{#1}");
  // Paragraph/document direction switches and their environments.
  for cs in [
    "\\setRTL",
    "\\setLTR",
    "\\setRL",
    "\\setLR",
    "\\unsetRL",
    "\\unsetLR",
    "\\RTLdocument",
    "\\LTRdocument",
    "\\beginR",
    "\\endR",
    "\\beginL",
    "\\endL",
    "\\RTLmulticolcolumns",
    "\\LTRmulticolcolumns",
  ] {
    def_macro_noop(cs)?;
  }
  DefEnvironment!("{RTL}", "#body");
  DefEnvironment!("{LTR}", "#body");
});
