//! Perl's whole-block fallback (`Base_Utility.pool.ltxml:693`, `\lx@add@author` of the raw block) with its own preparation,
//! moved unchanged from `author_block/` (the plan's row "Knowledge in, driver out, one pass", user ruling 2026-11-08; the preparation
//! moves with it as the fallback's own, the keep-improvements ruling). [`whole_block_author`] is Perl's raw call, which the
//! emitter and `defs/author_macros.rs` (the queued entry) make with the block raw, the emitter's tabular juggles aside
//! ([`tabular_juggles_in_a_whole_block`]). The old driver's raw-token preparation (OXIDIZED_DESIGN_DIVERGENCES #52: note
//! environments as commands, math-active scripts, tabular juggles and ACM columns as line breaks, alignment environments
//! unwrapped, `\hspace` runs between names as separators, icons joined to their links) went with that driver; what is
//! left is a binding's too (`strip_linebreak_options`, `has_author_marks`, `glued_line_marks_as_marks`).

use super::{classify::title_code::title_code_author_statements, *};

/// Perl's whole-block creator (Base_Utility.pool.ltxml:693, `\lx@add@authors`' `{tabular}` fallback, and :588-589):
/// `\lx@add@author[keyvals]{…}` of the block `written`, as given. The one writer of a block kept whole, Perl's raw call —
/// the queued entry (`defs::author_macros`) and the emitter's for a block it can't read or whose reading loses a name's
/// words (`observer::emitter::emit`; OD #980) — not a footnote-counter or definition decline, which writes the block as
/// Perl's `\lx@add@authors` splits it ([`perl_author_calls`], this only for its tabular branch; OD #988). No raw-token preparation runs but the emitter's tabular juggles
/// ([`tabular_juggles_in_a_whole_block`], which digestion can't answer: 1511.05911): digestion does each rewrite's job in a
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

/// Which of Perl's `\lx@add@authors` branches (Base_Utility.pool.ltxml:686-720) a block takes ([`perl_author_branch`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::base_utilities) enum PerlAuthorBranch {
  /// `{tabular}`, `{minipage}` or `\halign` in the block: one `\lx@add@author` of it whole (:691-693)
  Whole,
  /// a superscript mark in the block: its lines cut at `@authoraffilsplits`, each an author line cut at its commas (and
  /// " and ") or a marked affiliation, a line of no mark continuing the entry before it (:694-712) — but across the `\and`
  /// family an author of its own (OD #52(g))
  Marks,
  /// neither: cut at `@authorsplits`, each piece an author whose `\\` lines are its affiliations (:713-719), the comma and
  /// " and " cutting only the piece's names line
  Split,
}

impl PerlAuthorBranch {
  /// What the decline's counted Warning says the block is written as.
  pub(in crate::base_utilities) fn written_as(self) -> &'static str {
    match self {
      PerlAuthorBranch::Whole => {
        "the block is written whole, as Perl's \\lx@add@authors writes a block holding a tabular"
      },
      PerlAuthorBranch::Marks => {
        "the block is written as Perl's \\lx@add@authors splits a block with superscript marks, its pieces digested in order"
      },
      PerlAuthorBranch::Split => {
        "the block is written as Perl's \\lx@add@authors splits it at its separators, its pieces digested in order"
      },
    }
  }
}

/// Perl's `@authorsup_markers` (Base_Utility.pool.ltxml:684).
fn perl_authorsup_markers() -> [Token; 2] { [T_SUPER!(), T_CS!("\\textsuperscript")] }

/// The branch Perl's `\lx@add@authors` takes for the block `stuff` (Base_Utility.pool.ltxml:691-694): `ToString($stuff) =~
/// /\{tabular\}|\{minipage\}|\\halign/`, else `positionOf($stuff, @authorsup_markers)`.
pub(in crate::base_utilities) fn perl_author_branch(stuff: &Tokens) -> PerlAuthorBranch {
  let text = stuff.to_string();
  if text.contains("{tabular}") || text.contains("{minipage}") || text.contains("\\halign") {
    PerlAuthorBranch::Whole
  } else if position_of(stuff, &perl_authorsup_markers()).is_some() {
    PerlAuthorBranch::Marks
  } else {
    PerlAuthorBranch::Split
  }
}

