//! Author rows: line-break options, `\hspace` separators, rows of names, spacing runs.

use super::*;

/// Drop the optional `*` and `[<len>]` that follow a `\\` line-break token in an
/// author/affiliation block, before the block is split into lines. The
/// author/affiliation splitters key on the bare `\\` control sequence, so a
/// `\\[1em]` (or `\\*[1em]`) would otherwise orphan `[1em]` at the head of the
/// next segment — where it leaks as literal `[1em]` text AND, by pushing a
/// following `\textsuperscript` past the author-vs-affiliation position
/// threshold, flips a comma-bearing affiliation line into phantom author
/// creators. Perl's own `\lx@add@authors` flags this exact gap ("matching `\\`
/// this way fails to catch `\\[1em]`, so really should Let it") but never fixes
/// it, so both engines garble it identically; consuming it here is a beyond-Perl
/// improvement (KNOWN_PERL_ERRORS #75, witness arXiv:2605.23553). The `[...]` is
/// matched only as a balanced OTHER-catcode bracket group following the `\\` and
/// the spaces after it, which `\@ifnextchar` skips (`\\ [0.5em]`, 2604.20841), so
/// ordinary bracketed content elsewhere in a name/affiliation is untouched.
pub fn strip_linebreak_options(tokens: Tokens) -> Tokens {
  linebreak_options_stripped(tokens, false)
}

/// [`strip_linebreak_options`], an author block's positive `\\[len]` kept as the gap it prints: written `\\ \\`, the
/// spacer line pdflatex sets the same gap with (`\\ \vspace{len} \\`), which the unmarked branch reads as a gap before
/// the next line ([`name_groups`]: gr-qc9906006's `… Puerto Rico 00931-3343 \\[12pt] Palash B. Pal`; the 85a r1
/// review's finding 6). Only at the block's top level, outside braces and environments (a tabular's row is no line),
/// and not after a row of names, a line holding a wide space or a run of spacing at its top level (`Ivy Ives\;\;\; Jon Jay \\[0.2em] Lou Lee\;\;\; Max Moss \\[0.2em] Oli Ott`,
/// 2605.24414: rows going on with the names line, [`normalize_hspace_separators`]). The first line break of a group of
/// names, the one ending their names line, keeps its gap too, though no lines stand above it for the gap to part: a
/// gap there and before every later line is the block's spacing, parting no authors (`Ann Able \\[2pt] University of X
/// \\[2pt] Hong Kong`, the 85a r2 review's finding 3; [`name_groups`]). A negative or zero length (`\\[-0.1cm]`,
/// `\\[0pt]`) is no gap.
pub(in crate::base_utilities) fn strip_linebreak_options_keeping_gaps(tokens: Tokens) -> Tokens {
  linebreak_options_stripped(tokens, true)
}

/// Whether the tokens of a `\\[…]` option, between its brackets, are a positive length: not led by a minus sign, and
/// a register (`\bigskipamount`, `\baselineskip`) or a number with a non-zero digit (`12pt`, `.5cm`; not `0pt`).
fn positive_length(option: &[Token]) -> bool {
  let mut tokens = option.iter().filter(|t| **t != T_SPACE!());
  let Some(first) = tokens.next() else {
    return false;
  };
  *first != T_OTHER!("-")
    && std::iter::once(first).chain(tokens).any(|t| {
      t.get_catcode() == Catcode::CS || t.with_str(|s| s.chars().any(|c| ('1'..='9').contains(&c)))
    })
}

fn linebreak_options_stripped(tokens: Tokens, keep_gaps: bool) -> Tokens {
  let src = tokens.unlist();
  let n = src.len();
  let mut out: Vec<Token> = Vec::with_capacity(n);
  let brk = T_CS!("\\\\");
  // (the brace depth and the environment depth outside braces; whether the line since the last line break holds a wide
  // space or a run of spacing at its top level, a row)
  let (mut depth, mut environments) = (0usize, 0usize);
  let mut row = false;
  let mut i = 0;
  while i < n {
    out.push(src[i]);
    if depth == 0
      && (is_separating_space(&src[i])
        || (is_spacing_command(&src[i])
          && src[i + 1..]
            .iter()
            .find(|t| **t != T_SPACE!())
            .is_some_and(is_spacing_command)))
    {
      row = true;
    }
    match src[i].get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth = depth.saturating_sub(1),
      _ if depth == 0 && src[i] == T_CS!("\\begin") => environments += 1,
      _ if depth == 0 && src[i] == T_CS!("\\end") => environments = environments.saturating_sub(1),
      _ => {},
    }
    if src[i] == brk {
      let top = depth == 0 && environments == 0;
      let after_row = std::mem::take(&mut row);
      let mut j = i + 1;
      let mut consumed = false;
      // (the spaces before the optional argument skipped, as `\@ifnextchar` skips them: latex.ltx's `\@xnewline`,
      // `\@xarraycr`; `\\ [0.5em]`, 2604.20841, the r7 re-review's finding 8. Not before a star: `\\ * Equal
      // contribution` stays a legend line, though `\@ifstar` skips them too and pdflatex prints no star there —
      // 2608.23637, a SYNC_STATUS residual)
      let after_spaces = |from: usize| {
        from
          + src[from.min(n)..]
            .iter()
            .take_while(|t| **t == T_SPACE!())
            .count()
      };
      if j < n && src[j] == T_OTHER!("*") {
        j += 1;
        consumed = true;
      }
      if after_spaces(j) < n && src[after_spaces(j)] == T_OTHER!("[") {
        j = after_spaces(j);
        let mut k = j + 1;
        let mut depth = 1usize;
        while k < n && depth > 0 {
          if src[k] == T_OTHER!("[") {
            depth += 1;
          } else if src[k] == T_OTHER!("]") {
            depth -= 1;
          }
          k += 1;
        }
        if depth == 0 {
          if keep_gaps && top && !after_row && positive_length(&src[j + 1..k - 1]) {
            out.extend([T_SPACE!(), brk]);
          }
          j = k;
          consumed = true;
        }
      }
      if consumed {
        i = j;
        continue;
      }
    }
    i += 1;
  }
  Tokens::new(out)
}

