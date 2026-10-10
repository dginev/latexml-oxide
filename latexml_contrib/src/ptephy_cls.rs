//! Stub for ptephy.cls (Progress of Theoretical and Experimental Physics).
use latexml_package::prelude::*;

/// The argument of `tokens` at `i` — a brace group's content, or one token — and the index after it.
fn argument_at(tokens: &[Token], i: usize) -> Option<(Vec<Token>, usize)> {
  let first = tokens.get(i)?;
  if first.get_catcode() != Catcode::BEGIN {
    return Some((vec![*first], i + 1));
  }
  let mut depth = 0usize;
  for (j, t) in tokens.iter().enumerate().skip(i) {
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => {
        depth -= 1;
        if depth == 0 {
          return Some((tokens[i + 1..j].to_vec(), j + 1));
        }
      },
      _ => {},
    }
  }
  None
}

/// The first index from `i` that is not a space.
fn skip_spaces(tokens: &[Token], mut i: usize) -> usize {
  while tokens.get(i) == Some(&T_SPACE!()) {
    i += 1;
  }
  i
}

/// One `\affil` of an `\address`: its mark (`\affil{mark}` or `\affil[mark]`), its text when a brace group follows
/// (ptephy.cls:1390 `\affil#1#2`), and the index after it.
struct Affil {
  mark: Vec<Token>,
  text: Option<Vec<Token>>,
  end:  usize,
}

/// The `\affil` at `i`.
fn affil_at(tokens: &[Token], i: usize) -> Option<Affil> {
  let at = skip_spaces(tokens, i + 1);
  let (mark, after) = if tokens.get(at) == Some(&T_OTHER!("[")) {
    let close = tokens[at..].iter().position(|t| *t == T_OTHER!("]"))? + at;
    (tokens[at + 1..close].to_vec(), close + 1)
  } else {
    argument_at(tokens, at)?
  };
  let next = skip_spaces(tokens, after);
  match tokens.get(next) {
    Some(t) if t.get_catcode() == Catcode::BEGIN => {
      let (text, end) = argument_at(tokens, next)?;
      Some(Affil { mark, text: Some(text), end })
    },
    _ => Some(Affil { mark, text: None, end: after }),
  }
}

/// `tokens` without the calls of `commands` among them, each with its argument; and those calls. A name's marks
/// without their notes (`\name{D.~Tomono}{8, \thanks{Present Address: …}}`, 1306.3810), an affiliation without its
/// emails (`\affil{}{Department of Physics, … \email{jido@tmu.ac.jp}}`, 1605.07339).
fn split_out(tokens: &[Token], commands: &[&str]) -> (Vec<Token>, Vec<Token>) {
  let (mut rest, mut calls) = (Vec::new(), Vec::new());
  let mut i = 0;
  while i < tokens.len() {
    if commands.iter().any(|cs| tokens[i] == T_CS!(*cs))
      && let Some((_, next)) = argument_at(tokens, skip_spaces(tokens, i + 1))
    {
      calls.extend_from_slice(&tokens[i..next]);
      i = next;
      continue;
    }
    rest.push(tokens[i]);
    i += 1;
  }
  (rest, calls)
}

/// A footnote symbol, however typed, as the frontmatter labels it: `∗` (the class's `\ast`, `*`), `†`, `‡`, `§`, `¶`,
/// `⋆`, `•`, `◦` — the symbols `is_footnote_symbol_operand` (base_utilities.rs) shows as marks, `★`, `♦`, `\diamond`,
/// `\sharp`, `\#`, `\|` among them.
fn footnote_symbol(t: &Token) -> Option<Token> {
  let any =
    |names: &[&str], chars: &[Token]| names.iter().any(|cs| *t == T_CS!(*cs)) || chars.contains(t);
  if any(&["\\ast"], &[T_OTHER!("*"), T_OTHER!("\u{2217}")]) {
    Some(T_OTHER!("\u{2217}"))
  } else if any(&["\\dag", "\\dagger", "\\textdagger"], &[T_OTHER!(
    "\u{2020}"
  )]) {
    Some(T_OTHER!("\u{2020}"))
  } else if any(&["\\ddag", "\\ddagger", "\\textdaggerdbl"], &[T_OTHER!(
    "\u{2021}"
  )]) {
    Some(T_OTHER!("\u{2021}"))
  } else if any(&["\\S", "\\textsection"], &[T_OTHER!("\u{A7}")]) {
    Some(T_OTHER!("\u{A7}"))
  } else if any(&["\\P", "\\textparagraph"], &[T_OTHER!("\u{B6}")]) {
    Some(T_OTHER!("\u{B6}"))
  } else if any(&["\\star"], &[T_OTHER!("\u{22C6}")]) {
    Some(T_OTHER!("\u{22C6}"))
  } else if any(&["\\bullet", "\\textbullet"], &[T_OTHER!("\u{2022}")]) {
    Some(T_OTHER!("\u{2022}"))
  } else if any(&["\\circ"], &[T_OTHER!("\u{25E6}")]) {
    Some(T_OTHER!("\u{25E6}"))
  } else if any(&[], &[T_OTHER!("\u{2605}"), T_OTHER!("\u{2666}")]) {
    Some(*t)
  } else if any(&["\\diamond", "\\sharp", "\\#", "\\|"], &[]) {
    // (no character the frontmatter's symbol test takes for these: kept as written)
    Some(*t)
  } else {
    None
  }
}

