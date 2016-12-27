use std::collections::HashMap;
use regex::Regex;
use rtx_core::Core;
use rtx_core::state::{State, ObjectStore, Scope};
use rtx_core::common::{Error};
use rtx_core::token::*;
use rtx_core::parameter::{Parameter, Parameters};
use rtx_core::mouth::Mouth;
use rtx_core::definition::{Definition, BeforeDigestClosure, ConstructionClosure};
use rtx_core::document::Document;
use rtx_core::document::resource::*;
use rtx_core::document::tag::{TagOptions, TagOptionName};
use rtx_core::util::pathname;

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


//======================================================================
// Convenience macros for writing definitions.
//======================================================================

#[macro_export]
macro_rules! LookupValue {
  ($name:expr, $state:expr) => ($state.lookup_value($name))
}

#[macro_export]
macro_rules! LookupBool {
  ($name:expr, $state:expr) => ($state.lookup_bool($name))
}

#[macro_export]
macro_rules! AssignValue {
  ($name:expr, $value:expr, $scope:expr, $state:expr) => ($state.assign_value($name, $value, $scope))
}

#[macro_export]
macro_rules! RemoveValue {
  ($name:expr, $state:expr) => ($state.remove_value($name))
}

#[macro_export]
macro_rules! PushValue {
  ($name:expr, $values:expr, $state:expr) => ($state.pushValue($name, $values))
}

#[macro_export]
macro_rules! PopValue {
  ($name:expr, $state:expr) => ($state.pop_value($name))
}

#[macro_export]
macro_rules! UnshiftValue {
  ($name:expr, $values:expr, $state:expr) => ($state.unshift_value($name, $values))
}

#[macro_export]
macro_rules! ShiftValue {
  ($name:expr, $state:expr) => ($state.shift_value($name))
}

#[macro_export]
macro_rules! LookupMapping {
  ($map:expr, $key:expr, $state:expr) => ($state.lookup_mapping($map, $key))
}

#[macro_export]
macro_rules! AssignMapping {
  ($map:expr, $key:expr, $value:expr, $state:expr) => ($state.assign_mapping($map, $key, $value))
}

#[macro_export]
macro_rules! LookupMappingKeys {
  ($map:expr, $state:expr) => ($state.lookup_mapping_keys($map))
}

#[macro_export]
macro_rules! LookupCatcode {
  ($char:expr, $state:expr) => ($state.lookup_catcode($char))
}

#[macro_export]
macro_rules! AssignCatcode {
  ($char:expr, $catcode:expr, $scope:expr, $state:expr) => ($state.assign_catcode($char, $catcode, $scope))
}

#[macro_export]
macro_rules! LookupMeaning {
  ($name:expr, $state:expr) => ($state.lookup_meaning($name))
}

#[macro_export]
macro_rules! LookupDefinition {
  ($name:expr, $state:expr) => ($state.lookup_definition($name))
}

#[macro_export]
macro_rules! InstallDefinition {
  ($name:expr, $definition:expr, $scope:expr, $state:expr) => ($state.install_definition($name, $definition, $scope))
}

// #[macro_export]
// macro_rules! XEquals {
//   ($token1:expr, $token2) => (
//   let def1 = LookupMeaning($token1);    # token, definition object or undef
//   let def2 = LookupMeaning($token2);    # ditto
//   if (defined $def1 != defined $def2) { # False, if only one has 'meaning'
//     return; }
//   elsif (!defined $def1 && !defined $def2) {    # true if both undefined
//     return 1; }
//   elsif ($def1->equals($def2)) {                # If both have defns, must be same defn!
//     return 1; }
//   return; }

/// Is defined in the LaTeX-y sense of also not being let to \relax.
pub fn is_defined(name: &str, state: &mut State) -> bool {
  let cs = T_CS!(name);
  is_defined_token(&cs, state)
}
pub fn is_defined_token(cs: &Token, state: &mut State) -> bool {
  match state.lookup_meaning(cs) {
    Some(store) => match store {
      & ObjectStore::Token(ref m) => true,
      & ObjectStore::Expandable(ref m) => m.get_cs_name() != "\relax",
      & ObjectStore::Primitive(ref m) => m.get_cs_name() != "\relax",
      & ObjectStore::Constructor(ref m) => m.get_cs_name() != "\relax",
      _ => false
    },
  _ => false }
}

