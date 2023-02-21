use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use crate::common::error::*;
use crate::common::font::Font;
use crate::common::object::Object;
use crate::common::store::Stored;
use crate::state::{Scope, State};

use crate::definition::{
  BeforeDigestClosure, ConstructionClosure, Definition, DigestionClosure, PropertiesClosure, ReplacementClosure, Reversion, SizingClosure, FontDirective
};
use crate::document::Document;
use crate::gullet::Gullet;
use crate::parameter::Parameters;
use crate::stomach::Stomach;
use crate::token::*;
use crate::tokens::Tokens;
use crate::whatsit::Whatsit;
use crate::{BoxOps, Digested, Locator};

#[derive(Clone)]
pub struct ConstructorOptions {
  pub nargs: Option<usize>,
  pub bounded: bool,
  pub mode: Option<String>,
  pub sizer: Option<SizingClosure>,
  pub before_digest: Vec<BeforeDigestClosure>,
  pub after_digest: Vec<DigestionClosure>,
  pub before_construct: Vec<ConstructionClosure>,
  pub after_construct: Vec<ConstructionClosure>,

  // environment-specific
  pub require_math: bool,
  pub forbid_math: bool,
  pub properties: PropertiesClosure,
  pub capture_body: bool,
  pub font: Option<FontDirective>,