/// Rewrite horizontal-space macros into `\quad`, so the author/affiliation
/// splitter treats them as separators. `\author{A \hspace{1cm} B \hspace{1cm} C}`
/// (and `\hfill`-separated variants) is a "regular" way to lay out co-authors
/// that LaTeXML's `\and`/`\quad` splitter otherwise misses, collapsing every name
/// into one `<personname>`. `\quad` is already a hard separator in every author /
/// affiliation split set, so rewriting to it needs no new delimiter plumbing.
/// `\hspace`'s optional `*` and mandatory `{len}` argument are consumed together
/// so the length cannot leak as literal text. OXIDIZED_DESIGN #52; witness
/// arXiv:2506.06941 (six authors separated by `\hspace{0.5cm}`).
///
/// A run of two or more spacing commands ([`is_spacing_command`]) between two names at the block's top level is such a
/// separator too (user ruling 2026-10-19, batch 77p): `\textbf{Ann Able}\textsuperscript{1}\;\;\textbf{Bob Baker}`
/// (2610.20409), `Ann Able$^{1}$ \ \ \ \ Bob Baker` (2310.10873), ACL's `Shanshan Wang$^1$~~~~ Derek F. Wong$^1$`
/// (2603.20851; `~~`, 2605.30274), 1702.05821, 1508.00093, 2009.12421 (OXIDIZED_DESIGN #52(o)). One spacing command
/// stays inside a name (`J.~Smith`, `Ann\ Able`, `van~der~Berg`); see [`spacing_run_separator`] for what counts as the
/// names on either side. A run or `\quad` inside a group parts nothing here: neurips_2024's 2406.09417 (one tie between
/// spaces) and 2603.20775 (`{\large \bf A \quad B}`), named with the ruling, are that shape.
pub(in crate::base_utilities) fn normalize_hspace_separators(tokens: Tokens) -> Tokens {
  /// A length written as zero, `0pt`, `0.0mm`, `0`, `\z@`: an `\hspace` of it is no separator.
  static ZERO_LENGTH: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    Regex::new(r"^\s*(?:[+-]?(?:0+(?:\.0*)?|\.0+)\s*(?:pt|mm|cm|in|em|ex|bp|sp|pc|dd|cc|mu)?|\\z@(?:skip)?)\s*$")
      .unwrap()
  });
  // (marked as the branch choice reads it, a footnote-symbol mark no mark: `\textbf{Vaibhav Singh}$^{*}$`, 2603.09835)
  let marked_block = has_author_marks(&rewrite_symbol_superscripts(tokens.clone()));
  let src = spacing_math_unwrapped(tokens.unlist());
  let n = src.len();
  let mut out: Vec<Token> = Vec::with_capacity(n);
  // (the group depth and math mode at `src[i]`: a run is a separator only at the top level, outside math; whether a
  // name letter — outside math, outside a mark command's argument, at the top level or inside a group that shows its
  // letters as name text ([`shows_name_text`]) — came since the last top-level separator, so a run after a
  // line-leading mark is the mark's space, `$\ast$~~Department of Physics`, hep-ph9708397, and one after an icon is
  // the icon's, `\raisebox{-0.3ex}{\includegraphics[height=1.1em]{daedalus.jpg}}\;\;Daedalus Research Team`,
  // 2604.10005; the depth of the mark argument being read)
  let mut depth = 0usize;
  let mut in_math = false;
  let mut named = false;
  let mut mark_argument: Option<usize> = None;
  // (for each open group, whether its letters are name text; and the depth of an optional argument being read, whose
  // letters are not, `\includegraphics[height=0.7em]`)
  let mut shown: Vec<bool> = Vec::new();
  let mut closed_shown = true;
  let mut optional_argument = false;
  // (whether the line opens with a mark — an affiliation line, `$^1$The University of Hong Kong~~~~~~$^2$Zhejiang Lab`,
  // 2310.15160 —, where a run before the next mark parts two entries; `None` before its first token)
  let mut mark_led: Option<bool> = None;
  // (whether this is a line of names — the block's first, or the first after an `\and`; a line break ends it — and
  // where in `out` the piece since the last top-level separator starts)
  let mut names_line = true;
  let mut piece_start = 0usize;
  // (in an unmarked block, the line break that began this line, where in `out` it stands, after a line of names: a
  // run that parts two names on this line makes it a row of names going on with the one before, the break a `\quad`
  // — `\textbf{Pratham Grover}\\ \textbf{Ramakrishna B.~Bairi}\ \ \textbf{Ritabrata Maiti}`, 2603.09835, where
  // "Ramakrishna B. Bairi" had read as an address line of Pratham Grover; a marked block reads its rows by their marks)
  let mut row_break: Option<usize> = None;
  // (where in `out` the names line starts, and where in `src` the line after it does: a row of names goes on with the
  // names line only when it lists another number of names than it, or a line that is no row of as many entries
  // follows it — [`row_of_names`])
  let mut names_start = 0usize;
  let mut row_start = 0usize;
  // (where in `src` the last top-level mark or note ended, for an `\enskip` right after it)
  let mut mark_end: Option<usize> = None;
  let mut i = 0;
  while i < n {
    let t = src[i];
    if depth == 0 && !in_math && mark_end.is_none_or(|at| at <= i) {
      let marks = leading_marks_end(&src[i..]);
      if marks > 0 {
        mark_end = Some(i + marks);
      }
    }
    if mark_led.is_none() && t.get_catcode() != Catcode::SPACE && t != T_CS!("\\par") {
      mark_led = Some(starts_with_mark(after_leading_markup(&src[i..])));
    }
    // (the horizontal space a name sets before its own mark or note is the name's, no separator: `Oleg
    // Lychkovskiy\thanks{…}\hspace*{2mm}$^{\rm a,b}$, Sergei Blinnikov\hspace*{2mm}$^{\rm a,c}$`, 0905.3658, where the
    // `\quad` it made parted ", Sergei Blinnikov" off as an affiliation line; `\hspace*{1.3mm}$^{,\ddag}$`, 1312.1414)
    if depth == 0
      && !in_math
      && named
      && let Some(end) = space_before_own_mark(&src, i)
    {
      i = end;
      continue;
    }
    match t.get_catcode() {
      Catcode::BEGIN => {
        depth += 1;
        if mark_argument.is_none() && out.last().is_some_and(is_mark_command) {
          mark_argument = Some(depth);
        }
        let outer = shown.last().copied().unwrap_or(true);
        shown.push(outer && !optional_argument && shows_name_text(out.last(), closed_shown));
      },
      Catcode::END => {
        if mark_argument == Some(depth) {
          mark_argument = None;
        }
        depth = depth.saturating_sub(1);
        closed_shown = shown.pop().unwrap_or(true);
      },
      Catcode::MATH => in_math = !in_math,
      Catcode::LETTER
        if !in_math
          && mark_argument.is_none()
          && !optional_argument
          && shown.last().copied().unwrap_or(true) =>
      {
        named = true
      },
      Catcode::OTHER
        if t == T_OTHER!("[") && out.last().is_some_and(|p| p.get_catcode() == Catcode::CS) =>
      {
        optional_argument = true
      },
      Catcode::OTHER if t == T_OTHER!("]") => optional_argument = false,
      _ if depth == 0 && is_line_separator(&t) => {
        (named, mark_led) = (false, None);
        piece_start = out.len() + 1;
        if t == T_CS!("\\\\") || t == T_CS!("\\newline") {
          row_break = (names_line && !marked_block).then_some(out.len());
          row_start = i + 1;
          names_line = false;
        } else if t == T_CS!("\\and") || t == T_CS!("\\And") || t == T_CS!("\\AND") {
          (names_line, row_break) = (true, None);
          names_start = out.len() + 1;
        }
      },
      _ => {},
    }
    // (a line opens with a mark only in a marked block, a mark being what the branch choice reads as one: in a block
    // marked only with footnote symbols — no mark there, [`rewrite_symbol_superscripts`] — a symbol-led line is an
    // affiliation line read as any other line off the names line, `$^{*}$\daplablogo\ Columbia University\ \ \
    // $^{\dagger}$University at Buffalo` (2609.35807; was a creator "†University at Buffalo", 77p r4 re-review))
    let entry_mark_led = marked_block && mark_led == Some(true);
    if depth == 0
      && !in_math
      && named
      && is_spacing_command(&t)
      && let Some(run) = spacing_run_separator(&src, &out, i, RunContext {
        mark_led: entry_mark_led,
        names_line,
        piece_start,
      })
      // (in an unmarked block, off the names line, a run between two names parts them only on a row of names going on
      // with the names line — the line right after it, [`row_of_names`]: not institutions under the names, `Ann
      // Able~~~~Bob Baker \\ Carnegie Mellon~~~~Johns Hopkins`, nor names under an affiliation, `Ann Able \\ Univ A
      // \\ Bob Baker~~~~Cat Cole`, which read as the affiliation lines they stand as, nor on a line a footnote symbol
      // leads, `$^{*}$Univ A~~~~$^{\dagger}$Univ B` (2609.35807); a marked block reads its lines by their marks)
      && (names_line
        || marked_block
        || row_break
          .is_some_and(|at| row_of_names(&out[names_start.min(at)..at], &src[row_start..])))
    {
      if !names_line
        && !entry_mark_led
        && let Some(at) = row_break.take()
      {
        out[at] = T_CS!("\\quad");
        names_line = true;
      }
      if run.and_word {
        out.extend([
          T_SPACE!(),
          T_LETTER!("a"),
          T_LETTER!("n"),
          T_LETTER!("d"),
          T_SPACE!(),
        ]);
      } else {
        out.push(T_CS!("\\quad"));
      }
      (named, mark_led) = (false, None);
      piece_start = out.len();
      i = run.end;
      continue;
    }
    if t == T_CS!("\\hspace") {
      let mut j = i + 1;
      // optional `*` (\hspace*)
      if j < n && src[j] == T_OTHER!("*") {
        j += 1;
      }
      // mandatory `{len}` group
      let length_start = j + 1;
      if j < n && src[j] == T_BEGIN!() {
        let mut depth = 1usize;
        j += 1;
        while j < n && depth > 0 {
          if src[j] == T_BEGIN!() {
            depth += 1;
          } else if src[j] == T_END!() {
            depth -= 1;
          }
          j += 1;
        }
      }
      // (a zero length inside a group, a mark's argument or math prints no gap and parts nothing:
      // `\textsuperscript{1\hspace{0pt}2}` prints the label 12, as `\hskip0pt` is no spacing,
      // `util_text::is_blank_item`; at the top level of the author line it stays the `\quad` that parts two names,
      // `Ann Able\hspace{0pt}Bob Baker` two creators)
      let zero = (depth > 0 || in_math || mark_argument.is_some())
        && length_start < j
        && ZERO_LENGTH.is_match(&Tokens::new(src[length_start..j - 1].to_vec()).to_string());
      if !zero {
        out.push(T_CS!("\\quad"));
      }
      i = j;
      continue;
    }
    if t == T_CS!("\\hfill") || t == T_CS!("\\hfil") {
      out.push(T_CS!("\\quad"));
      i += 1;
      continue;
    }
    // (latex.ltx's `\enskip`, `\hskip.5em\relax`, is such a space too, at the top level right after a mark or note:
    // `\normalsize Dominic W.\ Berry\thanks{…} \enskip \normalsize Andrew M.\ Childs\thanks{…}…`, 1312.1414, five
    // names read as one; elsewhere it is a spacing command, one of which stays inside a name, `J.\enskip Smith$^{1}$`)
    if t == T_CS!("\\enskip")
      && depth == 0
      && !in_math
      && mark_end.is_some_and(|at| src[at..i].iter().all(|u| u.get_catcode() == Catcode::SPACE))
    {
      out.push(T_CS!("\\quad"));
      i += 1;
      continue;
    }
    out.push(t);
    i += 1;
  }
  Tokens::new(out)
}

