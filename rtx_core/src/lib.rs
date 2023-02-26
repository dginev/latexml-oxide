#![allow(dead_code, unused_variables, unused_mut, unused_macros, clippy::trivial_regex)]

#[macro_use]
pub mod aux_macros;
#[macro_use]
pub mod token;
#[macro_use]
pub mod common;
#[macro_use]
pub mod tokens;
#[macro_use]
pub mod binding;
pub mod definition;
pub mod comment;
pub mod document;
pub mod gullet;
pub mod keyval;
pub mod keyvals;
pub mod ligature;
pub mod list;
pub mod mouth;
pub mod parameter;
pub mod rewrite;
pub mod state;
pub mod stomach;
pub mod tbox;
pub mod util;
pub mod whatsit;

use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, RwLock}; //,RwLockReadGuard,RwLockWriteGuard};
                              //use lazy_static::lazy_static;

use crate::comment::Comment;
use crate::common::dimension::Dimension;
pub use crate::common::error::*;
use crate::common::font::Font;
use crate::common::locator::Locator;
use crate::common::model::Model;
use crate::common::numeric_ops::NumericOps;
use crate::common::object::Object;
use crate::common::store::Stored;
use crate::definition::register::RegisterValue;
use crate::document::Document;
use crate::keyvals::KeyVals;
use crate::list::List;
use crate::state::{State, StateOptions};
use crate::stomach::Stomach;
use crate::tbox::Tbox;
use crate::token::{Catcode, Token};
use crate::tokens::Tokens;
use crate::whatsit::Whatsit;

// DG: I have experimented with doing the Perl-style "global singleton STATE with interior mutability"
//     and it just takes away from the elegance and guarantees of Rust style code. It's nasty.
//     consider that in a long chain of invocations (e.g. \input loading a binding, which loads another binding)
//     we have a dependency hierarchy of a mutable "&mut state" getting passed around.
//     During which time we can not *safely* obtain a "reading lock" over a RwLock wrapper around state.
//
//     To make anything work, we would need to hide the *entire* API of State behind a lock request/grant/release lifecycle
//     with a State struct that wraps: State(Arc<RwLock<StateData>>), where each call to say "lookup_value" will have to get+release a lock.
//
//     It is certainly possible. But at what cost? Runtime locking + reference counting costs, and then a *real risk* of deadlocking when locking
//      in complicated call chains. (Remember that RwLock allows for multiple readers, but the moment there is a writer,
//      no further locks will be granted until the writer is done)
//
//      I am leaving the trace that this has been tried. But I will continue to give it my all to avoid the global setup.
// lazy_static! {
//   static ref STATE: Arc<RwLock<State>> = Arc::new(RwLock::new(State::new(StateOptions::default())));
// }

pub struct Core {
  pub state: State,
  pub stomach: Arc<RwLock<Stomach>>,
  pub preload: Vec<String>,
}
impl Object for Core {
  fn get_locator(&self) -> Option<Cow<Locator>> { unimplemented!() }
}
#[derive(Default)]
pub struct CoreOptions {
  // First, state-related options:
  pub model: Option<Model>,
  pub verbosity: Option<i32>,
  pub strict: Option<bool>,
  pub include_comments: Option<bool>,
  pub include_styles: Option<bool>,
  pub nomathparse: Option<bool>,
  pub documentid: Option<String>,
  pub search_paths: Option<Vec<String>>,
  pub graphics_paths: Option<Vec<String>>,
  pub input_encoding: Option<String>,
  // The core-related
  pub preload: Option<Vec<String>>,
}

impl Core {
  pub fn new(options: CoreOptions) -> Self {
    let preload = match options.preload {
      None => Vec::new(),
      Some(p) => p,
    };

    // pass on the state options, defaults are handled in State::new
    let state_options = StateOptions {
      model: options.model,
      verbosity: options.verbosity,
      strict: options.strict,
      include_comments: options.include_comments,
      documentid: options.documentid,
      search_paths: options.search_paths,
      graphics_paths: options.graphics_paths,
      include_styles: options.include_styles,
      input_encoding: options.input_encoding,
      nomathparse: options.nomathparse,
      ..StateOptions::default()
    };
    let stomach = Arc::new(RwLock::new(Stomach::default()));
    let mut state = State::new(state_options);
    state.stomach = Arc::clone(&stomach);

    // *STATE.write().unwrap() = istate;
    // Core { state: Arc::clone(&STATE), stomach, preload }
    Core { state, stomach, preload }
  }

