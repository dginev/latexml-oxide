use libxml::tree::Node;
use std::borrow::Cow;

use crate::common::error::*;
// use crate::common::font::Font;
use crate::common::arena::SymHashMap as HashMap;
use crate::common::object::Object;
use crate::common::store::Stored;
use crate::state::Scope;

use crate::Digested;
use crate::definition::{
  BeforeDigestClosure, ConstructionClosure, Definition, DigestionClosure, FontDirective,
  PrimitiveClosure, Reversion,
};
use crate::document::Document;
use crate::parameter::Parameters;
use crate::token::*;
use crate::tokens::Tokens;
use crate::whatsit::Whatsit;

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
  pub bounded:          bool,
  pub mode:             Option<String>,
  pub before_digest:    Vec<BeforeDigestClosure>,
  pub after_digest:     Vec<DigestionClosure>,
  pub before_construct: Vec<ConstructionClosure>,
  pub after_construct:  Vec<ConstructionClosure>,
  pub is_prefix:        bool,
  pub scope:            Option<Scope>,
  pub font:             Option<FontDirective>,
  pub require_math:     bool,
  pub forbid_math:      bool,
  pub locked:           bool,
  pub alias:            Option<String>,
  pub decl_id:          Option<String>,
  pub replace:          Option<String>,
  pub protected:        bool,
  pub robust:           bool,

  // Math specific
  pub name:                   Option<String>,
  pub meaning:                Option<String>,
  pub omcd:                   Option<String>,
  pub reversion:              Option<Reversion>,
  pub sizer:                  Option<SizingClosure>,
  pub role:                   Option<String>,
  pub operator_role:          Option<String>,
  pub reorder:                bool,
  pub dual:                   bool,
  pub mathstyle:              Option<String>,
  /// Dynamic mathstyle: compute "display"/"text" based on current font mathstyle at invocation
  /// time Perl: mathstyle => \&doVariablesizeOp
  pub dynamic_mathstyle:      bool,
  pub scriptpos:              Option<String>,
  /// Dynamic scriptpos: compute "mid"/"post" based on current font mathstyle at invocation time
  /// Perl: scriptpos => \&doScriptpos
  pub dynamic_scriptpos:      bool,
  pub operator_scriptpos:     Option<usize>,
  pub stretchy:               Option<bool>,
  pub operator_stretchy:      Option<bool>,
  pub nogroup:                bool,
  pub hide_content_reversion: bool,
  pub revert_as:              Option<Cow<'static, str>>,
  pub lpadding:               Option<usize>,
  pub rpadding:               Option<usize>,
}
impl Default for MathPrimitiveOptions {
  fn default() -> Self {
    MathPrimitiveOptions {
      bounded:          false,
      before_digest:    Vec::new(),
      after_digest:     Vec::new(),
      before_construct: Vec::new(),
      after_construct:  Vec::new(),
      mode:             None,
      is_prefix:        false,
      scope:            None,
      require_math:     false,
      forbid_math:      false,
      locked:           false,
      alias:            None,
      font:             None,
      decl_id:          None,
      replace:          None,
      protected:        false,
      robust:           false,

      // math-specific
      name:                   None,
      meaning:                None,
      omcd:                   None,
      reversion:              None,
      sizer:                  None,
      role:                   None,
      operator_role:          None,
      reorder:                false,
      dual:                   false,
      mathstyle:              None,
      dynamic_mathstyle:      false,
      scriptpos:              None,
      dynamic_scriptpos:      false,
      operator_scriptpos:     None,
      stretchy:               None,
      operator_stretchy:      None,
      nogroup:                true,
      hide_content_reversion: false,
      revert_as:              None,
      lpadding:               None,
      rpadding:               None,
    }
  }
}
impl PartialEq for MathPrimitiveOptions {
  fn eq(&self, other: &MathPrimitiveOptions) -> bool {
    self.name == other.name && self.meaning == other.meaning && self.role == other.role
  }
}