#[macro_export]
macro_rules! IsDefined {
  ($name:expr, $state:expr) => (is_defined($name, $state))
}
macro_rules! IsDefinedToken {
  ($name:expr, $state:expr) => (is_defined_token($name, $state))
}

#[macro_export]
macro_rules! Let {
  ($token1:expr, $token2:expr, $state:expr) => ({
    LetI!(T_CS!($token1), T_CS!($token2), $state)
  });
  ($token1:expr, $token2:expr, $scope:expr, $state:expr) => ({
    LetI!(T_CS!($token1), T_CS!($token2), $scope, $state)
  });
}

#[macro_export]
macro_rules! LetI {
  ($token1:expr, $token2:expr, $state:expr) => ({
    // If strings are given, assume CS tokens (most common case)
    let meaning = match $state.lookup_meaning(&$token2) {
      Some(m) => m.clone(),
      None => ObjectStore::Token($token2)
    };
    $state.assign_meaning(&$token1, meaning, None);
    AfterAssignment!();
  });
  ($token1:expr, $token2:expr, $scope:expr, $state:expr) => ({
    // If strings are given, assume CS tokens (most common case)
    let meaning = match $state.lookup_meaning(&$token2) {
      Some(m) => m.clone(),
      None => ObjectStore::Token($token2)
    };
    $state.assign_meaning(&$token1, meaning, $scope);
    AfterAssignment!();
  });
}

#[macro_export]
macro_rules! AfterAssignment {
  () => ({
    // TODO
  })
}


//======================================================================
// Defining new Control-sequence Parameter types.
//======================================================================

#[macro_export]
macro_rules! DefParameterType (
  ($name:expr, $param:expr, $state:expr) => ($state.assign_mapping("PARAMETER_TYPES", $name, Some(ObjectStore::Parameter($param))))
);


pub struct InputDefinitionOptions {
  pub extension: Option<&'static str>,
  pub options: Vec<String>,
  pub after: Vec<Token>,
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
      after: Vec::new(),
      notex: false,
      noerror: false,
      noltxml: false,
      withoptions: Vec::new(),
      handleoptions: false,
      as_class: false
    }
  }
}

/// TODO: Flesh out with the full infrastructure, incremental functionality for now.
pub fn input_definitions(raw_file: String, options: InputDefinitionOptions, mut state: &mut State) {
  let mut file : String = raw_file.to_string().trim().to_string();

  // let prevname = if options.handleoptions {
  //   match state.lookup_definition(T_CS!("\@currname")) {
  //     Some(ObjectStore::Expandable(name)) => Digest!(T_CS!("\@currname")).to_string()
  // }
  // let prevext = options.handleoptions && $STATE->lookupDefinition(T_CS('\@currext')) && ToString(Digest(T_CS('\@currext')));


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
        return;
      }
    }
  }

  // Mark as loaded, then process the definitions
  println_stderr!("Loading {:?} definitions...", file);
  state.assign_value(&loaded_flag,
                     ObjectStore::Bool(true),
                     Some(Scope::Global));

  match file.as_ref() {
    "TeX.pool" => pool::tex::load_definitions(&mut state),
    "LaTeX.pool" => pool::latex::load_definitions(&mut state),
    "article.cls" => pool::article_cls::load_definitions(&mut state),
    "alltt.sty" => pool::alltt_sty::load_definitions(&mut state),
    other => { panic!("TODO: unknown binding {:?}, can't load", other);}
  };
}

#[macro_export]
macro_rules! LoadPool(
  ($name: expr, $state: expr) => (input_definitions($name.to_string(),
    InputDefinitionOptions {
      extension: Some("pool"),
      ..InputDefinitionOptions::default()
    }, $state))
);


pub fn input_content(core: &mut Core, request: &str) -> Result<(), Error> {
  match find_file(request, false) { // TODO: type => $options{type}, noltxml => 1
    Some(path) => Ok(load_tex_content(core, path)),
    None => Err(Error::MissingFile),
    // TODO:
    // Error("missing_file", request, state.get_stomach().get_gullet(),
    // "Can't find TeX file "+request, maybeReportSearchPaths(state)))
  }
}

pub fn load_tex_content(core: &mut Core, path: String) {
  let mut mouth = Mouth { notes: true, ..Mouth::default() };
  mouth.open(&path, &mut core.state);
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

/// This (& FindFile) needs to evolve a bit to support reading raw .sty (.def, etc) files from
/// the standard texmf directories.  Maybe even use kpsewhich itself (INSTEAD of pathname_find ???)
/// Another potentially useful option might be that if we are reading a raw file,
/// perhaps it should just get digested immediately, since it shouldn't contribute any boxes.
pub fn require_package(name: String, mut options: RequireOptions, state: &mut State) {
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
  }, state);
}

