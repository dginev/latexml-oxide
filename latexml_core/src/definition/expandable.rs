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
  /// A binding's port of a TeX macro that is not `\long`: it checks its arguments for `\par` as the original does
  /// ([`Expandable::checks_par`]), wherever it is made. amsppt.sty:804/:814's `\footnote`/`\footnotetext`, whose
  /// arguments stop at a paragraph's end (amsppt_sty.rs).
  pub short:              bool,
  /// Wraps the name's current meaning and delegates to it (OmniBus's `\bibitem`, which loads natbib for a
  /// `[\protect\citeauthoryear…]` label and otherwise calls the `\bibitem` it saved): it extends the name rather
  /// than replacing it, so a provide-only load installs it too (`state::provide_only_declines`).
  pub extends:            bool,
  /// The arguments (their numbers, `#n`) this binding macro drops by design: the real package or class prints nothing of
  /// them (hyperref's `\texorpdfstring` #2, the PDF bookmark's text), or they are page furniture (FIDELITY_CHARTER §2).
  /// The title-page check wants none of their words (`state::DocumentArgument::used`, OXIDIZED_DESIGN_DIVERGENCES #953
  /// "106t"); the expansion is unchanged ([`Expandable::dropped_by_design`]).
  pub drops:              &'static [u8],
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
  /// A document's parameterless override (a value) of a locked binding that owns its calls (`state::install_definition`): a call — the
  /// next raw token an explicit `{`, or a `[`/`*` the binding's signature opens with — runs the binding, kept as
  /// `\lx@locked@<cs>`; any other use expands this definition. Not part of the meaning (`PartialEq`).
  pub yields_calls:       bool,
  /// The token-list body's [`Tokens::substitution_plan`], built at the first call that substitutes.
  pub plan:               std::cell::OnceCell<Box<[BodyPiece]>>,
  /// The arguments the binding declares this macro drops by design ([`ExpandableOptions::drops`]), bit `n` for `#n`.
  /// Not part of the meaning (`PartialEq`).
  pub dropped_by_design:  u16,
  /// Made while a raw class or a raw file of the document's own directory was read
  /// ([`crate::definition::origin::reading_own_file`]), which the title-page check's third source reads
  /// (`Self::watched_call`). Not part of the meaning (`PartialEq`).
  pub made_in_own_file:   bool,
  /// The body's [`StoredParameters`], read at the first call the store watch sees.
  pub stored:             std::cell::OnceCell<Box<StoredParameters>>,
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
      yields_calls:       false,
      plan:               std::cell::OnceCell::new(),
      made_in_own_file:   false,
      stored:             std::cell::OnceCell::new(),
      dropped_by_design:  0,
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
    if self.may_be_watched()
      && let Some(watched) = self.watched_call()
    {
      return self.expand_watched(once_only, watched);
    }
    self.expand(once_only)
  }

  /// [`Self::invoke`] onto the input, a token-list body without building the expansion: the parameterless
  /// body is pushed as stored, a body with parameters through its substitution plan
  /// ([`crate::gullet::unread_substituted`]). Closures, argument-only definitions and a frontmatter setter (whose
  /// expansion is noted, as [`Self::invoke`] notes it) take the general path.
  fn invoke_onto_input(&self, once_only: bool) -> Result<()> {
    if self.may_be_watched()
      && let Some(watched) = self.watched_call()
    {
      crate::gullet::unread_expansion(self.expand_watched(once_only, watched)?);
      return Ok(());
    }
    if self.yields_calls
      && let Some(owner) = self.call_owner()?
    {
      crate::gullet::unread_expansion(Tokens!(owner));
      return Ok(());
    }
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

/// What a setter watch observes of a macro call ([`Expandable::watched_call`]).
#[derive(Clone, Copy)]
struct WatchedCall {
  /// a frontmatter setter's: its expansion noted ([`note_frontmatter_setter_expansion`])
  noted:    bool,
  /// a document call's: its arguments kept ([`note_document_call`])
  recorded: bool,
  /// a binding macro's while the document's frontmatter entry is digested: the arguments it drops kept
  /// ([`note_dropped_argument`])
  dropping: bool,
  /// a raw class's or own-directory file's setter's while the document's text is read: the arguments it stores kept
  /// ([`note_store_call`])
  stored:   bool,
}

impl Expandable {
  /// Can a watch that is on see this call at all ([`crate::definition::origin::call_watch`])? [`Self::watched_call`] says
  /// what it observes, on a cold path.
  #[inline(always)]
  fn may_be_watched(&self) -> bool {
    crate::definition::origin::call_watch().is_some_and(|only_owned_with_parameters| {
      !only_owned_with_parameters
        || (self.paramlist.is_some() && (self.made_in_own_file || self.origin.is_latexml_owned()))
    })
  }

  /// What a setter watch (`definition::origin::setter_watch`) observes of this call, if anything: a frontmatter setter's
  /// ([`note_setter_call`], whose expansion is noted), a call of LaTeXML's own macro that reads its arguments from the
  /// document's text (`origin::document_call_watch`, [`crate::gullet::reading_document_text`]: its arguments kept,
  /// [`note_document_call`]), and a binding's macro called while a frontmatter entry the document gave is digested
  /// (`origin::dropped_argument_watch`, `DefinitionOrigin::is_binding_made`: the arguments it drops kept,
  /// [`note_dropped_argument`]); and a setter a raw class or a file of the document's own directory made, while the
  /// current mouth reads the document's text, before the title page (`origin::store_call_watch`, off from the title page
  /// on: the arguments it writes into a store kept, [`note_store_call`]) — its arguments may come back through
  /// `\@ifnextchar` (nj_style's `\ead`, CUP-JNL-DTM's `\address`), so no [`crate::gullet::reading_document_text`]: a class's
  /// own literal is no word the document's source writes. Not while a raw class's setter replays or a speculative
  /// run digests.
  #[cold]
  #[inline(never)]
  fn watched_call(&self) -> Option<WatchedCall> {
    let noted = crate::definition::origin::setter_name_watch() && note_setter_call(&self.cs);
    let recorded = crate::definition::origin::document_call_watch()
      && self.paramlist.is_some()
      && self.origin.is_latexml_owned()
      && crate::gullet::reading_document_text();
    let dropping = crate::definition::origin::dropped_argument_watch()
      && self.paramlist.is_some()
      && self.origin.is_binding_made()
      && !matches!(self.expansion, Some(ExpansionBody::Closure(_)));
    let stored = crate::definition::origin::store_call_watch()
      && self.paramlist.is_some()
      && self.made_in_own_file
      && self.stored_parameters().numbers != 0
      && !lookup_bool("lx_replaying_class_setter")
      && !lookup_bool("lx_depositing_class_maketitle");
    (noted || recorded || dropping || stored).then_some(WatchedCall {
      noted,
      recorded,
      dropping,
      stored,
    })
  }

  /// What this macro's token body writes into a store ([`stored_parameters`]), once per definition; nothing of a closure.
  fn stored_parameters(&self) -> &StoredParameters {
    self.stored.get_or_init(|| {
      Box::new(match self.expansion.as_ref() {
        Some(ExpansionBody::Tokens(body)) => stored_parameters(body.unlist_ref()),
        _ => StoredParameters::default(),
      })
    })
  }

  /// [`Self::expand`] for a watched call ([`Self::watched_call`]): a frontmatter setter's expansion noted once its
  /// arguments are read, the input after them what it is read before (the frontmatter observer's ledger); a document
  /// call's arguments kept, each with whether its parameter is optional and whether the body names it (the title-page
  /// check); and the required arguments a binding's macro drops while a document's frontmatter entry is digested.
  #[cold]
  #[inline(never)]
  fn expand_watched(&self, once_only: bool, watched: WatchedCall) -> Result<Tokens> {
    let expansion = if watched.recorded || watched.dropping || watched.stored {
      let mut captured = None;
      let expansion = self.expand_capturing::<true>(once_only, &mut captured)?;
      if let (Some(args), Some(parms)) = (captured, self.paramlist.as_ref()) {
        let parameters = parms.get_parameters();
        // (an optional argument, or one set in brackets by a `\def`'s parameter text, is a store's key or label:
        // `\@@address[#1]#2`, `\cortext[cor1]{…}` of an elsarticle copy, 1603.09660, 1005.1545)
        let keyed: Vec<bool> = parameters
          .iter()
          .filter(|parameter| !parameter.novalue)
          .map(|parameter| parameter.optional || parameter.spec == crate::pin!("Until:]"))
          .collect();
        let optional = parameters
          .iter()
          .filter(|parameter| !parameter.novalue)
          .map(|parameter| parameter.optional);
        // (a parameter a token body never names, `#n`, is dropped by the call: quantumarticle's `\ead[]{}` →
        // `\email{#1}` drops the address; a closure's use is unknown, so its arguments are taken as handed on; one the
        // binding declares dropped by design is accounted for — hyperref's `\texorpdfstring` #2, 2306.06741; and only
        // a binding's drop can lose what pdflatex prints: the kernel's is TeX's own, as the format's `\@firstoftwo{T}{F}`
        // drops the branch ifthen's `\ifthenelse` does not take around a document's `\title`, 2202.08832)
        let used = |at: usize| {
          !self.origin.is_binding_made()
            || self.dropped_by_design & (1 << at) != 0
            || match self.expansion.as_ref() {
              Some(ExpansionBody::Tokens(body)) => names_parameter(body, at),
              Some(ExpansionBody::Closure(_)) => true,
              None => false,
            }
        };
        let args: Vec<DocumentArgument> = optional
          .zip(args.iter())
          .enumerate()
          .filter_map(|(at, (optional, arg))| match arg {
            ArgWrap::Tokens(tokens) => Some(DocumentArgument {
              number: at + 1,
              optional,
              used: used(at + 1),
              tokens: tokens.clone(),
            }),
            _ => None,
          })
          .collect();
        if watched.dropping {
          for arg in args.iter().filter(|arg| !arg.optional && !arg.used) {
            note_dropped_argument(self.cs, arg);
          }
        }
        if watched.stored {
          let stored = self.stored_parameters();
          let (keys, values): (Vec<_>, Vec<_>) = args
            .iter()
            .filter(|arg| stored.numbers & (1 << arg.number) != 0 && !arg.tokens.is_empty())
            .partition(|arg| keyed.get(arg.number - 1).copied().unwrap_or(false));
          let tokens =
            |args: Vec<&DocumentArgument>| args.into_iter().map(|arg| arg.tokens.clone()).collect();
          note_store_call(self.cs, stored, tokens(values), tokens(keys));
        }
        if watched.recorded {
          note_document_call(self.cs, args);
        }
      }
      expansion
    } else {
      self.expand(once_only)?
    };
    if watched.noted {
      note_frontmatter_setter_expansion(&self.cs);
    }
    Ok(expansion)
  }

  /// The expansion of a call ([`Definition::invoke`]), its arguments read.
  #[inline(always)]
  fn expand(&self, once_only: bool) -> Result<Tokens> {
    self.expand_capturing::<false>(once_only, &mut None)
  }

  /// [`Self::expand`], its arguments copied into `captured` when `CAPTURE` ([`Self::expand_watched`]).
  #[inline(always)]
  fn expand_capturing<const CAPTURE: bool>(
    &self,
    once_only: bool,
    captured: &mut Option<Vec<ArgWrap>>,
  ) -> Result<Tokens> {
    // Perl shortcut for "trivial" macros that were tracing- or
    // profiling-aware. Neither tracing nor profiling is implemented
    // in the Rust port (the returned `_tracing` / `_profiled` values
    // were discarded), so the two state lookups were pure overhead
    // on every macro expansion (\~350k calls in si.tex alone per
    // callgrind). Removed — re-introduce only alongside the actual
    // tracing/profiling features if/when they land.
    if self.yields_calls
      && let Some(owner) = self.call_owner()?
    {
      return Ok(Tokens!(owner));
    }
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
        if CAPTURE {
          *captured = Some(args.clone());
        }
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
          if CAPTURE {
            *captured = Some(args.clone());
          }
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
          let args = self.read_call_arguments(parms)?;
          if CAPTURE {
            *captured = args;
          }
        }
        Ok(NO_TOKENS)
      },
    }
  }

  /// For a [`Self::yields_calls`] override: `\lx@locked@<cs>`, the locked binding, when the next raw token opens a
  /// call of it — an explicit catcode-1 `{`, or a `[`/`*` its signature opens with
  /// ([`crate::state::call_openers`]) — and `None` for any other use, which expands the override. The peek reads
  /// without expanding and puts the token back. Like TeX's argument scan it may read past the end of the macro body
  /// the override was met in: `\def\wrap{\text}\wrap{x}` runs the binding on `{x}`. In an expansion-only context
  /// (`\edef`, `\write`) the kept name `\lx@locked@<cs>` is what remains (OXIDIZED_DESIGN #500 residual).
  fn call_owner(&self) -> Result<Option<Token>> {
    let Some(next) = crate::gullet::peek_token()? else {
      return Ok(None);
    };
    let owner = locked_binding_token(&self.cs);
    let opens_call = next.get_catcode() == Catcode::BEGIN
      || next.get_catcode() == Catcode::OTHER && call_openers(&owner).contains(&next);
    Ok(opens_call.then_some(owner))
  }

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
    // A macro's arguments are read as tokens, so they're there to read in place (tex.web §390).
    if args.iter().all(|arg| matches!(arg, ArgWrap::Tokens(_))) {
      let plan = self.plan.get_or_init(|| tokens.substitution_plan());
      crate::gullet::unread_substituted_args(tokens.unlist_ref(), plan, args);
      return Ok(());
    }
    // The arguments as token lists; TeX's nine at most (§476), so on the stack.
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
        && reads_arguments
        && (traits.short
          || traits.tex_declared
            && matches!(
              crate::definition::origin::current_origin(),
              crate::definition::origin::DefinitionOrigin::Format
                | crate::definition::origin::DefinitionOrigin::File
                | crate::definition::origin::DefinitionOrigin::Document
            )),
      has_cc_arg,
      alias: traits.alias,
      dropped_by_design: traits.drops.iter().fold(0, |bits, &number| {
        // (a macro's arguments are #1-#9: a bit each in the u16)
        debug_assert!(number <= 9, "a dropped argument number past #9: {number}");
        bits | 1 << number
      }),
      made_in_own_file: crate::definition::origin::reading_own_file(),
      ..Expandable::default()
    })
  }
}

