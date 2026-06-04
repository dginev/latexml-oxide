use core::slice::Iter;
use libxml::tree::Node;
use rustc_hash::FxHashMap as HashMap;
use std::borrow::Cow;
use std::fmt;

use super::keyval::{has_keyval, keyval_get, keyval_qname};
use crate::common::arena::SymHashMap;
use crate::common::error::*;
use crate::common::font::Font;
use crate::common::object::Object;
use crate::common::store::Stored;
use crate::definition::argument::ArgWrap;
use crate::document::Document;
use crate::gullet::{self, ExpansionLevel};
use crate::state;
use crate::token::{Catcode, Token};
use crate::tokens::Tokens;
use crate::{BoxOps, Digested, NO_PROPERTIES};

#[derive(Debug, Clone)]
struct KVData {
  key:            String,
  value:          Option<ArgWrap>,
  use_default:    bool,
  primary_keyset: String,
  keysets:        Vec<String>,
  digested_value: Option<Digested>,
}

#[allow(dead_code)] // TODO: remove when KeyVals is fully implemented
#[derive(Debug, Clone)]
pub struct KeyVals {
  // which KeyVals are we parsing and how do we behave?
  prefix:               String,
  /// `keysets should be a list of keysets to find keys inside of.
  /// It defaults to ["_anonymous_"] if empty.
  keysets:              Vec<String>,
  skip:                 Vec<String>,
  set_all:              bool,
  set_internals:        bool,
  skip_missing:         SkipMissing,
  was_digested:         bool,
  hook_missing:         Option<Token>,
  // all the internal representations
  tuples:               Vec<KVData>,
  cached_pairs:         Vec<(String, ArgWrap)>,
  cached_hash:          HashMap<String, Vec<ArgWrap>>,
  cached_hash_digested: HashMap<String, Vec<Digested>>,
}

impl Default for KeyVals {
  fn default() -> Self {
    KeyVals {
      prefix:               "KV".to_string(),
      keysets:              vec!["_anonymous_".to_string()],
      skip:                 Vec::new(),
      set_all:              false,
      set_internals:        false,
      skip_missing:         SkipMissing::None,
      was_digested:         false,
      hook_missing:         None,
      tuples:               Vec::new(),
      cached_pairs:         Vec::new(),
      cached_hash:          HashMap::default(),
      cached_hash_digested: HashMap::default(),
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
    let mut first = true;
    for (key, value) in &self.cached_pairs {
      if !first {
        // Perl uses comma without space for KeyVals serialization
        write!(f, ",")?;
      }
      write!(f, "{}={}", key, value)?;
      first = false;
    }
    Ok(())
  }
}

impl Object for KeyVals {
  fn stringify(&self) -> String { self.to_string() }

