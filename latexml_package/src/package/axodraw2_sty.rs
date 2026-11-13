//! axodraw2.sty — the TeX Live successor of axodraw v1, raw-loaded.
//!
//! axodraw2 is a different package (keyval options on the drawing commands,
//! `{axopicture}`, `\Line[arrow](…)`), drawn through its own `axohelp` or
//! PostScript code. It has no binding; this entry only keeps the request from
//! the versioned-name fallback (content.rs `find_file_fallback`, Perl
//! Package.pm:2118-2121), which strips the `2` and would answer
//! `\usepackage{axodraw2}` with the v1 binding (`axodraw_sty.rs`): the real
//! file is read, as it was before v1 had a binding (1701.01404).
use crate::prelude::*;

LoadDefinitions!({
  InputDefinitions!("axodraw2", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