/// Spacing between marks (`{1 \, \ast}`, `{2,~3}`), none of the label.
fn is_mark_spacing(t: &Token) -> bool {
  *t == T_SPACE!()
    || t.get_catcode() == Catcode::ACTIVE
    || ["\\,", "\\;", "\\:", "\\!", "\\ ", "\\quad", "\\thinspace"]
      .iter()
      .any(|cs| *t == T_CS!(*cs))
}

/// Marks as the frontmatter compares them: expanded (`\newcommand{\AFFicrr}{1}`, on the names and the affiliations
/// alike: 2209.07273, 1409.7469 `\affil{\INSTC}{…}`), each footnote symbol written as `footnote_symbol` labels it, and
/// a symbol glued to the mark before it a mark of its own (`{1\ast}` is `1,∗`, `{1\dagger*}` `1,†,∗`; 1512.04524,
/// 1504.04965, 1408.5182, 1502.00715) — spacing and braces dropped.
fn expanded_marks(marks: Vec<Token>) -> Result<Vec<Token>> {
  let normalized = |tokens: Vec<Token>| -> Vec<Token> {
    tokens
      .into_iter()
      .filter(|t| {
        !is_mark_spacing(t)
          && !matches!(
            t.get_catcode(),
            Catcode::MATH | Catcode::BEGIN | Catcode::END
          )
      })
      .map(|t| footnote_symbol(&t).unwrap_or(t))
      .collect()
  };
  let mut out: Vec<Token> = Vec::new();
  for t in normalized(do_expand(Tokens::new(normalized(marks)))?.unlist()) {
    if let Some(prev) = out.last()
      && *prev != T_OTHER!(",")
      && t != T_OTHER!(",")
      && (footnote_symbol(prev).is_some() || footnote_symbol(&t).is_some())
    {
      out.push(T_OTHER!(","));
    }
    out.push(t);
  }
  Ok(out)
}

/// The frontmatter call adding `email` (its own label text, `E-mail:`, as the contact's name) for the author marked
/// `symbol`, or, with none, for the whole author block.
fn email_call(symbol: Option<&str>, email: Vec<Token>) -> Result<Vec<Token>> {
  // a font group around it (`{\rm E-mail: …}`, 1304.2086), the font aside
  let email = match email.as_slice() {
    [open, inner @ .., close]
      if open.get_catcode() == Catcode::BEGIN
        && close.get_catcode() == Catcode::END
        && inner
          .iter()
          .all(|t| !matches!(t.get_catcode(), Catcode::BEGIN | Catcode::END)) =>
    {
      let fonts = [
        "\\rm",
        "\\it",
        "\\tt",
        "\\sf",
        "\\bf",
        "\\normalfont",
        "\\upshape",
      ];
      let start = inner
        .iter()
        .position(|t| *t != T_SPACE!() && !fonts.iter().any(|cs| *t == T_CS!(*cs)))
        .unwrap_or(inner.len());
      inner[start..].to_vec()
    },
    _ => email,
  };
  let text = Tokens::new(email.clone()).to_string();
  let lower = text.trim_start().to_lowercase();
  let labelled = ["e-mail", "email"]
    .iter()
    .find_map(|word| lower.strip_prefix(word))
    .is_some_and(|after| after.trim_start().starts_with(':'));
  let colon = email.iter().position(|t| *t == T_OTHER!(":"));
  let (name, email) = match colon {
    Some(colon) if labelled => {
      let name = Tokens::new(email[..=colon].to_vec())
        .to_string()
        .trim()
        .to_string();
      (Some(name), email[skip_spaces(&email, colon + 1)..].to_vec())
    },
    _ => (None, email),
  };
  let mut options = match symbol {
    Some(symbol) => s!("label=affiliation:{symbol}"),
    None => s!("label=addresses:block"),
  };
  if let Some(name) = name {
    options.push_str(&s!(",name={{{name} }}"));
  }
  Ok(
    Invocation!(T_CS!("\\lx@add@email"), vec![
      Some(mouth::tokenize_internal(TeXString::assembled(options))),
      Some(Tokens::new(email))
    ])
    .unlist(),
  )
}