/// The kernel's commands that write a definition's name: the `\def` family, `\let`, `\g@addto@macro`. The unclaimed-text
/// net's stores read these alone (latexml_engine `frontmatter_stores::stores_raw_code_writes`), which change the output
/// where the check's cannot; the check reads [`ETOOLBOX_WRITERS_OF_NAMES`] too.
pub const KERNEL_WRITERS_OF_NAMES: &[&str] = &[
  "\\def",
  "\\gdef",
  "\\edef",
  "\\xdef",
  "\\let",
  "\\protected@edef",
  "\\protected@xdef",
  "\\g@addto@macro",
];

/// The other commands that write a definition's name, the title-page check's alone (`stored_parameters`, latexml_engine
/// `title_page_check::StoreReaders`): etoolbox's, and the kernel's `\@namedef`.
pub const ETOOLBOX_WRITERS_OF_NAMES: &[&str] = &[
  "\\@namedef",
  "\\appto",
  "\\gappto",
  "\\eappto",
  "\\xappto",
  "\\preto",
  "\\gpreto",
  "\\epreto",
  "\\xpreto",
  "\\listadd",
  "\\listgadd",
  "\\listeadd",
  "\\listxadd",
  "\\csdef",
  "\\csgdef",
  "\\csedef",
  "\\csxdef",
  "\\csappto",
  "\\csgappto",
  "\\cseappto",
  "\\csxappto",
  "\\cspreto",
  "\\csgpreto",
  "\\csepreto",
  "\\csxpreto",
  "\\listcsadd",
  "\\listcsgadd",
  "\\listcseadd",
  "\\listcsxadd",
];

