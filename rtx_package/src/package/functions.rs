use lazy_static::lazy_static;
use libxml::tree::Node;
use log::*;
use regex::Regex;
use std::borrow::Cow;
use std::collections::{VecDeque, HashMap};
use std::path::Path;
use std::rc::Rc;
use unidecode::unidecode;

use rtx_core::common::error::*;
use rtx_core::common::font::Font;
use rtx_core::common::number::Number;
use rtx_core::common::xml::XML_NS;
use rtx_core::definition::conditional::{Conditional, ConditionalOptions, ConditionalType};
use rtx_core::definition::constructor::{Constructor, ConstructorOptions};
use rtx_core::definition::expandable::{Expandable, ExpandableOptions};
use rtx_core::definition::math_primitive::{MathPrimitive, MathPrimitiveOptions};
use rtx_core::definition::primitive::{Primitive, PrimitiveOptions};
use rtx_core::definition::register::{NumericOps, Register, RegisterGetterClosure, RegisterSetterClosure, RegisterType, RegisterValue};
use rtx_core::common::glue::{Glue, MuGlue};
use rtx_core::common::dimension::{Dimension, MuDimension};
use rtx_core::definition::{
  BeforeDigestClosure, ConditionalClosure, ConstructionClosure, Definition, DigestionClosure, ExpansionBody, PrimitiveClosure, ReplacementClosure,
};
use rtx_core::document::resource::*;
use rtx_core::document::tag::{TagOptionName, TagOptions};
use rtx_core::document::Document;
use rtx_core::gullet::Gullet;
use rtx_core::mouth;
use rtx_core::mouth::{Mouth, MouthOptions};
use rtx_core::parameter::{Parameter, ParameterExtra, Parameters};
use rtx_core::state::{Scope, State, Stored};
use rtx_core::stomach::Stomach;
use rtx_core::tbox::Tbox;
use rtx_core::token::*;
use rtx_core::tokens::Tokens;
use rtx_core::util::pathname;
use rtx_core::util::pathname::PathnameFindOptions;
use rtx_core::whatsit::Whatsit;
use rtx_core::BoxOps;
use rtx_core::{Core, Digested};

use super::pool;

#[allow(clippy::trivial_regex)]
lazy_static! {
  static ref CSNAME_MACRO_RE: Regex = Regex::new(r"^\\csname\s+(.*)\\endcsname").unwrap();
  static ref CS_RE: Regex = Regex::new(r"^(\\[a-zA-Z@]+)").unwrap();
  static ref SINGLE_CHAR_RE: Regex = Regex::new(r"^(\\.)").unwrap();
  static ref ACTIVE_CHAR_RE: Regex = Regex::new(r"^(.)").unwrap();
  static ref CONDITIONAL_RE: Regex = Regex::new(r"^\\(?:if(.*)|unless)$").unwrap();
  static ref LEADING_PROTOCOL_RE: Regex = Regex::new(r"^\w+:").unwrap();
  static ref TRAILING_SLASH_RE: Regex = Regex::new(r"/$").unwrap();
  static ref SPACES_RE: Regex = Regex::new(r"\s+").unwrap();
  static ref DIRTY_ID_IDIOM_RE: Regex = Regex::new(r"\$\{\}\^\{(?P<label>[^\}]*)\}\$").unwrap();
  static ref NESTED_CHECK_RE: Regex = Regex::new(r"^(\{([^\}]*)\})\s*").unwrap();
  static ref OPTIONAL_CHECK_RE: Regex = Regex::new(r"^(\[([^\]]*)\])\s*").unwrap();
  static ref DEFAULT_CHECK_RE: Regex = Regex::new(r"^Default:(.*)$").unwrap();
  static ref PARAMSPECT_CHECK_RE: Regex = Regex::new(r"^((\w*)(:([^\s\{\[]*))?)\s*").unwrap();
  static ref NON_ID_CHARSET_RE: Regex = Regex::new(r"[^\w_\-.]+").unwrap();
  static ref TILDE_NOISE_RE: Regex = Regex::new(r"\\~\{\}").unwrap();
}

pub trait IntoOption<T>: Sized {
  /// Performs the conversion.
  fn into_option(self) -> T;
}

impl<'a> IntoOption<Option<String>> for &'a str {
  fn into_option(self) -> Option<String> { Some(self.to_string()) }
}

impl<T> IntoOption<Option<T>> for Option<T> {
  fn into_option(self) -> Option<T> { self }
}

impl IntoOption<bool> for bool {
  fn into_option(self) -> bool { self }
}

impl<T> IntoOption<Option<Vec<T>>> for Vec<T> {
  fn into_option(self) -> Option<Vec<T>> { Some(self) }
}

impl<T> IntoOption<Option<VecDeque<T>>> for VecDeque<T> {
  fn into_option(self) -> Option<VecDeque<T>> { Some(self) }
}

pub trait IntoTokensResult<T>: Sized {
  /// Performs the conversion, used for DefMacro return values etc
  fn into_tokens_result(self) -> Result<Tokens>;
}

impl IntoTokensResult<Result<Tokens>> for Token {
  fn into_tokens_result(self) -> Result<Tokens> { Ok(Tokens!(self)) }
}

impl IntoTokensResult<Result<Tokens>> for Vec<Token> {
  fn into_tokens_result(self) -> Result<Tokens> { Ok(Tokens(self)) }
}

impl IntoTokensResult<Result<Tokens>> for Tokens {
  fn into_tokens_result(self) -> Result<Tokens> { Ok(self) }
}

impl IntoTokensResult<Result<Tokens>> for Result<Tokens> {
  fn into_tokens_result(self) -> Result<Tokens> { self }
}

impl IntoTokensResult<Result<Tokens>> for () {
  fn into_tokens_result(self) -> Result<Tokens> { Ok(Tokens!()) }
}

pub trait IntoBoolResult<T>:Sized {
  /// Performs the conversion, used for DefConditional return values etc
  fn into_bool_result(self) -> Result<bool>;
}
impl IntoBoolResult<Result<bool>> for bool {
  fn into_bool_result(self) -> Result<bool> { Ok(self) }
}
impl IntoBoolResult<Result<bool>> for Result<bool> {
  fn into_bool_result(self) -> Result<bool> { self }
}

pub trait IntoDigestedResult<T>: Sized {
  /// Performs the conversion, used for DefPrimitive return values etc
  fn into_digested_result(self) -> Result<Vec<Digested>>;
}
impl IntoDigestedResult<Result<Vec<Digested>>> for () {
  fn into_digested_result(self) -> Result<Vec<Digested>> { Ok(Vec::new()) }
}
impl IntoDigestedResult<Result<Vec<Digested>>> for Tbox {
  fn into_digested_result(self) -> Result<Vec<Digested>> { Ok(vec![self.into()]) }
}

impl IntoDigestedResult<Result<Vec<Digested>>> for Digested {
  fn into_digested_result(self) -> Result<Vec<Digested>> { Ok(vec![self]) }
}

impl IntoDigestedResult<Result<Vec<Digested>>> for Vec<Digested> {
  fn into_digested_result(self) -> Result<Vec<Digested>> { Ok(self) }
}

impl IntoDigestedResult<Result<Vec<Digested>>> for Result<Vec<Digested>> {
  fn into_digested_result(self) -> Result<Vec<Digested>> { self }
}

pub trait IntoRegisterValueOption<T>: Sized {
  fn into_register_value_option(self) -> Option<RegisterValue>;
}
impl IntoRegisterValueOption<Option<RegisterValue>> for () {
  fn into_register_value_option(self) -> Option<RegisterValue> { None }
}
impl IntoRegisterValueOption<Option<RegisterValue>> for Option<RegisterValue> {
  fn into_register_value_option(self) -> Option<RegisterValue> { self }
}
impl IntoRegisterValueOption<Option<RegisterValue>> for Number {
  fn into_register_value_option(self) -> Option<RegisterValue> { Some(RegisterValue::Number(self)) }
}

impl IntoRegisterValueOption<Option<RegisterValue>> for Option<Number> {
  fn into_register_value_option(self) -> Option<RegisterValue> { 
    match self {
      Some(n) => Some(RegisterValue::Number(n)),
      None => None
    }
  }
}

// Convenience methods for predigest closures that require Result<Option<Digested>>
pub trait IntoDigestedOptionResult<T>: Sized {
  fn into_digested_option_result(self) -> Result<Option<Digested>>;
}

impl IntoDigestedOptionResult<Result<Option<Digested>>> for Glue {
  fn into_digested_option_result(self) -> Result<Option<Digested>> { RegisterValue::Glue(self).into_digested_option_result() }
}
impl IntoDigestedOptionResult<Result<Option<Digested>>> for MuGlue {
  fn into_digested_option_result(self) -> Result<Option<Digested>> { RegisterValue::MuGlue(self).into_digested_option_result() }
}
impl IntoDigestedOptionResult<Result<Option<Digested>>> for Dimension {
  fn into_digested_option_result(self) -> Result<Option<Digested>> { RegisterValue::Dimension(self).into_digested_option_result() }
}
impl IntoDigestedOptionResult<Result<Option<Digested>>> for MuDimension {
  fn into_digested_option_result(self) -> Result<Option<Digested>> { RegisterValue::MuDimension(self).into_digested_option_result() }
}

impl IntoDigestedOptionResult<Result<Option<Digested>>> for Number {
  fn into_digested_option_result(self) -> Result<Option<Digested>> { RegisterValue::Number(self).into_digested_option_result() }
}



