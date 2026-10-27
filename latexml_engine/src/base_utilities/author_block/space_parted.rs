//! Names parted by horizontal space alone: a wide space, or a name's marks and any space after them.

use super::*;

/// An author line's names parted by horizontal space alone, each such space written as the comma the split reads
/// (OXIDIZED_DESIGN_DIVERGENCES #897): pdflatex sets the names apart, where the split saw one name. At the line's top
/// level, outside math, after a name letter, two kinds of space part two names:
///  - a wide horizontal space ([`wide_space_end`]: `\quad`, `\qquad`, a group of them, `\hspace{…}`, `\hskip`/`\kern`
///    a wide length, `\enspace`, `\enskip`, `\hfill`), marked names or not: `Sergio Antoy\inst{1} \kern1em Michael
///    Hanus\inst{2}` (llncs 1909.09562), `{\normalsize Georgios Milis\textsuperscript{1} \quad Panagiotis P.
///    Filntisis…}` (2312.06613), `Jiahao Su\thanks{…}$^{\;\;12}$\thanks{…} \hskip2em Wonmin Byeon` (2002.09131);
///  - a name's marks or notes ([`name_marks_end`]) and any space after them (` `, `\ `, `\:`, `\,`, `\;`, `~`, a line
///    ending in `\`): `Sijia Chen$^{1,2,3,*}$ \: Yibo Wang$^{1,2,3,}$\thanks{…} \: Yi-Feng Wu$^{3}$` (2406.07115),
///    `Junxian Li$^{1}$\thanks{…} \⏎Ruixuan Yang$^{2*}$` (2609.32353), `Li Lucy\textsuperscript{2} \, Su Lin Blodgett`
///    (2310.15398), `J. Wang\altaffilmark{1} Y. F. Mao \altaffilmark{1} and J. Y. Wei` (aastex 0811.3255),
///    `\textsc{Francesco Tombesi}\altaffilmark{3,4}⏎\textsc{Demosthenes Kazanas}` (1311.0077), `Edward
///    Liverts\inst{1}\fnmsep\thanks{…}⏎Michael Mond\inst{1}` (0806.3855), `Hao Wang$^{1,2,*}$⏎Defu Lian$^{1,2}$`
///    (2308.08563).
///
/// A narrow space with no mark before it parts nothing (`Florian P.\ Busch`, 2212.12575; ecai's `Ann Able \ Bob Baker`,
/// 77n r2). In a line whose marks or notes lead its names a mark is the name's after it, so there a space parts two
/// names before the next name's marks (`$^{1}$Ann Able \: $^{2}$Bob Baker`; `\thanks{…} {Carla Schenker,} ⏎\thanks{…}
/// {Xiulin Wang,}`, 2406.12338, each note its own name's) or a wide space before the name. Only where every piece then
/// reads as a name or qualifies one and two or more read as names ([`name_count`]): `John Smith\quad Bell Labs` and
/// `Mc\kern1pt Donald` stay as written.
pub(in crate::base_utilities) fn space_parted_names(line: Tokens) -> Tokens {
  let v = line.unlist_ref();
  if !v.iter().any(|t| {
    matches!(
      t.get_catcode(),
      Catcode::CS | Catcode::MATH | Catcode::SUPER | Catcode::BEGIN
    )
  }) {
    return line;
  }
  // (the names led by their marks or notes: `$^{1}$Ann Able`, `\thanks{…} {Carla Schenker,}`, 2406.12338)
  let lead = leading_spacing_end(v);
  let prefix_marked = marker_leads(&line) || name_marks_end(v, lead) > lead;
  let mut cuts: Vec<(usize, usize)> = Vec::new();
  let mut piece_start = 0;
  let mut depth = 0usize;
  let mut in_math = false;
  let mut i = 0;
  while i < v.len() {
    if depth == 0 && !in_math {
      if prefix_marked {
        // a space, then the next name's marks or notes and the name
        if is_narrow_space(&v[i]) || wide_space_end(v, i).is_some() {
          let gap = space_run_end(v, i);
          let wide = (i..gap).any(|k| wide_space_end(v, k).is_some());
          let marks = name_marks_end(v, gap);
          if names_before(&v[piece_start..i])
            && ((marks > gap && begins_a_name(v, space_run_end(v, marks), false))
              || (wide && begins_a_name(v, gap, true)))
          {
            cuts.push((i, gap));
            piece_start = gap;
          }
          i = gap;
          continue;
        }
      } else {
        let marks = name_marks_end(v, i);
        if marks > i {
          let gap = space_run_end(v, marks);
          if gap > marks && names_before(&v[piece_start..i]) && begins_a_name(v, gap, false) {
            cuts.push((marks, gap));
            piece_start = gap;
            i = gap;
          } else {
            i = marks;
          }
          continue;
        }
        if wide_space_end(v, i).is_some() {
          let gap = space_run_end(v, i);
          if names_before(&v[piece_start..i]) && begins_a_name(v, gap, false) {
            // (the spaces before it go with it)
            let mut start = i;
            while start > piece_start && is_narrow_space(&v[start - 1]) {
              start -= 1;
            }
            cuts.push((start, gap));
            piece_start = gap;
          }
          i = gap;
          continue;
        }
      }
    }
    // (`\(`…`\)` is math as `$`…`$` is: a mark's space inside `Ann Able\( ^{1} \)` parts nothing)
    match v[i].get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth = depth.saturating_sub(1),
      Catcode::MATH if depth == 0 => in_math = !in_math,
      Catcode::CS if depth == 0 && v[i] == T_CS!("\\(") => in_math = true,
      Catcode::CS if depth == 0 && v[i] == T_CS!("\\)") => in_math = false,
      _ => {},
    }
    i += 1;
  }
  if cuts.is_empty() {
    return line;
  }
  let mut out: Vec<Token> = Vec::with_capacity(v.len() + cuts.len());
  let mut from = 0;
  for (start, end) in cuts {
    out.extend_from_slice(&v[from..start]);
    out.extend([T_OTHER!(","), T_SPACE!()]);
    from = end;
  }
  out.extend_from_slice(&v[from..]);
  // (a comma set in no width, `\rlap,`/`\llap,`, is the comma it prints: kept, the marks the split moves before a
  // glued comma become `\rlap`'s argument — `Tatsuo~{\sc Kobayashi}\rlap,${}^{1,}$\footnote{…}⏎Hiroaki…`,
  // hep-ph0411288's `\rlap${}^{1,}$`, its math broken)
  let lapped = |k: usize| {
    (out[k] == T_CS!("\\rlap") || out[k] == T_CS!("\\llap"))
      && out.get(k + 1) == Some(&T_OTHER!(","))
  };
  let out: Vec<Token> = (0..out.len())
    .filter(|&k| !lapped(k))
    .map(|k| out[k])
    .collect();
  if name_count(visible_name_text(&out).trim()) >= 2 {
    Tokens::new(out)
  } else {
    line
  }
}

