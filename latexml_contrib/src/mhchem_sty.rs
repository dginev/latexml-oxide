//! mhchem.sty — chemical formula typesetting (`\ce`, `\cee`, `\cf`, `\cesplit`,
//! `\bond`, …).
//!
//! **2026-06-27: RAW-LOAD the genuine TL `mhchem.sty`.** Perl LaTeXML ships no
//! `mhchem.sty.ltxml` and raw-loads the real package; the engine's expl3 /
//! xparse / chemgreek support is now mature enough that we can do the same, so
//! this binding simply `InputDefinitions(noltxml)` the real file. Chemistry
//! therefore renders with proper digit subscripts (`\ce{H2O}` → H₂O), charge
//! superscripts (`\ce{SO4^2-}` → SO₄²⁻), reaction arrows (`->`/`<=>`/`->[..]`),
//! bonds, states, `\cesplit`, etc.
//!
//! This SUPERSEDES the former minimal "`\ce`-as-roman-text" stub (retired here;
//! see git history at `latexml_contrib/src/mhchem_sty.rs` before this commit).
//!
//! **2026-11-14 (issue 824): a formula's units are chemistry math.** mhchem's own parser
//! still reads `\ce`; only its output layer's formula hook is bound
//! (`semantic_formula`): elements, isotopes, counts and charges (OXIDIZED_DESIGN_DIVERGENCES #1040).
//! Arrows, `+` between species, bonds, states and amounts keep mhchem's output (stage 2).
//!
//! Known edge-case residuals under raw-load (tracked in docs/SYNC_STATUS.md):
//! `\ce` used inside an amsmath `align*` alignment, and a few `$`-toggle /
//! `\cesplit`-derived example patterns, can still emit `\lx@begin@alignment` /
//! `\lx@end@inline@math` errors. Simple `\ce` (the overwhelmingly common case)
//! is clean.
use latexml_core::{binding::content::merge_font, common::font::Font};
use latexml_package::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  InputDefinitions!("mhchem", noltxml => true, extension => Some(Cow::Borrowed("sty")));
  // `\__mhchem_output_begin_cf:` is mhchem's empty hook before a formula's `\ensuremath` (mhchem.sty:1424);
  // there the formula's output list is rewritten as chemistry (issue 824). A text-mode `\ce` sets its
  // formula in the text font (mhchem.sty:2760; 2206.12015 `\textbf{\ce{NO}}`), the font math was entered from
  // (`savedfont`), inside the math font command, which `tex=` keeps; a nested formula keeps the outer's.
  RawTeX!(r"\def\lx@mhchem@textfont#1{\lx@mhchem@mathfont{\lx@mhchem@copytextfont#1}}
\ExplSyntaxOn
\cs_set_protected:Npn \__mhchem_output_begin_cf:
  {
    \bool_lazy_or:nnF
      { \l__mhchem_output_isMathMode_bool }
      { \cs_if_eq_p:NN \mhchem@option@mathFont \lx@mhchem@textfont }
      {
        \cs_set_eq:NN \lx@mhchem@mathfont \mhchem@option@mathFont
        \cs_set_eq:NN \mhchem@option@mathFont \lx@mhchem@textfont
      }
    \lx@mhchem@formula
  }
\ExplSyntaxOff");
  // As nicefrac.sty.ltxml's `\nf@mathcopytextfont`.
  DefPrimitive!("\\lx@mhchem@copytextfont", {
    if let Some(Stored::Font(f)) = lookup_value("savedfont") {
      merge_font(Font {
        family: f.family.clone(),
        series: f.series.clone(),
        shape: f.shape.clone(),
        ..Font::default()
      });
    }
  });
  DefMacro!("\\lx@mhchem@formula", {
    let result = lookup_definition(&T_CS!("\\l__mhchem_cf_result_tl"))?
      .and_then(|d| d.get_expansion().cloned())
      .and_then(|b| match b {
        ExpansionBody::Tokens(t) => Some(t.unlist()),
        _ => None,
      });
    Ok(match result.and_then(|r| semantic_formula(&r)) {
      Some(formula) => {
        let mut out = vec![T_CS!("\\tl_set:Nn"), T_CS!("\\l__mhchem_cf_result_tl"), T_BEGIN!()];
        out.extend(formula);
        out.push(T_END!());
        Tokens::new(out)
      },
      None => Tokens!(),
    })
  });
});

/// One `\__mhchem_output_coreFive:nnnnnnn` call (mhchem.sty:2774): pre-superscript (mass number),
/// pre-subscript (atomic number), the element run, oxidation and stacked superscripts, the count, the charge.
struct CoreFive {
  presup:  Vec<Token>,
  presub:  Vec<Token>,
  element: Vec<Token>,
  ox:      Vec<Token>,
  kv:      Vec<Token>,
  count:   Vec<Token>,
  charge:  Vec<Token>,
}

