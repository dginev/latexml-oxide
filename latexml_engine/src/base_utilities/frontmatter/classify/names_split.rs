//! The text rules on printed ink and the token primitives of `author_block/names_split.rs`, moved unchanged (the plan's row
//! "Knowledge in, driver out, one pass", user ruling 2026-11-08).

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
/// institution word, no digit or address. In a list by its marks (the reading's `marked_name`, `resolver/lines.rs`) such
/// a piece is a marked name (rows N2-N3).
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

// (moved unchanged from `author_block/names_split.rs` by Z18m, which deleted `author_block/`: Splitting an author line
// into names: separators expanded, wrappers, leading spacing.)
/// A name separator written at the edge of a wrapper is the line's (`\textbf{Hang Xu$^{2}$,}`, 2310.10477;
/// `\textbf{, Jinyang Guo}`, 2505.02214), and a wrapper holding only one (`\textbf{,}`, 2403.11004; `{\rm and}`,
/// 2401.03205) is that separator: the font of a comma or an "and" is not a name's (KNOWN_PERL_ERRORS #552). A comma
/// closing a link's printed text is the line's too (`\href{https://orcid.org/…}{\includegraphics{ORCIDiD.png}, }`,
/// 2110.07418: each name its own iD; guard `perfect_kernel_batch131fix::orcid_icon_link_closing_with_the_comma_parts_the_names`).
pub(in crate::base_utilities) fn hoist_wrapped_separators(line: Tokens) -> Tokens {
  let v = line.unlist();
  let and = [T_LETTER!("a"), T_LETTER!("n"), T_LETTER!("d")];
  let mut out: Vec<Token> = Vec::with_capacity(v.len());
  let mut in_math = false;
  let mut i = 0;
  while i < v.len() {
    let t = v[i];
    // (a separator in math or in a script's argument is the formula's: `$^{\dagger, 1}$`, 2301.11477)
    if t.get_catcode() == Catcode::MATH {
      in_math = !in_math;
    }
    // (nor is a group after a command it does not wrap: a mark's argument, `\textsuperscript{\faRuler,\faBalanceScale}`,
    // 2503.01996)
    let script = t.get_catcode() == Catcode::BEGIN
      && out
        .last()
        .is_some_and(|p| matches!(p.get_catcode(), Catcode::SUPER | Catcode::SUB | Catcode::CS));
    if in_math || t.get_catcode() == Catcode::MATH || script {
      let end = if t.get_catcode() == Catcode::BEGIN {
        skip_group(&v, i).min(v.len())
      } else {
        i + 1
      };
      out.extend_from_slice(&v[i..end]);
      i = end;
      continue;
    }
    // (a link's printed text is a wrapper too, its address passed over: `C. J.~Law$^{5,8}$\href{https://orcid.org/…}
    // {\includegraphics[scale=0.08]{ORCIDiD.png}, }%⏎S. Burke-Spolaor`, 2110.07418 — the comma between the two names
    // printed inside the ORCID icon's link; only its edge comma moves out, so no link is dropped)
    let link_ink = (t == T_CS!("\\href")
      && v
        .get(i + 1)
        .is_some_and(|n| n.get_catcode() == Catcode::BEGIN))
    .then(|| skip_group(&v, i + 1))
    .filter(|&ink| {
      v.get(ink)
        .is_some_and(|n| n.get_catcode() == Catcode::BEGIN)
    });
    let command = link_ink.is_some()
      || t.get_catcode() == Catcode::CS
        && v
          .get(i + 1)
          .is_some_and(|n| n.get_catcode() == Catcode::BEGIN)
        && !is_footnote_mark(&t)
        && !is_name_annotation(&t)
        && !is_author_mark(&t);
    let open = if let Some(ink) = link_ink {
      ink
    } else if command {
      i + 1
    } else if t.get_catcode() == Catcode::BEGIN {
      i
    } else {
      out.push(t);
      i += 1;
      continue;
    };
    let end = skip_group(&v, open);
    if end > v.len()
      || v
        .get(end - 1)
        .is_none_or(|u| u.get_catcode() != Catcode::END)
    {
      out.push(t);
      i += 1;
      continue;
    }
    let close = end - 1;
    let inner = &v[open + 1..close];
    let declarations = if command {
      Vec::new()
    } else {
      opening_declarations(&Tokens::new(inner.to_vec()))
    };
    if !command && declarations.iter().all(|u| *u == T_SPACE!()) {
      out.extend_from_slice(&v[i..end]);
      i = end;
      continue;
    }
    let body: Vec<Token> = inner[declarations.len()..].to_vec();
    // (a tie or a space command beside the separator is spacing: `\textbf{Jiaming Ji$^{\diamond}$,~}`, 2505.20214;
    // `\textbf{Yuanpei Chen$^5$, \quad}`, 2407.15815)
    let spacing = is_spacing;
    let first = body.iter().position(|u| !spacing(u));
    let last = body.iter().rposition(|u| !spacing(u));
    let (Some(first), Some(last)) = (first, last) else {
      out.extend_from_slice(&v[i..end]);
      i = end;
      continue;
    };
    let trimmed = &body[first..=last];
    let rewrap = |out: &mut Vec<Token>, content: &[Token]| {
      out.extend_from_slice(&v[i..=open]);
      out.extend(declarations.iter().copied());
      out.extend_from_slice(content);
      out.push(v[close]);
    };
    if link_ink.is_some() && !(trimmed.len() > 1 && trimmed.last() == Some(&T_OTHER!(","))) {
      out.extend_from_slice(&v[i..end]);
    } else if trimmed == [T_OTHER!(",")] {
      out.extend([T_OTHER!(","), T_SPACE!()]);
    } else if trimmed == and {
      out.push(T_SPACE!());
      out.extend(and);
      out.push(T_SPACE!());
    } else if trimmed.len() > 1 && trimmed[trimmed.len() - 1] == T_OTHER!(",") {
      rewrap(&mut out, &trimmed[..trimmed.len() - 1]);
      out.push(T_OTHER!(","));
    } else if trimmed.len() > 1 && trimmed[0] == T_OTHER!(",") {
      out.extend([T_OTHER!(","), T_SPACE!()]);
      rewrap(&mut out, &trimmed[1..]);
    } else if trimmed.len() > 4 && trimmed[..3] == and && spacing(&trimmed[3]) {
      // (the word "and" opening it, a word of its own: `\mbox{and {Thomas~P.} Krichbaum $^{5}$}`, mdpi 2108.09054)
      let rest = trimmed[3..]
        .iter()
        .position(|u| !spacing(u))
        .map_or(trimmed.len(), |k| 3 + k);
      if is_name_qualifier(&visible_name_text(&trimmed[rest..])) {
        // (…but not before what names no one, which stays with the name before it in its wrapper's font: `Ann Able
        // \emph{and others}`, 77n r2 review)
        out.extend_from_slice(&v[i..end]);
      } else {
        out.push(T_SPACE!());
        out.extend(and);
        out.push(T_SPACE!());
        rewrap(&mut out, &trimmed[rest..]);
      }
    } else if trimmed.len() > 4
      && trimmed[trimmed.len() - 3..] == and
      && spacing(&trimmed[trimmed.len() - 4])
    {
      let name_end = trimmed[..trimmed.len() - 3]
        .iter()
        .rposition(|u| !spacing(u))
        .map_or(0, |k| k + 1);
      rewrap(&mut out, &trimmed[..name_end]);
      out.push(T_SPACE!());
      out.extend(and);
      out.push(T_SPACE!());
    } else {
      out.extend_from_slice(&v[i..end]);
    }
    i = end;
  }
  Tokens::new(out)
}