/// The commands that write a definition's name: [`KERNEL_WRITERS_OF_NAMES`] and [`ETOOLBOX_WRITERS_OF_NAMES`], one list
/// to search (`stored_parameters` asks for every control sequence of every definition's body).
pub const WRITERS_OF_NAMES: [&str;
  KERNEL_WRITERS_OF_NAMES.len() + ETOOLBOX_WRITERS_OF_NAMES.len()] = {
  let mut all = [""; KERNEL_WRITERS_OF_NAMES.len() + ETOOLBOX_WRITERS_OF_NAMES.len()];
  let mut at = 0;
  while at < all.len() {
    all[at] = if at < KERNEL_WRITERS_OF_NAMES.len() {
      KERNEL_WRITERS_OF_NAMES[at]
    } else {
      ETOOLBOX_WRITERS_OF_NAMES[at - KERNEL_WRITERS_OF_NAMES.len()]
    };
    at += 1;
  }
  all
};

/// A writer of [`WRITERS_OF_NAMES`] whose value joins what its store holds, past `\g@addto@macro`.
const APPENDERS: &[&str] = &[
  "\\g@addto@macro",
  "\\appto",
  "\\gappto",
  "\\eappto",
  "\\xappto",
  "\\preto",
  "\\gpreto",
  "\\epreto",
  "\\xpreto",
  "\\listadd",
  "\\listgadd",
  "\\listeadd",
  "\\listxadd",
  "\\csappto",
  "\\csgappto",
  "\\cseappto",
  "\\csxappto",
  "\\cspreto",
  "\\csgpreto",
  "\\csepreto",
  "\\csxpreto",
  "\\listcsadd",
  "\\listcsgadd",
  "\\listcseadd",
  "\\listcsxadd",
];

