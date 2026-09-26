use std::{
  cell::{Cell, RefCell},
  fmt,
  rc::Rc,
};

use once_cell::sync::Lazy;
#[cfg(feature = "codegen")]
use proc_macro2::TokenStream;
#[cfg(feature = "codegen")]
use quote::{ToTokens, quote};
use regex::Regex;

use crate::{
  Digested,
  common::{
    arena::{self, SymStr},
    error::{emit_warn, *},
    object::Object,
  },
  definition::{
    BeforeDigestClosure, Definition, DigestionClosure,
    argument::ArgWrap,
    constructor::Constructor,
    register::{RegisterType, RegisterValue},
  },
  gullet,
  mouth::Mouth,
  pin,
  state::*,
  token::{Catcode, Token},
  tokens::Tokens,
  whatsit::Whatsit,
};

pub type ReaderFn = dyn Fn(Option<&Parameters>, &[Tokens]) -> Result<ArgWrap>;
pub type ReaderPredigestFn = dyn Fn(ArgWrap, &[Tokens]) -> Result<Option<Digested>>;
pub type ReaderPredigestClosure = Rc<ReaderPredigestFn>;
pub type ReaderClosure = Rc<ReaderFn>;

// Rust Note:
// the reversion functions initially had "&mut Gullet" as a parameter.
// This turned out to be infeasible if we are to maintain the latexml code flow
// as we have calls into reversions from arbitrary binding closures, at ALL phases.
// Compromise: use the gated Stomach in state::whenever you need gullet in reversion, as in
// let mut stomach = state::stomach.borrow_mut();
//
//
pub type ReversionClosure =
  Rc<dyn Fn(Vec<Token>, Option<&Parameters>, &[Tokens]) -> Result<Tokens>>;

/// A reversion closure that operates on the original Digested argument,
/// enabling access to structured data (e.g., KeyVals) for custom reversion formatting.
/// Perl equivalent: the `reversion` option on DefParameterType, which receives the raw value.
pub type DigestedReversionClosure = Rc<dyn Fn(&Digested) -> Result<Tokens>>;

/// `LXML_TRACE_ARGS=\cs`, read ONCE: both argument readers below run on every
/// macro, primitive, conditional and constructor invocation, and
/// `std::env::var` takes the process environment lock and byte-scans the
/// environment each call — 14 % of a pgf/TikZ conversion (tikz-network's
/// 887 M token reads per picture; symbolized profile
/// `~/data/pk_agents/w23/perf_pgf/NOTES.md`), the same trap gullet.rs's
/// `TRACE_GROUP_END` records. A debug switch set before launch never changes.
static TRACE_ARGS: Lazy<Option<String>> = Lazy::new(|| std::env::var("LXML_TRACE_ARGS").ok());

static LAST_WCHAR_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\w$").unwrap());
static FIRST_WCHAR_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\w").unwrap());

#[derive(Clone)]
pub struct Parameter {
  pub novalue:            bool,
  /// A `\newcommand`-family optional argument: latex.ltx reads it through
  /// `\@protected@testopt` → `\@testopt` → `\kernel@ifnextchar[` (1249, 1261,
  /// 1259-1260), a futurelet peek, so its macro
  /// [`peeks_by_futurelet`](crate::definition::Definition::peeks_by_futurelet).
  pub testopt:            bool,
  pub semiverbatim:       Option<Vec<char>>,
  pub optional:           bool,
  pub name:               SymStr,
  pub spec:               SymStr,
  pub extra:              Vec<Tokens>,
  pub inner:              Option<Parameters>,
  pub reader:             ReaderClosure,
  pub predigest:          Option<ReaderPredigestClosure>,
  pub reversion:          Option<ReversionClosure>,
  /// Reversion closure that operates on the original Digested argument.
  /// Takes precedence over `reversion` when the argument is a digested value.
  /// Perl equivalent: `reversion` option on DefParameterType with `undigested => 1`.
  pub digested_reversion: Option<DigestedReversionClosure>,
  pub before_digest:      Vec<BeforeDigestClosure>,
  pub after_digest:       Vec<DigestionClosure>,
}
impl Default for Parameter {
  fn default() -> Self {
    Parameter {
      novalue:            false,
      testopt:            false,
      semiverbatim:       None,
      optional:           false,
      name:               pin!("parameter_default"),
      spec:               pin!(""),
      extra:              Vec::new(),
      inner:              None,
      reader:             Rc::new(|_args, _extra| {
        Warn!(
          "Parameter",
          "mock_reader",
          "Please define a real reader, this is a mock fallback!"
        );
        Ok(ArgWrap::None)
      }),
      predigest:          None,
      reversion:          None,
      digested_reversion: None,
      before_digest:      Vec::new(),
      after_digest:       Vec::new(),
    }
  }
}
impl fmt::Debug for Parameter {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    writeln!(
      f,
      "Parameter(\n\t name:{:?}, novalue:{:?}, semiverbatim:{:?},",
      self.name, self.novalue, self.semiverbatim,
    )?;
    writeln!(f, "\t optional:{:?}, spec:{:?}", self.optional, self.spec)?;
    writeln!(f, "\t inner: {:?}", self.inner)?;
    writeln!(
      f,
      "\t extra: {:?}\n\t reversion: {:?}, before_digest: {:?}, after_digest: {:?} )",
      self.extra,
      self.reversion.is_some(),
      self.before_digest.len(),
      self.after_digest.len()
    )
  }
}
impl fmt::Display for Parameter {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    arena::with(self.name, |name| write!(f, "{name}"))
  }
}

impl PartialEq for Parameter {
  fn eq(&self, other: &Parameter) -> bool { self.name == other.name }
}
impl Object for Parameter {
  fn stringify(&self) -> String { arena::to_string(self.spec) }
}

static OPTIONAL_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^Optional(.+)$").unwrap());
static SKIP_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^Skip(.+)$").unwrap());

/// A `{}` argument that is exactly one token whose MEANING is `{` (an
/// implicit begin-group such as `\bgroup`, not a literal brace): the `{}`
/// reader takes the single token, so the group body is still in the gullet.
fn is_implicit_begin_group(arg: &ArgWrap) -> bool {
  match arg {
    ArgWrap::Tokens(toks) => {
      let list = toks.unlist_ref();
      list.len() == 1
        && list[0].get_catcode() != Catcode::BEGIN
        && list[0].defined_as(&crate::T_BEGIN!())
    },
    _ => false,
  }
}

