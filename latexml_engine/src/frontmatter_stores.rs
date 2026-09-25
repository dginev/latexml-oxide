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
//! class actually defined: a setter is rerouted only when (a) its NAME is in
//! the surveyed table (`~/data/pk_agents/w23/frontmatter_stores/`, 655 TL
//! classes; an explicit synonym table, never a name pattern — `ead` matches
//! `\setoddhead`, `date` matches `\cref@updatelabeldata`) and (b) its macro
//! BODY is a pure one-argument store — a check on the tokens, never a guess.
//! Everything else the class defined is left alone; the kernel's own locked
//! `\title`/`\author`/`\date`/`\thanks` are not in the table. When at least
//! one store was rerouted the class's `\@maketitle` is discarded as Perl
//! does (it is scaffolding for the stores it can no longer read).
use crate::prelude::*;

/// Setter name (no backslash) → the frontmatter API body for its kind, with
/// `#1` the setter's argument. Kinds and roles follow OmniBus.cls.ltxml and
/// the kernel (`\lx@add@date[role=…]`, latex_constructs.pool.ltxml:1066).
const STORE_SETTERS: &[(&str, &str)] = &[
  ("subtitle", "\\lx@add@subtitle{#1}"),
  ("shorttitle", "\\lx@add@toctitle{#1}"),
  ("toctitle", "\\lx@add@toctitle{#1}"),
  ("titlerunning", "\\lx@add@toctitle{#1}"),
  ("email", "\\lx@add@email{#1}"),
  ("emailaddr", "\\lx@add@email{#1}"),
  ("ead", "\\lx@add@email{#1}"),
  ("address", "\\lx@add@address{#1}"),
  ("affaddr", "\\lx@add@address{#1}"),
  ("affil", "\\lx@add@affiliations{#1}"),
  ("affiliation", "\\lx@add@affiliations{#1}"),
  ("inst", "\\lx@add@affiliations{#1}"),
  ("institute", "\\lx@add@affiliations{#1}"),
  ("institution", "\\lx@add@affiliations{#1}"),
  ("keywords", "\\lx@add@keywords{#1}"),
  ("keyword", "\\lx@add@keywords{#1}"),
  ("kword", "\\lx@add@keywords{#1}"),
  ("kwd", "\\lx@add@keywords{#1}"),
  ("terms", "\\lx@add@keywords{#1}"),
  ("pacs", "\\lx@add@classification[scheme=pacs]{#1}"),
  ("abst", "\\lx@add@abstract{#1}"),
  ("resumen", "\\lx@add@abstract{#1}"),
  ("received", "\\lx@add@date[role=received]{#1}"),
  ("recdate", "\\lx@add@date[role=received]{#1}"),
  ("revised", "\\lx@add@date[role=revised]{#1}"),
  ("accepted", "\\lx@add@date[role=accepted]{#1}"),
  ("pubyear", "\\lx@add@date[role=publication]{#1}"),
  ("copyrightyear", "\\lx@add@copyrightyear{#1}"),
  ("communicated", "\\lx@add@date[role=communicated]{#1}"),
  ("presented", "\\lx@add@date[role=presented]{#1}"),
  ("preprint", "\\lx@add@pubnote[role=preprint]{#1}"),
  ("journal", "\\lx@add@pubnote[role=journal]{#1}"),
  ("jname", "\\lx@add@pubnote[role=journal]{#1}"),
  ("volume", "\\lx@add@pubnote[role=volume]{#1}"),
  ("issue", "\\lx@add@pubnote[role=issue]{#1}"),
  ("pubinfo", "\\lx@add@pubnote{#1}"),
  ("origin", "\\lx@add@pubnote{#1}"),
  ("doi", "\\lx@add@pubnote[role=doi]{#1}"),
  ("conferenceinfo", "\\lx@add@pubnote[role=conference]{#1}"),
  ("articletype", "\\lx@add@pubnote[role=type]{#1}"),
  ("orcid", "\\lx@add@orcid{#1}"),
  ("orcidID", "\\lx@add@orcid{#1}"),
  ("editor", "\\lx@add@editor{#1}"),
  ("editors", "\\lx@add@editor{#1}"),
  ("speaker", "\\lx@add@creator[role=speaker]{#1}"),
];