/// Perl's `\lx@add@authors` (Base_Utility.pool.ltxml:676-720) of a block the frontmatter emitter declines to read
/// (`observer::emitter::emit`'s footnote-counter and definition declines): its calls, in order, the `n`-th `\lx@add@author`
/// (from 0) given `keys(n)` — and how many creators they make —, its `dequeueFrontMatter` the emitter's own (it writes the queue's creators). The tabular branch is
/// [`whole_block_author`] of the block, its tabular juggles the line breaks they print
/// ([`tabular_juggles_in_a_whole_block`]); the two split branches cut it as Perl does (`@authorsplits`,
/// `@authoraffilsplits`, :678-682), each piece digested in its place, so the footnote counter and a definition step as
/// TeX steps them. Perl's comma and " and " cut only a names line, never a `\\` line under it (the kernel's own lists,
/// OXIDIZED_DESIGN_DIVERGENCES #52, #988: `author_affil_splits`, witness 2605.00347): Perl's cut PoS's `\\ Department of
/// Physics, College of Science, Swansea University, …` into four authors (1302.3028) and 1110.6375's `D. Schmidiger,\\
/// E. Wulf, S. Zhao and A. Zheludev$^\dag$` glued "D. SchmidigerE. Wulf, S. Zhao" (cr5_fncounter_and: Ann Able∗ and Bob Baker‡; 1003.2281, cs0404029, 2410.22720; the `\footremember`
/// of 1404.5602, 2503.21719, 0809.5152, 1110.6375). There a juggle — a macro closing the class's tabular,
/// `\alignauthor` — parts two authors, as `\and` does (`author_and_splits`; Perl's OmniBus makes it empty,
/// OmniBus.cls.ltxml:76, 1003.2281's sig-alternate). A `\\[len]` or `\\*` cut leaves no "[len]" or "*" at the head of
/// the next piece ([`strip_linebreak_options`], the kernel's own beyond-Perl fix of Perl's `\\[1em]` gap,
/// Base_Utility.pool.ltxml:670, KNOWN_PERL_ERRORS #75: 2007.01340's creator "[1em] Julia Fukuyama"). With the calls,
/// each creator's names as written, in order (the names line of its piece, not the affiliations under it): what the
/// emitter matches the reading's names against (`emit`'s owner labels). Beyond Perl, where the kernel reads a line
/// otherwise (OD #52, #988): a line of no mark that starts an author at the `\and` family is a names line, cut at its
/// commas as one is. As Perl's `next unless $block->unlist` (:714), an empty name between two cuts (`\corrauth, and
/// Zhengru Zhang`, 2004.03371) makes no creator. And each call's tokens as cut, its wrapping aside, with the creator
/// it makes (`None` for an affiliation): what a call digests of the block, in order ([`PerlCalls::cut`]).
pub(in crate::base_utilities) fn perl_author_calls(
  keys: impl Fn(usize) -> Option<Tokens>,
  stuff: Tokens,
) -> Result<PerlCalls> {
  let branch = perl_author_branch(&stuff);
  if branch == PerlAuthorBranch::Whole {
    return Ok(PerlCalls {
      calls: vec![whole_block_author(
        keys(0),
        tabular_juggles_in_a_whole_block(stuff.clone()),
      )?],
      cut:   vec![(Some(0), stuff.unlist_ref().to_vec())],
      names: vec![stuff],
    });
  }
  let mut cut: Vec<(Option<usize>, Vec<Token>)> = Vec::new();
  // (the creators the calls cut so far make: `cut`'s numbers, as `author` numbers its keys)
  let mut made = 0usize;
  let mut next = || {
    made += 1;
    made - 1
  };
  let stuff = strip_linebreak_options(tabular_juggles_as(stuff, T_CS!("\\and")));
  let mut pieces: Vec<Tokens> = Vec::new();
  let mut author = |names: Tokens, content: Tokens| {
    let n = pieces.len();
    pieces.push(names);
    whole_block_author(keys(n), content)
  };
  let affiliation = |content: Tokens| -> Result<Tokens> {
    Ok(Invocation!(T_CS!("\\lx@add@affiliation"), vec![
      None,
      Some(content)
    ]))
  };
  let markers = perl_authorsup_markers();
  // (the `\and` family parts two authors: OD #52(g)'s hard boundary, `author_and_splits`)
  let and_family = author_and_splits();
  let parts_authors = |cut: &[Token]| {
    cut.first().is_some_and(|t| {
      and_family
        .iter()
        .any(|d| matches!(d, SplitDelim::Token(u) if u == t))
    })
  };
  // (a names line's own cuts, Perl's comma and " and " — at the top level, as `SplitTokens` cuts: a note's text keeps its
  // commas)
  let names = |line: Tokens| split_tokens(line, vec![T_OTHER!(",").into(), literal_and()]);
  // (a names line's names, none empty: Perl's `next unless $block->unlist`, :714)
  let people_of = |line: Tokens| -> Vec<Vec<Token>> {
    names(line)
      .into_iter()
      .filter(|name| !name.is_empty())
      .map(Tokens::unlist)
      .collect()
  };
  let mut calls = Vec::new();
  if branch == PerlAuthorBranch::Marks {
    // (each entry: whether an author, and its tokens)
    let mut entries: Vec<(bool, Vec<Token>)> = Vec::new();
    for (cut, line) in split_tokens_delimited(stuff, author_affil_splits()) {
      if line.is_empty() {
        continue;
      }
      match position_of(&line, &markers) {
        // (a line of no mark continues the entry before it with the cut it was cut at — a line break, `\quad` the space
        // it prints —, which Perl's `Tokens(…)` drops: 1609.02993's "Soumith ChintalaFacebook AI Research", 1110.6375's
        // "G. EhlersOak Ridge" glued, their words lost; but one cut at the `\and` family starts an author, the kernel's
        // boundary, OD #52(g): `Ann Able$^{1}$… \and Bob Baker\\ $^{1}$Univ X` is two authors, as pdflatex prints them,
        // where Perl glues "Ann AbleBob Baker" — a names line, cut at its commas as one is: `\and Bob Baker, Carl Cole\\`
        // two authors, review revS5CR7b S3)
        None => match entries.last_mut() {
          Some(last) if !parts_authors(&cut) => {
            last.1.extend(cut);
            last.1.extend(line.unlist());
          },
          Some(_) => entries.extend(people_of(line).into_iter().map(|name| (true, name))),
          None => entries.push((true, line.unlist())),
        },
        Some(p) if p < 8 => entries.push((false, line.unlist())),
        Some(_) => entries.extend(names(line).into_iter().map(|name| (true, name.unlist()))),
      }
    }
    for (is_author, line) in entries {
      cut.push((is_author.then(&mut next), line.clone()));
      let line = Tokens::new(line);
      calls.push(if is_author {
        author(
          line.clone(),
          Invocation!(T_CS!("\\lx@author@withsup"), vec![Some(line)]),
        )?
      } else {
        affiliation(Invocation!(T_CS!("\\lx@affiliation@withsup"), vec![Some(
          line
        )]))?
      });
    }
  } else {
    let splits: Vec<SplitDelim> = vec![
      T_CS!("\\and").into(),
      T_CS!("\\And").into(),
      T_CS!("\\AND").into(),
      T_CS!("\\quad").into(),
      T_CS!("\\qquad").into(),
    ];
    for block in split_tokens(stuff, splits) {
      if block.is_empty() {
        continue;
      }
      let mut lines = split_tokens(block, vec![T_CS!("\\\\").into()]).into_iter();
      // (each person: their names as written, and their call's content)
      let mut people: Vec<(Vec<Token>, Vec<Token>)> = lines
        .next()
        .map(people_of)
        .unwrap_or_default()
        .into_iter()
        .map(|name| (name.clone(), name))
        .collect();
      if people.is_empty() {
        people.push((Vec::new(), Vec::new()));
      }
      // (the block's `\\` lines its last name's, as Perl's one author of the piece has them)
      let mut raw: Vec<Vec<Token>> = people.iter().map(|(names, _)| names.clone()).collect();
      if let Some(last) = people.last_mut() {
        for line in lines.filter(|line| !line.is_empty()) {
          if let Some(raw) = raw.last_mut() {
            raw.extend_from_slice(line.unlist_ref());
          }
          last.1.extend(affiliation(line)?.unlist());
        }
      }
      for ((names, person), raw) in people.into_iter().zip(raw) {
        cut.push((Some(next()), raw));
        calls.push(author(Tokens::new(names), Tokens::new(person))?);
      }
    }
  }
  Ok(PerlCalls { calls, names: pieces, cut })
}

