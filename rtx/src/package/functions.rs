use libxml::tree::Node;
use regex::Regex;
use std::collections::HashMap;
use std::rc::Rc;

use rtx_core::common::error::*;
use rtx_core::common::font::Font;
use rtx_core::common::number::Number;
use rtx_core::definition::conditional::{Conditional, ConditionalOptions, ConditionalType};
use rtx_core::definition::expandable::Expandable;
use rtx_core::definition::{ConditionalClosure, Definition, ExpansionClosure};
use rtx_core::document::resource::*;
use rtx_core::document::tag::{TagOptionName, TagOptions};
use rtx_core::document::Document;
use rtx_core::gullet::Gullet;
use rtx_core::mouth;
use rtx_core::mouth::Mouth;
use rtx_core::parameter::{Parameter, Parameters};
use rtx_core::state::{ObjectStore, Scope, State};
use rtx_core::stomach::Stomach;
use rtx_core::token::Token;
use rtx_core::tokens::Tokens;
use rtx_core::util::pathname;
use rtx_core::BoxOps;
use rtx_core::{Core, Digested};

use super::pool;

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
      ObjectStore::Expandable(ref m) => m.get_cs_name() != "\\relax",
      ObjectStore::Primitive(ref m) => m.get_cs_name() != "\\relax",
      ObjectStore::Constructor(ref m) => m.get_cs_name() != "\\relax",
      _ => false,
    },
    _ => false,
  }
}

/// TODO: Flesh out with the full infrastructure, incremental functionality for now.
pub fn input_definitions(
  raw_file: String,
  options: InputDefinitionOptions,
  mut state: &mut State,
) -> Result<()>
{
  let mut file: String = raw_file.trim().to_string();

  // let prevname = if options.handleoptions {
  //   match state.lookup_definition(T_CS!("\@currname")) {
  //     Some(ObjectStore::Expandable(name)) => Digest!(T_CS!("\@currname")).to_string()
  // }
  // let prevext = options.handleoptions && $state->lookupDefinition(T_CS!('\@currext')) &&
  // ToString(Digest(T_CS!('\@currext')));

  // Compute the exact name based on the type
  file = match options.extension {
    None => file,
    Some(ext) => file + "." + &ext,
  };

  let loaded_flag = file.clone() + "_loaded";
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
  note_begin(&s!("Loading {:?} definitions...", file));
  state.assign_value(&loaded_flag, ObjectStore::Bool(true), Some(Scope::Global));

  match file.as_ref() {
    "TeX.pool" => pool::tex::load_definitions(&mut state)?,
    "LaTeX.pool" => pool::latex::load_definitions(&mut state)?,
    "eTeX.pool" => pool::etex::load_definitions(&mut state)?,
    "pdfTeX.pool" => pool::pdftex::load_definitions(&mut state)?,
    "article.cls" => pool::article_cls::load_definitions(&mut state)?,
    "alltt.sty" => pool::alltt_sty::load_definitions(&mut state)?,
    "comment.sty" => pool::comment_sty::load_definitions(&mut state)?,
    other => fatal!(
      Package,
      Unknown,
      s!("TODO: unknown binding {:?}, can't load", other)
    ),
  };
  note_end(&s!("Loading {:?} definitions...", file));
  Ok(())
}

pub fn input_content(core: &mut Core, request: &str) -> Result<()> {
  match find_file(request, false) {
    // TODO: type => $options{type}, noltxml => 1
    Some(path) => load_tex_content(core, &path),
    None => fatal!(Package, MissingFile, request),
    /* TODO:
     * Error("missing_file", request, state.get_stomach().get_gullet(),
     * "Can't find TeX file "+request, maybeReportSearchPaths(state))) */
  }
}

