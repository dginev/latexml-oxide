//! Perl's whole-block fallback (`Base_Utility.pool.ltxml:693`, `\lx@add@author` of the raw block) with its own preparation,
//! moved unchanged from `author_block/` (the plan's row "Knowledge in, driver out, one pass", user ruling 2026-11-08; the preparation
//! moves with it as the fallback's own, the keep-improvements ruling). [`whole_block_author`] is Perl's raw call; the old
//! driver (`author_block::dispatch`, off) gives it the block through [`strip_linebreak_options`] and `lines_prepared`,
//! the raw-token rewrites of OXIDIZED_DESIGN_DIVERGENCES #52 that go beyond Perl: note environments as commands, math-active scripts, tabular juggles and ACM columns as line breaks,
//! alignment environments unwrapped, `\hspace` runs between names as separators (`normalize_hspace_separators` and its
//! row helpers), icons joined to their links, glued marks as marks, the ordinal and in-word superscripts. At
//! `LATEXML_FM_EMIT=2` the emitter and `defs/author_macros.rs` (the queued entry) call it raw, with none of them; three
//! items are a binding's too (`strip_linebreak_options`, `has_author_marks`, `glued_line_marks_as_marks`).

use super::*;

/// A block kept whole as the old driver gives it to [`whole_block_author`] (`author_block::dispatch`, off): its lines prepared
/// ([`lines_prepared`]) and its breaks' options gone ([`strip_linebreak_options`]).
pub(in crate::base_utilities) fn prepared_block(written: Tokens) -> Tokens {
  lines_prepared(strip_linebreak_options(written))
}

/// The lines of an author block as `\lx@add@authors` reads them, its marks, separators and alignments normalized
/// ([`add_authors_calls_sectioned`]; the tabular fallback's, [`whole_block_author`]).
pub(in crate::base_utilities) fn lines_prepared(stuff: Tokens) -> Tokens {
  let stuff = note_environments_as_commands(stuff);
  let stuff = math_active_scripts(stuff);
  let stuff = tabular_juggles_as_line_breaks(stuff);
  let stuff = unbrace_acm_author_columns(stuff);
  let stuff = unwrap_alignment_environment(stuff);
  let stuff = trailing_alignment_environment_as_lines(stuff);
  // Beyond-Perl (OXIDIZED_DESIGN #52), two composable normalizations applied
  // BEFORE branch selection so both branches benefit, and so a symbol mark can
  // no longer spuriously trigger the affiliation-marker branch:
  //   1. horizontal-space separators (`\hspace{len}` / `\hfill`) → `\quad`, the
  //      "regular" poor-man's author separator LaTeXML's `\and`/`\quad` splitter
  //      otherwise misses (witness arXiv:2506.06941, six authors bunched);
  //   2. footnote-SYMBOL superscripts (`$^{*}$`, `\textsuperscript{\dagger}` —
  //      equal-contribution / corresponding notes, NEVER affiliation numbers) →
  //      a visible `\lx@frontmatter@keepsup` sup, so they render instead of
  //      being consumed into an unmatched `affiliation:*` label and dropped.
  // For the witness, (2) removes its ONLY `^`, dropping the whole block into the
  // clean no-marker branch where (1)'s `\quad`s split all six authors.
  let stuff = normalize_hspace_separators(stuff);
  let stuff = join_icons_to_links(stuff);
  let stuff = glued_line_marks_as_marks(stuff, false);
  let stuff = rewrite_symbol_superscripts(stuff);
  let stuff = rewrite_ordinal_superscripts(stuff);
  rewrite_in_word_superscripts(stuff)
}