/// The horizontal spaces an author line parts its entries with ([`normalize_hspace_separators`]), which are separators
/// at its top level: `\hspace` (its `*` and length read with it), `\hfill`, `\hfil`, `\quad`, `\qquad`. (Not `\enskip`,
/// a separator only after a mark or note, else a spacing command, [`is_spacing_command`].)
fn is_separating_space(t: &Token) -> bool {
  t.get_catcode() == Catcode::CS
    && t.with_str(|s| matches!(s, "\\hspace" | "\\hfill" | "\\hfil" | "\\quad" | "\\qquad"))
}

/// The run of horizontal space that opens `tokens` — spaces, separating spaces ([`is_separating_space`], `\hspace`'s `*`
/// and length read with it) and spacing commands ([`is_spacing_command`]) —: where it ends, how many spacing commands
/// it holds, and whether it holds a separating space.
fn horizontal_space_run(tokens: &[Token]) -> (usize, usize, bool) {
  let (mut end, mut commands, mut separating) = (0, 0, false);
  while let Some(t) = tokens.get(end) {
    if *t == T_CS!("\\hspace") {
      separating = true;
      end += 1;
      if tokens.get(end) == Some(&T_OTHER!("*")) {
        end += 1;
      }
      end = skip_group(tokens, end);
    } else if is_separating_space(t) {
      separating = true;
      end += 1;
    } else if is_spacing_command(t) {
      commands += 1;
      end += 1;
    } else if t.get_catcode() == Catcode::SPACE {
      end += 1;
    } else {
      break;
    }
  }
  (end, commands, separating)
}

