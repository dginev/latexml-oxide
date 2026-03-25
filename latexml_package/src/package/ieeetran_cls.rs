use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: IEEEtran.cls.ltxml — IEEE Transactions document class

  // RawTeX block: conditionals and option declarations
  RawTeX!(r"\newif\ifCLASSOPTIONonecolumn       \CLASSOPTIONonecolumnfalse");
  RawTeX!(r"\newif\ifCLASSOPTIONtwocolumn       \CLASSOPTIONtwocolumntrue");
  RawTeX!(r"\newif\ifCLASSOPTIONoneside         \CLASSOPTIONonesidetrue");
  RawTeX!(r"\newif\ifCLASSOPTIONtwoside         \CLASSOPTIONtwosidefalse");
  RawTeX!(r"\newif\ifCLASSOPTIONfinal           \CLASSOPTIONfinaltrue");
  RawTeX!(r"\newif\ifCLASSOPTIONdraft           \CLASSOPTIONdraftfalse");
  RawTeX!(r"\newif\ifCLASSOPTIONdraftcls        \CLASSOPTIONdraftclsfalse");
  RawTeX!(r"\newif\ifCLASSOPTIONdraftclsnofoot  \CLASSOPTIONdraftclsnofootfalse");
  RawTeX!(r"\newif\ifCLASSOPTIONpeerreview      \CLASSOPTIONpeerreviewfalse");
  RawTeX!(r"\newif\ifCLASSOPTIONpeerreviewca    \CLASSOPTIONpeerreviewcafalse");
  RawTeX!(r"\newif\ifCLASSOPTIONjournal         \CLASSOPTIONjournaltrue");
  RawTeX!(r"\newif\ifCLASSOPTIONconference      \CLASSOPTIONconferencefalse");
  RawTeX!(r"\newif\ifCLASSOPTIONtechnote        \CLASSOPTIONtechnotefalse");
  RawTeX!(r"\newif\ifCLASSOPTIONnofonttune      \CLASSOPTIONnofonttunefalse");
  RawTeX!(r"\newif\ifCLASSOPTIONcaptionsoff     \CLASSOPTIONcaptionsofffalse");
  RawTeX!(r"\newif\ifCLASSOPTIONcomsoc          \CLASSOPTIONcomsocfalse");
  RawTeX!(r"\newif\ifCLASSOPTIONcompsoc         \CLASSOPTIONcompsocfalse");
  RawTeX!(r"\newif\ifCLASSOPTIONtransmag        \CLASSOPTIONtransmagfalse");
  RawTeX!(r"\newif\ifCLASSOPTIONromanappendices \CLASSOPTIONromanappendicesfalse");
  RawTeX!(r"\newif\ifCLASSINFOpdf               \CLASSINFOpdffalse");
  RawTeX!(r"\CLASSINFOpdftrue");

  // DeclareOption for paper sizes, layout, modes
  RawTeX!(r"\DeclareOption{9pt}{\def\CLASSOPTIONpt{9}\def\@ptsize{0}}");
  RawTeX!(r"\DeclareOption{10pt}{\def\CLASSOPTIONpt{10}\def\@ptsize{0}}");
  RawTeX!(r"\DeclareOption{11pt}{\def\CLASSOPTIONpt{11}\def\@ptsize{1}}");
  RawTeX!(r"\DeclareOption{12pt}{\def\CLASSOPTIONpt{12}\def\@ptsize{2}}");
  RawTeX!(r"\DeclareOption{letterpaper}{\setlength{\paperwidth}{8.5in}\setlength{\paperheight}{11in}\def\CLASSOPTIONpaper{letter}\def\CLASSINFOpaperwidth{8.5in}\def\CLASSINFOpaperheight{11in}}");
  RawTeX!(r"\DeclareOption{a4paper}{\setlength{\paperwidth}{210mm}\setlength{\paperheight}{297mm}\def\CLASSOPTIONpaper{a4}\def\CLASSINFOpaperwidth{210mm}\def\CLASSINFOpaperheight{297mm}}");
  RawTeX!(r"\DeclareOption{cspaper}{\setlength{\paperwidth}{7.875in}\setlength{\paperheight}{10.75in}\def\CLASSOPTIONpaper{ieeecs}\def\CLASSINFOpaperwidth{7.875in}\def\CLASSINFOpaperheight{10.75in}}");
  RawTeX!(r"\DeclareOption{oneside}{\@twosidefalse\@mparswitchfalse\CLASSOPTIONonesidetrue\CLASSOPTIONtwosidefalse}");
  RawTeX!(r"\DeclareOption{twoside}{\@twosidetrue\@mparswitchtrue\CLASSOPTIONtwosidetrue\CLASSOPTIONonesidefalse}");
  RawTeX!(r"\DeclareOption{onecolumn}{\CLASSOPTIONonecolumntrue\CLASSOPTIONtwocolumnfalse}");
  RawTeX!(r"\DeclareOption{twocolumn}{\CLASSOPTIONtwocolumntrue\CLASSOPTIONonecolumnfalse}");
  RawTeX!(r"\DeclareOption{draft}{\CLASSOPTIONdrafttrue\CLASSOPTIONdraftclstrue\CLASSOPTIONdraftclsnofootfalse}");
  RawTeX!(r"\DeclareOption{draftcls}{\CLASSOPTIONdraftfalse\CLASSOPTIONdraftclstrue\CLASSOPTIONdraftclsnofootfalse}");
  RawTeX!(r"\DeclareOption{draftclsnofoot}{\CLASSOPTIONdraftfalse\CLASSOPTIONdraftclstrue\CLASSOPTIONdraftclsnofoottrue}");
  RawTeX!(r"\DeclareOption{final}{\CLASSOPTIONdraftfalse\CLASSOPTIONdraftclsfalse\CLASSOPTIONdraftclsnofootfalse}");
  RawTeX!(r"\DeclareOption{journal}{\CLASSOPTIONpeerreviewfalse\CLASSOPTIONpeerreviewcafalse\CLASSOPTIONjournaltrue\CLASSOPTIONconferencefalse\CLASSOPTIONtechnotefalse}");
  RawTeX!(r"\DeclareOption{conference}{\CLASSOPTIONpeerreviewfalse\CLASSOPTIONpeerreviewcafalse\CLASSOPTIONjournalfalse\CLASSOPTIONconferencetrue\CLASSOPTIONtechnotefalse}");
  RawTeX!(r"\DeclareOption{technote}{\CLASSOPTIONpeerreviewfalse\CLASSOPTIONpeerreviewcafalse\CLASSOPTIONjournalfalse\CLASSOPTIONconferencefalse\CLASSOPTIONtechnotetrue}");
  RawTeX!(r"\DeclareOption{peerreview}{\CLASSOPTIONpeerreviewtrue\CLASSOPTIONpeerreviewcafalse\CLASSOPTIONjournalfalse\CLASSOPTIONconferencefalse\CLASSOPTIONtechnotefalse}");
  RawTeX!(r"\DeclareOption{peerreviewca}{\CLASSOPTIONpeerreviewtrue\CLASSOPTIONpeerreviewcatrue\CLASSOPTIONjournalfalse\CLASSOPTIONconferencefalse\CLASSOPTIONtechnotefalse}");
  RawTeX!(r"\DeclareOption{nofonttune}{\CLASSOPTIONnofonttunetrue}");
  RawTeX!(r"\DeclareOption{captionsoff}{\CLASSOPTIONcaptionsofftrue}");
  RawTeX!(r"\DeclareOption{comsoc}{\CLASSOPTIONcomsoctrue\CLASSOPTIONcompsocfalse\CLASSOPTIONtransmagfalse}");
  RawTeX!(r"\DeclareOption{compsoc}{\CLASSOPTIONcomsocfalse\CLASSOPTIONcompsoctrue\CLASSOPTIONtransmagfalse}");
  RawTeX!(r"\DeclareOption{transmag}{\CLASSOPTIONtransmagtrue\CLASSOPTIONcomsocfalse\CLASSOPTIONcompsocfalse}");
  RawTeX!(r"\DeclareOption{romanappendices}{\CLASSOPTIONromanappendicestrue}");

  // Catch-all: pass unknown options to article
  DeclareOption!(None, {
    Digest!("\\PassOptionsToClass{\\CurrentOption}{article}")?;
  });

  ProcessOptions!();
  load_class("article", Vec::new(), Tokens!())?;

  // Perl: DefMacro('\IEEEtitleabstractindextext{}', '#1');
  DefMacro!("\\IEEEtitleabstractindextext{}", "#1");
  DefMacro!("\\IEEEdisplaynontitleabstractindextext", None);
  DefMacro!("\\IEEEdisplaynotcompsoctitleabstractindextext", None);
  DefMacro!("\\IEEEcompsoctitleabstractindextext", None);
  Let!("\\IEEEpeerreviewmaketitle", "\\maketitle");
  DefMacro!("\\IEEEoverridecommandlockouts", None);
  DefMacro!("\\overrideIEEEmargins", None);

  DefMacro!("\\IEEEaftertitletext{}", None);
  DefMacro!("\\IEEEspecialpapernotice{}", None);

  DefMacro!("\\IEEEmembership{}", None);
  DefMacro!("\\IEEEauthorblockN{}", "#1");

  // Perl: DefConstructor('\@@@affiliation{}',
  //   "^ <ltx:contact role='affiliation'>#1</ltx:contact>");
  DefConstructor!("\\@@@affiliation{}",
    "^ <ltx:contact role='affiliation'>#1</ltx:contact>");
  // Perl: DefMacro('\IEEEauthorblockA{}',
  //   '\@add@to@frontmatter{ltx:creator}{\@@@affiliation{#1}}');
  DefMacro!("\\IEEEauthorblockA{}",
    "\\@add@to@frontmatter{ltx:creator}{\\@@@affiliation{#1}}");

  // IEEEkeywords environment
  DefMacro!(T_CS!("\\begin{IEEEkeywords}"), None, "\\@IEEEkeywords");
  DefMacro!(T_CS!("\\end{IEEEkeywords}"), None, "\\@endIEEEkeywords");
  Let!("\\@endIEEEkeywords", "\\relax");
  DefMacro!("\\@IEEEkeywords XUntil:\\@endIEEEkeywords",
    "\\@add@frontmatter{ltx:keywords}[name={\\IEEEkeywordsname}]{#1}");
  DefMacro!("\\IEEEraisesectionheading{}", "#1");
  DefMacro!("\\IEEEPARstart{}{}", "#1#2");

  DefMacro!("\\IEEEcompsocitemizethanks{}", "\\thanks{#1}");
  DefMacro!("\\IEEEcompsocthanksitem[]", None);
  DefMacro!("\\IEEEauthorrefmark", None);
  DefMacro!("\\IEEEtriggeratref{}", None);

  DefMacro!("\\IEEEpubid{}", "\\@add@frontmatter{ltx:note}[role=publicationid]{pubid: #1}");
  DefMacro!("\\IEEEpubidadjcol", None);

  // Section numbering: compsoc vs standard
  RawTeX!(r"\ifCLASSOPTIONcompsoc
\def\thesection{\arabic{section}}
\def\thesubsection{\thesection.\arabic{subsection}}
\def\thesubsubsection{\thesubsection.\arabic{subsubsection}}
\def\theparagraph{\thesubsubsection.\arabic{paragraph}}
\else
\def\thesection{\Roman{section}}
\def\thesubsection{\mbox{\thesection-\Alph{subsection}}}
\def\thesubsubsection{\thesubsection\arabic{subsubsection}}
\def\theparagraph{\thesubsubsection\alph{paragraph}}
\fi");

  // Font switches for section titles
  DefPrimitive!("\\ltx@ieeetran@it", None,
    font => {shape => "italic", family => "serif", series => "medium"}, locked => true);
  DefPrimitive!("\\ltx@ieeetran@sc", None,
    font => {shape => "smallcaps", family => "serif", series => "medium"}, locked => true);

  DefMacro!("\\format@title@font@section", "\\ltx@ieeetran@sc");
  DefMacro!("\\format@title@font@subsection", "\\ltx@ieeetran@it");
  DefMacro!("\\figurename", "Fig.");
  DefMacro!("\\tablename", "TABLE");
  DefMacro!("\\thetable", "\\Roman{table}");

  // IEEEQEDclosed — end-of-proof symbol
  DefConstructor!("\\IEEEQEDclosed",
    "?#isMath(<ltx:XMTok role='PUNCT'>\u{220E}</ltx:XMTok>)(\u{220E})",
    enter_horizontal => true);
  Let!("\\IEEEQEDopen", "\\IEEEQEDclosed");
  Let!("\\IEEEQED", "\\IEEEQEDclosed");

  // Perl: DefMacro('\IEEEQEDhere', sub { PopValue('QED@stack') ... })
  DefMacro!("\\IEEEQEDhere", sub[_args] {
    let t = state::pop_value("QED@stack")?.unwrap_or(Stored::None);
    match t {
      Stored::Tokens(toks) if !toks.is_empty() => toks,
      _ => Tokens!(),
    }
  });

  // IEEEproof environment
  DefEnvironment!("{IEEEproof} OptionalUndigested",
    "<ltx:proof class='#class'><ltx:title font='#titlefont' _force_font='true' class='#titleclass'>#title</ltx:title>#body",
    after_construct => sub[doc, _whatsit] {
      doc.maybe_close_element("ltx:proof")?;
    },
    after_digest_begin => sub[_whatsit] {
      state::push_value("QED@stack", Stored::Tokens(Tokens!(T_CS!("\\qed"))))?;
    },
    properties => {
      let title = stomach::digest(
        Tokenize!("\\textbf{\\textit{Proof:}}"))?;
      // Perl: extracts font from 2nd element of unlist for titlefont
      let titlefont = title.unlist().get(1)
        .and_then(|d| d.get_font().ok().flatten().map(|f| f.into_owned()));
      let mut props = stored_map!("title" => title, "titleclass" => "ltx_runin");
      if let Some(font) = titlefont {
        props.insert("titlefont", Stored::Font(Rc::new(font)));
      }
      Ok(props)
    },
    before_digest_end => {
      let qed = state::pop_value("QED@stack")?;
      match qed {
        Some(Stored::Tokens(toks)) if !toks.is_empty() => {
          return Ok(vec![stomach::digest(toks)?]);
        },
        _ => {},
      }
    }
  );

  // Lengths
  RawTeX!(r"\newlength\abovecaptionskip");
  RawTeX!(r"\newlength\belowcaptionskip");
  RawTeX!(r"\setlength\abovecaptionskip{0.5\baselineskip}");
  RawTeX!(r"\setlength\belowcaptionskip{0pt}");

  // IEEEbiography environments
  DefEnvironment!("{IEEEbiography}[]{}",
    "<ltx:float class='biography'><ltx:tabular><ltx:tr><ltx:td>#1</ltx:td><ltx:td><ltx:inline-block><ltx:text class='ltx_font_bold'>#2</ltx:text> #body</ltx:inline-block></ltx:td></ltx:tr></ltx:tabular></ltx:float>");
  DefEnvironment!("{IEEEbiographynophoto}[]{}",
    "<ltx:float class='biography'><ltx:tabular><ltx:tr><ltx:td><ltx:inline-block><ltx:text class='ltx_font_bold'>#2</ltx:text> #body</ltx:inline-block></ltx:td></ltx:tr></ltx:tabular></ltx:float>");

  // IEEEnonumber / IEEEyesnumber / IEEEyessubnumber / IEEEnosubnumber
  // Perl: These manipulate EQUATION_NUMBERING and EQUATIONROW_TAGS state
  // For now, simplified stubs that handle the common cases
  // Perl: IEEEnonumber manipulates EQUATION_NUMBERING and EQUATIONROW_TAGS
  DefPrimitive!("\\IEEEnonumber OptionalMatch:*", sub[(star)] {
    // Suppress numbering for current row (no star) or all rows (star)
    if star.is_some() {
      // Star: retract all future equations in this group
      with_value_mut("EQUATION_NUMBERING", |val_opt| {
        if let Some(Stored::HashStored(ref mut numbering)) = val_opt {
          numbering.insert("retract", true.into());
          numbering.remove("counter");
        }
      });
    } else {
      // No star: retract current equation only
      with_value_mut("EQUATIONROW_TAGS", |val_opt| {
        if let Some(Stored::HashStored(ref mut tags)) = val_opt {
          tags.insert("retract", true.into());
          tags.remove("counter");
        }
      });
    }
  });
  DefPrimitive!("\\IEEEyesnumber OptionalMatch:*", sub[(star)] {
    // Restore numbering for current row (no star) or all rows (star)
    if star.is_some() {
      // Star: enable numbering for all future equations in this group
      with_value_mut("EQUATION_NUMBERING", |val_opt| {
        if let Some(Stored::HashStored(ref mut numbering)) = val_opt {
          numbering.remove("retract");
          numbering.remove("counter");
        }
      });
    } else {
      // No star: force number on current equation
      with_value_mut("EQUATIONROW_TAGS", |val_opt| {
        if let Some(Stored::HashStored(ref mut tags)) = val_opt {
          tags.insert("noretract", true.into());
          tags.remove("counter");
        }
      });
    }
  });
  DefPrimitive!("\\IEEEyessubnumber OptionalMatch:*", sub[(star)] {
    // Switch to sub-equation counter
    if star.is_some() {
      with_value_mut("EQUATION_NUMBERING", |val_opt| {
        if let Some(Stored::HashStored(ref mut numbering)) = val_opt {
          numbering.insert("counter", Stored::from("subequation"));
        }
      });
    } else {
      with_value_mut("EQUATIONROW_TAGS", |val_opt| {
        if let Some(Stored::HashStored(ref mut tags)) = val_opt {
          tags.insert("counter", Stored::from("subequation"));
        }
      });
    }
    // If preset, step the subequation counter
    let has_preset = state::lookup_value("EQUATION_NUMBERING")
      .map(|v| if let Stored::HashStored(h) = v { h.contains_key("preset") } else { false })
      .unwrap_or(false)
      || state::lookup_value("EQUATIONROW_TAGS")
      .map(|v| if let Stored::HashStored(h) = v { h.contains_key("preset") } else { false })
      .unwrap_or(false);
    if has_preset {
      ref_step_counter("subequation", false)?;
    }
  });
  DefPrimitive!("\\IEEEnosubnumber OptionalMatch:*", sub[(star)] {
    // Switch back to equation counter
    if star.is_some() {
      with_value_mut("EQUATION_NUMBERING", |val_opt| {
        if let Some(Stored::HashStored(ref mut numbering)) = val_opt {
          numbering.insert("counter", Stored::from("equation"));
        }
      });
    } else {
      with_value_mut("EQUATIONROW_TAGS", |val_opt| {
        if let Some(Stored::HashStored(ref mut tags)) = val_opt {
          tags.insert("counter", Stored::from("equation"));
        }
      });
    }
  });

  // IEEEeqnarray => eqnarray
  DefMacro!("\\IEEEeqnarray{}", "\\eqnarray");
  Let!(T_CS!("\\endIEEEeqnarray"), T_CS!("\\endeqnarray"));
  // Perl: DefMacroI(T_CS('\IEEEeqnarray*'), '{}', T_CS('\eqnarray*'));
  // Must use T_CS! for the expansion so * is part of the CS name, not a separate token.
  {
    let params = parse_parameters("{}", &T_CS!("\\IEEEeqnarray*"), true)?;
    def_macro(T_CS!("\\IEEEeqnarray*"), params, Tokens!(T_CS!("\\eqnarray*")), None)?;
  }
  Let!(T_CS!("\\endIEEEeqnarray*"), T_CS!("\\endeqnarray*"));

  // Column types for IEEEeqnarray: L, C, R
  DefColumnType!("L", {
    with_current_build_template(|template_opt| {
      template_opt.unwrap().add_column(Cell {
        after: Some(Tokens!(T_CS!("\\hfil"))),
        ..Cell::default()
      })
    });
  });
  DefColumnType!("C", {
    with_current_build_template(|template_opt| {
      template_opt.unwrap().add_column(Cell {
        before: Some(Tokens!(T_CS!("\\hfil"))),
        after: Some(Tokens!(T_CS!("\\hfil"))),
        ..Cell::default()
      })
    });
  });
  DefColumnType!("R", {
    with_current_build_template(|template_opt| {
      template_opt.unwrap().add_column(Cell {
        before: Some(Tokens!(T_CS!("\\hfil"))),
        ..Cell::default()
      })
    });
  });

  // IEEEeqnarraybox
  DefMacro!("\\IEEEeqnarraybox",
    "\\ifmmode\\def\\@tempa{\\let\\endIEEEeqnarraybox\\endIEEEeqnarrayboxm\\IEEEeqnarrayboxm}\\else\\def\\@tempa{\\let\\endIEEEeqnarraybox\\endIEEEeqnarrayboxt\\IEEEeqnarrayboxt}\\fi\\@tempa");
  DefMacro!("\\IEEEeqnarrayboxm OptionalMatch:* {}",
    "\\@array@bindings{#2}\\@@IEEE@array{#2}\\lx@begin@alignment");
  DefMacro!(T_CS!("\\endIEEEeqnarrayboxm"), None,
    "\\lx@end@alignment\\@end@array");
  DefMacro!("\\IEEEeqnarrayboxt OptionalMatch:* {}",
    "\\lx@begin@inline@math\\@array@bindings{#2}\\@@IEEE@array{#2}\\lx@begin@alignment");
  DefMacro!(T_CS!("\\endIEEEeqnarrayboxt"), None,
    "\\lx@end@alignment\\@end@array\\lx@end@inline@math");


  DefConstructor!("\\@@IEEE@array[] Undigested DigestedBody", "#3",
    before_digest => sub { bgroup(); },
    reversion => "\\begin{IEEEeqnarraybox}[#1]{#2}#3\\end{IEEEeqnarraybox}"
  );

  DefMacro!("\\IEEEeqnarraynumspace", None);

  Let!(T_CS!("\\appendices"), T_CS!("\\appendix"));

  // BIBSTYLES
  state::assign_value("BIBSTYLES_IEEEtran_citestyle", Stored::String(arena::pin("numbers")), None);
  state::assign_value("BIBSTYLES_IEEEtran_sort", Stored::String(arena::pin("true")), None);

  // IED list support macros
  DefMacro!("\\IEEEsetlabelwidth{}", "\\settowidth{\\labelwidth}{#1}");
  DefMacro!("\\IEEEusemathlabelsep", None);
  DefMacro!("\\IEEEtriggercmd{}", None);
  DefMacro!("\\IEEElabelindent", None);
  DefMacro!("\\IEEEcalcleftmargin{}", None);
  DefMacro!("\\IEEEiedlabeljustifyc", None);
  DefMacro!("\\IEEEiedlabeljustifyl", None);
  DefMacro!("\\IEEEiedlabeljustifyr", None);

  // IEEEitemize, IEEEenumerate, IEEEdescription
  DefEnvironment!("{IEEEitemize}[]",
    "<ltx:itemize xml:id='#id'>#body</ltx:itemize>",
    properties => { BeginItemize!("itemize", "@item") },
    before_digest_end => { Digest!("\\par") },
    locked => true,
    mode => "internal_vertical"
  );
  DefEnvironment!("{IEEEenumerate}[]",
    "<ltx:enumerate xml:id='#id'>#body</ltx:enumerate>",
    properties => { BeginItemize!("enumerate", "enum") },
    before_digest_end => { Digest!("\\par") },
    locked => true,
    mode => "internal_vertical"
  );
  DefEnvironment!("{IEEEdescription}[]",
    "<ltx:description xml:id='#id'>#body</ltx:description>",
    before_digest => { Let!("\\makelabel", "\\descriptionlabel"); },
    properties => { BeginItemize!("description", "@desc") },
    before_digest_end => { Digest!("\\par") },
    locked => true,
    mode => "internal_vertical"
  );

  // Override LaTeX's default IED lists
  Let!("\\itemize", "\\IEEEitemize");
  Let!("\\enditemize", "\\endIEEEitemize");
  Let!("\\enumerate", "\\IEEEenumerate");
  Let!("\\endenumerate", "\\endIEEEenumerate");
  Let!("\\description", "\\IEEEdescription");
  Let!("\\enddescription", "\\endIEEEdescription");
  Let!(T_CS!("\\begin{itemize}"), "\\IEEEitemize");
  Let!(T_CS!("\\end{itemize}"), "\\endIEEEitemize");
  Let!(T_CS!("\\begin{enumerate}"), "\\IEEEenumerate");
  Let!(T_CS!("\\end{enumerate}"), "\\endIEEEenumerate");
  Let!(T_CS!("\\begin{description}"), "\\IEEEdescription");
  Let!(T_CS!("\\end{description}"), "\\endIEEEdescription");

  // String macros
  DefMacro!("\\contentsname", "Contents");
  DefMacro!("\\listfigurename", "List of Figures");
  DefMacro!("\\listtablename", "List of Tables");
  DefMacro!("\\refname", "References");
  DefMacro!("\\indexname", "Index");
  DefMacro!("\\figurename", "Fig.");
  DefMacro!("\\tablename", "TABLE");
  DefMacro!("\\figurename", "Figure");
  DefMacro!("\\partname", "Part");
  DefMacro!("\\appendixname", "Appendix");
  DefMacro!("\\abstractname", "Abstract");
  DefMacro!("\\IEEEkeywordsname", "Index Terms");
  DefMacro!("\\IEEEproofname", "Proof");

  // Legacy command aliases
  Let!("\\authorblockA", "\\IEEEauthorblockA");
  Let!("\\authorblockN", "\\IEEEauthorblockN");
  Let!("\\authorrefmark", "\\IEEEauthorrefmark");
  Let!("\\PARstart", "\\IEEEPARstart");
  Let!("\\pubid", "\\IEEEpubid");
  Let!("\\pubidadjcol", "\\IEEEpubidadjcol");
  Let!("\\specialpapernotice", "\\IEEEspecialpapernotice");

  // Legacy environment aliases
  DefMacro!(T_CS!("\\begin{keywords}"), None, "\\@IEEEkeywords");
  DefMacro!(T_CS!("\\end{keywords}"), None, "\\@endIEEEkeywords");
  // Perl: \keywords can take either {} or bare text
  // Simplified: just redirect to \@IEEEkeywords
  DefMacro!("\\keywords", "\\@IEEEkeywords");
  DefMacro!("\\keywords@onearg{}", "\\@IEEEkeywords #1 \\@endIEEEkeywords");

  // Legacy IED list aliases
  Let!("\\labelindent", "\\IEEElabelindent");
  Let!("\\calcleftmargin", "\\IEEEcalcleftmargin");
  Let!("\\setlabelwidth", "\\IEEEsetlabelwidth");
  Let!("\\usemathlabelsep", "\\IEEEusemathlabelsep");
  Let!("\\iedlabeljustifyc", "\\IEEEiedlabeljustifyc");
  Let!("\\iedlabeljustifyl", "\\IEEEiedlabeljustifyl");
  Let!("\\iedlabeljustifyr", "\\IEEEiedlabeljustifyr");

  // QED aliases
  Let!("\\QED", "\\IEEEQED");
  Let!("\\QEDclosed", "\\IEEEQEDclosed");
  Let!("\\QEDopen", "\\IEEEQEDopen");
  DefMacro!("\\qed", "\\ltx@qed");
  DefConstructor!("\\ltx@qed",
    "?#isMath(<ltx:XMTok role='PUNCT'>\u{220E}</ltx:XMTok>)(\u{220E})",
    enter_horizontal => true,
    reversion => "\\qed"
  );

  Let!("\\proof", "\\IEEEproof");
  Let!("\\endproof", "\\endIEEEproof");
  Let!("\\biography", "\\IEEEbiography");
  Let!("\\biographynophoto", "\\IEEEbiographynophoto");
  Let!("\\endbiography", "\\endIEEEbiography");
  Let!("\\endbiographynophoto", "\\endIEEEbiographynophoto");

  // BibTeX style control
  DefMacro!("\\bstctlcite[]{}", None);

  // Disable internal alignment environment
  DefMacro!(T_CS!("\\begin{@IEEEauthorhalign}"), None, "\\relax");
  DefMacro!(T_CS!("\\end{@IEEEauthorhalign}"), None, "\\relax");
});
