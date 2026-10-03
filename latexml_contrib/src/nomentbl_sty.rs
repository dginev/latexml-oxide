//! nomentbl: the raw package over the nomencl binding, its entries carrying a unit and a note.
//!
//! nomentbl.sty is a package of its own, not nomencl's `nomentbl` option: it loads nomencl
//! (`\RequirePackageWithOptions{nomencl}`, nomentbl.sty:33) and redefines the writer to four arguments,
//! `{symbol}{description}{unit}{note}` (:136-142), the unit typeset as written (it loads no siunitx). The nomencl
//! binding reads an entry by `\if@nomentbl` (nomencl_sty.rs), which nomentbl never sets, so the unit and the note
//! spilled into the text ("mSI base quantity"; nomentbl manual).
use latexml_package::prelude::*;

LoadDefinitions!({
  InputDefinitions!("nomentbl", noltxml => true, extension => Some(Cow::Borrowed("sty")));
  RawTeX!(
    r"\@nomentbltrue\def\lx@nomencl@unit#1{\IfBlankF{#1}{#1}}\let\lx@nomencl@ownnomgroup\nomgroup"
  );
});