  fn be_digested(mut self) -> Result<Digested> {
    if self.was_digested {
      Info!(
        "ignore",
        "keyvals",
        "Skipping digestion of \\setkeys as requested (did you digest a KeyVals twice?) "
      );
    } else {
      crate::stomach::digest(self.set_keys_expansion())?;
    }

    // iterate over the tuples, digesting the values
    for tuple in self.tuples.iter_mut() {
      let KVData {
        key,
        value,
        primary_keyset,
        digested_value,
        ..
      } = tuple;
      if digested_value.is_none() {
        // avoid accidental repeats?
        let keytype_opt = keyval_get(&keyval_qname(&self.prefix, primary_keyset, key), "type");
        let v = if let Some(Stored::Parameter(keytype)) = keytype_opt {
          if let Some(v) = value.take() {
            keytype.digest(v, None)?
          } else {
            None
          }
        } else if let Some(v) = value.take() {
          Some(v.be_digested()?)
        } else {
          None
        };
        tuple.digested_value = v;
      }
    }
    // TODO: DG: KeyVals digestion feels very iffy while porting it over to Rust.
    // had to add an explicit "rebuild" to cache the digested values in the new
    // "cached_hash_digested" It feels like the entire object should be reorganized to leverage
    // a little more of the well-typed capabilities we have here.
    self.rebuild(None);
    self.was_digested = true;
    Ok(self.into())
  }
}

impl BoxOps for KeyVals {
  fn with_properties<R, FnR>(&self, caller: FnR) -> R
  where FnR: FnOnce(&SymHashMap<Stored>) -> R {
    caller(&NO_PROPERTIES)
  }
  fn get_string(&self) -> Result<Cow<'_, str>> { Ok(Cow::Owned(self.to_string())) }
  fn set_property<T: Into<Stored>>(&mut self, _key: &str, _value: T) {
    log::warn!("set_property on KeyVals not supported");
  }
  fn be_absorbed(&self, _document: &mut Document) -> Result<Vec<Node>> { Ok(Vec::new()) } // TODO
  fn get_font(&self) -> Result<Option<Cow<'_, Font>>> { Ok(None) } // TODO
  fn compute_size(
    &self,
    _options: SymHashMap<Stored>,
  ) -> Result<(
    crate::common::dimension::Dimension,
    crate::common::dimension::Dimension,
    crate::common::dimension::Dimension,
  )> {
    use crate::common::dimension::Dimension;
    Ok((
      Dimension::default(),
      Dimension::default(),
      Dimension::default(),
    ))
  }
}
#[derive(Debug, Clone, Default, PartialEq)]
pub enum SkipMissing {
  #[default]
  /// throw errors
  None,
  /// silently ignore all missing keys
  All,
  /// store all missing keys under the provided token
  Store(Token),
}

#[derive(Default)]
pub struct KeyvalsConfig {
  pub prefix:        Option<String>,
  pub keysets:       Vec<String>,
  pub set_all:       bool,
  pub set_internals: bool,
  pub skip:          Vec<String>,
  pub skip_missing:  SkipMissing,
  pub hook_missing:  Option<Token>,
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
  pub fn new(options: KeyvalsConfig) -> Self {
    // parse all the arguments
    let KeyvalsConfig {
      prefix,
      mut keysets,
      set_all,
      set_internals,
      skip,
      skip_missing,
      hook_missing,
    } = options;
    let prefix = prefix.unwrap_or_else(|| String::from("KV"));
    // Perl KeyVals.pm #2777 (fdc8bf91, 2026-03-27):
    // filter empty strings from the keyset list. Split("," , ",pstricks")
    // (e.g. \pst@famlist accumulates as ",pstricks") yields ["", "pstricks"];
    // the empty keyset caused keyval_qname("psset","","ArrowInside") to
    // collide with raw \def\psset@@ArrowInside (a delimited-argument helper)
    // and emit spurious "Missing argument" errors. Hardening here matches
    // the Perl fix regardless of how keysets was constructed at the call
    // site.
    keysets.retain(|k| !k.is_empty());
    if keysets.is_empty() {
      keysets = vec![String::from("_anonymous_")];
    }
    KeyVals {
      prefix,
      keysets,
      skip,
      set_all,
      set_internals,
      skip_missing,
      hook_missing,
      ..KeyVals::default()
    }
  }

  //======================================================================
  // Resolution to KeySets
  //======================================================================

  /// Return a list of the keysets in which this key is defined
  fn resolve_keyval_for(&self, key: &str) -> Vec<String> {
    let prefix = &self.prefix;
    let allkeysets = &self.keysets;
    let keysets: Vec<_> = self
      .keysets
      .iter()
      .filter(|kset| has_keyval(prefix, kset, key))
      .collect();
    // throw an error (not really), unless we record the missing macros
    // Since we're not as obsessive about declaring ALL keys, we'll soften the blow
    if keysets.is_empty() {
      if self.skip_missing == SkipMissing::None {
        // Rate-limit: only emit Info the first time this (prefix, key,
        // keysets) tuple fires. A large `tabular` with 700 rows can
        // otherwise produce 700 identical "Encountered unknown KeyVals
        // key 'vattach'" messages (arxiv 1709.05096), each allocating
        // a formatted String + going through the log backend. Perl's
        // Info() has an equivalent deduper in Error.pm via
        // maxWarnings limits; our rate-limit is per (prefix,key,keysets)
        // and unbounded in count, so the first occurrence is always
        // visible but repeats are silently dropped.
        type SeenSet = rustc_hash::FxHashSet<(String, String, String)>;
        thread_local! {
          static SEEN_MISSING: std::cell::RefCell<SeenSet> =
            std::cell::RefCell::new(SeenSet::default());
        }
        let all_joined = allkeysets.join(",");
        let is_new = SEEN_MISSING.with(|cell| {
          cell
            .borrow_mut()
            .insert((prefix.clone(), key.to_string(), all_joined.clone()))
        });
        if is_new {
          // Intentional divergence from Perl (KeyVals.pm L97 uses Info).
          // An unknown KeyVal key in `\setkeys` (non-starred) is the
          // package binding admitting it doesn't recognise an option
          // the user actually requested — the key's effect (formatting,
          // rendering options) is silently dropped. For siunitx
          // specifically this cascades into broken `\SI{}` expansion,
          // which leaves bare control sequences in math and produces
          // duplicated xml:id (witness: 1410.8171). Promoted to Warn
          // so each unique missing key surfaces as a status_code=1
          // (`[warn]` in the canvas), and a binding gap can't ship
          // green.
          Warn!(
            "undefined",
            "Encountered unknown KeyVals key",
            s!(
              "'{key}' with prefix '{prefix}' not defined in '{all_joined}', were you perhaps using \\setkeys instead of \\setkeys*?"
            )
          );
        }
      }
      return Vec::new();
    }
    // return either the first or all of the KeyVal objects
    // TODO: SymStr would avoid the allocation.
    if self.set_all {
      keysets.into_iter().cloned().collect()
    } else {
      vec![keysets[0].clone()]
    }
  }

