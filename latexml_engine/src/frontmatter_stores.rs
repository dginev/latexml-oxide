//! K11 — a raw class's title-page STORES reroute to the frontmatter API.
//!
//! A class loaded raw keeps its title page in stores: `\newcommand\inst[1]
//! {\gdef\@inst{#1}}` (ptptex.cls:616), `\long\def\abst#1{\long\gdef\@abst{#1}}`
//! (jpsj2.cls:841-847), read only by `\@maketitle`, which LaTeXML's locked
//! `\maketitle` never runs. Perl drops every such store (its ~200 class
//! bindings — and the generic `OmniBus.cls.ltxml:62-247` table, bypassed
//! under raw class loading — are its only route); the Rust
//! `\lx@deposit@maketitle` surpass typeset them as body paragraphs. This
//! module is OmniBus's table applied after the raw load, keyed on what the
//! class actually defined — a setter a binding defined is the binding's, even under a raw class
//! that loads it (`latexml_core::binding::store_setters`): a setter is rerouted only when (a) its NAME is in
//! the surveyed table (`~/data/pk_agents/w23/frontmatter_stores/`, 655 TL
//! classes; an explicit synonym table, never a name pattern — `ead` matches
//! `\setoddhead`, `date` matches `\cref@updatelabeldata`) and (b) its macro
//! BODY is a pure store of its argument (`#1`, or `#2` of an `[optional]{mandatory}` setter) — a
//! check on the tokens, never a guess.
//! Everything else the class defined is left alone, but its contact setters that print their argument as a title-page
//! line (`reroute_in_place_contact_setters`, after a raw class or package: OXIDIZED_DESIGN_DIVERGENCES #936); the
//! kernel's own locked `\title`/`\author`/`\date`/`\thanks` are not in the table. A store is handed
//! to the API as it is set, except one that annotates a creator (an affiliation,
//! address, e-mail, ORCID) or makes one (an editor, a speaker), which is read at
//! `\maketitle` where the author list exists. The class's `\@maketitle` is deposited with the handed stores read as
//! empty (`\lx@deposit@maketitle`, sect05.rs), so it adds only what the
//! frontmatter does not carry.
//!
//! The table's keywords setters reach `\lx@add@keywords` in two more shapes a class gives them (68n), each found
//! by a check on what the class defined, never a name pattern: a one-argument setter the class defines only for
//! its abstract's body (`abstract_setter_definitions`: wiley-article.cls:446-489 inside the `\NewEnviron` its
//! `\abstract` becomes, astlb.cls:199-205 inside the `{pazhabstract}` its `\@maketitle` typesets the abstract in,
//! mtt.cls:176-181 inside its own `{abstract}`), which the kernel's locked `{abstract}` never runs, so `\keywords`
//! was undefined (330 of 373 run-353 papers: 1706.00599, 1801.08881, 1202.2889, astro-ph0306137, 1507.06302,
//! 2103.05801, 1310.0585); and an environment whose begin code only typesets a heading carrying the class's label
//! (`label_only_begin_code`: paper.cls:621-627 `\def\keywords{\small\paragraph*{\keywordname:}}`), which became a
//! run-in `<paragraph>` where OmniBus made keywords (OmniBus.cls.ltxml:126-156). OXIDIZED_DESIGN_DIVERGENCES #634.
use std::collections::VecDeque;

use latexml_core::{
  binding::store_setters::{STORE_SETTERS, defined_by_a_binding, defined_by_a_raw_file},
  definition::origin::{DefinitionOrigin, with_origin},
  document::tag::TagContent,
};
use rustc_hash::FxHashSet;

use crate::prelude::*;

/// The definers a store is made with: `\gdef\@name{#1}`, `\def`, `\xdef`/`\edef`, LaTeX's `\protected@xdef`, and
/// `\g@addto@macro\@name{#1}` (an accumulating store).
const STORE_DEFINERS: &[&str] = &[
  "\\gdef",
  "\\def",
  "\\xdef",
  "\\edef",
  "\\protected@xdef",
  "\\protected@edef",
  "\\g@addto@macro",
];
/// Prefixes and grouping a setter's definitions may carry, which typeset nothing.
const DEFINITION_FRAME: &[&str] = &[
  "\\long",
  "\\global",
  "\\protected",
  "\\relax",
  "\\begingroup",
  "\\endgroup",
];
/// LaTeX's command definers: `\renewcommand{\thanks}[1]{}` (tufte-common.def:612).
const COMMAND_DEFINERS: &[&str] = &[
  "\\newcommand",
  "\\renewcommand",
  "\\providecommand",
  "\\DeclareRobustCommand",
];

/// The control sequences a setter's body stores argument `#<arg>` in, when the body only makes definitions: each a
/// pure store of the argument (`\gdef\@name{#1}`, any [`STORE_DEFINERS`]) or a side definition — a `\def`-family
/// definition, a `\let`, a LaTeX `\renewcommand`-family one, in any [`DEFINITION_FRAME`] — that typesets nothing:
/// tufte-common.def:608-614's `\date` stores `\@date` and also `\protected@xdef`s `\thedate` in a group whose
/// `\renewcommand{\thanks}[1]{}` swallows a `\thanks`. `None` when the body does anything else (a `\RequirePackage`,
/// apa6.cls:780 `\journal`; a flag its title page reads, tADR2e.cls:450 `\global\keywordstrue`; any typesetting).
fn pure_stores(body: &[Token], arg: usize) -> Option<Vec<Token>> {
  let mut stores = Vec::new();
  // (a store made inside `\begingroup` … `\endgroup` lasts only when it is global)
  let (mut depth, mut global) = (0usize, false);
  let mut i = 0;
  while i < body.len() {
    let t = body[i];
    if t == T_CS!("\\begingroup") {
      depth += 1;
    } else if t == T_CS!("\\endgroup") {
      depth = depth.checked_sub(1)?;
    } else if t == T_CS!("\\global") {
      global = true;
    }
    if t.get_catcode() == Catcode::SPACE || cs_in(&t, DEFINITION_FRAME) {
      i += 1;
      continue;
    }
    let is_global = std::mem::take(&mut global)
      || t.with_str(|s| {
        matches!(
          s,
          "\\gdef" | "\\xdef" | "\\protected@xdef" | "\\g@addto@macro"
        )
      });
    if t.get_catcode() != Catcode::CS {
      return None;
    }
    if t == T_CS!("\\let") {
      // `\let\cs = token`
      let mut k = skip_spaces(body, i + 1);
      body.get(k).filter(|c| c.get_catcode() == Catcode::CS)?;
      k = skip_spaces(body, k + 1);
      if body.get(k) == Some(&T_OTHER!("=")) {
        k = skip_spaces(body, k + 1);
      }
      body.get(k)?;
      i = k + 1;
      continue;
    }
    if cs_in(&t, COMMAND_DEFINERS) {
      // `\renewcommand*{\cs}[n][default]{body}` or `\renewcommand\cs…`
      let mut k = skip_spaces(body, i + 1);
      if body.get(k) == Some(&T_OTHER!("*")) {
        k = skip_spaces(body, k + 1);
      }
      if body
        .get(k)
        .is_some_and(|b| b.get_catcode() == Catcode::BEGIN)
      {
        let past = group_end(body, k)?;
        (past == k + 3 && body[k + 1].get_catcode() == Catcode::CS).then_some(())?;
        k = past;
      } else {
        body.get(k).filter(|c| c.get_catcode() == Catcode::CS)?;
        k += 1;
      }
      for _ in 0..2 {
        k = skip_spaces(body, k);
        if body.get(k) == Some(&T_OTHER!("[")) {
          k += body[k..].iter().position(|b| *b == T_OTHER!("]"))? + 1;
        }
      }
      i = group_end(body, skip_spaces(body, k))?;
      continue;
    }
    if !cs_in(&t, STORE_DEFINERS) {
      return None;
    }
    // `\def\cs<parameter text>{body}`
    let k = skip_spaces(body, i + 1);
    let target = *body.get(k).filter(|c| c.get_catcode() == Catcode::CS)?;
    let mut open = k + 1;
    while body.get(open).is_some_and(|b| {
      matches!(
        b.get_catcode(),
        Catcode::PARAM | Catcode::ARG | Catcode::OTHER
      )
    }) {
      open += 1;
    }
    let past = group_end(body, open)?;
    let stored = &body[open + 1..past - 1];
    if open == k + 1
      && (depth == 0 || is_global)
      && stored.len() == 1
      && stored[0].get_catcode() == Catcode::ARG
      && stored[0].with_str(|s| s == arg.to_string())
    {
      stores.push(target);
    }
    i = past;
  }
  Some(stores)
}

/// The store a setter `\name` keeps its argument in, `@affil` for `\@affil`; its own `\@name` unless rule (a) found
/// another ([`reroute_raw_class_stores`]).
fn store_of(name: &str) -> String {
  let store = lookup_string(&s!("lx_store_cs_{name}"));
  if store.is_empty() {
    s!("@{name}")
  } else {
    store
  }
}

/// An argument read whole, not up to a delimiter (`\def\x#1\par{…}` reads `#1` up to `\par`).
fn undelimited(param: &Parameter) -> bool { !with(param.spec, |spec| spec.contains("Until")) }

/// Does the store's frontmatter API annotate a creator (`\lx@annotate@frontmatter{ltx:creator}…`)?
/// Such an annotation is placed only on a queued creator.
fn annotates_a_creator(name: &str) -> bool {
  STORE_SETTERS.iter().any(|(n, api)| {
    *n == name
      && [
        "\\lx@add@store@affiliations",
        "\\lx@add@email",
        "\\lx@add@address",
        "\\lx@add@orcid",
      ]
      .iter()
      .any(|creator_api| api.starts_with(creator_api))
  })
}

/// Does the store's frontmatter API make a creator of its own (an editor, a speaker)? Read with the
/// creator-annotating stores at `\maketitle`, after them, so their annotations stay the authors'.
fn creates_a_creator(name: &str) -> bool {
  STORE_SETTERS.iter().any(|(n, api)| {
    *n == name
      && ["\\lx@add@editor", "\\lx@add@creator"]
        .iter()
        .any(|c| api.starts_with(c))
  })
}

/// A store read at `\maketitle` rather than as it is set.
fn read_at_maketitle(name: &str) -> bool { annotates_a_creator(name) || creates_a_creator(name) }

/// A setter of one argument: a macro storing `#1`, or an `[optional]{mandatory}` one storing `#2` (bfhlayout.sty:738
/// `\providecommand*{\institute}[2][]{\def\@institute{#2}}`) — its body, the stored argument's number and the
/// control sequences its body stores that argument in ([`pure_stores`]).
fn setter_stores(cs: &Token) -> Result<Option<(Tokens, usize, Vec<Token>)>> {
  let Some(defn) = lookup_definition(cs)? else {
    return Ok(None);
  };
  let stored_arg = defn.get_parameters().and_then(|p| {
    let params = p.get_parameters();
    match params.as_slice() {
      [only] if !only.optional && undelimited(only) => Some(1),
      [first, second] if first.optional && !second.optional && undelimited(second) => Some(2),
      _ => None,
    }
  });
  let (Some(ExpansionBody::Tokens(body)), Some(arg)) = (defn.get_expansion(), stored_arg) else {
    return Ok(None);
  };
  Ok(pure_stores(body.unlist_ref(), arg).map(|stores| (body.clone(), arg, stores)))
}

/// The title-page code of the raw class — what its refused `\maketitle` (`\lx@dropped@maketitle`, state.rs) and
/// its `\@maketitle` run, and every macro a raw file defined that only they call, transitively — and the readers
/// of each macro: for every macro a raw file defined (`DefinitionOrigin::File`: the class and what it loaded, as
/// the class ends; the scan is bounded to them), the control sequences it reads ([`reads_of`]). A macro two
/// title-page macros call is title-page code; one any other macro calls too is not.
struct ClassReaders {
  /// The title-page code, by name.
  title_page:        FxHashSet<SymStr>,
  /// The title-page code with the class's own kernel fields among its roots (`\@title`, `\@author`, `\@date`, which
  /// LaTeX's `\@maketitle` prints): [`survey_off_table_stores`].
  title_page_fields: FxHashSet<SymStr>,
  /// What the class prints: every macro a raw file defined that the title-page roots — of any origin, a binding's
  /// `\@maketitle` too (titling_sty.rs's, which `\lx@deposit@maketitle` runs) — or the class's page styles (`\ps@…`,
  /// `\@oddhead`, …) reach, and what those roots read themselves.
  printed:           FxHashSet<SymStr>,
  /// For each control sequence, the macros a raw file defined that read it (itself, when it reads itself).
  readers:           HashMap<SymStr, Vec<SymStr>>,
}

/// The definers whose next control sequence is written, not read.
const WRITERS: &[&str] = &[
  "\\def",
  "\\gdef",
  "\\edef",
  "\\xdef",
  "\\protected@xdef",
  "\\protected@edef",
  "\\let",
  "\\g@addto@macro",
  "\\newcommand",
  "\\renewcommand",
  "\\providecommand",
  "\\DeclareRobustCommand",
];

/// The control sequences `body` reads: each one not written by a definer before it ([`WRITERS`]), and those named
/// by characters — `\@ifundefined{@affil}` (apa6.cls:793), `\@nameuse{X}`, `\csname X\endcsname` — as `\X`.
fn reads_of(body: &[Token]) -> Vec<Token> { reads_of_branches(body, false) }

/// The constant switch `t` is, with the branch it takes: an `\ifX` whose meaning is `\iftrue` or `\iffalse`, as
/// `\newif` and kvoptions make them, `\iftrue`/`\iffalse` themselves included.
fn constant_switch(t: &Token) -> Option<bool> {
  match lookup_meaning(t) {
    Some(Stored::Conditional(defn)) if defn.cs == T_CS!("\\iftrue") => Some(true),
    Some(Stored::Conditional(defn)) if defn.cs == T_CS!("\\iffalse") => Some(false),
    _ => None,
  }
}

/// Is `t` a conditional — one that opens a `\fi` level, not `\else`, `\or`, `\fi` or the prefix `\unless`?
fn opens_a_conditional(t: &Token) -> bool {
  match lookup_meaning(t) {
    Some(Stored::Conditional(defn)) => !["\\fi", "\\else", "\\or", "\\unless"]
      .into_iter()
      .any(|cs| defn.cs == T_CS!(cs)),
    _ => false,
  }
}

/// The branch an `\if` or `\ifx` at `body[at]` takes when its operands are known without running code, and the index
/// past them — the fallback of the speculative title-page run (`before_title_page_runs`), which lets TeX decide: `\ifx`
/// of two tokens that are the same, both undefined, or both parameterless macros of the same body; `\if` of the two
/// character tokens that the tokens after it give when each parameterless macro a raw file defined is expanded (tex.web
/// §506) — a class's home-made boolean, sigplanconf.cls:87-90 `\def\@true{TT}`/`\def\@false{FL}`
/// and :1118 `\if \@copyrightwanted \@copyrightspace \fi` (1603.03011). `None` for anything else.
fn evaluated_switch(body: &[Token], at: usize) -> Option<(bool, usize)> {
  let Some(Stored::Conditional(defn)) = lookup_meaning(body.get(at)?) else {
    return None;
  };
  let parameterless = |t: &Token| -> Option<Option<Vec<Token>>> {
    match lookup_meaning(t) {
      None => Some(None),
      Some(Stored::Expandable(d))
        if d
          .get_parameters()
          .is_none_or(|p| p.get_parameters().is_empty()) =>
      {
        match d.get_expansion() {
          None => Some(Some(Vec::new())),
          Some(ExpansionBody::Tokens(b)) => Some(Some(b.unlist_ref().to_vec())),
          Some(_) => None,
        }
      },
      _ => None,
    }
  };
  if defn.cs == T_CS!("\\ifx") {
    let (a, b) = (body.get(at + 1)?, body.get(at + 2)?);
    if a == b {
      return Some((true, at + 3));
    }
    if a.get_catcode() != Catcode::CS || b.get_catcode() != Catcode::CS {
      return None;
    }
    return Some((parameterless(a)? == parameterless(b)?, at + 3));
  }
  if defn.cs != T_CS!("\\if") {
    return None;
  }
  let is_char = |t: &Token| matches!(t.get_catcode(), Catcode::LETTER | Catcode::OTHER);
  let mut pending: VecDeque<Token> = VecDeque::new();
  let mut operands: Vec<Token> = Vec::new();
  let mut next = at + 1;
  // (a bounded number of expansions: `\def\x{\x}` never gives an operand)
  let mut expansions = 0;
  while operands.len() < 2 {
    let t = match pending.pop_front() {
      Some(t) => t,
      None => {
        let t = *body.get(next)?;
        next += 1;
        t
      },
    };
    if is_char(&t) {
      operands.push(t);
      continue;
    }
    expansions += 1;
    if t.get_catcode() != Catcode::CS || file_body(&t).is_none() || expansions > 32 {
      return None;
    }
    for t in parameterless(&t)??.into_iter().rev() {
      pending.push_front(t);
    }
  }
  // (what an expansion leaves after the two operands would be typeset in the branch: not known here)
  pending.is_empty().then(|| {
    (
      operands[0].with_str(|a| operands[1].with_str(|b| a == b)),
      next,
    )
  })
}

/// Does `t` write the control sequence after it — a definer of [`WRITERS`], or `\newif` — rather than read it?
fn writes_next(t: &Token) -> bool { cs_in(t, WRITERS) || *t == T_CS!("\\newif") }

/// The positions of `body` that are data rather than code: the name a definer writes ([`writes_next`]), and the
/// source of a `\let` (`\let\ifX\iftrue`, `\let\a=\b`). A conditional there opens no level.
fn data_positions(body: &[Token]) -> Vec<bool> {
  let mut data = vec![false; body.len()];
  let mut i = 0;
  while i < body.len() {
    let t = body[i];
    i += 1;
    if !matches!(t.get_catcode(), Catcode::CS | Catcode::ACTIVE) || !writes_next(&t) {
      continue;
    }
    let is_let = t == T_CS!("\\let") || t == T_CS!("\\futurelet");
    // (`\renewcommand{\cs}`: the brace before the written name)
    let mut written = 0;
    while i < body.len() && written < if is_let { 2 } else { 1 } {
      let next = body[i];
      let skip = matches!(next.get_catcode(), Catcode::BEGIN | Catcode::SPACE)
        || (is_let
          && written == 1
          && next.get_catcode() == Catcode::OTHER
          && next.with_str(|s| s == "="));
      if !skip {
        data[i] = true;
        written += 1;
      }
      i += 1;
    }
  }
  data
}

/// [`reads_of`]; with `live`, a branch a constant switch does not take is not read ([`constant_switch`], `\unless`
/// inverting it), and a conditional written as data opens no level ([`data_positions`]): the title page a copy's
/// switches leave ([`copy_title_page_prints`]).
fn reads_of_branches(body: &[Token], live: bool) -> Vec<Token> {
  let named = |from: usize, stop: &dyn Fn(&Token) -> bool| -> Option<(Token, usize)> {
    let mut name = String::new();
    let mut k = from;
    while let Some(t) = body.get(k) {
      if stop(t) {
        return Some((T_CS!(&s!("\\{name}")), k + 1));
      }
      if !matches!(t.get_catcode(), Catcode::LETTER | Catcode::OTHER) {
        return None;
      }
      t.with_str(|s| name.push_str(s));
      k += 1;
    }
    None
  };
  let data = if live {
    data_positions(body)
  } else {
    Vec::new()
  };
  let mut out = Vec::new();
  let mut written = false;
  let mut i = 0;
  while i < body.len() {
    let t = body[i];
    i += 1;
    if !matches!(t.get_catcode(), Catcode::CS | Catcode::ACTIVE) {
      // (`\renewcommand{\cs}`: the brace before the written name)
      written &= t.get_catcode() == Catcode::BEGIN || t.get_catcode() == Catcode::SPACE;
      continue;
    }
    if written {
      written = false;
      continue;
    }
    if cs_in(&t, WRITERS) {
      written = true;
      continue;
    }
    if live && !data[i - 1] {
      // (`\unless\ifX`: the switch after it, inverted)
      let (switch_at, inverted) = if t == T_CS!("\\unless") {
        (i, true)
      } else {
        (i - 1, false)
      };
      let switch = body
        .get(switch_at)
        .filter(|_| !data[switch_at])
        .and_then(|t| {
          constant_switch(t)
            .map(|taken| (taken, switch_at + 1))
            .or_else(|| evaluated_switch(body, switch_at))
        });
      if let Some((taken, start)) = switch {
        // the switch's `\else` and `\fi`, at its own depth
        let (mut depth, mut else_at, mut fi_at) = (0usize, None, None);
        for (k, token) in body.iter().enumerate().skip(start) {
          if data[k] || !matches!(token.get_catcode(), Catcode::CS | Catcode::ACTIVE) {
            continue;
          }
          match lookup_meaning(token) {
            Some(Stored::Conditional(defn)) if defn.cs == T_CS!("\\fi") => {
              if depth == 0 {
                fi_at = Some(k);
                break;
              }
              depth -= 1;
            },
            Some(Stored::Conditional(defn)) if defn.cs == T_CS!("\\else") => {
              if depth == 0 && else_at.is_none() {
                else_at = Some(k);
              }
            },
            _ if opens_a_conditional(token) => depth += 1,
            _ => {},
          }
        }
        if let Some(fi_at) = fi_at {
          let branch = if taken != inverted {
            &body[start..else_at.unwrap_or(fi_at)]
          } else {
            else_at.map_or(&body[fi_at..fi_at], |at| &body[at + 1..fi_at])
          };
          out.extend(reads_of_branches(branch, live));
          i = fi_at + 1;
          continue;
        }
      }
    }
    if (t == T_CS!("\\@ifundefined") || t == T_CS!("\\@nameuse"))
      && body
        .get(i)
        .is_some_and(|b| b.get_catcode() == Catcode::BEGIN)
      && let Some((cs, past)) = named(i + 1, &|b| b.get_catcode() == Catcode::END)
    {
      out.push(t);
      out.push(cs);
      i = past;
      continue;
    }
    if t == T_CS!("\\csname")
      && let Some((cs, past)) = named(i, &|b| *b == T_CS!("\\endcsname"))
    {
      out.push(cs);
      i = past;
      continue;
    }
    out.push(t);
  }
  out
}

/// Is `name` a page style's or a running head's (`\ps@…`, `\@oddhead`, …), which prints as the title page does?
fn page_style_name(name: &str) -> bool {
  name.starts_with("\\ps@")
    || matches!(
      name,
      "\\@oddhead" | "\\@evenhead" | "\\@oddfoot" | "\\@evenfoot"
    )
}

/// Does a stand-in's copy print the store `store` (`\@author`, `\@date`, `\@title`) — on its own title page (the
/// `\maketitle`/`\@maketitle` it defined, `state::copy_provides_title_pages`) or in a page style a raw file defined, and
/// every macro they reach that the copy (its own version of a name the binding owns, `copy_provides_overlaid_meaning`)
/// or another raw file defined, read as `ClassReaders` reads a raw class (`reads_of`: `\csname…\endcsname`,
/// `\@nameuse`), under the switches it set (a branch of a `\newif`/kvoptions flag that is now false is not printed,
/// `reads_of_branches`)? A title-page name it reaches with neither a copy's nor a raw file's meaning is the
/// binding's, which prints the frontmatter: the copy's `\renewcommand\maketitle{\begingroup\@maketitle\endgroup}`.
/// `None` when no copy defined a title page (the binding's prints the frontmatter). atlasdoc.cls (2019, 1902.05892)
/// prints `\@date` only in its `\ATLAS@TitleHeader` (:567, under `\ifADOC@datetop`), which its `\maketitle` (:716)
/// calls only without `cernpreprint`; `\@author` under the title.
pub fn copy_title_page_prints(store: &Token) -> Option<bool> {
  let roots = copy_provides_title_pages();
  let id = roots.first()?.0;
  let target = store.get_cs_name();
  let from_a_file = |stored: &Stored| match stored {
    Stored::Expandable(defn) => defn.get_origin() == DefinitionOrigin::File,
    _ => false,
  };
  let mut work: Vec<(u32, Tokens)> = roots;
  for (key, stored) in current_meanings_where(from_a_file) {
    if let Stored::Expandable(defn) = stored
      && let Some(ExpansionBody::Tokens(body)) = defn.get_expansion()
      && with(key, page_style_name)
    {
      work.push((id, body.clone()));
    }
  }
  let mut seen: FxHashSet<SymStr> = FxHashSet::default();
  while let Some((id, body)) = work.pop() {
    for cs in reads_of_branches(body.unlist_ref(), true) {
      let key = cs.get_cs_name();
      if key == target {
        return Some(true);
      }
      if !seen.insert(key) {
        continue;
      }
      let meaning =
        copy_provides_overlaid_meaning(id, &cs).or_else(|| lookup_meaning(&cs).filter(from_a_file));
      match meaning {
        Some(Stored::Expandable(defn)) => {
          if let Some(ExpansionBody::Tokens(callee)) = defn.get_expansion() {
            work.push((id, callee.clone()));
          }
        },
        None if cs == T_CS!("\\maketitle") || cs == T_CS!("\\@maketitle") => return Some(true),
        _ => {},
      }
    }
  }
  Some(false)
}

/// The label a class's title page prints right before a store, as read off its code ([`ClassReaders::label_in`]).
struct StoreLabel {
  /// The title-page macro it is in.
  reader:      String,
  /// The tokens the label takes up there (balanced; the store itself with them when the label is in the store's group).
  part:        Tokens,
  /// The label's text ([`LabelText`]).
  text:        Tokens,
  /// Is it in the store's own group, which is the argument of a control sequence (not a font command's)?
  in_argument: bool,
}

impl ClassReaders {
  /// One pass over the raw files' macros builds the caller index; the title page then grows from its roots along the
  /// callees of each macro it takes in, a macro joining once every other caller of it has (a worklist: each macro's
  /// callees are visited once, when it joins).
  fn survey() -> Self {
    let mut readers: HashMap<SymStr, Vec<SymStr>> = HashMap::default();
    let mut callees: HashMap<SymStr, Vec<SymStr>> = HashMap::default();
    let from_a_file = |stored: &Stored| match stored {
      Stored::Expandable(defn) => defn.get_origin() == DefinitionOrigin::File,
      _ => false,
    };
    for (key, stored) in current_meanings_where(from_a_file) {
      let Stored::Expandable(defn) = stored else {
        continue;
      };
      let Some(ExpansionBody::Tokens(body)) = defn.get_expansion() else {
        continue;
      };
      // (the meaning table's key is the name: a control sequence's when it starts with a backslash)
      if !with(key, |k| k.starts_with('\\')) {
        continue;
      }
      let caller = key;
      let body = body.unlist_ref();
      let mut seen = FxHashSet::default();
      let called = callees.entry(caller).or_default();
      for read in reads_of(body) {
        let callee = read.get_cs_name();
        if seen.insert(callee) {
          readers.entry(callee).or_default().push(caller);
          called.push(callee);
        }
      }
    }
    const ROOTS: [&str; 3] = [
      "\\lx@dropped@maketitle",
      "\\@maketitle",
      "\\lx@dropped@@maketitle",
    ];
    let mut printed: FxHashSet<SymStr> = FxHashSet::default();
    let mut reached: Vec<SymStr> = Vec::new();
    for root in ROOTS {
      let root = T_CS!(root);
      let name = root.get_cs_name();
      if callees.contains_key(&name) {
        reached.push(name);
      } else if let Some(Stored::Expandable(defn)) = lookup_meaning(&root)
        && let Some(ExpansionBody::Tokens(body)) = defn.get_expansion()
      {
        for read in reads_of(body.unlist_ref()) {
          reached.push(read.get_cs_name());
        }
      }
    }
    // The page styles print too: the running heads and feet a class defines (gGAF2e.cls:1085-1099 `\ps@plain`'s head
    // prints its `\@jname` default, "Geophysical and Astrophysical Fluid Dynamics": 2304.14584, 2512.18461).
    reached.extend(callees.keys().copied().filter(|cs| {
      with(*cs, |name| {
        name.starts_with("\\ps@")
          || matches!(
            name,
            "\\@oddhead" | "\\@evenhead" | "\\@oddfoot" | "\\@evenfoot"
          )
      })
    }));
    while let Some(cs) = reached.pop() {
      if printed.insert(cs) {
        reached.extend(callees.get(&cs).into_iter().flatten().copied());
      }
    }
    let grow = |roots: &[&str]| -> FxHashSet<SymStr> {
      let mut title_page: FxHashSet<SymStr> = FxHashSet::default();
      let mut joined: Vec<SymStr> = roots
        .iter()
        .map(|root| T_CS!(*root).get_cs_name())
        .filter(|root| callees.contains_key(root) && title_page.insert(*root))
        .collect();
      while let Some(cs) = joined.pop() {
        for callee in callees.get(&cs).into_iter().flatten().copied() {
          if title_page.contains(&callee) || !callees.contains_key(&callee) {
            continue;
          }
          let mut callers = readers
            .get(&callee)
            .into_iter()
            .flatten()
            .filter(|caller| **caller != callee)
            .peekable();
          if callers.peek().is_some() && callers.all(|caller| title_page.contains(caller)) {
            title_page.insert(callee);
            joined.push(callee);
          }
        }
      }
      title_page
    };
    let title_page = grow(&ROOTS);
    let mut with_fields: Vec<&str> = ROOTS.to_vec();
    with_fields.extend(KERNEL_FIELDS);
    let title_page_fields = grow(&with_fields);
    ClassReaders {
      title_page,
      title_page_fields,
      printed,
      readers,
    }
  }

  /// [`ClassReaders::read_only_by_the_title_page`], the class's own kernel fields counted as title-page code.
  fn read_only_by_the_title_page_or_fields(&self, store: &Token) -> bool {
    self
      .readers
      .get(&store.get_cs_name())
      .is_some_and(|readers| {
        !readers.is_empty() && readers.iter().all(|cs| self.title_page_fields.contains(cs))
      })
  }

  /// Is `store` read by the title-page code and by nothing else a raw file defined?
  fn read_only_by_the_title_page(&self, store: &Token) -> bool {
    self
      .readers
      .get(&store.get_cs_name())
      .is_some_and(|readers| {
        !readers.is_empty() && readers.iter().all(|cs| self.title_page.contains(cs))
      })
  }

  /// The title-page macros that read `store`, with their bodies.
  fn title_page_readers(&self, store: &Token) -> Vec<Tokens> {
    let Some(readers) = self.readers.get(&store.get_cs_name()) else {
      return Vec::new();
    };
    readers
      .iter()
      .filter(|cs| self.title_page.contains(*cs))
      .filter_map(
        |cs| match lookup_meaning(&T_CS!(&with(*cs, |name| name.to_string()))) {
          Some(Stored::Expandable(defn)) => match defn.get_expansion() {
            Some(ExpansionBody::Tokens(body)) => Some(body.clone()),
            _ => None,
          },
          _ => None,
        },
      )
      .collect()
  }

  /// The label the title page prints right before `store`, in the same line: in the first title-page macro reading it
  /// (by name) where the code from the line's start — the macro's start, a `\par`, a `\\`/`\newline` (with its `*`
  /// and `[<skip>]`), another parameterless value of a raw file's, or the open brace of the store's group — up to the
  /// store is label material and layout only ([`label_only_begin_code`]): `Resumen: \@resumen`, `\par KW: \@keywords`.
  /// When the store's own group holds only layout before it (`{\itshape \@abst}`), the line goes on before that group.
  /// Static, from the code: the label is the class's only when TeX's run of the title page prints its text right before
  /// the store (`verify_title_page_labels`).
  fn title_page_label(&self, store: &Token) -> Option<StoreLabel> {
    self.label_in(store, &self.title_page)
  }

