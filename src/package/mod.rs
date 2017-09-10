pub use std::collections::HashMap;
pub use regex::Regex;
pub use std::rc::Rc;
pub use std::collections::VecDeque;
pub use libxml::tree::{Node, Namespace};

pub use rtx_core::{Core, Digested, BoxOps};
pub use rtx_core::tbox::Tbox;
pub use rtx_core::state::{State, ObjectStore, Scope};
pub use rtx_core::common::error::*;
pub use rtx_core::common::font::Font;
pub use rtx_core::token::*;
pub use rtx_core::parameter::{Parameter, Parameters};
pub use rtx_core::mouth;
pub use rtx_core::mouth::Mouth;
pub use rtx_core::definition::{Definition, BeforeDigestClosure, DigestionClosure,
  ConstructionClosure, ExpansionClosure, ReplacementClosure};
pub use rtx_core::document::Document;
pub use rtx_core::document::resource::*;
pub use rtx_core::document::tag::{TagOptions, TagOptionName};
pub use rtx_core::util::pathname;
pub use rtx_core::token::Token;
pub use rtx_core::tokens::Tokens;
pub use rtx_core::gullet::Gullet;
pub use rtx_core::stomach::Stomach;
pub use rtx_core::whatsit::Whatsit;
pub use rtx_core::definition::ConditionalClosure;
pub use rtx_core::definition::expandable::Expandable;
pub use rtx_core::definition::primitive::{Primitive,PrimitiveOptions};
pub use rtx_core::definition::math_primitive::{MathPrimitive,MathPrimitiveOptions};
pub use rtx_core::definition::constructor::{ConstructorOptions};
pub use rtx_core::definition::conditional::{Conditional, ConditionalType};

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
  match state.lookup_meaning(cs) {
    Some(store) => match *store {
      ObjectStore::Token(ref m) => true,
      ObjectStore::Expandable(ref m) => m.get_cs_name() != "\relax",
      ObjectStore::Primitive(ref m) => m.get_cs_name() != "\relax",
      ObjectStore::Constructor(ref m) => m.get_cs_name() != "\relax",
      _ => false
    },
  _ => false }
}

/// TODO: Flesh out with the full infrastructure, incremental functionality for now.
pub fn input_definitions(raw_file: String, options: InputDefinitionOptions, mut state: &mut State) -> Result<()> {
  let mut file : String = raw_file.trim().to_string();

  // let prevname = if options.handleoptions {
  //   match state.lookup_definition(T_CS!("\@currname")) {
  //     Some(ObjectStore::Expandable(name)) => Digest!(T_CS!("\@currname")).to_string()
  // }
  // let prevext = options.handleoptions && $state->lookupDefinition(T_CS('\@currext')) && ToString(Digest(T_CS('\@currext')));


  // Compute the exact name based on the type
  file = match options.extension {
    None => file,
    Some(ext) => file + "." + ext
  };

  let loaded_flag = file.clone()+"_loaded";
  {
    // Only load definitions once
    if let Some(&ObjectStore::Bool(flag)) = state.lookup_value(&loaded_flag) {
      if flag {
        // do nothing if we've loaded before
        return Ok(());
      }
    }
  }

  // Mark as loaded, then process the definitions
  info!("Loading {:?} definitions...", file);
  state.assign_value(&loaded_flag,
                     ObjectStore::Bool(true),
                     Some(Scope::Global));

  match file.as_ref() {
    "TeX.pool" => try!(pool::tex::load_definitions(&mut state)),
    "LaTeX.pool" => try!(pool::latex::load_definitions(&mut state)),
    "eTeX.pool" => try!(pool::etex::load_definitions(&mut state)),
    "pdfTeX.pool" => try!(pool::pdftex::load_definitions(&mut state)),
    "article.cls" => try!(pool::article_cls::load_definitions(&mut state)),
    "alltt.sty" => try!(pool::alltt_sty::load_definitions(&mut state)),
    "comment.sty" => try!(pool::comment_sty::load_definitions(&mut state)),
    other => { fatal!(Package, Unknown, format!("TODO: unknown binding {:?}, can't load", other)) }
  };

  Ok(())
}

pub fn input_content(core: &mut Core, request: &str) -> Result<()> {
  match find_file(request, false) { // TODO: type => $options{type}, noltxml => 1
    Some(path) => load_tex_content(core, &path),
    None => fatal!(Package, MissingFile, request),
    // TODO:
    // Error("missing_file", request, state.get_stomach().get_gullet(),
    // "Can't find TeX file "+request, maybeReportSearchPaths(state)))
  }
}

pub fn load_tex_content(core: &mut Core, path: &str) -> Result<()> {
  let mut mouth = Mouth { notes: true, ..Mouth::default() };
  try!(mouth.open(path, &mut core.state));
  // TODO:
  // If there is a file-specific declaration file (name.latexml), load it first!
  // let file = path;
  // file =~ s/\.tex//;
  // if (let conf = !pathname_is_literaldata($pathname)
  //   && pathname_find("$file.latexml", paths => LookupValue('SEARCHPATHS'))) {
  //   loadLTXML($conf, $conf); }

  // TODO: Caching
  // content => LookupValue($pathname . '_contents')

  // Open a mouth for that TeX content
  let gullet = core.stomach.get_gullet_mut();
  gullet.open_mouth(mouth, true);
  Ok(())
}

pub struct RequireOptions {
  pub options: Vec<String>,
  pub withoptions: bool,
  pub extension: Option<&'static str>,
  pub as_class: bool,
  pub noltxml: bool,
  pub notex: bool,
  pub raw: bool,
  pub after: bool
}
impl Default for RequireOptions {
  fn default() -> Self {
    RequireOptions {
      options: Vec::new(),
      withoptions: false,
      extension: None,
      as_class: false,
      noltxml: false,
      notex: true,
      raw: false,
      after: false
    }
  }
}

/// Tokenize($string); Tokenizes the string using the standard cattable, returning a LaTeXML::Core::Tokens
macro_rules! Tokenize {
  ($string:expr)=>(mouth::tokenize($string, None));
  ($string:expr, $state:ident)=>(mouth::tokenize($string, Some($state)));
}

/// TokenizeInternal($string); Tokenizes the string using the internal cattable, returning a LaTeXML::Core::Tokens
macro_rules! TokenizeInternal {
  ($string:expr)=>(mouth::tokenize_internal($string, None));
  ($string:expr, $state:ident)=>(mouth::tokenize_internal($string, Some($state)));
}

/// This (and `FindFile`) needs to evolve a bit to support reading raw .sty (.def, etc) files from
/// the standard texmf directories.  Maybe even use kpsewhich itself (INSTEAD of `pathname_find` ???)
/// Another potentially useful option might be that if we are reading a raw file,
/// perhaps it should just get digested immediately, since it shouldn't contribute any boxes.
pub fn require_package(name: String, mut options: RequireOptions, state: &mut State) -> Result<()> {
  if options.raw {
    options.raw = false;
    // Warn('deprecated', 'raw', $STATE->getStomach->getGullet,
    //   "RequirePackage option raw is obsolete; it is not needed");
  }

  // We'll usually disallow raw TeX, unless the option explicitly given, or globally set.
  // $options{notex} = 1
  //   if !defined $options{notex} && !LookupValue('INCLUDE_STYLES') && !$options{noltxml};
  if options.extension.is_none() {
    options.extension = Some("sty");
  }
  // TODO: Ideally we want to use the same struct for the RequirePackage options as for the InputDefinitions options
  input_definitions(name, InputDefinitionOptions {
    extension: options.extension,
    handleoptions: true,
    // Pass classes options if we have NONE!
    withoptions: options.options,
    ..InputDefinitionOptions::default()
  }, state)
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

pub fn load_class(name: String, options: Vec<String>, after: Tokens, state: &mut State) -> Result<()> {
  input_definitions(name, InputDefinitionOptions {
    extension: Some("cls"),
    after: after,
    notex: true,
    handleoptions: true,
    noerror: true,
    ..InputDefinitionOptions::default()
  }, state)
  // if (let success = InputDefinitions($class, type => 'cls', notex => 1, handleoptions => 1, noerror => 1,
  //     %options)) {
  //   return $success; }
  // else {
  //   $STATE->noteStatus(missing => $class . '.cls');
  //   let alternate = 'OmniBus';    # was 'article'
  //   Warn('missing_file', $class, $STATE->getStomach->getGullet,
  //     "Can't find binding for class $class (using $alternate)",
  //     maybeReportSearchPaths());
  //   if (let success = InputDefinitions($alternate, type => 'cls', noerror => 1, handleoptions => 1, %options)) {
  //     return $success; }
  //   else {
  //     Fatal('missing_file', $alternate . '.cls.ltxml', $STATE->getStomach->getGullet,
  //       "Can't find binding for class $alternate (installation error)");
  //     return; } } }
}

pub fn find_file(request: &str, _forbid_ltxml: bool) -> Option<String> {
  // TODO: Actually find it!
  Some(request.to_string())

}

pub fn coerce_cs(t: &str) -> Token {
  T_CS!(t)
}

lazy_static! {
  static ref CSNAME_MACRO_REGEX : Regex = Regex::new(r"^\\csname\s+(.*)\\endcsname").unwrap();
  static ref CS_REGEX : Regex = Regex::new(r"^(\\[a-zA-Z@]+)").unwrap();
  static ref SINGLE_CHAR_REGEX : Regex = Regex::new(r"^(\\.)").unwrap();
  static ref ACTIVE_CHAR_REGEX : Regex = Regex::new(r"^(.)").unwrap();
}

pub fn parse_prototype(proto: &str, state: &mut State) -> Result<((Token, Option<Parameters>))> {
  let mut cs = T_CS!("\\".to_string()); // Should never happen
  let mut final_proto = if CSNAME_MACRO_REGEX.is_match(proto) {
    let captures = CSNAME_MACRO_REGEX.captures(proto).unwrap();
    cs = T_CS!("\\".to_string() + captures.at(0).unwrap());
    // also replace in proto
    CSNAME_MACRO_REGEX.replace(proto, "")
  } else if CS_REGEX.is_match(proto) {
    // Match a cs
    let captures = CS_REGEX.captures(proto).unwrap();
    let csname = captures.at(0).unwrap().to_string();
    cs = T_CS!(csname);
    // also replace in proto
    CS_REGEX.replace(proto, "")
  } else if SINGLE_CHAR_REGEX.is_match(proto) {
    // Match a single char cs, env name,...
    let captures = SINGLE_CHAR_REGEX.captures(proto).unwrap();
    cs = T_CS!(captures.at(0).unwrap().to_string());
    // also replace in proto
    SINGLE_CHAR_REGEX.replace(proto, "")
  } else if ACTIVE_CHAR_REGEX.is_match(proto) {
    // Match an active char
    let captures = ACTIVE_CHAR_REGEX.captures(proto).unwrap();
    cs = TokenizeInternal!(captures.at(0).unwrap()).unlist().first().unwrap().clone();
    // also replace in proto
    ACTIVE_CHAR_REGEX.replace(proto, "")
  } else {
    // Fatal('misdefined', prototype, $STATE->getStomach,
    //   "Definition prototype doesn't have proper control sequence: \"prototype\""); }
    proto.to_string()
  };
  final_proto = final_proto.trim_left().to_string();
  let paramlist = try!(parse_parameters(final_proto, &cs, state));
  Ok((cs, paramlist))
}

lazy_static! {
  static ref NESTED_CHECK : Regex = Regex::new(r"^(\{([^\}]*)\})\s*").unwrap();
  static ref OPTIONAL_CHECK : Regex = Regex::new(r"^(\[([^\]]*)\])\s*").unwrap();
  static ref DEFAULT_CHECK : Regex = Regex::new(r"^Default:(.*)$").unwrap();
  static ref PARAMSPECT_CHECK : Regex = Regex::new(r"^((\w*)(:([^\s\{\[]*))?)\s*").unwrap();
}
pub fn parse_parameters(mut prototype: String, cs: &Token, state: &mut State) -> Result<Option<Parameters>> {
  let mut parameters = Vec::new();
  while !prototype.is_empty() {
    let mut next_proto;
    // Handle possibly nested cases, such as {Number}
    if NESTED_CHECK.is_match(&prototype) {
      let captures = NESTED_CHECK.captures(&prototype).unwrap();
      next_proto = NESTED_CHECK.replace(&prototype, "");
      let spec = captures.at(1).unwrap();
      let inner_spec = captures.at(2).unwrap();
      let inner: Option<Parameters> = if inner_spec.is_empty() {
        None
      } else {
        try!(parse_parameters(inner_spec.to_string(), cs, state))
      };
      parameters.push(try!(Parameter {
                        name: "Plain".to_string(),
                        spec: spec.to_string(),
                        extra: vec![inner],
                        ..Parameter::default()
                      }
                      .init(state)));

    } else if OPTIONAL_CHECK.is_match(&prototype) {
      // Ditto for Optional
      let captures = OPTIONAL_CHECK.captures(&prototype).unwrap();
      next_proto = OPTIONAL_CHECK.replace(&prototype, "");
      let spec = captures.at(1).unwrap();
      let inner_spec = captures.at(2).unwrap();

      if DEFAULT_CHECK.is_match(inner_spec) {
        // let default_captures = DEFAULT_CHECK.captures(&inner_spec).unwrap();
        parameters.push(try!(Parameter {
                          name: "Optional".to_string(),
                          spec: spec.to_string(),
                          // extra: vec![TokenizeInternal(default_captures.at(0).unwrap()), None]});
                          extra: Vec::new(),
                          ..Parameter::default()
                        }
                        .init(state)));
      } else if !inner_spec.is_empty() {
        parameters.push(try!(Parameter {
                          name: "Optional".to_string(),
                          spec: spec.to_string(),
                          extra: vec![None, try!(parse_parameters(inner_spec.to_string(), cs, state))],
                          ..Parameter::default()
                        }
                        .init(state)));
      } else {
        parameters.push(try!(Parameter {
                          name: "Optional".to_string(),
                          spec: spec.to_string(),
                          extra: Vec::new(),
                          ..Parameter::default()
                        }
                        .init(state)));
      }
    } else if PARAMSPECT_CHECK.is_match(&prototype) {
      let captures = PARAMSPECT_CHECK.captures(&prototype).unwrap();
      next_proto = PARAMSPECT_CHECK.replace(&prototype, "");
      let spec = captures.at(1).unwrap();
      let spec_type = captures.at(2).unwrap();
      let extra = match captures.at(4) {
        None => Vec::new(),
        Some(_extra_string) => {
          // TODO: Ask Bruce about the "extra" functionality and its types
          // extra_string.split("|").map(|t| tokenize_internal(t)).collect::<Vec<Token>>();
          Vec::new()
        }
      };
      parameters.push(try!(Parameter {
                        name: spec_type.to_string(),
                        spec: spec.to_string(),
                        extra: extra,
                        ..Parameter::default()
                      }
                      .init(state)));

    } else {
      // Fatal('misdefined', cs, undef, "Unrecognized parameter specification at \"prototype\""); }
      panic!("Fatal:misdefined:{:?} Unrecognized parameter specification at \"prototype\"", cs);
    }
    prototype = next_proto;
  }
  if parameters.is_empty() {
    Ok(None)
  } else {
    Ok(Some(Parameters { params: parameters }))
  }
}

pub fn revert(_arg: &[Token]) -> Tokens {
  Tokens!()
}

//======================================================================
// Declaring and Adjusting the Document Model.
//======================================================================

pub fn install_tag(tag: &str, mut properties: TagOptions, state: &mut State) {
  let mut options = state.tag_properties.entry(tag.to_string()).or_insert_with(TagOptions::default);
  options.auto_open = properties.auto_open;
  options.auto_close = properties.auto_close;

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

pub struct InputDefinitionOptions {
  pub extension: Option<&'static str>,
  pub options: Vec<String>,
  pub after: Tokens,
  pub notex: bool,
  pub noerror: bool,
  pub noltxml: bool,
  pub withoptions: Vec<String>,
  pub handleoptions: bool,
  pub as_class: bool,
}
impl Default for InputDefinitionOptions {
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
      as_class: false
    }
  }
}

