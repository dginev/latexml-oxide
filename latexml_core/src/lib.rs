//! The Core of latexml - roughly the equivalent of TeX conversion.
#![feature(thread_local)]
#![allow(missing_docs)]
#![allow(clippy::invisible_characters)] // Font metrics contain real zero-width Unicode chars from TFM files
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
pub mod cycle_guard;
pub mod stack_guard;
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
/// A global, singleton, mutable state - hosts almost all TeX-facing runtime information for the
/// conversion
#[macro_use]
pub mod state;
/// Code generator: dump file → compiled Rust module
pub mod dump_codegen;
/// Reader for Rust-native kernel dump files
pub mod dump_reader;
/// Writer for Rust-native kernel dump files
pub mod dump_writer;
/// The stomach is an abstraction responsible for digesting `Tokens` and `Register`s prepared by the
/// Gullet into Boxes
#[macro_use]
pub mod stomach;
/// A TeX-like digested Box
pub mod tbox;
/// Per-job structured telemetry: phase wall times, counts, resource peaks.
/// See `docs/TELEMETRY.md` for the design contract.
pub mod telemetry;
/// Auxilary utilities that do not participate in the main conversion abstraction
pub mod util;
/// Main-level wall-clock watchdog that forcibly aborts the process after a deadline.
/// Complements the cooperative `stomach::check_timeout` polling for native-code hotspots
/// (Marpa, libxml2, libxslt) that don't return to the digestion loop.
pub mod watchdog;
/// A TeX-like digested Whatsit
pub mod whatsit;

use std::{borrow::Cow, fmt, rc::Rc};

use libxml::tree::Node;
use once_cell::sync::Lazy;
/// Initialize libxml2 for thread safety. Must be called before any libxml2
/// operations that don't go through `libxml::parser::Parser`. Delegates to
/// the safe wrapper in rust-libxml, which uses its own `std::sync::Once` to
/// guarantee exactly-once initialisation even across threads.
///
/// See: <https://dev.w3.org/XInclude-Test-Suite/libxml2-2.4.24/doc/threads.html>
pub fn ensure_libxml_init() { libxml::init_parser(); }

/// Free this thread's accumulated engine state — the three `State`
/// singletons (`STATE`, `STD_STATE`, `STY_STATE`) **and** the
/// string-interner arena — returning them to a fresh baseline.
///
/// **Why this exists.** The engine's roots (`STATE`, `arena::ARENA`, …)
/// are `#[thread_local]` *attribute* statics. Unlike the `thread_local!`
/// macro, the attribute does **not** run destructors on thread exit, so a
/// thread that builds a full engine and then exits *leaks* it (~110 MB
/// for a typical document). The single-conversion `latexml_oxide` binary
/// never notices — it runs one conversion and the process exits. But any
/// process that runs **many** conversions across **many** threads
/// (notably the test harness, where libtest spawns a fresh thread per
/// test) accumulates one leaked engine per conversion (measured: ~4.9 GB
/// across `50_structure`, which then trips the per-process RSS fuse in
/// `stomach::check_timeout`). Resetting between conversions frees that
/// memory before the thread exits (peak fell ~4.9 GB → ~2.9 GB at -j20).
///
/// **Why reset the interner here.** The interner *could* be kept across
/// conversions — that is the faithful daemon design (Perl keeps its
/// symbol table and resets only the binding stack via
/// `pushDaemonFrame`/`popDaemonFrame` in `LaTeXML.pm`), and re-interning
/// the same ~110k base symbols next conversion is deduped. But that only
/// pays off when the **same thread** handles multiple conversions. The
/// test harness gets a **fresh thread per test**, so its interner can
/// never be reused — keeping it would just leak it on thread exit. So we
/// reset it too. A future *thread-reusing* daemon should instead keep the
/// interner by calling [`state::reset_thread_state`] alone (State only).
///
/// **Soundness.** Resetting the interner invalidates *every* live
/// `SymStr` on the thread, so this is sound only between fully
/// independent conversions — when the prior conversion's output has
/// already been serialized to owned data and nothing will read a
/// pre-reset symbol again. The test harness satisfies this (each test
/// serializes to owned `String`s, then resets before its thread exits).
/// It does **not** reclaim libxml2's process-global C state (parser
/// dictionaries) — that residual (~24 MB/test) is left as-is rather than
/// risk the global `xmlCleanupParser`.
pub fn reset_thread_engine() {
  state::reset_thread_state();
  common::arena::reset();
}