impl Parameter {
  pub fn new<T: AsRef<str>>(name: T, spec: T, extra: Option<Vec<Tokens>>) -> Result<Self> {
    Parameter {
      name: arena::pin(name),
      spec: arena::pin(spec),
      extra: extra.unwrap_or_default(),
      ..Parameter::default()
    }
    .init()
  }
  pub fn init(mut self) -> Result<Self> {
    // Create a parameter reading object for a specific type.
    // If either a declared entry or a function Read<Type> accessible from LaTeXML::Package::Pool
    // is defined.
    let mut descriptor: Option<Rc<Parameter>> =
      with_mapping_sym(pin!("PARAMETER_TYPES"), self.name, |looked_up_mapping| {
        if let Some(Stored::Parameter(d_lookup)) = looked_up_mapping {
          Some(Rc::clone(d_lookup))
        } else {
          None
        }
      });
    if descriptor.is_none() {
      // TODO: see discussion on line 168
      let basetype_opt = arena::with(self.name, |name| {
        OPTIONAL_REGEX
          .captures(name)
          .map(|captures| captures.get(1).map_or("", |m| m.as_str()).to_string())
      })
      .map(arena::pin);
      if let Some(basetype) = basetype_opt {
        descriptor = with_mapping_sym(pin!("PARAMETER_TYPES"), basetype, |basetype_param_opt| {
          match basetype_param_opt {
            Some(Stored::Parameter(d_lookup)) => Ok(Some(d_lookup.clone())),
            _ => match Parameter::check_reader_function(&arena::with(self.name, |name| {
              s!("Read{name}")
            })) {
              Some(reader) => Ok(Some(Rc::new(Parameter {
                reader,
                optional: true,
                ..Parameter::default()
              }))),
              None => match Parameter::check_reader_function(&arena::with(basetype, |type_str| {
                s!("Read{type_str}")
              })) {
                Some(reader) => Ok(Some(Rc::new(Parameter {
                  reader,
                  optional: true,
                  novalue: true,
                  ..Parameter::default()
                }))),
                None => fatal!(
                  Parameter,
                  Init,
                  s!("Can't initialize parameter {:?}, unknown?", self.name)
                ),
              },
            },
          }
        })?;
        self.optional = true;
      } else {
        // TODO: This looks like a code smell. Do we need a new arena method?
        // We start with a ticket, do a non-allocation operation on the underlying &str,
        // Then want to ping the newly acquired &str slice in the arena. Clearly that is only
        // possible *AFTER* the original &str is released, but how do we avoid allocating?
        // Is this a use for unsafe{} or is there a more idiomatic way?
        // Maybe, with_mut... which allows an inner pin?
        let basetype_opt = arena::with(self.name, |name| {
          SKIP_REGEX
            .captures(name)
            .map(|captures| captures.get(1).map_or("", |m| m.as_str()).to_string())
        })
        .map(arena::pin);
        if let Some(basetype) = basetype_opt {
          descriptor = with_mapping_sym(pin!("PARAMETER_TYPES"), basetype, |basetype_param_opt| {
            match basetype_param_opt {
              Some(Stored::Parameter(d_lookup)) => Some(d_lookup.clone()),
              _ => match arena::with(self.name, |name| Parameter::check_reader_function(name)) {
                Some(reader) => Some(Rc::new(Parameter {
                  reader,
                  optional: true,
                  novalue: true,
                  ..Parameter::default()
                })),
                None => Parameter::check_reader_function(&arena::with(basetype, |type_str| {
                  s!("Read{type_str}")
                }))
                .map(|reader| {
                  Rc::new(Parameter {
                    reader,
                    optional: true,
                    novalue: true,
                    ..Parameter::default()
                  })
                }),
              },
            }
          });
          if let Some(ref _desc) = descriptor {
            self.novalue = true;
            self.optional = true;
          }
        } else {
          descriptor =
            Parameter::check_reader_function(&arena::with(self.name, |name| s!("Read{name}")))
              .map(|reader| Rc::new(Parameter { reader, ..Parameter::default() }));
        }
      }
    }
    match descriptor {
      Some(descriptor) => {
        // descriptor needs to get integrated into Self
        //  except `spec` and `name` which are always preserved!
        self.reader = descriptor.reader.clone(); // What else?
        if descriptor.novalue {
          self.novalue = true;
        }
        self.semiverbatim.clone_from(&descriptor.semiverbatim);
        // Also doing optional setting on the fly, so don't override unless true
        // self.optional = descriptor.optional;
        if descriptor.optional {
          self.optional = true;
        }
        self.reversion.clone_from(&descriptor.reversion);
        self
          .digested_reversion
          .clone_from(&descriptor.digested_reversion);
        self.before_digest.clone_from(&descriptor.before_digest);
        self.after_digest.clone_from(&descriptor.after_digest);
        self.predigest.clone_from(&descriptor.predigest);
      },
      None => fatal!(
        Parameter,
        Unknown,
        arena::with2(self.name, self.spec, |name, spec| s!(
          "Unrecognized parameter type with name {:?}, spec {:?}",
          name,
          spec
        ))
      ),
    }
    // Last but not least, initialize any "inner" parameters
    self.inner = self.inner.map(|inner_ps| match inner_ps.clone().init() {
      Ok(ps) => ps,
      Err(e) => {
        emit_warn(
          "internal",
          "parameter",
          &format!("inner parameter init failed: {e}"),
        );
        inner_ps
      },
    });
    Ok(self)
  }

  /// Obtain the reader of a given parameter name, if available
  pub fn check_reader_function(name: &str) -> Option<ReaderClosure> {
    // TODO: This function doesn't have a direct Rust equivalent, since the metaprogramming isn't
    // possible But what is the exact purpose of seeking through the pool namespace? Wouldn't
    // any parameter be already assigned in the state::
    with_mapping("PARAMETER_TYPES", name, |param_opt| {
      if let Some(Stored::Parameter(param)) = param_opt {
        Some(param.reader.clone())
      } else {
        None
      }
    })
  }

  /// ExpandedSemiverbatim wants the semiverbatim DIGESTION protection
  /// (Parameter::digest's neutralize-under-ASCII branch — without it an
  /// OTHER `_` font-decodes to the OT1 dot-above in the constructor's
  /// output) but must NOT flip catcodes at READ time: its whole point is
  /// tokenizing the argument at the CALLER's catcodes so an expl3
  /// `\l_..._tl` file name stays one control sequence.
  fn semiverbatim_read_setup(&self) -> bool {
    self.semiverbatim.is_some() && !arena::with(self.name, |n| n == "ExpandedSemiverbatim")
  }

