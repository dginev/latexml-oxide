//! The creator fields' name readouts (`\lx@add@author`'s `creator_call`): a membership grade written after the name, a
//! collaboration the whole name or its wrapper writes, and the definitions a title-code block makes at `\author`. Moved
//! unchanged from `author_block/name_text.rs`, `collaborations.rs` and `email_lines.rs` (Z18a): field logic, not
//! author-block parsing. The readers it shares with the bindings (`collaboration_line_reading`, `only_marks_and_notes`)
//! moved here after it, unchanged (Z18j).

use super::*;

/// Whether `text` is a professional membership grade as an author writes it after the name and its comma: the words of
/// one ([`grade_words`]: `Fellow, IEEE`, `Senior Member, IEEE`, `Member, OSA`), or an abbreviated grade before its
/// society (`M.ASCE`, `A. M. ASCE`; [`NAME_GRADE_ABBREVIATIONS`]).
pub(in crate::base_utilities) fn is_membership_grade(text: &str) -> bool {
  let text = strip_glued_marks(text);
  let compact: String = text.chars().filter(|c| !c.is_whitespace()).collect();
  grade_words(text, true)
    || NAME_GRADE_ABBREVIATIONS.iter().any(|grade| {
      compact
        .strip_prefix(grade)
        .is_some_and(|society| NAME_GRADE_SOCIETIES.contains(&society))
    })
}

/// A name with the membership grade written after its comma taken out as the author's `membership` contact: `Cat
/// Cole\textsuperscript{1}, \textit{Fellow, IEEE}` is "Cat Cole" and the contact "Fellow, IEEE" in its italics, the
/// contact IEEEtran's `\IEEEmembership` makes of the grade it prints (ieeetran_cls.rs `\lx@IEEE@membership`;
/// 2107.05093, 2302.11403, 2602.16174), and so are `Bob Baker, Member, OSA` and ascelike's `Matthew R. Kuhn, M.ASCE`
/// (ascexmpl.tex:48). The grade is the text after the first comma of the name's own (outside its groups and math) after
/// which only a grade is printed ([`is_membership_grade`]); the marks and notes written after it stay the name's, after
/// the contact, as they print after the grade. `None` when the name ends in no such grade (`John Smith, Jr.`, `Jane
/// Doe, MD, PhD` qualify the name).
pub(in crate::base_utilities) fn written_membership_out(name: &[Token]) -> Option<Vec<Token>> {
  let (mut depth, mut in_math, mut found) = (0, false, None);
  let commas: Vec<usize> = (name.iter().enumerate())
    .filter_map(|(c, t)| {
      depth += t.group_step();
      in_math ^= t.get_catcode() == Catcode::MATH;
      (depth == 0 && !in_math && *t == T_OTHER!(",")).then_some(c)
    })
    .collect();
  // (a grade is a suffix of grade words, so its comma is read from the last leftward, the first tail holding a word no grade
  // has ending the reading: every tail before it holds that word too. The first comma after an alphabetic head opening a
  // grade is the one read from the first comma on, each tail read once — not the rest of the name per comma, quadratic on
  // Perl's whole-block creator of a block of 418 names, 1507.02068)
  for c in commas.into_iter().rev() {
    let (grade, annotations) = grade_and_annotations(&name[c + 1..]);
    let text = visible_name_text(&grade);
    if is_membership_grade(&text)
      && visible_name_text(&name[..c])
        .chars()
        .any(char::is_alphabetic)
    {
      found = Some((c, grade, annotations));
    } else if text.chars().any(char::is_alphanumeric)
      && !grade_words(strip_glued_marks(&text), false)
    {
      break;
    }
  }
  // (one grade, the line's last: a names line printing a grade before it holds a name after a grade, `Ann Able,
  // \textit{Member, IEEE}; Bob Baker, \textit{Fellow, IEEE}`, whose grades all stay — taking out the last alone would
  // leave the names before it one creator: 1510.03162, 2507.19818)
  let (c, grade, annotations) = found.filter(|(c, ..)| !holds_grade(&name[..*c]))?;
  let head = &name[..c];
  let end = head
    .iter()
    .rposition(|t| !is_spacing(t))
    .map_or(0, |k| k + 1);
  let mut out = head[..end].to_vec();
  out.extend(TokenizeInternal!(r"\lx@add@contact[role=membership]").unlist());
  out.push(T_BEGIN!());
  out.extend(grade);
  out.push(T_END!());
  out.extend(annotations);
  Some(out)
}