pub use crate::common::error::*;
use crate::{
  common::{
    arena::SymHashMap as HashMap, dimension::Dimension, font::Font, locator::Locator, model::Model,
    numeric_ops::NumericOps, object::Object, store::Stored,
  },
  definition::register::RegisterValue,
  digested::{Digested, DigestedData},
  document::Document,
  state::{State, StateOptions, set_state},
  stomach::Stomach,
  tbox::Tbox,
  tokens::Tokens,
};

pub static NO_PROPERTIES: Lazy<HashMap<Stored>> = Lazy::new(HashMap::default);

/// The Core conversion runtime
pub struct Core {
  /// a list of library names to be preloaded before the main conversion begins
  pub preload: Vec<String>,
}

/// Configuration for the Core processing
#[derive(Default)]
pub struct CoreOptions {
  // First, state::related options:
  /// a custom schema-induced model (default is `None`) for the final XML
  pub model:            Option<Model>,
  /// default is 0, sub-zero values are quiet, positive values are verbose
  pub verbosity:        Option<i32>,
  /// strict error-reporting (is this still used?)
  pub strict:           Option<bool>,
  /// toggle preserving comments in the XML on/off
  pub include_comments: Option<bool>,
  /// toggle loading raw .sty modules on/off
  pub include_styles:   Option<bool>,
  /// disable math parsing (enabled by default)
  pub nomathparse:      Option<bool>,
  /// enable source-locator (`--source-map`) tracking + emission (off by
  /// default). See `docs/SOURCE_PROVENANCE.md`.
  pub source_map:       Option<bool>,
  /// an optional, fixed id prefix for all xml:id attributes
  pub documentid:       Option<String>,
  /// the list of paths used for loading TeX sources and packages
  pub search_paths:     Option<Vec<String>>,
  /// the list of paths used for loading graphics assets
  pub graphics_paths:   Option<Vec<String>>,
  /// set an explicit encoding of the input text
  pub input_encoding:   Option<String>,
  /// a list of package names to preload before processing start
  pub preload:          Option<Vec<String>>,
}

impl Core {
  /// instantiate a new Core processor
  pub fn new(options: CoreOptions) -> Self {
    // Eagerly initialize the engine's `#[thread_local]` roots, LEAVES-FIRST,
    // on THIS thread before any of them is touched re-entrantly. ARENA is the
    // universal leaf (every root's Lazy initializer interns via arena::pin);
    // the token constants, MODEL, the gullet roots and the STD/STY catcode
    // templates all reach into ARENA (and the gullet/template roots build
    // tokens). Forcing them in dependency order up front — before
    // `set_stomach`/`set_state` below trigger their own initializers, and
    // before expansion lazily touches the gullet/catcode roots mid-run —
    // guarantees no root's `Lazy` init ever runs another root's init
    // *re-entrantly*. That cross-`#[thread_local]`-during-init pattern is
    // benign on Linux/ELF TLS but is the documented macOS hazard
    // (rust-lang/rust#29594) behind the macOS worker-thread memory
    // corruption in issue #217 (varying garbage node types → panics /
    // SIGSEGV / SIGBUS, only on macOS, only in libtest's worker threads —
    // the single-conversion main-thread CLI was never affected). Forcing
    // just ARENA+MODEL cut the failures 4→1; this completes the set. No
    // behavioral change on Linux (these all initialize during any
    // conversion anyway — this only fixes the ORDER).
    common::arena::force_init(); // leaf
    token::force_init(); // token constants -> arena
    common::model::force_init(); // Model::new -> arena
    gullet::force_init(); // DEFERRED_COMMANDS / COLUMN_ENDS / GULLET -> arena
    state::force_init(); // STD_STATE / STY_STATE templates -> arena
    let preload = options.preload.unwrap_or_default();
    // pass on the state::options, defaults are handled in state::new
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
      source_map: options.source_map,
      ..StateOptions::default()
    };
    stomach::set_stomach(Stomach::default());
    set_state(State::new(state_options));
    Core { preload }
  }
}
/// Common operations for Box-like (digested) data
pub trait BoxOps: Object {
  /// If composite, unwrap into the contained digested objects (or return self)
  fn unlist(&self) -> Vec<Digested> { Vec::new() }
  fn unlist_ref(&self) -> Vec<Cow<'_, Digested>> { Vec::new() }
  /// absorb the current object into the `Document` XML - returning the corresponding nodes
  fn be_absorbed(&self, document: &mut Document) -> Result<Vec<Node>>;
  /// be_absorbed but with allowed side-effects on the carrier (for `Alignment` only)
  fn be_absorbed_mut(&mut self, _document: &mut Document) -> Result<Vec<Node>> {
    self.be_absorbed(_document)
  }
  /// build a string representation of the underlying digested data
  fn get_string(&self) -> Result<Cow<'_, str>>;
  /// get the underlying tokens (preceding digestion)
  fn get_tokens(&self) -> Option<&Tokens> { None }
  /// deprecated: get the map of named properties. This can not be usable as long as we have any
  /// data behind a RefCell wrapper.
  /// Use `with_properties` instead.
  fn get_properties(&self) -> &HashMap<Stored> { &NO_PROPERTIES }