/// The kernel's and the common classes' scratch names, which code writes and reads again at once: no store a setter fills,
/// whatever else reads them later (`\subject{secondary}` through `\@tempa`, 1803.03153; `\newtheorem*` through
/// `\reserved@a`, 1912.10847; 2505.24263). What they hold hands on to what they are written into (`stored_parameters`).
const SCRATCH_NAMES: &[&str] = &[
  "\\@tempa",
  "\\@tempb",
  "\\@tempc",
  "\\@tempd",
  "\\@tempe",
  "\\@tempf",
  "\\@gtempa",
  "\\@temp",
  "\\@tmp",
  "\\@tmpa",
  "\\@tmpb",
  "\\@tmpc",
  "\\temp",
  "\\tempa",
  "\\tempb",
  "\\tempc",
  "\\tmp",
  "\\tmpa",
  "\\tmpb",
  "\\tmpc",
  "\\reserved@a",
  "\\reserved@b",
  "\\reserved@c",
  "\\reserved@d",
  "\\reserved@e",
  "\\reserved@f",
  "\\toks@",
  "\\@temptokena",
  "\\@toks",
];

/// Is the control sequence named `name` a scratch name (`SCRATCH_NAMES`)?
pub fn is_scratch_name(name: &str) -> bool { SCRATCH_NAMES.contains(&name) }