  /// [`ClassReaders::title_page_label`] in the title-page code `code`: the class's own kernel fields counted
  /// ([`ClassReaders::title_page_fields`]) for an off-table store (jfpc.cls:176-177 `{\footnotesize\it\sffamily Actes
  /// JFPC \@lannee}` in its `\def\@title`, 1503.05530).
  fn label_in(&self, store: &Token, code: &FxHashSet<SymStr>) -> Option<StoreLabel> {
    let mut readers: Vec<String> = self
      .readers
      .get(&store.get_cs_name())?
      .iter()
      .filter(|cs| code.contains(*cs))
      .map(|cs| with(*cs, ToString::to_string))
      .collect();
    readers.sort();
    for reader in readers {
      let Some(body) = parameterless_body(&T_CS!(&reader)) else {
        continue;
      };
      let body = body.unlist_ref();
      for (at, t) in body.iter().enumerate() {
        if t != store
          || body[..at]
            .iter()
            .rev()
            .find(|b| b.get_catcode() != Catcode::SPACE)
            .is_some_and(|b| cs_in(b, WRITERS))
        {
          continue;
        }
        if let Some((start, end, text, in_argument)) = line_label_before(body, at)
          && balanced(&body[start..end])
        {
          return Some(StoreLabel {
            reader,
            part: Tokens::new(body[start..end].to_vec()),
            text,
            in_argument,
          });
        }
      }
    }
    None
  }

  /// Does the class print `store` — its title page or a page style ([`ClassReaders::printed`])? A store left at the
  /// class's default is printed only there, if at all: tlp.cls:697-728's `\@volume`, `\@doi`, `\@journal`
  /// placeholders ("TLP 1 (1): 1–000, January 20XX") are read by its `\@journal` running head, which the class's own
  /// `\@j@urnal` switch (:760-764) does not print without its `prodtf` option (1209.2617); lion-msc.cls:200's
  /// `\@affiliation` default is printed by its `\posttitle` (:244-275), which titling's `\@maketitle` reads; gGAF2e's
  /// `\@jname` by its `\ps@plain` head.
  fn read_by_the_title_page(&self, store: &Token) -> bool {
    let name = store.get_cs_name();
    self.printed.contains(&name)
      || self
        .readers
        .get(&name)
        .is_some_and(|readers| readers.iter().any(|cs| self.printed.contains(cs)))
  }

  /// How the title page prints `store` among the class's printed text, read off its code: is an occurrence glued to
  /// the class's text ([`glued_to_class_text`]), and which other values share its lines — in the innermost group
  /// around the store, between the line breaks around it ([`ends_a_line`]: `\par`, `\\`, `\newline`, `\break`,
  /// `\linebreak`, `\vskip`), each other `\@` macro not one of `carried`. A store printed in one line with a value
  /// the frontmatter does not carry is part of the class's line and stays with it: tlp.cls:627 `{\affilsize\it
  /// submitted \@submitted; revised \@revised; accepted \@accepted}`, where `\submitted` is no title-page setter of
  /// the table — handing on the other two left "submitted 30 November 2010; revised ; accepted" in the body
  /// (1209.2617); uiucthesis.cls:177 `for the degree of \@degree\ in \@dept\\`. Only printed text is a value (user
  /// ruling 2026-10-22, OXIDIZED_DESIGN_DIVERGENCES #662), decided by expansion ([`prints_as_a_value`]): a macro whose
  /// expansion is font, size or shape switches, spacing, rules or nothing ([`prints_text`]) is the line's
  /// typesetting, as in `{\@addrfont\@address\par}` with `\def\@addrfont{\itshape}` (review 68x r2 probe ka;
  /// bgteubner, umich-thesis, philosophersimprint). The other values are judged twice: as the class leaves them, when
  /// it is loaded (`reroute_raw_class_stores`), and with the document's, when `\maketitle` prints the title page
  /// (`lx_store_line_values_<name>`, [`harvest_stores`]): a store the class leaves empty, `\gdef\@submitted{}`, prints
  /// the document's `\submitted{1 May 2026}` there, which makes `{\itshape submitted \@submitted; e-mail
  /// \@email\par}` one line of the class's (review rev79a Q1). A value the line ends before is in another line:
  /// `\vbox{…\@jinfo\par\@author\par\@address\par}` (probe kb).
  fn store_lines(&self, store: &Token, carried: &dyn Fn(&Token) -> bool) -> StoreLines {
    let other = |t: &Token| {
      t != store
        && t.get_catcode() == Catcode::CS
        && t.with_str(|s| s.starts_with("\\@"))
        && !carried(t)
    };
    let mut lines = StoreLines::default();
    for body in self.title_page_readers(store) {
      let body = body.unlist_ref();
      let furniture = running_head_arguments(body);
      let mut open: Vec<usize> = Vec::new();
      for (k, t) in body.iter().enumerate() {
        // (a store a running head prints is in no line of the title page: preprint.cls's `\maketitle`
        // `\ihead{\@shortauthor:~\@shorttitle}`, 2512.04983, whose short title is the frontmatter's `<toctitle>`)
        if furniture.iter().any(|(from, to)| (*from..*to).contains(&k)) {
          continue;
        }
        // (a store written, ptptex.cls:110 `\gdef\@abst{}`, is not printed there)
        let written = || {
          body[..k]
            .iter()
            .rev()
            .find(|b| b.get_catcode() != Catcode::SPACE)
            .is_some_and(|b| cs_in(b, WRITERS))
        };
        match t.get_catcode() {
          Catcode::BEGIN => open.push(k),
          Catcode::END => {
            open.pop();
          },
          _ if t == store && !written() => {
            if glued_to_class_text(body, k, open.last().copied()) {
              lines.glued = true;
            }
            if let Some(&start) = open.last()
              && let Some(end) = group_end(body, start)
            {
              let from = body[start..k]
                .iter()
                .rposition(ends_a_line)
                .map_or(start, |b| start + b + 1);
              let to = body[k + 1..end]
                .iter()
                .position(ends_a_line)
                .map_or(end, |b| k + 1 + b);
              for value in body[from..to].iter().filter(|t| other(t)) {
                if !lines.others.contains(value) {
                  lines.others.push(*value);
                }
              }
            }
          },
          _ => {},
        }
      }
    }
    lines
  }
}

/// The spans of `body` that set page furniture — the arguments of LaTeX's marks (`\markboth`, `\markright`,
/// `\@mkboth`), of fancyhdr's head and foot commands (`\fancyhead[<places>]{…}`, `\lhead{…}`, …), of
/// scrlayer-scrpage's (`\ihead[<plain>]{…}`, `\ohead*{…}`, …) and of titleps's (`\sethead[…][…][…]{…}{…}{…}`,
/// `\setfoot*{…}{…}{…}`, titleps.sty:420-433), and the bodies of the head and foot macros a definition there sets
/// (`{\def\@oddhead{\@shortauthor: \@shorttitle\hfil}}`, `\renewcommand{\@evenfoot}{…}`; ptptex.cls's `\maketitle`
/// head, review rev79a2 finding 1): the running heads and feet LaTeXML drops, printed on no line of the title page
/// ([`ClassReaders::store_lines`]), and no text a store there is glued to ([`glued_to_class_text`]).
fn running_head_arguments(body: &[Token]) -> Vec<(usize, usize)> {
  const HEADS: &[(&str, usize)] = &[
    ("\\markboth", 2),
    ("\\markright", 1),
    ("\\@mkboth", 2),
    ("\\fancyhead", 1),
    ("\\fancyfoot", 1),
    ("\\fancyhf", 1),
    ("\\lhead", 1),
    ("\\chead", 1),
    ("\\rhead", 1),
    ("\\lfoot", 1),
    ("\\cfoot", 1),
    ("\\rfoot", 1),
    ("\\ihead", 1),
    ("\\ohead", 1),
    ("\\ifoot", 1),
    ("\\ofoot", 1),
    ("\\lehead", 1),
    ("\\cehead", 1),
    ("\\rehead", 1),
    ("\\lohead", 1),
    ("\\cohead", 1),
    ("\\rohead", 1),
    ("\\lefoot", 1),
    ("\\cefoot", 1),
    ("\\refoot", 1),
    ("\\lofoot", 1),
    ("\\cofoot", 1),
    ("\\rofoot", 1),
    ("\\sethead", 3),
    ("\\setfoot", 3),
  ];
  let mut spans = Vec::new();
  for (i, t) in body.iter().enumerate() {
    if let Some(span) = head_definition_body(body, i) {
      spans.push(span);
      continue;
    }
    let Some((_, nargs)) = HEADS.iter().find(|(cs, _)| cs_in(t, &[cs])) else {
      continue;
    };
    let mut k = skip_spaces(body, i + 1);
    if body.get(k) == Some(&T_OTHER!("*")) {
      k = skip_spaces(body, k + 1);
    }
    // The span starts before any optional arguments: titleps's three are the even-page heads, scrlayer-scrpage's
    // `[<plain>]` and fancyhdr's old `\lhead[<even>]` are printed heads too (fancyhdr's `[<places>]` hold no stores).
    let start = k;
    match skip_optional_arguments(body, k) {
      Some(past) => k = past,
      None => continue,
    }
    for _ in 0..*nargs {
      match group_end(body, skip_spaces(body, k)) {
        Some(past) => k = past,
        None => break,
      }
    }
    if k > start {
      spans.push((start, k));
    }
  }
  spans
}

/// The span of the body a definition at `body[i]` gives LaTeX's head or foot macro (`\@oddhead`, `\@evenhead`,
/// `\@oddfoot`, `\@evenfoot`): `\def`, `\gdef`, `\edef`, `\xdef` with any parameter text, `\renewcommand` with its
/// `*`, the name braced or not, and its `[<n>]` ([`running_head_arguments`]).
fn head_definition_body(body: &[Token], i: usize) -> Option<(usize, usize)> {
  const HEAD_MACROS: &[&str] = &["\\@oddhead", "\\@evenhead", "\\@oddfoot", "\\@evenfoot"];
  let t = body.get(i)?;
  let mut k = skip_spaces(body, i + 1);
  if cs_in(t, &["\\def", "\\gdef", "\\edef", "\\xdef"]) {
    if !cs_in(body.get(k)?, HEAD_MACROS) {
      return None;
    }
    // (the parameter text, up to the body's brace)
    k = k
      + 1
      + body[k + 1..]
        .iter()
        .position(|b| b.get_catcode() == Catcode::BEGIN)?;
  } else if cs_in(t, &["\\renewcommand"]) {
    if body.get(k) == Some(&T_OTHER!("*")) {
      k = skip_spaces(body, k + 1);
    }
    if cs_in(body.get(k)?, HEAD_MACROS) {
      k += 1;
    } else if body.get(k)?.get_catcode() == Catcode::BEGIN
      && body.get(k + 1).is_some_and(|name| cs_in(name, HEAD_MACROS))
      && body
        .get(k + 2)
        .is_some_and(|b| b.get_catcode() == Catcode::END)
    {
      k += 3;
    } else {
      return None;
    }
    k = skip_optional_arguments(body, skip_spaces(body, k))?;
  } else {
    return None;
  }
  Some((k, group_end(body, k)?))
}

/// The index past the optional arguments (`[…]`, each with its braced groups whole) from `body[k]` on, spaces
/// skipped; `None` when one never closes.
fn skip_optional_arguments(body: &[Token], mut k: usize) -> Option<usize> {
  while body.get(k) == Some(&T_OTHER!("[")) {
    let mut depth = 0usize;
    let close = body[k + 1..].iter().position(|b| {
      match b.get_catcode() {
        Catcode::BEGIN => depth += 1,
        Catcode::END => depth = depth.saturating_sub(1),
        _ => {},
      }
      depth == 0 && *b == T_OTHER!("]")
    })?;
    k = skip_spaces(body, k + 1 + close + 1);
  }
  Some(k)
}

/// How the title page prints a store among the class's text ([`ClassReaders::store_lines`]).
#[derive(Default)]
struct StoreLines {
  /// Is an occurrence glued to the class's printed text ([`glued_to_class_text`])?
  glued:  bool,
  /// The other `\@` macros in its lines the frontmatter does not carry, values when they print text
  /// ([`prints_as_a_value`]).
  others: Vec<Token>,
}

/// Is the macro `t` a value of a class's line: a parameterless macro of a raw file's or the document's whose expansion
/// prints text ([`prints_text`]), read with its current meaning — the class's default when the class is loaded, the
/// document's at `\maketitle`?
fn prints_as_a_value(t: &Token) -> bool {
  match lookup_meaning(t) {
    Some(Stored::Expandable(defn)) => {
      matches!(
        defn.get_origin(),
        DefinitionOrigin::File | DefinitionOrigin::Document
      ) && defn
        .get_parameters()
        .is_none_or(|p| p.get_parameters().is_empty())
        && matches!(defn.get_expansion(), Some(ExpansionBody::Tokens(value))
          if prints_text(value.unlist_ref(), PRINTS_TEXT_DEPTH))
    },
    _ => false,
  }
}

/// Can the macro `t`, a value of a store's line ([`ClassReaders::store_lines`]) that prints nothing as the class leaves
/// it, print one when `\maketitle` judges it again ([`prints_as_a_value`])? Only a placeholder the document can fill:
/// undefined, `\relax` (which LaTeX finds undefined, `\@ifundefined`, `\newcommand`: `\let\@volume\relax`), or a
/// parameterless macro that expands to nothing (`\gdef\@submitted{}`, `\let\@shortauthor\empty`; review rev79a
/// Q1) — a store the class never defaulted still waits for `\maketitle`. The line's typesetting code is no
/// placeholder: a macro with parameters (`\@setfontsize`, `\@ifundefined`), a primitive, a register, or a font
/// macro (USG.cls:1590 `\smash{\@journalfont\@journal}`, whose `\journal` deferred for it, 2605.00042's pubnote
/// order; review rev79a2 finding 5).
fn may_become_a_value(t: &Token) -> bool {
  match lookup_meaning(t) {
    None => true,
    Some(Stored::Expandable(defn)) => {
      defn
        .get_parameters()
        .is_none_or(|p| p.get_parameters().is_empty())
        && match defn.get_expansion() {
          // (an empty body is none)
          None => true,
          Some(ExpansionBody::Tokens(value)) => value.unlist_ref().iter().all(|b| {
            matches!(
              b.get_catcode(),
              Catcode::SPACE | Catcode::BEGIN | Catcode::END
            ) || cs_in(b, &["\\relax", "\\@empty", "\\empty"])
          }),
          _ => false,
        }
    },
    meaning => meaning == lookup_meaning(&TOKEN_RELAX),
  }
}

/// Is the occurrence of a store at `body[k]` glued to the class's printed text — the character token right before or
/// after it a letter or a digit, or a `.`, `/` or `-` inside a word (with a letter, a digit or another of them beyond
/// it), with no space, glue or line break between? The class's text and the store's then print as one word, which is
/// one value the frontmatter does not carry, and the line stays the class's: piparticle-final.cls:272 `{DOI:
/// 10.4279/PIP.\@doi}`, handed on, left "020001" as the paper's DOI (1002.3829, 1012.4507, 1503.03445, 1503.07722,
/// 1510.06950); ar-1col.cls:335 `https://doi.org/\@doi\endgraf` (1903.12215, 2004.12182, 2603.20393). Not glued: a
/// sentence's full stop after the store (`\enspace\@ACMCCS.}`, CSML.cls's shape, perfect_kernel_batch75a), a label's
/// `:` or a bracket (`doi:\@doi`, lms; `(\@pubyear`, mnras), an operand of `\if`, `\ifx` or `\ifcat` (`\if!\@subtitle`,
/// lipics-v2021, llncs), which TeX compares and does not print, and the first argument of `\href` (`open`, the
/// innermost group's brace: `\href{mailto:\@email}`, modernposter), a link's target.
fn glued_to_class_text(body: &[Token], k: usize, open: Option<usize>) -> bool {
  const CONDITIONALS: &[&str] = &["\\if", "\\ifx", "\\ifcat"];
  // (`Some(true)`: a letter or a digit; `Some(false)`: a `.`, `/` or `-`)
  let word_char = |t: Option<&Token>| -> Option<bool> {
    let t = t?;
    match t.get_catcode() {
      Catcode::LETTER => Some(true),
      Catcode::OTHER => t.with_str(|s| {
        let mut chars = s.chars();
        match (chars.next(), chars.next()) {
          (Some(c), None) if c.is_ascii_digit() => Some(true),
          (Some('.' | '/' | '-'), None) => Some(false),
          _ => None,
        }
      }),
      _ => None,
    }
  };
  let before = |j: usize| j.checked_sub(1).map(|i| &body[i]);
  let glued_before = match word_char(before(k)) {
    Some(true) => true,
    Some(false) => word_char(before(k - 1)).is_some(),
    None => false,
  } && !in_a_dimension(body, k);
  let glued_after = match word_char(body.get(k + 1)) {
    Some(true) => true,
    Some(false) => word_char(body.get(k + 2)).is_some(),
    None => false,
  };
  let operand = (k >= 1 && cs_in(&body[k - 1], CONDITIONALS))
    || (k >= 2 && cs_in(&body[k - 2], CONDITIONALS) && body[k - 1].get_catcode() != Catcode::SPACE);
  let link_target = open.is_some_and(|start| {
    body[..start]
      .iter()
      .rev()
      .find(|b| b.get_catcode() != Catcode::SPACE)
      .is_some_and(|b| *b == T_CS!("\\href"))
  });
  !operand && !link_target && (glued_before || glued_after)
}

/// Is `body[k]` in or right after the dimension a skip, kern, penalty or register assignment reads (`\hskip.5em\@kword`,
/// jps-cp.cls; `\vskip 4ex\@subtitle`, jurabook.cls; `\hangindent 3.8pc\@keywords`, w-art.cls)? Its characters are no
/// printed text ([`glue_end`]).
fn in_a_dimension(body: &[Token], k: usize) -> bool {
  const READS_A_NUMBER: &[&str] = &[
    "\\penalty",
    "\\raise",
    "\\lower",
    "\\moveleft",
    "\\moveright",
  ];
  let Some(j) = body[..k]
    .iter()
    .rposition(|t| t.get_catcode() == Catcode::CS)
  else {
    return false;
  };
  let t = &body[j];
  (cs_in(t, GLUE_SETTERS)
    || cs_in(t, READS_A_NUMBER)
    || matches!(lookup_meaning(t), Some(Stored::Register(_))))
    && glue_end(body, j + 1).is_some_and(|end| end >= k)
}

/// Does `t` end the line TeX prints: `\par` (a blank line in the code is one; `\endgraf` is plain's `\par`), `\\`,
/// `\newline`, `\break` and `\linebreak`, an alignment's `\cr`, and the primitive `\vskip`, which in horizontal mode
/// ends the paragraph?
fn ends_a_line(t: &Token) -> bool {
  cs_in(t, &[
    "\\par",
    "\\endgraf",
    "\\\\",
    "\\newline",
    "\\break",
    "\\linebreak",
    "\\cr",
    "\\crcr",
    "\\vskip",
  ])
}

/// How many macros deep [`prints_text`] reads a value's expansion; past it, a macro counts as printing.
const PRINTS_TEXT_DEPTH: usize = 6;

/// Spacing, rules and breaks of TeX's and LaTeX's that print no text, taking no argument (besides [`LAYOUT`]).
const SPACING: &[&str] = &[
  "\\hfil",
  "\\hfill",
  "\\hss",
  "\\vfil",
  "\\vfill",
  "\\vss",
  "\\quad",
  "\\qquad",
  "\\enskip",
  "\\enspace",
  "\\thinspace",
  "\\,",
  "\\ ",
  "\\/",
  "\\strut",
  "\\null",
  "\\break",
  "\\endgraf",
  "\\cr",
  "\\crcr",
  "\\protect",
];

/// Does `code` print text — a character, math, or a command not known to print nothing? What prints nothing is a
/// space, a group's braces, a font, size or shape switch (a command whose definition carries a font: LaTeX's
/// `\itshape`, `\large`, and [`LAYOUT`]), spacing ([`SPACING`], [`LAYOUT_ARGS`] with their arguments, [`GLUE_SETTERS`]
/// and a register with their glue), a rule (`\hrule`/`\vrule` with their `height`/`width`/`depth`, LaTeX's
/// `\rule[<raise>]{<width>}{<height>}`), a line break with its `*` and `[<skip>]`, and a parameterless macro whose
/// expansion prints nothing, read `depth` macros deep. Only printed text is a value of a class's line (user ruling
/// 2026-10-22; [`ClassReaders::store_lines`]).
fn prints_text(code: &[Token], depth: usize) -> bool {
  let mut k = 0;
  while k < code.len() {
    let t = code[k];
    k += 1;
    match t.get_catcode() {
      Catcode::SPACE | Catcode::BEGIN | Catcode::END => continue,
      Catcode::ACTIVE if t == T_ACTIVE!('~') => continue,
      Catcode::CS | Catcode::ACTIVE => {},
      _ => return true,
    }
    if cs_in(&t, &["\\\\", "\\newline", "\\linebreak"]) {
      // `\\*[<skip>]`, `\linebreak[<number>]`
      k = skip_spaces(code, k);
      if code.get(k) == Some(&T_OTHER!("*")) {
        k = skip_spaces(code, k + 1);
      }
      if code.get(k) == Some(&T_OTHER!("[")) {
        match code[k..].iter().position(|b| *b == T_OTHER!("]")) {
          Some(close) => k += close + 1,
          None => return true,
        }
      }
      continue;
    }
    if cs_in(&t, LAYOUT) || cs_in(&t, SPACING) {
      continue;
    }
    if let Some((_, nargs)) = LAYOUT_ARGS.iter().find(|(cs, _)| cs_in(&t, &[cs])) {
      k = skip_spaces(code, k);
      if code.get(k) == Some(&T_OTHER!("*")) {
        k += 1;
      }
      for _ in 0..*nargs {
        match group_end(code, skip_spaces(code, k)) {
          Some(past) => k = past,
          None => return true,
        }
      }
      continue;
    }
    if t == T_CS!("\\rule") {
      k = skip_spaces(code, k);
      if code.get(k) == Some(&T_OTHER!("[")) {
        match code[k..].iter().position(|b| *b == T_OTHER!("]")) {
          Some(close) => k += close + 1,
          None => return true,
        }
      }
      for _ in 0..2 {
        match group_end(code, skip_spaces(code, k)) {
          Some(past) => k = past,
          None => return true,
        }
      }
      continue;
    }
    if cs_in(&t, &["\\hrule", "\\vrule"]) {
      // `\hrule height 0.4pt depth 0pt width \textwidth`
      loop {
        let j = skip_spaces(code, k);
        let keyword = ["height", "width", "depth"].into_iter().find(|keyword| {
          keyword.chars().enumerate().all(|(n, c)| {
            code.get(j + n).is_some_and(|b| {
              b.get_catcode() == Catcode::LETTER && b.with_str(|s| s.len() == 1 && s.starts_with(c))
            })
          })
        });
        let Some(keyword) = keyword else {
          break;
        };
        match glue_end(code, j + keyword.len()) {
          Some(past) => k = past,
          None => return true,
        }
      }
      continue;
    }
    if cs_in(&t, GLUE_SETTERS) || matches!(lookup_meaning(&t), Some(Stored::Register(_))) {
      // (a skip or a register's assignment prints nothing: `\vskip 12\p@`, `\parindent=0pt`; a register read some
      // other way — a `\chardef`'d character, a count set to a number — counts as printing)
      match glue_end(code, k) {
        Some(past) => k = past,
        None => return true,
      }
      continue;
    }
    if let Ok(Some(defn)) = lookup_definition(&t)
      && defn.declared_font().is_some()
      && defn
        .get_parameters()
        .is_none_or(|p| p.get_parameters().is_empty())
    {
      continue;
    }
    match parameterless_body(&t) {
      Some(expansion) if depth > 0 && !prints_text(expansion.unlist_ref(), depth - 1) => continue,
      _ => return true,
    }
  }
  false
}

/// The class's setter `\name`, if it stores its argument for the title page ([`setter_stores`]): in its own
/// `\@name` (ptptex.cls:616 `\inst` → `\@inst`), or — rule (a) — in one other `\@X` that only the class's title-page
/// code reads ([`ClassReaders`]): apa6.cls:760 `\long\def\affiliation#1{\long\def\@affil{#1}}` and :779
/// `\authornote` → `\@acks`, read only by its `\maketitle` (:1126-1234, :1367-1466, :1571-1707) and the
/// `\check@author` it calls (:786-795). A store some other class macro reads stays raw: letter.cls:155 `\address`
/// fills `\fromaddress` for `\opening` (:221-237), afthesis.cls:535 `\addr@ss` its vita (:744-749) (sweep 93:
/// rerouting those left the readers without their value). A kernel-owned store (`\@date`) is not a class's.
/// Returns the body, the stored argument's number and the store.
fn store_setter_body(
  name: &str,
  readers: &mut Option<ClassReaders>,
) -> Result<Option<(Tokens, usize, Token)>> {
  let Some((body, arg, stores)) = setter_stores(&T_CS!(&s!("\\{name}")))? else {
    return Ok(None);
  };
  let own = T_CS!(&s!("\\@{name}"));
  if stores.contains(&own) {
    return Ok(Some((body, arg, own)));
  }
  let [store] = stores
    .iter()
    .filter(|t| t.with_str(|s| s.starts_with("\\@")))
    .copied()
    .collect::<Vec<_>>()[..]
  else {
    return Ok(None);
  };
  let class_owned = match lookup_meaning(&store) {
    None => true,
    Some(Stored::Expandable(defn)) => defn.get_origin() == DefinitionOrigin::File,
    Some(_) => false,
  };
  if !class_owned {
    return Ok(None);
  }
  let readers = readers.get_or_insert_with(ClassReaders::survey);
  Ok(
    readers
      .read_only_by_the_title_page(&store)
      .then_some((body, arg, store)),
  )
}

/// A kernel field setter the raw class redefined — tufte-common.def:608-614 `\renewcommand{\date}[1]{\gdef\@date{#1}…
/// \protected@xdef\thedate{#1}…}` (2603.01440; 25 tufte papers) — is still that field's setter (K11): when it stores
/// its argument in the field's store among side definitions only ([`pure_stores`]), the class's body stays and the
/// store is handed to the frontmatter as the kernel's `\date` hands it (sect05.rs), so the side definitions are kept.
/// The kernel's `\title` and `\author` are locked; `\date` is not, and is not locked for this.
fn reroute_kernel_field_setters(cls: &str) -> Result<()> {
  const FIELDS: &[(&str, &str)] = &[(
    "date",
    r"\expandafter\lx@add@date@halved\expandafter{\@date}\expandafter\lx@date@after\expandafter{\@date}",
  )];
  for (name, handing) in FIELDS {
    let cs = T_CS!(&s!("\\{name}"));
    let Some(defn) = lookup_definition(&cs)? else {
      continue;
    };
    if defn.get_origin() != DefinitionOrigin::File {
      continue;
    }
    let Some((body, 1, stores)) = setter_stores(&cs)? else {
      continue;
    };
    if !stores.contains(&T_CS!(&s!("\\@{name}"))) {
      continue;
    }
    let mut body = body.unlist();
    body.extend(mouth::tokenize_internal(*handing).unlist());
    DefMacro!(cs, convert_latex_args(1, None)?, Tokens::new(body));
    Info!(
      "frontmatter",
      cls,
      s!("The raw class's \\{name} keeps its definitions and sets the frontmatter's {name}")
    );
  }
  Ok(())
}

/// The Frontmatter API's contact setters (OmniBus.cls.ltxml:77-94, the generic class Perl gives a paper whose class has
/// no binding) and the API call each is when a raw file's own definition of it sets its argument as a title-page line
/// ([`in_place_contact_setter`]). An address or affiliation printed where it is called stands under the names printed
/// above it: the authors new since the last such entry (`annotate=run`, `frontmatter_api.rs`), or those its marks
/// request; an e-mail line goes where the API's evidence puts each of its addresses (the author it spells, …), a
/// list written out one address each, as a class's address list is (`\lx@split@addresses`, OD #805): 1303.3004's
/// `\email{rnarayan@…, jmcclintock@…, atchekho@…}` is one address of each of its three authors. `ARG` the line; an
/// `[optional]{mandatory}` setter's optional argument is read by [`optional_contact`].
const IN_PLACE_CONTACT_SETTERS: &[(&str, &str)] = &[
  ("address", "\\lx@add@address[annotate=run]{ARG}"),
  ("affil", "\\lx@add@affiliation[annotate=run]{ARG}"),
  ("affiliation", "\\lx@add@affiliation[annotate=run]{ARG}"),
  ("institute", "\\lx@add@affiliation[annotate=run]{ARG}"),
  ("email", "\\lx@split@addresses{\\lx@add@email}{ARG}"),
];

/// What ends a line or a paragraph, or starts a list item (`\begin{indented}\item[]`): a setter whose executed code has
/// one around its argument sets the argument as a line of its own. (Not an environment's `\begin`/`\end` as such: a
/// font environment, aptmod.cls:137 `\begin{footnotesize}Email address: #1\end{footnotesize}`, sets no line.)
const LINE_BREAKS: &[&str] = &[
  "\\par",
  "\\endgraf",
  "\\\\",
  "\\newline",
  "\\linebreak",
  "\\break",
  "\\vskip",
  "\\vspace",
  "\\smallskip",
  "\\medskip",
  "\\bigskip",
  "\\noindent",
  "\\item",
  "\\centerline",
  "\\leftline",
  "\\rightline",
  "\\cr",
  "\\crcr",
];

/// The control sequences `body` writes — the next control sequence after a definer ([`WRITERS`]), past the
/// `\expandafter`s of an append (`\expandafter\def\expandafter\@aabuffer\expandafter{\@aabuffer …}`) — each with
/// whether the text it writes holds argument `#<arg>` (the definer's next group, or `\g@addto@macro`'s second
/// argument).
fn written_stores(body: &[Token], arg: Option<usize>) -> Vec<(Token, bool)> {
  let holds_arg = |text: &[Token]| {
    arg.is_some_and(|arg| {
      text
        .iter()
        .any(|t| t.get_catcode() == Catcode::ARG && t.with_str(|s| s == arg.to_string()))
    })
  };
  let mut out = Vec::new();
  for (i, t) in body.iter().enumerate() {
    if !cs_in(t, WRITERS) || *t == T_CS!("\\let") {
      continue;
    }
    let mut k = skip_spaces(body, i + 1);
    while body.get(k) == Some(&T_CS!("\\expandafter")) {
      k = skip_spaces(body, k + 1);
    }
    // (`\newcommand{\x}`: the braced name)
    if body
      .get(k)
      .is_some_and(|b| b.get_catcode() == Catcode::BEGIN)
    {
      k = skip_spaces(body, k + 1);
    }
    let Some(target) = body.get(k).filter(|b| b.get_catcode() == Catcode::CS) else {
      continue;
    };
    let mut open = k + 1;
    while open < body.len() && body[open].get_catcode() != Catcode::BEGIN {
      open += 1;
    }
    let written = group_end(body, open).map_or(&[][..], |past| &body[open..past]);
    out.push((*target, holds_arg(written)));
  }
  out
}

