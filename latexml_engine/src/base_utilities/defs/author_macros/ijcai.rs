//! The IJCAI sectioned author block's split (`ijcai_split_call`), the old `\lx@add@authors` driver's one live step: the
//! sole-author and separator-macro expansions, the notes taken out of marks, a line of notes joining the names above,
//! the
//! section-marker test. Moved unchanged by Z18m from `defs/author_macros.rs` and `author_block/` (`names_split.rs`,
//! `preprocess.rs`, `separators.rs`), which it deleted; Option C P3 reroutes it on the reading (the plan's Plan A row).

use super::*;
use crate::base_utilities::defs::frontmatter_api::line_notes;

/// Whether a sectioned caller's block (`\lx@add@authors`, `@adaptive`, `@list`) is an IJCAI sectioned one
/// (`has_ijcai_section_marker`, after the sole-author and separator macro expansions): it goes to the section split
/// ([`ijcai_split_call`]), whose names `\lx@ijcai@names` then queues whole — a piece cut through `\affiliations` is
/// digested where a class defines it only in its `\@maketitle` (kr.sty, 2202.05511). The ledger records it too
/// (`AuthorBlock::sectioned`), whose held Errors are an IJCAI block's section markers (`resolver::Input::Block`'s
/// `markers`) — never a raw `\@maketitle`'s own `\name`/`\email` held in a plain `\author` (1908.02923's JAIR rows;
/// `perfect_kernel_batch63::author_markup_title_code_forms`), whose reading ended at the marker, its address a note.
pub(super) fn ijcai_block(stuff: &Tokens) -> Result<bool> {
  has_ijcai_section_marker(&expand_separator_macros(expand_sole_author_macro(
    stuff.clone(),
  )))
}

/// An IJCAI sectioned block's split ([`ijcai_block`]), the old `\lx@add@authors` driver's first step, kept as the one
/// path that still cuts a block at the call. Beyond-Perl (surpasses Perl; KNOWN_PERL_ERRORS #100): IJCAI-style author
/// blocks — ijcai97.sty and its derivatives (e.g. the ttm.sty in arXiv:2401.03955) — pack names, `\affiliations` and a
/// comma-separated `\emails` list into ONE `\author{}`. Perl's splitter does not recognise those section markers, so the
/// comma-joined email list is shredded into phantom author creators and the affiliation is dropped (Perl 0.8.8, which
/// raises `\affiliations`/`\emails` as undefined). The block goes to the shared sectioned-author machinery
/// (`\lx@ijcai@authorsplit`, also used by `ijcai_sty`): it consumes the markers as `Until:` delimiters (so they no longer
/// error) and splits names / affiliations / emails, each email placed as a class's address list is (OD #908); the names
/// re-enter as `\lx@ijcai@names`, queued whole. Witness html_feedback#1361 + #1362. A marker counts at the block's top
/// level only, and only as a separator: undefined there or the ijcai binding's no-op. A document's own
/// `\emails`/`\affiliations` with text is author text, which LaTeX expands — counted, the split handed it back unsplit,
/// without end (`\emails` before any `\affiliations`, 2407.10582; `\affiliations` inside `\thanks`, 2505.05474;
/// `PushbackLimit`). The block is read as the driver read it first: a `\let` alias of a mark command is the command
/// (`\let\up\textsuperscript`, 2209.14855; [`unalias_mark_commands`]); a note written inside a mark is taken out after
/// it (`\textsuperscript{\rm 1,\rm 2\thanks{…}}`, 2512.05464; [`notes_out_of_marks`]); an empty mark is none
/// ([`drop_empty_marks`]); a mark opening a line that prints no label is none (2303.08793;
/// [`blank_line_marks_dropped`]); a line of notes only after a `\\` joins the line above, its notes no name requests
/// the document's, added after the split (2609.17106, 2411.05212, 2312.10807; [`notes_line_joins_names`],
/// OXIDIZED_DESIGN_DIVERGENCES #957). Where those steps leave no top-level marker, the block is queued whole
/// ([`whole_block_call`]), as any other.
pub(super) fn ijcai_split_call(stuff: Tokens, replace: bool, late: bool) -> Result<Tokens> {
  let read = unalias_mark_commands(expand_separator_macros(expand_sole_author_macro(
    stuff.clone(),
  )));
  let read = blank_line_marks_dropped(drop_empty_marks(notes_out_of_marks(read)));
  let (read, document_notes) = notes_line_joins_names(read);
  if !has_ijcai_section_marker(&read)? {
    return whole_block_call(stuff, replace, None, late);
  }
  let mut out = vec![T_CS!("\\lx@ijcai@authorsplit")];
  out.extend(read.unlist());
  out.push(T_CS!("\\affiliations"));
  out.push(T_CS!("\\done"));
  out.extend(document_notes);
  Ok(Tokens::new(out))
}

