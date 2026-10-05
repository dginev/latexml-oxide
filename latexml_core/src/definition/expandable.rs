use std::borrow::Cow;

// use std::fmt;
use libxml::tree::Node;

use crate::{
  Digested,
  common::{error::*, locator::Locator, object::Object},
  definition::{
    BeforeDigestClosure, Definition, DigestionClosure, ExpansionBody, argument::ArgWrap,
  },
  state::*,
};

/// Returns true when `\protect` currently has no meaning, or is
/// `\let`-equivalent to `\relax`. Used by the recursion guard in
/// `Expandable::invoke` to distinguish `\def\foo{\protect\foo}`
/// definitions under safe (`\@unexpandable@protect`/`\string`/…)
/// vs. unsafe (`\relax`/undefined) `\protect` regimes. Only the
/// unsafe regime actually runaways at full expansion.
/// True when `\protect` currently means `\@typeset@protect` — i.e. normal
/// typesetting, where real LaTeX's `\@protected@testopt` DOES expand an
/// optional-argument `\newcommand`. Under any other regime
/// (`\protected@edef` sets `\@unexpandable@protect`, `\protected@write`
/// sets `\@unexpandable@protect`, etc.) the real macro emits
/// `\protect\cs` unexpanded. Used by the gullet's partial-expansion gate
/// for binding-defined optional-argument macros (see
/// `wisdom_replayed_tokens_param_packing` sibling entry; titlecaps OOM,
/// perfect-kernel sweep 16).
pub fn protect_is_typeset() -> bool {
  let protect = lookup_meaning(&T_CS!("\\protect"));
  let typeset = lookup_meaning(&T_CS!("\\@typeset@protect"));
  match (protect, typeset) {
    (Some(p), Some(t)) => p == t,
    // No \protect regime established yet (early bootstrap): behave as
    // typesetting (expand), the historical behavior.
    _ => true,
  }
}

pub(crate) fn protect_is_relax_or_undefined() -> bool {
  let protect = T_CS!("\\protect");
  match lookup_meaning(&protect) {
    None => true,
    // Meaning equal to \relax's meaning ⇒ unsafe.
    Some(stored) => match lookup_meaning(&T_CS!("\\relax")) {
      None => false,
      Some(relax) => stored == relax,
    },
  }
}
use crate::{
  document::Document,
  parameter::Parameters,
  token::*,
  tokens::{BodyPiece, NO_TOKENS, Tokens},
  whatsit::Whatsit,
};

#[derive(Debug, Clone, Default)]
pub struct ExpandableOptions {
  pub locked:             bool,
  pub protected:          bool,
  pub outer:              bool,
  pub long:               bool,
  pub scope:              Option<Scope>,
  pub alias:              Option<String>,
  pub mathactive:         bool,
  pub robust:             bool,
  pub nopack_parameters:  bool,
  /// See [`Definition::peeks_by_futurelet`].
  pub peeks_by_futurelet: bool,
  /// Declared with TeX's long-ness: the `\def` family, LaTeX's starred `\newcommand` family. Such a macro, made
  /// by a format, a raw-loaded file or the document, checks its arguments for `\par` ([`Expandable::checks_par`]).
  pub tex_declared:       bool,
}