/// Perl's whole-block creator (Base_Utility.pool.ltxml:693, `\lx@add@authors`' `{tabular}` fallback, and :588-589):
/// `\lx@add@author[keyvals]{…}` of the block `written`, as given. The one writer of a block kept whole: the old driver's
/// tabular fallback (`author_block::dispatch`, which gives it the block [`prepared_block`]), and at `LATEXML_FM_EMIT=2`
/// Perl's raw call — the queued entry
/// (`defs::author_macros`) and the emitter's for a block it can't read or whose reading loses a name's words
/// (`observer::writer::emit`; OD #980). At `=2` no raw-token preparation runs: digestion does each rewrite's job in a
/// block nothing splits — `\\[len]` is `\lx@newline`'s own optional, a `\begin{footnote}` runs `\footnote`, a
/// mathcode-"8000 `^` reads as `\sp`, braces and superscripts digest to the same markup — and three of them hurt there
/// (the gap of `\\[0.5cm]`, hep-ph0207056; `\hspace` turned `\quad`, hep-th0111116, 1511.05911; a symbol superscript's
/// `\lx@frontmatter@symbolmark`, whose digestion glued 2608.19750's "*Corresponding author" into an affiliation):
/// measured on 1,538 papers, none helped (OD #52).
pub(in crate::base_utilities) fn whole_block_author(
  keyvals: Option<Tokens>,
  written: Tokens,
) -> Result<Tokens> {
  Ok(Invocation!(T_CS!("\\lx@add@author"), vec![
    keyvals,
    Some(written)
  ]))
}

/// `tokens` with each `\quad`/`\qquad` that parts an icon from the link after it a space: a piece of an author block
/// that is only an icon — what prints no word, an icon macro or a graphic ([`without_non_printing_arguments`]) — before
/// a link line ([`is_link_line`]) is that link's, one link line with it, a URL contact ([`is_url_line`]), not an
/// affiliation holding the logo alone (`\github \quad \url{…}`, 2601.17879's
/// `\def\github{\raisebox{-1.5pt}{\includegraphics…}}`; the r8 re-review's finding 9).
pub(in crate::base_utilities) fn join_icons_to_links(tokens: Tokens) -> Tokens {
  let mut v = tokens.unlist();
  let separator = |t: &Token| {
    [
      T_CS!("\\\\"),
      T_CS!("\\newline"),
      T_CS!("\\and"),
      T_CS!("\\And"),
      T_CS!("\\AND"),
      T_CS!("\\quad"),
      T_CS!("\\qquad"),
    ]
    .contains(t)
  };
  let mut delimiters = Vec::new();
  let mut depth = 0usize;
  for (at, t) in v.iter().enumerate() {
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth = depth.saturating_sub(1),
      _ if depth == 0 && separator(t) => delimiters.push(at),
      _ => {},
    }
  }
  let icon_only = |piece: &[Token]| {
    let piece = Tokens::new(piece.to_vec());
    !prints_nothing(&piece)
      && prints_nothing(&Tokens::new(without_non_printing_arguments(
        piece.unlist_ref(),
      )))
  };
  for (index, at) in delimiters.iter().enumerate() {
    let before = if index == 0 {
      0
    } else {
      delimiters[index - 1] + 1
    };
    let after = delimiters.get(index + 1).copied().unwrap_or(v.len());
    if (v[*at] == T_CS!("\\quad") || v[*at] == T_CS!("\\qquad"))
      && icon_only(&v[before..*at])
      && is_link_line(&Tokens::new(v[at + 1..after].to_vec()))
    {
      v[*at] = T_SPACE!();
    }
  }
  Tokens::new(v)
}