/// Whether a names line prints a membership grade or a part of one ([`is_membership_part`]) between its names'
/// separators — a comma, a semicolon, an "and".
fn holds_grade(line: &[Token]) -> bool {
  visible_name_text(line)
    .replace('~', " ")
    .split([',', ';'])
    .flat_map(|piece| piece.split(" and "))
    .map(str::trim)
    .any(|piece| !piece.is_empty() && is_membership_part(piece))
}

/// Where the name line a creator's content holds begins and ends: inside the block reader's `\lx@author@withsup{…}` and
/// a command the name line is set in as its one argument (a raw IEEE class's `\IEEEauthorblockN{…}`, which the kernel's
/// split puts round each name: icrc2009, 0907.0548, 1111.2718), each holding the whole content; the whole content
/// otherwise.
pub(in crate::base_utilities) fn name_line_span(v: &[Token]) -> (usize, usize) {
  let (mut start, mut end) = (0, v.len());
  for _ in 0..2 {
    while end > start && v[end - 1] == T_SPACE!() {
      end -= 1;
    }
    if end - start < 3 {
      break;
    }
    let cs = v[start];
    if cs.get_catcode() != Catcode::CS
      || v[start + 1].get_catcode() != Catcode::BEGIN
      || skip_group(&v[..end], start + 1) != end
      || (cs != T_CS!("\\lx@author@withsup")
        && (is_name_annotation(&cs)
          || is_footnote_mark(&cs)
          || is_author_mark(&cs)
          || cs.with_str(|n| n.starts_with("\\lx@"))))
    {
      break;
    }
    start += 2;
    end -= 1;
  }
  (start, end)
}

/// A creator's content as `\lx@add@author` reads it (frontmatter_fields.rs): the membership grade its name line writes
/// after the name's comma the author's contact ([`written_membership_out`]), inside what the line is set in
/// ([`name_line_span`]) as in a bare name.
pub(in crate::base_utilities) fn membership_read_out(content: Tokens) -> Tokens {
  let v = content.unlist_ref();
  let (start, end) = name_line_span(v);
  match written_membership_out(&v[start..end]) {
    Some(name) => {
      let mut out = v[..start].to_vec();
      out.extend(name);
      out.extend_from_slice(&v[end..]);
      Tokens::new(out)
    },
    None => content,
  }
}

/// What follows a name's comma parted into the text it prints, its spacing trimmed, and the marks and notes written in
/// it (a note or mark command with its arguments, [`is_name_annotation`], [`is_footnote_mark`], [`is_mark_command`]; a
/// formula, `$^{1}$`; a block reader's contact call after the name, `\lx@add@affiliation[annotate=3]{…}`: 1611.04834's
/// `and Warren J. Gross, \emph{Senior Member, IEEE}\\ Department …`).
fn grade_and_annotations(tail: &[Token]) -> (Vec<Token>, Vec<Token>) {
  let kernel_call = |t: &Token| t.with_str(|s| s.starts_with("\\lx@add@"));
  let (mut text, mut annotations, mut i) = (Vec::new(), Vec::new(), 0);
  while i < tail.len() {
    let t = tail[i];
    let start = i;
    i += 1;
    if t.get_catcode() == Catcode::MATH {
      while i < tail.len() && tail[i].get_catcode() != Catcode::MATH {
        i += 1;
      }
      i = (i + 1).min(tail.len());
      annotations.extend_from_slice(&tail[start..i]);
    } else if t.get_catcode() == Catcode::CS
      && (is_name_annotation(&t) || is_footnote_mark(&t) || is_mark_command(&t) || kernel_call(&t))
    {
      if tail.get(i) == Some(&T_OTHER!("["))
        && let Some(close) = tail[i..].iter().position(|u| *u == T_OTHER!("]"))
      {
        i += close + 1;
      }
      if kernel_call(&t) {
        while tail
          .get(i)
          .is_some_and(|u| u.get_catcode() == Catcode::BEGIN)
        {
          i = skip_group(tail, i).min(tail.len());
        }
      } else if !is_footnote_mark(&t) {
        i = skip_group(tail, i).min(tail.len());
      }
      annotations.extend_from_slice(&tail[start..i]);
    } else if t.get_catcode() == Catcode::BEGIN {
      i = skip_group(tail, start).min(tail.len());
      text.extend_from_slice(&tail[start..i]);
    } else {
      text.push(t);
    }
  }
  let from = text
    .iter()
    .position(|t| !is_spacing(t))
    .unwrap_or(text.len());
  let to = text
    .iter()
    .rposition(|t| !is_spacing(t))
    .map_or(from, |k| k + 1);
  (text[from..to].to_vec(), annotations)
}

