//! latex.ltx:9521-9522 `\gdef\do#1{\global\let#1\@notprerr}\@preamblecmds`: after `\begin{document}` each listed
//! command IS `\@notprerr`, which raw code tests with `\ifx` (ltnews23.tex:42-43). LaTeXML also reads a package, class
//! or sub-file preamble in the body (Perl's recovery for a body `\usepackage`, a binding's load, standalone and
//! subfiles), which pdflatex never does: there the walk's commands are reopened, and disabled again after it. The walk
//! keeps each meaning under `\lx@onlypreamble@<name>`; the State holds the list (OXIDIZED_DESIGN_DIVERGENCES #226).
use crate::{
  common::store::Stored,
  gullet::unread_after_current_file,
  state::{
    Scope, assign_meaning, assign_value, lookup_bool, lookup_meaning, lookup_tokens, x_equals,
  },
  token::Token,
  tokens::Tokens,
};

/// The commands the walk let to `\@notprerr`, in the walk's order.
const DISABLED: &str = "lx@preamble@disabled";
/// Set while the disabled commands are reopened (a body load or a sub-file preamble), so a nested one leaves them.
const REOPENED: &str = "lx@preamble@reopened";
/// Set while a sub-file's preamble holds the reopening ([`reopen_for_subfile_preamble`]).
const SUBFILE: &str = "lx@preamble@subfile";

fn saved_cs(cs: &Token) -> Token {
  let name = cs.to_string();
  T_CS!(s!("\\lx@onlypreamble@{}", name.trim_start_matches('\\')))
}

/// `\global\let<to><from>`, the engine's own write: no file's lock applies to the kernel's walk.
fn global_let(to: &Token, from: &Token) {
  assign_meaning(
    to,
    lookup_meaning(from).unwrap_or(Stored::None),
    Some(Scope::Global),
  );
}

fn let_notprerr(cs: &Token) {
  global_let(&saved_cs(cs), cs);
  global_let(cs, &T_CS!("\\@notprerr"));
}

/// The walk's `\global\let<cs>\@notprerr`, its meaning kept for a later [`reopen_preamble_commands`].
pub fn disable_preamble_command(cs: &Token) {
  // (once: a second `\begin{document}` walks again, sect02.rs)
  if x_equals(cs, &T_CS!("\\@notprerr")) {
    return;
  }
  let_notprerr(cs);
  let mut list = lookup_tokens(DISABLED)
    .map(Tokens::unlist)
    .unwrap_or_default();
  list.push(*cs);
  assign_value(DISABLED, Tokens::new(list), Some(Scope::Global));
}

/// Give the disabled commands back their meanings (a body load, a sub-file's preamble); false when there is nothing
/// to reopen (still in the preamble, or reopened by an enclosing load).
pub fn reopen_preamble_commands() -> bool {
  if lookup_bool(REOPENED) {
    return false;
  }
  let Some(list) = lookup_tokens(DISABLED) else {
    return false;
  };
  for cs in list.unlist_ref() {
    global_let(cs, &saved_cs(cs));
  }
  assign_value(REOPENED, true, Some(Scope::Global));
  true
}

/// The walk again over the reopened commands: what the load (re)defined is kept for the next reopening.
pub fn close_preamble_commands() {
  if !lookup_bool(REOPENED) {
    return;
  }
  if let Some(list) = lookup_tokens(DISABLED) {
    for cs in list.unlist_ref() {
      let_notprerr(cs);
    }
  }
  assign_value(REOPENED, false, Some(Scope::Global));
}

/// [`reopen_preamble_commands`] for a sub-file's preamble (standalone, subfiles): closed at its `\begin{document}` or,
/// without one, where the sub-file ends (`\lx@subfile@preamble@end`, [`close_subfile_preamble`]).
pub fn reopen_for_subfile_preamble() {
  if reopen_preamble_commands() {
    assign_value(SUBFILE, true, Some(Scope::Global));
    unread_after_current_file(Tokens::new(vec![T_CS!("\\lx@subfile@preamble@end")]));
  }
}

/// The close of the reopening a sub-file's preamble holds; false when it holds none (closed at its `\begin{document}`).
pub fn close_subfile_preamble() -> bool {
  if !lookup_bool(SUBFILE) {
    return false;
  }
  assign_value(SUBFILE, false, Some(Scope::Global));
  close_preamble_commands();
  true
}

/// `\lx@subfile@preamble@end`: a sub-file that ends with no `\begin{document}` ends its preamble there (pdflatex:
/// "File ended while scanning use of \sa@gobble").
pub fn end_subfile_preamble() {
  if close_subfile_preamble() {
    assign_value("inPreamble", false, None);
    assign_value("lx@subpreamble@begindocument", false, None);
  }
}

/// [`reopen_preamble_commands`] for the extent of a load, closed when dropped (also on an Error's early return).
pub struct ReopenedForLoad(bool);

impl ReopenedForLoad {
  pub fn new() -> Self { ReopenedForLoad(reopen_preamble_commands()) }
}

impl Default for ReopenedForLoad {
  fn default() -> Self { Self::new() }
}

impl Drop for ReopenedForLoad {
  fn drop(&mut self) {
    if self.0 {
      close_preamble_commands();
    }
  }
}