#[derive(Debug, Clone)]
pub struct Expandable {
  pub is_protected:       bool,
  /// See [`Definition::peeks_by_futurelet`]: set by the kernel's peeking
  /// closures, and for every macro whose leading argument is a `\newcommand`
  /// optional one ([`crate::parameter::Parameter::testopt`]).
  pub peeks_by_futurelet: bool,
  pub is_long:            bool,
  /// tex.web §392/§399: reading this macro's arguments, a `\par` token ends the call — "Paragraph ended before
  /// \x was complete" (§396). Only a non-`\long` macro whose long-ness is TeX's (`tex_declared`, made at a
  /// Format, File or Document origin): LaTeXML's own `DefMacro!`s are non-long without meaning it (Perl stores
  /// `isLong` and never reads it, Expandable.pm:46), so they are not checked. A `\let` copy keeps it (the meaning
  /// carries the long-ness). Witness 1001.1670; guards `scanner_status::a_par_*`, DIVERGENCES #442.
  pub checks_par:         bool,
  pub is_outer:           bool,
  pub has_cc_arg:         bool,
  pub alias:              Option<String>,
  pub locator:            Locator,
  pub cs:                 Token,
  pub paramlist:          Option<Parameters>,
  pub expansion:          Option<ExpansionBody>,
  pub origin:             crate::definition::origin::DefinitionOrigin,
  /// The token-list body's [`Tokens::substitution_plan`], built at the first call that substitutes.
  pub plan:               std::cell::OnceCell<Box<[BodyPiece]>>,
}
impl Default for Expandable {
  fn default() -> Self {
    Expandable {
      is_protected:       false,
      peeks_by_futurelet: false,
      is_long:            false,
      checks_par:         false,
      is_outer:           false,
      has_cc_arg:         false,
      alias:              None,
      locator:            Locator::default(),
      cs:                 T_CS!("Expandable"),
      paramlist:          None,
      expansion:          None,
      origin:             crate::definition::origin::current_origin(),
      plan:               std::cell::OnceCell::new(),
    }
  }
}
impl PartialEq for Expandable {
  fn eq(&self, other: &Expandable) -> bool {
    self.paramlist == other.paramlist && self.expansion == other.expansion
  }
}

