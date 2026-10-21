//! `LoadDefinitions!` part 3: the frontmatter field macros (author markup, field marks, abstract, keywords, contacts,
//! ORCID, thanks, notes).

use super::*;

pub(in crate::base_utilities) fn load() -> Result<()> {
  //======================================================================
  // Some shorthands?

  DefMacro!(
    "\\lx@add@title[]{}",
    "\\lx@clear@frontmatter{ltx:title}\\lx@add@frontmatter{ltx:title}[#1]{#2}"
  );
  DefMacro!(
    "\\lx@add@toctitle[]{}",
    "\\lx@clear@frontmatter{ltx:toctitle}\\lx@add@frontmatter{ltx:toctitle}[#1]{#2}"
  );
  DefMacro!(
    "\\lx@add@subtitle[]{}",
    "\\lx@clear@frontmatter{ltx:subtitle}\\lx@add@frontmatter{ltx:subtitle}[#1]{#2}"
  );

  // careful: the "name", #2, can contain much more than just the name!
  DefMacro!(
    "\\lx@add@creator [] {}",
    "\\lx@add@frontmatter{ltx:creator}[role=author,#1]{\\lx@personname{\\lx@author@withinst{#2}}}"
  );
  DefMacro!(
    "\\lx@add@author[]{}",
    "\\lx@add@frontmatter{ltx:creator}[role=author,#1]{\\lx@personname{\\lx@author@withinst{#2}}}"
  );
  // Author content is digested at `\author` time, before a class's title
  // box can define `\inst` (bfhsciposter.cls:445,476 sets it to a
  // superscript inside its title box; beamerbasetitle.sty:262
  // `\providecommand\inst[1]{}`), so `\inst` is provided for the author
  // content ONLY — the shape of Perl's `\lx@author@withsup`
  // (Base_Utility.pool.ltxml:729) — and with its frontmatter MEANING: an
  // affiliation-link request (llncs.cls.ltxml:50
  // `\lx@request@frontmatter@annotation[affiliation]`), never a typeset
  // superscript — except a footnote-SYMBOL mark (`\inst{*}`, `\inst{\dagger}`:
  // an equal-contribution note, not an affiliation number), which keeps its
  // glyph exactly as `rewrite_symbol_superscripts` keeps `$^{*}$` (OXIDIZED
  // #52), so the poster's `*` stays visible as pdflatex renders it.
  // `\providecommand`, so a class binding's own `\inst` wins; scoped, so a
  // class's later `\newcommand\inst` (ptptex.cls:616) is not blocked.
  // SURPASS, KNOWN_PERL_ERRORS #201 (Perl: `undefined:\inst` on the poster
  // witness, whose class defines `\inst` only inside its title box).
  // `\def`, not `\providecommand`: inside author content `\inst{n}` ALWAYS
  // means "link to affiliation n" (llncs' own `\inst` is this same request),
  // and K11 may have rerouted a raw class's document-level `\inst` (the
  // affiliation LIST setter, ptptex/jpsj2) before any author is read. A
  // save/restore pair, not a group: author content may carry a `\\` line
  // break (keyval2e-guide `\author{Name \Email{…}\\Preston, …}`), and an
  // extra `\bgroup…\egroup` around it adds one more mode-check error per
  // author.
  DefMacro!(
    "\\lx@author@withinst{}",
    "\\let\\lx@saved@inst\\inst\\def\\inst##1{\\lx@inst@mark{##1}}\\lx@author@markup@begin\\expandafter\\lx@author@markup@run\\expandafter{\\lx@author@markup@unprovide}{#1}\\let\\inst\\lx@saved@inst"
  );
  // A raw class's own author markup — `\fnms`/`\snm`/`\inits`/`\degs`/`\roles`, its `\orcid` — is often defined only
  // inside the class's `\author` (IOS-Book-Article.cls:1146-1184, econsocart.cls:2095-2160), which the locked kernel
  // `\author` replaces, so the author content met it undefined (IOS: 2407.04130, 2609.06231, 13776, 15113, 28673;
  // econsocart: 2609.06865; OXIDIZED_DESIGN_DIVERGENCES #444). Only when the lock refused an `\author`
  // (`\author:redefined`), each one still undefined takes, for the author content, OmniBus's meaning
  // (OmniBus.cls.ltxml:97-102: `\prefix`, `\suffix`, `\fnms`, `\snm`, `\inits`; `\orcid`, omnibus_cls.rs) or the
  // classes' own identity one (`\degs`, `\roles`: IOS-Book-Article.cls:1153-1174; `\particle`: econsocart.cls:2104) and
  // is undefined again after it; a class's or binding's own definition stands, and elsewhere an undefined one still
  // errors as in pdflatex. The restore list is captured before the content runs, so a nested author keeps it.
  DefMacro!(
    "\\lx@author@markup@provide{}{}",
    "\\ifx#1\\@undefined\\let#1#2\\expandafter\\def\\expandafter\\lx@author@markup@unprovide\\expandafter{\\lx@author@markup@unprovide\\let#1\\@undefined}\\fi"
  );
  DefMacro!("\\lx@author@markup@begin", sub[()] {
    // A binding's author markup for a class that defines its own only where its title code prints the author list
    // (`\lx@author@markup@class`, `\lx@author@markup@provide` calls: vmsta.cls:278-291 `\name@m@front`, which prints
    // `\fnm`/`\snm` as "Yuliya~Mishura" and drops `\inits`; latexml_contrib vmsta_cls.rs) comes first, before
    // OmniBus's identity meanings would take the names it gives.
    let mut out = TokenizeInternal!(r"\def\lx@author@markup@unprovide{}").unlist();
    if IsDefined!(&T_CS!("\\lx@author@markup@class")) {
      out.push(T_CS!("\\lx@author@markup@class"));
    }
    out.extend(if lookup_bool("\\author:redefined") {
      TokenizeInternal!(
        r"\lx@author@markup@provide\prefix\@firstofone\lx@author@markup@provide\suffix\@firstofone\lx@author@markup@provide\particle\@firstofone\lx@author@markup@provide\fnms\@firstofone\lx@author@markup@provide\snm\@firstofone\lx@author@markup@provide\inits\@firstofone\lx@author@markup@provide\degs\@firstofone\lx@author@markup@provide\roles\@firstofone\lx@author@markup@provide\orcid\lx@author@orcid"
      )
      .unlist()
    } else {
      Vec::new()
    });
    // The author markup a class defines only where its title code reads `\@author` is in force for the author content
    // too: melba.cls:279-283 `{\def\aff{…}\def\name{…}\@author …}` (2405.09787, 2407.21368, 2105.14711), dmlr2e.sty:
    // 249-251 (2404.08403), amta2024.sty:77-78 (2405.16969), ssArxiv.sty:109-111 (2503.22625), a preamble's own
    // `\@maketitle` (2601.05137). In pdflatex `\@author` expands inside that group, while here the author content is
    // read at the frontmatter, before the title code runs. Each definition of a control sequence still undefined runs,
    // in source order, for the author content, which it is undefined again after, as the OmniBus author markup is
    // (#444); a definition already in force stands. Perl errs the same (`undefined:\name`).
    let definitions = title_code_author_definitions();
    let undefined: Vec<bool> = definitions
      .iter()
      .map(|(definee, _)| lookup_meaning(definee).is_none())
      .collect();
    for ((definee, statement), undefined) in definitions.into_iter().zip(undefined) {
      if !undefined {
        continue;
      }
      out.extend(statement);
      out.extend(
        TokenizeInternal!(r"\expandafter\def\expandafter\lx@author@markup@unprovide\expandafter{\lx@author@markup@unprovide\let")
          .unlist(),
      );
      out.extend([definee, T_CS!("\\@undefined"), T_END!()]);
    }
    Ok(Tokens::new(out))
  });
  DefMacro!("\\lx@author@markup@run{}{}", "#2#1");
  DefMacro!("\\lx@author@orcid[]{}", "\\lx@add@orcid{#2}");
  DefMacro!("\\lx@inst@mark{}", sub[(label)] {
    let call = if is_footnote_symbol_operand(label.unlist_ref()) {
      Tokens::new(keepsup(label.unlist()))
    } else {
      Invocation!(
        T_CS!("\\lx@request@frontmatter@annotation"),
        vec![Some(Tokens!(T_LETTER!("a"), T_LETTER!("f"), T_LETTER!("f"), T_LETTER!("i"), T_LETTER!("l"), T_LETTER!("i"), T_LETTER!("a"), T_LETTER!("t"), T_LETTER!("i"), T_LETTER!("o"), T_LETTER!("n"))), Some(label)]
      )
    };
    Ok(call)
  });
  DefMacro!(
    "\\lx@add@editor[]{}",
    "\\lx@add@frontmatter{ltx:creator}[role=editor,#1]{\\lx@personname{#2}}"
  );
  DefMacro!(
    "\\lx@add@translator[]{}",
    "\\lx@add@frontmatter{ltx:creator}[role=translator,#1]{\\lx@personname{#2}}"
  );

  DefMacro!(
    "\\lx@add@date[]{}",
    "\\lx@clear@frontmatter{ltx:date}[role=created,#1]\\lx@add@frontmatter{ltx:date}[role=created,#1]{#2}"
  ); // no duplicates w/same role
  DefMacro!("\\lx@copyright@holder", "");
  DefMacro!("\\lx@copyright@date", "");
  DefMacro!("\\lx@add@copyright{}", "\\lx@add@date[role=copyright]{#1}");
  // Next two are for when copyright holder & year are given by 2 separate macros
  DefMacro!(
    "\\lx@add@copyrightholder{}",
    "\\gdef\\lx@copyright@holder{#1}\\lx@add@copyright{\\lx@copyright@holder\\ \\lx@copyright@date}"
  );
  // A year with no holder is the year alone: Perl's `\ifx.\lx@copyright@holder.` (Base_Utility.pool
  // .ltxml:606-608; aipproc's `\copyrightyear`) compares `.` with the macro and never holds, so it read ", 2020" (KPE #418).
  DefMacro!(
    "\\lx@add@copyrightyear{}",
    "\\gdef\\lx@copyright@date{#1}\\lx@add@copyright{\\ifx\\lx@copyright@holder\\@empty\\else\\lx@copyright@holder, \\fi\\lx@copyright@date}"
  );

  // Where the abstract SITS in the source: an empty internal marker constructed at
  // the current point when the abstract is queued (both the `{abstract}` env and a
  // class's `\abstract{…}` store route through the two macros below). The
  // abstract-only fallback reads it to tell title-page LAYOUT (what precedes the
  // abstract) from BODY (what follows it) — the flush point (first `\section`, or
  // `\end{document}`) cannot: a sectionless manual would otherwise see its whole
  // body as cover. `_Capture_`-suffixed names are admitted anywhere by the model
  // (model.rs) so no paragraph is auto-opened; every marker is removed by
  // `insert_frontmatter`/`remove_frontmatter_marks` before output. OXIDIZED_DESIGN #246.
  DefConstructor!("\\lx@frontmatter@mark{}", sub[document, _args] {
    // Only meaningful BEFORE the frontmatter is placed: a later abstract (lips'
    // `\DocInput`-ed `\begin{abstract}` after `\maketitle`) is handled by
    // `insert_late_frontmatter`, and a marker built then would outlive every
    // removal point (s107: 8 docs leaked `<_Frontmatter_Capture_>`).
    // …and only once there is an open element to sit in: a preamble `\abstract{…}`
    // (svjour's setter, fixture structure/svabstract) is constructed BEFORE the
    // whatsit that opens <ltx:document>, and inserting anything there auto-creates
    // the root early (a duplicated RelaxNGSchema PI). Construction runs after
    // digestion, so the digest-time `inPreamble` flag is useless here — ask the
    // tree via the current element (which, unlike the root element, also holds
    // inside a streaming fragment; 114_streaming_structure faketitlepage).
    if !lookup_bool("frontmatter_done") && document.get_element().is_some() {
      document.insert_element("ltx:_Frontmatter_Capture_", Vec::new(), None)?;
    }
  });
  // Field provenance (frontmatter model M3, stage S1): `\lx@field@mark<prefix><store>{<field>}` — the store, a macro
  // without parameters, redefined (`<prefix>` `\def` or `\gdef`, the store's own scope at the setter) with the same
  // tokens, each now carrying out-of-band provenance "token <i> of field <field>" (`Token::prov`). Nothing is added to
  // the store: every token operation sees exactly the tokens pdflatex sees (`\ifx\@title\same`, `\equal`, xstring's
  // `\StrLen`/`\StrLeft`, `\pdfstrcmp`, `\@gobble`, `\@car`, `\csname`, `\write`), since token equality and every
  // character form ignore the provenance. What expands from a stamped token inherits it and what is digested from
  // one carries it (`fieldProv`), so the document's own print of the store is known for the field's ink by
  // construction (core `Document::absorb`'s field runs; [`lift_marked_field_ink`]). The kernel `\title` and `\author`
  // (sect05.rs) call it after the frontmatter took its copy. An empty or blank value is left as it is; anything but
  // a parameterless macro is left alone. OXIDIZED_DESIGN #715.
  DefMacro!("\\lx@field@mark Token DefToken {}", sub[(prefix, store, field)] {
    let field = field.to_string();
    let Some(field_id) = field_id(field.trim()) else {
      return Ok(Tokens::default());
    };
    let Some(defn) = lookup_definition(&store)? else {
      return Ok(Tokens::default());
    };
    if defn.get_parameters().is_some_and(|p| !p.get_parameters().is_empty()) {
      return Ok(Tokens::default());
    }
    let Some(ExpansionBody::Tokens(value)) = defn.get_expansion() else {
      return Ok(Tokens::default());
    };
    if value.unlist_ref().iter().all(|t| t.get_catcode() == Catcode::SPACE) {
      return Ok(Tokens::default());
    }
    // Each call's positions follow the previous call's (`lx_field_base_<id>`), so a copy of an earlier value
    // (`\let\old\@title`, then `\title{Short}`) is never taken for a print of this one (review finding 7).
    let base = usize::try_from(lookup_int(&s!("lx_field_base_{field_id}"))).unwrap_or(0);
    let len = value.unlist_ref().len();
    if base + len > usize::from(FIELD_PROV_POS_MASK) {
      // out of positions: this value is not stamped, and no earlier copy can be whole of it
      assign_value(&s!("lx_field_extent_{field_id}"), String::new(), Some(Scope::Global));
      assign_value(&s!("lx_field_range_{field_id}"), String::new(), Some(Scope::Global));
      // (the frontmatter observer's ledger, under its shadow switch only: OXIDIZED_DESIGN_DIVERGENCES #837)
      note_field_stamp(field_id, None, len);
      return Ok(Tokens::default());
    }
    assign_value(&s!("lx_field_base_{field_id}"), (base + len) as i64, Some(Scope::Global));
    // this call's positions: a print is of the value it sets only within them (an earlier call's print or copy, r12,
    // r13, is not joined with it, [`field_ink_runs`])
    assign_value(
      &s!("lx_field_range_{field_id}"),
      s!("{base}:{}", base + len),
      Some(Scope::Global),
    );
    // The field's extent: the position of every token of the value that bears ink or makes it (a character, a
    // control sequence, an active character; a macro's expansion and a primitive's arguments are covered from its
    // position, core `gullet::field_cover_of`), which a whole print covers, all of them: a fragment the document
    // takes of the value (`\StrLeft{\@title}{5}` of `About \nm`, `\@car`, `\@gobble`) leaves some out (review
    // finding 2). Not a space, a brace, `$`, `^`, `_`, `&`, `#`: what they make is covered by the tokens around them.
    // (as `start:end` intervals, end exclusive, joined by `,`: the form of a run's cover, [`field_ink_runs`])
    // The value's characters (a letter, an other character) are kept apart too: a print digests each once, so a run
    // that holds one twice read the value again ([`field_ink_runs`]).
    let mut extent: Vec<(usize, usize)> = Vec::new();
    let mut chars: Vec<(usize, usize)> = Vec::new();
    let push = |intervals: &mut Vec<(usize, usize)>, at: usize| match intervals.last_mut() {
      Some((_, end)) if *end == at => *end += 1,
      _ => intervals.push((at, at + 1)),
    };
    for (i, t) in value.unlist_ref().iter().enumerate() {
      let code = t.get_catcode();
      if matches!(
        code,
        Catcode::LETTER | Catcode::OTHER | Catcode::CS | Catcode::ACTIVE
      ) {
        push(&mut extent, base + i);
      }
      if matches!(code, Catcode::LETTER | Catcode::OTHER) {
        push(&mut chars, base + i);
      }
    }
    if extent.is_empty() {
      return Ok(Tokens::default());
    }
    let joined = |intervals: &[(usize, usize)]| {
      intervals
        .iter()
        .map(|(start, end)| s!("{start}:{end}"))
        .collect::<Vec<_>>()
        .join(",")
    };
    assign_value(&s!("lx_field_extent_{field_id}"), joined(&extent), Some(Scope::Global));
    assign_value(&s!("lx_field_chars_{field_id}"), joined(&chars), Some(Scope::Global));
    document::note_field_prov_stamped();
    note_field_stamp(field_id, Some(base), len);
    let mut out = vec![prefix, store, T_BEGIN!()];
    for (i, t) in value.unlist_ref().iter().enumerate() {
      let stamped = Token { prov: field_prov(field_id, base + i), ..*t };
      // A `#` in the value is text here, not a parameter of the definition (as `\lx@deposit@field`).
      if t.get_catcode() == Catcode::PARAM {
        out.push(stamped);
      }
      out.push(stamped);
    }
    out.push(T_END!());
    Ok(Tokens::new(out))
  });
  // The frontmatter observer (FM S2, OXIDIZED_DESIGN_DIVERGENCES #837): `\lx@field@stamp{<field>}{<tokens>}` — the
  // tokens of an argument, each stamped with its position in field <field> as `\lx@field@mark` stamps a store's
  // value, the positions following the field's earlier stamps (`lx_field_base_<id>`): M3's "marked field argument",
  // for a binding's `\lx@add@authors*` call, whose argument no store holds. Nothing else changes (the extent and the
  // range of the field's current call are a store's, `\lx@field@mark`'s). An unknown field, a blank argument or one
  // past the field's positions comes back unstamped. No call site yet: the observer's shadow ledger projects these
  // positions for the bindings' calls (`record_author_call`), and stage S2b stamps them.
  DefMacro!("\\lx@field@stamp{}{}", sub[(field, tokens)] {
    let Some(field_id) = field_id(field.to_string().trim()) else {
      return Ok(tokens);
    };
    if tokens.unlist_ref().iter().all(|t| t.get_catcode() == Catcode::SPACE) {
      return Ok(tokens);
    }
    let len = tokens.unlist_ref().len();
    let base = usize::try_from(lookup_int(&s!("lx_field_base_{field_id}"))).unwrap_or(0);
    if base + len > usize::from(FIELD_PROV_POS_MASK) {
      return Ok(tokens);
    }
    assign_value(&s!("lx_field_base_{field_id}"), (base + len) as i64, Some(Scope::Global));
    document::note_field_prov_stamped();
    Ok(field_stamped(field_id, base, tokens.unlist_ref()))
  });
  DefMacro!(
    "\\lx@add@abstract[]{}",
    "\\lx@frontmatter@mark{ltx:abstract}\\lx@clear@frontmatter{ltx:abstract}\\lx@add@frontmatter{ltx:abstract}[#1]{#2}"
  );
  DefMacro!(
    "\\lx@add@keywords[]{}",
    "\\lx@clear@frontmatter{ltx:keywords}\\lx@add@frontmatter{ltx:keywords}[#1]{#2}"
  );
  DefMacro!(
    "\\lx@add@classification[]{}",
    "\\lx@add@frontmatter{ltx:classification}[#1]{#2}"
  );
  // To handle the above as environments
  // (the body's first token, `\lx@abstract@class@setters`, defines there the keywords setters a raw class's refused
  // abstract defines for its body: frontmatter_stores.rs, 68n — wiley-article 1706.00599, astlb 1202.2889, mtt
  // 1310.0585)
  DefMacro!(
    "\\lx@begin@abstract[]",
    "\\lx@frontmatter@mark{ltx:abstract}\\lx@clear@frontmatter{ltx:abstract}\\lx@add@frontmatter@until{ltx:abstract}[#1]{\\lx@end@abstract}\\lx@abstract@class@setters"
  );

  // Like \let \relax, but \relax not def yet!
  // The until-body terminals: no-ops when the body's loop sees them; inside a
  // nested group they end the body from there (stomach.rs).
  // A keywords body a command form opened inside the abstract ends with it: ieeeconf's `\keywords` is a label with no
  // extent (`\begin{abstract} … \keywords \MyKeywords \end{abstract}`, 1401.5657), and read as the open body it kept the
  // abstract open past its `\end{abstract}`, so the whole document went into `<ltx:keywords>`.
  DefPrimitive!("\\lx@end@abstract", {
    if lookup_token("lx@until@terminal") == Some(T_CS!("\\lx@end@keywords")) {
      unread(Tokens!(
        T_CS!("\\lx@end@keywords"),
        T_CS!("\\lx@end@abstract")
      ));
      return Ok(Vec::new());
    }
    until_terminal_inside_group(&T_CS!("\\lx@end@abstract"))?;
  });

  DefMacro!(
    "\\lx@begin@keywords[]",
    "\\lx@clear@frontmatter{ltx:keywords}\\lx@add@frontmatter@until{ltx:keywords}[#1]{\\lx@end@keywords}"
  );

  DefPrimitive!("\\lx@end@keywords", {
    until_terminal_inside_group(&T_CS!("\\lx@end@keywords"))?;
  });

  // `\fnmsep` in frontmatter content (`digest_frontmatter_item`): nothing before a mark the frontmatter turns into a
  // link or a note, the class's comma (`#1`, its saved meaning) otherwise. webofc's `\fnsep` (webofc.cls:217) is the
  // same separator under another name (2301.07438, 1411.7792).
  DefMacro!(
    "\\lx@frontmatter@marksep{}",
    r"\@ifnextchar\thanks{}{\@ifnextchar\inst{}{\@ifnextchar\footnote{}{\@ifnextchar\thanksref{}{#1}}}}"
  );
  DefMacro!(
    "\\lx@frontmatter@fnmsep",
    r"\lx@frontmatter@marksep\lx@saved@fnmsep"
  );
  DefMacro!(
    "\\lx@frontmatter@fnsep",
    r"\lx@frontmatter@marksep\lx@saved@fnsep"
  );

  // A note the title block of a class whose `\thanks` is `\footnotemark` and its text makes (article, report, book:
  // `lx_title_counts_footnotes`, latex_constructs `declare_fnsymbol_title_block`) steps the footnote counter, as the
  // `\maketitle` digests it (`lx_frontmatter_explicit`): `\thanks` and `\footnote` in the title, an author or the date
  // (`lx_frontmatter_item`), so a `\footnotemark[\arabic{footnote}]` after one prints its mark (2201.11183, 2111.13260;
  // KNOWN_PERL_ERRORS #1060, OXIDIZED_DESIGN_DIVERGENCES #851). A flush without `\maketitle` typesets no title, and
  // steps nothing.
  DefPrimitive!("\\lx@title@footnote@step", sub[_args] {
    if lookup_bool("lx_title_counts_footnotes")
      && lookup_bool("lx_frontmatter_explicit")
      && matches!(
        lookup_string("lx_frontmatter_item").as_str(),
        "ltx:creator" | "ltx:title" | "ltx:date"
      )
    {
      step_counter("footnote", false)?;
    }
    Ok(())
  });
  // Add random notes about the document itself
  DefMacro!(
    "\\lx@add@pubnote[]{}",
    "\\lx@title@footnote@step\\lx@add@frontmatter{ltx:pubnote}[#1]{#2}"
  );
  DefMacro!(
    "\\lx@add@pubnote@thanks[]{}",
    "\\lx@title@footnote@step\\lx@add@frontmatter{ltx:pubnote}[role=thanks,#1]{#2}"
  );

  // Other kinds of notes?

  // The following add various forms of contact information to a creator
  DefMacro!(
    "\\lx@add@contact []{}",
    "\\lx@annotate@frontmatter{ltx:creator}{ltx:contact}[#1]{#2}"
  );

  DefMacro!(
    "\\lx@add@affiliation[]{}",
    "\\lx@annotate@frontmatter{ltx:creator}{ltx:contact}[role=affiliation,#1]{\\lx@affiliation@withinst{#2}}"
  );
  // The affiliation-side twin of `\lx@author@withinst`: inside affiliation
  // content `\inst{n}` SETS the affiliation's label (beamerbasetitle.sty:148
  // `\institute{\inst{1}Univ A \and \inst{2}Univ B}` marks each institute
  // with the number the authors' `\inst{1}` request) — the meaning
  // `\lx@affiliation@withsup` gives a `\textsuperscript` mark, never a
  // typeset `<sup>`; `\def`, as in `\lx@author@withinst` (a class's
  // document-level `\inst` is the affiliation LIST, not a label). The author markup is in force here too: an
  // affiliation line of the author block is author content (`$^{3}$Chan Zuckerberg Biohub \\ \email Correspondence:
  // …` under auth_detailed.sty's `\@maketitle` definitions, 2406.07496).
  DefMacro!(
    "\\lx@affiliation@withinst{}",
    "\\let\\lx@saved@inst\\inst\\def\\inst##1{\\lx@sup@setlabel@affiliation{##1}}\\lx@author@markup@begin\\expandafter\\lx@author@markup@run\\expandafter{\\lx@author@markup@unprovide}{#1}\\let\\inst\\lx@saved@inst"
  );
  DefMacro!(
    "\\lx@add@altaffiliation[]{}",
    "\\lx@annotate@frontmatter{ltx:creator}{ltx:contact}[role=altaffiliation,#1]{#2}"
  );
  DefMacro!(
    "\\lx@add@address[]{}",
    "\\lx@annotate@frontmatter{ltx:creator}{ltx:contact}[role=address,#1]{#2}"
  );
  DefMacro!(
    "\\lx@add@altaddress[]{}",
    "\\lx@annotate@frontmatter{ltx:creator}{ltx:contact}[role=altaddress,#1]{#2}"
  );
  DefMacro!(
    "\\lx@add@currentaddress[]{}",
    "\\lx@annotate@frontmatter{ltx:creator}{ltx:contact}[role=currentaddress,#1]{#2}"
  );
  DefMacro!(
    "\\lx@add@email [] Semiverbatim",
    "\\lx@annotate@frontmatter{ltx:creator}{ltx:contact}[role=email,#1]{#2}"
  );
  // Rust-only: a contact a class's command gives apart from the names (OSA's `\email{x@y}` and `\homepage{…}` after
  // the author groups, opex3.sty:52/:57), placed among the queued authors by [`placed_contact_owner`] (`label=author:N`,
  // or the mark's own label, or `label=addresses:block`, the shared creator of #159). `#1` is the contact's keyvals
  // (its `role`, its `name`), `#2` the operand of the mark written on it (`$^*$x@y`: `*`; empty for none), `#3` its
  // content, already read.
  DefMacro!("\\lx@add@contact@placed[]{}{}", sub[(attr, mark, content)] {
    let mark: Tokens = mark;
    let content: Tokens = content;
    // (`owner=first` in the keyvals: a contact that names no one goes to the group's first author, not to the block —
    // AASTeX's `\authoremail` given before the authors, aas_support_sty.rs)
    let attr_text = attr.as_ref().map(ToString::to_string).unwrap_or_default();
    let owner_first = attr_text.split(',').any(|kv| kv.trim() == "owner=first");
    // (`owner=sequence`: one of a class's list of addresses printed apart from the names, in the order the document
    // gives them — jheppub's `\emailAdd`, iopart's `\ead`: placed as below unless the list proves to be the authors'
    // in order, which the frontmatter's flush decides, `sequenced_contact_labels`)
    let owner_sequence = attr_text.split(',').any(|kv| kv.trim() == "owner=sequence");
    // (`mark=address`: a contact the class prints as an address block of its own right after the address before it —
    // iopart's `\ead`, iopart.cls:363 `\def\ead#1{\vspace*{5pt}\address{E-mail: \mailto{#1}}}` —, whose mark is that
    // address's (`\lx@address@mark@set`): weaker evidence than the spelling, so it places only an address that spells
    // no one)
    let flushing = lookup_bool("lx_frontmatter_flushing");
    let under_address = attr_text.split(',').any(|kv| kv.trim() == "mark=address");
    let attr_rest: Vec<&str> = attr_text
      .split(',')
      .map(str::trim)
      .filter(|kv| !kv.is_empty() && !["owner=first", "owner=sequence", "mark=address"].contains(kv))
      .collect();
    let (mark, weak_mark) = match lookup_value("lx_address_mark") {
      Some(Stored::Tokens(address_mark)) if under_address && mark.is_empty() && !flushing => {
        (address_mark, true)
      },
      _ => (mark, false),
    };
    // (an address list's contact digested within another entry, `\author[a]{Ann Able \emailAdd{x}}`, `\address{…
    // \ead{x}}`, is that entry's, as any contact digested there: no list's and placed by nothing of its own, rev78f
    // probes j13, i04)
    if owner_sequence && flushing {
      return Ok(Invocation!(T_CS!("\\lx@annotate@frontmatter"), vec![
        Some(mouth::tokenize_internal("ltx:creator")),
        Some(mouth::tokenize_internal("ltx:contact")),
        Some(mouth::tokenize_internal(TeXString::assembled(attr_rest.join(",")))),
        Some(content)
      ]));
    }
    // (`label=` and `_answers=` are fixed text; a value is a mark's label and is text: tokenized, `$^{1\#}$`'s `#` was
    // a parameter token and a `%` a comment, so each of its characters is taken as it is)
    let (value, answers) = match placed_contact_owner(&mark, &content, weak_mark) {
      ContactOwner::Mark(label) => (label, None),
      // (placed by a footnote-symbol mark its author requests: the contact answers that mark on that author,
      // `show_unanswered_marks`)
      ContactOwner::Author(author) => (s!("author:{author}"), answered_symbol_key(mark.unlist_ref())),
      ContactOwner::Block if owner_first => (s!("author:{}", frontmatter_group_first()), None),
      ContactOwner::Block => (s!("addresses:block"), None),
    };
    let mut keyvals: Vec<Token> = mouth::tokenize_internal("label=").unlist();
    keyvals.extend(text_tokens(&value));
    if let Some(key) = answers {
      keyvals.extend(mouth::tokenize_internal(",_answers=").unlist());
      keyvals.extend(text_tokens(&key));
    }
    if owner_sequence {
      keyvals.extend(
        mouth::tokenize_internal(TeXString::assembled(format!(",_sequence={}", lookup_int("lx_sequence_call"))))
          .unlist(),
      );
      // (the authors it may be of, for the corresponding-author step of `sequenced_contact_labels`, as queue
      // positions: its author group's, where the class marks groups (iopart's `\author`,
      // `\lx@frontmatter@group@start`), and those requesting the mark of the address it is printed under (`_marks`,
      // read where the document writes an e-mail between its addresses); rev78f2 probes i1, i2)
      if lookup_value("lx_frontmatter_group_first").is_some() {
        let group: Vec<String> = (frontmatter_group_first()..=queued_author_marks().len())
          .map(|at| at.to_string())
          .collect();
        keyvals.extend(mouth::tokenize_internal(",_group=").unlist());
        keyvals.extend(text_tokens(&group.join(";")));
      }
      if weak_mark {
        let requesting: Vec<String> = requesting_authors(&queued_author_marks(), mark.unlist_ref())
          .iter()
          .map(ToString::to_string)
          .collect();
        if !requesting.is_empty() {
          keyvals.extend(mouth::tokenize_internal(",_marks=").unlist());
          keyvals.extend(text_tokens(&requesting.join(";")));
        }
      }
      // (placed by its address's mark: where it goes without it, for a document whose addresses are all before its
      // e-mails, `sequenced_contact_labels`)
      if weak_mark {
        let unmarked = match placed_contact_owner(&Tokens::new(Vec::new()), &content, false) {
          ContactOwner::Author(author) => s!("author:{author}"),
          ContactOwner::Mark(label) => label,
          ContactOwner::Block => s!("addresses:block"),
        };
        if unmarked != value {
          keyvals.extend(mouth::tokenize_internal(",_unmarked=").unlist());
          keyvals.extend(text_tokens(&unmarked));
        }
      }
    }
    if owner_first || owner_sequence || under_address {
      if !attr_rest.is_empty() {
        keyvals.push(T_OTHER!(","));
        keyvals.extend(mouth::tokenize_internal(TeXString::assembled(attr_rest.join(","))).unlist());
      }
    } else if let Some(attr) = attr.as_ref().filter(|a| !a.is_empty()) {
      keyvals.push(T_OTHER!(","));
      keyvals.extend(attr.unlist_ref().iter().copied());
    }
    Ok(Invocation!(T_CS!("\\lx@annotate@frontmatter"), vec![
      Some(mouth::tokenize_internal("ltx:creator")),
      Some(mouth::tokenize_internal("ltx:contact")),
      Some(Tokens::new(keyvals)),
      Some(content)
    ]))
  });
  // Rust-only (OXIDIZED_DESIGN_DIVERGENCES #808): the mark of a class's address block, for the contact the class prints
  // right after it as a block of its own (`\lx@add@contact@placed[mark=address]`: iopart's `\ead` after its `\address`,
  // iopart.cls:240-248, :363): the operand of the block's one leading mark (`$^1$ Waseda University`: 1), else none — an
  // unmarked block, or one of several marked lines (`\address{$^1$ A \\ $^2$ B}`), names no one. `\lx@address@mark@set{}`
  // clears it (a class's `\author`, after which an address-list contact is under no address).
  DefPrimitive!("\\lx@address@mark@set{}", sub[(line)] {
    let line: Tokens = line;
    let marks = author_mark_operands(line.unlist_ref());
    let text = |m: &Vec<Token>| Tokens::new(m.clone()).to_string();
    let one = marker_leads(&line)
      && marks.first().is_some_and(|first| marks.iter().all(|m| text(m) == text(first)));
    let mark = if one { Tokens::new(marks[0].clone()) } else { Tokens::new(Vec::new()) };
    assign_value("lx_address_mark", Stored::Tokens(mark), Some(Scope::Global));
  });
  // Rust-only: one call of a class's address-list command (jheppub's `\emailAdd{a, b}`, iopart's `\ead`), whose
  // addresses `\lx@add@contact@placed[owner=sequence]` places: the n-th call is the n-th author's when the list is the
  // authors' in order (`sequenced_contact_labels`).
  DefPrimitive!("\\lx@sequence@call", {
    let call = lookup_int("lx_sequence_call") + 1;
    assign_value("lx_sequence_call", Stored::Int(call), Some(Scope::Global));
  });
  // Rust-only: a class's address-list command's body (`\emailAdd{#1}` = `\lx@add@contact@list{\lx@jhep@email}{#1}`):
  // one call (`\lx@sequence@call`) of `op{address}` for each address of the list (`\lx@split@addresses`). A frontmatter
  // call by its name, so an author line holding the command (`\author[a]{Ann Able \emailAdd{x}}`) keeps it as the
  // name's annotation, as the `\lx@add@contact` body it replaced was (`adds_to_frontmatter`, `is_note_macro`).
  DefMacro!(
    "\\lx@add@contact@list{}{}",
    "\\lx@sequence@call\\lx@split@addresses{#1}{#2}"
  );
  // Rust-only: the start of an author group of a class whose `\author` adds a group per call (OSA's, each typesetting
  // its names where it stands): a contact placed after it ([`placed_contact_owner`]) is the group's only author's
  // (`\author{D. L. Boiko}\address{…}\email{…}\author{N. J. Gunther}…`, 0906.4563; guard
  // `perfect_kernel_batch64u::osa_group_only_author_owns_the_email`).
  DefPrimitive!("\\lx@frontmatter@group@start", {
    let first = queued_author_marks().len() + 1;
    assign_value(
      "lx_frontmatter_group_first",
      Stored::Number(Number::new(first as i64)),
      Some(Scope::Global),
    );
  });
  // Rust-only (OXIDIZED_DESIGN #479): what the document says about an author but does not print (AASTeX 7's `\email`
  // without `[show]`, aastex701.cls:13341-13353; 2608.05283) — an `ltx:rdf` annotation of the creator, `property` the
  // RDFa term and `content` the value, hidden in HTML. An empty value records nothing (AASTeX 7's `\email{}` placeholder).
  DefMacro!("\\lx@add@creator@rdf [] {} Semiverbatim", sub[(attr, property, value)] {
    // The value as written, without the spaces around it (2608.13718 `\email{koberg@cfa.harvard.edu }`)
    let mut value = value.unlist();
    while value.last().is_some_and(|t| t.get_catcode() == Catcode::SPACE) {
      value.pop();
    }
    let lead = value.iter().take_while(|t| t.get_catcode() == Catcode::SPACE).count();
    value.drain(..lead);
    if value.is_empty() {
      return Ok(Tokens!());
    }
    let mut opts = mouth::tokenize_internal("property=").unlist();
    opts.push(T_BEGIN!());
    opts.extend(property.unlist());
    opts.push(T_END!());
    opts.extend(mouth::tokenize_internal(",content=").unlist());
    opts.push(T_BEGIN!());
    opts.extend(value);
    opts.push(T_END!());
    if let Some(a) = attr.as_ref().filter(|a| !a.is_empty()) {
      opts.push(T_OTHER!(","));
      opts.extend(a.unlist_ref().iter().copied());
    }
    Ok(Invocation!(T_CS!("\\lx@annotate@frontmatter"),
      vec![Some(mouth::tokenize_internal("ltx:creator")),
           Some(mouth::tokenize_internal("ltx:rdf")),
           Some(Tokens::new(opts)), Some(Tokens!())]))
  });
  DefMacro!(
    "\\lx@add@email@metadata [] Semiverbatim",
    "\\lx@add@creator@rdf[#1]{schema:email}{#2}"
  );
  DefMacro!(
    "\\lx@add@url [] Semiverbatim",
    "\\lx@annotate@frontmatter{ltx:creator}{ltx:contact}[role=url,#1]{#2}"
  );
  // Rust-only: an email or web address set as a class sets it with `\url`, `\texttt` or `\ttfamily` — url.sty's
  // `\UrlFont` (url_sty.rs `\UrlFont`): the typewriter family, in the identity ASCII encoding. A class that reads the
  // address with `_` and `~` made other (revtex4.cls:4476-4493 `\sanitize@url` … `\href{mailto:#2}{#2}`, `\href` the
  // `\url` = `\texttt` of :4490-4493) prints them as cmtt does, '_' and '~'; the same characters in the roman OT1 font
  // are its dot and tilde accents (Perl TeX_Fonts.pool.ltxml:214), "ann˙able@a.edu" (1001.1076). A class binding
  // passes its address through this only where the class sets it in typewriter type.
  DefMacro!(
    "\\lx@address@urlfont{}",
    "{\\lx@fontencoding{ASCII}\\ttfamily #1}"
  );
  // Rust-only: an email or web address a class sets in the current (roman) font — revtex4-1/4-2's `\href`, which
  // without hyperref is `\@href` (revtex4-1.cls:1959-1961), AASTeX's, svjour3's, jmlr's — in that font, but in the
  // identity ASCII encoding: its `_` and `~` print as typed. In the OT1 font pdflatex prints them as the dot and tilde
  // accents ("ann˙able@a.edu", 2301.03570, 2301.03067); the address is the author's, not the accident of the font (user
  // ruling 2026-10-11).
  DefMacro!("\\lx@address@textfont{}", "{\\lx@fontencoding{ASCII}#1}");
  // The ORCID iD logo — the green "iD" badge — as ONE self-contained SVG asset,
  // defined in the kernel so every orcid rendering path reuses it (orcidlink.sty's
  // \orcidlogo aliases this; \lx@add@orcid embeds it). Adapted from
  // https://orcid.org/assets/vectors/orcid.logo.icon.svg; the viewBox exactly
  // frames the 72×72 disk and the `.ltx_orcidlogo` CSS rule sizes it to the text
  // (1em), so it renders as an inline glyph — not the oversized 1.7em badge that
  // overflowed the line (html_feedback #6895, #6016, #5789, #2176, #5615).
  DefConstructor!("\\lx@orcidlogo", sub[document, _args, _props] {
    document.open_element("svg:svg", Some(string_map!(
      "class" => "ltx_orcidlogo", "width" => "1em", "height" => "1em",
      "viewBox" => "0 0 72 72", "version" => "1.1")), None)?;
    document.open_element("svg:path", Some(string_map!(
      "fill" => "#A6CE39",
      "d" => "M72,36 C72,55.884375 55.884375,72 36,72 C16.115625,72 0,55.884375 0,36 C0,16.115625 16.115625,0 36,0 C55.884375,0 72,16.115625 72,36 Z")), None)?;
    document.close_element("svg:path")?;
    document.open_element("svg:g", Some(string_map!(
      "fill" => "#FFFFFF", "transform" => "translate(18.868966, 12.910345)")), None)?;
    document.open_element("svg:polygon", Some(string_map!(
      "points" => "5.03734929 39.1250878 0.695429861 39.1250878 0.695429861 9.14431787 5.03734929 9.14431787 5.03734929 22.6930505 5.03734929 39.1250878")), None)?;
    document.close_element("svg:polygon")?;
    document.open_element("svg:path", Some(string_map!(
      "d" => "M11.409257,9.14431787 L23.1380784,9.14431787 C34.303014,9.14431787 39.2088191,17.0664074 39.2088191,24.1486995 C39.2088191,31.846843 33.1470485,39.1530811 23.1944669,39.1530811 L11.409257,39.1530811 L11.409257,9.14431787 Z M15.7511765,35.2620194 L22.6587756,35.2620194 C32.49858,35.2620194 34.7541226,27.8438084 34.7541226,24.1486995 C34.7541226,18.1301509 30.8915059,13.0353795 22.4332213,13.0353795 L15.7511765,13.0353795 L15.7511765,35.2620194 Z")), None)?;
    document.close_element("svg:path")?;
    document.open_element("svg:path", Some(string_map!(
      "d" => "M5.71401206,2.90182329 C5.71401206,4.441452 4.44526937,5.72914146 2.86638958,5.72914146 C1.28750978,5.72914146 0.0187670918,4.441452 0.0187670918,2.90182329 C0.0187670918,1.33420133 1.28750978,0.0745051096 2.86638958,0.0745051096 C4.44526937,0.0745051096 5.71401206,1.36219458 5.71401206,2.90182329 Z")), None)?;
    document.close_element("svg:path")?;
    document.close_element("svg:g")?;
    document.close_element("svg:svg")?;
  });
  // A link to https://orcid.org/<#1> wrapping content #2 (class ltx_orcid). Shared
  // by \lx@add@orcid and orcidlink.sty's \orcidlink family. Perl orcidlink #2681.
  DefConstructor!(
    "\\lx@orcidlink{}{}",
    "<ltx:ref title='ORCID #1' class='ltx_orcid' href='https://orcid.org/#1'>#2</ltx:ref>"
  );
  // The single canonical author-ORCID frontmatter macro: a `ltx:contact[role=orcid]`
  // (consistent with \lx@add@email / \lx@add@affiliation &c.) whose value is the iD
  // logo linked to the author's orcid.org page. The id is preserved in the href for
  // the metadata. Every class binding routes its \orcid here (html_feedback #6571).
  // An iD given as its orcid.org address (`https://orcid.org/0000-…`, with or without the scheme or `www.`) is the iD:
  // its link was `https://orcid.org/https://orcid.org/0000-…` (2104.12510, 2304.13458, 2407.18715 `\orcidauthor`;
  // DIVERGENCES #616). Perl's contact holds the text as given (Base_Utility.pool.ltxml:659-660).
  DefMacro!(
    "\\lx@add@orcid [] Semiverbatim",
    "\\lx@annotate@frontmatter{ltx:creator}{ltx:contact}[role=orcid,#1]{\\expandafter\\lx@orcidlink\\expandafter{\\lx@orcid@id{#2}}{\\lx@orcidlogo}}"
  );
  DefMacro!("\\lx@orcid@id Semiverbatim", sub[(id)] { Ok(orcid_id(id)) });
  // Beyond-Perl (OXIDIZED_DESIGN #52): the arXiv "\thanks abuse" idiom smuggles
  // affiliations into an author \thanks{...}, linking them to authors by a
  // leading superscript mark ($^{n}$ / \textsuperscript{n}). \thanks is
  // semantically an acknowledgement, so we re-route ONLY when the content BEGINS
  // with such a mark (the abuse signature — a genuine acknowledgement never
  // starts with a bare superscript); every other \thanks stays a role=thanks
  // note. When detected we feed each $^{n}$-delimited segment through
  // \lx@affiliation@withsup, which sets the affiliation:N label that the authors'
  // own marks already request (relocate_annotations then links
  // author<->affiliation). Witness arXiv:2606.00313.
  // The argument is an ordinary one, as LaTeX's `\thanks` reads it and as a title's `\thanks`
  // (`\lx@add@pubnote@thanks`) does: Perl's Semiverbatim (Base_Utility.pool.ltxml:661) froze `$`, `^`, `_` to
  // catcode other, so an author's `\thanks{... 10 m$^{2}$ lab.}` printed its math as text (KNOWN_PERL_ERRORS #515;
  // repro sectioning-frontmatter/author_thanks_and_footnote_keep_their_math).
  DefMacro!("\\lx@add@thanks [] {}", sub[(attr, content)] {
    if starts_with_affiliation_mark(&content) {
      let mut calls: Vec<Token> = vec![T_CS!("\\lx@title@footnote@step")];
      for seg in split_wrapped_affiliation_marks(content) {
        if spacing_end(seg.unlist_ref()) == 0 {
          continue;
        }
        let withsup = Invocation!(T_CS!("\\lx@affiliation@withsup"), vec![Some(seg)]);
        calls.extend(
          Invocation!(T_CS!("\\lx@add@affiliation"), vec![None, Some(withsup)]).unlist());
      }
      Ok(Tokens::new(calls))
    } else {
      // SURPASS over Perl (OXIDIZED_DESIGN #156): render an author-attached `\thanks`
      // as a MARKED note (a superscript mark on the author name + margin/footnote
      // content) instead of a `role=thanks` CONTACT that reads inline like an
      // affiliation next to the name. Route to `<ltx:note role="thanks">` so the
      // existing `ltx:note` footnote template (mark + `ltx_note_outer`/`ltx_note_content`)
      // renders it, and attach semantic CSS hooks so a theme can style each kind of
      // thanks content: `ltx_note_frontmatter` (the ar5iv frontmatter-note hook) plus a
      // best-effort content-kind class `ltx_thanks_<kind>` from `classify_thanks`. Perl
      // keeps the inline contact (SHARED readability gap). Witnesses arXiv 2512.24601
      // (correspondence), 1510.02728 (funding).
      let kind = classify_thanks(content.clone().untex_string().as_ref());
      let mut opts = mouth::tokenize_internal(TeXString::assembled(s!(
        "role=thanks,class=ltx_note_frontmatter ltx_thanks_{kind}"
      )))
      .unlist();
      if let Some(a) = &attr {
        opts.push(T_OTHER!(","));
        opts.extend(a.unlist_ref().iter().copied());
      }
      let mut calls = vec![T_CS!("\\lx@title@footnote@step")];
      calls.extend(Invocation!(T_CS!("\\lx@annotate@frontmatter"),
        vec![Some(mouth::tokenize_internal("ltx:creator")),
             Some(mouth::tokenize_internal("ltx:note")),
             Some(Tokens::new(opts)), Some(note_body(content))]).unlist());
      Ok(Tokens::new(calls))
    }
  });
  // An ordinary argument, as `\lx@add@thanks` (an author's `\footnote`; KNOWN_PERL_ERRORS #515).
  DefMacro!("\\lx@add@note [] {}", sub[(attr, content)] {
    let mut opts = mouth::tokenize_internal("role=note").unlist();
    if let Some(a) = &attr {
      opts.push(T_OTHER!(","));
      opts.extend(a.unlist_ref().iter().copied());
    }
    let mut calls = vec![T_CS!("\\lx@title@footnote@step")];
    calls.extend(Invocation!(T_CS!("\\lx@annotate@frontmatter"),
      vec![Some(mouth::tokenize_internal("ltx:creator")),
           Some(mouth::tokenize_internal("ltx:contact")),
           Some(Tokens::new(opts)), Some(note_body(content))]).unlist());
    Ok(Tokens::new(calls))
  });
  Ok(())
}