/// The stores the class's own `\author` — the definition the kernel's locked `\author` refused, kept as
/// `\lx@dropped@author` (state.rs `install_definition`, sect08.rs's `\newcommand` bail) — writes, itself or through the
/// macros a raw file defined that it calls (ae100prg.cls:170 `\@ifnextchar[{\@sauthor}{\@fauthor}`).
fn class_author_stores() -> Vec<Token> {
  let dropped = T_CS!("\\lx@dropped@author");
  let mut stores = Vec::new();
  for name in live_read_order(&[dropped], &[], false) {
    let cs = with(name, |n| T_CS!(n));
    let body = match lookup_meaning(&cs) {
      Some(Stored::Expandable(defn))
        if cs == dropped || defn.get_origin() == DefinitionOrigin::File =>
      {
        match defn.get_expansion() {
          Some(ExpansionBody::Tokens(body)) => body.clone(),
          _ => continue,
        }
      },
      _ => continue,
    };
    stores.extend(
      written_stores(body.unlist_ref(), None)
        .into_iter()
        .map(|(store, _)| store),
    );
  }
  stores
}

/// The commands whose braced argument is printed where they stand — LaTeX's text faces and boxes, an e-mail link
/// (ae100prg.cls:185 `\address{E-mail: \mailto{#1}}`) — with the number of arguments before it they do not print (a
/// link's target, a parbox's width).
const PRINTED_ARGUMENTS: &[(&str, usize)] = &[
  ("\\textbf", 0),
  ("\\textit", 0),
  ("\\textsl", 0),
  ("\\textsc", 0),
  ("\\textup", 0),
  ("\\textmd", 0),
  ("\\textrm", 0),
  ("\\textsf", 0),
  ("\\texttt", 0),
  ("\\textnormal", 0),
  ("\\emph", 0),
  ("\\underline", 0),
  ("\\mbox", 0),
  ("\\fbox", 0),
  ("\\centerline", 0),
  ("\\leftline", 0),
  ("\\rightline", 0),
  ("\\mailto", 0),
  ("\\url", 0),
  ("\\href", 1),
  ("\\parbox", 1),
];

/// The primitives that read a braced text without printing it: the log, a write, a mark, a special.
const UNPRINTED_ARGUMENTS: &[&str] = &[
  "\\message",
  "\\errmessage",
  "\\write",
  "\\mark",
  "\\marks",
  "\\special",
  "\\showtokens",
  "\\detokenize",
  "\\scantokens",
];

/// The arguments the command `t` reads where it stands, each whether it is optional (`[…]`, read only when one
/// follows) and whether the command prints it: a face, a box or a link the argument it prints ([`PRINTED_ARGUMENTS`]); a
/// macro a raw file defined each argument its own code prints ([`prints_argument`], a few levels down: ae100prg.cls:185
/// `\email` passing `#1` to its `\address`); any other command's arguments, by its parameters (an undefined command one
/// braced argument), unprinted — it may store them (spr-sola-addons.sty:317 `\add@xtok\adresses@list{…#2…}`,
/// 2005.09586), make them a footnote (`\thanks{#1}`) or write them to the log ([`UNPRINTED_ARGUMENTS`]). `None` when it
/// takes none (a switch, a register, a conditional's `\fi`), and `Some` of nothing when its arguments are delimited,
/// which the token walk cannot follow.
fn command_arguments(t: &Token, depth: usize) -> Option<Vec<(bool, bool)>> {
  if let Some((_, unprinted)) = PRINTED_ARGUMENTS
    .iter()
    .find(|(cs, _)| t.with_str(|s| s == *cs))
  {
    let mut args = vec![(false, false); *unprinted];
    args.push((false, true));
    return Some(args);
  }
  if cs_in(t, UNPRINTED_ARGUMENTS) {
    return Some(vec![(false, false)]);
  }
  let meaning = lookup_meaning(t);
  let params = match &meaning {
    Some(Stored::Expandable(defn)) => defn.get_parameters(),
    Some(Stored::Primitive(defn)) => defn.get_parameters(),
    Some(Stored::Constructor(defn)) => defn.get_parameters(),
    None => return Some(vec![(false, false)]),
    _ => return None,
  }
  .map(|p| p.get_parameters().to_vec())
  .unwrap_or_default();
  if params.is_empty() {
    return None;
  }
  let raw_body = match &meaning {
    Some(Stored::Expandable(defn)) if defn.get_origin() == DefinitionOrigin::File && depth < 4 => {
      if params.iter().any(|p| !undelimited(p)) {
        return Some(Vec::new());
      }
      match defn.get_expansion() {
        Some(ExpansionBody::Tokens(body)) => Some(executed_code(body.unlist_ref())),
        _ => None,
      }
    },
    _ => None,
  };
  Some(
    params
      .iter()
      .enumerate()
      .map(|(k, p)| {
        let printed = raw_body
          .as_deref()
          .is_some_and(|code| prints_argument(code, k + 1, depth + 1));
        (p.optional, printed)
      })
      .collect(),
  )
}

/// Whether `code` (a body's executed code, [`executed_code`]) prints its argument `#<arg>` where it is called: the
/// argument stands at its top level or in a plain group (`{\reset@font\affilfont#1\par}`, ar-1col.cls:341; one after a
/// switch, `{\eaddfnt{\par #1}}`), or in an argument a command prints ([`command_arguments`]).
fn prints_argument(code: &[Token], arg: usize, depth: usize) -> bool {
  let is_arg = |t: &Token| t.get_catcode() == Catcode::ARG && t.with_str(|s| s == arg.to_string());
  // the index past one argument at `k` (a group or one token), spaces before it skipped
  let past_argument = |k: usize| -> usize {
    let k = skip_spaces(code, k);
    group_end(code, k).unwrap_or((k + 1).min(code.len()))
  };
  let mut i = 0;
  while i < code.len() {
    let t = code[i];
    if is_arg(&t) {
      return true;
    }
    if t.get_catcode() == Catcode::BEGIN {
      let end = group_end(code, i).unwrap_or(code.len());
      if prints_argument(&code[i + 1..end.saturating_sub(1).max(i + 1)], arg, depth) {
        return true;
      }
      i = end;
      continue;
    }
    if t.get_catcode() == Catcode::CS
      && let Some(arguments) = command_arguments(&t, depth)
    {
      if arguments.is_empty() {
        // (delimited arguments: what follows is the command's to read)
        return false;
      }
      let mut k = i + 1;
      for (optional, printed) in arguments {
        let at = skip_spaces(code, k);
        let end = if optional {
          if code.get(at) != Some(&T_OTHER!("[")) {
            continue;
          }
          code[at..]
            .iter()
            .position(|b| *b == T_OTHER!("]"))
            .map_or(code.len(), |p| at + p + 1)
        } else {
          past_argument(k)
        };
        if printed && prints_argument(&code[at.min(end)..end], arg, depth) {
          return true;
        }
        k = end;
      }
      i = k.max(i + 1);
      continue;
    }
    i += 1;
  }
  false
}

/// Whether the raw file's own `\<name>` is a contact setter of the title page, by its code: a macro of one argument
/// (or of an optional one and one, setting the second) that sets the argument as a line of its own where it is called
/// — its executed code ([`executed_code`]) prints the argument there ([`prints_argument`]) beside a line break
/// ([`LINE_BREAKS`]): ae100prg.cls:188
/// `\begin{indented}\item[]\normalsize #1\end{indented}`, :185 `\vspace*{5pt}\address{E-mail: …#1…}` (1303.3004),
/// ar-1col.cls:341 `\par…{\reset@font\affilfont#1\par}` (1602.05576, 2405.08158) — or that appends it to a store the
/// class's own `\author` writes too ([`class_author_stores`]), the author block its title page prints and the kernel's
/// `\author` carries: 11lomcon.sty:15-18 `\@aabuffer` (hep-ph0312288), jetpl.cls:286-303 `\@authoraddress`
/// (cond-mat0611585). Its argument number when it is one, and whether it prints the line where it is called (else it
/// stores it for the title page). Anything else stays the class's: a mark
/// (agujournal2019.cls `\def\affil#1{$^{#1}$\ignorespaces}`), an inline face of an address used inside a note or a line
/// (`\newcommand{\email}[1]{\texttt{#1}}`, `\href{mailto:#1}{#1}`), a footnote (ecai.cls `\institute` =
/// `\thanks{#1}`), a store only its own title page reads (K11's; interspeech.cls's `\gdef\@address{#1\\}`;
/// spr-sola-addons.sty:317's `\add@xtok\adresses@list{…}`, 2005.09586), a `\message` that prints nothing
/// (dmtcs-episciences.cls), a switch or a dispatcher without the argument (jmlr2e.sty `\def\email{\hfill\small\sc}`,
/// imsart.sty `\def\address{\@ifnextchar[…}`).
fn in_place_contact_setter(name: &str) -> Result<Option<(usize, bool)>> {
  let cs = T_CS!(&s!("\\{name}"));
  let Some(Stored::Expandable(defn)) = lookup_meaning(&cs) else {
    return Ok(None);
  };
  let arg = defn
    .get_parameters()
    .and_then(|p| match p.get_parameters().as_slice() {
      [only] if !only.optional && undelimited(only) => Some(1),
      [first, second] if first.optional && !second.optional && undelimited(second) => Some(2),
      _ => None,
    });
  let (Some(ExpansionBody::Tokens(body)), Some(arg)) = (defn.get_expansion(), arg) else {
    return Ok(None);
  };
  let body = body.unlist_ref();
  let executed = executed_code(body);
  let sets_a_line =
    prints_argument(&executed, arg, 0) && executed.iter().any(|t| cs_in(t, LINE_BREAKS));
  if sets_a_line {
    return Ok(Some((arg, true)));
  }
  // (not the kernel's own `\@author`: the rows a class appends to it are its authors' already, each the rows after
  // its own `\author` — `\lx@author@flush`, 59d: cjs-rcs-article.cls:361 `\newcommand\affil[1]{\g@addto@macro
  // \@author{\\#1}}`)
  let appended: Vec<Token> = written_stores(body, Some(arg))
    .into_iter()
    .filter_map(|(store, holds)| holds.then_some(store))
    .filter(|store| *store != T_CS!("\\@author"))
    .collect();
  if appended.is_empty() {
    return Ok(None);
  }
  let author_stores = class_author_stores();
  Ok(
    appended
      .iter()
      .any(|s| author_stores.contains(s))
      .then_some((arg, false)),
  )
}

/// A rerouted setter's call ([`reroute_in_place_contact_setters`]): `api` where the line it sets is a contact of the
/// author creators, else `class`, the file's own definition (`\lx@class@contact@<name>`). The API's inside a creator's
/// own content while it digests (`creator_entry_in_progress`: ar-1col's `\affil{$^1$…}` inside `\author{}`,
/// 1602.05576, 2405.08158; every one of ci2018's `\affil`s in one `\author`, 1805.12376), or while the author creators
/// it annotates are being set — one stands (`has_author_creators`), and neither has a `\maketitle` placed them
/// (`author_creators_placed`) nor has the document body sealed them (`frontmatter_entry_sealed`: the paper's authors
/// came before its first section) — for a setter that prints its line where it is called (`in_place`) only a call that
/// is a line of its own in the title block: between paragraphs at the document body's own level ([`at_the_body_level`]:
/// in a group of the body, as an author group's environment is — imsart's `{aug}`, 1301.1466 —, never inside a box, a
/// note, a list item, a theorem, a minipage, a float, a `\marginpar`, a `\caption` or an alignment's cell, each of
/// which binds a mode of its own: `\footnote{\email{…}}`, `\begin{thm}\address{…} holds.\end{thm}`, `\item \email{…}`,
/// `\parbox{5cm}{\address{…}}`, `\begin{tabular}{c}Cy Cole\\ \affil{Univ C}\end{tabular}` — review rev94b r2 B3),
/// outside any other frontmatter entry digesting (an abstract). Content is never moved out of its sentence, box or
/// note: such a setter's call inside a paragraph, a box or an abstract (before `\maketitle`, or in a class without one:
/// "…from the curator, `\email{data@archive.org}`, until 2030.", review rev94b B3; commat.cls's `\address` after the
/// "Affiliation:" label of its title page's footnote, 2204.02666) is the class's line where it stands, and so is every
/// rerouted setter the class's own definition calls (`class_bodies_call_class_copies`: ae100prg.cls's `\email` prints
/// through its `\address` after a `\vspace*`, 1303.3004). So a contact line of a later unit is its text, not the
/// paper's authors' contact; a file whose names never become creators — osameet2.sty:49 `\let\author\MEETauthor`,
/// saj.cls's `\authors` (1710.05603, 2110.09949, 2212.09079) — keeps its lines beside the names it prints, not on a
/// nameless creator; and a line set after `\maketitle` placed the names stays where it prints, as the late frontmatter
/// path reads such a list right only under a binding that declares the class's mark conventions (cpc-hepnp.cls's
/// `\address` after `\maketitle`: its `,n)` e-mail marks and typed-number lists, OD #912, task 91c; just.cls's binding
/// routes the same list).
fn class_contact(api: Tokens, class: Tokens, in_place: bool) -> Tokens {
  // (not where the API call is an `[optional]{mandatory}` setter's, which decides again with the class's line itself,
  // [`optional_contact`]: a line in doubt is printed once)
  let doubt = (in_place && api.unlist_ref().first() != Some(&T_CS!("\\lx@class@contact@optional")))
    .then(|| unrequested_mark(last_group(&class)))
    .flatten();
  contact_call(api, class, in_place, doubt)
}

/// [`class_contact`]'s choice, `doubt` the operand of a mark the line leads with that no author requests yet
/// ([`unrequested_mark`]): a title-block line in doubt prints where it is called, its API call queued in doubt beside
/// it ([`doubtful`]) — review rev94b r4 S1, r5 BL1: a later `\author{and Bob Baker\inst{2}}` requests the
/// `\address[\inst{2}]{…}` before it, which the class's line had left in the body.
fn contact_call(api: Tokens, class: Tokens, in_place: bool, doubt: Option<Vec<Token>>) -> Tokens {
  if creator_entry_in_progress() {
    api
  } else if title_block_line(in_place) {
    match doubt {
      Some(operand) => doubtful(api, class, &operand),
      None => api,
    }
  } else {
    class
  }
}

/// Whether a rerouted setter's call is a line of the authors being set ([`class_contact`]): an author creator stands to
/// carry it, none placed nor sealed yet, and — for a setter printing its line where it is called — the call stands
/// between paragraphs at the body's own level, outside any other frontmatter entry digesting.
fn title_block_line(in_place: bool) -> bool {
  (!in_place || (at_the_body_level() && !frontmatter_entry_in_progress()))
    && has_author_creators()
    && !author_creators_placed()
    && !frontmatter_entry_sealed("ltx:creator", "author")
}

/// The value naming the rerouted setter's call in doubt being queued ([`doubtful`]): the queue marks its entry
/// (`queue.rs`, [`CONTACT_DOUBT_ATTR`]), the observer's ledger its call, and the flush digesting it the contact it makes
/// (`_doubt`, base_utilities `place::settle_contacts_in_doubt`).
pub(crate) const CONTACT_DOUBT: &str = "lx_class_contact_doubt";
/// The queued entry's attribute naming the call in doubt it came from ([`CONTACT_DOUBT`]).
pub(crate) const CONTACT_DOUBT_ATTR: &str = "_class_contact_doubt";

/// A title-block line in doubt (review rev94b r3 B2, r4 S1, r5 BL1-BL2): it leads with a mark no author requests yet,
/// and a later `\author` may. The class's line prints where it is called, as the class prints it, between the two
/// marks of its ink (`\lx@class@contact@ink`); the API's labelled call is queued beside it in doubt
/// (`\lx@class@contact@doubt{k}`). The `\maketitle` flush that places the authors keeps the call when one of its
/// authors requests the mark, and the frontmatter's placement lifts the line's ink then — when it is exactly a whole
/// paragraph of its own, else the contact goes and the line stays (base_utilities `place::settle_calls_in_doubt`,
/// `place::settle_contacts_in_doubt`). Any other flush — no author requesting it, the fallback at a `\section` or
/// `\end{document}` — drops the call: the line stays where the class printed it (orph3's `\address[\inst{3}]{Lab C,
/// Place}` under `Ann Able\inst{1} and Bob Baker\inst{2}`; s1's `\address[\inst{9}]{Univ Z, Zed}` before `Pre text.`:
/// no nameless creator, the words where pdflatex prints them, in its order). A line holding a note, a label or a
/// counter step is not queued at all ([`counts_or_labels`]): kept, the call would digest them a second time.
fn doubtful(api: Tokens, class: Tokens, operand: &[Token]) -> Tokens {
  if counts_or_labels(class.unlist_ref()) {
    return class;
  }
  let k = lookup_int("lx_class_contact_doubts") + 1;
  assign_value(
    "lx_class_contact_doubts",
    Stored::Int(k),
    Some(Scope::Global),
  );
  assign_value(
    &s!("lx_class_contact_doubt_key_{k}"),
    Stored::from(Tokens::new(operand.to_vec()).to_string()),
    Some(Scope::Global),
  );
  let k = ExplodeText!(&k.to_string());
  let mut out = vec![T_CS!("\\lx@class@contact@doubt"), T_BEGIN!()];
  out.extend(k.iter().copied());
  out.push(T_END!());
  out.extend(api.unlist());
  out.extend([T_CS!("\\lx@class@contact@doubt"), T_BEGIN!(), T_END!()]);
  let ink = |out: &mut Vec<Token>, at: &str| {
    out.extend([T_CS!("\\lx@class@contact@ink"), T_BEGIN!()]);
    out.extend(k.iter().copied());
    out.extend([T_END!(), T_BEGIN!()]);
    out.extend(ExplodeText!(at));
    out.push(T_END!());
  };
  ink(&mut out, "begin");
  out.extend(class.unlist());
  ink(&mut out, "end");
  Tokens::new(out)
}

/// Whether `tokens` hold a command that steps a counter or makes a note or a label (`\footnote`, `\thanks`, `\label`,
/// `\stepcounter`, …): a line in doubt holding one is not queued in doubt ([`doubtful`]), since a call kept beside its
/// printed line digests it twice.
fn counts_or_labels(tokens: &[Token]) -> bool {
  tokens.iter().any(|t| {
    t.get_catcode() == Catcode::CS
      && t.with_cs_name(|n| {
        matches!(
          n,
          "\\footnote"
            | "\\footnotemark"
            | "\\footnotetext"
            | "\\thanks"
            | "\\label"
            | "\\stepcounter"
            | "\\refstepcounter"
            | "\\addtocounter"
            | "\\setcounter"
            | "\\marginpar"
            | "\\index"
        )
      })
  })
}

/// The operand of the mark the call in doubt `k` leads with ([`doubtful`]), which an author of the flush requests or not
/// (base_utilities `place::settle_calls_in_doubt`).
pub(crate) fn doubt_key(k: &str) -> String { lookup_string(&s!("lx_class_contact_doubt_key_{k}")) }

/// The content of the last braced group of `tokens`: a rerouted setter's class call's mandatory argument.
fn last_group(tokens: &Tokens) -> &[Token] {
  let tokens = tokens.unlist_ref();
  let Some(close) = tokens.iter().rposition(|t| t.get_catcode() == Catcode::END) else {
    return &[];
  };
  let mut depth = 0usize;
  for at in (0..=close).rev() {
    match tokens[at].get_catcode() {
      Catcode::END => depth += 1,
      Catcode::BEGIN => {
        depth -= 1;
        if depth == 0 {
          return &tokens[at + 1..close];
        }
      },
      _ => {},
    }
  }
  &[]
}

/// The operand of the printed mark `line` leads with — `\inst{3}`, `\textsuperscript{3}`, `\@textsuperscript{3}`,
/// `$^{3}$`, `$^3$` —, if it does.
fn leading_mark_operand(line: &[Token]) -> Option<Vec<Token>> {
  let mut at = line
    .iter()
    .position(|t| t.get_catcode() != Catcode::SPACE)?;
  // (a braced argument's content, or the one token)
  let operand = |at: usize| -> Option<Vec<Token>> {
    let first = line.get(at)?;
    if first.get_catcode() != Catcode::BEGIN {
      return Some(vec![*first]);
    }
    let mut depth = 0usize;
    for (k, t) in line.iter().enumerate().skip(at) {
      match t.get_catcode() {
        Catcode::BEGIN => depth += 1,
        Catcode::END => {
          depth -= 1;
          if depth == 0 {
            return Some(line[at + 1..k].to_vec());
          }
        },
        _ => {},
      }
    }
    None
  };
  let first = line[at];
  if first.get_catcode() == Catcode::CS
    && first.with_cs_name(|n| matches!(n, "\\inst" | "\\textsuperscript" | "\\@textsuperscript"))
  {
    return operand(at + 1);
  }
  if first.get_catcode() != Catcode::MATH {
    return None;
  }
  at += 1;
  // (an empty group a script hangs on, `${}^3$`)
  if line
    .get(at)
    .is_some_and(|t| t.get_catcode() == Catcode::BEGIN)
    && line
      .get(at + 1)
      .is_some_and(|t| t.get_catcode() == Catcode::END)
  {
    at += 2;
  }
  if line.get(at)?.get_catcode() != Catcode::SUPER {
    return None;
  }
  operand(at + 1)
}

/// The operand of the printed mark `line` leads with ([`leading_mark_operand`]) when its label no author's name requests,
/// queued or digested in the flush (`queued_authors_requesting`, `digested_authors_requesting`), the names showing
/// marks of their own (`authors_show_marks`): a contact line no creator asks for, which the
/// API would give to a creator of no name (review rev94b r3 B2: orph3's `\address[\inst{3}]{Lab C, Place}`, orph6's
/// `\affil{\inst{3}Lab C, Place}` under `Ann Able\inst{1} and Bob Baker\inst{2}`).
fn unrequested_mark(line: &[Token]) -> Option<Vec<Token>> {
  leading_mark_operand(line).filter(|operand| {
    operand
      .iter()
      .any(|t| matches!(t.get_catcode(), Catcode::LETTER | Catcode::OTHER))
      && authors_show_marks()
      && queued_authors_requesting(operand).is_empty()
      && !digested_authors_requesting(&Tokens::new(operand.clone()).to_string())
  })
}

/// A rerouted `[optional]{mandatory}` setter's call (`\lx@class@contact@optional`): the API call its optional argument
/// `opt` makes of the line `arg`, by the API's convention — Perl's own bindings pass it as the entry's label
/// (jheppub.sty.ltxml:35-37 `\affiliation[a]{…}` → `label={a}`, `\author[a]` → `annotations={a}`) or as its caption
/// (ams_support.sty.ltxml:115 `\address[]{}` → `\lx@add@address[name={#1}]{#2}`) —, then given to [`class_contact`]
/// as the call without one is. Empty, it is dropped, as OmniBus drops it (OmniBus.cls.ltxml:92). A printed mark (a
/// superscript, `\textsuperscript`, `\inst`: w-art.cls's `\address[\inst{1}]{…}`, 0910.5837, 1105.1753,
/// quant-ph0606237) leads the line, read by its marks as any marked line is. A key an `\author[key]` gave its authors
/// (jheppub-like `\author[a,b]{…}` with `\affiliation[b]{…}`, 2303.16316; stacs.cls's `\address[uniroma]{…}`,
/// 0902.1737; `lx_author_keys`, `\lx@author@keys`) labels the line for each of those authors (`author:N`); a key the
/// names' marks print (`\address[a]{…}` under `Ann$^a$`) labels it by that mark. Words no creator requests are its
/// caption (amsart copies' `\address[Current address]{…}`: "(Current address) Lab C, Place" — smfart, nyjm, birkmult,
/// birkart, ECACRM). Anything else is in doubt: an in-place setter's call stays the class's own, a stored one's drops
/// it (review rev94b r1-r2 B2: a caption written for every optional made w-art's marks and jheppub's keys names).
fn optional_contact(name: &str, in_place: bool, opt: Tokens, arg: Tokens) -> Result<Tokens> {
  let Some((_, template)) = IN_PLACE_CONTACT_SETTERS.iter().find(|(n, _)| *n == name) else {
    return Ok(arg);
  };
  let opt_tokens: Vec<Token> = opt.unlist();
  let trimmed: Vec<&Token> = opt_tokens
    .iter()
    .filter(|t| t.get_catcode() != Catcode::SPACE)
    .collect();
  // (the class's own call, the optional argument braced when it holds a `]`)
  let class = {
    let mut tokens = vec![T_CS!(&s!("\\lx@class@contact@{name}")), T_OTHER!("[")];
    let bracket = opt_tokens.iter().any(|t| *t == T_OTHER!("]"));
    if bracket {
      tokens.push(T_BEGIN!());
    }
    tokens.extend(opt_tokens.iter().cloned());
    if bracket {
      tokens.push(T_END!());
    }
    tokens.extend([T_OTHER!("]"), T_BEGIN!()]);
    tokens.extend(arg.unlist_ref().iter().cloned());
    tokens.push(T_END!());
    Tokens::new(tokens)
  };
  // (the API call with `options` added to its own, its line `line`)
  let call = |options: &str, line: Tokens| -> Tokens {
    let template = if options.is_empty() {
      template.to_string()
    } else if template.contains("[annotate=run]") {
      template.replace("[annotate=run]", &s!("[{options}]"))
    } else {
      template.replace("\\lx@add@email}", &s!("\\lx@add@email[{options}]}}"))
    };
    let placeholder = T_CS!("\\lx@class@contact@line");
    let caption = T_CS!("\\lx@class@contact@caption");
    let mut out = Vec::new();
    for t in mouth::tokenize_internal(TeXString::assembled(
      template.replace("ARG", "\\lx@class@contact@line"),
    ))
    .unlist()
    {
      if t == placeholder {
        out.extend(line.unlist_ref().iter().cloned());
      } else if t == caption {
        out.extend(opt_tokens.iter().cloned());
      } else {
        out.push(t);
      }
    }
    Tokens::new(out)
  };
  let decide = |api: Tokens| class_contact(api, class.clone(), in_place);
  if trimmed.is_empty() {
    return Ok(decide(call("", arg)));
  }
  let mark = trimmed.iter().any(|t| {
    t.get_catcode() == Catcode::SUPER
      || (t.get_catcode() == Catcode::CS
        && t.with_cs_name(|n| matches!(n, "\\inst" | "\\textsuperscript" | "\\@textsuperscript")))
  });
  if mark {
    // (a mark no queued name requests is in doubt — review rev94b r3 B2: the address it labels a nameless creator's; an
    // in-place setter's call decided where the authors are placed, [`doubtful`], review rev94b r4 S1)
    let doubt = unrequested_mark(&opt_tokens);
    if doubt.is_some() && !in_place {
      return Ok(decide(call("", arg)));
    }
    // (an address's line read for its marks as an affiliation's is, `\lx@affiliation@withinst`: the mark labels it and
    // prints nothing of its own in it)
    let mut line = opt_tokens.clone();
    line.extend(arg.unlist_ref().iter().cloned());
    if template.starts_with("\\lx@add@address") {
      let mut wrapped = vec![T_CS!("\\lx@affiliation@withinst"), T_BEGIN!()];
      wrapped.extend(line);
      wrapped.push(T_END!());
      line = wrapped;
    }
    return Ok(contact_call(
      call("", Tokens::new(line)),
      class,
      in_place,
      doubt,
    ));
  }
  let plain = trimmed
    .iter()
    .all(|t| matches!(t.get_catcode(), Catcode::LETTER | Catcode::OTHER) && **t != T_OTHER!("]"));
  if plain {
    let text: String = opt_tokens.iter().map(ToString::to_string).collect();
    let keys: Vec<&str> = text
      .split(',')
      .map(str::trim)
      .filter(|k| !k.is_empty())
      .collect();
    // (the authors an `\author[key]` gave each key)
    let records = lookup_string(AUTHOR_KEYS);
    let mut authors: Vec<usize> = Vec::new();
    let mut every_key = !keys.is_empty();
    for key in &keys {
      let mut found = false;
      for record in records.split('\u{1}').filter(|r| !r.is_empty()) {
        let mut parts = record.split('\u{2}');
        let (Some(record_keys), Some(from), Some(to)) = (parts.next(), parts.next(), parts.next())
        else {
          continue;
        };
        if record_keys.split(',').map(str::trim).any(|k| k == *key)
          && let (Ok(from), Ok(to)) = (from.parse::<usize>(), to.parse::<usize>())
        {
          found = true;
          authors.extend(from + 1..=to);
        }
      }
      every_key &= found;
    }
    if every_key && !authors.is_empty() {
      authors.sort_unstable();
      authors.dedup();
      let mut tokens = Vec::new();
      for n in authors {
        tokens.extend(call(&s!("label={{author:{n}}}"), arg.clone()).unlist());
      }
      return Ok(decide(Tokens::new(tokens)));
    }
    if !keys.is_empty()
      && keys.iter().all(|key| {
        let operand: Vec<Token> = key
          .chars()
          .map(|c| {
            let c = c.to_string();
            if c.chars().all(char::is_alphabetic) {
              T_LETTER!(&c)
            } else {
              T_OTHER!(&c)
            }
          })
          .collect();
        !queued_authors_requesting(&operand).is_empty() || digested_authors_requesting(key)
      })
    {
      return Ok(decide(call(&s!("label={{{}}}", keys.join(",")), arg)));
    }
    // (a list of single words, `c, d`, is a key list no creator requests — in doubt, no caption: review rev94b r3 B2b,
    // `\address[c, d]{…}` was captioned "c, d")
    let key_list = text.contains(',')
      && text
        .split(',')
        .all(|piece| piece.split_whitespace().count() <= 1);
    if !key_list
      && text
        .split_whitespace()
        .filter(|w| w.chars().any(char::is_alphabetic))
        .count()
        >= 2
    {
      // (an address or an affiliation keeps its run, an e-mail the API's own placement)
      let options = if template.contains("[annotate=run]") {
        "annotate=run,name={\\lx@class@contact@caption}"
      } else {
        "name={\\lx@class@contact@caption}"
      };
      return Ok(decide(call(options, arg)));
    }
  }
  // (in doubt: an in-place setter's line is the class's, printed where it is called — a key no author can request, so
  // no call is queued beside it (review rev94b r5 BL1: deferred, two class lines swapped) —, a stored setter's call the
  // API's without the optional)
  Ok(if in_place {
    class
  } else {
    decide(call("", arg))
  })
}

/// The records of the kernel's `\author[key]{…}` calls (`\lx@author@keys`): each its keys (`a,b`), the number of the
/// author creators standing before it and after it, `\u{2}`-parted, the records `\u{1}`-parted.
const AUTHOR_KEYS: &str = "lx_author_keys";

/// Whether the digestion stands between paragraphs at the document body's own level: in vertical mode, and no frame
/// binds a mode of its own (`BOUND_MODE`: the body's is `\begin{document}`'s, bound in no frame of its own,
/// sect02.rs) — a group of the body binds none (`{\small …}`, an environment that is a group, imsart's `{aug}`); a box,
/// a note, a list, a theorem, a minipage, a float or an alignment's cell does (`begin_mode`) — or in the box the
/// authors were set in, still open, no box opened since (`\lx@author@box`): the title block's own box, imsart.sty's
/// `\open@fm` `\global\setbox\fm@box=\vbox\bgroup` around `{frontmatter}` (1301.1466's `\affiliation` in its
/// `{aug}`).
fn at_the_body_level() -> bool {
  let boxes = mode_frames();
  matches!(
    lookup_string("MODE").as_str(),
    "vertical" | "internal_vertical"
  ) && (boxes == 0 || lookup_string(AUTHOR_BOX) == author_box())
}