// impl fmt::Display for Expandable {
//   fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
//     todo!();
//   }
// }
impl Object for Expandable {
  fn is_definition(&self) -> bool { true }
  fn is_expandable(&self) -> bool { true }
  fn get_locator(&self) -> Option<Locator> { Some(self.locator) }
  fn get_origin(&self) -> crate::definition::origin::DefinitionOrigin { self.origin }
  fn stringify(&self) -> String { <Self as Definition>::stringify_type(self, "Expandable") }
}
impl Definition for Expandable {
  fn is_protected(&self) -> bool { self.is_protected }
  fn peeks_by_futurelet(&self) -> bool { self.peeks_by_futurelet }
  fn get_parameters(&self) -> Option<&Parameters> { self.paramlist.as_ref() }
  fn get_num_args(&self) -> usize {
    match self.paramlist {
      Some(ref params) => params.get_num_args(),
      None => 0,
    }
  }
  fn get_cs(&self) -> Cow<'_, Token> { Cow::Borrowed(&self.cs) }
  fn get_cs_name(&self) -> Cow<'_, str> {
    match self.alias {
      Some(ref alias) => Cow::Borrowed(alias),
      None => Cow::Owned(self.cs.with_cs_name(ToString::to_string)),
    }
  }
  // fn with_cs_name<R, FnR>(&self, caller: FnR) -> R
  // where FnR: FnOnce(&str) -> R {
  //   match self.alias {
  //     Some(ref alias) => caller(alias),
  //     None => self.cs.with_cs_name(caller),
  //   }
  // }
  fn get_expansion(&self) -> Option<&ExpansionBody> { self.expansion.as_ref() }
  fn get_alias(&self) -> Option<&String> { self.alias.as_ref() }

  /// Expand the expandable control sequence. This should be carried out by the Gullet.
  fn invoke(&self, once_only: bool) -> Result<Tokens> {
    // Perl shortcut for "trivial" macros that were tracing- or
    // profiling-aware. Neither tracing nor profiling is implemented
    // in the Rust port (the returned `_tracing` / `_profiled` values
    // were discarded), so the two state lookups were pure overhead
    // on every macro expansion (\~350k calls in si.tex alone per
    // callgrind). Removed — re-introduce only alongside the actual
    // tracing/profiling features if/when they land.
    match &self.expansion {
      Some(ExpansionBody::Closure(closure)) => {
        // Harder to emulate \tracingmacros here.
        let args = if let Some(ref parms) = self.paramlist {
          // An argument that ran off a file's end abandons the call.
          match self.read_call_arguments(parms)? {
            Some(args) => args,
            None => return Ok(Tokens!()),
          }
        } else {
          Vec::new()
        };
        // Profiling: not implemented (Perl: startProfiling($profiled, 'expand'))
        let result = closure(args)?;
        // Tracing: Perl prints tracingCSName ==> tracetoString(result)
        // Not implemented — silently skip to avoid panic on \tracingmacros=1
        Ok(result)
      },
      Some(ExpansionBody::Tokens(tokens)) => {
        let result = if self.paramlist.is_none() {
          // Case: Trivial macro
          // Profiling: not implemented (Perl: startProfiling($profiled, 'expand'))
          // Tracing: Perl prints tracingCSName -> tracetoString(expansion)
          // Not implemented — silently skip to avoid panic on \tracingmacros=1
          Tokens::new(self.parameterless_body(tokens, once_only)?.to_vec())
        } else {
          let args = if let Some(ref parms) = self.paramlist {
            // A call that does not match its `\def` (tex.web §398): reported,
            // and the macro is ignored; so is one whose argument ran off a
            // file's end (§392).
            match self.read_call_arguments(parms)? {
              Some(args) => args,
              None => return Ok(Tokens!()),
            }
          } else {
            Vec::new()
          };
          if self.has_cc_arg {
            // Do we actually need to substitute the args in?
            // Pre-size: one entry per argument; avoids Vec doublings on
            // macros with many args.
            let mut args_tks = Vec::with_capacity(args.len());
            for arg in args.iter() {
              args_tks.push(arg.as_tokens()?);
            }
            tokens.substitute_parameters(args_tks.as_slice())
          } else {
            tokens.clone()
          }
        };
        // Profiling: Perl appends T_MARKER(profiled) for exclusive profiling
        // Not implemented — silently skip
        Ok(result)
      },
      None => {
        // we always need to read the arguments, for e.g. things like \@gobble
        if let Some(ref parms) = self.paramlist {
          self.read_call_arguments(parms)?;
        }
        Ok(NO_TOKENS)
      },
    }
  }

  /// [`Self::invoke`] onto the input, a token-list body without building the expansion: the parameterless
  /// body is pushed as stored, a body with parameters through its substitution plan
  /// ([`crate::gullet::unread_substituted`]). Closures and argument-only definitions take the general path.
  fn invoke_onto_input(&self, once_only: bool) -> Result<()> {
    let Some(ExpansionBody::Tokens(tokens)) = &self.expansion else {
      crate::gullet::unread_expansion(self.invoke(once_only)?);
      return Ok(());
    };
    let Some(parms) = &self.paramlist else {
      let body = self.parameterless_body(tokens, once_only)?;
      crate::gullet::unread_substituted(body, &[BodyPiece::Run(0, body.len() as u32)], &[]);
      return Ok(());
    };
    // A call that does not match its `\def` (tex.web §398) or whose argument ran off a file's end (§392)
    // expands to nothing.
    let Some(args) = self.read_call_arguments(parms)? else {
      return Ok(());
    };
    if !self.has_cc_arg {
      crate::gullet::unread_substituted(
        tokens.unlist_ref(),
        &[BodyPiece::Run(0, tokens.len() as u32)],
        &[],
      );
      return Ok(());
    }
    self.substitute_onto_input(tokens, &args)
  }

  // Not implemented for expandable
  fn invoke_primitive(&self) -> Result<Vec<Digested>> { Ok(Vec::new()) }
  fn before_digest(&self) -> Option<&Vec<BeforeDigestClosure>> { None }
  fn after_digest(&self) -> Option<&Vec<DigestionClosure>> { None }
  fn do_absorption(&self, _document: &mut Document, _whatsit: &Whatsit) -> Result<Vec<Node>> {
    fatal!(
      Definition,
      Unexpected,
      "do_absorption on Expandable should never be called!"
    );
  }
}

