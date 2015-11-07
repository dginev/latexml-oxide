use core::{Core};
use state::State;
use core::token::*;
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

pub fn load_class(state: &mut State, name : String) {
  
}

pub fn find_file(request : String, forbid_ltxml : bool) -> Option<String> {
  // TODO: Actually find it!
  Some(request)

}

pub fn coerce_cs(t : String) -> Token {
  T_CS(t)
}

pub fn tokenize_internal(some : String) -> Vec<Token> {
  vec![T_CS(some)]
}

pub fn parse_prototype(proto : String) -> ((Token, String)) {
  let csname_macro_regex = regex!(r"/^\\csname\s+(.*)\\endcsname/");
  let cs_regex = regex!(r"/^(\\[a-zA-Z@]+)/");
  let single_char_regex = regex!(r"/^(\\.)/");
  let active_char_regex = regex!(r"/^(.)/");
  let mut final_proto = proto;
  let mut cs;

  if csname_macro_regex.is_match(&proto) {
    let captures = csname_macro_regex.captures(&proto).unwrap();
    cs = T_CS("\\".to_string() + captures.at(0).unwrap()); 
    // also replace in proto
    final_proto = csname_macro_regex.replace(&proto,"");
  } else if cs_regex.is_match(&proto) { // Match a cs
    let captures = cs_regex.captures(&proto).unwrap();
    cs = T_CS(captures.at(0).unwrap().to_string()); 
    // also replace in proto
    final_proto = cs_regex.replace(&proto,"");
  } else if single_char_regex.is_match(&proto) { // Match a single char cs, env name,...
    let captures = single_char_regex.captures(&proto).unwrap();
    cs = T_CS(captures.at(0).unwrap().to_string()); 
    // also replace in proto
    final_proto = single_char_regex.replace(&proto,"");
  } else if active_char_regex.is_match(&proto) { // Match an active char
    let captures = active_char_regex.captures(&proto).unwrap();
    cs = *tokenize_internal(captures.at(0).unwrap().to_string()).first().unwrap();
    // also replace in proto
    final_proto = active_char_regex.replace(&proto,"");
  } else {
    // Fatal('misdefined', $proto, $STATE->getStomach,
    //   "Definition prototype doesn't have proper control sequence: \"$proto\""); }
  }
  final_proto = final_proto.trim_left().to_string();

  return (cs, final_proto); }

/// Macros and pool come at the end, so that they load seamlessly
use core::definition::expandable::Expandable;
#[macro_export]
macro_rules! DefMacroI(
  ($cs:expr, $paramlist:expr, $expansion:expr, $state:expr) => (
  {
    use $crate::core::definition::expandable::Expandable;
    use $crate::core::package;
    $state.install_definition(Expandable { cs: package::coerce_cs( $cs ), paramlist: $paramlist, expansion: $expansion, ..Expandable::default()}, &None);
  }
  )
);

// macro_rules! DefMacroI(
//     ($cs:expr, $paramlist:expr, $expansion:expr, $state:expr) => (
//       {//, $options:tt
//       // Optimization: Defer till macro actually used
//       // if !$cs.is_empty() { // && $options{mathactive}
//         // $state.assign_mathcode($cs, 0x8000, $options{scope}); }
//       $state.install_definition(Expandable{ cs: coerce_cs( $cs ), paramlist: $paramlist, expansion: $expansion});//, %options), $options{scope});
//       // if $options{locked} {
//       //   $state.assign_value(ToString($cs)+":locked", true, "global")
//       // }
//       }
//     )
//   );

use core::definition::constructor::{Constructor, ConstructorOptions};
#[macro_export]
macro_rules! DefConstructorI(
  ($cs:expr, $paramlist:expr, $replacement:expr, $options: expr, $state:expr) => (
  {
    use $crate::core::definition::constructor::Constructor;
    use $crate::core::package;
    let mode    = $options.mode;
    let bounded = $options.bounded;
    $state.install_definition(Constructor { cs: package::coerce_cs( $cs ),
      paramlist: $paramlist, replacement: $replacement, ..Constructor::default()}, &None);

    //   beforeDigest => flatten(($options{requireMath} ? (sub { requireMath($cs); }) : ()),
    //     ($options{forbidMath} ? (sub { forbidMath($cs); }) : ()),
    //     ($mode ? (sub { $_[0]->beginMode($mode); })
    //       : ($bounded ? (sub { $_[0]->bgroup; }) : ())),
    //     ($options{font} ? (sub { MergeFont(%{ $options{font} }); }) : ()),
    //     $options{beforeDigest}),
    //   afterDigest => flatten($options{afterDigest},
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
    return; 
  }
  );
);

pub fn DefConstructor(proto : String, replacement : String, options : ConstructorOptions, state: &mut State) {
  // check_options("DefConstructor ($proto)", $constructor_options, %options);
  let (cs, paramlist) = parse_prototype(proto);
  DefConstructorI!(cs, paramlist, replacement, options, state);
  return; 
}

pub mod pool;