/// How many frames bind a mode of their own ([`at_the_body_level`]).
fn mode_frames() -> usize {
  (0..get_frame_depth())
    .filter(|&n| is_value_bound("BOUND_MODE", Some(n)))
    .count()
}

/// The box the digestion stands in: the serial (`lx@frame@id`, `push_stack_frame`) of the innermost frame binding a
/// mode of its own ([`mode_frames`]) — the same while that box is open and no box opened inside it, whatever groups
/// opened or closed in it; another box, even at the same depth (a second minipage), another.
fn author_box() -> String {
  (0..get_frame_depth())
    .find(|&n| is_value_bound("BOUND_MODE", Some(n)))
    .and_then(|n| value_in_frame("lx@frame@id", n))
    .map(|id| match id {
      Stored::Number(n) => n.0.to_string(),
      other => format!("{other:?}"),
    })
    .unwrap_or_default()
}

/// The box the kernel's last `\author` call stood in ([`author_box`]), recorded globally (`\lx@author@box`): a group
/// it was called in — imsart's `{aug}`, the kernel's own call — closes before the authors' box does (review rev94b r3:
/// imsart's `\affiliation` after `\end{aug}` in its `\open@fm` box went to the class); recorded anew at each
/// `\author`, read only while the authors are being set (`class_contact`).
const AUTHOR_BOX: &str = "lx_author_box";

/// Each kept class definition (`\lx@class@contact@<name>`) of the rerouted setters `names`, rewritten so that its calls
/// to another rerouted setter — or to itself — call that setter's class definition while the setter still means the
/// reroute, else its meaning then (`\lx@class@contact@dispatch@<name>`, one token at the call site, which expands to
/// that one token: a document's
/// `\renewcommand{\address}[1]{…Addr: #1…}` is what ae100prg's `\email` prints through — review rev94b r3 S3): a setter run as the class's
/// prints all of its line where the class prints it, never part of it through the API (ae100prg.cls:185 `\def\email#1
/// {\vspace*{5pt}\address{E-mail: #1}}`, 1303.3004: its `\address` the class's `\lx@class@contact@address`). Static, at
/// the reroute: nothing is left in the call's input for the class's own look-ahead to see (a class's
/// `\def\email#1{\\{\ttfamily #1}\ignorespaces}` skips the spaces after the call, `\@ifnextchar`/`\@ifstar` read the
/// token after it — review rev94b r2 S3).
fn class_bodies_call_class_copies(names: &[&str]) {
  let rerouted: Vec<(Token, Token)> = names
    .iter()
    .map(|name| {
      (
        T_CS!(&s!("\\{name}")),
        T_CS!(&s!("\\lx@class@contact@{name}")),
      )
    })
    .collect();
  for (_, copy) in &rerouted {
    let Some(Stored::Expandable(defn)) = lookup_meaning(copy) else {
      continue;
    };
    let Some(ExpansionBody::Tokens(body)) = defn.get_expansion() else {
      continue;
    };
    if !body
      .unlist_ref()
      .iter()
      .any(|t| rerouted.iter().any(|(cs, _)| cs == t))
    {
      continue;
    }
    let tokens = body.unlist_ref();
    let mut calls: Vec<Token> = Vec::new();
    for (k, t) in tokens.iter().enumerate() {
      match rerouted.iter().position(|(cs, _)| cs == t) {
        // (an `\ifx` operand compares the class's own definition, as the file's test did: `\ifx\address\relax`)
        Some(at)
          if [1, 2]
            .iter()
            .any(|back| k >= *back && tokens[k - back] == T_CS!("\\ifx")) =>
        {
          calls.push(rerouted[at].1);
        },
        // (one token, whatever reads it: `\expandafter\address\expandafter{…}`, `\let\x\address`, `\@ifnextchar[\address`
        // — review rev94b r4 S2, cexp.cls's `\email`: a braced name after one dispatcher left the class copy's `#1` to
        // the `\expandafter`)
        Some(at) => calls.push(T_CS!(&s!("\\lx@class@contact@dispatch@{}", names[at]))),
        None => calls.push(*t),
      }
    }
    let mut rewritten = (*defn).clone();
    rewritten.expansion = Some(ExpansionBody::Tokens(Tokens::new(calls)));
    rewritten.plan = std::cell::OnceCell::new();
    assign_meaning(
      copy,
      Stored::Expandable(Rc::new(rewritten)),
      Some(Scope::Global),
    );
  }
}

/// After a raw file is read (a class, `\lx@class@loaded@raw`; a package, `\lx@package@loaded@raw`): each Frontmatter
/// API contact setter it defined (`IN_PLACE_CONTACT_SETTERS`; still its own definition, not a binding's, nor one K11
/// rerouted as a store) that sets its argument as a title-page line (`in_place_contact_setter`) is the API's, as the
/// kernel's locked `\author` stays the kernel's against the file's own: the line becomes a contact of the creators,
/// keyed by its printed marks, and the file's in-place typesetting of it is not kept — in pdflatex it prints under the
/// names its `\author` printed, which the kernel's `\author` made creators. Beyond Perl, which reads no shipped file and
/// gives the paper OmniBus's setters. OXIDIZED_DESIGN_DIVERGENCES #936.
pub fn reroute_in_place_contact_setters(file: &str) -> Result<()> {
  // (each decided on the file's own definitions before any is rerouted: ae100prg.cls's `\email` prints through its
  // `\address`)
  let mut setters = Vec::new();
  for (name, api) in IN_PLACE_CONTACT_SETTERS {
    if defined_by_a_raw_file(name)?
      && let Some((arg, in_place)) = in_place_contact_setter(name)?
    {
      setters.push((*name, *api, arg, in_place));
    }
  }
  let mut rerouted = Vec::new();
  for (name, api, arg, in_place) in setters {
    let (params, api, class_args) = if arg == 2 {
      (
        convert_latex_args(2, Some(Tokens::new(Vec::new())))?,
        s!(
          "\\lx@class@contact@optional{{{name}}}{{{}}}{{#1}}{{#2}}",
          u8::from(in_place)
        ),
        "[#1]{#2}",
      )
    } else {
      (
        convert_latex_args(1, None)?,
        api.replace("ARG", "#1"),
        "{#1}",
      )
    };
    // (the file's own definition kept, for the calls `\lx@class@contact` gives it; both global, as the file's own
    // definition was the document's for the rest of the run)
    let cs = T_CS!(&s!("\\{name}"));
    Let!(T_CS!(&s!("\\lx@class@contact@{name}")), cs, Scope::Global);
    // (a setter that prints its line where it is called decides by where the call stands, `\lx@class@contact`; one
    // that stores it for the title page sets a title-page line wherever it is called, `\lx@class@contact@stored`)
    let decider = if in_place {
      "\\lx@class@contact"
    } else {
      "\\lx@class@contact@stored"
    };
    let body = mouth::tokenize_internal(TeXString::assembled(s!(
      "{decider}{{{api}}}{{\\lx@class@contact@{name}{class_args}}}"
    )));
    DefMacro!(cs, params, body, scope => Some(Scope::Global));
    // (the reroute's own meaning, which a class body's call is dispatched by: `\lx@class@contact@dispatch@<name>`, one
    // token where the body called the setter)
    Let!(
      T_CS!(&s!("\\lx@class@contact@rerouted@{name}")),
      cs,
      Scope::Global
    );
    let dispatch = mouth::tokenize_internal(TeXString::assembled(s!(
      "\\lx@class@contact@dispatch{{{name}}}"
    )));
    DefMacro!(
      T_CS!(&s!("\\lx@class@contact@dispatch@{name}")),
      None,
      dispatch,
      scope => Some(Scope::Global)
    );
    rerouted.push(name);
  }
  class_bodies_call_class_copies(&rerouted);
  if !rerouted.is_empty() {
    Info!(
      "frontmatter",
      file,
      s!(
        "The raw file's contact setters that print a title-page line set the frontmatter's contacts: \\{}",
        rerouted.join(", \\")
      )
    );
  }
  Ok(())
}

/// Reroute every table setter the raw class `cls` defined as a store; a binding's own setter
/// (acmart.cls's `\copyrightyear`, ported as the class's store) is not the raw class's.
pub fn reroute_raw_class_stores(cls: &str) -> Result<()> {
  let mut rerouted: Vec<&str> = Vec::new();
  let mut readers: Option<ClassReaders> = None;
  let mut setters = Vec::new();
  for (name, api) in STORE_SETTERS {
    if defined_by_a_binding(name)? {
      continue;
    }
    if let Some(setter) = store_setter_body(name, &mut readers)? {
      setters.push((*name, *api, setter));
    }
  }
  // What the frontmatter carries: these setters' stores and the kernel's fields, which the deposit nulls.
  let carried_stores: Vec<Token> = setters
    .iter()
    .map(|(_, _, (_, _, store_cs))| *store_cs)
    .chain(["\\@title", "\\@author", "\\@date", "\\@thanks"].map(|cs| T_CS!(cs)))
    .collect();
  let carried = |t: &Token| carried_stores.contains(t);
  for (name, api, (store, arg, store_cs)) in setters {
    let readers = readers.get_or_insert_with(ClassReaders::survey);
    let lines = readers.store_lines(&store_cs, &carried);
    if lines.glued || lines.others.iter().any(prints_as_a_value) {
      Info!(
        "frontmatter",
        cls,
        s!(
          "The raw class's \\{name} stays the class's: its title page prints it in one line with another value"
        )
      );
      continue;
    }
    // The other values of its lines print nothing as the class leaves them; `\maketitle` judges them again with the
    // document's (`harvest_stores`), so a store handed on as it is set waits for it (`lx_store_deferred_<name>`) —
    // when one of them is a placeholder the document can fill there ([`may_become_a_value`]), not the line's code
    // (`\@setfontsize`, `\@journalfont`; review rev79a2 finding 5).
    let others: Vec<Token> = lines
      .others
      .into_iter()
      .filter(may_become_a_value)
      .collect();
    let line_values = !others.is_empty();
    assign_value(
      &s!("lx_store_line_values_{name}"),
      Stored::Tokens(Tokens::new(others)),
      Some(Scope::Global),
    );
    if line_values && !read_at_maketitle(name) {
      assign_value(&s!("lx_store_deferred_{name}"), true, Some(Scope::Global));
    }
    // A store the document never set is handed on at `\maketitle` only when the title page prints it.
    assign_value(
      &s!("lx_store_title_page_reads_{name}"),
      readers.read_by_the_title_page(&store_cs),
      Some(Scope::Global),
    );
    assign_value(
      &s!("lx_store_cs_{name}"),
      Stored::String(pin(
        store_cs.with_str(|cs| cs.trim_start_matches('\\').to_string()),
      )),
      Some(Scope::Global),
    );
    // The setter stores as the class does, so the class's other readers of `\@<name>` (gaceta
    // checks `\@editor` for its section editor line) keep seeing the value; only `\@maketitle` is
    // discarded. A store that annotates a creator (an affiliation, address, e-mail, ORCID) or makes
    // one (an editor, a speaker) is then only recorded: it is read once, at `\maketitle`, where the author list it annotates exists
    // (`harvest_stores`; set later, it is handed on at once, `\lx@store@late`). Any other store is
    // handed to its frontmatter API as it is set — keywords set after `\maketitle` for an
    // abstract that prints them (pittetd.cls:503-527), an abstract at its place in the body —, save
    // an abstract or keywords the title page names by its label, which `\maketitle` reads too
    // (`labelled_api`, silent-loss #6a).
    let (params, arg_ref) = if arg == 2 {
      (convert_latex_args(2, Some(Tokens::new(Vec::new())))?, "#2")
    } else {
      (convert_latex_args(1, None)?, "#1")
    };
    // The label the class's title page prints before the store ("Resumen: \@resumen") names the entry, and the deposit
    // takes it out of the title page with the store (`labelled_api`).
    let api = labelled_api(name, api, &store_cs, readers);
    let api = api.as_str();
    let mut body = store.unlist();
    let tail = if read_at_maketitle(name) {
      s!("\\lx@store@set{{{name}}}\\lx@store@late{{{name}}}{{{arg_ref}}}")
    } else {
      s!(
        "\\lx@store@set{{{name}}}\\lx@store@hand{{{name}}}{{{}}}",
        api.replace("#1", arg_ref)
      )
    };
    body.extend(mouth::tokenize_internal(TeXString::assembled(tail)).unlist());
    DefMacro!(T_CS!(&s!("\\{name}")), params, Tokens::new(body));
    rerouted.push(name);
  }
  reroute_keywords_environments(cls, &mut readers)?;
  reroute_kernel_field_setters(cls)?;
  survey_off_table_stores(cls, &mut readers)?;
  if !rerouted.is_empty() {
    // A raw class that `\LoadClass`es another raw class reroutes twice: the
    // list accumulates, so the base class's stores stay captured too.
    let prior = lookup_string("lx_rerouted_stores");
    let mut captured: Vec<&str> = prior.split(',').filter(|n| !n.is_empty()).collect();
    for name in &rerouted {
      if !captured.contains(name) {
        captured.push(name);
      }
    }
    // Inside the deposit group a store handed to the frontmatter reads as empty, so a kept
    // `\@maketitle` never typesets it twice; one not handed on (an affiliation with no author to
    // carry it: courseoutline.cls:150 `\@department` in a document with no `\author`) stays the
    // class's to print. One the document never set and the class gives no value before its own
    // `\maketitle` — an.cls:532 `\ch@ckobl{institute}{Address(es) of author(s) should be given}`
    // runs in the class's `\maketitle`, which the kernel's replaces — is defined: its undefined
    // store raised an error that dropped the whole deposit, the paper's `\keywords` line with it
    // (0 errors; pdflatex prints the line beside the class's placeholder). As `\relax` or `\@empty`,
    // whichever the title page prints fewer characters under in TeX's runs (`decide_unset_nulls`): `\relax`
    // where a class tests for the unset store, ptptex.cls:88-104's journal line and "(Received …)"
    // printed unless `\@ifundefined{@pubinfo}`/`{@recdate}` (hep-th9803240, 1309.6905); `\@empty`
    // where it tests `\ifx\@institute\@empty` after its own `\maketitle` defaulted it so.
    DefMacro!("\\lx@captured@stores", sub[_args] {
      let names = lookup_string("lx_rerouted_stores");
      let mut nulls: Vec<Token> = Vec::new();
      for name in names.split(',').filter(|n| !n.is_empty()) {
        let store = store_of(name);
        let null = if lookup_bool(&s!("lx_store_handed_{name}")) {
          s!("\\let\\{store}\\@empty")
        } else if lookup_bool(&s!("lx_store_unset_empty_{name}")) {
          s!("\\lx@ifdefined\\{store}\\else\\let\\{store}\\@empty\\fi")
        } else {
          s!("\\lx@ifdefined\\{store}\\else\\let\\{store}\\relax\\fi")
        };
        nulls.extend(mouth::tokenize_internal(TeXString::assembled(null)).unlist());
      }
      Ok(Tokens::new(nulls))
    });
    // A store the document never set keeps the class's default (lion-msc.cls:
    // 196-203 `\gdef\@affiliation{Huygens-Kamerlingh Onnes Laboratory, …}`),
    // which `\@maketitle` typesets; nulled in the deposit, it reached neither
    // the frontmatter nor the body. The kernel `\maketitle` hands it to the
    // frontmatter (`\lx@store@defaults` in `\lx@maketitle@body`, sect05.rs),
    // as pdflatex prints it only there — when the title page prints it
    // (`lx_store_title_page_reads_<name>`, set above; 1209.2617's tlp placeholders are its running head's).
    assign_value(
      "lx_rerouted_stores",
      Stored::String(pin(captured.join(","))),
      Some(Scope::Global),
    );
    // The class's `\@maketitle` is NOT discarded here: `\lx@deposit@maketitle`
    // (sect05.rs) digests it with the captured stores nulled and keeps the
    // result only if it typeset anything — ptptex's (every field captured)
    // yields nothing and is dropped; bfhthesis's degree/advisor block beside
    // its captured `\@institution` is kept (sweep 93 lost half of two bfh-ci
    // manuals' recall to an unconditional discard).
    Info!(
      "frontmatter",
      cls,
      s!(
        "Rerouted the raw class's title-page stores to the frontmatter API: \\{}",
        rerouted.join(", \\")
      )
    );
  }
  Ok(())
}

/// The frontmatter call of the store `\<name>` (the table's `api`), named by the label the class's title page prints
/// right before the store ([`ClassReaders::title_page_label`]) when the entry is one shown under a name: an abstract,
/// keywords or a classification. rcresumen's `\def\@maketitle{… Resumen: \@resumen}`: the class's abstract is named
/// "Resumen:", as astlb.cls:200's in-abstract `Key words:` names its keywords (1202.2889). An abstract or keywords store
/// the title page reads, labelled or not, and a labelled classification, is the class's store, which LaTeX expands
/// where the class's `\maketitle` prints it: its setter only stores (`lx_store_deferred_<name>`, `\lx@store@hand`) and
/// `\maketitle` queues the entry from the store (`harvest_stores`), with no clearing of other entries, digested there
/// with the rest — so a value using a macro or package defined later in the preamble reads as pdflatex reads it (review
/// probe `early`: "Uses bar and red"), and the document's own `{abstract}`, which clears what is queued, does not remove
/// it: the class's store and the document's abstract are two elements (OXIDIZED_DESIGN_DIVERGENCES #154), as pdflatex
/// prints both — "Resumen:" and "Abstract", the document's first when its `{abstract}` comes before `\maketitle`; an
/// unlabelled `\begin{quote}\itshape\@resumen\end{quote}` too (review probe `d_nolab`; silent-loss #6a). The label's
/// tokens are recorded for the title-page deposit (`lx_store_label_part_<name>`), which takes them out with the store
/// when it is handed on (`\lx@store@handed`): the label is the entry's name, not a line of its own before the body —
/// once `\maketitle`'s run of the title page shows it printed right before the store (`verify_title_page_labels`;
/// else the entry is unnamed, `lx_store_api_unnamed_<name>`). The api of any other store is the table's. Returns the
/// api, stored for the later calls (`store_api`).
fn labelled_api(name: &str, api: &str, store: &Token, readers: &ClassReaders) -> String {
  // (`_store`: the class's store's entry, which only another of the same store replaces — the document's own abstract
  // under the same name does not, `frontmatter_clear_same_name`, while the title page prints the store,
  // `release_unprinted_store_entries`. The store's name, not the setter's: two setters of one store, a class's `\let\keyword
  // \keywords` or `\let\resumen\abst`, make one entry — review probes `al1`, `al2`)
  let tag = store.with_str(|cs| cs.trim_start_matches('\\').to_string());
  let (named, unnamed) = if api.starts_with("\\lx@add@abstract") {
    (
      Some(s!(
        "\\lx@frontmatter@mark{{ltx:abstract}}\\lx@add@frontmatter{{ltx:abstract}}[_store={tag},name={{"
      )),
      s!(
        "\\lx@frontmatter@mark{{ltx:abstract}}\\lx@add@frontmatter{{ltx:abstract}}[_store={tag}]{{#1}}"
      ),
    )
  } else if api.starts_with("\\lx@add@keywords") {
    (
      Some(s!(
        "\\lx@add@frontmatter{{ltx:keywords}}[_store={tag},name={{"
      )),
      s!("\\lx@add@frontmatter{{ltx:keywords}}[_store={tag}]{{#1}}"),
    )
  } else if api.starts_with("\\lx@add@classification") {
    (None, api.to_string())
  } else {
    return api.to_string();
  };
  let deferred = |call: &str| {
    assign_value(
      &s!("lx_store_api_{name}"),
      Stored::String(pin(call)),
      Some(Scope::Global),
    );
    assign_value(&s!("lx_store_deferred_{name}"), true, Some(Scope::Global));
  };
  let Some(StoreLabel {
    reader,
    part,
    text,
    in_argument,
  }) = readers.title_page_label(store)
  else {
    if named.is_some() && readers.read_by_the_title_page(store) {
      deferred(&unnamed);
      return unnamed;
    }
    return api.to_string();
  };
  // (the label as printed, without the space before the store)
  let label = text.to_string().trim_end().to_string();
  let call = match named {
    Some(head) => s!("{head}{label}}}]{{#1}}"),
    // `\lx@add@classification[scheme=…]{#1}`: the name joins its options
    None => match api.split_once('[') {
      Some((command, rest)) => s!("{command}[name={{{label}}},{rest}"),
      None => api.replacen("{#1}", &s!("[name={{{label}}}]{{#1}}"), 1),
    },
  };
  assign_value(
    &s!("lx_store_label_reader_{name}"),
    Stored::String(pin(reader)),
    Some(Scope::Global),
  );
  assign_value(
    &s!("lx_store_label_part_{name}"),
    Stored::Tokens(part),
    Some(Scope::Global),
  );
  assign_value(
    &s!("lx_store_label_{name}"),
    Stored::Tokens(text),
    Some(Scope::Global),
  );
  assign_value(
    &s!("lx_store_label_in_argument_{name}"),
    in_argument,
    Some(Scope::Global),
  );
  assign_value(
    &s!("lx_store_api_unnamed_{name}"),
    Stored::String(pin(unnamed.as_str())),
    Some(Scope::Global),
  );
  deferred(&call);
  call
}

/// The store `\<name>` is handed to the frontmatter: the title-page deposit reads it as empty
/// (`\lx@captured@stores`), and the label its title page prints before it ([`labelled_api`]), now the entry's name,
/// is taken out with it, once (`note_title_page_part`, read by `\lx@replay@records`).
fn note_store_handed(name: &str) {
  assign_value(&s!("lx_store_handed_{name}"), true, Some(Scope::Global));
  if let Some(Stored::Tokens(part)) = lookup_value(&s!("lx_store_label_part_{name}"))
    && !lookup_bool(&s!("lx_store_label_noted_{name}"))
  {
    assign_value(
      &s!("lx_store_label_noted_{name}"),
      true,
      Some(Scope::Global),
    );
    crate::latex_constructs::note_title_page_part(
      &lookup_string(&s!("lx_store_label_reader_{name}")),
      part,
    );
  }
}

/// The store `\<name>` set after `\maketitle` read the stores was handed on as it was set (`\lx@store@late`,
/// `\lx@store@hand`): a harvest re-armed for a superseding `\maketitle` does not hand it again (`lx_stores_late`).
fn note_store_late(name: &str) {
  let late = lookup_string("lx_stores_late");
  if !late.split(',').any(|n| n == name) {
    let late = if late.is_empty() {
      name.to_string()
    } else {
      s!("{late},{name}")
    };
    assign_value(
      "lx_stores_late",
      Stored::String(pin(late)),
      Some(Scope::Global),
    );
  }
}

/// The frontmatter call of the store `\<name>`: the class's label's ([`labelled_api`]) or the table's.
fn store_api(name: &str) -> Option<String> {
  let named = lookup_string(&s!("lx_store_api_{name}"));
  if !named.is_empty() {
    return Some(named);
  }
  STORE_SETTERS
    .iter()
    .find(|(n, _)| *n == name)
    .map(|(_, api)| api.to_string())
}

/// The kernel's title fields, which LaTeX's `\@maketitle` prints (article.cls:209-229) and the title-page deposit and
/// replay read as empty (`\lx@deposit@maketitle`, sect05.rs).
const KERNEL_FIELDS: [&str; 4] = ["\\@title", "\\@author", "\\@date", "\\@thanks"];

/// Silent-loss #5 (a), the generic rule for the class's off-table stores (75a row (a)): a one-argument setter the raw
/// class defined, not in the table (`STORE_SETTERS`) and not one of its commands with an `@`, whose body is a pure store
/// of its argument ([`setter_stores`]) in one control sequence of the class's that only its title-page code reads
/// (rule (a)'s [`ClassReaders::read_only_by_the_title_page`], the class's own `\@title`/`\@author`/`\@date` counted
/// as title-page code, LaTeX's `\@maketitle` printing them): jfpc.cls:121-132 `\titre`, `\auteurs`,
/// `\institutions`, `\mels`, read only by its `\def\@title`/`\def\@author` (:176-191; 1503.05530), which the
/// deposit reads as empty, so the authors were lost; risethesis's cover fields (2208.03566), vmsta's `\fnm`. Each is
/// recorded with its class default (`lx_offtable_stores`); at `\maketitle` `\lx@store@untypeset` hands on the value
/// the document gave one whose title-page code did not typeset it.
fn survey_off_table_stores(cls: &str, readers: &mut Option<ClassReaders>) -> Result<()> {
  let from_a_file = |stored: &Stored| match stored {
    Stored::Expandable(defn) => defn.get_origin() == DefinitionOrigin::File,
    _ => false,
  };
  let mut found: Vec<(String, Token)> = Vec::new();
  let mut names: Vec<String> = current_meanings_where(from_a_file)
    .into_iter()
    .map(|(key, _)| with(key, ToString::to_string))
    .filter(|name| {
      name.len() > 1 && name.starts_with('\\') && name[1..].chars().all(|c| c.is_ascii_alphabetic())
    })
    .collect();
  names.sort();
  let known = lookup_string("lx_offtable_stores");
  for cs_name in names {
    let name = &cs_name[1..];
    if STORE_SETTERS.iter().any(|(n, _)| *n == name)
      || ["title", "author", "date", "thanks", "maketitle", "and"].contains(&name)
      || known.split(',').any(|n| n == name)
    {
      continue;
    }
    let Some((_, _, stores)) = setter_stores(&T_CS!(&cs_name))? else {
      continue;
    };
    let [store] = stores[..] else {
      continue;
    };
    let class_owned = match lookup_meaning(&store) {
      None => true,
      Some(Stored::Expandable(defn)) => defn.get_origin() == DefinitionOrigin::File,
      Some(_) => false,
    };
    if !class_owned || store.with_str(|s| KERNEL_FIELDS.contains(&s)) {
      continue;
    }
    let readers = readers.get_or_insert_with(ClassReaders::survey);
    if readers.read_only_by_the_title_page_or_fields(&store) {
      found.push((name.to_string(), store));
    }
  }
  if found.is_empty() {
    return Ok(());
  }
  let mut list: Vec<String> = known
    .split(',')
    .filter(|n| !n.is_empty())
    .map(String::from)
    .collect();
  for (name, store) in &found {
    let default = match lookup_meaning(store) {
      Some(Stored::Expandable(defn)) => match defn.get_expansion() {
        Some(ExpansionBody::Tokens(body)) => Stored::Tokens(body.clone()),
        _ => Stored::Tokens(Tokens::default()),
      },
      _ => Stored::Bool(false),
    };
    assign_value(
      &s!("lx_offtable_default_{name}"),
      default,
      Some(Scope::Global),
    );
    assign_value(
      &s!("lx_offtable_store_{name}"),
      Stored::Token(*store),
      Some(Scope::Global),
    );
    if let Some(StoreLabel {
      part, text: label, in_argument, ..
    }) = readers
      .as_ref()
      .and_then(|r| r.label_in(store, &r.title_page_fields))
    {
      assign_value(
        &s!("lx_offtable_label_in_argument_{name}"),
        in_argument,
        Some(Scope::Global),
      );
      // (with the space the title page prints between the label and the store, a tie in the note's name, as
      // acmart_cls.rs:178's `name={Price:~}`: jfpc.cls:177 "Actes JFPC \@lannee", "Actes JFPC 2015")
      let mut label = label.unlist();
      let separated = part.unlist_ref().windows(2).any(|pair| {
        pair[1] == *store
          && (pair[0].get_catcode() == Catcode::SPACE
            || pair[0].get_catcode() == Catcode::ACTIVE
            || pair[0] == T_CS!("\\ "))
      });
      if separated
        && label
          .last()
          .is_some_and(|t| t.get_catcode() != Catcode::SPACE)
      {
        label.push(T_ACTIVE!('~'));
      }
      assign_value(
        &s!("lx_offtable_label_{name}"),
        Stored::Tokens(Tokens::new(label)),
        Some(Scope::Global),
      );
    }
    list.push(name.clone());
  }
  // (the stores the class's own `\@author` reads, in its order, for `hand_untypeset_stores`)
  let author_reads: Vec<String> = file_body(&T_CS!("\\@author"))
    .map(|body| {
      let mut reads: Vec<String> = Vec::new();
      for t in body.unlist_ref() {
        if found.iter().any(|(_, store)| store == t) {
          let cs = t.with_str(ToString::to_string);
          if !reads.contains(&cs) {
            reads.push(cs);
          }
        }
      }
      reads
    })
    .unwrap_or_default();
  assign_value(
    "lx_offtable_author_reads",
    Stored::String(pin(author_reads.join(","))),
    Some(Scope::Global),
  );
  assign_value(
    "lx_offtable_stores",
    Stored::String(pin(list.join(","))),
    Some(Scope::Global),
  );
  Info!(
    "frontmatter",
    cls,
    s!(
      "The raw class's title-page stores outside the table, handed on at \\maketitle when its title page does not typeset them: \\{}",
      found
        .iter()
        .map(|(n, _)| n.as_str())
        .collect::<Vec<_>>()
        .join(", \\")
    )
  );
  Ok(())
}

/// `body` without what it stores and does not run (tex.web §473): the parameter and replacement texts of the definitions
/// it makes (`\def`-family, LaTeX's `\newcommand` family, `\g@addto@macro`), and the page furniture it sets
/// (`\markboth`, `\markright`, `\@mkboth`; the running heads LaTeXML drops), and what it puts in a box or measures
/// (`\setbox`, `\sbox`, `\settowidth`, …), which prints only where the box is used: degruyter-journal-a.cls:574-586
/// `\def\@headlineauthor{\xprint{\@nameshortone}…}` and acmlarge.cls:1021 `\def\@firstfoot{…\@acmVolume…}` (1704.03582,
/// 1508.03599) print their stores in a running head, not on the title page.
fn executed_code(body: &[Token]) -> Vec<Token> {
  const DEFS: &[&str] = &[
    "\\def",
    "\\gdef",
    "\\edef",
    "\\xdef",
    "\\protected@edef",
    "\\protected@xdef",
  ];
  const COMMANDS: &[&str] = &[
    "\\newcommand",
    "\\renewcommand",
    "\\providecommand",
    "\\DeclareRobustCommand",
  ];
  // (with the boxes it fills and the material it measures, which print only if used: degruyter-journal-a.cls:620
  // `\global\setbox\shorttitrun=\hbox{…\@headlinetitle…}`, measured for its running head)
  const FURNITURE: &[(&str, usize)] = &[
    ("\\markboth", 2),
    ("\\markright", 1),
    ("\\@mkboth", 2),
    ("\\g@addto@macro", 2),
    ("\\sbox", 2),
    ("\\savebox", 2),
    ("\\settowidth", 2),
    ("\\settoheight", 2),
    ("\\settodepth", 2),
    ("\\settototalheight", 2),
  ];
  // the index past one argument at `k`: a group or a single token
  let argument = |k: usize| -> usize {
    let k = skip_spaces(body, k);
    group_end(body, k).unwrap_or((k + 1).min(body.len()))
  };
  let mut out = Vec::with_capacity(body.len());
  let mut i = 0;
  while i < body.len() {
    let t = body[i];
    if cs_in(&t, DEFS) {
      // the definer and its name stay (the name is written, not read); the parameter text and the replacement text go
      out.push(t);
      if let Some(name) = body.get(skip_spaces(body, i + 1)) {
        out.push(*name);
      }
      let mut k = i + 1;
      while k < body.len() && body[k].get_catcode() != Catcode::BEGIN {
        k += 1;
      }
      i = group_end(body, k).unwrap_or(body.len());
      continue;
    }
    if cs_in(&t, COMMANDS) {
      let mut k = skip_spaces(body, i + 1);
      if body.get(k) == Some(&T_OTHER!("*")) {
        k += 1;
      }
      k = argument(k);
      for _ in 0..2 {
        let j = skip_spaces(body, k);
        if body.get(j) == Some(&T_OTHER!("[")) {
          k = body[j..]
            .iter()
            .position(|b| *b == T_OTHER!("]"))
            .map_or(body.len(), |p| j + p + 1);
        }
      }
      i = argument(k);
      continue;
    }
    if let Some((_, n)) = FURNITURE.iter().find(|(cs, _)| cs_in(&t, &[cs])) {
      let mut k = i + 1;
      for _ in 0..*n {
        k = argument(k);
        // (`\savebox{\x}[<width>][<pos>]{…}`)
        while body.get(skip_spaces(body, k)) == Some(&T_OTHER!("[")) {
          let j = skip_spaces(body, k);
          k = body[j..]
            .iter()
            .position(|b| *b == T_OTHER!("]"))
            .map_or(body.len(), |p| j + p + 1);
        }
      }
      i = k;
      continue;
    }
    if t == T_CS!("\\setbox") {
      // `\setbox<register>=\hbox{…}`: up to the box's group, and the group
      let mut k = i + 1;
      while k < body.len() && body[k].get_catcode() != Catcode::BEGIN {
        k += 1;
      }
      i = group_end(body, k).unwrap_or(body.len());
      continue;
    }
    out.push(t);
    i += 1;
  }
  out
}

