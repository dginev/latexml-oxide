use std::sync::Arc;
use regex::Regex;
use common::object::Object;
use core::{Core};
use state::{State};
use core::token::*;
use core::parameter::{Parameter,Parameters};
use core::mouth::Mouth;
// use common::{Error};

pub fn input_definitions(core: &mut Core, file : String) -> Result<(),()> {
  match file.as_ref() { // TODO?
    "TeX.pool" => pool::tex::load_definitions(&mut core.state),
    _ => {}
  };
  Ok(())
}

pub fn input_content(core : &mut Core, request : String) -> Result<(),()> {
  match find_file(request, false) { // TODO: type => $options{type}, noltxml => 1
    Some(path) => Ok(load_tex_content(core, path)),
    None => Err(())
      // TODO:
      // Error("missing_file", request, state.get_stomach().get_gullet(),
      // "Can't find TeX file "+request, maybeReportSearchPaths(state)))
  }
}

pub fn load_tex_content(core: &mut Core, path : String) {
  let mut mouth = Mouth{notes: true, ..Mouth::default()};
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
  let gullet = core.stomach.get_gullet();
  gullet.open_mouth(mouth, true);

}

pub fn load_class(_state: &mut State, class : String, _options : Vec<String>, _after : Vec<Token>) {
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

pub fn find_file(request : String, _forbid_ltxml : bool) -> Option<String> {
  // TODO: Actually find it!
  Some(request)

}

pub fn coerce_cs(t : String) -> Token {
  T_CS!(t)
}

pub fn tokenize_internal(some : String) -> Vec<Token> {
  vec![T_CS!(some)]
}

pub fn parse_prototype(proto : String, state: &mut State) -> ((Token, Option<Parameters>)) {
  let csname_macro_regex = Regex::new(r"^\\csname\s+(.*)\\endcsname").unwrap();
  let cs_regex = Regex::new(r"^(\\[a-zA-Z@]+)").unwrap();
  let single_char_regex = Regex::new(r"^(\\.)").unwrap();
  let active_char_regex = Regex::new(r"^(.)").unwrap();
  let mut final_proto = proto.clone();
  let mut cs = T_CS!("\\".to_string()); // Should never happen
  if csname_macro_regex.is_match(&proto) {
    let captures = csname_macro_regex.captures(&proto).unwrap();
    cs = T_CS!("\\".to_string() + captures.at(0).unwrap());
    // also replace in proto
    final_proto = csname_macro_regex.replace(&proto,"");
  } else if cs_regex.is_match(&proto) { // Match a cs
    let captures = cs_regex.captures(&proto).unwrap();
    let csname = captures.at(0).unwrap().to_string();
    cs = T_CS!(csname);
    // also replace in proto
    final_proto = cs_regex.replace(&proto,"");
  } else if single_char_regex.is_match(&proto) { // Match a single char cs, env name,...
    let captures = single_char_regex.captures(&proto).unwrap();
    cs = T_CS!(captures.at(0).unwrap().to_string());
    // also replace in proto
    final_proto = single_char_regex.replace(&proto,"");
  } else if active_char_regex.is_match(&proto) { // Match an active char
    let captures = active_char_regex.captures(&proto).unwrap();
    cs = tokenize_internal(captures.at(0).unwrap().to_string()).first().unwrap().clone();
    // also replace in proto
    final_proto = active_char_regex.replace(&proto,"");
  } else {
    // Fatal('misdefined', prototype, $STATE->getStomach,
    //   "Definition prototype doesn't have proper control sequence: \"prototype\""); }
  }
  final_proto = final_proto.trim_left().to_string();
  let paramlist = parse_parameters(final_proto, &cs, state);
  return (cs, paramlist)
}

pub fn parse_parameters(mut prototype : String, cs : &Token, state : &mut State) -> Option<Parameters> {
  let mut parameters = Vec::new();
  let nested_check = Regex::new(r"^(\{([^\}]*)\})\s*").unwrap();
  let optional_check = Regex::new(r"^(\[([^\]]*)\])\s*").unwrap();
  let default_check = Regex::new(r"^Default:(.*)$").unwrap();
  let paramspec_check = Regex::new(r"^((\w*)(:([^\s\{\[]*))?)\s*").unwrap();

  while !prototype.is_empty() {
    let mut next_proto = String::new();
    // Handle possibly nested cases, such as {Number}
    if nested_check.is_match(&prototype) {
      let captures = nested_check.captures(&prototype).unwrap();
      next_proto = nested_check.replace(&prototype,"");
      let spec = captures.at(1).unwrap();
      let inner_spec = captures.at(2).unwrap();
      let inner : Option<Parameters> = if inner_spec.is_empty() { None } else { parse_parameters(inner_spec.to_string(), cs, state)};
      parameters.push(Parameter { name: "Plain".to_string(), spec: spec.to_string(), extra: vec![inner], ..Parameter::default()}.init(state));

    }
    else if optional_check.is_match(&prototype) { // Ditto for Optional
      let captures = optional_check.captures(&prototype).unwrap();
      next_proto = optional_check.replace(&prototype,"");
      let spec = captures.at(1).unwrap();
      let inner_spec = captures.at(2).unwrap();

      if default_check.is_match(inner_spec) {
        let default_captures = default_check.captures(&inner_spec).unwrap();
        parameters.push(Parameter{name: "Optional".to_string(), spec: spec.to_string(),
          // extra: vec![TokenizeInternal(default_captures.at(0).unwrap()), None]});
          extra: Vec::new(), ..Parameter::default()}.init(state));
      } else if !inner_spec.is_empty() {
        parameters.push(Parameter{name: "Optional".to_string(), spec: spec.to_string(),
          extra: vec![None, parse_parameters(inner_spec.to_string(), cs, state)], ..Parameter::default()}.init(state)); }
      else {
        parameters.push(Parameter{name: "Optional".to_string(), spec: spec.to_string(), extra: Vec::new(), ..Parameter::default()}.init(state));
      }
    }

    else if paramspec_check.is_match(&prototype) {
      let captures = paramspec_check.captures(&prototype).unwrap();
      next_proto = paramspec_check.replace(&prototype,"");
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
      parameters.push(Parameter{ name: spec_type.to_string(), spec: spec.to_string(), extra: extra, ..Parameter::default()}.init(state));

    }
    // else {
    //   Fatal('misdefined', cs, undef, "Unrecognized parameter specification at \"prototype\""); } }
    prototype = next_proto;
  }
  if parameters.len() == 0 {
    None
  } else {
    Some(Parameters { params : parameters })
  }
}

/// Macros and pool come at the end, so that they load seamlessly
// TODO: package::coerce_cs on $cs
#[macro_export]
macro_rules! DefMacroI(
  ($cs:expr, $paramlist:expr, $expansion:expr, $state:expr) => (
  {
    use $crate::core::definition::expandable::{Expandable};
    //       // Optimization: Defer till macro actually used
    //       // if !$cs.is_empty() { // && $options{mathactive}
    //         // $state.assign_mathcode($cs, 0x8000, $options{scope}); }
    //       $state.install_definition(Expandable{ cs: coerce_cs( $cs ), paramlist: $paramlist, expansion: $expansion});//, %options), $options{scope});
    //       // if $options{locked} {
    //       //   $state.assign_value(ToString($cs)+":locked", true, "global")
    //       // }

    $state.install_definition(::state::ObjectStore::ExpandableStore(Arc::new(Box::new(
      Expandable { cs: $cs, paramlist: $paramlist, expansion: Arc::new(Box::new($expansion)),
       ..Expandable::default()}))),
      &None);
  }
  )
);

#[macro_export]
macro_rules! DefMacro(
  ($proto:expr, $expansion:expr, $state:expr) => (
  {
    // check_options("DefMacro (prototype)", $constructor_options, %options);
    let (cs, paramlist) = parse_prototype($proto.to_string(), $state);
    DefMacroI!(cs, paramlist, $expansion, $state);
  }
  )
);

#[macro_export]
macro_rules! DefConstructorI(
  ($cs:expr, $paramlist:expr, $replacement:expr, $options: expr, $state:expr) => (
  {
    use $crate::core::definition::constructor::Constructor;
    use $crate::core::package;
    // let mode    = $options.mode;
    // let bounded = $options.bounded;
    let mut constructor = Constructor { cs: $cs, paramlist: $paramlist, replacement: $replacement, ..Constructor::default()};
    constructor.compile();
    $state.install_definition(::state::ObjectStore::ConstructorStore(Arc::new(Box::new(constructor))), &None);

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

pub fn DefParameterType(param_type: String, options: Parameter, state : &mut State) {
  // CheckOptions("DefParameterType param_type", $parameter_options, %options);
  state.assign_mapping("PARAMETER_TYPES", &param_type, options);
  return;
}

pub fn Revert(arg: Vec<Token>) -> Vec<Token> {
  Vec::new()
}


pub mod pool;