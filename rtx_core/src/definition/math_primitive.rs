use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use libxml::tree::Node;

use crate::common::error::*;
// use crate::common::font::Font;
use crate::common::object::Object;
use crate::common::store::Stored;
use crate::state::{Scope, State};

use crate::definition::{BeforeDigestClosure, Definition, DigestionClosure, FontDirective, PrimitiveClosure, ConstructionClosure, Reversion};
use crate::document::Document;
use crate::gullet::Gullet;
use crate::parameter::Parameters;
use crate::stomach::Stomach;
use crate::token::*;
use crate::tokens::Tokens;
use crate::whatsit::Whatsit;
use crate::{Digested, Locator};

use super::SizingClosure;

// DefMath Define a Mathematical symbol or function.
// There are two sets of cases:
//  (1) If the presentation appears to be TeX code, we create an XMDual,
// since the presentation may end up with structure, etc.
//  (2) But if the presentation is a simple string, or unicode,
// it is just the content of the symbol; even if the function takes arguments.
// ALSO
//  arrange that the operator token gets cs="$cs"
// ALSO
//  Possibly some trick with SUMOP/INTOP affecting limits ?
//  Well, not exactly, but....
// HMM.... Still fishy.
// When to make a dual ?
// If the $presentation seems to be TeX (ie. it involves #1... but not ONLY!)

#[derive(Clone)]
pub struct MathPrimitiveOptions {
  pub bounded: bool,
  pub mode: Option<String>,
  pub before_digest: Vec<BeforeDigestClosure>,
  pub after_digest: Vec<DigestionClosure>,
  pub before_construct: Vec<ConstructionClosure>,
  pub after_construct: Vec<ConstructionClosure>,
  pub is_prefix: bool,
  pub scope: Option<Scope>,
  pub font: Option<FontDirective>,
  pub require_math: bool,
  pub forbid_math: bool,
  pub locked: bool,
  pub alias: Option<String>,
  pub decl_id: Option<String>,
  pub replace: Option<String>,
  pub protected: bool,
  pub robust: bool,

  // Math specific
  pub name: Option<String>,
  pub meaning: Option<String>,
  pub omcd: Option<String>,
  pub reversion: Option<Reversion>,
  pub sizer: Option<SizingClosure>,
  pub role: Option<String>,
  pub operator_role: Option<String>,
  pub reorder: bool,
  pub dual: bool,
  pub mathstyle: Option<String>,
  pub scriptpos: Option<usize>,
  pub operator_scriptpos: Option<usize>,
  pub stretchy: Option<bool>,
  pub operator_stretchy: Option<bool>,
  pub nogroup: bool,
  pub hide_content_reversion: bool,
  pub revert_as: Option<Cow<'static, str>>,
  pub lpadding: Option<usize>,
  pub rpadding: Option<usize>,
}
impl Default for MathPrimitiveOptions {
  fn default() -> Self {
    MathPrimitiveOptions {
      bounded: false,
      before_digest: Vec::new(),
      after_digest: Vec::new(),
      before_construct: Vec::new(),
      after_construct: Vec::new(),
      mode: None,
      is_prefix: false,
      scope: None,
      require_math: false,
      forbid_math: false,
      locked: false,
      alias: None,
      font: None,
      decl_id: None,
      replace: None,
      protected: false,
      robust: false,

      // math-specific
      name: None,
      meaning: None,
      omcd: None,
      reversion: None,
      sizer: None,
      role: None,
      operator_role: None,
      reorder: false,
      dual: false,
      mathstyle: None,
      scriptpos: None,
      operator_scriptpos: None,
      stretchy: None,
      operator_stretchy: None,
      nogroup: true,
      hide_content_reversion: false,
      revert_as: None,
      lpadding: None,
      rpadding: None,
    }
  }
}
impl PartialEq for MathPrimitiveOptions {
  fn eq(&self, other: &MathPrimitiveOptions) -> bool { self.name == other.name && self.meaning == other.meaning && self.role == other.role }
}

impl MathPrimitiveOptions {
  pub fn to_hash_stored(&self) -> HashMap<String, Stored> {
    let mut h = HashMap::new();
    if let Some(ref meaning) = self.meaning {
      h.insert("meaning".to_string(), meaning.into());
    }
    if let Some(ref name) = self.name {
      h.insert("name".to_string(), name.into());
    }
    if let Some(ref omcd) = self.omcd {
      h.insert("omcd".to_string(), omcd.into());
    }
    if let Some(ref role) = self.role {
      h.insert("role".to_string(), role.into());
    }
    if let Some(ref decl_id) = self.decl_id {
      h.insert("decl_id".to_string(), decl_id.into());
    }
    if let Some(ref operator_role) = self.operator_role {
      h.insert("operator_role".to_string(), operator_role.into());
    }
    if let Some(ref mathstyle) = self.mathstyle {
      h.insert("mathstyle".to_string(), mathstyle.into());
    }
    if let Some(ref scriptpos) = self.scriptpos {
      h.insert("scriptpos".to_string(), Stored::Int(*scriptpos as i64));
    }
    if let Some(ref operator_scriptpos) = self.operator_scriptpos {
      h.insert("operator_scriptpos".to_string(), Stored::Int(*operator_scriptpos as i64));
    }
    if let Some(ref stretchy) = self.stretchy {
      h.insert("stretchy".to_string(), (*stretchy).into());
    }
    if let Some(ref stretchy) = self.operator_stretchy {
      h.insert("operator_stretchy".to_string(), (*stretchy).into());
    }
    if let Some(ref mode) = self.mode {
      h.insert("mode".to_string(), mode.into());
    }
    // TODO: Do we want to run the font closures here? Maybe?
    if let Some(ref font_directive) = self.font {
      h.insert("font".to_string(), Stored::FontDirective(font_directive.clone()));
    }
    if let Some(ref lpadding) = self.lpadding {
      h.insert("lpadding".to_string(), (*lpadding).into());
    }
    if let Some(ref rpadding) = self.rpadding {
      h.insert("rpadding".to_string(), (*rpadding).into());
    }

    h
  }

