//! Splitting an author line into names: separators expanded, wrappers, leading spacing.

use super::*;

/// An author argument that is one parameterless macro, alone or with an empty group (`\author{\Authors}`,
/// 2401.13568; `\IEEEauthorblockN{\hpcaauthors{}}`, 2407.02944), read as TeX typesets it: expanded once, the macro's body
/// in its place, so its names and separators are seen; a body that is a `\newif` toggle through the branch TeX takes, any
/// other conditional body left whole (KNOWN_PERL_ERRORS #552).
pub(in crate::base_utilities) fn expand_sole_author_macro(stuff: Tokens) -> Tokens {
  let content: Vec<Token> = stuff
    .unlist_ref()
    .iter()
    .copied()
    .filter(|t| *t != T_SPACE!())
    .collect();
  if let Some(read) = sole_input_file(&content) {
    return read;
  }
  let cs = match content.as_slice() {
    [cs] => *cs,
    [cs, open, close]
      if open.get_catcode() == Catcode::BEGIN && close.get_catcode() == Catcode::END =>
    {
      *cs
    },
    _ => return stuff,
  };
  if cs.get_catcode() != Catcode::CS || lookup_conditional(&cs).is_some() {
    return stuff;
  }
  let Ok(Some(defn)) = lookup_definition(&cs) else {
    return stuff;
  };
  // (no parameter text at all: `\def\Authors.{…}` reads a delimiter TeX would demand)
  if !defn.is_expandable()
    || defn
      .get_parameters()
      .is_some_and(|p| !p.get_parameters().is_empty())
  {
    return stuff;
  }
  let Some(ExpansionBody::Tokens(body)) = defn.get_expansion() else {
    return stuff;
  };
  // a body holding a conditional is read through the branch a `\newif` toggle takes (`\ifanon Anonymous\else Ann Able,
  // Bob Baker\fi`, as the PDF shows it); any other is left whole: split at its commas, the `\if` and its `\else`/`\fi`
  // landed in different authors (`Error:expected:\fi`, the other branch's names leaking)
  let body = body.unlist_ref();
  let read = if body.iter().any(|t| lookup_conditional(t).is_some()) {
    match toggled_branch(body) {
      Some(branch) => branch,
      None => return stuff,
    }
  } else {
    body.to_vec()
  };
  if read.iter().any(|t| t.get_catcode() == Catcode::LETTER) {
    Tokens::new(read)
  } else {
    stuff
  }
}

/// An author argument that is one `\input{file}` (`\author{\input{author_grid.tex}}`, 2609.38746, its names and
/// affiliations in the file), read as TeX typesets it, as [`expand_sole_author_macro`] reads a sole macro: the file's
/// content in its place, read by a file mouth under the document's catcodes, so its names and separators are seen; read whole,
/// the file's names were one author. A file not found, or one that changes how it is read (`\catcode`,
/// `\makeatletter`, `\endinput`, a further `\input`), is left to `\input` as written.
fn sole_input_file(content: &[Token]) -> Option<Tokens> {
  let [input, open, name @ .., close] = content else {
    return None;
  };
  if *input != T_CS!("\\input")
    || open.get_catcode() != Catcode::BEGIN
    || close.get_catcode() != Catcode::END
    || name.is_empty()
    || name
      .iter()
      .any(|t| matches!(t.get_catcode(), Catcode::CS | Catcode::BEGIN | Catcode::END))
  {
    return None;
  }
  let name: String = name.iter().map(|t| t.to_string()).collect();
  let path = find_file(&name, None).or_else(|| find_file(&s!("{name}.tex"), None))?;
  // (its bytes decoded as a mouth decodes them, a Latin-1 line as its bytes — `mouth::decode_input_bytes` —, and each
  // command matched whole: `\inputencoding`, `\includegraphics` are none of them; the rev107b review's N3)
  let text = mouth::decode_input_bytes(&std::fs::read(&path).ok()?);
  let holds = |cs: &str| {
    text.match_indices(cs).any(|(at, _)| {
      !text[at + cs.len()..]
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic())
    })
  };
  if [
    "\\catcode",
    "\\makeatletter",
    "\\makeatother",
    "\\endinput",
    "\\input",
    "\\include",
  ]
  .iter()
  .any(|cs| holds(cs))
  {
    return None;
  }
  // (read as `\input` reads it, a file mouth: its tokens located in the file)
  let read = Mouth::create(&path, MouthOptions::default())
    .ok()?
    .read_tokens();
  read
    .unlist_ref()
    .iter()
    .any(|t| t.get_catcode() == Catcode::LETTER)
    .then_some(read)
}