impl IntoDigestedOptionResult<Result<Option<Digested>>> for RegisterValue {
  fn into_digested_option_result(self) -> Result<Option<Digested>> { Ok(Some(self.into())) }
}
impl IntoDigestedOptionResult<Result<Option<Digested>>> for Option<Digested> {
  fn into_digested_option_result(self) -> Result<Option<Digested>> { Ok(self) }
}
impl IntoDigestedOptionResult<Result<Option<Digested>>> for Result<Option<Digested>> {
  fn into_digested_option_result(self) -> Result<Option<Digested>> { self }
}


//**********************************************************************
//   Initially, I thought LaTeXML Packages should try to be like perl modules:
// once loaded, you didn't need to re-load them, only `initialize' them to
// install their definitions into the current stomach.  I tried to achieve
// that through various package tricks.
//    But ultimately, most of a package _is_ installing defns in the stomach,
// and it's probably better to allow a more TeX-like evaluation of definitions
// in order, so \let and such work as expected.
//    So, it got simpler!
// Still, it would be nice if there were `compiled' forms of .ltxml files!
//**********************************************************************

/// Is defined in the `LaTeX`-y sense of also not being let to \relax.
pub fn is_defined(name: &str, state: &mut State) -> bool {
  let cs = T_CS!(name);
  is_defined_token(&cs, state)
}

pub fn is_defined_token(cs: &Token, state: &mut State) -> bool {
  let meaning = state.lookup_meaning(cs);
  match meaning {
    Some(store) => match store {
      Stored::Token(ref m) => true,
      Stored::Expandable(ref m) => m.get_cs_name() != "\\relax",
      Stored::Primitive(ref m) => m.get_cs_name() != "\\relax",
      Stored::Constructor(ref m) => m.get_cs_name() != "\\relax",
      _ => false,
    },
    _ => false,
  }
}

pub fn load_external_binding(file: &str, state: &mut State, mut with_stomach: Option<&mut Stomach>) -> Result<bool> {
  let taken_dispatcher = state.extra_bindings_dispatch.take();
  match taken_dispatcher {
    Some(ref dispatcher) => {
      let result_opt = match with_stomach {
        None => dispatcher(&file, state, None),
        Some(ref mut st) => dispatcher(&file, state, Some(st)),
      };
      match result_opt {
        Some(result) => match result {
          Ok(()) => true,
          Err(e) => return Err(e),
        },
        None => false,
      }
    },
    None => false,
  };

  let is_contrib: bool = match taken_dispatcher {
    Some(ref dispatcher) => {
      note_begin(&s!("Loading {:?} definitions", file));
      let result_opt = match with_stomach {
        None => dispatcher(&file, state, None),
        Some(ref mut st) => dispatcher(&file, state, Some(st)),
      };
      note_end(&s!("Loading {:?} definitions", file));
      match result_opt {
        Some(result) => match result {
          Ok(()) => true,
          Err(e) => return Err(e),
        },
        None => false,
      }
    },
    None => false,
  };
  state.extra_bindings_dispatch = taken_dispatcher;

  Ok(is_contrib)
}

/// TODO: Flesh out with the full infrastructure, incremental functionality for now.
pub fn input_definitions(raw_file: &str, options: InputDefinitionOptions, mut state: &mut State) -> Result<()> {
  let mut file: String = raw_file.trim().to_string();

  // let prevname = if options.handleoptions {
  //   match state.lookup_definition(T_CS!("\@currname")) {
  //     Some(Stored::Expandable(name)) => Digest!(T_CS!("\@currname")).to_string()
  // }
  // let prevext = options.handleoptions && $state->lookupDefinition(T_CS!('\@currext')) &&
  // ToString(Digest(T_CS!('\@currext')));

  // Compute the exact name based on the type
  file = match options.extension {
    None => file,
    Some(ext) => file + "." + &ext,
  };
  let mut with_stomach = options.with_stomach;
  let loaded_flag = file.clone() + "_loaded";
  {
    // Only load definitions once
    if let Some(&Stored::Bool(flag)) = state.lookup_value(&loaded_flag) {
      if flag {
        // do nothing if we've loaded before
        return Ok(());
      }
    }
  }

  // Mark as loaded, then process the definitions
  note_begin(&s!("Loading {:?} definitions", file));
  state.assign_value(&loaded_flag, true, Some(Scope::Global));

  let is_contrib = match with_stomach {
    None => load_external_binding(&file, state, None)?,
    Some(ref mut stomach_mut) => load_external_binding(&file, state, Some(stomach_mut))?,
  };

  if !is_contrib {
    match file.as_ref() {
      "TeX.pool" => pool::tex::load_definitions(&mut state, with_stomach)?,
      "LaTeX.pool" => pool::latex::load_definitions(&mut state, with_stomach)?,
      "eTeX.pool" => pool::etex::load_definitions(&mut state, with_stomach)?,
      "pdfTeX.pool" => pool::pdftex::load_definitions(&mut state, with_stomach)?,
      "article.cls" => pool::article_cls::load_definitions(&mut state, with_stomach)?,
      "alltt.sty" => pool::alltt_sty::load_definitions(&mut state, with_stomach)?,
      "amsmath.sty" => pool::amsmath_sty::load_definitions(&mut state, with_stomach)?,
      "amsthm.sty" => pool::amsthm_sty::load_definitions(&mut state, with_stomach)?,
      "comment.sty" => pool::comment_sty::load_definitions(&mut state, with_stomach)?,
      "IEEEtran.cls" => pool::ieeetran_cls::load_definitions(&mut state, with_stomach)?,
      "url.sty" => pool::url_sty::load_definitions(&mut state, with_stomach)?,
      "verbatim.sty" => pool::verbatim_sty::load_definitions(&mut state, with_stomach)?,

      other => fatal!(Package, Unknown, s!("TODO: unknown binding {:?}, can't load", other)),
    };
  }
  note_end(&s!("Loading {:?} definitions", file));
  Ok(())
}

pub fn input_content(core: &mut Core, request: &str) -> Result<()> {
  match find_file(request, None, &mut core.state) {
    // TODO: type => $options{type}, noltxml => 1
    Some(path) => load_tex_content(core, &path),
    None => fatal!(Package, MissingFile, request),
    /* TODO:
     * Error("missing_file", request, state.get_stomach().get_gullet(),
     * "Can't find TeX file "+request, maybeReportSearchPaths(state))) */
  }
}

pub fn input(file: String, gullet: &mut Gullet, state: &mut State) {
  unimplemented!();
}

pub fn load_tex_content(core: &mut Core, path: &str) -> Result<()> {
  // If there is a file-specific declaration file (name_tex.rs), load it first!
  // let namespace = path;
  // state.extra_bindings_dispatch
  if !pathname::is_literaldata(path) {
    let (dir, base, ext) = pathname::split(path);
    load_external_binding(&base, &mut core.state, Some(&mut core.stomach.borrow_mut()))?;
  }
  // TODO: Caching
  // content => LookupValue($pathname . '_contents')

  // Open a mouth for that TeX content
  core.stomach.borrow_mut().get_gullet_mut().open_mouth(
    Mouth::create(
      path,
      MouthOptions {
        notes: true,
        ..MouthOptions::default()
      },
      &mut core.state,
    )?,
    true,
  );
  Ok(())
}

pub struct RequireOptions<'a> {
  pub options: Vec<String>,
  pub withoptions: bool,
  pub extension: Option<String>,
  pub as_class: bool,
  pub noltxml: bool,
  pub notex: bool,
  pub raw: bool,
  pub after: bool,
  pub with_stomach: Option<&'a mut Stomach>,
}
impl<'a> Default for RequireOptions<'a> {
  fn default() -> Self {
    RequireOptions {
      options: Vec::new(),
      withoptions: false,
      extension: None,
      as_class: false,
      noltxml: false,
      notex: true,
      raw: false,
      after: false,
      with_stomach: None,
    }
  }
}

/// This (and `FindFile`) needs to evolve a bit to support reading raw .sty (.def, etc) files from
/// the standard texmf directories.  Maybe even use kpsewhich itself (INSTEAD of `pathname_find`
/// ???) Another potentially useful option might be that if we are reading a raw file,
/// perhaps it should just get digested immediately, since it shouldn't contribute any boxes.
pub fn require_package(name: &str, mut options: RequireOptions, state: &mut State) -> Result<()> {
  if options.raw {
    options.raw = false;
    // Warn('deprecated', 'raw', $STATE->getStomach->getGullet,
    //   "RequirePackage option raw is obsolete; it is not needed");
  }

  // We'll usually disallow raw TeX, unless the option explicitly given, or globally set.
  // $options{notex} = 1
  //   if !defined $options{notex} && !LookupValue('INCLUDE_STYLES') && !$options{noltxml};
  if options.extension.is_none() {
    options.extension = Some(String::from("sty"));
  }
  // TODO: Ideally we want to use the same struct for the RequirePackage options as for the
  // InputDefinitions options
  input_definitions(
    name,
    InputDefinitionOptions {
      extension: options.extension,
      handleoptions: true,
      with_stomach: options.with_stomach,
      // Pass classes options if we have NONE!
      withoptions: options.options,
      ..InputDefinitionOptions::default()
    },
    state,
  )
}

