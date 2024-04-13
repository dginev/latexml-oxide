use crate::package::*;

LoadDefinitions!({

DefKeyVal!("xargs", "usedefault", "");

// DefParameterType!(XArgsOptional, sub[inner, extra] {
//     // my ($gullet, $default, $usedefault, $inner) = @_;
//     // my $value = $gullet->readOptional;
//     // if (($usedefault && ToString($value) eq ToString($usedefault)) ||
//     //   (!defined $usedefault && ToString($value) eq '')) {
//     //   $value = $default; }
//     // $value; 
//   },
//   optional => true);

// Macros

DefPrimitive!("\\CheckCommandx OptionalMatch:* DefToken [] OptionalKeyVals:xargs {}", None);

DefPrimitive!("\\newcommandx OptionalMatch:* DefToken [] OptionalKeyVals:xargs {}", sub[(star,cs,nargs_opt,defaults,body)] {
  if !is_definable(&cs) {
    Info!("ignore", cs, "Ignoring redefinition (\\newcommandx) of '{}'",cs);
  } else {
    let scope = if get_xargs_is_global(star, defaults.as_ref()) { Some(Scope::Global) } else {None};
    let nargs = if let Some(nargs_tks) = nargs_opt {
      nargs_tks.to_string().parse::<usize>()?
    } else {0};
    DefMacro!(cs, convert_xargs_args(nargs, defaults.as_ref())?, body, scope => scope);
  }
});

// DefPrimitive('\renewcommandx OptionalMatch:* DefToken [] OptionalKeyVals:xargs {}', sub {
//     my ($stomach, $star, $cs, $nargs, $defaults, $body) = @_;
//     DefMacroI($cs, convertXArgsArgs($nargs, $defaults), $body, (getXArgsIsGlobal($star, $defaults) ? (scope => 'global') : ())); });

// DefPrimitive('\providecommandx OptionalMatch:* DefToken [] OptionalKeyVals:xargs {}', sub {
//     my ($stomach, $star, $cs, $nargs, $defaults, $body) = @_;
//     return unless isDefinable($cs);
//     DefMacroI($cs, convertXArgsArgs($nargs, $defaults), $body, (getXArgsIsGlobal($star, $defaults) ? (scope => 'global') : ())); });

// DefPrimitive('\DeclareRobustCommandx OptionalMatch:* DefToken [] OptionalKeyVals:xargs {}', sub {
//     my ($stomach, $star, $cs, $nargs, $defaults, $body) = @_;
//     my @scope    = (getXArgsIsGlobal($star, $defaults) ? (scope => 'global') : ());
//     my $mungedcs = T_CS($cs->getString . ' ');
//     DefMacroI($mungedcs, convertLaTeXArgs($nargs, $defaults), $body, @scope);
//     DefMacroI($cs,       undef, Tokens(T_CS('\protect'), $mungedcs), @scope); });

// DefPrimitive('\newenvironmentx OptionalMatch:* {} [] OptionalKeyVals:xargs {}{}', sub {
//     my ($stomach, $star, $cs, $nargs, $defaults, $preamble, $postamble) = @_;
//     if (LookupDefinition(T_CS("\\$cs"))) {
//       Info('ignore', $cs, $stomach,
//         "Ignoring redefinition (\\newenvironmentx) of Environment '$cs'");
//       return; }
//     $cs = ToString($cs);
//     DefMacroI(T_CS("\\$cs"), convertXArgsArgs($nargs, $defaults), $preamble, (getXArgsIsGlobal($star, $defaults) ? (scope => 'global') : ()));
//     DefMacroI(T_CS("\\end$cs"), undef, $postamble, (getXArgsIsGlobal($star, $defaults) ? (scope => 'global') : ())); });

// DefPrimitive('\renewenvironmentx OptionalMatch:* {} [] OptionalKeyVals:xargs {}{}', sub {
//     my ($stomach, $star, $cs, $nargs, $defaults, $preamble, $postamble) = @_;
//     $cs = ToString($cs);
//     DefMacroI(T_CS("\\$cs"), convertXArgsArgs($nargs, $defaults), $preamble, (getXArgsIsGlobal($star, $defaults) ? (scope => 'global') : ()));
//     DefMacroI(T_CS("\\end$cs"), undef, $postamble, (getXArgsIsGlobal($star, $defaults) ? (scope => 'global') : ())); });
});

// Utils

// generate paramlist
fn convert_xargs_args(nargs:usize, keyval:Option<&KeyVals>) -> Result<Option<Parameters>> {
  let mut paramlist = Vec::new();
  for i in 1 .. nargs {
    if let Some(val) = keyval.map(|kv| kv.get_value(&i.to_string())).unwrap_or(None) {
//       my $usedef = ($keyval) ? $keyval->getValue('usedefault') : undef;
//       if (defined $usedef) {
//         push(@paramlist, LaTeXML::Core::Parameter->new(
//             'XArgsOptional',
//             "XArgsOptional:" . $val->toString() . "|" . ToString($usedef),
//             extra => [$val, $usedef]
//         )); }
//       else {
//         push(@paramlist, LaTeXML::Core::Parameter->new(
//             'Optional',
//             "Optional:" . $val->toString(),
//             extra => [$val, $usedef]
//         )); } 
    } else {
       paramlist.push(Parameter::new("Plain", "{}")?); 
    }
  }
  Ok(
    if paramlist.is_empty() { None } else {
      Some(Parameters::new(paramlist))
    })
}

/// generate command prefix (\global, \long, ...; but not \outer)
fn get_xargs_is_global(star:Option<Tokens>, keyval_opt:Option<&KeyVals>) -> bool {
  let mut prefix = String::new();
  if star.is_none() {
    // defaults to \long for unstarred form
    prefix = String::from("\\long"); 
  }
  if let Some(keyval) = keyval_opt {
    if let Some(p) = keyval.get_value("addprefix") {
      prefix.push_str(&p.to_string());
    }
  }
  // true if global in prefix
  prefix.contains("global")
}