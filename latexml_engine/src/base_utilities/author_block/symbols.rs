//! Footnote-symbol marks and the superscript rewrites (symbols, ordinals, in-word).

use super::*;

/// Does `line` open (after spaces, math shifts and braces) with a footnote symbol (`* Equal contribution`,
/// `$\dagger$ Corresponding author`, `\dag Corresponding author`) — a legend for the authors' symbol marks, not an
/// affiliation line? The text-mode `\dag`/`\ddag` count here only: as superscript marks (`$^\dag$`) they still label
/// affiliations (`is_footnote_symbol_operand` leaves them out, so `rewrite_symbol_superscripts` keeps them as marks).
pub(in crate::base_utilities) fn starts_with_footnote_symbol(line: &Tokens) -> bool {
  line
    .unlist_ref()
    .iter()
    .find(|t| {
      !matches!(
        t.get_catcode(),
        Catcode::SPACE | Catcode::MATH | Catcode::BEGIN | Catcode::END
      )
    })
    .is_some_and(|t| {
      *t == T_CS!("\\dag")
        || *t == T_CS!("\\ddag")
        || is_footnote_symbol_operand(std::slice::from_ref(t))
    })
}

/// A legend-shaped `line` that is an affiliation its symbol labels: led by a symbol mark [`keepsup`] made
/// (`$^{\diamond}$ National University of Singapore`), the symbol one an author of the block carries (`carried`, the
/// authors' [`symbol_marks`]), and the rest reading as an affiliation ([`reads_as_affiliation`]): the line with that mark
/// written back as the superscript it was, which `\lx@affiliation@withsup` labels the affiliation by, so it goes to the
/// authors carrying the symbol as a numbered one goes to the authors carrying the number (2601.05633's ⋄ line under its ♡
/// line, to Yang Zhang, Zhiyong Huang and Tat-Seng Chua, not the last author; batch 77p r6). A legend
/// (`$^{*}$ Equal contribution`) or a symbol no author carries stays as it is.
pub(in crate::base_utilities) fn symbol_labelled_affiliation(
  line: &Tokens,
  carried: &[String],
) -> Option<Tokens> {
  let v = line.unlist_ref();
  let lead = v.len() - after_leading_markup(&v[leading_spacing_end(v)..]).len();
  if v.get(lead) != Some(&T_CS!("\\lx@frontmatter@keepsup")) {
    return None;
  }
  let shown_end = skip_group(v, lead + 1);
  if v.get(shown_end) != Some(&T_CS!("\\lx@frontmatter@symbolmark")) {
    return None;
  }
  let end = skip_group(v, shown_end + 1);
  let mark = symbol_marks(&Tokens::new(v[shown_end..end].to_vec()));
  let rest = Tokens::new(v[end..].to_vec());
  if !mark.first().is_some_and(|mark| carried.contains(mark)) || !reads_as_affiliation(&rest) {
    return None;
  }
  let mut out = v[..lead].to_vec();
  out.push(T_CS!("\\textsuperscript"));
  out.extend_from_slice(&v[lead + 1..shown_end]);
  out.extend(rest.unlist());
  Some(Tokens::new(out))
}

/// Whether `line` is an institution no mark leads ([`reads_as_affiliation`]): not led, past its spacing and font
/// commands, by a footnote symbol or the visible superscript [`rewrite_symbol_superscripts`] makes one — a symbol-led line
/// is its symbol's carriers', or a legend (2604.02578's `\textsuperscript{*}Department of Computer Science` under names
/// marked `*†`, `†‡`, `*†‡`; 2604.03394's `\small ${\dag}$ Department …`) — nor by math (`$(a)$Univ A`, a mark no author
/// requests); no note naming the corresponding author; and an institution by a word naming one, not by a role or contact
/// word alone (`ROLE_WORDS`: "This research was supported by …", "for the ATLAS Collaboration", "Corresponding author:
/// Professor …", the rev77t review's r4, r6, r8).
pub(in crate::base_utilities) fn unmarked_institution_line(line: &Tokens) -> bool {
  /// The words [`NON_NAME_WORDS`] holds for a role or a contact, not an institution.
  const ROLE_WORDS: &[&str] = &[
    "research",
    "professor",
    "collaboration",
    "member",
    "team",
    "email",
    "e-mail",
  ];
  let v = line.unlist_ref();
  let lead = after_leading_markup(&v[leading_spacing_end(v)..]);
  let text = visible_name_text(v);
  reads_as_affiliation(line)
    && lead.first() != Some(&T_CS!("\\lx@frontmatter@keepsup"))
    && lead.first().map(Token::get_catcode) != Some(Catcode::MATH)
    && !starts_with_footnote_symbol(&Tokens::new(lead.to_vec()))
    && !names_correspondence(&text)
    && text.split_whitespace().any(|word| {
      non_name_word(word) && !word_forms(word).any(|form| ROLE_WORDS.contains(&form.as_str()))
    })
}

