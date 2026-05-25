use libxml::tree::Node;
use std::borrow::Cow;
use std::fmt;
use std::rc::Rc;

use crate::common::arena::SymHashMap;
use crate::common::error::*;
use crate::common::font::Font;
use crate::common::object::Object;
use crate::state::*;

use crate::definition::{
  BeforeDigestClosure, ConstructionClosure, Definition, DigestionClosure, FontDirective,
  PropertiesClosure, ReplacementClosure, Reversion, SizingClosure,
};
use crate::common::locator::Locator;
use crate::document::Document;
use crate::parameter::Parameters;
use crate::stomach::digest_next_body;
use crate::token::*;
use crate::tokens::Tokens;
use crate::whatsit::Whatsit;
use crate::{BoxOps, Digested};

/// A `--source-map` construct's source extent: the union (first `from` → last
/// `to`) of its children's spans, or `None` if none carries a position (the
/// caller then falls back to the gullet locator). docs/SOURCE_PROVENANCE.md §3.1.
fn assemble_locator(args: &[Option<Digested>]) -> Option<Locator> {
  args
    .iter()
    .flatten()
    .filter_map(child_span)
    .reduce(|a, b| Locator::new_range(a, b).unwrap_or(a))
}

/// A child's located span: its own `get_locator()` if set, else — under
/// `token-locators` — recovered from the per-token origin handles still riding
/// its reverted tokens. (Origins survive revert/re-digest; `get_locator` merely
/// fails to aggregate undigested/composite content — §3.1.3.) Off the feature,
/// only `get_locator` is consulted (byte-identical behavior).
///
/// `pub` so out-of-band construction paths that open an element *around*
/// already-digested content — e.g. `insert_frontmatter` building `<ltx:title>`
/// from the stored, deferred `\title{…}` boxes — can recover the same span and
/// feed it to `Document::set_current_box_locator`.
pub fn child_span(d: &Digested) -> Option<Locator> {
  if let Some(l) = d.get_locator().filter(|l| l.from_line != 0) {
    return Some(l);
  }
  #[cfg(feature = "token-locators")]
  return d
    .revert()
    .ok()?
    .unlist_ref()
    .iter()
    .filter_map(|t| crate::token::get_token_origin(t.loc))
    .map(|o| crate::common::arena::with(o.source, |s| Locator::new(s, o.line, o.col, o.line, o.col)))
    .reduce(|a, b| Locator::new_range(a, b).unwrap_or(a));
  #[cfg(not(feature = "token-locators"))]
  None
}

/// configuration for creating a new Constructor
#[derive(Clone)]
pub struct ConstructorOptions {
  /// number of arguments (if any)
  pub nargs:            Option<usize>,
  /// bouded mode (default: false)
  pub bounded:          bool,
  /// begin a named mode
  pub mode:             Option<String>,
  /// a `SizingClosure` to estimate the size of the digested box
  pub sizer:            Option<SizingClosure>,
  /// custom code to run immediately before the digestion phase
  pub before_digest:    Vec<BeforeDigestClosure>,
  /// custom code to run immediately after the digestion phase
  pub after_digest:     Vec<DigestionClosure>,
  /// custom code to run immediately before the construction phase
  pub before_construct: Vec<ConstructionClosure>,
  /// custom code to run immediately after the construction phase
  pub after_construct:  Vec<ConstructionClosure>,

  /// switch to horizontal mode before digesting (Perl: enterHorizontal => 1)
  pub enter_horizontal: bool,
  /// switch to vertical mode before digesting (Perl: leaveHorizontal => 1)
  pub leave_horizontal: bool,

