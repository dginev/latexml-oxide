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
  definition::origin::{DefinitionOrigin, with_origin},
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
  title_page: FxHashSet<SymStr>,
  /// For each control sequence, the macros a raw file defined that read it (itself, when it reads itself).
  readers:    HashMap<SymStr, Vec<SymStr>>,
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
fn reads_of(body: &[Token]) -> Vec<Token> {
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
      let mut seen = FxHashSet::default();
      let called = callees.entry(caller).or_default();
      for read in reads_of(body.unlist_ref()) {
        let callee = read.get_cs_name();
        if seen.insert(callee) {
          readers.entry(callee).or_default().push(caller);
          called.push(callee);
        }
      }
    }
    let mut title_page: FxHashSet<SymStr> = FxHashSet::default();
    let mut joined: Vec<SymStr> = [
      "\\lx@dropped@maketitle",
      "\\@maketitle",
      "\\lx@dropped@@maketitle",
    ]
    .into_iter()
    .map(|root| T_CS!(root).get_cs_name())
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
    ClassReaders { title_page, readers }
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

/// Reroute every table setter the raw class `cls` defined as a store; a binding's own setter
/// (acmart.cls's `\copyrightyear`, ported as the class's store) is not the raw class's.
pub fn reroute_raw_class_stores(cls: &str) -> Result<()> {
  let mut rerouted: Vec<&str> = Vec::new();
  let mut readers: Option<ClassReaders> = None;
  for (name, api) in STORE_SETTERS {
    if defined_by_a_binding(name)? {
      continue;
    }
    let Some((store, arg, store_cs)) = store_setter_body(name, &mut readers)? else {
      continue;
    };
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
  reroute_keywords_environments(cls, &mut readers)?;
  reroute_kernel_field_setters(cls)?;
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
            mouth::tokenize_internal(TeXString::assembled(s!("\\let\\{}\\@empty", store_of(name)))).unlist(),
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
    let in_place = lookup_bool("lx_frontmatter_kept_in_place");
    for (name, api, definer, own) in abstract_setter_definitions() {
      let cs = T_CS!(&s!("\\{name}"));
      if definer == Definer::Keeps && lookup_definition(&cs)?.is_some() {
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
  });
});