// (moved unchanged from `author_block/names_split.rs` by Z18m, which deleted `author_block/`: Splitting an author line
// into names: separators expanded, wrappers, leading spacing.)
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

// (moved unchanged from `author_block/preprocess.rs` by Z18m, which deleted `author_block/`: Author-block
// preprocessing: math-active scripts and environments read as commands.)
/// Whether a mark's operand, its notes taken out, prints nothing: braces, spaces, `\relax` and font switches only
/// (`^{\thanks{…}}`, `^{\mathbf{}}`).
pub(in crate::base_utilities) fn mark_prints_nothing(mark: &[Token]) -> bool {
  mark.iter().all(|t| {
    matches!(
      t.get_catcode(),
      Catcode::BEGIN | Catcode::END | Catcode::SPACE
    ) || *t == T_CS!("\\relax")
      || (t.get_catcode() == Catcode::CS && t.with_str(|name| FONT_SWITCH.is_match(name)))
  })
}

/// An author block's notes written inside a mark (`Nataliia Babina\textsuperscript{\rm 1,\rm 2\thanks{Work done …}}`,
/// `Ann$^{1,\thanks{Corresponding author.}}$`, `$^{1\footnotemark[1]}$`) taken out after it: LaTeX prints the labels
/// and the note's own mark in the superscript, its text as a footnote, so the note is the author's as one written
/// after the mark is (`\textsuperscript{\rm 1,\rm 2}\thanks{…}`), and the mark's labels are read without the note's
/// text in them — the request was the label `2\thanks{…}`, which no list answers and which printed nothing, so the
/// mark `2` and the note were lost (2512.05464, 1911.03118, 1912.00833, 2211.09622, 2305.16614, 2306.17472,
/// 2309.12862, 2402.18447, 2403.04164, 2410.12564, 2501.01423, 2502.15412, 2503.05905, 2603.17529, 2605.17958). A
/// mark holding nothing else (`$^{\thanks{Topical Group Conveners}}$`, 2212.00809) goes, its note kept. The mark
/// readers keep any note left in an operand all the same ([`split_mark_notes`], `\lx@sup@request@affiliation`). A
/// macro the mark holds that makes a note is taken out too (`\rm 1,2,\equalcontrib`, 2603.17529; `1\samethanks`,
/// 2309.12862). Perl reads the operand as the label too (Base_Utility.pool.ltxml:729-734; KNOWN_PERL_ERRORS #1136,
/// OXIDIZED_DESIGN_DIVERGENCES #881).
pub(in crate::base_utilities) fn notes_out_of_marks(stuff: Tokens) -> Tokens {
  let v = stuff.unlist_ref();
  if !v
    .iter()
    .any(|t| t.get_catcode() == Catcode::CS && !is_author_mark(t))
    || !v.iter().any(is_author_mark)
  {
    return stuff;
  }
  let mut out: Vec<Token> = Vec::with_capacity(v.len());
  let mut changed = false;
  let mut i = 0;
  while i < v.len() {
    if is_author_mark(&v[i]) {
      let open = i + 1 + v[i + 1..].iter().take_while(|t| **t == T_SPACE!()).count();
      if v
        .get(open)
        .is_some_and(|t| t.get_catcode() == Catcode::BEGIN)
      {
        let close = skip_group(v, open);
        if close <= v.len()
          && v[close - 1].get_catcode() == Catcode::END
          && let Some((mark, notes)) = split_mark_notes(&v[open + 1..close - 1])
        {
          if !mark_prints_nothing(&mark) {
            out.push(v[i]);
            out.push(T_BEGIN!());
            out.extend(mark);
            out.push(T_END!());
          }
          out.extend(notes);
          i = close;
          changed = true;
          continue;
        }
      }
    }
    out.push(v[i]);
    i += 1;
  }
  if changed { Tokens::new(out) } else { stuff }
}

