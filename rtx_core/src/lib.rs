//! The Core of rtx - roughly the equivalent of TeX conversion.

#![allow(missing_docs)]
extern crate rustc_hash;

/// Auxiliary macros
#[macro_use]
pub mod aux_macros;
/// The Token and its Catcode
#[macro_use]
pub mod token;
/// Common abstractions that are useful at various stages of Core processing
#[macro_use]
pub mod common;
/// Grouping Tokens together
#[macro_use]
pub mod tokens;
/// The programmable API foundation for creating bindings of LaTeX sty/cls libraries
#[macro_use]
pub mod binding;
/// Support for TeX-like Alignments
pub mod alignment;
/// TeX comments as standalone objects
pub mod comment;
/// All possible definitions for TeX-native commands (expandable, primitive, constructor,...)
pub mod definition;
/// a shared interface for digested objects
pub mod digested;
/// An abstraction layer over the converted XML document
pub mod document;
/// The Gullet is responsible for reading Tokens and other data from the Mouth
pub mod gullet;
/// A LaTeX-like Key-Value object
pub mod keyval;
/// A collection of Key-Value objects, typically from a single LaTeX argument
pub mod keyvals;
/// Rules for combining together characters and other text rules
pub mod ligature;
/// A list of `Digested` objects
pub mod list;
/// The mouth is a thin interface over a file, responsible for reading characters and associating
/// them with catcodes
pub mod mouth;
/// The abstraction layer used by the Gullet to read arguments for the various kinds of TeX object
/// definitions
pub mod parameter;
/// Rules for rewriting the constructed XML document, after core processing has completed
pub mod rewrite;
/// A global, singleton, mutable State - hosts almost all TeX-facing runtime information for the
/// conversion
pub mod state;
/// The stomach is an abstraction responsible for digesting `Tokens` and `Register`s prepared by the
/// Gullet into Boxes
pub mod stomach;
/// A TeX-like digested Box
pub mod tbox;
/// Auxilary utilities that do not participate in the main conversion abstraction
pub mod util;
/// A TeX-like digested Whatsit
pub mod whatsit;

use rustc_hash::FxHashMap as HashMap;
use std::borrow::Cow;
use std::fmt;
use std::cell::RefCell;
use std::rc::Rc;
use once_cell::sync::Lazy;
use libxml::tree::Node;

use crate::common::dimension::Dimension;
pub use crate::common::error::*;
use crate::common::font::Font;
use crate::common::locator::Locator;
use crate::common::model::Model;
use crate::common::numeric_ops::NumericOps;
use crate::common::object::Object;
use crate::common::store::Stored;
use crate::definition::register::RegisterValue;
use crate::digested::{Digested, DigestedData};
use crate::document::Document;
use crate::state::{State, StateOptions};
use crate::stomach::Stomach;
use crate::tbox::Tbox;
use crate::tokens::Tokens;

pub static NO_PROPERTIES : Lazy<HashMap<String,Stored>> = Lazy::new(HashMap::default);

/// The Core conversion runtime
pub struct Core {
  /// the singleton State which bookkeeps all TeX-related state
  pub state: State,
  /// the singleton stomach executing the digestion
  pub stomach: Rc<RefCell<Stomach>>,
  /// a list of library names to be preloaded before the main conversion begins
  pub preload: Vec<String>,
}
impl Object for Core {
  fn get_locator(&self) -> Option<Cow<Locator>> { unimplemented!() }
}

/// Configuration for the Core processing
#[derive(Default)]
pub struct CoreOptions {
  // First, state-related options:
  /// a custom schema-induced model (default is `None`) for the final XML
  pub model: Option<Model>,
  /// default is 0, sub-zero values are quiet, positive values are verbose
  pub verbosity: Option<i32>,
  /// strict error-reporting (is this still used?)
  pub strict: Option<bool>,
  /// toggle preserving comments in the XML on/off
  pub include_comments: Option<bool>,
  /// toggle loading raw .sty modules on/off
  pub include_styles: Option<bool>,
  /// disable math parsing (enabled by default)
  pub nomathparse: Option<bool>,
  /// an optional, fixed id prefix for all xml:id attributes
  pub documentid: Option<String>,
  /// the list of paths used for loading TeX sources and packages
  pub search_paths: Option<Vec<String>>,
  /// the list of paths used for loading graphics assets
  pub graphics_paths: Option<Vec<String>>,
  /// set an explicit encoding of the input text
  pub input_encoding: Option<String>,
  /// a list of package names to preload before processing start
  pub preload: Option<Vec<String>>,
}

impl Core {
  /// instantiate a new Core processor
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
    let stomach = Rc::new(RefCell::new(Stomach::default()));
    let mut state = State::new(state_options);
    state.stomach = Rc::clone(&stomach);

