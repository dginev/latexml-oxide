extern crate rtx_codegen;

// use std::sync::Arc;
use regex::Regex;
use rtx_core::Core;
// use rtx_core::common::object::Object;
use rtx_core::state::{State}; //ObjectStore
use rtx_core::token::*;
use rtx_core::parameter::{Parameter, Parameters};
use rtx_core::mouth::Mouth;

pub fn input_definitions(core: &mut Core, file: String) -> Result<(), ()> {
  match file.as_ref() { // TODO?
    "TeX.pool" => pool::tex::load_definitions(&mut core.state),
    "LaTeX.pool" => pool::latex::load_definitions(&mut core.state),
    other => { println!("TODO: load {:?}", other);}
  };
  Ok(())
}

pub fn input_content(core: &mut Core, request: String) -> Result<(), ()> {
  match find_file(request, false) { // TODO: type => $options{type}, noltxml => 1
    Some(path) => Ok(load_tex_content(core, path)),
    None => Err(()),
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
  // if (my $conf = !pathname_is_literaldata($pathname)
  //   && pathname_find("$file.latexml", paths => LookupValue('SEARCHPATHS'))) {
  //   loadLTXML($conf, $conf); }

  // TODO: Caching
  // content => LookupValue($pathname . '_contents')

  // Open a mouth for that TeX content
  let gullet = core.stomach.get_gullet_mut();
  gullet.open_mouth(mouth, true);

}

pub fn load_class(_state: &mut State, _class: String, _options: Vec<String>, _after: Vec<Token>) {
  // CheckOptions("LoadClass ($class)", $loadclass_options, %options);
  // PushValue(class_options => ($options{options} ? @{ $options{options} } : ()));
  // Note that we'll handle errors specifically for this case.
  // if (my $success = InputDefinitions($class, type => 'cls', notex => 1, handleoptions => 1, noerror => 1,
  //     %options)) {
  //   return $success; }
  // else {
  //   $STATE->noteStatus(missing => $class . '.cls');
  //   my $alternate = 'OmniBus';    # was 'article'
  //   Warn('missing_file', $class, $STATE->getStomach->getGullet,
  //     "Can't find binding for class $class (using $alternate)",
  //     maybeReportSearchPaths());
  //   if (my $success = InputDefinitions($alternate, type => 'cls', noerror => 1, handleoptions => 1, %options)) {
  //     return $success; }
  //   else {
  //     Fatal('missing_file', $alternate . '.cls.ltxml', $STATE->getStomach->getGullet,
  //       "Can't find binding for class $alternate (installation error)");
  //     return; } } }
}

pub fn find_file(request: String, _forbid_ltxml: bool) -> Option<String> {
  // TODO: Actually find it!
  Some(request)

}

pub fn coerce_cs(t: String) -> Token {
  T_CS!(t)
}

pub fn tokenize_internal(some: String) -> Vec<Token> {
  vec![T_CS!(some)]
}

pub fn parse_prototype(proto: &str, state: &mut State) -> ((Token, Option<Parameters>)) {
  lazy_static! {
    static ref CSNAME_MACRO_REGEX : Regex = Regex::new(r"^\\csname\s+(.*)\\endcsname").unwrap();
    static ref CS_REGEX : Regex = Regex::new(r"^(\\[a-zA-Z@]+)").unwrap();
    static ref SINGLE_CHAR_REGEX : Regex = Regex::new(r"^(\\.)").unwrap();
    static ref ACTIVE_CHAR_REGEX : Regex = Regex::new(r"^(.)").unwrap();
  }

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

pub fn parse_parameters(mut prototype: String, cs: &Token, state: &mut State) -> Option<Parameters> {
  lazy_static! {
    static ref NESTED_CHECK : Regex = Regex::new(r"^(\{([^\}]*)\})\s*").unwrap();
    static ref OPTIONAL_CHECK : Regex = Regex::new(r"^(\[([^\]]*)\])\s*").unwrap();
    static ref DEFAULT_CHECK : Regex = Regex::new(r"^Default:(.*)$").unwrap();
    static ref PARAMSPECT_CHECK : Regex = Regex::new(r"^((\w*)(:([^\s\{\[]*))?)\s*").unwrap();
  }
  let mut parameters = Vec::new();
  while !prototype.is_empty() {
    let mut next_proto = String::new();
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

    }
    // else {
    //   Fatal('misdefined', cs, undef, "Unrecognized parameter specification at \"prototype\""); } }
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

    $state.install_definition(::rtx_core::state::ObjectStore::ExpandableStore(Arc::new(
      Expandable { cs: $cs, paramlist: $paramlist, expansion: Arc::new($expansion),
       ..Expandable::default()})),
      &None);
  }
  )
);

#[macro_export]
macro_rules! DefMacro(
  ($proto:expr, $expansion:expr, $state:expr) => (
  {
// check_options("DefMacro (prototype)", $constructor_options, %options);
    let (cs, paramlist) = parse_prototype($proto, $state);
    DefMacroI!(cs, paramlist, $expansion, $state);
  }
  )
);

#[macro_export]
macro_rules! DefConstructorI(
  ($cs:expr, $paramlist:expr, $replacement:expr, $options: expr, $state:expr) => (
  {
    use rtx_core::definition::constructor::Constructor;
    use rtx_core::document::Document;
    use rtx_core::tbox::TBox;
    use std::collections::HashMap;
    // use libxml::tree::Node;
    use rtx_core::definition::constructor::ReplacementClosure;

// let mode    = $options.mode;
// let bounded = $options.bounded;
    let compiled_replacement;
    compile_replacement!(compiled_replacement, $replacement);
    let constructor = Constructor {
      cs: $cs,
      paramlist: $paramlist,
      replacement: compiled_replacement,
      ..Constructor::default()};

    $state.install_definition(::rtx_core::state::ObjectStore::ConstructorStore(Arc::new(constructor)), &None);

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
    DefConstructorI!(cs, paramlist, $replacement, $options, $state);
  }
  )
);

#[macro_export]
macro_rules! arg(
  ($num:expr) => (
  {

  }
  )
);

#[macro_export]
macro_rules! DefParameterType(
  ($param_type:expr, $options:expr, $state:expr) => (
  {
    // CheckOptions("DefParameterType param_type", $parameter_options, %options);
    $state.assign_mapping("PARAMETER_TYPES", &$param_type, $options);
  }
));

pub fn revert(_arg: Vec<Token>) -> Vec<Token> {
  Vec::new()
}


pub mod pool;