  pub fn setup_catcodes(&self) {
    if self.semiverbatim_read_setup() {
      begin_semiverbatim(self.semiverbatim.as_deref());
    }
  }

  pub fn revert_catcodes(&self) -> Result<()> {
    if self.semiverbatim_read_setup() {
      end_semiverbatim()?;
    }
    Ok(())
  }

  pub fn read(&self, fordefn: Option<&dyn Definition>) -> Result<ArgWrap> {
    // For semiverbatim, I had messed with catcodes, but there are cases
    // (eg. \caption(...\label{badchars}}) where you really need to
    // cleanup after the fact!
    // Hmmm, seem to still need it...
    self.setup_catcodes();

    let closure = &self.reader;
    let value_from_reader: ArgWrap = closure(self.inner.as_ref(), &self.extra)?;
    // Direct enum destructure: was `is_tokens() then owned_tokens()`
    // which matched twice (once for the is_tokens check, again for
    // the owned_tokens dispatch over all ArgWrap variants). This
    // function fires on every parameter read of every macro call —
    // ~2M times on si.tex per callgrind.
    let value_arg = match value_from_reader {
      ArgWrap::Tokens(mut value) => {
        if let Some(ref semi_chars) = self.semiverbatim {
          value = value.neutralize(semi_chars);
        }
        ArgWrap::Tokens(value)
      },
      other => other,
    };
    self.revert_catcodes()?;

    // Single arena borrow to compute both name-prefix checks that
    // this function needs — was two separate `arena::with` calls
    // (each a RefCell borrow + interner resolve), now a single
    // closure that returns the pair.
    let (is_optional_match, is_until) = arena::with(self.name, |name| {
      (name.starts_with("OptionalMatch"), name.starts_with("Until"))
    });

    // Perl: experiment: skip spaces after a successful OptionalMatch read
    // (not when the call is being abandoned: nothing more is read).
    if !value_arg.is_none() && self.optional && is_optional_match && !gullet::argument_runaway() {
      gullet::skip_spaces()?;
    }

    // A `\def` parameter text's leading delimiter (`\def\lp\x{…}`) is a
    // value-less required `Match`: its miss IS an improper macro call (below).
    let checked_value = if !self.optional
      && (!self.novalue || self.is_macro_delimiter())
      && (value_arg.is_none() && self.predigest.is_none())
      // A call being abandoned (tex.web §392) has nothing more to report.
      && !gullet::argument_runaway()
    {
      // `Until:` readers return a DISTINGUISHABLE EOF (read_until → None
      // when the delimiter never appeared; a matched-but-empty arg is
      // Some(empty)). Real TeX's runaway-argument ERROR applies to REAL
      // file ends — but many of our mouth ends are ARTIFICIAL (isolated
      // constructor-argument mouths, reading_from_mouth), where real TeX
      // would keep scanning the enclosing stream and find the delimiter
      // (l3tl replace sentinels: spath3/litetable/zref-check — a
      // per-iteration Error here regressed 63 corpus docs, sweep 23).
      // So: stay QUIET for Until misses; zero-progress delimited-scan
      // loops terminate via the stomach cycle guard instead. Perl errors
      // here (Parameter.pm L93-97) and equally mis-fires on the
      // artificial-EOF shapes. (The `Until` reader itself never yields
      // None — it maps a miss to an empty argument, and reports the one
      // REAL runaway, a miss at `gullet::at_end_of_all_input`, itself.)
      // A `\def` delimiter missed because an ISOLATED token list ran out (an
      // index phrase digested at `\index` time, a constructor argument) is the
      // same artificial end: real TeX would read on — manyind.sty:98
      // `\def\mgobblepgeref, #1 {}` eats makeindex's `, <page>`, which never
      // follows here. Quiet, and the call proceeds as before. Only a present,
      // different token is TeX's improper call (tex.web §398).
      let ran_out = self.is_macro_delimiter()
        && match gullet::read_token()? {
          Some(next) => {
            gullet::unread_one(next);
            false
          },
          None => true,
        };
      if ran_out && gullet::argument_ran_off_a_file()? {
        // At a file's end the call is abandoned instead (tex.web §392).
        value_arg
      } else if ran_out {
        ArgWrap::Tokens(Tokens::new(Vec::new()))
      } else if !is_until {
        let fordefn_str = fordefn.map(|fdefn| fdefn.stringify()).unwrap_or_default();
        Error!(
          "expected",
          self,
          s!("Missing argument {} for {}", self.stringify(), fordefn_str)
        );
        if self.novalue {
          ArgWrap::None
        } else {
          ArgWrap::Tokens(Tokens!(T_OTHER!("missing")))
        }
      } else {
        value_arg
      }
    } else {
      value_arg
    };
    Ok(checked_value)
  }

  /// The leading delimiter of a `\def` parameter text (`\def\lp\x{…}`,
  /// base_utilities.rs `parse_def_parameters`): a required `Match` that carries
  /// no value. Binding `Match:to`-style parameters carry their value.
  fn is_macro_delimiter(&self) -> bool {
    self.novalue && !self.optional && self.name == crate::pin!("Match")
  }

