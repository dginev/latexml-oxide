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

/// Rewrite footnote-SYMBOL author superscripts — `$^{*}$`, `${}^{\dagger}$`,
/// `\textsuperscript{\ddagger}`, a bare `^{*}` — onto the visible
/// `\lx@frontmatter@keepsup` sentinel, BEFORE author-block branch selection.
///
/// Two effects, both wanted (OXIDIZED_DESIGN #52; witness arXiv:2506.06941, where
/// Iman Mirzadeh's literal `$^{*}$` was silently dropped):
///   * the mark no longer counts as an affiliation superscript, so a block whose
///     ONLY superscript is such a note-mark takes the clean no-marker author
///     branch instead of the affiliation-linking one;
///   * the symbol renders as a real superscript instead of being consumed into an
///     `affiliation:*` label that matches no affiliation and is discarded.
///
/// A number or letter glued to such a symbol (`$^{1*}$`, `\textsuperscript{a\dagger}`, the symbol first too:
/// `\textsuperscript{*1}`, 2505.22662) is the affiliation mark it is and the symbol beside it (`split_glued_symbol`): the
/// mark requests affiliation 1, the symbol is shown, as for `$^{1,*}$` (else the one request `affiliation:1*` matched no
/// affiliation, and affiliation 1 went to no one; authblk's `\author{Ann$^{1*}$}`, 77n r3; the neurips fixture's Wenqi
/// Shao, 2507.06670). Both are written in the order they print, for an unanswered mark to show there
/// (`show_unanswered_marks`: 2610.07087's `$^{1*}$`). The symbol is shown as written: a math `*` as `∗`, as the unsplit
/// `$^{1,*}$` keeps it (2210.14772, 2508.09827, 2505.11121; [`keepsup_in_math`]), a text one as `*`.
///
/// NUMERIC/lettered affiliation marks (`$^{1}$`, `\textsuperscript{a}`) are left
/// untouched, so affiliation linking is unaffected. Only the specific
/// superscript-mark token shapes are matched; a superscript inside real math
/// (`$x^2$`) is not (its base is not empty), so math content is preserved.
pub(in crate::base_utilities) fn rewrite_symbol_superscripts(tokens: Tokens) -> Tokens {
  let src = tokens.unlist();
  let n = src.len();
  let mut out: Vec<Token> = Vec::with_capacity(n);
  let mut i = 0;
  // Read a superscript operand at `src[k]`: a braced `{...}` group, or a single
  // token. Returns (operand tokens, index past the operand) or None.
  let read_operand = |k: usize| -> Option<(Vec<Token>, usize)> {
    if k >= n {
      return None;
    }
    if src[k] == T_BEGIN!() {
      let mut depth = 1usize;
      let mut m = k + 1;
      let mut inner = Vec::new();
      while m < n && depth > 0 {
        if src[m] == T_BEGIN!() {
          depth += 1;
        } else if src[m] == T_END!() {
          depth -= 1;
          if depth == 0 {
            break;
          }
        }
        inner.push(src[m]);
        m += 1;
      }
      if depth == 0 {
        Some((inner, m + 1))
      } else {
        None
      }
    } else {
      Some((vec![src[k]], k + 1))
    }
  };
  while i < n {
    let t = src[i];
    // `\textsuperscript{sym}` — always braced in practice.
    if t == T_CS!("\\textsuperscript")
      && src.get(i + 1) == Some(&T_BEGIN!())
      && let Some((sym, next)) = read_operand(i + 1)
      && is_footnote_symbol_operand(&sym)
    {
      out.extend(keepsup(sym));
      i = next;
      continue;
    }
    if t == T_CS!("\\textsuperscript")
      && src.get(i + 1) == Some(&T_BEGIN!())
      && let Some((operand, next)) = read_operand(i + 1)
      && let Some(glued) = glued_marks(&operand)
    {
      for part in glued {
        match part {
          GluedPart::Symbol(symbol) => out.extend(keepsup(symbol)),
          GluedPart::Marks(marks) => {
            out.extend([t, T_BEGIN!()]);
            out.extend(marks);
            out.push(T_END!());
          },
        }
      }
      i = next;
      continue;
    }
    // `$ [ {} ] ^ sym $` — a math span whose whole content is a superscript on an
    // empty base. Rewrite the entire span (delimiters included) to a text sup.
    if t == T_MATH!()
      && let Some(close) = (i + 1..n).find(|&m| src[m] == T_MATH!())
    {
      // optional empty base `{}` before the superscript
      let p = if i + 2 < close && src[i + 1] == T_BEGIN!() && src[i + 2] == T_END!() {
        i + 3
      } else {
        i + 1
      };
      if p < close
        && src[p] == T_SUPER!()
        && let Some((sym, next)) = read_operand(p + 1)
        && next == close
        && is_footnote_symbol_operand(&sym)
      {
        out.extend(keepsup(sym));
        i = close + 1;
        continue;
      }
      if p < close
        && src[p] == T_SUPER!()
        && let Some((operand, next)) = read_operand(p + 1)
        && next == close
        && let Some(glued) = glued_marks(&operand)
      {
        for part in glued {
          match part {
            GluedPart::Symbol(symbol) => out.extend(keepsup_in_math(symbol)),
            GluedPart::Marks(marks) => {
              out.extend_from_slice(&src[i..=p]);
              out.push(T_BEGIN!());
              out.extend(marks);
              out.extend([T_END!(), T_MATH!()]);
            },
          }
        }
        i = close + 1;
        continue;
      }
    }
    // Only the math-delimited (`$…$`) and `\textsuperscript{…}` spellings are
    // rewritten — the correct ways to write a text superscript. A bare `^` is NOT
    // matched here: outside math it is invalid LaTeX, and a `^` reached mid-scan is
    // the superscript operator of real inline math on a non-empty base (`$a^{*}$`),
    // which must be left intact.
    out.push(t);
    i += 1;
  }
  Tokens::new(out)
}

