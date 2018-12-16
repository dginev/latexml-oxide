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
use crate::package::*;

lazy_static! {
  static ref OPTS_REGEX: Regex = Regex::new(r",\s*").unwrap();
}

pub fn load_definitions(mut state: &mut State) -> Result<()> {
  SetupBindingMacros!(state);
  //**********************************************************************
  // Organized following
  //  "LaTeX: A Document Preparation System"
  //   by Leslie Lamport
  //   2nd edition
  // Addison Wesley, 1994
  // Appendix C. Reference Manual
  //**********************************************************************
  // NOTE: This will be loaded after TeX.pool.ltxml, so it inherits.
  //**********************************************************************

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
  state.assign_value("inPreamble", true, None); // \begin{document} will clear this.

  DefConstructor!("\\documentclass OptionalSemiverbatim SkipSpaces Semiverbatim []",
                  "<?latexml class='#2' ?#1(options='#1')?>",
    after_digest => vec!(afterproc!(_stomach, whatsit, state, {
      let options: Option<&Digested> = whatsit.get_arg(1);
      let class_opts = match options {
        Some(opts) => OPTS_REGEX.split(&opts.to_string()).map(|s| s.to_string()).collect(),
        None => Vec::new(),
      };
      load_class(&(whatsit.get_arg(2).unwrap().to_string()),
                class_opts,
                Tokens!(T_CS!("\\AtBeginDocument"), T_CS!("\\warn@unusedclassoptions")),
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

  AssignValue!("current_environment", String::new(), Some(Scope::Global));
  // DefMacroI!("\@currenvir", "", Rc::new(move |state| {}), state);
  // DefPrimitive("\lx@setcurrenvir{}", sub {
  //     DefMacro("\@currenvir", $_[1]);
  //     state.assign_value(current_environment => ToString($_[1])); });
  // Let("\@currenvline", "\@empty");

  DefMacro!("\\begin{}", sub [gullet, args, state] {
    unpack!(args => name);
    let begin_name = s!("\\begin{{{}}}", name);
    if is_defined(&begin_name, state) {
      Ok(Tokens!(T_CS!(begin_name))) // Magic cs!
    } else {
      let token = T_CS!(s!("\\{}", name));
      if !is_defined_token(&token, state) {
        let undef = s!("{{{}}}", name); // this creates {name} , {{ and }} are escapes in Rust's format!
        let category_object = s!("undefined:{}", undef);
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

  DefMacro!("\\end{}", sub[gullet, args, state]{
    let name: String = args[0].to_string();
    let mut t = T_CS!(s!("\\end{{{}}}", name));
    if is_defined_token(&t, state) {
      // Magic CS!
      Ok(Tokens!(t))
    } else {
      t = T_CS!(s!("\\end{}", name));
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
    Some(Rc::new(|document: &mut Document, args: &Vec<Option<Digested>>, props: &HashMap<String, Stored>, state: &mut State| {
      let id = prop_str!(props,"id");
      let body = prop_whatsit!(props,"body");
      if let Some(mut docel) = document.findnode("/ltx:document", None, state) { // Already (auto) created?
        if !id.is_empty() {
          document.set_attribute(&mut docel, "xml:id", id)?;
        }
        document.absorb(body, state)?;
      } else {
        let attrib = string_map!("xml:id" => id);
        document.insert_element("ltx:document", vec![body], Some(attrib), state)?;
      }
      Ok(())
    })),
    before_digest => vec!(beforeproc!(_stomach, state, { state.assign_value("inPreamble", false, None); })),
    // after_digest_begin => vec![|stomach, whatsit, state| {
    //   whatsit.set_property("id", Expand!(T_CS!("\\thedocument@ID"), state));
    //   if let Some(ops) = LookupValue!("@at@begin@document", state) {
    //     let boxes = Digest!(ops, stomach);
    //     whatsit.set_font(LookupValue!("font")); // Start w/ whatever font was selected.
    //     return boxes
    //   } else {
    //     return Vec::new()
    //   }
    // }],
    before_digest_end => sub!(|stomach, state| {
      stomach.get_gullet_mut().flush(state);
      if let Some(Stored::VecToken(ops)) = RemoveValue!("@at@end@document", state) {
        Ok(vec![stomach.digest(Tokens::new(ops.to_vec()), state)?]) // TODO: Can we improve to the regular Digest!(ops) syntax?
      } else {
        Ok(Vec::new())
      }
    }),
    mode => Some(s!("text"))
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
  for tag in &[
    "part",
    "chapter",
    "section",
    "subsection",
    "subsubsection",
    "paragraph",
    "subparagraph",
  ] {
    Tag!(&s!("ltx:{}",tag), auto_close => true);
  }

  DefMacro!("\\secdef {}{} OptionalMatch:*", sub[gullet, args, state] {
    unpack!(args=>token1, token2);
    if args.len() == 3 {
      Ok(token2) // can't move out without clone, how to circumvent?
    } else {
      Ok(token1)
    }
  });

  DefMacro!("\\@startsection@hook", "");

  NewCounter!("secnumdepth");
  SetCounter!("secnumdepth", Number!(3), None);
  DefMacro!(
    "\\@startsection{}{}{}{}{}{} OptionalMatch:*", sub[gullet,args,state] {
      unpack!(args => type_tokens, level_arg, ignore3, ignore4, ignore5, ignore6, flag);

      let stype = type_tokens.to_string();
      let level = level_arg.to_string();

      // Dead code in master?
      // let mut ctr = state.lookup_string(&s!("counter_for_{}", stype));
      // if ctr.is_empty() {
      //   ctr = stype
      // };
      let mut tokens: Vec<Token>;
      if !flag.is_empty() {
        // No number, not in TOC
        tokens = vec![
          T_CS!("\\@startsection@hook"),
          T_CS!("\\@@unnumbered@section"),
          T_BEGIN!(),
        ];
        tokens.append(&mut type_tokens.unlist());
        tokens.append(&mut vec![T_END!(), T_BEGIN!(), T_END!()]);
      } else if !level.is_empty()
        && (level.parse::<i32>().unwrap() > CounterValue!("secnumdepth", state).value_of())
        || LookupBool!("no_number_sections", state)
      {
        // No number, but in TOC
        tokens = vec![
          T_CS!("\\@startsection@hook"),
          T_CS!("\\@@unnumbered@section"),
          T_BEGIN!(),
        ];
        tokens.append(&mut type_tokens.unlist());
        tokens.append(&mut vec![T_END!(), T_BEGIN!(), T_OTHER!("toc"), T_END!()]);
      } else {
        // Number and in TOC
        tokens = vec![
          T_CS!("\\@startsection@hook"),
          T_CS!("\\@@numbered@section"),
          T_BEGIN!(),
        ];
        tokens.append(&mut type_tokens.unlist());
        tokens.append(&mut vec![T_END!(), T_BEGIN!(), T_OTHER!("toc"), T_END!()]);
      }
      Ok(Tokens::new(tokens))
    }
  );

  DefConstructor!(
     "\\@@numbered@section{} Undigested OptionalUndigested Undigested",
      sub[document, args, props, state] {
       // TODO: This bizarre argument API interaction needs to be simplified down to Perl's
       // intuitive level of:       let (x,y,z, ...) = @args;
       unpack_to_string!(args => stype, inlist, toctitle, title);
       let clean_id = prop_str!(props,"id"); // TODO: CleanID($id);
       document.open_element(&s!("ltx:{}", stype),
         Some(string_map!("xml:id" => clean_id, "inlist" => inlist)),
         None,
         state,
       )?;
       // TODO: Another instance where the immutability of props causes endless cloning
       //       which is slow and wasteful.
       // The big problem is that for props to be mutable, the entire parent whatsit needs to
       // be mutable, and Rust hits a mutability conflict between the parent, and the
       // "args" and "props" children ... will come back here after performance becomes
       // an issue again
       //
       // Part 2: I have now, with great attention and profiling, solidified the position that Whatsits are immutable
       // during the absorbtion phase -- and hense the args and props passed in here will remain immutable in rtx.
       // Hence, for this absorb call to run correctly, it must either:
       // 1) Accept a cloned value as currently, paying with performance
       // 2) Accept immutable references to digested objects, which may lead to far-reaching borrowing constraints
       //   e.g. unlist()-ing a digested List will have to produce box references, rather than provide the owned boxes directly.
       //   would have to experiment with this - as it is of course much lighter on performance
       if let Some(Stored::Digested(tags)) = props.get("tags") {
         document.absorb((**tags).clone(), state)?;
       }
       let title = prop_digested!(props, "title");
       document.insert_element("ltx:title", title, None, state)?;

       let toctitle = prop_digested!(props, "toctitle");
       if !toctitle.is_empty() {
         document.insert_element("ltx:toctitle", toctitle, None, state)?;
       }
     },
     properties => properties!(sub[stomach, args, state] {
       unpack!(args => stype, inlist, toctitle_arg, title);
       let mut props = ref_step_counter(&stype.to_string(), false, stomach, state)?;
       let toctitle = if toctitle_arg.to_string().is_empty() {
         toctitle_arg
       } else {
         title.clone()
       };

       let invoked_title;
       {
         let gullet = stomach.get_gullet_mut();
         invoked_title = Invocation!(T_CS!("\\lx@format@title@@"), vec![&stype, &title], gullet, state)?;
       }
       let xtitle    = stomach.digest(invoked_title, state)?;
       props.insert(s!("title"), xtitle.into());

       // TODO
       // let invoked_toctitle;
       // {
       //   let gullet = stomach.get_gullet_mut();
       //   invoked_toctitle = Invocation!(T_CS!("\\lx@format@toctitle@@"), vec![&stype, &toctitle], gullet, state)?;
       // }
       // let xtoctitle = stomach.digest(invoked_toctitle, state)?;
       //
       // if xtoctitle.to_string() != xtitle.to_string() {
       //   props.insert(s!("toctitle"), xtoctitle.into());
       // }

       Ok(props)
     })
  );

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

  // DefConstructorI!(
  //   "\\@@section{}{}{}{}{}{}",
  //   replacement!(document, args, props, inner_state, {
  //     unpack!(args => stype, id, refnum_arg, frefnum_arg, toctitle, title);
  //     let refnum = refnum_arg.to_string();
  //     let mut frefnum = frefnum_arg.to_string();
  //     if frefnum == refnum {
  //       frefnum = String::new();
  //     }

  //     let clean_id = id; // TODO: CleanID($id);
  //     let has_toctitle =
  //       !toctitle.to_string().is_empty() && (toctitle.to_string() != title.to_string());
  //     document.open_element(
  //       &s!("ltx:{}", stype.to_string()),
  //       Some(string_map!("xml:id" => clean_id, "refnum" => refnum, "frefnum" => frefnum)),
  //       None,
  //       inner_state,
  //     )?;
  //     document.insert_element("ltx:title", vec![title], None, inner_state)?;
  //     if has_toctitle {
  //       document.insert_element("ltx:toctitle", vec![toctitle], None, inner_state)?;
  //     }
  //   }),
  //   state
  // );

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
      let category_object = s!("unexpected:{}", cs);
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
          require_package(&package, RequireOptions {
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
  ]
  .iter()
  .map(|s| s.to_string())
  {
    DefMacroI!(T_CS!(ltxtrigger), None, Tokens!());
  }

  //======================================================================
  // C.5.4 The Title Page and Abstract
  //======================================================================

  // See frontmatter support in TeX.ltxml
  DefMacro!("\\title{}", "\\@add@frontmatter{ltx:title}{#1}");

  DefMacro!("\\sectionmark{}", "");
  DefMacro!("\\subsectionmark{}", "");
  DefMacro!("\\subsubsectionmark{}", "");
  DefMacro!("\\paragraphmark{}", "");
  DefMacro!("\\subparagraphmark{}", "");

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
    mode => Some(s!("inline_math"))
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
      DefMacroI!(cs.clone(), None, body, state);
    })
  );

  //======================================================================
  // C.8.4 Numbering
  //======================================================================
  // For LaTeX documents, We want id's on para, as well as sectional units.
  // However, para get created implicitly on Document construction, rather than
  // explicitly during digestion (via a whatsit), we can't use the usual LaTeX counter mechanism.
  Tag!("ltx:para", after_open => tagsub!(document, node, state, {
    generate_id(document, node, "p", state)?;
  }));

  // DefPrimitive('\newcounter{}[]', sub {
  //     NewCounter(ToString(Expand($_[1])), $_[2] && ToString(Expand($_[2])));
  //     return; });
  // DefPrimitive('\setcounter{}{Number}', sub { SetCounter(ToString(Expand($_[1])), $_[2]); });
  // DefPrimitive('\addtocounter{}{Number}', sub { AddToCounter(ToString(Expand($_[1])), $_[2]); });
  // DefPrimitive('\stepcounter{}',    sub { StepCounter(ToString(Expand($_[1])));    return; });
  // DefPrimitive('\refstepcounter{}', sub { RefStepCounter(ToString(Expand($_[1]))); return; });

  // DefPrimitive('\@addtoreset{}{}', sub {
  //     my ($stomach, $ctr, $within) = @_;
  //     $ctr    = ToString(Expand($ctr));
  //     $within = ToString(Expand($within));
  //     my $unctr = "UN$ctr";    # UNctr is counter for generating ID's for UN-numbered items.
  //     AssignValue("\\cl\@$within" =>
  //         Tokens(T_CS($ctr), T_CS($unctr),
  //         (LookupValue("\\cl\@$within") ? LookupValue("\\cl\@$within")->unlist : ())),
  //       'global');
  //     # This counter might be doing double duty generating ID's as well, so we may need to patch
  // up.     my $prefix = LookupValue('@ID@prefix@' . $ctr);
  //     if (defined $prefix) {
  //       DefMacroI(T_CS("\\the$ctr\@ID"), undef,
  //         "\\expandafter\\ifx\\csname the$within\@ID\\endcsname\\\@empty"
  //           . "\\else\\csname the$within\@ID\\endcsname.\\fi"
  //           . " $prefix\\csname \@$ctr\@ID\\endcsname",
  //         scope => 'global');
  //       DefMacroI(T_CS("\\\@$ctr\@ID"), undef, "0", scope => 'global'); }
  //     return; });

  DefMacro!("\\value{}", sub[gullet, args, inner_state] {
    unpack!(args => value);
    let ctr_expansion = Expand!(value, gullet, inner_state)?.to_string();
    let ctr_value = CounterValue!(&ctr_expansion, inner_state).value_of();
    Ok(Tokens::new(
      ExplodeText!(ctr_value)
    ))
  });

  // DefMacro('\@arabic{Number}', sub {
  //     ExplodeText(ToString($_[1]->valueOf)); });
  DefMacro!("\\arabic{}", sub[gullet, args, inner_state] {
    unpack!(args => value);
    let ctr_expansion = Expand!(value, gullet, inner_state)?.to_string();
    let ctr_value = CounterValue!(&ctr_expansion, inner_state).value_of();
    Ok(Tokens::new(
      ExplodeText!(ctr_value)
    ))
  });

  // DefMacro('\@roman{Number}', sub {
  //     ExplodeText(radix_roman(ToString($_[1]->valueOf))); });
  // DefMacro('\roman{}', sub {
  //     ExplodeText(radix_roman(CounterValue(ToString(Expand($_[1])))->valueOf)); });
  // DefMacro('\@Roman{Number}', sub {
  //     ExplodeText(radix_Roman(ToString($_[1]->valueOf))); });
  // DefMacro('\Roman{}', sub {
  //     ExplodeText(radix_Roman(CounterValue(ToString(Expand($_[1])))->valueOf)); });
  // DefMacro('\@alph{Number}', sub {
  //     ExplodeText(radix_alpha($_[1]->valueOf)); });
  // DefMacro('\alph{}', sub {
  //     ExplodeText(radix_alpha(CounterValue(ToString(Expand($_[1])))->valueOf)); });
  // DefMacro('\@Alph{Number}', sub {
  //     ExplodeText(radix_Alpha($_[1]->valueOf)); });
  // DefMacro('\Alph{}', sub {
  //     ExplodeText(radix_Alpha(CounterValue(ToString(Expand($_[1])))->valueOf)); });

  // our @fnsymbols = ("*", "\x{2020}", "\x{2021}", UTF(0xA7), UTF(0xB6),
  //   "\x{2225}", "**", "\x{2020}\x{2020}", "\x{2021}\x{2021}");
  // DefMacro('\@fnsymbol{Number}', sub {
  //     ExplodeText(radix_format($_[1]->valueOf, @fnsymbols)); });
  // DefMacro('\fnsymbol{}', sub {
  //     ExplodeText(radix_format(CounterValue(ToString(Expand($_[1])))->valueOf, @fnsymbols)); });

  // lines 4413-4563
  InnerPool!(latex_font_selection);

  //======================================================================
  // Hair
  DefPrimitive!("\\makeatletter", sub[stomach, whatsit, state] { state.assign_catcode('@', Catcode::LETTER, Some(Scope::Local)); Ok(vec![]) });
  DefPrimitive!("\\makeatother",  sub[stomach, whatsit, state] { state.assign_catcode('@', Catcode::OTHER, Some(Scope::Local)); Ok(vec![]) });

  //**********************************************************************
  // Sundry (is this ams ?)
  DefMacro!("\\textprime", "\u{00B4}"); // ACUTE ACCENT

  Let!("\\endgraf", "\\par");
  Let!("\\endline", "\\cr");
  //**********************************************************************
  // Should be defined in each (or many) package, but it"s not going to
  // get set correctly or maintained, so...
  DefMacro!("\\fileversion", "");
  DefMacro!("\\filedate", "");

  // Ultimately these may be overridden by babel, or otherwise,
  // various of these are defined in various places by different classes.
  DefMacro!("\\chaptername", "Chapter");
  DefMacro!("\\partname", "Part");
  // The rest of these are defined in some classes, but not most.
  //DefMacroI("\sectionname",       undef, "Section");
  //DefMacroI("\subsectionname",    undef, "Subsection");
  //DefMacroI("\subsubsectionname", undef, "Subsubsection");
  //DefMacroI("\paragraphname",     undef, "Paragraph");
  //DefMacroI("\subparagraphname",  undef, "Subparagraph");

  DefMacro!("\\appendixname", "Appendix");
  // These aren"t defined in LaTeX,
  // these definitions will give us more meaningful typerefnum"s
  DefMacro!(
    "\\sectiontyperefname",
    "\\lx@sectionsign\\lx@ignorehardspaces"
  );
  DefMacro!(
    "\\subsectiontyperefname",
    "\\lx@sectionsign\\lx@ignorehardspaces"
  );
  DefMacro!(
    "\\subsubsectiontyperefname",
    "\\lx@sectionsign\\lx@ignorehardspaces"
  );
  DefMacro!(
    "\\paragraphtyperefname",
    "\\lx@paragraphsign\\lx@ignorehardspaces"
  );
  DefMacro!(
    "\\subparagraphtyperefname",
    "\\lx@paragraphsign\\lx@ignorehardspaces"
  );

  Ok(())
}