  pub fn digest(
    &self,
    mut value_arg: ArgWrap,
    _fordefn: Option<&Constructor>,
  ) -> Result<Option<Digested>> {
    // Perl Parameter.pm lines 122,139-141: capture MODE, check after digest
    let mode = lookup_string_from_sym(crate::pin!("MODE"));
    // If semiverbatim, Expand (before digest), so tokens can be neutralized; BLECH!!!!
    if self.semiverbatim.is_some() {
      // Digest-time protection applies to ExpandedSemiverbatim too (its
      // read-time carve-out lives in semiverbatim_read_setup) — use the
      // raw begin, not setup_catcodes.
      begin_semiverbatim(self.semiverbatim.as_deref());
      if value_arg.is_tokens() {
        if let Some(value) = value_arg.owned_tokens() {
          let neutralized = gullet::reading_from_mouth(Mouth::default(), move || {
            gullet::unread(value);
            let mut tokens = Vec::new();
            loop {
              match gullet::get_pending_comment() {
                Some(token) => tokens.push(token),
                None => match gullet::read_x_token(Some(true), false, None) {
                  Ok(token_opt) => match token_opt {
                    Some(token) => tokens.push(token),
                    None => break,
                  },
                  Err(x) => return Err(x),
                },
              }
            }
            Ok(Tokens::new(tokens).neutralize(&[]))
          })?;
          value_arg = ArgWrap::Tokens(neutralized);
        } else {
          value_arg = ArgWrap::default();
        }
      }
    }

    for pre in self.before_digest.iter() {
      // Done for effect only.
      pre()?; // maybe pass extras?
    }
    let digested_value = if let Some(closure) = &self.predigest {
      closure(value_arg, &self.extra)?
    } else {
      // Note: we have an open question for the type interface.
      //  What happens when a wrapped "None" value,
      // (such as the missing value of an Optional [] argument)
      // gets digested?
      //
      // currently a `Digested::default` gets returned, which has an empty TBox and also gets
      // returned for e.g. empty mandatory Plain arguments {}.
      // But we need *different* values, as the explicit "\foo[]" is an override to empty, while
      // "\foo" will use the default value for the Optional.
      if self.optional && value_arg.is_none() {
        None
      } else if self.name == pin!("Plain") && is_implicit_begin_group(&value_arg) {
        // `\mbox\bgroup A … B\egroup` (syntax.sty:158 `\syn@assist`, whose
        // `\egroup` is even inserted by `\readupto`'s `\aftergroup`; the
        // newcommand manual): TeX hands `\bgroup` to `\mbox#1` as its one-token
        // argument, and the `\hbox{` that `\mbox` opens is then closed by the
        // `\egroup`, its own `}` having closed the `\bgroup` — so the box runs to
        // the `\egroup`. A `\bgroup`-only argument therefore reads its group by
        // DIGESTION from the live gullet until the frame closes (the `{`
        // primitive: `bgroup` + `digest_next_body`), tex.web §1063/§1068 pairing
        // an implicit brace with whatever closes the group. Perl reads the lone
        // `\bgroup`, opens a boxing frame inside the constructor's mode frame,
        // and errors at `end_mode` (SHARED; pdflatex clean). Guard:
        // `perfect_kernel_batch54::implicit_bgroup_argument_reads_its_group`.
        crate::stomach::invoke_token(&crate::T_BEGIN!())?
          .into_iter()
          .next()
      } else {
        Some(value_arg.be_digested()?)
      }
    };
    for post in self.after_digest.iter() {
      // Done for effect only.
      let mut w = Whatsit::default();
      post(&mut w)?; // maybe pass extras?
    }

    // Pairs with the raw begin_semiverbatim at the top of digest().
    if self.semiverbatim.is_some() {
      end_semiverbatim()?;
    }

    // Perl Parameter.pm lines 139-141: avoid mode change leaking out of parameter digestion
    let newmode = lookup_string_from_sym(crate::pin!("MODE"));
    if mode != newmode && mode != "horizontal" {
      crate::stomach::leave_horizontal_internal();
    }

    Ok(digested_value)
  }

  pub fn revert(&self, value_opt: Option<Tokens>) -> Result<Option<Tokens>> {
    if let Some(ref reverter) = self.reversion {
      if let Some(value) = value_opt {
        Ok(Some((reverter)(
          value.unlist(),
          self.inner.as_ref(),
          &self.extra,
        )?))
      } else {
        Ok(None)
      }
    } else if let Some(value) = value_opt {
      Ok(Some(Tokens::new(value.revert())))
    } else {
      Ok(None)
    }
  }

  /// This is needed by structured parameter types like KeyVals
  /// where the argument may already have been tokenized before the KeyVals
  /// (and the parameter types for the keys) had a chance to properly parse.
  // Yuck!
  pub fn reparse(&self, tokens: Tokens) -> Result<ArgWrap> {
    // Needs neutralization, since the keyvals may have been tokenized already???
    // perhaps a better test would involve whether $tokens is, in fact, Tokens?
    if self.name == pin!("Plain") || self.predigest.is_some() {
      // Gack!
      Ok(ArgWrap::Tokens(tokens))
    } else if self.semiverbatim.is_some() {
      // Needs neutralization
      // but maybe specific to catcodes
      Ok(ArgWrap::Tokens(
        tokens.neutralize(self.semiverbatim.as_ref().unwrap().as_slice()),
      ))
    } else {
      gullet::reading_from_mouth(Mouth::default(), || {
        // start with empty mouth
        let mut tokens = tokens.unlist();
        if !tokens.is_empty() // Strip outer braces from dimensions & friends
          && arena::with(self.name,|name|
              matches!(name, "Number"|"Dimension"|"Glue"|"MuDimension"|"MuGlue"))
          && tokens.first().map(|t| t.get_catcode() == Catcode::BEGIN)
              .unwrap_or(false)
          && tokens.last().map(|t| t.get_catcode() == Catcode::END).unwrap_or(false)
        {
          tokens.remove(0);
          tokens.pop();
        }
        gullet::unread_vec(tokens); // but put back tokens to be read
        let value = self.read(None)?;
        gullet::skip_spaces()?;
        Ok(value)
      })
    }
  }
}

#[derive(Clone, Debug, Default)]
pub struct Parameters(Vec<Parameter>);

impl PartialEq for Parameters {
  fn eq(&self, other: &Parameters) -> bool { self.0 == other.0 }
}
impl Object for Parameters {
  fn stringify(&self) -> String {
    let mut result = String::new();
    for parameter in self.0.iter() {
      let s = parameter.stringify();
      let lead_letter = match s.chars().next() {
        Some(c) => c.is_alphanumeric(),
        None => false,
      };
      let trail_letter = match result.chars().last() {
        Some(c) => c.is_alphanumeric(),
        None => false,
      };
      if lead_letter && trail_letter {
        result.push(' ');
      }
      result.push_str(&s);
    }
    result
  }
}

impl Parameters {
  pub fn new(params: Vec<Parameter>) -> Self { Parameters(params) }
  pub fn get_num_args(&self) -> usize { self.0.iter().filter(|&p| !p.novalue).count() }
  pub fn get_parameters(&self) -> Vec<&Parameter> { self.0.iter().collect() }
  pub fn take_parameters(self) -> Vec<Parameter> { self.0 }
  pub fn revert_arguments(&self, args: Vec<Option<Tokens>>) -> Result<Vec<Token>> {
    let mut tokens = Vec::new();
    for (parameter, arg) in self.0.iter().zip(args) {
      if !parameter.novalue
        && let Some(reverted_tks) = parameter.revert(arg)?
      {
        tokens.extend(reverted_tks.unlist());
      }
    }
    Ok(tokens)
  }