/// Does an author block mark its names (`$^{a}$`, `\textsuperscript{1}`)? A superscript inside a `\footnote` or
/// `\thanks` is the note's own text, not a mark: 0911.0568's `LPT Orsay (CNRS)~\footnote{$^a$Laboratoire …}` line
/// made its PoS author block a marked list, its affiliation and email line read as a person (63k).
pub fn has_author_marks(stuff: &Tokens) -> bool {
  let tokens = stuff.unlist_ref();
  let mut i = 0;
  while i < tokens.len() {
    let t = &tokens[i];
    if *t == T_CS!("\\footnote") || *t == T_CS!("\\thanks") {
      i += 1;
      while i < tokens.len() && tokens[i] == T_SPACE!() {
        i += 1;
      }
      if i < tokens.len() && tokens[i] == T_OTHER!("[") {
        while i < tokens.len() && tokens[i] != T_OTHER!("]") {
          i += 1;
        }
        i += 1;
      }
      if i < tokens.len() && tokens[i].code == Catcode::BEGIN {
        let mut depth = 0usize;
        while i < tokens.len() {
          match tokens[i].code {
            Catcode::BEGIN => depth += 1,
            Catcode::END => {
              depth -= 1;
              if depth == 0 {
                break;
              }
            },
            _ => {},
          }
          i += 1;
        }
      }
    } else if is_author_mark(t) {
      return true;
    }
    i += 1;
  }
  false
}

/// An author block's math-active `^` and `_` (catcode 12, mathcode "8000: mathstyle.sty, which breqn loads through
/// flexisym.sty:418, sets them so — :233-236 its default `mathactivechars` option, `\AtBeginDocument{\catcode`\^=12
/// …}`, and :251-258 `\mathcode`\^="8000` with the active `^` `\let` to `\sp`; latexml_contrib's breqn_sty.rs:138-163) written as the
/// superscript and subscript they are in math, where TeX reads the active character's meaning, `\sp`/`\sb` (tex.web
/// §1151): the marks of `{Mohamed Benzaghta$^{\star}$, …, David
/// López-Pérez}$^{\sharp}$` (2504.02633) are marks, as they are with `^` of catcode 7.
pub(in crate::base_utilities) fn math_active_scripts(stuff: Tokens) -> Tokens {
  let v = stuff.unlist_ref();
  let active = |c: &str| lookup_mathcode(c) == Some(0x8000);
  let (sup, sub) = (
    v.contains(&T_OTHER!("^")) && active("^"),
    v.contains(&T_OTHER!("_")) && active("_"),
  );
  if !(sup || sub) {
    return stuff;
  }
  let mut in_math = false;
  Tokens::new(
    v.iter()
      .map(|t| {
        if t.get_catcode() == Catcode::MATH {
          in_math = !in_math;
        }
        if in_math && sup && *t == T_OTHER!("^") {
          T_SUPER!()
        } else if in_math && sub && *t == T_OTHER!("_") {
          T_SUB!()
        } else {
          *t
        }
      })
      .collect(),
  )
}

/// An author block's `\begin{footnote}{text}…\end{footnote}` as LaTeX reads it: `\begin{footnote}` runs `\footnote`,
/// which takes the braced text after it as its argument, so what follows up to `\end{footnote}` is the author line's
/// (`Velibor \v{Z}eli\begin{footnote}{Linn\'{e} FLOW Centre, …},\end{footnote} Du\v{s}an Zorica\begin{footnote}…`,
/// 1703.02032: the comma printed after the first note's mark parts the names). Written `\footnote{text}` and the rest,
/// the note is the name's and its comma a separator; read as one unit, the environment hid it. Only a `{footnote}` whose
/// braced text and `\end{footnote}` follow. The environment's group (`\begin` is `\begingroup`, `\end` its
/// `\endgroup`, latex.ltx) is not kept: it would open in one name and close in the next. So a declaration between the
/// note and `\end{footnote}` is no longer confined to it; in an author block that span is the separator after the mark.
pub(in crate::base_utilities) fn note_environments_as_commands(stuff: Tokens) -> Tokens {
  let v = stuff.unlist_ref();
  let name_is = |at: usize, name: &str| -> Option<usize> {
    if v.get(at)?.get_catcode() != Catcode::BEGIN {
      return None;
    }
    let close = (at + 1..v.len()).find(|&k| v[k].get_catcode() == Catcode::END)?;
    let read: String = v[at + 1..close].iter().map(ToString::to_string).collect();
    (read == name).then_some(close + 1)
  };
  if !v.iter().any(|t| *t == T_CS!("\\begin")) {
    return stuff;
  }
  let mut out: Vec<Token> = Vec::with_capacity(v.len());
  let mut changed = false;
  let mut i = 0;
  while i < v.len() {
    if v[i] == T_CS!("\\begin")
      && let Some(mut j) = name_is(i + 1, "footnote")
    {
      while v.get(j) == Some(&T_SPACE!()) {
        j += 1;
      }
      let text_end =
        (v.get(j).is_some_and(|t| t.get_catcode() == Catcode::BEGIN)).then(|| skip_group(v, j));
      let end = text_end.and_then(|from| {
        (from..v.len()).find_map(|k| {
          (v[k] == T_CS!("\\end"))
            .then(|| name_is(k + 1, "footnote").map(|e| (k, e)))
            .flatten()
        })
      });
      if let (Some(text_end), Some((end_at, past_end))) = (text_end, end)
        && text_end <= v.len()
        && !v[text_end..end_at].iter().any(|t| *t == T_CS!("\\begin"))
      {
        out.push(T_CS!("\\footnote"));
        out.extend_from_slice(&v[j..text_end]);
        out.extend_from_slice(&v[text_end..end_at]);
        i = past_end;
        changed = true;
        continue;
      }
    }
    out.push(v[i]);
    i += 1;
  }
  if changed { Tokens::new(out) } else { stuff }
}

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