/// `tokens` with a mark that leads a line of labelled entries written without the symbols glued to it (`$^{*1}$Kyoto
/// University`, `$^{1*}$Univ X`, 2606.23258): the mark labels the line, the symbol with it, and a line prints no label in
/// its text — it read "* Univ X", or matched no name once the names' `*1` asks for 1 and `*` apart (`glued_marks`).
/// From the first line if `first_line` (an affiliation list), else from after the first line break (an author block,
/// whose first line is names, its marks after them).
pub fn glued_line_marks_as_marks(tokens: Tokens, first_line: bool) -> Tokens {
  let v = tokens.unlist_ref();
  if !v
    .iter()
    .any(|t| *t == T_SUPER!() || *t == T_CS!("\\textsuperscript"))
  {
    return tokens;
  }
  let mut out: Vec<Token> = Vec::with_capacity(v.len());
  // (the group depth; whether a letter came since the last separator; whether the names line is over)
  let (mut depth, mut lettered, mut broke) = (0usize, !first_line, first_line);
  let mut i = 0;
  while i < v.len() {
    let t = v[i];
    if depth == 0 && !lettered {
      // (`\textsuperscript{…}`; `$^{…}$`, `${}^{…}$`)
      let at = match t.get_catcode() {
        Catcode::MATH => {
          let p = i
            + 1
            + usize::from(v.get(i + 1) == Some(&T_BEGIN!()) && v.get(i + 2) == Some(&T_END!())) * 2;
          (v.get(p) == Some(&T_SUPER!()) && v.get(p + 1) == Some(&T_BEGIN!())).then_some(p + 1)
        },
        _ if t == T_CS!("\\textsuperscript") && v.get(i + 1) == Some(&T_BEGIN!()) => Some(i + 1),
        _ => None,
      };
      if let Some(open) = at {
        let close = skip_group(v, open);
        let math = t.get_catcode() == Catcode::MATH;
        let end = if math { close + 1 } else { close };
        if close <= v.len()
          && (!math || v.get(close) == Some(&T_MATH!()))
          && let Some(glued) = glued_marks(&v[open + 1..close - 1])
        {
          out.extend_from_slice(&v[i..=open]);
          for marks in glued.into_iter().filter_map(|part| match part {
            GluedPart::Marks(marks) => Some(marks),
            GluedPart::Symbol(_) => None,
          }) {
            out.extend(marks);
          }
          out.push(T_END!());
          if math {
            out.push(T_MATH!());
          }
          i = end;
          continue;
        }
      }
    }
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth = depth.saturating_sub(1),
      Catcode::LETTER => lettered = true,
      _ if depth == 0 && is_line_separator(&t) => {
        broke |= first_line || t == T_CS!("\\\\") || t == T_CS!("\\newline");
        lettered &= !broke;
      },
      _ => {},
    }
    out.push(t);
    i += 1;
  }
  Tokens::new(out)
}

