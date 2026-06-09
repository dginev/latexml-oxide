//! `ConstructorBuilder` — the shared lowering for `DefConstructor`.
//!
//! Both binding front-ends target this one builder, so they cannot drift:
//! * the compile-time `DefConstructor!` macro (`latexml_engine`), and
//! * the runtime Rhai script layer (`latexml_contrib::script_bindings`).
//!
//! It is front-end-agnostic: it takes already-native values and closures, so it
//! lives in `latexml_core` and pulls in neither the macro machinery nor Rhai.
//!
//! Two kinds of options, by how much can be shared:
//! * **Scalar options** (`mode`, `bounded`, …) go through one generic
//!   [`ConstructorBuilder::set_option`] — the key→field mapping lives in exactly
//!   one place, so adding a scalar option updates both front-ends at once.
//! * **Closure options** (`afterDigest`, …) have typed setters: the field and
//!   `install` are shared, while the closure itself is produced by whichever
//!   front-end (a macro `$body:block`, or a Rhai trampoline).

use crate::common::def_parser::parse_prototype;
use crate::common::error::{Error, Result};
use crate::definition::constructor::ConstructorOptions;
use crate::definition::{
  BeforeDigestClosure, DigestionClosure, PropertiesClosure, ReplacementClosure,
};
use crate::parameter::Parameters;
use crate::token::Token;

use super::dialect::def_constructor;

/// A scalar option value handed to [`ConstructorBuilder::set_option`]. Both
/// front-ends produce these (the macro from a literal, Rhai from a `Dynamic`),
/// so the key→field switch is single-source.
pub enum OptionValue {
  Str(String),
  Bool(bool),
  Int(i64),
}

impl OptionValue {
  fn into_string(self) -> Result<String> {
    match self {
      OptionValue::Str(s) => Ok(s),
      _ => Err(Error::from("constructor option expected a string value")),
    }
  }

  fn into_bool(self) -> Result<bool> {
    match self {
      OptionValue::Bool(b) => Ok(b),
      OptionValue::Int(i) => Ok(i != 0),
      OptionValue::Str(s) => Ok(!s.is_empty()),
    }
  }
}

/// Accumulates a constructor definition and installs it via [`def_constructor`].
pub struct ConstructorBuilder {
  cs: Token,
  paramlist: Option<Parameters>,
  replacement: Option<ReplacementClosure>,
  options: ConstructorOptions,
}

impl ConstructorBuilder {
  /// Parse the prototype (shared with the macro path via `parse_prototype`).
  pub fn new(proto: &str) -> Result<Self> {
    let (cs, paramlist) = parse_prototype(proto, true)?;
    Ok(Self { cs, paramlist, replacement: None, options: ConstructorOptions::default() })
  }

  /// Set the XML replacement (template- or closure-derived; built by the caller).
  pub fn replacement(mut self, repl: ReplacementClosure) -> Self {
    self.replacement = Some(repl);
    self
  }

  /// Apply a **scalar** option by name. THE single source of truth for the
  /// option-name → `ConstructorOptions`-field mapping — both front-ends route
  /// scalar options through here, so a new scalar option is added in one place.
  /// Unknown keys are ignored (runtime-forgiving, matching Perl `%options`).
  pub fn set_option(mut self, key: &str, value: OptionValue) -> Result<Self> {
    match key {
      "mode" => self.options.mode = Some(value.into_string()?),
      "bounded" => self.options.bounded = value.into_bool()?,
      "requireMath" => self.options.require_math = value.into_bool()?,
      "forbidMath" => self.options.forbid_math = value.into_bool()?,
      "enterHorizontal" => self.options.enter_horizontal = value.into_bool()?,
      "leaveHorizontal" => self.options.leave_horizontal = value.into_bool()?,
      "captureBody" => self.options.capture_body = value.into_bool()?,
      "alias" => self.options.alias = Some(value.into_string()?),
      _ => log::debug!("ConstructorBuilder: ignoring unknown scalar option '{key}'"),
    }
    Ok(self)
  }

  /// Push an `afterDigest` hook. Typed setter: the field + `install` are shared;
  /// the closure is produced by the front-end (macro block or Rhai trampoline).
  /// The other closure options (`beforeDigest`, `properties`, `reversion`,
  /// `sizer`, `before/afterConstruct`) follow this identical shape.
  pub fn after_digest(mut self, hook: DigestionClosure) -> Self {
    self.options.after_digest.push(hook);
    self
  }

  /// Set the `properties` closure (computes the whatsit's property map from the
  /// digested args — Perl's `properties => sub {…}` / `properties => {…}`).
  /// Same typed-setter shape as [`Self::after_digest`]: the closure is produced
  /// by whichever front-end (a macro `sub [args]` block, or a Rhai trampoline).
  pub fn properties(mut self, props: PropertiesClosure) -> Self {
    self.options.properties = props;
    self
  }

  /// Push a `beforeDigest` hook (runs before the arguments are digested —
  /// Perl's `beforeDigest => sub {…}`, e.g. `\footnote`'s `neutralize_font`).
  pub fn before_digest(mut self, hook: BeforeDigestClosure) -> Self {
    self.options.before_digest.push(hook);
    self
  }

  /// Install the accumulated definition.
  pub fn install(self) -> Result<()> {
    def_constructor(self.cs, self.paramlist, self.replacement, self.options);
    Ok(())
  }
}