/// What a raw setter's body writes into a store (`stored_parameters`).
#[derive(Debug, Clone, Default)]
pub struct StoredParameters {
  /// a bit `n` for each `#n` written
  pub numbers:     u16,
  /// each write replaces a known store, and none reads a store written before it: a later call of the macro supersedes
  /// this one's text (`\def\@x{#1}`, as against `\g@addto@macro\@x{#1}`, `\xdef\@x{\@x #1}`, `\write`, a `\csname`
  /// store, `\def\@x{#1}\xdef\y{\y\@x}`)
  pub replaces:    bool,
  /// the stores written, by name (a `\csname…\endcsname` one has none: no reader of it can be told)
  pub stores:      Vec<Token>,
  /// one is written out (`\write`), whose reader is the next run
  pub written_out: bool,
}

/// The parameters the token body `body` writes into a store: inside the value `{…}` of a writer of a named store
/// ([`WRITERS_OF_NAMES`] but `\let`, which takes one name; `\renewcommand`, `\newcommand`, `\providecommand`), of a token
/// register's assignment, or of a `\write` to a named stream. CUP-JNL-DTM.cls's `\g@addto@macro\auaddress{…#2…}`
/// (2305.00867), aeb_emulateapj's `\global\alt@affil@toks\expandafter{\the\alt@affil@toks … #2}` (1011.2770),
/// naverlabseurope.cls's `\def\@authors{#1}` (2408.05088), bosonai_techreport.sty's
/// `\renewcommand{\@affiliation}{#1}` (2605.02209), nj_style's `\immediate\write\@auxout{…{#1}…}` (1401.7920). A store the
/// body names by a parameter (`\gdef#1{#2}`) is a shorthand maker's, none; a `\csname` name is one, by no name. A name
/// written earlier in the body hands its parameters on to a value that names it (`\toks@{#1}\xdef\x{\the\toks@}`,
/// `\def\@tempa{#1}\global\let\x\@tempa`, SCIS2026.cls's temporary `\@authoremail`); a scratch name itself
/// ([`SCRATCH_NAMES`]) is no store.
fn stored_parameters(body: &[Token]) -> StoredParameters {
  const WRITERS_OF_COMMANDS: &[&str] = &["\\renewcommand", "\\newcommand", "\\providecommand"];
  let mut stored = StoredParameters {
    replaces: true,
    ..StoredParameters::default()
  };
  // (what each name written so far holds: its parameters, and every token of its value)
  let mut written: Vec<(Token, u16, Vec<Token>)> = Vec::new();
  let mut at = 0;
  while at < body.len() {
    let token = &body[at];
    at += 1;
    if token.get_catcode() != Catcode::CS {
      continue;
    }
    let is = |set: &[&str]| token.with_str(|name| set.contains(&name));
    let register = at >= 2 && body[at - 2].with_str(|name| name == "\\the");
    let is_let = token.with_str(|name| name == "\\let");
    // (the store's name, and from where the search for its value starts)
    let (store, value_search) = if is(WRITERS_OF_COMMANDS) || is(&WRITERS_OF_NAMES) {
      match written_store_name(body, at) {
        Some(found) => found,
        None => continue,
      }
    } else if token.with_str(|name| name == "\\write") {
      match body.get(at) {
        Some(stream) if stream.get_catcode() == Catcode::CS => (Some(*stream), at + 1),
        _ => continue,
      }
    } else if !register && is_token_register(token) {
      (Some(*token), at)
    } else {
      continue;
    };
    let scratch = store.is_some_and(|store| store.with_str(is_scratch_name));
    // (a value: a group's inside, or `\let`'s one token)
    let range = if is_let {
      let mut from = value_search;
      while body
        .get(from)
        .is_some_and(|t| t.get_catcode() == Catcode::SPACE || t.with_str(|text| text == "="))
      {
        from += 1;
      }
      (from < body.len()).then_some((from, from + 1))
    } else {
      value_group(body, value_search, token)
    };
    let Some((from, to)) = range else {
      continue;
    };
    let value = &body[from..to];
    // (a write that reads a store this body wrote first: `\gdef\@tmpz{#1}\g@addto@macro\x\@tmpz`, so a later call does
    // not supersede this one. Two stores side by side do: acm's `\conferenceinfo`'s `\global\conf={#1}\global\confinfo{#2}`,
    // called twice, 1612.06202)
    if value.iter().any(|inner| stored.stores.contains(inner)) {
      stored.replaces = false;
    }
    let mut named = 0u16;
    let mut holds: Vec<Token> = value.to_vec();
    for inner in value {
      if inner.get_catcode() == Catcode::ARG
        && let Some(number) = inner.with_str(|text| text.parse::<u16>().ok())
      {
        named |= 1 << number;
      } else if let Some((_, numbers, tokens)) = written.iter().find(|(name, ..)| name == inner) {
        named |= numbers;
        holds.extend_from_slice(tokens);
      }
    }
    if named != 0 && !scratch {
      stored.numbers |= named;
      let appends = is(APPENDERS) || token.with_str(|name| name == "\\write");
      match store {
        // (a `\csname` store: which one a later call writes, its counter's next, cannot be told)
        None => stored.replaces = false,
        Some(store) => {
          if appends || holds.contains(&store) {
            stored.replaces = false;
          }
          if token.with_str(|name| name == "\\write") {
            stored.written_out = true;
          } else {
            stored.stores.push(store);
          }
        },
      }
    }
    if let Some(store) = store
      && !(token.with_str(|name| name == "\\write"))
    {
      written.retain(|(name, ..)| *name != store);
      written.push((store, named, holds));
    }
    at = to;
  }
  stored
}