pub fn require_resource(mut resource: Resource, state: &mut State) {
  if resource.name.is_empty() && resource.content.is_empty() {
    warn!(target: "expected:resource", "Resource must have a resource pathname or content; skipping");
    return;
  }
  if resource.mimetype.is_empty() && !resource.name.is_empty() {
    let ext = pathname::extension(&resource.name);
    resource.mimetype = resource_type(&ext);
  }
  if resource.mimetype.is_empty() {
    warn!(target: "expected:mime-type", "Resource must have a mime-type; skipping");
    return;
  }

  // If we've got a document, go ahead & put the resource in.
  // if (state.document.is_some()) {
  //   state.document.as_mut().unwrap().add_resource(resource, resource);
  // } else {
  state.pending_resources.push(resource);
  // }
}

pub fn load_class(name: &str, options: Vec<String>, after: Tokens, with_stomach: Option<&mut Stomach>, state: &mut State) -> Result<()> {
  input_definitions(
    name,
    InputDefinitionOptions {
      extension: Some(String::from("cls")),
      after,
      notex: true,
      handleoptions: true,
      noerror: true,
      with_stomach,
      ..InputDefinitionOptions::default()
    },
    state,
  )
  // if (let success = InputDefinitions($class, type => 'cls', notex => 1, handleoptions => 1,
  // noerror => 1,     %options)) {
  //   return $success; }
  // else {
  //   $STATE->noteStatus(missing => $class . '.cls');
  //   let alternate = 'OmniBus';    # was 'article'
  //   Warn('missing_file', $class, $STATE->getStomach->getGullet,
  //     "Can't find binding for class $class (using $alternate)",
  //     maybeReportSearchPaths());
  // if (let success = InputDefinitions($alternate, type => 'cls', noerror => 1, handleoptions =>
  // 1, %options)) {     return $success; }
  //   else {
  //     Fatal('missing_file', $alternate . '.cls.ltxml', $STATE->getStomach->getGullet,
  //       "Can't find binding for class $alternate (installation error)");
  //     return; } } }
}

pub struct FindFileOptions {
  forbid_ltxml: bool,
  raw: bool,
  notex: bool,
  ext_type: Option<String>,
}

impl Default for FindFileOptions {
  fn default() -> Self {
    FindFileOptions {
      forbid_ltxml: false,
      raw: false,
      notex: false,
      ext_type: None,
    }
  }
}

pub fn find_file(file: &str, options: Option<FindFileOptions>, state: &mut State) -> Option<String> {
  let mut options = options.unwrap_or_default();
  if options.raw {
    options.raw = false;
    warn!(target: "deprecated:raw", "FindFile option raw is deprecated; it is not needed");
  }

  if pathname::is_literaldata(file) {
    // If literal protocol return immediately (unless notex!)
    if options.notex {
      None
    } else {
      // TODO: Consider returning a Cow<str> instead to optimize
      Some(file.to_string())
    }
  } else if pathname::is_literaldata(file) || pathname::is_url(file) {
    // If a known special protocol return immediately
    Some(file.to_string())
  } else if let Some(ref ext) = options.ext_type {
    // Otherwise, it's some kind of "real" file, and we might have to search for it
    // Specific type requested? Search for it.
    // Add the extension, if it isn't already there.
    let aux_file = if file.ends_with(ext) { file.to_string() } else { s!("{}.{}", file, ext) };
    find_file_aux(&aux_file, &options, state)
  } else if file.ends_with(".tex") {
    // If no type given, we MAY expect .tex, or maybe NOT!!
    // No requested type, then .tex; Of course, it may already have it!
    find_file_aux(file, &options, state)
  } else {
    match find_file_aux(&s!("{}.tex", file), &options, state) {
      None => find_file_aux(file, &options, state),
      Some(f) => Some(f),
    }
  }
}

pub fn find_file_aux(file: &str, options: &FindFileOptions, state: &mut State) -> Option<String> {
  // If cached, return simple path (it's a key into the cache)
  let cached = state.lookup_string(&s!("{}_contents", file));
  if !cached.is_empty() {
    Some(file.to_string())
  } else if pathname::is_absolute(file) {
    // And if we've got an absolute path,
    if !options.forbid_ltxml && Path::new(&s!("{}.ltxml", file)).exists() {
      // No need to search, just check if it exists.
      Some(s!("{}.ltxml", file))
    } else if Path::new(file).exists() {
      // No need to search, just check if it exists.
      Some(file.to_string())
    } else {
      // otherwise we're never going to find it.
      None
    }
  } else if pathname::is_nasty(file) {
    // If it is a nasty filename, we won't touch it.
    // we DO NOT want to pass this to kpathse or such!
    None
  } else {
    // Note that the strategy is complicated by the fact that
    // (1) we prefer .ltxml bindings, if present
    // (2) those MAY be present in kpsewhich's DB (although our searchpaths take precedence!)
    // (3) BUT we want to avoid kpsewhich if we can, since it's slower
    // (4) depending on switches we may EXCLUDE .ltxml OR raw tex OR allow both.
    let paths: Vec<String> = state.search_paths.iter().cloned().collect();
    let urlbase = state.lookup_value("URLBASE");
    let nopaths = state.lookup_bool("REMOTE_REQUEST");
    let ltxml_paths: Vec<String> = if nopaths { vec![] } else { paths.clone() };

    // If we're looking for ltxml, look within our paths & installation first (faster than kpse)
    if !options.forbid_ltxml {
      if let Some(path) = pathname::find(
        &s!("{}.ltxml", file),
        NewDefaultV!(PathnameFindOptions, paths => ltxml_paths, installation_subdir => "Package"),
      ) {
        return Some(path);
      }
    }
    // If we're looking for TeX, look within our paths & installation first (faster than kpse)
    if !options.notex {
      if let Some(path) = pathname::find(file, NewDefaultV!(PathnameFindOptions, paths => paths)) {
        return Some(path);
      }
    }
    // Otherwise, pass on to kpsewhich
    // Depending on flags, maybe search for ltxml in texmf or for plain tex in ours!
    // The main point, though, is to we make only ONE (more) call.
    // return if grep { pathname::is_nasty($_) } @$paths;    // SECURITY! No nasty paths in cmdline
    //       // Do we need to sanitize these environment variables?
    // my $kpsewhich = which($ENV{LATEXML_KPSEWHICH} || 'kpsewhich');
    // local $ENV{TEXINPUTS} = join($Config::Config{'path_sep'},
    //   @$paths, $ENV{TEXINPUTS} || $Config::Config{'path_sep'});
    // my @candidates = (((!$options{noltxml} && !$nopaths) ? ("$file.ltxml") : ()),
    //   (!$options{notex} ? ($file) : ()));
    // if (my $result = pathname::kpsewhich(@candidates)) {
    //   return (-f $result ? $result : undef); }
    // if ($urlbase && ($path = url_find($file, urlbase => $urlbase))) {
    //   return $path; }
    // return; }
    info!("No path found for: {:?}", file);
    None
  }
}

pub fn coerce_cs(t: &str) -> Token { T_CS!(t) }

pub fn parse_prototype(proto: &str, state: &mut State) -> Result<((Token, Option<Parameters>))> {
  let mut cs = T_CS!(s!("\\")); // Should never happen
  let mut final_proto = if CSNAME_MACRO_RE.is_match(proto) {
    let captures = CSNAME_MACRO_RE.captures(proto).unwrap();
    cs = T_CS!(s!("\\") + captures.get(0).map_or("", |m| m.as_str()));
    // also replace in proto
    CSNAME_MACRO_RE.replace(proto, "").to_string()
  } else if CS_RE.is_match(proto) {
    // Match a cs
    let captures = CS_RE.captures(proto).unwrap();
    let csname = captures.get(0).map_or("", |m| m.as_str()).to_string();
    cs = T_CS!(csname);
    // also replace in proto
    CS_RE.replace(proto, "").to_string()
  } else if SINGLE_CHAR_RE.is_match(proto) {
    // Match a single char cs, env name,...
    let captures = SINGLE_CHAR_RE.captures(proto).unwrap();
    cs = T_CS!(captures.get(0).map_or("", |m| m.as_str()).to_string());
    // also replace in proto
    SINGLE_CHAR_RE.replace(proto, "").to_string()
  } else if ACTIVE_CHAR_RE.is_match(proto) {
    // Match an active char
    let captures = ACTIVE_CHAR_RE.captures(proto).unwrap();
    cs = TokenizeInternal!(captures.get(0).map_or("", |m| m.as_str()), state)
      .unlist()
      .first()
      .unwrap()
      .clone();
    // also replace in proto
    ACTIVE_CHAR_RE.replace(proto, "").to_string()
  } else {
    // Fatal('misdefined', prototype, $STATE->getStomach,
    //   "Definition prototype doesn't have proper control sequence: \"prototype\""); }
    proto.to_string()
  };
  final_proto = final_proto.trim_start().to_string();
  let paramlist = parse_parameters(final_proto, &cs, state)?;
  Ok((cs, paramlist))
}

