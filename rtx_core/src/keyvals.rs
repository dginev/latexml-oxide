use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;
// use std::rc::Rc;

use crate::common::error::*;
use crate::common::font::Font;
use crate::common::locator::Locator;
use crate::common::object::Object;
use crate::common::store::Stored;
use crate::gullet::Gullet;
use crate::stomach::Stomach;
// use crate::definition::expandable::Expandable;
// use crate::definition::Definition;
use crate::document::Document;
// use crate::list::List;
use crate::keyval::KeyVal;
use crate::state::State;
use crate::token::{Catcode, Token};
use crate::tokens::Tokens;
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
  was_digested: bool,
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
      was_digested: false,
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
  fn stringify(&self) -> String { "KeyVals:TODO".to_string() }

  fn be_digested(mut self, stomach: &mut Stomach, state: &mut State) -> Result<Digested> {
    if self.was_digested {
      Info!(
        "ignore",
        "keyvals",
        stomach,
        state,
        "Skipping digestion of \\setkeys as requested (did you digest a KeyVals twice?) "
      );
    } else {
      stomach.digest(self.set_keys_expansion(), state)?;
    }

    // new tuples we want to create
    let mut new_tuples: Vec<KVTuple> = Vec::new();

    // iterate over them
    for tuple in self.tuples.drain(..) {
      let (key, value, use_default, resolution, keyval) = tuple;
      // digest a single token
      let value_tokens_opt: Option<Tokens> = (&value).into();
      let value_tokens = value_tokens_opt.unwrap_or_default();
      let digested_value: Digested = if let Some(keydef) = keyval.get_type(state) {
        // keydefs are actual Parameter objects, which should be able to digest their own values!
        // Hmmm, so we need to add Parameter to Store
        // This comes together with the DefKeyVal infrastructure, which assigns keydef parameters to keyval specifications.
        keydef.digest(stomach, value_tokens, None, state)?.unwrap()
      } else {
        value_tokens.be_digested(stomach, state)?
      };
      new_tuples.push((key, digested_value.into(), use_default, resolution, keyval));
    }

    // read all our current state
    // my ($punct, $assign) = ($$self{punct}, $$self{assign});

    // then re-create the current object
    // let new = KeyVals {
    //   prefix,
    //   keysets,
    //   set_all => set_all,
    //   set_internals => set_internals,
    //   skip,
    //   skip_missing => $skip_missing, hookMissing => $hookMissing,
    //   was_digested => 1,
    //   punct => $punct, assign => $assign);
    let mut new = self;
    new.was_digested = true;
    new.set_tuples(new_tuples);
    Ok(new.into())
  }
}
impl BoxOps for KeyVals {
  fn get_properties_mut(&mut self) -> &mut HashMap<String, Stored> { unimplemented!() }
  fn unlist(&self) -> Vec<Digested> { Vec::new() } // TODO
  fn be_absorbed(&self, document: &mut Document, state: &mut State) -> Result<()> { Ok(()) } // TODO
  fn get_font(&self) -> Option<Cow<Font>> { None } // TODO
}

pub struct KeyValsOptions {
  pub prefix: Option<String>,
  pub keysets: Vec<String>,
  pub set_all: bool,
  pub set_internals: bool,
  pub skip: bool,
  pub skip_missing: bool,
}
impl Default for KeyValsOptions {
  fn default() -> Self {
    KeyValsOptions {
      prefix: None,
      keysets: Vec::new(),
      set_all: false,
      set_internals: false,
      skip: false,
      skip_missing: false,
    }
  }
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
  pub fn new(options: KeyValsOptions, state: &State) -> Self {
    // parse all the arguments
    let prefix = options.prefix.unwrap_or_else(|| String::from("KV"));
    // $keysets = [split(',', ToString(defined($keysets) ? $keysets : '_anonymous_'))] unless (ref($keysets) eq 'ARRAY');
    // let skip = options.get("skip").unwrap_or(false);
    // $skip = [split(',', ToString(defined($options{skip}) ? $options{skip} : ''))] unless (ref($options{skip}) eq 'ARRAY');
    // my $set_all       = $options{set_all}       ? 1 : 0;
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
      ..KeyVals::default()
    }
    // keysets     => $keysets,
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
  fn resolve_keyval_for(&self, key: &str) -> Vec<KeyVal> {
    // my $prefix  = $self->get_Prefix;
    // my @keysets = $self->get_keySets;
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
    //       'were you perhaps using \setkeys instead of \setkeys*?') unless defined($self->getskip_missing);
    //   return; }

    // // return either the first or all of the elements
    // return ($sets[0]) unless $self->getset_all;
    Vec::new()
  }