/// Whether a piece of an author line prints a letter of name text ([`visible_name_text`]: not a graphic's file name,
/// `\includegraphics[scale=0.06]{orcid.pdf}\hspace{1mm}Hossein Bahak`, 2312.07592).
fn names_before(tokens: &[Token]) -> bool {
  visible_name_text(tokens).chars().any(char::is_alphabetic)
}

/// Whether `t` is a narrow space: a space, a tie, a control space or return, `\,`, `\:`, `\>`, `\;`, `\thinspace`,
/// `\space` ([`is_spacing`], [`is_spacing_command`]).
fn is_narrow_space(t: &Token) -> bool { is_spacing(t) || is_spacing_command(t) }

/// The end of the run of horizontal space that starts at `v[i]` (`i` when none): narrow spaces ([`is_narrow_space`]) and
/// wide ones ([`wide_space_end`]).
fn space_run_end(v: &[Token], mut i: usize) -> usize {
  while i < v.len() {
    if let Some(end) = wide_space_end(v, i) {
      i = end;
    } else if is_narrow_space(&v[i]) {
      i += 1;
    } else {
      break;
    }
  }
  i
}

/// The end of the wide horizontal space at `v[i]`, if one stands there: `\quad`, `\qquad`, `\enspace`, `\enskip`
/// (latex.ltx `\kern.5em`, `\hskip.5em`), `\hfill`, `\hfil`, `\hspace` with its `*` and length, `\hskip` or `\kern` with
/// a length of 4pt or more ([`past_glue`], [`wide_length`]: not a kern inside a word; `\hspace`'s length the same), or a
/// group holding only spacing
/// and a wide space (`{\quad}`, 2609.03563).
pub(in crate::base_utilities) fn wide_space_end(v: &[Token], i: usize) -> Option<usize> {
  let t = v.get(i)?;
  if t.get_catcode() == Catcode::BEGIN {
    let end = skip_group(v, i);
    if end > v.len() || end < i + 3 || v[end - 1].get_catcode() != Catcode::END {
      return None;
    }
    let inner = &v[i + 1..end - 1];
    let mut j = 0;
    let mut wide = false;
    while j < inner.len() {
      if let Some(next) = wide_space_end(inner, j) {
        (wide, j) = (true, next);
      } else if is_narrow_space(&inner[j]) {
        j += 1;
      } else {
        return None;
      }
    }
    return wide.then_some(end);
  }
  if t.get_catcode() != Catcode::CS {
    return None;
  }
  t.with_str(|s| match s {
    "\\quad" | "\\qquad" | "\\enspace" | "\\enskip" | "\\hfill" | "\\hfil" => Some(i + 1),
    "\\hspace" => {
      let mut j = i + 1;
      if v.get(j) == Some(&T_OTHER!("*")) {
        j += 1;
      }
      while v.get(j) == Some(&T_SPACE!()) {
        j += 1;
      }
      if v.get(j).is_none_or(|g| g.get_catcode() != Catcode::BEGIN) {
        return None;
      }
      let end = skip_group(v, j);
      wide_length(&v[j + 1..end.saturating_sub(1).max(j + 1)]).then_some(end)
    },
    "\\hskip" | "\\kern" => {
      let end = past_glue(v, i + 1);
      wide_length(&v[i + 1..end]).then_some(end)
    },
    _ => None,
  })
}