pub fn parse_parameters(mut prototype: String, cs: &Token, state: &mut State) -> Result<Option<Parameters>> {
  let mut parameters = Vec::new();
  while !prototype.is_empty() {
    let mut next_proto: String;
    // Handle possibly nested cases, such as {Number}
    if NESTED_CHECK_RE.is_match(&prototype) {
      let captures = NESTED_CHECK_RE.captures(&prototype).unwrap();
      next_proto = NESTED_CHECK_RE.replace(&prototype, "").to_string();
      let spec = captures.get(1).map_or("", |m| m.as_str());
      let inner_spec = captures.get(2).map_or("", |m| m.as_str());
      let inner: Option<Parameters> = if inner_spec.is_empty() {
        None
      } else {
        parse_parameters(inner_spec.to_string(), cs, state)?
      };
      parameters.push(
        Parameter {
          name: s!("Plain"),
          spec: spec.to_string(),
          extra: vec![inner.into()],
          ..Parameter::default()
        }
        .init(state)?,
      );
    } else if let Some(captures) = OPTIONAL_CHECK_RE.captures(&prototype) {
      // Ditto for Optional
      let spec = captures.get(1).map_or("", |m| m.as_str());
      let inner_spec = captures.get(2).map_or("", |m| m.as_str());
      next_proto = OPTIONAL_CHECK_RE.replace(&prototype, "").to_string();
      if let Some(default_captures) = DEFAULT_CHECK_RE.captures(inner_spec) {
        // TODO: Add the defaults !
        parameters.push(
          Parameter {
            name: s!("Optional"),
            spec: spec.to_string(),
            // extra: vec![TokenizeInternal!(default_captures.get(0).map_or("", |m| m.as_str())), None]});
            extra: Vec::new(),
            ..Parameter::default()
          }
          .init(state)?,
        );
      } else if !inner_spec.is_empty() {
        parameters.push(
          Parameter {
            name: s!("Optional"),
            spec: spec.to_string(),
            extra: vec![
              ParameterExtra::ParametersOption(None),
              parse_parameters(inner_spec.to_string(), cs, state)?.into(),
            ],
            ..Parameter::default()
          }
          .init(state)?,
        );
      } else {
        parameters.push(
          Parameter {
            name: s!("Optional"),
            spec: spec.to_string(),
            extra: Vec::new(),
            ..Parameter::default()
          }
          .init(state)?,
        );
      }
    } else if let Some(captures) = PARAMSPECT_CHECK_RE.captures(&prototype) {
      let spec = captures.get(1).map_or("", |m| m.as_str()).to_string();
      let name = captures.get(2).map_or("", |m| m.as_str()).to_string();
      let extra_str = captures.get(4).map_or("", |m| m.as_str()).to_string();
      next_proto = PARAMSPECT_CHECK_RE.replace(&prototype, "").to_string();
      // TODO: Ask Bruce about the "extra" functionality and its types
      let extra = extra_str
        .split('|')
        .flat_map(|t| mouth::tokenize_internal(t, None).unlist())
        .map(|t| t.into())
        .collect::<Vec<ParameterExtra>>();
      parameters.push(
        Parameter {
          name,
          spec,
          extra,
          ..Parameter::default()
        }
        .init(state)?,
      );
    } else {
      fatal!(
        Parameter,
        Misdefined,
        s!("Unrecognized parameter specification at \"prototype\" {:?}", cs)
      );
    }
    prototype = next_proto.to_string();
  }
  if parameters.is_empty() {
    Ok(None)
  } else {
    Ok(Some(Parameters { params: parameters }))
  }
}

pub fn revert(_arg: &[Token]) -> Tokens { unimplemented!() }

//======================================================================
// Declaring and Adjusting the Document Model.
//======================================================================

pub fn install_tag(tag: &str, mut properties: TagOptions, state: &mut State) {
  let mut options = state.tag_properties.entry(tag.to_string()).or_insert_with(TagOptions::default);
  if properties.auto_open.is_some() {
    options.auto_open = properties.auto_open;
  }
  if properties.auto_close.is_some() {
    options.auto_close = properties.auto_close;
  }

  for name in &TagOptionName::all() {
    if name.is_prepend() {
      options.prepend(name, properties.remove(name));
    } else if name.is_append() {
      options.append(name, properties.remove(name));
    } else {
      // we'll handle the regular ones out of the loop
    }
  }
}

pub struct InputDefinitionOptions<'a> {
  pub extension: Option<String>,
  pub options: Vec<String>,
  pub after: Tokens,
  pub notex: bool,
  pub noerror: bool,
  pub noltxml: bool,
  pub withoptions: Vec<String>,
  pub handleoptions: bool,
  pub as_class: bool,
  pub with_stomach: Option<&'a mut Stomach>,
}
impl<'a> Default for InputDefinitionOptions<'a> {
  fn default() -> Self {
    InputDefinitionOptions {
      extension: None,
      options: Vec::new(),
      after: Tokens!(),
      notex: false,
      noerror: false,
      noltxml: false,
      withoptions: Vec::new(),
      handleoptions: false,
      as_class: false,
      with_stomach: None,
    }
  }
}

// Selects the RelaxNG schema defining the XML output language
pub fn select_relaxng_schema(schema: String, namespaces: Option<HashMap<String, String>>, state: &mut State) {
  // What verb here? Set, Choose,...
  let model = &mut state.model;
  model.set_relaxng_schema(schema);
  if let Some(namespaces) = namespaces {
    for (prefix, value) in namespaces {
      model.register_document_namespace(&prefix, Some(value));
    }
  }
  return;
}

pub fn def_macro<T: Into<Option<ExpansionBody>>>(
  cs: Token,
  paramlist: Option<Parameters>,
  expansion: T,
  options_opt: Option<ExpandableOptions>,
  state: &mut State,
)
{
  let expansion = expansion.into();
  let options = options_opt.unwrap_or_default();
  let options_locked = options.locked;
  let locked_key = if options_locked { s!("{}:locked", cs) } else { String::new() };
  state.install_definition(
    Expandable {
      cs,
      paramlist,
      expansion,
      ..Expandable::default()
    },
    options.scope,
  );
  if options_locked {
    state.assign_value(&locked_key, true, Some(Scope::Global));
  }
}

pub struct RegisterOptions {
  pub getter: Option<RegisterGetterClosure>,
  pub setter: Option<RegisterSetterClosure>,
  pub readonly: bool,
}
impl Default for RegisterOptions {
  fn default() -> Self {
    RegisterOptions {
      getter: None,
      setter: None,
      readonly: false,
    }
  }
}

//======================================================================
// Defining Conditional Control Sequences.
//======================================================================
// Define a conditional control sequence. Its processing takes place in
// the Gullet.  The test is applied to the arguments (if any),
// which determines which branch is executed.
// If the test is undefined, the conditional is a "user defined" one;
// Two additional primitives are defined \footrue and \foofalse;
// the test is then determined by the most recently called of those.
//
// If you supply a skipper instead of a test, it is also applied to the arguments
// and should skip to the right place in the following \or, \else, \fi.

pub fn def_conditional(cs: Token, paramlist: Option<Parameters>, test: Option<ConditionalClosure>, options: ConditionalOptions, state: &mut State) {
  let cs_name = cs.get_cs_name();
  let locked_key = if let Some(true) = options.locked {
    s!("{}:locked", cs_name)
  } else {
    String::new()
  };
  match cs_name {
    "\\fi" | "\\else" | "\\or" => state.install_definition(
      Conditional {
        cs: cs.clone(),
        paramlist: None,
        test: None,
        conditional_type: ConditionalType::from(&cs_name),
        locked: options.locked,
        skipper: options.skipper,
      },
      options.scope,
    ),
    custom => {
      if CONDITIONAL_RE.is_match(custom) {
        let captures = CONDITIONAL_RE.captures(custom).unwrap();
        let name = captures.get(1).map_or("", |m| m.as_str()).to_string();
        if !name.is_empty() && name != "case" && test.is_none() {
          // user-defined conditional, like with \newif
          // Note: setting up these macros is compile-time expensive, maybe there is some way to
          // avoid...
          BindState!(state);
          // Note: the double clones are technically correct Rust if annoying to write and read.
          //       first, we want to capture a cloned value of cs, to be able to keep using cs here.
          // second, each invocation of the conditional macro needs to create new tokens to
          // return,       hence a clone is required on each call.
          DefMacroI!(T_CS!(s!("\\{}true", name)), None, Tokens!(T_CS!("\\let"), cs.clone(), T_CS!("\\iftrue")));
          DefMacroI!(
            T_CS!(s!("\\{}false", name)),
            None,
            Tokens!(T_CS!("\\let"), cs.clone(), T_CS!("\\iffalse"))
          );
          state.let_i(&cs, T_CS!("\\iffalse"), None);
        } else {
          //  For \ifcase, the parameter list better be a single Number !!
          state.install_definition(
            Conditional {
              cs,
              paramlist,
              test,
              conditional_type: ConditionalType::If,
              locked: options.locked,
              skipper: options.skipper,
            },
            options.scope,
          );
        }
      } else {
        error!(
          target: &s!("misdefined:{}", cs),
          "The conditional {} is being defined but doesn't start with \\if", cs
        );
      }
    },
  }

  if let Some(true) = options.locked {
    state.assign_value(&locked_key, true, None);
  }
  return;
}