#[macro_export]
macro_rules! RequirePackage(
  ($package:expr, $options:expr, $state:expr) => (
  {
    require_package($package, $options, $state);
  }
));

pub fn load_class(name: String, options: Vec<String>, after: Vec<Token>, state: &mut State) {
  input_definitions(name, InputDefinitionOptions {
    extension: Some("cls"),
    after: after,
    notex: true,
    handleoptions: true,
    noerror: true,
    ..InputDefinitionOptions::default()
  }, state);
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

#[macro_export]
macro_rules! LoadClass(
  ($class:expr, $options:expr, $after:expr, $state:expr) => (
  {
    load_class($class, $options, $after , $state);
  }
));

pub fn find_file(request: &str, _forbid_ltxml: bool) -> Option<String> {
  // TODO: Actually find it!
  Some(request.to_string())

}

pub fn coerce_cs(t: String) -> Token {
  T_CS!(t)
}

pub fn tokenize_internal(some: String) -> Vec<Token> {
  vec![T_CS!(some)]
}

lazy_static! {
  static ref CSNAME_MACRO_REGEX : Regex = Regex::new(r"^\\csname\s+(.*)\\endcsname").unwrap();
  static ref CS_REGEX : Regex = Regex::new(r"^(\\[a-zA-Z@]+)").unwrap();
  static ref SINGLE_CHAR_REGEX : Regex = Regex::new(r"^(\\.)").unwrap();
  static ref ACTIVE_CHAR_REGEX : Regex = Regex::new(r"^(.)").unwrap();
}
pub fn parse_prototype(proto: &str, state: &mut State) -> ((Token, Option<Parameters>)) {
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
    cs = tokenize_internal(captures.at(0).unwrap().to_string()).first().unwrap().clone();
    // also replace in proto
    ACTIVE_CHAR_REGEX.replace(proto, "")
  } else {
    // Fatal('misdefined', prototype, $STATE->getStomach,
    //   "Definition prototype doesn't have proper control sequence: \"prototype\""); }
    proto.to_string()
  };
  final_proto = final_proto.trim_left().to_string();
  let paramlist = parse_parameters(final_proto, &cs, state);
  (cs, paramlist)
}

lazy_static! {
  static ref NESTED_CHECK : Regex = Regex::new(r"^(\{([^\}]*)\})\s*").unwrap();
  static ref OPTIONAL_CHECK : Regex = Regex::new(r"^(\[([^\]]*)\])\s*").unwrap();
  static ref DEFAULT_CHECK : Regex = Regex::new(r"^Default:(.*)$").unwrap();
  static ref PARAMSPECT_CHECK : Regex = Regex::new(r"^((\w*)(:([^\s\{\[]*))?)\s*").unwrap();
}
pub fn parse_parameters(mut prototype: String, cs: &Token, state: &mut State) -> Option<Parameters> {
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
        parse_parameters(inner_spec.to_string(), cs, state)
      };
      parameters.push(Parameter {
                        name: "Plain".to_string(),
                        spec: spec.to_string(),
                        extra: vec![inner],
                        ..Parameter::default()
                      }
                      .init(state));

    } else if OPTIONAL_CHECK.is_match(&prototype) {
      // Ditto for Optional
      let captures = OPTIONAL_CHECK.captures(&prototype).unwrap();
      next_proto = OPTIONAL_CHECK.replace(&prototype, "");
      let spec = captures.at(1).unwrap();
      let inner_spec = captures.at(2).unwrap();

      if DEFAULT_CHECK.is_match(inner_spec) {
        // let default_captures = DEFAULT_CHECK.captures(&inner_spec).unwrap();
        parameters.push(Parameter {
                          name: "Optional".to_string(),
                          spec: spec.to_string(),
                          // extra: vec![TokenizeInternal(default_captures.at(0).unwrap()), None]});
                          extra: Vec::new(),
                          ..Parameter::default()
                        }
                        .init(state));
      } else if !inner_spec.is_empty() {
        parameters.push(Parameter {
                          name: "Optional".to_string(),
                          spec: spec.to_string(),
                          extra: vec![None, parse_parameters(inner_spec.to_string(), cs, state)],
                          ..Parameter::default()
                        }
                        .init(state));
      } else {
        parameters.push(Parameter {
                          name: "Optional".to_string(),
                          spec: spec.to_string(),
                          extra: Vec::new(),
                          ..Parameter::default()
                        }
                        .init(state));
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
          // extra_string.split("|").map(|t| tokenize_internal(t.to_string())).collect::<Vec<Token>>();
          Vec::new()
        }
      };
      parameters.push(Parameter {
                        name: spec_type.to_string(),
                        spec: spec.to_string(),
                        extra: extra,
                        ..Parameter::default()
                      }
                      .init(state));

    } else {
      // Fatal('misdefined', cs, undef, "Unrecognized parameter specification at \"prototype\""); }
      panic!("Fatal:misdefined:{:?} Unrecognized parameter specification at \"prototype\"", cs);
    }
    prototype = next_proto;
  }
  if parameters.is_empty() {
    None
  } else {
    Some(Parameters { params: parameters })
  }
}

