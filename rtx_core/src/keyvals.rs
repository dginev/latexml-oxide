use core::slice::Iter;
use libxml::tree::Node;
use rustc_hash::FxHashMap as HashMap;
use std::borrow::Cow;
use std::fmt;

use crate::common::arena::SymHashMap;
use crate::common::error::*;
use crate::common::font::Font;
use crate::common::object::Object;
use crate::common::store::Stored;
use crate::gullet;
use crate::definition::argument::ArgWrap;
use crate::document::Document;
use crate::token::{Catcode, Token};
use crate::tokens::Tokens;
use crate::{BoxOps, Digested, NO_PROPERTIES};
use super::keyval::{keyval_qname, keyval_get,has_keyval};

#[derive(Debug,Clone)]
struct KVData {
  key: String,
  value: Option<ArgWrap>,
  use_default: bool,
  primary_keyset: String,
  keysets: Vec<String>,
  digested_value: Option<Digested>  
}

#[allow(dead_code)] // TODO: remove when KeyVals is fully implemented
#[derive(Debug, Clone)]
pub struct KeyVals {
  // which KeyVals are we parsing and how do we behave?
  prefix: String,
  /// `keysets should be a list of keysets to find keys inside of.
  /// It defaults to ["_anonymous_"] if empty.
  keysets: Vec<String>,
  skip: Vec<String>,
  set_all: bool,
  set_internals: bool,
  skip_missing: bool,
  was_digested: bool,
  hook_missing: Option<Token>,
  // all the internal representations
  tuples: Vec<KVData>,
  cached_pairs: Vec<(String, ArgWrap)>,
  cached_hash: HashMap<String, Vec<ArgWrap>>,
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
      was_digested: false,
      hook_missing: None,
      tuples: Vec::new(),
      cached_pairs: Vec::new(),
      cached_hash: HashMap::default(),
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
  fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
    todo!();
  }
}

impl Object for KeyVals {
  fn stringify(&self) -> String { "KeyVals:TODO".to_string() }

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
      let KVData {key, value, primary_keyset, digested_value, ..} = tuple;
      if digested_value.is_none() {// avoid accidental repeats?
        let keytype_opt = keyval_get(&keyval_qname(&self.prefix, primary_keyset, key), "type");
        let v       = if let Some(Stored::Parameter(keytype)) = keytype_opt {
          if let Some(v) = value.take() {
            keytype.digest(v, None)?
          } else { None }
        } else if let Some(v) = value.take() {
          Some(v.be_digested()?)
        } else { None };
        tuple.digested_value = v;
      }
    }   
    self.was_digested = true;
    Ok(self.into())
  }
}

impl BoxOps for KeyVals {
  fn with_properties<R, FnR>(&self, caller: FnR) -> R
  where FnR: FnOnce(&SymHashMap<Stored>) -> R {
    caller(&NO_PROPERTIES)
  }
  fn get_string(&self) -> Result<Cow<str>> { Ok(Cow::Owned(self.to_string())) }
  fn set_property<T: Into<Stored>>(&mut self, _key: &str, _value: T) {
    todo!();
  }
  fn be_absorbed(&self, _document: &mut Document) -> Result<Vec<Node>> {
    Ok(Vec::new())
  } // TODO
  fn get_font(&self) -> Result<Option<Cow<Font>>> { Ok(None) } // TODO
  fn compute_size(
    &self,
    _options: SymHashMap<Stored>,
  ) -> Result<(
    crate::common::dimension::Dimension,
    crate::common::dimension::Dimension,
    crate::common::dimension::Dimension,
  )> {
    todo!() // TODO
  }
}

