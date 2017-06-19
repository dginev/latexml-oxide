///**********************************************************************
/// Organized following
///  "`LaTeX`: A Document Preparation System"
///   by Leslie Lamport
///   2nd edition
/// Addison Wesley, 1994
/// Appendix C. Reference Manual
///**********************************************************************
/// NOTE: This will be loaded after `TeX.pool`, so it inherits.
///**********************************************************************

use package::*;

lazy_static!{
  static ref OPTS_REGEX : Regex = Regex::new(r",\s*").unwrap();
}

pub fn load_definitions(state: &mut State) -> Result<()> {
  SetupBindingMacros!(state);
  LoadPool!("TeX");

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
    after_digest => vec!(afterproc!(_stomach, whatsit, state, {
      let options: Option<&Digested> = whatsit.get_arg(1);
      let class_opts = match options {
        Some(opts) => OPTS_REGEX.split(&opts.to_string()).map(|s| s.to_string()).collect(),
        None => Vec::new(),
      };
      try!(load_class(whatsit.get_arg(2).unwrap().to_string(),
                class_opts,
                vec![T_CS!("\\AtBeginDocument".to_string()), T_CS!("\\warn@unusedclassoptions".to_string())],
                state));
    }))
  );


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

  AssignValue!("current_environment", ObjectStore::String(String::new()), Some(Scope::Global));
  // DefMacroI!("\@currenvir", "", Rc::new(move |state| {}), state);
  // DefPrimitive("\lx@setcurrenvir{}", sub {
  //     DefMacro("\@currenvir", $_[1]);
  //     state.assign_value(current_environment => ToString($_[1])); });
  // Let("\@currenvline", "\@empty");

  DefMacro!("\\begin{}",
    |gullet, args, state| {
    let name = &args[0].to_string();
    let begin_name = "\\begin{".to_string()+name+"}";
    if is_defined(&begin_name, state) {
      Ok(vec![T_CS!(begin_name)]) // Magic cs!
    }
    else {
      let token = T_CS!("\\".to_string() + name);
      if !is_defined_token(&token, state) {
        let undef = "{".to_string() + name + "}";
        let category_object = format!("undefined:{:?}", undef);
        error!(target: &category_object,"The environment is not defined.");
        // state.note_status("undefined", undef);
        //   Error("undefined", $undef, $gullet, "The environment " . $undef . " is not defined.");
        // state.install_definition(LaTeXML::Core::Definition::Constructor->new($token, undef,
        //       sub { LaTeXML::Core::Stomach::makeError($_[0], "undefined", $undef); })); }
        //(T_CS!("\begingroup"), Invocation(T_CS!("\lx@setcurrenvir"), $env), $token); } });
      }
      Ok(Vec::new())
    }
  });

  DefMacro!("\\end{}", |gullet, args, state| {
    let name = args[0].to_string();
    let mut t = T_CS!("\\end{".to_string()+&name+"}");
    if is_defined_token(&t, state) {// Magic CS!
    Ok(vec![t])
  } else {
    t = T_CS!("\\end".to_string()+&name);
    if is_defined_token(&t, state) {
      Ok(vec![t, T_CS!("\\endgroup")])
    } else {
      Ok(vec![T_CS!("\\endgroup")])
    }
  }});


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

  DefEnvironmentC!("{document}",
    Some(Rc::new(|document: &mut Document, args: &Vec<Option<Digested>>, props: &HashMap<String, ObjectStore>, state: &mut State| {
      let id = match props.get("id") {
        Some(& ObjectStore::String(ref id)) => id,
        _ => ""
      };
      // TODO: Cloning here ought to be terribly inefficient and should be avoided. How?
      let body = match props.get("body") {
        Some(& ObjectStore::Digested(ref rc)) => (**rc).clone(),
        _ => Digested::Whatsit(Whatsit::default())
      };
      if let Some(mut docel) = document.findnode("/ltx:document", None, state) { // Already (auto) created?
        if !id.is_empty() {
          document.set_attribute(&mut docel, "xml:id", id);
        }
        try!(document.absorb(body, state));
      } else {
        let mut attrib : HashMap<String, String> = HashMap::new();
        attrib.insert("xml:id".to_string(), id.to_string());
        try!(document.insert_element("ltx:document", vec![body], Some(attrib), state));
      }
      Ok(())
    })),
    before_digest => vec!(beforeproc!(_stomach, state, { state.assign_value("inPreamble", ObjectStore::Bool(false), None); })),
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
    before_digest_end => sub!(|stomach, state| {
      stomach.get_gullet_mut().flush(state);
      if let Some(ops) = LookupValue_F!("@at@end@document", state) {
        // TODO:
        // Ok(Digest!(Tokens!(ops)))
        Ok(Vec::new())
      } else {
        Ok(Vec::new())
      }
    }),
    mode => Some("text".to_string())
  );

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
  fn only_preamble(cs: &str, state: &mut State) {
    if !state.lookup_bool("inPreamble") {
      let category_object = format!("unexpected:{:?}", cs);
      error!(target: &category_object, "The current command can only appear in the preamble");
    }
  }

  DefConstructor!("\\usepackage OptionalSemiverbatim Semiverbatim []",
                  "<?latexml package='#2' ?#1(options='#1')?>",
      before_digest => vec!(beforeproc!(_stomach, state, { only_preamble("\\usepackage", state); })),
      after_digest => sub!(|_stomach: &mut Stomach, whatsit: &mut Whatsit, state: &mut State| -> Result<Vec<Digested>> {
        let options: Option<&Digested> = whatsit.get_arg(1);
        let packages: Option<&Digested> = whatsit.get_arg(2);
        let package_list = match packages {
          Some(value) => OPTS_REGEX.split(&value.to_string()).map(|s| s.to_string()).filter(|s| !s.starts_with('%')).collect(),
          None => Vec::new(),
        };
        let options_list = match options {
          Some(opts) => OPTS_REGEX.split(&opts.to_string()).map(|s| s.to_string()).collect(),
          None => Vec::new(),
        };

        for package in package_list {
          try!(require_package(package, RequireOptions {
            options: options_list.clone(),
            ..RequireOptions::default()
          }, state))
        }
        Ok(Vec::new())
      })
  );



  // STUBS:
  for ltxtrigger in ["\\renewcommand",
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
    DefMacroI!(T_CS!(ltxtrigger), None,
      move |_gullet, _args, _state| Ok(Vec::new())
    );
  }

  //**********************************************************************
  // C.7 Mathematical Formulas
  //**********************************************************************

  //======================================================================
  // C.7.1 Math Mode Environments
  //======================================================================

  // TODO: Implement environment modes properly, some work still to go
  // TODO: Re-add ltx: namespace when compiler can parse it
  DefEnvironment!("{math}",
    "<ltx:Math mode=\"inline\"><ltx:XMath>#body</ltx:XMath></ltx:Math>",
    mode => Some("inline_math".to_string())
  );
  // My first inclination is to Lock {math}, but it is surprisingly common to redefine it in silly ways... So...?


  //**********************************************************************
  // C.8 Definitions, Numbering and Programming
  //**********************************************************************

  //======================================================================
  // C.8.1 Defining Commands
  //======================================================================

  // DefMacro('\@tabacckludge {}', '\csname\string#1\endcsname');

  DefPrimitiveI!("\\newcommand OptionalMatch:* DefToken [Number][]{}", |stomach, args, state| {
      // my ($stomach, $star, $cs, $nargs, $opt, $body) = @_;
      let star = &args[0];
      let cs = &args[1].tokens[0];
      let nargs = &args[2];
      let opt = &args[3];
      let body = args[4].clone().unlist();

      // if (!isDefinable(cs)) {
      //   Info('ignore', $cs, $stomach,
      //     "Ignoring redefinition (\\newcommand) of '" . Stringify($cs) . "'")
      //     unless LookupValue(ToString($cs) . ':locked');
      //   return; }

      // TODO: convertLaTeXArgs($nargs, $opt)
      let body_closure = move |gullet:&mut Gullet, args:Vec<Tokens>, state:&mut State|{ Ok(body.clone()) };
      DefMacroI_F!(cs.clone(), None, body_closure, state);
      Ok(Vec::new())
  });

  //======================================================================
  // C.8.4 Numbering
  //======================================================================
  // For LaTeX documents, We want id's on para, as well as sectional units.
  // However, para get created implicitly on Document construction, rather than
  // explicitly during digestion (via a whatsit), we can't use the usual LaTeX counter mechanism.
  Tag!("ltx:para", after_open => tagsub!(document, node, state, {
    generate_id(document, node, "p", state);
  }));

  Ok(())
}