  fn get_primary_keyval_of(&self, key: &str, keysets: &[KeyVal]) -> KeyVal {
    if keysets.is_empty() {
      KeyVal::new(Some(self.prefix.clone()), self.keysets[0].clone(), key.to_string())
    } else {
      keysets[0].clone()
    }
  }

  fn read_keyword_from(&self, gullet: &mut Gullet, ignore: &[&Token], state: &mut State) -> Result<(Vec<Token>, Option<Token>)> {
    // set of tokens we will expand
    let mut tokens = Vec::new();

    // we do not want any spaces
    gullet.skip_spaces(state);

    // read tokens one-by-one
    let mut last_token = None;
    while let Some(token) = gullet.read_x_token(false, false, state)? {
      // skip to the next iteration if we have a paragraph
      if token == T_CS!("\\par") {
        continue;
      }

      // if we have one of out delimiters, we end
      if ignore.iter().any(|delim| &token == *delim) {
        last_token = Some(token);
        break;
      }

      // push a token unless we have a space
      // TODO: remove or normalize
      if token.get_catcode() != Catcode::SPACE {
        tokens.push(token);
      }
    }

    // return the tokens and the last token
    Ok((tokens, last_token))
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
    // if ($definedrm && $STATE->lookupMeaning($rmmacro)) {
    //   @rmtokens = LaTeXML::Package::Expand($rmmacro)->unlist; }

    // define some xkeyval internals
    let mut tokens = Vec::new();
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
    //     push(@tokens, $hookMissing, T_BEGIN, $self->revertKeyVal($keyval, $value, $useDefault, 1, 1, $punct, $assign), T_END) if $hookMissing;
    //     next; }

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
  // sub revert {
  //   my ($self) = @_;

  //   # read values from class
  //   my ($punct, $assign) = ($$self{punct}, $$self{assign});

  //   my @tokens = ();

  //   # iterate over the key-value pairs
  //   foreach my $tuple (@{ $$self{tuples} }) {
  //     my ($key, $value, $useDefault, $resolution, $keyval) = @$tuple;
  //     # revert a single token
  //     if ($keyval) {    # when is this undef?
  //       push(@tokens, $self->revertKeyVal($keyval, $value, $useDefault, (@tokens ? 0 : 1), 0, $punct, $assign)); } }

  //   # and return the list of tokens
  //   return Tokens(@tokens); }

  //======================================================================
  // Changing contained values
  //======================================================================

  pub fn add_value(&mut self, key: &str, value: Stored, use_default: bool, no_rebuild: bool, state: &State) {
    // figure out the keyset(s) for the key to be added
    let keysets = self.resolve_keyval_for(key);
    let headset = self.get_primary_keyval_of(key, &keysets);

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

  fn set_tuples(&mut self, tuples: Vec<KVTuple>) {
    self.tuples = tuples;
    // we need to build all the caches
    self.rebuild(None);
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

  //======================================================================
  // parsing values from a gullet
  //======================================================================

  // A KeyVal argument MUST be delimited by either braces or brackets (if optional)
  // This method reads the keyval pairs INCLUDING the delimiters, (rather than
  // parsing after the fact), since some values may have special catcode needs.

  pub fn read_from(&mut self, gullet: &mut Gullet, until: Token, state: &mut State) -> Result<()> {
    // TODO
    // # if we want to force skip_missing keys, we set it up here
    // my $silenceMissing = $options{silenceMissing} ? 1 : 0;

    // my $skip_missing = $self->getskip_missing;
    // my $hookMissing = $self->getHookMissing;

    // # if we want to silence all missing errors, store them in a hook
    // if ($silenceMissing) {
    //   $$self{skip_missing} = 1;
    //   $$self{hookMissing} = undef; }

    // read the opening token and figure out where we are
    let startloc = gullet.get_locator().into_owned();

    // set and read tokens
    let open = gullet.read_token(state);
    let assign = T_OTHER!("=");
    let punct = T_OTHER!(",");
    let punct_tks = Tokens!(T_OTHER!(","));
    let until_tks = Tokens!(until.clone());
    // my ($punct, $assign) = ($$self{punct}, $$self{assign});

    // create arrays for key-value pairs and explicit values
    // TODO:
    // let mut kv        = Vec::new();
    // let mut explicits = Vec::new();

    // iterate over all the key-value pairs to read
    loop {
      // Read a single keyword, get a delimiter and a set of keyword tokens
      let (ktoks, mut delim_opt) = self.read_keyword_from(gullet, &[&until, &assign, &punct], state)?;
      // if there was no delimiter at the end, we throw an error
      if delim_opt.is_none() {
        let message = s!("Fell off end expecting {} while reading KeyVal key", until.stringify());
        let message2 = s!("key started at {}", startloc.to_string());
        Error!("expected", until, gullet, state, message, message2);
      }

      // turn the key tokens into a string and normalize
      let mut key = Tokens!(ktoks).to_string();
      key = key.split_whitespace().collect::<Vec<&str>>().join("");

      // if we have a non-empty key
      if !key.is_empty() {
        let mut value = Tokens!();
        let is_default: bool = delim_opt.is_none() || delim_opt.as_ref().unwrap() != &assign;

        // if we have an '=', we explcity assign a value
        if !is_default {
          // setup the key-codes to properly read
          let keyval = self.get_primary_keyval_of(&key, &self.resolve_keyval_for(&key));
          let keydef_opt = keyval.get_type(state);
          if let Some(ref keydef) = keydef_opt {
            // TODO:
            // keydef.setup_catcodes();
          }

          // read until $punct
          let mut toks = Vec::new();
          loop {
            delim_opt = match gullet.read_match(&[&punct_tks, &until_tks], state)? {
              Some(tks) => Some(tks.into()), // Tokens reader, but we look for single Token delim
              None => None,
            };
            if delim_opt.is_some() {
              break; // only until we hit a delim.
            }
            if let Some(tok) = gullet.read_token(state) {
              // Copy next token to args
              let mut rest = Vec::new();
              if tok.get_catcode() == Catcode::BEGIN {
                rest.append(&mut gullet.read_balanced(state)?.unlist());
                rest.push(T_END!());
              }
              // record for keyvals
              toks.push(tok);
              toks.append(&mut rest);
            } else {
              break;
            }
          }
          // reparse (and expand) the tokens representing the value
          if !toks.is_empty() {
            value = Tokens::new(toks);
            if let Some(ref keydef) = keydef_opt {
              // TODO:
              // value = keydef.reparse(gullet, value)
            }
          }
          // and cleanup
          if let Some(ref keydef) = keydef_opt {
            // TODO:
            // keydef.revert_catcodes()
          }
        }

        // and store our value please
        // if !silence_missing || self.can_resolve_keyval_for(key) {
        self.add_value(&key, Stored::Tokens(value), is_default, false, state);
        // }
      }

      // we finish if we have the last element
      if delim_opt.is_some() && delim_opt.as_ref().unwrap() == &until {
        break;
      }
    }

    // rebuild and return nothing
    // $self->rebuild;

    // # restore all settings if we silenced the missing keys
    // if ($silenceMissing) {
    //   $$self{skip_missing} = $skip_missing;
    //   $$self{hookMissing} = $hookMissing; }
    Ok(())
  }

  // returns a key => ToString(value)
  pub fn get_hash(&self) -> HashMap<String, String> {
    let mut hashed = HashMap::new();
    for (k, v) in &self.cached_hash {
      hashed.insert(k.to_string(), v.iter().map(ToString::to_string).collect::<Vec<String>>().join(""));
    }
    hashed
  }

  /// TODO: This is an improvised method for switching KeyVals into Tokens, but losing all collected metadata
  ///       the long-term solution ought to be via a type system extension, where the arguments to our before-digest closures
  ///       are a vector of a new type ReadValue ::= [Token, KeyVals, RegisterValue]
  ///       potentially? On the other hand, we can also put the extra effort of *postponing* the build of KV metadata until digestion,
  ///       this way not losing any time reserializing metadata
  pub fn into_tokens(self) -> Tokens {
    let mut tks: Vec<Token> = Vec::new();
    for (k, v) in self.cached_pairs.into_iter() {
      tks.push(T_OTHER!(k));
      tks.push(match v {
        Stored::Tokens(vtks) => vtks.into(),
        Stored::Token(vtk) => vtk,
        Stored::String(vstr) => T_OTHER!(vstr),
        _ => Token::default(),
      });
    }
    Tokens::new(tks)
  }
}

impl From<KeyVals> for Result<Option<Digested>> {
  fn from(value: KeyVals) -> Result<Option<Digested>> {
    let tmp: Digested = value.into();
    tmp.into()
  }
}