enum FormulaItem {
  Unit(CoreFive),
  /// `\__mhchem_output_withFont:n {…}`: the formula's own text, its brackets (`Fe(CN)6`).
  Text(Vec<Token>),
  /// `\__mhchem_output_amount:n {…}` (mhchem.sty:2769): a stoichiometric amount, before its species.
  Amount(Vec<Token>),
  Raw(Token),
}

/// The next balanced `{…}` argument's tokens, spaces before it skipped.
fn braced(tokens: &mut impl Iterator<Item = Token>) -> Option<Vec<Token>> {
  let mut open = tokens.next()?;
  while open.get_catcode() == Catcode::SPACE {
    open = tokens.next()?;
  }
  if open.get_catcode() != Catcode::BEGIN {
    return None;
  }
  let mut depth = 0usize;
  let mut arg = Vec::new();
  loop {
    let t = tokens.next()?;
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END if depth == 0 => return Some(arg),
      Catcode::END => depth -= 1,
      _ => {},
    }
    arg.push(t);
  }
}

/// mhchem's formula output list (`\l__mhchem_cf_result_tl`) with each coreFive unit as chemistry math, in
/// mhchem's math-mode fonts: an element is an ordinary atom (`\mathord`, so `MoO` stays `Mo`,`O`), its
/// isotope prescripts sit on its first element, the count on its last, and a closing charge goes on the
/// whole formula (`\ce{^{95}MoO4^{2-}}`, `\ce{Fe(CN)6^{3-}}`). `None` (mhchem's own output) when the list
/// has no unit or an argument is not braced.
fn semantic_formula(result: &[Token]) -> Option<Vec<Token>> {
  let core_five = T_CS!("\\__mhchem_output_coreFive:nnnnnnn");
  let with_font = T_CS!("\\__mhchem_output_withFont:n");
  let amount = T_CS!("\\__mhchem_output_amount:n");
  let mut items = Vec::new();
  let mut tokens = result.iter().copied();
  while let Some(t) = tokens.next() {
    if t == with_font {
      items.push(FormulaItem::Text(braced(&mut tokens)?));
    } else if t == amount {
      items.push(FormulaItem::Amount(braced(&mut tokens)?));
    } else if t == core_five {
      items.push(FormulaItem::Unit(CoreFive {
        presup:  braced(&mut tokens)?,
        presub:  braced(&mut tokens)?,
        element: braced(&mut tokens)?,
        ox:      braced(&mut tokens)?,
        kv:      braced(&mut tokens)?,
        count:   braced(&mut tokens)?,
        charge:  braced(&mut tokens)?,
      }));
    } else {
      items.push(FormulaItem::Raw(t));
    }
  }
  let last_unit = items
    .iter()
    .rposition(|i| matches!(i, FormulaItem::Unit(_)))?;
  // The species: the items after the last amount (`\ce{2e-}`, `\ce{3SO4^{2-}}`).
  let start = items
    .iter()
    .rposition(|i| matches!(i, FormulaItem::Amount(_)))
    .map_or(0, |a| a + 1);
  let space =
    |i: &FormulaItem| matches!(i, FormulaItem::Raw(t) if t.get_catcode() == Catcode::SPACE);
  let more = items[start.min(last_unit)..last_unit]
    .iter()
    .any(|i| !space(i));
  // A closing charge belongs to the species when it is more than one element (`\ce{\textit{Ln}O^{2-}}`).
  let lifted = match &items[last_unit] {
    FormulaItem::Unit(u) if !u.charge.is_empty() && items[last_unit + 1..].iter().all(space) => {
      (more || element_items(&u.element).len() != 1).then(|| u.charge.clone())
    },
    _ => None,
  };
  let mut body = Vec::new();
  let mut species = 0;
  let mut before = Before::Nothing;
  for (i, item) in items.into_iter().enumerate() {
    if i == start {
      species = body.len();
    }
    match item {
      FormulaItem::Raw(t) => {
        if t.get_catcode() != Catcode::SPACE {
          before = Before::Atom;
        }
        body.push(t);
      },
      FormulaItem::Amount(a) => {
        body.extend(call("\\__mhchem_output_amount:n", a));
        before = Before::Nothing;
      },
      FormulaItem::Text(text) => {
        body.extend(math_mode(font(text)));
        before = Before::Atom;
      },
      FormulaItem::Unit(mut u) => {
        if i == last_unit && lifted.is_some() {
          u.charge.clear();
        }
        // A unit with no element scripts the bracket or text it follows (`Fe(CN)6`, 2602.09975 `\textit{LnPn}2`),
        // after a unit the species so far (`\ce{H2_{(g)}}`; `\ce{SO4^{2-}_{(aq)}}`, the state after the charge).
        if u.element.is_empty() && before == Before::Unit {
          let formula = body.split_off(species);
          body.extend(call("\\mathord", formula));
        }
        unit_tokens(&mut body, u, before != Before::Nothing);
        before = Before::Unit;
      },
    }
  }
  match lifted {
    Some(charge) => {
      let formula = body.split_off(species);
      body.extend(call("\\mathord", formula));
      body.push(T_SUPER!());
      body.extend(call("", math_mode(script(charge))));
      Some(body)
    },
    None => Some(body),
  }
}