pub fn load_tex_content(core: &mut Core, path: &str) -> Result<()> {
  let mut mouth = Mouth {
    notes: true,
    ..Mouth::default()
  };
  mouth.open(path, &mut core.state)?;
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
  pub extension: Option<String>,
  pub as_class: bool,
  pub noltxml: bool,
  pub notex: bool,
  pub raw: bool,
  pub after: bool,
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
      after: false,
    }
  }
}

/// This (and `FindFile`) needs to evolve a bit to support reading raw .sty (.def, etc) files from
/// the standard texmf directories.  Maybe even use kpsewhich itself (INSTEAD of `pathname_find`
/// ???) Another potentially useful option might be that if we are reading a raw file,
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
    options.extension = Some(String::from("sty"));
  }
  // TODO: Ideally we want to use the same struct for the RequirePackage options as for the
  // InputDefinitions options
  input_definitions(
    name,
    InputDefinitionOptions {
      extension: options.extension,
      handleoptions: true,
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

pub fn load_class(
  name: String,
  options: Vec<String>,
  after: Tokens,
  state: &mut State,
) -> Result<()>
{
  input_definitions(
    name,
    InputDefinitionOptions {
      extension: Some(String::from("cls")),
      after: after,
      notex: true,
      handleoptions: true,
      noerror: true,
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

pub fn find_file(request: &str, _forbid_ltxml: bool) -> Option<String> {
  // TODO: Actually find it!
  Some(request.to_string())
}

pub fn coerce_cs(t: &str) -> Token { T_CS!(t) }

lazy_static! {
  static ref CSNAME_MACRO_REGEX: Regex = Regex::new(r"^\\csname\s+(.*)\\endcsname").unwrap();
  static ref CS_REGEX: Regex = Regex::new(r"^(\\[a-zA-Z@]+)").unwrap();
  static ref SINGLE_CHAR_REGEX: Regex = Regex::new(r"^(\\.)").unwrap();
  static ref ACTIVE_CHAR_REGEX: Regex = Regex::new(r"^(.)").unwrap();
  static ref CONDITIONAL_REGEX: Regex = Regex::new(r"^\\(?:if(.*)|unless)$").unwrap();
}

pub fn parse_prototype(proto: &str, state: &mut State) -> Result<((Token, Option<Parameters>))> {
  let mut cs = T_CS!(s!("\\")); // Should never happen
  let mut final_proto = if CSNAME_MACRO_REGEX.is_match(proto) {
    let captures = CSNAME_MACRO_REGEX.captures(proto).unwrap();
    cs = T_CS!(s!("\\") + captures.get(0).map_or("", |m| m.as_str()));
    // also replace in proto
    CSNAME_MACRO_REGEX.replace(proto, "").to_string()
  } else if CS_REGEX.is_match(proto) {
    // Match a cs
    let captures = CS_REGEX.captures(proto).unwrap();
    let csname = captures.get(0).map_or("", |m| m.as_str()).to_string();
    cs = T_CS!(csname);
    // also replace in proto
    CS_REGEX.replace(proto, "").to_string()
  } else if SINGLE_CHAR_REGEX.is_match(proto) {
    // Match a single char cs, env name,...
    let captures = SINGLE_CHAR_REGEX.captures(proto).unwrap();
    cs = T_CS!(captures.get(0).map_or("", |m| m.as_str()).to_string());
    // also replace in proto
    SINGLE_CHAR_REGEX.replace(proto, "").to_string()
  } else if ACTIVE_CHAR_REGEX.is_match(proto) {
    // Match an active char
    let captures = ACTIVE_CHAR_REGEX.captures(proto).unwrap();
    cs = TokenizeInternal!(captures.get(0).map_or("", |m| m.as_str()))
      .unlist()
      .first()
      .unwrap()
      .clone();
    // also replace in proto
    ACTIVE_CHAR_REGEX.replace(proto, "").to_string()
  } else {
    // Fatal('misdefined', prototype, $STATE->getStomach,
    //   "Definition prototype doesn't have proper control sequence: \"prototype\""); }
    proto.to_string()
  };
  final_proto = final_proto.trim_left().to_string();
  let paramlist = parse_parameters(final_proto, &cs, state)?;
  Ok((cs, paramlist))
}

lazy_static! {
  static ref NESTED_CHECK: Regex = Regex::new(r"^(\{([^\}]*)\})\s*").unwrap();
  static ref OPTIONAL_CHECK: Regex = Regex::new(r"^(\[([^\]]*)\])\s*").unwrap();
  static ref DEFAULT_CHECK: Regex = Regex::new(r"^Default:(.*)$").unwrap();
  static ref PARAMSPECT_CHECK: Regex = Regex::new(r"^((\w*)(:([^\s\{\[]*))?)\s*").unwrap();
}
pub fn parse_parameters(
  mut prototype: String,
  cs: &Token,
  state: &mut State,
) -> Result<Option<Parameters>>
{
  let mut parameters = Vec::new();
  while !prototype.is_empty() {
    let mut next_proto: String;
    // Handle possibly nested cases, such as {Number}
    if NESTED_CHECK.is_match(&prototype) {
      let captures = NESTED_CHECK.captures(&prototype).unwrap();
      next_proto = NESTED_CHECK.replace(&prototype, "").to_string();
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
          extra: vec![inner],
          ..Parameter::default()
        }.init(state)?,
      );
    } else if OPTIONAL_CHECK.is_match(&prototype) {
      // Ditto for Optional
      let captures = OPTIONAL_CHECK.captures(&prototype).unwrap();
      next_proto = OPTIONAL_CHECK.replace(&prototype, "").to_string();
      let spec = captures.get(1).map_or("", |m| m.as_str());
      let inner_spec = captures.get(2).map_or("", |m| m.as_str());

      if DEFAULT_CHECK.is_match(inner_spec) {
        // let default_captures = DEFAULT_CHECK.captures(&inner_spec).unwrap();
        parameters.push(
          Parameter {
            name: s!("Optional"),
            spec: spec.to_string(),
            // extra: vec![TokenizeInternal(default_captures.get(0).map_or("", |m| m.as_str())),
            // None]});
            extra: Vec::new(),
            ..Parameter::default()
          }.init(state)?,
        );
      } else if !inner_spec.is_empty() {
        parameters.push(
          Parameter {
            name: s!("Optional"),
            spec: spec.to_string(),
            extra: vec![None, parse_parameters(inner_spec.to_string(), cs, state)?],
            ..Parameter::default()
          }.init(state)?,
        );
      } else {
        parameters.push(
          Parameter {
            name: s!("Optional"),
            spec: spec.to_string(),
            extra: Vec::new(),
            ..Parameter::default()
          }.init(state)?,
        );
      }
    } else if PARAMSPECT_CHECK.is_match(&prototype) {
      let captures = PARAMSPECT_CHECK.captures(&prototype).unwrap();
      next_proto = PARAMSPECT_CHECK.replace(&prototype, "").to_string();
      let spec = captures.get(1).map_or("", |m| m.as_str());
      let spec_type = captures.get(2).map_or("", |m| m.as_str());
      let extra = match captures.get(4) {
        None => Vec::new(),
        Some(_extra_string) => {
          // TODO: Ask Bruce about the "extra" functionality and its types
          // extra_string.split("|").map(|t| tokenize_internal(t)).collect::<Vec<Token>>();
          Vec::new()
        },
      };
      parameters.push(
        Parameter {
          name: spec_type.to_string(),
          spec: spec.to_string(),
          extra: extra,
          ..Parameter::default()
        }.init(state)?,
      );
    } else {
      // Fatal('misdefined', cs, undef, "Unrecognized parameter specification at \"prototype\""); }
      panic!(
        "Fatal:misdefined:{:?} Unrecognized parameter specification at \"prototype\"",
        cs
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

pub fn revert(_arg: &[Token]) -> Tokens { Tokens!() }

//======================================================================
// Declaring and Adjusting the Document Model.
//======================================================================

pub fn install_tag(tag: &str, mut properties: TagOptions, state: &mut State) {
  let mut options = state
    .tag_properties
    .entry(tag.to_string())
    .or_insert_with(TagOptions::default);
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

pub struct InputDefinitionOptions {
  pub extension: Option<String>,
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
      as_class: false,
    }
  }
}

// Selects the RelaxNG schema defining the XML output language
pub fn select_relaxng_schema(
  schema: String,
  namespaces: Option<HashMap<String, String>>,
  state: &mut State,
)
{
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

pub fn def_macro(
  cs: Token,
  paramlist: Option<Parameters>,
  expansion: Option<ExpansionClosure>,
  state: &mut State,
)
{
  //       // Optimization: Defer till macro actually used
  //       // if !$cs.is_empty() { // && $options{mathactive}
  //         // $state.assign_mathcode($cs, 0x8000, $options{scope}); }
  // $state.install_definition(Expandable{ cs: coerce_cs( $cs ), paramlist: $paramlist,
  // expansion: $expansion});//, %options), $options{scope});       // if $options{locked} {
  //       //   $state.assign_value(ToString($cs)+":locked", true, "global")
  //       // }

  state.install_definition(
    ObjectStore::Expandable(Rc::new(Expandable {
      cs: cs,
      paramlist: paramlist,
      expansion: expansion,
      ..Expandable::default()
    })),
    None,
  );
}

pub struct RegisterOptions {
  pub getter: bool,
  pub setter: bool,
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

pub fn def_conditional(
  cs: Token,
  paramlist: Option<Parameters>,
  test: Option<ConditionalClosure>,
  options: ConditionalOptions,
  state: &mut State,
)
{
  let cs_name = cs.get_cs_name();
  match cs_name.as_str() {
    "\\fi" | "\\else" | "\\or" => state.install_definition(
      ObjectStore::Conditional(Rc::new(Conditional {
        cs: cs.clone(),
        paramlist: None,
        test: None,
        conditional_type: Some(ConditionalType::from(&cs_name)),
        locked: options.locked,
        skipper: options.skipper,
      })),
      options.scope,
    ),
    custom => {
      if CONDITIONAL_REGEX.is_match(custom) {
        let captures = CONDITIONAL_REGEX.captures(custom).unwrap();
        let name = captures.get(0).map_or("", |m| m.as_str()).to_string();
        if !name.is_empty() && name != "case" && test.is_none() {
          // user-defined conditional, like with \newif
          // Note: setting up these macros is compile-time expensive, maybe there is some way to
          // avoid...
          SetupBindingMacros!(state);
          // Note: the double clones are technically correct Rust if annoying to write and read.
          //       first, we want to capture a cloned value of cs, to be able to keep using cs here.
          // second, each invocation of the conditional macro needs to create new tokens to
          // return,       hence a clone is required on each call.
          let cs_c1 = cs.clone();
          DefMacroTS!(
            T_CS!(s!("\\{}true", name)),
            None,
            Tokens!(T_CS!("\\let"), cs_c1.clone(), T_CS!("\\iftrue")),
            state
          );
          let cs_c2 = cs.clone();
          DefMacroTS!(
            T_CS!(s!("\\{}false", name)),
            None,
            Tokens!(T_CS!("\\let"), cs_c2.clone(), T_CS!("\\iffalse")),
            state
          );
          state.let_i(&cs, T_CS!("\\iffalse"), None);
        } else {
          //  For \ifcase, the parameter list better be a single Number !!
          state.install_definition(
            ObjectStore::Conditional(Rc::new(Conditional {
              cs: cs.clone(),
              paramlist,
              test,
              conditional_type: Some(ConditionalType::If),
              locked: options.locked,
              skipper: options.skipper,
            })),
            options.scope,
          );
        }
      } else {
        error!(
          target: &s!("misdefined:{}", cs),
          "The conditional {} is being defined but doesn't start with \\if",
          cs
        );
      }
    },
  }

  if let Some(true) = options.locked {
    state.assign_value(&s!("{}:locked", cs), ObjectStore::Bool(true), None);
  }
  return;
}

pub fn def_register(
  cs: Token,
  paramlist: Option<Parameters>,
  value_opt: Option<Number>,
  options: Option<RegisterOptions>,
  state: &mut State,
)
{
  // TODO:
  //   my $type   = $register_types{ ref $value };
  //   my $name   = ToString($cs);
  //   my $getter = $options{getter}
  //     || sub { LookupValue(join('', $name, map { ToString($_) } @_)) || $value; };
  //   my $setter = $options{setter}
  //     || ($options{readonly}
  //     ? sub { my ($v, @args) = @_;
  //       Warn('unexpected', $name, $STATE->getStomach,
  //         "Can't assign to register $name"); return; }
  //     : sub { my ($v, @args) = @_;
  //       AssignValue(join('', $name, map { ToString($_) } @args) => $v); });
  //   # Not really right to set the value!
  //   AssignValue(ToString($cs) => $value) if defined $value;
  //   $STATE->installDefinition(LaTeXML::Core::Definition::Register->new($cs, $paramlist,
  //       registerType => $type,
  //       getter       => $getter, setter => $setter,
  //       readonly     => $options{readonly}),
  //     'global');
  return;
}

//**********************************************************************
/// This function computes an xml:id for a node, if it hasn't already got one.
/// It is suitable for use in Tag afterOpen as
///  `Tag('ltx:para',afterOpen=>sub { GenerateID(@_,'p'); });`
/// It generates an id of the form <parentid>.<prefix><number>
/// The parent node (the one with ID=<parentid>) also maintains a counter
/// stored in an attribute `_ID_counter_<prefix>` recording the last used
/// <number> for <prefix> amongst its descendents.
pub fn generate_id(
  document: &mut Document,
  mut node: &mut Node,
  mut prefix: &str,
  state: &mut State,
) -> Result<()>
{
  // If node doesn't already have an id, and can
  let node_qname = document.get_node_qname(node, state);
  if node.get_attribute("xml:id").is_none() && document.can_have_attribute(&node_qname, "xml:id", state)
    // but isn't a _Capture_ node (which ultimately should disappear)
    && (node_qname != "ltx:_Capture_")
  {
    let mut ancestor = document
      .findnode("ancestor::*[@xml:id][1]", Some(node), state)
      .unwrap_or_else(|| document.get_document().get_root_element().unwrap());
    //// Old versions don't like ancestor.getAttribute('xml:id');
    let ancestor_id = ancestor.get_attribute_ns("id", "http://www.w3.org/XML/1998/namespace");
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
    } + prefix + &ctr;

    ancestor.set_attribute(&ctrkey, &ctr)?;
    node.set_attribute("xml:id", &id)?;
  }
  Ok(())
}

pub fn merge_font(font: Font, state: &mut State) {
  let mut current_font = match state.remove_value("font") {
    Some(ObjectStore::Font(f)) => f,
    _ => Rc::new(Font::text_default()),
  };
  let newfont = current_font.merge(font);
  state.assign_value(
    "font",
    ObjectStore::Font(Rc::new(newfont)),
    Some(Scope::Local),
  );
  return;
}

pub fn digest_text(stuff: Tokens, stomach: &mut Stomach, state: &mut State) -> Result<Digested> {
  stomach.begin_mode("text", state)?;
  let value = stomach.digest(stuff, state);
  // TODO: ??? : Tokens!(map { (ref $_ ? $_ : TokenizeInternal($_)) } @stuff));
  stomach.end_mode("text", state)?;
  value
}

pub fn digest_literal(stuff: Tokens, stomach: &mut Stomach, state: &mut State) -> Result<Digested> {
  // Perhaps should do StartSemiverbatim, but is it safe to push a frame? (we might cover over
  // valid changes of state!)
  stomach.begin_mode("text", state)?;

  let font = state.lookup_font().unwrap(); // TODO: raise error if font missing
  state.assign_value(
    "font",
    ObjectStore::Font(Rc::new(font.merge(Font {
      encoding: Some(s!("ASCII")),
      ..Font::default()
    }))),
    Some(Scope::Local),
  ); // try to stay as ASCII as possible

  let value = stomach.digest(stuff, state);

  state.assign_value("font", ObjectStore::Font(font), None); // TODO: maybe we need .assign_font ?
  stomach.end_mode("text", state)?;
  value
}

pub fn digest_if(token: Token, stomach: &mut Stomach, state: &mut State) -> Result<Vec<Digested>> {
  if let Some(defn) = state.lookup_definition(&token) {
    match stomach.digest(Tokens!(token), state) {
      Ok(t) => Ok(vec![t]),
      Err(e) => Err(e),
    }
  } else {
    Ok(Vec::new())
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

pub fn new_counter(ctr: &str, within: &str, options: Option<NewCounterOptions>, state: &mut State) {
  let unctr = s!("UN{}", ctr); // UNctr is counter for generating ID's for UN-numbered items.
  let cctr = s!("\\c@{}", ctr);
  let clctr = s!("\\cl@{}", ctr);
  let cunctr = s!("\\c@{}", unctr);
  let clunctr = s!("\\cl@{}", unctr);

  def_register(T_CS!(cctr), None, Some(Number::new(0)), None, state);
  // state.assign_value(cctr, Number!(0), Some(Scope::Global));
  // // TODO:
  // // AfterAssignment!();
  // if !state.lookup_bool(&clctr) {
  //   state.assign_value(clctr, Tokens!(), Some(Scope::Global));
  // }
  // // TODO:
  // // DefRegisterI!(T_CS!(cunctr), None, Number!(0));
  // state.assign_value(cunctr, Number!(0), Some(Scope::Global));
  // if !state.lookup_bool(clunctr) {
  //   state.assign_value(clunctr, Tokens!(), Some(Scope::Global));
  // }

  // if !within.is_empty() {
  //   let clwithin = s!("\\cl@{}",within);
  //   let clunwithin = s!("\\cl@UN{}",within);
  //   let x = if let Some(ObjectStore::Tokens(cl)) = state.lookup_value(clwithin) {
  //    cl.unlist()
  //   } else {
  //     Vec::new()
  //   };
  //   let mut clwithin_tokens = vec![T_CS!(ctr), T_CS!(unctr)];
  //   clwithin_tokens.append(x);
  // state.assign_value(clwithin, ObjectStore::Tokens(Tokens{tokens: clwithin_tokens}),
  // Some(Scope::Global));

  //   let unx = if let Some(ObjectStore::Tokens(clun)) = state.lookup_value(clunwithin) {
  //     clun.unlist()
  //   } else {
  //    Vec::new()
  //   };
  //   let mut clunwithin_tokens = T_CS!(unctr);
  //   clunwithin_tokens.append(unx);

  // state.assign_value(clunwithin, ObjectStore::Tokens(Tokens{tokens: clunwithin_tokens}),
  // Some(Scope::Global)) }

  // if let Some(nested_val) = options.get("nested") {
  // state.assign_value(s!("nested_counters_{}", ctr), ObjectStore::String(nested_val),
  // Some(Scope::Global)) }

  // // default is equivalent to \arabic{ctr}, but w/o using the LaTeX macro!
  // DefMacroI!(T_CS!(s!("\\the{}",ctr)), None, move |gullet, args, inner_state| {
  //   let counter_value = CounterValue!(ctr, inner_state).value_of();
  //   Ok(ExplodeText!(counter_value))
  // },
  // scope => Some(Scope::Global));

  // let mut prefix = options.get("idprefix").unwrap_or(String::new());
  // if !prefix.is_empty() {
  // state.assign_value(s!("@ID@prefix@{}",ctr), ObjectStore::String(prefix),
  // Some(Scope::Global)); } else {
  //   prefix = state.lookup_string(s!("@ID@prefix@{}",ctr));
  //   if prefix.is_empty() {
  //     prefix = clean_id(ctr);
  //   }
  // }
  // if !prefix.is_empty() {
  //   let idwithin = options.get("idwithin").unwrap_or(within.clone());
  //   if !idwithin.is_empty() {
  //     DefMacro!(s!("\\the{}@ID",ctr),
  //       concat!(s!("\\expandafter\\ifx\\csname the{}@ID\\endcsname\\@empty",idwithin),
  //               s!("\\else\\csname the{}@ID\\endcsname.\\fi",idwithin),
  //               s!(" {}\\csname @{}@ID\\endcsname",prefix,ctr)),
  //       scope => Some(Scope::Global));
  //   }
  //   else {
  //     DefMacro!(s!("\\the{}@ID",ctr), s!("{}\\csname @{}@ID\\endcsname",prefix,ctr),
  //       scope => Some(Scope::Global));
  //   }
  //   DefMacro!(s!("\\@{}@ID",ctr), "0", scope => Some(Scope::Global));
  // }
  return;
}

pub fn counter_value(ctr: &str, state: &mut State) -> Number {
  match state.lookup_number(&s!("\\c@{}", ctr)) {
    None => {
      warn!(
        target: &s!("undefined:{:?}", ctr),
        "Counter {} was not defined; assuming 0",
        ctr
      );
      Number!(0)
    },
    Some(value) => value,
  }
}

pub fn add_to_counter(ctr: &str, value: Number, gullet: &mut Gullet, state: &mut State) {
  let v = counter_value(ctr, state).add(value);
  state.assign_value(
    &s!("\\c@{}", ctr),
    ObjectStore::Number(v.clone()),
    Some(Scope::Global),
  );
  after_assignment(gullet, state);
  SetupBindingMacros!(state);
  let id_cs = T_CS!(s!("\\@{}@ID", ctr));
  DefMacroTS!(id_cs.clone(), None, Tokens::new(Explode!(v.value_of())),
    scope => Some(Scope::Global));
}

pub fn step_counter(
  ctr: &str,
  noreset: bool,
  stomach: &mut Stomach,
  state: &mut State,
) -> Result<()>
{
  SetupBindingMacros!(state);
  let value = counter_value(ctr, state);
  state.assign_value(
    &s!("\\c@{}", ctr),
    ObjectStore::Number(value.add(Number!(1))),
    Some(Scope::Global),
  );
  {
    let gullet = stomach.get_gullet_mut();
    after_assignment(gullet, state);
  }
  let token_value = Tokens::new(Explode!(counter_value(ctr, state).value_of()));
  DefMacroTS!(T_CS!(s!("\\@{}@ID",ctr)), None, 
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

pub fn ref_step_counter(
  ctype: &str,
  noreset: bool,
  stomach: &mut Stomach,
  state: &mut State,
) -> Result<RefStepValue>
{
  let ctr = match state.lookup_mapping("counter_for_type", ctype) {
    Some(ObjectStore::String(ctr)) => ctr.to_string(),
    _ => ctype.to_string(),
  };
  step_counter(&ctr, noreset, stomach, state)?;

  let iddef_opt = state.lookup_definition(&T_CS!(s!("\\the{}@ID", ctr)));
  let has_id: bool = match iddef_opt {
    Some(ObjectStore::Expandable(iddef)) => match iddef.get_parameters() {
      Some(params) => params.get_num_args() == 0,
      None => false,
    },
    _ => false,
  };

  SetupBindingMacros!(state);
  let the_cs = T_CS!(s!("\\the{}", ctr));
  let the_id_cs = T_CS!(s!("\\the{}@ID", ctr));
  DefMacroT!(T_CS!("\\@currentlabel"), None, the_cs.clone(), scope => Some(Scope::Global));
  if has_id {
    DefMacroT!(T_CS!("\\@currentID"), None, the_id_cs.clone(), scope => Some(Scope::Global))
  }

  let id = if has_id {
    digest_literal(Tokens!(T_CS!(s!("\\the{}@ID", ctr))), stomach, state)?.to_string()
  } else {
    String::new()
  };

  let refnum = digest_text(Tokens!(T_CS!(s!("\\the{}", ctr))), stomach, state)?;
  // let tags = digest(Invocation!(T_CS!("\\lx@make@tags"), ctype));

  // Any scopes activated for previous value of this counter (& any nested counters) must be
  // removed. This may also include scopes activated for \label
  deactivate_counter_scope(&ctr, state);

  // And install the scope (if any) for this reference number.
  state.assign_value(
    "current_counter",
    ObjectStore::String(ctr.to_string()),
    Some(Scope::Local),
  );

  let scope = s!("{}:{}", ctr, refnum.to_string());
  state.assign_value(
    &s!("scopes_for_counter:{}", ctr),
    ObjectStore::VecString(vec![scope.clone()]),
    Some(Scope::Local),
  );
  state.activate_scope(&scope);

  Ok(RefStepValue {
    //   ($tags   ? (tags => $tags) : ()),
    tags: None,
    id: if has_id { Some(id) } else { None },
  })
}

fn deactivate_counter_scope(ctr: &str, state: &mut State) {
  //  print STDERR "Unusing scopes for $ctr\n";
  let scopes = if let Some(ObjectStore::VecString(stored_scopes)) =
    state.lookup_value(&s!("scopes_for_counter:{}", ctr))
  {
    stored_scopes.clone()
  } else {
    Vec::new()
  };
  for scope in scopes.iter() {
    state.deactivate_scope(scope);
  }

  let counters = if let Some(ObjectStore::VecString(stored_counters)) =
    state.lookup_value(&s!("nested_counters_{}", ctr))
  {
    stored_counters.clone()
  } else {
    Vec::new()
  };
  for inner_ctr in counters.iter() {
    state.deactivate_counter_scope(inner_ctr);
  }

  return;
}

//   // For UN-numbered units
//   #[macro_export]
//   macro_rules! RefStepID {
//   my ($ctr) = @_;
//   my $unctr = "UN$ctr";
//   StepCounter($unctr);
//   DefMacroI(T_CS!("\\\@$ctr\@ID"), undef,
//     Tokens(T_OTHER('x'), Explode(LookupValue('\c@' . $unctr)->valueOf)),
//     scope => Some(Scope::Global));
//   DefMacroI(T_CS!('\@currentID'), undef, T_CS!("\\the$ctr\@ID"));
//   return (id => ToString(DigestLiteral(T_CS!("\\the$ctr\@ID")))); }

fn reset_counter(ctr: &str, state: &mut State) {
  state.assign_value(
    &s!("\\c@{}", ctr),
    ObjectStore::Number(Number!(0)),
    Some(Scope::Global),
  );
  // and reset any within counters!
  let nested = if let Some(ObjectStore::Tokens(nested)) = state.lookup_value(&s!("\\cl@{}", ctr)) {
    nested.clone()
  } else {
    Tokens!()
  };

  for c in nested.unlist().iter() {
    reset_counter(&c.to_string(), state);
  }

  return;
}

fn after_assignment(gullet: &mut Gullet, state: &mut State) {
  if let Some(ObjectStore::Tokens(after)) = state.remove_value("afterAssignment") {
    gullet.unread(after); // primitive returns boxes, so these need to be digested!
  }
}
