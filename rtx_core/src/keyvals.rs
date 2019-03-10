use std::borrow::Cow;
use std::fmt;
use std::collections::HashMap;
// use std::rc::Rc;

use crate::common::error::*;
use crate::common::font::Font;
use crate::common::locator::Locator;
use crate::common::store::Stored;
use crate::common::object::Object;
// use crate::definition::expandable::Expandable;
// use crate::definition::Definition;
use crate::document::Document;
// use crate::list::List;
use crate::keyval::KeyVal;
use crate::state::State;
use crate::token::Token;
// use crate::tokens::Tokens;
use crate::{BoxOps, Digested};

type KVTuple = (String, Stored, bool, Vec<KeyVal>, KeyVal);

#[derive(Debug, Clone)]
pub struct KeyVals {
  // which KeyVals are we parsing and how do we behave?
  prefix: String,
  keysets: Vec<String>,
  skip: Vec<String>,
  set_all: bool,
  set_internals: bool,
  skip_missing: bool,
  hook_missing: Option<Token>,
  // all the internal representations
  tuples: Vec<KVTuple>,
  cached_pairs: Vec<(String, Stored)>,
  cached_hash: HashMap<String, Vec<Stored>>,
  // all the character tokens we used
  punct: Vec<char>,
  assign: Vec<char>,
}

impl Default for KeyVals {
  fn default() -> Self {
    KeyVals {
      prefix: "KV".to_string(),
      keysets: vec!["_anonymous_".to_string()],
      skip: Vec::new(),
      set_all: false,
      set_internals: false,
      skip_missing: false,
      hook_missing: None,
      tuples: Vec::new(),
      cached_pairs: Vec::new(),
      cached_hash: HashMap::new(),
      punct: Vec::new(),
      assign: Vec::new(),
    }
  }
}

impl PartialEq for KeyVals {
  fn eq(&self, _other: &KeyVals) -> bool {
    false // TODO ?
  }
}

impl fmt::Display for KeyVals {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    unimplemented!();
  }
}

impl Object for KeyVals {
  fn get_locator(&self) -> Cow<Locator> {
    unimplemented!(); 
  }
  fn stringify(&self) -> String { unimplemented!(); }
}
impl BoxOps for KeyVals {
  fn get_properties_mut(&mut self) -> &mut HashMap<String, Stored> { unimplemented!() }
  fn unlist(&self) -> Vec<Digested> { Vec::new() } // TODO
  fn be_absorbed(&self, document: &mut Document, state: &mut State) -> Result<()> { Ok(()) } // TODO
  fn get_font(&self) -> Option<Cow<Font>> { None } // TODO
}


impl KeyVals {
  ///======================================================================
  /// The KeyVals constructor
  ///======================================================================
  /// This defines the KeyVals data object that can appear in the datastream
  /// along with tokens, boxes, etc.
  /// Thus it has to be digestible, however we may not want to digest it more
  /// than once.
  ///**********************************************************************
  pub fn new(prefix_opt: Option<String>, keysets: Option<Vec<String>>, options: HashMap<String, bool>, state: &State) -> Self {
    // parse all the arguments
    let prefix = prefix_opt.unwrap_or_else(|| String::from("KV"));
    // $keysets = [split(',', ToString(defined($keysets) ? $keysets : '_anonymous_'))] unless (ref($keysets) eq 'ARRAY');
    // let skip = options.get("skip").unwrap_or(false);
    // $skip = [split(',', ToString(defined($options{skip}) ? $options{skip} : ''))] unless (ref($options{skip}) eq 'ARRAY');
    // my $setAll       = $options{setAll}       ? 1 : 0;
    // my $setInternals = $options{setInternals} ? 1 : 0;
    // my $skipMissing  = $options{skipMissing};
    // my $hookMissing  = $options{hookMissing};
    // // hook missing, if defined, must be a token
    // if (defined($hookMissing) && $hookMissing) {
    //   $hookMissing = ref($hookMissing) ? $hookMissing : T_CS(ToString($hookMissing)); }
    // else { $hookMissing = undef; }
    // // skip missing may be a token (=store all the missing macros there)
    // unless (ref($skipMissing)) {
    //   // may be undef or 0 (= throw errors)
    //   unless (defined($skipMissing)) { $skipMissing = undef; }
    //   elsif  ($skipMissing eq '0')   { $skipMissing = undef; }
    //   // may be 1 (= ignore all missing keys)
    //   elsif ($skipMissing eq '1') { $skipMissing = 1; }
    //   // may be a string (= store all the missing keys there)
    //   else { $skipMissing = T_CS($skipMissing); } }
    // my %hash = ();
    KeyVals {
      prefix,
      ..KeyVals::default()
    }
    // keysets     => $keysets,
    // skip        => $skip,        setAll      => $setAll, setInternals => $setInternals,
    // skipMissing => $skipMissing, hookMissing => $hookMissing,
    // // all the internal representations
    // tuples => [], cachedPairs => [()], cachedHash => \%hash,
    // // all the character tokens we used
    // punct => $options{punct}, assign => $options{assign} },
  }