  // pub fn get_state(&self) -> RwLockReadGuard<'_, State> { self.state.read().unwrap() }
  // pub fn get_state_mut(&mut self) -> RwLockWriteGuard<'_, State> { self.state.write().unwrap() }
  pub fn get_state(&self) -> &State { &self.state }
  pub fn get_state_mut(&mut self) -> &mut State { &mut self.state }
}
pub trait BoxOps: Object {
  fn unlist(&self) -> Vec<Digested> { unimplemented!() }
  fn be_absorbed(&self, document: &mut Document, state: &mut State) -> Result<()>;
  fn get_string(&self, state: &State) -> Result<Cow<str>>;
  fn get_tokens(&self) -> Option<&Tokens> { None }
  fn get_properties(&self) -> &HashMap<String, Stored>;
  fn set_property<T: Into<Stored>>(&mut self, key: &str, value: T);

  fn get_property(&self, key: &str) -> Option<Cow<Stored>> {
    if key == "isSpace" {
      match self.get_properties().get(key) {
        Some(value) => Some(Cow::Borrowed(value)),
        None => {
          let tex = self.get_tokens().map(|tks| tks.untex()).unwrap_or_default(); // !
          if !tex.is_empty() && tex.chars().all(char::is_whitespace) {
            // Check the TeX code, not (just) the string!
            Some(Cow::Owned(Stored::Bool(true)))
          } else {
            None
          }
        },
      }
    } else {
      self.get_properties().get(key).map(Cow::Borrowed)
    }
  }

  fn has_property(&self, key: &str) -> bool;
  fn get_property_bool(&self, _key: &str) -> bool;
  fn get_body(&self) -> Option<Digested> {
    Error!("boxops", "get_body", self, None, "Generic BoxOps::get_body should never be called!");
    None
  }
  fn get_font(&self, state: &mut State) -> Result<Option<Cow<Font>>>;
  fn set_font(&mut self, font: Arc<Font>) { unimplemented!() }

  fn set_width<T: Into<Stored>>(&mut self, width: T) { self.set_property("width", width); }

  // For the dimensions of boxes, we'll store the (lazily) computed size as:
  //    cwidth, cheight, cdepth
  // and the explicitly requested/assigned size as
  //    width, height, depth.
  // Generally speaking, an XML element should only get width, height, depth
  // attributes when they were explicitly set.
  // However, when requesting the size of a box, you'd get either (w/ explicit size overriding)

  fn get_width(&mut self, options: Option<HashMap<String, Stored>>, state: &mut State) -> Result<Option<RegisterValue>> {
    if !self.has_property("width") && !self.has_property("cwidth") {
      // TODO: Restore caching?
      // self.compute_size_store(options.unwrap_or_default(), state)?
      let (w, _, _) = self.compute_size(options.unwrap_or_default(), state)?;
      return Ok(Some(RegisterValue::Dimension(w)));
    }

    Ok(match self.get_property("width") {
      Some(val) => (&*val).into(),
      None => match self.get_property("cwidth") {
        Some(val) => (&*val).into(),
        None => Some(RegisterValue::Dimension(Dimension::default())),
      },
    })
  }
  fn set_height<T: Into<Stored>>(&mut self, width: T) { self.set_property("height", width); }
  fn get_height(&self, state: &State) -> Option<RegisterValue> {
    match self.get_property("height") {
      Some(val) => (&*val).into(),
      None => Some(RegisterValue::Dimension(Dimension::default())),
    }
  }
  fn set_depth<T: Into<Stored>>(&mut self, width: T) { self.set_property("depth", width); }
  fn get_depth(&self, state: &State) -> Option<RegisterValue> {
    match self.get_property("depth") {
      Some(val) => (&*val).into(),
      None => Some(RegisterValue::Dimension(Dimension::default())),
    }
  }