pub(in crate::base_utilities) fn linebreak_options_stripped(
  tokens: Tokens,
  keep_gaps: bool,
) -> Tokens {
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
/// the end come after it —, so the space is the name's: `Oleg L.\hspace*{2mm}$^{\rm a,b}$, Sergei B.` (0905.3658). A
/// `\footnotemark[n]` is such a mark here, `\protect` before a mark no text ([`own_marks_end`]).
/// `None` before a mark that leads an entry (`$^1$Univ A\quad$^2$Univ B`, an affiliation line's two entries; with one
/// tie or control space after the mark, the entry's own, `$^1$~Univ A\quad$^2$~Univ B` and `\textsuperscript{2}\ Univ
/// B`, whose two universities read as one) and before anything else.
fn space_before_own_mark(src: &[Token], start: usize) -> Option<usize> {
  if !src.get(start).is_some_and(is_separating_space) {
    return None;
  }
  let end = start + horizontal_space_run(&src[start..]).0;
  let marks = own_marks_end(&src[end..]);
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

/// [`leading_marks_end`], with what a name also sets as its own mark after a space: a `\footnotemark[n]`, and `\protect`
/// before a mark (`Zhengyi Han\hspace{0.1em}\protect\footnotemark[1]`, 2609.40220, a creator of the mark alone once
/// authblk's gate read the space, `has_author_block_separator`).
fn own_marks_end(tokens: &[Token]) -> usize {
  let mut end = 0;
  loop {
    let at = end + usize::from(tokens.get(end) == Some(&T_CS!("\\protect")));
    let next = if tokens.get(at).is_some_and(is_footnote_mark) {
      let optional = (tokens.get(at + 1) == Some(&T_OTHER!("[")))
        .then(|| tokens[at + 1..].iter().position(|t| *t == T_OTHER!("]")))
        .flatten();
      at + 1 + optional.map_or(0, |close| close + 1)
    } else {
      at + leading_marks_end(&tokens[at..])
    };
    if next == at {
      return end;
    }
    end = next;
  }
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

/// An alignment environment (`center`, `flushleft`, `flushright`) that wraps a whole author block, or a whole `\and`
/// group, is layout around the lines the splitter reads, not one unit of them: its content, so its names, `\\`
/// lines and marks split as written (`\author{\begin{center}Ann Able$^{1}$, Bob Baker$^{2}$ \\ $^1$Univ A
/// ...\end{center}}`). One inside a piece — a `{center}` affiliation under the names — stays whole
/// (`split_tokens_delimited`; 2609.01563, 17357). Nested wrappers unwrap in turn.
///
/// The declarations around such an environment go with it ([`is_wrapping_declaration`]): a declaration environment
/// (latex.ltx `\begin{small}` is `\begingroup\small`), a group opening with declarations (`{\small …}`), or bare
/// declarations before it (`\small\bfseries\begin{center}…`) — the 2015 BESIII template's
/// `\author{\begin{small}\begin{center} names \\ (BESIII Collaboration)\\ {\it $^{1}$ aff\\ …}\end{center}
/// \vspace{0.4cm}\end{small}}` (1507.02068's 385 names, 1512.08197's 377, 2201.02056's 472) had stayed one creator.
/// The size or font they set is dropped with them (OXIDIZED_DESIGN #947). They are peeled only on the way to an
/// alignment environment: a declaration wrapper with no alignment environment inside keeps its declarations on the
/// names as before (`\author{\begin{small}Ann Able$^{1}$ \\ $^{1}$Univ A\end{small}}`), and a command that prints
/// (`\lead\begin{center}…`) is no declaration. After a wrapper's close only spacing may follow ([`only_spacing`]: that
/// template's `\vspace{0.4cm}` after `\end{center}`).
pub(in crate::base_utilities) fn unwrap_alignment_environment(tokens: Tokens) -> Tokens {
  let v = tokens.unlist_ref();
  // (each unwrapping keeps a span of the one before: the span as of the last alignment environment unwrapped, the
  // declarations peeled past it with no alignment environment inside staying)
  let (mut lo, mut hi) = (0, v.len());
  let mut committed: Option<(usize, usize)> = None;
  loop {
    let toks = &v[lo..hi];
    let start = leading_wrapping_declarations_end(toks);
    let end = toks.len() - toks.iter().rev().take_while(|t| **t == T_SPACE!()).count();
    if start >= end {
      break;
    }
    let ends_block = |close: usize| close <= end && only_spacing(&toks[close..end]);
    if toks[start] == T_CS!("\\begin") {
      let stream: VecDeque<Token> = toks[start + 1..end].iter().copied().collect();
      let Some(span) = environment_closes_ahead(&stream) else {
        break;
      };
      let name_end = (start + 2..end)
        .find(|&k| toks[k].get_catcode() == Catcode::END)
        .unwrap_or(start + 1);
      let name: String = toks[start + 2..name_end]
        .iter()
        .map(|t| t.to_string())
        .collect();
      let close = start + 1 + span;
      let alignment = matches!(name.as_str(), "center" | "flushleft" | "flushright");
      // The environment must end the block, its `\end{name}` the last thing that prints.
      if !ends_block(close) || !(alignment || is_wrapping_declaration(&s!("\\{name}"))) {
        break;
      }
      let end_cs = (name_end + 1..close)
        .rev()
        .find(|&k| toks[k] == T_CS!("\\end"))
        .unwrap_or(close);
      (lo, hi) = (lo + name_end + 1, lo + end_cs);
      if alignment {
        committed = Some((lo, hi));
      }
    } else if toks[start].get_catcode() == Catcode::BEGIN && ends_block(skip_group(toks, start)) {
      // a group around the block: its declarations are peeled at the next turn
      let close = skip_group(toks, start);
      (lo, hi) = (lo + start + 1, lo + close - 1);
    } else {
      break;
    }
  }
  match committed {
    Some((lo, hi)) => Tokens::new(v[lo..hi].to_vec()),
    None => tokens,
  }
}

/// The declarations that may wrap an alignment environment around an author block ([`unwrap_alignment_environment`]),
/// as a command or as an environment's name: the font and size declarations of [`NAME_TEXT_COMMANDS`] (not the
/// commands among them that take their text as an argument, `\textbf`, `\emph`) and the paragraph layout ones.
pub(in crate::base_utilities) fn is_wrapping_declaration(cs: &str) -> bool {
  (NAME_TEXT_COMMANDS.contains(&cs) && !cs.starts_with("\\text") && cs != "\\emph")
    || matches!(
      cs,
      "\\centering" | "\\raggedright" | "\\raggedleft" | "\\noindent" | "\\par"
    )
}

/// Where the spaces, spacing ([`leading_spacing_end`]) and wrapping declarations ([`is_wrapping_declaration`]) that open
/// `v` end.
fn leading_wrapping_declarations_end(v: &[Token]) -> usize {
  let mut i = 0;
  loop {
    i += leading_spacing_end(&v[i..]);
    match v.get(i) {
      Some(t) if t.get_catcode() == Catcode::CS && t.with_str(is_wrapping_declaration) => i += 1,
      _ => return i,
    }
  }
}

/// An alignment environment (`center`, `flushleft`, `flushright`) that opens after the names and ends the block is
/// layout too: it starts a line of its own, its content the lines under the names — `\author{Ann Able and Bob Baker
/// \begin{center}Department of Physics and Astronomy\end{center}}` prints the department centred under both names, read
/// as `… Bob Baker \\ Department …`, every name's of the row (R12; 2609.01563, 2609.17357, 2609.33831 — it had been kept
/// whole inside the last personname). One opening the block is [`unwrap_alignment_environment`]'s. Spacing after its
/// `\end{…}` is no line ([`only_spacing`]: `…\end{center}\vspace{2mm}}`); an `\end{small}` there is none — it
/// closes an environment opened before the names (2012.14750's `\author{\author{Author list} \begin{small}
/// \begin{center}…\end{center}\vspace{0.6cm}\end{small}}`, whose `\begin{small}` a names line had kept open).
pub(in crate::base_utilities) fn trailing_alignment_environment_as_lines(tokens: Tokens) -> Tokens {
  let toks = tokens.unlist();
  let end = toks.len() - toks.iter().rev().take_while(|t| **t == T_SPACE!()).count();
  let mut depth = 0i32;
  for start in 0..end {
    match toks[start].get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth -= 1,
      _ => {},
    }
    if depth != 0
      || toks[start] != T_CS!("\\begin")
      || toks[..start].iter().all(|t| *t == T_SPACE!())
    {
      continue;
    }
    let stream: VecDeque<Token> = toks[start + 1..end].iter().copied().collect();
    let Some(span) = environment_closes_ahead(&stream) else {
      continue;
    };
    let name_end = (start + 2..end)
      .find(|&k| toks[k].get_catcode() == Catcode::END)
      .unwrap_or(start + 1);
    let name: String = toks[start + 2..name_end]
      .iter()
      .map(|t| t.to_string())
      .collect();
    // (only spacing follows its close: `\end{center}\vspace{2mm}`)
    let close = start + 1 + span;
    if close > end
      || !only_spacing(&toks[close..end])
      || !matches!(name.as_str(), "center" | "flushleft" | "flushright")
    {
      continue;
    }
    let end_cs = (name_end + 1..close)
      .rev()
      .find(|&k| toks[k] == T_CS!("\\end"))
      .unwrap_or(close);
    let mut out = toks[..start].to_vec();
    out.push(T_CS!("\\\\"));
    out.extend_from_slice(&toks[name_end + 1..end_cs]);
    return Tokens::new(out);
  }
  Tokens::new(toks)
}

/// A parameterless macro whose expansion closes the title page's tabular and opens another, not an author separator
/// ([`author_and_splits`]): the ACM SIG template's `\def\sharedaffiliation{\end{tabular}\begin{tabular}{c}}` in `\author`
/// (sig-alternate.cls:283's `\alignauthor` juggles the same way) prints a line break between the names above and the
/// affiliation below (1704.03168, 1301.5612, 1401.6981, 0712.1016). Read by its meaning, whatever its name: the
/// rehearsal binds it to the column or the row break it is (`observer::rehearsal`), as the juggle written inline. It
/// reopens one after what it prints between (1301.5612's `\def\sharedaffiliation{\end{tabular}\newline
/// \begin{tabular}{c}}`), which this gives; a macro that only closes one (`\newcommand{\etab}{\end{tabular}}`, closing a
/// tabular the block opened itself) is no juggle.
pub(in crate::base_utilities) fn tabular_juggle_between(t: &Token) -> Option<Vec<Token>> {
  let rest = tabular_close_rest(t)?;
  (0..rest.len())
    .find(|&at| environment_call_len(&rest[at..], T_CS!("\\begin"), "tabular").is_some())
    .map(|at| rest[..at].to_vec())
}

/// The old driver's looser test, kept as it was until the switch flips (the switch-off output stays as it was): a
/// macro whose expansion starts by closing a tabular, written `\\` ([`tabular_juggles_as_line_breaks`]).
fn closes_a_tabular(t: &Token) -> bool { tabular_close_rest(t).is_some() }

/// The rest of `body` past the environment command `cs` and the `{name}` group after it (`\end{tabular}`), if it
/// starts with them.
fn past_environment(body: &[Token], cs: Token, name: &str) -> Option<Vec<Token>> {
  environment_call_len(body, cs, name).map(|len| body[len..].to_vec())
}

/// How many tokens the environment command `cs` and the `{name}` group after it (`\end{tabular}`) take, if `body`
/// starts with them.
pub(in crate::base_utilities) fn environment_call_len(
  body: &[Token],
  cs: Token,
  name: &str,
) -> Option<usize> {
  // (the command and the open brace first: this runs at every token position of a block, `inline_juggles_as_columns`)
  let (first, group) = body.split_first()?;
  let (open, group) = group.split_first()?;
  if *first != cs || open.get_catcode() != Catcode::BEGIN {
    return None;
  }
  let close = group.iter().position(|t| t.get_catcode() == Catcode::END)?;
  spells(&group[..close], name).then_some(close + 3)
}

/// Whether `group`'s tokens, written out as `to_string()` writes them, spell `name`: matched as they are written, no
/// `String` built.
fn spells(group: &[Token], name: &str) -> bool {
  /// What is left of the name to match; `fmt::Error` at the first difference.
  struct Rest<'a>(&'a str);
  impl std::fmt::Write for Rest<'_> {
    fn write_str(&mut self, text: &str) -> std::fmt::Result {
      self.0 = self.0.strip_prefix(text).ok_or(std::fmt::Error)?;
      Ok(())
    }
  }
  let mut rest = Rest(name);
  group
    .iter()
    .all(|t| std::fmt::Write::write_fmt(&mut rest, format_args!("{t}")).is_ok())
    && rest.0.is_empty()
}