/// An author block's line break that opens a line of notes only (`line_notes`) taken out, with its `*` and
/// `[len]` and any spacer `\\` after it: the notes join the line above, as written without the break. Such a line
/// prints only its notes' marks; its notes are the names' above (`Ann Able$^{1,*}$, … and Cat Cole$^{1}$\\
/// \thanks{$^{1}$Ann Able and Cat Cole are with Univ A, Town.}…`: 2609.17106, 2411.05212, 2312.10807's `\\[3.0ex]`).
/// Read as a line of its own, it led with the mark inside its first note — `author_mark_position` (marks.rs) counts a
/// mark inside a `\thanks`, `has_author_marks` skips it, and the two disagree — and became one affiliation answering
/// mark 1 while its notes went to the document: Ann's and Cat's affiliation lost, all three notes left unowned (Perl
/// the same, Base_Utility.pool.ltxml:702-712; KNOWN_PERL_ERRORS #1206, OXIDIZED_DESIGN_DIVERGENCES #957). Only that
/// misreading: a block whose names carry marks (`has_author_marks`, outside its notes) and a line with a note whose
/// leading mark a name requests ([`notes_the_names_request`]: its other notes the document's, returned to be added after
/// the block's calls — review rev100c S1); a line of unmarked notes stays a line — in a marked block it continues the entry above as joined it would, and in an
/// unmarked block it is the document's notes, no author's by position (ieeetran_cls.rs `without_stop_after_grade`, batch
/// 86d: 2112.14209, 2409.11755, 2510.15773, 2605.22853). And only after a names line: its `\and` group's first, or one
/// whose marks follow its text — after an affiliation or an `Emails: …` line the notes stay a line of their own
/// (joined, 1911.03356's address list no longer read as one). Only a break at the block's top level, outside braces
/// and environments (a tabular's row end is no line), and only up to the next line break or `\and`.
pub(in crate::base_utilities) fn notes_line_joins_names(stuff: Tokens) -> (Tokens, Vec<Token>) {
  let v = stuff.unlist_ref();
  let brk = T_CS!("\\\\");
  if !v.contains(&brk) || !has_author_marks(&stuff) {
    return (stuff, Vec::new());
  }
  let ends_line = |t: &Token| {
    *t == brk
      || author_and_splits()
        .iter()
        .any(|split| matches!(split, SplitDelim::Token(cs) if cs == t))
  };
  let past_spaces = |mut at: usize| {
    while v.get(at) == Some(&T_SPACE!()) {
      at += 1;
    }
    at
  };
  // (past a `\\` at `at`: its `*` and `[…]`, the spaces before the `[` skipped as `\@ifnextchar` skips them)
  let past_break = |at: usize| {
    let mut j = at + 1;
    if v.get(j) == Some(&T_OTHER!("*")) {
      j += 1;
    }
    let open = past_spaces(j);
    if v.get(open) == Some(&T_OTHER!("["))
      && let Some(close) = v[open..].iter().position(|t| *t == T_OTHER!("]"))
    {
      j = open + close + 1;
    }
    j
  };
  // (the labels the block's names request, outside their notes)
  let requested: Vec<String> = author_mark_operands(v)
    .iter()
    .flat_map(|operand| mark_keys(operand))
    .collect();
  let mut out: Vec<Token> = Vec::with_capacity(v.len());
  let mut document_notes: Vec<Token> = Vec::new();
  let mut changed = false;
  let (mut depth, mut environments) = (0usize, 0usize);
  // (where the line above starts in `out`, and whether it is its `\and` group's first)
  let (mut above_start, mut first_line) = (0usize, true);
  let mut i = 0;
  while i < v.len() {
    let t = v[i];
    if t == brk && depth == 0 && environments == 0 {
      // (the line above a names line: its group's first, or one whose marks follow its text — not an affiliation its
      // mark leads, nor an unmarked line, `Emails: …` whose address list a note joined would no longer read as one,
      // 1911.03356)
      let above = Tokens::new(out[above_start..].to_vec());
      let names_above = first_line || (has_author_marks(&above) && !marker_leads(&above));
      let mut j = past_break(i);
      // (a spacer `\\` right after it, `\\ \\`)
      while v.get(past_spaces(j)) == Some(&brk) {
        j = past_break(past_spaces(j));
      }
      let mut level = 0usize;
      let line_end = (j..v.len())
        .find(|&k| {
          match v[k].get_catcode() {
            Catcode::BEGIN => level += 1,
            Catcode::END => level = level.saturating_sub(1),
            _ => {},
          }
          level == 0 && ends_line(&v[k])
        })
        .unwrap_or(v.len());
      let line = &v[j..line_end];
      if names_above
        && let Some(notes) = line_notes(line)
        && let Some((joined, document)) = notes_the_names_request(line, &notes, &requested)
      {
        out.extend(joined);
        document_notes.extend(document);
        changed = true;
        i = line_end;
        continue;
      }
    }
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth = depth.saturating_sub(1),
      _ if depth == 0 && t == T_CS!("\\begin") => environments += 1,
      _ if depth == 0 && t == T_CS!("\\end") => environments = environments.saturating_sub(1),
      _ => {},
    }
    out.push(t);
    i += 1;
    if depth == 0 && environments == 0 && ends_line(&t) {
      above_start = out.len();
      first_line = t != brk;
    }
  }
  if changed {
    (Tokens::new(out), document_notes)
  } else {
    (stuff, document_notes)
  }
}