  /// Revert arguments from their digested form, using `digested_reversion` when available.
  /// This allows parameter types (like BoxSpecification) to control reversion formatting
  /// based on the structured digested data rather than token-level reversion.
  /// Perl equivalent: `$parameters->revertArguments($self->getArgs)`
  pub fn revert_digested_arguments(
    &self,
    digested_args: &[Option<Digested>],
  ) -> Result<Vec<Token>> {
    let mut tokens = Vec::new();
    for (parameter, arg_opt) in self.0.iter().zip(digested_args) {
      if !parameter.novalue {
        let reverted = if let Some(ref digested_rev) = parameter.digested_reversion {
          // Use digested_reversion: operates on the raw Digested value
          match arg_opt {
            Some(arg) => Some(digested_rev(arg)?),
            None => None,
          }
        } else {
          // Fall back to standard reversion: Digested → Tokens → Parameter::revert
          let token_reverted = match arg_opt {
            Some(arg) => Some(arg.revert()?),
            None => None,
          };
          parameter.revert(token_reverted)?
        };
        if let Some(tks) = reverted {
          tokens.extend(tks.unlist());
        }
      }
    }
    Ok(tokens)
  }
  // Try to initialize each associated Parameter
  pub fn init(mut self) -> Result<Self> {
    let mut initialized = Vec::new();
    for param in self.0.drain(..) {
      initialized.push(param.init()?);
    }
    self.0 = initialized;
    Ok(self)
  }

  pub fn read_arguments(&self, fordefn: Option<&dyn Definition>) -> Result<Vec<ArgWrap>> {
    Ok(
      self
        .read_parameter_list(fordefn, false)?
        .unwrap_or_default(),
    )
  }

  /// `read_arguments` for a macro call: `None` when the call does not match its
  /// definition — a `\def` parameter text's leading delimiter is not there
  /// (tex.web §397-398 "Use of \x doesn't match its definition": reported, the
  /// token backed up, the macro IGNORED). Perl reports it and expands the macro
  /// anyway, so a self-calling `\def\lp\x{\lp}` met with `\lp\y` re-reports to
  /// its error cap; the expansion loop has no cap and ran to the digestion fuse.
  /// Arguments after the miss are not read (TeX stops at the mismatch).
  /// OXIDIZED_DESIGN #295; guard
  /// `perfect_kernel_batch56::macro_delimiter_mismatch_ignores_the_call`.
  ///
  /// `None` too when an argument ran off the end of a file
  /// ([`gullet::argument_runaway`]): tex.web §392 "Report a runaway argument
  /// and abort" abandons the call and drops the arguments read so far; the
  /// caller that set the `matching` status consumes the mark
  /// (`Expandable::read_call_arguments`). Guards: `scanner_status::*`.
  pub fn read_macro_arguments(
    &self,
    fordefn: Option<&dyn Definition>,
  ) -> Result<Option<Vec<ArgWrap>>> {
    self.read_parameter_list(fordefn, true)
  }

  /// The parameter loop of [`Self::read_macro_arguments`] (`macro_call`) and
  /// [`Self::read_arguments`]. Only a macro call stops at a runaway argument:
  /// a nested re-parse of an argument already read (`reparse_argument`) must
  /// deliver its values.
  fn read_parameter_list(
    &self,
    fordefn: Option<&dyn Definition>,
    macro_call: bool,
  ) -> Result<Option<Vec<ArgWrap>>> {
    let mut args = Vec::with_capacity(self.0.len());
    // `LXML_TRACE_ARGS=\cs`: see `read_arguments_and_digest` (macros and
    // primitives read their parameters here).
    let traced = fordefn.is_some_and(|d| {
      TRACE_ARGS
        .as_deref()
        .is_some_and(|want| d.get_cs().to_string() == want)
    });
    let tails = self.may_defer_tails().then(ArgumentTails::open);
    for parameter in &self.0 {
      let values = parameter.read(fordefn)?;
      if traced && let Some(d) = fordefn {
        let shown = values.revert().map(|t| t.to_string()).unwrap_or_default();
        eprintln!("ARGS {}: {} = {shown}", d.get_cs(), parameter.stringify());
      }
      if parameter.predigest.is_some() {
        // TODO: Sometimes we legitimately want to use e.g. Number parameters without the predigest
        // closure... so this shouldn't be an error, not even an info -- but leaving it here
        // if something changes in the future. error!(
        //   target: &s!("parameter:{}", parameter.name),
        //   "parameter with predigest closure was invoked in an expandable context. Parameter
        // digestion won't execute." );
      }
      if macro_call && gullet::argument_runaway() {
        return Ok(None);
      }
      if parameter.is_macro_delimiter() && values.is_none() {
        return Ok(None);
      }
      if !parameter.novalue {
        args.push(values);
      }
    }
    if let Some(tails) = tails {
      tails.hand_back(fordefn);
    }
    Ok(Some(args))
  }

  pub fn read_arguments_and_digest(&self, fordefn: &Constructor) -> Result<Vec<Option<Digested>>> {
    let mut args = Vec::with_capacity(self.0.len());
    // `LXML_TRACE_ARGS=\cs` prints every argument this constructor reads, as
    // reverted tokens, before it is digested — a bisect aid for "which tokens
    // did the `{}` scan swallow" questions (examdesign's version-loop
    // re-execution; wave 14). Off by default.
    let traced = TRACE_ARGS
      .as_deref()
      .is_some_and(|want| fordefn.get_cs().to_string() == want);
    let tails = self.may_defer_tails().then(ArgumentTails::open);
    for parameter in &self.0 {
      let value = parameter.read(Some(fordefn))?;
      if traced {
        let shown = value.revert().map(|t| t.to_string()).unwrap_or_default();
        eprintln!(
          "ARGS {}: {} = {shown}",
          fordefn.get_cs(),
          parameter.stringify()
        );
      }
      if !parameter.novalue {
        let digested_value = parameter.digest(value, Some(fordefn))?;
        args.push(digested_value);
      }
    }
    if let Some(tails) = tails {
      tails.hand_back(Some(fordefn));
    }
    Ok(args)
  }

