use std::sync::Arc;
use regex::Regex;
use state::State;
use core::package::*;
use core::token::*;
use core::tbox::TBox;
use core::stomach::Stomach;
use core::whatsit::Whatsit;
use core::definition::constructor::ConstructorOptions;

pub fn load_definitions(state : &mut State) {
  println!("If you are seeing this, someone invoked latex::load_definitions !!! ");

  DefConstructor("\\documentclass OptionalSemiverbatim SkipSpaces Semiverbatim []".to_string(),
    "<?latexml class='#2' ?#1(options='#1')?>".to_string(), ConstructorOptions{
      after_digest: Some(Arc::new(Box::new(
        |_stomach : &mut Stomach, whatsit : &mut Whatsit, state : &mut State| {
          let options : Option<&TBox> = whatsit.get_arg(1);
          let opts_regex = Regex::new(r",\s*").unwrap();
          let class_opts = match options {
            Some(opts) => opts_regex.split(&opts.to_string()).map(|s| s.to_string()).collect(),
            None => Vec::new()
          };
          load_class(state, whatsit.get_arg(2).unwrap().to_string(),
            class_opts,
            vec![T_CS!("\\AtBeginDocument".to_string()), T_CS!("\\warn@unusedclassoptions".to_string())]);
          return; 
        }))),
      ..ConstructorOptions::default() }, state );


  //======================================================================
  // C.1.2 Environments
  //======================================================================

  // In LaTeX, \newenvironment{env} defines \env and \endenv.
  // \begin{env} & \end{env} open/close a group, and invoke these.
  // In fact, the \env & \endenv don't have to have been created by
  // \newenvironment; And in fact \endenv doesn't even have to be defined!
  // [it is created by \csname, and equiv to \relax if no previous defn]

  // We need to respect these usages here, but we also want to be able
  // to define environment constructors that `capture' the body so that
  // it can be processed specially, if needed.  These are the magic
  // "\begin{env}", "\end{env}" control sequences created by DefEnvironment.

  // TODO:
  // state.assign_value(current_environment, String::new(), Scope::Global);
  // DefMacroI!("\@currenvir", "", Arc::new(Box::new( move |state| {})), state);
  // DefPrimitive("\lx@setcurrenvir{}", sub {
  //     DefMacro("\@currenvir", $_[1]);
  //     state.assign_value(current_environment => ToString($_[1])); });
  // Let("\@currenvline", "\@empty");

  // TODO:
  DefMacro!("\\begin{}".to_string(), |_gullet, _args, _state| {
      // let env = args.get_arg(1);
      // let name = match env {
      //   Some(e) => e.to_string(),
      //   None => String::new()
      // };

      // if (IsDefined("\\begin{$name}")) {
      //   T_CS!("\\begin{$name}"); }    // Magic cs!
      // else {
        // let token = T_CS!("\\".to_string() + name);
        // if (!IsDefined($token)) {
        //   my $undef = "{" . $name . "}";
        //   $STATE->noteStatus(undefined => $undef);
        //   Error("undefined", $undef, $gullet, "The environment " . $undef . " is not defined.");
        //   $STATE->installDefinition(LaTeXML::Core::Definition::Constructor->new($token, undef,
        //       sub { LaTeXML::Core::Stomach::makeError($_[0], "undefined", $undef); })); }
        // (T_CS!("\begingroup"), Invocation(T_CS!("\lx@setcurrenvir"), $env), $token); } });

    Vec::new()
  } ,state);

  DefMacro!("\\end{}".to_string(), |_gullet, _args, _state| {
      // let env = args.get_arg(1);
      // my $name = $env && ToString($env);
      // my $t;
      // if (IsDefined($t = T_CS!("\\end{$name}"))) { $t; }                         // Magic CS!
      // elsif (IsDefined($t = T_CS!("\\end$name"))) { ($t, T_CS!("\endgroup")); }
      // else { (T_CS!("\endgroup")); } });

    Vec::new()
  }, state);
}