/// Where a run of horizontal space that starts at `src[start]` with a separating space ([`is_separating_space`]) ends,
/// when what follows it is a mark or a note of the name before it ([`leading_marks_end`]: `$^{a,b}$`,
/// `\textsuperscript{1}`, `\thanks{…}`) that no entry text follows — a comma, `;`, the word "and", a separator, a line
/// break, a separating run of spacing (a separating space, or two or more spacing commands, OXIDIZED_DESIGN #52(o)) or
/// the end come after it —, so the space is the name's: `Oleg L.\hspace*{2mm}$^{\rm a,b}$, Sergei B.` (0905.3658).
/// `None` before a mark that leads an entry (`$^1$Univ A\quad$^2$Univ B`, an affiliation line's two entries; with one
/// tie or control space after the mark, the entry's own, `$^1$~Univ A\quad$^2$~Univ B` and `\textsuperscript{2}\ Univ
/// B`, whose two universities read as one) and before anything else.
fn space_before_own_mark(src: &[Token], start: usize) -> Option<usize> {
  if !src.get(start).is_some_and(is_separating_space) {
    return None;
  }
  let end = start + horizontal_space_run(&src[start..]).0;
  let marks = leading_marks_end(&src[end..]);
  if marks == 0 {
    return None;
  }
  let rest = &src[end + marks..];
  // (the spacing after the marks: a separating run ends the entry there — an `\enskip` too, which right after a mark or
  // note is a separator, `Childs\thanks{…}\hspace*{1.1mm}$^{,}$\thanks{…} \enskip \normalsize Richard Cleve`,
  // 1312.1414 —; one other spacing command, and the spaces around it, is the next entry's, which the token after it
  // tells)
  let (after, commands, separating) = horizontal_space_run(rest);
  if separating || commands >= 2 || rest[..after].contains(&T_CS!("\\enskip")) {
    return Some(end);
  }
  let rest = &rest[after..];
  let own = match rest.first() {
    None => true,
    Some(t) if *t == T_OTHER!(",") || *t == T_OTHER!(";") || *t == T_END!() => true,
    Some(t) if is_line_separator(t) || *t == T_CS!("\\par") => true,
    Some(_) => and_word_end(rest, false).is_some(),
  };
  own.then_some(end)
}

/// `tokens` with each math group of its top level that holds only spacing — `$\qquad$` — written as that spacing,
/// outside math, which the separators read as written so (`Michael J. Wilber$^{1,2}$ $\qquad$ Vitaly
/// Shmatikov$^{1,2}$ $\qquad$ Serge Belongie$^{1,2}$`, 1602.04504, whose three names read as one); and so each brace
/// group of its top level that holds only spacing with a `\quad` or `\qquad` in it, no command's argument (`Byeongjun
/// Park$^{1}${\quad} Byung-Hoon Kim$^{1,2,*}${\quad} …\\ $^{1}$EverEx{\quad}$^{2}$Yonsei University`, 2609.03563:
/// one creator, its institutions one; a glue's group changes nothing, OXIDIZED_DESIGN_DIVERGENCES #897).
fn spacing_math_unwrapped(tokens: Vec<Token>) -> Vec<Token> {
  let spacing = |t: &Token| {
    t.get_catcode() == Catcode::SPACE
      || is_spacing_command(t)
      || (t.get_catcode() == Catcode::CS && t.with_str(|s| matches!(s, "\\quad" | "\\qquad")))
  };
  let mut out = Vec::with_capacity(tokens.len());
  let mut depth = 0usize;
  let mut in_math = false;
  let mut i = 0;
  while i < tokens.len() {
    let t = tokens[i];
    match t.get_catcode() {
      Catcode::BEGIN
        if depth == 0
          && !in_math
          && !out.last().is_some_and(|p: &Token| {
            p.get_catcode() == Catcode::CS || *p == T_OTHER!("*") || *p == T_OTHER!("]")
          }) =>
      {
        let end = skip_group(&tokens, i);
        let inner = &tokens[i + 1..end.saturating_sub(1).max(i + 1)];
        if end <= tokens.len()
          && tokens[end - 1].get_catcode() == Catcode::END
          && !inner.is_empty()
          && inner.iter().all(spacing)
          && inner
            .iter()
            .any(|u| u.with_str(|s| matches!(s, "\\quad" | "\\qquad")))
        {
          out.extend(
            inner
              .iter()
              .copied()
              .filter(|u| u.get_catcode() != Catcode::SPACE),
          );
          i = end;
          continue;
        }
        depth += 1;
      },
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth = depth.saturating_sub(1),
      Catcode::MATH if depth == 0 && !in_math => {
        let close = tokens[i + 1..]
          .iter()
          .position(|u| u.get_catcode() == Catcode::MATH);
        if let Some(close) = close {
          let inner = &tokens[i + 1..i + 1 + close];
          if inner.iter().all(spacing) && inner.iter().any(|u| u.get_catcode() != Catcode::SPACE) {
            out.extend(
              inner
                .iter()
                .copied()
                .filter(|u| u.get_catcode() != Catcode::SPACE),
            );
            i += close + 2;
            continue;
          }
        }
        in_math = true;
      },
      Catcode::MATH => in_math = !in_math,
      _ => {},
    }
    out.push(t);
    i += 1;
  }
  out
}