  /// execute a function using this object's named properties
  fn with_properties<R, FnR>(&self, caller: FnR) -> R
  where FnR: FnOnce(&HashMap<Stored>) -> R;
  /// get a mutable reference to the map of named properties
  fn get_properties_mut(&mut self) -> &mut HashMap<Stored> {
    panic!("get_properties_mut called on type without mutable properties");
  }
  /// set a named property (allows all `Stored` types for values)
  fn set_property<T: Into<Stored>>(&mut self, key: &str, value: T) {
    self.get_properties_mut().insert(key, value.into());
  }
  /// get a single named property (with special "isSpace" check)
  fn get_property(&self, key: &str) -> Option<Cow<'_, Stored>> {
    self.with_properties(|props| {
      if key == "isSpace" {
        match props.get(key) {
          Some(value) => Some(Cow::Owned(value.clone())),
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
        props.get(key).map(|v| Cow::Owned(v.clone()))
      }
    })
  }
  fn get_property_string(&self, key: &str) -> String {
    self
      .get_property(key)
      .map(|v| v.to_string())
      .unwrap_or_default()
  }
  /// get a mutable reference to a single named property (does NOT have the "isSpace" check)
  fn get_property_mut(&mut self, key: &str) -> Option<&mut Stored> {
    self.get_properties_mut().get_mut(key)
  }
  /// checks if a property key has been set
  fn has_property(&self, key: &str) -> bool {
    self.with_properties(|props| props.contains_key(key))
  }
  /// obtains a boolean property value (false unless `Stored::Bool`)
  fn get_property_bool(&self, key: &str) -> bool {
    self.with_properties(|props| matches!(props.get(key), Some(Stored::Bool(true))))
  }
  /// obtains the "body" of a digested object which captured it
  fn get_body(&self) -> Result<Option<Digested>> {
    Error!(
      "boxops",
      "get_body",
      "Generic BoxOps::get_body should never be called!"
    );
    Ok(None)
  }
  /// gets the associated font, if any
  fn get_font(&self) -> Result<Option<Cow<'_, Font>>>;
  /// sets an associated font
  fn set_font(&mut self, _font: Rc<Font>) { /* no-op for types without font */
  }
  /// sets a "width" property, for sizing
  fn set_width<T: Into<Stored>>(&mut self, width: T) { self.set_property("width", width); }

  // For the dimensions of boxes, we'll store the (lazily) computed size as:
  //    cached_width, cached_height, cached_depth
  // and the explicitly requested/assigned size as
  //    width, height, depth.
  // Generally speaking, an XML element should only get width, height, depth
  // attributes when they were explicitly set.
  // However, when requesting the size of a box, you'd get either (w/ explicit size overriding)