  fn can_resolve_keyval_for(&self, key: &str) -> bool {
    // iterate over the keysets
    self
      .keysets
      .iter()
      .any(|keyset| has_keyval(&self.prefix, keyset, key))
  }

  /// Return the 1st of the keysets, or the 1st one of the KeyVals itself
  fn get_primary_keyval<'a>(&'a self, keysets: &'a [String]) -> &'a str {
    match keysets.first() {
      None => self.keysets[0].as_str(),
      Some(kset) => kset.as_str(),
    }
  }

  fn read_keyword_from(&self, close: Token) -> Result<(Tokens, Option<Token>)> {
    // set of tokens we will expand
    let mut tokens = Vec::new();
    let delim = &[close, T_OTHER!(","), T_OTHER!("=")];
    // skip leading spaces
    gullet::skip_spaces()?;

    let mut last_token = None;
    while let Some(token) = gullet::read_x_token(None, false, None)? {
      // skip to the next iteration if we have a paragraph
      if token == T_CS!("\\par") {
        continue;
      }
      // if we have one of out delimiters, we end
      if delim.contains(&token) {
        last_token = Some(token);
        break;
      }
      tokens.push(token);
    }
    // return the tokens and the last token
    Ok((Tokens::new(tokens), last_token))
  }

  //======================================================================
  // Public accessors of all the values
  //======================================================================
  // Note: The API of this need to be stable, as people may be using it

  /// return the value of a given key. If multiple values are given, return the last one.
  pub fn get_value(&self, key: &str) -> Option<&ArgWrap> {
    // Since we (by default) accumulate lists of values when repeated,
    // we need to provide the "common" thing: return the last value given.
    match self.cached_hash.get(key) {
      None => None,
      Some(value) => value.last(),
    }
  }
  /// return the digested value of a given key. If multiple values are given, return the last one.
  /// This call does *not* digest the value, and will return None if called pre-digestion
  pub fn get_value_digested(&self, key: &str) -> Option<&Digested> {
    // Since we (by default) accumulate lists of values when repeated,
    // we need to provide the "common" thing: return the last value given.
    match self.cached_hash_digested.get(key) {
      None => None,
      Some(value) => value.last(),
    }
  }

  /// return a list of values for a given key
  pub fn get_values(&self, key: &str) -> Option<&Vec<ArgWrap>> { self.cached_hash.get(key) }

  /// return the set of key-value pairs
  pub fn get_pairs(&self) -> Iter<'_, (String, ArgWrap)> { self.cached_pairs.iter() }
  /// consume KeyVals and return a flat HashMap
  pub fn as_flat_hash(self) -> HashMap<String, ArgWrap> {
    let mut flat_hash = HashMap::default();
    for (k, mut vec) in self.cached_hash {
      if let Some(v) = vec.pop() {
        flat_hash.insert(k, v);
      }
    }
    flat_hash
  }
  /// consume KeyVals and return the cached HashMap of input values
  pub fn as_hash(self) -> HashMap<String, Vec<ArgWrap>> { self.cached_hash }
  /// consume KeyVals and return the cached HashMap of digested values
  pub fn as_hash_digested(self) -> HashMap<String, Vec<Digested>> { self.cached_hash_digested }
  /// returns a key => ToString(value)
  pub fn get_hash(&self) -> HashMap<String, String> {
    let mut hashed = HashMap::default();
    for (k, v) in &self.cached_hash {
      hashed.insert(
        k.clone(),
        v.iter()
          .map(ToString::to_string)
          .collect::<Vec<String>>()
          .join(""),
      );
    }
    hashed
  }
  /// returns a key => ToString(value)
  pub fn get_hash_digested(&self) -> HashMap<String, String> {
    let mut hashed = HashMap::default();
    for (k, v) in &self.cached_hash_digested {
      hashed.insert(
        k.clone(),
        v.iter()
          .map(ToString::to_string)
          .collect::<Vec<String>>()
          .join(""),
      );
    }
    hashed
  }

