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
use latexml_core::binding::store_setters::{STORE_SETTERS, defined_by_a_binding};

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

/// The calls handing the last `\author` call's tail (`\lx@author@handed`) to the creators it made.
fn author_tail_calls() -> Result<Vec<Token>> {
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
  affiliation_calls(Some(attr), Some(marked), tail, queued_creators_have_marks())
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
    let calls = if lookup_bool("\\author:redefined") { author_tail_calls()? } else { Vec::new() };
    assign_value("lx_author_handed", Stored::Bool(false), Some(Scope::Global));
    assign_value(
      "lx_author_creators_before",
      Stored::Int(queued_creator_count() as i64),
      Some(Scope::Global),
    );
    Ok(Tokens::new(calls))
  });
  DefMacro!("\\lx@author@tail", sub[_args] {
    let calls = author_tail_calls()?;
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
  // Fired by `input_definitions` after a `.cls` is loaded raw
  // (latexml_core/src/binding/content.rs).
  DefPrimitive!("\\lx@class@loaded@raw{}", sub[(cls)] {
    let cls = cls.to_string();
    reroute_raw_class_stores(&cls)?;
  });
});