/// Whether a length as written (`1em`, `2em plus 1fil`, ` 10pt`) is wide enough to part two words: 4pt or more, an em
/// read as 10pt, an ex as 4.3pt (cmr10), or `\fill`, `\stretch{…}` (`\hspace{\fill}`). Negative and unreadable lengths
/// (`\hspace{1mm}` beside a graphic, a register) are not.
fn wide_length(tokens: &[Token]) -> bool {
  if tokens
    .iter()
    .find(|t| t.get_catcode() != Catcode::SPACE)
    .is_some_and(|t| *t == T_CS!("\\fill") || *t == T_CS!("\\stretch"))
  {
    return true;
  }
  let text: String = tokens
    .iter()
    .filter(|t| t.get_catcode() != Catcode::SPACE)
    .map(|t| t.to_string())
    .collect();
  let number_end = text
    .find(|c: char| !(c.is_ascii_digit() || c == '.' || c == '+'))
    .unwrap_or(text.len());
  let Ok(value) = text[..number_end].trim_start_matches('+').parse::<f64>() else {
    return false;
  };
  let unit = text[number_end..].get(..2).unwrap_or("");
  let points = match unit {
    "pt" => 1.0,
    "em" => 10.0,
    "ex" => 4.3,
    "mm" => 2.845,
    "cm" => 28.45,
    "in" => 72.27,
    "bp" => 1.004,
    "pc" => 12.0,
    "dd" => 1.07,
    "cc" => 12.84,
    "sp" => 1.0 / 65536.0,
    _ => return false,
  };
  value * points >= 4.0
}