/// For a parameterless macro, not an author separator ([`author_and_splits`]), whose expansion starts with
/// `\end{tabular}`: the rest of its expansion (spaces out).
fn tabular_close_rest(t: &Token) -> Option<Vec<Token>> {
  if t.get_catcode() != Catcode::CS {
    return None;
  }
  let defn = lookup_definition(t).ok()??;
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
  let unspaced = || {
    body
      .unlist_ref()
      .iter()
      .filter(|t| t.get_catcode() != Catcode::SPACE)
  };
  // (its `\end` read before the body is copied: this runs for every macro of every `=2` rehearsal)
  if unspaced().next() != Some(&T_CS!("\\end")) {
    return None;
  }
  let body: Vec<Token> = unspaced().copied().collect();
  past_environment(&body, T_CS!("\\end"), "tabular").filter(|_| {
    !author_and_splits()
      .iter()
      .any(|s| matches!(s, SplitDelim::Token(d) if d == t))
  })
}

/// An author block's parameterless macro whose expansion closes the title page's tabular and opens another — the ACM
/// SIG template's `\def\sharedaffiliation{\end{tabular}\begin{tabular}{c}}`, set inside the class's own author tabular
/// (sig-alternate.cls:283's `\alignauthor` juggles the same way) — is the line break it prints between the names
/// above and the affiliation below: digested, it closed a tabular no author block opens and left one open, whose
/// lines (the shared affiliation) were lost (1704.03168, 1301.5612, 1401.6981, 0712.1016). The author separators
/// (`\alignauthor`) stay for the splitter. The old driver's test ([`closes_a_tabular`]) reads a macro closing a tabular
/// alone as one too; the rehearsal binds a juggle only ([`tabular_juggle_between`]) to its column or row break.
pub(in crate::base_utilities) fn tabular_juggles_as_line_breaks(tokens: Tokens) -> Tokens {
  let toks = tokens.unlist();
  if !toks.iter().any(closes_a_tabular) {
    return Tokens::new(toks);
  }
  Tokens::new(
    toks
      .into_iter()
      .map(|t| {
        if closes_a_tabular(&t) {
          T_CS!("\\\\")
        } else {
          t
        }
      })
      .collect(),
  )
}