  // return a hash of key-value pairs
  pub fn get_keyvals(&self) -> &HashMap<String, Vec<ArgWrap>> { &self.cached_hash }

  // checks if the value for a given key exists
  pub fn has_key(&self, key: &str) -> bool { self.cached_hash.contains_key(key) }

  //======================================================================
  // Value Related Reversion
  //======================================================================
  pub fn set_keys_expansion(&self) -> Tokens {
    let skip_keys = &self.skip;
    let set_internals = self.set_internals;
    let prefix = &self.prefix;

    // Handle skipMissing store token (xkeyval feature)
    let rmmacro = match &self.skip_missing {
      SkipMissing::Store(token) => Some(*token),
      _ => None,
    };
    let hook_missing = self.hook_missing;

    // Read existing tokens from rmmacro (if defined and has meaning)
    let mut rmtokens: Vec<Token> = Vec::new();
    if let Some(rm) = rmmacro {
      if state::has_meaning(&rm) {
        if let Ok(expanded) = gullet::do_expand(Tokens!(rm)) {
          rmtokens = expanded.unlist();
        }
      }
    }

    let mut tokens: Vec<Token> = Vec::new();

    // Define xkeyval internals if needed
    if set_internals {
      let keysets_joined = self.keysets.join(",");
      let skip_joined = self.skip.join(",");
      tokens.push(T_CS!("\\def"));
      tokens.push(T_CS!("\\XKV@fams"));
      tokens.push(T_BEGIN!());
      tokens.extend(Explode!(keysets_joined));
      tokens.push(T_END!());
      tokens.push(T_CS!("\\def"));
      tokens.push(T_CS!("\\XKV@na"));
      tokens.push(T_BEGIN!());
      tokens.extend(Explode!(skip_joined));
      tokens.push(T_END!());
    }

    // Iterate over key-value pairs
    for tuple in &self.tuples {
      let KVData {
        key,
        value,
        use_default,
        primary_keyset,
        keysets,
        ..
      } = tuple;

      // Skip keys in the skip list
      if skip_keys.iter().any(|s| s == key) {
        continue;
      }

      // If no keysets resolved for this key
      if keysets.is_empty() {
        // Store in rmmacro if defined
        if rmmacro.is_some() {
          if let Ok(rev) = self.revert_keyval(
            key,
            primary_keyset,
            value.as_ref(),
            *use_default,
            rmtokens.is_empty(),
          ) {
            rmtokens.extend(rev);
          }
        }
        // Call hookMissing if defined
        if let Some(hm) = hook_missing {
          if let Ok(rev) =
            self.revert_keyval(key, primary_keyset, value.as_ref(), *use_default, true)
          {
            tokens.push(hm);
            tokens.push(T_BEGIN!());
            tokens.extend(rev);
            tokens.push(T_END!());
          }
        }
        continue;
      }

      // Iterate over all valid keysets
      for keyset in keysets {
        let qname = keyval_qname(prefix, keyset, key);
        if !has_keyval(prefix, keyset, key) {
          Info!(
            "undefined",
            "Encountered unknown KeyVals key",
            s!("'{key}' with prefix '{prefix}' not defined in '{keyset}'")
          );
        } else if matches!(keyval_get(&qname, "disabled"), Some(Stored::Bool(true))) {
          Warn!("undefined", "keyval", s!("`{key}' has been disabled. "));
        } else {
          // Define xkeyval internals per-key if needed
          if set_internals {
            tokens.push(T_CS!("\\def"));
            tokens.push(T_CS!("\\XKV@prefix"));
            tokens.push(T_BEGIN!());
            tokens.extend(Explode!(s!("{prefix}@")));
            tokens.push(T_END!());
            tokens.push(T_CS!("\\def"));
            tokens.push(T_CS!("\\XKV@tfam"));
            tokens.push(T_BEGIN!());
            tokens.extend(Explode!(keyset));
            tokens.push(T_END!());
            tokens.push(T_CS!("\\def"));
            tokens.push(T_CS!("\\XKV@header"));
            tokens.push(T_BEGIN!());
            tokens.extend(Explode!(s!("{prefix}@{keyset}@")));
            tokens.push(T_END!());
            tokens.push(T_CS!("\\def"));
            tokens.push(T_CS!("\\XKV@tkey"));
            tokens.push(T_BEGIN!());
            tokens.extend(Explode!(key));
            tokens.push(T_END!());
          }

          // Perl: if ($useDefault) { push(@tokens, T_CS('\\' . $qname . '@default')); }
          //       else { push(@tokens, T_CS('\\' . $qname), T_BEGIN, Revert($value), T_END); }
          // Note: Perl unconditionally emits \qname@default for bare keys. In Rust, we guard
          // with has_meaning to avoid undefined-CS errors when @default was never registered
          // (e.g., xkeyval DeclareOptionX keys without default values).
          if *use_default && state::has_meaning(&T_CS!(s!("\\{qname}@default"))) {
            // Call the @default macro (bare key with registered default)
            tokens.push(T_CS!(s!("\\{qname}@default")));
          } else {
            // Call the macro with the value (or empty if bare key without default)
            tokens.push(T_CS!(s!("\\{qname}")));
            tokens.push(T_BEGIN!());
            if let Some(v) = value {
              if let Ok(reverted) = v.revert() {
                tokens.extend(reverted.unlist());
              }
            }
            tokens.push(T_END!());
          }

          // Reset xkeyval internals per-key
          if set_internals {
            tokens.push(T_CS!("\\def"));
            tokens.push(T_CS!("\\XKV@prefix"));
            tokens.push(T_BEGIN!());
            tokens.push(T_END!());
            tokens.push(T_CS!("\\def"));
            tokens.push(T_CS!("\\XKV@tfam"));
            tokens.push(T_BEGIN!());
            tokens.push(T_END!());
            tokens.push(T_CS!("\\def"));
            tokens.push(T_CS!("\\XKV@header"));
            tokens.push(T_BEGIN!());
            tokens.push(T_END!());
            tokens.push(T_CS!("\\def"));
            tokens.push(T_CS!("\\XKV@tkey"));
            tokens.push(T_BEGIN!());
            tokens.push(T_END!());
          }
        }
      }
    }

    // Assign rmmacro with collected missing keys
    if let Some(rm) = rmmacro {
      tokens.push(T_CS!("\\def"));
      tokens.push(rm);
      tokens.push(T_BEGIN!());
      tokens.extend(rmtokens);
      tokens.push(T_END!());
    }

    // Reset all internals if applicable
    if set_internals {
      tokens.push(T_CS!("\\def"));
      tokens.push(T_CS!("\\XKV@fams"));
      tokens.push(T_BEGIN!());
      tokens.push(T_END!());
      tokens.push(T_CS!("\\def"));
      tokens.push(T_CS!("\\XKV@na"));
      tokens.push(T_BEGIN!());
      tokens.push(T_END!());
    }

    Tokens::new(tokens)
  }