/// What the live code (`reads_of_branches`, the switches as they are now) reached from `roots` reads, in the order it
/// reads it (depth first, each control sequence once), following the macros a raw file defined and never entering
/// `blocked` (the fields a title-page deposit reads as empty); with `executed`, only the code that runs
/// ([`executed_code`]).
fn live_read_order(roots: &[Token], blocked: &[Token], executed: bool) -> Vec<SymStr> {
  struct Walk<'a> {
    blocked:  &'a [Token],
    executed: bool,
    seen:     FxHashSet<SymStr>,
    order:    Vec<SymStr>,
  }
  impl Walk<'_> {
    fn visit(&mut self, body: &Tokens, depth: usize) {
      let code = if self.executed {
        executed_code(body.unlist_ref())
      } else {
        body.unlist_ref().to_vec()
      };
      for cs in reads_of_branches(&code, true) {
        if self.blocked.contains(&cs) || !self.seen.insert(cs.get_cs_name()) {
          continue;
        }
        self.order.push(cs.get_cs_name());
        if depth < 64
          && let Some(callee) = file_body(&cs)
        {
          self.visit(&callee, depth + 1);
        }
      }
    }
  }
  let mut walk = Walk {
    blocked,
    executed,
    seen: FxHashSet::default(),
    order: Vec::new(),
  };
  for root in roots {
    if blocked.contains(root) || !walk.seen.insert(root.get_cs_name()) {
      continue;
    }
    walk.order.push(root.get_cs_name());
    if let Some(body) = file_body(root) {
      walk.visit(&body, 0);
    }
  }
  walk.order
}

/// The token body of `cs` when a raw file defined it.
fn file_body(cs: &Token) -> Option<Tokens> {
  match lookup_meaning(cs) {
    Some(Stored::Expandable(defn)) if defn.get_origin() == DefinitionOrigin::File => {
      match defn.get_expansion() {
        Some(ExpansionBody::Tokens(body)) => Some(body.clone()),
        _ => None,
      }
    },
    _ => None,
  }
}

/// A sentinel value of the speculative title-page runs ([`before_title_page_runs`]), one per store and kind. `Letters`:
/// `LxSentinel`, its index spelled in the letters `a`–`p`, and `zZ` — in mixed case, so that a class's `\MakeUppercase`
/// and its `\MakeLowercase` both show (`confirmed_printed`) —, matched lower-cased ([`sentinel_found`]);
/// `Digits`: its index in the digits `1`–`8` between `9090` and `0909`. No sentinel is part of another of its kind. A
/// store is printed only when both its sentinels reach the text (`sentinels_printed`): a numeric context consumes the
/// digits, while TeX's "Missing number" recovery typesets the letters — `\setcounter{page}{\@fp}` (gammas.cls:305,
/// kluwer.cls:1049; review probe `num`) prints neither value. A test of the value takes its branch with the sentinel's
/// value, not the document's: `\ifnum\@x<100 No.~\@x\fi` with 7 takes the other branch with the digits (the store
/// counts as not printed; pdflatex prints "No. 7"), and `\romannumeral\@va` prints neither sentinel (pdflatex: "iii") —
/// residuals, OXIDIZED_DESIGN_DIVERGENCES #767; a store tested against a word the class prints instead,
/// `\ifx\@vc\@open OPEN\else\@vc\fi`, is caught by the confirming run with the document's values
/// ([`confirmed_printed`]).
#[derive(Clone, Copy)]
enum Sentinel {
  Letters,
  Digits,
}

fn sentinel(i: usize, kind: Sentinel) -> String {
  let (base, first) = match kind {
    Sentinel::Letters => (16, b'a'),
    Sentinel::Digits => (8, b'1'),
  };
  let mut digits = Vec::new();
  let mut n = i;
  loop {
    digits.push(first + (n % base) as u8);
    n /= base;
    if n == 0 {
      break;
    }
  }
  digits.reverse();
  let index = String::from_utf8_lossy(&digits);
  match kind {
    Sentinel::Letters => s!("LxSentinel{index}zZ"),
    Sentinel::Digits => s!("9090{index}0909"),
  }
}

/// The sentinel `i` of `kind` as it is looked for in a run's lower-cased text.
fn sentinel_found(i: usize, kind: Sentinel) -> String { sentinel(i, kind).to_lowercase() }

/// `\def<store>{<sentinel i>}` for each store, the `i`-th its index.
fn sentinel_definitions(stores: &[Token], kind: Sentinel) -> Vec<Token> {
  let mut defs = Vec::new();
  for (i, store) in stores.iter().enumerate() {
    defs.push(T_CS!("\\def"));
    defs.push(*store);
    defs.push(T_BEGIN!());
    defs.extend(mouth::tokenize_internal(TeXString::assembled(sentinel(i, kind))).unlist());
    defs.push(T_END!());
  }
  defs
}

/// The stores both of whose sentinels ([`sentinel_definitions`], same order) the two speculative runs of the same
/// title-page code printed — `letters` and `digits`, their texts —, in the order the first printed them, as a
/// comma-separated list of their names.
fn sentinels_printed(stores: &[Token], letters: &str, digits: &str) -> String {
  let mut found: Vec<(usize, String)> = stores
    .iter()
    .enumerate()
    .filter(|(i, _)| digits.contains(&sentinel(*i, Sentinel::Digits)))
    .filter_map(|(i, store)| {
      letters
        .find(&sentinel_found(i, Sentinel::Letters))
        .map(|at| (at, store.with_str(ToString::to_string)))
    })
    .collect();
  found.sort();
  found
    .into_iter()
    .map(|(_, name)| name)
    .collect::<Vec<_>>()
    .join(",")
}

/// The stores the title-page code `build` makes (given the sentinel definitions to make before it runs) prints, each
/// given its sentinel of each kind in two speculative runs ([`sentinels_printed`]; the second only when the first
/// printed a letters sentinel), with the first run's text in its own case. `None` when a run cannot be made. A run that
/// ended in a Fatal of its own ([`SpeculativeRun::Aborted`]) printed nothing when `aborted_printed_nothing` — the run of
/// the whole title page: a store is not handed on on the word of a run that died —, else it is no run either: the
/// deposit's and the replay's (`None`, so the code walk decides, `walked_title_page_reads`) — a Fatal the sentinel value
/// alone caused does not stop the real deposit printing the store, which would then be handed on as well.
fn sentinel_runs(
  stores: &[Token],
  build: &dyn Fn(&[Token]) -> Tokens,
  aborted_printed_nothing: bool,
) -> Result<Option<(String, String)>> {
  use crate::latex_constructs::{SpeculativeRun, speculative_title_page_run};
  let letters =
    match speculative_title_page_run(build(&sentinel_definitions(stores, Sentinel::Letters)))? {
      SpeculativeRun::Printed(text) => text,
      SpeculativeRun::Aborted if aborted_printed_nothing => {
        return Ok(Some((String::new(), String::new())));
      },
      SpeculativeRun::Aborted | SpeculativeRun::Unavailable => return Ok(None),
    };
  let lowered = letters.to_lowercase();
  if !lowered.contains("lxsentinel") {
    return Ok(Some((String::new(), letters)));
  }
  let digits =
    match speculative_title_page_run(build(&sentinel_definitions(stores, Sentinel::Digits)))? {
      SpeculativeRun::Printed(text) => text,
      SpeculativeRun::Aborted if aborted_printed_nothing => {
        return Ok(Some((String::new(), letters)));
      },
      SpeculativeRun::Aborted | SpeculativeRun::Unavailable => return Ok(None),
    };
  Ok(Some((
    sentinels_printed(stores, &lowered, &digits),
    letters,
  )))
}

/// A store's value as plain text — characters and spaces only, at least one character —, without its spaces; `None`
/// for any other value (a macro, an accent, a group), whose printed form is not its tokens.
fn plain_value_text(value: &Tokens) -> Option<String> {
  let mut text = String::new();
  for t in value.unlist_ref() {
    match t.get_catcode() {
      Catcode::LETTER | Catcode::OTHER => text.push_str(&t.to_string()),
      Catcode::SPACE => {},
      _ => return None,
    }
  }
  (!text.is_empty()).then_some(text)
}

/// The letters and digits of a text, as the comparisons of source and typeset text read it ([`confirmed_printed`],
/// [`printed_right_before`]): what TeX typesets of `'`, ``` `` ``` and `''` is ’, “ and ”, of `--` a dash, and the spaces
/// are the layout's — only the alphanumerics are the same on both sides (`O'Brien`, ``` ``Quoted'' words```; review
/// probes `q3`, `q4`, `ap`).
fn alphanumerics(text: &str) -> String { text.chars().filter(|c| c.is_alphanumeric()).collect() }

/// The confirming check of the two-sentinel rule ([`before_title_page_runs`]): of the stores the sentinel runs found
/// printed (`printed`, names; `letters`, the letters run's text in its own case), those whose value is plain text
/// ([`plain_value_text`]) stay printed only when its letters and digits ([`alphanumerics`]) are in the title page printed
/// with the document's own values (`real`) — in their case, unless the run printed the store's mixed-case sentinel in
/// another case (a class's `\MakeUppercase` or `\MakeLowercase`). A class that tests the value and prints its own word
/// instead, `\ifx\@vc\@open OPEN\else\@vc\fi` with "open" (review probe `w`), printed the letters sentinel, which is no
/// "open", but not the value. A value with no letter or digit is not checked.
fn confirmed_printed(stores: &[Token], printed: &str, letters: &str, real: &str) -> String {
  let real = alphanumerics(real);
  let real_lowered = real.to_lowercase();
  printed
    .split(',')
    .filter(|n| !n.is_empty())
    .filter(|name| {
      let Some(i) = stores.iter().position(|t| t.with_str(|cs| cs == *name)) else {
        return true;
      };
      let Some(value) = parameterless_body(&stores[i])
        .as_ref()
        .and_then(plain_value_text)
        .map(|text| alphanumerics(&text))
        .filter(|text| !text.is_empty())
      else {
        return true;
      };
      if letters.contains(&sentinel(i, Sentinel::Letters)) {
        real.contains(&value)
      } else {
        real_lowered.contains(&value.to_lowercase())
      }
    })
    .collect::<Vec<_>>()
    .join(",")
}

/// A label's text ([`LabelText`]) as the title page prints it, lower-cased, its letters and digits only
/// ([`alphanumerics`]): its characters, and a class macro that is text read as its characters.
fn label_plain_text(label: &Tokens) -> String {
  let mut out = String::new();
  for t in label.unlist_ref() {
    match t.get_catcode() {
      Catcode::LETTER | Catcode::OTHER => out.push_str(&t.to_string()),
      Catcode::CS => {
        if let Some(body) = parameterless_body(t) {
          for e in body.unlist_ref() {
            if matches!(e.get_catcode(), Catcode::LETTER | Catcode::OTHER) {
              out.push_str(&e.to_string());
            }
          }
        }
      },
      _ => {},
    }
  }
  alphanumerics(&out).to_lowercase()
}

/// Does `text` (lower-cased) print `label` ([`label_plain_text`]) right before an occurrence of `sentinel`, its letters
/// and digits compared ([`alphanumerics`]: the typeset ’ of the class's `Author's keywords:`)?
fn printed_right_before(text: &str, sentinel: &str, label: &str) -> bool {
  if label.is_empty() {
    return false;
  }
  let mut from = 0;
  while let Some(found) = text[from..].find(sentinel) {
    let at = from + found;
    if alphanumerics(&text[..at]).ends_with(label) {
      return true;
    }
    from = at + sentinel.len();
  }
  false
}

/// At `\maketitle`, before its stores are handed on (`\lx@store@defaults`): the labels read off the class's code
/// ([`ClassReaders::title_page_label`] — `lx_store_label_<name>` of a rerouted store, `lx_offtable_label_<name>` of an
/// off-table one) stay the entries' names only when TeX's output says so: one speculative run of the class's whole
/// title page (`title_page_print_tokens`), each labelled store given its letters sentinel, must print the label's text
/// right before the store's sentinel ([`printed_right_before`]). A label the run does not print there is none: the URL
/// argument's "mailto:" of `\href{mailto:\@vg}{\@vg}`, whose page shows only the address (review probe `ctx3`). Without
/// a run (none can be made, or it ended in a Fatal of its own), a label in a group that is a control sequence's argument
/// (`in_argument`) is none either. A rejected label stays in the title page, and the entry is unnamed
/// (`lx_store_api_unnamed_<name>`). Once per document.
fn verify_title_page_labels() -> Result<()> {
  use crate::latex_constructs::{
    SpeculativeRun, speculative_title_page_run, title_page_print_tokens,
  };
  if lookup_bool("lx_depositing_class_maketitle")
    || lookup_bool("lx_replaying_class_setter")
    || lookup_bool("lx_title_page_labels_verified")
  {
    return Ok(());
  }
  // (whose label: a rerouted setter's name, or an off-table one's; its store; the label's text; in an argument)
  let mut labelled: Vec<(bool, String, Token, String, bool)> = Vec::new();
  for name in rerouted_names() {
    if let Some(Stored::Tokens(label)) = lookup_value(&s!("lx_store_label_{name}")) {
      let store = T_CS!(&s!("\\{}", store_of(&name)));
      let in_argument = lookup_bool(&s!("lx_store_label_in_argument_{name}"));
      labelled.push((true, name, store, label_plain_text(&label), in_argument));
    }
  }
  for name in lookup_string("lx_offtable_stores")
    .split(',')
    .filter(|n| !n.is_empty())
  {
    if let Some(Stored::Tokens(label)) = lookup_value(&s!("lx_offtable_label_{name}"))
      && let Some(Stored::Token(store)) = lookup_value(&s!("lx_offtable_store_{name}"))
    {
      let in_argument = lookup_bool(&s!("lx_offtable_label_in_argument_{name}"));
      labelled.push((
        false,
        name.to_string(),
        store,
        label_plain_text(&label),
        in_argument,
      ));
    }
  }
  if labelled.is_empty() {
    return Ok(());
  }
  assign_value("lx_title_page_labels_verified", true, Some(Scope::Global));
  let mut stores: Vec<Token> = Vec::new();
  for (_, _, store, ..) in &labelled {
    if !stores.contains(store) {
      stores.push(*store);
    }
  }
  let run = speculative_title_page_run(title_page_print_tokens(&sentinel_definitions(
    &stores,
    Sentinel::Letters,
  )))?;
  let text = match run {
    SpeculativeRun::Printed(text) => Some(text.to_lowercase()),
    SpeculativeRun::Aborted | SpeculativeRun::Unavailable => None,
  };
  for (rerouted, name, store, label, in_argument) in labelled {
    let kept = match &text {
      Some(text) => {
        let i = stores.iter().position(|t| *t == store).unwrap_or_default();
        printed_right_before(text, &sentinel_found(i, Sentinel::Letters), &label)
      },
      None => !in_argument,
    };
    DebugFeature!(
      "frontmatter",
      "FRONT title-page label of \\{name} ({label}): {}",
      if kept {
        "printed right before the store"
      } else {
        "not printed there"
      }
    );
    if kept {
      continue;
    }
    if rerouted {
      let unnamed = lookup_string(&s!("lx_store_api_unnamed_{name}"));
      if !unnamed.is_empty() {
        assign_value(
          &s!("lx_store_api_{name}"),
          Stored::String(pin(unnamed)),
          Some(Scope::Global),
        );
      }
      for key in ["lx_store_label_part_", "lx_store_label_"] {
        assign_value(&s!("{key}{name}"), false, Some(Scope::Global));
      }
    } else {
      assign_value(&s!("lx_offtable_label_{name}"), false, Some(Scope::Global));
    }
    Info!(
      "frontmatter",
      s!("\\{name}"),
      s!(
        "The class's title page does not print the label read before the store of \\{name} right before it: the entry has no name"
      )
    );
  }
  Ok(())
}

/// The stores a speculative run recorded (`lx_sentinel_<run>`): their names.
fn sentinel_list(run: &str) -> Vec<SymStr> {
  lookup_string(&s!("lx_sentinel_{run}"))
    .split(',')
    .filter(|n| !n.is_empty())
    .map(|n| T_CS!(n).get_cs_name())
    .collect()
}

/// The off-table stores ([`survey_off_table_stores`]) the document set — a value no longer the class's default and not
/// blank —, once per store (two setters of one store, sigplanconf.cls:939-942 `\copyrightDOI` and its `\let`
/// `\copyrightdoi`: one value): each setter's name, its store and the value.
fn offtable_candidates() -> Vec<(String, Token, Tokens)> {
  let mut out: Vec<(String, Token, Tokens)> = Vec::new();
  for name in lookup_string("lx_offtable_stores")
    .split(',')
    .filter(|n| !n.is_empty())
  {
    let Some(Stored::Token(store)) = lookup_value(&s!("lx_offtable_store_{name}")) else {
      continue;
    };
    if out.iter().any(|(_, s, _)| *s == store) {
      continue;
    }
    let Some(value) = parameterless_body(&store) else {
      continue;
    };
    let default = match lookup_value(&s!("lx_offtable_default_{name}")) {
      Some(Stored::Tokens(default)) => Some(default),
      _ => None,
    };
    if is_blank(&value)
      || default
        .as_ref()
        .is_some_and(|d| d.unlist_ref() == value.unlist_ref())
    {
      continue;
    }
    out.push((name.to_string(), store, value));
  }
  out
}

/// The rerouted stores ([`reroute_raw_class_stores`]): each setter's name.
fn rerouted_names() -> Vec<String> {
  lookup_string("lx_rerouted_stores")
    .split(',')
    .filter(|n| !n.is_empty())
    .map(String::from)
    .collect()
}

/// Before the title-page deposit and replay of a raw class's title page (`\lx@deposit@maketitle`, sect05.rs), in the
/// state they meet: what the title page prints is decided by TeX, not read off its code. Each candidate store is given
/// a sentinel value and the title page is digested speculatively (`speculative_title_page_run`: output discarded,
/// assignments undone, diagnostics discarded), once with letters sentinels and once with digits ones
/// (`sentinel_runs`); a store both of whose sentinels reach the digested text is printed there (a numeric context —
/// `\setcounter{page}{\@fp}` — consumes the digits, TeX's recovery typesets the letters). A class's own booleans (sigplanconf.cls:87-90 `\def\@true{TT}`, :1118 `\if \@copyrightwanted
/// \@copyrightspace \fi`: 1603.03011 `[preprint,nocopyrightspace]`), its `\newif` switches (aiaa-tc.cls:199-200 under
/// `[handcarry]`), its `\@ifundefined` and `\ifx…\@empty` tests and what it only puts in a box or a running head
/// (`\setbox`, `\markboth`: 1704.03582, 1508.03599) are then what they are in pdflatex. Three runs:
/// - the class's whole title page with the document's values (`title_page_print_tokens`), the off-table stores the
///   document set (`offtable_candidates`) and the rerouted stores handed to the frontmatter as candidates
///   (`lx_sentinel_printed`): an off-table store is printed there (`hand_untypeset_stores`); a handed one was placed
///   by this `\maketitle` (#6b, `\lx@store@hand`);
/// - the deposit and the replay, each as it will run (`title_page_deposit_tokens`, `title_page_replay_tokens`),
///   the off-table candidates given sentinels (`lx_sentinel_deposit`, `lx_sentinel_replay`): a store either typesets,
///   when it is kept, is not handed on a second time.
///
/// Before them, the value of each rerouted store the document never set that is still undefined
/// (`decide_unset_nulls`). When no run can be made (a journal is open), or the deposit's or the replay's ended in a Fatal
/// of its own (`sentinel_runs`), `lx_sentinel_ran` stays false and the stores' readers are walked instead
/// (`live_read_order`, `evaluated_switch`).
pub fn before_title_page_runs() -> Result<()> {
  if lookup_bool("lx_depositing_class_maketitle") || lookup_bool("lx_replaying_class_setter") {
    return Ok(());
  }
  assign_value("lx_sentinel_ran", false, Some(Scope::Global));
  let rerouted = rerouted_names();
  let offtable = if lookup_bool("lx_offtable_handed") {
    Vec::new()
  } else {
    offtable_candidates()
  };
  if rerouted.is_empty() && offtable.is_empty() {
    return Ok(());
  }
  decide_unset_nulls(&rerouted)?;
  let off_stores: Vec<Token> = offtable.iter().map(|(_, store, _)| *store).collect();
  let mut stores = off_stores.clone();
  for name in &rerouted {
    let store = T_CS!(&s!("\\{}", store_of(name)));
    if lookup_bool(&s!("lx_store_handed_{name}")) && !stores.contains(&store) {
      stores.push(store);
    }
  }
  if stores.is_empty() {
    return Ok(());
  }
  use crate::latex_constructs::{
    title_page_deposit_tokens, title_page_print_tokens, title_page_replay_tokens,
  };
  let no_run = || {
    Info!(
      "frontmatter",
      "\\maketitle",
      "No speculative run of the class's title page could be made or kept (an assignment journal is open, or the run could not be undone): what it prints is read off its code"
    );
  };
  let Some((printed, letters)) = sentinel_runs(&stores, &title_page_print_tokens, true)? else {
    no_run();
    return Ok(());
  };
  // (the confirming check with the document's own values: one more run when a store found printed has a plain value)
  let printed = if printed.split(',').any(|name| {
    stores
      .iter()
      .find(|t| t.with_str(|cs| cs == name))
      .and_then(parameterless_body)
      .as_ref()
      .and_then(plain_value_text)
      .is_some()
  }) {
    match crate::latex_constructs::speculative_title_page_run(title_page_print_tokens(&[]))? {
      crate::latex_constructs::SpeculativeRun::Printed(real) => {
        confirmed_printed(&stores, &printed, &letters, &real)
      },
      _ => printed,
    }
  } else {
    printed
  };
  DebugFeature!(
    "frontmatter",
    "FRONT title-page sentinels: printed [{printed}] of {} candidate stores",
    stores.len()
  );
  let (mut deposit, mut replay) = (String::new(), String::new());
  if !off_stores.is_empty() {
    match sentinel_runs(&off_stores, &title_page_deposit_tokens, false)? {
      Some((list, _)) => deposit = list,
      None => {
        no_run();
        return Ok(());
      },
    }
    if title_page_replay_tokens(&[]).is_some() {
      let replay_tokens =
        |defs: &[Token]| title_page_replay_tokens(defs).unwrap_or_else(|| Tokens::new(Vec::new()));
      match sentinel_runs(&off_stores, &replay_tokens, false)? {
        Some((list, _)) => replay = list,
        None => {
          no_run();
          return Ok(());
        },
      }
    }
  }
  for (run, list) in [
    ("printed", printed),
    ("deposit", deposit),
    ("replay", replay),
  ] {
    assign_value(
      &s!("lx_sentinel_{run}"),
      Stored::String(pin(list)),
      Some(Scope::Global),
    );
  }
  assign_value("lx_sentinel_ran", true, Some(Scope::Global));
  Ok(())
}

/// Silent-loss 77h/23b: the value a rerouted store the document never set, still undefined, takes in the title-page
/// deposit and replay (`\lx@captured@stores`): the one under which the title page prints fewer characters — a rule over
/// the text TeX typesets for each candidate in speculative runs of the deposit and replay as they will run
/// (`speculative_title_page_run`). Undefined, it
/// raised an error that dropped the whole deposit — an.cls:532 `\ch@ckobl{institute}{…}`, which runs in the class's own
/// `\maketitle` the kernel's replaces, took the paper's "Key words" line with it (row 23b). `\relax` keeps a class's
/// "is it undefined?" test true: ptptex.cls:88-104 prints its journal line and "(Received …)" unless
/// `\@ifundefined{@pubinfo}`/`{@recdate}` (hep-th9803240, 1309.6905, 0911.1235, 0708.3877, 0801.2466, 1009.4028,
/// 1007.1507, cond-mat0411234; jpsj: 0903.1491, 0912.1097, 1101.2977, 1203.5392); `\@empty` keeps an emptiness test
/// true: `\ifx\@institute\@empty\else Institute: \@institute\fi` after the class's own `\maketitle` defaulted it to
/// `\@empty`. All `\relax` first, then all `\@empty`: when the title page prints as much text, all stay `\relax`; else
/// each store alone `\@empty`, kept so when the title page then prints less (a tie keeps `\relax`). Recorded as
/// `lx_store_unset_empty_<name>`.
fn decide_unset_nulls(rerouted: &[String]) -> Result<()> {
  use crate::latex_constructs::{
    SpeculativeRun, speculative_title_page_run, title_page_deposit_tokens, title_page_replay_tokens,
  };
  let unset: Vec<&String> = rerouted
    .iter()
    .filter(|name| {
      let key = s!("lx_store_unset_empty_{name}");
      assign_value(&key, false, Some(Scope::Global));
      !lookup_bool(&s!("lx_store_handed_{name}"))
        && lookup_meaning(&T_CS!(&s!("\\{}", store_of(name)))).is_none()
    })
    .collect();
  // (a deposit runs unless `\@maketitle` is the kernel's empty one)
  let deposits = match lookup_meaning(&T_CS!("\\@maketitle")) {
    Some(Stored::Expandable(d)) => {
      matches!(d.get_expansion(), Some(ExpansionBody::Tokens(t)) if !t.is_empty())
    },
    _ => true,
  };
  let replay = title_page_replay_tokens(&[]);
  if unset.is_empty() || (!deposits && replay.is_none()) {
    return Ok(());
  }
  // (`None`: no run, or one that ended in a Fatal of its own — which is no "fewer characters": the value that killed
  // its run is not chosen, `\relax` stays)
  let ink = || -> Result<Option<usize>> {
    let mut tokens = title_page_deposit_tokens(&[]).unlist();
    if let Some(replay) = title_page_replay_tokens(&[]) {
      tokens.extend(replay.unlist());
    }
    Ok(match speculative_title_page_run(Tokens::new(tokens))? {
      SpeculativeRun::Printed(text) => Some(text.chars().filter(|c| !c.is_whitespace()).count()),
      SpeculativeRun::Aborted | SpeculativeRun::Unavailable => None,
    })
  };
  let Some(base) = ink()? else {
    return Ok(());
  };
  // (all `\@empty` first: when the title page prints as much, no store's value matters)
  let keys: Vec<String> = unset
    .iter()
    .map(|name| s!("lx_store_unset_empty_{name}"))
    .collect();
  for key in &keys {
    assign_value(key, true, Some(Scope::Global));
  }
  let all_empty = ink()?;
  for key in &keys {
    assign_value(key, false, Some(Scope::Global));
  }
  if all_empty.is_none_or(|n| n == base) {
    return Ok(());
  }
  for name in &unset {
    let key = s!("lx_store_unset_empty_{name}");
    assign_value(&key, true, Some(Scope::Global));
    let emptier = ink()?.is_some_and(|n| n < base);
    assign_value(&key, emptier, Some(Scope::Global));
    if emptier {
      Info!(
        "frontmatter",
        s!("\\{name}"),
        s!("The unset store of \\{name} is empty in the title page: undefined it would print more")
      );
    }
  }
  // (each decided alone: chosen together they are what each was chosen against, unless they interact)
  Ok(())
}

/// The stores whose values the frontmatter owns: the kernel's title fields (and their short forms) and the raw class's
/// stores K11 hands to the frontmatter — the rerouted table setters' (`reroute_raw_class_stores`) and the off-table
/// ones (`survey_off_table_stores`). A class command run in place after the document body began puts them back as
/// they were (sect05.rs `sealed_storing_run`, silent-loss #8).
pub fn frontmatter_owned_stores() -> Vec<Token> {
  let mut stores: Vec<Token> = KERNEL_FIELDS
    .iter()
    .chain(&["\\@shorttitle", "\\@shortauthor"])
    .map(|cs| T_CS!(*cs))
    .collect();
  for name in rerouted_names() {
    stores.push(T_CS!(&s!("\\{}", store_of(&name))));
  }
  for name in lookup_string("lx_offtable_stores")
    .split(',')
    .filter(|n| !n.is_empty())
  {
    if let Some(Stored::Token(store)) = lookup_value(&s!("lx_offtable_store_{name}")) {
      stores.push(store);
    }
  }
  stores.dedup();
  stores
}

/// Silent-loss #6b: the rerouted stores this `\maketitle` placed — handed to the frontmatter and printed by the class's
/// title page (`lx_sentinel_printed`, [`before_title_page_runs`]; without a speculative run, read by it:
/// `lx_store_title_page_reads_<name>`) — whose setter, called again, only re-sets the store (`\lx@store@hand`). A store
/// the title page does not print stays the later call's: a class abstract's end code `Keywords: \@keywords` prints the
/// value set after `\maketitle` (review probe `late`).
fn settle_placed_stores() {
  for name in rerouted_names() {
    let placed = title_page_prints_store(&name) && lookup_bool(&s!("lx_store_handed_{name}"));
    assign_value(&s!("lx_store_placed_{name}"), placed, Some(Scope::Global));
  }
}

/// The state naming the store whose frontmatter entries are being queued (`\lx@store@from`): their title-page
/// provenance (`frontmatter_push`, base_utilities `TITLE_PAGE_ENTRY`: a late abstract's run holding one the title page
/// may print, [`title_page_may_print`], is not moved, OXIDIZED_DESIGN_DIVERGENCES #903).
pub const STORE_ENTRY_FROM: &str = "lx_store_entry_from";

/// `\lx@store@from{<name>}`: the entries queued next are handed on from the store `\<name>`; empty, from none.
fn store_from_tokens(name: &str) -> Vec<Token> {
  mouth::tokenize_internal(TeXString::assembled(s!("\\lx@store@from{{{name}}}"))).unlist()
}

/// Digest the frontmatter calls of handed-on stores, `\lx@store@from`-bracketed, and clear the provenance after them
/// however the digestion ended: a call that stopped before its closing `\lx@store@from{}` stamps no later entry.
fn digest_store_calls(calls: Vec<Token>) -> Result<Digested> {
  let digested = digest(Tokens::new(calls));
  assign_value(
    STORE_ENTRY_FROM,
    Stored::String(pin("")),
    Some(Scope::Global),
  );
  digested
}

