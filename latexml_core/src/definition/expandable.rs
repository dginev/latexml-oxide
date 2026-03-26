use std::borrow::Cow;
// use std::fmt;
use libxml::tree::Node;

use crate::common::error::*;
use crate::common::locator::Locator;
use crate::common::object::Object;
use crate::state::*;

use crate::Digested;
use crate::definition::{BeforeDigestClosure, Definition, DigestionClosure, ExpansionBody};
use crate::document::Document;
use crate::parameter::Parameters;
use crate::token::*;
use crate::tokens::{NO_TOKENS, Tokens};
use crate::whatsit::Whatsit;

#[derive(Debug, Clone, Default)]
pub struct ExpandableOptions {
  pub locked:            bool,
  pub protected:         bool,
  pub outer:             bool,
  pub long:              bool,
  pub scope:             Option<Scope>,
  pub alias:             Option<String>,
  pub mathactive:        bool,
  pub robust:            bool,
  pub nopack_parameters: bool,
}

#[derive(Debug, Clone)]
pub struct Expandable {
  pub is_protected: bool,
  pub is_long:      bool,
  pub is_outer:     bool,
  pub has_cc_arg:   bool,
  pub alias:        Option<String>,
  pub locator:      Locator,
  pub cs:           Token,
  pub paramlist:    Option<Parameters>,
  pub expansion:    Option<ExpansionBody>,
}
impl Default for Expandable {
  fn default() -> Self {
    Expandable {
      is_protected: false,
      is_long:      false,
      is_outer:     false,
      has_cc_arg:   false,
      alias:        None,
      locator:      Locator::default(),
      cs:           T_CS!("Expandable"),
      paramlist:    None,
      expansion:    None,
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
  fn get_locator(&self) -> Locator { self.locator }
  fn stringify(&self) -> String { <Self as Definition>::stringify_type(self, "Expandable") }
}
impl Definition for Expandable {
  fn is_protected(&self) -> bool { self.is_protected }
  fn get_parameters(&self) -> Option<&Parameters> { self.paramlist.as_ref() }
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
    // shortcut for "trivial" macros; but only if not tracing & profiling!!!!
    let _tracing = lookup_int("tracingmacros") > 0;
    let _profiled = lookup_bool("PROFILING");
    match &self.expansion {
      Some(ExpansionBody::Closure(closure)) => {
        // Harder to emulate \tracingmacros here.
        let args = if let Some(ref parms) = self.paramlist {
          parms.read_arguments(Some(self))?
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
          // For trivial expansion, make sure we don't get \cs or \relax\cs direct recursion!
          let is_recursion = if !once_only {
            let token_vec = tokens.unlist_ref();
            let t0_opt = token_vec.first();
            let t1_opt = token_vec.get(1);
            if let Some(t0) = t0_opt {
              if t0 == &self.cs {
                true
              } else if let Some(t1) = t1_opt {
                t1 == &self.cs && t0 == &T_CS!("\\protect")
              } else {
                false
              }
            } else {
              false
            }
          } else {
            false
          };
          if is_recursion {
            // TODO: port the Error
            // if (!$onceonly && $$self{cs}) {
            //   my ($t0, $t1) = ($etype eq 'LaTeXML::Core::Tokens'
            //     ? ($$expansion[0], $$expansion[1]) : ($expansion, undef));
            //   if ($t0 && ($t0->equals($$self{cs})
            //       || ($t1 && $t1->equals($$self{cs}) && $t0->equals(T_CS('\protect'))))) {
            Error!(
              "recursion",
              self.cs,
              format!("Token {} expands into itself!", self.cs),
              "defining as empty"
            );
            //     $expansion = TokensI(); } }
            Tokens!()
          } else {
            tokens.clone()
          }
        } else {
          let args = if let Some(ref parms) = self.paramlist {
            parms.read_arguments(Some(self))?
          } else {
            Vec::new()
          };
          if self.has_cc_arg {
            // Do we actually need to substitute the args in?
            let mut args_tks = Vec::new();
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
          parms.read_arguments(Some(self))?;
        }
        Ok(NO_TOKENS)
      },
    }
  }

  // Not implemented for expandable
  fn invoke_primitive(&self) -> Result<Vec<Digested>> { Ok(Vec::new()) }
  fn before_digest(&self) -> Option<&Vec<BeforeDigestClosure>> { None }
  fn after_digest(&self) -> Option<&Vec<DigestionClosure>> { None }
  fn do_absorbtion(&self, _document: &mut Document, _whatsit: &Whatsit) -> Result<Vec<Node>> {
    fatal!(
      Definition,
      Unexpected,
      "do_absorbtion on Expandable should never be called!"
    );
  }
}

impl Expandable {
  pub fn new(
    cs: Token,
    paramlist: Option<Parameters>,
    mut expansion_opt: Option<ExpansionBody>,
    traits: Option<ExpandableOptions>,
  ) -> Result<Self> {
    let traits = traits.unwrap_or_default();
    if !traits.nopack_parameters {
      if let Some(ExpansionBody::Tokens(expansion_tokens)) = expansion_opt {
        // Perl: Fatal if expansion is unbalanced (mismatched {/})
        if !expansion_tokens.is_balanced() {
          Error!(
            "misdefined",
            cs,
            s!("Expansion of '{}' has unbalanced {{}}", cs),
            "skipping pack_parameters"
          );
          // Store as-is without packing
          expansion_opt = Some(ExpansionBody::Tokens(expansion_tokens));
        } else {
          expansion_opt = Some(ExpansionBody::Tokens(expansion_tokens.pack_parameters()?));
        }
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

    Ok(Expandable {
      cs,
      paramlist,
      expansion,
      // locator           => $source->getLocator,
      is_protected: traits.protected || get_prefix("protected"),
      is_outer: traits.outer || get_prefix("outer"),
      is_long: traits.long || get_prefix("long"),
      has_cc_arg,
      ..Expandable::default()
    })
  }
}