/// An `\author` line with each `\name{name}{marks}`'s marks expanded (`expanded_marks`: `\name{H.~Sekiya}{\AFFicrr,
/// \AFFipmu}`, 2209.07273; `{\Tokushima,}`, 1801.03251), its notes after it, and a comma between two names nothing
/// but spaces, notes and line breaks separates (`\name{…}{1,2}` on one line, `\name{…}{1,\ast}` on the next: the class
/// prints them side by side, two people; 2210.05569, 1512.04524, 2404.08725, a commented-out comma in 2101.03480; a
/// `\\` before the closing `\name{(The T2K Collaboration)}{}`, 1409.7469); and whether a name is marked `∗`, the
/// corresponding author, whose email ptephy.cls:1318 prints `$^{\ast}$E-mail:`.
fn author_line(list: &Tokens) -> Result<(Tokens, bool)> {
  let tokens = list.unlist_ref();
  let mut out: Vec<Token> = Vec::new();
  let mut starred = false;
  // what came since the last name, while it is only spaces and line breaks (`\\`, `\newline`); a `~`, `\quad` or
  // `\par` (blank line) between two names still joins them, as no paper of the 204 writes it
  // (`ptephy_names_without_separator`)
  let mut since_name: Option<Vec<Token>> = None;
  let mut i = 0;
  while i < tokens.len() {
    // (`\collaborator{X}`, ptephy_v1.cls:1403, a name of its own: 2101.03480, 2404.08725)
    let named = if tokens[i] == T_CS!("\\name") {
      argument_at(tokens, skip_spaces(tokens, i + 1)).and_then(|(name, after)| {
        argument_at(tokens, skip_spaces(tokens, after)).map(|(marks, next)| (name, marks, next))
      })
    } else if tokens[i] == T_CS!("\\collaborator") {
      argument_at(tokens, skip_spaces(tokens, i + 1)).map(|(name, next)| (name, Vec::new(), next))
    } else {
      None
    };
    if let Some((name, marks, next)) = named {
      // (the line breaks between two names are the line of names going on)
      if since_name.take().is_some() {
        out.extend([T_OTHER!(","), T_SPACE!()]);
      }
      let (marks, notes) = split_out(&marks, &["\\thanks", "\\footnote"]);
      let marks = expanded_marks(marks)?;
      starred |= marks.contains(&T_OTHER!("\u{2217}"));
      out.extend(
        Invocation!(T_CS!("\\name"), vec![
          Some(Tokens::new(name)),
          Some(Tokens::new(marks))
        ])
        .unlist(),
      );
      out.extend(notes);
      since_name = Some(Vec::new());
      i = next;
      continue;
    }
    if (tokens[i] == T_CS!("\\thanks") || tokens[i] == T_CS!("\\footnote"))
      && let Some((_, next)) = argument_at(tokens, skip_spaces(tokens, i + 1))
    {
      out.extend(since_name.as_mut().map(std::mem::take).unwrap_or_default());
      out.extend_from_slice(&tokens[i..next]);
      i = next;
      continue;
    }
    if let Some(pending) = since_name.as_mut() {
      if tokens[i] == T_SPACE!() {
        pending.push(tokens[i]);
        i += 1;
        continue;
      }
      if tokens[i] == T_CS!("\\\\") || tokens[i] == T_CS!("\\newline") {
        // the break, its `*` and `[length]`
        let mut next = i + 1;
        if tokens.get(next) == Some(&T_OTHER!("*")) {
          next += 1;
        }
        if tokens.get(next) == Some(&T_OTHER!("["))
          && let Some(k) = tokens[next..].iter().position(|t| *t == T_OTHER!("]"))
        {
          next += k + 1;
        }
        pending.extend_from_slice(&tokens[i..next]);
        i = next;
        continue;
      }
      out.append(pending);
      since_name = None;
    }
    out.push(tokens[i]);
    i += 1;
  }
  out.extend(since_name.unwrap_or_default());
  Ok((Tokens::new(out), starred))
}