#[derive(Default)]
pub struct KeyvalsConfig {
  pub prefix: Option<String>,
  pub keysets: Vec<String>,
  pub set_all: bool,
  pub set_internals: bool,
  pub skip: bool,
  pub skip_missing: bool,
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
    let prefix = options.prefix.unwrap_or_else(|| String::from("KV"));
    let keysets = if options.keysets.is_empty() {
      vec![String::from("_anonymous_")]
    } else {
      options.keysets
    };
    // $skip = [split(',', ToString(defined($options{skip}) ? $options{skip} : ''))] unless
    // (ref($options{skip}) eq 'ARRAY'); my $set_all       = $options{set_all}       ? 1 : 0;
    // my $set_internals = $options{set_internals} ? 1 : 0;
    // my $skip_missing  = $options{skip_missing};
    // my $hookMissing  = $options{hookMissing};
    // // hook missing, if defined, must be a token
    // if (defined($hookMissing) && $hookMissing) {
    //   $hookMissing = ref($hookMissing) ? $hookMissing : T_CS(ToString($hookMissing)); }
    // else { $hookMissing = undef; }
    // // skip missing may be a token (=store all the missing macros there)
    // unless (ref($skip_missing)) {
    //   // may be undef or 0 (= throw errors)
    //   unless (defined($skip_missing)) { $skip_missing = undef; }
    //   elsif  ($skip_missing eq '0')   { $skip_missing = undef; }
    //   // may be 1 (= ignore all missing keys)
    //   elsif ($skip_missing eq '1') { $skip_missing = 1; }
    //   // may be a string (= store all the missing keys there)
    //   else { $skip_missing = T_CS($skip_missing); } }
    // my %hash = ();
    KeyVals {
      prefix,
      keysets,
      ..KeyVals::default()
    }
    // skip        => $skip,        set_all      => $set_all, set_internals => $set_internals,
    // skip_missing => $skip_missing, hookMissing => $hookMissing,
    // // all the internal representations
    // tuples => [], cachedPairs => [()], cachedHash => \%hash,
    // // all the character tokens we used
    // punct => $options{punct}, assign => $options{assign} },
  }

  //======================================================================
  // Resolution to KeySets
  //======================================================================
  
  /// Return a list of the keysets in which this key is defined
  fn resolve_keyval_for(&self, key: &str) -> Vec<String> {
    let prefix     = &self.prefix;
    let allkeysets = &self.keysets;
    let keysets : Vec<_> = self.keysets.iter().filter(|kset| has_keyval(prefix, kset, key)).collect();
    // throw an error (not really), unless we record the missing macros
    // Since we're not as obsessive about declaring ALL keys, we'll soften the blow
    if keysets.is_empty() {
      let all_joined = allkeysets.join(",");
      if !self.skip_missing {
        Info!("undefined", "Encountered unknown KeyVals key",
          s!("'{key}' with prefix '{prefix}' not defined in '{all_joined}', were you perhaps using \\setkeys instead of \\setkeys*?"));
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

  fn can_resolve_keyval_for(&self, key:&str) -> bool {
    // iterate over the keysets
    self.keysets.iter().any(|keyset| has_keyval(&self.prefix, keyset, key) )
  }

  /// Return the 1st of the keysets, or the 1st one of the KeyVals itself
  fn get_primary_keyval<'a>(&'a self, keysets: &'a [String]) -> &'a str {
    match keysets.first() {
      None => self.keysets[0].as_str(),
      Some(kset) => kset.as_str()
    }
  }

  fn read_keyword_from(
    &self,
    close: Token,
  ) -> Result<(Tokens, Option<Token>)> {
    // set of tokens we will expand
    let mut tokens = Vec::new();
    let delim = &[close, T_OTHER!(","), T_OTHER!("=")];
    // skip leading spaces
    gullet::skip_spaces()?;
    
    let mut last_token = None;
    while let Some(token) = gullet::read_x_token(None, false)? {
      // skip to the next iteration if we have a paragraph
      if token == T_CS!("\\par") {
        continue;
      }
      // if we have one of out delimiters, we end
      if delim.iter().any(|d| token == *d) {
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
  /// consume KeyVals and return the cached HashMap
  pub fn as_hash(self) -> HashMap<String, Vec<ArgWrap>> { self.cached_hash }
  /// returns a key => ToString(value)
  pub fn get_hash(&self) -> HashMap<String, String> {
    let mut hashed = HashMap::default();
    for (k, v) in &self.cached_hash {
      hashed.insert(
        k.to_string(),
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
  fn set_keys_expansion(&mut self) -> Tokens {
    // let skip         = self.skip;
    // let setInternals = $self->getSetInternals;

    // my ($punct, $assign) = ($$self{punct}, $$self{assign});

    // // we might have to store values in a seperate token
    // let rmmacro     = $self->getSkipMissing;
    // let hookMissing = $self->getHookMissing;
    // let definedrm   = ref($rmmacro) ? 1 : 0;
    // let rmtokens    = ();

    // // read in existing tokens (if they are defined)
    // if ($definedrm && $state->lookupMeaning($rmmacro)) {
    //   @rmtokens = LaTeXML::Package::Expand($rmmacro)->unlist; }

    // define some xkeyval internals
    let tokens = Vec::new();
    // let tokens = $setInternals ? (
    //   T_CS('\def'), T_CS('\XKV@fams'), T_BEGIN, Explode(join(',', $self->getKeySets)), T_END,
    //   T_CS('\def'), T_CS('\XKV@na'), T_BEGIN, Explode(join(',', @skip)), T_END
    // ) : ();

    // // iterate over the key-value pairs
    // for tuple in &self.tuples {
    //   let (key, value, useDefault, keyvals, keyval) = tuple;

    //   // we might want to skip to the next iteration if key is to be omitted
    //   next if (grep { $_ eq $key } @skip);

    //   // we might need to save the macros that weren't saved
    //   if (scalar @keyvals == 0) {
    //     if ($definedrm) {
    //       push(@rmtokens, $self->revertKeyVal($keyval, $value, $useDefault, (@rmtokens ? 0 : 1),
    //           1, $punct, $assign)); }
    //     my @reversion = $self->revertKeyVal($keyval, $value, $useDefault, 1, 1, $punct, $assign);
    //     push(@tokens, $hookMissing, T_BEGIN, $self->revertKeyVal($keyval, $value, $useDefault, 1,
    // 1, $punct, $assign), T_END) if $hookMissing;     next; }

    //   // and iterate over all valid keysets
    //   foreach my $keyset (@keyvals) {
    //     my $expansion = $keyset->setKeysExpansion($value, $useDefault, 1, 1, $setInternals);
    //     next unless defined($expansion);
    //     push(@tokens, $expansion->unlist); } }

    // // and assign the macro with the other keys
    // push(@tokens, T_CS('\def'), $rmmacro, T_BEGIN, @rmtokens, T_END) if $definedrm;

    // // reset all the internals (if applicable)
    // push(@tokens,
    //   T_CS('\def'), T_CS('\XKV@fams'), T_BEGIN, T_END,
    //   T_CS('\def'), T_CS('\XKV@na'), T_BEGIN, T_END) if $setInternals;

    // and return the list of tokens
    Tokens::new(tokens)
  }
  
  pub fn revert(&self) -> Result<Tokens> {  
    let mut tokens = Vec::new();
    // iterate over the key-value pairs
    for tuple in &self.tuples {
      let KVData { key, value, use_default, keysets:_, primary_keyset, digested_value:_ } = tuple;
      if !primary_keyset.is_empty() {
        let reverted = self.revert_keyval(key, primary_keyset, value.as_ref(), *use_default, !tokens.is_empty())?;
        tokens.extend(reverted); 
      }
    }
    // and return the list of tokens
    Ok(Tokens::new(tokens))
  }

  fn revert_keyval(&self, key:&str, keyset: &str, value_opt:Option<&ArgWrap>, use_default:bool, is_first:bool) -> Result<Vec<Token>> {
    // get the key-value definition
    let keytype_stored = keyval_get(&keyval_qname(&self.prefix, keyset, key), "type");
    // define the tokens
    let mut tokens = Vec::new();
    // write comma and key, unless in the first iteration
    if !is_first {
      tokens.push(T_OTHER!(",")); }
    tokens.extend(Explode!(key));
    // write the default (if applicable)
    if !use_default {
      if let Some(value) = value_opt {
        tokens.push(T_OTHER!("="));
        if let Some(Stored::Parameter(keytype)) = keytype_stored {
          // TODO: The types here are a little curious. The stored value must be cast back into
          // Tokens if Parameter's revert works on Tokens. Or should that revert call work on ArgWrap?
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
    no_rebuild: bool
  ) -> Result<()> {
    // figure out the keyset(s) for the key to be added
    let keysets = self.resolve_keyval_for(key);
    let primary_keyset = self.get_primary_keyval(keysets.as_slice()).to_owned();

    // and add the new tuple to the set of tuples
    let value = if use_default {
      match keyval_get(&keyval_qname(&self.prefix,&primary_keyset,key),"default") {
        None => None,
        Some(v) => {
          let arg: Result<ArgWrap> = v.into();
          Some(arg?)
        }
      }
    } else {
      Some(value_arg)
    };
    self
      .tuples
      .push(KVData{key: key.to_string(), value, use_default, keysets,
         primary_keyset, digested_value: None});
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
    // set normally
    self.add_value(key, value, use_default, false)
  }

  fn rebuild(&mut self, skip_opt: Option<&str>) {
    // the new data structures to create
    let mut newtuples: Vec<KVData> = Vec::new();
    let mut pairs = Vec::new();
    let mut hash: HashMap<String, Vec<ArgWrap>> = HashMap::default();

    for tuple in self.tuples.drain(..) {
      // take all the elements we need from the stack
      let KVData {key, value, use_default, primary_keyset, keysets, digested_value} = tuple;
      // if we want to skip some values, we need to store new tuples
      let key_str = key.as_str();
      if Some(key_str) == skip_opt {
        continue;
      }
      if let Some(v) = value.as_ref() {
        // push key / value into the pair
        pairs.push((key.to_string(), v.clone()));

        // if we do not have a value yet, set it
        let entry = hash.entry(key.to_string()).or_default();

        // If we get a third value, push into an array
        // This is unlikely to be what the caller expects!! But what else?
        entry.push(v.clone());
      }
      
      // Record.
      newtuples.push(KVData {
        key,
        value,
        use_default,
        primary_keyset,
        keysets,
        digested_value
      });
    }

    // store all of the values
    self.cached_pairs = pairs;
    self.cached_hash = hash;
    if skip_opt.is_some() {
      self.tuples = newtuples;
    }
  }

  //======================================================================
  // parsing values from a gullet
  //======================================================================

  // A KeyVal argument MUST be delimited by either braces or brackets (if optional)
  // This method reads the keyval pairs INCLUDING the delimiters, (rather than
  // parsing after the fact), since some values may have special catcode needs.

  pub fn read_from(&mut self, until: Token, silence_missing: bool) -> Result<()> {
    // if we want to force skipMissing keys, we set it up here
    let skip_missing = self.skip_missing;
    let hook_missing = self.hook_missing;
    // if we want to silence all missing errors, store them in a hook
    if silence_missing {
      self.skip_missing = true;
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
      if gullet::if_next(T_BEGIN!())? { // Protect against redundant {} wrapping
        gullet::read_token()?;
        gullet::unread(gullet::read_balanced(false,false,false)?.strip_braces());
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
        let mut value = Tokens!();
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
            //       but we expect the singular Token as a delimiter result, since we are matching on a char separator
            delim_opt = gullet::read_match(&[&punct_tks, &until_tks])?
              .map(|tks| tks.into());
            if delim_opt.is_some() {
              break; // only until we hit a delim.
            }
            if let Some(tok) = gullet::read_token()? {
              // Copy next token to args
              toks.push(tok);
              if tok.get_catcode() == Catcode::BEGIN {
                let balanced_arg = gullet::read_balanced(false,false,false)?;
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
            value = Tokens::new(toks).strip_braces();
            if !value.is_empty() {
              if let Some(Stored::Parameter(ref keytype)) = keytype_opt {
                value = keytype.reparse(value)?;
              }
            }
          }
          // and cleanup
          if let Some(Stored::Parameter(ref keydef)) = keytype_opt {
            keydef.revert_catcodes()?;
          }
        }
        // and store our value please
        if !silence_missing || self.can_resolve_keyval_for(key) {
          self.add_value(key, ArgWrap::Tokens(value), !is_explicit, false)?;
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
        _ => todo!(),
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