  fn get_size(&self, options: Option<HashMap<String, Stored>>, state: &mut State) -> Result<(Dimension, Dimension, Dimension)> {
    // TODO: Reintroduce caching?
    if !(self.has_property("cwidth") && self.has_property("cheight") && self.has_property("cdepth")) {
      return self.compute_size(options.unwrap_or_default(), state);
    }
    let props = self.get_properties();

    // Debug("SIZE of $self"
    //     . "\n preassigned: " . _showsize($$props{width},  $$props{height},  $$props{depth})
    //     . "\n calculated : " . _showsize($$props{cwidth}, $$props{cheight}, $$props{cdepth})
    //     . "\n w/options " . join(',', map { $_ . "=" . ToString($options{$_}); } sort keys %options)
    //     . "\n =>: " . _showsize($$props{width} || $$props{cwidth}, $$props{height} || $$props{cheight}, $$props{depth} || $$props{cdepth})
    //     . "\n   Of " . ToString($self)) if $LaTeXML::DEBUG{size};
    Ok((
      match props.get("width") {
        Some(Stored::Dimension(w)) => *w,
        _ => match props.get("cwidth") {
          Some(Stored::Dimension(w)) => *w,
          _ => Dimension::default(),
        },
      },
      match props.get("height") {
        Some(Stored::Dimension(h)) => *h,
        _ => match props.get("cheight") {
          Some(Stored::Dimension(h)) => *h,
          _ => Dimension::default(),
        },
      },
      match props.get("depth") {
        Some(Stored::Dimension(d)) => *d,
        _ => match props.get("cdepth") {
          Some(Stored::Dimension(d)) => *d,
          _ => Dimension::default(),
        },
      },
    ))
  }

  /// omg
  ///  Fake computing the dimensions of strings (typically single chars).
  ///  Eventually, this needs to link into real font data
  fn compute_size_store(&mut self, mut options: HashMap<String, Stored>, state: &mut State) -> Result<()> {
    for key in ["width", "height", "depth", "vattach", "layout"] {
      if let Some(v) = self.get_property(key) {
        options.insert(String::from(key), v.into_owned());
      }
    }

    let (w, h, d) = self.compute_size(options, state)?;

    if !self.has_property("cwidth") {
      self.set_property("cwidth", w);
    }
    if !self.has_property("cheight") {
      self.set_property("cheight", h);
    }
    if !self.has_property("cdepth") {
      self.set_property("cdepth", d);
    }
    Ok(())
  }

  fn compute_size(&self, options: HashMap<String, Stored>, state: &mut State) -> Result<(Dimension, Dimension, Dimension)>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TexMode {
  Math,
  Text,
}

/// These are all kinds of data which rtx considers officially supported
/// as outputs from the digestion phase of TeX, i.e. from invoking a token.
/// Each variant is wrapped in an `Arc`, for cheap(er) cloning when passing around
/// these objects to various auxiliary state (e.g. bookkeeping current box),
/// but also for repeatedly passing them as owned into binding closures
/// while also storing them in their owner Box.
//
// This model is incredibly hard to achieve with lifetimes, so
// we employ reference counting instead (close to their original Perl design).
// A strict OO-hierarchy of object ownership (with no auxiliary state metadata)
// would allow a Rust-like redesign. But it could be too hard to achieve in practice.
#[derive(Clone)]
pub struct Digested(Arc<DigestedData>);
pub enum DigestedData {
  TBox(Tbox),
  Whatsit(RwLock<Whatsit>),
  List(List),
  Postponed(Tokens),
  KeyVals(KeyVals),
  RegisterValue(RegisterValue),
  Comment(Comment),
}

// Digested and DigestedData are transparent for debugging -- just show the inner data
impl fmt::Debug for Digested {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{:?}", *self.0) }
}
impl fmt::Debug for DigestedData {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    use DigestedData::*;
    match self {
      TBox(v) => write!(f, "{v:?}"),
      Whatsit(v) => write!(f, "{v:?}"),
      List(v) => write!(f, "{v:?}"),
      Postponed(v) => write!(f, "{v:?}"),
      KeyVals(v) => write!(f, "{v:?}"),
      RegisterValue(v) => write!(f, "{v:?}"),
      Comment(v) => write!(f, "{v:?}"),
    }
  }
}