    // *STATE.write().unwrap() = istate;
    // Core { state: Rc::clone(&STATE), stomach, preload }
    Core {
      state,
      stomach,
      preload,
    }
  }

  /// borrow the current state
  pub fn get_state(&self) -> &State { &self.state }
  /// mutably borrow the current state
  pub fn get_state_mut(&mut self) -> &mut State { &mut self.state }
}
/// Common operations for Box-like (digested) data
pub trait BoxOps: Object {
  /// If composite, unwrap into the contained digested objects (or return self)
  fn unlist(&self) -> Vec<Digested> { unimplemented!() }
  fn unlist_ref(&self) -> Vec<&Digested> { unimplemented!() }
  /// absorb the current object into the `Document` XML - returning the corresponding nodes
  fn be_absorbed(&self, document: &mut Document, state: &mut State) -> Result<Vec<Node>>;
  /// be_absorbed but with allowed side-effects on the carrier (for `Alignment` only)
  fn be_absorbed_mut(&mut self, _document: &mut Document, _state: &mut State) -> Result<Vec<Node>> { unimplemented!(); }
  /// build a string representation of the underlying digested data
  fn get_string(&self, state: &State) -> Result<Cow<str>>;
  /// get the underlying tokens (preceding digestion)
  fn get_tokens(&self) -> Option<&Tokens> { None }
  /// get the map of named properties
  fn get_properties(&self) -> &HashMap<String, Stored> {
    &NO_PROPERTIES
  }
  /// get a mutable reference to the map of named properties
  fn get_properties_mut(&mut self) -> &mut HashMap<String, Stored> { unimplemented!() }
  /// set a named property (allows all `Stored` types for values)
  fn set_property<T: Into<Stored>>(&mut self, key: &str, value: T) {
    self.get_properties_mut().insert(key.to_string(), value.into());
  }
  /// get a single named property (with special "isSpace" check)
  fn get_property(&self, key: &str) -> Option<Cow<Stored>> {
    if key == "isSpace" {
      match self.get_properties().get(key) {
        Some(value) => Some(Cow::Borrowed(value)),
        None => {
          let tex = self
            .get_tokens()
            .map(|tks| tks.clone().untex())
            .unwrap_or_default(); // !
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
  /// get a mutable reference to a single named property (does NOT have the "isSpace" check)
  fn get_property_mut(&mut self, key:&str) -> Option<&mut Stored> {
    self.get_properties_mut().get_mut(key)
  }
  /// checks if a property key has been set
  fn has_property(&self, key: &str) -> bool { self.get_properties().contains_key(key) }
  /// obtains a boolean property value (false unless `Stored::Bool`)
  fn get_property_bool(&self, key: &str) -> bool {
    matches!(self.get_properties().get(key), Some(Stored::Bool(true)))
  }
  /// obtains the "body" of a digested object which captured it
  fn get_body(&self) -> Option<Digested> {
    Error!(
      "boxops",
      "get_body",
      self,
      None,
      "Generic BoxOps::get_body should never be called!"
    );
    None
  }
  /// gets the associated font, if any
  fn get_font(&self, state: &mut State) -> Result<Option<Cow<Font>>>;
  /// sets an associated font
  fn set_font(&mut self, _font: Rc<Font>) { unimplemented!() }
  /// sets a "width" property, for sizing
  fn set_width<T: Into<Stored>>(&mut self, width: T) { self.set_property("width", width); }

  // For the dimensions of boxes, we'll store the (lazily) computed size as:
  //    cwidth, cheight, cdepth
  // and the explicitly requested/assigned size as
  //    width, height, depth.
  // Generally speaking, an XML element should only get width, height, depth
  // attributes when they were explicitly set.
  // However, when requesting the size of a box, you'd get either (w/ explicit size overriding)

  /// gets the "width" property value, if any
  fn get_width(
    &self,
    options: Option<HashMap<String, Stored>>,
    state: &mut State,
  ) -> Result<Option<RegisterValue>> {
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
  /// sets a "height" property value, for sizing
  fn set_height<T: Into<Stored>>(&mut self, width: T) { self.set_property("height", width); }
  /// gets the "height" property value, if any
  fn get_height(&self, _state: &State) -> Option<RegisterValue> {
    match self.get_property("height") {
      Some(val) => (&*val).into(),
      None => Some(RegisterValue::Dimension(Dimension::default())),
    }
  }
  /// sets a "depth" property value, for sizing
  fn set_depth<T: Into<Stored>>(&mut self, width: T) { self.set_property("depth", width); }
  /// gets the "depth" property value, if any
  fn get_depth(&self, _state: &State) -> Option<RegisterValue> {
    match self.get_property("depth") {
      Some(val) => (&*val).into(),
      None => Some(RegisterValue::Dimension(Dimension::default())),
    }
  }
  /// gets the box size as a triple of (width, height, depth)
  fn get_size(
    &self,
    options: Option<HashMap<String, Stored>>,
    state: &mut State,
  ) -> Result<(Dimension, Dimension, Dimension)> {
    // TODO: Reintroduce caching?
    if !(self.has_property("cwidth") && self.has_property("cheight") && self.has_property("cdepth"))
    {
      return self.compute_size(options.unwrap_or_default(), state);
    }
    let props = self.get_properties();

    // Debug("SIZE of $self"
    //     . "\n preassigned: " . _showsize($$props{width},  $$props{height},  $$props{depth})
    //     . "\n calculated : " . _showsize($$props{cwidth}, $$props{cheight}, $$props{cdepth})
    //     . "\n w/options " . join(',', map { $_ . "=" . ToString($options{$_}); } sort keys
    // %options)     . "\n =>: " . _showsize($$props{width} || $$props{cwidth}, $$props{height}
    // || $$props{cheight}, $$props{depth} || $$props{cdepth})     . "\n   Of " .
    // ToString($self)) if $LaTeXML::DEBUG{size};
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

  /// deprecated/to be revisited - computes and caches the size of a box-like object
  fn compute_size_and_cache(
    &mut self,
    mut options: HashMap<String, Stored>,
    state: &mut State,
  ) -> Result<(Dimension, Dimension, Dimension)> {
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
    Ok((w,h,d))
  }

  /// computes and returns the size of a box-like object
  fn compute_size(
    &self,
    options: HashMap<String, Stored>,
    state: &mut State,
  ) -> Result<(Dimension, Dimension, Dimension)>;
}

/// The current TeX processing mode
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TexMode {
  /// TeX's math mode
  Math,
  /// TeX's text mode
  Text,
}