pub fn def_register<T: Into<RegisterValue>>(cs: Token, parameters: Option<Parameters>, value: T, options: Option<RegisterOptions>, state: &mut State) {
  let options: RegisterOptions = options.unwrap_or_else(RegisterOptions::default);
  let value: RegisterValue = value.into();
  let name = cs.to_string();
  let register_type: RegisterType = (&value).into();
  // Prepare clones to move into closures
  let getter_value = value.clone();
  let setter_name = name.clone();

  let getter: RegisterGetterClosure = match options.getter {
    Some(getter) => getter.clone(),
    None => Rc::new(move |args: Vec<Token>, state: &State| -> Option<RegisterValue> {
      let args_string: String = args.iter().map(|arg: &Token| arg.to_string()).collect::<Vec<String>>().join("");
      match state.lookup_value(&(name.clone() + &args_string)) {
        None => Some(getter_value.clone()),
        Some(v) => v.into(),
      }
    }),
  };
  let readonly = options.readonly;

  let setter: RegisterSetterClosure = match options.setter {
    Some(setter) => setter.clone(),
    None => {
      if readonly {
        Rc::new(move |value, args, state| {
          warn!(target: &s!("unexpected:{}", setter_name), "Can't assign to register {}", setter_name);
        })
      } else {
        Rc::new(move |value, args, state| {
          let args_string: String = args.iter().map(|arg: &Tokens| arg.to_string()).collect::<Vec<String>>().join("");

          state.assign_value(&(setter_name.clone() + &args_string), value, None);
        })
      }
    },
  };

  // Not really right to set the value!
  state.assign_value(&cs.to_string(), value, None);
  state.install_definition(
    Register {
      cs,
      parameters,
      register_type,
      readonly,
      getter,
      setter,
      internalcs: None,
      internalvalue: None,
    },
    Some(Scope::Global),
  );
  return;
}

pub fn def_primitive(cs: Token, paramlist: Option<Parameters>, compiled_replacement: PrimitiveClosure, options: PrimitiveOptions, state: &mut State) {
  let options_locked = options.locked;
  let scope = options.scope;
  let mut before_digest_env: Vec<BeforeDigestClosure> = Vec::new();
  let cs_name = cs.get_cs_name().to_owned();

  if options.require_math {
    let cs_name_cloned = cs_name.clone();
    let require_math_closure = before_digest_single!(stomach, state, { requireMath!(cs_name_cloned, state) });
    before_digest_env.push(require_math_closure);
  }

  if options.forbid_math {
    let cs_name_cloned = cs_name.clone();
    let forbid_math_closure = before_digest_single!(stomach, state, { forbidMath!(cs_name_cloned, state) });
    before_digest_env.push(forbid_math_closure);
  }
  if let Some(ref mode) = options.mode {
    let mode_clone = mode.clone();
    let begin_mode_closure = before_digest_single!(stomach, state, {
      stomach.begin_mode(&mode_clone, state)?;
    });
    before_digest_env.push(begin_mode_closure);
  } else if options.bounded {
    let bgroup_closure = before_digest_single!(stomach, state, {
      stomach.bgroup(state);
    });
    before_digest_env.push(bgroup_closure);
  }
  if let Some(chosen_font) = options.font {
    let merge_font_closure = before_digest_single!(stomach, state, {
      MergeFont!(chosen_font.clone(), state);
    });
    before_digest_env.push(merge_font_closure);
  }
  before_digest_env.extend(options.before_digest);

  let mut after_digest_env: Vec<DigestionClosure> = Vec::new();
  after_digest_env.extend(options.after_digest);
  if let Some(ref mode) = options.mode {
    let mode_clone = mode.clone();
    let end_mode_closure: Vec<DigestionClosure> = after_digest!(stomach, whatsit, state, {
      stomach.end_mode(&mode_clone, state)?;
    });
    after_digest_env.extend(end_mode_closure);
  } else if options.bounded {
    let egroup_closure: Vec<DigestionClosure> = after_digest!(stomach, whatsit, state, {
      stomach.egroup(state)?;
    });
    after_digest_env.extend(egroup_closure);
  }

  state.install_definition(
    Primitive {
      cs: cs.clone(),
      paramlist,
      replacement: Some(compiled_replacement),
      before_digest: before_digest_env,
      after_digest: after_digest_env,
      alias: options.alias,
      nargs: options.nargs,
    },
    scope,
  );
  if options_locked {
    state.assign_value(&s!("{}:locked", cs_name), true, None);
  }
}

pub fn def_math_primitive(cs: Token, paramlist: Option<Parameters>, presentation: String, mut options: MathPrimitiveOptions, state: &mut State) {
  options.locked = false;
  options.font = None;
  let scope = options.scope;
  let reqfont = match options.font {
    Some(ref fnt) => fnt.clone(),
    None => Font::default()
  };
  state.install_definition(
    MathPrimitive {
      cs: cs.clone(),
      paramlist: None, // never any parameters, this is intentional
      replacement: Some(Rc::new(move |stomach, args, state| {
        // let locator    = $stomach->getGullet->getLocator;
        let mut properties = HashMap::new(); // TODO: sync with perl master here
        properties.insert(s!("mode"), Stored::String(String::from("math")));
        // TODO: Improve font precision here, the defaults may not belong in this lookup
        let font = state
          .lookup_font()
          .unwrap_or_else(|| Rc::new(Font::default()))
          .merge(reqfont.clone())
          .specialize(&presentation);
        let font = Rc::new(font);
        // foreach my $key (keys %properties) {
        //   my $value = $properties{$key};
        //   if (ref $value eq 'CODE') {
        //     $properties{$key} = &$value(); } }
        // info!("defmath_prim: {}, tokens: {:?}", &$presentation, $cs);
        Ok(vec![Digested::TBox(Rc::new(
          // TODO: Can we reduce boilerplate?
          Tbox {
            text: presentation.clone(),
            tokens: Tokens!(cs.clone()),
            font,
            properties,
            ..Tbox::default()
          },
        ))])
      })),
      options,
      ..MathPrimitive::default()
    },
    scope,
  );
}

pub fn def_constructor(
  cs: Token,
  paramlist: Option<Parameters>,
  compiled_replacement: Option<ReplacementClosure>,
  options: ConstructorOptions,
  state: &mut State,
)
{
  // TODO: This won't work, as we can only invoke method calls on paramlist in runtime
  //*rtx_codegen::constructable::NARGS = $paramlist.get_num_args();
  let scope = options.scope;
  let is_locked = options.locked;
  let cs_name = cs.get_cs_name().to_owned();
  let locked_key = if is_locked { s!("{}:locked", cs_name) } else { String::new() };

  let mut before_digest_closures: Vec<BeforeDigestClosure> = Vec::new();

  if options.require_math {
    let cs_name_cloned = cs_name.clone();
    let require_math_closure = before_digest_single!(stomach, state, { requireMath!(cs_name_cloned, state) });
    before_digest_closures.push(require_math_closure);
  }
  if options.forbid_math {
    let cs_name_cloned = cs_name.clone();
    let forbid_math_closure = before_digest_single!(stomach, state, { forbidMath!(cs_name_cloned, state) });
    before_digest_closures.push(forbid_math_closure);
  }
  if let Some(ref mode) = options.mode {
    let mode_clone = mode.clone();
    let begin_mode_closure = before_digest_single!(stomach, state, {
      stomach.begin_mode(&mode_clone, state)?;
    });
    before_digest_closures.push(begin_mode_closure);
  } else if options.bounded {
    let bgroup_closure = before_digest_single!(stomach, state, {
      stomach.bgroup(state);
    });
    before_digest_closures.push(bgroup_closure);
  }
  if let Some(chosen_font) = options.font {
    let merge_font_closure = before_digest_single!(stomach, state, {
      MergeFont!(chosen_font.clone(), state);
    });
    before_digest_closures.push(merge_font_closure);
  }
  before_digest_closures.extend(options.before_digest);

  let mut after_digest_closures: Vec<DigestionClosure> = Vec::new();
  after_digest_closures.extend(options.after_digest);
  if let Some(ref mode) = options.mode {
    let mode_clone = mode.clone();
    let end_mode_closure: Vec<DigestionClosure> = after_digest!(stomach, whatsit, state, {
      stomach.end_mode(&mode_clone, state)?;
    });
    after_digest_closures.extend(end_mode_closure);
  } else if options.bounded {
    let egroup_closure: Vec<DigestionClosure> = after_digest!(stomach, whatsit, state, {
      stomach.egroup(state)?;
    });
    after_digest_closures.extend(egroup_closure);
  }

  let constructor = Constructor {
    cs,
    paramlist,
    replacement: compiled_replacement,
    before_digest: before_digest_closures,
    after_digest: after_digest_closures,
    before_construct: options.before_construct,
    after_construct: options.after_construct,
    nargs: options.nargs,
    alias: options.alias,
    reversion: options.reversion,
    // sizer
    capture_body: options.capture_body,
    properties: options.properties,
    // outer
    // long
    ..Constructor::default()
  };
  state.install_definition(constructor, scope);

  if is_locked {
    state.assign_value(&locked_key, true, None);
  }
}