/// The index past a spacing command at `i` and its argument (`\vspace{1mm}`, `\\[2mm]`, `\vskip 2pt`, `\kern1pt`), whose
/// letters are units, not text; `None` for any other token.
fn past_spacing(tokens: &[Token], i: usize) -> Option<usize> {
  let is = |names: &[&str]| names.iter().any(|cs| tokens[i] == T_CS!(*cs));
  let star = |j: usize| {
    if tokens.get(j) == Some(&T_OTHER!("*")) {
      j + 1
    } else {
      j
    }
  };
  if is(&["\\vspace", "\\hspace"]) {
    argument_at(tokens, skip_spaces(tokens, star(i + 1))).map(|(_, next)| next)
  } else if is(&["\\\\", "\\newline"]) {
    let j = skip_spaces(tokens, star(i + 1));
    if tokens.get(j) == Some(&T_OTHER!("[")) {
      tokens[j..]
        .iter()
        .position(|t| *t == T_OTHER!("]"))
        .map(|k| j + k + 1)
    } else {
      Some(i + 1)
    }
  } else if is(&["\\vskip", "\\hskip", "\\kern", "\\mskip"]) {
    // the glue or dimension (tex.web §1057, §1061): a run of digits, signs, points and unit letters
    let mut j = skip_spaces(tokens, i + 1);
    while tokens.get(j).is_some_and(|t| {
      t.get_catcode() == Catcode::LETTER
        || (t.get_catcode() == Catcode::OTHER
          && t.with_str(|s| s.chars().all(|c| c.is_ascii_digit() || ".,+-".contains(c))))
    }) {
      j += 1;
    }
    Some(j)
  } else {
    None
  }
}

/// A mark at `i` in an address's text — `{\dag}`, `$^\dagger$`, `${}^\ast$`, `\textsuperscript{\ddag}`, a bare `\dag` —
/// its first footnote symbol and the index after it; balanced, so the text around it stays whole.
fn mark_run(tokens: &[Token], i: usize) -> Option<(Token, usize)> {
  let opening =
    |t: &Token| matches!(t.get_catcode(), Catcode::BEGIN | Catcode::SUPER) || is_mark_spacing(t);
  let mut j = i;
  let mut symbol = None;
  while let Some(t) = tokens.get(j) {
    if let Some(sym) = footnote_symbol(t) {
      symbol.get_or_insert(sym);
    } else if !(matches!(
      t.get_catcode(),
      Catcode::MATH | Catcode::SUPER | Catcode::BEGIN | Catcode::END
    ) || *t == T_CS!("\\textsuperscript")
      || is_mark_spacing(t))
    {
      break;
    }
    j += 1;
  }
  while j > i && opening(&tokens[j - 1]) {
    j -= 1;
  }
  let run = &tokens[i..j];
  let count = |code: Catcode| run.iter().filter(|t| t.get_catcode() == code).count();
  (count(Catcode::BEGIN) == count(Catcode::END) && count(Catcode::MATH) % 2 == 0)
    .then_some(symbol?)
    .map(|symbol| (symbol, j))
}

/// The frontmatter calls adding an `\email`'s argument, split where a footnote symbol leads a piece of it
/// (`\email{knabe@…, $^\dagger$E-mail: hisao@…}`, 1408.5182): each piece the email of the author its symbol marks, the
/// lead `lead`'s (`email_call`).
fn email_calls(tokens: &[Token], lead: Option<&str>) -> Result<Vec<Token>> {
  let (mut first, mut pieces): (Vec<Token>, Vec<(Token, Vec<Token>)>) = (Vec::new(), Vec::new());
  let mut i = 0;
  while i < tokens.len() {
    if let Some((symbol, next)) = mark_run(tokens, i) {
      pieces.push((symbol, Vec::new()));
      i = next;
      continue;
    }
    match pieces.last_mut() {
      Some((_, piece)) => piece.push(tokens[i]),
      None => first.push(tokens[i]),
    }
    i += 1;
  }
  let balanced = |v: &[Token]| {
    v.iter()
      .filter(|t| t.get_catcode() == Catcode::BEGIN)
      .count()
      == v.iter().filter(|t| t.get_catcode() == Catcode::END).count()
  };
  if !balanced(&first) || pieces.iter().any(|(_, piece)| !balanced(piece)) {
    return email_call(lead, tokens.to_vec());
  }
  let mut calls = Vec::new();
  if let Some(first) = trimmed_text(first) {
    calls.extend(email_call(lead, first)?);
  }
  for (symbol, piece) in pieces {
    if let Some(piece) = trimmed_text(piece) {
      calls.extend(email_call(Some(&symbol.to_string()), piece)?);
    }
  }
  Ok(calls)
}

