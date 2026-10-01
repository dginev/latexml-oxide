//! K11's table of title-page setters (`latexml_engine/src/frontmatter_stores.rs` reroutes a raw
//! class's stores through it), and which of them a binding defined: a setter whose definition
//! changed while a binding loaded is the binding's own (acmart.cls's `\copyrightyear`, ported as the
//! class's store), never a raw class's store, though a raw class that loads the binding
//! (`\LoadClass[nonacm]{acmart}` in a wrapper class) ends with it defined. `input_definitions`
//! snapshots the setters around each binding load; the snapshot is a Rust local, so nested loads
//! pair by the call stack and nothing transient enters the State (or a format dump). So: raw TeX a
//! binding loads itself (`noltxml`) counts as the binding's, a raw class a binding loads still
//! reroutes at its own end, and a raw class's later redefinition of a binding's setter is the raw
//! class's. Only loads through `input_definitions` are tracked (a plain-TeX `.tex` binding loaded
//! by `load_binding` directly is not).
use std::rc::Rc;

use crate::{
  common::{error::Result, store::Stored},
  state::{Scope, assign_value, lookup_definition, lookup_definition_stored, lookup_value},
};

/// Setter name (no backslash) → the frontmatter API body for its kind, with
/// `#1` the setter's argument (an `[optional]{mandatory}` setter's `#2`). Kinds and roles follow OmniBus.cls.ltxml and
/// the kernel (`\lx@add@date[role=…]`, latex_constructs.pool.ltxml:1066).
pub const STORE_SETTERS: &[(&str, &str)] = &[
  ("subtitle", "\\lx@add@subtitle{#1}"),
  ("shorttitle", "\\lx@add@toctitle{#1}"),
  ("toctitle", "\\lx@add@toctitle{#1}"),
  ("titlerunning", "\\lx@add@toctitle{#1}"),
  ("email", "\\lx@add@email{#1}"),
  ("emailaddr", "\\lx@add@email{#1}"),
  ("ead", "\\lx@add@email{#1}"),
  ("address", "\\lx@add@address{#1}"),
  ("affaddr", "\\lx@add@address{#1}"),
  ("affil", "\\lx@add@store@affiliations{affil}{#1}"),
  (
    "affiliation",
    "\\lx@add@store@affiliations{affiliation}{#1}",
  ),
  ("inst", "\\lx@add@store@affiliations{inst}{#1}"),
  ("department", "\\lx@add@store@affiliations{department}{#1}"),
  ("institute", "\\lx@add@store@affiliations{institute}{#1}"),
  (
    "institution",
    "\\lx@add@store@affiliations{institution}{#1}",
  ),
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

/// The setter `\name`'s current definition, as stored; none when it is undefined, `\let` to an
/// undefined control sequence, or `\let` to a character (for which `lookup_definition_stored`
/// makes a new definition on each call).
fn setter_definition(name: &str) -> Result<Option<Stored>> {
  let cs = T_CS!(&s!("\\{name}"));
  if lookup_definition(&cs)?.is_none() {
    return Ok(None);
  }
  lookup_definition_stored(&cs)
}

/// Whether two stored definitions are the same definition (not merely equal ones).
fn same_definition(a: &Stored, b: &Stored) -> bool {
  match (a, b) {
    (Stored::Expandable(a), Stored::Expandable(b)) => Rc::ptr_eq(a, b),
    (Stored::Primitive(a), Stored::Primitive(b)) => Rc::ptr_eq(a, b),
    (Stored::Constructor(a), Stored::Constructor(b)) => Rc::ptr_eq(a, b),
    (Stored::MathPrimitive(a), Stored::MathPrimitive(b)) => Rc::ptr_eq(a, b),
    (Stored::Conditional(a), Stored::Conditional(b)) => Rc::ptr_eq(a, b),
    (Stored::Register(a), Stored::Register(b)) => Rc::ptr_eq(a, b),
    _ => false,
  }
}

/// Each table setter's definition before a binding loads.
pub fn snapshot() -> Result<Vec<Option<Stored>>> {
  STORE_SETTERS
    .iter()
    .map(|(name, _)| setter_definition(name))
    .collect()
}

/// After a binding loaded: mark each table setter whose definition changed since `before` as the
/// binding's.
pub fn mark_bound(before: &[Option<Stored>]) -> Result<()> {
  for ((name, _), before) in STORE_SETTERS.iter().zip(before) {
    let Some(after) = setter_definition(name)? else {
      continue;
    };
    if !before
      .as_ref()
      .is_some_and(|before| same_definition(before, &after))
    {
      assign_value(&s!("lx_bound_setter_{name}"), after, Some(Scope::Global));
    }
  }
  Ok(())
}

/// Whether the setter's current definition is one a binding made.
pub fn defined_by_a_binding(name: &str) -> Result<bool> {
  let Some(current) = setter_definition(name)? else {
    return Ok(false);
  };
  Ok(
    lookup_value(&s!("lx_bound_setter_{name}"))
      .is_some_and(|bound| same_definition(&bound, &current)),
  )
}
