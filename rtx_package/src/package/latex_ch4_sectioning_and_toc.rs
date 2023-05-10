//**********************************************************************
// C.4 Sectioning and Table of Contents
//**********************************************************************
use crate::package::*;

LoadDefinitions!(outer_stomach, outer_state, {
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

  // not locked since sometimes redefined as partition?
  DefMacro!("\\part", "\\@startsection{part}{-1}{}{}{}{}");
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

  DefMacro!("\\secdef {}{} OptionalMatch:*", sub[gullet, (token1, token2, star), state] {
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
    sub[ gullet, (type_tokens, level_arg, ignore3, ignore4, ignore5, ignore6, flag), state ] {
      let stype = type_tokens.to_string();
      let level = level_arg.to_string();
      let level_int = if level.is_empty() { 0 } else { level.parse::<i64>().expect(&level) };
      let ctr = match state.lookup_value(&s!("counter_for_{stype}")) {
        Some(v) => v.to_string(),
        None => stype
      };
      let mut tokens: Vec<Token>;
      if flag.is_some() { // No number, not in TOC
        tokens = vec![
          T_CS!("\\par"), T_CS!("\\@startsection@hook"), T_CS!("\\@@unnumbered@section"),
        T_BEGIN!()];
        tokens.extend(type_tokens.unlist());
        tokens.extend(vec![T_END!(), T_BEGIN!(), T_END!()]);
      } else if level_int > CounterValue!("secnumdepth", state).value_of() ||
        state.lookup_bool("no_number_sections") {
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
    sub[document, args, props, state] {
      // args:=(stype,inlist,toctitle,title)
      let stype = args[0].as_ref().unwrap().to_string();
      let inlist = args[1].as_ref().unwrap().to_string();
      // TODO: This bizarre argument API interaction needs to be simplified down to Perl's
      // intuitive level of:       let (x,y,z, ...) = @args;
      let clean_id = prop_string!(props,"id"); // TODO: CleanID($id);
      document.open_element(&s!("ltx:{stype}"),
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
        document.absorb(tags, None, state)?;
      }
      let title = prop_digested!(props, "title");
      document.insert_element("ltx:title", title, None, state)?;

      let toctitle = prop_digested!(props, "toctitle");
      if !toctitle.is_empty() {
        document.insert_element("ltx:toctitle", toctitle, None, state)?;
      }
    },
    properties => sub[stomach, args, state] {
      let stype = args[0].as_ref().unwrap();
      let inlist = args[1].as_ref().unwrap();
      let toctitle_arg = args[2].as_ref();
      let title = args[3].as_ref().unwrap();

      let mut props = ref_step_counter(&stype.to_string(), false, stomach, state)?;
      let toctitle = match toctitle_arg {
        Some(v) => if !v.to_string().is_empty() {
          args[2].as_ref().unwrap()
        } else {
          title
        },
        None => title
      };
      let stype_tokens = stype.revert(state)?;
      let title_tokens = title.revert(state)?;
      let invoked_title;
      {
        let gullet = stomach.get_gullet_mut();
        invoked_title =
          Invocation!(T_CS!("\\lx@format@title@@"), vec![stype_tokens, title_tokens], gullet)?;
      }
      let xtitle    = stomach.digest(invoked_title, state)?;

      let invoked_toctitle;
      {
        let gullet = stomach.get_gullet_mut();
        invoked_toctitle = Invocation!(T_CS!("\\lx@format@toctitle@@"),
          vec![stype.revert(state)?, toctitle.revert(state)?], gullet, state)?;
      }
      let xtoctitle = stomach.digest(invoked_toctitle, state)?;

      if xtoctitle.to_string() != xtitle.to_string() {
        props.insert(s!("toctitle"), xtoctitle.into());
      }
      props.insert(s!("title"), xtitle.into());

      Ok(props)
    }
  );

  // No tags, at all? Consider...
  DefConstructor!("\\@@unnumbered@section{} Undigested OptionalUndigested Undigested",
  sub[document, args, props, state] {
      let stype = args[0].as_ref().unwrap();
      let inlist = args[1].as_ref().unwrap();
      // let toctitle_arg = args[2].as_ref();
      // let title = args[3].as_ref().unwrap();

      let id = props.get("id").unwrap().to_string();
      document.open_element(&s!("ltx:{stype}"),
        Some(string_map!(
          "xml:id" => clean_id(&id),
          "inlist"  => inlist.to_string()
        )), None, state
      )?;
      let title = prop_digested!(props, "title");
      document.insert_element("ltx:title", title, None, state)?;

      let toctitle = prop_digested!(props, "toctitle");
      if !toctitle.is_empty() {
        document.insert_element("ltx:toctitle", toctitle, None, state)?;
      }
    },
    properties => sub[stomach, args, state] {
      use DigestedData::*;
      let stype = args[0].as_ref().unwrap();
      let inlist = args[1].as_ref().unwrap();
      let toctitle_arg = args[2].as_ref();
      let title = args[3].as_ref().unwrap();
      let mut props = RefStepID!(&stype.to_string())?;
      let title_digested = if let Postponed(tokens) = title.data() {
        // TODO: is .clone() on the tokens before they are unlisted a sign that
        // the DigestedData::Postponed variant isn't ideal?
        // should we be draining it? Or is there a better conceptual organization?
        stomach.digest(
          Tokens!(T_CS!("\\@hidden@bgroup"), tokens.clone().unlist(), T_CS!("\\@hidden@egroup")),
          state)?
      } else {
        title.clone()
      };
      props.insert("title".to_string(), title_digested.into());

      if let Some(toctitle) = toctitle_arg {
        if let Postponed(toctokens) = toctitle.data() {
          if !toctokens.is_empty() {
            let toctitle_digested = stomach.digest(
              Tokens!(T_CS!("\\@hidden@bgroup"),
                toctokens.clone().unlist(), T_CS!("\\@hidden@egroup")),
              state)?;
            props.insert("toctitle".to_string(), toctitle_digested.into());
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
});
