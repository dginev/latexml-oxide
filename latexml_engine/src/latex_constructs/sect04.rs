//! `latex_constructs` section 4: C.4 Sectioning and Table of Contents
//!
//! Split verbatim from the single `LoadDefinitions!` body (2026-09-03); the
//! definitions run in the original order from `super::load_definitions`.

use super::*;

#[rustfmt::skip]
pub(crate) fn load() -> Result<()> {
  // ======================================================================
  // C.4 Sectioning and Table of Contents
  // ======================================================================

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
  // Also auto-close structural/backmatter containers so papers that open
  // `\acknowledgments` / `\appendix` / `\index` without a matching `\end...`
  // (common in mn/jheppub/pos classes) don't leave the element open until
  // `\end{document}` and produce schema-violation errors when a following
  // bibliography or section is emitted.
  // Perl: ltx:bibliography already has autoClose=1 (latex_constructs L4078);
  // these siblings match its container-with-trailing-content semantics.
  // arXiv-fork: acknowledgements joins the navigation TOC (see the
  // ltx:abstract Tag above for the design note).
  Tag!("ltx:acknowledgements", auto_close => true,
    after_open => sub[document, node] {
      document.set_attribute(node, "inlist", "toc")?;
      document.generate_id(node, "acknowledgements")?;
  });
  Tag!("ltx:appendix", auto_close => true);
  Tag!("ltx:index", auto_close => true);
  // NOTE: tried Tag!("ltx:itemize"/"ltx:enumerate"/"ltx:description",
  // auto_close=>true) to address schema errors like "ltx:bibitem in
  // <ltx:itemize>" from malformed user input (e.g. 0801.4271). That
  // BROKE the 10_expansion/partial test because itemize would close
  // immediately before items are added. Perl's L1337 only marks
  // `ltx:item` as autoClose/autoOpen — container remains
  // explicit-close-only. Leaving these alone for now.

  // latex.ltx:16187 `\def\secdef#1#2{\@ifstar{#2}{\@dblarg{#1}}}` — the
  // unstarred form doubles the title into the `[#1]` slot. Perl's shortcut
  // (`($_[3] ? $_[2] : $_[1])`, pool:567) drops the `\@dblarg`, so a raw
  // `\long\def\@book[#1]#2` reached from memoir.cls:2787 `\secdef\@book\@sbook`
  // (srbook-mem Test/TestLight/SerbianBookMem `\book{…}`) scanned to EOF for
  // its `[` (`Until:]`). Only raw callers reach `\secdef` — our
  // `\@startsection` dispatches natively. Guard:
  // `perfect_kernel_batch54::secdef_doubles_the_title_for_the_unstarred_form`.
  DefMacro!("\\secdef{}{}", "\\@ifstar{#2}{\\@dblarg{#1}}");

  def_macro_noop("\\@startsection@hook")?;

  NewCounter!("secnumdepth");
  SetCounter!("secnumdepth", Number::new(3));
  DefMacro!(
    "\\@startsection{}{}{}{}{}{} OptionalMatch:*",
    sub[(type_tokens, level_arg, _ignore3, _ignore4, _ignore5, _ignore6, flag)] {
      // Aside: Guard mode
      // Never start sections in math mode -- this is a good recovery point for broken documents
      if lookup_bool_sym(pin!("IN_MATH")) {
        let mode = lookup_string_from_sym(pin!("MODE"));
        if mode.contains("math") { // double-check we're really in math
          end_mode(&mode)?;
        } else { // otherwise, just unset the flag?
          assign_value("IN_MATH", false, Some(Scope::Global));
        }
      }
      // Main logic. The level is a TeX <number> — latex.ltx `\@sect` compares
      // it with `\ifnum #2>\c@secnumdepth` — so a non-literal level is READ as
      // one: scrartcl.cls L3421/L3425 pass every heading's level as
      // `{\numexpr #2\relax}` with `#2` = `\csname <name>numdepth\endcsname`.
      // Perl's `$level > …` string-coerces that to 0, numbering every KOMA
      // heading down to `\subparagraph` (witness tudaexercise / any raw KOMA
      // class; guard `perfect_kernel_batch53::startsection_level_is_a_tex_number`).
      // An empty or unreadable level still coerces to 0 as in Perl.
      let level = level_arg.to_string();
      let level_int = match level.trim().parse::<i64>() {
        Ok(n) => n,
        Err(_) if level.trim().is_empty() => 0,
        Err(_) => {
          let level_tokens = level_arg.clone();
          let mouth = Mouth::new("", None)?;
          reading_from_mouth(mouth, move || {
            unread(level_tokens);
            read_number()
          })
          .map(|n| n.value_of())
          .unwrap_or(0)
        },
      };
      // A section type the schema does not know (`\DeclareSectionCommand`
      // headings) is bound to the element of its level here — the only place
      // the level is visible; see `section_element_for_type`.
      let stype = strip_trailing_cs(type_tokens.to_string().trim());
      if !stype.is_empty() && !is_known_section_type(&stype) && stype != "app" {
        assign_mapping("SECTION_ELEMENT", &stype, Some(pin(section_element_for_level(level_int))));
      }
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
      // Sanitize stype: under some upstream conditions (figure-block + section
      // sequencing — see math0010095 / Cluster A) the {} parameter reader picks
      // up a trailing \par token that pollutes the section type identifier.
      // \par should never appear inside a section type; strip any trailing
      // backslash-prefixed CS to recover the bare identifier.
      let stype = section_type_name(args[0].as_ref().unwrap());
      let inlist = args[1].as_ref().unwrap().to_string();
      // TODO: This bizarre argument API interaction needs to be simplified down to Perl's
      // intuitive level of:       let (x,y,z, ...) = @args;
      // If backmatter, find insertion point as if inserting the backmatter element type
      if let Some(asif) = props.get("backmatterelement") {
        let target = backmatter_insertion_target(
          document, &asif.to_string(),
          &section_element_for_type_maybe(&stype).unwrap_or_else(|| "ltx:section".into()));
        let point = document.find_insertion_point(&target, None)?;
        document.set_node(&point);
      }
      let clean_id = prop_string!(props,"id"); // TODO: CleanID($id);
      // Mirror Perl `latex_constructs.pool.ltxml:599-607`: sanitize the type
      // name so we don't open arbitrary elements outside the schema. Perl
      // checks `isKnownTag(ltx:$type)`; if unknown, special-cases `ltx:app
      // → ltx:appendix`, otherwise falls back to `ltx:section` and warns.
      // Without this guard, papers that do
      // `\newcommand\Proof{\@startsection{Proof}{5}{...}}` (e.g.
      // mst-stylefile.sty in 1608.04650) opened `<ltx:Proof>` and cascaded
      // 1500+ malformed errors on every nested element.
      let tagname = section_element_for_type(&stype, true);
      document.open_element(&tagname,
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
      //       Whatsits are immutable during the absorption phase -- and hence
      // the args and props passed in here will remain immutable in latexml_oxide.
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

      maybe_peek_label()?;
      // See Cluster A note in the body closure above; sanitize identical here.
      let stype_str = section_type_name(stype);
      let mut props = ref_step_counter(&stype_str, false)?;
      // For appendix, look up the backmatter element mapping
      if stype_str == "appendix"
        && let Some(bme) = lookup_mapping("BACKMATTER_ELEMENT", &s!("ltx:{stype_str}")) {
          props.insert("backmatterelement", bme);
        }
      let toctitle = match toctitle_arg {
        Some(v) => if !v.to_string().is_empty() {
          args[2].as_ref().unwrap()
        } else {
          title
        },
        None => title
      };
      // Cluster A: rebuild tokens from sanitized stype_str so the trailing
      // \par token doesn't propagate into \lx@format@title@@'s body.
      let stype_clean_tokens = Tokens::new(Explode!(stype_str));
      let title_tokens = title.revert()?;
      let invoked_title =
        Invocation!(T_CS!("\\lx@format@title@@"), vec![stype_clean_tokens.clone(), title_tokens]);
      let xtitle    = digest(invoked_title)?;
      let invoked_toctitle = Invocation!(T_CS!("\\lx@format@toctitle@@"),
          vec![stype_clean_tokens, toctitle.revert()?]);
      let xtoctitle = digest(invoked_toctitle)?;

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
      // If backmatter, find insertion point as if inserting the backmatter element type
      if let Some(asif) = props.get("backmatterelement") {
        let target = backmatter_insertion_target(
          document, &asif.to_string(),
          &section_element_for_type_maybe(&section_type_name(stype))
            .unwrap_or_else(|| "ltx:section".into()));
        let point = document.find_insertion_point(&target, None)?;
        document.set_node(&point);
      }
      let id = props.get("id").unwrap().to_string();
      // Mirror the same schema sanitization as \@@numbered@section above.
      // Cluster A: strip trailing CS (e.g. \par) from stype.
      let stype_str = section_type_name(stype);
      let tagname = section_element_for_type(&stype_str, false);
      document.open_element(&tagname,
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
      maybe_peek_label()?;
      // Cluster A sanitization (see \@@numbered@section).
      let stype_str = section_type_name(stype);
      let mut props = RefStepID!(&stype_str)?;
      // For appendix, look up the backmatter element mapping
      if stype_str == "appendix"
        && let Some(bme) = lookup_mapping("BACKMATTER_ELEMENT", &s!("ltx:{stype_str}")) {
          props.insert("backmatterelement", bme);
        }
      let title_digested = if let Postponed(tokens) = title.data() {
        // TODO: is .clone() on the tokens before they are unlisted a sign that
        // the DigestedData::Postponed variant isn't ideal?
        // should we be draining it? Or is there a better conceptual organization?
        digest(
          Tokens!(T_CS!("\\lx@hidden@bgroup"), tokens.clone().unlist(), T_CS!("\\lx@hidden@egroup")))?
      } else {
        title.clone()
      };
      props.insert("title", title_digested.into());

      if let Some(toctitle) = toctitle_arg
        && let Postponed(toctokens) = toctitle.data()
          && !toctokens.is_empty() {
            let toctitle_digested = digest(
              Tokens!(T_CS!("\\lx@hidden@bgroup"),
                toctokens.clone().unlist(), T_CS!("\\lx@hidden@egroup")))?;
            props.insert("toctitle", toctitle_digested.into());
          }
      Ok(props)
    }
  );

  //----------------------------------------------------------------------
  // The following macros provide a few layers of customization
  // in particular for supporting localization for different languages.
  //----------------------------------------------------------------------
  // \lx@format@title@@{type}{title} — implemented in base_utilities.rs
  // \lx@format@toctitle@@{type}{toctitle} — implemented in base_utilities.rs
  // \lx@@compose@title{}{} — implemented in base_utilities.rs
  // \lx@tag[][ ]{}{} — implemented in base_utilities.rs
  //
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
  // \appendixname / \appendixesname / \@@appendix all live in
  // `latex_constructs_rust_only.rs` section 8 / 7a (Perl
  // `latex_base.pool.ltxml` L287 + sandbox-derived helpers).

  //======================================================================
  // C.4.3 Table of Contents
  //======================================================================
  // Insert stubs that will be filled in during post processing.
  // \contentsname lives in `latex_constructs_rust_only.rs` section 8.
  // Real latex.ltx defines `\tableofcontents` (and the \listof* pair) as
  // MACROS; packages patch them (`\appto{\tableofcontents}{\bigskip}`,
  // algxpar-doc via packdoc). Both engines historically bound the
  // constructor to the user name directly, so etoolbox's `\expandonce`
  // (= expand once) got the UNEXPANDABLE constructor token back and built
  // a self-recursive def ("expands into itself", TOC lost; Perl shares).
  // Layer: user name = macro delegating to the internal constructor, so
  // one-step expansion yields a patchable body — the real kernel shape.
  DefMacro!("\\tableofcontents", "\\lx@kernel@tableofcontents");
  DefConstructor!("\\lx@kernel@tableofcontents",
    "<ltx:TOC lists='toc' scope='global' select='#select'><ltx:title>#name</ltx:title></ltx:TOC>",
    properties => {
      let s  = ["ltx:part", "ltx:chapter", "ltx:section", "ltx:subsection", "ltx:subsubsection",
          "ltx:paragraph", "ltx:subparagraph"];
      // Perl latex_constructs.pool.ltxml L727-733: `$td = tocdepth+1`, clamp to
      // the last section-type index, then take `s[0 .. $td]`. Perl's `0 .. $td`
      // is an EMPTY range when `$td < 0`, so `\setcounter{tocdepth}{-1}` (parts
      // only) yields `[part]` and `{-2}` or lower yields `select=''` (an empty
      // ToC). Compute in SIGNED space: `tocdepth` is a signed counter, so the
      // old `value_of() as usize` wrapped on negatives — a debug overflow panic,
      // and in release a silently over-full ToC (`{-2}` listed everything).
      let td = (CounterValue!("tocdepth").value_of() + 1).min((s.len() - 1) as i64);
      let take = usize::try_from(td + 1).unwrap_or(0);
      let mut s_depth : Vec<&'static str> = s.into_iter().take(take).collect();
      if !s_depth.is_empty() {
        s_depth.push("ltx:appendix");
        s_depth.push("ltx:index");
        s_depth.push("ltx:bibliography");
      }

      Ok(stored_map!("select" => s_depth.join(" | "),
        "name" => digest(T_CS!("\\contentsname"))?))
    }
  );

  // arXiv-fork (23771504 + 085c9fb6): abstract and acknowledgements carry
  // `inlist="toc"` + a generated xml:id so the navigation TOC (Post::Scan +
  // CrossRef gen_toc) can list them. The user-visible `\tableofcontents`
  // select list above is deliberately EXEMPT (the fork reverted that half).
  Tag!("ltx:abstract", after_open => sub[document, node] {
    document.set_attribute(node, "inlist", "toc")?;
    document.generate_id(node, "abstract")?;
  });

  // \listfigurename / \listtablename live in `latex_constructs_rust_only.rs` section 8.
  DefMacro!("\\listoffigures", "\\lx@kernel@listoffigures");
  DefConstructor!("\\lx@kernel@listoffigures",
    "<ltx:TOC lists='lof' scope='global'><ltx:title>#name</ltx:title></ltx:TOC>",
    properties => { Ok(stored_map!("name" => digest(T_CS!("\\listfigurename"))?)) });

  DefMacro!("\\listoftables", "\\lx@kernel@listoftables");
  DefConstructor!("\\lx@kernel@listoftables",
    "<ltx:TOC lists='lot' scope='global'><ltx:title>#name</ltx:title></ltx:TOC>",
    properties => { Ok(stored_map!("name" => digest(T_CS!("\\listtablename"))?)) });

  def_primitive_noop("\\numberline{}{}")?;
  def_primitive_noop("\\addtocontents{}{}")?;

  // The title (#3) is `Undigested`: TeX's `\addcontentsline` (latex.ltx
  // L17351-17363) hands #3 to `\protected@write`, where `\protect` is
  // `\@unexpandable@protect` and the text is written to the .toc, NEVER
  // typeset. Perl digests it (latex_constructs.pool.ltxml L749 `{}{}{}`) and
  // discards the result — dead work that hangs on LaTeX's write-only
  // self-`\protect` idiom `\def\appfmt#1{\protect\appfmt{#1}}`
  // (nlctuserguide.sty L1553 `\@loe@disable@cmds`): `\protect`=`\relax`
  // under digestion, so the macro re-expands to itself forever. Witness
  // glossaries-user (Fatal:Timeout:Recursion / TokenLimit, reached once the
  // raw KOMA `\numberline` became 1-arg and stopped swallowing the title).
  // Perl hangs on the 8-line repro. KNOWN_PERL_ERRORS #120; guard
  // `perfect_kernel_batch53::addcontentsline_title_is_not_digested`.
  DefConstructor!("\\addcontentsline{}{} Undigested", sub[document,args] {
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

  Ok(())
}