  //======================================================================
  // Resolution to KeySets
  //======================================================================
  fn resolve_key_val_for(&self, key: &str) -> Vec<KeyVal> {
    // my $prefix  = $self->getPrefix;
    // my @keysets = $self->getKeySets;
    // let sets = Vec::new();

    // // iterate over the keysets
    // foreach my $keyset (@keysets) {
    //   my $bkeyval = LaTeXML::Core::KeyVal->new($prefix, $keyset, $key);
    //   push(@sets, $bkeyval) if $bkeyval->isDefined(1); }

    // // throw an error, unless we record the missing macros
    // if (scalar @sets == 0) {
    //   Error(
    //     'undefined', 'Encountered unknown KeyVals key',
    //     "'$key' with prefix '$prefix' not defined in '" . join(",", @keysets) . "', " .
    //       'were you perhaps using \setkeys instead of \setkeys*?') unless defined($self->getSkipMissing);
    //   return; }

    // // return either the first or all of the elements
    // return ($sets[0]) unless $self->getSetAll;
    Vec::new()
  }

  fn get_primary_key_val_of(&self, key: &str, keysets: &[KeyVal]) -> KeyVal {
    if keysets.is_empty() {
      KeyVal::new(Some(self.prefix.clone()), self.keysets[0].clone(), key.to_string())
    } else {
      keysets[0].clone()
    }
  }

  //======================================================================
  // Public accessors of all the values
  //======================================================================
  // Note: The API of this need to be stable, as people may be using it

  /// return the value of a given key. If multiple values are given, return the last one.
  pub fn get_value(&self, key: &str) -> Option<&Stored> {
    // Since we (by default) accumulate lists of values when repeated,
    // we need to provide the "common" thing: return the last value given.
    match self.cached_hash.get(key) {
      None => None,
      Some(value) => value.last(),
    }
  }

  //======================================================================
  // Changing contained values
  //======================================================================

  pub fn add_value(&mut self, key: &str, value: Stored, use_default: bool, no_rebuild: bool, state: &State) {
    // figure out the keyset(s) for the key to be added
    let keysets = self.resolve_key_val_for(key);
    let headset = self.get_primary_key_val_of(key, &keysets);

    // and add the new tuple to the set of tuples
    let value = if use_default {
      headset.get_default(state).unwrap_or_else(|| Stored::String(String::new()))
    } else {
      value
    };
    self.tuples.push((key.to_string(), value, use_default, keysets, headset));
    // we now need to rebuild, unless we were asked not to
    // TODO: Maybe only update the last element?
    if !no_rebuild {
      self.rebuild(None);
    }
  }

  pub fn set_value(&mut self, key: &str, value: Stored, use_default: bool, state: &State) {
    // delete the existing values by skipping key
    self.rebuild(Some(key));
    // set normally
    self.add_value(key, value, use_default, false, state);
  }

  fn rebuild(&mut self, skip_opt: Option<&str>) {
    // the new data structures to create
    let mut newtuples: Vec<KVTuple> = Vec::new();
    let mut pairs = Vec::new();
    let mut hash: HashMap<String, Vec<Stored>> = HashMap::new();

    for tuple in &self.tuples {
      // take all the elements we need from the stack
      let (key, value, use_default, resolution, keyval) = tuple;
      // if we want to skip some values, we need to store new tuples
      if let Some(skip) = skip_opt {
        if key == skip {
          continue;
        }
        newtuples.push((key.to_string(), value.clone(), *use_default, resolution.to_vec(), keyval.clone()));
      }
      // push key / value into the pair
      pairs.push((key.to_string(), value.clone()));

      // if we do not have a value yet, set it
      let mut entry = hash.entry(key.to_string()).or_insert_with(Vec::new);

      // If we get a third value, push into an array
      // This is unlikely to be what the caller expects!! But what else?
      entry.push(value.clone());
    }

    // store all of the values
    self.cached_pairs = pairs;
    self.cached_hash = hash;
    if skip_opt.is_some() {
      self.tuples = newtuples;
    }
  }
}

impl From<KeyVals> for Result<Option<Digested>> {
  fn from(value: KeyVals) -> Result<Option<Digested>> {
    let tmp: Digested = value.into();
    tmp.into()
  }
}