/// The ACM SIG classes' author columns (DIVERGENCES #445), read through their own commands: the brace group that
/// follows `\alignauthor` and is the whole column — sigchi.cls writes each column as one, `\alignauthor{Name\\ \affaddr{…}\\ \email{…}}` — is
/// its content, so the column's `\\` lines split (1906.01122); and an `\affaddr{…}` is its content, so a mark that
/// leads it (`\affaddr{\textsuperscript{1} Univ…}`, 1608.06253) leads the affiliation line, as Perl's OmniBus reads
/// `\affaddr` as `\address` (OmniBus.cls.ltxml:92). Only at depth 0 and only after these two commands, which no
/// TeX Live file defines: a brace group anywhere else (`\author{{Smith, Jr., John}}`) keeps its braces.
pub(in crate::base_utilities) fn unbrace_acm_author_columns(tokens: Tokens) -> Tokens {
  let toks = tokens.unlist();
  if !toks
    .iter()
    .any(|t| *t == T_CS!("\\alignauthor") || *t == T_CS!("\\affaddr"))
  {
    return Tokens::new(toks);
  }
  let mut out = Vec::with_capacity(toks.len());
  let mut depth = 0usize;
  let mut i = 0;
  while i < toks.len() {
    let t = toks[i];
    i += 1;
    let column = t == T_CS!("\\alignauthor");
    if depth == 0 && (column || t == T_CS!("\\affaddr")) {
      if column {
        out.push(t);
      }
      let mut k = i;
      while k < toks.len() && toks[k].get_catcode() == Catcode::SPACE {
        k += 1;
      }
      if k < toks.len() && toks[k].get_catcode() == Catcode::BEGIN {
        let mut level = 0usize;
        let mut close = None;
        for (n, u) in toks[k..].iter().enumerate() {
          match u.get_catcode() {
            Catcode::BEGIN => level += 1,
            Catcode::END => {
              level -= 1;
              if level == 0 {
                close = Some(k + n);
                break;
              }
            },
            _ => {},
          }
        }
        // A column's group is its whole content only when nothing but spaces and `\\` follow it before the next
        // author: `\alignauthor {\large Ann}\\ \affaddr{…}` is a name's font group, kept.
        let whole = |close: usize| {
          let mut n = close + 1;
          while n < toks.len()
            && (toks[n].get_catcode() == Catcode::SPACE || toks[n] == T_CS!("\\\\"))
          {
            n += 1;
          }
          n == toks.len()
            || [
              T_CS!("\\alignauthor"),
              T_CS!("\\and"),
              T_CS!("\\And"),
              T_CS!("\\AND"),
            ]
            .contains(&toks[n])
        };
        if let Some(close) = close
          && (!column || whole(close))
        {
          // The column's own `\affaddr`s are read the same way.
          out.extend(unbrace_acm_author_columns(Tokens::new(toks[k + 1..close].to_vec())).unlist());
          i = close + 1;
          continue;
        }
      }
      if !column {
        out.push(t);
      }
      continue;
    }
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth = depth.saturating_sub(1),
      _ => {},
    }
    out.push(t);
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
