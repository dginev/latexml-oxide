use latexml_engine::latex_constructs::{
  external_label_field, external_label_tex, external_ref_properties_with,
};

use crate::prelude::*;

// Perl: cleveref.sty.ltxml — cleveref cross-referencing
// Provides \cref, \Cref, \crefrange, etc. with type-aware formatting
#[rustfmt::skip]
LoadDefinitions!({
  // Save original \label
  Let!("\\lx@cleverref@save@label", "\\label");
  // cleveref.sty `\label[type]{key}`: the optional type is the label's reference type, `\cref` naming the object by it
  // (`\label[appendixtable]{tab:a}`, "Appendix Table 1" where the table's own type says "table 1"; 2404.11739,
  // 2605.13648). Perl's binding (cleveref.sty.ltxml:20-22) reads and drops it. The type goes on as `\lx@label@type`, as
  // the sites that move a trailing `\label` give it (sect09.rs `\@caption@postlabel`, OXIDIZED_DESIGN #552). Guard
  // `perfect_kernel_batch77s::cleveref_label_type_names_the_reference`.
  DefMacro!("\\lx@cleverref@label[]", sub[(ty)] {
    let mut tokens = Vec::new();
    if let Some(ty) = ty {
      tokens.extend([T_CS!("\\lx@label@type"), T_BEGIN!()]);
      tokens.extend(ty.unlist());
      tokens.push(T_END!());
    }
    tokens.push(T_CS!("\\lx@cleverref@save@label"));
    Tokens::new(tokens)
  });
  DefMacro!("\\lx@label@type{}", sub[(ty)] {
    if ty.unlist_ref().iter().all(|t| t.get_catcode() == Catcode::SPACE) {
      return Ok(Tokens!());
    }
    let mut out = vec![T_CS!("\\lx@cleverref@labeltype"), T_BEGIN!()];
    out.extend(ty.unlist());
    out.push(T_END!());
    Ok(Tokens::new(out))
  });
  // The object the label names takes the type in its cleveref tags (`creftype`, …, the forms `\cref` prints), the
  // object found as `\lx@label` finds it. cleveref keeps the type per label (its `\newlabel{<key>@cref}`); here it is
  // the object's, which one label names. The other forms (`refnum`, `typerefnum`) are the kernel's, unchanged.
  // (a label after a counter step a group has undone — after a closed float, after a `\footnote` — names the object
  // still in scope, as the kernel's `\lx@label` finds it, `Document::float_to_label_scoped`; else the type went to
  // the object just closed and the label to the one in scope. Guard
  // `perfect_kernel_batch78j::cleveref_label_type_after_a_closed_float_names_the_object_in_scope`)
  DefConstructor!("\\lx@cleverref@labeltype{}", sub[document, _args, props] {
    let Some(Stored::VecDigested(tags)) = props.get("cref_tags") else {
      return Ok(());
    };
    let raw_step = raw_refstep_of(props);
    let after_undone_step = matches!(props.get("after_undone_step"), Some(Stored::Bool(true)));
    if let Some(savenode) =
      document.float_to_label_scoped(raw_step.as_ref().map(|step| step.serial), after_undone_step)
    {
      // (numbered first, as `\lx@label` numbers it: cleveref's `\label` gives the type before the label;
      // 2506.01072 `\newtcbtheorem`, 2505.20088 `[auto counter]`)
      if let Some(step) = raw_step {
        document.take_raw_refstep_number(&step.tags, step.serial)?;
      }
      let target = document.get_node().clone();
      for ((role, _), tag) in cref_label_type_roles().iter().zip(tags.iter()) {
        let xpath = s!("ltx:tags/ltx:tag[@role='{role}']");
        let olds = document.findnodes(&xpath, Some(&target));
        // A form the object's own type has no name for (its empty tag removed: a tcolorbox counter, numbered by the
        // box's raw step, `\newtcbtheorem[crefname={theorem}{theorems}]` with `label type=<box>`, tcolorbox.sty:2270,
        // :955-968; 2506.01072) is the label's type's.
        if olds.is_empty() {
          if let Some(tags_node) = document.findnodes("ltx:tags", Some(&target)).into_iter().next() {
            let before = tags_node.get_child_nodes();
            document.set_node(&tags_node);
            document.absorb(tag, None)?;
            for new in tags_node.get_child_nodes() {
              if !before.contains(&new) {
                remove_if_empty_held(document, new);
              }
            }
          }
          continue;
        }
        for mut old in olds {
          let Some(tags_node) = old.get_parent() else {
            continue;
          };
          // The new tag in the old one's place; a form the type has no name for (an empty tag, removed) leaves none.
          let before = tags_node.get_child_nodes();
          document.set_node(&tags_node);
          document.absorb(tag, None)?;
          for mut new in tags_node.get_child_nodes() {
            if !before.contains(&new) {
              new.unlink_node();
              old.add_prev_sibling(&mut new).ok();
              remove_if_empty_held(document, new);
            }
          }
          document.remove_node(old);
        }
      }
      // The object's own number in the label type's number forms: its refnum's content where the forms were built
      // with the mark (`\lx@cref@numbermark`, `cref_number`), as a `\tag{$\mathrm{P}$}`'s "P" (2609.28438's
      // `\label[prob]` under `\crefformat{prob}{prob.~(#2#1#3)}`: "prob. (P)"), which no counter holds.
      let number = document
        .findnodes("ltx:tags/ltx:tag[@role='refnum']", Some(&target))
        .into_iter()
        .next()
        .map(|refnum| refnum.get_child_nodes())
        .unwrap_or_default();
      for mut mark in document.findnodes("ltx:tags/ltx:tag//ltx:text[@class='ltx_cref_number_mark']", Some(&target)) {
        document.append_clone(&mut mark, number.clone())?;
        document.unwrap_nodes(mark)?;
      }
      document.set_node(&savenode);
    }
  },
  reversion => "",
  // (a label makes no box, as the kernel's `\lx@label`: KNOWN_PERL_ERRORS #1144)
  sizer => 0,
  properties => sub[args] {
    let ty = args[0].as_ref().map(|a| a.to_string()).unwrap_or_default();
    let ty = Tokens::new(ExplodeText!(ty.trim()));
    // Each built as `\lx@make@tags` builds the target's (base_utilities.rs `tag_forms`): a text-mode tag in the
    // neutral font, wherever the label stands (an equation, a theorem's italic body); a form only a reference prints
    // holds its diagnostics (`type_tag_deferred`, `\lx@tag@intags@held`), as the target's own does. The label type's
    // number forms (`creflabel`, the document's formats) replace the object's type's, an empty one (the label type's
    // label format printing the number as it is, no format of the document's) removing it.
    // (the number the object's: cleveref's `\label[<type>]` changes the type of the label it writes, not its number,
    // cleveref.sty:82-96 `\cref@override@label@type`; a mark the constructor replaces with the object's refnum,
    // `cref_number`)
    assign_value("cleveref_label_number", Tokens!(T_CS!("\\lx@cref@numbermark")), Some(Scope::Global));
    let tags = cref_label_type_tags(&ty);
    assign_value("cleveref_label_number", Stored::None, Some(Scope::Global));
    let tags = tags?;
    let mut props = stored_map!("cref_tags" => Stored::VecDigested(tags),
      "alignmentSkippable" => true, "alignmentPreserve" => true);
    // (the object `\lx@label` names after a raw step, `current_raw_refstep`; 2506.01072, 2505.20088)
    if let Some(step) = current_raw_refstep() {
      props.insert("raw_refstep_tags", Stored::Digested(step.tags));
      props.insert("raw_refstep_serial", Stored::Int(step.serial));
    }
    if label_follows_an_undone_step() {
      props.insert("after_undone_step", Stored::Bool(true));
    }
    Ok(props)
  });

  // Load the raw cleveref.sty (for language-dependent definitions)
  // Pretend amsmath is loaded to avoid errors. Per OXIDIZED_DESIGN
  // #23, "amsmath is loaded" means EITHER `amsmath.sty_loaded` OR
  // `amsmath.sty_raw_loaded` is set.
  // (a flag set `false`, `latexml_core::state::mark_unloaded`'s — amsmath loaded beyond a class's original,
  // `amsmath_sty::require_amsmath_beyond_original` — is not loaded)
  let ams_loaded = lookup_bool("amsmath.sty_loaded") || lookup_bool("amsmath.sty_raw_loaded");
  assign_value("amsmath.sty_loaded", true, Some(Scope::Local));
  // The pretense includes amsmath's `parentequation` counter (amsmath.sty:1127), which cleveref's amsmath branch reads
  // for every equation label (cleveref.sty:700-707): "\c@parentequation is not a register" otherwise.
  RawTeX!(r"\@ifundefined{c@parentequation}{\newcounter{parentequation}}{}");
  // (defined before cleveref is read: a cleveref loaded after `\begin{document}` runs its hooks at once; 2512.04983,
  // 2608.14769)
  // Which formats the document set: each `\crefformat`, `\Crefformat`, `\crefmultiformat`, `\Crefmultiformat`,
  // `\labelcrefformat` outside cleveref's own `\begin{document}` code, which builds the defaults (`\lx@cref@building`),
  // the other variant with it when that one is derived (:1888-1904), and a copy cleveref makes of one for a sub-type
  // (:1688-1736 `\@crefcopyformats`: `\crefformat{section}{\S#2#1#3}` is the subsection's too).
  DefPrimitive!("\\lx@cref@building{}", sub[(flag)] {
    let building = flag.to_string().trim() == "1";
    assign_value("cleveref_building_formats", building, Some(Scope::Global));
    // (cleveref's copies of a type's formats for its sub-types, cleveref.sty:8035-8046 `\@crefcopyformats`: a copy of
    // the document's format is the document's, `\crefformat{section}{\S#2#1#3}` the subsection's too)
    if !building {
      for (from, to) in CREF_COPIED_FORMATS {
        cref_note_copy(from, to);
      }
    }
    Ok(Vec::new())
  });
  DefPrimitive!("\\lx@cref@noteformat{}{}{}", sub[(prefix, ty, kind)] {
    if lookup_bool("cleveref_building_formats") {
      return Ok(Vec::new());
    }
    let (prefix, ctype, kind) = (prefix.to_string(), do_expand(ty)?.to_string(), kind.to_string());
    let ctype = ctype.trim();
    if let Some(other) = cref_other_variant(&prefix)
      && !cref_defined(&T_CS!(s!("\\{other}@{ctype}@{kind}")))
    {
      note_cref_doc_format(&other, ctype, &kind);
    }
    note_cref_doc_format(&prefix, ctype, &kind);
    Ok(Vec::new())
  });
  RawTeX!(r"\AtBeginDocument{\lx@cref@building{1}}");
  InputDefinitions!("cleveref", noltxml => true, extension => Some(Cow::Borrowed("sty")));
  if !ams_loaded {
    assign_value("amsmath.sty_loaded", Stored::None, Some(Scope::Local));
  }
  // (the document's formats noted, `\lx@cref@noteformat`, above; cleveref's own `\begin{document}` code, which builds
  // the defaults, bracketed by `\lx@cref@building`, its first hook before the file's and its last after: cleveref's
  // builders are left as they are, which packages patch (clevethm's `\patchcmd\@crefdefineformat`, 2608.12136))
  RawTeX!(r"\let\lx@cref@raw@crefformat\@crefformat
\def\@crefformat#1#2{\lx@cref@noteformat{#1}{#2}{format}\lx@cref@raw@crefformat{#1}{#2}}
\let\lx@cref@raw@crefmultiformat\@crefmultiformat
\def\@crefmultiformat#1#2{\lx@cref@noteformat{#1}{#2}{format@first}\lx@cref@noteformat{#1}{#2}{format@second}%
\lx@cref@noteformat{#1}{#2}{format@middle}\lx@cref@noteformat{#1}{#2}{format@last}\lx@cref@raw@crefmultiformat{#1}{#2}}
\let\lx@cref@raw@labelcrefformat\labelcrefformat
\def\labelcrefformat#1{\lx@cref@noteformat{labelcref}{#1}{format}\lx@cref@raw@labelcrefformat{#1}}
\AtBeginDocument{\lx@cref@building{0}}");
  // cleveref's `\appendix` (cleveref.sty:174-287; its hyperref form :2063-2175 and its revtex4/revtex4-1 form
  // :2276-2367, each wrapping the `\appendix` it found, `\cref@old@appendix`) aliases the sectioning counters to the
  // appendix types, by counter name, never undone: a section stepped after it is an `appendix` (a subsection a
  // `subappendix`, …; in a class with chapters the chapter is the `appendix`) wherever it is, inside appendix.sty's
  // `{appendices}` too, whether the environment was opened after it (1810.06583, 1904.07153, 2208.03406) or around it
  // (2112.09636); before it, or with no `\appendix` at all, a section keeps its counter's type (2507.16108,
  // 2511.19260). The call is noted by a wrapper of the patched `\appendix`, as cleveref wraps the one it found, with
  // the branch it takes (`\@ifundefined{chapter}`); `cref_type` reads it (KNOWN_PERL_ERRORS #1145). The note is global;
  // cleveref's redefinition is global only in the form without hyperref or chapters (:178 `\gdef\refstepcounter@noarg`):
  // its chapter form (:227) and every hyperref and revtex form (:2066, :2115, :2279, :2319 `\def\H@refstepcounter`)
  // are local, which differs only for a unit stepped after the group holding `\appendix` closes (not modelled; not
  // checked against pdflatex, whose hyperref plug TL 2025 breaks with the first aid off; review rev79d2 finding 5).
  DefPrimitive!("\\lx@cref@appendix@aliases{}", sub[(top)] {
    assign_value(CREF_APPENDIX_ALIASES, top.to_string(), Some(Scope::Global));
  });
  RawTeX!(r"\@ifundefined{appendix}{}{\let\lx@cref@patched@appendix\appendix
\def\appendix{\@ifundefined{chapter}{\lx@cref@appendix@aliases{section}}{\lx@cref@appendix@aliases{chapter}}%
\lx@cref@patched@appendix}}");

  // Perl L29-30: AtBeginDocument(sub { Let('\label', '\lx@cleverref@label') })
  // Deferred so any later package's `\label` redefinition lands BEFORE
  // cleveref wraps it; eager Let here would clobber the wrong target.
  at_begin_document(TokenizeInternal!(r"\let\label\lx@cleverref@label"))?;

  // \crefname{type}{singular}{plural} / \Crefname{...}: register the cleveref type
  // name so \cref/\Cref render "<name> <num>". A faithful reimplementation of raw
  // cleveref's \@crefname core (see cref_define_name) replacing the former no-op
  // stubs. The raw macros use \toksdef/\expandafter chains that mis-consumed tokens
  // here; this clean port avoids them (the same approach thmtools_sty.rs already
  // uses for \declaretheorem[refname=]). An explicit \crefname now populates
  // \cref@<type>@name, so it wins over the theorem-heading fallback (OXIDIZED_DESIGN #131).
  DefPrimitive!("\\crefname{}{}{}", sub[(type_arg, sg, pl)] {
    cref_define_name("cref", type_arg, sg, pl)?;
    Ok(Vec::new())
  });
  DefPrimitive!("\\Crefname{}{}{}", sub[(type_arg, sg, pl)] {
    cref_define_name("Cref", type_arg, sg, pl)?;
    Ok(Vec::new())
  });
  // `\crefalias{<counter>}{<type>}` is raw cleveref's (cleveref.sty:1277-1278, `\def\cref@<counter>@alias{<type>}`),
  // read by `cref_type`: a label stepped by the counter is referenced as the type (`\crefalias{section}{appsec}` in
  // appendix.sty's `{appendices}` — after `\appendix` the type is the appendix's alias, `\crefalias{appendix}{…}` —,
  // enumitem lists in a theorem through thmtools' hooks). The binding's no-op (Perl's
  // cleveref.sty.ltxml:114-117 alike; KNOWN_PERL_ERRORS #796) kept every such label its counter's name. Witnesses
  // 1706.03576, 2210.04778, 2002.10700 (`\addtotheorempostheadhook[theorem]{\crefalias{thmlisti}{theorem}}`),
  // 2002.10118, 2106.14806 and 1810.01223 (`\crefalias{section}{appsec}`), 2506.14950 (`\crefalias{lemma}{theorem}`).

  // Helper: produces the literal `~` (U+007E) as text — a CS so an active `~`
  // can't collapse to a space during tokenization, and a direct Tbox string so
  // it does NOT go through the font map that would turn a `~` char into the
  // tilde accent ˜ (U+02DC). The literal `DefPrimitive!(_, "~")` form reverts the
  // Tbox to its own CS token (`\lx@tilde`), which then leaked into the `tex=`
  // attribute of a `\cref` inside math, where Perl reverts a plain `~`. Give the
  // Tbox a `~` reversion instead so `tex=` matches Perl while the `show`
  // attribute stays U+007E (html_feedback#6876).
  DefPrimitive!("\\lx@tilde", sub[_args] {
    Tbox::new(
      pin_static("~"),
      None,
      None,
      Tokens!(T_OTHER!("~")),
      SymHashMap::default(),
    )
  });

  // The `|` between a show's alternatives (crossref.rs `show_alternatives`), as text: a font's `|` (OT1's em dash) would
  // not read back.
  DefPrimitive!("\\lx@cref@or", sub[_args] {
    Tbox::new(
      pin_static("|"),
      None,
      None,
      Tokens!(T_OTHER!("|")),
      SymHashMap::default(),
    )
  });

  // \lx@cref: the core constructor for cleveref references
  // Perl: DefConstructor('\lx@cref OptionalMatch:* HyperVerbatim Semiverbatim', ...)
  // A label only another document holds prints what cleveref prints there from the type and number xr imported for it
  // (`external_cref_text`).
  DefConstructor!("\\lx@cref OptionalMatch:* {} Semiverbatim",
    "?#reftext(<ltx:ref class='#refclass' labelref='#label' show='#2' ?#refhref(href='#refhref')() _force_font='true'>\
     #reftext</ltx:ref>)(<ltx:ref labelref='#label' show='#2' ?#1(class='ltx_nolink')() _force_font='true'/>)",
    enter_horizontal => true,
    properties => sub[args] {
      let raw = args[2].as_ref().map(|a| a.to_string()).unwrap_or_default();
      let label = clean_label(raw.trim(), None).to_string();
      let show = args[1].as_ref().map(|a| a.to_string()).unwrap_or_default();
      let mut props = stored_map!("label" => label.clone());
      if let Some(text) = external_cref_text(&show, &label)? {
        external_ref_properties_with(&label, args[0].is_some(), text, &mut props);
      }
      Ok(props)
    });

  // Where a `\label[<type>]`'s number forms print the object's number (`\lx@cleverref@labeltype` puts its refnum there).
  DefConstructor!("\\lx@cref@numbermark", "<ltx:text class='ltx_cref_number_mark'/>");

  // The conjunction between two references of a `\cref` list, which crossref.rs `settle_cref_conjunctions` drops
  // where the reference after it prints a format of the document's holding its own, and unwraps elsewhere.
  DefConstructor!("\\lx@cref@conjunction{}", "<ltx:text class='ltx_cref_conjunction'>#1</ltx:text>",
    enter_horizontal => true);

  // \cref, \Cref: main user commands
  // Perl: DefMacro('\cref OptionalMatch:* Semiverbatim', sub { crefMulti(...) })
  DefMacro!("\\cref OptionalMatch:* Semiverbatim", sub[args] {
    let starred = !args[0].is_none();
    let label_tokens = args.into_iter().nth(1).and_then(|a| a.owned_tokens()).unwrap_or_default();
    Ok(cref_multi(starred, label_tokens, true, false, CREF_LABEL)?)
  });
  DefMacro!("\\Cref OptionalMatch:* Semiverbatim", sub[args] {
    let starred = !args[0].is_none();
    let label_tokens = args.into_iter().nth(1).and_then(|a| a.owned_tokens()).unwrap_or_default();
    Ok(cref_multi(starred, label_tokens, true, true, CREF_LABEL)?)
  });

  // \crefrange, \Crefrange
  // \crefrange: Perl uses ~ (ACTIVE) in expansion which becomes space via HyperVerbatim.
  // show="creftypeplural refnum" in Perl output (space, not tilde).
  // (both ends in the type's label format, `creflabel`: "eqs. (1) to (3)")
  DefMacro!("\\crefrange OptionalMatch:* Semiverbatim Semiverbatim",
    "\\lx@cref#1{creftypeplural~creflabel}{#2}\\crefrangeconjunction\\lx@cref#1{creflabel}{#3}");
  DefMacro!("\\Crefrange OptionalMatch:* Semiverbatim Semiverbatim",
    "\\lx@cref#1{creftypepluralcap~creflabel}{#2}\\crefrangeconjunction\\lx@cref#1{creflabel}{#3}");

  // Page refs (same as regular refs for now)
  DefMacro!("\\cpageref OptionalMatch:* Semiverbatim", sub[args] {
    let starred = !args[0].is_none();
    let label_tokens = args.into_iter().nth(1).and_then(|a| a.owned_tokens()).unwrap_or_default();
    Ok(cref_multi(starred, label_tokens, true, false, "refnum")?)
  });
  DefMacro!("\\Cpageref OptionalMatch:* Semiverbatim", sub[args] {
    let starred = !args[0].is_none();
    let label_tokens = args.into_iter().nth(1).and_then(|a| a.owned_tokens()).unwrap_or_default();
    Ok(cref_multi(starred, label_tokens, true, true, "refnum")?)
  });
  DefMacro!("\\cpagerefrange OptionalMatch:* Semiverbatim Semiverbatim",
    "\\lx@cref#1{creftype~refnum}{#2}\\crefrangeconjunction\\lx@cref#1{creftype~refnum}{#3}");
  DefMacro!("\\Cpagerefrange OptionalMatch:* Semiverbatim Semiverbatim",
    "\\lx@cref#1{creftypecap~refnum}{#2}\\crefrangeconjunction\\lx@cref#1{creftype~refnum}{#3}");

  // Name refs
  DefMacro!("\\namecref Semiverbatim",    "\\lx@cref{creftype}{#1}");
  DefMacro!("\\nameCref Semiverbatim",    "\\lx@cref{creftypecap}{#1}");
  DefMacro!("\\namecrefs Semiverbatim",   "\\lx@cref{creftypeplural}{#1}");
  DefMacro!("\\nameCrefs Semiverbatim",   "\\lx@cref{creftypepluralcap}{#1}");
  DefMacro!("\\lcnamecref Semiverbatim",  "\\lx@cref{creftype}{#1}");
  DefMacro!("\\lcnamecrefs Semiverbatim", "\\lx@cref{creftypeplural}{#1}");

  DefMacro!("\\labelcref Semiverbatim", sub[args] {
    let label_tokens = args.into_iter().next().and_then(|a| a.owned_tokens()).unwrap_or_default();
    Ok(cref_multi(false, label_tokens, false, false, CREF_LABEL)?)
  });
  DefMacro!("\\labelcpageref Semiverbatim", sub[args] {
    let label_tokens = args.into_iter().next().and_then(|a| a.owned_tokens()).unwrap_or_default();
    Ok(cref_multi(false, label_tokens, false, false, "refnum")?)
  });

  // Type formatter macros. `theorem_fallback` (the singular creftype/creftypecap
  // variants) supplies the surpass-Perl auto-naming — see cleverref_type_name.
  DefMacro!("\\lx@cleverrefnum@@{}", sub[args] {
    Ok(cleverref_type_name(&args[0].to_string(), "cref", "name", true))
  });
  DefMacro!("\\lx@cleverrefnumplural@@{}", sub[args] {
    Ok(cleverref_type_name(&args[0].to_string(), "cref", "name@plural", false))
  });
  DefMacro!("\\lx@cleverrefnumcap@@{}", sub[args] {
    Ok(cleverref_type_name(&args[0].to_string(), "Cref", "name", true))
  });
  DefMacro!("\\lx@cleverrefnumpluralcap@@{}", sub[args] {
    Ok(cleverref_type_name(&args[0].to_string(), "Cref", "name@plural", false))
  });

  // The number as `\cref` prints it: the target's refnum in its type's label format, `\cref@<type>@label{<number>}{}{}`
  // (`\creflabelformat`, cleveref.sty:1270-1272), else `\cref@default@label` (`\crefdefaultlabelformat`, :1264-1265;
  // :1329-1334 `\@crefconstructcomponents`): "(1)" for an equation (:7838-7842 `\creflabelformat{equation}{\textup
  // {(#2#1#3)}}`), where Perl's binding (cleveref.sty.ltxml:46-76) and the former one printed the refnum "1". `#2`/`#3`
  // are the hyperlink's ends, empty here: the whole reference is the link. A tag form like the type names, built with
  // the target's tags (base_utilities `tag_forms`); under `\label[<type>]` it is the label type's
  // (`\lx@cleverref@labeltype`). KNOWN_PERL_ERRORS #1073; guard
  // `perfect_kernel_batch78j::cleveref_equation_label_format`.
  // A type whose format prints the number as it is (the default `#2#1#3`) has no such tag: the refnum is the form
  // (crossref.rs `ref_fallbacks`), and the target's tags stay as they were. That is read from the format's definition,
  // not by expanding it, so the format is expanded once, where the tag is built (a `\creflabelformat{enumi}{label*=
  // \arabic*}` raised its diagnostics twice).
  DefMacro!("\\lx@creflabel@@{}", sub[(ty)] {
    let ctype = cref_type(&ty.to_string());
    if lookup_bool("cleveref_in_form") || cref_label_format_is_plain(&ctype) {
      return Ok(Tokens!());
    }
    Ok(cref_in_form(cref_label_format(&ctype, cref_number(ty)?)))
  });
  // A format typeset into a target's form makes no forms of its own: `\crefformat{footnote}{#2\footnotemark[#1]#3}`
  // (1601.06262, 1711.01177, 2005.13962, 2008.04624), which pdflatex typesets only at a `\cref`, made a footnote mark
  // whose forms made a mark again (Fatal `Stomach:Recursion`). Local to the form's group.
  DefPrimitive!("\\lx@cref@inform", sub[_args] {
    assign_value("cleveref_in_form", true, Some(Scope::Local));
    Ok(Vec::new())
  });

  // A reference format the document sets replaces the whole reference: `\crefformat{<type>}{<format>}` (cleveref.sty
  // :1851, :1886-1906 `\@crefformat`) is what `\cref` prints for one label of that type, `#1` its number and `#2`/`#3`
  // the link's ends (cleveref.sty:1047-1071 `\@@setcref`), so `\crefformat{equation}{#2#1#3}` prints "1" and
  // `\Crefformat{equation}{Equation~#2#1#3}` "Equation 1", with no name and no label format of cleveref's. cleveref
  // builds the format of every type the document set none for from its names and label format at `\begin{document}`
  // (:8020-8032 `\@crefdefineformat`); that default is our `creftype~creflabel`. A format the document set is a tag
  // form of its own, built with the target's tags (`crefformat`, `crefformatcap`; the multi-reference formats of
  // `\crefmultiformat`, `crefmulti<position>`), whole in the link: the reference prints it when its target has one
  // (crossref.rs `generate_ref_aux`'s `|` alternatives), else the default. `\labelcref` prints the format `\labelcref
  // format` set (`labelcrefformat`), and the bare number for a type whose `\crefformat` the document set, for which
  // cleveref builds no `\labelcref` format (:1438-1440, `\labelcref@default@format`). KNOWN_PERL_ERRORS #1073,
  // OXIDIZED_DESIGN_DIVERGENCES #845; witnesses 2004.14422, 2312.00252, 2406.15983; guard
  // `perfect_kernel_batch78j::cleveref_document_formats_replace_the_reference`.
  DefPrimitive!("\\lx@cref@docformat{}{}{}", sub[(prefix, kind, ty)] {
    let ctype = cref_type(&ty.to_string());
    let (prefix, kind) = (prefix.to_string(), kind.to_string());
    if lookup_bool("cleveref_in_form") {
      return Ok(Vec::new());
    }
    let number = cref_number(ty)?;
    let format = if prefix == "labelcref" && kind == "multi" {
      // (a `\labelcref` list of a type whose `\crefmultiformat` the document set prints each number in the default
      // `\labelcref` format: cleveref builds a type's `\labelcref` multi formats only where the type has no
      // `\cref` multi format yet, cleveref.sty:8028-8030, :1543-1545, and else reads `\labelcref@default@format@first`
      // …, :1620-1669, the number as it is: "1 and 3" under siamart's `\crefmultiformat{equation}`)
      if !cref_doc_format_set("cref", &ctype, "format@first") {
        return Ok(Vec::new());
      }
      Some(T_CS!("\\labelcref@default@format")).filter(cref_defined)
    } else if prefix == "labelcref" {
      if !(cref_doc_format_set("labelcref", &ctype, &kind) || cref_doc_format_set("cref", &ctype, &kind)) {
        return Ok(Vec::new());
      }
      [s!("\\labelcref@{ctype}@{kind}"), s!("\\labelcref@default@{kind}")]
        .into_iter()
        .map(|cs| T_CS!(cs))
        .find(cref_defined)
    } else {
      Some(T_CS!(s!("\\{prefix}@{ctype}@{kind}"))).filter(|cs| cref_doc_format_set(&prefix, &ctype, &kind) && cref_defined(cs))
    };
    // (a format that is text around the number: one that computes at the reference — reads the label being
    // referenced, `\StrCount{\@lastusedlabel}…\nameCref{\ThmLabel}` (2608.12136, a Fatal PushbackLimit at the
    // target), `\cref`s itself — gives no form, and the reference prints cleveref's default)
    let Some(format) = format.filter(cref_format_is_text) else {
      return Ok(Vec::new());
    };
    // A form holds the format's text, not an object of its own: a format that typesets a numbered object
    // (`\crefformat{footnote}{#2\footnotemark[#1]#3}`, a mark with its own tags: 1601.06262, 1711.01177, 2005.13962,
    // 2008.04624), which pdflatex typesets only at the `\cref`, gives no form, and the reference prints cleveref's
    // default.
    // The form is built ahead of its use, as cleveref typesets it only at a reference: its diagnostics are the held
    // tag's (`\lx@tag@intags@held`, raised where a reference prints the form, `replay_shown_deferred`), and a form
    // whose typesetting errs or leaves its groups unbalanced gives none — the reference prints cleveref's default.
    // The derived `\Cref` variant of `\crefformat{figure}{#2\color{blue}Fig.~#1#3}` is `\MakeUppercase#2\color{blue}…`
    // (cleveref.sty:1886-1906), whose `\MakeUppercase` takes `\color` alone, an error in pdflatex too where a `\Cref`
    // prints it ("Argument of \@declaredcolor has an extra }"); built at the target, its open group was left to the
    // tag's `\endgroup` ("Attempt to close non-boxing group", the figure's mode unended, the reference "id1").
    // The form is digested in a group of its own, which undoes what its text leaves open, the alignment brace count
    // too (`stomach::digest_in_own_group`). The Error names the format, its type and where the target stands (the
    // label comes after the target's tags are built). Witnesses 2008.12750, 2604.24399; guard
    // `perfect_kernel_batch78j::cleveref_document_format_opening_a_group`.
    let hold = util::logger::DiagnosticsHold::begin();
    let (form, balance) = digest_in_own_group(|| digest(cref_in_form(cref_apply_format(format, number))))?;
    let form = form?;
    let balanced = balance == 0;
    if !balanced && hold.errors_raised() == 0 {
      let problem = if balance > 0 {
        s!("leaves {balance} group(s) open")
      } else {
        s!("closes a group it did not open")
      };
      let at = get_source_location().map(|at| s!(" ({at})")).unwrap_or_default();
      Error!(
        "unexpected",
        format,
        s!("The cleveref format of the {ctype}{at} {problem}; a reference to it prints cleveref's default")
      );
    }
    // A stray `&` errs where it is constructed (tex_tables.rs): typeset here, its Error is the form's, held with it,
    // and the tab is dropped as TeX drops it ("Misplaced alignment tab character &", tex.web §792), "F 1" for
    // `\crefformat{figure}{F & #2#1#3}`.
    let clean = balanced && hold.errors_raised() == 0;
    if clean {
      for misplaced in remove_stray_tabs(&form) {
        if misplaced {
          Warn!("unexpected", "&", "Stray alignment \"&\" (in the cascade of a misplaced alignment command above)");
        } else {
          Error!("unexpected", "&", "Stray alignment \"&\"");
        }
      }
    }
    hold.commit();
    // (an erring form's diagnostics stay the held tag's, which an empty tag keeps until its target ties them to its
    // labels, or the document's end does: tags_refnums.rs `\lx@tag@intags`, sect11.rs `tie_held_tags`,
    // `settle_empty_held_tags`)
    if !clean || makes_numbered_object(&form) {
      return Ok(Vec::new());
    }
    Ok(vec![form])
  });
  RawTeX!(r"\def\lx@crefformat@@#1{\lx@cref@docformat{cref}{format}{#1}}
\def\lx@crefformatcap@@#1{\lx@cref@docformat{Cref}{format}{#1}}
\def\lx@labelcrefformat@@#1{\lx@cref@docformat{labelcref}{format}{#1}}
\def\lx@crefmultifirst@@#1{\lx@cref@docformat{cref}{format@first}{#1}}
\def\lx@crefmultifirstcap@@#1{\lx@cref@docformat{Cref}{format@first}{#1}}
\def\lx@crefmultisecond@@#1{\lx@cref@docformat{cref}{format@second}{#1}}
\def\lx@crefmultimiddle@@#1{\lx@cref@docformat{cref}{format@middle}{#1}}
\def\lx@crefmultilast@@#1{\lx@cref@docformat{cref}{format@last}{#1}}
\def\lx@labelcrefmulti@@#1{\lx@cref@docformat{labelcref}{multi}{#1}}");

  // Register type_tag_formatter mappings
  for (role, formatter) in CREF_TAG_ROLES {
    AssignMapping!("type_tag_formatter", role => formatter);
  }
  AssignMapping!("type_tag_formatter", "creflabel" => "\\lx@creflabel@@");
  // The label format is typeset only where a reference prints it (cleveref.sty:1329-1334): its diagnostics wait for
  // that (`\lx@tag@intags@held`, ruling 7e), as hyperref's `autoref` name's do — 2506.11592's
  // `\creflabelformat{enumi}{label*=\arabic*}`, never referenced, raised 96 Warnings over its 48 items. So do the
  // document's formats.
  AssignMapping!("type_tag_deferred", "creflabel" => true);
});

/// Removes `tag`, a label type's tag `\lx@cleverref@labeltype` put in place, when it typeset nothing — unless it holds
/// diagnostics (an empty held tag, base_utilities `note_empty_held_tag`), which a label type's tags, put in place where
/// the label is read, may hold after their target closed (a `\label[<type>]` in a caption: the float already closed;
/// guard `perfect_kernel_batch77s::label_type_in_caption_text`): the document's end ties those to the target's labels
/// and removes them (latex_constructs `settle_empty_held_tags`).
fn remove_if_empty_held(document: &mut Document, tag: Node) {
  if tag.get_name() == "tag" && tag.get_first_child().is_none() && !tag.has_attribute("_deferred") {
    document.remove_node(tag);
  }
}

/// Whether document format `format` is text around its arguments, which a form at the target can hold: each command
/// of its body typesets (not expandable, protected, or robust, `\protect\<cs>`, as `\textup`, `\ensuremath`), or is a
/// macro of no arguments made only of such (`\S`, `~`). A macro that computes (`\StrCount`, `\nameCref`, `\cref`, a
/// conditional) is not: cleveref typesets the format at the reference (cleveref.sty:1047-1071), where it reads what
/// the target cannot know.
fn cref_format_is_text(format: &Token) -> bool {
  let Ok(Some(defn)) = lookup_definition(format) else {
    return false;
  };
  let Some(ExpansionBody::Tokens(body)) = defn.get_expansion() else {
    return false;
  };
  // (the case changer cleveref puts first in a variant it derives, `\cref@othervariant`'s `\MakeUppercase`/
  // `\MakeLowercase`, cleveref.sty:1283-1295, :1888-1904, is cleveref's own: the variant is the text it changes)
  let body = body.unlist_ref();
  let body = match body.split_first() {
    Some((first, rest)) if [T_CS!("\\MakeUppercase"), T_CS!("\\MakeLowercase")].contains(first) => {
      rest
    },
    _ => body,
  };
  body.iter().all(|token| cref_text_token(token, true))
}

/// One token of a format's body as [`cref_format_is_text`] reads it; `deep`, whether a macro of no arguments may be
/// read through, once.
fn cref_text_token(token: &Token, deep: bool) -> bool {
  if !matches!(token.get_catcode(), Catcode::CS | Catcode::ACTIVE) {
    return true;
  }
  // (a command let to a character is that character: `\egroup`, whose `}` a format may close a group with that it did
  // not open, an Error where a reference prints it, as pdflatex's "Extra }, or forgotten \endgroup")
  if matches!(lookup_meaning(token), Some(Stored::Token(_))) {
    return true;
  }
  let Ok(Some(defn)) = lookup_definition(token) else {
    return false;
  };
  if !defn.is_expandable() || defn.is_protected() {
    return true;
  }
  if defn.get_conditional_type().is_some() {
    return false;
  }
  let Some(ExpansionBody::Tokens(body)) = defn.get_expansion() else {
    return false;
  };
  let body = body.unlist_ref();
  body.first() == Some(&T_CS!("\\protect"))
    || (deep && defn.get_num_args() == 0 && body.iter().all(|token| cref_text_token(token, false)))
}

/// Whether `digested` holds a numbered object of its own: an object with tags (a note mark's, `make_note_tags`; an
/// `\lx@tags`).
fn makes_numbered_object(digested: &Digested) -> bool {
  match digested.data() {
    DigestedData::List(list) => list.borrow().boxes.iter().any(makes_numbered_object),
    DigestedData::Whatsit(whatsit) => {
      let whatsit = whatsit.borrow();
      whatsit.get_definition().get_cs_name() == "\\lx@tags"
        || whatsit.get_property("tags").is_some()
        || whatsit
          .get_args()
          .iter()
          .flatten()
          .any(makes_numbered_object)
        || matches!(whatsit.get_property("body").as_deref(), Some(Stored::Digested(body)) if makes_numbered_object(body))
    },
    _ => false,
  }
}

/// Removes the stray alignment tabs of a digested form (`&` outside an alignment, tex_tables.rs, an error where it is
/// constructed); for each, whether it was in the cascade of a misplaced alignment command (a Warning there).
fn remove_stray_tabs(digested: &Digested) -> Vec<bool> {
  let is_tab = |boxed: &Digested| match boxed.data() {
    DigestedData::Whatsit(whatsit) => {
      let whatsit = whatsit.borrow();
      (whatsit.get_definition().get_cs().get_catcode() == Catcode::ALIGN)
        .then(|| whatsit.get_property("misplaced_cascade").is_some())
    },
    _ => None,
  };
  let mut removed = Vec::new();
  match digested.data() {
    DigestedData::List(list) => {
      list.borrow_mut().boxes.retain(|boxed| match is_tab(boxed) {
        Some(misplaced) => {
          removed.push(misplaced);
          false
        },
        None => true,
      });
      for boxed in list.borrow().boxes.iter() {
        removed.extend(remove_stray_tabs(boxed));
      }
    },
    DigestedData::Whatsit(whatsit) => {
      let whatsit = whatsit.borrow();
      for arg in whatsit.get_args().iter().flatten() {
        removed.extend(remove_stray_tabs(arg));
      }
      if let Some(Stored::Digested(body)) = whatsit.get_property("body").as_deref() {
        removed.extend(remove_stray_tabs(body));
      }
    },
    _ => {},
  }
  removed
}

/// A form's tokens after `\lx@cref@inform`, which keeps the forms its typesetting makes from making forms.
fn cref_in_form(tokens: Tokens) -> Tokens {
  let mut out = vec![T_CS!("\\lx@cref@inform")];
  out.extend(tokens.unlist());
  Tokens::new(out)
}

/// `format` applied to `number`: `<format>{{<number>}}{}{}`, `#2`/`#3`, the hyperlink's ends, empty (the whole
/// reference is the link). `#1` is one group, as cleveref's is one macro (`\@templabel`, cleveref.sty:1071): the
/// derived `\Cref` format `\MakeUppercase#2#1#3` (:1888-1904) takes it whole, where the bare refnum's `\lx@therefnum@@`
/// alone was taken from its argument ("You can't use `}' after \the"; 2307.12991, 2408.03413
/// `\crefformat{appendix}{#2#1#3}`).
fn cref_apply_format(format: Token, number: Tokens) -> Tokens {
  let mut tokens = vec![format, T_BEGIN!(), T_BEGIN!()];
  tokens.extend(number.unlist());
  tokens.extend([
    T_END!(),
    T_END!(),
    T_BEGIN!(),
    T_END!(),
    T_BEGIN!(),
    T_END!(),
  ]);
  Tokens::new(tokens)
}

/// `number` in the label format of cleveref type `ctype`: `\cref@<ctype>@label{<number>}{}{}` (`\creflabelformat`,
/// cleveref.sty:1270-1272), else `\cref@default@label` (`\crefdefaultlabelformat`, :1264-1265), as
/// `\@crefconstructcomponents` picks it (:1329-1334); `#2`/`#3`, the hyperlink's ends, empty.
fn cref_label_format(ctype: &str, number: Tokens) -> Tokens {
  let format = [s!("\\cref@{ctype}@label"), s!("\\cref@default@label")]
    .into_iter()
    .map(|cs| T_CS!(cs))
    .find(cref_defined);
  match format {
    Some(format) => cref_apply_format(format, number),
    None => number,
  }
}

/// Whether cleveref type `ctype`'s label format prints the number as it is: its definition's body is `#2#1#3` (or
/// `#1`), spaces aside. Read from the definition, so the format is not expanded here.
fn cref_label_format_is_plain(ctype: &str) -> bool {
  let format = [s!("\\cref@{ctype}@label"), s!("\\cref@default@label")]
    .into_iter()
    .map(|cs| T_CS!(cs))
    .find(cref_defined);
  let Some(format) = format else {
    return true;
  };
  let Ok(Some(defn)) = lookup_definition(&format) else {
    return false;
  };
  let Some(ExpansionBody::Tokens(body)) = defn.get_expansion() else {
    return false;
  };
  let arg = |t: &Token, n: &str| t.get_catcode() == Catcode::ARG && t.with_str(|s| s == n);
  let kept: Vec<&Token> = body
    .unlist_ref()
    .iter()
    .filter(|t| t.get_catcode() != Catcode::SPACE && !arg(t, "2") && !arg(t, "3"))
    .collect();
  kept.len() == 1 && arg(kept[0], "1")
}

/// The other variant of a format family, which cleveref derives when the document sets one (`\cref@othervariant`,
/// cleveref.sty:1283-1295): `cref` ↔ `Cref`; `labelcref` has none.
fn cref_other_variant(prefix: &str) -> Option<String> {
  match prefix {
    "cref" => Some("Cref".to_string()),
    "Cref" => Some("cref".to_string()),
    _ => None,
  }
}

/// Note that the document set format `\<prefix>@<ctype>@<kind>` (`cref_doc_format`), and that a reference of its
/// family may print one (`cleveref_doc_formats`, read by [`cref_multi`]).
fn note_cref_doc_format(prefix: &str, ctype: &str, kind: &str) {
  assign_mapping(
    "cref_doc_format",
    &s!("{prefix}@{ctype}@{kind}"),
    Some(true),
  );
  let family = cref_doc_family(prefix, kind);
  assign_value(
    &s!("cleveref_doc_formats:{family}"),
    true,
    Some(Scope::Global),
  );
  // The family's forms are built from the first format the document sets: a document that sets none builds none.
  for (role, role_prefix, role_kind) in CREF_DOC_FORMAT_ROLES {
    let role_family = cref_doc_family(role_prefix, role_kind);
    if role_family == family || (family == "single" && role_family == "label") {
      assign_mapping("type_tag_formatter", role, Some(cref_doc_formatter(role)));
      assign_mapping("type_tag_deferred", role, Some(true));
    }
  }
}

/// The top sectioning counter (`section`, or `chapter` in a class with chapters) once the document has called
/// cleveref's patched `\appendix`, which aliases the sectioning counters to the appendix types for the rest of the
/// document (cleveref.sty:174-287): set by `\lx@cref@appendix@aliases`, global and never cleared, read by `cref_type`.
/// (cleveref's redefinition is a `\gdef` only without hyperref or chapters, :178; its chapter form, :227, and its
/// hyperref and revtex forms, :2066, :2115, :2279, :2319, are local `\def`s, which differ only for a unit stepped
/// after the group holding `\appendix` closes.)
const CREF_APPENDIX_ALIASES: &str = "cleveref_appendix_aliases";

/// cleveref's copies of a type's formats for its sub-types at `\begin{document}` (cleveref.sty:8035-8046
/// `\@crefcopyformats`), made when the sub-type has no names or label format of its own.
const CREF_COPIED_FORMATS: [(&str, &str); 11] = [
  ("section", "subsection"),
  ("subsection", "subsubsection"),
  ("appendix", "subappendix"),
  ("subappendix", "subsubappendix"),
  ("figure", "subfigure"),
  ("table", "subtable"),
  ("equation", "subequation"),
  ("enumi", "enumii"),
  ("enumii", "enumiii"),
  ("enumiii", "enumiv"),
  ("enumiv", "enumv"),
];

/// A format of type `from` the document set, which cleveref copied to sub-type `to` (the same meaning), is the
/// document's for `to` too.
fn cref_note_copy(from: &str, to: &str) {
  for prefix in ["cref", "Cref", "labelcref"] {
    for kind in [
      "format",
      "format@first",
      "format@second",
      "format@middle",
      "format@last",
    ] {
      let (src, dest) = (
        T_CS!(s!("\\{prefix}@{from}@{kind}")),
        T_CS!(s!("\\{prefix}@{to}@{kind}")),
      );
      if cref_doc_format_set(prefix, from, kind)
        && !cref_doc_format_set(prefix, to, kind)
        && cref_defined(&dest)
        && x_equals(&src, &dest)
      {
        note_cref_doc_format(prefix, to, kind);
      }
    }
  }
}

/// Whether `cs` is defined as LaTeX's `\@ifundefined` tests it, which cleveref's formats are read by: a name `\csname`
/// made `\relax` (`\@ifundefined{labelcref@<type>@format}`, cleveref.sty:1438) is not.
fn cref_defined(cs: &Token) -> bool { has_meaning(cs) && !x_equals(cs, &T_CS!("\\relax")) }

/// Whether the document set format `\<prefix>@<ctype>@<kind>`.
fn cref_doc_format_set(prefix: &str, ctype: &str, kind: &str) -> bool {
  lookup_mapping("cref_doc_format", &s!("{prefix}@{ctype}@{kind}")).is_some()
}

/// The reference a format is for: one label (`single`), several (`multi`), `\labelcref` (`label`).
fn cref_doc_family(prefix: &str, kind: &str) -> &'static str {
  match (prefix, kind) {
    ("labelcref", _) => "label",
    (_, "format") => "single",
    _ => "multi",
  }
}

/// Whether the document set any format of `family` ([`cref_doc_family`]); a document that set none prints its
/// references as before.
fn cref_doc_family_set(family: &str) -> bool { lookup_bool(&s!("cleveref_doc_formats:{family}")) }

/// The tag roles of the formats the document sets, with each one's format family and kind: `\cref`'s and `\Cref`'s
/// for one label, `\labelcref`'s, and the multi-reference ones by position (cleveref.sty:1000-1018; `\Cref`'s capital
/// variant for the first, cleveref's other positions being the `\cref` ones unless the document set them apart).
/// `\labelcref`'s in a list, for a type whose multi format the document set (`labelcrefmulti`, its family the
/// `\cref` multi formats').
const CREF_DOC_FORMAT_ROLES: [(&str, &str, &str); 9] = [
  ("crefformat", "cref", "format"),
  ("crefformatcap", "Cref", "format"),
  ("labelcrefformat", "labelcref", "format"),
  ("crefmultifirst", "cref", "format@first"),
  ("crefmultifirstcap", "Cref", "format@first"),
  ("crefmultisecond", "cref", "format@second"),
  ("crefmultimiddle", "cref", "format@middle"),
  ("crefmultilast", "cref", "format@last"),
  ("labelcrefmulti", "cref", "format@first"),
];

/// The formatter macro of a document-format role.
fn cref_doc_formatter(role: &str) -> String { s!("\\lx@{role}@@") }

/// The roles a `\label[<type>]` replaces with its type's: the type names, the label format and the document's formats
/// (those whose family the document set, `note_cref_doc_format`).
fn cref_label_type_roles() -> Vec<(&'static str, String)> {
  CREF_TAG_ROLES
    .iter()
    .map(|(role, formatter)| (*role, formatter.to_string()))
    .chain(std::iter::once((
      CREF_LABEL,
      "\\lx@creflabel@@".to_string(),
    )))
    .chain(
      CREF_DOC_FORMAT_ROLES
        .iter()
        .filter(|(role, ..)| lookup_mapping("type_tag_formatter", role).is_some())
        .map(|(role, ..)| (*role, cref_doc_formatter(role))),
    )
    .collect()
}

/// The tags of a `\label[<type>]`'s roles ([`cref_label_type_roles`]), each built as `\lx@make@tags` builds the
/// target's (base_utilities `tag_forms`), held where the role's are (`type_tag_deferred`).
fn cref_label_type_tags(ty: &Tokens) -> Result<Vec<Digested>> {
  let mut tags = Vec::new();
  for (role, formatter) in cref_label_type_roles() {
    let constructor = if with_mapping("type_tag_deferred", role, |held| held.is_some()) {
      "\\lx@tag@intags@held"
    } else {
      "\\lx@tag@intags"
    };
    let mut tokens = TokenizeInternal!(r"\begingroup\let~\lx@tag@texttilde").unlist();
    tokens.extend([T_CS!(constructor), T_OTHER!("[")]);
    tokens.extend(ExplodeText!(role));
    tokens.extend([T_OTHER!("]"), T_BEGIN!(), T_CS!(formatter), T_BEGIN!()]);
    tokens.extend(ty.clone().unlist());
    tokens.extend([T_END!(), T_END!(), T_CS!("\\endgroup")]);
    tags.push(Digest!(Tokens::new(tokens))?);
  }
  Ok(tags)
}

/// The number a form of type `ty` prints: the target's refnum (`\lx@therefnum@@`), or under `\label[<type>]` the
/// object's own (`cleveref_label_number`: the mark `\lx@cleverref@labeltype` replaces with the object's refnum).
fn cref_number(ty: Tokens) -> Result<Tokens> {
  Ok(match lookup_tokens("cleveref_label_number") {
    Some(number) => {
      let mut tokens = vec![T_BEGIN!(), T_CS!("\\normalfont")];
      tokens.extend(number.unlist());
      tokens.push(T_END!());
      Tokens::new(tokens)
    },
    None => Invocation!(T_CS!("\\lx@therefnum@@"), vec![ty]),
  })
}

/// The tag form `\cref`, `\labelcref` and `\crefrange` print a target's number by (`\lx@creflabel@@`); the page
/// references print the refnum.
const CREF_LABEL: &str = "creflabel";

/// cleveref's tag roles (`type_tag_formatter`) and their formatters: the forms of a target's type `\cref` prints.
const CREF_TAG_ROLES: [(&str, &str); 4] = [
  ("creftype", "\\lx@cleverrefnum@@"),
  ("creftypeplural", "\\lx@cleverrefnumplural@@"),
  ("creftypecap", "\\lx@cleverrefnumcap@@"),
  ("creftypepluralcap", "\\lx@cleverrefnumpluralcap@@"),
];

/// Port of raw cleveref's `\@crefname` core (`\newcommand\crefname[3]` →
/// `\@crefname{cref}{type}{sg}{pl}{}`): define `\<prefix>@<type>@name` and
/// `\<prefix>@<type>@name@plural` from the singular/plural arguments. `prefix` is
/// `"cref"` (for `\crefname`) or `"Cref"` (for `\Crefname`). The names are stored as
/// **raw** token bodies (never expanded), so markup such as `\textsc{lemma}` survives,
/// matching cleveref's `\def`. The raw macro's `\toksdef`/`\expandafter` chains are not
/// reproduced — the same clean approach `thmtools_sty.rs` uses for
/// `\declaretheorem[refname=]`. The other variant, when undefined, is derived as
/// cleveref.sty:1296-1325 derives it (`\cref@othervariant`, :1283-1295): `\Cref@<type>@name` is
/// `\MakeUppercase` before the `\crefname` singular (its first token), and `\cref@…` from a lone
/// `\Crefname` is `\MakeLowercase` before it, or the name itself under `capitalise`
/// (`\crefname{appendixtable}{Appendix Table}{…}`: `\Cref` printed no name; 2404.11739).
fn cref_define_name(
  prefix: &str,
  type_arg: Tokens,
  singular: Tokens,
  plural: Tokens,
) -> Result<()> {
  let ctype = do_expand(type_arg)?.to_string();
  let ctype = ctype.trim();
  def_macro(
    T_CS!(s!("\\{}@{}@name", prefix, ctype)),
    None,
    singular.clone(),
    None,
  )?;
  def_macro(
    T_CS!(s!("\\{}@{}@name@plural", prefix, ctype)),
    None,
    plural.clone(),
    None,
  )?;
  let other = if prefix == "cref" { "Cref" } else { "cref" };
  if !has_meaning(&T_CS!(s!("\\{}@{}@name", other, ctype))) {
    let change_case = if prefix == "cref" {
      Some(T_CS!("\\MakeUppercase"))
    } else if IsDefined!(&T_CS!("\\if@cref@capitalise"))
      && Expand!(TokenizeInternal!(r"\if@cref@capitalise1\else0\fi"))
        .to_string()
        .trim()
        == "1"
    {
      None
    } else {
      Some(T_CS!("\\MakeLowercase"))
    };
    // Both names change case, or neither, as the singular is empty or not (cleveref.sty:1309 `\expandafter\ifx\@tempa
    // \@empty`, the singular alone): `\crefname{x}{}{xs}` derives the plural unchanged.
    let change_case = change_case.filter(|_| !singular.unlist_ref().is_empty());
    // cleveref applies the case change to the name's first token (`\expandafter\@tempd\@tempa`, :1311-1318): a letter
    // is changed alone ("eq." → "Eq."), a macro is read whole by `\MakeUppercase` (`\crefname{proposition}
    // {\nameProposition}…` → "PROPOSITION", 1704.08542). A command that takes its own argument (`\crefname{line}
    // {\text{line}}…`, 2604.22361; 2206.03535) would be split from it by the kernel's grouped `\MakeUppercase` (latex.ltx
    // :22385 `{{…}}`), an error in pdflatex too, but only where that variant is typeset; our tags typeset every variant,
    // so a name led by a command is changed whole, braced.
    let derive = |name: Tokens| -> Tokens {
      match change_case {
        Some(cs) => {
          let mut tokens = vec![cs];
          if name
            .unlist_ref()
            .first()
            .is_some_and(|t| t.code.is_active_or_cs())
          {
            tokens.push(T_BEGIN!());
            tokens.extend(name.unlist());
            tokens.push(T_END!());
          } else {
            tokens.extend(name.unlist());
          }
          Tokens::new(tokens)
        },
        _ => name,
      }
    };
    def_macro(
      T_CS!(s!("\\{}@{}@name", other, ctype)),
      None,
      derive(singular),
      None,
    )?;
    def_macro(
      T_CS!(s!("\\{}@{}@name@plural", other, ctype)),
      None,
      derive(plural),
      None,
    )?;
  }
  Ok(())
}

/// Resolve a cleveref type-name control sequence (`\cref@<type>@name`,
/// `\Cref@<type>@name@plural`, …) to the tokens the `type_tag_formatter` emits.
///
/// When the primary CS is undefined and `theorem_fallback` is set, fall back to the
/// theorem's stored heading `\lx@name@<type>`. This is a **surpass-Perl** divergence
/// (OXIDIZED_DESIGN #131): real cleveref patches `\@ynthm`/`\@xnthm`/`\@othm` so that
/// `\newtheorem{arch}{Architecture}` auto-registers "Architecture" as the cref name,
/// but LaTeXML's `\newtheorem` (`define_new_theorem`) is a native primitive that never
/// routes through those patches — so Perl and Rust alike leave `\cref@arch@name`
/// undefined and `\cref{...}` renders bare "1" instead of "Architecture 1". LaTeXML
/// already stores the heading as `\lx@name@<type>`, so the singular creftype/creftypecap
/// formatters reuse it, matching the PDF. Only the *singular* names get the fallback:
/// cleveref's theorem patches set only `cref@<type>@name@preamble` (never `@plural`).
/// The heading is emitted verbatim; cleveref's first-letter `capitalize` case transform
/// is not reproduced, so a lowercase-`\cref` under the default (non-`capitalize`) option
/// keeps the heading's own case.
///
/// `\lx@name@<type>` is also set by `\floatname`/`\newfloat` (`float_sty.rs`), so custom
/// floats get the same auto-naming — matching real cleveref; standard figure/table/equation
/// keep their raw-cleveref primary name (fallback stays dormant). An explicit `\crefname`
/// (now a real definition — see `cref_define_name`) populates the primary CS, so it wins
/// over this fallback. Witness arXiv 2305.10391 (`\usepackage[capitalize,…]{cleveref}` +
/// `\newtheorem{arch}{Architecture}`).
fn cleverref_type_name(
  type_arg: &str,
  prefix: &str,
  suffix: &str,
  theorem_fallback: bool,
) -> Tokens {
  cref_name_of(&cref_type(type_arg), prefix, suffix, theorem_fallback)
}

/// `\<prefix>@<ctype>@<suffix>`, cleveref's name of the type `ctype`, else (`theorem_fallback`) the type's own name
/// (`\lx@name@<ctype>`), else nothing.
fn cref_name_of(ctype: &str, prefix: &str, suffix: &str, theorem_fallback: bool) -> Tokens {
  let cs = s!("\\{}@{}@{}", prefix, ctype, suffix);
  if has_meaning(&T_CS!(&cs)) {
    return Tokens!(T_CS!(&cs));
  }
  if theorem_fallback {
    let name_cs = s!("\\lx@name@{}", ctype);
    if has_meaning(&T_CS!(&name_cs)) {
      return Tokens!(T_CS!(&name_cs));
    }
  }
  Tokens!()
}

/// Perl: crefType($type) — resolve type alias. cleveref keys an alias by the counter (`\cref@<counter>@alias`,
/// cleveref.sty:1277-1278, read at `\refstepcounter`, :147-170): a type with no alias of its own takes its
/// counter's (`counter_for_type`: a section of appendix.sty's `{appendices}` is stepped by `section`, so
/// `\crefalias{section}{appsec}` there names it, as pdflatex prints; 2002.10118, 2106.14806, 1810.01223).
///
/// After cleveref's `\appendix` ([`CREF_APPENDIX_ALIASES`]) a sectioning unit's counter is aliased to its appendix type
/// (`appendix_type`, cleveref.sty:183-224, :232-284), else an appendix unit is its counter's type: appendix.sty's
/// `{appendices}` letters the sections without `\appendix`, so pdflatex names such a section "Sec. A" (2507.16108,
/// 2511.19260), and once `\appendix` has run every section is an `appendix`, inside the environment or not ("appendix
/// A": 1810.06583, 1904.07153, 2112.09636, 2208.03406; KNOWN_PERL_ERRORS #1145).
fn cref_type(ctype: &str) -> String {
  let counter: Option<String> = lookup_mapping("counter_for_type", ctype)
    .as_ref()
    .map(String::from);
  let top = lookup_string(CREF_APPENDIX_ALIASES);
  if !top.is_empty() {
    if let Some(appendix_type) = appendix_type(&top, counter.as_deref().unwrap_or(ctype)) {
      let alias_cs = s!("\\cref@{}@alias", appendix_type);
      if has_meaning(&T_CS!(&alias_cs))
        && let Ok(expanded) = do_expand(Tokens!(T_CS!(&alias_cs)))
      {
        return expanded.to_string();
      }
      return appendix_type.to_string();
    }
  } else if ctype == "appendix"
    && let Some(counter) = counter
  {
    let alias_cs = s!("\\cref@{}@alias", counter);
    if has_meaning(&T_CS!(&alias_cs))
      && let Ok(expanded) = do_expand(Tokens!(T_CS!(&alias_cs)))
    {
      return expanded.to_string();
    }
    return counter;
  }
  for key in std::iter::once(ctype.to_string()).chain(counter) {
    let alias_cs = s!("\\cref@{}@alias", key);
    if has_meaning(&T_CS!(&alias_cs))
      && let Ok(expanded) = do_expand(Tokens!(T_CS!(&alias_cs)))
    {
      return expanded.to_string();
    }
  }
  ctype.to_string()
}

/// The appendix type cleveref's patched `\appendix` gives a sectioning counter's labels (cleveref.sty:183-224 without
/// chapters, `top` `section`; :232-284 with them, `top` `chapter`); other counters keep their own.
fn appendix_type(top: &str, counter: &str) -> Option<&'static str> {
  const TYPES: [&str; 4] = [
    "appendix",
    "subappendix",
    "subsubappendix",
    "subsubsubappendix",
  ];
  let levels: &[&str] = if top == "chapter" {
    &["chapter", "section", "subsection", "subsubsection"]
  } else {
    &["section", "subsection", "subsubsection"]
  };
  levels
    .iter()
    .position(|level| *level == counter)
    .map(|i| TYPES[i])
}

/// Perl: crefMulti($starred, $labels, $showtype, $capitalized)
/// Generates tokens for \cref{label1,label2,...}
/// Trim leading/trailing SPACE tokens from a label group.
fn trim_space_tokens(tokens: Tokens) -> Tokens {
  let mut v = tokens.unlist();
  while v
    .first()
    .map(|t| t.get_catcode() == Catcode::SPACE)
    .unwrap_or(false)
  {
    v.remove(0);
  }
  while v
    .last()
    .map(|t| t.get_catcode() == Catcode::SPACE)
    .unwrap_or(false)
  {
    v.pop();
  }
  Tokens::new(v)
}

/// Split a Semiverbatim label argument on top-level OTHER commas, trimming
/// surrounding spaces and dropping empties. Mirrors Perl cleveref `splitLabels`,
/// which splits the **Tokens** object — NOT a stringified form. Critical for
/// labels whose name contains a control word, e.g.
/// `\cref{… the \SW moduli space …}` (`\SW` is a user `\newcommand`): the space
/// after the control word `\SW` is consumed at tokenization, so the tokens are
/// `… \SW moduli …` (no space token between). Stringifying and re-tokenizing
/// (as the old code did via `\lx@cref{…}{label-string}`) rejoins them into a
/// bogus `\SWmoduli` control sequence that then gets DIGESTED → `undefined`.
/// Keeping the original tokens and splicing them straight into the `\lx@cref`
/// invocation (as Perl's `Invocation(T_CS('\lx@cref'), …, $label_tokens)` does)
/// avoids the round-trip entirely. Witness 1704.05859.
fn split_label_tokens(tokens: Tokens) -> Vec<Tokens> {
  let mut groups: Vec<Tokens> = Vec::new();
  let mut cur: Vec<Token> = Vec::new();
  for t in tokens.unlist() {
    if t.get_catcode() == Catcode::OTHER && t.with_str(|s| s == ",") {
      groups.push(Tokens::new(std::mem::take(&mut cur)));
    } else {
      cur.push(t);
    }
  }
  groups.push(Tokens::new(cur));
  groups
    .into_iter()
    .map(trim_space_tokens)
    .filter(|g| !g.is_empty())
    .collect()
}

/// What `\cref` prints for a label only another document holds, from the type and number of its `\r@<label>@cref`, which
/// xr imports (`record_external_cref`): the roles of the reference's show string in turn — the type's names from this
/// document's `\crefname`s (`creftype`, …), the number in the type's label format (`creflabel`) or as it is (`refnum`),
/// the format the document set for the type (`crefformat`, `crefformatcap`, `labelcrefformat`) — as cleveref builds the
/// reference from the label's type and number (cleveref.sty:1047-1071 `\@@setcref`). The first alternative whose roles
/// can all be given is printed (the document's format where it set one, else cleveref's default). A label of the other
/// document with no cleveref record prints "??" in pdflatex, and is left a missing label. Witnesses 2412.14256,
/// 2508.18808, 2608.12136 (their supplements' `\cref` targets).
fn external_cref_text(show: &str, label: &str) -> Result<Option<Digested>> {
  let (Some(ctype), Some(number)) = (
    external_label_field(label, "creftype"),
    external_label_tex(label, "crefnum"),
  ) else {
    return Ok(None);
  };
  let show = show.replace("\\lx@tilde", "~").replace("\\lx@cref@or", "|");
  'alternatives: for alternative in show.split('|') {
    let mut tokens = Vec::new();
    for (index, role) in alternative.split('~').enumerate() {
      if index > 0 {
        tokens.push(T_ACTIVE!('~'));
      }
      let part = match role.trim() {
        "creftype" => cref_name_of(&ctype, "cref", "name", true),
        "creftypeplural" => cref_name_of(&ctype, "cref", "name@plural", false),
        "creftypecap" => cref_name_of(&ctype, "Cref", "name", true),
        "creftypepluralcap" => cref_name_of(&ctype, "Cref", "name@plural", false),
        "creflabel" if cref_label_format_is_plain(&ctype) => number.clone(),
        "creflabel" => cref_label_format(&ctype, number.clone()),
        "refnum" => number.clone(),
        role @ ("crefformat" | "crefformatcap" | "labelcrefformat") => {
          let prefix = match role {
            "crefformatcap" => "Cref",
            "labelcrefformat" => "labelcref",
            _ => "cref",
          };
          let format = T_CS!(s!("\\{prefix}@{ctype}@format"));
          if !(cref_doc_format_set(prefix, &ctype, "format")
            && cref_defined(&format)
            && cref_format_is_text(&format))
          {
            continue 'alternatives;
          }
          cref_apply_format(format, number.clone())
        },
        _ => continue 'alternatives,
      };
      tokens.extend(part.unlist());
    }
    return Ok(Some(digest(Tokens::new(tokens))?));
  }
  Ok(None)
}