/// Whether the line after a line of names `names` (as rewritten so far, the rows it already goes on with included) —
/// `rest`, from just after the line break —, which a spacing run parts into names, is a row of more names going on with
/// them rather than the institutions under them, one under each name (`Ann Able~~~~Bob Baker \\ Carnegie
/// Mellon~~~~Johns Hopkins`, a column per author; `Ann Able \\ Carnegie Mellon~~~~Johns Hopkins`). The names line holds
/// two or more names, and the row lists another number of names (2603.09835, 2607.07318, 2603.07624, 2603.16856: rows
/// of five over three, two over four), or a line follows it that is no row of as many entries — the address under all
/// the rows (2605.24414's six names over six over three, then `\textit{JIUTIAN Research}`; 2608.21409's three over
/// three over its e-mail line; 2607.09029's five over five over `LiAuto Inc.`). A row of institutions under the names
/// ends the block, or has a row of as many entries under it (their e-mails). Markup does not tell the two apart:
/// 2607.07318's and 2603.16856's second rows open with `{\bf`/`\bf` under plain first rows.
///
/// A line of addresses under the row says whose they are ([`row_spelled_by_addresses`]): the row is names only when an
/// address spells one of its pieces, or the addresses are as many as the names of both rows together — not institutions
/// over the authors' addresses, `Ann Able~~~~Bob Baker \\ Carnegie Mellon~~~~Johns Hopkins \\
/// \texttt{\{aa,bb\}@x.edu}`, nor three over three over one address (probes v1, v3: were creators "Carnegie Mellon",
/// "Johns Hopkins", 77p r4 re-review).
fn row_of_names(names: &[Token], rest: &[Token]) -> bool {
  let names_count = row_name_count(names);
  if names_count < 2 {
    return false;
  }
  let rest = after_line_break_options(rest);
  let mut at = rest.iter().position(is_row_break).unwrap_or(rest.len());
  let row = &rest[..at];
  let row_count = row_name_count(row);
  // (the next line that prints something, past the lines of spacing alone: `\vspace{2mm}\\`)
  let mut under: Option<&[Token]> = None;
  while rest
    .get(at)
    .is_some_and(|t| *t == T_CS!("\\\\") || *t == T_CS!("\\newline"))
  {
    let start = rest.len() - after_line_break_options(&rest[at + 1..]).len();
    let end = start
      + rest[start..]
        .iter()
        .position(is_row_break)
        .unwrap_or(rest.len() - start);
    let line = &rest[start..end];
    if spacing_end(line) > 0 {
      under = Some(line);
      break;
    }
    at = end;
  }
  if let Some(addresses) = under.and_then(|line| email_addresses(&Tokens::new(line.to_vec()))) {
    return addresses.len() == names_count + row_count
      || row_spelled_by_addresses(names, row, &addresses);
  }
  if row_count != names_count {
    return true;
  }
  under.is_some_and(|line| row_entry_count(line) != names_count)
}

/// Whether one of `addresses` spells a piece of `row` ([`spelled_owners`], the names line's pieces `names` spelled
/// against too, so an address spells the row's piece only when it spells it best).
fn row_spelled_by_addresses(names: &[Token], row: &[Token], addresses: &[String]) -> bool {
  let pieces = |tokens: &[Token]| -> Vec<NameKeys> {
    let mut keys = Vec::new();
    let mut i = 0;
    while i < tokens.len() && !is_row_break(&tokens[i]) {
      let piece = next_name_piece(&tokens[i..]);
      if piece.is_empty() {
        i += 1;
        continue;
      }
      if has_name_letter(piece) {
        keys.push(NameKeys::of(&Tokens::new(outside_notes(piece))));
      }
      i += piece.len();
    }
    keys
  };
  let mut keys = pieces(names);
  let row_from = keys.len();
  keys.extend(pieces(row));
  addresses.iter().any(|address| {
    spelled_owners(&keys, address)
      .iter()
      .any(|owner| *owner >= row_from)
  })
}

/// `tokens` after the star and the optional `[…]` argument of the line break before them, and the spaces before each
/// (`\\[0.2em]`, `\\*`).
fn after_line_break_options(tokens: &[Token]) -> &[Token] {
  let mut i = tokens
    .iter()
    .take_while(|t| t.get_catcode() == Catcode::SPACE)
    .count();
  if tokens.get(i) == Some(&T_OTHER!("*")) {
    i += 1;
  }
  if tokens.get(i) == Some(&T_OTHER!("["))
    && let Some(close) = tokens[i..].iter().position(|t| *t == T_OTHER!("]"))
  {
    i += close + 1;
  }
  &tokens[i..]
}

/// How many entries an author line lists up to its first line break: the pieces between its separators
/// ([`next_name_piece`]) that print text (`\texttt{a@x}~~~~\texttt{b@y}` two).
fn row_entry_count(tokens: &[Token]) -> usize {
  let mut count = 0;
  let mut i = 0;
  while i < tokens.len() && !is_row_break(&tokens[i]) {
    let piece = next_name_piece(&tokens[i..]);
    if piece.is_empty() {
      i += 1;
      continue;
    }
    if !visible_name_text(piece).trim().is_empty() {
      count += 1;
    }
    i += piece.len();
  }
  count
}

/// A token that ends an author line's row of names: a line break, or the `\and` family.
fn is_row_break(t: &Token) -> bool {
  t.with_str(|s| matches!(s, "\\\\" | "\\newline" | "\\and" | "\\And" | "\\AND"))
}

/// How many names an author line lists up to its first line break ([`is_row_break`]): each piece between its
/// separators ([`next_name_piece`]) with a name letter counts the names its text lists ([`name_count`], "Ann Able and
/// Bob Baker" two), one at least.
fn row_name_count(tokens: &[Token]) -> usize {
  let mut count = 0;
  let mut i = 0;
  while i < tokens.len() && !is_row_break(&tokens[i]) {
    let piece = next_name_piece(&tokens[i..]);
    if piece.is_empty() {
      i += 1;
      continue;
    }
    if has_name_letter(piece) {
      count += name_count(visible_name_text(&outside_notes(piece)).trim()).max(1);
    }
    i += piece.len();
  }
  count
}