  pub after_digest_begin: Vec<DigestionClosure>,
  pub before_digest_end: Vec<BeforeDigestClosure>,
  pub after_digest_body: Vec<DigestionClosure>,
  pub reversion: Option<Reversion>,
  pub scope: Option<Scope>,
  pub locked: bool,
  pub alias: Option<String>,
}
impl Default for ConstructorOptions {
  fn default() -> Self {
    ConstructorOptions {
      nargs: None,
      bounded: false,
      before_digest: vec![],
      after_digest: vec![],
      before_construct: vec![],
      after_construct: vec![],
      mode: None,
      // environment-specific
      require_math: false,
      forbid_math: false,
      properties: Arc::new(|stomach, whatsit, state| Ok(HashMap::new())),
      capture_body: false,
      font: None,
      after_digest_begin: vec![],
      before_digest_end: vec![],
      after_digest_body: vec![],
      scope: None,
      locked: false,
      alias: None,
      reversion: None,
      sizer: None,
    }
  }
}
impl fmt::Debug for ConstructorOptions {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(
      f,
      "\nConstructorOptions {{nargs:{:?}, bounded:{:?}, mode:{:?}, \n\
       \tbefore_digest:{:?}, after_digest_begin:{:?}, before_digest_end:{:?},\n\
       \tafter_digest:{:?}, after_digest_body:{:?}, before_construct:{:?}, after_construct:{:?},\n\
       \trequire_math:{:?}, forbid_math:{:?}, capture_body:{:?}, scope:{:?},\n\
       \tlocked:{:?}, alias:{:?} }}\n",
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
  pub cs: Token,
  pub nargs: Option<usize>,
  pub paramlist: Option<Parameters>,
  pub replacement: Option<ReplacementClosure>,
  pub sizer: Option<SizingClosure>,
  pub before_digest: Vec<BeforeDigestClosure>,
  pub after_digest: Vec<DigestionClosure>,
  pub before_construct: Vec<ConstructionClosure>,
  pub after_construct: Vec<ConstructionClosure>,
  pub properties: PropertiesClosure,
  pub capture_body: bool,
  // environment-specific
  pub after_digest_body: Vec<DigestionClosure>,
  pub reversion: Option<Reversion>,
  pub alias: Option<String>,
}
impl Default for Constructor {
  fn default() -> Self {
    Constructor {
      cs: T_CS!("Constructor"),
      nargs: None,
      paramlist: None,
      replacement: None,
      before_digest: vec![],
      after_digest: vec![],
      before_construct: vec![],
      after_construct: vec![],
      properties: Arc::new(|stomach, whatsit, state| Ok(HashMap::new())),
      capture_body: false,
      after_digest_body: vec![],
      reversion: None,
      alias: None,
      sizer: None,
    }
  }
}
impl PartialEq for Constructor {
  fn eq(&self, other: &Constructor) -> bool { self.cs == other.cs }
}

impl fmt::Display for Constructor {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    unimplemented!();
  }
}
impl Object for Constructor {
  fn stringify(&self) -> String { <Self as Definition>::stringify_type(self, "Constructor") }
  fn get_locator(&self) -> Option<Cow<Locator>> { unimplemented!() }
}
impl Definition for Constructor {
  fn before_digest(&self) -> Option<&Vec<BeforeDigestClosure>> { Some(&self.before_digest) }
  fn after_digest(&self) -> Option<&Vec<DigestionClosure>> { Some(&self.after_digest) }
  fn after_digest_body(&self) -> Option<&Vec<DigestionClosure>> { Some(&self.after_digest_body) }
  fn capture_body(&self) -> bool { self.capture_body }
  fn get_sizer(&self) -> Option<SizingClosure> { self.sizer.clone() }
  fn invoke(&self, _gullet: &mut Gullet, _once_only: bool, _state: &mut State) -> Result<Tokens> { Ok(Tokens!()) }
  /// Digest the constructor; This should occur in the Stomach to create a Whatsit.
  /// The whatsit which will be further processed to create the document.
  fn invoke_primitive(&self, stomach: &mut Stomach, caller: Arc<dyn Definition>, state: &mut State) -> Result<Vec<Digested>> {
    Debug!("invoke for {:?}", self.get_cs());
    // Call any `Before' code.
    // TODO: profiling / tracing
    // let profiled = state.lookup_value("PROFILING") && ($LaTeXML::CURRENT_TOKEN || $$self{cs});
    // let tracing = state.lookup_value("TRACINGCOMMANDS");
    // LaTeXML::Definition::startProfiling($profiled, "digest") if $profiled;

    let mut result = self.execute_before_digest(stomach, state)?;

    // info!("{" + $self->tracingCSName . "}\n" if $tracing;
    // Get some info before we process arguments...
    let state_font = state.lookup_font();
    let ismath = state.lookup_bool("IN_MATH");
    // info!(target: "constructor", "invoke for {:?} ({:?})", self.get_cs(), ismath);
    // Parse AND digest the arguments to the Constructor
    let mut args: Vec<Option<Digested>> = match self.get_parameters() {
      None => Vec::new(),
      Some(params) => params.read_arguments_and_digest(stomach, self, state)?,
    };
    // info!($self->tracingArgs(@args) . "\n" if $tracing && @args;
    let nargs = self.get_num_args();
    args.truncate(nargs);

    // Compute any extra Whatsit properties (many end up as element attributes)

    let mut properties = (self.properties)(stomach, &args, state)?;
    // for (key, value) in properties.iter() {
    //   if (ref $value eq 'CODE') {
    //     $properties{$key} = &$value($stomach, @args); } }

    properties.entry(s!("font")).or_insert_with(|| match state_font {
      Some(f) => Stored::Font(Arc::clone(&f)),
      None => Stored::Font(Arc::new(Font::text_default())), // should never happen?
    });
    // $properties{locator} = $stomach->getGullet->getMouth->getLocator unless defined
    // $properties{locator};
    properties.entry(s!("isMath")).or_insert_with(|| Stored::Bool(ismath));
    // $properties{level}   = $stomach->getBoxingLevel;

    // Now create the Whatsit, itself.
    let mut whatsit = Whatsit {
      definition: caller,
      args,
      properties,
      ..Whatsit::default()
    };

    // Call any 'After' code.
    let mut post = self.execute_after_digest(stomach, &mut whatsit, state)?;

    if self.capture_body {
      let captured = stomach.digest_next_body(None, state)?;
      // info!(target:"constructor:digest_next_body", "\n{:?}\n----\n",captured);
      post.extend(captured);

      whatsit.set_body(post, state);
      post = vec![];
      //info!(target: "constructor:capture", "whatsit: {:?}", whatsit);
      // info!(target: "constructor:capture", "constructor: {:?}", self.get_cs_name());
    }
    let post_post = self.execute_after_digest_body(stomach, &mut whatsit, state)?;
    // LaTeXML::Core::Definition::stopProfiling($profiled, 'digest') if $profiled;

    // Package the result boxes
    result.push(whatsit.into());
    result.extend(post);
    result.extend(post_post);
    Ok(result)
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
    // self.nargs = Some(nargs);
  }

  fn do_absorbtion(&self, document: &mut Document, whatsit: &Whatsit, state: &mut State) -> Result<()> {
    for pre_closure in &self.before_construct {
      pre_closure(document, whatsit, state)?;
    }

    match self.replacement {
      None => {
        // info!(target:"constructor:replacement", "no replacement for {:?}", self.get_cs_name());
      },
      Some(ref main_closure) => {
        // info!(target:"constructor:replacement", "invoked for {:?}", self.get_cs_name());
        main_closure(document, whatsit.get_args(), whatsit.get_properties(), state)?
      },
    };

    for post_closure in &self.after_construct {
      post_closure(document, whatsit, state)?;
    }
    Ok(())
  }
  fn get_reversion_spec(&self) -> Option<Reversion> { self.reversion.clone() }
}