fn cref_multi(
  starred: bool,
  label_tokens: Tokens,
  showtype: bool,
  capitalized: bool,
  number: &str,
) -> Result<Tokens> {
  let labels = split_label_tokens(label_tokens);
  let n = labels.len();
  let mut out: Vec<Token> = Vec::new();

  // Emit one `\lx@cref [*] {<show>} {<label tokens>}`. The `<show>` is a fixed
  // internal string (no user macros) so tokenizing it is safe; the `<label>`
  // is spliced from the ORIGINAL tokens (never re-tokenized), matching Perl's
  // `Invocation(T_CS('\lx@cref'), $starred, T_OTHER(show), $label)`.
  // `\lx@tilde` carries the inter-word `~` (HyperVerbatim in Perl).
  let emit = |out: &mut Vec<Token>, show: &str, label: &Tokens| {
    out.push(T_CS!("\\lx@cref"));
    if starred {
      out.push(T_OTHER!("*"));
    }
    out.push(T_BEGIN!());
    out.extend(mouth::tokenize_internal(TeXString::assembled(show.to_string())).unlist());
    out.push(T_END!());
    out.push(T_BEGIN!());
    out.extend(label.clone().unlist());
    out.push(T_END!());
  };

  // The forms the document's formats give (`\lx@cref@docformat`), first when the document set any of the family:
  // the target prints its own when it has one, else cleveref's default (crossref.rs `generate_ref_aux`).
  let labelled = number == CREF_LABEL;
  let single_doc = labelled && cref_doc_family_set("single");
  let label_doc = labelled && (cref_doc_family_set("label") || cref_doc_family_set("single"));
  let multi_doc = labelled && showtype && cref_doc_family_set("multi");
  let label_multi_doc = labelled && !showtype && cref_doc_family_set("multi");
  let prefer = |doc: bool, role: &str, show: String| {
    if doc {
      s!("{role}\\lx@cref@or {show}")
    } else {
      show
    }
  };
  if n < 2 {
    let show = if showtype {
      if capitalized {
        prefer(
          single_doc,
          "crefformatcap",
          s!("creftypecap\\lx@tilde {number}"),
        )
      } else {
        prefer(single_doc, "crefformat", s!("creftype\\lx@tilde {number}"))
      }
    } else {
      prefer(label_doc, "labelcrefformat", number.to_string())
    };
    let empty = Tokens::new(Vec::new());
    let label = labels.first().unwrap_or(&empty);
    emit(&mut out, &show, label);
  } else {
    let show = if showtype {
      if capitalized {
        prefer(
          multi_doc,
          "crefmultifirstcap",
          s!("creftypepluralcap\\lx@tilde {number}"),
        )
      } else {
        prefer(
          multi_doc,
          "crefmultifirst",
          s!("creftypeplural\\lx@tilde {number}"),
        )
      }
    } else {
      prefer(label_multi_doc, "labelcrefmulti", number.to_string())
    };
    // A later position's show: its multi format's for `\cref`, the `\labelcref` list's for `\labelcref`.
    let later = |role: &str| {
      if showtype {
        prefer(multi_doc, role, number.to_string())
      } else {
        prefer(label_multi_doc, "labelcrefmulti", number.to_string())
      }
    };
    // A position's format holds its conjunction (cleveref.sty:1514-1519, `\crefpairconjunction` before the second
    // label's): the conjunction between is dropped where the next reference prints the document's format
    // (`\lx@cref@conjunction`, crossref.rs `settle_cref_conjunctions`).
    let conjunction = |out: &mut Vec<Token>, cs: &str| {
      if multi_doc {
        out.extend([
          T_CS!("\\lx@cref@conjunction"),
          T_BEGIN!(),
          T_CS!(cs),
          T_END!(),
        ]);
      } else {
        out.push(T_CS!(cs));
      }
    };
    emit(&mut out, &show, &labels[0]);
    if n == 2 {
      conjunction(&mut out, "\\crefpairconjunction");
      emit(&mut out, &later("crefmultisecond"), &labels[1]);
    } else {
      for label in &labels[1..n - 1] {
        conjunction(&mut out, "\\crefmiddleconjunction");
        emit(&mut out, &later("crefmultimiddle"), label);
      }
      conjunction(&mut out, "\\creflastconjunction");
      emit(&mut out, &later("crefmultilast"), &labels[n - 1]);
    }
  }
  Ok(Tokens::new(out))
}
