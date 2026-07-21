//! aligned-overset.sty — visually re-centers `\overset`/`\underset` on the
//! alignment point of an amsmath cell (so the *base* lines up on the `&`, not
//! the accent). Purely a PDF-visual cosmetic with no MathML meaning: LaTeXML
//! renders `\overset`/`\underset` as `<mover>`/`<munder>`, which centre
//! intrinsically, so the `\hbox_set:`/`\box_wd:`/`\hspace` box-nudging the
//! package performs is irrelevant to the output.
//!
//! The raw package is an expl3 (`\ProvidesExplPackage`) file that rewrites
//! `\overset` to `\group_align_safe_begin: … \group_align_safe_end:` around an
//! `\hbox_set:` measurement. That alignment-tab group trickery is not something
//! LaTeXML's `\lx@begin@alignment` digestion can follow — when raw-loaded (only
//! under INCLUDE_STYLES / the ar5iv profile; bare it is simply ignored) an
//! `\overset` inside an `align` cell fires `\lx@begin@alignment Attempt to close
//! a group that switched to mode math`, corrupts math mode for the rest of the
//! block, and cascades into hundreds of `unexpected:_`/`^` once `_` is (correctly)
//! catcode SUB. Binding it to keep amsmath's plain `\overset`/`\underset` and drop
//! the cosmetic restores the output.
//!
//! Perl LaTeXML ships no `aligned-overset` binding either — it raw-loads and hangs
//! (witness 2203.05327: same-host Perl 6m19s → `Fatal:timeout:token_limit`, 0 bytes),
//! so this is a beyond-Perl faithfulness win. Witness 2203.05327 (ar5iv): 411
//! errors → 0, 831 KB → full paper.
use latexml_package::prelude::*;

LoadDefinitions!({
  // aligned-overset does `\RequirePackage{xparse,amsmath,mathtools}`. Provide the
  // user-facing math deps (so `\overset`/`\underset` and mathtools commands exist
  // even if the document loads only aligned-overset), but drop xparse — it was
  // pulled in solely for the `\NewExpandableDocumentCommand` that implements the
  // visual cosmetic we intentionally omit.
  RequirePackage!("amsmath");
  RequirePackage!("mathtools");
});