impl PartialEq for Digested {
  fn eq(&self, other: &Digested) -> bool {
    use DigestedData::*;
    match *self.0 {
      TBox(ref tb) => {
        if let TBox(ref tb2) = *other.0 {
          tb == tb2
        } else {
          false
        }
      },
      Whatsit(ref tb) => {
        if let Whatsit(ref tb2) = *other.0 {
          *tb.read().unwrap() == *tb2.read().unwrap()
        } else {
          false
        }
      },
      List(ref tb) => {
        if let List(ref tb2) = *other.0 {
          tb == tb2
        } else {
          false
        }
      },
      Postponed(ref tb) => {
        if let Postponed(ref tb2) = *other.0 {
          tb == tb2
        } else {
          false
        }
      },
      KeyVals(ref tb) => {
        if let KeyVals(ref tb2) = *other.0 {
          tb == tb2
        } else {
          false
        }
      },
      RegisterValue(ref tb) => {
        if let RegisterValue(ref tb2) = *other.0 {
          tb == tb2
        } else {
          false
        }
      },
      Comment(ref tb) => {
        if let Comment(ref tb2) = *other.0 {
          tb == tb2
        } else {
          false
        }
      },
    }
  }
}

// Important: we need to postpone the creation of a box until a time where
// we have the most current font information
impl<'a> From<&'a String> for Digested {
  fn from(value: &'a String) -> Digested { Digested(Arc::new(DigestedData::Postponed(Tokens::new(ExplodeText!(value))))) }
}
impl From<String> for Digested {
  fn from(value: String) -> Digested { Digested(Arc::new(DigestedData::Postponed(Tokens::new(ExplodeText!(value))))) }
}
impl From<Tokens> for Digested {
  fn from(value: Tokens) -> Digested { Digested(Arc::new(DigestedData::Postponed(value))) }
}
impl From<Tbox> for Digested {
  fn from(value: Tbox) -> Digested { Digested(Arc::new(DigestedData::TBox(value))) }
}
impl From<List> for Digested {
  fn from(value: List) -> Digested { Digested(Arc::new(DigestedData::List(value))) }
}
impl From<Whatsit> for Digested {
  fn from(value: Whatsit) -> Digested { Digested(Arc::new(DigestedData::Whatsit(RwLock::new(value)))) }
}
impl From<KeyVals> for Digested {
  fn from(value: KeyVals) -> Digested { Digested(Arc::new(DigestedData::KeyVals(value))) }
}
impl From<RegisterValue> for Digested {
  fn from(value: RegisterValue) -> Digested { Digested(Arc::new(DigestedData::RegisterValue(value))) }
}

impl<'a> From<&'a Digested> for Option<crate::Digested> {
  fn from(value: &'a Digested) -> Option<crate::Digested> { Some(value.clone()) }
}

// impl<'a> From<&'a Digested> for Tokens {
//   fn from(value: &'a Digested) -> Tokens { value.revert(state).unwrap() }
// }
// impl From<Digested> for Tokens {
//   fn from(value: Digested) -> Tokens { value.revert(state).unwrap() }
// }
impl From<Digested> for Result<Digested> {
  fn from(value: Digested) -> Result<Digested> { Ok(value) }
}
impl From<Digested> for Result<Vec<Digested>> {
  fn from(value: Digested) -> Result<Vec<Digested>> { Ok(vec![value]) }
}
impl From<Digested> for Result<Option<Digested>> {
  fn from(value: Digested) -> Result<Option<Digested>> { Ok(Some(value)) }
}

impl Default for Digested {
  fn default() -> Self { Digested(Arc::new(DigestedData::TBox(Tbox::default()))) }
}

impl fmt::Display for Digested {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    use DigestedData::*;
    match *self.0 {
      TBox(ref b) => write!(f, "{b}"),
      List(ref l) => write!(f, "{l}"),
      Whatsit(ref w) => write!(f, "{}", w.read().unwrap()),
      Postponed(ref t) => write!(f, "{t}"),
      KeyVals(ref kvs) => write!(f, "{kvs}"),
      Comment(ref c) => write!(f, "{c}"),
      RegisterValue(ref rv) => write!(f, "{rv}"),
    }
  }
}
impl Object for Digested {
  fn stringify(&self) -> String {
    use DigestedData::*;
    match *self.0 {
      TBox(ref b) => b.stringify(),
      List(ref l) => l.stringify(),
      Whatsit(ref w) => w.read().unwrap().stringify(),
      Postponed(ref t) => (*t).stringify(),
      KeyVals(ref kvs) => kvs.stringify(),
      Comment(ref c) => c.stringify(),
      RegisterValue(ref rv) => (*rv).stringify(),
    }
  }
  fn get_locator(&self) -> Option<Cow<Locator>> {
    use DigestedData::*;
    match *self.0 {
      TBox(ref b) => b.get_locator(),
      List(ref l) => l.get_locator(),
      Comment(ref c) => c.get_locator(),
      Whatsit(ref w) => w.read().unwrap().get_locator().map(|l| Cow::Owned(l.into_owned())),
      KeyVals(ref kvs) => kvs.get_locator(), // KeyVals locator?
      RegisterValue(ref rv) => rv.get_locator(),
      Postponed(ref t) => None, // Tokens locator?
    }
  }
  fn revert(&self, state: &State) -> Result<Tokens> {
    use DigestedData::*;
    match *self.0 {
      TBox(ref b) => b.revert(state),
      List(ref l) => l.revert(state),
      Whatsit(ref w) => w.read().unwrap().revert(state),
      Postponed(ref t) => Ok(t.clone()),
      KeyVals(ref kvs) => kvs.revert(state),
      Comment(ref c) => c.revert(state),
      RegisterValue(ref rv) => rv.revert(state),
    }
  }
}

impl BoxOps for Digested {
  fn unlist(&self) -> Vec<Digested> {
    use DigestedData::*;
    match *self.0 {
      TBox(_) | Whatsit(_) | KeyVals(_) | Comment(_) | Postponed(_) | RegisterValue(_) => vec![self.clone()],
      List(ref l) => l.unlist(),
    }
  }