/// Whether a names-line piece reads as one person's name: by its shape, or marked after the name and no institution or
/// address, as the cautious split reads a list by its marks (`marked_names`).
fn reads_as_one_name(piece: &Tokens) -> bool {
  let text = visible_name_text(piece.unlist_ref());
  name_count(&text) == 1
    || (marked_after_name(piece) && !reads_as_affiliation(piece) && !text.contains('@'))
}

/// The name a piece holds before the collaboration it speaks for or that brackets set after it, and that
/// collaboration: `J.~Zapatero$^{5}$ (The MAGIC Collaboration)` (0910.4381), `C.~Cole$^{1,2}$ \vspace{0.2cm} (XYZ
/// Collaboration)` (2212.06489), `Bob Baker for the ATLAS Collaboration` (1502.03246's `for the Hall A DVCS
/// collaboration`); `None` when no such remark follows a name.
fn name_and_remark(piece: &Tokens) -> Option<(Tokens, Tokens)> {
  let v = piece.unlist_ref();
  let (mut depth, mut in_math) = (0i32, false);
  for j in 0..v.len() {
    let t = v[j];
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth -= 1,
      Catcode::MATH => in_math = !in_math,
      _ => {},
    }
    if j == 0 || depth != 0 || in_math {
      continue;
    }
    // (a word after a space, a tie, a mark or a command: `J.K. de Jong \IEEEauthorrefmark{2}for the MINOS
    // collaboration.`, 1111.2718)
    let word_start = !matches!(v[j - 1].get_catcode(), Catcode::LETTER | Catcode::OTHER);
    let candidate = t == T_OTHER!("(")
      || t == T_OTHER!("[")
      || (word_start && (t == T_LETTER!("f") || t == T_LETTER!("o")));
    if !candidate {
      continue;
    }
    let name = Tokens::new(v[..spacing_end(&v[..j])].to_vec());
    let remark = Tokens::new(v[j..].to_vec());
    let remark_text = visible_name_text(remark.unlist_ref());
    if (reads_as_collaboration(&remark_text, false)
      || (annotated(&v[..j]) && speaks_for_after_notes(&remark_text)))
      && reads_as_one_name(&name)
    {
      return Some(name_takes_annotations(name, remark));
    }
  }
  None
}

/// Whether a name carries a note or a mark: a note or mark command ([`is_name_annotation`], [`is_footnote_mark`],
/// [`is_author_mark`]), a superscript or a formula.
fn annotated(name: &[Token]) -> bool {
  name.iter().any(|t| {
    *t == T_SUPER!()
      || t.get_catcode() == Catcode::MATH
      || (t.get_catcode() == Catcode::CS
        && (is_name_annotation(t)
          || is_footnote_mark(t)
          || is_author_mark(t)
          || *t == T_CS!("\\inst")))
  })
}

/// A name and its collaboration remark with the name's annotations the remark holds moved onto the name — a note
/// reference, a footnote, a `\thanks` (`is_name_annotation`, `is_footnote_mark`) and its arguments: `Jan Fiete
/// Grosse-Oetringhaus (for the ALICE\fnref{col1} Collaboration)` (1408.0414, 1408.5384) is the name's `\fnref`, which
/// in the contact asked for a label no creator answers (two `Warning:unexpected:annotation`); so are the marks and
/// notes after the remark and llncs's `\fnmsep` (webofc's `\fnsep`) between them (`Laure Massacrier on behalf of the
/// LHCb Collaboration\inst{1,2}\fnsep\thanks{\email{…}}`, 1512.03605: her institutes and e-mail).
fn name_takes_annotations(name: Tokens, remark: Tokens) -> (Tokens, Tokens) {
  let v = remark.unlist_ref();
  let (mut kept, mut moved) = (Vec::with_capacity(v.len()), Vec::new());
  let mut i = 0;
  while i < v.len() {
    let t = v[i];
    if t == T_CS!("\\fnmsep") || t == T_CS!("\\fnsep") {
      moved.push(t);
      i += 1;
      continue;
    }
    if t.get_catcode() != Catcode::CS || !(is_name_annotation(&t) || is_footnote_mark(&t)) {
      kept.push(t);
      i += 1;
      continue;
    }
    let start = i;
    i += 1;
    if v.get(i) == Some(&T_OTHER!("["))
      && let Some(close) = v[i..].iter().position(|u| *u == T_OTHER!("]"))
    {
      i += close + 1;
    }
    while v.get(i).is_some_and(|u| u.get_catcode() == Catcode::BEGIN) {
      i = skip_group(v, i);
    }
    moved.extend_from_slice(&v[start..i.min(v.len())]);
  }
  if moved.is_empty() {
    return (name, remark);
  }
  let mut named = name.unlist();
  named.extend(moved);
  (Tokens::new(named), Tokens::new(kept))
}

