//! The markup a line opens with (`leading_markup`, which authblk's gate `rows::lines_under_names_read_as_names` reads). Moved unchanged by Z18m, which deleted `author_block/`, from `author_block/name_text.rs`.

use super::*;

/// The markup an author line opens with, before its first letter: its control sequences and the groups they open
/// (`{\small\em Bartol …}` → `{`, `\small`, `\em`; `T. Stelzer` → none). Lines styled alike share it.
pub(in crate::base_utilities) fn leading_markup(line: &Tokens) -> Vec<Token> {
  line
    .unlist_ref()
    .iter()
    .take_while(|t| t.get_catcode() != Catcode::LETTER)
    .filter(|t| matches!(t.get_catcode(), Catcode::CS | Catcode::BEGIN))
    .copied()
    .collect()
}