impl MathPrimitiveOptions {
  pub fn to_hash_stored(&self) -> HashMap<Stored> {
    let mut h = HashMap::default();
    if let Some(ref meaning) = self.meaning {
      h.insert("meaning", meaning.into());
    }
    if let Some(ref name) = self.name {
      h.insert("name", name.into());
    }
    if let Some(ref omcd) = self.omcd {
      h.insert("omcd", omcd.into());
    }
    if let Some(ref role) = self.role {
      h.insert("role", role.into());
    }
    if let Some(ref decl_id) = self.decl_id {
      h.insert("decl_id", decl_id.into());
    }
    if let Some(ref operator_role) = self.operator_role {
      h.insert("operator_role", operator_role.into());
    }
    if let Some(ref mathstyle) = self.mathstyle {
      h.insert("mathstyle", mathstyle.into());
    }
    if let Some(ref scriptpos) = self.scriptpos {
      h.insert("scriptpos", scriptpos.into());
    }
    if let Some(ref operator_scriptpos) = self.operator_scriptpos {
      h.insert(
        "operator_scriptpos",
        Stored::Int(*operator_scriptpos as i64),
      );
    }
    if let Some(ref stretchy) = self.stretchy {
      h.insert("stretchy", (*stretchy).into());
    }
    if let Some(ref stretchy) = self.operator_stretchy {
      h.insert("operator_stretchy", (*stretchy).into());
    }
    if let Some(ref mode) = self.mode {
      h.insert("mode", mode.into());
    }
    // TODO: Do we want to run the font closures here? Maybe?
    if let Some(ref font_directive) = self.font {
      h.insert("font", Stored::FontDirective(font_directive.clone()));
    }
    if let Some(ref lpadding) = self.lpadding {
      h.insert("lpadding", (*lpadding).into());
    }
    if let Some(ref rpadding) = self.rpadding {
      h.insert("rpadding", (*rpadding).into());
    }

    h
  }

  /// Like `to_hash_stored` but applies per-invocation overrides without
  /// cloning the whole options. Used in DefMath closures (hot path —
  /// one call per math token invocation).
  pub fn to_hash_stored_with_overrides(
    &self,
    mode_override: Option<&'static str>,
    mathstyle_override: Option<&'static str>,
    scriptpos_override: Option<&'static str>,
  ) -> HashMap<Stored> {
    let mut h = self.to_hash_stored();
    if let Some(m) = mode_override {
      h.insert("mode", Stored::String(crate::common::arena::pin_static(m)));
    }
    if let Some(ms) = mathstyle_override {
      h.insert(
        "mathstyle",
        Stored::String(crate::common::arena::pin_static(ms)),
      );
    }
    if let Some(sp) = scriptpos_override {
      h.insert(
        "scriptpos",
        Stored::String(crate::common::arena::pin_static(sp)),
      );
    }
    h
  }

  // Attempt at emulating the `%simpletoken_options` check in Perl
  /// Checks if complex options are present,
  /// suggestive of using a `Constructor` instead of a `Primitive`
  pub fn has_complex_option(&self) -> bool {
    //DG: note that `nogroup` is true by default, so checking for it is counter-intuitive (should
    // we even?)
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
  pub cs:          Token,
  pub paramlist:   Option<Parameters>,
  pub nargs:       Option<usize>,
  pub replacement: Option<PrimitiveClosure>,
  pub options:     MathPrimitiveOptions,
  pub alias:       Option<String>,
}
impl Default for MathPrimitive {
  fn default() -> Self {
    MathPrimitive {
      cs:          T_CS!("MathPrimitive"),
      paramlist:   None,
      nargs:       None,
      replacement: None,
      options:     MathPrimitiveOptions::default(),
      alias:       None,
    }
  }
}
impl PartialEq for MathPrimitive {
  fn eq(&self, other: &MathPrimitive) -> bool { self.cs == other.cs }
}

// impl fmt::Display for MathPrimitive {
//   fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//     todo!();
//   }
// }
impl Object for MathPrimitive {
  fn stringify(&self) -> String { <Self as Definition>::stringify_type(self, "MathPrimitive") }
}
impl Definition for MathPrimitive {
  fn before_digest(&self) -> Option<&Vec<BeforeDigestClosure>> { Some(&self.options.before_digest) }
  fn after_digest(&self) -> Option<&Vec<DigestionClosure>> { Some(&self.options.after_digest) }
  fn invoke(&self, _once_only: bool) -> Result<Tokens> { Ok(Tokens!()) }
  fn invoke_primitive(&self) -> Result<Vec<Digested>> {
    // Info!("MathPrimitive", "invoke", stomach, "invoke for {:?}", self.cs);
    // my $profiled = $state->lookupValue('PROFILING') && ($LaTeXML::CURRENT_TOKEN || $$self{cs});
    // my $tracing = $state->lookupValue('tracingcommands');
    // LaTeXML::Core::Definition::startProfiling($profiled, 'digest') if $profiled;
    // print STDERR '{' . $self->tracingCSName . "}\n" if $tracing;
    let mut result: Vec<Digested> = self.execute_before_digest()?;
    let args = self.read_arguments()?;
    // print STDERR $self->tracingArgs(@args) . "\n" if $tracing && @args;
    let replacement_result = match self.replacement {
      None => Vec::new(),
      Some(ref closure) => closure(args)?,
    };
    result.extend(replacement_result);
    let mut w = Whatsit::default();
    let after_result = self.execute_after_digest(&mut w)?;
    result.extend(after_result);

    // LaTeXML::Core::Definition::stopProfiling($profiled, 'digest') if $profiled;
    Ok(result)
  }