/// A creator's whole content that names a collaboration the authors made before it speak for, framed as a remark
/// ([`Framing::Framed`]: "on behalf of", "for", "representing", brackets) — elsarticle's `\author{On behalf of the KM3NeT
/// Consortium}` after its three authors (1201.5050), which pdflatex prints in the names line: the collaboration contact
/// of every author before it that carries none (`annotate=new`, [`collaboration_contact_call`]), no creator. Not
/// without an author before it, nor with a note or mark of its own; a collaboration named alone (`\collab{The XYZ
/// Collaboration}`, webofc's `\and The SND Collaboration`, 1912.05868) stays a creator, as before.
pub(in crate::base_utilities) fn whole_name_collaboration(
  content: &Tokens,
) -> Result<Option<Tokens>> {
  let v = content.unlist_ref();
  let (start, end) = name_line_span(v);
  let inner = &v[start..end];
  if queued_author_count() == 0
    || inner.iter().any(|t| {
      t.get_catcode() == Catcode::CS
        && (is_name_annotation(t) || is_footnote_mark(t) || is_author_mark(t))
    })
  {
    return Ok(None);
  }
  if collaboration_framing(&visible_name_text(inner), false, true) != Some(Framing::Framed) {
    return Ok(None);
  }
  Ok(Some(Tokens::new(collaboration_contact_call(
    &Tokens::new(inner.to_vec()),
    None,
  )?)))
}

/// The collaboration a creator's name line ends with inside the command it is set in ([`name_line_span`]), which the
/// author line's own reading (`collaboration_split`) does not look into: a line of its own after the name's `\\`
/// (`\IEEEauthorblockN{… Monica Brigida\IEEEauthorrefmark{3}\\on behalf of the \emph{Fermi}-LAT Collaboration}`,
/// icrc2009's raw IEEE block, 0907.0548) or a remark after the name ([`name_and_remark`]: `\IEEEauthorblockN{J.K. de
/// Jong \IEEEauthorrefmark{1}\IEEEauthorrefmark{2}for the MINOS collaboration.}`, 1111.2718). The content without it,
/// and its contact, made after the creator: every author before it that carries none, as a collaboration after the
/// names covers the names of their line (`annotate=new`, [`collaboration_contact_call`]). `None` for a bare name line,
/// which the author line's reading read.
pub(in crate::base_utilities) fn wrapped_collaboration_out(
  content: &Tokens,
) -> Result<Option<(Tokens, Tokens)>> {
  let v = content.unlist_ref();
  let (start, end) = name_line_span(v);
  if start == 0 || (start == 2 && v[0] == T_CS!("\\lx@author@withsup")) {
    return Ok(None);
  }
  let line = &v[start..end];
  let mut depth = 0i32;
  let breaks: Vec<usize> = (0..line.len())
    .filter(|&k| {
      match line[k].get_catcode() {
        Catcode::BEGIN => depth += 1,
        Catcode::END => depth -= 1,
        _ => {},
      }
      depth == 0 && line[k] == T_CS!("\\\\")
    })
    .collect();
  let split = if let [.., last] = breaks.as_slice() {
    let mut from = last + 1;
    if line.get(from) == Some(&T_OTHER!("*")) {
      from += 1;
    }
    if line.get(from) == Some(&T_OTHER!("["))
      && let Some(close) = line[from..].iter().position(|u| *u == T_OTHER!("]"))
    {
      from += close + 1;
    }
    let rest = Tokens::new(line[from..].to_vec());
    let name = &line[..spacing_end(&line[..*last])];
    (visible_name_text(name).chars().any(char::is_alphabetic)
      && matches!(collaboration_line_reading(&rest), Some(f) if f != Framing::Unframed))
    .then(|| (Tokens::new(name.to_vec()), rest))
  } else {
    name_and_remark(&Tokens::new(line.to_vec()))
  };
  let Some((name, remark)) = split else {
    return Ok(None);
  };
  let mut out = v[..start].to_vec();
  out.extend(name.unlist());
  out.extend_from_slice(&v[end..]);
  Ok(Some((
    Tokens::new(out),
    Tokens::new(collaboration_contact_call(&remark, None)?),
  )))
}

