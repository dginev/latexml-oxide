//! The text rules on printed ink and the token primitives of `author_block/email_owners.rs`, moved unchanged (the plan's row
//! "Knowledge in, driver out, one pass", user ruling 2026-11-08).

use super::*;

/// A `\\`-delimited line in a no-marker author block is really an EMAIL, not an
/// affiliation, when its visible text is a single bare address: it contains `@`
/// and NO whitespace. Institution names always contain spaces, so this is
/// conservative by construction (near-zero false positives). Relabels the
/// trailing `\texttt{user@host}` line of the NeurIPS idiom, which would
/// otherwise become a bogus `role="affiliation"` (OXIDIZED_DESIGN #52; witness
/// arXiv:2606.00315). Visible text = letters/other chars only; control sequences
/// (`\texttt`, `\textit`, `\{`, `\}`) and grouping are skipped.
pub(in crate::base_utilities) fn line_is_email(line: &Tokens) -> bool {
  let mut visible = String::new();
  for t in line.unlist_ref() {
    if t.code == Catcode::LETTER || t.code == Catcode::OTHER {
      t.with_str(|s| visible.push_str(s));
    } else if t.code == Catcode::SPACE || *t == T_CS!("\\quad") {
      // (the `\quad` of a row of addresses, `dispatch`'s join, parts them as a space does)
      visible.push(' ');
    }
  }
  let v = visible.trim();
  !v.is_empty() && v.contains('@') && !v.chars().any(|c| c.is_whitespace())
}

/// Whether `line` prints nothing: spacing (`leading_spacing_end`, `\vspace{2pt}`), a paragraph break, and the groups, font and size
/// commands around them (`\large{~~}`, 2609.05430); a strut, an alignment declaration, an environment's `\begin{…}` or
/// `\end{…}` itself (`\end{@IEEEauthorhalign}\hfill\strut`, 2309.15562's IEEE block, whose environment name had read as an
/// address, then as an empty affiliation).
pub fn prints_nothing(line: &Tokens) -> bool {
  let v = line.unlist_ref();
  let mut i = 0;
  while i < v.len() {
    let t = &v[i];
    if matches!(
      t.get_catcode(),
      Catcode::BEGIN | Catcode::END | Catcode::SPACE
    ) || *t == T_CS!("\\par")
      || is_spacing_command(t)
      || name_text_command(t)
      || (t.get_catcode() == Catcode::CS
        && t.with_str(|s| {
          matches!(
            s,
            "\\strut" | "\\centering" | "\\raggedright" | "\\raggedleft"
          )
        }))
    {
      i += 1;
    } else if *t == T_CS!("\\begin") || *t == T_CS!("\\end") {
      i = skip_group(v, i + 1).max(i + 1);
    } else if *t == T_CS!("\\vspace") || *t == T_CS!("\\hspace") {
      // (its length, after a star)
      let mut at = i + 1;
      while v
        .get(at)
        .is_some_and(|u| *u == T_SPACE!() || *u == T_OTHER!("*"))
      {
        at += 1;
      }
      i = skip_group(v, at);
    } else {
      match leading_spacing_end(&v[i..]) {
        0 => return false,
        skipped => i += skipped,
      }
    }
  }
  true
}

/// Whether an e-mail address is a pattern for the authors' addresses, not one of them: its local part is only the words
/// of a name template (`FirstName.LastName@SIT.Fraunhofer.de`, `{firstname.lastname}@…`, `name.surname@…`; 2006.12418;
/// `forename.surname@tu-dortmund.de`, 2410.09681, review rev106a S2).
pub(in crate::base_utilities) fn is_pattern_address(address: &str) -> bool {
  let Some((local, _)) = address.split_once('@') else {
    return false;
  };
  let words: Vec<String> = local
    .split(|c: char| !c.is_alphanumeric())
    .filter(|w| !w.is_empty())
    .map(str::to_lowercase)
    .collect();
  !words.is_empty()
    && words.iter().all(|w| {
      matches!(
        w.as_str(),
        "first"
          | "last"
          | "firstname"
          | "lastname"
          | "name"
          | "surname"
          | "givenname"
          | "familyname"
          | "given"
          | "family"
          | "initial"
          | "initials"
          | "fname"
          | "lname"
          | "forename"
          | "forenames"
          | "firstnames"
          | "givennames"
          | "middle"
          | "middlename"
          | "lastnames"
          | "surnames"
          | "familynames"
          | "vorname"
          | "nachname"
          | "prenom"
      )
    })
}

/// Whether `address` is a mailbox its entry names, no one's surname: its local part two letters alone (digits aside,
/// as a short surname spells one, [`NameKeys`]) that are a word the entry prints beside its addresses (`ai@x.org` under
/// "AI Lab, Univ X" and Wei Ai, review rev100b S3). A state code after a comma is no such word: "Cambridge, MA 02138"
/// leaves Jun Ma's `ma@` his (review rev100b2 S3). The Link step reads it as spelling no one.
pub(in crate::base_utilities) fn is_entry_word_mailbox(address: &str, entry: &str) -> bool {
  let local: String = address
    .split('@')
    .next()
    .unwrap_or_default()
    .chars()
    .filter(|c| c.is_alphabetic())
    .flat_map(char::to_lowercase)
    .collect();
  if local.chars().count() != 2 {
    return false;
  }
  // (each word of letters, with whether a comma is the last mark before it)
  let mut words: Vec<(String, bool)> = Vec::new();
  let mut after_comma = false;
  for raw in entry.split_whitespace().filter(|word| !word.contains('@')) {
    let mut word = String::new();
    for c in raw.chars() {
      if c.is_alphabetic() {
        word.push(c);
        continue;
      }
      if !word.is_empty() {
        words.push((std::mem::take(&mut word), after_comma));
      }
      after_comma = c == ',';
    }
    if !word.is_empty() {
      words.push((word, after_comma));
      after_comma = false;
    }
  }
  words.iter().any(|(word, after_comma)| {
    let state_code = *after_comma && word.chars().all(|c| c.is_uppercase());
    word.to_lowercase() == local && !state_code
  })
}
