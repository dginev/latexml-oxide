//! uiucthesis.cls: the class is loaded raw. Its `\title`/`\author` (uiucthesis.cls:77-80) also store uppercase copies,
//! `\@Utitle`/`\@Uauthor`, which its title page prints (:162-169); LaTeXML's own `\title`/`\author` replace the
//! class's setters (the title and authors are frontmatter), so the copies are never made and the replay gate
//! rejected the title page (`\lx@deposit@maketitle`, sect05.rs) — its degree statement, schools and
//! Urbana-Champaign line were lost (thesis-ex 86.1 %). The copies are empty here, as the replay empties every store
//! the frontmatter holds. Witness uiucthesis/thesis-ex. Guard
//! `perfect_kernel_batch61::uiucthesis_title_page_is_replayed`.
use latexml_package::prelude::*;

LoadDefinitions!({
  InputDefinitions!("uiucthesis", noltxml => true, extension => Some(Cow::Borrowed("cls")), handleoptions => true);
  RawTeX!(r"\providecommand\@Utitle{}\providecommand\@Uauthor{}");
});