// Selects the RelaxNG schema defining the XML output language
pub fn select_relaxng_schema(schema : String, namespaces : Option<HashMap<String,String>>, state: &mut State) {
  // What verb here? Set, Choose,...
  let model = &mut state.model;
  model.set_relaxng_schema(schema);
  if let Some(namespaces) = namespaces {
    for (prefix, value) in namespaces {
      model.register_document_namespace(&prefix, Some(value)); }
  }
  return; }

pub fn def_macro_i(cs: Token, paramlist: Option<Parameters>, expansion: Option<ExpansionClosure>, state: &mut State) {
//       // Optimization: Defer till macro actually used
//       // if !$cs.is_empty() { // && $options{mathactive}
//         // $state.assign_mathcode($cs, 0x8000, $options{scope}); }
//       $state.install_definition(Expandable{ cs: coerce_cs( $cs ), paramlist: $paramlist, expansion: $expansion});//, %options), $options{scope});
//       // if $options{locked} {
//       //   $state.assign_value(ToString($cs)+":locked", true, "global")
//       // }

  state.install_definition(ObjectStore::Expandable(Rc::new(
    Expandable { cs: cs, paramlist: paramlist, expansion: expansion,
     ..Expandable::default()})),
    None);
}


//**********************************************************************
/// This function computes an xml:id for a node, if it hasn't already got one.
/// It is suitable for use in Tag afterOpen as
///  `Tag('ltx:para',afterOpen=>sub { GenerateID(@_,'p'); });`
/// It generates an id of the form <parentid>.<prefix><number>
/// The parent node (the one with ID=<parentid>) also maintains a counter
/// stored in an attribute `_ID_counter_<prefix>` recording the last used
/// <number> for <prefix> amongst its descendents.
pub fn generate_id(document: &mut Document, mut node: &mut Node, mut prefix: &str, state: &mut State) {
  // If node doesn't already have an id, and can
  let node_qname = document.get_node_qname(node, state);
  if node.get_attribute("xml:id").is_none() && document.can_have_attribute(&node_qname, "xml:id", state)
    // but isn't a _Capture_ node (which ultimately should disappear)
    && (node_qname != "ltx:_Capture_") {

    let mut ancestor = document.findnode("ancestor::*[@xml:id][1]", Some(node), state).unwrap_or_else(|| document.get_document().get_root_element());
    //// Old versions don't like ancestor.getAttribute('xml:id');
    let ancestor_id = ancestor.get_attribute_ns("id", "http://www.w3.org/XML/1998/namespace");
    // If we've got no ancestor_id, then we've got no ancestor (no document yet!),
    // or ancestor IS the root element (but without an id);
    // If we also have no prefix, we'll end up with an illegal id (just digits)!!!
    // We'll use "id" for an id prefix; this will work whether or not we have an ancestor.
    if prefix.is_empty() && ancestor_id.is_none() {
      prefix = "id";
    }

    let ctrkey = "_ID_counter_".to_string() + prefix + "_";
    let a_ctr = ancestor.get_attribute(&ctrkey).unwrap_or_else(|| "0".to_string());

    let ctr_int = 1 + a_ctr.parse::<u32>().unwrap_or(0);
    let ctr = ctr_int.to_string();

    let id = match ancestor_id {
      Some(aid) => aid + ".",
      None => String::new()
    } + prefix + &ctr;

    ancestor.set_attribute(&ctrkey, &ctr);
    node.set_attribute("xml:id", &id);
  }
}

pub fn merge_font(font: Font, state: &mut State) {
  let mut current_font = match state.remove_value("font") {
    Some(ObjectStore::Font(f)) => *f,
    _ => Font::text_default(),
  };
  let newfont = current_font.merge(font);
  state.assign_value("font", ObjectStore::Font(Box::new(newfont)), Some(Scope::Local));
  return;
}

// Macros requiring repetitions need to be handled outside of the main setup macro, as nested macros currently don't support repetition
// Details at: https://github.com/rust-lang/rust/issues/35853

macro_rules! Font {
  ($($key:ident => $value:expr),*) => (
    Some(Font { $($key: Some($value.to_string()),)* .. Font::default() })
)}

#[macro_export]
macro_rules! NewDefault {
  ($name:ident, $($key:ident => $value:expr),*) => ($name {
    $($key: $value,)*
    ..$name::default()
  })
}

#[macro_export]
macro_rules! transfer_default {
  ($val:ident, $struct_source:ident, $hash_receiver:ident) => (
    $hash_receiver.entry(stringify!($val).to_owned()).or_insert($struct_source.$val.clone().to_string());
  )
}
#[macro_export]
macro_rules! transfer_opt_default {
  ($val:ident, $struct_source:ident, $hash_receiver:ident) => (
    if let &Some(ref $val) = &$struct_source.$val {
      $hash_receiver.entry(stringify!($val).to_owned()).or_insert($val.to_owned());
    }
  )
}

#[macro_export]
macro_rules! sub {
  ($body:expr) => (vec![Rc::new($body)])
}

#[macro_export]
macro_rules! tagsub {
  ($document:ident, $node:ident, $state:ident, $body:expr) => (vec![Rc::new(
    |$document:&mut Document, mut $node:&mut Node, $state:&mut State| -> Result<()>  {
      $body;
      Ok(())
    })])
}

#[macro_export]
macro_rules! noreplacement {
  () => (|doc,whatsit,props,state|{Ok(())})
}

#[macro_export]
macro_rules! replacement {
  ($doc:ident, $args:ident, $props:ident, $state:ident, $body:expr) => (
    |$doc:&mut Document,$args: &Vec<Option<Digested>>,$props: &HashMap<String, ObjectStore>, $state: &mut State| -> Result<()> {
    $body
    Ok(())
  })
}

#[macro_export]
macro_rules! noprimitive {
  () => ( |stomach:&mut Stomach, args : Vec<Tokens>, state:&mut State| {Ok(Vec::new())})
}

#[macro_export]
macro_rules! primitivesub {
  ($stomach:ident, $args:ident, $state:ident, $body:expr) => (
    |$stomach:&mut Stomach, mut $args : Vec<Tokens>, $state:&mut State| {
      $body
    }
  )
}
#[macro_export]
macro_rules! primitiveproc {
  ($stomach:ident, $args:ident, $state:ident, $body:expr) => (
    |$stomach:&mut Stomach, mut $args : Vec<Tokens>, $state:&mut State| {
      $body
      Ok(Vec::new())
    }
  )
}


#[macro_export]
macro_rules! beforesub {
  ($stomach:ident, $state:ident, $body:expr) => (
    |$stomach:&mut Stomach, $state:&mut State| {
      $body
    }
  )
}
#[macro_export]
macro_rules! beforeproc { // just as beforesub! but with a default return value
  ($stomach:ident, $state:ident, $body:expr) => (
    Rc::new(move |$stomach:&mut Stomach, $state:&mut State| {
      $body;
      Ok(Vec::new())
    }
  ))
}

#[macro_export]
macro_rules! aftersub {
  ($stomach:ident, $whatsit:ident, $state:ident, $body:expr) => (
    |$stomach:&mut Stomach, $whatsit:&mut Whatsit, $state:&mut State| {
      $body
    }
  )
}
#[macro_export]
macro_rules! afterproc {
  ($stomach:ident, $whatsit:ident, $state:ident, $body:expr) => (
    Rc::new(move |$stomach:&mut Stomach, $whatsit:&mut Whatsit, $state:&mut State| {
      $body
      Ok(Vec::new())
    }
  ))
}


// Discussion: It is unclear what the best authoring syntax is for our family of latexml binding macros.
// One idea is to keep them very close to the Rust internals, but we suffer from a variety of boilerplate, such as
// needing to spell out `key => Some(value.to_string())`, rather than a direct `key => value`.
//
// For now I am making the decision to keep writing out the verbose form,
// and will refactor at a later date, when the trade-offs become more clear. Smart use of the Cow struct is another idea.
// I will use a helper though:

#[macro_export]
macro_rules! v {
  ($val:expr) => (Some($val.to_string()))
}
/// Macros and pool come at the end, so that they load seamlessly

// We need to invoke constructors within constructors. This is only possible with locally passed State arguments,
// IF we have a macro form that explicitly accepts state and has no pseudo-global $state in its initialization.