/// A line led by a mark in parentheses set in math, `$(a)$Department of Physics, …`, whose letter or number an author of
/// the block requests as its superscript mark (`requested`: `Etsuko Itou$^a$`): the line with that mark written as the
/// superscript `\lx@affiliation@withsup` labels the affiliation by, so it goes to the authors carrying it as a `$^a$`-led
/// line does (1011.0516, PoS.cls, five such lines welded into the last name). A parenthesized lead no author requests
/// stays the line's text.
pub(in crate::base_utilities) fn parenthesized_mark_line(
  line: &Tokens,
  requested: &[String],
) -> Option<Tokens> {
  let v = line.unlist_ref();
  let lead = leading_spacing_end(v);
  let rest = &v[lead..];
  let close = rest
    .iter()
    .skip(1)
    .position(|t| t.get_catcode() == Catcode::MATH)?
    + 1;
  if rest.first().map(Token::get_catcode) != Some(Catcode::MATH) {
    return None;
  }
  let inner: Vec<Token> = rest[1..close]
    .iter()
    .copied()
    .filter(|t| *t != T_SPACE!())
    .collect();
  let [open, label @ .., shut] = inner.as_slice() else {
    return None;
  };
  if *open != T_OTHER!("(")
    || *shut != T_OTHER!(")")
    || label.is_empty()
    || !label.iter().all(|t| {
      matches!(t.get_catcode(), Catcode::LETTER | Catcode::OTHER)
        && t.with_str(|s| s.chars().all(char::is_alphanumeric))
    })
  {
    return None;
  }
  let mark = Tokens::new(label.to_vec()).to_string();
  if !requested.contains(&mark) {
    return None;
  }
  let mut out = v[..lead].to_vec();
  out.extend([T_CS!("\\textsuperscript"), T_BEGIN!()]);
  out.extend_from_slice(label);
  out.push(T_END!());
  out.extend_from_slice(&rest[close + 1..]);
  Some(Tokens::new(out))
}

/// Whether `line` is a legend: a footnote symbol ([`starts_with_footnote_symbol`]), also as the visible superscript
/// [`rewrite_symbol_superscripts`] makes it (`\textsuperscript{*}\textit{Equal contribution.}`, 2509.20748), and text
/// after it.
pub(in crate::base_utilities) fn is_legend_line(line: &Tokens) -> bool {
  let v = line.unlist_ref();
  let v = after_leading_markup(&v[leading_spacing_end(v)..]);
  (starts_with_footnote_symbol(&Tokens::new(v.to_vec()))
    || v.first() == Some(&T_CS!("\\lx@frontmatter@keepsup")))
    && has_name_letter(v)
}

/// The footnote-symbol marks [`keepsup`] wrote into `tokens` (the `\lx@frontmatter@symbolmark` operands, as text), at
/// any depth: `Ann Able$^{\dagger}$` and `$^{\dagger}$Univ A` both hold `\dagger`.
pub(in crate::base_utilities) fn symbol_marks(tokens: &Tokens) -> Vec<String> {
  let v = tokens.unlist_ref();
  let mut marks = Vec::new();
  for (i, t) in v.iter().enumerate() {
    if *t == T_CS!("\\lx@frontmatter@symbolmark") && v.get(i + 1) == Some(&T_BEGIN!()) {
      let mut depth = 0usize;
      let mut operand: Vec<Token> = Vec::new();
      for &u in &v[i + 1..] {
        match u.get_catcode() {
          Catcode::BEGIN => depth += 1,
          Catcode::END => depth -= 1,
          _ => {},
        }
        if depth == 0 {
          break;
        }
        if !(depth == 1 && u.get_catcode() == Catcode::BEGIN) {
          operand.push(u);
        }
      }
      let mark = Tokens::new(operand).to_string();
      let mark = mark.trim();
      if !mark.is_empty() {
        marks.push(mark.to_string());
      }
    }
  }
  marks
}