/// What [`perl_author_calls`] makes of a block: Perl's calls, in order.
pub(in crate::base_utilities) struct PerlCalls {
  /// the `\lx@add@author` and `\lx@add@affiliation` calls
  pub calls: Vec<Tokens>,
  /// each creator's names as written (its piece's names line)
  pub names: Vec<Tokens>,
  /// each call's tokens of the block as cut, its wrapping aside, with the creator it makes (`None` for an affiliation):
  /// a subsequence of the block's tokens, in order
  pub cut:   Vec<(Option<usize>, Vec<Token>)>,
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
    .map(|at| without_optional_line_breaks(&rest[..at]))
}

/// `between` without a `\linebreak[n]` of `n` below 4, which allows a break and forces none (latex.ltx:9235-9270
/// `\@no@lnbk`, `\penalty-\@getpen{n}`: 0, `\@lowpenalty`, `\@medpenalty`, `\@highpenalty`; only 4 is `\@M`), so
/// prints no row break: neurips_2022.sty:285's `\NewAnd`, `\hfil\linebreak[0]\hfil` between its columns, which the PDF
/// sets side by side as `\and`'s (2205.14141; review revPB S2: an unmarked name after it was read as an address line).
fn without_optional_line_breaks(between: &[Token]) -> Vec<Token> {
  let mut out = Vec::with_capacity(between.len());
  let mut i = 0;
  while i < between.len() {
    let optional = between[i] == T_CS!("\\linebreak")
      && between.get(i + 1) == Some(&T_OTHER!("["))
      && between.get(i + 3) == Some(&T_OTHER!("]"))
      && between
        .get(i + 2)
        .is_some_and(|n| ["0", "1", "2", "3"].iter().any(|d| n.with_str(|s| s == *d)));
    if optional {
      i += 4;
    } else {
      out.push(between[i]);
      i += 1;
    }
  }
  out
}

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
  tabular_close_rest_of_any(t).filter(|_| {
    !author_and_splits()
      .iter()
      .any(|s| matches!(s, SplitDelim::Token(d) if d == t))
  })
}