#[macro_export]
macro_rules! SetupBindingMacros {($state:ident) => (
  #[allow(unused_macros)]
  //============================================
  // Convenience macros for writing definitions.
  //============================================
  macro_rules! LookupValue {
    ($name:expr) => (LookupValue!($name, $state));
    ($name:expr, $state_arg:ident) => ($state_arg.lookup_value($name))
  }
  macro_rules! LookupBool {
    ($name:expr) => (LookupBool!($name, $state));
    ($name:expr, $state_arg:ident) => ($state_arg.lookup_bool($name))
  }
  macro_rules! LookupString {
    ($name:expr) => (LookupString!($name, $state));
    ($name:expr, $state_arg:ident) => ($state_arg.lookup_string($name))
  }
  macro_rules! AssignValue {
    ($name:expr, $value:expr) => (AssignValue!($name, $value, None, $state));
    ($name:expr, $value:expr, $scope:expr) => (AssignValue!($name, $value, $scope, $state));
    ($name:expr, $value:expr, $scope:expr, $state_arg:ident) => ($state_arg.assign_value($name, $value, $scope))
  }
  macro_rules! RemoveValue {
    ($name:expr) => (RemoveValue!($name, $state));
    ($name:expr, $state_arg:ident) => ($state_arg.remove_value($name))
  }
  macro_rules! PushValue {
    ($name:expr, $values:expr) => (PushValue!($name, $values, $state));
    ($name:expr, $values:expr, $state_arg:ident) => ($state_arg.push_value($name, $values))
  }
  macro_rules! PopValue  {
    ($name:expr) => (PopValue!($name, $state));
    ($name:expr, $state_arg:ident) => ($state_arg.pop_value($name))
  }
  macro_rules! UnshiftValue {
    ($name:expr, $values:expr) => (UnshiftValue!($name, $values, $state));
    ($name:expr, $values:expr,$state_arg:ident) => ($state_arg.unshift_value($name, $values))
  }
  macro_rules! ShiftValue {
    ($name:expr) => (ShiftValue!($name, $state));
    ($name:expr,$state_arg:ident) => ($state_arg.shift_value($name))
  }
  macro_rules! LookupMapping {
    ($map:expr, $key:expr) => (LookupValue!($map, $key, $state));
    ($map:expr, $key:expr, $state_arg:ident) => ($state_arg.lookup_mapping($map, $key))
  }
  macro_rules! AssignMapping {
    ($map:expr, $key:expr, $value:expr) => (AssignMapping!($map, $key, $value, $state));
    ($map:expr, $key:expr, $value:expr, $state_arg:ident) => ($state_arg.assign_mapping($map, $key, $value))
  }
  macro_rules! LookupMappingKeys {
    ($map:expr) => (LookupMappingKeys!($map, $state));
    ($map:expr, $state_arg:ident) => ($state_arg.lookup_mapping_keys($map))
  }
  macro_rules! LookupCatcode {
    ($char:expr) => (LookupCatcode!($char, $state));
    ($char:expr, $state_arg:ident) => ($state_arg.lookup_catcode($char))
  }
  macro_rules! AssignCatcode {
    ($char:expr, $catcode:expr, $scope:expr) => (AssignCatcode!($char, $catcode, $scope, $state));
    ($char:expr, $catcode:expr, $scope:expr, $state_arg:ident) => ($state_arg.assign_catcode($char, $catcode, $scope));
  }
  macro_rules! LookupMeaning {
    ($name:expr) => (LookupMeaning!($name, $state));
    ($name:expr, $state_arg:ident) => ($state_arg.lookup_meaning($name))
  }
  macro_rules! LookupDefinition {
    ($name:expr) => (LookupDefinition!($name, $state));
    ($name:expr, $state_arg:ident) => ($state_arg.lookup_definition($name))
  }

  macro_rules! InstallDefinition {
    ($name:expr, $definition:expr, $scope:expr) => (InstallDefinition!($name, $definition, $scope, $state));
    ($name:expr, $definition:expr, $scope:expr, $state_arg:ident) => ($state_arg.install_definition($name, $definition, $scope))
  }

  macro_rules! XEquals {
    ($token1:expr, $token2:expr) => (XEquals!($token1, $token2, $state));
    ($token1:expr, $token2:expr, $state_arg:ident) => ($state_arg.x_equals($token1, $token2))
  }

  macro_rules! IsDefined {
    ($name:expr) => (IsDefined!($name, $state));
    ($name:expr, $state_arg:ident) => (is_defined_token($name, $state_arg))
  }
  macro_rules! IsDefinedToken {($name:expr) => (IsDefinedToken!($name, $state))}
  macro_rules! Let {
    ($token1:expr, $token2:expr) => (Let!($token1, $token2, $state));
    ($token1:expr, $token2:expr, $state_arg:ident) => ({
      LetI!(&T_CS!($token1), T_CS!($token2), $state_arg)
    });
    ($token1:expr, $token2:expr, $scope:expr, $state_arg:ident) => ({
      LetI!(&T_CS!($token1), T_CS!($token2), $scope, $state_arg)
    });
  }
  macro_rules! LetI {
    ($token1:expr, $token2:expr) => (LetI!($token1, $token2, $state));
    ($token1:expr, $token2:expr, $state_arg:ident) => ($state_arg.let_i($token1, $token2, None));
    ($token1:expr, $token2:expr, $scope:expr, $state_arg:ident) => ($state_arg.let_i($token1, $token2, $scope));
  }
  // macro_rules! Digest {
    // ($tokens:expr) => (Digest!($tokens, $state))
    //   ($tokens:expr, $core:ident) => ($core.stomach.digest($tokens, $core.state);)
  // }
  macro_rules! AfterAssignment {
    () => (AfterAssignment!($state));
    ($state_arg:ident) => ({
      // TODO
    })
  }
  // Merge the current font with the style specifications
  macro_rules! MergeFont {
    ($kv:expr) => (MergeFont!($kv, $state));
    ($kv:expr, $state_arg:ident) => (merge_font($kv, $state_arg))
  }

  //======================================================================
  // Defining new Control-sequence Parameter types.
  //======================================================================
  macro_rules! DefParameterType{
    ($name:expr) => (DefParameterType!($name, $state));
    ($name:expr, $key1:ident => $val1:expr)=>(DefParameterType!($name, $key1=>$val1, $state));
    ($name:expr, $key1:ident => $val1:expr,
      $key2:ident=>$val2:expr)=>(DefParameterType!($name, $key1=>$val1, $key2=>$val2, $state));
    ($name:expr, $key1:ident => $val1:expr,
      $key2:ident=>$val2:expr,
      $key3:ident=>$val3:expr)=>(DefParameterType!($name, $key1=>$val1, $key2=>$val2, $key3=>$val3, $state));
    ($name:expr, $key1:ident => $val1:expr,
      $key2:ident=>$val2:expr,
      $key3:ident=>$val3:expr,
      $key4:ident=>$val4:expr)=>(DefParameterType!($name, $key1=>$val1, $key2=>$val2, $key3=>$val3, $key4=>$val4, $state));
    ($name:expr, $key1:ident => $val1:expr,
      $key2:ident=>$val2:expr,
      $key3:ident=>$val3:expr,
      $key4:ident=>$val4:expr,
      $key5:ident=>$val5:expr)=>(DefParameterType!($name, $key1=>$val1, $key2=>$val2, $key3=>$val3, $key4=>$val4, $key5=>$val5, $state));
    ($name:expr, $key1:ident => $val1:expr,
      $key2:ident=>$val2:expr,
      $key3:ident=>$val3:expr,
      $key4:ident=>$val4:expr,
      $key5:ident=>$val5:expr,
      $key6:ident=>$val6:expr)=>(DefParameterType!($name, $key1=>$val1, $key2=>$val2, $key3=>$val3, $key4=>$val4, $key5=>$val5, $key6=>$val6, $state));

    // Explicit state form
    ($name:expr, $state_arg:ident) => (DefParameterTypeWO!($name, Parameter::default(), $state_arg));

    ($name:expr,
     $key1:ident => $val1:expr, $state_arg:ident
    ) => (DefParameterTypeWO!($name, NewDefault!(Parameter,
     $key1 => $val1), $state_arg));

    ($name:expr,
     $key1:ident => $val1:expr,
     $key2:ident => $val2:expr, $state_arg:ident
    ) => (DefParameterTypeWO!($name, NewDefault!(Parameter,
     $key1 => $val1,
     $key2 => $val2
    ), $state_arg));

    ($name:expr,
     $key1:ident => $val1:expr,
     $key2:ident => $val2:expr,
     $key3:ident => $val3:expr, $state_arg:ident
    ) => (DefParameterTypeWO!($name, NewDefault!(Parameter,
     $key1 => $val1,
     $key2 => $val2,
     $key3 => $val3
    ), $state_arg));

    ($name:expr,
     $key1:ident => $val1:expr,
     $key2:ident => $val2:expr,
     $key3:ident => $val3:expr,
     $key4:ident => $val4:expr, $state_arg:ident
    ) => (DefParameterTypeWO!($name, NewDefault!(Parameter,
     $key1 => $val1,
     $key2 => $val2,
     $key3 => $val3,
     $key4 => $val4
    ), $state_arg));

    ($name:expr,
     $key1:ident => $val1:expr,
     $key2:ident => $val2:expr,
     $key3:ident => $val3:expr,
     $key4:ident => $val4:expr,
     $key5:ident => $val5:expr, $state_arg:ident
    ) => (DefParameterTypeWO!($name, NewDefault!(Parameter,
     $key1 => $val1,
     $key2 => $val2,
     $key3 => $val3,
     $key4 => $val4,
     $key5 => $val5,
    ), $state_arg));

    ($name:expr,
     $key1:ident => $val1:expr,
     $key2:ident => $val2:expr,
     $key3:ident => $val3:expr,
     $key4:ident => $val4:expr,
     $key6:ident => $val6:expr, $state_arg:ident
    ) => (DefParameterTypeWO!($name, NewDefault!(Parameter,
     $key1 => $val1,
     $key2 => $val2,
     $key3 => $val3,
     $key4 => $val4,
     $key5 => $val5,
     $key6 => $val6
    ), $state_arg));
  }
  macro_rules! DefParameterTypeWO {
    ($name:expr, $param:expr, $state_arg:ident) => ($state_arg.assign_mapping("PARAMETER_TYPES", $name, Some(ObjectStore::Parameter($param))))
  }

  macro_rules! LoadPool{
    ($name:expr) => (LoadPool!($name, $state));
    ($name:expr, $state_arg:ident) => (try!(input_definitions($name.to_string(),
      InputDefinitionOptions {
        extension: Some("pool"),
        ..InputDefinitionOptions::default()
      }, $state_arg)))
  }
  /// Loader shorthand for pool dependencies
  macro_rules! InnerPool {
    ($name:ident) => (InnerPool!($name, $state));
    ($name:ident, $state_arg:ident) => (try!(pool::$name::load_definitions(&mut $state_arg)))
  }

  macro_rules! RequirePackage{
    ($package:expr, $options:expr) => (RequirePackage!($package, $options, $state));
    ($package:expr, $options:expr, $state_arg:ident) => (require_package($package, $options, $state_arg))
  }
  macro_rules! LoadClass{
    ($class:expr, $options:expr, $after:expr) => (LoadClass!($class, $options, $after, $state));
    ($class:expr, $options:expr, $after:expr, $state_arg:ident) => (load_class($class, $options, $after, $state_arg))
  }
  macro_rules! DefMacroI(
    ($cs:expr, $paramlist:expr, $expansion:expr) => (DefMacroI!($cs, $paramlist, $expansion, $state));
    ($cs:expr, $paramlist:expr, $expansion:expr, $key1:ident=>$val1:expr) => (DefMacroI!($cs, $paramlist, $expansion, $key1=>$val1, $state));

    // With explicit state
    // TODO: package::coerce_cs on $cs
    ($cs:expr, $paramlist:expr, None, $state_arg:expr) => (def_macro_i($cs, $paramlist, None, $state_arg));
    ($cs:expr, $paramlist:expr, $expansion:expr, $state_arg:expr) => (def_macro_i($cs, $paramlist, Some(Rc::new($expansion)), $state_arg));
    // TODO: Use the definitional options such as "locked"
    ($cs:expr, $paramlist:expr, $expansion:expr, $key1:ident=>$val1:expr, $state_arg:expr) => (def_macro_i($cs, $paramlist, Some(Rc::new($expansion)), $state_arg));
  );

  macro_rules! DefMacroT(
    // Tokens form
    ($cs:expr, $paramlist:expr, $arg:expr) => (DefMacroT!($cs, $paramlist, $arg, $state));
    ($cs:expr, $paramlist:expr, $body:expr, $state_arg:ident) => ({
      DefMacroI!($cs, $paramlist, move |_gullet, _args, _state| {Ok(Tokens!($body))}, $state_arg)
    });
  );
  macro_rules! DefMacro(
    // String expansion forms
    ($proto:expr, $expansion:expr) => (DefMacro!($proto, $expansion, $state));
    ($proto:expr, $expansion:expr, $state_arg:ident) => ({
      let (cs, paramlist) = try!(parse_prototype($proto, $state_arg));
      let expansion;
      compile_expansion!(expansion, $expansion);
      def_macro_i(cs, paramlist, expansion, $state_arg);
    });
    // Rust closure expansion form
    ($proto:expr, $gullet:ident, $args:ident, $inner_state:ident, $block:expr) => (DefMacro!($proto, $gullet, $args, $inner_state, $block, $state));
    ($proto:expr, $gullet:ident, $args:ident, $inner_state:ident, $block:expr, $state_arg:ident) => ({
      let (cs, paramlist) = try!(parse_prototype($proto, $state_arg));
      def_macro_i(cs, paramlist, Some(Rc::new(|$gullet, $args, $inner_state| {$block})), $state_arg);
    })
  );


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

// This is ONLY used for \ifcase.
// my $conditional_options = {    # [CONSTANT]
//   scope => 1, locked => 1, skipper => 1 };
  macro_rules! DefConditional(
    // test is always a rust closure
    ($proto:expr, $gullet:ident, $args:ident, $inner_state:ident, $block:expr) => (DefConditional!($proto, $gullet, $args, $inner_state, $block, $state));
    ($proto:expr, $gullet:ident, $args:ident, $inner_state:ident, $block:expr, $state_arg:ident) => ({
      let (cs, paramlist) = try!(parse_prototype($proto, $state_arg));
      DefConditionalI!(cs, paramlist, $gullet, $args, $inner_state, $block, $state_arg)
    })
  );

  macro_rules! DefConditionalI(
    // test is always a rust closure
    ($cs:expr, $paramlist:expr, $gullet:ident, $args:ident, $inner_state:ident, $block:expr) =>
      (DefConditionalI!($cs, $paramlist, $gullet, $args, $inner_state, $block, $state));
    ($cs:expr, $paramlist:expr, $gullet:ident, $args:ident, $inner_state:ident, $block:expr, $state_arg:ident) => ({
      let test : ConditionalClosure = Rc::new(|$gullet, $args, $inner_state| {$block});
      // match $cs.get_cs_name() {
      //   "\\fi" => $state_arg.install_definition(ObjectStore::Conditional(Rc::new(
      //               Conditional { cs: $cs, paramlist: None, test: None, conditional_type: ConditionalType::Fi}
      //               )))
      //   "\\else" => ,
      //   "\\or" => ,
      //   csname => {
      //     if

      //   }
      // }
      $state_arg.install_definition(ObjectStore::Conditional(Rc::new(
        Conditional { cs: $cs, paramlist: $paramlist, test: Some(test),
         ..Conditional::default()})),
        None);
    })
  );
//   if ($csname eq '\fi') {
//     $STATE->installDefinition(LaTeXML::Core::Definition::Conditional->new(
//         $cs, undef, undef, conditional_type => 'fi', %options),
//       $options{scope}); }
//   elsif ($csname eq '\else') {
//     $STATE->installDefinition(LaTeXML::Core::Definition::Conditional->new(
//         $cs, undef, undef, conditional_type => 'else', %options),
//       $options{scope}); }
//   elsif ($csname eq '\or') {
//     $STATE->installDefinition(LaTeXML::Core::Definition::Conditional->new(
//         $cs, undef, undef, conditional_type => 'or', %options),
//       $options{scope}); }
//   elsif ($csname =~ /^\\(?:if(.*)|unless)$/) {
//     my $name = $1;
//     if ((defined $name) && ($name ne 'case')
//       && (!defined $test)) {    # user-defined conditional, like with \newif
//       DefMacroI(T_CS('\\' . $name . 'true'),  undef, Tokens(T_CS('\let'), $cs, T_CS('\iftrue')));
//       DefMacroI(T_CS('\\' . $name . 'false'), undef, Tokens(T_CS('\let'), $cs, T_CS('\iffalse')));
//       Let($cs, T_CS('\iffalse')); }
//     else {
//       # For \ifcase, the parameter list better be a single Number !!
//       $STATE->installDefinition(LaTeXML::Core::Definition::Conditional->new($cs, $paramlist, $test,
//           conditional_type => 'if', %options),
//         $options{scope}); }
//   }
//   else {
//     Error('misdefined', $cs, $STATE->getStomach,
//       "The conditional " . Stringify($cs) . " is being defined but doesn't start with \\if"); }
//   AssignValue(ToString($cs) . ":locked" => 1) if $options{locked};
//   return; }

// sub IfCondition {
//   my ($if, @args) = @_;
//   my $gullet = $STATE->getStomach->getGullet;
//   $if = coerceCS($if);
//   my ($defn, $test);
//   if (($defn = $STATE->lookupDefinition($if))
//     && (($$defn{conditional_type} || '') eq 'if') && ($test = $defn->getTest)) {
//     return &$test($gullet, @args); }
//   elsif (XEquals($if, T_CS('\iftrue'))) {
//     return 1; }
//   elsif (XEquals($if, T_CS('\iffalse'))) {
//     return 0; }
//   else {
//     Error('expected', 'conditional', $gullet,
//       "Expected a conditional, got '" . ToString($if) . "'");
//     return; } }

// # Used only for regular \newif type conditions
// sub SetCondition {
//   my ($if, $value, $scope) = @_;
//   my ($defn, $test);
//   # We'll accept any conditional \ifxxx, providing it takes no arguments
//   if (($defn = $STATE->lookupDefinition($if)) && (($$defn{conditional_type} || '') eq 'if')
//     && !$defn->getParameters) {
//     Let($if, ($value ? T_CS('\iftrue') : T_CS('\iffalse')), $scope) }
//   else {
//     Error('expected', 'conditional', $STATE->getStomach,
//       "Expected a conditional defined by \\newif, got '" . ToString($if) . "'"); }
//   return; }

  ///======================================================================
  /// Define a primitive control sequence.
  ///======================================================================
  /// Primitives are executed in the Stomach.
  /// The $replacement should be a sub which returns nothing, or a list of `Box`'s or `Whatsit`'s.
  /// The options are:
  ///    isPrefix  : 1 for things like \global, \long, etc.
  ///    registerType : for parameters (but needs to be worked into `DefParameter`, below).
  macro_rules! DefPrimitive{
    ($proto:expr, $replacement:expr, $options:expr) => (DefPrimitive!($proto, $replacement, $options, $state));
    ($proto:expr, $replacement:expr, $options:expr, $state_arg:ident) => ({
      // TODO:
      // let compiled_replacement = || Tbox{text: $replacement, Invocation($options{alias} || $cs, @_[1 .. $#_])); }
      let compiled_replacement = $replacement;

      DefPrimitiveIWO!($proto, compiled_replacement, $options, $state_arg);
    });
  }

  macro_rules! DefPrimitiveI{
    ($proto:expr, $compiled_replacement:expr) => (DefPrimitiveI!($proto, $compiled_replacement, $state));
    ($proto:expr, $compiled_replacement:expr,
      $key1:ident => $val1:expr)=>(DefPrimitiveI!($proto, $compiled_replacement, $key1=>$val1, $state));
    ($proto:expr, $compiled_replacement:expr,
      $key1:ident => $val1:expr,
      $key2:ident=>$val2:expr)=>(DefPrimitiveI!($proto, $compiled_replacement, $key1=>$val1, $key2=>$val2, $state));
    ($proto:expr, $compiled_replacement:expr,
      $key1:ident => $val1:expr,
      $key2:ident=>$val2:expr,
      $key3:ident=>$val3:expr)=>(DefPrimitiveI!($proto, $compiled_replacement, $key1=>$val1, $key2=>$val2, $key3=>$val3, $state));
    ($proto:expr, $compiled_replacement:expr,
      $key1:ident => $val1:expr,
      $key2:ident=>$val2:expr,
      $key3:ident=>$val3:expr,
      $key4:ident=>$val4:expr)=>(DefPrimitiveI!($proto, $compiled_replacement, $key1=>$val1, $key2=>$val2, $key3=>$val3, $key4=>$val4, $state));
    ($proto:expr, $compiled_replacement:expr,
      $key1:ident => $val1:expr,
      $key2:ident=>$val2:expr,
      $key3:ident=>$val3:expr,
      $key4:ident=>$val4:expr,
      $key5:ident=>$val5:expr)=>(DefPrimitiveI!($proto, $compiled_replacement, $key1=>$val1, $key2=>$val2, $key3=>$val3, $key4=>$val4, $key5=>$val5, $state));

    ($proto:expr, $compiled_replacement:expr, $state_arg:ident) => (DefPrimitiveIWO!($proto,$compiled_replacement, PrimitiveOptions::default(), $state_arg));

    ($proto:expr, $compiled_replacement:expr,
      $key1:ident=>$val1:expr, $state_arg:ident
    ) => (DefPrimitiveIWO!($proto, $compiled_replacement, NewDefault!(PrimitiveOptions,
      $key1 => $val1
    ), $state_arg));

    ($proto:expr, $compiled_replacement:expr,
      $key1:ident=>$val1:expr,
      $key2:ident=>$val2:expr, $state_arg:ident
    ) => (DefPrimitiveIWO!($proto, $compiled_replacement, NewDefault!(PrimitiveOptions,
      $key1 => $val1,
      $key2 => $val2
    ), $state_arg));

    ($proto:expr, $compiled_replacement:expr,
      $key1:ident=>$val1:expr,
      $key2:ident=>$val2:expr,
      $key3:ident=>$val3:expr, $state_arg:ident
    ) => (DefPrimitiveIWO!($proto, $compiled_replacement, NewDefault!(PrimitiveOptions,
      $key1 => $val1,
      $key2 => $val2,
      $key3 => $val3
    ), $state_arg));

    ($proto:expr, $compiled_replacement:expr,
      $key1:ident=>$val1:expr,
      $key2:ident=>$val2:expr,
      $key3:ident=>$val3:expr,
      $key4:ident=>$val4:expr, $state_arg:ident
    ) => (DefPrimitiveIWO!($proto, $compiled_replacement, NewDefault!(PrimitiveOptions,
      $key1 => $val1,
      $key2 => $val2,
      $key3 => $val3,
      $key4 => $val4
    ), $state_arg));

    ($proto:expr, $compiled_replacement:expr,
      $key1:ident=>$val1:expr,
      $key2:ident=>$val2:expr,
      $key3:ident=>$val3:expr,
      $key4:ident=>$val4:expr,
      $key5:ident=>$val5:expr, $state_arg:ident
    ) => (DefPrimitiveIWO!($proto, $compiled_replacement, NewDefault!(PrimitiveOptions,
      $key1 => $val1,
      $key2 => $val2,
      $key3 => $val3,
      $key4 => $val4,
      $key5 => $val5
    ), $state_arg));
  }

  macro_rules! DefPrimitiveII{
    ($cs:expr, $paramlist:expr, $compiled_replacement:expr, $options:expr) => (DefPrimitiveII!($cs, $paramlist, $compiled_replacement, $options, $state));
    ($cs:expr, $paramlist:expr, $compiled_replacement:expr, $options:expr, $state_arg:ident) => ({
      let options = $options;
      let options_locked = options.locked;
      let scope = options.scope.clone();
      let mut before_digest_env : Vec<BeforeDigestClosure> = Vec::new();

      if options.require_math {
        let cs_name = $cs.get_cs_name();
        let require_math_closure = beforeproc!(stomach, state, { requireMath!(cs_name, state) });
        before_digest_env.push(require_math_closure);
      }

      if options.forbid_math {
        let cs_name = $cs.get_cs_name();
        let forbid_math_closure = beforeproc!(stomach, state, { forbidMath!(cs_name, state) });
        before_digest_env.push(forbid_math_closure);
      }
      if let Some(ref mode) = options.mode {
        let mode_clone = mode.clone();
        let begin_mode_closure = beforeproc!(stomach, state, { try!(stomach.begin_mode(&mode_clone, state)); });
        before_digest_env.push(begin_mode_closure);
      }
      if options.bounded {
        let bgroup_closure = beforeproc!(stomach, state, { stomach.bgroup(state); });
        before_digest_env.push(bgroup_closure);
      }
      if let Some(chosen_font) = options.font {
        let merge_font_closure = beforeproc!(stomach, state, {
          MergeFont!(chosen_font.clone(), state);
        });
        before_digest_env.push(merge_font_closure);
      }
      before_digest_env.extend(options.before_digest);

      let mut after_digest_env : Vec<DigestionClosure> = Vec::new();
      after_digest_env.extend(options.after_digest);
      if let Some(ref mode) = options.mode {
        let mode_clone = mode.clone();
        let end_mode_closure = afterproc!(stomach, whatsit, state, { try!(stomach.end_mode(&mode_clone, state)); });
        after_digest_env.push(end_mode_closure);
      }
      if options.bounded {
        let egroup_closure = afterproc!(stomach, whatsit,state, { try!(stomach.egroup(state)); });
        after_digest_env.push(egroup_closure);
      }

      $state_arg.install_definition(ObjectStore::Primitive(Rc::new(Primitive{
          cs: $cs.clone(),
          paramlist: $paramlist,
          replacement: Some(Rc::new($compiled_replacement)),
          options: PrimitiveOptions {
            before_digest: before_digest_env,
            after_digest: after_digest_env,
            ..PrimitiveOptions::default()
          }
        })),
        scope);
      if options_locked {
        AssignValue!(&($cs.to_string()+":locked"), ObjectStore::Bool(true), None, $state_arg);
      }
    })
  }

  macro_rules! DefPrimitiveIWO(
    ($proto:expr, $compiled_replacement:expr, $options:expr, $state_arg:ident) => ({
      let (cs, paramlist) = try!(parse_prototype($proto, $state_arg));
      DefPrimitiveII!(cs, paramlist, $compiled_replacement, $options, $state_arg);
    })
  );

  macro_rules! DefConstructorI {
    ($cs:expr, $paramlist:expr, $compiled_replacement:expr) => (DefConstructorI!($cs, $paramlist, $compiled_replacement, $state));
    ($cs:expr, $paramlist:expr, $compiled_replacement:expr,
      $key1:ident => $val1:expr)=>(DefConstructorI!($cs, $paramlist, $compiled_replacement, $key1=>$val1, $state));
    ($cs:expr, $paramlist:expr, $compiled_replacement:expr,
      $key1:ident => $val1:expr,
      $key2:ident=>$val2:expr)=>(DefConstructorI!($cs, $paramlist, $compiled_replacement, $key1=>$val1, $key2=>$val2, $state));
    ($cs:expr, $paramlist:expr, $compiled_replacement:expr,
      $key1:ident => $val1:expr,
      $key2:ident=>$val2:expr,
      $key3:ident=>$val3:expr)=>(DefConstructorI!($cs, $paramlist, $compiled_replacement, $key1=>$val1, $key2=>$val2, $key3=>$val3, $state));
    ($cs:expr, $paramlist:expr, $compiled_replacement:expr,
      $key1:ident => $val1:expr,
      $key2:ident=>$val2:expr,
      $key3:ident=>$val3:expr,
      $key4:ident=>$val4:expr)=>(DefConstructorI!($cs, $paramlist, $compiled_replacement, $key1=>$val1, $key2=>$val2, $key3=>$val3, $key4=>$val4, $state));
    ($cs:expr, $paramlist:expr, $compiled_replacement:expr,
      $key1:ident => $val1:expr,
      $key2:ident=>$val2:expr,
      $key3:ident=>$val3:expr,
      $key4:ident=>$val4:expr,
      $key5:ident=>$val5:expr)=>(DefConstructorI!($cs, $paramlist, $compiled_replacement, $key1=>$val1, $key2=>$val2, $key3=>$val3, $key4=>$val4, $key5=>$val5, $state));

    // with explicit state:
    ($cs:expr, $paramlist:expr, $compiled_replacement:expr, $state_arg:ident) => (DefConstructorIWO!($cs, $paramlist, Some(Rc::new($compiled_replacement)), ConstructorOptions::default(), $state_arg));
    ($cs:expr, $paramlist:expr, $compiled_replacement:expr,
      $key1:ident => $val1:expr,
      $state_arg:ident
    ) => (DefConstructorIWO!($cs, $paramlist, Some(Rc::new($compiled_replacement)), NewDefault!(ConstructorOptions,
      $key1 => $val1
    ),$state_arg));

    ($cs:expr, $paramlist:expr, $compiled_replacement:expr,
      $key1:ident => $val1:expr,
      $key2:ident => $val2:expr,
      $state_arg:ident
    ) => (DefConstructorIWO!($cs, $paramlist, Some(Rc::new($compiled_replacement)), NewDefault!(ConstructorOptions,
      $key1 => $val1,
      $key2 => $val2
    ), $state_arg));

    ($cs:expr, $paramlist:expr, $compiled_replacement:expr,
      $key1:ident => $val1:expr,
      $key2:ident => $val2:expr,
      $key3:ident => $val3:expr,
      $state_arg:ident
    ) => (DefConstructorIWO!($cs, $paramlist, Some(Rc::new($compiled_replacement)), NewDefault!(ConstructorOptions,
      $key1 => $val1,
      $key2 => $val2,
      $key3 => $val3
    ), $state_arg));

    ($cs:expr, $paramlist:expr, $compiled_replacement:expr,
      $key1:ident=>$val1:expr,
      $key2:ident=>$val2:expr,
      $key3:ident=>$val3:expr,
      $key4:ident=>$val4:expr,
      $state_arg:ident
    ) => (DefConstructorIWO!($cs, $paramlist, Some(Rc::new($compiled_replacement)), NewDefault!(ConstructorOptions,
      $key1 => $val1,
      $key2 => $val2,
      $key3 => $val3,
      $key4 => $val4
    ), $state_arg));

    ($cs:expr, $paramlist:expr, $compiled_replacement:expr,
      $key1:ident=>$val1:expr,
      $key2:ident=>$val2:expr,
      $key3:ident=>$val3:expr,
      $key4:ident=>$val4:expr,
      $key5:ident=>$val5:expr,
      $state_arg:ident
    ) => (DefConstructorIWO!($cs, $paramlist, Some(Rc::new($compiled_replacement)), NewDefault!(ConstructorOptions,
      $key1 => $val1,
      $key2 => $val2,
      $key3 => $val3,
      $key4 => $val4,
      $key5 => $val5
    ), $state_arg));
  }

  macro_rules! DefConstructorIWO {
    ($cs:expr, $paramlist:expr, $compiled_replacement:expr, $options:expr, $state_arg:ident) => (
    {
      use rtx_core::definition::constructor::Constructor;
      let options = $options;
      // TODO: This won't work, as we can only invoke method calls on paramlist in runtime
      //*rtx_codegen::constructable::NARGS = $paramlist.get_num_args();
      if options.locked {
        $state_arg.assign_value(&format!("{}:locked",$cs.get_cs_name()), ObjectStore::Bool(true), None)
      }
      let constructor = Constructor {
        cs: $cs,
        paramlist: $paramlist,
        replacement: $compiled_replacement,
        options: options};

      $state_arg.install_definition(ObjectStore::Constructor(Rc::new(constructor)), None);
  //   before_digest => flatten(($options{requireMath} ? (sub { requireMath($cs); }) : ()),
  //     ($options{forbidMath} ? (sub { forbidMath($cs); }) : ()),
  //     ($mode ? (sub { $_[0]->beginMode($mode); })
  //       : ($bounded ? (sub { $_[0]->bgroup; }) : ())),
  //     ($options{font} ? (sub { MergeFont(%{ $options{font} }); }) : ()),
  //     $options{before_digest}),
  //   after_digest => flatten($options{after_digest},
  //     ($mode ? (sub { $_[0]->endMode($mode) })
  //       : ($bounded ? (sub { $_[0]->egroup; }) : ()))),
  //   beforeConstruct => flatten($options{beforeConstruct}),
  //   afterConstruct  => flatten($options{afterConstruct}),
  //   nargs           => $options{nargs},
  //   alias           => $options{alias},
  //   reversion       => $options{reversion},
  //   sizer           => inferSizer($options{sizer}, $options{reversion}),
  //   captureBody     => $options{captureBody},
  //   properties      => $options{properties} || {}),
  // $options{scope});
   })
  }

  macro_rules! DefConstructor(
    // String replacement flavors
    ($cs:expr, $replacement:expr) => (DefConstructor!($cs, $replacement, $state));
    ($cs:expr, $replacement:expr,
      $key1:ident => $val1:expr)=>(DefConstructor!($cs, $replacement, $key1=>$val1, $state));
    ($cs:expr, $replacement:expr,
      $key1:ident => $val1:expr,
      $key2:ident=>$val2:expr)=>(DefConstructor!($cs, $replacement, $key1=>$val1, $key2=>$val2, $state));
    ($cs:expr, $replacement:expr,
      $key1:ident => $val1:expr,
      $key2:ident=>$val2:expr,
      $key3:ident=>$val3:expr)=>(DefConstructor!($cs, $replacement, $key1=>$val1, $key2=>$val2, $key3=>$val3, $state));
    ($cs:expr, $replacement:expr,
      $key1:ident => $val1:expr,
      $key2:ident=>$val2:expr,
      $key3:ident=>$val3:expr,
      $key4:ident=>$val4:expr)=>(DefConstructor!($cs, $replacement, $key1=>$val1, $key2=>$val2, $key3=>$val3, $key4=>$val4, $state));
    ($cs:expr, $replacement:expr,
      $key1:ident => $val1:expr,
      $key2:ident=>$val2:expr,
      $key3:ident=>$val3:expr,
      $key4:ident=>$val4:expr,
      $key5:ident=>$val5:expr)=>(DefConstructor!($cs, $replacement, $key1=>$val1, $key2=>$val2, $key3=>$val3, $key4=>$val4, $key5=>$val5, $state));

    // with explicit state:
    ($cs:expr, $replacement:expr, $state_arg:ident) => (DefConstructorWO!($cs, $replacement, ConstructorOptions::default(), $state_arg));
    ($cs:expr, $replacement:expr, $key1:ident=>$val1:expr, $state_arg:ident) =>
      (DefConstructorWO!($cs, $replacement, NewDefault!(ConstructorOptions, $key1 => $val1), $state_arg));
    ($cs:expr, $replacement:expr,
      $key1:ident=>$val1:expr,
      $key2:ident=>$val2:expr, $state_arg:ident
    ) => (DefConstructorWO!($cs, $replacement, NewDefault!(ConstructorOptions,
      $key1 => $val1,
      $key2 => $val2
    ), $state_arg));

    ($cs:expr, $replacement:expr,
      $key1:ident=>$val1:expr,
      $key2:ident=>$val2:expr,
      $key3:ident=>$val3:expr, $state_arg:ident
    ) => (DefConstructorWO!($cs, $replacement, NewDefault!(ConstructorOptions,
      $key1 => $val1,
      $key2 => $val2,
      $key3 => $val3
    ), $state_arg));

    ($cs:expr, $replacement:expr,
      $key1:ident=>$val1:expr,
      $key2:ident=>$val2:expr,
      $key3:ident=>$val3:expr,
      $key4:ident=>$val4:expr, $state_arg:ident
    ) => (DefConstructorWO!($cs, $replacement, NewDefault!(ConstructorOptions,
      $key1 => $val1,
      $key2 => $val2,
      $key3 => $val3,
      $key4 => $val4
    ), $state_arg));

    ($cs:expr, $replacement:expr,
      $key1:ident=>$val1:expr,
      $key2:ident=>$val2:expr,
      $key3:ident=>$val3:expr,
      $key4:ident=>$val4:expr,
      $key5:ident=>$val5:expr, $state_arg:ident
    ) => (DefConstructorWO!($cs, $replacement, NewDefault!(ConstructorOptions,
      $key1 => $val1,
      $key2 => $val2,
      $key3 => $val3,
      $key4 => $val4,
      $key5 => $val5
    ), $state_arg));

    // Closure replacement flavors:
    ($cs:expr, $document:ident, $args:ident, $props:ident, $inner_state:ident, $body:expr)=>(
        DefConstructor!($cs, $document, $args, $props, $inner_state, $body,
                        $state));
    ($cs:expr, $document:ident, $args:ident, $props:ident, $inner_state:ident, $body:expr,
      $key1:ident => $val1:expr)=>(
        DefConstructor!($cs, $document, $args, $props, $inner_state, $body,
                        $key1=>$val1, $state));
    ($cs:expr, $document:ident, $args:ident, $props:ident, $inner_state:ident, $body:expr,
      $key1:ident => $val1:expr,
      $key2:ident=>$val2:expr)=>(
        DefConstructor!($cs, $document, $args, $props, $inner_state, $body,
                        $key1=>$val1, $key2=>$val2, $state));
    ($cs:expr, $document:ident, $args:ident, $props:ident, $inner_state:ident, $body:expr,
      $key1:ident => $val1:expr,
      $key2:ident=>$val2:expr,
      $key3:ident=>$val3:expr)=>(
        DefConstructor!($cs, $document, $args, $props, $inner_state, $body,
                        $key1=>$val1, $key2=>$val2, $key3=>$val3, $state));
    ($cs:expr, $document:ident, $args:ident, $props:ident, $inner_state:ident, $body:expr,
      $key1:ident => $val1:expr,
      $key2:ident=>$val2:expr,
      $key3:ident=>$val3:expr,
      $key4:ident=>$val4:expr)=>(
        DefConstructor!($cs, $document, $args, $props, $inner_state, $body,
                        $key1=>$val1, $key2=>$val2, $key3=>$val3, $key4=>$val4, $state));
    ($cs:expr, $document:ident, $args:ident, $props:ident, $inner_state:ident, $body:expr,
      $key1:ident => $val1:expr,
      $key2:ident=>$val2:expr,
      $key3:ident=>$val3:expr,
      $key4:ident=>$val4:expr,
      $key5:ident=>$val5:expr)=>(
        DefConstructor!($cs, $document, $args, $props, $inner_state, $body,
                        $key1=>$val1, $key2=>$val2, $key3=>$val3, $key4=>$val4, $key5=>$val5, $state));
    // Closure replacement, explicit state
    ($cs:expr, $document:ident, $args:ident, $props:ident, $inner_state:ident, $body:expr, $state_arg:ident) => (
      DefConstructorWO!($cs, $document, $args, $props, $inner_state, $body, ConstructorOptions::default(), $state_arg)
    );
    ($cs:expr, $document:ident, $args:ident, $props:ident, $inner_state:ident, $body:expr, $key1:ident=>$val1:expr, $state_arg:ident) => (
      let replacement = replacement!($document, $args, $props, $body);
      DefConstructorWO!($cs, replacement, NewDefault!(ConstructorOptions, $key1 => $val1), $state_arg)
    );
    ($cs:expr, $document:ident, $args:ident, $props:ident, $inner_state:ident, $body:expr,
      $key1:ident=>$val1:expr,
      $key2:ident=>$val2:expr, $state_arg:ident
    ) => (
      DefConstructorWO!($cs, $document, $args, $props, $inner_state, $body, NewDefault!(ConstructorOptions,
        $key1 => $val1,
        $key2 => $val2),
      $state_arg));

    ($cs:expr, $document:ident, $args:ident, $props:ident, $inner_state:ident, $body:expr,
      $key1:ident=>$val1:expr,
      $key2:ident=>$val2:expr,
      $key3:ident=>$val3:expr, $state_arg:ident
    ) => (
      DefConstructorWO!($cs, $document, $args, $props, $inner_state, $body, NewDefault!(ConstructorOptions,
        $key1 => $val1,
        $key2 => $val2,
        $key3 => $val3
      ), $state_arg));

    ($cs:expr, $document:ident, $args:ident, $props:ident, $inner_state:ident, $body:expr,
      $key1:ident=>$val1:expr,
      $key2:ident=>$val2:expr,
      $key3:ident=>$val3:expr,
      $key4:ident=>$val4:expr, $state_arg:ident
    ) => (
      DefConstructorWO!($cs, $document, $args, $props, $inner_state, $body, NewDefault!(ConstructorOptions,
      $key1 => $val1,
      $key2 => $val2,
      $key3 => $val3,
      $key4 => $val4
    ), $state_arg));

    ($cs:expr, $document:ident, $args:ident, $props:ident, $inner_state:ident, $body:expr,
      $key1:ident=>$val1:expr,
      $key2:ident=>$val2:expr,
      $key3:ident=>$val3:expr,
      $key4:ident=>$val4:expr,
      $key5:ident=>$val5:expr, $state_arg:ident
    ) => (
      DefConstructorWO!($cs, $document, $args, $props, $inner_state, $body, NewDefault!(ConstructorOptions,
      $key1 => $val1,
      $key2 => $val2,
      $key3 => $val3,
      $key4 => $val4,
      $key5 => $val5
    ), $state_arg));
  );

  macro_rules! DefConstructorWO(
    ($proto:expr, $replacement:expr, $options:expr, $state_arg:ident) => ({
      // check_options("DefConstructor (prototype)", $constructor_options, %options);
      let (cs, paramlist) = try!(parse_prototype($proto, $state_arg));
      let compiled_replacement;
      compile_replacement!(compiled_replacement, $replacement);
      DefConstructorIWO!(cs, paramlist, compiled_replacement, $options, $state_arg);
    });
    ($proto:expr, $document:ident, $args:ident, $props:ident, $inner_state:ident, $body:expr, $options:expr, $state_arg:ident) => ({
      let compiled_replacement : Option<ReplacementClosure> = Some(Rc::new(replacement!($document, $args, $props, $inner_state, $body)));
      let (cs, paramlist) = try!(parse_prototype($proto, $state_arg));
      DefConstructorIWO!(cs, paramlist, compiled_replacement, $options, $state_arg);
    });
  );
  //=====================================================================
  // Define a LaTeX environment
  // Note that the body of the environment is treated is the 'body' parameter in the constructor.
  macro_rules! DefEnvironment(
    ($proto_raw:expr, $replacement:expr) => (DefEnvironment!($proto_raw, $replacement, $state));
    ($proto_raw:expr, $replacement:expr,
      $key1:ident=>$val1:expr) => (DefEnvironment!($proto_raw, $replacement, $key1=>$val1, $state));
    ($proto_raw:expr, $replacement:expr,
      $key1:ident=>$val1:expr,
      $key2:ident=>$val2:expr) => (DefEnvironment!($proto_raw, $replacement, $key1=>$val1, $key2=>$val2, $state));
    ($proto_raw:expr, $replacement:expr,
      $key1:ident=>$val1:expr,
      $key2:ident=>$val2:expr,
      $key3:ident=>$val3:expr) => (DefEnvironment!($proto_raw, $replacement, $key1=>$val1, $key2=>$val2, $key3=>$val3, $state));
    ($proto_raw:expr, $replacement:expr,
      $key1:ident=>$val1:expr,
      $key2:ident=>$val2:expr,
      $key3:ident=>$val3:expr,
      $key4:ident=>$val4:expr) => (DefEnvironment!($proto_raw, $replacement, $key1=>$val1, $key2=>$val2, $key3=>$val3, $key4=>$val4, $state));
    ($proto_raw:expr, $replacement:expr,
      $key1:ident=>$val1:expr,
      $key2:ident=>$val2:expr,
      $key3:ident=>$val3:expr,
      $key4:ident=>$val4:expr,
      $key5:ident=>$val5:expr) => (DefEnvironment!($proto_raw, $replacement, $key1=>$val1, $key2=>$val2, $key3=>$val3, $key4=>$val4, $key5=>$val5, $state));

    // with explicit state:
    ($proto_raw:expr, $replacement:expr, $state_arg:ident) => (DefEnvironmentWO!($proto_raw, $replacement, ConstructorOptions::default(), $state_arg));

    ($proto_raw:expr, $replacement:expr,
     $key1:ident => $val1:expr, $state_arg:ident
    ) => (DefEnvironmentWO!($proto_raw, $replacement, NewDefault!(ConstructorOptions,
     $key1 => $val1), $state_arg));

    ($proto_raw:expr, $replacement:expr,
     $key1:ident => $val1:expr,
     $key2:ident => $val2:expr, $state_arg:ident
    ) => (DefEnvironmentWO!($proto_raw, $replacement, NewDefault!(ConstructorOptions,
     $key1 => $val1,
     $key2 => $val2
    ), $state_arg));

    ($proto_raw:expr, $replacement:expr,
     $key1:ident => $val1:expr,
     $key2:ident => $val2:expr,
     $key3:ident => $val3:expr, $state_arg:ident
    ) => (DefEnvironmentWO!($proto_raw, $replacement, NewDefault!(ConstructorOptions,
     $key1 => $val1,
     $key2 => $val2,
     $key3 => $val3
    ), $state_arg));

    ($proto_raw:expr, $replacement:expr,
     $key1:ident => $val1:expr,
     $key2:ident => $val2:expr,
     $key3:ident => $val3:expr,
     $key4:ident => $val4:expr, $state_arg:ident
    ) => (DefEnvironmentWO!($proto_raw, $replacement, NewDefault!(ConstructorOptions,
     $key1 => $val1,
     $key2 => $val2,
     $key3 => $val3,
     $key4 => $val4
    ), $state_arg));

    ($proto_raw:expr, $replacement:expr,
     $key1:ident => $val1:expr,
     $key2:ident => $val2:expr,
     $key3:ident => $val3:expr,
     $key4:ident => $val4:expr,
     $key5:ident => $val5:expr, $state_arg:ident
    ) => (DefEnvironmentWO!($proto_raw, $replacement, NewDefault!(ConstructorOptions,
     $key1 => $val1,
     $key2 => $val2,
     $key3 => $val3,
     $key4 => $val4,
     $key5 => $val5
    ), $state_arg));
  );

  macro_rules! DefEnvironmentC(
    ($proto_raw:expr, $compiled_replacement:expr) => (DefEnvironmentC!($proto_raw, $compiled_replacement, $state));
    ($proto_raw:expr, $compiled_replacement:expr,
      $key1:ident=>$val1:expr) => (DefEnvironmentC!($proto_raw, $compiled_replacement, $key1=>$val1, $state));
    ($proto_raw:expr, $compiled_replacement:expr,
      $key1:ident=>$val1:expr,
      $key2:ident=>$val2:expr) => (DefEnvironmentC!($proto_raw, $compiled_replacement, $key1=>$val1, $key2=>$val2, $state));
    ($proto_raw:expr, $compiled_replacement:expr,
      $key1:ident=>$val1:expr,
      $key2:ident=>$val2:expr,
      $key3:ident=>$val3:expr) => (DefEnvironmentC!($proto_raw, $compiled_replacement, $key1=>$val1, $key2=>$val2, $key3=>$val3, $state));
    ($proto_raw:expr, $compiled_replacement:expr,
      $key1:ident=>$val1:expr,
      $key2:ident=>$val2:expr,
      $key3:ident=>$val3:expr,
      $key4:ident=>$val4:expr) => (DefEnvironmentC!($proto_raw, $compiled_replacement, $key1=>$val1, $key2=>$val2, $key3=>$val3, $key4=>$val4, $state));
    ($proto_raw:expr, $compiled_replacement:expr,
      $key1:ident=>$val1:expr,
      $key2:ident=>$val2:expr,
      $key3:ident=>$val3:expr,
      $key4:ident=>$val4:expr,
      $key5:ident=>$val5:expr) => (DefEnvironmentC!($proto_raw, $compiled_replacement, $key1=>$val1, $key2=>$val2, $key3=>$val3, $key4=>$val4, $key5=>$val5, $state));

    // with explicit state:
    ($proto_raw:expr, $compiled_replacement:expr, $state_arg:ident) => (DefEnvironmentCWO!($proto_raw, $paramlist, $compiled_replacement, ConstructorOptions::default()));
    ($proto_raw:expr, $compiled_replacement:expr,
      $key1:ident=>$val1:expr,
      $state_arg:ident
    ) => (DefEnvironmentCWO!($proto_raw, $compiled_replacement, NewDefault!(ConstructorOptions,
      $key1 => $val1
    ), $state_arg));

    ($proto_raw:expr, $compiled_replacement:expr,
      $key1:ident=>$val1:expr,
      $key2:ident=>$val2:expr,
      $state_arg:ident
    ) => (DefEnvironmentCWO!($proto_raw, $compiled_replacement, NewDefault!(ConstructorOptions,
      $key1 => $val1,
      $key2 => $val2
    ), $state_arg));

    ($proto_raw:expr, $compiled_replacement:expr,
      $key1:ident=>$val1:expr,
      $key2:ident=>$val2:expr,
      $key3:ident=>$val3:expr,
      $state_arg:ident
    ) => (DefEnvironmentCWO!($proto_raw, $compiled_replacement, NewDefault!(ConstructorOptions,
      $key1 => $val1,
      $key2 => $val2,
      $key3 => $val3
    ), $state_arg));

    ($proto_raw:expr, $compiled_replacement:expr,
      $key1:ident=>$val1:expr,
      $key2:ident=>$val2:expr,
      $key3:ident=>$val3:expr,
      $key4:ident=>$val4:expr,
      $state_arg:ident
    ) => (DefEnvironmentCWO!($proto_raw, $compiled_replacement, NewDefault!(ConstructorOptions,
      $key1 => $val1,
      $key2 => $val2,
      $key3 => $val3,
      $key4 => $val4
    ), $state_arg));

    ($proto_raw:expr, $compiled_replacement:expr,
      $key1:ident=>$val1:expr,
      $key2:ident=>$val2:expr,
      $key3:ident=>$val3:expr,
      $key4:ident=>$val4:expr,
      $key5:ident=>$val5:expr,
      $state_arg:ident
    ) => (DefEnvironmentCWO!($proto_raw, $compiled_replacement, NewDefault!(ConstructorOptions,
      $key1 => $val1,
      $key2 => $val2,
      $key3 => $val3,
      $key4 => $val4,
      $key5 => $val5
    ), $state_arg));
  );
  macro_rules! DefEnvironmentI{
    ($name_raw:expr, $paramlist:expr, $compiled_replacement:expr, $cc_copy:expr, $options:expr) =>
      (DefEnvironmentI!($name_raw, $paramlist, $compiled_replacement, $cc_copy, $options, $state));
    ($name_raw:expr, $paramlist:expr, $compiled_replacement:expr, $cc_copy:expr, $options:expr, $state_arg:ident) => ({
      use rtx_core::stomach::Stomach;
      use rtx_core::whatsit::Whatsit;
      use rtx_core::definition::constructor::Constructor;
      let name = $name_raw.to_string();
      let options = $options;
      let begin_name = "\\begin{".to_string()+&name+"}";
      let end_name = "\\end{".to_string()+&name+"}";
      // This is for the common case where the environment is opened by \begin{env}
      // let sizer = inferSizer($options.sizer, $options.reversion);
      let mut before_digest_env : Vec<BeforeDigestClosure> = Vec::new();
      match &options.mode {
        &Some(ref mode) => {
          let bmode = mode.clone();
          let mode_closure = Rc::new(move |stomach: &mut Stomach, state: &mut State| {
            try!(stomach.begin_mode(&bmode, state));
            Ok(Vec::new())
          });
          before_digest_env.push(mode_closure);
        },
        &None => {
          let bgroup_closure = beforeproc!(stomach, state, {stomach.bgroup(state);});
          before_digest_env.push(bgroup_closure);
        }
      };
      if options.require_math {
        let require_name = begin_name.clone();
        let require_math_closure = beforeproc!(stomach, state, { requireMath!(require_name, state) });
        before_digest_env.push(require_math_closure);
      }
      if options.forbid_math {
        let forbid_name = begin_name.clone();
        let forbid_math_closure = beforeproc!(stomach, state, { forbidMath!(forbid_name, state) });
        before_digest_env.push(forbid_math_closure);
      }

      let env_name = name.clone();
      let current_environment_closure = beforeproc!(stomach, state, {
        AssignValue!("current_environment", ObjectStore::String(env_name.clone()), None, state);
        let body = T_LETTER!(env_name.clone());
        DefMacroT!(T_CS!("\\@currenvir"), None, body.clone(), state);
      });
      before_digest_env.push(current_environment_closure);

      if let Some(chosen_font) = options.font {
        let merge_font_closure = beforeproc!(stomach, state, {
          MergeFont!(chosen_font.clone(), state);
        });
        before_digest_env.push(merge_font_closure);
      }
      before_digest_env.extend(options.before_digest);

      let push_frame_closure = Rc::new(|_document: &mut Document, _whatsit: &Whatsit, state: &mut State| {
        state.push_frame();
      });
      let mut before_construct_with_frame : Vec<ConstructionClosure> = vec![push_frame_closure];
      before_construct_with_frame.extend(options.before_construct);

      let mut after_construct_with_frame : Vec<ConstructionClosure> = options.after_construct;

      let pop_frame_closure = Rc::new(|_document: &mut Document, _whatsit: &Whatsit, state: &mut State| {
        state.pop_frame();
      });
      after_construct_with_frame.push(pop_frame_closure);

      let begin_name_constructor = Rc::new(Constructor {
          cs: T_CS!(begin_name),
          paramlist: $paramlist,
          replacement: $compiled_replacement,
          options: ConstructorOptions {
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
            ..ConstructorOptions::default()
          }});
      $state_arg.install_definition(ObjectStore::Constructor(begin_name_constructor), options.scope.clone());


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
            try!(stomach.end_mode(&emode, state));
            Ok(Vec::new())
          });
          after_digest_env.push(emode_closure);
        },
        None => {
          let egroup_closure = Rc::new(|stomach: &mut Stomach, _whatsit: &mut Whatsit, state: &mut State| {
            try!(stomach.egroup(state));
            Ok(Vec::new())
          });
          after_digest_env.push(egroup_closure);
        }
      };

      let end_envname_constructor = Rc::new(Constructor {
        cs: T_CS!(end_name),
        replacement: None,
        paramlist: None,
        options: ConstructorOptions {
          before_digest: options.before_digest_end,
          after_digest: after_digest_env,
          ..ConstructorOptions::default()
        }
      });
      $state_arg.install_definition(ObjectStore::Constructor(end_envname_constructor), options.scope.clone());

      // For the uncommon case opened by \csname env\endcsname
      let name_constructor = Rc::new(Constructor{
        cs: T_CS!("\\".to_string() +&name),
        paramlist: $paramlist,
        replacement: $cc_copy,
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
        options: ConstructorOptions {
          nargs: options.nargs,
          capture_body: true,
          properties: options.properties.clone(),
          // (defined $options{reversion} ? (reversion => $options{reversion}) : ()),
          // (defined $sizer ? (sizer => $sizer) : ()),
          // ), $options{scope});
          ..ConstructorOptions::default()
        }
      });
      $state_arg.install_definition(ObjectStore::Constructor(name_constructor), options.scope.clone());

      let end_name_constructor = Rc::new(Constructor {
        cs: T_CS!("\\end".to_string() + &name),
        paramlist: None,
        replacement: Some(Rc::new(|document, whatsit, properties, state|{
          let env = state.lookup_value("current_environment");
          // Error('unexpected', "\\end{$name}", $_[0],
          //   "Can't close environment $name",
          //   "Current are "
          //     . join(', ', state->lookupStackedValues('current_environment')))
          //   unless $env && $name eq $env;
          Ok(()) })),
        // beforeDigest => flatten($options{beforeDigestEnd}),
        // afterDigest  => flatten($options{afterDigest},
        //   ($mode ? (sub { $_[0]->endMode($mode); }) : ())),
        // ), $options{scope});
        options: ConstructorOptions::default()
      });
      $state_arg.install_definition(ObjectStore::Constructor(end_name_constructor), options.scope);

      if options.locked {
        AssignValue!(&("\\begin{".to_string() + &name+"}:locked"), ObjectStore::Bool(true), None, $state_arg);
        AssignValue!(&("\\end{".to_string()+&name+"}:locked")  , ObjectStore::Bool(true), None, $state_arg);
        AssignValue!(&("\\".to_string()+&name+":locked")       , ObjectStore::Bool(true), None, $state_arg);
        AssignValue!(&("\\end".to_string()+&name+":locked")    , ObjectStore::Bool(true), None, $state_arg);
      }
    })
  }

  macro_rules! Tag {
    ($tag:expr, $key1:ident => $val1:expr)=>(Tag!($tag, $key1=>$val1, $state));
    ($tag:expr, $key1:ident => $val1:expr,
      $key2:ident=>$val2:expr)=>(Tag!($tag, $key1=>$val1, $key2=>$val2, $state));
    ($tag:expr, $key1:ident => $val1:expr,
      $key2:ident=>$val2:expr,
      $key3:ident=>$val3:expr)=>(Tag!($tag, $key1=>$val1, $key2=>$val2, $key3=>$val3, $state));
    ($tag:expr, $key1:ident => $val1:expr,
      $key2:ident=>$val2:expr,
      $key3:ident=>$val3:expr,
      $key4:ident=>$val4:expr)=>(Tag!($tag, $key1=>$val1, $key2=>$val2, $key3=>$val3, $key4=>$val4, $state));
    ($tag:expr, $key1:ident => $val1:expr,
      $key2:ident=>$val2:expr,
      $key3:ident=>$val3:expr,
      $key4:ident=>$val4:expr,
      $key5:ident=>$val5:expr)=>(Tag!($tag, $key1=>$val1, $key2=>$val2, $key3=>$val3, $key4=>$val4, $key5=>$val5, $state));

    // with explicit state:
    ($tag:expr,
     $key1:ident => $val1:expr,
     $state_arg:ident
    ) => (TagWO!($tag, NewDefault!(TagOptions,
     $key1 => $val1), $state_arg));

    ($tag:expr,
     $key1:ident => $val1:expr,
     $key2:ident => $val2:expr,
     $state_arg:ident
    ) => (TagWO!($tag, NewDefault!(TagOptions,
     $key1 => $val1,
     $key2 => $val2
    ), $state_arg));

    ($tag:expr,
     $key1:ident => $val1:expr,
     $key2:ident => $val2:expr,
     $key3:ident => $val3:expr,
     $state_arg:ident
    ) => (TagWO!($tag, NewDefault!(TagOptions,
     $key1 => $val1,
     $key2 => $val2,
     $key3 => $val3
    ), $state_arg));

    ($tag:expr,
     $key1:ident => $val1:expr,
     $key2:ident => $val2:expr,
     $key3:ident => $val3:expr,
     $key4:ident => $val4:expr,
     $state_arg:ident
    ) => (TagWO!($tag, NewDefault!(TagOptions,
     $key1 => $val1,
     $key2 => $val2,
     $key3 => $val3,
     $key4 => $val4
    ),$state_arg));

    ($tag:expr,
     $key1:ident => $val1:expr,
     $key2:ident => $val2:expr,
     $key3:ident => $val3:expr,
     $key4:ident => $val4:expr,
     $key5:ident => $val5:expr,
     $state_arg:ident
    ) => (TagWO!($tag, NewDefault!(TagOptions,
     $key1 => $val1,
     $key2 => $val2,
     $key3 => $val3,
     $key4 => $val4,
     $key5 => $val5
    ),$state_arg));
  }

  macro_rules! TagWO {
    ($tag:expr, $properties:expr, $state_arg:ident) => (install_tag($tag, $properties, $state_arg))
  }
  // sub DocType {
  //   my ($rootelement, $pubid, $sysid, %namespaces) = @_;
  //   let model = state->getModel;
  //   $model->setDocType($rootelement, $pubid, $sysid);
  //   foreach let prefix (keys %namespaces) {
  //     $model->registerDocumentNamespace($prefix => $namespaces{$prefix}); }
  //   return; }


  macro_rules! DefEnvironmentWO (
    ($proto_raw:expr, $replacement:expr, $options:expr, $state_arg:ident) => ({
    use rtx_core::util::text::*;
    let mut proto = $proto_raw.to_string().trim_left().to_string();
    let name = extract_bracketed(&mut proto, Some(&Delimiter::Brace));

    let compiled_replacement;
    compile_replacement!(compiled_replacement, $replacement);
    let cc_copy;
    compile_replacement!(cc_copy, $replacement);

    let options = $options;

    DefEnvironmentI!(name, None, compiled_replacement, cc_copy, options, $state_arg);
  }));

  macro_rules! DefEnvironmentCWO (
    ($proto_raw:expr, $compiled_replacement:expr, $options:expr, $state_arg:ident) => ({
    use rtx_core::util::text::*;
    let mut proto = $proto_raw.to_string().trim_left().to_string();
    let name = extract_bracketed(&mut proto, Some(&Delimiter::Brace));
    // TODO: What do we do with param lists?
    //let paramlist_str = proto.trim_left().to_string();
    DefEnvironmentI!(name, None, $compiled_replacement, $compiled_replacement, $options, $state_arg);
  }));


  macro_rules! RelaxNGSchema{
    ($name:expr) => (RelaxNGSchema!($name, $state));
    ($name:expr,$state_arg:ident) => (select_relaxng_schema($name.to_string(), None, $state_arg))
  }
  macro_rules! RegisterNamespace(
    ($prefix:expr, $namespace:expr) => (RegisterNamespace!($prefix, $namespace, $state));
    ($prefix:expr, $namespace:expr,$state_arg:ident) => ($state_arg.model.register_namespace($prefix, Some($namespace.to_string())))
  );
  macro_rules! RegisterDocumentNamespace(
    ($prefix:expr, $namespace:expr) => (RegisterDocumentNamespace!($prefix, $namespace, $state));
    ($prefix:expr, $namespace:expr,$state_arg:ident) => ($state_arg.model.register_document_namespace($prefix, Some($namespace.to_string())))
  );
  macro_rules! RequireResource(
    ($resource:expr) => (RequireResource!($resource, $state));
    ($resource:expr,$state_arg:ident) => (require_resource(Resource{name: $resource.to_string(), ..Resource::default()}, $state_arg))
  );

  // sub DefMath {
  //   my ($proto,
  //     $presentation, %options) = @_;
  //   CheckOptions("DefMath ($proto)", $math_options, %options);
  //   DefMathI(parsePrototype($proto), $presentation, %options);
  //   return; }
  macro_rules! DefMathI(
    ($text:expr,$paramlist:expr,$presentation:expr,
      $key1:ident => $val1:expr)=>(DefMathI!($text, $paramlist, $presentation, $key1=>$val1, $state));
    ($text:expr,$paramlist:expr,$presentation:expr,
      $key1:ident => $val1:expr,
      $key2:ident => $val2:expr)=>(DefMathI!($text, $paramlist, $presentation, $key1=>$val1, $key2=>$val2, $state));
    ($text:expr,$paramlist:expr,$presentation:expr,
      $key1:ident => $val1:expr,
      $key2:ident => $val2:expr,
      $key3:ident => $val3:expr)=>(DefMathI!($text, $paramlist, $presentation, $key1=>$val1, $key2=>$val2, $key3=>$val3, $state));
    ($text:expr,$paramlist:expr,$presentation:expr,
      $key1:ident => $val1:expr,
      $key2:ident => $val2:expr,
      $key3:ident => $val3:expr,
      $key4:ident => $val4:expr)=>(DefMathI!($text, $paramlist, $presentation, $key1=>$val1, $key2=>$val2, $key3=>$val3, $key4=>$val4, $state));
    ($text:expr,$paramlist:expr,$presentation:expr,
      $key1:ident => $val1:expr,
      $key2:ident => $val2:expr,
      $key3:ident => $val3:expr,
      $key4:ident => $val4:expr,
      $key5:ident => $val5:expr)=>(DefMathI!($text, $paramlist, $presentation, $key1=>$val1, $key2=>$val2, $key3=>$val3, $key4=>$val4, $key5=>$val5, $state));

    // with explicit state:
    ($text:expr,$paramlist:expr,$presentation:expr,
     $key1:ident => $val1:expr,
     $state_arg:ident
    ) => (DefMathWO!($text,$paramlist, $presentation, NewDefault!(MathPrimitiveOptions,
     $key1 => $val1),$state_arg));

    ($text:expr,$paramlist:expr,$presentation:expr,
     $key1:ident => $val1:expr,
     $key2:ident => $val2:expr,
     $state_arg:ident
    ) => (DefMathWO!($text,$paramlist, $presentation, NewDefault!(MathPrimitiveOptions,
     $key1 => $val1,
     $key2 => $val2
    ), $state_arg));

    ($text:expr,$paramlist:expr,$presentation:expr,
     $key1:ident => $val1:expr,
     $key2:ident => $val2:expr,
     $key3:ident => $val3:expr,
     $state_arg:ident
    ) => (DefMathWO!($text,$paramlist, $presentation, NewDefault!(MathPrimitiveOptions,
     $key1 => $val1,
     $key2 => $val2,
     $key3 => $val3
    ),$state_arg));

    ($text:expr,$paramlist:expr,$presentation:expr,
     $key1:ident => $val1:expr,
     $key2:ident => $val2:expr,
     $key3:ident => $val3:expr,
     $key4:ident => $val4:expr,
     $state_arg:ident
    ) => (DefMathWO!($text,$paramlist, $presentation, NewDefault!(MathPrimitiveOptions,
     $key1 => $val1,
     $key2 => $val2,
     $key3 => $val3,
     $key4 => $val4
    ), $state_arg));

    ($text:expr,$paramlist:expr,$presentation:expr,
     $key1:ident => $val1:expr,
     $key2:ident => $val2:expr,
     $key3:ident => $val3:expr,
     $key4:ident => $val4:expr,
     $key5:ident => $val5:expr,
     $state_arg:ident
    ) => (DefMathWO!($text,$paramlist, $presentation, NewDefault!(MathPrimitiveOptions,
     $key1 => $val1,
     $key2 => $val2,
     $key3 => $val3,
     $key4 => $val4,
     $key5 => $val5
    ), $state_arg));
  );

  macro_rules! DefMathWO {
    ($cstext:expr, $paramlist:expr, $presentation:expr, $options:expr, $state_arg:ident) => ({
      let mut options = $options;
      let cs = T_CS!($cstext.to_string());
      let presentation = $presentation.to_string();
      // Can't defer parsing parameters since we need to know number of args!
      // $paramlist = parseParameters($paramlist, $cs) if defined $paramlist && !ref $paramlist;
      let paramlist : Option<Parameters> = $paramlist;
      let nargs = match paramlist {
        Some(plist) => plist.get_num_args(),
        None => 0
      };
      let csname = cs.get_string().to_string();
      let mut name = options.alias.clone().unwrap_or_else(|| csname.clone());
      if name.starts_with('\\') {
        name = name.replacen('\\', "", 1)
      }
      if let Some(options_name) = options.name {
        name = options_name;
      }
      let name_opt = if (name == presentation) || (name.is_empty()) || (options.meaning == Some(name.clone())) {
        None
      } else {
        Some(name)
      };
      options.name = name_opt;
      if nargs == 0 && options.role.is_none() {
        options.role = Some("UNKNOWN".to_string())
      }
      if nargs > 0 && options.operator_role.is_none() {
        options.operator_role = Some("UNKNOWN".to_string())
      }

      // Store some data for introspection
      // defmath_introspective(cs, $paramlist, presentation, %options);

      // If single character, handle with a rewrite rule
      if csname.len() == 1 {
        // WAS: defmath_rewrite!($cs, options);
        // No, do NOT make mathactive; screws up things like babel french, or... ?
        // EXPERIMENT: store XMTok attributes for if this char ends up a Math Token.
        // But only some DefMath options make sense!
        // let rw_options = { name => 1, meaning => 1, omcd => 1, role => 1, mathstyle => 1, stretchy => 1 }; # (well, mathstyle?)
        // CheckOptions("DefMath reimplemented as DefRewrite ($csname)", $rw_options, %options);
        let mut math_attr_hash : HashMap<String, String> = HashMap::new();
        transfer_opt_default!(name, options, math_attr_hash);
        transfer_opt_default!(meaning, options, math_attr_hash);
        transfer_opt_default!(omcd, options, math_attr_hash);
        transfer_opt_default!(role, options, math_attr_hash);
        transfer_opt_default!(mathstyle, options, math_attr_hash);
        transfer_default!(stretchy, options, math_attr_hash);

        $state_arg.assign_value(&format!("math_token_attributes_{}",csname), ObjectStore::HashStr(math_attr_hash), Some(Scope::Global));
      }
      // TODO:
      // // If the presentation is complex, and involves arguments,
      // // we will create an XMDual to separate content & presentation.
      // elsif ((ref presentation eq "CODE")
      //   || ((ref presentation) && grep { $_->equals(T_PARAM) } presentation->unlist)
      //   || (!(ref presentation) && (presentation =~ /\//\d|\\./))
      //   || ((ref presentation) && (grep { $_->isExecutable } presentation->unlist))) {
      //   defmath_dual($cs, $paramlist, presentation, %options); }

      // EXPERIMENT: Introduce an intermediate case for simple symbols
      // Define a primitive that will create a Box with the appropriate set of XMTok attributes.
      if nargs == 0 {// && !grep { !$$simpletoken_options{$_} } keys %options) {
        defmath_prim!(cs, paramlist, $presentation.to_string(), options, $state_arg);
      }

      // else {
      //   defmath_cons($cs, $paramlist, $presentation, %options); }
      // AssignValue($csname . ":locked" => 1) if $options{locked};
    })
  }

  macro_rules! defmath_prim {
    ($cs:expr, $_paramlist:expr, $presentation:expr, $options:expr, $state_arg:ident) => ({
    let mut prim_options = $options;
    prim_options.locked = false;
    prim_options.font = None;
    let scope = prim_options.scope.clone();
    let reqfont = prim_options.font.clone();
    $state_arg.install_definition(ObjectStore::MathPrimitive(Rc::new(MathPrimitive{
      cs: $cs.clone(),
      paramlist: None, // never any parameters, this is intentional
      replacement: Some(Rc::new(move |stomach, args, state| {
        // let locator    = $stomach->getGullet->getLocator;
        let mut properties = HashMap::new(); // TODO: sync with perl master here
        properties.insert("mode".to_owned(), "math".to_owned());
        let font       = state.lookup_font().unwrap().merge(reqfont.clone().unwrap()).specialize(&$presentation);
        // foreach my $key (keys %properties) {
        //   my $value = $properties{$key};
        //   if (ref $value eq 'CODE') {
        //     $properties{$key} = &$value(); } }
        info!("defmath_prim: {}, tokens: {:?}", &$presentation, $cs);
        Ok(vec![Digested::Box( // TODO: Can we reduce boilerplate?
          Tbox{ text: $presentation, tokens: Tokens!($cs.clone()), font: font, properties: properties, ..Tbox::default()}
        )])
      })),
      options: prim_options,
      ..MathPrimitive::default()
      })), scope);
    })
  }

  #[macro_export]
  macro_rules! requireMath {
    ($cs_name:expr, $state_arg:ident) => (
      if !LookupBool!("IN_MATH", $state_arg) {
        warn!(target: "unexpected", "{} should only appear in math mode",$cs_name);
      }
    )
  }
  #[macro_export]
  macro_rules! forbidMath {
    ($cs_name:expr) => (forbidMath!($cs_name, $state));
    ($cs_name:expr, $state_arg:ident) => (
      if LookupBool!("IN_MATH", $state_arg) {
        warn!(target: "unexpected", "{} should not appear in math mode",$cs_name);
      }
    )
  }

)}

pub mod pool;
