//! Author rows: line-break options, `\hspace` separators, rows of names, spacing runs.

use super::*;

/// [`strip_linebreak_options`], an author block's positive `\\[len]` kept as the gap it prints: written `\\ \\`, the
/// spacer line pdflatex sets the same gap with (`\\ \vspace{len} \\`), which the unmarked branch reads as a gap before
/// the next line ([`name_groups`]: gr-qc9906006's `… Puerto Rico 00931-3343 \\[12pt] Palash B. Pal`; the 85a r1
/// review's finding 6). Only at the block's top level, outside braces and environments (a tabular's row is no line),
/// and not after a row of names, a line holding a wide space or a run of spacing at its top level (`Ivy Ives\;\;\; Jon Jay \\[0.2em] Lou Lee\;\;\; Max Moss \\[0.2em] Oli Ott`,
/// 2605.24414: rows going on with the names line, [`normalize_hspace_separators`]). The first line break of a group of
/// names, the one ending their names line, keeps its gap too, though no lines stand above it for the gap to part: a
/// gap there and before every later line is the block's spacing, parting no authors (`Ann Able \\[2pt] University of X
/// \\[2pt] Hong Kong`, the 85a r2 review's finding 3; [`name_groups`]). A negative or zero length (`\\[-0.1cm]`,
/// `\\[0pt]`) is no gap.
pub(in crate::base_utilities) fn strip_linebreak_options_keeping_gaps(tokens: Tokens) -> Tokens {
  linebreak_options_stripped(tokens, true)
}

/// Whether a line of an author block holds e-mail addresses and nothing else that reads as a word: a bare list, in its
/// wrapper or link (`\texttt{a@x}`, `(a@x)`, `\href{mailto:a@x}{a@x}`), behind an e-mail label (`E-mail: a@x`). No name
/// stands on it, so it names no one ([`resolver::only_addresses`]).
pub(in crate::base_utilities) fn address_only_line(line: &Tokens) -> bool {
  let text = visible_name_text(line.unlist_ref());
  line_is_email_list(line)
    || (!resolver::addresses_in(&text).is_empty() && resolver::only_addresses(&text))
    || email_commands_alone(line)
}

/// Whether a line holds a class's `\email{…}` commands and nothing else (`\email{a@x}\qquad \email{b@y}`, 2404.04659,
/// whose second was a creator).
fn email_commands_alone(line: &Tokens) -> bool {
  let v = line.unlist_ref();
  let mut seen = false;
  let mut i = 0;
  while i < v.len() {
    if v[i] == T_CS!("\\email") && v.get(i + 1) == Some(&T_BEGIN!()) {
      seen = true;
      i = skip_group(v, i + 1);
    } else if v[i].get_catcode() == Catcode::SPACE || is_line_separator(&v[i]) {
      i += 1;
    } else {
      return false;
    }
  }
  seen
}

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

/// Whether a line of an author block prints an icon or a graphic and no word (`\orcidlink{…}`,
/// `\includegraphics{…}`): every command on it an ORCID icon's, one whose arguments print no name
/// ([`NON_PRINTING_ARGUMENTS`]) or a font's, one such at least, no text left. A macro might print a name (`\AuthorOne
/// \quad \AuthorTwo`), which only its expansion tells.
pub(in crate::base_utilities) fn names_no_one(line: &Tokens) -> bool {
  let v = line.unlist_ref();
  let annotation = |t: &Token| {
    t.get_catcode() == Catcode::CS
      && t.with_str(|s| {
        matches!(s, "\\orcidlink" | "\\orcidID" | "\\lx@aas@checkorcid")
          || NON_PRINTING_ARGUMENTS.iter().any(|(c, ..)| *c == s)
      })
  };
  v.iter().any(annotation)
    && v.iter().all(|t| {
      t.get_catcode() != Catcode::CS
        || annotation(t)
        || t.with_str(|s| NAME_TEXT_COMMANDS.contains(&s))
    })
    && !visible_name_text(&printed_words(v))
      .chars()
      .any(char::is_alphanumeric)
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

/// Whether `tokens` print only the word "and", in any case and wrapper (`and`, `\textnormal{AND}`, `{and}`): the
/// separator between two names, never a name or an affiliation.
pub(in crate::base_utilities) fn is_and_piece(tokens: &[Token]) -> bool {
  visible_name_text(tokens).trim().eq_ignore_ascii_case("and")
}