  /// gets the "width" property value, if any
  fn get_width(&self, options: Option<HashMap<Stored>>) -> Result<Option<RegisterValue>> {
    if !self.has_property("width") && !self.has_property("cached_width") {
      // TODO: Restore caching?
      // self.compute_size_store(options.unwrap_or_default())?
      let (w, ..) = self.compute_size(options.unwrap_or_default())?;
      return Ok(Some(RegisterValue::Dimension(w)));
    }

    // Convert MuGlue/MuDimension widths to pt (1mu = font_size/18). `\the\wd`
    // is a dimension query — the result must be Dimension-typed. Without
    // this conversion `\hbox{\,}` width came back as `MuGlue(3mu)` and
    // formatted as `3.0pt` (raw mu treated as pt) instead of `1.66663pt`.
    // Order of ops (div by 18 then mul by fs) matches Perl integer
    // truncation: see `mu_to_pt_value` in store.rs.
    fn coerce_mu(val: &Stored) -> Option<RegisterValue> {
      match val {
        Stored::MuGlue(g) => {
          let fs = state::lookup_font()
            .and_then(|f| f.get_size())
            .unwrap_or(10.0);
          let muwidth = (fs * common::numeric_ops::UNITY_F64 / 18.0) as i64;
          let pt_scaled =
            (g.value_of() as f64 * muwidth as f64 / common::numeric_ops::UNITY_F64).trunc();
          Some(RegisterValue::Dimension(Dimension::new(pt_scaled as i64)))
        },
        Stored::MuDimension(d) => {
          let fs = state::lookup_font()
            .and_then(|f| f.get_size())
            .unwrap_or(10.0);
          let pt_scaled = (d.value_of() / 18) as f64 * fs;
          Some(RegisterValue::Dimension(Dimension::new(pt_scaled as i64)))
        },
        _ => val.into(),
      }
    }
    Ok(match self.get_property("width") {
      Some(val) => coerce_mu(&val),
      None => match self.get_property("cached_width") {
        Some(val) => coerce_mu(&val),
        None => Some(RegisterValue::Dimension(Dimension::default())),
      },
    })
  }
  /// sets a "height" property value, for sizing
  fn set_height<T: Into<Stored>>(&mut self, width: T) { self.set_property("height", width); }
  /// gets the "height" property value, if any.
  /// Checks "height", then "cached_height", then computes from font if needed.
  fn get_height(&self) -> Option<RegisterValue> {
    match self.get_property("height") {
      Some(val) => (&*val).into(),
      None => match self.get_property("cached_height") {
        Some(val) => (&*val).into(),
        None => match self.compute_size(HashMap::default()) {
          Ok((_, h, _)) => Some(RegisterValue::Dimension(h)),
          _ => Some(RegisterValue::Dimension(Dimension::default())),
        },
      },
    }
  }
  /// sets a "depth" property value, for sizing
  fn set_depth<T: Into<Stored>>(&mut self, width: T) { self.set_property("depth", width); }
  /// gets the "depth" property value, if any.
  /// Checks "depth", then "cached_depth", then computes from font if needed.
  fn get_depth(&self) -> Option<RegisterValue> {
    match self.get_property("depth") {
      Some(val) => (&*val).into(),
      None => match self.get_property("cached_depth") {
        Some(val) => (&*val).into(),
        None => match self.compute_size(HashMap::default()) {
          Ok((_, _, d)) => Some(RegisterValue::Dimension(d)),
          _ => Some(RegisterValue::Dimension(Dimension::default())),
        },
      },
    }
  }
  /// gets the box size as a triple of (width, height, depth)
  /// the generic implementation is immutable and will recompute the size on each call
  /// see `Digested::get_size` for a variant with interior mutability which caches the box size
  fn get_size(
    &mut self,
    options: Option<HashMap<Stored>>,
  ) -> Result<(
    Dimension,
    Dimension,
    Dimension,
    Dimension,
    Dimension,
    Dimension,
  )> {
    // TODO: Reintroduce caching?
    if !(self.has_property("cached_width")
      && self.has_property("cached_height")
      && self.has_property("cached_depth"))
    {
      self.compute_size_and_cache(options.unwrap_or_default())?;
    }
    self.with_properties(|props| {
      let (width, height, depth, cached_width, cached_height, cached_depth) = (
        props.get("width"),
        props.get("height"),
        props.get("depth"),
        props.get("cached_width"),
        props.get("cached_height"),
        props.get("cached_depth"),
      );

      // eprintln!("SIZE of {} {}", std::any::type_name::<Self>(), self.get_string()?);
      //     . "\n preassigned: " . _showsize($$props{width},  $$props{height},  $$props{depth})
      //     . "\n calculated : " . _showsize($$props{cached_width}, $$props{cached_height},
      // $$props{cached_depth})     . "\n w/options " . join(',', map { $_ . "=" .
      // ToString($options{$_}); } sort keys %options)     . "\n =>: " .
      // _showsize($$props{width} || $$props{cached_width}, $$props{height}
      // || $$props{cached_height}, $$props{depth} || $$props{cached_depth})     . "\n   Of " .
      // ToString($self)) if $LaTeXML::DEBUG{size};
      // Helper: extract a Dimension from a Stored value.
      // Handles Dimension directly, plus Glue/MuGlue/MuDimension by extracting the base value.
      // MuGlue/MuDimension values are in scaled mu (1mu = font_size/18);
      // convert to scaled pt using the current font size.
      fn stored_to_dim(s: Option<&Stored>) -> Option<Dimension> {
        match s {
          Some(Stored::Dimension(d)) => Some(*d),
          Some(Stored::Glue(g)) => Some(Dimension::new(g.value_of())),
          Some(Stored::MuGlue(g)) => {
            // Convert mu to pt: 1mu = font_size / 18
            let fs = state::lookup_font()
              .and_then(|f| f.get_size())
              .unwrap_or(10.0);
            let muwidth = (fs * common::numeric_ops::UNITY_F64 / 18.0) as i64;
            let pt_scaled =
              (g.value_of() as f64 * muwidth as f64 / common::numeric_ops::UNITY_F64).trunc();
            Some(Dimension::new(pt_scaled as i64))
          },
          Some(Stored::MuDimension(d)) => {
            let fs = state::lookup_font()
              .and_then(|f| f.get_size())
              .unwrap_or(10.0);
            let mu_val = d.value_of() as f64;
            let pt_scaled = mu_val * fs / 18.0;
            Some(Dimension::new(pt_scaled as i64))
          },
          _ => None,
        }
      }
      Ok((
        stored_to_dim(width).unwrap_or_else(|| stored_to_dim(cached_width).unwrap_or_default()),
        stored_to_dim(height).unwrap_or_else(|| stored_to_dim(cached_height).unwrap_or_default()),
        stored_to_dim(depth).unwrap_or_else(|| stored_to_dim(cached_depth).unwrap_or_default()),
        stored_to_dim(cached_width).unwrap_or_else(|| stored_to_dim(width).unwrap_or_default()),
        stored_to_dim(cached_height).unwrap_or_else(|| stored_to_dim(height).unwrap_or_default()),
        stored_to_dim(cached_depth).unwrap_or_else(|| stored_to_dim(depth).unwrap_or_default()),
      ))
    })
  }

  /// computes and caches (via named properties) the size of a box-like object
  fn compute_size_and_cache(
    &mut self,
    mut options: HashMap<Stored>,
  ) -> Result<(Dimension, Dimension, Dimension)> {
    for key in ["width", "height", "depth", "vattach", "layout"] {
      if let Some(v) = self.get_property(key) {
        options.insert(key, v.into_owned());
      }
    }

    let (w, h, d) = self.compute_size(options)?;

    if !self.has_property("cached_width") {
      self.set_property("cached_width", w);
    }
    if !self.has_property("cached_height") {
      self.set_property("cached_height", h);
    }
    if !self.has_property("cached_depth") {
      self.set_property("cached_depth", d);
    }
    Ok((w, h, d))
  }

  /// computes and returns the size of a box-like object
  fn compute_size(&self, options: HashMap<Stored>) -> Result<(Dimension, Dimension, Dimension)>;
}

/// The current TeX processing mode
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TexMode {
  /// TeX's math mode
  Math,
  /// TeX's text mode
  Text,
}