  // environment-specific
  /// requires to be used in math mode
  pub require_math:       bool,
  /// forbids use in math mode
  pub forbid_math:        bool,
  /// custom directives for computing box properties
  pub properties:         PropertiesClosure,
  /// should it capture the body as `#body` (default: false)
  pub capture_body:       bool,
  /// specify a font to use, or instructions how to compute which font to use
  pub font:               Option<FontDirective>,
  /// custom code to run as digestion begins
  pub after_digest_begin: Vec<DigestionClosure>,
  /// custom code to run just before digestion ends
  pub before_digest_end:  Vec<BeforeDigestClosure>,
  /// custom code to run after `#body` has been digested
  pub after_digest_body:  Vec<DigestionClosure>,
  /// provide tokens to revert to, or custom code for computing them
  pub reversion:          Option<Reversion>,
  /// Local/Global scope of installing this definition (default: Local)
  pub scope:              Option<Scope>,
  /// is this a robust command sequence (default: false)
  pub robust:             bool,
  /// lock the definition for raw TeX overrides (default: false)
  pub locked:             bool,
  /// alternative (command sequence) name, used for reversion
  pub alias:              Option<String>,
}
impl Default for ConstructorOptions {
  fn default() -> Self {
    ConstructorOptions {
      nargs:              None,
      bounded:            false,
      before_digest:      vec![],
      after_digest:       vec![],
      before_construct:   vec![],
      after_construct:    vec![],
      mode:               None,
      enter_horizontal:   false,
      leave_horizontal:   false,
      // environment-specific
      require_math:       false,
      forbid_math:        false,
      properties:         Rc::new(|_whatsit| Ok(SymHashMap::default())),
      capture_body:       false,
      font:               None,
      after_digest_begin: vec![],
      before_digest_end:  vec![],
      after_digest_body:  vec![],
      scope:              None,
      robust:             false,
      locked:             false,
      alias:              None,
      reversion:          None,
      sizer:              None,
    }
  }
}
impl fmt::Debug for ConstructorOptions {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(
      f,
      "\nConstructorOptions {{nargs:{:?}, bounded:{:?}, mode:{:?}, \n\tbefore_digest:{:?}, \
       after_digest_begin:{:?}, before_digest_end:{:?},\n\tafter_digest:{:?}, \
       after_digest_body:{:?}, before_construct:{:?}, after_construct:{:?},\n\trequire_math:{:?}, \
       forbid_math:{:?}, capture_body:{:?}, scope:{:?},\n\tlocked:{:?}, alias:{:?} }}\n",
      self.nargs,
      self.bounded,
      self.mode,
      self.before_digest.len(),
      self.after_digest_begin.len(),
      self.before_digest_end.len(),
      self.after_digest.len(),
      self.after_digest_body.len(),
      self.before_construct.len(),
      self.after_construct.len(),
      self.require_math,
      self.forbid_math,
      self.capture_body,
      self.scope,
      self.locked,
      self.alias
    )
  }
}

#[derive(Clone)]
pub struct Constructor {
  pub cs:                Token,
  pub nargs:             Option<usize>,
  pub paramlist:         Option<Parameters>,
  pub replacement:       Option<ReplacementClosure>,
  pub sizer:             Option<SizingClosure>,
  pub before_digest:     Vec<BeforeDigestClosure>,
  pub after_digest:      Vec<DigestionClosure>,
  pub before_construct:  Vec<ConstructionClosure>,
  pub after_construct:   Vec<ConstructionClosure>,
  pub properties:        PropertiesClosure,
  pub capture_body:      bool,
  // environment-specific
  pub after_digest_body: Vec<DigestionClosure>,
  pub reversion:         Option<Reversion>,
  pub alias:             Option<String>,
}
impl Default for Constructor {
  fn default() -> Self {
    Constructor {
      cs:                T_CS!("Constructor"),
      nargs:             None,
      paramlist:         None,
      replacement:       None,
      before_digest:     vec![],
      after_digest:      vec![],
      before_construct:  vec![],
      after_construct:   vec![],
      properties:        Rc::new(|_whatsit| Ok(SymHashMap::default())),
      capture_body:      false,
      after_digest_body: vec![],
      reversion:         None,
      alias:             None,
      sizer:             None,
    }
  }
}
impl fmt::Debug for Constructor {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(
      f,
      "\nConstructor {{
        cs:{:?}
        nargs:{:?}
        paramlist:{:?}
        replacement:{:?}
        before_digest:{:?}
        after_digest:{:?}
        before_construct:{:?}
        after_construct:{:?}
        capture_body:{:?}
        after_digest_body:{:?}
        reversion:{:?}
        alias:{:?}
        sizer:{:?} }}\n",
      self.cs,
      self.nargs,
      self.paramlist,
      self.replacement.is_some(),
      self.before_digest.len(),
      self.after_digest.len(),
      self.before_construct.len(),
      self.after_construct.len(),
      self.capture_body,
      self.after_digest_body.len(),
      self.reversion.is_some(),
      self.alias,
      self.sizer.is_some(),
    )
  }
}

impl PartialEq for Constructor {
  fn eq(&self, other: &Constructor) -> bool { self.cs == other.cs }
}