/// May the class's title page print the store of `\<name>`, whose entry carries its title-page provenance
/// (OXIDIZED_DESIGN_DIVERGENCES #903)? Unless the speculative run of the whole title page was made and its sentinel did
/// not reach it ([`before_title_page_runs`], `lx_sentinel_printed`): a store the class prints elsewhere — dmtcs's
/// `\keywords` after its abstract, 0905.1102 — is not the title page's; with no run made, it may be.
pub fn title_page_may_print(name: &str) -> bool {
  !lookup_bool("lx_sentinel_ran") || {
    let store = T_CS!(&s!("\\{}", store_of(name))).get_cs_name();
    sentinel_list("printed").contains(&store)
  }
}

/// Does the class's title page print the store of `\<name>`: its sentinel reached the speculative run of the whole title
/// page (`lx_sentinel_printed`, [`before_title_page_runs`]) — without a run, its code reads it
/// (`lx_store_title_page_reads_<name>`)?
fn title_page_prints_store(name: &str) -> bool {
  if lookup_bool("lx_sentinel_ran") {
    let store = T_CS!(&s!("\\{}", store_of(name))).get_cs_name();
    sentinel_list("printed").contains(&store)
  } else {
    lookup_bool(&s!("lx_store_title_page_reads_{name}"))
  }
}

/// Silent-loss #6a: an abstract or keywords store `\maketitle` handed on is the class's own entry (`_store`,
/// [`labelled_api`]), which the document's own abstract does not replace, only when the title page prints it
/// ([`title_page_prints_store`]) — not when its code only reads it: `\hypersetup{pdfsubject={\@abst}}` in `\@maketitle`
/// beside the document's `{abstract}`, which pdflatex prints once (review probe `md`). Such an entry is an ordinary one
/// (base_utilities.rs `frontmatter_release_store`), as before the class's title page was consulted.
fn release_unprinted_store_entries() {
  for name in rerouted_names() {
    if lookup_bool(&s!("lx_store_deferred_{name}"))
      && lookup_bool(&s!("lx_store_handed_{name}"))
      && !title_page_prints_store(&name)
    {
      frontmatter_release_store(&store_of(&name));
    }
  }
}

/// The frontmatter call `api` of the store `\<name>` as an ordinary entry's, without the class's store's tag
/// ([`labelled_api`]): a value no title page printed — set where no `\maketitle` follows, or after the one that read the
/// store.
fn untagged_api(api: &str, name: &str) -> String {
  let tag = s!("_store={}", store_of(name));
  api
    .replacen(&s!("[{tag}]"), "", 1)
    .replacen(&s!("{tag},"), "", 1)
}

/// Does the class's own refused `\<field>` (`\lx@dropped@<field>`, state.rs) hand its argument to the setter `\<name>`
/// (jfpc.cls:122 `\renewcommand{\title}[1]{\titre{#1}}`, :126 `\author` → `\auteurs`)? Then that setter's store is the
/// class's record of the field.
fn class_field_setter(field: &str, name: &str) -> bool {
  let Some(body) = parameterless_or_any_body(&T_CS!(&s!("\\lx@dropped@{field}"))) else {
    return false;
  };
  let body = body.unlist_ref();
  let setter = T_CS!(&s!("\\{name}"));
  body.iter().enumerate().any(|(i, t)| {
    *t == setter && {
      let k = skip_spaces(body, i + 1);
      body
        .get(k)
        .is_some_and(|b| b.get_catcode() == Catcode::BEGIN)
        && body
          .get(k + 1)
          .is_some_and(|b| b.get_catcode() == Catcode::ARG)
        && body
          .get(k + 2)
          .is_some_and(|b| b.get_catcode() == Catcode::END)
    }
  })
}

/// The token body of `cs`, whatever its parameters.
fn parameterless_or_any_body(cs: &Token) -> Option<Tokens> {
  match lookup_meaning(cs) {
    Some(Stored::Expandable(defn)) => match defn.get_expansion() {
      Some(ExpansionBody::Tokens(body)) => Some(body.clone()),
      _ => None,
    },
    _ => None,
  }
}

/// [`hand_untypeset_stores`] without a speculative run ([`before_title_page_runs`]): what the class's title page reads,
/// walked through its live code (the switches as they are, `evaluated_switch`), in reading order — the code of its
/// refused `\maketitle`, else of LaTeX's, which runs `\@maketitle` and, when the class has none, prints the kernel
/// fields — and what a kept deposit or replay may have typeset (any code it reaches, a stored definition's too).
fn walked_title_page_reads() -> (Vec<SymStr>, FxHashSet<SymStr>) {
  let fields: Vec<Token> = KERNEL_FIELDS.iter().map(|cs| T_CS!(*cs)).collect();
  let dropped = T_CS!("\\lx@dropped@maketitle");
  let maketitle = T_CS!("\\@maketitle");
  let printed_roots: Vec<Token> = if file_body(&dropped).is_some() {
    vec![dropped]
  } else if file_body(&maketitle).is_some() {
    vec![maketitle]
  } else {
    fields.clone()
  };
  let mut blocked = fields.clone();
  blocked.extend(["\\thetitle", "\\theauthor", "\\thedate"].map(|cs| T_CS!(cs)));
  for name in rerouted_names() {
    if lookup_bool(&s!("lx_store_handed_{name}")) {
      blocked.push(T_CS!(&s!("\\{}", store_of(&name))));
    }
  }
  let printed = live_read_order(&printed_roots, &[], true);
  let mut typeset: FxHashSet<SymStr> = FxHashSet::default();
  if lookup_bool("lx_title_page_deposit_kept") {
    typeset.extend(live_read_order(&[maketitle], &blocked, false));
  }
  if lookup_bool("lx_title_page_replay_kept") {
    let mut replay_blocked = blocked.clone();
    replay_blocked.push(maketitle);
    typeset.extend(live_read_order(&[dropped], &replay_blocked, false));
  }
  (printed, typeset)
}

/// Silent-loss #5 (a). At `\maketitle`, after the title-page deposit and replay (`\lx@store@untypeset`): each off-table
/// store the document set ([`offtable_candidates`]) that the class's title page prints — its sentinel reached the
/// speculative run of the whole title page ([`before_title_page_runs`]; without one, [`walked_title_page_reads`]) — and
/// that no title-page run kept here typeset (its sentinel did not reach the run of the deposit or replay that was kept)
/// is handed to the frontmatter, in the order the title page prints them, with no visible label of ours (the
/// meta XSLT shows a note's `@role` as its label, LaTeXML-meta-xhtml.xsl:79-85): a store the class's own refused
/// `\title` hands its argument to ([`class_field_setter`]) is the document's `ltx:title` when it has none; the store of
/// the names the class's own `\@author` prints — the one its refused `\author` hands on, else the first store it reads
/// (`lx_offtable_author_reads`) — its authors when it has none (jfpc.cls:122-126, 176-191: 1503.05530), and a store its
/// `\@author` prints beside the names rows of their author block (`\lx@add@store@authorblock`), attached to the creators
/// as any affiliation is, not a note on the document (both are hover popups under the default `ltx_authors_1line`
/// layout, LaTeXML.css:74-88 and :107-120: the gain is what the rows are, not where they show); a store that only repeats the creators' names is not handed on ([`repeats_creator_names`]:
/// nowfnt's names-only cover list, 2308.02698); any other an `ltx:pubnote` (Base_Utility.pool.ltxml:634, a note about
/// the document), its `\and` between authors' blocks a line break (2108.12618), named by the label the title page prints
/// before it (`lx_offtable_label_<name>`) when it prints one. A run that typeset it keeps it, so nothing is shown
/// twice; a store the title page prints only under a switch that is off — a `\newif` (aiaa-tc.cls:199-200
/// `\aiaa@conference` under `[handcarry]`), a class's own boolean (sigplanconf.cls:1118, 1603.03011's "nnnnnnn.nnnnnnn"
/// placeholder DOI) — or only in a box or a running head is not printed and not handed on. Perl drops them all (no
/// binding, its `\@maketitle` relaxed, latex_constructs.pool.ltxml:1105).
fn hand_untypeset_stores() -> Result<Vec<Digested>> {
  if lookup_string("lx_offtable_stores").is_empty() || lookup_bool("lx_offtable_handed") {
    return Ok(Vec::new());
  }
  assign_value("lx_offtable_handed", true, Some(Scope::Global));
  let candidates = offtable_candidates();
  if candidates.is_empty() {
    return Ok(Vec::new());
  }
  let (printed, typeset) = if lookup_bool("lx_sentinel_ran") {
    let mut typeset: FxHashSet<SymStr> = FxHashSet::default();
    if lookup_bool("lx_title_page_deposit_kept") {
      typeset.extend(sentinel_list("deposit"));
    }
    if lookup_bool("lx_title_page_replay_kept") {
      typeset.extend(sentinel_list("replay"));
    }
    (sentinel_list("printed"), typeset)
  } else {
    walked_title_page_reads()
  };
  // in the order the title page prints them
  let mut handed: Vec<(usize, &(String, Token, Tokens))> = candidates
    .iter()
    .filter_map(|candidate| {
      let key = candidate.1.get_cs_name();
      printed
        .iter()
        .position(|cs| *cs == key)
        .filter(|_| !typeset.contains(&key))
        .map(|at| (at, candidate))
    })
    .collect();
  handed.sort_by_key(|(at, _)| *at);
  // The stores the class's own `\@author` reads (`lx_offtable_author_reads`, jfpc.cls:188-191): the one its refused
  // `\author` hands its argument to ([`class_field_setter`]), else the first, holds the names; the others beside it are
  // the author block's rows (`\lx@add@store@authorblock`).
  let author_reads = lookup_string("lx_offtable_author_reads");
  let author_reads: Vec<&str> = author_reads.split(',').filter(|n| !n.is_empty()).collect();
  let names_store = candidates
    .iter()
    .find(|(name, ..)| class_field_setter("author", name))
    .map(|(name, ..)| name.as_str())
    .or_else(|| {
      author_reads.first().and_then(|store| {
        candidates
          .iter()
          .find(|(_, cs, _)| cs.with_str(|n| n == *store))
          .map(|(name, ..)| name.as_str())
      })
    });
  let mut has_creator = has_front_matter("ltx:creator");
  let creator_words = creator_name_words();
  let mut calls: Vec<Token> = Vec::new();
  let mut rows: Vec<Token> = Vec::new();
  for (_, (name, store, value)) in handed {
    let in_author = store.with_str(|cs| author_reads.contains(&cs));
    let (head, what) = if class_field_setter("title", name) && !has_front_matter("ltx:title") {
      (
        mouth::tokenize_internal(r"\lx@add@title").unlist(),
        "the document's title",
      )
    } else if names_store == Some(name.as_str()) && !has_creator {
      has_creator = true;
      (
        mouth::tokenize_internal(r"\lx@add@authors@adaptive").unlist(),
        "the document's authors",
      )
    } else if in_author && names_store.is_some() && has_creator {
      // (after the names, which make the creators these rows annotate)
      rows.extend(
        mouth::tokenize_internal(TeXString::assembled(s!(
          "\\lx@add@store@authorblock{{{name}}}"
        )))
        .unlist(),
      );
      rows.push(T_BEGIN!());
      if let Some(Stored::Tokens(label)) = lookup_value(&s!("lx_offtable_label_{name}")) {
        rows.extend(label.unlist());
      }
      rows.extend(value.unlist_ref().iter().copied());
      rows.push(T_END!());
      Info!(
        "frontmatter",
        s!("\\{name}"),
        s!(
          "The class's title page prints the store of \\{name} in its author block, which no title-page run here typeset: it is handed on as rows of the authors"
        )
      );
      continue;
    } else if !matches!(
      lookup_value(&s!("lx_offtable_label_{name}")),
      Some(Stored::Tokens(_))
    ) && repeats_creator_names(value, &creator_words)
    {
      Info!(
        "frontmatter",
        s!("\\{name}"),
        s!(
          "The class's title page prints the store of \\{name}, which only repeats the authors' names: not handed on"
        )
      );
      continue;
    } else {
      let mut head = mouth::tokenize_internal(r"\lx@add@frontmatter{ltx:pubnote}").unlist();
      if let Some(Stored::Tokens(label)) = lookup_value(&s!("lx_offtable_label_{name}")) {
        head.extend(mouth::tokenize_internal("[name={").unlist());
        head.extend(label.unlist());
        head.extend(mouth::tokenize_internal("}]").unlist());
      }
      (head, "a note on the document")
    };
    // (a note the class's title page printed: its title-page provenance, base_utilities `TITLE_PAGE_ENTRY`)
    let note = what == "a note on the document";
    if note {
      calls.extend(store_from_tokens(name));
    }
    calls.extend(head);
    calls.push(T_BEGIN!());
    // (in a note, the class's `\and` between authors' blocks — nowfnt.cls:813-818 starts a new block — is a line break
    // between them: 2108.12618's cover list, which ran "…@gmail.com Matan Gavish")
    let and = T_CS!("\\and");
    calls.extend(value.unlist_ref().iter().map(|t| {
      if *t == and && what == "a note on the document" {
        T_CS!("\\\\")
      } else {
        *t
      }
    }));
    calls.push(T_END!());
    if note {
      calls.extend(store_from_tokens(""));
    }
    Info!(
      "frontmatter",
      s!("\\{name}"),
      s!(
        "The class's title page prints the store of \\{name}, which no title-page run here typeset: it is handed on as {what}"
      )
    );
  }
  calls.extend(rows);
  if calls.is_empty() {
    return Ok(Vec::new());
  }
  Ok(vec![digest_store_calls(calls)?])
}

/// The words of each creator's name in the frontmatter (each creator's own digested content, its personname; not the
/// contacts annotating it), for [`repeats_creator_names`].
fn creator_name_words() -> Vec<FxHashSet<String>> {
  let mut out: Vec<FxHashSet<String>> = Vec::new();
  with_value("frontmatter", |v| {
    if let Some(Stored::HashTagData(frontmatter)) = v
      && let Some(creators) = frontmatter.get("ltx:creator")
    {
      for creator in creators {
        let mut text = String::new();
        for item in &creator.content {
          if let TagContent::Box(d) = item {
            ink_text(d, &mut text);
            text.push(' ');
          }
        }
        let words: FxHashSet<String> = text.split_whitespace().map(String::from).collect();
        if !words.is_empty() {
          out.push(words);
        }
      }
    }
  });
  out
}

/// Does the store value `value` only repeat the creators' names — is every line of it (split at `\and`, `\\`,
/// `\newline` and `\par`) plain text, characters and spaces only, each of whose words is a word of a creator's name,
/// and does it name every creator (each creator's words are among its words: `creators`, [`creator_name_words`])? A
/// cover's author list that holds only the names (nowfnt.cls:803-806 `\maintitleauthorlist`, "AUTHORS FOR COVER
/// PAGE": 2308.02698) is a duplicate of the creators, a print flourish of the cover (the page-furniture ruling); one
/// that carries more (2108.12618's affiliations and e-mails) is not, nor is one that names some of the authors only — a
/// role, "Corresponding author: \@corr" with `\corresponding{Bob Jones}` (review probe `corr`; and a store the class
/// labels is never dropped, `hand_untypeset_stores`). By words, as the creators may hold a name in another order or
/// split (`\author[1]{Sung, Yoonchang}` gives two creators, SYNC_STATUS row "nowfnt creators").
fn repeats_creator_names(value: &Tokens, creators: &[FxHashSet<String>]) -> bool {
  if creators.is_empty() {
    return false;
  }
  let breaks = ["\\and", "\\\\", "\\newline", "\\par"].map(|cs| T_CS!(cs));
  let mut text = String::new();
  for t in value.unlist_ref() {
    match t.get_catcode() {
      Catcode::LETTER | Catcode::OTHER => text.push_str(&t.to_string()),
      Catcode::SPACE | Catcode::ACTIVE => text.push(' '),
      Catcode::BEGIN | Catcode::END => {},
      Catcode::CS if breaks.contains(t) => text.push(' '),
      _ => return false,
    }
  }
  let words: FxHashSet<&str> = text.split_whitespace().collect();
  !words.is_empty()
    && words
      .iter()
      .all(|word| creators.iter().any(|creator| creator.contains(*word)))
    && creators
      .iter()
      .all(|creator| creator.iter().all(|word| words.contains(word.as_str())))
}

/// The table's keywords setters: those whose frontmatter API is `\lx@add@keywords`.
fn keywords_setters() -> impl Iterator<Item = (&'static str, &'static str)> {
  STORE_SETTERS
    .iter()
    .copied()
    .filter(|(_, api)| api.starts_with("\\lx@add@keywords"))
}

/// The tokens of `body` from `i` on, spaces skipped.
fn skip_spaces(body: &[Token], mut i: usize) -> usize {
  while body
    .get(i)
    .is_some_and(|t| t.get_catcode() == Catcode::SPACE)
  {
    i += 1;
  }
  i
}

/// The index past the balanced group opening at `body[i]` (a `{`), or `None` when it does not open one or never closes.
fn group_end(body: &[Token], i: usize) -> Option<usize> {
  if body.get(i)?.get_catcode() != Catcode::BEGIN {
    return None;
  }
  let mut depth = 0usize;
  for (k, t) in body.iter().enumerate().skip(i) {
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => {
        depth -= 1;
        if depth == 0 {
          return Some(k + 1);
        }
      },
      _ => {},
    }
  }
  None
}

/// How a definition nested in a body makes its command: `\def`, `\gdef`, `\renewcommand` replace a meaning the name
/// has; `\newcommand` and `\providecommand` keep it (LaTeX's `\newcommand` of a defined name is an error that leaves it,
/// latex.ltx `\@ifdefinable`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Definer {
  Replaces,
  Keeps,
}

/// Where a definition's body first uses its argument `#1` — as stored in it (`#1`), or doubled in an enclosing
/// definition's body (`##1`, kept as a parameter token and `1`).
fn argument_at(body: &[Token]) -> Option<usize> {
  (0..body.len()).find(|&k| argument_length(body, k).is_some())
}

/// The number of tokens of a use of `#1` at `body[k]` ([`argument_at`]), if there is one there.
fn argument_length(body: &[Token], k: usize) -> Option<usize> {
  let t = body.get(k)?;
  if t.get_catcode() == Catcode::ARG && t.with_str(|s| s == "1") {
    return Some(1);
  }
  (t.get_catcode() == Catcode::PARAM
    && body
      .get(k + 1)
      .is_some_and(|d| d.get_catcode() != Catcode::PARAM && d.with_str(|s| s == "1")))
  .then_some(2)
}

/// A one-argument command `\<name>` that `body` defines — `\def\name#1{`, `\newcommand{\name}[1]{` (with any `##`
/// doubling and `\long`/`\global` prefix), and nothing that reads a delimited or optional argument: its definer and
/// its body, as stored (a doubled `##1` kept as a parameter token and `1`) — the first whose body uses its argument
/// ([`argument_at`]), else the first. A definition that drops or only warns about its argument is not a setter of it
/// (beilstein.cls:854 `\newcommand{\keywords}[1]{\ClassWarning{beilstein}{Keywords should not be part of …}}` beside
/// :853's typesetting one, fbithesis.cls:272 `{\fbi@warning{\keywords}}`). `None` when it defines none.
fn nested_one_arg_definition(body: &[Token], name: &str) -> Option<(Definer, Vec<Token>)> {
  let target = T_CS!(&s!("\\{name}"));
  let mut first: Option<(Definer, Vec<Token>)> = None;
  for (i, t) in body.iter().enumerate() {
    if t.get_catcode() != Catcode::CS {
      continue;
    }
    let definer = t.with_str(|s| match s {
      "\\def" | "\\gdef" | "\\edef" | "\\xdef" => Some((true, Definer::Replaces)),
      "\\renewcommand" | "\\DeclareRobustCommand" => Some((false, Definer::Replaces)),
      "\\newcommand" | "\\providecommand" => Some((false, Definer::Keeps)),
      _ => None,
    });
    let Some((primitive, definer)) = definer else {
      continue;
    };
    let mut j = skip_spaces(body, i + 1);
    if primitive {
      if body.get(j) != Some(&target) {
        continue;
      }
      // the parameter text: `#1` (stored `#`… `1`, a doubled `##1` in a definition's own body)
      let mut digits: Vec<String> = Vec::new();
      j += 1;
      while let Some(p) = body.get(j)
        && p.get_catcode() != Catcode::BEGIN
      {
        match p.get_catcode() {
          Catcode::PARAM => {},
          Catcode::ARG | Catcode::OTHER => digits.push(p.with_str(str::to_string)),
          _ => {
            digits.push(String::new());
            break;
          },
        }
        j += 1;
      }
      if digits == ["1"]
        && let Some(past) = group_end(body, j)
      {
        let own = &body[j + 1..past - 1];
        if argument_at(own).is_some() {
          return Some((definer, own.to_vec()));
        }
        first.get_or_insert_with(|| (definer, own.to_vec()));
      }
      continue;
    }
    if body.get(j).is_some_and(|b| *b == T_OTHER!("*")) {
      j = skip_spaces(body, j + 1);
    }
    if body.get(j) == Some(&target) {
      j += 1;
    } else if body
      .get(j)
      .is_some_and(|b| b.get_catcode() == Catcode::BEGIN)
      && body.get(skip_spaces(body, j + 1)) == Some(&target)
    {
      j = skip_spaces(body, skip_spaces(body, j + 1) + 1);
      if body.get(j).is_none_or(|b| b.get_catcode() != Catcode::END) {
        continue;
      }
      j += 1;
    } else {
      continue;
    }
    j = skip_spaces(body, j);
    let one_arg = [T_OTHER!("["), T_OTHER!("1"), T_OTHER!("]")];
    if body.get(j..j + 3) == Some(&one_arg[..]) {
      let k = skip_spaces(body, j + 3);
      if let Some(past) = group_end(body, k) {
        let own = &body[k + 1..past - 1];
        if argument_at(own).is_some() {
          return Some((definer, own.to_vec()));
        }
        first.get_or_insert_with(|| (definer, own.to_vec()));
      }
    }
  }
  first
}

/// The commands a body hands control to: each control sequence; `\begin{X}`/`\end{X}` as `\X`/`\endX`; a
/// `\csname X\endcsname` of characters as `\X` (environ.sty's `\csname env@X@process\endcsname`); a command whose
/// redefinition the kernel's lock refused also as the refused definition (`\lx@dropped@X`); and the kernel's locked
/// `\maketitle` as the class's `\@maketitle`, which it digests (`\lx@deposit@maketitle`, sect05.rs) as LaTeX's
/// `\maketitle` calls it.
fn handed_to(body: &[Token], out: &mut Vec<Token>) {
  let chars_until = |from: usize, stop: &dyn Fn(&Token) -> bool| -> Option<(String, usize)> {
    let mut name = String::new();
    let mut k = from;
    while let Some(t) = body.get(k) {
      if stop(t) {
        return Some((name, k + 1));
      }
      if !matches!(t.get_catcode(), Catcode::LETTER | Catcode::OTHER) {
        return None;
      }
      t.with_str(|s| name.push_str(s));
      k += 1;
    }
    None
  };
  let mut i = 0;
  while i < body.len() {
    let t = body[i];
    i += 1;
    if !matches!(t.get_catcode(), Catcode::CS | Catcode::ACTIVE) {
      continue;
    }
    if (t == T_CS!("\\begin") || t == T_CS!("\\end"))
      && body
        .get(i)
        .is_some_and(|b| b.get_catcode() == Catcode::BEGIN)
      && let Some((env, past)) = chars_until(i + 1, &|b| b.get_catcode() == Catcode::END)
    {
      let prefix = if t == T_CS!("\\end") { "end" } else { "" };
      out.push(T_CS!(&s!("\\{prefix}{env}")));
      i = past;
      continue;
    }
    if t == T_CS!("\\csname")
      && let Some((cs, past)) = chars_until(i, &|b| *b == T_CS!("\\endcsname"))
    {
      out.push(T_CS!(&s!("\\{cs}")));
      i = past;
      continue;
    }
    out.push(t);
    let dropped = T_CS!(&t.with_cs_name(|cs| s!("\\lx@dropped@{}", cs.trim_start_matches('\\'))));
    if lookup_meaning(&dropped).is_some() {
      out.push(dropped);
    } else if t == T_CS!("\\maketitle") {
      out.push(T_CS!("\\@maketitle"));
    }
  }
}

/// The definition of `target` a body makes with `\def`, `\gdef`, `\edef` or `\xdef` (a `\long`/`\global`/
/// `\protected` prefix kept): the tokens from the prefix through the definition's closing brace, its parameters as
/// stored (a doubled `##1` is a parameter token and `1`), so digesting them defines `target` as the body would. `None`
/// when the body makes none, or only one that reads the body's own arguments (it depends on the call), or only one
/// inside a conditional (`\ifshowfoo\def\foo{…}\fi`: whether the body makes it depends on a test that is not run
/// here — pdflatex leaves `\foo` undefined when the test is false).
fn nested_definition_of(body: &[Token], target: Token) -> Option<Vec<Token>> {
  let mut conditional_depth = 0usize;
  for (i, t) in body.iter().enumerate() {
    if t.get_catcode() != Catcode::CS {
      continue;
    }
    match conditional_kind(t) {
      Some(ConditionalType::If) => {
        conditional_depth += 1;
        continue;
      },
      Some(ConditionalType::Fi) => {
        conditional_depth = conditional_depth.saturating_sub(1);
        continue;
      },
      _ => {},
    }
    if conditional_depth > 0
      || !t.with_str(|s| matches!(s, "\\def" | "\\gdef" | "\\edef" | "\\xdef"))
    {
      continue;
    }
    let j = skip_spaces(body, i + 1);
    if body.get(j) != Some(&target) {
      continue;
    }
    let mut k = j + 1;
    while body
      .get(k)
      .is_some_and(|p| p.get_catcode() != Catcode::BEGIN)
    {
      k += 1;
    }
    let Some(past) = group_end(body, k) else {
      continue;
    };
    if body[j..past]
      .iter()
      .any(|p| p.get_catcode() == Catcode::ARG)
    {
      continue;
    }
    let mut from = i;
    while from > 0
      && body[from - 1].with_str(|s| matches!(s, "\\long" | "\\global" | "\\protected"))
    {
      from -= 1;
    }
    return Some(body[from..past].to_vec());
  }
  None
}

/// Whether `t` opens (`\if…`) or closes (`\fi`) a conditional: by its meaning, or, undefined, by an `\if` name (a
/// `\newif` switch the class declares later).
fn conditional_kind(t: &Token) -> Option<ConditionalType> {
  match lookup_definition(t) {
    Ok(Some(defn)) => defn.get_conditional_type(),
    _ => t
      .with_str(|s| s.starts_with("\\if"))
      .then_some(ConditionalType::If),
  }
}

/// An undefined `token` that an open environment defines for its body in the class's own definition, which the
/// kernel's environment shadows: a `\<env>` the class (or the document) made, which `\begin{<env>}` never runs when the
/// kernel's `\begin{<env>}` constructor is defined (sect01.rs), or one the kernel's lock refused (`\lx@dropped@<env>`,
/// state.rs) — bmcart.cls:712-717 `\def\@figure[#1]{…\def\csentence##1{…}`
/// (its captions' lead sentence, 2305.08284, 1611.01327, 1909.07026) and :1505-1510 `\def\abstract@fmt[#1]{…
/// \def\parttitle##1{…}` (its structured abstract's headings, 2007.07886, 1907.07303, 2106.01700): the definition
/// is made where the call stands, as the class's environment would have made it, and the call retried
/// ([`latexml_core::binding::kernel_autoload::try_refused_environment_definition`]). Searched: the open environments
/// innermost first (the kernel's abstract, no environment frame, first inside its body; a frontmatter field's own
/// setter before them) whose class code never ran (refused, or shadowed by a constructor), each one's definitions and
/// the commands they hand control to (`handed_to`), only definitions a raw file or the document made, outside any
/// conditional, at most 64 per environment. An environment whose begin code ran is not searched:
/// `\newenvironment{myenv}{\ifshowfoo\def\foo{…}\fi}{}` leaves `\foo` undefined in its body, as in pdflatex. Guard:
/// `perfect_kernel_batch68v::an_environment_whose_code_ran_is_not_searched`. OXIDIZED_DESIGN_DIVERGENCES #647.
pub fn refused_environment_definition(token: &Token) -> bool {
  if token.get_catcode() != Catcode::CS || lookup_bool("lx_refused_environment_search") {
    return false;
  }
  let mut envs: Vec<String> = with_stacked_values_sym(pin("current_environment"), |vals| {
    vals.iter().rev().map(|v| v.to_string()).collect()
  });
  if lookup_bool("lx_in_kernel_abstract") {
    envs.insert(0, "abstract".to_string());
  }
  // A frontmatter field's content is typeset by the class where its setter put it: the commands the class's own
  // setter (refused: `\lx@dropped@author`) and what it hands control to define for the field's text — vmsta.cls:278-283
  // `\csdef{name@m}{\def\fnm##1{##1}\def\snm##1{##1}…}`, which its `\@@author` applies to each author
  // (`\author{\inits{Yu.}\fnm{Yuliya}\snm{Mishura}}`: 1508.02842, 1411.4499, 1601.01109).
  let item = lookup_string("lx_frontmatter_item");
  if let Some(field) = item.strip_prefix("ltx:") {
    envs.insert(
      0,
      if field == "creator" {
        "author".to_string()
      } else {
        field.to_string()
      },
    );
  }
  class_body_definition(token, &envs)
}

/// Define `token` as the first of `envs` (environment or command names) defines it for its body in the class's own
/// code (`\lx@dropped@<name>` or `\<name>`, and what they hand control to): [`refused_environment_definition`].
fn class_body_definition(token: &Token, envs: &[String]) -> bool {
  let mut definition: Option<Vec<Token>> = None;
  'envs: for env in envs.iter().filter(|e| !e.is_empty()) {
    // Only code that never ran: the class's definition the kernel's lock refused (`\lx@dropped@<name>`), and its
    // `\<name>` when a constructor `\begin{<name>}` shadows it (`\begin`, sect01.rs). An environment whose own
    // `\<name>` began it has made what its body defines; a command it did not make stays undefined, as in pdflatex.
    let mut queue: VecDeque<Token> = VecDeque::new();
    let dropped = T_CS!(&s!("\\lx@dropped@{env}"));
    if lookup_meaning(&dropped).is_some() {
      queue.push_back(dropped);
    }
    if lookup_meaning(&T_CS!(&s!("\\begin{{{env}}}"))).is_some() {
      queue.push_back(T_CS!(&s!("\\{env}")));
    }
    let mut seen: Vec<Token> = Vec::new();
    let mut next: Vec<Token> = Vec::new();
    while let Some(cs) = queue.pop_front() {
      if seen.contains(&cs) || seen.len() >= 64 {
        continue;
      }
      seen.push(cs);
      let Some(Stored::Expandable(defn)) = lookup_meaning(&cs) else {
        continue;
      };
      if !matches!(
        defn.get_origin(),
        DefinitionOrigin::File | DefinitionOrigin::Document
      ) {
        continue;
      }
      let Some(ExpansionBody::Tokens(body)) = defn.get_expansion() else {
        continue;
      };
      let body = body.unlist_ref();
      if let Some(found) = nested_definition_of(body, *token) {
        definition = Some(found);
        break 'envs;
      }
      next.clear();
      handed_to(body, &mut next);
      queue.extend(next.iter().copied());
    }
  }
  let Some(definition) = definition else {
    return false;
  };
  assign_value("lx_refused_environment_search", true, Some(Scope::Global));
  let made = with_origin(DefinitionOrigin::File, || digest(Tokens::new(definition)));
  assign_value("lx_refused_environment_search", false, Some(Scope::Global));
  made.is_ok() && lookup_meaning(token).is_some()
}