/// A line of notes only ([`line_notes`]: its `notes`, their spans) as it joins the names above
/// ([`notes_line_joins_names`]) — the line's tokens to join, and the notes taken out of the block as the document's —, or
/// `None` when it stays a line: a note joins when the mark leading it is one the names request (`requested`,
/// [`mark_keys`]); a note no name requests (`\thanks{This work was funded by …}`, a symbol only a note prints,
/// 2312.10807's `$^{*}$Equal contribution` under `\IEEEauthorrefmark{1}`) is the document's, as the line read whole made
/// it — digested in a contact, `\thanks` is `\lx@add@pubnote@thanks` and `\footnote` `\lx@add@pubnote` (queue_digest.rs,
/// [`document_note_call`]) —, not the last name's (FIDELITY_CHARTER R1: 2609.17106's funding note on Markus Vincze, review rev100c S1); taken
/// out of the block, so no mark its text prints is read as a name's (`$^{\dagger}$Corresponding author`). No note
/// requested, or one of the document's that is no plain `\thanks{…}` or `\footnote{…}`: the line stays.
fn notes_the_names_request(
  line: &[Token],
  notes: &[(usize, usize)],
  requested: &[String],
) -> Option<(Vec<Token>, Vec<Token>)> {
  let asked: Vec<bool> = notes
    .iter()
    .map(|&(start, end)| {
      leading_note_mark(&line[start..end]).is_some_and(|operand| {
        mark_keys(&operand)
          .iter()
          .any(|key| requested.contains(key))
      })
    })
    .collect();
  if !asked.contains(&true) {
    return None;
  }
  let (mut joined, mut document) = (Vec::with_capacity(line.len()), Vec::new());
  let mut at = 0;
  for (&(start, end), asked) in notes.iter().zip(asked) {
    joined.extend_from_slice(&line[at..start]);
    at = end;
    if asked {
      joined.extend_from_slice(&line[start..end]);
      continue;
    }
    // (no `[…]` before its text: an optional argument is no pubnote's)
    document.extend(document_note_call(&line[start..end])?);
  }
  joined.extend_from_slice(&line[at..]);
  Some((joined, document))
}

/// The operand of the mark leading a note's text (`\thanks{$^{1}$Univ A}`, `\thanks{$*$ Equal contribution}`): a mark
/// before any letter of it, or a math group of no letter it opens with (`$*$`, `$\dagger$`).
fn leading_note_mark(note: &[Token]) -> Option<Vec<Token>> {
  let open = note
    .iter()
    .position(|t| t.get_catcode() == Catcode::BEGIN)?;
  let text = &note[open + 1..note.len().saturating_sub(1)];
  let text = &text[text.iter().take_while(|t| **t == T_SPACE!()).count()..];
  let tokens = Tokens::new(text.to_vec());
  if let Some(at) = author_mark_position(&tokens)
    && !name_precedes_marker(&tokens, at)
  {
    return mark_operands(text).into_iter().next();
  }
  let close = text
    .iter()
    .skip(1)
    .position(|t| t.get_catcode() == Catcode::MATH)?
    + 1;
  let inner = &text[1..close];
  (text.first()?.get_catcode() == Catcode::MATH
    && !inner.is_empty()
    && inner.iter().all(|t| t.get_catcode() != Catcode::LETTER))
  .then(|| inner.to_vec())
}