/// `tokens` without the separators around it (spaces, spacing, `,`, `;`, `\\`), when it holds text (letters).
fn trimmed_text(mut tokens: Vec<Token>) -> Option<Vec<Token>> {
  let separator = |t: &Token| {
    is_mark_spacing(t) || *t == T_OTHER!(",") || *t == T_OTHER!(";") || *t == T_CS!("\\\\")
  };
  while tokens.first().is_some_and(separator) {
    tokens.remove(0);
  }
  while tokens.last().is_some_and(separator) {
    tokens.pop();
  }
  tokens
    .iter()
    .any(|t| t.get_catcode() == Catcode::LETTER)
    .then_some(tokens)
}

/// What an `\address` of `\affil{mark}{text}`s prints beside them, its `\email`s and spacing aside (`\vspace{1mm}`,
/// 1407.3513), when it holds text: each piece led by a footnote symbol the email (or note) of the author that symbol
/// marks (`\email{a@x}, {\dag}b@y, {\ddag}c@z`, 1401.4647, 1507.04527, 1610.06306; `$^\dagger$E-mail: …`, 1408.5182),
/// the rest an address of the whole block (`${}^\ast$ {\rm E-mail: …}` is the starred author's, 1304.7885).
fn remainder_calls(tokens: &[Token]) -> Result<Vec<Token>> {
  let mut rest: Vec<Token> = Vec::new();
  let mut i = 0;
  while i < tokens.len() {
    let skip = if tokens[i] == T_CS!("\\affil") {
      affil_at(tokens, i).map(|affil| affil.end)
    } else if tokens[i] == T_CS!("\\email") {
      argument_at(tokens, skip_spaces(tokens, i + 1)).map(|(_, next)| next)
    } else {
      past_spacing(tokens, i)
    };
    match skip {
      Some(next) => i = next,
      None => {
        rest.push(tokens[i]);
        i += 1;
      },
    }
  }
  let (mut lead, mut pieces): (Vec<Token>, Vec<(Token, Vec<Token>)>) = (Vec::new(), Vec::new());
  let mut i = 0;
  while i < rest.len() {
    if let Some((symbol, next)) = mark_run(&rest, i) {
      pieces.push((symbol, Vec::new()));
      i = next;
      continue;
    }
    match pieces.last_mut() {
      Some((_, piece)) => piece.push(rest[i]),
      None => lead.push(rest[i]),
    }
    i += 1;
  }
  let balanced = |v: &[Token]| {
    v.iter()
      .filter(|t| t.get_catcode() == Catcode::BEGIN)
      .count()
      == v.iter().filter(|t| t.get_catcode() == Catcode::END).count()
  };
  if !balanced(&lead) || pieces.iter().any(|(_, piece)| !balanced(piece)) {
    pieces.clear();
    lead = rest;
  }
  let mut calls: Vec<Token> = Vec::new();
  if let Some(lead) = trimmed_text(lead) {
    calls.extend(
      Invocation!(T_CS!("\\lx@ptephy@remainder"), vec![Some(Tokens::new(
        lead
      ))])
      .unlist(),
    );
  }
  for (symbol, piece) in pieces {
    let Some(piece) = trimmed_text(piece) else {
      continue;
    };
    let symbol = symbol.to_string();
    let text = Tokens::new(piece.clone()).to_string();
    if text.contains('@') || text.contains("(at)") {
      calls.extend(email_call(Some(&symbol), piece)?);
    } else {
      calls.extend(
        Invocation!(T_CS!("\\lx@add@address"), vec![
          Some(mouth::tokenize_internal(TeXString::assembled(s!(
            "label=affiliation:{symbol}"
          )))),
          Some(Tokens::new(piece))
        ])
        .unlist(),
      );
    }
  }
  Ok(calls)
}