/// Is `body` a pure one-argument store into the setter's OWN `\@<name>` —
/// `\gdef\@name{#1}`, `\def\@name{#1}`, `\xdef`/`\edef`,
/// `\g@addto@macro\@name{#1}`, with any `\long`/`\global`/`\protected`
/// prefixes — and nothing else? The `\@<name>` convention is the title-page
/// store (`\@title`, ptptex `\@inst`, jpsj2 `\@abst`); a setter storing
/// elsewhere feeds some other reader — letter.cls's `\address` fills
/// `\fromaddress` for `\opening`, afthesis's `\addr@ss` its own title code
/// (sweep 93: rerouting those left the readers undefined) — and stays raw.
pub fn is_store_body(name: &str, body: &Tokens) -> bool {
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
    && rest[3].with_str(|s| s == "1")
    && rest[4].get_catcode() == Catcode::END
}

/// The class's setter `\name`, if it is a one-argument macro whose body is a
/// pure store.
fn store_setter_body(name: &str) -> Result<Option<Tokens>> {
  let Some(defn) = lookup_definition(&T_CS!(&s!("\\{name}")))? else {
    return Ok(None);
  };
  let one_arg = defn.get_parameters().is_some_and(|p| p.get_num_args() == 1);
  match defn.get_expansion() {
    Some(ExpansionBody::Tokens(body)) if one_arg && is_store_body(name, body) => {
      Ok(Some(body.clone()))
    },
    _ => Ok(None),
  }
}

/// Reroute every table setter the raw class `cls` defined as a store.
pub fn reroute_raw_class_stores(cls: &str) -> Result<()> {
  let mut rerouted: Vec<&str> = Vec::new();
  for (name, api) in STORE_SETTERS {
    let Some(store) = store_setter_body(name)? else {
      continue;
    };
    // The API call, then the class's own store as it was: the class's other
    // readers of `\@<name>` (gaceta checks `\@editor` for its section
    // editor line) keep seeing the value; only `\@maketitle` is discarded.
    let params = convert_latex_args(1, None)?;
    let mut body =
      mouth::tokenize_internal(TeXString::assembled(s!("\\lx@store@set{{{name}}}{api}"))).unlist();
    body.extend(store.unlist());
    DefMacro!(T_CS!(&s!("\\{name}")), params, Tokens::new(body));
    rerouted.push(name);
  }
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
    // Inside the deposit group the captured stores read as empty, so a kept
    // `\@maketitle` never typesets them twice.
    let mut nulls: Vec<Token> = Vec::new();
    for name in &captured {
      nulls.extend(
        mouth::tokenize_internal(TeXString::assembled(s!("\\let\\@{name}\\@empty"))).unlist(),
      );
    }
    DefMacro!(T_CS!("\\lx@captured@stores"), None, Tokens::new(nulls));
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

/// The rerouted stores the document left at their class default, handed to
/// the frontmatter API as their setter would have been: whatever non-blank
/// default pdflatex's `\@maketitle` would typeset, placeholder text included.
fn harvest_store_defaults() -> Result<Vec<Digested>> {
  let names = lookup_string("lx_rerouted_stores");
  let mut calls: Vec<Token> = Vec::new();
  for name in names.split(',').filter(|n| !n.is_empty()) {
    if lookup_bool(&s!("lx_store_set_{name}")) {
      continue;
    }
    let Some(api) = STORE_SETTERS
      .iter()
      .find(|(n, _)| *n == name)
      .map(|(_, api)| *api)
    else {
      continue;
    };
    let Some(defn) = lookup_definition(&T_CS!(&s!("\\@{name}")))? else {
      continue;
    };
    let value = match defn.get_expansion() {
      // A `\newcommand`-defined default carries an empty parameter list
      // (as in `\lx@deposit@maketitle`, sect05.rs).
      Some(ExpansionBody::Tokens(body))
        if defn
          .get_parameters()
          .is_none_or(|p| p.get_parameters().is_empty()) =>
      {
        body.clone()
      },
      _ => continue,
    };
    if value
      .unlist_ref()
      .iter()
      .all(|t| t.get_catcode() == Catcode::SPACE)
    {
      continue;
    }
    // Harvested once: a second `\maketitle` finds it set.
    assign_value(&s!("lx_store_set_{name}"), true, Some(Scope::Global));
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
  // A rerouted setter records that the document set its store.
  DefPrimitive!("\\lx@store@set{}", sub[(name)] {
    assign_value(&s!("lx_store_set_{}", name.to_string()), true, Some(Scope::Global));
  });
  DefPrimitive!("\\lx@store@defaults", sub[_args] {
    let harvested = harvest_store_defaults()?;
    Ok(harvested)
  });
  // Fired by `input_definitions` after a `.cls` is loaded raw
  // (latexml_core/src/binding/content.rs).
  DefPrimitive!("\\lx@class@loaded@raw{}", sub[(cls)] {
    let cls = cls.to_string();
    reroute_raw_class_stores(&cls)?;
  });
});
