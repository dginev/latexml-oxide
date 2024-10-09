//**********************************************************************
// C.4 Sectioning and Table of Contents
//**********************************************************************
use crate::prelude::*;

LoadDefinitions!({
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
  DefMacro!("\\chapter", "\\@startsection{chapter}{0}{}{}{}{}", locked=>true);

  // not locked since sometimes redefined as partition?
  DefMacro!("\\part", "\\@startsection{part}{-1}{}{}{}{}");
  DefMacro!("\\section", "\\@startsection{section}{1}{}{}{}{}", locked=>true);
  DefMacro!("\\subsection", "\\@startsection{subsection}{2}{}{}{}{}", locked => true);
  DefMacro!(
    "\\subsubsection",
    "\\@startsection{subsubsection}{3}{}{}{}{}",
    locked => true);
  DefMacro!("\\paragraph", "\\@startsection{paragraph}{4}{}{}{}{}", locked => true);
  DefMacro!("\\subparagraph", "\\@startsection{subparagraph}{5}{}{}{}{}", locked => true);

  Tag!("ltx:part", auto_close=>true);
  Tag!("ltx:chapter", auto_close=>true);
  Tag!("ltx:section", auto_close=>true);
  Tag!("ltx:subsection", auto_close=>true);
  Tag!("ltx:subsubsection", auto_close=>true);
  Tag!("ltx:paragraph", auto_close=>true);
  Tag!("ltx:subparagraph", auto_close=>true);

  DefMacro!("\\secdef {}{} OptionalMatch:*", sub[(token1, token2, star)] {
    if star.is_some() {
      Ok(token2) // can't move out without clone, how to circumvent?
    } else {
      Ok(token1)
    }
  });

  DefMacro!("\\@startsection@hook", "");

  NewCounter!("secnumdepth");
  SetCounter!("secnumdepth", Number::new(3));
  DefMacro!(
    "\\@startsection{}{}{}{}{}{} OptionalMatch:*",
    sub[(type_tokens, level_arg, _ignore3, _ignore4, _ignore5, _ignore6, flag)] {
      // Aside: Guard mode
      // Never start sections in math mode -- this is a good recovery point for broken documents
      if lookup_bool("IN_MATH") {
        let mode = state::lookup_string("MODE");
        if mode.contains("math") { // double-check we're really in math
          end_mode(&mode)?;
        } else { // otherwise, just unset the flag?
          state::assign_value("IN_MATH", false, Some(Scope::Global));
        }
      }
      // Main logic
      let level = level_arg.to_string();
      let level_int = if level.is_empty() { 0 } else { level.parse::<i64>().expect(&level) };
      let mut tokens: Vec<Token>;
      if flag.is_some() { // No number, not in TOC
        tokens = vec![
          T_CS!("\\par"), T_CS!("\\@startsection@hook"), T_CS!("\\@@unnumbered@section"),
        T_BEGIN!()];
        tokens.extend(type_tokens.unlist());
        tokens.extend(vec![T_END!(), T_BEGIN!(), T_END!()]);
      } else if level_int > CounterValue!("secnumdepth").value_of() ||
        lookup_bool("no_number_sections") {
        // No number, but in TOC
        tokens = vec![
          T_CS!("\\par"), T_CS!("\\@startsection@hook"), T_CS!("\\@@unnumbered@section"),
        T_BEGIN!()];
        tokens.extend(type_tokens.unlist());
        tokens.extend(vec![T_END!(), T_BEGIN!(), T_OTHER!("toc"), T_END!()]);
      } else { // Number and in TOC
        tokens = vec![T_CS!("\\par"), T_CS!("\\@startsection@hook"), T_CS!("\\@@numbered@section"),
        T_BEGIN!()];
        tokens.extend(type_tokens.unlist());
        tokens.extend(vec![T_END!(), T_BEGIN!(), T_OTHER!("toc"), T_END!()]);
      };
      Ok(Tokens::new(tokens))
    },
    locked => true
  );

  DefConstructor!(
    "\\@@numbered@section{} Undigested OptionalUndigested Undigested",
    sub[document, args, props] {
      // args:=(stype,inlist,toctitle,title)
      let stype = args[0].as_ref().unwrap().to_string();
      let inlist = args[1].as_ref().unwrap().to_string();
      // TODO: This bizarre argument API interaction needs to be simplified down to Perl's
      // intuitive level of:       let (x,y,z, ...) = @args;
      let clean_id = prop_string!(props,"id"); // TODO: CleanID($id);
      document.open_element(&s!("ltx:{stype}"),
        Some(string_map!("xml:id" => clean_id, "inlist" => inlist)),
        None,
          )?;
      // TODO: Another instance where the immutability of props causes endless cloning
      //       which is slow and wasteful.
      // The big problem is that for props to be mutable, the entire parent whatsit needs to
      // be mutable, and Rust hits a mutability conflict between the parent, and the
      // "args" and "props" children ... will come back here after performance becomes
      // an issue again
      //
      // Part 2: I have now, with great attention and profiling, solidified the position that
      //       Whatsits are immutable during the absorbtion phase -- and hence
      // the args and props passed in here will remain immutable in rtx.
      // Hence, for this absorb call to run correctly, it must either:
      // 1) Accept a cloned value as currently, paying with performance
      // 2) Accept immutable references to digested objects,
      // which may lead to far-reaching borrowing constraints
      //   e.g. unlist()-ing a digested List will have to produce box references,
      //  rather than provide the owned boxes directly.
      //   would have to experiment with this - as it is of course much lighter on performance
      //

      // Update 2022: The notes are generally still accurate,
      // but cloning a Digested object is now cheap enough,
      // as each enum variant is guarded by an Rc reference counter. Rc<Tbox>, Rc<List>, etc.
      if let Some(Stored::Digested(tags)) = props.get("tags") {
        document.absorb(tags, None)?;
      }
      let title = prop_digested!(props, "title");
      document.insert_element("ltx:title", title, None)?;

      let toctitle = prop_digested!(props, "toctitle");
      if !toctitle.is_empty() {
        document.insert_element("ltx:toctitle", toctitle, None)?;
      }
    },
    properties => sub[args] {
      let stype = args[0].as_ref().unwrap();
      // let inlist = args[1].as_ref().unwrap();
      let toctitle_arg = args[2].as_ref();
      let title = args[3].as_ref().unwrap();

      let mut props = ref_step_counter(&stype.to_string(), false)?;
      let toctitle = match toctitle_arg {
        Some(v) => if !v.to_string().is_empty() {
          args[2].as_ref().unwrap()
        } else {
          title
        },
        None => title
      };
      let stype_tokens = stype.revert()?;
      let title_tokens = title.revert()?;
      let invoked_title =
        Invocation!(T_CS!("\\lx@format@title@@"), vec![stype_tokens, title_tokens]);
      let xtitle    = stomach::digest(invoked_title)?;
      let invoked_toctitle = Invocation!(T_CS!("\\lx@format@toctitle@@"),
          vec![stype.revert()?, toctitle.revert()?]);
      let xtoctitle = stomach::digest(invoked_toctitle)?;

      if xtoctitle.to_string() != xtitle.to_string() {
        props.insert("toctitle", xtoctitle.into());
      }
      props.insert("title", xtitle.into());

      Ok(props)
    }
  );

  // No tags, at all? Consider...
  DefConstructor!("\\@@unnumbered@section{} Undigested OptionalUndigested Undigested",
  sub[document, args, props] {
      let stype = args[0].as_ref().unwrap();
      let inlist = args[1].as_ref().unwrap();
      // let toctitle_arg = args[2].as_ref();
      // let title = args[3].as_ref().unwrap();

      let id = props.get("id").unwrap().to_string();
      document.open_element(&s!("ltx:{stype}"),
        Some(string_map!(
          "xml:id" => clean_id(&id),
          "inlist"  => inlist.to_string()
        )), None)?;
      let title = prop_digested!(props, "title");
      document.insert_element("ltx:title", title, None)?;

      let toctitle = prop_digested!(props, "toctitle");
      if !toctitle.is_empty() {
        document.insert_element("ltx:toctitle", toctitle, None)?;
      }
    },
    properties => sub[args] {
      use DigestedData::*;
      let stype = args[0].as_ref().unwrap();
      // let inlist = args[1].as_ref().unwrap();
      let toctitle_arg = args[2].as_ref();
      let title = args[3].as_ref().unwrap();
      let mut props = RefStepID!(&stype.to_string())?;
      let title_digested = if let Postponed(tokens) = title.data() {
        // TODO: is .clone() on the tokens before they are unlisted a sign that
        // the DigestedData::Postponed variant isn't ideal?
        // should we be draining it? Or is there a better conceptual organization?
        stomach::digest(
          Tokens!(T_CS!("\\@hidden@bgroup"), tokens.clone().unlist(), T_CS!("\\@hidden@egroup")))?
      } else {
        title.clone()
      };
      props.insert("title", title_digested.into());

      if let Some(toctitle) = toctitle_arg {
        if let Postponed(toctokens) = toctitle.data() {
          if !toctokens.is_empty() {
            let toctitle_digested = stomach::digest(
              Tokens!(T_CS!("\\@hidden@bgroup"),
                toctokens.clone().unlist(), T_CS!("\\@hidden@egroup")))?;
            props.insert("toctitle", toctitle_digested.into());
          }
        }
      }
      Ok(props)
    }
  );

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
  // ]{#1}#2");
  // DefConstructor!("\\@tag[][]{}", "?#3(<ltx:tag open='#1' close='#2'>#3</ltx:tag>)()");

  //// NOTE that a 3rd form seems desirable: an concise form that cannot rely on context for the
  //// type. This would be useful for the titles in links; thus can be plain (unicode) text.
  //// However, I hate setting up even more machinery & options and dragging yet another form
  //// around....
  // \@@section{type}{id}{refnum}{formattedrefnum}{toctitle}{title}

  // DefConstructor!(
  //   "\\@@section{}{}{}{}{}{}",
  //   replacement!(document, args, props, inner{
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
  //       inner_state::
  //     )?;
  //     document.insert_element("ltx:title", vec![title], None, inner_state::?;
  //     if has_toctitle {
  //       document.insert_element("ltx:toctitle", vec![toctitle], None, inner_state::?;
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
  // TODO: add the rest...
  DefMacro!("\\@@appendix", "\\@startsection{appendix}{0}{}{}{}{}");

  //======================================================================
  // C.4.3 Table of Contents
  //======================================================================
  // Insert stubs that will be filled in during post processing.
  DefMacro!("\\contentsname", "Contents");
  DefConstructor!("\\tableofcontents",
    "<ltx:TOC lists='toc' scope='global' select='#select'><ltx:title>#name</ltx:title></ltx:TOC>",
    properties => {
      let mut td = CounterValue!("tocdepth").value_of() as usize + 1;
      let s  = ["ltx:part", "ltx:chapter", "ltx:section", "ltx:subsection", "ltx:subsubsection",
          "ltx:paragraph", "ltx:subparagraph"];
      let max_level = s.len()-1;
      td = std::cmp::min(td,max_level);
      let mut s_depth : Vec<&'static str> = s.into_iter().take(td+1).collect();
      if !s_depth.is_empty() {
        s_depth.push("ltx:appendix");
        s_depth.push("ltx:index");
        s_depth.push("ltx:bibliography");
      }
      
      Ok(stored_map!("select" => s_depth.join(" | "),
        "name" => digest(T_CS!("\\contentsname"))?))
    }
  );
  
  DefMacro!("\\listfigurename", "List of Figures");
  DefConstructor!("\\listoffigures",
    "<ltx:TOC lists='lof' scope='global'><ltx:title>#name</ltx:title></ltx:TOC>",
    properties => { Ok(stored_map!("name" => stomach::digest(T_CS!("\\listfigurename"))?)) });
  
  DefMacro!("\\listtablename", "List of Tables");
  DefConstructor!("\\listoftables",
    "<ltx:TOC lists='lot' scope='global'><ltx:title>#name</ltx:title></ltx:TOC>",
    properties => { Ok(stored_map!("name" => stomach::digest(T_CS!("\\listtablename"))?)) });
  
  DefPrimitive!("\\numberline{}{}", None);
  DefPrimitive!("\\addtocontents{}{}", None);
  
  DefConstructor!("\\addcontentsline{}{}{}", sub[document,args] {
      if let [inlist,_vtype,_title @ ..] = args.as_slice() {
        // Note that the node can be inlist $inlist.
        // Could conceivably want to add $title as toctitle???
        if let Some(savenode) = document.float_to_label() {
          // DG: The Document+Node mutability API is strange 
          //     w.r.t the original Perl ergonomics.
          // if we use `.get_node_mut()` we can no longer `doc.set_attribute(node)`,
          // as it induces TWO simultaneous mutable pointers into document. 
          // cloning Node is now cheap enough (as the Node data lives in C's libxml)
          // but it's not yet an idiomatic Rust interface. Something to ponder...
          let mut node  = document.get_node().clone();
          let inlist_str = inlist.as_ref().map(|v|v.to_string()).unwrap_or_default();
          let inlist_v = if let Some(lists) = node.get_attribute("inlist") {
            if !lists.is_empty() {
              s!("{lists} {inlist_str}")
            } else { inlist_str }
          } else {
            inlist_str
          };
          document.set_attribute(&mut node, "inlist", &inlist_v)?;
          document.set_node(&savenode); 
        }
      }     
    }
  );
  

  //======================================================================
  // C.4.4 Style registers
  //======================================================================
  NewCounter!("tocdepth");
});
