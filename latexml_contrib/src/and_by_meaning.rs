//! The author separators of a binding's argument, found by meaning as well as by spelling (Option C step P2-5,
//! OXIDIZED_DESIGN_DIVERGENCES #995): the gates and the cuts of l_aa, ecai, interact, pasj00, gretsi and lipics part
//! at each `\and` written in the argument, as before, and at each separator the query (`argument_separators`) reads
//! meaning the kernel's `\and` — held in a macro of the document's, a `\let` copy —, which the spelling missed. A
//! written `\and` stays a separator whatever the document made it (trap T8): these classes set their own `\and` where
//! they print the block ([`and_runs`]), and where they do not (ecai, gretsi) today's cut is kept. P2-4's eptcs `cells`
//! (`eptcs_cls.rs`) cuts at the kernel meaning alone; the two are written apart until one shared helper serves both
//! (the extraction P22b names).
use std::collections::BTreeMap;

use latexml_package::prelude::*;

/// The `\and`s of an argument a binding parts at.
pub struct AndRuns {
  /// each separator's own run of the argument, `start → end`: a token spelled as one of the binding's, and a kernel
  /// separator's tokens where they print nothing else (`FmArgSeparator::span`)
  pub runs:  BTreeMap<usize, usize>,
  /// the kernel separators whose tokens print ink too (a macro holding names and the `\and`, `None` spans): no cut by
  /// tokens parts them, the reading's cut does; a gate counts them
  pub inked: usize,
}

impl AndRuns {
  /// The runs that stand at `tokens`' own brace level (no group holds them), `start → end`.
  pub fn top_level(&self, tokens: &[Token]) -> BTreeMap<usize, usize> {
    let mut depth = 0usize;
    let mut at_top = Vec::with_capacity(tokens.len());
    for t in tokens {
      at_top.push(depth == 0);
      match t.get_catcode() {
        Catcode::BEGIN => depth += 1,
        Catcode::END => depth = depth.saturating_sub(1),
        _ => {},
      }
    }
    self
      .runs
      .iter()
      .filter(|(start, _)| at_top.get(**start).copied().unwrap_or(false))
      .map(|(start, end)| (*start, *end))
      .collect()
  }
}

/// The `\and`s of `tokens` ([`AndRuns`]): each token spelled as one of `spelled`, whatever it means where the binding
/// reads it (the parts the binding cut before P2-5: these classes print their own `\and` where they set the block —
/// interact.cls:266 `\def\author#1{\def\and{and }…}`, pasj00.cls:1878 `\def\and{…and…}` in `\maketitle`, lipics.cls:553
/// `\let\and\AB@pand` — over a document's renewed one), and each separator the query reads meaning the kernel's `\and`:
/// held in a macro of the document's, a `\let` copy (its own run), which the spelling missed.
pub fn and_runs(tokens: &[Token], spelled: &[Token]) -> Result<AndRuns> {
  let mut runs: BTreeMap<usize, usize> = tokens
    .iter()
    .enumerate()
    .filter(|(_, t)| spelled.contains(t))
    .map(|(at, _)| (at, at + 1))
    .collect();
  let mut inked = 0;
  if let Some(separators) = argument_separators(&Tokens::new(tokens.to_vec()))? {
    for separator in separators
      .iter()
      .filter(|found| found.meaning == FmSepMeaning::Kernel)
    {
      match separator.span {
        Some((start, end)) => {
          runs.insert(start, end);
        },
        None => inked += 1,
      }
    }
  }
  Ok(AndRuns { runs, inked })
}