/// The labels a mark's operand keys (`clean_frontmatter_labels`, no prefix), each comma piece's, and a piece's number
/// or letter and its symbol apart as well when they are glued ([`split_glued_symbol`]: `$^{1*}$` keys `1*`, `1` and
/// `*`, as 2411.05212's `$^*$Work done …` asks).
fn mark_keys(operand: &[Token]) -> Vec<String> {
  let mut keys = Vec::new();
  for piece in operand.split(|t| *t == T_OTHER!(",")) {
    let piece: Vec<Token> = piece
      .iter()
      .filter(|t| **t != T_SPACE!())
      .copied()
      .collect();
    let mut parts = vec![piece.clone()];
    if let Some((lead, symbol)) = split_glued_symbol(&piece) {
      parts.extend([lead, symbol]);
    }
    for part in parts {
      keys.extend(clean_frontmatter_labels(&Tokens::new(part).to_string(), ""));
    }
  }
  keys
}

// (moved unchanged from `author_block/separators.rs` by Z18m, which deleted `author_block/`: Author-line separators:
// line parts, words, `and` splits, alignment and ACM column unwrapping.)
/// Whether an author block is an IJCAI sectioned one: `\affiliations` or `\emails` at its top level (the split's
/// `Until:` delimiters match nothing deeper), each undefined, `\relax` or a no-op there — the ijcai binding's
/// (`ijcai_sty.rs`), or a raw derivative's that defines them only in `\@maketitle` (ttm.sty).
pub(in crate::base_utilities) fn has_ijcai_section_marker(stuff: &Tokens) -> Result<bool> {
  let markers = [T_CS!("\\affiliations"), T_CS!("\\emails")];
  let mut depth = 0i32;
  for token in stuff.unlist_ref() {
    match token.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth -= 1,
      _ if depth == 0 && markers.contains(token) => {
        let separator = token.defined_as(&TOKEN_RELAX)
          || match lookup_definition_stored(token)? {
            None => true,
            Some(Stored::Expandable(e)) => {
              e.paramlist
                .as_ref()
                .is_none_or(|p| p.get_parameters().is_empty())
                && matches!(&e.expansion, Some(ExpansionBody::Tokens(t)) if t.is_empty())
            },
            Some(_) => false,
          };
        if separator {
          return Ok(true);
        }
      },
      _ => {},
    }
  }
  Ok(false)
}

#[cfg(test)]
mod tests {
  use latexml_core::state::{State, StateOptions, set_state};

  use super::*;

  fn setup() {
    // T_CS!/T_BEGIN! etc. and tokenize_internal need a thread-local State for
    // string interning.
    set_state(State::new(StateOptions::default()));
  }
  fn tk(s: &'static str) -> Tokens { mouth::tokenize_internal(s) }

  /// 78u: a note written inside a mark is taken out after it, the mark's labels left (2512.05464), and a mark holding
  /// only a note goes (2212.00809).
  #[test]
  fn notes_out_of_marks_takes_the_note_after_the_mark() {
    setup();
    let out = |s: &'static str| notes_out_of_marks(tk(s)).to_string();
    let tk = |s: &'static str| tk(s).to_string();
    assert_eq!(
      out(r"Ann\textsuperscript{\rm 1,\rm 2\thanks{Work {done}.}}, Bob"),
      tk(r"Ann\textsuperscript{\rm 1,\rm 2}\thanks{Work {done}.}, Bob")
    );
    assert_eq!(
      out(r"Ann$^{\mathbf{1,\thanks{Equal.}}}$ and Cy$^{\thanks{Convener.}}$"),
      tk(r"Ann$^{\mathbf{1,}}\thanks{Equal.}$ and Cy$\thanks{Convener.}$")
    );
    assert_eq!(
      out(r"Ann$^{1\footnotemark[1]}$"),
      tk(r"Ann$^{1}\footnotemark[1]$")
    );
    // (a note after the mark, and a mark holding none, are left as they are)
    assert_eq!(
      out(r"Ann$^{1}$\thanks{X}, Bob\textsuperscript{2}"),
      tk(r"Ann$^{1}$\thanks{X}, Bob\textsuperscript{2}")
    );
  }
}