/// The spacing commands a run of which parts two author names (user ruling 2026-10-19): the control space `\ ` — also
/// as the control return a line ending in `\ ` leaves, TeX dropping the line's trailing space (latex.ltx's `\^^M` is
/// `\ `; cs0207008's `K.V. Hindriks \affila\ \ ⏎ W. van der Hoek`) —, the medium and thick spaces `\:`, `\>`, `\;`,
/// the tie `~`, `\enspace` and `\enskip` (latex.ltx `\hskip.5em\relax`, a separator by itself only after a mark or note,
/// [`normalize_hspace_separators`]: `A.\enskip B. Smith` is one name).
pub(in crate::base_utilities) fn is_spacing_command(t: &Token) -> bool {
  *t == T_ACTIVE!('~')
    || (t.get_catcode() == Catcode::CS
      && t.with_str(|s| {
        matches!(
          s,
          "\\ " | "\\\r" | "\\:" | "\\>" | "\\;" | "\\enspace" | "\\enskip"
        )
      }))
}

/// `tokens` after the markup a line or an entry may open with before its mark: spaces, group openings and the font and
/// size commands of [`NAME_TEXT_COMMANDS`] (`\normalsize{$^1$I3A - Universidad de Zaragoza} ~~~~~~
/// \normalsize{$^2$École Polytechnique Fédérale de Lausanne}`, 2308.10525).
pub(in crate::base_utilities) fn after_leading_markup(tokens: &[Token]) -> &[Token] {
  let lead = tokens
    .iter()
    .take_while(|t| {
      matches!(t.get_catcode(), Catcode::SPACE | Catcode::BEGIN)
        || (t.get_catcode() == Catcode::CS && t.with_str(|s| NAME_TEXT_COMMANDS.contains(&s)))
    })
    .count();
  &tokens[lead..]
}

/// Whether `tokens` open with an author mark: `$^…$` or `\(^…\)` (a math shift before a superscript), a superscript, a
/// mark command ([`is_author_mark`]).
fn starts_with_mark(tokens: &[Token]) -> bool {
  match tokens {
    [t, ..] if is_author_mark(t) => true,
    // (`$^1$`, and `\(^{1}\)`: 2604.12896's `\(^{1}\)Huawei Technologies, Canada~~\(^2\)University of Alberta`)
    [open, sup, ..] => {
      (open.get_catcode() == Catcode::MATH || *open == T_CS!("\\(")) && *sup == T_SUPER!()
    },
    _ => false,
  }
}

/// A token that ends an author line or a name, before any spacing run reads it so: the `\and` family, `\quad`,
/// `\qquad`, a line break, the horizontal spaces [`normalize_hspace_separators`] makes `\quad`. Not `\par`: an author
/// block is set in a `c` column (latex.ltx's `\@maketitle`, `\begin{tabular}[t]{c}\@author\end{tabular}`; ACL's the
/// same), restricted horizontal mode, where a blank line's `\par` ends no paragraph and prints nothing (tex.web §1094
/// `hmode+par_end` calls `end_graf`, §1096 acts only when `mode=hmode`): 2009.12421's `Ehsan
/// Shareghi$^\spadesuit$$^\clubsuit$ <blank line> \ \ Nigel Collier$^\clubsuit$` prints the two names on one row.
pub(in crate::base_utilities) fn is_line_separator(t: &Token) -> bool {
  t.get_catcode() == Catcode::CS
    && (author_affil_splits()
      .iter()
      .chain(author_and_splits().iter())
      .chain(author_line_breaks().iter())
      .any(|d| matches!(d, SplitDelim::Token(d) if d == t))
      || t.with_str(|s| matches!(s, "\\qquad" | "\\hfill" | "\\hfil" | "\\hspace")))
}

/// Whether a group opened after `before` shows its letters as name text, for a spacing run to part names after them: a
/// group of its own (`{\scshape Vitaly~Bergelson}`, also after a separator or a spacing command: `~~ {Kyungtae Lim}`
/// once the run before it is `\quad`, 2605.30545) or a font command's argument (`\textbf{Ann Able}`); not another
/// command's argument (`\raisebox{-0.3ex}{\includegraphics[…]{daedalus.jpg}}`, 2604.10005; `\mbox{\orcidlink{…}}`,
/// 2605.06250; `\url{…}`, 2605.28131). A group right after another (`}{`) is shown as that one was (`closed_shown`):
/// the next argument of the same command.
fn shows_name_text(before: Option<&Token>, closed_shown: bool) -> bool {
  match before {
    Some(t) if t.get_catcode() == Catcode::CS => {
      t.with_str(|s| {
        NAME_TEXT_COMMANDS.contains(&s) || matches!(s, "\\par" | "\\relax" | "\\noindent")
      }) || is_line_separator(t)
        || is_spacing_command(t)
    },
    Some(t) if *t == T_END!() || *t == T_OTHER!("]") => closed_shown,
    _ => true,
  }
}

/// The font and size commands an author line styles a name or an entry with: their argument, or the group after the
/// declaration, prints as the name it styles ([`shows_name_text`]), and an entry may open with them before its mark
/// ([`after_leading_markup`]).
pub(in crate::base_utilities) const NAME_TEXT_COMMANDS: &[&str] = &[
  "\\footnotesize",
  "\\scriptsize",
  "\\tiny",
  "\\LARGE",
  "\\huge",
  "\\Huge",
  "\\rm",
  "\\sf",
  "\\tt",
  "\\em",
  "\\sl",
  "\\upshape",
  "\\itshape",
  "\\slshape",
  "\\scshape",
  "\\bfseries",
  "\\mdseries",
  "\\rmfamily",
  "\\sffamily",
  "\\ttfamily",
  "\\normalfont",
  "\\textbf",
  "\\textit",
  "\\textsc",
  "\\textsl",
  "\\textup",
  "\\textmd",
  "\\textrm",
  "\\textsf",
  "\\textnormal",
  "\\emph",
  "\\bf",
  "\\it",
  "\\sc",
  "\\large",
  "\\Large",
  "\\normalsize",
  "\\small",
];

/// What a spacing run's place on its author block tells [`spacing_run_separator`].
struct RunContext {
  /// the line of a marked block opens with a mark (an affiliation line, `$^1$Univ A~~~~$^2$Univ B`, 2310.15160)
  mark_led:    bool,
  /// the line is a line of names: the block's first, or the first after an `\and` (a line break ends it)
  names_line:  bool,
  /// where in the tokens before the run the piece since the last top-level separator starts
  piece_start: usize,
}