/// The table's keywords setters that the class's (or the document's) abstract defines for its body and the kernel's
/// locked `{abstract}` therefore never runs, with their API and definer: a one-argument definition
/// ([`nested_one_arg_definition`]) in the refused `\abstract` or `\endabstract` (`\lx@dropped@…`, state.rs) or in a
/// command they hand control to ([`handed_to`]), read only in definitions a raw file or the document made — at most
/// 512 of them, nearest first — each with its API, definer and the class's own body for it.
fn abstract_setter_definitions() -> Vec<(&'static str, &'static str, Definer, Vec<Token>)> {
  let mut found: Vec<(&'static str, &'static str, Definer, Vec<Token>)> = Vec::new();
  let mut queue: VecDeque<Token> = VecDeque::from([
    T_CS!("\\lx@dropped@abstract"),
    T_CS!("\\lx@dropped@endabstract"),
  ]);
  if queue.iter().all(|t| lookup_meaning(t).is_none()) {
    return found;
  }
  let mut seen: Vec<Token> = Vec::new();
  let mut next: Vec<Token> = Vec::new();
  while let Some(cs) = queue.pop_front() {
    if seen.contains(&cs) || seen.len() >= 512 {
      continue;
    }
    seen.push(cs);
    let Some(Stored::Expandable(defn)) = lookup_meaning(&cs) else {
      continue;
    };
    if !matches!(
      defn.get_origin(),
      DefinitionOrigin::File | DefinitionOrigin::Document
    ) {
      continue;
    }
    let Some(ExpansionBody::Tokens(body)) = defn.get_expansion() else {
      continue;
    };
    let body = body.unlist_ref();
    for (name, api) in keywords_setters() {
      // (a setter that typesets its argument wins over one found before that does not)
      let at = found.iter().position(|(n, ..)| *n == name);
      if at.is_some_and(|k| argument_at(&found[k].3).is_some()) {
        continue;
      }
      if let Some((definer, own)) = nested_one_arg_definition(body, name) {
        match at {
          None => found.push((name, api, definer, own)),
          Some(k) if argument_at(&own).is_some() => found[k] = (name, api, definer, own),
          Some(_) => {},
        }
      }
    }
    next.clear();
    handed_to(body, &mut next);
    queue.extend(next.iter().copied());
  }
  found
}

/// Declarations and spacing that typeset nothing (font, size, paragraph shape), taking no argument (also the
/// declarations an author line's separator takes the gap before: `base_utilities::spaced_name_separators`).
pub(crate) const LAYOUT: &[&str] = &[
  "\\tiny",
  "\\scriptsize",
  "\\footnotesize",
  "\\small",
  "\\normalsize",
  "\\large",
  "\\Large",
  "\\normalfont",
  "\\rmfamily",
  "\\sffamily",
  "\\ttfamily",
  "\\bfseries",
  "\\mdseries",
  "\\itshape",
  "\\upshape",
  "\\slshape",
  "\\scshape",
  "\\em",
  "\\bf",
  "\\it",
  "\\rm",
  "\\sf",
  "\\sl",
  "\\sc",
  "\\selectfont",
  "\\noindent",
  "\\indent",
  "\\par",
  "\\ignorespaces",
  "\\relax",
  "\\leavevmode",
  "\\unskip",
  "\\nobreak",
  "\\centering",
  "\\raggedright",
  "\\raggedleft",
  "\\bgroup",
  "\\smallskip",
  "\\medskip",
  "\\bigskip",
];
/// Spacing and font commands that typeset nothing, with the number of braced arguments they take (`\vspace*` too).
const LAYOUT_ARGS: &[(&str, usize)] = &[
  ("\\fontsize", 2),
  ("\\fontfamily", 1),
  ("\\fontseries", 1),
  ("\\fontshape", 1),
  ("\\fontencoding", 1),
  ("\\usefont", 4),
  ("\\vspace", 1),
  ("\\addvspace", 1),
  ("\\hspace", 1),
];
/// Glue and dimension parameters (and skips) set or inserted by a glue that follows them: `\rightskip5.5pc`,
/// `\vskip 12\p@`, `\leftskip2pc\rightskip2pc plus1fill` (pasa.cls:56, tADR2e.cls:85).
const GLUE_SETTERS: &[&str] = &[
  "\\leftskip",
  "\\rightskip",
  "\\parindent",
  "\\parskip",
  "\\vskip",
  "\\hskip",
  "\\kern",
];
/// Sectioning commands whose (starred) heading carries the label: paper.cls:621 `\paragraph*{\keywordname:}`,
/// acmconf.cls:340 `\section*{KEYWORDS}`.
const HEADINGS: &[&str] = &[
  "\\section",
  "\\subsection",
  "\\subsubsection",
  "\\paragraph",
  "\\subparagraph",
];
/// Font commands whose argument is label text.
const STYLED: &[&str] = &[
  "\\textbf", "\\textit", "\\textsc", "\\textsf", "\\emph", "\\textrm",
];

fn cs_in(t: &Token, set: &[&str]) -> bool {
  t.get_catcode() == Catcode::CS && t.with_str(|s| set.contains(&s))
}

/// The expansion of `t` when it is a parameterless macro with a token body.
fn parameterless_body(t: &Token) -> Option<Tokens> {
  let Some(Stored::Expandable(defn)) = lookup_meaning(t) else {
    return None;
  };
  if defn
    .get_parameters()
    .is_some_and(|p| !p.get_parameters().is_empty())
  {
    return None;
  }
  match defn.get_expansion() {
    Some(ExpansionBody::Tokens(body)) => Some(body.clone()),
    _ => None,
  }
}

/// The index past the glue `body[i..]` starts with — an optional `=`, a sign, a number and a unit (two letters, or a
/// dimension command as `\p@`), then `plus`/`minus` glue (a `fil` unit there) — or `None` when it is not one.
fn glue_end(body: &[Token], i: usize) -> Option<usize> {
  let ch = |k: usize| -> Option<char> {
    body
      .get(k)
      .filter(|t| matches!(t.get_catcode(), Catcode::OTHER | Catcode::LETTER))
      .and_then(|t| t.with_str(|s| s.chars().next()))
  };
  let dimen = |mut k: usize, fil: bool| -> Option<usize> {
    k = skip_spaces(body, k);
    while ch(k).is_some_and(|c| c == '-' || c == '+') {
      k = skip_spaces(body, k + 1);
    }
    let digits = k;
    while ch(k).is_some_and(|c| c.is_ascii_digit() || c == '.' || c == ',') {
      k += 1;
    }
    let had_digits = k > digits;
    k = skip_spaces(body, k);
    if let Some(t) = body.get(k)
      && t.get_catcode() == Catcode::CS
    {
      // `12\p@`, `\z@`, `0.5\baselineskip`: a dimension or glue command
      return matches!(
        lookup_meaning(t),
        Some(Stored::Register(_)) | Some(Stored::Expandable(_))
      )
      .then_some(k + 1);
    }
    if !had_digits {
      return None;
    }
    let unit: String = (k..k + 2).filter_map(ch).collect();
    const UNITS: &[&str] = &[
      "pt", "pc", "in", "cm", "mm", "bp", "dd", "cc", "sp", "em", "ex",
    ];
    if fil && unit == "fi" && ch(k + 2) == Some('l') {
      let mut e = k + 3;
      while ch(e) == Some('l') {
        e += 1;
      }
      return Some(e);
    }
    UNITS.contains(&unit.as_str()).then_some(k + 2)
  };
  let mut k = skip_spaces(body, i);
  if ch(k) == Some('=') {
    k += 1;
  }
  k = dimen(k, false)?;
  for keyword in ["plus", "minus"] {
    let j = skip_spaces(body, k);
    let word: String = (j..j + keyword.len()).filter_map(ch).collect();
    if word == keyword {
      k = dimen(j + keyword.len(), true)?;
    }
  }
  Some(k)
}

/// A label's text as it is collected from the class's code: its characters and text macros, the layout around them
/// dropped — a source space, a horizontal skip or a kern between two pieces (a class macro's too: `\quad`) read as one
/// space, an explicit space (`\ `, `~`) kept as one — so that neither a font nor a layout command reaches the `name` it
/// becomes, nor the keywords after it: emsprocart's `{\noindent\bfseries{\keywordname.}\vspace{\z@}}` is "Keywords."
/// (1202.4161, 1006.0632, 1610.08545), ieice.cls's `{\bfseries key words:\ \ignorespaces}` "key words:" (2002.00666,
/// 1604.03276, 2509.21881), lni.cls's `\noindent{\bfseries Keywords:}` "Keywords:" (1604.03313). A class macro that is
/// text stays itself, read when the label is (a language's `\keywordname`).
#[derive(Default)]
struct LabelText {
  text:          Vec<Token>,
  pending_space: bool,
}

impl LabelText {
  /// A space between pieces of the label, if one follows.
  fn space(&mut self) { self.pending_space |= !self.text.is_empty(); }

  /// A piece of the label's text (a space token: an explicit one).
  fn push(&mut self, t: Token) {
    let is_space = t.get_catcode() == Catcode::SPACE;
    let after_space = self
      .text
      .last()
      .is_some_and(|l| l.get_catcode() == Catcode::SPACE);
    if self.pending_space && !is_space && !after_space {
      self.text.push(T_SPACE!());
    }
    self.pending_space = false;
    if is_space && (after_space || self.text.is_empty()) {
      return;
    }
    self.text.push(t);
  }

  /// Is there text in it: a letter, or a class macro that is text?
  fn has_text(&self) -> bool {
    self
      .text
      .iter()
      .any(|t| matches!(t.get_catcode(), Catcode::LETTER | Catcode::CS))
  }
}

/// Read one item of a label's code from `body[i]` into `out`, returning the index past it: layout that typesets nothing,
/// or label material, of which its text is kept ([`LabelText`]) — `None` for anything else: an assignment, a definition,
/// a list, a box, a conditional, math. `depth` bounds the class macros followed.
fn label_item(body: &[Token], i: usize, depth: usize, out: &mut LabelText) -> Option<usize> {
  let t = body[i];
  match t.get_catcode() {
    Catcode::SPACE => {
      out.space();
      return Some(i + 1);
    },
    Catcode::LETTER | Catcode::OTHER => {
      out.push(t);
      return Some(i + 1);
    },
    Catcode::ACTIVE if t == T_ACTIVE!('~') => {
      out.push(T_SPACE!());
      return Some(i + 1);
    },
    Catcode::BEGIN => {
      // a group of label material: `{\bfseries Keywords:}`, `{\itshape Some key words}` (biometrika.cls:526)
      let past = group_end(body, i)?;
      label_items(&body[i + 1..past - 1], depth, out)?;
      return Some(past);
    },
    Catcode::CS => {},
    _ => return None,
  }
  if cs_in(&t, LAYOUT) {
    return Some(i + 1);
  }
  if t == T_CS!("\\ ") {
    out.push(T_SPACE!());
    return Some(i + 1);
  }
  if t == T_CS!("\\/") {
    return Some(i + 1);
  }
  if let Some((cs, nargs)) = LAYOUT_ARGS.iter().find(|(cs, _)| cs_in(&t, &[cs])) {
    let mut k = skip_spaces(body, i + 1);
    if body.get(k) == Some(&T_OTHER!("*")) {
      k += 1;
    }
    for _ in 0..*nargs {
      k = group_end(body, skip_spaces(body, k))?;
    }
    if *cs == "\\hspace" {
      out.space();
    }
    return Some(k);
  }
  if cs_in(&t, GLUE_SETTERS) {
    // (a horizontal skip or a kern between two pieces: one space, `\textbf{Key}\kern3pt\textbf{words}`)
    if t == T_CS!("\\hskip") || t == T_CS!("\\kern") {
      out.space();
    }
    return glue_end(body, i + 1);
  }
  if cs_in(&t, HEADINGS) || cs_in(&t, STYLED) {
    // a heading's argument is the label, the heading the class's presentation of it; a font command's is label text
    let mut k = skip_spaces(body, i + 1);
    if cs_in(&t, HEADINGS) && body.get(k) == Some(&T_OTHER!("*")) {
      k = skip_spaces(body, k + 1);
    }
    let past = group_end(body, k)?;
    label_items(&body[k + 1..past - 1], depth, out)?;
    return Some(past);
  }
  // a macro of the class's own: its label text (`\keywordname`, letters only) or its layout (pasa.cls:56
  // `\def\keywordfont{\small\rightskip5.5pc}`)
  let expansion = parameterless_body(&t)?;
  let expansion = expansion.unlist_ref();
  if !expansion.is_empty()
    && expansion.iter().all(|e| {
      matches!(
        e.get_catcode(),
        Catcode::LETTER | Catcode::OTHER | Catcode::SPACE
      )
    })
  {
    out.push(t);
    return Some(i + 1);
  }
  if depth == 0 {
    return None;
  }
  // (read into the label itself, so that a space it makes stays — `\textbf{Key}\quad\textbf{words}` — and refused
  // when it typeset text)
  let typeset = out.text.len();
  label_items(expansion, depth - 1, out)?;
  (out.text.len() == typeset).then_some(i + 1)
}

/// Read every item of a label's code ([`label_item`]), or `None` when one is not label material or layout.
fn label_items(body: &[Token], depth: usize, out: &mut LabelText) -> Option<()> {
  let mut k = 0;
  while k < body.len() {
    k = label_item(body, k, depth, out)?;
  }
  Some(())
}

/// The class's label in the begin code of a `{keywords}` environment that typesets only a label: the code is layout
/// that typesets nothing ([`LAYOUT`], [`LAYOUT_ARGS`], [`GLUE_SETTERS`], a class macro made of them) around label
/// material — text, a heading's or font command's argument, a group of them, a class macro that is text — returned as
/// its text ([`LabelText`]): paper.cls:621 `\small\paragraph*{\keywordname:}`, pasa.cls:101-103
/// `\noindent\keywordfont\textbf{Keywords: }`, biometrika.cls:525-527, irmaems.cls:1131 `\noindent\small Keywords:`,
/// ieice.cls:952-954, emsprocart.cls (1202.4161), lni.cls (1604.03313's copy).
/// Code doing anything else — a flag its title page reads (`\global\keywordstrue`, tADR2e.cls:450), a box it fills
/// (ar-1col's `\global\setbox\@kwdbox\vbox\bgroup`), a list, a definition (TeX Live's lni.cls:561-565 `\def\and`), a
/// conditional (TeX Live's emsprocart.cls `\if@twocolumn`) — is not this shape, and the environment stays the class's.
fn label_only_begin_code(body: &[Token]) -> Option<Tokens> {
  let mut out = LabelText::default();
  label_items(body, 2, &mut out)?;
  // (an explicit space the label ends with is the class's spacing before the keywords, not the label's)
  while out
    .text
    .last()
    .is_some_and(|t| t.get_catcode() == Catcode::SPACE)
  {
    out.text.pop();
  }
  out.has_text().then(|| Tokens::new(out.text))
}

/// The class's label in a keywords setter's body, ahead of its argument: the code before its first `#1`, without the
/// group or font command the argument is in (`\emph{##1}`, `{\raggedright ##1\par}`), read as
/// [`label_only_begin_code`] reads a begin code — astlb.cls:200 `\medskip Key words:~~\emph{##1}` "Key words:"
/// (1202.2889), pazhastl.cls:191 `\medskip\emph{Keywords:}~~##1` "Keywords:" (astro-ph0306137) — or `None` when it
/// is not that shape: wiley-article.cls:449's `\allcaps[…]{keywords}` (1706.00599), mtt.cls:181's `\item` and
/// language-chosen `\mtt@keywords` (1310.0585) keep the default name.
fn label_before_argument(body: &[Token]) -> Option<Tokens> {
  let prefix = &body[..argument_at(body)?];
  let mut open: Vec<usize> = Vec::new();
  for (k, t) in prefix.iter().enumerate() {
    match t.get_catcode() {
      Catcode::BEGIN => open.push(k),
      Catcode::END => {
        open.pop();
      },
      _ => {},
    }
  }
  let mut dropped = vec![false; prefix.len()];
  for &k in &open {
    dropped[k] = true;
    let mut p = k;
    while p > 0 && prefix[p - 1].get_catcode() == Catcode::SPACE {
      p -= 1;
    }
    if p > 0 && (cs_in(&prefix[p - 1], STYLED) || cs_in(&prefix[p - 1], HEADINGS)) {
      dropped[p - 1] = true;
    }
  }
  let kept: Vec<Token> = prefix
    .iter()
    .zip(&dropped)
    .filter(|(_, d)| !**d)
    .map(|(t, _)| *t)
    .collect();
  label_only_begin_code(&kept)
}

/// Is `toks` balanced in its braces?
fn balanced(toks: &[Token]) -> bool {
  let mut depth = 0i32;
  for t in toks {
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => {
        depth -= 1;
        if depth < 0 {
          return false;
        }
      },
      _ => {},
    }
  }
  depth == 0
}

/// The label in the line before `body[at]` ([`ClassReaders::title_page_label`]): the start of the label's tokens, the
/// end of what a deposit takes out with it (past `body[at]` when the label is in the store's own group, else the start
/// of that group), and the label's text.
fn line_label_before(body: &[Token], at: usize) -> Option<(usize, usize, Tokens, bool)> {
  let is_value = |t: &Token| {
    t.get_catcode() == Catcode::CS
      && matches!(lookup_meaning(t), Some(Stored::Expandable(ref d))
        if d.get_origin() == DefinitionOrigin::File
          && d.get_parameters().is_none_or(|p| p.get_parameters().is_empty()))
      && parameterless_body(t).is_some_and(|b| {
        !b.unlist_ref().is_empty()
          && !b.unlist_ref().iter().all(|e| {
            matches!(
              e.get_catcode(),
              Catcode::LETTER | Catcode::OTHER | Catcode::SPACE
            )
          })
      })
  };
  let mut end = at + 1;
  let mut k = at;
  let mut start = 0;
  let mut after_break = false;
  // (the label is in the store's own group, which is the argument of a control sequence: `\footnote{KW: \@keywords}`,
  // `\href{mailto:\@email}{…}`)
  let mut in_argument = false;
  // (past the group last skipped, which starts at `k` while it is the item before)
  let mut group_past: Option<usize> = None;
  // (an implicit `\bgroup` or `\egroup`, `\let` to a brace, opens or closes a group as one does: the store's own group
  // in `\insert\footins\bgroup\footnotesize Keywords: \@keywords\par\egroup`, 86g r3 review probe i3)
  let group_catcode = |t: &Token| {
    if *t == T_CS!("\\bgroup") {
      Catcode::BEGIN
    } else if *t == T_CS!("\\egroup") {
      Catcode::END
    } else {
      t.get_catcode()
    }
  };
  while k > 0 {
    let t = body[k - 1];
    match group_catcode(&t) {
      Catcode::END => {
        // a group before the store: one item of the line
        group_past = Some(k);
        let mut depth = 0i32;
        let mut open = None;
        for j in (0..k).rev() {
          match group_catcode(&body[j]) {
            Catcode::END => depth += 1,
            Catcode::BEGIN => {
              depth -= 1;
              if depth == 0 {
                open = Some(j);
                break;
              }
            },
            _ => {},
          }
        }
        k = open?;
        continue;
      },
      Catcode::BEGIN => {
        // the store's own group: the line goes on before it when the group holds only layout before the store
        let mut inner = LabelText::default();
        if end == at + 1 && label_items(&body[k..at], 0, &mut inner).is_some() && !inner.has_text()
        {
          end = k - 1;
          k -= 1;
          if k > 0 && (cs_in(&body[k - 1], STYLED) || cs_in(&body[k - 1], LAYOUT)) {
            end = k - 1;
            k -= 1;
          }
          continue;
        }
        in_argument = body[..k - 1]
          .iter()
          .rev()
          .find(|b| b.get_catcode() != Catcode::SPACE)
          .is_some_and(|b| {
            b.get_catcode() == Catcode::CS && !cs_in(b, STYLED) && !cs_in(b, HEADINGS)
          });
        start = k;
        break;
      },
      Catcode::CS
        if t == T_CS!("\\par")
          || t == T_CS!("\\\\")
          || t == T_CS!("\\newline")
          || t == T_CS!("\\cr") =>
      {
        after_break = t != T_CS!("\\par");
        start = k;
        break;
      },
      // an environment's begin or end starts the line after it: `\end{center}\noindent Resumen: \@resumen` (review
      // probes `d_own2`, `d_own3`), `\begin{quote}\itshape\@resumen` (`d_nolab`: no label)
      Catcode::CS
        if (t == T_CS!("\\begin") || t == T_CS!("\\end"))
          && group_past.is_some()
          && body
            .get(k)
            .is_some_and(|b| b.get_catcode() == Catcode::BEGIN) =>
      {
        start = group_past.unwrap_or(k);
        break;
      },
      Catcode::CS if is_value(&t) => {
        start = k;
        break;
      },
      _ => {},
    }
    k -= 1;
  }
  if after_break {
    // `\\*[<skip>]`
    start = skip_spaces(body, start);
    if body.get(start) == Some(&T_OTHER!("*")) {
      start = skip_spaces(body, start + 1);
    }
    if body.get(start) == Some(&T_OTHER!("[")) {
      start += body[start..end].iter().position(|b| *b == T_OTHER!("]"))? + 1;
    }
  }
  let label_end = if end == at + 1 { at } else { end };
  // The label is the phrase printed right before the store: a sentence before it in the same line stays where it is
  // (`\footnote{Funded. KW: \@keywords}`, review probe `fnm`: "KW:", the footnote keeps "Funded.").
  let mut depth = 0i32;
  let mut phrase = start;
  for j in start..label_end {
    match body[j].get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth -= 1,
      Catcode::OTHER
        if depth == 0
          && body[j].with_str(|c| matches!(c, "." | "!" | "?"))
          && body
            .get(j + 1)
            .is_some_and(|b| b.get_catcode() == Catcode::SPACE) =>
      {
        let next = skip_spaces(body, j + 1);
        if next < label_end {
          phrase = next;
        }
      },
      _ => {},
    }
  }
  let start = phrase;
  let label = label_only_begin_code(&body[start..label_end])?;
  Some((start, end, label, in_argument))
}

/// Does a keywords setter's body only typeset its label and its argument (layout, label material and `#1`, read as
/// [`label_only_begin_code`] reads a begin code)? Then it typesets nothing the kernel's abstract lacks, and an abstract
/// kept in place may run it: kwmtt.cls's `\par\noindent{\bfseries Keywords:} \itshape##1`, not mtt.cls:181's `\item`
/// of the class's own abstract list (1310.0585).
fn typesets_label_and_argument(body: &[Token]) -> bool {
  let mut text: Vec<Token> = Vec::with_capacity(body.len());
  let mut k = 0;
  while k < body.len() {
    if let Some(length) = argument_length(body, k) {
      text.push(T_LETTER!("x"));
      k += length;
    } else {
      text.push(body[k]);
      k += 1;
    }
  }
  label_only_begin_code(&text).is_some()
}

/// Is a `{keywords}` environment's begin code environ.sty's `\Collect@Body\X`, whose `\X` stores the collected body
/// — with the class's own wrapper around it — in one macro that only the title-page code reads ([`ClassReaders`])?
/// cclayout.sty:71-73 `\long\def\for@cc@keywords#1{\gdef\@keywords@macro{\begin{@abstr}{\cckeywordsname}#1\end
/// {@abstr}}}` `\newenvironment{keywords}{\Collect@Body\for@cc@keywords}{}`, `\@keywords@macro` read only by its
/// `\maketitle` (:889-898): the keywords of 1905.07821 and the cc papers reached neither the frontmatter nor the
/// body. The wrapper is the class's presentation of the keywords, as rule (a)'s title page is.
fn collects_body_for_the_title_page(begin: &[Token], readers: &mut Option<ClassReaders>) -> bool {
  let code: Vec<Token> = begin
    .iter()
    .filter(|t| t.get_catcode() != Catcode::SPACE)
    .copied()
    .collect();
  let [collect, setter] = code[..] else {
    return false;
  };
  if collect != T_CS!("\\Collect@Body") {
    return false;
  }
  let Some(Stored::Expandable(defn)) = lookup_meaning(&setter) else {
    return false;
  };
  let one_arg = defn.get_parameters().is_some_and(
    |p| matches!(p.get_parameters().as_slice(), [only] if !only.optional && undelimited(only)),
  );
  let Some(ExpansionBody::Tokens(body)) = defn.get_expansion() else {
    return false;
  };
  if !one_arg || defn.get_origin() != DefinitionOrigin::File {
    return false;
  }
  let body = body.unlist_ref();
  let mut i = 0;
  while body
    .get(i)
    .is_some_and(|t| t.get_catcode() == Catcode::SPACE || cs_in(t, DEFINITION_FRAME))
  {
    i += 1;
  }
  if !body.get(i).is_some_and(|t| cs_in(t, STORE_DEFINERS)) {
    return false;
  }
  let k = skip_spaces(body, i + 1);
  let Some(store) = body
    .get(k)
    .copied()
    .filter(|c| c.get_catcode() == Catcode::CS)
  else {
    return false;
  };
  let Some(past) = group_end(body, k + 1) else {
    return false;
  };
  argument_at(&body[k + 2..past - 1]).is_some()
    && skip_spaces(body, past) == body.len()
    && readers
      .get_or_insert_with(ClassReaders::survey)
      .read_only_by_the_title_page(&store)
}

/// Reroute each table keywords setter the raw class `cls` defined as a `{keywords}`-like environment whose begin
/// code only typesets a label ([`label_only_begin_code`]) — `\<name>` taking no argument with its `\end<name>` — to the
/// frontmatter's keywords, named by the class's label (`\lx@class@keywords`): the environment as
/// `\lx@begin@keywords[name={label}]` … `\lx@end@keywords`, as mn2e's binding's (mn2e_support_sty.rs), the command
/// with a braced argument that ends the keywords as `\lx@add@keywords[name={label}]{…}`; the class's own begin and end
/// code stay for any other use (`\keywords {alpha}, beta`, a bare `\keywords alpha \endkeywords`). A document's own
/// environment, defined after the class, replaces it.
fn reroute_keywords_environments(cls: &str, readers: &mut Option<ClassReaders>) -> Result<()> {
  let mut rerouted: Vec<&str> = Vec::new();
  for (name, _) in keywords_setters() {
    if defined_by_a_binding(name)? {
      continue;
    }
    let begin = T_CS!(&s!("\\{name}"));
    let end = T_CS!(&s!("\\end{name}"));
    let Some(defn) = lookup_definition(&begin)? else {
      continue;
    };
    if lookup_definition(&end)?.is_none()
      || defn
        .get_parameters()
        .is_some_and(|p| !p.get_parameters().is_empty())
      || !matches!(defn.get_origin(), DefinitionOrigin::File)
    {
      continue;
    }
    let Some(ExpansionBody::Tokens(body)) = defn.get_expansion() else {
      continue;
    };
    let label = match label_only_begin_code(body.unlist_ref()) {
      Some(label) => Some(label),
      None if collects_body_for_the_title_page(body.unlist_ref(), readers) => None,
      None => continue,
    };
    // The class's own begin and end code stay, for a bare `\keywords text` (`\lx@class@keywords`) and its
    // `\endkeywords` (`\lx@class@keywords@end`) outside the environment.
    for (own, cs) in [
      ("\\lx@class@keywords@", &begin),
      ("\\lx@class@keywords@end@", &end),
    ] {
      assign_meaning(
        &T_CS!(&s!("{own}{name}")),
        lookup_meaning(cs).unwrap_or(Stored::None),
        Some(Scope::Global),
      );
    }
    if let Some(label) = label {
      assign_value(
        &s!("lx_class_keywords_label_{name}"),
        Stored::Tokens(label),
        Some(Scope::Global),
      );
    }
    for (cs, reader) in [
      (begin, "\\lx@class@keywords{"),
      (end, "\\lx@class@keywords@end{"),
    ] {
      let mut call = mouth::tokenize_internal(reader).unlist();
      call.extend(ExplodeText!(name));
      call.push(T_END!());
      DefMacro!(cs, None, Tokens::new(call));
    }
    rerouted.push(name);
  }
  if !rerouted.is_empty() {
    Info!(
      "frontmatter",
      cls,
      s!(
        "Rerouted the raw class's keywords environment to the frontmatter API: {{{}}}",
        rerouted.join("}, {")
      )
    );
  }
  Ok(())
}

/// The call `api` (`\lx@add@keywords{#1}`, the table's) named by the class's `label`, when there is one:
/// `\lx@add@keywords[name={label}]{#1}` (astlb.cls:200's "Key words:", 1202.2889; paper.cls:621's `\keywordname:`,
/// synthetic: kwpaper.cls's command forms).
fn named_keywords_call(api: &str, label: Option<Tokens>) -> Vec<Token> {
  let Some(label) = label else {
    return mouth::tokenize_internal(TeXString::assembled(api.to_string())).unlist();
  };
  let (command, rest) = api.split_at(api.find(['[', '{']).unwrap_or(api.len()));
  let mut call = mouth::tokenize_internal(TeXString::assembled(s!("{command}[name={{"))).unlist();
  call.extend(label.unlist());
  call.push(T_END!());
  call.push(T_OTHER!("]"));
  call.extend(mouth::tokenize_internal(TeXString::assembled(rest.to_string())).unlist());
  call
}

/// Does the input go on with a braced group that ends the keywords — `\par`, a blank line, an `\end…` or the end of
/// the input after it, spaces skipped? Every token read is put back. `\keywords {alpha}, beta, gamma` is not that:
/// its first group is part of the keywords (synthetic, the review's t3 shape). Nor is a group a `\section`,
/// `\maketitle` or `\begin{…}` follows: there the class's own code runs, as before 68n.
fn braced_group_ends_the_keywords() -> Result<bool> {
  let Some(open) = read_token()? else {
    return Ok(false);
  };
  let mut read = vec![open];
  let mut ends = false;
  if open.get_catcode() == Catcode::BEGIN {
    let (group, close) = read_balanced_with_close(ExpansionLevel::Off, false, false)?;
    read.extend(group.unlist());
    if let Some(close) = close {
      read.push(close);
      loop {
        let next = read_token()?;
        if let Some(t) = next
          && matches!(t.get_catcode(), Catcode::SPACE | Catcode::COMMENT)
        {
          read.push(t);
          continue;
        }
        ends = next.is_none_or(|t| {
          t == T_CS!("\\par")
            || (t.get_catcode() == Catcode::CS && t.with_str(|s| s.starts_with("\\end")))
        });
        read.extend(next);
        break;
      }
    }
  }
  unread(Tokens::new(read));
  Ok(ends)
}

