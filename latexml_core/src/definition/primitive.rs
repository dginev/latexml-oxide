use libxml::tree::Node;
use std::borrow::Cow;

use crate::Digested;
use crate::common::arena::SymHashMap;
use crate::common::error::*;
use crate::common::object::Object;
use crate::common::store::Stored;
use crate::definition::{
  BeforeDigestClosure, Definition, DigestionClosure, FontDirective, PrimitiveBody, Reversion,
};
use crate::document::Document;
use crate::parameter::Parameters;
use crate::pin;
use crate::state::Scope;
use crate::tbox::Tbox;
use crate::token::*;
use crate::tokens::Tokens;
use crate::whatsit::Whatsit;

#[derive(Clone, Default)]
pub struct PrimitiveOptions {
  pub bounded:          bool,
  pub is_prefix:        bool,
  pub require_math:     bool,
  pub forbid_math:      bool,
  pub robust:           bool,
  pub locked:           bool,
  pub enter_horizontal: bool,
  pub leave_horizontal: bool,
  pub nargs:            Option<usize>,
  pub scope:            Option<Scope>,
  pub font:             Option<FontDirective>,
  pub mode:             Option<String>,
  pub alias:            Option<String>,
  pub before_digest:    Vec<BeforeDigestClosure>,
  pub after_digest:     Vec<DigestionClosure>,
  pub reversion:        Option<Reversion>,
  /// The fontinfo lookup key for `\font`-defined primitives. See
  /// `Primitive::font_id`.
  pub font_id:          Option<crate::common::arena::data::SymStr>,
}

#[derive(Clone)]
pub struct Primitive {
  pub cs:            Token,
  pub paramlist:     Option<Parameters>,
  // TODO: we have a case where the replacement is a simple string/character
  //       which gets auto-wrapped with a Tbox during invoke.
  pub replacement:   Option<PrimitiveBody>,
  pub before_digest: Vec<BeforeDigestClosure>,
  pub after_digest:  Vec<DigestionClosure>,
  pub alias:         Option<String>,
  pub nargs:         Option<usize>,
  pub reversion:     Option<Reversion>,
  pub is_prefix:     bool,
  /// Set on `\font`-defined primitives (Perl `LaTeXML::Core::Definition::FontDef::fontID`).
  /// Holds the value-table key under which this CS's fontinfo hash lives
  /// (e.g. `\tenrm` → `Some("fontinfo_\\tenrm")`). Lets the dumper round-trip
  /// font-defined primitives via Perl's `FD(<cs>)` record (see
  /// `Core/Dumper.pm` L356-389) — closures aren't serializable but the
  /// font_id + the dumped `Stored::Font` value at that key let the reader
  /// rebuild an equivalent merge-font Primitive.
  pub font_id:       Option<crate::common::arena::data::SymStr>,
}
impl Default for Primitive {
  fn default() -> Self {
    Primitive {
      cs:            T_CS!("Primitive"),
      paramlist:     None,
      replacement:   None,
      alias:         None,
      before_digest: Vec::new(),
      after_digest:  Vec::new(),
      nargs:         None,
      reversion:     None,
      is_prefix:     false,
      font_id:       None,
    }
  }
}
impl PartialEq for Primitive {
  fn eq(&self, other: &Primitive) -> bool { self.cs == other.cs }
}

// impl fmt::Display for Primitive {
//   fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
//     todo!();
//   }
// }
impl Object for Primitive {
  fn stringify(&self) -> String { <Self as Definition>::stringify_type(self, "Primitive") }
}
impl Definition for Primitive {
  fn before_digest(&self) -> Option<&Vec<BeforeDigestClosure>> { Some(&self.before_digest) }
  fn after_digest(&self) -> Option<&Vec<DigestionClosure>> { Some(&self.after_digest) }
  fn is_prefix(&self) -> bool { self.is_prefix }

  fn invoke(&self, _once_only: bool) -> Result<Tokens> { Ok(Tokens!()) }
  fn invoke_primitive(&self) -> Result<Vec<Digested>> {
    Debug!("primitive invoke for {:?}", self.cs);
    // my $profiled = $state->lookupValue('PROFILING') && ($LaTeXML::CURRENT_TOKEN || $$self{cs});
    // my $tracing = $state->lookupValue('tracingcommands');
    // LaTeXML::Core::Definition::startProfiling($profiled, 'digest') if $profiled;
    // print STDERR '{' . $self->tracingCSName . "}\n" if $tracing;
    let mut invoked_boxes: Vec<Digested> = self.execute_before_digest()?;
    let args = self.read_arguments()?;
    // print STDERR $self->tracingArgs(@args) . "\n" if $tracing && @args;
    match self.replacement {
      Some(PrimitiveBody::Closure(ref closure)) => invoked_boxes.extend(closure(args)?),
      Some(PrimitiveBody::String(symbol)) => {
        // Perl L67: $stomach->enterHorizontal if defined $replacement
        crate::stomach::enter_horizontal();
        let cs_token = self
          .alias
          .as_ref()
          .map(|alias| Token::from(alias.as_str()))
          .unwrap_or(self.cs);
        let mut box_tokens = vec![cs_token];
        // Perl L69: append revertArguments for parameterized string primitives
        if let Some(ref params) = self.paramlist {
          for arg in &args {
            box_tokens.extend(arg.revert()?.unlist());
          }
          let _ = params; // acknowledge usage
        }
        let box_props = SymHashMap::default();
        invoked_boxes.push(Digested::from(Tbox::new(
          symbol,
          None,
          None,
          Tokens::new(box_tokens),
          box_props,
        )));
      },
      None => {
        // Perl: Box(undef, undef, undef, Tokens($self->getCSorAlias, ...), isEmpty => 1)
        // Even with no replacement, Perl creates a Box with the CS as reversion and isEmpty flag.
        // This is essential for font switches (\rm, \it, etc.) to appear in tex attributes.
        let cs_token = self
          .alias
          .as_ref()
          .map(|alias| Token::from(alias.as_str()))
          .unwrap_or(self.cs);
        let box_tokens = vec![cs_token];
        // TODO: add revert_arguments for ArgWrap type when needed
        let mut box_props = SymHashMap::default();
        box_props.insert("isEmpty", Stored::Bool(true));
        invoked_boxes.push(Digested::from(Tbox::new(
          pin!(""),
          None,
          None,
          Tokens::new(box_tokens),
          box_props,
        )));
      },
    }
    if !self.after_digest.is_empty() {
      // optimize to avoid needless generation of whatsits
      let mut w = Whatsit::default();
      let after_boxes = self.execute_after_digest(&mut w)?;
      invoked_boxes.extend(after_boxes);
    }

    // LaTeXML::Core::Definition::stopProfiling($profiled, 'digest') if $profiled;
    Ok(invoked_boxes)
  }