/// A spacing run that parts two names ([`spacing_run_separator`]): where it ends, and whether the word "and" stands in
/// it (`~~AND~~`, `~~{and}~~`), which then is the separator, written as the plain word between spaces that every author
/// split reads (`Cat Cole~~\textrm{and}~~Dan Dorn` → `Cat Cole and Dan Dorn`; a marked block's line welds a piece a
/// `\quad` parts off into the entry before it).
struct RunSeparator {
  end:      usize,
  and_word: bool,
}

/// Where a run of spacing commands that starts at `src[start]` ends, when it parts two names: two or more spacing
/// commands ([`is_spacing_command`], spaces and a blank line's `\par` between them), after a name — `out` holds what
/// precedes it, a name letter is on its line (the caller's test), and it does not end with a separator already (`\\`,
/// `\and`, `\quad`, a comma or `;`), nor with an initial (`R.~~Beyer`, 2603.17646; `L.~~Caballero-Ontanaya`,
/// 2604.25437; `Rafa\l ~R.\ ~Suszek`, 2603.20914: one person) — and before the next one, which starts with a letter, a
/// group or a command that is no name annotation (a run before `\thanks{…}`, `\textsuperscript{1}` or a `$^1$` mark
/// leaves the mark on its name) and has a name letter before the next separator (not `~~\JMorcid,`, 2605.12398;
/// `~~\mbox{\orcidlink{…}}`, 2605.06250); or, on a line that opens with a mark (`ctx.mark_led`), with the next mark.
/// Off the names line (`ctx.names_line`: after a line break, before the next `\and`) both sides must read as names, not
/// as an address, an e-mail or a link (`Ann Able \\ Department of Physics~~~~University of X`; `\url{…}~~~ \url{…}`,
/// 2605.28131). A run before the word "and" — in any case, alone or in one wrapper ([`and_word_end`]: `~~AND~~`,
/// `~~{and}~~`, `\;\;\textnormal{AND}\;\;`, 2606.27261, 2607.03677) — parts with it, the word the separator
/// ([`RunSeparator`]). `None` for a single spacing command, inside one name, and for a run after the word "and", or
/// after a byline's "By" (`By~~{\scshape Vitaly~Bergelson}~~and~~…`, 2603.25922, 2604.12864, 2603.27258, 2604.07186),
/// which separate the names themselves.
fn spacing_run_separator(
  src: &[Token],
  out: &[Token],
  start: usize,
  ctx: RunContext,
) -> Option<RunSeparator> {
  let mut end = start;
  let mut commands = 0;
  while let Some(t) = src.get(end) {
    if is_spacing_command(t) {
      commands += 1;
    } else if t.get_catcode() != Catcode::SPACE && *t != T_CS!("\\par") {
      break;
    }
    end += 1;
  }
  if commands < 2 {
    return None;
  }
  let unprinted = |t: &&Token| t.get_catcode() == Catcode::SPACE || **t == T_CS!("\\par");
  let mut before_run = out.iter().rev().skip_while(unprinted);
  let before = *before_run.next()?;
  if is_line_separator(&before)
    || before == T_CS!("\\&")
    || matches!(before.get_catcode(), Catcode::OTHER if before.with_str(|s| s == "," || s == ";"))
  {
    return None;
  }
  // (an initial: one letter and its period, after no letter)
  if before == T_OTHER!(".")
    && before_run
      .next()
      .is_some_and(|t| t.get_catcode() == Catcode::LETTER)
    && before_run
      .next()
      .is_none_or(|t| t.get_catcode() != Catcode::LETTER)
  {
    return None;
  }
  // (a run after the word "and" — in any case, alone or in one wrapper — leaves the separating to it: `Cat
  // Cole and~~Dan Dorn`; so does one after a byline's "by")
  let word_before: String = word_letters(out.iter().rev()).chars().rev().collect();
  if and_word_end(&out.iter().rev().copied().collect::<Vec<_>>(), true).is_some()
    || word_before.eq_ignore_ascii_case("by")
  {
    return None;
  }
  // (a run before the word "and" — `~~and~~`, `~~AND~~`, `~~{and}~~`, `\;\;\textnormal{AND}\;\;`, 2606.27261,
  // 2607.03677 — is one separator with it and the space after it: what follows must open a name as after a run)
  let (end, and_word) = match and_word_end(&src[end..], false) {
    Some(word_end) => (
      end
        + word_end
        + src[end + word_end..]
          .iter()
          .take_while(|t| {
            t.get_catcode() == Catcode::SPACE || is_spacing_command(t) || **t == T_CS!("\\par")
          })
          .count(),
      true,
    ),
    None => (end, false),
  };
  let after = src.get(end)?;
  if *after == T_CS!("\\&") {
    return None;
  }
  // (on a mark-led line, a run before the next mark parts its entries: `$^1$Univ A~~~~$^2$Univ B`; the word "and" in
  // it too, off the names line as the `\quad` that parts affiliation entries, `$^1$Univ A~~and~~$^2$Univ B` (probe
  // w10, was "Univ A and" an entry))
  if ctx.mark_led && starts_with_mark(after_leading_markup(&src[end..])) {
    return Some(RunSeparator {
      end,
      and_word: and_word && ctx.names_line,
    });
  }
  let starts_name = match after.get_catcode() {
    Catcode::LETTER | Catcode::BEGIN => true,
    Catcode::OTHER => after.with_str(|s| s.chars().next().is_some_and(char::is_alphabetic)),
    Catcode::CS => {
      !is_line_separator(after)
        && !is_name_annotation(after)
        && !is_footnote_mark(after)
        && !is_mark_command(after)
    },
    _ => false,
  };
  if !starts_name {
    return None;
  }
  let next_piece = next_name_piece(&src[end..]);
  if !has_name_letter(next_piece) {
    return None;
  }
  if !ctx.names_line
    && !(reads_as_name_piece(&out[ctx.piece_start.min(out.len())..])
      && reads_as_name_piece(next_piece))
  {
    return None;
  }
  Some(RunSeparator { end, and_word })
}

/// Whether `tokens` print only the word "and", in any case and wrapper (`and`, `\textnormal{AND}`, `{and}`): the
/// separator between two names, never a name or an affiliation.
pub(in crate::base_utilities) fn is_and_piece(tokens: &[Token]) -> bool {
  visible_name_text(tokens).trim().eq_ignore_ascii_case("and")
}