/// The calls handing the last `\author` call's tail (`\lx@author@handed`) to the creators it made.
/// `at_maketitle`: no `\author` call follows, so when this one made every author, all are queued and
/// the tail's unmarked lines may be placed by the marks (`affiliation_calls`); else they are rows of
/// this call's creators, as a later author's marks are not yet known.
fn author_tail_calls(at_maketitle: bool) -> Result<Vec<Token>> {
  let Some(Stored::Tokens(handed)) = lookup_value("lx_author_handed") else {
    return Ok(Vec::new());
  };
  let Some(current) = store_value("author")? else {
    return Ok(Vec::new());
  };
  let (handed, current) = (handed.unlist_ref(), current.unlist_ref());
  if current.len() <= handed.len() || current[..handed.len()] != *handed {
    return Ok(Vec::new());
  }
  let tail = &current[handed.len()..];
  let and = T_CS!("\\and");
  let mut depth = 0i32;
  for token in tail {
    match token.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth -= 1,
      _ if depth == 0 && *token == and => return Ok(Vec::new()),
      _ => {},
    }
  }
  let tail = Tokens::new(tail.to_vec());
  let made = lookup_int("lx_author_made");
  // (a call that made no creator has none to hand its tail to)
  if made <= 0 || is_blank(&tail) {
    return Ok(Vec::new());
  }
  let attr = mouth::tokenize_internal(TeXString::assembled(s!(
    "_store=author,annotate={made},role=authorblock"
  )));
  let marked = mouth::tokenize_internal(TeXString::assembled(
    "_store=author,role=authorblock".to_string(),
  ));
  let authors_queued = at_maketitle && made as usize == queued_creator_count();
  affiliation_calls(
    Some(attr),
    Some(marked),
    tail,
    queued_creators_have_marks(),
    authors_queued,
  )
}

/// A store value of spaces only.
fn is_blank(value: &Tokens) -> bool {
  value
    .unlist_ref()
    .iter()
    .all(|t| t.get_catcode() == Catcode::SPACE)
}

/// The value of the setter `\<name>`'s store (`\@<name>`, or the one rule (a) found: [`store_of`]): its expansion, when it is a parameterless macro (a
/// `\newcommand`-defined value carries an empty parameter list, as in `\lx@deposit@maketitle`).
fn store_value(name: &str) -> Result<Option<Tokens>> {
  let Some(defn) = lookup_definition(&T_CS!(&s!("\\{}", store_of(name))))? else {
    return Ok(None);
  };
  Ok(match defn.get_expansion() {
    Some(ExpansionBody::Tokens(body))
      if defn
        .get_parameters()
        .is_none_or(|p| p.get_parameters().is_empty()) =>
    {
      Some(body.clone())
    },
    _ => None,
  })
}

/// Does the title page print the store of `\<name>` in one line with another value of the class's, as they are now:
/// one of the other values of its lines the class left printing nothing (`lx_store_line_values_<name>`,
/// `reroute_raw_class_stores`) prints text ([`prints_as_a_value`])?
fn prints_in_a_line_with_a_value(name: &str) -> bool {
  matches!(lookup_value(&s!("lx_store_line_values_{name}")),
    Some(Stored::Tokens(values)) if values.unlist_ref().iter().any(prints_as_a_value))
}

/// At `\maketitle` (`\lx@store@defaults`), or at the frontmatter fallback in a document without
/// one (`at_maketitle` false; again at insertion, a no-op once read): hand the captured stores not yet handed on to their
/// frontmatter API, once — every creator-annotating or creator-making store the document set (in
/// the order it set them), every named abstract or keywords store it set (`labelled_api`: the class's store, which
/// LaTeX expands at `\maketitle`), and, only at `\maketitle` (nothing reads one otherwise), any store left
/// at its class default that the title page prints (lion-msc.cls:196-203 `\gdef\@affiliation{Huygens-Kamerlingh Onnes
/// Laboratory, …}`, which pdflatex prints; not tlp.cls:697-728's running-head placeholders,
/// `ClassReaders::read_by_the_title_page`). A blank store is skipped. The creator-making stores go
/// after the annotating ones when there is an author (their annotations stay the authors'), before
/// them when there is none (an affiliation then annotates the speaker); a creator-annotating store
/// with no creator at all is not handed on and stays the class's to print.
pub fn harvest_stores(at_maketitle: bool) -> Result<Vec<Digested>> {
  if lookup_bool("lx_stores_harvested") {
    return Ok(Vec::new());
  }
  let names = lookup_string("lx_rerouted_stores");
  if names.is_empty() {
    return Ok(Vec::new());
  }
  assign_value("lx_stores_harvested", true, Some(Scope::Global));
  let captured: Vec<&str> = names.split(',').filter(|n| !n.is_empty()).collect();
  let set_order = lookup_string("lx_store_set_order");
  let mut ordered: Vec<&str> = set_order
    .split(',')
    .filter(|n| captured.contains(n))
    .collect();
  for name in &captured {
    if !ordered.contains(name) {
      ordered.push(name);
    }
  }
  // A store set after the last harvest was handed on when it was set (`\lx@store@late`): a harvest
  // re-armed for a superseding `\maketitle` (`\lx@maketitle@supersede`) does not hand it again.
  let late = lookup_string("lx_stores_late");
  ordered.retain(|name| !late.split(',').any(|n| n == *name));
  assign_value(
    "lx_stores_late",
    Stored::String(pin("")),
    Some(Scope::Global),
  );
  // Without a `\maketitle` nothing reads a class default (pdflatex prints none): only the stores the
  // document set are handed on.
  if !at_maketitle {
    ordered.retain(|name| lookup_bool(&s!("lx_store_set_{name}")));
  }
  // An editor or speaker store makes its creator after the authors' annotations are placed; with no
  // author, before them, so they annotate it.
  let has_author = has_front_matter("ltx:creator");
  let makes_creator = ordered.iter().any(|name| {
    creates_a_creator(name)
      && store_value(name)
        .ok()
        .flatten()
        .is_some_and(|v| !is_blank(&v))
  });
  if has_author {
    ordered.sort_by_key(|name| creates_a_creator(name));
  } else {
    ordered.sort_by_key(|name| !creates_a_creator(name));
  }
  let has_creator = has_author || makes_creator;
  let mut calls: Vec<Token> = Vec::new();
  // (the stores handed on here: a second setter of one, a class's `\let\keyword\keywords`, hands it on no second time —
  // review probes `al1`, `al2`)
  let mut handed: Vec<String> = Vec::new();
  for name in ordered {
    let Some(api) = store_api(name) else {
      continue;
    };
    // (without a `\maketitle`, no title page prints the store: an ordinary entry, `untagged_api`)
    let api = if at_maketitle {
      api
    } else {
      untagged_api(&api, name)
    };
    let api = api.as_str();
    // (a deferred store's value as the setter left it, when the store no longer holds it: a class's own `\maketitle`
    // that runs — not refused — prints it and clears it, ptp-prep.clo:27-34 `\@abst … \gdef\@abst{}`, before the
    // fallback reads the stores: 0906.3938, 0907.2968)
    let recorded = match lookup_value(&s!("lx_store_value_{name}")) {
      Some(Stored::Tokens(value)) if lookup_bool(&s!("lx_store_deferred_{name}")) => Some(value),
      _ => None,
    };
    let value = match store_value(name)? {
      Some(value) if !is_blank(&value) => value,
      current => match recorded.or(current) {
        Some(value) => value,
        None => continue,
      },
    };
    if is_blank(&value) {
      continue;
    }
    if annotates_a_creator(name) {
      if !has_creator {
        continue;
      }
    } else if !creates_a_creator(name)
      && lookup_bool(&s!("lx_store_set_{name}"))
      && !lookup_bool(&s!("lx_store_deferred_{name}"))
    {
      // handed on when it was set (a named abstract or keywords store only stored, `labelled_api`)
      continue;
    }
    // A class default only the title page prints (`ClassReaders::read_by_the_title_page`).
    if !lookup_bool(&s!("lx_store_set_{name}"))
      && !lookup_bool(&s!("lx_store_title_page_reads_{name}"))
    {
      continue;
    }
    // A store the title page prints in one line with another value of the class's, now that the document has set them
    // (`ClassReaders::store_lines`): the line stays the class's, which the deposit prints whole, the store un-nulled
    // (`\lx@captured@stores`) — `{\itshape submitted \@submitted; e-mail \@email\par}` with the document's
    // `\submitted{1 May 2026}` (review rev79a Q1).
    if at_maketitle && prints_in_a_line_with_a_value(name) {
      Info!(
        "frontmatter",
        "\\maketitle",
        s!(
          "The raw class's \\{name} stays the class's: its title page prints it in one line with another value"
        )
      );
      continue;
    }
    let store = store_of(name);
    if handed.contains(&store) {
      assign_value(&s!("lx_store_handed_{name}"), true, Some(Scope::Global));
      continue;
    }
    handed.push(store);
    note_store_handed(name);
    // (what the title page prints of it is the class's title page's, its title-page provenance, base_utilities
    // `TITLE_PAGE_ENTRY`; a creator and what annotates one are no frontmatter a late abstract's cover can follow)
    let from = at_maketitle && !annotates_a_creator(name) && !creates_a_creator(name);
    if from {
      calls.extend(store_from_tokens(name));
    }
    let (before, after) = api.split_once("#1").unwrap_or((api, ""));
    calls.extend(mouth::tokenize_internal(TeXString::assembled(before.to_string())).unlist());
    calls.extend(value.unlist());
    calls.extend(mouth::tokenize_internal(TeXString::assembled(after.to_string())).unlist());
    if from {
      calls.extend(store_from_tokens(""));
    }
  }
  let digested = if calls.is_empty() {
    Vec::new()
  } else {
    vec![digest_store_calls(calls)?]
  };
  // (what this `\maketitle` placed, whose setter called again only re-sets the store, is settled once its title page
  // ran: `settle_placed_stores`)
  Ok(digested)
}

LoadDefinitions!({
  // A class's affiliation-kind store (`\institution`, `\department`, `\institute`, `\inst`, …) is
  // printed once under the whole author list (bfhlayout.sty:744-754, courseoutline.cls:150), so its
  // affiliations annotate every creator (`annotate=all`; marker-labelled ones go by their labels
  // when the authors carry marks).
  // Each is tagged `_store=<name>`: an empty one (a default that only warns, umich-thesis.cls:71) is
  // not kept (`insert_frontmatter_entry`).
  DefMacro!("\\lx@add@store@affiliations{}{}", sub[(name, stuff)] {
    let name = name.to_string();
    // (an llncs-style `\institute{A \and B}` the class numbers 1, 2 on its title page: the entries labelled by their
    // numbers, which the authors' numeric marks — `$^{1}$`, `\inst{1}` — request, `class_numbered_institutes`)
    let (stuff, numbered) = match class_numbered_institutes(&stuff) {
      Some(numbered) => (numbered, true),
      None => (stuff, false),
    };
    let attr = mouth::tokenize_internal(TeXString::assembled(s!("_store={name},annotate=all")));
    let marked = mouth::tokenize_internal(TeXString::assembled(s!("_store={name}")));
    // Superscript marks label the affiliations only when the authors carry marks to match; else
    // they are the class's text, and the affiliations annotate every author.
    Ok(Tokens::new(affiliation_calls(
      Some(attr),
      Some(marked),
      stuff,
      numbered || queued_creators_have_marks(),
      true,
    )?))
  });
  // An off-table store the class's own `\@author` prints beside the names (jfpc.cls:188-191 `\@lesinstitutions`,
  // `\@lesmels`; 1503.05530): rows of the author block (`role=authorblock`, unlabelled unless the class prints a label
  // before it), annotating every creator, as an `\author` call's tail is (`author_tail_calls`): the creators' data,
  // where an `ltx:pubnote` made them a note on the document (`hand_untypeset_stores`; under `ltx_authors_1line` a
  // creator's contacts are a hover popup too, LaTeXML.css:74-88).
  DefMacro!("\\lx@add@store@authorblock{}{}", sub[(name, stuff)] {
    let name = name.to_string();
    let attr = mouth::tokenize_internal(TeXString::assembled(s!("_store={name},annotate=all,role=authorblock")));
    let marked = mouth::tokenize_internal(TeXString::assembled(s!("_store={name},role=authorblock")));
    Ok(Tokens::new(affiliation_calls(
      Some(attr),
      Some(marked),
      stuff,
      queued_creators_have_marks(),
      true,
    )?))
  });
  // The author block: what code appends to `\@author` after `\author` stored it — a template's
  // `\address`, `\correspondence`, `\email` rows (`\g@addto@macro\@author{\\…}`; hindawi's template,
  // cjs-rcs-article.cls:361 `\affil`) — is what LaTeX's `\@maketitle` prints with the names
  // (article.cls:247). The creators carry the names already, so each `\author` call's tail is handed
  // to the creators that call made, as author-block rows (`role=authorblock`, unlabelled). Perl drops
  // it (latex_constructs.pool.ltxml:1076-1079, :1099-1107). `\lx@author@handed` (after the creators
  // are queued) records what `\author` stored and how many creators it made; `\lx@author@flush`
  // (before a redefined, appending `\author` stores the next) and `\lx@author@tail` (at `\maketitle`)
  // hand the tail on. Only a tail after exactly what `\author` stored: a class that rebuilds
  // `\@author` in its own form (revtex's `{{#2}{}}`, acmart) is left alone. A tail holding `\and`
  // (more authors) is left too.
  DefPrimitive!("\\lx@author@handed", sub[_args] {
    let stored = store_value("author")?.filter(|stored| !is_blank(stored));
    match stored {
      Some(stored) => {
        let made = queued_creator_count().saturating_sub(lookup_int("lx_author_creators_before") as usize);
        assign_value("lx_author_handed", Stored::Tokens(stored), Some(Scope::Global));
        assign_value("lx_author_made", Stored::Int(made as i64), Some(Scope::Global));
      },
      None => assign_value("lx_author_handed", Stored::Bool(false), Some(Scope::Global)),
    }
  });
  DefMacro!("\\lx@author@flush", sub[_args] {
    // Only an appending `\author` (a class redefined it) keeps the earlier authors; the kernel's
    // replaces them and `\@author` with them, the tail too, as LaTeX does.
    let calls = if lookup_bool("\\author:redefined") { author_tail_calls(false)? } else { Vec::new() };
    assign_value("lx_author_handed", Stored::Bool(false), Some(Scope::Global));
    assign_value(
      "lx_author_creators_before",
      Stored::Int(queued_creator_count() as i64),
      Some(Scope::Global),
    );
    Ok(Tokens::new(calls))
  });
  DefMacro!("\\lx@author@tail", sub[_args] {
    let calls = author_tail_calls(true)?;
    assign_value("lx_author_handed", Stored::Bool(false), Some(Scope::Global));
    Ok(Tokens::new(calls))
  });
  // A rerouted setter records that the document set its store, in order.
  DefPrimitive!("\\lx@store@set{}", sub[(name)] {
    let name = name.to_string();
    assign_value(&s!("lx_store_set_{name}"), true, Some(Scope::Global));
    let order = lookup_string("lx_store_set_order");
    if !order.split(',').any(|n| n == name) {
      let order = if order.is_empty() { name } else { s!("{order},{name}") };
      assign_value("lx_store_set_order", Stored::String(pin(order)), Some(Scope::Global));
    }
  });
  // A store handed to the frontmatter: the deposited `\@maketitle` reads it as empty, and the label its title page
  // prints before it (`labelled_api`), now the entry's name, is taken out with it.
  DefPrimitive!("\\lx@store@handed{}", sub[(name)] {
    note_store_handed(&name.to_string());
  });
  // The frontmatter entries queued from here on are handed on from the store `\<name>` (empty: no more), the
  // title-page provenance (base_utilities `TITLE_PAGE_ENTRY`).
  DefPrimitive!("\\lx@store@from{}", sub[(name)] {
    assign_value(STORE_ENTRY_FROM, Stored::String(pin(name.to_string())), Some(Scope::Global));
  });
  // A store handed on as it is set (`#2`, its frontmatter call): once the `\maketitle` whose title page printed it has
  // placed its value (`lx_store_placed_<name>`, `settle_placed_stores`: handed on, and its sentinel reached the
  // speculative run of the title page), the setter called again only re-sets the store, for what reads it later, and
  // does not replace the placed entry — the title page printed the first value: a class's `\keywords{alpha, beta}`
  // before `\maketitle` and `\keywords{gamma}` after it, "KW: alpha, beta" in the PDF (silent-loss #6b; as a `.bbl`'s
  // re-call of a raw class's `\doi` after the body began, 1710.01977, re-sets only the store under the seal,
  // `seal_digested_frontmatter`). A store the title page does not print takes the later value (a class abstract's end
  // code `Keywords: \@keywords`), and one first set after `\maketitle` is still handed on (pittetd.cls:503-527's
  // abstract prints keywords set there).
  // A named abstract or keywords store (`labelled_api`) set before `\maketitle` only stores: `\maketitle` hands the
  // store on (`harvest_stores`); set after it, it is handed on at once, as `\lx@store@late` hands one.
  DefMacro!("\\lx@store@hand{}{}", sub[(name, call)] {
    let name = name.to_string();
    let harvested = lookup_bool("lx_stores_harvested");
    let deferred = lookup_bool(&s!("lx_store_deferred_{name}"));
    if deferred && !harvested {
      // (the value it stores, for a harvest that finds the store cleared, `harvest_stores`)
      if let Some(value) = store_value(&name)? {
        assign_value(&s!("lx_store_value_{name}"), Stored::Tokens(value), Some(Scope::Global));
      }
      return Ok(Tokens::new(Vec::new()));
    }
    if harvested && lookup_bool(&s!("lx_store_placed_{name}")) {
      Info!(
        "frontmatter",
        s!("\\{name}"),
        s!("\\{name} set again after \\maketitle placed it re-sets the class's store; the frontmatter keeps the title page's value")
      );
      return Ok(Tokens::new(Vec::new()));
    }
    let mut out = store_from_tokens(&name);
    out.extend(
      mouth::tokenize_internal(TeXString::assembled(s!("\\lx@store@handed{{{name}}}"))).unlist(),
    );
    if deferred {
      note_store_late(&name);
      // (the call `\maketitle` settled — named by the class's label only when its title page printed it,
      // `verify_title_page_labels` —, of the value just stored)
      // (an ordinary entry: the title page that read the store printed another value, `untagged_api`)
      if let (Some(api), Some(value)) = (store_api(&name), store_value(&name)?) {
        let api = untagged_api(&api, &name);
        let (before, after) = api.split_once("#1").unwrap_or((api.as_str(), ""));
        out.extend(mouth::tokenize_internal(TeXString::assembled(before.to_string())).unlist());
        out.extend(value.unlist());
        out.extend(mouth::tokenize_internal(TeXString::assembled(after.to_string())).unlist());
        out.extend(store_from_tokens(""));
        return Ok(Tokens::new(out));
      }
    }
    out.extend(call.unlist());
    out.extend(store_from_tokens(""));
    Ok(Tokens::new(out))
  });
  // A creator-annotating or creator-making store set after `\maketitle` read the stores: its value
  // — the setter's argument, not an accumulated store (`\g@addto@macro`) — is handed on at once.
  DefMacro!("\\lx@store@late{}{}", sub[(name, value)] {
    let name = name.to_string();
    if !lookup_bool("lx_stores_harvested") {
      return Ok(Tokens::new(Vec::new()));
    }
    let Some(api) = store_api(&name) else {
      return Ok(Tokens::new(Vec::new()));
    };
    let api = api.as_str();
    note_store_late(&name);
    let mut calls =
      mouth::tokenize_internal(TeXString::assembled(s!("\\lx@store@handed{{{name}}}"))).unlist();
    let (before, after) = api.split_once("#1").unwrap_or((api, ""));
    calls.extend(mouth::tokenize_internal(TeXString::assembled(before.to_string())).unlist());
    calls.extend(value.unlist());
    calls.extend(mouth::tokenize_internal(TeXString::assembled(after.to_string())).unlist());
    Ok(Tokens::new(calls))
  });
  DefPrimitive!("\\lx@store@untypeset", sub[_args] {
    if lookup_bool("lx_depositing_class_maketitle") || lookup_bool("lx_replaying_class_setter") {
      return Ok(Vec::new());
    }
    settle_placed_stores();
    release_unprinted_store_entries();
    hand_untypeset_stores()
  });
  DefPrimitive!("\\lx@store@defaults", sub[_args] {
    verify_title_page_labels()?;
    hand_copy_frontmatter_defaults();
    let harvested = harvest_stores(true)?;
    Ok(harvested)
  });
  // The first token of the kernel abstract's body (`\lx@begin@abstract`, base_utilities.rs), in the frame it is
  // digested in: the table's keywords setters the refused class abstract defines for its body
  // ([`abstract_setter_definitions`]) are defined there, for the body only, as the frontmatter's keywords named by
  // the class's label where its code shows one ([`label_before_argument`]: astlb.cls:200 "Key words:", 1202.2889) —
  // made as the class makes them: a `\def` replaces the document's own `\keywords` there, as in pdflatex, a
  // `\newcommand` leaves it (1706.00599, 1202.2889, 1310.0585).
  // An abstract kept in place as a supplement's (after the document body began, `\lx@add@frontmatter@until`) keeps
  // its keywords in place with it, and the paper's keywords stay the frontmatter's (mtt.cls papers' English and
  // Ukrainian abstracts at the end: 1310.0585, 1310.0589, 1212.3890): there the setter is the class's own definition
  // when it only typesets its label and argument ([`typesets_label_and_argument`]), else a paragraph of the class's
  // label where its code shows one and the keywords — mtt.cls:181's `\item` belongs to the class's own abstract
  // list, which the kernel's abstract is not, and its language-chosen `\mtt@keywords` cannot be read, so its
  // keywords have no label in place: never the English default name where pdflatex prints "Ключовi слова:". A
  // setter that does not typeset its argument (it warns, or drops it) is the class's own definition, as in pdflatex.
  DefPrimitive!("\\lx@abstract@class@setters", {
    // The body is the abstract's, for [`refused_environment_definition`]: the kernel's abstract is no environment
    // frame (`\@currenvir` stays the enclosing one).
    assign_value("lx_in_kernel_abstract", true, Some(Scope::Local));
    if lookup_bool("lx_abstract_begun") {
      assign_value("lx_abstract_begun", false, Some(Scope::Global));
      assign_value("lx@until@begun", "abstract", Some(Scope::Local));
    }
    let in_place = lookup_bool("lx_frontmatter_kept_in_place");
    let setters = abstract_setter_definitions();
    // (whether the document defined each before the class's begin code runs: a `\newcommand` there leaves it)
    let mut defined = Vec::with_capacity(setters.len());
    for (name, ..) in &setters {
      defined.push(lookup_definition(&T_CS!(&s!("\\{name}")))?.is_some());
    }
    // The class's refused begin code runs here for its effects, held in the abstract's group
    // (`replay_class_begin_code`, sect05.rs: frontiersFPHY.cls:520 `\let\section\absection`, 1508.06756); the
    // keywords setters below are the body's.
    let keep_out: Vec<SymStr> = setters
      .iter()
      .map(|(name, ..)| meaning_key(&T_CS!(&s!("\\{name}"))))
      .collect();
    // (not for a brace-less `\abstract`: `\lx@abstract@braceless`, sect05.rs)
    let braceless = lookup_bool("lx_abstract_braceless");
    if braceless {
      assign_value("lx_abstract_braceless", false, Some(Scope::Global));
    } else {
      crate::latex_constructs::replay_class_begin_code("abstract", &keep_out)?;
    }
    for ((name, api, definer, own), defined) in setters.into_iter().zip(defined) {
      let cs = T_CS!(&s!("\\{name}"));
      if definer == Definer::Keeps && defined {
        continue;
      }
      let label = label_before_argument(&own);
      let body = if argument_at(&own).is_none() || (in_place && typesets_label_and_argument(&own)) {
        own
      } else if in_place {
        // (no label when the class's cannot be read: the default name is English where the PDF's need not be)
        let mut body = mouth::tokenize_internal("\\par").unlist();
        if let Some(label) = label {
          body.push(T_BEGIN!());
          body.extend(label.unlist());
          body.push(T_END!());
          body.push(T_SPACE!());
        }
        body.extend(mouth::tokenize_internal("#1").unlist());
        body
      } else {
        named_keywords_call(api, label)
      };
      DefMacro!(cs, convert_latex_args(1, None)?, Tokens::new(body));
      Info!(
        "frontmatter",
        s!("\\{name}"),
        s!("The class's abstract defines \\{name} for its body: it sets the abstract's keywords")
      );
    }
  });
  // A raw class's label-only `{keywords}` (`reroute_keywords_environments`): `\begin{keywords}` … `\end{keywords}`
  // is the frontmatter's keywords, read up to the end; called as a command, `\keywords{…}` (paper.cls's papers write
  // both) is the keywords of its argument when that group ends them — `\par`, a blank line or an `\end…` follows it
  // ([`braced_group_ends_the_keywords`]); any other `\keywords`, whose extent only the class's layout knows
  // (`\keywords {alpha}, beta, gamma`), runs the class's own begin code (`\lx@class@keywords@<name>`), as before.
  DefMacro!("\\lx@class@keywords{}", sub[(name)] {
    let name = name.to_string();
    let label = match lookup_value(&s!("lx_class_keywords_label_{name}")) {
      Some(Stored::Tokens(label)) => Some(label),
      _ => None,
    };
    let current = Expand!(Tokens!(T_CS!("\\@currenvir"))).to_string();
    let api = if current == name {
      "\\lx@begin@keywords"
    } else if braced_group_ends_the_keywords()? {
      "\\lx@add@keywords"
    } else {
      return Ok(Tokens!(T_CS!(&s!("\\lx@class@keywords@{name}"))));
    };
    Ok(Tokens::new(named_keywords_call(api, label)))
  });
  // Its `\end<name>`: in the environment the end of the frontmatter's keywords, else (a bare `\keywords alpha
  // \endkeywords`) the class's own end code, kept as `\lx@class@keywords@end@<name>` (paper.cls:622 `\par\bigskip`).
  DefMacro!("\\lx@class@keywords@end{}", sub[(name)] {
    let name = name.to_string();
    let current = Expand!(Tokens!(T_CS!("\\@currenvir"))).to_string();
    Ok(if current == name {
      Tokens!(T_CS!("\\lx@end@keywords"))
    } else {
      Tokens!(T_CS!(&s!("\\lx@class@keywords@end@{name}")))
    })
  });
  // Fired by `input_definitions` after a `.cls` is loaded raw
  // (latexml_core/src/binding/content.rs).
  DefPrimitive!("\\lx@class@loaded@raw{}", sub[(cls)] {
    let cls = cls.to_string();
    reroute_raw_class_stores(&cls)?;
    reroute_in_place_contact_setters(&cls)?;
  });
  // A title-block line in doubt being queued ([`doubtful`]): the call `k` the queue marks its entry with (and the
  // observer's ledger its call), none when empty.
  DefPrimitive!("\\lx@class@contact@doubt{}", sub[(k)] {
    let k = k.to_string().trim().to_string();
    assign_value(CONTACT_DOUBT, Stored::from(k), Some(Scope::Global));
  });
  // A rerouted contact setter's call that prints its line where it is called ([`class_contact`]).
  DefMacro!("\\lx@class@contact{}{}", sub[(api, class)] {
    Ok(class_contact(api, class, true))
  });
  // …and the call of a rerouted setter that stores its line for the title page (11lomcon.sty's `\@aabuffer`, jetpl's
  // `\@authoraddress`, amsart copies' and stacs.cls's `\addresses`): its class's run prints nothing where it stands, so
  // where the call stands decides nothing — it is the API's while the authors are being set, wherever it is called
  // (stacs's `\address[uniroma]{…}` after the kernel's `\author` left the class's third argument "Gianni Franceschini"
  // as a paragraph, 0902.1737: the class's run stored the address for a title page the kernel's `\maketitle` does not
  // print, and it was lost)
  DefMacro!("\\lx@class@contact@stored{}{}", sub[(api, class)] {
    Ok(class_contact(api, class, false))
  });
  // …and the call of a rerouted `[optional]{mandatory}` setter: the setter's name, whether it prints its line where it
  // is called, its two arguments ([`optional_contact`])
  DefMacro!("\\lx@class@contact@optional{}{}{}{}", sub[(name, in_place, opt, arg)] {
    optional_contact(&name.to_string(), in_place.to_string() == "1", opt, arg)
  });
  // The kernel's `\author[key]{…}` (sect05.rs): the box the call stands in (`\lx@author@box`, [`at_the_body_level`]),
  // a contact line set in it the title block's;
  DefPrimitive!("\\lx@author@box", {
    let boxed = if mode_frames() > 0 {
      author_box()
    } else {
      String::new()
    };
    assign_value(AUTHOR_BOX, Stored::from(boxed), Some(Scope::Global));
  });
  // …the number of author creators before the call (`\lx@author@keys@from`), then its keys and that number after it
  // (`\lx@author@keys`): the authors a rerouted setter's `[key]` names ([`optional_contact`]). A short author
  // (`\author[B.F. Roukema]{…}`) records as a key too, which only a setter's equal key reads.
  DefPrimitive!("\\lx@author@keys@from", {
    // (the records of the authors placed or sealed are no new author's: cleared, review rev94b r3 (a))
    if author_creators_placed() || frontmatter_entry_sealed("ltx:creator", "author") {
      assign_value(
        AUTHOR_KEYS,
        Stored::from(String::new()),
        Some(Scope::Global),
      );
    }
    assign_value(
      "lx_author_keys_from",
      Stored::Int(author_creator_total() as i64),
      Some(Scope::Global),
    );
  });
  DefPrimitive!("\\lx@author@keys{}", sub[(keys)] {
    let keys = keys.to_string();
    let keys = keys.trim();
    let to = author_creator_total();
    let from = usize::try_from(lookup_int("lx_author_keys_from")).unwrap_or(to).min(to);
    if !keys.is_empty() && to > from {
      let mut records = lookup_string(AUTHOR_KEYS);
      if !records.is_empty() {
        records.push('\u{1}');
      }
      records.push_str(&s!("{keys}\u{2}{from}\u{2}{to}"));
      assign_value(AUTHOR_KEYS, Stored::from(records), Some(Scope::Global));
    }
  });
  // A kept class body's call of a rerouted setter (`class_bodies_call_class_copies`, through the one token
  // `\lx@class@contact@dispatch@<name>`): the setter's class definition while the setter means the reroute, else what it
  // means now (a document's redefinition) — one token, nothing left after it for the class's look-ahead.
  DefMacro!("\\lx@class@contact@dispatch{}", sub[(name)] {
    let name = name.to_string();
    let name = name.trim();
    let cs = T_CS!(&s!("\\{name}"));
    Ok(if x_equals(&cs, &T_CS!(&s!("\\lx@class@contact@rerouted@{name}"))) {
      Tokens!(T_CS!(&s!("\\lx@class@contact@{name}")))
    } else {
      Tokens!(cs)
    })
  });
  // A cut entry's font (`observer::writer::plan::entry_font_tokens`): the font changes in effect where the entry's
  // cut starts in its call's content, registered as the `n`-th — the entry digested under them, as in the call
  // (review rev94b r3 B1: `\affil{\itshape $^1$Univ A\\ $^2$Univ B}` lost the italic on Univ B).
  DefPrimitive!("\\lx@fm@entry@font{}", sub[(n)] {
    if let Some(Stored::Font(font)) = lookup_value(&s!("lx_fm_entry_font_{}", n.to_string().trim())) {
      merge_font_ref(&font);
    }
  });
  // …and after a `.sty` is loaded raw: its contact setters only (`reroute_in_place_contact_setters`; 11lomcon.sty,
  // hep-ph0312288). Its stores stay a class's concern (K11).
  DefPrimitive!("\\lx@package@loaded@raw{}", sub[(sty)] {
    reroute_in_place_contact_setters(&sty.to_string())?;
  });
});
