///**********************************************************************
/// Organized following
///  "LaTeX: A Document Preparation System"
///   by Leslie Lamport
///   2nd edition
/// Addison Wesley, 1994
/// Appendix C. Reference Manual
///**********************************************************************
/// NOTE: This will be loaded after TeX.pool, so it inherits.
///**********************************************************************

use std::sync::Arc;
use std::collections::HashMap;
use regex::Regex;
use rtx_core::state::{Scope, State, ObjectStore};
use rtx_core::token::*;
use rtx_core::tbox::TBox;
use rtx_core::stomach::Stomach;
use rtx_core::whatsit::Whatsit;
use rtx_core::definition::constructor::ConstructorOptions;
use package::*;

lazy_static!{
  static ref OPTS_REGEX : Regex = Regex::new(r",\s*").unwrap();
}

pub fn load_definitions(state: &mut State) {
  LoadPool!("TeX", state);

  // Apparently LaTeX does NOT define \magnification,
  // and babel uses that to determine whether we're runing LaTeX!!!
  // Let('\magnification', '\@undefined');
  //**********************************************************************
  // Basic \documentclass & \documentstyle

  //AssignValue('2.09_COMPATIBILITY'=>0);
  // DefConditionalI('\if@compatibility', undef, sub { LookupValue('2.09_COMPATIBILITY'); });
  // DefMacro('\@compatibilitytrue',  '');
  // DefMacro('\@compatibilityfalse', '');

  // Let('\@currentlabel', '\@empty');

  // Let's try just starting with this set (since we've loaded LaTeX)
  state.assign_value("inPreamble", ObjectStore::Bool(true), None);    // \begin{document} will clear this.


  DefConstructor!("\\documentclass OptionalSemiverbatim SkipSpaces Semiverbatim []",
                  "<?latexml class='#2' ?#1(options='#1')?>",
    ConstructorOptions {
      after_digest: vec![Arc::new(|_stomach: &mut Stomach, whatsit: &mut Whatsit, state: &mut State| -> Vec<Digested> {
        let options: Option<&Digested> = whatsit.get_arg(1);
        let class_opts = match options {
          Some(opts) => OPTS_REGEX.split(&opts.to_string()).map(|s| s.to_string()).collect(),
          None => Vec::new(),
        };
        LoadClass!(whatsit.get_arg(2).unwrap().to_string(),
                   class_opts,
                   vec![T_CS!("\\AtBeginDocument".to_string()), T_CS!("\\warn@unusedclassoptions".to_string())],
                   state);
        Vec::new()
      })],
      ..ConstructorOptions::default()
    },
    state);


  // ======================================================================
  // C.1.2 Environments
  // ======================================================================

  // In LaTeX, \newenvironment{env} defines \env and \endenv.
  // \begin{env} & \end{env} open/close a group, and invoke these.
  // In fact, the \env & \endenv don't have to have been created by
  // \newenvironment; And in fact \endenv doesn't even have to be defined!
  // [it is created by \csname, and equiv to \relax if no previous defn]

  // We need to respect these usages here, but we also want to be able
  // to define environment constructors that `capture' the body so that
  // it can be processed specially, if needed.  These are the magic
  // "\begin{env}", "\end{env}" control sequences created by DefEnvironment.

  state.assign_value("current_environment", ObjectStore::String(String::new()), Some(Scope::Global));
  // DefMacroI!("\@currenvir", "", Arc::new(move |state| {}), state);
  // DefPrimitive("\lx@setcurrenvir{}", sub {
  //     DefMacro("\@currenvir", $_[1]);
  //     state.assign_value(current_environment => ToString($_[1])); });
  // Let("\@currenvline", "\@empty");

  DefMacro!("\\begin{}",
    |gullet, args, state| {
    let ref name = args[0].to_string();
    let begin_name = "\\begin{".to_string()+&name+"}";
    if IsDefined!(&begin_name, state) {
      vec![T_CS!(begin_name)] // Magic cs!
    }
    else {
      let token = T_CS!("\\".to_string() + name);
      if !IsDefinedToken!(&token, state) {
        let undef = "{".to_string() + &name + "}";
        println_stderr!("Error:undefined:{:?}: The environment is not defined.",undef);
        // state.note_status("undefined", undef);
        //   Error("undefined", $undef, $gullet, "The environment " . $undef . " is not defined.");
        // state.install_definition(LaTeXML::Core::Definition::Constructor->new($token, undef,
        //       sub { LaTeXML::Core::Stomach::makeError($_[0], "undefined", $undef); })); }
        //(T_CS!("\begingroup"), Invocation(T_CS!("\lx@setcurrenvir"), $env), $token); } });
      }
      Vec::new()
    }
  },
  state);

  DefMacro!("\\end{}",
  |gullet, args, state| {
  let name = args[0].to_string();
  let mut t = T_CS!("\\end{$name}");
  if IsDefinedToken!(&t, state) {// Magic CS!
    vec![t]
  } else {
    t = T_CS!("\\end$name");
    if IsDefinedToken!(&t, state) {
      vec![t, T_CS!("\\endgroup")]
    } else {
      vec![T_CS!("\\endgroup")]
    }
  }}, state);


  //**********************************************************************
  // C.2. The Structure of the Document
  //**********************************************************************
  //   prepended files (using filecontents environment)
  //   preamble (starting with \documentclass)
  //   \begin{document}
  //    text
  //   \end{document}

  // DefMacro('\AtBeginDocument{}', sub {
  //     AssignValue('@at@begin@document', []) unless LookupValue('@at@begin@document');
  //     PushValue('@at@begin@document', $_[1]->unlist); });
  // DefMacro('\AtEndDocument{}', sub {
  //     AssignValue('@at@end@document', []) unless LookupValue('@at@end@document');
  //     PushValue('@at@end@document', $_[1]->unlist); });

  DefEnvironment!("{document}", |document, whatsit, props, state| {
      //       "<ltx:document xml:id='#id'>#body</ltx:document>",
      let id   = match props.get("id") {
        Some(& ObjectStore::String(ref id)) => id,
        _ => ""
      };
      // let body = props.get("body").unwrap_or(Digested::default());
      // if let Some(docel) = document.findnode("/ltx:document") { // Already (auto) created?
      //   if !id.is_empty() {
      //     document.set_attribute(docel, "xml:id", id);
      //   }
        // document.absorb(body, state);
      // } else {
      //   document.insert_element("ltx:document", body, vec!["xml:id"], vec![id]);
      // }
    },
    ConstructorOptions {
    // before_digest: |stomach, state| { AssignValue!("inPreamble", ObjectStore::Bool(false), state); },
    // after_digest_begin => |stomach, whatsit, state| {
    //   whatsit.set_property("id", Expand!(T_CS!("\thedocument@ID"), state));
    //   if let Some(ops) = LookupValue!("@at@begin@document", state) {
    //     let boxes = Digest!(Tokens!(ops));
    //     whatsit.set_font(LookupValue!("font")); // Start w/ whatever font was selected.
    //     return boxes
    //   } else {
    //     return Vec::new()
    //   }
    // },
    // before_digest_end => |stomach, whatsit, state| {
    //   stomach.get_gullet().flush();
    //   if let Some(ops) = LookupValue!("@at@end@document", state) {
    //     return Digest!(Tokens!(ops));
    //   } else {
    //     return Vec::new();
    //   }
    // },
    mode: "text".to_string(),
    ..ConstructorOptions::default()}, state);

  // ======================================================================
  // C.5.2 Packages
  // ======================================================================
  // We'll prefer to load package.pm, but will try package.sty or
  // package.tex (the latter being unlikely to work, but....)
  // See Stomach.pm for details
  // Ignorable packages ??
  // pre-defined packages??

  // DefMacroI('\@clsextension', undef, 'cls');
  // DefMacroI('\@pkgextension', undef, 'sty');
  // Let('\@currext',  '\@empty');
  // Let('\@currname', '\@empty');

  DefConstructor!("\\usepackage OptionalSemiverbatim Semiverbatim []",
                  "<?latexml package='#2' ?#1(options='#1')?>",
                  ConstructorOptions {
                    before_digest: vec![Arc::new(|_stomach: &mut Stomach, state: &mut State| -> Vec<Digested> {
                      // onlyPreamble('\usepackage');
                      Vec::new()
                    })],
                    after_digest: vec![Arc::new(|_stomach: &mut Stomach, whatsit: &mut Whatsit, state: &mut State| -> Vec<Digested> {
                      let options: Option<&Digested> = whatsit.get_arg(1);
                      let packages: Option<&Digested> = whatsit.get_arg(2);
                      // my @pkgs     = grep { $_ } grep { !/^\s*%/ } split(/,\s*/, ToString($packages));
                      // $options = [($options ? split(/,\s*/, (ToString($options))) : ())];
                      // map { RequirePackage($_, options => $options) } @pkgs;
                      Vec::new()
                    })],
                    ..ConstructorOptions::default()
                  },
                  state);



  // STUBS:
  for ltxtrigger in ["\\newcommand",
                     "\\renewcommand",
                     "\\newenvironment",
                     "\\renewenvironment",
                     "\\NeedsTeXFormat",
                     "\\ProvidesPackage",
                     "\\RequirePackage",
                     "\\ProvidesFile",
                     "\\makeatletter",
                     "\\makeatother",
                     "\\typeout",
                     "\\listfiles"]
                      .into_iter()
                      .map(|s| s.to_string()) {
    DefMacroI!(T_CS!(ltxtrigger),
               None,
               move |_gullet, _args, _state| Vec::new(),
               state);
  }
}