/// The text a collaboration's contact holds: as written, without the layout around it (`\center`, `\vspace{0.2cm}`)
/// and the affiliation marks on it (`LHD experimental group$^{2}$`, 1711.07097: a mark no contact answers); a footnote
/// symbol it shows stays (`on behalf of the "Flat Blazars" collaboration$^*$`, astro-ph9712234).
fn collaboration_content(text: &Tokens) -> Tokens {
  let v = text.unlist_ref();
  let mut start = 0;
  loop {
    let past = start + leading_spacing_end(&v[start..]);
    let past = past
      + v[past..]
        .iter()
        .take_while(|t| {
          t.get_catcode() == Catcode::CS
            && t.with_str(|s| {
              matches!(
                s,
                "\\center"
                  | "\\centering"
                  | "\\raggedright"
                  | "\\raggedleft"
                  | "\\noindent"
                  | "\\par"
              )
            })
        })
        .count();
    if past == start {
      break;
    }
    start = past;
  }
  let v = &v[start..];
  let mut v = &v[..spacing_end(v)];
  // (a heading's colon is no part of the collaboration's name, 1204.5689)
  while let [rest @ .., last] = v
    && (*last == T_OTHER!(":") || *last == T_SPACE!())
  {
    v = rest;
  }
  let mut out = Vec::with_capacity(v.len());
  let mut i = 0;
  while i < v.len() {
    let t = v[i];
    if t.get_catcode() == Catcode::MATH
      && let Some(close) = v[i + 1..]
        .iter()
        .position(|u| u.get_catcode() == Catcode::MATH)
      && only_marks_and_notes(&v[i + 1..i + 1 + close])
    {
      i += close + 2;
      continue;
    }
    if is_author_mark(&t) || t == T_CS!("\\altaffilmark") {
      i = skip_group(v, i + 1);
      continue;
    }
    out.push(t);
    i += 1;
  }
  Tokens::new(out)
}

/// The contact a collaboration makes (Base_Utility.pool.ltxml:517-521): on the last `covers` authors made
/// (`annotate=N`), the names it covers counted where it is read (`Collaborations`) — `Some(1)` the creator being made,
/// read in its content —; with `None`, on every author before it that carries no collaboration yet (`annotate=new`, as
/// aastex's `\collaboration` reaches them, aas_support.sty.ltxml:126).
fn collaboration_contact_call(text: &Tokens, covers: Option<usize>) -> Result<Vec<Token>> {
  let annotate = covers.map_or_else(|| s!("new"), |covers| covers.max(1).to_string());
  let keyvals = mouth::tokenize_internal(TeXString::assembled(s!(
    "role=collaboration,annotate={annotate}"
  )));
  Ok(
    Invocation!(T_CS!("\\lx@add@contact"), vec![
      Some(keyvals),
      Some(collaboration_content(text))
    ])
    .unlist(),
  )
}

/// The definitions in force where a class's title code reads `\@author`: the statements of the groups still open there
/// in its `\@maketitle` (else a dropped `\maketitle`'s) body — `\def`, `\gdef`, `\let`, `\newcommand`, `\providecommand`
/// — with a global one kept past its group, in source order; each with its definee.
pub(in crate::base_utilities) fn title_code_author_definitions() -> Vec<(Token, Vec<Token>)> {
  [T_CS!("\\@maketitle"), T_CS!("\\lx@dropped@maketitle")]
    .iter()
    .find_map(|cs| match lookup_meaning(cs) {
      Some(Stored::Expandable(definition)) => match definition.get_expansion() {
        Some(ExpansionBody::Tokens(body)) => definitions_at_author(body.unlist_ref()),
        _ => None,
      },
      _ => None,
    })
    .unwrap_or_default()
}