impl Expandable {
  /// The expansion of a parameterless body: the body, or nothing when it expands into itself
  /// ([`Self::expands_into_itself`]), with `Error:recursion` (Perl Expandable.pm L81-89). The one place for
  /// both [`Definition::invoke`] and [`Definition::invoke_onto_input`].
  fn parameterless_body<'a>(&self, tokens: &'a Tokens, once_only: bool) -> Result<&'a [Token]> {
    // For trivial expansion, make sure we don't get \cs or
    // \relax\cs direct recursion!  Perl: Expandable.pm L81-89.
    //   if (!$onceonly && $$self{cs}) {
    //     my ($t0, $t1) = ($$expansion[0], $$expansion[1]);
    //     if ($t0 && ($t0->equals($$self{cs})
    //         || ($t1 && $t1->equals($$self{cs})
    //              && $t0->equals(T_CS('\protect'))))) {
    //       Error('recursion', $$self{cs}, …,
    //         "Token X expands into itself!", "defining as empty");
    //       $expansion = TokensI(); } }
    //
    // Detect `\def\foo{\foo}` and `\def\foo{\protect\foo}`. Both
    // are runaway-expansion landmines under any full-expansion
    // context (`\edef`, `\xdef`, `\write`, `\message`). Perl
    // reports an `Error:recursion` and substitutes an empty
    // expansion for this invocation; the stored definition is
    // unchanged (subsequent invocations re-detect and re-error).
    //
    // A previous Rust port tried to re-install the CS as
    // `Stored::Token(self.cs)` to preserve `\ifx` identity for
    // expl3 quarks (`\q_no_value`, `\q_nil`, …) and PGF keys
    // (`\pgfkeys@mainstop`). That was a no-op: `assign_meaning`'s
    // `token == mt` short-circuit (state.rs:1918-1922) rejects
    // the `\foo → \foo` self-let, so the Expandable definition
    // stayed in place and the recursion guard re-fired forever.
    // Witness: cleveref × algorithmicx × hyperref on 2403.15855,
    // where `\xdef\cref@currentprefix{\cref@currentprefix}` hung
    // at the 60 s wall-clock guard.
    //
    // Identity for expl3 quarks is independent of this path: the
    // quarks are defined `\cs_new_protected:Npn`, so they are
    // protected expandables. Under partial expansion (the normal
    // path) protected expandables aren't expanded at all — the
    // recursion guard never fires, and the stored body keeps the
    // CS as its first token, so `\ifx`-by-meaning comparisons
    // remain distinct. Under full expansion the Error+empty
    // recovery matches Perl exactly.
    if self.expands_into_itself(tokens, once_only) {
      Error!(
        "recursion",
        &self.cs.to_string(),
        s!("Token {} expands into itself!", self.cs)
      );
      Ok(&[])
    } else {
      Ok(tokens.unlist_ref())
    }
  }

  /// The substitution half of [`Definition::invoke_onto_input`], out of line: its argument arrays stay out of
  /// the frame of the argument reading, which recurses through `read_x_token`.
  #[inline(never)]
  fn substitute_onto_input(&self, tokens: &Tokens, args: &[ArgWrap]) -> Result<()> {
    // The arguments as token lists (borrowed: a macro's arguments are read as tokens); TeX's nine at most
    // (§476), so on the stack.
    let mut owned: [Option<Cow<'_, Tokens>>; 9] = Default::default();
    if args.len() > owned.len() {
      crate::gullet::unread_expansion(
        tokens.substitute_parameters(
          &args
            .iter()
            .map(|arg| arg.as_tokens())
            .collect::<Result<Vec<_>>>()?,
        ),
      );
      return Ok(());
    }
    for (slot, arg) in owned.iter_mut().zip(args.iter()) {
      *slot = arg.as_tokens()?;
    }
    let mut slices: [&[Token]; 9] = [&[]; 9];
    for (slice, arg) in slices.iter_mut().zip(owned.iter()) {
      if let Some(arg) = arg {
        *slice = arg.unlist_ref();
      }
    }
    let plan = self.plan.get_or_init(|| tokens.substitution_plan());
    crate::gullet::unread_substituted(tokens.unlist_ref(), plan, &slices[..args.len()]);
    Ok(())
  }

  /// A parameterless body that begins with the control sequence being expanded (or with `\protect` and it,
  /// while `\protect` is `\relax`) expands into itself: `Error:recursion`, and the call expands to nothing
  /// (Perl Expandable.pm L81-89; the cases below).
  fn expands_into_itself(&self, tokens: &Tokens, once_only: bool) -> bool {
    if !once_only {
      let token_vec = tokens.unlist_ref();
      let t0_opt = token_vec.first();
      let t1_opt = token_vec.get(1);
      // OXIDIZED_DESIGN #185: anchor on the token actually being
      // expanded, not the definition's home CS: a `\let` alias shares the Expandable
      // (`self.cs` = the original), and musixlyr.tex:709-722 legitimately
      // stores a body beginning with `\cont@<verse>` that is invoked
      // through the alias `\der@kontext` after the original was cleared
      // (Perl Expandable.pm:84 uses `$$self{cs}` and errs the same way:
      // recorder-fingering, undar-digitacion-doc). A genuine
      // `\def\x{\x}` invoked as `\x` still fires; through an alias the
      // loop is caught one step later when `\x` itself expands.
      let invoker = get_current_token().unwrap_or(self.cs);
      if let Some(t0) = t0_opt {
        if t0 == &self.cs && *t0 == invoker {
          true
        } else if let Some(t1) = t1_opt {
          // `\protect\foo` is only an actual runaway when
          // `\protect` currently expands to `\relax` (or is
          // undefined). Under `\protected@edef` it is `\let`
          // to `\@unexpandable@protect`, which turns the body
          // into `\noexpand\protect\noexpand\foo` — both tokens
          // become un-expandable and the loop terminates after
          // one expansion. msg.sty (loaded transitively from
          // french.sty, czech.sty, … under INCLUDE_STYLES=true)
          // uses exactly this idiom for `\msgheader`, so the
          // earlier blanket `\protect\foo`-is-runaway check
          // fired ~3 errors per language-style paper. Witness:
          // math9903002, gr-qc9511021, alg-geom9611022,
          // math9807030/.../math9810088 (8 papers).
          t1 == &self.cs && t0 == &T_CS!("\\protect") && protect_is_relax_or_undefined()
        } else {
          false
        }
      } else {
        false
      }
    } else {
      false
    }
  }

  /// Read this macro's arguments as tex.web §389-391 `macro_call` does, at
  /// `matching` status naming the invoked token (an alias's own name, as
  /// TeX's `warning_index:=cur_cs`). `None` abandons the call: a delimiter
  /// that does not match (§398), or an argument that ran off the end of a file
  /// ("File ended while scanning use of \foo", §338-339 and §392; or, for a
  /// non-`\long` macro whose argument held a `\par`, "Paragraph ended before
  /// \foo was complete", §396: [`crate::gullet::abandon_runaway_call`]).
  /// Guards: `scanner_status::*`.
  fn read_call_arguments(&self, parms: &Parameters) -> Result<Option<Vec<ArgWrap>>> {
    let cs = get_current_token().unwrap_or(self.cs);
    let matching = crate::gullet::set_scanner_status_checking_par(
      crate::gullet::ScannerStatus::Matching,
      Some(cs),
      self.checks_par,
    );
    let args = parms.read_macro_arguments(Some(self))?;
    match matching.end_taking_runaway_argument() {
      None => Ok(args),
      Some(runaway) => {
        crate::gullet::abandon_runaway_call(cs, self.is_long, self.checks_par, runaway)?;
        Ok(None)
      },
    }
  }

  pub fn new(
    cs: Token,
    paramlist: Option<Parameters>,
    mut expansion_opt: Option<ExpansionBody>,
    traits: Option<ExpandableOptions>,
  ) -> Result<Self> {
    let traits = traits.unwrap_or_default();
    if !traits.nopack_parameters
      && let Some(ExpansionBody::Tokens(expansion_tokens)) = expansion_opt
    {
      // Perl Core/Definition/Expandable.pm:35: FATAL if the expansion is
      // unbalanced (mismatched {/}). An Error-and-store-as-is let
      // jarticle.cls:94-97's `\ds@tate` (an ISO-2022-JP byte pair whose `%`
      // eats `\message`'s closing brace) proceed into japanese-otf's kanji
      // scanners, an aperiodic 250 s `\advance` loop the cycle guard cannot
      // see (platexcheat sample/platexsheet; Perl aborts in 1.3 s). Fatal
      // stays Fatal. Guard: `perfect_kernel_batch56::unbalanced_expansion_is_fatal`.
      if !expansion_tokens.is_balanced() {
        Fatal!(
          Stomach,
          Misdefined,
          s!("Expansion of '{}' has unbalanced {{}}", cs)
        );
      } else {
        expansion_opt = Some(ExpansionBody::Tokens(expansion_tokens.pack_parameters()?));
      }
    }
    let has_cc_arg = match expansion_opt {
      Some(ExpansionBody::Tokens(ref tks)) => tks
        .unlist_ref()
        .iter()
        .any(|t| t.get_catcode() == Catcode::ARG),
      _ => false,
    };
    // simplify: treat empty tokens as None
    let expansion = match expansion_opt {
      Some(ExpansionBody::Tokens(tks)) if tks.is_empty() => None,
      real_body => real_body,
    };

    // A `\newcommand` optional argument is read by `\@protected@testopt` →
    // `\kernel@ifnextchar` (latex.ltx:1249, 1259-1261), a futurelet peek.
    let peeks_by_futurelet = traits.peeks_by_futurelet
      || paramlist.as_ref().is_some_and(|params| {
        params
          .get_parameters()
          .first()
          .is_some_and(|param| param.testopt)
      });
    let is_long = traits.long || get_prefix_sym(crate::pin!("long"));
    let reads_arguments = paramlist
      .as_ref()
      .is_some_and(|params| !params.get_parameters().is_empty());
    Ok(Expandable {
      cs,
      paramlist,
      peeks_by_futurelet,
      expansion,
      // locator           => $source->getLocator,
      // Hot path: Expandable::new fires on every \def/\edef; pin!-cached keys
      // skip the per-call arena probe (same policy as Conditional::invoke).
      is_protected: traits.protected || get_prefix_sym(crate::pin!("protected")),
      is_outer: traits.outer || get_prefix_sym(crate::pin!("outer")),
      is_long,
      checks_par: !is_long
        && traits.tex_declared
        && reads_arguments
        && matches!(
          crate::definition::origin::current_origin(),
          crate::definition::origin::DefinitionOrigin::Format
            | crate::definition::origin::DefinitionOrigin::File
            | crate::definition::origin::DefinitionOrigin::Document
        ),
      has_cc_arg,
      alias: traits.alias,
      ..Expandable::default()
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn expandable_default_flags_false() {
    let e = Expandable::default();
    assert!(!e.is_protected);
    assert!(!e.peeks_by_futurelet);
    assert!(!e.is_long);
    assert!(!e.is_outer);
    assert!(!e.has_cc_arg);
    assert!(e.alias.is_none());
    assert!(e.paramlist.is_none());
    assert!(e.expansion.is_none());
  }

  #[test]
  fn expandable_default_has_default_cs() {
    let e = Expandable::default();
    // Default cs is a T_CS with empty text (produced by Token::default
    // or similar). We can at least confirm the code is CS.
    assert_eq!(e.cs.code, Catcode::CS);
  }

  #[test]
  fn expandable_is_definition_and_expandable() {
    let e = Expandable::default();
    assert!(e.is_definition());
    assert!(e.is_expandable());
  }

  #[test]
  fn expandable_partial_eq_by_paramlist_and_expansion() {
    // PartialEq ignores flags (protected/long/outer) and cs — it
    // compares paramlist and expansion only.
    let mut a = Expandable::default();
    let mut b = Expandable::default();
    // Both have paramlist=None, expansion=None → equal.
    assert_eq!(a, b);
    // Changing flags doesn't affect equality.
    a.is_protected = true;
    b.is_protected = false;
    assert_eq!(a, b);
  }

  #[test]
  fn expandable_get_num_args_zero_without_paramlist() {
    let e = Expandable::default();
    assert_eq!(e.get_num_args(), 0);
  }

  #[test]
  fn expandable_get_parameters_none_by_default() {
    let e = Expandable::default();
    assert!(e.get_parameters().is_none());
  }

  #[test]
  fn expandable_options_default_all_false() {
    let o = ExpandableOptions::default();
    assert!(!o.locked);
    assert!(!o.protected);
    assert!(!o.outer);
    assert!(!o.long);
    assert!(o.scope.is_none());
    assert!(o.alias.is_none());
    assert!(!o.mathactive);
    assert!(!o.robust);
    assert!(!o.nopack_parameters);
    assert!(!o.peeks_by_futurelet);
  }
}