  pub fn revert(&self) -> Result<Tokens> {
    let mut tokens = Vec::new();
    // iterate over the key-value pairs
    for tuple in &self.tuples {
      let KVData {
        key,
        value,
        use_default,
        keysets: _,
        primary_keyset,
        digested_value: _,
      } = tuple;
      if !primary_keyset.is_empty() {
        let reverted = self.revert_keyval(
          key,
          primary_keyset,
          value.as_ref(),
          *use_default,
          tokens.is_empty(),
        )?;
        tokens.extend(reverted);
      }
    }
    // and return the list of tokens
    Ok(Tokens::new(tokens))
  }

  fn revert_keyval(
    &self,
    key: &str,
    keyset: &str,
    value_opt: Option<&ArgWrap>,
    use_default: bool,
    is_first: bool,
  ) -> Result<Vec<Token>> {
    // get the key-value definition
    let keytype_stored = keyval_get(&keyval_qname(&self.prefix, keyset, key), "type");
    // define the tokens
    let mut tokens = Vec::new();
    // write comma and key, unless in the first iteration
    if !is_first {
      tokens.push(T_OTHER!(","));
    }
    tokens.extend(Explode!(key));
    // write the default (if applicable)
    if !use_default {
      if let Some(value) = value_opt {
        tokens.push(T_OTHER!("="));
        if let Some(Stored::Parameter(keytype)) = keytype_stored {
          // TODO: The types here are a little curious. The stored value must be cast back into
          // Tokens if Parameter's revert works on Tokens. Or should that revert call work on
          // ArgWrap?
          if let Some(reverted) = keytype.revert(Some(value.revert()?))? {
            tokens.extend(reverted.unlist());
          }
        } else {
          tokens.extend(value.revert()?.unlist());
        }
      }
    }
    Ok(tokens)
  }

  //======================================================================
  // Changing contained values
  //======================================================================