/// What a unit with no element follows.
#[derive(PartialEq)]
enum Before {
  Nothing,
  Atom,
  Unit,
}

fn is_letter(t: &Token) -> bool {
  t.get_catcode() == Catcode::LETTER && t.with_str(|s| s.chars().all(|c| c.is_ascii_alphabetic()))
}

/// The element run split at its capitals (`MoO` is `Mo`, `O`) when it is letters only; else the run whole.
fn element_items(element: &[Token]) -> Vec<&[Token]> {
  if element.is_empty() || !element.iter().all(is_letter) {
    return if element.is_empty() {
      Vec::new()
    } else {
      vec![element]
    };
  }
  let mut items = Vec::new();
  let mut start = 0;
  for (i, t) in element.iter().enumerate().skip(1) {
    if t.with_str(|s| s.starts_with(|c: char| c.is_ascii_uppercase())) {
      items.push(&element[start..i]);
      start = i;
    }
  }
  items.push(&element[start..]);
  items
}

/// One unit, in a group where mhchem's output takes its math-mode branch (`\ensuremath` holds the formula):
/// `{}^{mass}_{number}\mathord{El}…\mathord{El}_{count}^{ox charge}`; the charge of a unit of several elements
/// is the unit's (`\ce{OH^-(H2O)}`, 2405.01836). A unit with no element scripts what precedes it when
/// `scripts_previous`, else an empty base.
fn unit_tokens(out: &mut Vec<Token>, u: CoreFive, scripts_previous: bool) {
  let mut unit = Vec::new();
  if !u.presup.is_empty() || !u.presub.is_empty() {
    unit.extend([T_BEGIN!(), T_END!()]);
    if !u.presup.is_empty() {
      scripted(&mut unit, T_SUPER!(), u.presup);
    }
    if !u.presub.is_empty() {
      scripted(&mut unit, T_SUB!(), u.presub);
    }
  }
  let items = element_items(&u.element);
  let empty = items.is_empty();
  let unit_charge = items.len() > 1 && !u.charge.is_empty();
  let atoms = u.element.iter().all(is_letter);
  for item in items {
    let font = font(item.to_vec());
    if atoms {
      unit.extend(call("\\mathord", font));
    } else {
      unit.extend(font);
    }
  }
  if empty && !scripts_previous {
    unit.extend([T_BEGIN!(), T_END!()]);
  }
  if !u.count.is_empty() {
    scripted(&mut unit, T_SUB!(), u.count);
  }
  let mut sup: Vec<Token> = u.ox.into_iter().chain(u.kv).collect();
  if !unit_charge {
    sup.extend(u.charge.iter().copied());
  }
  if !sup.is_empty() {
    scripted(&mut unit, T_SUPER!(), sup);
  }
  if unit_charge {
    unit = call("\\mathord", unit);
    scripted(&mut unit, T_SUPER!(), u.charge);
  }
  out.extend(math_mode(unit));
}

fn scripted(out: &mut Vec<Token>, mark: Token, body: Vec<Token>) {
  out.push(mark);
  out.extend(call("", script(body)));
}

/// mhchem's script font (`\__mhchem_output_scriptWithFont:n`, mhchem.sty:2918).
fn script(body: Vec<Token>) -> Vec<Token> { call("\\__mhchem_output_scriptWithFont:n", body) }

fn font(body: Vec<Token>) -> Vec<Token> { call("\\__mhchem_output_withFont:n", body) }

/// `\cs{body}`, or `{body}` for no `cs`.
fn call(cs: &str, body: Vec<Token>) -> Vec<Token> {
  let mut out = Vec::with_capacity(body.len() + 3);
  if !cs.is_empty() {
    out.push(T_CS!(cs));
  }
  out.push(T_BEGIN!());
  out.extend(body);
  out.push(T_END!());
  out
}

/// `body` in a group with `\l__mhchem_output_isMathMode_bool` true.
fn math_mode(body: Vec<Token>) -> Vec<Token> {
  let mut out = Vec::from([
    T_CS!("\\group_begin:"),
    T_CS!("\\bool_set_true:N"),
    T_CS!("\\l__mhchem_output_isMathMode_bool"),
  ]);
  out.extend(body);
  out.push(T_CS!("\\group_end:"));
  out
}
