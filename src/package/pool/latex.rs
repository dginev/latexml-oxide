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
  state.assign_value("inPreamble", ObjectStore::Bool(true), None); // \begin{document} will clear this.

  DefConstructor!("\\documentclass OptionalSemiverbatim SkipSpaces Semiverbatim []",
                  "<?latexml class='#2' ?#1(options='#1')?>",
    after_digest => vec!(afterproc!(_stomach, whatsit, state, {
      let options: Option<&Digested> = whatsit.get_arg(1);
      let class_opts = match options {
        Some(opts) => OPTS_REGEX.split(&opts.to_string()).map(|s| s.to_string()).collect(),
        None => Vec::new(),
      };
      load_class(whatsit.get_arg(2).unwrap().to_string(),
                class_opts,
                Tokens!(T_CS!("\\AtBeginDocument".to_string()), T_CS!("\\warn@unusedclassoptions".to_string())),
                state)?;
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

  AssignValue!(
    "current_environment",
    ObjectStore::String(String::new()),
    Some(Scope::Global)
  );
  // DefMacroI!("\@currenvir", "", Rc::new(move |state| {}), state);
  // DefPrimitive("\lx@setcurrenvir{}", sub {
  //     DefMacro("\@currenvir", $_[1]);
  //     state.assign_value(current_environment => ToString($_[1])); });
  // Let("\@currenvline", "\@empty");

  DefMacro!("\\begin{}", gullet, args, state, {
    let name = &args[0].to_string();
    let begin_name = "\\begin{".to_string() + name + "}";
    if is_defined(&begin_name, state) {
      Ok(Tokens!(T_CS!(begin_name))) // Magic cs!
    } else {
      let token = T_CS!("\\".to_string() + name);
      if !is_defined_token(&token, state) {
        let undef = "{".to_string() + name + "}";
        let category_object = s!("undefined:{:?}", undef);
        error!(target: &category_object, "The environment is not defined.");
        // state.note_status("undefined", undef);
        //   Error("undefined", $undef, $gullet, "The environment " . $undef . " is not defined.");
        // state.install_definition(LaTeXML::Core::Definition::Constructor->new($token, undef,
        //       sub { LaTeXML::Core::Stomach::makeError($_[0], "undefined", $undef); })); }
        //(T_CS!("\begingroup"), Invocation(T_CS!("\lx@setcurrenvir"), $env), $token); } });
      }
      Ok(Tokens!())
    }
  });

  DefMacro!("\\end{}", gullet, args, state, {
    let name: String = args[0].to_string();
    let mut t = T_CS!("\\end{".to_string() + &name + "}");
    if is_defined_token(&t, state) {
      // Magic CS!
      Ok(Tokens!(t))
    } else {
      t = T_CS!("\\end".to_string() + &name);
      if is_defined_token(&t, state) {
        Ok(Tokens!(t, T_CS!("\\endgroup")))
      } else {
        Ok(Tokens!(T_CS!("\\endgroup")))
      }
    }
  });

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
        document.absorb(body, state)?;
      } else {
        let mut attrib : HashMap<String, String> = HashMap::new();
        attrib.insert("xml:id".to_string(), id.to_string());
        document.insert_element("ltx:document", vec![body], Some(attrib), state)?;
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
      if let Some(ops) = LookupValue!("@at@end@document", state) {
        // TODO:
        // Ok(Digest!(Tokens!(ops)))
        Ok(Vec::new())
      } else {
        Ok(Vec::new())
      }
    }),
    mode => Some("text".to_string())
  );

  //**********************************************************************
  // C.4 Sectioning and Table of Contents
  //**********************************************************************

  //======================================================================
  // C.4.1 Sectioning Commands.
  //======================================================================
  // Note that LaTeX allows fairly arbitrary stuff in \the<ctr>, although
  // it can get you in trouble.  However, in almost all cases, the result
  // is plain text.  So, I'm putting refnum as an attribute, where I like it!
  // You want something else? Redefine!

  // Also, we're adding an id to each, that is parallel to the refnum, but
  // valid as an ID.  You can tune the representation by defining, eg. \thesection@ID

  // A little more messy than seems necessary:
  //  We don't know whether to step the counter and update \@currentlabel until we see the '*',
  // but we have to know it before we digest the title, since \label can be there!

  // These are defined in terms of \@startsection so that
  // casual user redefinitions work, too.
  DefMacro!("\\chapter", "\\@startsection{chapter}{0}{}{}{}{}"); // TODO: locked => true);
  DefMacro!("\\part", "\\@startsection{part}{-1}{}{}{}{}"); // not locked since sometimes redefined as partition?
  DefMacro!("\\section", "\\@startsection{section}{1}{}{}{}{}"); // TODO: locked => true);
  DefMacro!("\\subsection", "\\@startsection{subsection}{2}{}{}{}{}"); // TODO: locked => true);
  DefMacro!(
    "\\subsubsection",
    "\\@startsection{subsubsection}{3}{}{}{}{}"
  ); // TODO: locked => true);
  DefMacro!("\\paragraph", "\\@startsection{paragraph}{4}{}{}{}{}"); // TODO: locked => true);
  DefMacro!("\\subparagraph", "\\@startsection{subparagraph}{5}{}{}{}{}"); // TODO: locked => true);
  for tag in [
    "part",
    "chapter",
    "section",
    "subsection",
    "subsubsection",
    "paragraph",
    "subparagraph",
  ].iter()
  {
    Tag!(&s!("ltx:{:?}",tag), auto_close => true);
  }

  DefMacro!("\\secdef {}{} OptionalMatch:*", gullet, args, state, {
    if args.len() == 3 {
      Ok(args[1].clone()) // can't move out without clone, how to circumvent?
    } else {
      Ok(args[2].clone())
    } // ($_[3] ? ($_[2]) : ($_[1])); });
  });

  DefMacro!("\\@startsection@hook", "");

  NewCounter!("secnumdepth");
  SetCounter!("secnumdepth", Number!(3), None);
  DefMacro!(
    "\\@startsection{}{}{}{}{}{} OptionalMatch:*",
    gullet,
    args,
    state,
    {
      let type_tokens = args[0].clone();
      let stype = type_tokens.to_string();
      let mut ctr = state.lookup_string(&s!("counter_for_{}", stype));
      if ctr.is_empty() {
        ctr = stype
      };
      let level = args[1].to_string();
      let flag = args[6].to_string();
      if !flag.is_empty() { // No number, not in TOC
        //|| (!level.is_empty() && (level > CounterValue!("secnumdepth").value_of())) {
        // RefStepID!(ctr);
        let mut tokens: Vec<Token> = vec![
          T_CS!("\\@startsection@hook"),
          T_CS!("\\@@unnumbered@section"),
          T_BEGIN!(),
        ];
        tokens.append(&mut type_tokens.unlist());
        tokens.push(T_END!());
        tokens.push(T_BEGIN!());
        tokens.push(T_END!());
        Ok(Tokens::new(tokens))
      } else if !level.is_empty() && (level.parse::<i32>().unwrap() > CounterValue!("secnumdepth", state).value_of())
                || LookupBool!("no_number_sections", state) {
        // No number, but in TOC
        let mut tokens: Vec<Token> = vec![ 
          T_CS!("\\@startsection@hook"), T_CS!("\\@@unnumbered@section"), T_BEGIN!()
        ];
        tokens.append(&mut type_tokens.unlist());
        tokens.push(T_END!());
        tokens.push(T_BEGIN!());
        tokens.push(T_OTHER!("toc"));
        tokens.push(T_END!());
        Ok(Tokens::new(tokens))
      } else { // Number and in TOC
        let mut tokens : Vec<Token> = vec![ 
          T_CS!("\\@startsection@hook"), T_CS!("\\@@numbered@section"),
          T_BEGIN!()
        ];
        tokens.append(&mut type_tokens.unlist());
        tokens.push(T_END!());
        tokens.push(T_BEGIN!());
        tokens.push(T_OTHER!("toc"));
        tokens.push(T_END!());
        Ok(Tokens::new(tokens))
      }
    }
  );

DefConstructor!("\\@@numbered@section{} Undigested OptionalUndigested Undigested",
    document,
    args,
    props,
    state,
    {
      // TODO: This bizarre argument API interaction needs to be simplified down to Perl's
      // intuitive level of:       let (x,y,z, ...) = @args;
      let (stype, inlist, toctitle, title) = (
        args[0].clone().unwrap_or_default().to_string(),
        args[1].clone().unwrap_or_default().to_string(),
        args[2].clone().unwrap_or_default().to_string(),
        args[3].clone().unwrap_or_default().to_string(),
      );
      let id = match props.get("id") {
        Some(& ObjectStore::String(ref id)) => id,
        _ => ""
      };
      let clean_id = id; // TODO: CleanID($id);
      document.open_element(
        &s!("ltx:{}", stype),
        Some(string_map!("xml:id" => clean_id, "inlist" => inlist)),
        None,
        state,
      )?;
      // TODO: Another instance where the immutability of props causes endless cloning
      //       which is slow and wasteful.
      //       The big problem is that for props to be mutable, the entire parent whatsit needs to be mutable,
      //       and Rust hits a mutability conflict between the parent, and the "args" and "props" children
      //       ... will come back here after performance becomes an issue again
      if let Some(ObjectStore::Digested(tags)) = props.get("tags") {
        document.absorb((**tags).clone(), state)?; 
      }
      let title_prop = props.get("title");
      let title_digested = match title_prop {
        Some(ObjectStore::VecDigested(vd)) => vd.clone(),
        Some(ObjectStore::Digested(d)) => vec![(**d).clone()],
        _ => Vec::new()
      };
      document.insert_element("ltx:title", title_digested, None, state)?;

      let toctitle_prop = props.get("toctitle");
      let toctitle_digested = match toctitle_prop {
        Some(ObjectStore::VecDigested(vd)) => vd.clone(),
        Some(ObjectStore::Digested(d)) => vec![(**d).clone()],
        _ => Vec::new()
      };
      if !toctitle_digested.is_empty() {
        document.insert_element("ltx:toctitle", toctitle_digested, None, state)?;
      }
    });
//   properties => sub {
//     my ($stomach, $type, $inlist, $toctitle, $title) = @_;
//     my %props     = RefStepCounter(ToString($type));
//     my $xtitle    = Digest(Invocation(T_CS('\lx@format@title@@'), $type, $title));
//     my $xtoctitle = Digest(Invocation(T_CS('\lx@format@toctitle@@'), $type, $toctitle || $title));
//     $props{title}    = $xtitle;
//     $props{toctitle} = $xtoctitle
//       if $xtoctitle && $xtoctitle->unlist && (ToString($xtoctitle) ne ToString($xtitle));
//     return %props; });

// # No tags, at all? Consider...
// DefConstructor('\@@unnumbered@section{} Undigested OptionalUndigested Undigested', sub {
//     my ($document, $type, $inlist, $toctitle, $title, %props) = @_;
//     my $id = $props{id};
//     $document->openElement("ltx:" . ToString($type),
//       'xml:id' => CleanID($id),
//       inlist   => ToString($inlist));
//     $document->insertElement('ltx:title', $props{title});
//     $document->insertElement('ltx:toctitle', $props{toctitle}) if $props{toctitle}; },
//   properties => sub {
//     my ($stomach, $type, $inlist, $toctitle, $title) = @_;
//     my %props = RefStepID(ToString($type));
//     $props{title} = Digest(T_CS('\@hidden@bgroup'), $title, T_CS('\@hidden@egroup'));
//     $props{toctitle} = $toctitle
//       && Digest(T_CS('\@hidden@bgroup'), $toctitle, T_CS('\@hidden@egroup'));
//     return %props; });



  //----------------------------------------------------------------------
  // The following macros provide a few layers of customization
  // in particular for supporting localization for different languages.
  //----------------------------------------------------------------------
  // \format@title@{type}{title}
  // Format a title (or caption) appropriately for type.
  // This is usually somewhat verbose, but establishes the context that this is a Chapter, or
  // Figure, or whatever invokes \format@title@type{title} if that macro is defined, else
  // composes \lx@fnum@@{type} title. Define \format@title@type{title} if the default is not
  // appropriate.

  // TODO:
  // DefMacro!("\\format@title@{}{}",
  // "{\\@ifundefined{format@title@#1}{\\@@compose@title{\\lx@fnum@@{#1}}{#2}}{\\csname
  // format@title@#1\\endcsname{#2}}}");

  // \format@toctitle@{type}{toctitle}
  // Format a toctitle (or toccaption) appropriately for type.
  // This is usually somewhat concise, and the context implies that this is a Chapter, Figure or
  // whatever invokes \format@toctitle@type{title} if that macro is defined, else composes
  // \lx@fnum@toc@@{type} title Define \format@toctitle@type{title} if the default is not
  // appropriate.

  // TODO:
  // DefMacro!("\\format@toctitle@{}{}",
  // "{\\@ifundefined{format@toctitle@#1}{\\@@compose@title{\\lx@fnum@toc@@{#1}}{#2}}{\\csname
  // format@toctitle@#1\\endcsname{#2}}}"); DefMacro!("\\@@compose@title{}{}", "\\@tag[][
  // ]{#1}#2"); DefConstructor!("\\@tag[][]{}", "?#3(<ltx:tag open='#1'
  // close='#2'>#3</ltx:tag>)()");

  //// NOTE that a 3rd form seems desirable: an concise form that cannot rely on context for the
  //// type. This would be useful for the titles in links; thus can be plain (unicode) text.
  //// However, I hate setting up even more machinery & options and dragging yet another form
  //// around....
  // \@@section{type}{id}{refnum}{formattedrefnum}{toctitle}{title}
  DefConstructor!(
    "\\@@section{}{}{}{}{}{}",
    document,
    args,
    props,
    inner_state,
    {
      // TODO: This bizarre argument API interaction needs to be simplified down to Perl's
      // intuitive level of:       let (x,y,z, ...) = @args;
      let (stype, id, refnum, mut frefnum, toctitle, title) = (
        args[0].clone().unwrap().to_string(),
        args[1].clone().unwrap().to_string(),
        args[2].clone().unwrap().to_string(),
        args[3].clone().unwrap().to_string(),
        args[4].clone().unwrap(),
        args[5].clone().unwrap(),
      );

      if frefnum == refnum {
        frefnum = String::new();
      }

      let clean_id = id; // TODO: CleanID($id);
      let has_toctitle =
        !toctitle.to_string().is_empty() && (toctitle.to_string() != title.to_string());
      document.open_element(
        &s!("ltx:{}", stype),
        Some(string_map!("xml:id" => clean_id, "refnum" => refnum, "frefnum" => frefnum)),
        None,
        inner_state,
      )?;
      document.insert_element("ltx:title", vec![title], None, inner_state)?;
      if has_toctitle {
        document.insert_element("ltx:toctitle", vec![toctitle], None, inner_state)?;
      }
    }
  );

  // Not sure if this is best, but if no explicit \section'ing...
  //### Tag('ltx:section',autoOpen=>1);

  //======================================================================
  // C.4.2 The Appendix
  //======================================================================
  // Handled in article,report or book.
  DefMacro!("\\appendixname", "Appendix");
  DefMacro!("\\appendixesname", "Appendixes");

  // ======================================================================
  // C.5.2 Packages
  // ======================================================================
  // We'll prefer to load package.pm, but will try package.sty or
  // package.tex (the latter being unlikely to work, but....)
  // See Stomach.pm for details
  // Ignorable packages ??
  // pre-defined packages??

  DefMacro!("\\@clsextension", "cls");
  DefMacro!("\\@pkgextension", "sty");
  Let!("\\@currext", "\\@empty");
  Let!("\\@currname", "\\@empty");

  fn only_preamble(cs: &str, state: &mut State) {
    if !state.lookup_bool("inPreamble") {
      let category_object = s!("unexpected:{:?}", cs);
      error!(
        target: &category_object,
        "The current command can only appear in the preamble"
      );
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
          require_package(package, RequireOptions {
            options: options_list.clone(),
            ..RequireOptions::default()
          }, state)?
        }
        Ok(Vec::new())
      })
  );

  // STUBS:
  for ltxtrigger in [
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
    "\\listfiles",
  ].into_iter()
    .map(|s| s.to_string())
  {
    DefMacroI!(T_CS!(ltxtrigger), None, move |_gullet, _args, _state| Ok(
      Tokens!()
    ));
  }

  //======================================================================
  // C.5.4 The Title Page and Abstract
  //======================================================================

  // See frontmatter support in TeX.ltxml
  DefMacro!("\\title{}", "\\@add@frontmatter{ltx:title}{#1}");

  //======================================================================
  // C.6.2 List-Making environments
  //======================================================================
  Tag!("ltx:item",        auto_close => true, auto_open => true);
  Tag!("ltx:inline-item", auto_close => true, auto_open => true);

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
  // My first inclination is to Lock {math}, but it is surprisingly common to redefine it in silly
  // ways... So...?

  //**********************************************************************
  // C.8 Definitions, Numbering and Programming
  //**********************************************************************

  //======================================================================
  // C.8.1 Defining Commands
  //======================================================================

  // DefMacro('\@tabacckludge {}', '\csname\string#1\endcsname');

  DefPrimitiveI!(
    "\\newcommand OptionalMatch:* DefToken [Number][]{}",
    primitiveproc!(stomach, args, state, {
      // my ($stomach, $star, $cs, $nargs, $opt, $body) = @_;
      let star = &args[0];
      let cs = &args[1].tokens[0];
      let nargs = &args[2];
      let opt = &args[3];
      let body = args[4].clone();

      // if (!isDefinable(cs)) {
      //   Info('ignore', $cs, $stomach,
      //     "Ignoring redefinition (\\newcommand) of '" . Stringify($cs) . "'")
      //     unless LookupValue(ToString($cs) . ':locked');
      //   return; }

      // TODO: convertLaTeXArgs($nargs, $opt)
      let body_closure =
        move |gullet: &mut Gullet, args: Vec<Tokens>, state: &mut State| Ok(body.clone());
      DefMacroI!(cs.clone(), None, body_closure, state);
    })
  );

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