  pub fn add_value(
    &mut self,
    key: &str,
    value_arg: ArgWrap,
    use_default: bool,
    no_rebuild: bool,
  ) -> Result<()> {
    // figure out the keyset(s) for the key to be added
    let keysets = self.resolve_keyval_for(key);
    let primary_keyset = self.get_primary_keyval(keysets.as_slice()).to_owned();

    // and add the new tuple to the set of tuples
    let value = if use_default {
      match keyval_get(&keyval_qname(&self.prefix, &primary_keyset, key), "default") {
        None => Some(ArgWrap::Tokens(Tokens!())), // bare key with no default: empty value
        Some(v) => {
          let arg: Result<ArgWrap> = v.into();
          Some(arg?)
        },
      }
    } else {
      Some(value_arg)
    };
    self.tuples.push(KVData {
      key: key.to_string(),
      value,
      use_default,
      keysets,
      primary_keyset,
      digested_value: None,
    });
    // we now need to rebuild, unless we were asked not to
    // TODO: Maybe only update the last element?
    if !no_rebuild {
      self.rebuild(None);
    }
    Ok(())
  }

  pub fn set_value(&mut self, key: &str, value: ArgWrap, use_default: bool) -> Result<()> {
    // delete the existing values by skipping key
    self.rebuild(Some(key));
    // Perl: if (ref $value eq 'ARRAY') { foreach ... addValue(..., 1) } rebuild()
    //       elsif (defined($value)) { addValue($key, $value, $useDefault) }
    //       else { just delete (already done by rebuild above) }
    match &value {
      ArgWrap::None => {
        // undef — just delete (already done by rebuild above)
        Ok(())
      },
      _ => {
        // single value — set normally
        self.add_value(key, value, use_default, false)
      },
    }
  }

  fn rebuild(&mut self, skip_opt: Option<&str>) {
    // the new data structures to create
    let mut newtuples: Vec<KVData> = Vec::new();
    let mut pairs = Vec::new();
    let mut hash: HashMap<String, Vec<ArgWrap>> = HashMap::default();
    let mut hash_digested: HashMap<String, Vec<Digested>> = HashMap::default();

    for tuple in self.tuples.drain(..) {
      // take all the elements we need from the stack
      let KVData {
        key,
        value,
        use_default,
        primary_keyset,
        keysets,
        digested_value,
      } = tuple;
      // if we want to skip some values, we need to store new tuples
      let key_str = key.as_str();
      if let Some(skip) = skip_opt {
        if skip == key_str {
          continue;
        }
      }
      if let Some(v) = value.as_ref() {
        // push key / value into the pair
        pairs.push((key.clone(), v.clone()));

        // we always use Vec<ArgWrap> storage, just push the new value in
        let entry = hash.entry(key.clone()).or_default();
        entry.push(v.clone());
      } else if let Some(ref dv) = digested_value {
        // After digestion, value is taken but digested_value is set.
        // Populate cached_pairs from the digested value (matching Perl's rebuild behavior).
        let fallback = ArgWrap::Tokens(dv.revert().unwrap_or_default());
        pairs.push((key.clone(), fallback.clone()));
        let entry = hash.entry(key.clone()).or_default();
        entry.push(fallback);
      }
      // if we have a digested value, push that in the Vec<Digested> hash storage
      if let Some(ref dvalue) = digested_value {
        let entry = hash_digested.entry(key.clone()).or_default();
        entry.push(dvalue.clone());
      }

      // Record.
      newtuples.push(KVData {
        key,
        value,
        use_default,
        primary_keyset,
        keysets,
        digested_value,
      });
    }
    // store all of the values
    self.cached_pairs = pairs;
    self.cached_hash = hash;
    self.cached_hash_digested = hash_digested;
    self.tuples = newtuples;
  }

  //======================================================================
  // parsing values from a gullet
  //======================================================================

  // A KeyVal argument MUST be delimited by either braces or brackets (if optional)
  // This method reads the keyval pairs INCLUDING the delimiters, (rather than
  // parsing after the fact), since some values may have special catcode needs.