  /// Re-read an argument already read as tokens (`{Dimension}`, `[Number]`,
  /// `CommaList:Number`: a braced or bracketed argument whose inner spec gives
  /// its type). Perl `Parameters::reparseArgument` (Parameters.pm:78-85).
  ///
  /// What follows the value inside the braces is NOT dropped (Perl's
  /// `readingFromMouth` closes the argument's mouth unread, Gullet.pm:131-135;
  /// KNOWN_PERL_ERRORS #275): TeX leaves it in the input — `\setlength#1#2{#1
  /// #2\relax}` (latex.ltx:10253), through which latex.ltx passes every such
  /// length — so `A\hspace{1em\foo}B` typesets the `x` of `\def\foo{x}`. The
  /// tail is handed back at the end of the command's argument list
  /// (`ArgumentTails`). With calc loaded, a length is evaluated as a whole
  /// expression ([`gullet::braced_length_evaluator`]). OXIDIZED_DESIGN #317.
  pub fn reparse_argument(&self, value: ArgWrap) -> Result<Vec<ArgWrap>> {
    if value.is_none() {
      return Ok(Vec::new());
    }
    let calc = self
      .braced_length_type()
      .zip(gullet::braced_length_evaluator());
    let (values, tail) = read_braced(value.revert()?, || match calc {
      Some((kind, evaluate)) => Ok(vec![ArgWrap::from(coerce_length(evaluate(kind)?, kind))]),
      None => self.read_arguments(None),
    })?;
    ArgumentTails::defer(tail);
    Ok(values)
  }

  /// Whether reading these parameters can defer a tail: only a parameter with
  /// an inner spec re-parses its argument ([`Self::reparse_argument`]). The
  /// others open no `ArgumentTails` scope, which every macro call would pay for.
  fn may_defer_tails(&self) -> bool { self.0.iter().any(|parameter| parameter.inner.is_some()) }

  /// The register type of a lone `Dimension`/`Glue` inner spec: the braced
  /// lengths latex.ltx hands to `\setlength` (see [`gullet::BracedLengthFn`]).
  fn braced_length_type(&self) -> Option<RegisterType> {
    match self.0.as_slice() {
      [only] if only.name == pin!("Dimension") => Some(RegisterType::Dimension),
      [only] if only.name == pin!("Glue") => Some(RegisterType::Glue),
      _ => None,
    }
  }

  pub fn as_keysets(&self) -> Vec<String> { self.0.iter().map(|p| p.stringify()).collect() }
}

/// Read a value from a braced argument's `tokens`, in a mouth of their own,
/// and return it with the argument's TAIL: whatever `read` left unread — what
/// an expanding scan looked ahead at and put back, and the rest of the
/// argument. Perl `readingFromMouth` (Gullet.pm:108-146) drops the tail; TeX
/// has never seen braces here and leaves it in the input, so every caller hands
/// it back (`ArgumentTails`, or [`read_braced_value`]'s callers).
pub fn read_braced<R>(tokens: Tokens, read: impl FnOnce() -> Result<R>) -> Result<(R, Vec<Token>)> {
  let init_if_depth = crate::definition::conditional::if_stack_depth();
  gullet::reading_from_mouth(Mouth::new("", None)?, || {
    gullet::unread(tokens);
    let value = {
      let _braced = BracedRead::enter();
      read()?
    };
    // A conditional opened inside the argument and cut by the scan
    // (`\hspace{\ifx\a\b\Lreg\else\a\U\fi}`, read_dimension stops at `\U`)
    // is expanded to its `\fi` HERE, as TeX's `get_x_token` after `scan_dimen`
    // (tex.web §448/§461) meets it next — the conditional must not outlive the
    // argument's mouth (OXIDIZED_DESIGN #193; witness typog-example,
    // `parbox_dimen_conditional_double.tex`). What the taken branch still
    // holds heads the tail. One expansion at a time, stopping as the frame
    // closes: what follows the `\fi` is expanded where TeX expands it, after
    // the command (`\setlength\x{\ifx…\U\fi\the\x}` reads `\x`'s new value).
    let mut tail = Vec::new();
    while crate::definition::conditional::if_stack_depth() > init_if_depth {
      match gullet::read_token()? {
        Some(token) => {
          if !gullet::expand_once_partial(token)? {
            tail.push(token);
          }
        },
        None => {
          if crate::definition::conditional::if_stack_depth() > init_if_depth {
            // A genuinely unbalanced argument (`\hspace{\iftrue 3pt}`):
            // pdflatex reports "\iftrue … was incomplete", Perl warns at
            // `\end{document}`; say so here, then drop the orphaned frames so
            // they cannot leak outward.
            Warn!(
              "expected",
              "\\fi",
              "Missing \\fi: conditional opened inside an argument fell off its end"
            );
            while crate::definition::conditional::if_stack_depth() > init_if_depth {
              crate::definition::conditional::pop_if_frame()?;
            }
          }
          break;
        },
      }
    }
    if tail.is_empty() {
      gullet::skip_spaces()?;
    }
    tail.extend(gullet::take_rest_of_mouth());
    strip_scanned_error_stubs(&mut tail);
    Ok((value, tail))
  })
}

/// Read a `kind` value from a braced argument's `tokens` the way LaTeX's
/// assignments scan it — `\setlength#1#2{#1 #2\relax}`, `\addtolength`,
/// `\setcounter`, `\addtocounter` (latex.ltx:10253-10254, 10115-10122): by
/// the register's own type, a length through calc when it is loaded (see
/// [`gullet::BracedLengthFn`]). Returns the value and the argument's tail; the
/// caller assigns, then puts the tail back (`gullet::unread_vec`) — exactly
/// where TeX leaves it, after the assignment.
pub fn read_braced_value(
  tokens: Tokens,
  kind: RegisterType,
) -> Result<(RegisterValue, Vec<Token>)> {
  let calc = if matches!(kind, RegisterType::Dimension | RegisterType::Glue) {
    gullet::braced_length_evaluator()
  } else {
    None
  };
  read_braced(tokens, || match calc {
    Some(evaluate) => Ok(coerce_length(evaluate(kind)?, kind)),
    None => gullet::read_value(kind),
  })
}

/// A calc result (always a skip, calc.sty:56) as the `kind` of length read.
fn coerce_length(value: RegisterValue, kind: RegisterType) -> RegisterValue {
  match kind {
    RegisterType::Dimension => RegisterValue::Dimension(value.into()),
    RegisterType::Glue => RegisterValue::Glue(value.into()),
    _ => value,
  }
}