/// An `\address` with a single-argument `\affil`, its `\email`s taken out: each the email of the author marked by the
/// mark-only `\affil` right before it (`\affil{\dag}\email{…}`, 1601.07691), which is dropped from the lines, else of
/// the starred author (1703.03659 `\name{T. Fukuda}{1,*}`), else of the whole block.
fn routed_emails(tokens: &[Token], starred: bool) -> Result<(Vec<Token>, Vec<Token>)> {
  let (mut rest, mut calls) = (Vec::new(), Vec::new());
  let mut i = 0;
  while i < tokens.len() {
    if tokens[i] == T_CS!("\\affil")
      && let Some(Affil { mark, text: None, end }) = affil_at(tokens, i)
      && !mark.iter().any(|t| t.get_catcode() == Catcode::LETTER)
      && let at = skip_spaces(tokens, end)
      && tokens.get(at) == Some(&T_CS!("\\email"))
      && let Some((email, next)) = argument_at(tokens, skip_spaces(tokens, at + 1))
    {
      let marks = expanded_marks(mark)?;
      let label: Vec<Token> = marks
        .into_iter()
        .take_while(|t| *t != T_OTHER!(","))
        .collect();
      calls.extend(email_calls(&email, Some(&Tokens::new(label).to_string()))?);
      i = next;
      continue;
    }
    if tokens[i] == T_CS!("\\email")
      && let Some((email, next)) = argument_at(tokens, skip_spaces(tokens, i + 1))
    {
      calls.extend(email_calls(&email, starred.then_some("\u{2217}"))?);
      i = next;
      continue;
    }
    rest.push(tokens[i]);
    i += 1;
  }
  Ok((rest, calls))
}