  pub fn read_from(&mut self, until: Token, silence_missing: bool) -> Result<()> {
    // if we want to force skip_missing keys, we set it up here
    let skip_missing = self.skip_missing.clone();
    let hook_missing = self.hook_missing;
    // if we want to silence all missing errors, store them in a hook
    if silence_missing {
      self.skip_missing = SkipMissing::All;
      self.hook_missing = None;
    }

    // read the opening token and figure out where we are
    let startloc = gullet::get_locator();
    // set and read tokens
    let _open = gullet::read_token()?;

    let punct_tks = Tokens!(T_OTHER!(","));
    let until_tks = Tokens!(until);
    // iterate over all the key-value pairs to read
    loop {
      // gobble leading spaces
      gullet::skip_spaces()?;
      if gullet::if_next(T_BEGIN!())? {
        // Protect against redundant {} wrapping
        gullet::read_token()?;
        gullet::unread(gullet::read_balanced(ExpansionLevel::Off, false, false)?.strip_braces());
        gullet::skip_spaces()?;
      }
      // Read a single keyword, get a delimiter and a set of keyword tokens
      let (ktoks, mut delim_opt) = self.read_keyword_from(until)?;

      // if there was no delimiter at the end, we throw an error
      if delim_opt.is_none() {
        let message = s!(
          "Fell off end expecting {} while reading KeyVal key",
          until.stringify()
        );
        let message2 = s!("key started at {}", startloc.to_string());
        Error!("expected", until, message, message2);
      }

      // turn the key tokens into a string and trim whitespace
      let key_str = ktoks.to_string();
      let key = key_str.trim();

      // if we have a non-empty key
      if !key.is_empty() {
        let mut value = ArgWrap::None;
        // if we have an '=', we explcity assign a value
        let is_explicit = delim_opt == Some(T_OTHER!("="));
        if is_explicit {
          // setup the key-codes to properly read
          let resolved_kv = self.resolve_keyval_for(key);
          let keyset = self.get_primary_keyval(&resolved_kv);
          let keytype_opt = keyval_get(&keyval_qname(&self.prefix, keyset, key), "type");
          if let Some(Stored::Parameter(ref keytype)) = keytype_opt {
            keytype.setup_catcodes();
          }
          // read until comma
          let mut toks = Vec::new();
          loop {
            // TODO: The types are a bit unnatural here - we need the plural Tokens for read_match,
            //       but we expect the singular Token as a delimiter result, since we are matching
            // on a char separator
            delim_opt = gullet::read_match(&[&punct_tks, &until_tks])?.map(|tks| tks.into());
            if delim_opt.is_some() {
              break; // only until we hit a delim.
            }
            if let Some(tok) = gullet::read_token()? {
              // Copy next token to args
              toks.push(tok);
              if tok.get_catcode() == Catcode::BEGIN {
                let balanced_arg = gullet::read_balanced(ExpansionLevel::Off, false, false)?;
                if !balanced_arg.is_empty() {
                  toks.extend(balanced_arg.unlist());
                }
                toks.push(T_END!());
              }
            } else {
              break;
            }
          }
          // reparse (and expand) the tokens representing the value
          if !toks.is_empty() {
            let stripped_toks = Tokens::new(toks).strip_braces_n(2);
            if !stripped_toks.is_empty() {
              if let Some(Stored::Parameter(ref keytype)) = keytype_opt {
                value = keytype.reparse(stripped_toks)?;
              } else {
                value = ArgWrap::Tokens(stripped_toks);
              }
            }
          }
          // An explicit `=` always assigns a value, even when it is empty
          // (`key=` or `key={}`): that is an EXPLICIT empty override, distinct
          // from a missing key. Keep it as empty Tokens rather than the
          // `ArgWrap::None` the value was initialised to — `None` is reserved
          // for a missing key and its Display is the literal string "None",
          // which leaks into consumers that stringify the value. Concretely, a
          // starred matrix with no alignment bracket emits `alignment=` (empty);
          // without this the keyval value was "None", so `\lx@gen@matrix@bindings`
          // saw `alignment="None"` instead of defaulting to "c", producing a
          // malformed column alignment that made a `\dots` cell swallow the next
          // `&` → "Stray alignment". Witness 1910.00678.
          if value.is_none() {
            value = ArgWrap::Tokens(Tokens!());
          }
          // and cleanup
          if let Some(Stored::Parameter(ref keydef)) = keytype_opt {
            keydef.revert_catcodes()?;
          }
        }
        // and store our value please
        if !silence_missing || self.can_resolve_keyval_for(key) {
          self.add_value(key, value, !is_explicit, false)?;
        }
      }

      // we finish if we have the last element
      if delim_opt.as_ref() == Some(&until) {
        break;
      }
    }

    // rebuild and return nothing
    self.rebuild(None);

    // restore all settings if we silenced the missing keys
    if silence_missing {
      self.skip_missing = skip_missing;
      self.hook_missing = hook_missing;
    }
    Ok(())
  }