  // Attempt at emulating the `%simpletoken_options` check in Perl
  /// Checks if complex options are present,
  /// suggestive of using a `Constructor` instead of a `Primitive`
  pub fn has_complex_option(&self) -> bool {
    //DG: note that `nogroup` is true by default, so checking for it is counter-intuitive (should we even?)
    self.bounded
      || self.mode.is_some()
      || !self.before_digest.is_empty()
      || !self.after_digest.is_empty()
      || self.is_prefix
      || self.require_math
      || self.forbid_math
      || self.alias.is_some()
      || self.decl_id.is_some()
      || self.replace.is_some()
      || self.reversion.is_some()
      || self.sizer.is_some()
      || self.operator_role.is_some()
      || self.reorder
      || self.dual
      || self.operator_scriptpos.is_some()
      || self.stretchy.is_some()
      || self.operator_stretchy.is_some()
      || self.hide_content_reversion
      || self.revert_as.is_some()
  }
}

#[derive(Clone)]
pub struct MathPrimitive {
  pub cs: Token,
  pub paramlist: Option<Parameters>,
  pub nargs: Option<usize>,
  pub replacement: Option<PrimitiveClosure>,
  pub options: MathPrimitiveOptions,
  pub alias: Option<String>,
}
impl Default for MathPrimitive {
  fn default() -> Self {
    MathPrimitive {
      cs: T_CS!("MathPrimitive"),
      paramlist: None,
      nargs: None,
      replacement: None,
      options: MathPrimitiveOptions::default(),
      alias: None,
    }
  }
}
impl PartialEq for MathPrimitive {
  fn eq(&self, other: &MathPrimitive) -> bool { self.cs == other.cs }
}

impl fmt::Display for MathPrimitive {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    unimplemented!();
  }
}
impl Object for MathPrimitive {
  fn stringify(&self) -> String { <Self as Definition>::stringify_type(self, "MathPrimitive") }
  fn get_locator(&self) -> Option<Cow<Locator>> { unimplemented!() }
}
impl Definition for MathPrimitive {
  fn before_digest(&self) -> Option<&Vec<BeforeDigestClosure>> { Some(&self.options.before_digest) }
  fn after_digest(&self) -> Option<&Vec<DigestionClosure>> { Some(&self.options.after_digest) }
  fn invoke(&self, _gullet: &mut Gullet, _once_only: bool, _state: &mut State) -> Result<Tokens> { Ok(Tokens!()) }
  fn invoke_primitive(&self, stomach: &mut Stomach, _caller: Arc<dyn Definition>, state: &mut State) -> Result<Vec<Digested>> {
    // Info!("MathPrimitive", "invoke", stomach, state, "invoke for {:?}", self.cs);
    // my $profiled = $STATE->lookupValue('PROFILING') && ($LaTeXML::CURRENT_TOKEN || $$self{cs});
    // my $tracing = $STATE->lookupValue('TRACINGCOMMANDS');
    // LaTeXML::Core::Definition::startProfiling($profiled, 'digest') if $profiled;
    // print STDERR '{' . $self->tracingCSName . "}\n" if $tracing;
    let mut result: Vec<Digested> = self.execute_before_digest(stomach, state)?;
    let args = self.read_arguments(stomach.get_gullet_mut(), state)?;
    // print STDERR $self->tracingArgs(@args) . "\n" if $tracing && @args;
    let replacement_result = match self.replacement {
      None => Vec::new(),
      Some(ref closure) => closure(stomach, args, state)?,
    };
    result.extend(replacement_result);
    let mut w = Whatsit::default();
    let after_result = self.execute_after_digest(stomach, &mut w, state)?;
    result.extend(after_result);

    // LaTeXML::Core::Definition::stopProfiling($profiled, 'digest') if $profiled;
    Ok(result)
  }

  fn do_absorbtion(&self, _document: &mut Document, _whatsit: &Whatsit, _state: &mut State) -> Result<Vec<Node>> {
    fatal!(Definition, Unexpected, "do_absorbtion on MathPrimitive should never be called!");
  }

  fn get_cs(&self) -> Cow<Token> { Cow::Borrowed(&self.cs) }
  fn get_cs_name(&self) -> Cow<str> { Cow::Borrowed(self.cs.get_cs_name()) }
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
