//! authblk's `\author` gates on an author line's rows (`authblk_sty.rs` `has_separated_names`,
//! `lines_of_names_before_affil`): a piece that is one macro printing a link (`holds_a_macro_printing_a_link`), lines
//! under the names that read as more names (`lines_under_names_read_as_names`), and those lines' breaks written as
//! name separators (`line_breaks_as_name_separators`). Moved unchanged by Z18m, which deleted `author_block/`, from `author_block/rows.rs`.

use super::{space_parted::space_parted_rows_joined, *};

/// Whether `stuff` holds a piece, parted as the kernel's author read parts its names and lines, that is one macro whose
/// expansion prints a link alone (`\newcommand{\githubrepo}{\href{https://github.com/NVIDIA/kvpress}{{\faGithub}
/// NVIDIA/kvpress}\xspace}`, `… \\ \begin{center} \hflink{…}{NVIDIA/KVzap}\quad \githubrepo \end{center}`, 2601.07891): no
/// person, which the read cannot tell, a macro there reading as a name (`\AuthorOne \quad \AuthorTwo`). For a binding that
/// reads an `\author` as one name otherwise, and counts the merge: authblk's.
pub fn holds_a_macro_printing_a_link(stuff: &Tokens) -> bool {
  let separators = author_group_splits()
    .into_iter()
    .chain(author_affil_splits())
    .collect();
  // (without an environment's `\begin{…}`/`\end{…}`, which no part of a piece's text is and which `split_tokens` leaves whole)
  let v = stuff.unlist_ref();
  let mut bare = Vec::with_capacity(v.len());
  let mut at = 0;
  while at < v.len() {
    if (v[at] == T_CS!("\\begin") || v[at] == T_CS!("\\end")) && v.get(at + 1) == Some(&T_BEGIN!())
    {
      at = skip_group(v, at + 1);
    } else {
      bare.push(v[at]);
      at += 1;
    }
  }
  split_tokens(normalize_hspace_separators(Tokens::new(bare)), separators)
    .iter()
    .any(|piece| {
      let mut printed = piece.unlist_ref().iter().filter(|t| **t != T_SPACE!());
      let (Some(only), None) = (printed.next(), printed.next()) else {
        return false;
      };
      only.get_catcode() == Catcode::CS
        && matches!(lookup_definition_stored(only), Ok(Some(Stored::Expandable(definition)))
        if definition.get_expansion().is_some_and(|body| match body {
          ExpansionBody::Tokens(body) => {
            let body: Vec<Token> = body
              .unlist_ref()
              .iter()
              .filter(|t| **t != T_CS!("\\xspace"))
              .copied()
              .collect();
            is_link_line(&Tokens::new(body))
          },
          _ => false,
        }))
    })
}

/// Whether the lines under an unmarked author line's names read as more names and nothing tells them from an
/// institution: each a name by its words, styled as the names line is, with no address, link, note or mark on it
/// (`Ann Able\\ Bob Baker`, `Ann Able\quad Bob Baker\\ Cat Cole`; 2610.38214's `\\ Stefano Soatto` under two rows of
/// names; not `Ann Able\\ Meta AI`, an acronym beside a word in small letters, as `flag_merged_creators` reads a line,
/// nor `\\ \textit{Univ X}`, nor `\\ University of X`). The kernel's read gives such a line to the names above it as
/// their affiliation (Perl's first-line rule), a name made an affiliation where it is one; a binding that reads an
/// `\author` as one name otherwise must not route it there unread: authblk's `\author` (2610.38214). A row of names
/// parted by horizontal space goes on with the names first (`space_parted_rows_joined`), as the kernel reads it.
pub fn lines_under_names_read_as_names(stuff: &Tokens) -> bool {
  let prepared = rewrite_symbol_superscripts(normalize_hspace_separators(strip_linebreak_options(
    drop_empty_marks(stuff.clone()),
  )));
  if has_author_marks(&prepared) {
    return false;
  }
  let joined = space_parted_rows_joined(prepared);
  let lines: Vec<Tokens> = split_tokens(joined, author_line_breaks())
    .into_iter()
    .filter(|line| spacing_end(line.unlist_ref()) > 0 && !prints_nothing(line))
    .collect();
  let Some((names, under)) = lines.split_first() else {
    return false;
  };
  let reads_as_a_name = |line: &Tokens| {
    let text = visible_name_text(line.unlist_ref());
    let bare = outside_notes(line.unlist_ref()).len() == line.unlist_ref().len();
    bare
      && name_count(text.trim()) >= 1
      && !text.contains('@')
      && !holds_mixed_case_acronym(&text)
      && !is_link_line(line)
      && !collaboration_line(line)
      && classify_thanks(&text) == "note"
      && leading_markup(line) == leading_markup(names)
  };
  !under.is_empty() && under.iter().all(reads_as_a_name)
}

/// `stuff` with its top-level line breaks (a `\\`, its options read with it; `\newline`) written as `\quad`: lines
/// under the names read as more names ([`lines_under_names_read_as_names`]), the row they print in.
pub fn line_breaks_as_name_separators(stuff: &Tokens) -> Tokens {
  let mut out: Vec<Token> = Vec::new();
  let mut depth = 0usize;
  for t in strip_linebreak_options(stuff.clone()).unlist() {
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth = depth.saturating_sub(1),
      _ => {},
    }
    if depth == 0 && (t == T_CS!("\\\\") || t == T_CS!("\\newline")) {
      out.push(T_CS!("\\quad"));
    } else {
      out.push(t);
    }
  }
  Tokens::new(out)
}
