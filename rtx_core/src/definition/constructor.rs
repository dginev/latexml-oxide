use std::rc::Rc;
use std::collections::HashMap;
use state::{ObjectStore, Scope, State};
use common::object::Object;
use common::error::*;
use common::font::Font;

use token::*;
use tokens::Tokens;
use Digested;
use gullet::Gullet;
use stomach::Stomach;
use whatsit::Whatsit;
use parameter::Parameters;
use definition::{BeforeDigestClosure, ConstructionClosure, Definition, DigestionClosure,
                 ReplacementClosure};
use document::Document;

#[derive(Clone)]
pub struct ConstructorOptions {
  pub nargs: Option<usize>,
  pub bounded: bool,
  pub mode: Option<String>,
  pub before_digest: Vec<BeforeDigestClosure>,
  pub after_digest: Vec<DigestionClosure>,
  pub before_construct: Vec<ConstructionClosure>,
  pub after_construct: Vec<ConstructionClosure>,

  // environment-specific
  pub require_math: bool,
  pub forbid_math: bool,
  pub properties: HashMap<String, ObjectStore>,
  pub capture_body: bool,
  pub font: Option<Font>,

  pub after_digest_begin: Vec<DigestionClosure>,
  pub before_digest_end: Vec<BeforeDigestClosure>,
  pub after_digest_body: Vec<DigestionClosure>,
  // reversion       : 1,
  // sizer           : 1,
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
      properties: HashMap::new(),
      capture_body: false,
      font: None,
      after_digest_begin: vec![],
      before_digest_end: vec![],
      after_digest_body: vec![],
      scope: None,
      locked: false,
      alias: None,
    }
  }
}

#[derive(Clone)]
pub struct Constructor {
  pub cs: Token,
  pub paramlist: Option<Parameters>,
  pub replacement: Option<ReplacementClosure>,
  pub options: ConstructorOptions,
}
impl Default for Constructor {
  fn default() -> Self {
    Constructor {
      cs: T_CS!(s!("Constructor")),
      paramlist: None,
      replacement: None,
      options: ConstructorOptions::default(),
    }
  }
}
impl PartialEq for Constructor {
  fn eq(&self, other: &Constructor) -> bool { self.cs == other.cs }
}

impl Object for Constructor {}
impl Definition for Constructor {
  fn before_digest(&self) -> Option<&Vec<BeforeDigestClosure>> { Some(&self.options.before_digest) }
  fn after_digest(&self) -> Option<&Vec<DigestionClosure>> { Some(&self.options.after_digest) }
  fn after_digest_body(&self) -> Option<&Vec<DigestionClosure>> {
    Some(&self.options.after_digest_body)
  }
  fn capture_body(&self) -> bool { self.options.capture_body }
  fn invoke(&self, _gullet: &mut Gullet, _state: &mut State) -> Result<Tokens> { Ok(Tokens!()) }
  /// Digest the constructor; This should occur in the Stomach to create a Whatsit.
  /// The whatsit which will be further processed to create the document.
  fn invoke_primitive(
    &self,
    stomach: &mut Stomach,
    caller: Rc<Definition>,
    state: &mut State,
  ) -> Result<Vec<Digested>>
  {
    debug!(target: "constructor", "invoke for {:?}", self.get_cs());
    // Call any `Before' code.
    // TODO: profiling / tracing
    // let profiled = state.lookup_value("PROFILING") && ($LaTeXML::CURRENT_TOKEN || $$self{cs});
    // let tracing = state.lookup_value("TRACINGCOMMANDS");
    // LaTeXML::Definition::startProfiling($profiled, "digest") if $profiled;

    let mut result = self.execute_before_digest(stomach, state)?;

    // info!("{" + $self->tracingCSName . "}\n" if $tracing;
    // Get some info before we process arguments...

    let ismath = state.lookup_bool("IN_MATH");

    // Parse AND digest the arguments to the Constructor
    let mut args: Vec<Option<Digested>> = match *self.get_parameters() {
      None => Vec::new(),
      Some(ref params) => params.read_arguments_and_digest(stomach, self, state)?,
    };
    // info!($self->tracingArgs(@args) . "\n" if $tracing && @args;
    let nargs = self.get_num_args();
    args.truncate(nargs);

    // Compute any extra Whatsit properties (many end up as element attributes)

    let mut props = self.options.properties.clone();
    // for (key, value) in props.iter() {
    //   if (ref $value eq 'CODE') {
    //     $props{$key} = &$value($stomach, @args); } }

    let this_font = match self.options.font {
      Some(ref f) => f.clone(),
      None => match state.lookup_font() {
        Some(f) => f,
        None => Font::text_default(), // should never happen?
      },
    };

    props.insert(s!("font"), ObjectStore::Font(Box::new(this_font)));
    // $props{locator} = $stomach->getGullet->getMouth->getLocator unless defined $props{locator};
    props
      .entry(s!("isMath"))
      .or_insert(ObjectStore::Bool(ismath));
    // $props{level}   = $stomach->getBoxingLevel;

    // Now create the Whatsit, itself.
    let mut whatsit = Whatsit {
      definition: caller,
      args: args,
      properties: props,
    };

    // Call any 'After' code.
    let mut post = self.execute_after_digest(stomach, &mut whatsit, state)?;

    if self.options.capture_body {
      post.extend(stomach.digest_next_body(false, state)?);
      // info!(" -- Captured body: {:?}", post);
      whatsit.set_body(post);
      post = vec![];
    }
    let post_post = self.execute_after_digest_body(stomach, &mut whatsit, state)?;
    // LaTeXML::Core::Definition::stopProfiling($profiled, 'digest') if $profiled;

    // Package the result boxes
    result.push(Digested::Whatsit(whatsit));
    result.extend(post);
    result.extend(post_post);
    Ok(result)
  }

  fn get_cs(&self) -> Token { self.cs.clone() }
  fn get_cs_name(&self) -> String { self.cs.get_cs_name() }
  fn get_locator(&self) -> String { unimplemented!() }
  fn get_parameters(&self) -> &Option<Parameters> { &self.paramlist }
  fn get_num_args(&self) -> usize {
    match self.options.nargs {
      Some(n) => n,
      None => match self.paramlist {
        Some(ref params) => params.get_num_args(),
        None => 0,
      },
    }
    // self.nargs = Some(nargs);
  }

  fn do_absorbtion(
    &self,
    document: &mut Document,
    whatsit: &Whatsit,
    state: &mut State,
  ) -> Result<()>
  {
    for pre_closure in &self.options.before_construct {
      pre_closure(document, whatsit, state);
    }

    match self.replacement {
      None => {},
      Some(ref main_closure) => main_closure(
        document,
        whatsit.get_args(),
        whatsit.get_properties(),
        state
      )?,
    };

    for post_closure in &self.options.after_construct {
      post_closure(document, whatsit, state);
    }
    Ok(())
  }
}