  /// TODO: This is an improvised method for switching KeyVals into Tokens, but losing all collected
  /// metadata.
  /// The long-term solution ought to be via a type system extension, where the
  /// arguments to our before-digest closures are a vector of a new type
  /// ReadValue ::= [Token, KeyVals, RegisterValue]       potentially?
  /// On the other hand, we can also put the
  /// extra effort of *postponing* the build of KV metadata until digestion,
  /// this way not losing any time reserializing metadata
  pub fn into_tokens(self) -> Result<Tokens> {
    let mut tks: Vec<Token> = Vec::new();
    for (k, v) in self.cached_pairs.into_iter() {
      tks.push(T_OTHER!(k));
      match v {
        ArgWrap::Tokens(vtks) => {
          let expanded = gullet::do_expand(vtks)?;
          let mut exp_str = expanded.to_string();
          if exp_str == "{}" {
            exp_str = String::new();
          }
          tks.push(T_OTHER!(exp_str));
        },
        ArgWrap::Token(vtk) => tks.push(vtk),
        other => {
          log::warn!("Unexpected ArgWrap variant in KeyVals revert: {:?}", other);
        },
      }
    }
    Ok(Tokens::new(tks))
  }
}

impl From<KeyVals> for Result<Option<Digested>> {
  fn from(value: KeyVals) -> Result<Option<Digested>> {
    let tmp: Digested = value.into();
    tmp.into()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn skip_missing_default_is_none() {
    let s = SkipMissing::default();
    assert_eq!(s, SkipMissing::None);
  }

  #[test]
  fn skip_missing_variants_not_equal() {
    assert_ne!(SkipMissing::None, SkipMissing::All);
  }

  #[test]
  fn keyvals_config_default_all_empty() {
    let c = KeyvalsConfig::default();
    assert!(c.prefix.is_none());
    assert!(c.keysets.is_empty());
    assert!(!c.set_all);
    assert!(!c.set_internals);
    assert!(c.skip.is_empty());
    assert_eq!(c.skip_missing, SkipMissing::None);
    assert!(c.hook_missing.is_none());
  }

  #[test]
  fn keyvals_default_prefix_and_anonymous_keyset() {
    // Default KeyVals has prefix=KV, keysets=["_anonymous_"].
    let kv = KeyVals::default();
    assert_eq!(kv.prefix, "KV");
    assert_eq!(kv.keysets, vec!["_anonymous_".to_string()]);
    assert!(!kv.set_all);
    assert!(!kv.set_internals);
  }

  #[test]
  fn keyvals_new_with_empty_keysets_defaults_to_anonymous() {
    let kv = KeyVals::new(KeyvalsConfig::default());
    assert_eq!(kv.keysets, vec!["_anonymous_".to_string()]);
  }

  #[test]
  fn keyvals_new_with_custom_keysets_preserved() {
    let cfg = KeyvalsConfig {
      keysets: vec!["tabular".to_string(), "array".to_string()],
      ..KeyvalsConfig::default()
    };
    let kv = KeyVals::new(cfg);
    assert_eq!(kv.keysets.len(), 2);
    assert_eq!(kv.keysets[0], "tabular");
  }

  #[test]
  fn keyvals_new_custom_prefix() {
    let cfg = KeyvalsConfig {
      prefix: Some("P".to_string()),
      ..KeyvalsConfig::default()
    };
    let kv = KeyVals::new(cfg);
    assert_eq!(kv.prefix, "P");
  }

  #[test]
  fn keyvals_new_default_prefix_on_none() {
    let cfg = KeyvalsConfig {
      prefix: None,
      ..KeyvalsConfig::default()
    };
    let kv = KeyVals::new(cfg);
    assert_eq!(kv.prefix, "KV");
  }

  #[test]
  fn keyvals_new_set_all_flag() {
    let cfg = KeyvalsConfig {
      set_all: true,
      ..KeyvalsConfig::default()
    };
    let kv = KeyVals::new(cfg);
    assert!(kv.set_all);
  }

  #[test]
  fn keyvals_new_filters_empty_keysets() {
    // Perl KeyVals.pm #2777 (fdc8bf91): \pst@famlist accumulates as
    // ",pstricks"; a naive split yields ["", "pstricks"]. The empty
    // entry would collide with `\def\psset@@ArrowInside` via the
    // keyval_qname("psset","","ArrowInside") → "psset@@ArrowInside"
    // path. Empty entries must be filtered before any default fallback.
    let cfg = KeyvalsConfig {
      keysets: vec!["".to_string(), "pstricks".to_string()],
      ..KeyvalsConfig::default()
    };
    let kv = KeyVals::new(cfg);
    assert_eq!(kv.keysets, vec!["pstricks".to_string()]);
  }

  #[test]
  fn keyvals_new_all_empty_keysets_defaults_to_anonymous() {
    // If every keyset entry is empty, we still fall back to
    // _anonymous_ (not retain an empty keyset).
    let cfg = KeyvalsConfig {
      keysets: vec!["".to_string(), "".to_string()],
      ..KeyvalsConfig::default()
    };
    let kv = KeyVals::new(cfg);
    assert_eq!(kv.keysets, vec!["_anonymous_".to_string()]);
  }
}