/// An author line's parameterless macros that print only a name separator — `\newcommand{\authcomma}{\textmd{,}
/// \hskip0.5em}` (2403.07809), tibop-article.cls:41's `\lastand` = `, and ` (2401.13365) — written as that separator,
/// where the split reads it: hidden in the macro, the names around it were one author. The separator is written as the
/// macro prints it, a space where its body has spacing and no more (`AT\amp T Labs` stays "AT&T", `NSF\comma NIH`
/// "NSF,NIH"), and an empty group ending the macro's name (`\amp{}`) prints nothing.
pub(in crate::base_utilities) fn expand_separator_macros(stuff: Tokens) -> Tokens {
  let v = stuff.unlist_ref();
  if !v.iter().any(|t| t.get_catcode() == Catcode::CS) {
    return stuff;
  }
  let mut out: Vec<Token> = Vec::with_capacity(v.len());
  let mut changed = false;
  let mut i = 0;
  while i < v.len() {
    let t = v[i];
    i += 1;
    let Some(sep) = separator_macro_text(&t) else {
      out.push(t);
      continue;
    };
    changed = true;
    if sep.lead && out.last() != Some(&T_SPACE!()) {
      out.push(T_SPACE!());
    }
    for c in sep.text.chars() {
      out.push(match c {
        ' ' => T_SPACE!(),
        '&' => T_CS!("\\&"),
        ',' => T_OTHER!(","),
        _ => T_LETTER!(&c.to_string()),
      });
    }
    if v.get(i) == Some(&T_BEGIN!()) && v.get(i + 1) == Some(&T_END!()) {
      i += 2;
    }
    if sep.trail {
      out.push(T_SPACE!());
      while v.get(i) == Some(&T_SPACE!()) {
        i += 1;
      }
    }
  }
  if changed { Tokens::new(out) } else { stuff }
}

/// The font commands that set the upright roman text a separator prints anyway: around a separator's group they
/// print nothing of their own (`\authcomma` = `\textmd{,}\hskip0.5em`, 2403.07809).
const PLAIN_FONT_COMMANDS: [&str; 8] = [
  "\\textmd",
  "\\textup",
  "\\textrm",
  "\\textnormal",
  "\\normalfont",
  "\\rm",
  "\\rmfamily",
  "\\upshape",
];

/// What a separator macro prints: `text` (`,`, `and`, `&` or `, and`), and whether spacing comes before (`lead`) and
/// after it (`trail`) in the macro's body — written as one space.
struct SeparatorText {
  text:  &'static str,
  lead:  bool,
  trail: bool,
}

