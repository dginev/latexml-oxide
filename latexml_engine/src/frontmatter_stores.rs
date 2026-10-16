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
//! Everything else the class defined is left alone; the kernel's own locked
//! `\title`/`\author`/`\date`/`\thanks` are not in the table. A store is handed
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
  binding::store_setters::{STORE_SETTERS, defined_by_a_binding},
  definition::origin::DefinitionOrigin,
};

use crate::prelude::*;

/// Is `body` a pure store of argument `#<arg>` into the setter's OWN `\@<name>` —
/// `\gdef\@name{#1}`, `\def\@name{#1}`, `\xdef`/`\edef`,
/// `\g@addto@macro\@name{#1}`, with any `\long`/`\global`/`\protected`
/// prefixes — and nothing else? The `\@<name>` convention is the title-page
/// store (`\@title`, ptptex `\@inst`, jpsj2 `\@abst`); a setter storing
/// elsewhere feeds some other reader — letter.cls's `\address` fills
/// `\fromaddress` for `\opening`, afthesis's `\addr@ss` its own title code
/// (sweep 93: rerouting those left the readers undefined) — and stays raw.
pub fn is_store_body(name: &str, body: &Tokens, arg: usize) -> bool {
  let toks = body.unlist_ref();
  let mut i = 0;
  while i < toks.len() && toks[i].with_str(|s| matches!(s, "\\long" | "\\global" | "\\protected")) {
    i += 1;
  }
  let rest = &toks[i..];
  if rest.len() != 5 {
    return false;
  }
  let is_def = rest[0].get_catcode() == Catcode::CS
    && rest[0].with_str(|s| {
      matches!(
        s,
        "\\gdef" | "\\def" | "\\xdef" | "\\edef" | "\\g@addto@macro"
      )
    });
  let own_store = s!("\\@{name}");
  is_def
    && rest[1].get_catcode() == Catcode::CS
    && rest[1].with_str(|s| s == own_store)
    && rest[2].get_catcode() == Catcode::BEGIN
    && rest[3].get_catcode() == Catcode::ARG
    && rest[3].with_str(|s| s == arg.to_string())
    && rest[4].get_catcode() == Catcode::END
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

/// The class's setter `\name`, if it is a pure store: a one-argument macro storing `#1`, or an
/// `[optional]{mandatory}` one storing `#2` (bfhlayout.sty:738 `\providecommand*{\institute}[2][]
/// {\def\@institute{#2}}`). Returns the body and the stored argument's number.
fn store_setter_body(name: &str) -> Result<Option<(Tokens, usize)>> {
  let Some(defn) = lookup_definition(&T_CS!(&s!("\\{name}")))? else {
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
  match (defn.get_expansion(), stored_arg) {
    (Some(ExpansionBody::Tokens(body)), Some(arg)) if is_store_body(name, body, arg) => {
      Ok(Some((body.clone(), arg)))
    },
    _ => Ok(None),
  }
}

/// Reroute every table setter the raw class `cls` defined as a store; a binding's own setter
/// (acmart.cls's `\copyrightyear`, ported as the class's store) is not the raw class's.
pub fn reroute_raw_class_stores(cls: &str) -> Result<()> {
  let mut rerouted: Vec<&str> = Vec::new();
  for (name, api) in STORE_SETTERS {
    if defined_by_a_binding(name)? {
      continue;
    }
    let Some((store, arg)) = store_setter_body(name)? else {
      continue;
    };
    // The setter stores as the class does, so the class's other readers of `\@<name>` (gaceta
    // checks `\@editor` for its section editor line) keep seeing the value; only `\@maketitle` is
    // discarded. A store that annotates a creator (an affiliation, address, e-mail, ORCID) or makes
    // one (an editor, a speaker) is then only recorded: it is read once, at `\maketitle`, where the author list it annotates exists
    // (`harvest_stores`; set later, it is handed on at once, `\lx@store@late`). Any other store is
    // handed to its frontmatter API as it is set — keywords set after `\maketitle` for an
    // abstract that prints them (pittetd.cls:503-527), an abstract at its place in the body.
    let (params, arg_ref) = if arg == 2 {
      (convert_latex_args(2, Some(Tokens::new(Vec::new())))?, "#2")
    } else {
      (convert_latex_args(1, None)?, "#1")
    };
    let mut body = store.unlist();
    let tail = if read_at_maketitle(name) {
      s!("\\lx@store@set{{{name}}}\\lx@store@late{{{name}}}{{{arg_ref}}}")
    } else {
      s!(
        "\\lx@store@set{{{name}}}\\lx@store@handed{{{name}}}{}",
        api.replace("#1", arg_ref)
      )
    };
    body.extend(mouth::tokenize_internal(TeXString::assembled(tail)).unlist());
    DefMacro!(T_CS!(&s!("\\{name}")), params, Tokens::new(body));
    rerouted.push(name);
  }
  reroute_keywords_environments(cls)?;
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
    // class's to print.
    DefMacro!("\\lx@captured@stores", sub[_args] {
      let names = lookup_string("lx_rerouted_stores");
      let mut nulls: Vec<Token> = Vec::new();
      for name in names.split(',').filter(|n| !n.is_empty()) {
        if lookup_bool(&s!("lx_store_handed_{name}")) {
          nulls.extend(
            mouth::tokenize_internal(TeXString::assembled(s!("\\let\\@{name}\\@empty"))).unlist(),
          );
        }
      }
      Ok(Tokens::new(nulls))
    });
    // A store the document never set keeps the class's default (lion-msc.cls:
    // 196-203 `\gdef\@affiliation{Huygens-Kamerlingh Onnes Laboratory, …}`),
    // which `\@maketitle` typesets; nulled in the deposit, it reached neither
    // the frontmatter nor the body. The kernel `\maketitle` hands it to the
    // frontmatter (`\lx@store@defaults` in `\lx@maketitle@body`, sect05.rs),
    // as pdflatex prints it only there.
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

/// The definer of a one-argument command `\<name>` that `body` defines — `\def\name#1{`, `\newcommand{\name}[1]{`
/// (with any `##` doubling and `\long`/`\global` prefix), and nothing that reads a delimited or optional argument —
/// or `None` when it defines none.
fn nested_one_arg_definition(body: &[Token], name: &str) -> Option<Definer> {
  let target = T_CS!(&s!("\\{name}"));
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
        && body
          .get(j)
          .is_some_and(|b| b.get_catcode() == Catcode::BEGIN)
      {
        return Some(definer);
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
      if body
        .get(k)
        .is_some_and(|b| b.get_catcode() == Catcode::BEGIN)
      {
        return Some(definer);
      }
    }
  }
  None
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

/// The table's keywords setters that the class's (or the document's) abstract defines for its body and the kernel's
/// locked `{abstract}` therefore never runs, with their API and definer: a one-argument definition
/// ([`nested_one_arg_definition`]) in the refused `\abstract` or `\endabstract` (`\lx@dropped@…`, state.rs) or in a
/// command they hand control to ([`handed_to`]), read only in definitions a raw file or the document made — at most
/// 512 of them, nearest first.
fn abstract_setter_definitions() -> Vec<(&'static str, &'static str, Definer)> {
  let mut found: Vec<(&'static str, &'static str, Definer)> = Vec::new();
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
      if found.iter().all(|(n, ..)| *n != name)
        && let Some(definer) = nested_one_arg_definition(body, name)
      {
        found.push((name, api, definer));
      }
    }
    next.clear();
    handed_to(body, &mut next);
    queue.extend(next.iter().copied());
  }
  found
}

/// Declarations and spacing that typeset nothing (font, size, paragraph shape), taking no argument.
const LAYOUT: &[&str] = &[
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

/// What a begin code's item from `body[i]` is, and the index past it: layout that typesets nothing (`None` range),
/// label material (the range of its label text: a heading's argument, else the item itself), or `None` for anything
/// else — an assignment, a definition, a list, a box, a conditional, math. `depth` bounds the class macros followed.
fn label_item(body: &[Token], i: usize, depth: usize) -> Option<(Option<(usize, usize)>, usize)> {
  let t = &body[i];
  match t.get_catcode() {
    Catcode::SPACE => return Some((None, i + 1)),
    Catcode::LETTER | Catcode::OTHER => return Some((Some((i, i + 1)), i + 1)),
    Catcode::ACTIVE if *t == T_ACTIVE!('~') => return Some((Some((i, i + 1)), i + 1)),
    Catcode::BEGIN => {
      // a group of label material: `{\bfseries Keywords:}`, `{\itshape Some key words}` (biometrika.cls:526)
      let past = group_end(body, i)?;
      let inner = &body[i + 1..past - 1];
      let mut k = 0;
      let mut text = false;
      while k < inner.len() {
        let (range, next) = label_item(inner, k, depth)?;
        text |= range.is_some();
        k = next;
      }
      return Some((text.then_some((i, past)), past));
    },
    Catcode::CS => {},
    _ => return None,
  }
  if cs_in(t, LAYOUT) {
    return Some((None, i + 1));
  }
  if *t == T_CS!("\\ ") || *t == T_CS!("\\/") {
    return Some((Some((i, i + 1)), i + 1));
  }
  if let Some((_, nargs)) = LAYOUT_ARGS.iter().find(|(cs, _)| cs_in(t, &[cs])) {
    let mut k = skip_spaces(body, i + 1);
    if body.get(k) == Some(&T_OTHER!("*")) {
      k += 1;
    }
    for _ in 0..*nargs {
      k = group_end(body, skip_spaces(body, k))?;
    }
    return Some((None, k));
  }
  if cs_in(t, GLUE_SETTERS) {
    return Some((None, glue_end(body, i + 1)?));
  }
  if cs_in(t, HEADINGS) {
    // the heading's argument is the label; the heading itself is the class's presentation of it
    let mut k = skip_spaces(body, i + 1);
    if body.get(k) == Some(&T_OTHER!("*")) {
      k = skip_spaces(body, k + 1);
    }
    let past = group_end(body, k)?;
    return Some((Some((k + 1, past - 1)), past));
  }
  if cs_in(t, STYLED) {
    let past = group_end(body, skip_spaces(body, i + 1))?;
    return Some((Some((i, past)), past));
  }
  // a macro of the class's own: its label text (`\keywordname`, letters only) or its layout (pasa.cls:56
  // `\def\keywordfont{\small\rightskip5.5pc}`)
  let expansion = parameterless_body(t)?;
  let expansion = expansion.unlist_ref();
  if !expansion.is_empty()
    && expansion.iter().all(|e| {
      matches!(
        e.get_catcode(),
        Catcode::LETTER | Catcode::OTHER | Catcode::SPACE
      )
    })
  {
    return Some((Some((i, i + 1)), i + 1));
  }
  if depth == 0 {
    return None;
  }
  let mut k = 0;
  while k < expansion.len() {
    let (range, next) = label_item(expansion, k, depth - 1)?;
    if range.is_some() {
      return None;
    }
    k = next;
  }
  Some((None, i + 1))
}

/// The class's label in the begin code of a `{keywords}` environment that typesets only a label: the code is layout
/// that typesets nothing ([`LAYOUT`], [`LAYOUT_ARGS`], [`GLUE_SETTERS`], a class macro made of them) around one run of
/// label material — text, a heading's or font command's argument, a group of them, a class macro that is text —
/// returned as written, from its first piece to its last: paper.cls:621 `\small\paragraph*{\keywordname:}`,
/// pasa.cls:101-103 `\noindent\keywordfont\textbf{Keywords: }`, biometrika.cls:525-527, irmaems.cls:1131
/// `\noindent\small Keywords:`, ieice.cls:952-954.
/// Code doing anything else — a flag its title page reads (`\global\keywordstrue`, tADR2e.cls:450), a box it fills
/// (ar-1col's `\global\setbox\@kwdbox\vbox\bgroup`), a list, a definition (lni.cls:561-565 `\def\and`), a
/// conditional (emsprocart's `\if@twocolumn`) — is not this shape, and the environment stays the class's.
fn label_only_begin_code(body: &[Token]) -> Option<Tokens> {
  let mut span: Option<(usize, usize)> = None;
  let mut has_text = false;
  let mut i = 0;
  while i < body.len() {
    let (range, next) = label_item(body, i, 2)?;
    if let Some((from, to)) = range {
      // (layout between two pieces of the label is kept in it, and typesets nothing there)
      has_text |= body[from..to]
        .iter()
        .any(|t| t.get_catcode() == Catcode::LETTER || t.get_catcode() == Catcode::CS);
      span = Some((span.map_or(from, |(start, _)| start), to));
    }
    i = next;
  }
  let (start, end) = span?;
  has_text.then(|| Tokens::new(body[start..end].to_vec()))
}

/// Reroute each table keywords setter the raw class `cls` defined as a `{keywords}`-like environment whose begin
/// code only typesets a label ([`label_only_begin_code`]) — `\<name>` taking no argument with its `\end<name>` — to the
/// frontmatter's keywords, named by the class's label (`\lx@class@keywords`): the environment as
/// `\lx@begin@keywords[name={label}]` … `\lx@end@keywords`, as mn2e's binding's (mn2e_support_sty.rs), the command
/// with a braced argument as `\lx@add@keywords[name={label}]{…}`. A document's own environment, defined after the
/// class, replaces it.
fn reroute_keywords_environments(cls: &str) -> Result<()> {
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
    let Some(label) = label_only_begin_code(body.unlist_ref()) else {
      continue;
    };
    // The class's own begin code stays, for a bare `\keywords text` outside the environment (`\lx@class@keywords`).
    let original = lookup_meaning(&begin);
    assign_meaning(
      &T_CS!(&s!("\\lx@class@keywords@{name}")),
      original.unwrap_or(Stored::None),
      Some(Scope::Global),
    );
    assign_value(
      &s!("lx_class_keywords_label_{name}"),
      Stored::Tokens(label),
      Some(Scope::Global),
    );
    let mut call = mouth::tokenize_internal("\\lx@class@keywords{").unlist();
    call.extend(ExplodeText!(name));
    call.push(T_END!());
    DefMacro!(begin, None, Tokens::new(call));
    DefMacro!(end, None, "\\lx@end@keywords");
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

/// The value of the store `\@<name>`: its expansion, when it is a parameterless macro (a
/// `\newcommand`-defined value carries an empty parameter list, as in `\lx@deposit@maketitle`).
fn store_value(name: &str) -> Result<Option<Tokens>> {
  let Some(defn) = lookup_definition(&T_CS!(&s!("\\@{name}")))? else {
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

/// At `\maketitle` (`\lx@store@defaults`), or at the frontmatter fallback in a document without
/// one (`at_maketitle` false; again at insertion, a no-op once read): hand the captured stores not yet handed on to their
/// frontmatter API, once — every creator-annotating or creator-making store the document set (in
/// the order it set them), and, only at `\maketitle` (nothing reads one otherwise), any store left
/// at its class default (lion-msc.cls:196-203 `\gdef\@affiliation{Huygens-Kamerlingh Onnes
/// Laboratory, …}`, which pdflatex prints). A blank store is skipped. The creator-making stores go
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
  for name in ordered {
    let Some(api) = STORE_SETTERS
      .iter()
      .find(|(n, _)| *n == name)
      .map(|(_, api)| *api)
    else {
      continue;
    };
    let Some(value) = store_value(name)? else {
      continue;
    };
    if is_blank(&value) {
      continue;
    }
    if annotates_a_creator(name) {
      if !has_creator {
        continue;
      }
    } else if !creates_a_creator(name) && lookup_bool(&s!("lx_store_set_{name}")) {
      // handed on when it was set
      continue;
    }
    assign_value(&s!("lx_store_handed_{name}"), true, Some(Scope::Global));
    let (before, after) = api.split_once("#1").unwrap_or((api, ""));
    calls.extend(mouth::tokenize_internal(TeXString::assembled(before.to_string())).unlist());
    calls.extend(value.unlist());
    calls.extend(mouth::tokenize_internal(TeXString::assembled(after.to_string())).unlist());
  }
  if calls.is_empty() {
    return Ok(Vec::new());
  }
  Ok(vec![digest(Tokens::new(calls))?])
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
    let attr = mouth::tokenize_internal(TeXString::assembled(s!("_store={name},annotate=all")));
    let marked = mouth::tokenize_internal(TeXString::assembled(s!("_store={name}")));
    // Superscript marks label the affiliations only when the authors carry marks to match; else
    // they are the class's text, and the affiliations annotate every author.
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
  // A store handed to the frontmatter: the deposited `\@maketitle` reads it as empty.
  DefPrimitive!("\\lx@store@handed{}", sub[(name)] {
    assign_value(&s!("lx_store_handed_{}", name.to_string()), true, Some(Scope::Global));
  });
  // A creator-annotating or creator-making store set after `\maketitle` read the stores: its value
  // — the setter's argument, not an accumulated store (`\g@addto@macro`) — is handed on at once.
  DefMacro!("\\lx@store@late{}{}", sub[(name, value)] {
    let name = name.to_string();
    if !lookup_bool("lx_stores_harvested") {
      return Ok(Tokens::new(Vec::new()));
    }
    let Some(api) = STORE_SETTERS.iter().find(|(n, _)| *n == name).map(|(_, api)| *api) else {
      return Ok(Tokens::new(Vec::new()));
    };
    let late = lookup_string("lx_stores_late");
    if !late.split(',').any(|n| n == name) {
      let late = if late.is_empty() { name.clone() } else { s!("{late},{name}") };
      assign_value("lx_stores_late", Stored::String(pin(late)), Some(Scope::Global));
    }
    let mut calls =
      mouth::tokenize_internal(TeXString::assembled(s!("\\lx@store@handed{{{name}}}"))).unlist();
    let (before, after) = api.split_once("#1").unwrap_or((api, ""));
    calls.extend(mouth::tokenize_internal(TeXString::assembled(before.to_string())).unlist());
    calls.extend(value.unlist());
    calls.extend(mouth::tokenize_internal(TeXString::assembled(after.to_string())).unlist());
    Ok(Tokens::new(calls))
  });
  DefPrimitive!("\\lx@store@defaults", sub[_args] {
    let harvested = harvest_stores(true)?;
    Ok(harvested)
  });
  // The first token of the kernel abstract's body (`\lx@begin@abstract`, base_utilities.rs), in the frame it is
  // digested in: the table's keywords setters the refused class abstract defines for its body
  // ([`abstract_setter_definitions`]) are defined there, for the body only, as the frontmatter's keywords — made as
  // the class makes them: a `\def` replaces the document's own `\keywords` there, as in pdflatex, a `\newcommand`
  // leaves it (1706.00599, 1202.2889, 1310.0585).
  // An abstract kept in place as a supplement's (after the document body began, `\lx@add@frontmatter@until`) keeps
  // its keywords in place with it, a paragraph of their text, and the paper's keywords stay the frontmatter's
  // (mtt.cls papers' English and Ukrainian abstracts at the end: 1310.0585, 1310.0589, 1212.3890).
  DefPrimitive!("\\lx@abstract@class@setters", {
    let in_place = lookup_bool("lx_frontmatter_kept_in_place");
    for (name, api, definer) in abstract_setter_definitions() {
      let cs = T_CS!(&s!("\\{name}"));
      if definer == Definer::Keeps && lookup_definition(&cs)?.is_some() {
        continue;
      }
      let body = if in_place { "\\par #1" } else { api };
      DefMacro!(
        cs,
        convert_latex_args(1, None)?,
        mouth::tokenize_internal(body)
      );
      Info!(
        "frontmatter",
        s!("\\{name}"),
        s!("The class's abstract defines \\{name} for its body: it sets the abstract's keywords")
      );
    }
  });
  // A raw class's label-only `{keywords}` (`reroute_keywords_environments`): `\begin{keywords}` … `\end{keywords}`
  // is the frontmatter's keywords, read up to the end; called as a command, `\keywords{…}` (paper.cls's papers write
  // both) is the keywords of its argument; a bare `\keywords text`, whose extent only the class's layout knows, runs
  // the class's own begin code (`\lx@class@keywords@<name>`), as before.
  DefMacro!("\\lx@class@keywords{}", sub[(name)] {
    let name = name.to_string();
    let label = match lookup_value(&s!("lx_class_keywords_label_{name}")) {
      Some(Stored::Tokens(label)) => label,
      _ => Tokens::default(),
    };
    let current = Expand!(Tokens!(T_CS!("\\@currenvir"))).to_string();
    let api = if current == name {
      "\\lx@begin@keywords"
    } else {
      let next = read_token()?;
      let braced = next.as_ref().is_some_and(|t| t.get_catcode() == Catcode::BEGIN);
      if let Some(t) = next {
        unread(Tokens!(t));
      }
      if !braced {
        return Ok(Tokens!(T_CS!(&s!("\\lx@class@keywords@{name}"))));
      }
      "\\lx@add@keywords"
    };
    let mut call = mouth::tokenize_internal(TeXString::assembled(s!("{api}[name={{"))).unlist();
    call.extend(label.unlist());
    call.extend(mouth::tokenize_internal("}]").unlist());
    Ok(Tokens::new(call))
  });
  // Fired by `input_definitions` after a `.cls` is loaded raw
  // (latexml_core/src/binding/content.rs).
  DefPrimitive!("\\lx@class@loaded@raw{}", sub[(cls)] {
    let cls = cls.to_string();
    reroute_raw_class_stores(&cls)?;
  });
});