  fn do_absorption(&self, _document: &mut Document, _whatsit: &Whatsit) -> Result<Vec<Node>> {
    fatal!(
      Definition,
      Unexpected,
      "do_absorption on MathPrimitive should never be called!"
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

  #[test]
  fn math_primitive_options_default_fields() {
    let o = MathPrimitiveOptions::default();
    // Spot-check representative fields from the ~30-field struct.
    assert!(!o.bounded);
    assert!(!o.is_prefix);
    assert!(!o.require_math);
    assert!(!o.forbid_math);
    assert!(!o.locked);
    assert!(!o.robust);
    assert!(!o.protected);
    assert!(!o.reorder);
    assert!(!o.dual);
    assert!(!o.dynamic_mathstyle);
    assert!(!o.dynamic_scriptpos);
    assert!(o.nogroup, "nogroup defaults to true (Perl parity)");
    assert!(!o.hide_content_reversion);
    assert!(o.name.is_none());
    assert!(o.meaning.is_none());
    assert!(o.role.is_none());
    assert!(o.operator_role.is_none());
    assert!(o.mathstyle.is_none());
    assert!(o.scriptpos.is_none());
    assert!(o.operator_scriptpos.is_none());
    assert!(o.stretchy.is_none());
    assert!(o.operator_stretchy.is_none());
    assert!(o.revert_as.is_none());
    assert!(o.lpadding.is_none());
    assert!(o.rpadding.is_none());
    assert!(o.before_digest.is_empty());
    assert!(o.after_digest.is_empty());
  }

  #[test]
  fn math_primitive_options_partial_eq_by_subset() {
    // PartialEq compares name + meaning + role only — defaults of
    // other fields don't affect equality.
    let mut a = MathPrimitiveOptions::default();
    let mut b = MathPrimitiveOptions::default();
    a.name = Some("plus".into());
    b.name = Some("plus".into());
    assert!(a == b, "same name equal");
    // Changing a non-compared field still keeps them equal.
    a.locked = true;
    assert!(a == b);
    // Changing a compared field breaks equality.
    b.name = Some("times".into());
    assert!(!(a == b));
  }

  #[test]
  fn math_primitive_options_to_hash_stored_empty_default() {
    // Default options with no string fields set produces an empty
    // HashMap.
    let o = MathPrimitiveOptions::default();
    let h = o.to_hash_stored();
    assert_eq!(h.len(), 0);
  }

  #[test]
  fn math_primitive_options_to_hash_stored_with_fields() {
    let o = MathPrimitiveOptions {
      meaning: Some("plus".into()),
      name: Some("+".into()),
      ..Default::default()
    };
    let h = o.to_hash_stored();
    assert!(h.contains_key("meaning"));
    assert!(h.contains_key("name"));
    assert!(
      !h.contains_key("role"),
      "role=None shouldn't populate the hash"
    );
  }

  #[test]
  fn math_primitive_default_fields() {
    let m = MathPrimitive::default();
    assert!(m.paramlist.is_none());
    assert!(m.replacement.is_none());
    assert!(m.alias.is_none());
    assert!(m.nargs.is_none());
    // options and flags live in m.options, not directly on the struct.
    assert!(m.options.reversion.is_none());
    assert!(!m.options.is_prefix);
  }

  #[test]
  fn math_primitive_partial_eq_by_cs() {
    let a = MathPrimitive::default();
    let b = MathPrimitive::default();
    // Both defaults have the same cs → equal.
    assert!(a == b);
  }
}