pub fn def_environment(
  name: String,
  paramlist: Option<Parameters>,
  compiled_replacement: Option<ReplacementClosure>,
  options: ConstructorOptions,
  state: &mut State,
)
{
  let begin_name = s!("\\begin{{{}}}", &name);
  let end_name = s!("\\end{{{}}}", &name);
  // This is for the common case where the environment is opened by \begin{env}
  // let sizer = inferSizer($options.sizer, $options.reversion);
  let mut before_digest_env: Vec<BeforeDigestClosure> = Vec::new();
  match &options.mode {
    Some(ref mode) => {
      let bmode = mode.clone();
      let mode_closure = Rc::new(move |stomach: &mut Stomach, state: &mut State| {
        stomach.begin_mode(&bmode, state)?;
        Ok(Vec::new())
      });
      before_digest_env.push(mode_closure);
    },
    None => {
      let bgroup_closure = before_digest_single!(stomach, state, {
        stomach.bgroup(state);
      });
      before_digest_env.push(bgroup_closure);
    },
  };
  if options.require_math {
    let require_name = begin_name.clone();
    let require_math_closure = before_digest_single!(stomach, state, { requireMath!(require_name, state) });
    before_digest_env.push(require_math_closure);
  }
  if options.forbid_math {
    let forbid_name = begin_name.clone();
    let forbid_math_closure = before_digest_single!(stomach, state, { forbidMath!(forbid_name, state) });
    before_digest_env.push(forbid_math_closure);
  }

  let env_name = name.clone();
  let current_environment_closure = before_digest_single!(stomach, state, {
    AssignValue!("current_environment", env_name.clone(), None, state);
    let body = T_LETTER!(env_name.clone());
    DefMacroI!(T_CS!("\\@currenvir"), None, body.clone(), state);
  });
  before_digest_env.push(current_environment_closure);

  if let Some(chosen_font) = options.font {
    let merge_font_closure = before_digest_single!(stomach, state, {
      MergeFont!(chosen_font.clone(), state);
    });
    before_digest_env.push(merge_font_closure);
  }
  before_digest_env.extend(options.before_digest);

  let push_frame_closure = Rc::new(|_document: &mut Document, _whatsit: &Whatsit, state: &mut State| {
    state.push_frame();
    Ok(())
  });
  let mut before_construct_with_frame: Vec<ConstructionClosure> = vec![push_frame_closure];
  before_construct_with_frame.extend(options.before_construct);

  let mut after_construct_with_frame: Vec<ConstructionClosure> = options.after_construct;

  let pop_frame_closure = Rc::new(|_document: &mut Document, _whatsit: &Whatsit, state: &mut State| {
    state.pop_frame()?;
    Ok(())
  });
  after_construct_with_frame.push(pop_frame_closure);

  let begin_name_constructor = Rc::new(Constructor {
    cs: T_CS!(begin_name),
    paramlist: paramlist.clone(),
    replacement: compiled_replacement.clone(),
    nargs: options.nargs,
    before_digest: before_digest_env,
    after_digest: options.after_digest_begin,
    after_digest_body: options.after_digest_body,
    before_construct: before_construct_with_frame,
    // Curiously, it's the \begin whose afterConstruct gets called.
    after_construct: after_construct_with_frame,
    capture_body: true,
    properties: options.properties.clone(),
    // (defined $options{reversion} ? (reversion => $options{reversion}) : ()),
    // (defined $sizer ? (sizer => $sizer) : ()),
    // ), $options{scope});
    reversion: options.reversion,
    alias: options.alias,
  });
  state.install_definition(begin_name_constructor, options.scope);

  let mut after_digest_env = options.after_digest;
  let unexpected_end_closure = Rc::new(|_stomach: &mut Stomach, _whatsit: &mut Whatsit, state: &mut State| {
    // let env = LookupValue!("current_environment", $state_arg);
    //     Error('unexpected', "\\end{$name}", $_[0],
    //       "Can't close environment $name",
    //       "Current are "
    //         . join(', ', state->lookupStackedValues('current_environment')))
    //       unless $env && $name eq $env;
    //     return; },
    Ok(Vec::new())
  });
  after_digest_env.push(unexpected_end_closure);

  match options.mode {
    Some(mode) => {
      let emode = mode.clone();
      let emode_closure = Rc::new(move |stomach: &mut Stomach, _whatsit: &mut Whatsit, state: &mut State| {
        stomach.end_mode(&emode, state)?;
        Ok(Vec::new())
      });
      after_digest_env.push(emode_closure);
    },
    None => {
      let egroup_closure = Rc::new(|stomach: &mut Stomach, _whatsit: &mut Whatsit, state: &mut State| {
        stomach.egroup(state)?;
        Ok(Vec::new())
      });
      after_digest_env.push(egroup_closure);
    },
  };

  let end_envname_constructor = Rc::new(Constructor {
    cs: T_CS!(end_name),
    replacement: None,
    paramlist: None,
    before_digest: options.before_digest_end,
    after_digest: after_digest_env,
    ..Constructor::default() // TODO ? fill in missing ones
  });
  state.install_definition(end_envname_constructor, options.scope);

  // For the uncommon case opened by \csname env\endcsname
  let name_constructor = Rc::new(Constructor {
    cs: T_CS!(s!("\\{}", &name)),
    paramlist,
    replacement: compiled_replacement,
    // beforeDigest => flatten(($options{requireMath} ? (sub { requireMath($name); }) : ()),
    //   ($options{forbidMath} ? (sub { forbidMath($name); })              : ()),
    //   ($mode                ? (sub { $_[0]->beginMode($mode); })        : ()),
    //   ($options{font}       ? (sub { MergeFont(%{ $options{font} }); }) : ()),
    //   $options{beforeDigest}),
    // afterDigest     => flatten($options{afterDigestBegin}),
    // afterDigestBody => flatten($options{afterDigestBody}),
    // beforeConstruct => flatten(sub { state->pushFrame; }, $options{beforeConstruct}),
    // Curiously, it's the \begin whose afterConstruct gets called.
    // afterConstruct => flatten($options{afterConstruct}, sub { state->popFrame; }),
    nargs: options.nargs,
    capture_body: true,
    properties: options.properties.clone(),
    // (defined $options{reversion} ? (reversion => $options{reversion}) : ()),
    // (defined $sizer ? (sizer => $sizer) : ()),
    // ), $options{scope});
    ..Constructor::default()
  });
  state.install_definition(name_constructor, options.scope);

  let end_name_constructor = Rc::new(Constructor {
    cs: T_CS!(s!("\\end{}", &name)),
    paramlist: None,
    replacement: Some(Rc::new(|document, whatsit, properties, state| {
      let env = state.lookup_value("current_environment");
      // Error('unexpected', "\\end{$name}", $_[0],
      //   "Can't close environment $name",
      //   "Current are "
      //     . join(', ', state->lookupStackedValues('current_environment')))
      //   unless $env && $name eq $env;
      Ok(())
    })),
    // beforeDigest => flatten($options{beforeDigestEnd}),
    // afterDigest  => flatten($options{afterDigest},
    //   ($mode ? (sub { $_[0]->endMode($mode); }) : ())),
    // ), $options{scope});
    ..Constructor::default()
  });
  state.install_definition(end_name_constructor, options.scope);

  if options.locked {
    state.assign_value(&s!("\\begin{{{}}}:locked", &name), true, None);
    state.assign_value(&s!("\\end{{{}}}:locked", &name), true, None);
    state.assign_value(&s!("\\{}:locked", &name), true, None);
    state.assign_value(&s!("\\end{}:locked", &name), true, None);
  }
}

//**********************************************************************
/// This function computes an xml:id for a node, if it hasn't already got one.
/// It is suitable for use in Tag afterOpen as
///  `Tag('ltx:para',afterOpen=>sub { GenerateID(@_,'p'); });`
/// It generates an id of the form <parentid>.<prefix><number>
/// The parent node (the one with ID=<parentid>) also maintains a counter
/// stored in an attribute `_ID_counter_<prefix>` recording the last used
/// <number> for <prefix> amongst its descendents.
pub fn generate_id(document: &mut Document, mut node: &mut Node, mut prefix: &str, state: &mut State) -> Result<()> {
  // If node doesn't already have an id, and can
  let node_qname = document.get_node_qname(node, state);
  // but isn't a _Capture_ node (which ultimately should disappear)
  if node.get_attribute("xml:id").is_none() && document.can_have_attribute(&node_qname, "xml:id", state) && (node_qname != "ltx:_Capture_") {
    let mut ancestor = document
      .findnode("ancestor::*[@xml:id][1]", Some(node), state)
      .unwrap_or_else(|| document.get_document().get_root_element().unwrap());
    //// Old versions don't like ancestor.getAttribute('xml:id');
    let ancestor_id = ancestor.get_attribute_ns("id", XML_NS);
    // If we've got no ancestor_id, then we've got no ancestor (no document yet!),
    // or ancestor IS the root element (but without an id);
    // If we also have no prefix, we'll end up with an illegal id (just digits)!!!
    // We'll use "id" for an id prefix; this will work whether or not we have an ancestor.
    if prefix.is_empty() && ancestor_id.is_none() {
      prefix = "id";
    }

    let ctrkey = s!("_ID_counter_") + prefix + "_";
    let a_ctr = ancestor.get_attribute(&ctrkey).unwrap_or_else(|| s!("0"));

    let ctr_int = 1 + a_ctr.parse::<u32>().unwrap_or(0);
    let ctr = ctr_int.to_string();

    let id = match ancestor_id {
      Some(aid) => aid + ".",
      None => String::new(),
    } + prefix
      + &ctr;

    ancestor.set_attribute(&ctrkey, &ctr)?;
    node.set_attribute("xml:id", &id)?;
  }
  Ok(())
}

pub fn merge_font(font: Font, state: &mut State) {
  let new_font = match state.lookup_font() {
    Some(ref f) => f.merge(font),
    _ => Font::text_default().merge(font),
  };
  state.assign_value("font", new_font, Some(Scope::Local));
}

pub fn digest_text(stuff: Tokens, stomach: &mut Stomach, state: &mut State) -> Result<Digested> {
  stomach.begin_mode("text", state)?;
  let value = stomach.digest(stuff, state);
  stomach.end_mode("text", state)?;
  value
}