/// The definition statements of `body`'s groups open at its first `\@author` ([`title_code_author_definitions`]),
/// in source order; `None` when it reads no `\@author`. Groups are brace groups, `\bgroup`/`\egroup`,
/// `\begingroup`/`\endgroup` and environments. A statement runs locally where it is replayed: a `\gdef` is a `\def`
/// there, and the `\global` before a definition is left out. `\edef`/`\xdef` are not replayed (expanded now, not at
/// the title code), nor `\renewcommand` (of a command already defined, which stands). A statement whose shape is not
/// read whole (a `\def` without its body's `{`, a `\newcommand` without a body) is left out, and so is one that closes
/// a group it did not open (2601.13359's `\def\affiliations{\egroup\par\Large\bgroup\rm}`, `\let\x\egroup`): the
/// title code's layout between its own groups, which the author content is not inside.
pub(in crate::base_utilities) fn definitions_at_author(
  body: &[Token],
) -> Option<Vec<(Token, Vec<Token>)>> {
  // the index after the balanced group starting at `i` (a BEGIN), else after the one token there
  let group_end = |i: usize| -> usize {
    if body.get(i).map(|t| t.get_catcode()) != Some(Catcode::BEGIN) {
      return (i + 1).min(body.len());
    }
    let mut depth = 0usize;
    for (j, t) in body.iter().enumerate().skip(i) {
      match t.get_catcode() {
        Catcode::BEGIN => depth += 1,
        Catcode::END => {
          depth -= 1;
          if depth == 0 {
            return j + 1;
          }
        },
        _ => {},
      }
    }
    body.len()
  };
  let skip_spaces = |mut j: usize| -> usize {
    while body.get(j) == Some(&T_SPACE!()) {
      j += 1;
    }
    j
  };
  // each statement with the group it is made in (None: global)
  let mut statements: Vec<(Token, Vec<Token>, Option<usize>)> = Vec::new();
  let mut open: Vec<usize> = vec![0];
  let mut groups = 0usize;
  let mut i = 0;
  while i < body.len() {
    let t = body[i];
    let name = t.with_str(str::to_string);
    let opens = t.get_catcode() == Catcode::BEGIN
      || matches!(name.as_str(), "\\bgroup" | "\\begingroup" | "\\begin");
    let closes = t.get_catcode() == Catcode::END
      || matches!(name.as_str(), "\\egroup" | "\\endgroup" | "\\end");
    if opens {
      groups += 1;
      open.push(groups);
      i += 1;
      continue;
    }
    if closes {
      if open.len() > 1 {
        open.pop();
      }
      i += 1;
      continue;
    }
    if t == T_CS!("\\@author") {
      return Some(
        statements
          .into_iter()
          .filter(|(_, _, group)| group.is_none_or(|g| open.contains(&g)))
          .map(|(definee, statement, _)| (definee, statement))
          .collect(),
      );
    }
    let global = name == "\\gdef" || (i > 0 && body[i - 1] == T_CS!("\\global"));
    // the definee, where the statement ends, and its first token as replayed
    let read: Option<(Option<Token>, usize, Token)> = match name.as_str() {
      "\\def" | "\\gdef" if t.get_catcode() == Catcode::CS => {
        // the parameter text runs to the body's `{`
        let mut j = i + 2;
        while j < body.len() && body[j].get_catcode() != Catcode::BEGIN {
          j += 1;
        }
        (j < body.len()).then(|| (body.get(i + 1).copied(), group_end(j), T_CS!("\\def")))
      },
      "\\let" if t.get_catcode() == Catcode::CS => {
        let mut j = skip_spaces(i + 2);
        if body.get(j) == Some(&T_OTHER!("=")) {
          j += 1;
        }
        j = skip_spaces(j);
        body
          .get(j)
          .filter(|target| !matches!(target.get_catcode(), Catcode::BEGIN | Catcode::END))
          .map(|_| (body.get(i + 1).copied(), j + 1, t))
      },
      "\\newcommand" | "\\providecommand" if t.get_catcode() == Catcode::CS => {
        let mut j = i + 1;
        if body.get(j) == Some(&T_OTHER!("*")) {
          j += 1;
        }
        j = skip_spaces(j);
        let definee = match body.get(j) {
          Some(b) if b.get_catcode() == Catcode::BEGIN => body.get(j + 1).copied(),
          other => other.copied(),
        };
        j = skip_spaces(group_end(j));
        while body.get(j) == Some(&T_OTHER!("[")) {
          while j < body.len() && body[j] != T_OTHER!("]") {
            j += 1;
          }
          j = skip_spaces(j + 1);
        }
        body
          .get(j)
          .filter(|b| b.get_catcode() != Catcode::END)
          .map(|_| (definee, group_end(j), t))
      },
      _ => None,
    };
    match read {
      Some((Some(definee), stop, head))
        if matches!(definee.get_catcode(), Catcode::CS | Catcode::ACTIVE)
          && stop <= body.len()
          && closes_only_its_own_groups(&body[i + 2..stop]) =>
      {
        let mut statement = vec![head];
        statement.extend_from_slice(&body[i + 1..stop]);
        statements.push((
          definee,
          statement,
          (!global).then(|| *open.last().unwrap_or(&0)),
        ));
        i = stop;
      },
      _ => i += 1,
    }
  }
  None
}