/// The letters of the word `tokens` open with, after their spaces.
fn word_letters<'a>(tokens: impl Iterator<Item = &'a Token>) -> String {
  tokens
    .skip_while(|t| t.get_catcode() == Catcode::SPACE)
    .take_while(|t| t.get_catcode() == Catcode::LETTER)
    .map(|t| t.with_str(str::to_string))
    .collect()
}

/// Where the word "and" that `tokens` open with ends (after their spaces), in any case (`and`, `And`, `AND`), written
/// alone or as the only word of one wrapper — a group, or a font command's argument (`{and}`, `\textrm{and}`,
/// `\textnormal{AND}`: 2606.27261, 2607.03677, 2608.24427). `reversed`: `tokens` run backwards (the tokens before a
/// run), the wrapper's closing brace first. `None` when they open with anything else (`Andrew`, `Anders`).
fn and_word_end(tokens: &[Token], reversed: bool) -> Option<usize> {
  let (open, close) = if reversed {
    (T_END!(), T_BEGIN!())
  } else {
    (T_BEGIN!(), T_END!())
  };
  let mut i = tokens
    .iter()
    .take_while(|t| t.get_catcode() == Catcode::SPACE)
    .count();
  // (a font command before its argument: read forwards, it opens the wrapper; read backwards, it stands after it)
  if !reversed
    && tokens.get(i).is_some_and(|t| {
      t.get_catcode() == Catcode::CS && t.with_str(|s| NAME_TEXT_COMMANDS.contains(&s))
    })
    && tokens.get(i + 1) == Some(&open)
  {
    i += 1;
  }
  let wrapped = tokens.get(i) == Some(&open);
  if wrapped {
    i += 1;
  }
  let word_start = i
    + tokens[i..]
      .iter()
      .take_while(|t| wrapped && t.get_catcode() == Catcode::SPACE)
      .count();
  let word: String = word_letters(tokens[word_start..].iter());
  let word: String = if reversed {
    word.chars().rev().collect()
  } else {
    word
  };
  if !word.eq_ignore_ascii_case("and") {
    return None;
  }
  i = word_start + 3;
  if wrapped {
    i += tokens[i..]
      .iter()
      .take_while(|t| t.get_catcode() == Catcode::SPACE)
      .count();
    if tokens.get(i) != Some(&close) {
      return None;
    }
    i += 1;
  }
  // (a letter or a letter command after it is a longer word's: `and\o`; the closing word of a reversed read must not
  // be glued to one before it either, `Rand`)
  tokens
    .get(i)
    .is_none_or(|t| !parts_words_after(t))
    .then_some(i)
}

/// Whether `t`, after a word, goes on with it: a letter, or a letter written as a command (`\o`, `\ss`).
fn parts_words_after(t: &Token) -> bool {
  t.get_catcode() == Catcode::LETTER
    || (t.get_catcode() == Catcode::CS && t.with_str(letter_command).is_some())
}

/// The tokens from the start of `tokens` to the next top-level separator: a line separator ([`is_line_separator`]), a
/// comma or `;`, a run of two or more spacing commands, or the mark that ends a name (`Rui Wang$^{*, 2}$ <blank line>
/// \vspace{2pt} $^1$Ant Group`, 2603.19223: the name, not the lines after it).
fn next_name_piece(tokens: &[Token]) -> &[Token] {
  let mut depth = 0usize;
  let mut lettered = false;
  for (j, t) in tokens.iter().enumerate() {
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END if depth == 0 => return &tokens[..j],
      Catcode::END => depth -= 1,
      Catcode::LETTER => lettered = true,
      _ if depth > 0 => {},
      _ if lettered && starts_with_mark(&tokens[j..]) => return &tokens[..j],
      _ if is_line_separator(t) || *t == T_OTHER!(",") || *t == T_OTHER!(";") => {
        return &tokens[..j];
      },
      _ if is_spacing_command(t)
        && tokens[j + 1..]
          .iter()
          .find(|u| u.get_catcode() != Catcode::SPACE)
          .is_some_and(is_spacing_command) =>
      {
        return &tokens[..j];
      },
      _ => {},
    }
  }
  tokens
}

/// Whether `tokens` hold a letter of name text: outside math, outside a mark command's argument and an optional
/// argument, at the top level or in a group that shows its letters ([`shows_name_text`]).
pub(in crate::base_utilities) fn has_name_letter(tokens: &[Token]) -> bool {
  let mut shown: Vec<bool> = Vec::new();
  let mut closed_shown = true;
  let mut in_math = false;
  let mut optional_argument = false;
  let mut previous: Option<&Token> = None;
  for t in tokens {
    match t.get_catcode() {
      Catcode::BEGIN => {
        let outer = shown.last().copied().unwrap_or(true);
        shown.push(
          outer
            && !optional_argument
            && !previous.is_some_and(is_mark_command)
            && shows_name_text(previous, closed_shown),
        );
      },
      Catcode::END => {
        closed_shown = shown.pop().unwrap_or(true);
      },
      Catcode::MATH => in_math = !in_math,
      Catcode::LETTER
        if !in_math && !optional_argument && shown.last().copied().unwrap_or(true) =>
      {
        return true;
      },
      Catcode::OTHER
        if *t == T_OTHER!("[") && previous.is_some_and(|p| p.get_catcode() == Catcode::CS) =>
      {
        optional_argument = true
      },
      Catcode::OTHER if *t == T_OTHER!("]") => optional_argument = false,
      _ => {},
    }
    previous = Some(t);
  }
  false
}

/// Whether a piece of an author line off its names line reads as a name, for a spacing run there to part two: its text
/// is names ([`name_count`]), not an affiliation ([`reads_as_affiliation`]), and it holds no address (`@`) or link
/// (`\url`, `\href`, `\texttt`).
fn reads_as_name_piece(piece: &[Token]) -> bool {
  // (a name's notes are no part of it: `Xinlei He\textsuperscript{1}\thanks{… \href{mailto:…}{…}}`, 2605.30366)
  let piece = outside_notes(piece);
  let text = visible_name_text(&piece);
  !text.contains('@')
    && !piece
      .iter()
      .any(|t| t.with_str(|s| matches!(s, "\\url" | "\\href" | "\\texttt" | "\\tt")))
    && name_count(&text) > 0
    && !reads_as_affiliation(&Tokens::new(piece))
}