pub fn digest_literal<T: Into<Tokens>>(stuff: T, stomach: &mut Stomach, state: &mut State) -> Result<Digested> {
  let stuff: Tokens = stuff.into();
  // Perhaps should do StartSemiverbatim, but is it safe to push a frame? (we might cover over
  // valid changes of state!)
  stomach.begin_mode("text", state)?;

  let font = state.lookup_font().unwrap(); // TODO: raise error if font missing
  state.assign_value("font", font.merge(fontmap!(encoding => "ASCII")), Some(Scope::Local)); // try to stay as ASCII as possible

  let value = stomach.digest(stuff, state);
  state.assign_value("font", font, None); // TODO: maybe we need .assign_font ?
  stomach.end_mode("text", state)?;
  value
}

pub fn digest_if(token: Token, stomach: &mut Stomach, state: &mut State) -> Result<Option<Digested>> {
  if let Some(defn) = state.lookup_definition(&token) {
    match stomach.digest(Tokens!(token), state) {
      Ok(t) => Ok(Some(t)),
      Err(e) => Err(e),
    }
  } else {
    Ok(None)
  }
}

pub struct NewCounterOptions<'ct> {
  pub idprefix: &'ct str,
  pub idwithin: &'ct str,
  pub nested: Vec<&'ct str>,
}
impl<'ct> Default for NewCounterOptions<'ct> {
  fn default() -> Self {
    NewCounterOptions {
      idprefix: "",
      idwithin: "",
      nested: Vec::new(),
    }
  }
}

pub fn new_counter(ctr: &str, within: &str, options_opt: Option<NewCounterOptions>, state: &mut State) -> Result<()> {
  BindState!(state);
  let unctr = s!("UN{}", ctr); // UNctr is counter for generating ID's for UN-numbered items.
  let cctr = s!("\\c@{}", ctr);
  let clctr = s!("\\cl@{}", ctr);
  let cunctr = s!("\\c@{}", unctr);
  let clunctr = s!("\\cl@{}", unctr);

  DefRegisterI!(T_CS!(cctr), None, Number::new(0.0), None);
  state.assign_value(&cctr, Number!(0), Some(Scope::Global));
  AfterAssignment!();
  if !state.lookup_bool(&clctr) {
    state.assign_value(&clctr, Tokens!(), Some(Scope::Global));
  }

  DefRegisterI!(T_CS!(cunctr), None, Number::new(0.0), None);
  state.assign_value(&cunctr, Number!(0), Some(Scope::Global));
  if !state.lookup_bool(&clunctr) {
    state.assign_value(&clunctr, Tokens!(), Some(Scope::Global));
  }

  if !within.is_empty() {
    let clwithin = s!("\\cl@{}", within);
    let clunwithin = s!("\\cl@UN{}", within);
    let mut x = if let Some(Stored::Tokens(cl)) = state.lookup_value(&clwithin) {
      cl.unlist()
    } else {
      Vec::new()
    };
    let mut clwithin_tokens = vec![T_CS!(ctr), T_CS!(unctr)];
    clwithin_tokens.append(&mut x);
    state.assign_value(&clwithin, Stored::Tokens(Tokens::new(clwithin_tokens)), Some(Scope::Global));

    let mut unx = if let Some(Stored::Tokens(clun)) = state.lookup_value(&clunwithin) {
      clun.unlist()
    } else {
      Vec::new()
    };
    let mut clunwithin_tokens = vec![T_CS!(unctr)];
    clunwithin_tokens.append(&mut unx);

    state.assign_value(&clunwithin, Stored::Tokens(Tokens::new(clunwithin_tokens)), Some(Scope::Global))
  }

  if let Some(ref options) = options_opt {
    if !options.nested.is_empty() {
      state.assign_value(&s!("nested_counters_{}", ctr), options.nested.clone(), Some(Scope::Global))
    }
  }

  // default is equivalent to \arabic{ctr}, but w/o using the LaTeX macro!
  let ctr_string = ctr.to_string();
  DefMacro!(&s!("\\the{}",ctr), sub[gullet, args, inner_state] {
    let counter_value = CounterValue!(&ctr_string, inner_state).value_of();
    Ok(Tokens::new(ExplodeText!(counter_value)))
  }, scope => Some(Scope::Global));

  if let Some(options) = options_opt {
    let mut prefix = options.idprefix.to_string();
    if !prefix.is_empty() {
      state.assign_value(&s!("@ID@prefix@{}", ctr), prefix.clone(), Some(Scope::Global));
    } else {
      prefix = state.lookup_string(&s!("@ID@prefix@{}", ctr));
      if prefix.is_empty() {
        prefix = clean_id(ctr);
      }
    }
    if !prefix.is_empty() {
      let mut idwithin = if !options.idwithin.is_empty() {
        options.idwithin.to_string()
      } else {
        within.to_string()
      };
      if !idwithin.is_empty() {
        let ctr_string = ctr.to_string();
        DefMacro!(&s!("\\the{}@ID",ctr), sub[gullet, args, inner_state] {
          Ok(TokenizeInternal!(
            &s!("\\expandafter\\ifx\\csname the{}@ID\\endcsname\\@empty\\else\\csname the{}@ID\\endcsname.\\fi {}\\csname @{}@ID\\endcsname",
          idwithin,idwithin,prefix,ctr_string), None
          ))
        },
        scope => Some(Scope::Global));
      } else {
        let ctr_string = ctr.to_string();
        DefMacro!(&s!("\\the{}@ID",ctr), sub[gullet,args, inner_state] {
          Ok(TokenizeInternal!(
              &s!("{}\\csname @{}@ID\\endcsname",prefix,ctr_string), None
          ))},
          scope => Some(Scope::Global));
      }
      DefMacro!(&s!("\\@{}@ID",ctr), "0", scope => Some(Scope::Global));
    }
  }

  Ok(())
}

pub fn counter_value(ctr: &str, state: &mut State) -> Number {
  match state.lookup_number(&s!("\\c@{}", ctr)) {
    None => {
      warn!(target: &s!("undefined:{:?}", ctr), "Counter {} was not defined; assuming 0", ctr);
      Number!(0)
    },
    Some(value) => value,
  }
}

pub fn add_to_counter(ctr: &str, value: Number, gullet: &mut Gullet, state: &mut State) {
  let v = counter_value(ctr, state).add(value);
  state.assign_value(&s!("\\c@{}", ctr), v, Some(Scope::Global));
  state.after_assignment();
  BindState!(state);
  let id_cs = T_CS!(s!("\\@{}@ID", ctr));
  DefMacroI!(id_cs.clone(), None, Tokens::new(Explode!(v.value_of())),
    scope => Some(Scope::Global));
}

pub fn step_counter(ctr: &str, noreset: bool, stomach: &mut Stomach, state: &mut State) -> Result<()> {
  BindState!(state);
  let value = counter_value(ctr, state);
  state.assign_value(&s!("\\c@{}", ctr), value.add(Number!(1)), Some(Scope::Global));
  state.after_assignment();
  let token_value = Tokens::new(Explode!(counter_value(ctr, state).value_of()));
  DefMacroI!(T_CS!(s!("\\@{}@ID",ctr)), None,
              token_value.clone(), scope => Some(Scope::Global));

  // and reset any within counters!
  if !noreset {
    if let Some(nested) = state.lookup_tokens(&s!("\\cl@{}", ctr)) {
      for c in nested.unlist() {
        reset_counter(&c.to_string(), state);
      }
    }
  }
  digest_if(T_CS!(s!("\\the{}", ctr)), stomach, state)?;
  Ok(())
}

pub struct RefStepValue {
  pub id: Option<String>,
  pub tags: Option<Tokens>,
}

pub fn ref_step_counter(ctype: &str, noreset: bool, stomach: &mut Stomach, state: &mut State) -> Result<HashMap<String, Stored>> {
  let ctr = match state.lookup_mapping("counter_for_type", ctype) {
    Some(Stored::String(ctr)) => ctr.to_string(),
    _ => ctype.to_string(),
  };
  step_counter(&ctr, noreset, stomach, state)?;

  let has_id: bool = if let Some(iddef) = state.lookup_definition(&T_CS!(s!("\\the{}@ID", ctr))) {
    if let Some(params) = iddef.get_parameters() {
      params.get_num_args() == 0
    } else {
      true
    }
  } else {
    false
  };

  BindState!(state);
  let the_cs = T_CS!(s!("\\the{}", ctr));
  let the_id_cs = T_CS!(s!("\\the{}@ID", ctr));
  DefMacroI!(T_CS!("\\@currentlabel"), None, the_cs.clone(), scope => Some(Scope::Global));
  if has_id {
    DefMacroI!(T_CS!("\\@currentID"), None, the_id_cs.clone(), scope => Some(Scope::Global))
  }

  let id = if has_id {
    digest_literal(Tokens!(T_CS!(s!("\\the{}@ID", ctr))), stomach, state)?.to_string()
  } else {
    String::new()
  };

  let refnum = digest_text(Tokens!(T_CS!(s!("\\the{}", ctr))), stomach, state)?;
  let invocation;
  {
    let gullet = stomach.get_gullet_mut();
    invocation = Invocation!(T_CS!("\\lx@make@tags"), vec![Tokens!(T_OTHER!(ctype))], gullet, state)?;
  }

  let tags = stomach.digest(invocation, state)?;

  // Any scopes activated for previous value of this counter (& any nested counters) must be
  // removed. This may also include scopes activated for \label
  deactivate_counter_scope(&ctr, state);

  // And install the scope (if any) for this reference number.
  state.assign_value("current_counter", ctr.to_string(), Some(Scope::Local));

  let scope = s!("{}:{}", ctr, refnum.to_string());
  state.assign_value(&s!("scopes_for_counter:{}", ctr), vec![scope.clone()], Some(Scope::Local));
  state.activate_scope(&scope);

  Ok(map!(
    "tags" => Stored::Digested(Box::new(tags)),
    "id" => Stored::String(id)
  ))
}