thread_local! {
  /// Braced-argument tails ([`read_braced`]) waiting for their command's
  /// argument list to end, in reading order. See [`ArgumentTails`].
  static ARGUMENT_TAILS: RefCell<Vec<Token>> = const { RefCell::new(Vec::new()) };
  /// How many [`ArgumentTails`] scopes are open.
  static TAIL_SCOPES: Cell<usize> = const { Cell::new(0) };
  /// How many [`read_braced`] reads are running, for [`in_braced_read`].
  static BRACED_READS: Cell<usize> = const { Cell::new(0) };
  /// Why tails are dropped instead of handed back, innermost last
  /// ([`dropping_argument_tails`]).
  static DROPPING_TAILS: RefCell<Vec<&'static str>> = const { RefCell::new(Vec::new()) };
}

/// One running [`read_braced`], counted for [`in_braced_read`]; the count is
/// restored on every exit, a caught panic included (cortex_worker reuses the
/// thread for the next paper).
struct BracedRead;
impl BracedRead {
  fn enter() -> Self {
    BRACED_READS.with(|reads| reads.set(reads.get() + 1));
    BracedRead
  }
}
impl Drop for BracedRead {
  fn drop(&mut self) { BRACED_READS.with(|reads| reads.set(reads.get().saturating_sub(1))); }
}

/// Where a handed-back tail would be misread, so `invoke` runs with the tails
/// its commands leave DROPPED, and warned about, `why` naming what TeX does
/// with the text:
/// - a column type (`\NC@rewrite@p`, alignment.rs): the template reader would
///   read a tail as more column letters (`p{\textwidth-1pt}c` a second `p`
///   column of width `t`); TeX typesets it in every cell (array.sty:189-191
///   `\@startpbox` → `\setlength\hsize{#1}`);
/// - the head of an alignment row, between rows (tex_tables.rs
///   `digest_alignment_column`): a tail would open the next row with a cell
///   of its own, where `\noalign` then fails (`\cmidrule[lr]{1-2}
///   \cmidrule[lr]{3-4}` for `(lr)`: 2605.27476, 2605.25272; pdflatex prints
///   the `lr` between the rows, with 2 errors).
///
/// The drop scope is restored on every exit, a caught panic included.
/// OXIDIZED_DESIGN #317.
pub fn dropping_argument_tails<R>(
  why: &'static str,
  invoke: impl FnOnce() -> Result<R>,
) -> Result<R> {
  struct Dropping;
  impl Drop for Dropping {
    fn drop(&mut self) {
      DROPPING_TAILS.with(|stack| {
        stack.borrow_mut().pop();
      });
    }
  }
  DROPPING_TAILS.with(|stack| stack.borrow_mut().push(why));
  let _dropping = Dropping;
  invoke()
}

/// TeX's treatment of a tail dropped between the rows of an alignment.
pub const BETWEEN_ALIGNMENT_ROWS: &str = "TeX typesets it between the rows";
/// TeX's treatment of a column type's tail.
pub const IN_EVERY_CELL: &str = "TeX typesets it in every cell of the column";

/// Drop `tail`, the rest of an argument of `cs` that has no place to go,
/// with the warning `ArgumentTails` gives when it holds more than spaces and
/// `\relax`.
pub fn drop_argument_tail(cs: &Token, tail: Vec<Token>, why: &str) {
  if let Some(first) = tail.iter().find(|token| !is_inert_tail_token(token)) {
    Warn!(
      "unexpected",
      first.stringify(),
      s!(
        "Unexpected text after the value in an argument of {} ('{}'); it is dropped ({why})",
        cs,
        Tokens::new(tail.clone())
      )
    );
  }
}

/// Whether a parameter reader runs on a braced argument's own mouth
/// ([`read_braced`]), where the rest of the current mouth is the rest of that
/// argument (`DefaultUnits`, base_parameter_types.rs).
pub fn in_braced_read() -> bool { BRACED_READS.with(|reads| reads.get() > 0) }

/// One argument list's share of the deferred braced-argument tails.
///
/// TeX grabs a macro's arguments as token lists and scans a quantity out of one
/// only afterwards (`\@rule[#1]#2#3{… \setlength\@tempdimb{#2}…}`,
/// latex.ltx:16360-16366), so what follows the quantity never meets the next
/// argument. A binding reads each typed argument as it goes, so a tail waits
/// here until the list is complete and is then put back in the input: read
/// next, after the command. For an assignment that is TeX's place (those read
/// their value with [`read_braced_value`] instead, without a warning); a command
/// that outputs a box or space (`\hspace`, `\makebox`, `\parbox`, `\rule`,
/// `minipage`) scans its length BEFORE that output in TeX, so there the tail
/// lands one step late. Either way a tail with more than spaces and `\relax` is
/// almost always an authoring slip (a length expression without calc), and a
/// warning names it. OXIDIZED_DESIGN #317.
struct ArgumentTails {
  mark: usize,
}

impl ArgumentTails {
  fn open() -> Self {
    TAIL_SCOPES.with(|scopes| scopes.set(scopes.get() + 1));
    ArgumentTails {
      mark: ARGUMENT_TAILS.with(|tails| tails.borrow().len()),
    }
  }

  /// Hold `tail` for the argument list being read; with none open (a re-parse
  /// outside a definition's arguments) it goes back in the input at once.
  fn defer(tail: Vec<Token>) {
    if tail.is_empty() {
    } else if TAIL_SCOPES.with(|scopes| scopes.get()) == 0 {
      gullet::unread_vec(tail);
    } else {
      ARGUMENT_TAILS.with(|tails| tails.borrow_mut().extend(tail));
    }
  }

  /// Put the tails deferred since [`Self::open`] back in the input. `fordefn`
  /// is the command whose output the tail now follows, and is warned about;
  /// None for a nested re-parse, whose tail travels on with its enclosing
  /// argument's.
  fn hand_back(self, fordefn: Option<&dyn Definition>) {
    let tail = ARGUMENT_TAILS.with(|tails| tails.borrow_mut().split_off(self.mark));
    if tail.is_empty() {
      return;
    }
    let Some(defn) = fordefn else {
      // A nested re-parse: the tail travels on with its enclosing argument's.
      gullet::unread_vec(tail);
      return;
    };
    if let Some(why) = DROPPING_TAILS.with(|stack| stack.borrow().last().copied()) {
      drop_argument_tail(&defn.get_cs(), tail, why);
      return;
    }
    if let Some(first) = tail.iter().find(|token| !is_inert_tail_token(token)) {
      Warn!(
        "unexpected",
        first.stringify(),
        s!(
          "Unexpected text after the value in an argument of {} ('{}'); it is read after {0}",
          defn.get_cs(),
          Tokens::new(tail.clone())
        )
      );
    }
    gullet::unread_vec(tail);
  }
}