/// [`keepsup`] for a symbol glued to a mark in math (`$^{1*}$`, `$^{1,2*}$`, 2505.11121): a math `*` prints `∗`, shown
/// so (FIDELITY_CHARTER §4, "stays as written"; `\textsuperscript{1*}` keeps its text `*`), its key the `*` the entries
/// answer; beside a second symbol too (`$^{2,1*\dagger}$` prints `∗†`: 2404.16821, 2505.17017). A lone `$^{*}$` shows
/// `*` as it always has, in [`rewrite_symbol_superscripts`]' own branch.
fn keepsup_in_math(sym: Vec<Token>) -> Vec<Token> {
  let shown = sym
    .iter()
    .map(|token| {
      if *token == T_OTHER!("*") {
        T_CS!("\\ast")
      } else {
        *token
      }
    })
    .collect();
  let mut v = visible_sup(shown);
  v.extend([T_CS!("\\lx@frontmatter@symbolmark"), T_BEGIN!()]);
  v.extend(sym);
  v.push(T_END!());
  v
}

/// `\lx@frontmatter@keepsup{<sym>}` as a token list: a superscript shown as it is, requesting nothing.
fn visible_sup(sym: Vec<Token>) -> Vec<Token> {
  let mut v = vec![T_CS!("\\lx@frontmatter@keepsup"), T_BEGIN!()];
  v.extend(sym);
  v.push(T_END!());
  v
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

/// A footnote-symbol author mark: shown (`visible_sup`), and requesting the affiliation of its symbol
/// (`\lx@frontmatter@symbolmark`).
pub(in crate::base_utilities) fn keepsup(sym: Vec<Token>) -> Vec<Token> {
  let mut v = visible_sup(sym.clone());
  v.extend([T_CS!("\\lx@frontmatter@symbolmark"), T_BEGIN!()]);
  v.extend(sym);
  v.push(T_END!());
  v
}

/// Rewrite an ordinal suffix superscript after a numeral (`5\textsuperscript{th}`, `21$^{st}$`) onto a visible
/// superscript: it is the number's, not an affiliation mark, so a marked list is not split at it nor labelled by it
/// (repro sectioning-frontmatter/affiliation_line_superscript_ordinal_is_not_a_mark; iopart's
/// `\address{$^1$Univ A, 5 \textsuperscript{th} floor}`, 2609.01831).
pub(in crate::base_utilities) fn rewrite_ordinal_superscripts(tokens: Tokens) -> Tokens {
  let is_ordinal = is_ordinal_suffix;
  let is_digit = |t: &Token| {
    t.get_catcode() == Catcode::OTHER
      && t.with_str(|s| !s.is_empty() && s.chars().all(|c| c.is_ascii_digit()))
  };
  let after_numeral = |out: &[Token]| {
    out
      .iter()
      .rev()
      .find(|t| **t != T_SPACE!())
      .is_some_and(is_digit)
  };
  // The suffix is shown in text: a math-only wrapper takes its text form (`$^{\mathrm{th}}$`).
  let text_mode = |operand: Vec<Token>| -> Vec<Token> {
    operand
      .into_iter()
      .map(|t| {
        if t == T_CS!("\\mathrm") {
          T_CS!("\\textrm")
        } else if t == T_CS!("\\mathup") {
          T_CS!("\\textup")
        } else if t == T_CS!("\\mathit") {
          T_CS!("\\textit")
        } else if t == T_CS!("\\mathbf") {
          T_CS!("\\textbf")
        } else if t == T_CS!("\\mathsf") {
          T_CS!("\\textsf")
        } else {
          t
        }
      })
      .collect()
  };
  let src = tokens.unlist();
  let mut out: Vec<Token> = Vec::with_capacity(src.len());
  let mut i = 0;
  while i < src.len() {
    let t = src[i];
    if t == T_CS!("\\textsuperscript")
      && let Some((operand, next)) = sup_operand_at(&src, i + 1)
      && is_ordinal(&operand)
      && after_numeral(&out)
    {
      out.extend(visible_sup(text_mode(operand)));
      i = next;
      continue;
    }
    if t == T_MATH!() {
      // `$^{th}$` after a numeral, or `$5^{th}$` with the numeral inside: the digits stay, the suffix is shown.
      let digits_end = (i + 1..src.len())
        .find(|&k| !is_digit(&src[k]))
        .unwrap_or(src.len());
      if src.get(digits_end) == Some(&T_SUPER!())
        && let Some((operand, next)) = sup_operand_at(&src, digits_end + 1)
        && src.get(next) == Some(&T_MATH!())
        && is_ordinal(&operand)
        && (digits_end > i + 1 || after_numeral(&out))
      {
        out.extend_from_slice(&src[i + 1..digits_end]);
        out.extend(visible_sup(text_mode(operand)));
        i = next + 1;
        continue;
      }
    }
    out.push(t);
    i += 1;
  }
  Tokens::new(out)
}

/// Rewrite a superscript inside a word, a letter on each side of it (`Craig M$^{\mathrm{c}}$Rae`, 2505.09041;
/// `M\textsuperscript{c}Donald`; `I$^2$R`, `NLP$^2$C`), onto a visible superscript: it is the name's spelling, not an
/// affiliation mark, which follows a name — read as one, its letter was a request and left the name ("Craig MRae", the
/// "c" shown after it). Not a mark list (`$^{1,3}$`: a list of marks is never spelling) nor a mark glued to the word
/// "and" after it (`Hiroshi Ito$^{1,3}$and Tetsuya Ogata`, 2605.00471; `Bipasha Kundu$^{1}$and Cristian Linte`,
/// 2604.27101): those are the name's marks, which kept as its text lost their affiliations (77n r3 review).
pub(in crate::base_utilities) fn rewrite_in_word_superscripts(tokens: Tokens) -> Tokens {
  let src = tokens.unlist();
  let letter = |t: Option<&Token>| t.is_some_and(|t| t.get_catcode() == Catcode::LETTER);
  // (the operand a mark's: a list of marks, or the word after it "and", the separator before the next name; `Andrew`
  // is a word, 77n r4 review probe a8. Only "and" itself: `Ann Able$^{1}$And Bob Baker$^{2}$`, `AND`, stays one name,
  // which `flag_merged_creators` reports — read as a mark, its "And" merged the names silently, 77n r5 review probes
  // a1, a6)
  let marks = |operand: &[Token], after: usize| {
    operand.contains(&T_OTHER!(","))
      || (src.get(after..after + 3) == Some(&[T_LETTER!("a"), T_LETTER!("n"), T_LETTER!("d")][..])
        && !letter(src.get(after + 3)))
  };
  // (the letters before it a word, not a length's unit: `\kern2pt$^{1}$Univ A` is a spacing, then a mark; 2407.15815)
  let after_a_word = |out: &[Token]| {
    let word = out
      .iter()
      .rev()
      .take_while(|t| t.get_catcode() == Catcode::LETTER)
      .count();
    word > 0
      && out[..out.len() - word].last().is_none_or(|t| {
        !(t.get_catcode() == Catcode::OTHER
          && t.with_str(|s| {
            s.chars()
              .all(|c| c.is_ascii_digit() || c == '.' || c == ',')
          }))
      })
  };
  let mut out: Vec<Token> = Vec::with_capacity(src.len());
  let mut i = 0;
  while i < src.len() {
    let t = src[i];
    if after_a_word(&out) {
      if t == T_CS!("\\textsuperscript")
        && let Some((operand, next)) = sup_operand_at(&src, i + 1)
        && letter(src.get(next))
        && !marks(&operand, next)
      {
        out.extend(visible_sup(operand));
        i = next;
        continue;
      }
      if t == T_MATH!()
        && src.get(i + 1) == Some(&T_SUPER!())
        && let Some((operand, next)) = sup_operand_at(&src, i + 2)
        && src.get(next) == Some(&T_MATH!())
        && letter(src.get(next + 1))
        && !marks(&operand, next + 1)
      {
        let mut math = vec![T_MATH!()];
        math.extend(operand);
        math.push(T_MATH!());
        out.extend(visible_sup(math));
        i = next + 1;
        continue;
      }
    }
    out.push(t);
    i += 1;
  }
  Tokens::new(out)
}