fn deactivate_counter_scope(ctr: &str, state: &mut State) {
  //  print STDERR "Unusing scopes for $ctr\n";
  if let Some(Stored::VecString(stored_scopes)) = state.lookup_value(&s!("scopes_for_counter:{}", ctr)) {
    for scope in stored_scopes.clone() {
      state.deactivate_scope(&scope);
    }
  }

  if let Some(Stored::VecString(stored_counters)) = state.lookup_value(&s!("nested_counters_{}", ctr)) {
    for inner_ctr in stored_counters.clone() {
      deactivate_counter_scope(&inner_ctr, state);
    }
  }
}

// For UN-numbered units
pub fn ref_step_id(ctype: &str, stomach: &mut Stomach, state: &mut State) -> Result<HashMap<String, Stored>> {
  BindState!(state, stomach);
  let ctr = match state.lookup_mapping("counter_for_type", ctype) {
    Some(map) => map.to_string(),
    None => ctype.to_string(),
  };
  let unctr = s!("UN{}", ctr);
  step_counter(&unctr, false, stomach, state)?;

  let cunctr_val = state.lookup_number(&s!("\\c@{}", unctr)).unwrap().value_of();
  DefMacroI!(T_CS!(&s!("\\@{}@ID",ctr)), None, Tokens!(T_OTHER!("x"), Explode!(cunctr_val)), scope => Some(Scope::Global));

  DefMacroI!(T_CS!("\\@currentID"), None, T_CS!(&s!("\\the{}@ID", ctr)));
  Ok(map!("id".to_string() => digest_literal(T_CS!(&s!("\\the{}@ID", ctr)), stomach, state)?.to_string().into()))
}

pub fn reset_counter(ctr: &str, state: &mut State) {
  state.assign_value(&s!("\\c@{}", ctr), Number!(0), Some(Scope::Global));
  // and reset any within counters!
  let nested = state.lookup_tokens(&s!("\\cl@{}", ctr)).unwrap_or_else(|| Tokens!());

  for c in &(nested.unlist()) {
    reset_counter(&c.to_string(), state);
  }

  return;
}

pub fn build_invocation<T: Into<Token>>(token: T, args: Vec<Tokens>, gullet: &mut Gullet, state: &mut State) -> Result<Tokens> {
  let token: Token = token.into();
  // Note: token may have been \let to another defn!
  if let Some(defn) = state.lookup_definition(&token) {
    let mut invoked_tokens = vec![token];
    let mut reverted_args = if let Some(params) = defn.get_parameters() {
      params.revert_arguments(args, gullet, state)?
    } else {
      Vec::new()
    };
    invoked_tokens.append(&mut reverted_args);
    Ok(Tokens::new(invoked_tokens))
  } else {
    error!(
      target: &s!("undefined:{}", token.get_cs_name()),
      "Can't invoke {:?}; it is undefined", token
    );
    let mut invoked_tokens = vec![token];
    // DefConstructorI!(token, convert_latex_args(args.len(), 0),
    // sub { LaTeXML::Core::Stomach::makeError($_[0], 'undefined', token); });
    let mut wrapped_args: Vec<Token> = args
      .into_iter()
      .flat_map(|arg| {
        let mut wrapped = vec![T_BEGIN!()];
        wrapped.append(&mut arg.unlist());
        wrapped.push(T_END!());
        wrapped
      })
      .collect();
    invoked_tokens.append(&mut wrapped_args);
    Ok(Tokens::new(invoked_tokens))
  }
}

pub fn do_expand<T: Into<Tokens>>(mut tokens: T, outer_gullet: &mut Gullet, outer_state: &mut State) -> Result<Tokens> {
  let mut tokens: Tokens = tokens.into();
  outer_gullet.reading_from_mouth(
    Mouth::default(),
    outer_state,
    Box::new(move |expand_gullet: &mut Gullet, expand_state: &mut State| -> Result<Tokens> {
      expand_gullet.unread(&tokens);
      let mut expanded = Vec::new();
      while let Some(t) = expand_gullet.read_x_token(false, false, expand_state)? {
        expanded.push(t);
      }
      Ok(Tokens::new(expanded))
    }),
  )
}

/// Convert a LaTeX-style argument spec to our Package form.
/// Ie. given $nargs and $optional, being the two optional arguments to
/// something like \newcommand, convert it to the form we use
pub fn convert_latex_args(mut nargs: usize, optional: Option<Tokens>, state: &mut State) -> Result<Option<Parameters>> {
  let mut params = Vec::new();
  if let Some(tks) = optional {
    params.push(
      Parameter {
        name: s!("Optional"),
        spec: s!("[Default:{}]", tks.untex(state)),
        extra: vec![ParameterExtra::Token(tks.into()), ParameterExtra::ParametersOption(None)],
        ..Parameter::default()
      }
      .init(state)?,
    );
    nargs -= 1;
  }

  for _ in 1..=nargs {
    params.push(
      Parameter {
        name: s!("Plain"),
        spec: "{}".to_string(),
        ..Parameter::default()
      }
      .init(state)?,
    );
  }
  if params.is_empty() {
    Ok(None)
  } else {
    Ok(Some(Parameters { params }))
  }
}

static RMLETTERS : [char; 7]= ['i', 'v', 'x', 'l', 'c', 'd', 'm'];
pub fn roman_aux<T: Into<i32>>(stuff: T) -> String {
  // let mut n = stuff.into();
  // let mut div = 1000;
  // let mut s : String = if n > div {  String::from_utf8(vec![b'm'; n/div]) } else { String::new() };
  // let mut p = 4;
  // while n %= div {
  //   div /= 10;
  //   let d = n / div;
  //   if d % 5 == 4 {
  //     s += RMLETTERS[p];
  //     d+=1;
  //   }
  //   if d > 4 {
  //     s += RMLETTERS[p + (d / 5)];
  //     d %= 5; 
  //   }
  //   if d!=0 {
  //     s += String::from_utf8(vec![RMLETTERS[p], d]);
  //   }
  //   p -= 2; 
  // }
  // s
  // TODO!
  unimplemented!()
}

//======================================================================
// Cleaners
//======================================================================

// Small rust experiment -- type casting into Cow<String> in intermediate steps can become a
// prolonged compiler negotiation, and can tire one down.
// Instead, use untyped intermediate variables _1 .. _n , and let the compiler fill in the gaps
fn clean_id(key: &str) -> String {
  let mut cleaned = Cow::Borrowed(key.trim_start().trim_end()); // Trim leading/trailing, in any case
  let cleaned_1 = SPACES_RE.replace_all(&cleaned, ""); // remove all spaces
                                                       // Remove common idiom:
  let cleaned_2 = DIRTY_ID_IDIOM_RE.replace_all(&cleaned_1, "$inner");
  // transform some forbidden chars
  let cleaned_3 = cleaned_2
    .replace(':', "..") // No colons!
    .replace('@', "-at-")
    .replace('*', "-star-")
    .replace('$', "-dollar-")
    .replace(',', "-comma-")
    .replace('%', "-pct-")
    .replace('&', "-amp-");
  let cleaned_4 = unidecode(&cleaned_3);
  let cleaned_5 = NON_ID_CHARSET_RE.replace_all(&cleaned_4, ""); // remove everything else.
  cleaned_5.to_string()
}

pub fn clean_bib_key(key: &str) -> String {
  // Originally lc() here, but let's preserve case till Postproc.
  let mut clean_key = key.trim_start();
  clean_key = clean_key.trim_end();
  // ??? key =~ s/\s//sg;
  clean_key.to_string()
}

pub fn clean_label(label: &str, prefix_opt: Option<&str>) -> String {
  let prefix = prefix_opt.unwrap_or("LABEL");
  let mut key = label;
  key = key.trim_start().trim_end(); // Trim leading/trailing, in any case
  s!("{}:{}", prefix, SPACES_RE.replace_all(key, "_"))
}

pub fn clean_url(url: &str) -> String {
  let cleaned = url.trim_start().trim_end(); // Trim leading/trailing, in any case
  TILDE_NOISE_RE.replace_all(&cleaned, "~").to_string()
}

pub fn compose_url(base: &str, url: &str, fragid_opt: Option<&str>) -> String {
  let mut base = TRAILING_SLASH_RE.replace(base, ""); //  remove trailing /
  let mut fragid = fragid_opt.unwrap_or("");
  let base: String = if !base.is_empty() && !LEADING_PROTOCOL_RE.is_match(url) {
    // already has protocol, so is absolute url
    base.to_string() + if url.starts_with('/') { "" } else { "/" } // else start w/base, possibly /
  } else {
    String::new()
  };
  let fragid: String = if !fragid.is_empty() {
    s!("#{}", clean_id(fragid))
  } else {
    String::new()
  };
  clean_url(&(base + url + &fragid))
}