  fn be_absorbed(&self, document: &mut Document, state: &mut State) -> Result<()> {
    use DigestedData::*;
    match &*self.0 {
      TBox(b) => b.be_absorbed(document, state),
      List(l) => l.be_absorbed(document, state),
      Comment(c) => c.be_absorbed(document, state),
      Whatsit(w) => w.read().unwrap().be_absorbed(document, state),
      KeyVals(kvs) => kvs.be_absorbed(document, state),
      Postponed(_) => unimplemented!(),
      RegisterValue(ref _rv) => unimplemented!(),
    }
  }

  fn get_properties(&self) -> &HashMap<String, Stored> {
    use DigestedData::*;
    match *self.0 {
      TBox(ref b) => b.get_properties(),
      List(ref l) => l.get_properties(),
      KeyVals(ref kvs) => kvs.get_properties(),
      Whatsit(ref w) => unimplemented!(), // Oooof; w.read().unwrap().get_properties(),
      Postponed(_) | RegisterValue(_) | Comment(_) => unimplemented!(),
    }
  }

  fn set_property<T: Into<Stored>>(&mut self, key: &str, value: T) {
    match *self.0 {
      // TODO: This is only possible if we have interior mutability for *ALL* Digested variants
      // i.e. Arc<RwLock<Tbox>>, Arc<RwLock<List>>, etc.
      //
      // Digested::TBox(ref b) => b.set_property(key, value),
      // Digested::List(ref l) => l.set_property(key, value),
      DigestedData::Whatsit(ref w) => w.write().unwrap().set_property(key, value),
      DigestedData::List(ref l) => Debug!("ignore", "set_property", None, None, format!("List::set_property({key},_)")),
      _ => unimplemented!(),
    }
  }