/// Macros and pool come at the end, so that they load seamlessly
// TODO: package::coerce_cs on $cs
#[macro_export]
macro_rules! DefMacroI(
  ($cs:expr, $paramlist:expr, $expansion:expr, $state:expr) => (
  {
    use rtx_core::definition::expandable::{Expandable};
//       // Optimization: Defer till macro actually used
//       // if !$cs.is_empty() { // && $options{mathactive}
//         // $state.assign_mathcode($cs, 0x8000, $options{scope}); }
//       $state.install_definition(Expandable{ cs: coerce_cs( $cs ), paramlist: $paramlist, expansion: $expansion});//, %options), $options{scope});
//       // if $options{locked} {
//       //   $state.assign_value(ToString($cs)+":locked", true, "global")
//       // }

    $state.install_definition(::rtx_core::state::ObjectStore::Expandable(Rc::new(
      Expandable { cs: $cs, paramlist: $paramlist, expansion: Rc::new($expansion),
       ..Expandable::default()})),
      None);
  }
  )
);

#[macro_export]
macro_rules! DefMacroT {
    ($cs:expr, $paramlist:expr, $body:expr, $state:expr) => ({
      DefMacroI!($cs, $paramlist, move |_gullet, _args, state| {$body}, $state)
    })
}

#[macro_export]
macro_rules! DefMacro(
  ($proto:expr, $expansion:expr, $state:expr) => (
  {
    let (cs, paramlist) = parse_prototype($proto, $state);
    DefMacroI!(cs, paramlist, $expansion, $state);
  }
  )
);


///======================================================================
/// Define a primitive control sequence.
///======================================================================
/// Primitives are executed in the Stomach.
/// The $replacement should be a sub which returns nothing, or a list of Box's or Whatsit's.
/// The options are:
///    isPrefix  : 1 for things like \global, \long, etc.
///    registerType : for parameters (but needs to be worked into DefParameter, below).

#[macro_export]
macro_rules! DefPrimitive(
  ($proto:expr, $replacement:expr, $options:expr, $state:expr) => ({
    // TODO:
    // let compiled_replacement = || Tbox{text: $replacement, Invocation($options{alias} || $cs, @_[1 .. $#_])); }
    let compiled_replacement = $replacement;

    DefPrimitiveI!($proto, compiled_replacement, $options, $state);
  })
);

#[macro_export]
macro_rules! DefPrimitiveI(
  ($proto:expr, $compiled_replacement:expr, $options:expr, $state:expr) => ({
    let (cs, paramlist) = parse_prototype($proto, $state);
    // let compiled_replacement = || Tbox{text: $replacement, Invocation($options{alias} || $cs, @_[1 .. $#_])); }
    DefPrimitiveII!(cs, paramlist, $compiled_replacement, $options, $state);
  })
);