/// The store a writer at `at - 1` names at `at` (`\x`, `{\x}`, `*\x`, `\expandafter\x`, a `\csname…\endcsname` or the
/// group `\@namedef` and etoolbox's `\csgdef` take: none known then), and where the search for its value starts; `None`
/// when it names no store (`\gdef#1{#2}`).
pub fn written_store_name(body: &[Token], mut at: usize) -> Option<(Option<Token>, usize)> {
  while let Some(token) = body.get(at)
    && (token.get_catcode() == Catcode::SPACE
      || token.with_str(|text| text == "*" || text == "\\expandafter"))
  {
    at += 1;
  }
  let token = body.get(at)?;
  match token.get_catcode() {
    Catcode::CS if token.with_str(|name| name == "\\csname") => {
      let end = body[at..]
        .iter()
        .position(|t| t.with_str(|name| name == "\\endcsname"))?;
      Some((None, at + end + 1))
    },
    Catcode::CS | Catcode::ACTIVE => Some((Some(*token), at + 1)),
    Catcode::BEGIN => {
      let name = body.get(at + 1)?;
      let close = body.get(at + 2)?;
      if name.get_catcode() == Catcode::CS && close.get_catcode() == Catcode::END {
        return Some((Some(*name), at + 3));
      }
      // (a name built in a group)
      let mut depth = 0usize;
      for (offset, token) in body[at..].iter().enumerate() {
        match token.get_catcode() {
          Catcode::BEGIN => depth += 1,
          Catcode::END => {
            depth -= 1;
            if depth == 0 {
              return Some((None, at + offset + 1));
            }
          },
          _ => {},
        }
      }
      None
    },
    _ => None,
  }
}