/// A block written whole (the emitter's, `observer::emitter::emit`; OD #980), each macro whose
/// expansion starts by closing a tabular — a juggle or a column, `\alignauthor` too, which nothing splits there — the line
/// break it prints between what it parts, one opening the block nothing: digested, it closed a tabular the block never
/// opened, inside the creator's `\lx@personname`, four Errors and every name lost (vldb.cls:270 `\def\alignauthor{\end{tabular}
/// \begin{tabular}[t]{p{\auwidth}}\centering}`, which its `\@maketitle` sets inside its own `\begin{tabular}[t]{c}\@author
/// \end{tabular}`; 1511.05911). The deleted old driver's own rewrite of a juggle (`tabular_juggles_as_line_breaks`),
/// the column's split ([`author_and_splits`]) a line break where nothing splits.
pub(in crate::base_utilities) fn tabular_juggles_in_a_whole_block(tokens: Tokens) -> Tokens {
  tabular_juggles_as(tokens, T_CS!("\\\\"))
}

/// `tokens` with each macro whose expansion starts by closing a tabular ([`tabular_juggles_in_a_whole_block`]) the
/// `separator`, one opening the block nothing.
fn tabular_juggles_as(tokens: Tokens, separator: Token) -> Tokens {
  let toks = tokens.unlist();
  if !toks.iter().any(|t| tabular_close_rest_of_any(t).is_some()) {
    return Tokens::new(toks);
  }
  let mut out: Vec<Token> = Vec::with_capacity(toks.len());
  for t in toks {
    if tabular_close_rest_of_any(&t).is_some() {
      if !out.is_empty() {
        out.push(separator);
      }
    } else {
      out.push(t);
    }
  }
  Tokens::new(out)
}

/// For a parameterless macro whose expansion starts with `\end{tabular}`: the rest of its expansion (spaces out). An
/// undefined one the class's title code defines where it reads `\@author` ([`title_code_juggle`]) is read by that
/// definition.
fn tabular_close_rest_of_any(t: &Token) -> Option<Vec<Token>> {
  if t.get_catcode() != Catcode::CS {
    return None;
  }
  let Some(defn) = lookup_definition(t).ok()? else {
    return title_code_juggle(t);
  };
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
  tabular_close_rest_of_body(body.unlist_ref())
}

/// The rest (spaces out) of a parameterless macro's expansion `body` past the `\end{tabular}` it starts with, if it does.
fn tabular_close_rest_of_body(body: &[Token]) -> Option<Vec<Token>> {
  let unspaced = || body.iter().filter(|t| t.get_catcode() != Catcode::SPACE);
  // (its `\end` read before the body is copied: this runs for every macro of every rehearsal)
  if unspaced().next() != Some(&T_CS!("\\end")) {
    return None;
  }
  let body: Vec<Token> = unspaced().copied().collect();
  past_environment(&body, T_CS!("\\end"), "tabular")
}

/// For `t` undefined at the block, the rest of the expansion past its `\end{tabular}` ([`tabular_close_rest_of_body`]) of
/// the parameterless `\def` the class's title code makes of it where it reads `\@author`
/// (`defs::frontmatter_fields::title_code_author_statements`; never replayed for the author content, closing a group it
/// did not open): neurips_2022.sty:285's `\def\NewAnd{\end{tabular}\hfil\linebreak[0]\hfil\begin{tabular}[t]{c}
/// \ignorespaces}`, defined by its `\@maketitle` beside `\And` and `\AND` (2205.14141, 2205.14539) — read as a juggle
/// defined at the block is (`\sharedaffiliation`, [`tabular_juggle_between`]).
fn title_code_juggle(t: &Token) -> Option<Vec<Token>> {
  title_code_author_statements()
    .into_iter()
    .find(|(definee, statement)| definee == t && statement.first() == Some(&T_CS!("\\def")))
    .and_then(|(_, statement)| {
      // (`\def\X{…}`: parameterless, its body's group right after the definee)
      let (open, rest) = statement.get(2..)?.split_first()?;
      let (close, inner) = rest.split_last()?;
      (open.get_catcode() == Catcode::BEGIN && close.get_catcode() == Catcode::END)
        .then_some(inner)
        .and_then(tabular_close_rest_of_body)
    })
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