#[macro_export]
macro_rules! DefPrimitiveII(
  ($cs:expr, $paramlist:expr, $compiled_replacement:expr, $options:expr, $state:expr) => ({

  let mode    = $options.mode;
  let bounded = $options.bounded;
  $state.install_definition(ObjectStore::Primitive(Rc::new(Primitive{
      cs: $cs.clone(),
      paramlist: $paramlist,
      replacement: Some(Rc::new($compiled_replacement)),
      // beforeDigest => flatten(($options{requireMath} ? (sub { requireMath($cs); }) : ()),
      //   ($options{forbidMath} ? (sub { forbidMath($cs); }) : ()),
      //   ($mode ? (sub { $_[0]->beginMode($mode); })
      //     : ($bounded ? (sub { $_[0]->bgroup; }) : ())),
      //   ($options{font} ? (sub { MergeFont(%{ $options{font} }); }) : ()),
      //   $options{beforeDigest}),
      // afterDigest => flatten($options{afterDigest},
      //   ($mode ? (sub { $_[0]->endMode($mode) })
      //     : ($bounded ? (sub { $_[0]->egroup; }) : ()))),
      options: $options,
      ..Primitive::default()
    })),
    $options.scope);
  if $options.locked {
    AssignValue!(&($cs.to_string()+":locked"), ObjectStore::Bool(true), None, $state);
  }
}));



#[macro_export]
macro_rules! DefConstructorI(
  ($cs:expr, $paramlist:expr, $compiled_replacement:expr, $options: expr, $state:expr) => (
  {
    use rtx_core::definition::constructor::Constructor;
    // use libxml::tree::Node;

// let mode    = $options.mode;
// let bounded = $options.bounded;

    // TODO: This won't work, as we can only invoke method calls on paramlist in runtime
    //*rtx_codegen::constructable::NARGS = $paramlist.get_num_args();
    let constructor = Constructor {
      cs: $cs,
      paramlist: $paramlist,
      replacement: $compiled_replacement,
      options: $options,
      ..Constructor::default()};

    $state.install_definition(::rtx_core::state::ObjectStore::Constructor(Rc::new(constructor)), None);

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

// if options.locked {
//   $state.assign_value(ToString($cs) + ":locked", Box::new(true))
// }
// return;
  }
  );
);

#[macro_export]
macro_rules! DefConstructor(
  ($proto:expr, $replacement:expr, $options:expr, $state:expr) => (
  {
// check_options("DefConstructor (prototype)", $constructor_options, %options);
    let (cs, paramlist) = parse_prototype($proto, $state);
    let compiled_replacement;
    compile_replacement!(compiled_replacement, $replacement);
    DefConstructorI!(cs, paramlist, compiled_replacement, $options, $state);
  }
  )
);

pub fn revert(_arg: Vec<Token>) -> Vec<Token> {
  Vec::new()
}


//=====================================================================
// Define a LaTeX environment
// Note that the body of the environment is treated is the 'body' parameter in the constructor.

#[macro_export]
macro_rules! DefEnvironment (
  ($proto_raw:expr, $replacement:expr, $options:expr, $state:expr) => ({
  use rtx_core::util::text::*;
  let mut proto = $proto_raw.to_string().trim_left().to_string();
  let name = extract_bracketed(&mut proto, Some(Delimiter::Brace));

  let compiled_replacement;
  compile_replacement!(compiled_replacement, $replacement);
  let cc_copy;
  compile_replacement!(cc_copy, $replacement);

  DefEnvironmentI!(name, None, compiled_replacement, cc_copy, $options, $state);
}));

#[macro_export]
macro_rules! DefEnvironmentC (
  ($proto_raw:expr, $compiled_replacement:expr, $options:expr, $state:expr) => ({
  use rtx_core::util::text::*;
  let mut proto = $proto_raw.to_string().trim_left().to_string();
  let name = extract_bracketed(&mut proto, Some(Delimiter::Brace));
  // TODO: What do we do with param lists?
  //let paramlist_str = proto.trim_left().to_string();
  DefEnvironmentI!(name, None, $compiled_replacement, $compiled_replacement, $options, $state);
}));

