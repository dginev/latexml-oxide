//! The text rules on printed ink and the token primitives of `author_block/names_split.rs`, moved unchanged (MV1,
//! user ruling 2026-11-08 "knowledge in, driver out").

use super::*;

/// The index in `body` past the glue specification starting at `i` (after `\hskip` or `\kern`): a dimension — a
/// number and a unit, two letters or `fil`, `fill`, `filll` — then `plus` and `minus` dimensions (`1em plus 1fil`).
pub(in crate::base_utilities) fn past_glue(body: &[Token], i: usize) -> usize {
  let letter_at = |j: usize, c: char| {
    body.get(j).is_some_and(|t| {
      t.get_catcode() == Catcode::LETTER && t.with_str(|s| s.len() == 1 && s.starts_with(c))
    })
  };
  let past_spaces = |mut j: usize| {
    while body.get(j) == Some(&T_SPACE!()) {
      j += 1;
    }
    j
  };
  let past_dimension = |j: usize| {
    let mut j = past_spaces(j);
    while body
      .get(j)
      .is_some_and(|u| u.with_str(|s| s.chars().all(|c| c.is_ascii_digit() || ".+-".contains(c))))
    {
      j += 1;
    }
    j = past_spaces(j);
    if letter_at(j, 'f') && letter_at(j + 1, 'i') && letter_at(j + 2, 'l') {
      j += 3;
      while letter_at(j, 'l') {
        j += 1;
      }
    } else {
      for _ in 0..2 {
        if body
          .get(j)
          .is_some_and(|u| u.get_catcode() == Catcode::LETTER)
        {
          j += 1;
        }
      }
    }
    j
  };
  let mut i = past_dimension(i);
  for keyword in ["plus", "minus"] {
    let j = past_spaces(i);
    if keyword
      .chars()
      .enumerate()
      .all(|(k, c)| letter_at(j + k, c))
    {
      i = past_dimension(j + keyword.len());
    }
  }
  i
}

/// Where a piece's leading spacing ends: spaces, ties, space commands (`\ `, `\quad`) and `\vspace`/`\hspace` with
/// their length.
pub(in crate::base_utilities) fn leading_spacing_end(v: &[Token]) -> usize {
  let mut i = 0;
  while i < v.len() {
    if is_spacing(&v[i]) || PARAMETERLESS_SPACING.iter().any(|cs| v[i] == T_CS!(*cs)) {
      i += 1;
    } else if v[i] == T_CS!("\\penalty") {
      // an integer follows (tex.web §1103 `\penalty` scans an int: `\penalty10000`, `\penalty -50`)
      let mut j = i + 1;
      while v
        .get(j)
        .is_some_and(|t| *t == T_SPACE!() || *t == T_OTHER!("-") || *t == T_OTHER!("+"))
      {
        j += 1;
      }
      while v.get(j).is_some_and(|t| {
        t.get_catcode() == Catcode::OTHER && t.with_str(|s| s.chars().all(|c| c.is_ascii_digit()))
      }) {
        j += 1;
      }
      i = j;
    } else if ["\\vskip", "\\hskip", "\\kern"]
      .iter()
      .any(|cs| v[i] == T_CS!(*cs))
    {
      // a glue or dimension follows (tex.web §1057, §1061: `\kern2pt`, `\vskip 1em plus 1fil`)
      i = past_glue(v, i + 1);
    } else if v[i] == T_CS!("\\rule") {
      // latex.ltx:16359 `\rule[raise]{width}{height}`, a rule box, no glyph
      let mut j = i + 1;
      if v.get(j) == Some(&T_OTHER!("[")) {
        while j < v.len() && v[j] != T_OTHER!("]") {
          j += 1;
        }
        j += 1;
      }
      for _ in 0..2 {
        if v.get(j).is_none_or(|t| t.get_catcode() != Catcode::BEGIN) {
          break;
        }
        j = skip_group(v, j).min(v.len());
      }
      i = j;
    } else if v[i] == T_CS!("\\vspace") || v[i] == T_CS!("\\hspace") || v[i] == T_CS!("\\addvspace")
    {
      let mut j = i + 1;
      if v.get(j) == Some(&T_OTHER!("*")) {
        j += 1;
      }
      while v.get(j) == Some(&T_SPACE!()) {
        j += 1;
      }
      // (only with its braced length: `\vspace\baselineskip` stops here, its length no wrapper)
      if v.get(j).is_none_or(|t| t.get_catcode() != Catcode::BEGIN) {
        break;
      }
      i = skip_group(v, j).min(v.len());
    } else {
      break;
    }
  }
  i
}

/// Whether `text` reads as a person's name whose last word, an institution's ([`non_name_word`]: `Centers`,
/// `College`, `Fellows`), follows the initial of a given name: two to five capitalised words, the one before the last
/// an initial (`Gary P. Centers`; not `W. M. Keck Observatory`, whose institution word follows a name), no other
/// institution word, no digit or address. In a list by its marks (`author_block`'s `marked_names`; the reading's
/// `marked_name`) such a piece is a marked name (rows N2-N3; moved from `author_block/names_split.rs`).
pub(in crate::base_utilities) fn surname_after_initial(text: &str) -> bool {
  let text = text
    .trim()
    .trim_end_matches(|c: char| ",;*".contains(c))
    .trim_end();
  if text
    .chars()
    .any(|c| c.is_ascii_digit() || "@:/()".contains(c))
  {
    return false;
  }
  let words: Vec<&str> = text.split_whitespace().collect();
  // (a place's abbreviation is no initial: `St.`, `Mt.`, `Ft.`)
  let initial = |w: &str| {
    let mut chars = w.chars();
    if ["St.", "Mt.", "Ft."].contains(&w) {
      return false;
    }
    chars.next().is_some_and(char::is_uppercase)
      && chars
        .as_str()
        .strip_suffix('.')
        .is_some_and(|rest| rest.chars().all(char::is_lowercase) && rest.len() <= 1)
  };
  let capitalised = |w: &&str| w.chars().next().is_some_and(char::is_uppercase);
  (3..=5).contains(&words.len())
    && words.iter().all(capitalised)
    && initial(words[words.len() - 2])
    && non_name_word(words[words.len() - 1])
    && !words[..words.len() - 1].iter().any(|w| non_name_word(w))
}