impl fmt::Display for Constructor {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(
      f,
      "{}",
      <Self as Definition>::stringify_type(self, "Constructor")
    )
  }
}
impl Object for Constructor {
  fn stringify(&self) -> String { <Self as Definition>::stringify_type(self, "Constructor") }
}
impl Definition for Constructor {
  fn before_digest(&self) -> Option<&Vec<BeforeDigestClosure>> { Some(&self.before_digest) }
  fn after_digest(&self) -> Option<&Vec<DigestionClosure>> { Some(&self.after_digest) }
  fn after_digest_body(&self) -> Option<&Vec<DigestionClosure>> { Some(&self.after_digest_body) }
  fn capture_body(&self) -> bool { self.capture_body }
  fn get_sizer(&self) -> Option<SizingClosure> { self.sizer.clone() }
  fn invoke(&self, _once_only: bool) -> Result<Tokens> { Ok(Tokens!()) }
  /// Digest the constructor; This should occur in the Stomach to create a Whatsit.
  /// The whatsit which will be further processed to create the document.
  fn invoke_primitive(&self) -> Result<Vec<Digested>> {
    Debug!("invoke_primitive for {:?}", self.get_cs());
    // Call any `Before' code.
    // TODO: profiling / tracing
    // let profiled = state!().lookup_value("PROFILING") && ($LaTeXML::CURRENT_TOKEN || $$self{cs});
    // let tracing = state!().lookup_value("tracingcommands");
    // LaTeXML::Definition::startProfiling($profiled, "digest") if $profiled;

    let mut result = self.execute_before_digest()?;

    // info!("{" + $self->tracingCSName . "}\n" if $tracing;
    // Get some info before we process arguments...
    let state_font = lookup_font();
    let ismath = crate::state::lookup_bool_sym(crate::pin!("IN_MATH"));
    // info!(target: "constructor", "invoke for {:?} ({:?})", self.get_cs(), ismath);
    // Parse AND digest the arguments to the Constructor
    let mut args: Vec<Option<Digested>> = match self.get_parameters() {
      None => Vec::new(),
      Some(params) => params.read_arguments_and_digest(self)?,
    };
    // info!($self->tracingArgs(@args) . "\n" if $tracing && @args;
    let nargs = self.get_num_args();
    args.truncate(nargs);

    // Compute any extra Whatsit properties (many end up as element attributes)

    let mut properties = (self.properties)(&args)?;
    // for (key, value) in properties.iter() {
    //   if (ref $value eq 'CODE') {
    //     $properties{$key} = &$value($stomach, @args); } }

    properties
      .entry("font")
      .or_insert_with(|| match state_font {
        Some(f) => Stored::Font(Rc::clone(&f)),
        None => Stored::Font(Rc::new(Font::text_default())), // should never happen?
      });
    // $properties{locator} = $stomach->getGullet->getMouth->getLocator unless defined
    // $properties{locator};
    properties
      .entry("isMath")
      .or_insert_with(|| Stored::Bool(ismath));
    // Perl: $mode = $properties{mode} || $state->lookupValue('MODE') || 'restricted_horizontal';
    // Set mode on whatsit so repackHorizontal can distinguish vertical vs horizontal items.
    properties.entry("mode").or_insert_with(|| {
      let mode = crate::state::lookup_string_from_sym(crate::pin!("MODE"));
      Stored::String(crate::common::arena::pin(if mode.is_empty() {
        "restricted_horizontal"
      } else {
        &mode
      }))
    });
    // $properties{level}   = $stomach->getBoxingLevel;

    // Now create the Whatsit, itself.
    let mut whatsit = Whatsit {
      definition: Rc::new(self.clone()),
      args,
      properties,
      ..Whatsit::default()
    };
    // Perl `Core/Definition/Constructor.pm` L106:
    //   `$props{locator} = $stomach->getGullet->getLocator`
    // — capture the construct's source position at digest time. Gated on
    // `--source-map`: the whatsit locator is consumed only by source-map
    // stamping + (untested) error messages, so the corpus/parity path skips
    // the per-construct `get_locator`/`arena::pin` cost and stays
    // byte-identical (the switch gates *all* locator tracking). Without this,
    // constructor-built elements carry `Locator::default()` (source =
    // `locator.rs`) and the source-map user-source filter drops them
    // (~53/265 → 128/… `article.tex` elements stamped once captured).
    if crate::state::source_map_enabled() {
      // --source-map: the construct's source extent is the union of its
      // children's spans (fixes the post-expansion eating-disorder, Experiment 2),
      // falling back to the gullet locator when no child carries a position.
      whatsit.locator =
        assemble_locator(&whatsit.args).or_else(|| Some(crate::gullet::get_locator()));
    }

    // Call any 'After' code.
    let mut post = self.execute_after_digest(&mut whatsit)?;

    if self.capture_body {
      let captured = digest_next_body(None)?;
      // info!(target:"constructor:digest_next_body", "\n{:?}\n----\n",captured);
      post.extend(captured);

      // token-locators: capture_body constructs (e.g. `\lx@begin@inline@math`)
      // carry their content as #body, not positional args, so the earlier
      // assemble_locator (over args) missed it and fell back to the gullet point.
      // Derive the span from the digested body and union it with any positional-
      // arg span, so the wrapper (e.g. `ltx:Math`) spans its content. §3.1.3.
      #[cfg(feature = "token-locators")]
      if crate::state::source_map_enabled() {
        if let Some(body_span) =
          post.iter().filter_map(child_span).reduce(|a, b| Locator::new_range(a, b).unwrap_or(a))
        {
          whatsit.locator = Some(match whatsit.locator {
            Some(prev) if prev.from_line != 0 => {
              Locator::new_range(prev, body_span).unwrap_or(body_span)
            },
            _ => body_span,
          });
        }
      }
      whatsit.set_body(post);
      post = vec![];
      //info!(target: "constructor:capture", "whatsit: {:?}", whatsit);
      // info!(target: "constructor:capture", "constructor: {:?}", self.get_cs_name());
    }
    let post_post = self.execute_after_digest_body(&mut whatsit)?;
    // LaTeXML::Core::Definition::stopProfiling($profiled, 'digest') if $profiled;

    // Package the result boxes
    result.push(whatsit.into());
    result.extend(post);
    result.extend(post_post);
    Ok(result)
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
    // self.nargs = Some(nargs);
  }