#[macro_export]
macro_rules! DefEnvironmentI (
  ($name_raw:expr, $paramlist:expr, $compiled_replacement:expr, $cc_copy:expr, $options:expr, $state:expr) => ({
  use rtx_core::stomach::Stomach;
  use rtx_core::whatsit::Whatsit;
  use rtx_core::definition::constructor::Constructor;
  let mode = $options.mode;
  let name = $name_raw.to_string();
  // This is for the common case where the environment is opened by \begin{env}
  // let sizer = inferSizer($options.sizer, $options.reversion);
  let bgroup_closure = Rc::new(|stomach: &mut Stomach, state: &mut State| {stomach.bgroup(state); Vec::new()});
  let mut before_digest_with_group : Vec<BeforeDigestClosure> = vec![bgroup_closure];
  before_digest_with_group.extend($options.before_digest);

  let push_frame_closure = Rc::new(|_document: &mut Document, _whatsit: &Whatsit, state: &mut State| {
    state.push_frame();
  });
  let mut before_construct_with_frame : Vec<ConstructionClosure> = vec![push_frame_closure];
  before_construct_with_frame.extend($options.before_construct);

  let mut after_construct_with_frame : Vec<ConstructionClosure> = $options.after_construct;

  let pop_frame_closure = Rc::new(|_document: &mut Document, _whatsit: &Whatsit, state: &mut State| {
    state.pop_frame();
  });
  after_construct_with_frame.push(pop_frame_closure);

  let begin_name_constructor = Rc::new(Constructor {
      cs: T_CS!("\\begin{".to_string()+&name+"}"),
      paramlist: $paramlist,
      replacement: $compiled_replacement,
      options: ConstructorOptions {
        nargs: $options.nargs,
        before_digest: before_digest_with_group,
        // beforeDigest => flatten(($options{requireMath} ? (sub { requireMath($name); }) : ()),
        //   ($options{forbidMath} ? (sub { forbidMath($name); }) : ()),
        //   ($mode ? (sub { $_[0]->beginMode($mode); })
        //     : (sub { $_[0]->bgroup; })),
        //   sub { AssignValue(current_environment => $name);
        //     DefMacroI('\@currenvir', undef, $name); },
        //   ($options{font} ? (sub { MergeFont(%{ $options{font} }); }) : ()),
        //   $options{beforeDigest}),
        after_digest: $options.after_digest_begin,
        after_digest_body: $options.after_digest_body,
        before_construct: before_construct_with_frame,
        // Curiously, it's the \begin whose afterConstruct gets called.
        after_construct: after_construct_with_frame,
        capture_body: true,
        properties: $options.properties,
        // (defined $options{reversion} ? (reversion => $options{reversion}) : ()),
        // (defined $sizer ? (sizer => $sizer) : ()),
        // ), $options{scope});
        ..ConstructorOptions::default()
      }});
  $state.install_definition(ObjectStore::Constructor(begin_name_constructor), $options.scope);


  let mut after_digest_with_egroup = $options.after_digest;
  let unexpected_end_closure = Rc::new(|_stomach: &mut Stomach, _whatsit: &mut Whatsit, state: &mut State| {
    let env = LookupValue!("current_environment", state);
    //     Error('unexpected', "\\end{$name}", $_[0],
    //       "Can't close environment $name",
    //       "Current are "
    //         . join(', ', $STATE->lookupStackedValues('current_environment')))
    //       unless $env && $name eq $env;
    //     return; },
    Vec::new()
  });
  let egroup_closure = Rc::new(move |stomach: &mut Stomach, _whatsit: &mut Whatsit, state: &mut State| {
    if mode.is_some() {
      // TODO:
      // stomach.end_mode(mode.unwrap(), state);
    } else {
      stomach.egroup(state);
    }
    Vec::new()
  });
  after_digest_with_egroup.push(unexpected_end_closure);
  after_digest_with_egroup.push(egroup_closure);
  let end_envname_constructor = Rc::new(Constructor {
    cs: T_CS!("\\end{".to_string()+&name+"}"),
    replacement: None,
    paramlist: None,
    options: ConstructorOptions {
      before_digest: $options.before_digest_end,
      after_digest: after_digest_with_egroup,
      ..ConstructorOptions::default()
    }
  });
  $state.install_definition(ObjectStore::Constructor(end_envname_constructor), $options.scope);

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
    // beforeConstruct => flatten(sub { $STATE->pushFrame; }, $options{beforeConstruct}),
    // Curiously, it's the \begin whose afterConstruct gets called.
    // afterConstruct => flatten($options{afterConstruct}, sub { $STATE->popFrame; }),
    options: ConstructorOptions {
      nargs: $options.nargs,
      capture_body: true,
      properties: $options.properties,
      // (defined $options{reversion} ? (reversion => $options{reversion}) : ()),
      // (defined $sizer ? (sizer => $sizer) : ()),
      // ), $options{scope});
      ..ConstructorOptions::default()
    }
  });
  $state.install_definition(ObjectStore::Constructor(name_constructor), $options.scope);

  let end_name_constructor = Rc::new(Constructor {
    cs: T_CS!("\\end".to_string() + &name),
    paramlist: None,
    replacement: Some(Rc::new(|document, whatsit, properties, state|{
      let env = LookupValue!("current_environment", state);
      // Error('unexpected', "\\end{$name}", $_[0],
      //   "Can't close environment $name",
      //   "Current are "
      //     . join(', ', $STATE->lookupStackedValues('current_environment')))
      //   unless $env && $name eq $env;
      return; })),
    // beforeDigest => flatten($options{beforeDigestEnd}),
    // afterDigest  => flatten($options{afterDigest},
    //   ($mode ? (sub { $_[0]->endMode($mode); }) : ())),
    // ), $options{scope});
    options: ConstructorOptions::default()
  });
  $state.install_definition(ObjectStore::Constructor(end_name_constructor), $options.scope);

  if ($options.locked) {
    AssignValue!(&("\\begin{".to_string() + &name+"}:locked"), ObjectStore::Bool(true), None, $state);
    AssignValue!(&("\\end{".to_string()+&name+"}:locked")  , ObjectStore::Bool(true), None, $state);
    AssignValue!(&("\\".to_string()+&name+":locked")       , ObjectStore::Bool(true), None, $state);
    AssignValue!(&("\\end".to_string()+&name+":locked")    , ObjectStore::Bool(true), None, $state);
  }
}));