impl Drop for ArgumentTails {
  /// An argument list abandoned by an error or a mismatched macro call (tex.web
  /// §392/§398 drop its arguments) takes its tails with it.
  fn drop(&mut self) {
    ARGUMENT_TAILS.with(|tails| tails.borrow_mut().truncate(self.mark));
    TAIL_SCOPES.with(|scopes| scopes.set(scopes.get().saturating_sub(1)));
  }
}

/// Drop the undefined control sequences at the head of `tail`: the scan's
/// look-ahead expanded them, which reported them and stubbed them as
/// `<ltx:ERROR>` (`generate_error_stub`), and TeX discards an undefined control
/// sequence it expands (tex.web §370) — it must not come back as text.
/// `\hspace{6\@p@t}` with USG.cls's `\@p@t` missing (2605.25073). Spaces the
/// look-ahead passed on the way go with them.
fn strip_scanned_error_stubs(tail: &mut Vec<Token>) {
  loop {
    let Some(first) = tail.iter().position(|t| t.get_catcode() != Catcode::SPACE) else {
      return;
    };
    if !is_error_stub(&tail[first]) {
      return;
    }
    tail.drain(..=first);
  }
}

/// A tail token that does nothing when read: a space or `\relax` (`\stretch{1}`
/// = `0pt plus 1fill\relax` leaves its `\relax`, which TeX's own `\relax` after
/// `#2` would have ended the scan with).
fn is_inert_tail_token(token: &Token) -> bool {
  token.get_catcode() == Catcode::SPACE || token.defined_as(&crate::token::TOKEN_RELAX)
}

impl fmt::Display for Parameters {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut content = String::new();
    for parameter in &self.0 {
      let param_content = parameter.to_string();
      if LAST_WCHAR_RE.is_match(&content) && FIRST_WCHAR_RE.is_match(&param_content) {
        content.push(' ');
      }
      content.push_str(&param_content);
    }
    write!(f, "{content}")
  }
}

impl From<Parameters> for Vec<Parameter> {
  fn from(ps: Parameters) -> Vec<Parameter> { ps.0 }
}

// ToTokens impls gated by `codegen` feature — see comment in
// `tokens.rs` for rationale (audit DEP-14, 2026-05-18).
#[cfg(feature = "codegen")]
impl ToTokens for Parameters {
  fn to_tokens(&self, stream: &mut TokenStream) {
    let params = &self.0;
    stream.extend(quote! {
        Parameters::new(<[Parameter]>::into_vec(Box::new([ #(#params),* ])))
    });
  }
}

#[cfg(feature = "codegen")]
impl ToTokens for Parameter {
  fn to_tokens(&self, stream: &mut TokenStream) {
    let name = arena::with(self.name, |name| quote!(arena::pin_static(#name)));
    let spec = arena::with(self.spec, |spec| quote!(arena::pin_static(#spec)));
    let extra = &self.extra;
    let inner = match &self.inner {
      None => quote!(None),
      Some(inner_ps) => quote!(Some(#inner_ps)),
    };
    stream.extend(quote! {
      Parameter {
        name: #name,
        spec: #spec,
        extra: <[Tokens]>::into_vec(Box::new([ #(#extra),* ])),
        inner: #inner,
        ..Parameter::default()
      }
    });
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parameter_default_has_expected_fields() {
    let p = Parameter::default();
    assert!(!p.novalue);
    assert!(p.semiverbatim.is_none());
    assert!(!p.optional);
    assert!(p.inner.is_none());
    assert!(p.extra.is_empty());
    assert!(p.before_digest.is_empty());
    assert!(p.after_digest.is_empty());
    assert!(p.predigest.is_none());
    assert!(p.reversion.is_none());
    assert!(p.digested_reversion.is_none());
    assert_eq!(arena::to_string(p.name), "parameter_default");
    assert_eq!(arena::to_string(p.spec), "");
  }

  #[test]
  fn parameter_display_is_name() {
    let p = Parameter {
      name: arena::pin("Plain"),
      ..Default::default()
    };
    assert_eq!(format!("{p}"), "Plain");
  }

  #[test]
  fn parameter_stringify_is_spec() {
    let p = Parameter {
      spec: arena::pin("{}"),
      ..Default::default()
    };
    assert_eq!(p.stringify(), "{}");
  }

  #[test]
  fn parameter_partial_eq_by_name() {
    // PartialEq compares by name only — Perl parity (closures can't
    // be structurally compared).
    let mut a = Parameter::default();
    let mut b = Parameter::default();
    a.name = arena::pin("x");
    b.name = arena::pin("x");
    assert_eq!(a, b);
    b.name = arena::pin("y");
    assert_ne!(a, b);
  }

  #[test]
  fn parameters_new_and_take() {
    let p = Parameter::default();
    let ps = Parameters::new(vec![p]);
    let taken = ps.take_parameters();
    assert_eq!(taken.len(), 1);
  }

  #[test]
  fn parameters_get_num_args_counts_valued() {
    // novalue=true parameters don't count toward num_args.
    let mut a = Parameter::default();
    let mut b = Parameter::default();
    let mut c = Parameter::default();
    a.novalue = false;
    b.novalue = true;
    c.novalue = false;
    let ps = Parameters::new(vec![a, b, c]);
    assert_eq!(ps.get_num_args(), 2);
  }

  #[test]
  fn parameters_empty() {
    let ps = Parameters::new(vec![]);
    assert_eq!(ps.get_num_args(), 0);
    assert_eq!(ps.get_parameters().len(), 0);
  }

  #[test]
  fn parameters_get_parameters_returns_refs_to_all() {
    // get_parameters returns ALL, including novalue ones (num_args
    // filters; get_parameters doesn't).
    let mut a = Parameter::default();
    let mut b = Parameter::default();
    a.novalue = false;
    b.novalue = true;
    let ps = Parameters::new(vec![a, b]);
    assert_eq!(ps.get_parameters().len(), 2);
  }
}
