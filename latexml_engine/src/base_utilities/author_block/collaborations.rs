//! A collaboration or team named in an author line: a `<contact role="collaboration">` on the names it covers, no
//! creator of its own and no part of a name (FIDELITY_CHARTER §4, user ruling 2026-11-01; OXIDIZED_DESIGN_DIVERGENCES
//! #946). The contact is the one Perl's aastex binding makes of `\collaboration{n}{name}`: `\lx@add@contact[role=
//! collaboration,annotate=new]` (aas_support.sty.ltxml:126), on the authors before it that carry none — here those of
//! the author call's `\and` group, `annotate=new:N`.

use super::*;

/// Whether a line of an author block, a line of its own, names a collaboration ([`reads_as_collaboration`], a team word
/// framed): `(XYZ Collaboration)` under the names (2410.11607), `on behalf of the LHCb collaboration` (1905.11825).
pub(in crate::base_utilities) fn collaboration_line(line: &Tokens) -> bool {
  collaboration_line_reading(line).is_some()
}

/// How the collaboration a line of an author block names is framed ([`collaboration_line_framing`]), `None` for a line
/// naming none or marked.
pub(in crate::base_utilities) fn collaboration_line_reading(line: &Tokens) -> Option<Framing> {
  if author_mark_position(line).is_some() {
    return None;
  }
  collaboration_line_framing(&visible_name_text(line.unlist_ref()))
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