  fn get_property(&self, key: &str) -> Option<Cow<Stored>> {
    use DigestedData::*;
    match *self.0 {
      TBox(ref b) => b.get_property(key),
      List(ref l) => l.get_property(key),
      Whatsit(ref w) => w.read().unwrap().get_property(key).map(|v| Cow::Owned(v.into_owned())),
      _ => unimplemented!(),
    }
  }
  fn get_string(&self, state: &State) -> Result<Cow<str>> {
    use DigestedData::*;
    match *self.0 {
      TBox(ref b) => b.get_string(state),
      List(ref l) => l.get_string(state),
      Whatsit(ref w) => match w.read().unwrap().get_string(state) {
        Ok(v) => Ok(Cow::Owned(v.into_owned())),
        Err(e) => Err(format!("failed Whatsit get_string: {e}").into()),
      },
      _ => unimplemented!(),
    }
  }
  fn has_property(&self, key: &str) -> bool {
    use DigestedData::*;
    match *self.0 {
      TBox(ref b) => b.has_property(key),
      List(ref l) => l.has_property(key),
      Whatsit(ref w) => w.read().unwrap().has_property(key),
      _ => unimplemented!(),
    }
  }
  fn get_body(&self) -> Option<Digested> {
    use DigestedData::*;
    match *self.0 {
      TBox(ref b) => {
        Error!("digested", "get_body", self, None, s!("Called get_body on Box: {:?}", b));
        None
      },
      List(ref l) => {
        Error!("digested", "get_body", self, None, s!("Called get_body on List: {:?}", l));
        None
      },
      Whatsit(ref w) => w.read().unwrap().get_body(),
      _ => unimplemented!(),
    }
  }
  fn get_property_bool(&self, key: &str) -> bool {
    use DigestedData::*;
    match *self.0 {
      TBox(ref b) => b.get_property_bool(key),
      List(ref l) => l.get_property_bool(key),
      Whatsit(ref w) => w.read().unwrap().get_property_bool(key),
      _ => unimplemented!(),
    }
  }
  fn get_font(&self, state: &mut State) -> Result<Option<Cow<Font>>> {
    use DigestedData::*;
    match *self.0 {
      TBox(ref b) => b.get_font(state),
      List(ref l) => l.get_font(state),
      Whatsit(ref w) => Ok(w.read().unwrap().get_font(state)?.map(|t| Cow::Owned(t.into_owned()))),
      Postponed(ref tks) => Ok(None),
      _ => unimplemented!(),
    }
  }

  fn compute_size(&self, options: HashMap<String, Stored>, state: &mut State) -> Result<(Dimension, Dimension, Dimension)> {
    use DigestedData::*;
    match *self.0 {
      TBox(ref b) => b.compute_size(options, state),
      List(ref l) => l.compute_size(options, state),
      KeyVals(ref kvs) => kvs.compute_size(options, state),
      Whatsit(ref w) => w.read().unwrap().compute_size(options, state),
      Postponed(_) | RegisterValue(_) | Comment(_) => unimplemented!(),
    }
  }
}

impl Digested {
  pub fn data(&self) -> &DigestedData { &self.0 }
  // convenience subset of NumericOps, added here for now as an experiment:
  pub fn value_of(&self) -> i32 {
    match &*self.0 {
      DigestedData::RegisterValue(rv) => rv.clone().value_of(),
      _ => 0,
    }
  }
  pub fn pt_value(&self, prec: Option<u8>) -> f32 {
    match &*self.0 {
      DigestedData::RegisterValue(rv) => rv.clone().pt_value(prec),
      _ => 0.0,
    }
  }
  pub fn any<F>(&self, mut check: F) -> bool
  where F: FnMut(&Self) -> bool {
    use DigestedData::*;
    match &*self.0 {
      TBox(_) | Whatsit(_) | Postponed(_) | KeyVals(_) | RegisterValue(_) => check(self),
      Comment(_) => true,
      List(l) => l.boxes.iter().any(check),
    }
  }

  pub fn all<F>(&self, mut check: F) -> bool
  where F: FnMut(&Self) -> bool {
    use DigestedData::*;
    match &*self.0 {
      TBox(_) | Whatsit(_) | Postponed(_) | KeyVals(_) | RegisterValue(_) => check(self),
      Comment(_) => true,
      List(l) => l.boxes.iter().all(check),
    }
  }

  pub fn is_empty(&self) -> bool {
    use DigestedData::*;
    match *self.0 {
      TBox(ref b) => b.is_empty(),
      List(ref l) => l.is_empty(),
      Whatsit(ref w) => w.read().unwrap().is_empty(),
      Postponed(ref tks) => tks.is_empty(),
      _ => unimplemented!(),
    }
  }

  /// Provide a way of emulating an `Undigested` argument, by requesting
  /// raw tokens, only when they are preserved -- empty otherwise.
  pub fn raw_tokens(&self) -> Option<&Tokens> {
    match *self.0 {
      DigestedData::Postponed(ref tks) => Some(tks),
      _ => None,
    }
  }

  pub fn to_attribute(&self) -> String { self.to_string() }
}