/// An `\address` whose `\affil`s are not all `\affil{mark}{text}`, as the lines it prints, for the frontmatter's
/// affiliation-list parser: an `\affil{mark}{text}` as `\textsuperscript{mark}text\\`, a single-argument one as the
/// mark it prints (ptephy.cls:1390 `$^{#1}$`, 1703.03659 `\affil{1}$^{1}${Nagoya University}`), or as its line when it
/// is an affiliation (letters: ptephy_v1's authblk-TI `\affil{text}`).
fn address_lines(tokens: &[Token]) -> Result<Tokens> {
  let mut out: Vec<Token> = Vec::new();
  let mut i = 0;
  while i < tokens.len() {
    if tokens[i] == T_CS!("\\affil")
      && let Some(Affil { mark, text, end }) = affil_at(tokens, i)
    {
      let is_text = mark.iter().any(|t| t.get_catcode() == Catcode::LETTER);
      let mark = if is_text { mark } else { expanded_marks(mark)? };
      match text {
        Some(text) => {
          out.extend([T_CS!("\\textsuperscript"), T_BEGIN!()]);
          out.extend(mark);
          out.push(T_END!());
          out.extend(text);
          out.push(T_CS!("\\\\"));
        },
        None if is_text => {
          out.extend(mark);
          out.push(T_CS!("\\\\"));
        },
        None => {
          out.extend([T_CS!("\\textsuperscript"), T_BEGIN!()]);
          out.extend(mark);
          out.push(T_END!());
        },
      }
      i = end;
      continue;
    }
    out.push(tokens[i]);
    i += 1;
  }
  Ok(Tokens::new(out))
}

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  RequirePackage!("amssymb");
  // Eager xcolor preload removed for Perl parity: it makes a later document
  // xcolor[table] load a no-op, so colortbl/array never load and array m{}/b{}
  // columns break (Unrecognized tabular template -> Extra alignment tab). The
  // document loads xcolor itself; color/definecolor stay via hyperref->color.
  // See ifacconf_cls.rs and SYNC_STATUS (eager-xcolor cluster).
  RequirePackage!("hyperref");
  RequirePackage!("graphicx");

  // The author block (ptephy.cls:426, 1388-1390; ptephy_v1.cls:429, 1396): `\author{\name{A. Author}{1} and
  // \name{B. Author}{2}}` prints each name with its superscript mark (`#1$^{#2}$`), and `\address{\affil{1}{Univ A}
  // \affil{2}{Univ B}}` the affiliations, each led by its mark (`$^{#1}$#2\par`), in italics under the names
  // (ptephy.cls:513). The marks are the frontmatter's links: a name's mark requests the affiliation it labels, each
  // `\affil` one affiliation. An `\address` of such `\affil`s is their list; what it prints beside them (a mark of its
  // own, `${}^1$\affil{1}{…}`, a line break) is the page's, so set aside, unless it is text, an address of its own
  // (`remainder_calls`); its `\email` is the starred author's (below). An `\address` without them is one
  // affiliation, and one with a single-argument `\affil` the lines it prints (`address_lines`), its emails routed
  // (`routed_emails`).
  // ptephy_v1.cls:1398 comments its `\affil` out for authblk-TI's (:485) `\affil[mark]{text}` and `\affil{text}`,
  // given after the names; the binding keeps every form for every version. Witnesses 1807.02967 (ptephy_v1),
  // 1412.6580 (ptephy), 1211.4904, 2209.07273 (`\affil[…]`), 1304.0533 (an `\address` without `\affil`), 1703.03659
  // (`\affil{1}$^{1}${…}`), 1606.03167 (`\affil{text}`).
  // A `\thanks` among the marks (`\name{D.~Tomono}{8, \thanks{Present Address: …}}`, 1306.3810, 1801.03251) is the
  // name's note, the rest its marks.
  // Each footnote symbol among the marks is a mark of its own, which the frontmatter shows as the PDF does (`1,†`:
  // 1409.3629, 2305.07680 `{1\dagger*}`; a symbol shares no request with the numbers, base_utilities.rs
  // `is_footnote_symbol_operand`), the numbers and letters around it the affiliations' requests.
  DefMacro!("\\name{}{}", sub[(name, marks)] {
    let (marks, notes) = split_out(marks.unlist_ref(), &["\\thanks", "\\footnote"]);
    let mut out = name.unlist();
    let mut run: Vec<Token> = Vec::new();
    for t in expanded_marks(marks)?.into_iter().chain([T_OTHER!(",")]) {
      let symbol = footnote_symbol(&t).is_some();
      if t == T_OTHER!(",") || symbol {
        if run.iter().any(|m| *m != T_SPACE!()) {
          out.extend(Invocation!(T_CS!("\\lx@inst@mark"), vec![Some(Tokens::new(run.clone()))]).unlist());
        }
        run.clear();
        if symbol {
          out.extend(Invocation!(T_CS!("\\lx@inst@mark"), vec![Some(Tokens!(t))]).unlist());
        }
      } else {
        run.push(t);
      }
    }
    out.extend(notes);
    Ok(Tokens::new(out))
  });
  // The kernel's `\author`, given the line `author_line` reads: its splitter reads a name's own marks as names when
  // they are macros (a "12" person, a `\thanks` folded into the name, 2209.07273, 1801.03251).
  Let!("\\lx@ptephy@kernel@author", "\\author");
  DefMacro!("\\author[]{}", sub[(short, list)] {
    let (line, starred) = author_line(&list)?;
    let mut out = if starred {
      TokenizeInternal!(r"\gdef\lx@ptephy@starred{1}").unlist()
    } else {
      Vec::new()
    };
    out.extend(Invocation!(T_CS!("\\lx@ptephy@kernel@author"), vec![short, Some(line)]).unlist());
    Ok(Tokens::new(out))
  });
  // ptephy.cls:1318 `\def\email#1{{$^{\ast}$E-mail: #1}}`: the email of the author marked `\ast`, else of the whole
  // block (1206.6924, 1212.6803, 1504.01717); a piece of it led by another symbol, that symbol's author's
  // (`email_calls`, 1408.5182). An `\address` given before the `\author` finds no starred author yet, so its email
  // is the block's (no corpus case).
  RawTeX!(r"\def\lx@ptephy@starred{}");
  DefMacro!("\\email{}", sub[(email)] {
    let starred = !do_expand(Tokens::new(vec![T_CS!("\\lx@ptephy@starred")]))?.is_empty();
    Ok(Tokens::new(email_calls(email.unlist_ref(), starred.then_some("\u{2217}"))?))
  });
  // ptephy_v1.cls:1403 `\def\collaborator#1{\textbf{#1}\par}`, the collaboration's line under the names: one more
  // author, as revtex's `\collaboration` (inside the `\author` line, `author_line`'s).
  DefMacro!("\\collaborator{}", "\\author{#1}");
  // ptephy_v1.cls:1400-1402: the name parts, each printing its argument (1304.0533).
  def_macro_identity("\\fname{}")?;
  def_macro_identity("\\surname{}")?;
  def_macro_identity("\\midname{}")?;
  DefMacro!("\\address{}", sub[(list)] {
    let tokens = list.unlist_ref();
    let affils: Vec<Option<Affil>> = (0..tokens.len())
      .filter(|&i| tokens[i] == T_CS!("\\affil"))
      .map(|i| affil_at(tokens, i))
      .collect();
    Ok(if affils.is_empty() {
      Invocation!(T_CS!("\\lx@ptephy@affil"), vec![Some(Tokens!()), Some(list)])
    } else if affils.iter().all(|affil| matches!(affil, Some(Affil { text: Some(_), .. }))) {
      let mut calls = Invocation!(T_CS!("\\lx@ptephy@affils"), vec![Some(list.clone())]).unlist();
      calls.extend(remainder_calls(tokens)?);
      Tokens::new(calls)
    } else {
      let starred = !do_expand(Tokens::new(vec![T_CS!("\\lx@ptephy@starred")]))?.is_empty();
      let (rest, emails) = routed_emails(tokens, starred)?;
      let mut calls = if rest.contains(&T_CS!("\\affil")) {
        Invocation!(T_CS!("\\lx@add@affiliations"), vec![None, Some(address_lines(&rest)?)]).unlist()
      } else if let Some(rest) = trimmed_text(rest) {
        Invocation!(T_CS!("\\lx@ptephy@affil"), vec![Some(Tokens!()), Some(Tokens::new(rest))]).unlist()
      } else {
        Vec::new()
      };
      calls.extend(emails);
      Tokens::new(calls)
    })
  });
  DefMacro!("\\lx@ptephy@affils{}", "\\setbox\\z@\\hbox{#1}");
  DefMacro!(
    "\\lx@ptephy@remainder{}",
    "\\lx@add@address[label=addresses:block]{#1}"
  );
  DefMacro!(
    "\\affil",
    "\\@ifnextchar[{\\lx@ptephy@affil@opt}{\\lx@ptephy@affil@mark}"
  );
  DefMacro!(
    "\\lx@ptephy@affil@mark{}",
    "\\@ifnextchar\\bgroup{\\lx@ptephy@affil{#1}}{\\lx@ptephy@affil{}{#1}}"
  );
  DefMacro!("\\lx@ptephy@affil@opt[]{}", "\\lx@ptephy@affil{#1}{#2}");
  // An affiliation labelled by its mark goes to the names showing it; one without (`\affil{}{…}`, `\affil{~}{…}`,
  // authblk-TI's `\affil{text}`) to the authors since the affiliation before it, and with it when they follow one
  // another (`annotate=run`; authblk-TI.sty:104-121 appends an unmarked `\affil` to the authors given since the last;
  // the class prints `\affil{}{A}\affil{}{B}` under all of them: 1605.07339, 2311.07297, 1902.10888). An `\email` in it is the starred author's, when there is one, or, beside an unlabelled
  // affiliation, the block's (1605.07339 `\affil{}{… \email{jido@tmu.ac.jp}}` under `\name{Daisuke Jido}{\ast}`);
  // inside a labelled one without a starred author, the affiliation's (1807.02967).
  DefMacro!("\\lx@ptephy@affil{}{}", sub[(mark, text)] {
    let labelled = mark
      .unlist_ref()
      .iter()
      .any(|t| *t != T_SPACE!() && t.get_catcode() != Catcode::ACTIVE);
    let starred = !do_expand(Tokens::new(vec![T_CS!("\\lx@ptephy@starred")]))?.is_empty();
    let (text, emails) = if starred || !labelled {
      split_out(text.unlist_ref(), &["\\email"])
    } else {
      (text.unlist(), Vec::new())
    };
    let mut out = if labelled {
      let mut content = vec![T_CS!("\\inst"), T_BEGIN!()];
      content.extend(expanded_marks(mark.unlist())?);
      content.push(T_END!());
      content.extend(text);
      Invocation!(T_CS!("\\lx@add@affiliation"), vec![None, Some(Tokens::new(content))]).unlist()
    } else {
      Invocation!(T_CS!("\\lx@add@affiliation"), vec![
        Some(TokenizeInternal!("annotate=run")),
        Some(Tokens::new(text))
      ])
      .unlist()
    };
    out.extend(emails);
    Ok(Tokens::new(out))
  });

  // ptephy frontmatter — preserve as ltx:note (content-preserving).
  // Both args carry author-typed data: a preprint identifier and a
  // PTEP subject-area code (used for indexing). Silent gobble would
  // lose both.
  DefMacro!(
    "\\preprintnumber[]{}",
    "\\@add@frontmatter{ltx:note}[role=preprintnumber]{#2}"
  );
  DefMacro!(
    "\\subjectindex{}",
    "\\@add@frontmatter{ltx:classification}[scheme=PTEP-subject]{#1}"
  );

  // \ack — Acknowledgements section opener (used in OUP / PTEP class).
  // Used as `\ack <paragraph>` (no body) — keep as starred section to
  // open a heading; the following paragraph is the natural body.
  DefMacro!("\\ack", "\\section*{Acknowledgements}");
  // \acknow{body} — bracketed form. Emit as structural
  // ltx:acknowledgements (post-processors map to canonical role/styling).
  DefConstructor!(
    "\\acknow{}",
    "<ltx:acknowledgements>#1</ltx:acknowledgements>"
  );
});