  fn do_absorption(&self, _document: &mut Document, _whatsit: &Whatsit) -> Result<Vec<Node>> {
    fatal!(
      Definition,
      Unexpected,
      "do_absorption on Primitive should never be called!"
    );
  }

  fn get_cs(&self) -> Cow<'_, Token> { Cow::Borrowed(&self.cs) }
  fn get_cs_name(&self) -> Cow<'_, str> { Cow::Owned(self.cs.with_cs_name(ToString::to_string)) }
  fn get_alias(&self) -> Option<&String> { self.alias.as_ref() }
  fn get_parameters(&self) -> Option<&Parameters> { self.paramlist.as_ref() }

  fn get_num_args(&self) -> usize {
    match self.nargs {
      Some(n) => n,
      None => match self.paramlist {
        Some(ref params) => params.get_num_args(),
        None => 0,
      },
    }
    // TODO: Rethink the memoize in this immutable setting
    // self.nargs = Some(nargs);
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::common::arena;

  #[test]
  fn primitive_default_fields() {
    let p = Primitive::default();
    assert_eq!(arena::to_string(p.cs.text), "Primitive");
    assert!(p.paramlist.is_none());
    assert!(p.replacement.is_none());
    assert!(p.alias.is_none());
    assert!(p.before_digest.is_empty());
    assert!(p.after_digest.is_empty());
    assert!(p.nargs.is_none());
    assert!(p.reversion.is_none());
    assert!(!p.is_prefix);
  }

  #[test]
  fn primitive_partial_eq_by_cs() {
    // PartialEq compares by cs only — Perl parity (closures can't
    // be structurally compared).
    // Primitive doesn't derive Debug, so assert_eq! / assert_ne!
    // can't format it on failure — use plain equality checks.
    let mut a = Primitive::default();
    let mut b = Primitive::default();
    a.cs = T_CS!("\\foo");
    b.cs = T_CS!("\\foo");
    assert!(a == b, "same cs should compare equal");
    b.cs = T_CS!("\\bar");
    assert!(!(a == b), "different cs should not be equal");
  }

  #[test]
  fn primitive_is_prefix_reflects_field() {
    let mut p = Primitive::default();
    assert!(!p.is_prefix());
    p.is_prefix = true;
    assert!(p.is_prefix());
  }

  #[test]
  fn primitive_get_num_args_zero_without_params() {
    let p = Primitive::default();
    assert_eq!(p.get_num_args(), 0);
  }

  #[test]
  fn primitive_get_num_args_uses_nargs_override() {
    // If nargs is explicitly set, it takes precedence over paramlist.
    let mut p = Primitive::default();
    p.nargs = Some(3);
    assert_eq!(p.get_num_args(), 3);
  }

  #[test]
  fn primitive_before_digest_ref_returns_some_empty() {
    let p = Primitive::default();
    let bd = p.before_digest().expect("Some(&Vec)");
    assert!(bd.is_empty());
  }

  #[test]
  fn primitive_after_digest_ref_returns_some_empty() {
    let p = Primitive::default();
    let ad = p.after_digest().expect("Some(&Vec)");
    assert!(ad.is_empty());
  }

  #[test]
  fn primitive_get_parameters_none_by_default() {
    let p = Primitive::default();
    assert!(p.get_parameters().is_none());
  }

  #[test]
  fn primitive_options_default_all_false() {
    let o = PrimitiveOptions::default();
    assert!(!o.bounded);
    assert!(!o.is_prefix);
    assert!(!o.require_math);
    assert!(!o.forbid_math);
    assert!(!o.robust);
    assert!(!o.locked);
    assert!(!o.enter_horizontal);
    assert!(!o.leave_horizontal);
    assert!(o.nargs.is_none());
    assert!(o.scope.is_none());
    assert!(o.font.is_none());
    assert!(o.mode.is_none());
    assert!(o.alias.is_none());
    assert!(o.before_digest.is_empty());
    assert!(o.after_digest.is_empty());
    assert!(o.reversion.is_none());
  }
}