//======================================================================
// Declaring and Adjusting the Document Model.
//======================================================================

pub fn install_tag(tag: &str, mut properties: TagOptions, state: &mut State) {
  let mut options = state.tag_properties.entry(tag.to_string()).or_insert(TagOptions::default());
  options.auto_open = properties.auto_open;
  options.auto_close = properties.auto_close;

  for name in TagOptionName::all().iter() {
    if name.is_prepend() {
      options.prepend(name, properties.remove(name));
    } else if name.is_append() {
      options.append(name, properties.remove(name));
    } else {
      // we'll handle the regular ones out of the loop
    }
  }
}

#[macro_export]
macro_rules! Tag (
  ($tag:expr, $properties:expr, $state:expr) => (install_tag($tag, $properties, $state);)
);

// sub DocType {
//   my ($rootelement, $pubid, $sysid, %namespaces) = @_;
//   let model = $STATE->getModel;
//   $model->setDocType($rootelement, $pubid, $sysid);
//   foreach let prefix (keys %namespaces) {
//     $model->registerDocumentNamespace($prefix => $namespaces{$prefix}); }
//   return; }

// Selects the RelaxNG schema defining the XML output language
pub fn select_relaxng_schema(schema : String, namespaces : Option<HashMap<String,String>>, state: &mut State) {
  // What verb here? Set, Choose,...
  let model = &mut state.model;
  model.set_relaxng_schema(schema);
  if let Some(namespaces) = namespaces {
    for (prefix, value) in namespaces.into_iter() {
      model.register_document_namespace(&prefix, Some(value)); }
  }
  return; }

#[macro_export]
macro_rules! RelaxNGSchema(
  ($name:expr, $state:expr) => (select_relaxng_schema($name.to_string(), None, $state))
);

#[macro_export]
macro_rules! RegisterNamespace(
  ($prefix:expr, $namespace:expr, $state:expr) => ($state.model.register_namespace($prefix, Some($namespace.to_string()));)
);

#[macro_export]
macro_rules! RegisterDocumentNamespace(
  ($prefix:expr, $namespace:expr, $state:expr) => ($state.model.register_document_namespace($prefix, Some($namespace.to_string()));)
);


pub fn require_resource(mut resource: Resource, state: &mut State) {
  if resource.name.is_empty() && resource.content.is_empty() {
    println_stderr!("Warn:expected:resource: Resource must have a resource pathname or content; skipping");
    return;
  }
  if resource.mimetype.is_empty() && !resource.name.is_empty() {
    let ext = pathname::extension(&resource.name);
    resource.mimetype = resource_type(&ext);
  }
  if resource.mimetype.is_empty() {
    println_stderr!("Warning:expected:mime-type Resource must have a mime-type; skipping");
    return;
  }

  // If we've got a document, go ahead & put the resource in.
  // if (state.document.is_some()) {
  //   state.document.as_mut().unwrap().add_resource(resource, resource);
  // } else {
  state.pending_resources.push(resource);
  // }

}

#[macro_export]
macro_rules! RequireResource(
  ($resource:expr, $state:expr) => (require_resource(Resource{name: $resource.to_string(), ..Resource::default()}, $state))
);

pub mod pool;
