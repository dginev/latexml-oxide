//! A separator printed inside a wrapper of its own moved out of it (`hoist_wrapped_separators`), for
//! `name_text::names_list_continues`. Moved unchanged by Z18m, which deleted `author_block/`, from `author_block/names_split.rs`.

use super::*;

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