  fn do_absorption(&self, document: &mut Document, whatsit: &Whatsit) -> Result<Vec<Node>> {
    for pre_closure in &self.before_construct {
      pre_closure(document, whatsit)?;
    }

    match self.replacement {
      None => {
        // info!(target:"constructor:replacement", "no replacement for {:?}", self.get_cs_name());
      },
      Some(ref main_closure) => {
        main_closure(document, whatsit.get_args(), whatsit.get_properties())?
      },
    };

    for post_closure in &self.after_construct {
      post_closure(document, whatsit)?;
    }
    Ok(Vec::new())
  }
  fn get_reversion_spec(&self) -> Option<Reversion> { self.reversion.clone() }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn constructor_options_default_all_false_none_empty() {
    let o = ConstructorOptions::default();
    assert!(o.nargs.is_none());
    assert!(!o.bounded);
    assert!(o.mode.is_none());
    assert!(!o.enter_horizontal);
    assert!(!o.leave_horizontal);
    assert!(!o.require_math);
    assert!(!o.forbid_math);
    assert!(!o.capture_body);
    assert!(o.font.is_none());
    assert!(o.scope.is_none());
    assert!(!o.robust);
    assert!(!o.locked);
    assert!(o.alias.is_none());
    assert!(o.reversion.is_none());
    assert!(o.sizer.is_none());
    assert!(o.before_digest.is_empty());
    assert!(o.after_digest.is_empty());
    assert!(o.before_construct.is_empty());
    assert!(o.after_construct.is_empty());
    assert!(o.after_digest_begin.is_empty());
    assert!(o.before_digest_end.is_empty());
    assert!(o.after_digest_body.is_empty());
  }

  #[test]
  fn constructor_default_fields() {
    let c = Constructor::default();
    assert!(c.nargs.is_none());
    assert!(c.paramlist.is_none());
    assert!(c.replacement.is_none());
    assert!(c.sizer.is_none());
    assert!(!c.capture_body);
    assert!(c.alias.is_none());
    assert!(c.reversion.is_none());
    assert!(c.before_digest.is_empty());
    assert!(c.after_digest.is_empty());
    assert!(c.before_construct.is_empty());
    assert!(c.after_construct.is_empty());
    assert!(c.after_digest_body.is_empty());
  }

  #[test]
  fn constructor_options_debug_includes_fields() {
    // The hand-written Debug impl formats the struct; just verify it
    // doesn't panic and produces a non-empty string that contains
    // key field names.
    let o = ConstructorOptions::default();
    let s = format!("{o:?}");
    assert!(s.contains("nargs"), "got {s:?}");
    assert!(s.contains("bounded"), "got {s:?}");
    assert!(s.contains("capture_body"), "got {s:?}");
  }
}