/// The separator a parameterless macro prints, read through its body past spacing (`\hskip0.5em`, `\quad`, ties), the
/// font commands around a group (`\textmd{,}`) and `\ignorespaces`, its letters in the case a `\MakeLowercase`,
/// `\MakeUppercase`, `\lowercase` or `\uppercase` gives its group (cimento's `\atque`); `None` for any other macro.
fn separator_macro_text(cs: &Token) -> Option<SeparatorText> {
  if cs.get_catcode() != Catcode::CS || lookup_conditional(cs).is_some() {
    return None;
  }
  // (an author-block delimiter is split on itself, whatever it prints: a class's `\renewcommand{\and}{{\normalfont
  // and}}`, ecca.cls:192, still parts its groups, as Perl splits the unexpanded `\and`, Base_Utility.pool.ltxml:682)
  let delimiter = |d: SplitDelim| matches!(d, SplitDelim::Token(t) if t == *cs);
  if author_group_splits()
    .into_iter()
    .chain(author_and_splits())
    .chain(author_affil_splits())
    .any(delimiter)
  {
    return None;
  }
  let defn = lookup_definition(cs).ok().flatten()?;
  if !defn.is_expandable()
    || defn
      .get_parameters()
      .is_some_and(|p| !p.get_parameters().is_empty())
  {
    return None;
  }
  let Some(ExpansionBody::Tokens(body)) = defn.get_expansion() else {
    return None;
  };
  let body = body.unlist_ref();
  let mut text = String::new();
  let (mut lead, mut spaced) = (false, false);
  // the case changes in force, each with the group depth it ends at (`\MakeLowercase{and\ \ignorespaces}`), and the
  // one a case command just asked of the group after it
  let mut cases: Vec<(usize, bool)> = Vec::new();
  let mut pending_case: Option<bool> = None;
  let mut depth = 0usize;
  let mut i = 0;
  while i < body.len() {
    let t = body[i];
    i += 1;
    // (a case command's argument that is no group is its one token: `\MakeUppercase a`)
    let single_case = if t.get_catcode() == Catcode::BEGIN {
      None
    } else {
      pending_case.take()
    };
    let spacing = if t.get_catcode() == Catcode::BEGIN {
      depth += 1;
      if let Some(upper) = pending_case.take() {
        cases.push((depth, upper));
      }
      continue;
    } else if t.get_catcode() == Catcode::END {
      if cases.last().is_some_and(|(at, _)| *at == depth) {
        cases.pop();
      }
      depth = depth.saturating_sub(1);
      continue;
    } else if t == T_CS!("\\ignorespaces") {
      // (prints nothing: cimento.cls:1525 `\atque` = `{\normalfont\MakeLowercase{and\ \ignorespaces}}`, every shipped
      // copy; 0711.2584, 0801.3557, 1402.1844, 2109.08173, 2309.14694, hep-th0703055)
      continue;
    } else if let Some(upper) = t.with_str(|s| match s {
      "\\MakeLowercase" | "\\lowercase" => Some(false),
      "\\MakeUppercase" | "\\uppercase" => Some(true),
      _ => None,
    }) {
      // (the case its group's letters print in)
      pending_case = Some(upper);
      continue;
    } else if is_spacing(&t) {
      true
    } else if t == T_CS!("\\hskip") || t == T_CS!("\\kern") {
      i = past_glue(body, i);
      true
    } else if t == T_CS!("\\hspace") {
      if body.get(i) == Some(&T_OTHER!("*")) {
        i += 1;
      }
      i = skip_group(body, i);
      true
    } else {
      false
    };
    if spacing {
      if text.is_empty() {
        lead = true;
      } else {
        spaced = true;
      }
      continue;
    }
    if t.get_catcode() == Catcode::CS && t != T_CS!("\\&") {
      // an upright-roman font command around the separator's group (`\textmd{,}`, `\rm`) prints nothing of its own;
      // a visible one (`\textbf{\&}`, `\large`) is the separator's look, kept as written
      if !t.with_str(|s| PLAIN_FONT_COMMANDS.contains(&s)) {
        return None;
      }
      continue;
    }
    if !(t == T_CS!("\\&") || matches!(t.get_catcode(), Catcode::LETTER | Catcode::OTHER)) {
      // (an alignment `&`, a parameter, math: no printed separator)
      return None;
    }
    if std::mem::take(&mut spaced) {
      text.push(' ');
    }
    if t == T_CS!("\\&") {
      text.push('&');
    } else {
      t.with_str(
        |s| match single_case.or(cases.last().map(|(_, upper)| *upper)) {
          Some(true) => text.push_str(&s.to_uppercase()),
          Some(false) => text.push_str(&s.to_lowercase()),
          None => text.push_str(s),
        },
      );
    }
  }
  let text = match text.as_str() {
    "," => ",",
    "and" => "and",
    "&" => "&",
    ", and" => ", and",
    _ => return None,
  };
  Some(SeparatorText { text, lead, trail: spaced })
}

/// The branch of `body` = `\ifX … [\else …] \fi` whose `\ifX` means `\iftrue` or `\iffalse` (a `\newif` toggle) that TeX
/// takes, when that branch holds no conditional of its own; `None` for any other shape.
fn toggled_branch(body: &[Token]) -> Option<Vec<Token>> {
  let first = body.iter().position(|t| *t != T_SPACE!())?;
  let last = body.iter().rposition(|t| *t != T_SPACE!())?;
  let test = lookup_definition(&body[first]).ok().flatten()?;
  let truth = if Some(&test)
    == lookup_definition(&T_CS!("\\iftrue"))
      .ok()
      .flatten()
      .as_ref()
  {
    true
  } else if Some(&test)
    == lookup_definition(&T_CS!("\\iffalse"))
      .ok()
      .flatten()
      .as_ref()
  {
    false
  } else {
    return None;
  };
  if last <= first || lookup_conditional(&body[last]) != Some(ConditionalType::Fi) {
    return None;
  }
  let inner = &body[first + 1..last];
  let otherwise = inner
    .iter()
    .position(|t| lookup_conditional(t) == Some(ConditionalType::Else));
  let branch = match (truth, otherwise) {
    (true, Some(e)) => &inner[..e],
    (true, None) => inner,
    (false, Some(e)) => &inner[e + 1..],
    (false, None) => &[][..],
  };
  if branch.iter().any(|t| lookup_conditional(t).is_some()) {
    return None;
  }
  Some(branch.to_vec())
}

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