/// The end of the run of a name's marks and notes that starts at `v[i]` (`i` when none): `$^{…}$` and other math holding
/// only marks ([`only_marks_and_notes`]), a superscript, a mark or note command with its `[…]` and braced argument (the
/// mark commands of [`mark_commands`], [`NAME_ANNOTATIONS`]: `\textsuperscript`, `\thanks`, `\inst`, `\altaffilmark`,
/// a document's note macro), `\footnotemark[…]`, and llncs's `\fnmsep`/`\fnsep` between two of them.
pub(in crate::base_utilities) fn name_marks_end(v: &[Token], mut i: usize) -> usize {
  let start = i;
  loop {
    let Some(t) = v.get(i) else {
      return i;
    };
    match t.get_catcode() {
      Catcode::MATH => {
        let Some(close) = v[i + 1..]
          .iter()
          .position(|u| u.get_catcode() == Catcode::MATH)
        else {
          return i;
        };
        let inner = &v[i + 1..i + 1 + close];
        if inner.is_empty() || !only_marks_and_notes(inner) {
          return i;
        }
        i += close + 2;
      },
      Catcode::SUPER => {
        let end = skip_group(v, i + 1);
        if end > v.len() {
          return i;
        }
        i = end;
      },
      // (a group holding only marks: `Jia-Bin Huang{$^{2}$} ~ David W. Jacobs{$^{2}$}`, 2406.09417)
      Catcode::BEGIN => {
        let end = skip_group(v, i);
        if end > v.len() || end < i + 3 || v[end - 1].get_catcode() != Catcode::END {
          return i;
        }
        let inner = &v[i + 1..end - 1];
        if name_marks_end(inner, 0) != inner.len() {
          return i;
        }
        i = end;
      },
      Catcode::CS if is_footnote_mark(t) || is_name_annotation(t) || is_mark_command(t) => {
        let mut j = i + 1;
        if v.get(j) == Some(&T_OTHER!("["))
          && let Some(close) = v[j..].iter().position(|u| *u == T_OTHER!("]"))
        {
          j += close + 1;
        }
        if !is_footnote_mark(t) {
          if v.get(j).is_none_or(|g| g.get_catcode() != Catcode::BEGIN) {
            return i;
          }
          let end = skip_group(v, j);
          if end > v.len() {
            return i;
          }
          j = end;
        }
        i = j;
      },
      Catcode::CS
        if i > start
          && (*t == T_CS!("\\fnmsep") || *t == T_CS!("\\fnsep"))
          && name_marks_end(v, i + 1) > i + 1 =>
      {
        i += 1;
      },
      _ => return i,
    }
  }
}

/// Whether a name begins at `v[i]`: a letter (not the word "and"), a group, a command that is no separator, mark or
/// note, or — when `mark_leads`, the names led by their marks — a mark. Not a comma, `;`, a line's end or a separator.
fn begins_a_name(v: &[Token], i: usize, mark_leads: bool) -> bool {
  let Some(t) = v.get(i) else {
    return false;
  };
  match t.get_catcode() {
    Catcode::LETTER => {
      let and = [T_LETTER!("a"), T_LETTER!("n"), T_LETTER!("d")];
      !(v[i..].starts_with(&and)
        && v
          .get(i + 3)
          .is_none_or(|u| u.get_catcode() != Catcode::LETTER))
    },
    Catcode::OTHER => t.with_str(|s| s.chars().next().is_some_and(char::is_alphabetic)),
    Catcode::BEGIN => true,
    Catcode::MATH | Catcode::SUPER => mark_leads && name_marks_end(v, i) > i,
    Catcode::CS => {
      if is_footnote_mark(t) || is_name_annotation(t) || is_mark_command(t) {
        return mark_leads;
      }
      !(is_line_separator(t)
        || *t == T_CS!("\\&")
        || *t == T_CS!("\\par")
        || *t == T_CS!("\\fnmsep")
        || *t == T_CS!("\\fnsep")
        || author_and_splits()
          .into_iter()
          .any(|d| matches!(d, SplitDelim::Token(d) if d == *t)))
    },
    _ => false,
  }
}

/// Whether `line`, which `delimiter` parts from the author `name` before it in a marked block, is more names on that
/// name's printed line: the delimiter a wide space (`\quad`, `\qquad`, the horizontal spaces OD #52(i) writes so), no
/// line break, and the name and the line read as two names or more ([`name_count`]: not `Ann Able$^{1}$ \quad Univ A`,
/// nor a qualifier, `Ann Able$^{1}$ \quad PhD`).
pub(in crate::base_utilities) fn space_parts_names(
  delimiter: &[Token],
  name: &Tokens,
  line: &Tokens,
) -> bool {
  let wide = delimiter
    .iter()
    .any(|t| *t == T_CS!("\\quad") || *t == T_CS!("\\qquad"));
  if !wide
    || delimiter.iter().any(|t| {
      author_line_breaks()
        .iter()
        .any(|d| matches!(d, SplitDelim::Token(d) if d == t))
    })
  {
    return false;
  }
  let text = format!(
    "{}, {}",
    visible_name_text(name.unlist_ref()).trim(),
    visible_name_text(line.unlist_ref()).trim()
  );
  name_count(&text) >= 2
}