/// The inside of the value group of `writer`, searched from `at`: the next `{…}`, balanced. A `\def`'s parameter text may
/// stand before it, a `\newcommand`'s `[n]`; a register's assignment or a `\write` has only `=`, spaces and `\expandafter`
/// there (`\toks@\the\x` assigns no group; `\@auxout\foo`).
pub fn value_group(body: &[Token], mut at: usize, writer: &Token) -> Option<(usize, usize)> {
  let by_group_only = !writer.with_str(|name| {
    WRITERS_OF_NAMES.contains(&name)
      || matches!(name, "\\renewcommand" | "\\newcommand" | "\\providecommand")
  });
  while let Some(token) = body.get(at) {
    match token.get_catcode() {
      Catcode::BEGIN => break,
      Catcode::END => return None,
      Catcode::SPACE => {},
      _ if !by_group_only => {},
      _ if token.with_str(|text| text == "=" || text == "\\expandafter") => {},
      _ => return None,
    }
    at += 1;
  }
  let from = at + 1;
  let mut depth = 0usize;
  for (offset, token) in body.get(at..)?.iter().enumerate() {
    match token.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => {
        depth -= 1;
        if depth == 0 {
          return Some((from, at + offset));
        }
      },
      _ => {},
    }
  }
  None
}

/// Does the token body `body` name its parameter `#number` (an argument token, `Catcode::ARG`)?
fn names_parameter(body: &Tokens, number: usize) -> bool {
  body.unlist_ref().iter().any(|token| {
    token.get_catcode() == Catcode::ARG && token.with_str(|text| text == number.to_string())
  })
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