/// Whether `tokens` (a definition's meaning) close only the groups and environments they open (`\bgroup`,
/// `\begingroup`, `\begin`): an author separator `\def\sep{\end{tabular}\begin{tabular}{c}}` does not.
fn closes_only_its_own_groups(tokens: &[Token]) -> bool {
  let mut depth = 0usize;
  for t in tokens {
    if t.get_catcode() != Catcode::CS {
      continue;
    }
    match t.with_str(str::to_string).as_str() {
      "\\bgroup" | "\\begingroup" | "\\begin" => depth += 1,
      "\\egroup" | "\\endgroup" | "\\end" => match depth.checked_sub(1) {
        Some(d) => depth = d,
        None => return false,
      },
      _ => {},
    }
  }
  true
}

/// Whether `tokens` hold nothing to read but marks and notes: spaces (`~`, `\ `, `\,`), a note or mark command with
/// its arguments ([`NAME_ANNOTATIONS`], `\footnotemark[…]`, the mark commands of [`mark_commands`]:
/// `\textsuperscript`, `\IEEEauthorrefmark`, `\authorrefmark`), a superscript or subscript, or math holding only those
/// (`$^{*}$`). An author written as a macro (`\A`) is something to read.
pub(in crate::base_utilities) fn only_marks_and_notes(tokens: &[Token]) -> bool {
  let mut i = 0;
  let mut any = false;
  while i < tokens.len() {
    let t = tokens[i];
    match t.get_catcode() {
      Catcode::SPACE => i += 1,
      Catcode::ACTIVE if t.with_str(|s| s == "~") => i += 1,
      Catcode::CS if t == T_CS!("\\ ") || t == T_CS!("\\,") => i += 1,
      // `${}^{1}$`'s empty group
      Catcode::BEGIN
        if tokens
          .get(i + 1)
          .is_some_and(|n| n.get_catcode() == Catcode::END) =>
      {
        i += 2
      },
      // (a mark command too: ieeeaccess's `\authorrefmark{2}` after a list's closing period, 2201.07394)
      Catcode::CS if is_footnote_mark(&t) || is_name_annotation(&t) || is_mark_command(&t) => {
        any = true;
        i += 1;
        if tokens.get(i) == Some(&T_OTHER!("[")) {
          while i < tokens.len() && tokens[i] != T_OTHER!("]") {
            i += 1;
          }
          i += 1;
        }
        if !is_footnote_mark(&t) {
          i = skip_group(tokens, i);
        }
      },
      Catcode::SUPER | Catcode::SUB => {
        any = true;
        i = skip_group(tokens, i + 1);
      },
      Catcode::MATH => {
        let Some(close) = tokens[i + 1..]
          .iter()
          .position(|t| t.get_catcode() == Catcode::MATH)
        else {
          return false;
        };
        if !only_marks_and_notes(&tokens[i + 1..i + 1 + close])
          || tokens[i + 1..i + 1 + close].is_empty()
        {
          return false;
        }
        any = true;
        i += close + 2;
      },
      _ => return false,
    }
  }
  any
}

/// How the collaboration a line of an author block names is framed ([`collaboration_line_framing`]), `None` for a line
/// naming none or marked.
pub(in crate::base_utilities) fn collaboration_line_reading(line: &Tokens) -> Option<Framing> {
  if author_mark_position(line).is_some() {
    return None;
  }
  collaboration_line_framing(&visible_name_text(line.unlist_ref()))
}
