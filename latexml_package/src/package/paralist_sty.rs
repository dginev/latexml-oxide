use crate::prelude::*;

LoadDefinitions!({
  // Package options via TeX conditionals
  TeX!(
    r#"""
\newif\if@plnewitem\@plnewitemtrue
\newif\if@plnewenum\@plnewenumtrue
\newif\if@plalwaysadjust\@plalwaysadjustfalse
\newif\if@plneveradjust\@plneveradjustfalse
\newif\if@plneverdecrease\@plneverdecreasefalse
\newif\if@pldefblank\@pldefblankfalse
\newif\if@plpointedenum\@plpointedenumfalse
\newif\if@plpointlessenum\@plpointlessenumfalse
\newif\if@plflushright\@plflushrighttrue
\newif\if@plloadcfg\@plloadcfgtrue
"""#
  );

  DeclareOption!("newitem", "\\@plnewitemtrue");
  DeclareOption!("olditem", "\\@plnewitemfalse");
  DeclareOption!("newenum", "\\@plnewenumtrue");
  DeclareOption!("oldenum", "\\@plnewenumfalse");
  DeclareOption!("alwaysadjust", "\\@plalwaysadjusttrue");
  DeclareOption!("neveradjust", "\\@plneveradjusttrue");
  DeclareOption!("neverdecrease", "\\@plneverdecreasetrue");
  DeclareOption!("increaseonly", "\\@plneverdecreasetrue");
  DeclareOption!("defblank", "\\@pldefblanktrue");
  DeclareOption!("pointedenum", "\\@plpointedenumtrue");
  DeclareOption!("pointlessenum", "\\@plpointlessenumtrue");
  DeclareOption!("cfg", "\\@plloadcfgtrue");
  DeclareOption!("nocfg", "\\@plloadcfgfalse");
  DeclareOption!("flushright", "\\@plflushrighttrue");
  DeclareOption!("flushleft", "\\@plflushrightfalse");
  execute_options(&["newitem", "newenum", "cfg", "flushright"])?;
  ProcessOptions!();

  // Registers
  DefRegister!("\\pltopsep", Dimension::new(0));
  DefRegister!("\\plpartopsep", Dimension::new(0));
  DefRegister!("\\plitemsep", Dimension::new(0));
  DefRegister!("\\plparsep", Dimension::new(0));

  // paralist.sty:75-77 `\if@empty`, :434-448 `\setdefaultleftmargin` (the list indents, read by nothing here) and the
  // deprecated four-argument `\defaultleftmargin` with its warning (24 papers: 1209.4819, 1301.6228, 1308.4189);
  // :404-408 `\defaultitem` and :429-433 `\defaultenum`, both warning and calling `\setdefaultitem` — `\defaultenum`
  // too, as the file writes it (0907.0929, 1409.3601, math0106001). Perl's binding defines none of them
  // (KNOWN_PERL_ERRORS #804).
  RawTeX!(
    r"\def\if@empty#1#2#3{\def\@tempa{#1}\ifx\@tempa\@empty#2\else#3\fi}
\def\setdefaultleftmargin#1#2#3#4#5#6{%
  \if@empty{#1}{}{\leftmargini#1}%
  \if@empty{#2}{}{\leftmarginii#2}%
  \if@empty{#3}{}{\leftmarginiii#3}%
  \if@empty{#4}{}{\leftmarginiv#4}%
  \if@empty{#5}{}{\leftmarginv#5}%
  \if@empty{#6}{}{\leftmarginvi#6}%
  \relax}
\def\defaultleftmargin#1#2#3#4{%
  \setdefaultleftmargin{#1}{#2}{#3}{#4}{}{}%
  \PackageWarning{Paralist}{Macro \protect\defaultleftmargin\space deprecated. \MessageBreak Use macro
    \protect\setdefaultleftmargin\space instead!! \MessageBreak But note that it has six arguments!
    \MessageBreak Used}}
\def\defaultitem{%
  \PackageWarning{Paralist}{Macro \protect\defaultitem\space deprecated. \MessageBreak Use macro
    \protect\setdefaultitem \space instead!! \MessageBreak Used}%
  \setdefaultitem}
\def\defaultenum{%
  \PackageWarning{Paralist}{Macro \protect\defaultenum\space deprecated. \MessageBreak Use macro
    \protect\setdefaultenum \space instead!! \MessageBreak Used}%
  \setdefaultitem}"
  );

  // Enumerations
  DefMacro!("\\setdefaultenum{}{}{}{}", sub[(tag1, tag2, tag3, tag4)] {
    set_enumeration_style(Some(&tag1), Some(1))?;
    set_enumeration_style(Some(&tag2), Some(2))?;
    set_enumeration_style(Some(&tag3), Some(3))?;
    set_enumeration_style(Some(&tag4), Some(4))?;
    Tokens::new(vec![])
  });

  DefEnvironment!("{inparaenum} OptionalUndigested",
    "<ltx:inline-enumerate xml:id='#id'>#body</ltx:inline-enumerate>",
    properties => sub[_args] {
      begin_itemize("inline@enumerate", Some("enum"), BeginItemizeOptions { opens_no_list: true, ..BeginItemizeOptions::default() })
    },
    after_digest_begin => sub[whatsit] {
      if let Some(arg) = whatsit.get_arg(1) {
        set_enumeration_style(arg.raw_tokens(), None)?;
      }
    },
    mode => "internal_vertical"
  );
  DefEnvironment!("{compactenum} OptionalUndigested",
    "<ltx:enumerate xml:id='#id'>#body</ltx:enumerate>",
    properties => sub[_args] { BeginItemize!("enumerate", "enum") },
    after_digest_begin => sub[whatsit] {
      if let Some(arg) = whatsit.get_arg(1) {
        set_enumeration_style(arg.raw_tokens(), None)?;
      }
    },
    mode => "internal_vertical"
  );
  DefEnvironment!("{asparaenum} OptionalUndigested",
    "<ltx:enumerate xml:id='#id'>#body</ltx:enumerate>",
    properties => sub[_args] { BeginItemize!("enumerate", "enum") },
    after_digest_begin => sub[whatsit] {
      if let Some(arg) = whatsit.get_arg(1) {
        set_enumeration_style(arg.raw_tokens(), None)?;
      }
    },
    mode => "internal_vertical"
  );

  // Conditionally redefine stock enumerate (Perl: if (IfCondition(T_CS('\if@plnewenum'))))
  if if_condition(&T_CS!("\\if@plnewenum"))? == Some(true) {
    DefEnvironment!("{enumerate} OptionalUndigested",
      "<ltx:enumerate xml:id='#id'>#body</ltx:enumerate>",
      properties => sub[_args] { BeginItemize!("enumerate", "enum") },
      after_digest_begin => sub[whatsit] {
        if let Some(arg) = whatsit.get_arg(1) {
          set_enumeration_style(arg.raw_tokens(), None)?;
        }
      },
      locked => true,
      mode => "internal_vertical"
    );
  }

  // Itemizations
  DefMacro!("\\setdefaultitem{}{}{}{}", sub[(tag1, tag2, tag3, tag4)] {
    set_itemization_style(Some(&tag1), Some(1))?;
    set_itemization_style(Some(&tag2), Some(2))?;
    set_itemization_style(Some(&tag3), Some(3))?;
    set_itemization_style(Some(&tag4), Some(4))?;
    Tokens::new(vec![])
  });

  DefEnvironment!("{inparaitem} OptionalUndigested",
    "<ltx:inline-itemize xml:id='#id'>#body</ltx:inline-itemize>",
    properties => sub[_args] {
      begin_itemize("inline@itemize", Some("@item"), BeginItemizeOptions { opens_no_list: true, ..BeginItemizeOptions::default() })
    },
    after_digest_begin => sub[whatsit] {
      if let Some(arg) = whatsit.get_arg(1) {
        set_itemization_style(arg.raw_tokens(), None)?;
      }
    },
    mode => "internal_vertical"
  );
  DefEnvironment!("{compactitem} OptionalUndigested",
    "<ltx:itemize xml:id='#id'>#body</ltx:itemize>",
    properties => sub[_args] { BeginItemize!("itemize", "@item") },
    after_digest_begin => sub[whatsit] {
      if let Some(arg) = whatsit.get_arg(1) {
        set_itemization_style(arg.raw_tokens(), None)?;
      }
    },
    mode => "internal_vertical"
  );
  DefEnvironment!("{asparaitem} OptionalUndigested",
    "<ltx:itemize xml:id='#id'>#body</ltx:itemize>",
    properties => sub[_args] { BeginItemize!("itemize", "@item") },
    after_digest_begin => sub[whatsit] {
      if let Some(arg) = whatsit.get_arg(1) {
        set_itemization_style(arg.raw_tokens(), None)?;
      }
    },
    mode => "internal_vertical"
  );

  // Conditionally redefine stock itemize (Perl: if (IfCondition(T_CS('\if@plnewitem'))))
  if if_condition(&T_CS!("\\if@plnewitem"))? == Some(true) {
    DefEnvironment!("{itemize} OptionalUndigested",
      "<ltx:itemize xml:id='#id'>#body</ltx:itemize>",
      properties => sub[_args] { BeginItemize!("itemize", "@item") },
      after_digest_begin => sub[whatsit] {
        if let Some(arg) = whatsit.get_arg(1) {
          set_itemization_style(arg.raw_tokens(), None)?;
        }
      },
      locked => true,
      mode => "internal_vertical"
    );
  }

  // Descriptions
  DefEnvironment!("{inparadesc}",
    "<ltx:inline-description xml:id='#id'>#body</ltx:inline-description>",
    properties => sub[_args] {
      begin_itemize("inline@description", Some("@desc"), BeginItemizeOptions { opens_no_list: true, ..BeginItemizeOptions::default() })
    },
    mode => "internal_vertical"
  );
  DefEnvironment!("{compactdesc}",
    "<ltx:description xml:id='#id'>#body</ltx:description>",
    properties => sub[_args] { BeginItemize!("description", "@desc") },
    mode => "internal_vertical"
  );
  DefEnvironment!("{asparadesc}",
    "<ltx:description xml:id='#id'>#body</ltx:description>",
    properties => sub[_args] { BeginItemize!("description", "@desc") },
    mode => "internal_vertical"
  );

  // paralist.sty:380-399: under `defblank`, `{asparablank}` (a list of paragraphs without labels, 1707.03997) and
  // `{inparablank}` (an in-paragraph list without labels, 15 papers: 1205.4564, 1306.3906, 1801.06340), as
  // `{asparaitem}`/`{inparaitem}` with an empty label. Perl's binding lacks them (KNOWN_PERL_ERRORS #804).
  if if_condition(&T_CS!("\\if@pldefblank"))? == Some(true) {
    DefEnvironment!("{inparablank}",
      "<ltx:inline-itemize xml:id='#id'>#body</ltx:inline-itemize>",
      properties => sub[_args] {
        begin_itemize("inline@itemize", Some("@item"), BeginItemizeOptions { opens_no_list: true, ..BeginItemizeOptions::default() })
      },
      after_digest_begin => sub[_whatsit] { blank_item_label()?; },
      mode => "internal_vertical"
    );
    DefEnvironment!("{asparablank}",
      "<ltx:itemize xml:id='#id'>#body</ltx:itemize>",
      properties => sub[_args] { BeginItemize!("itemize", "@item") },
      after_digest_begin => sub[_whatsit] { blank_item_label()?; },
      mode => "internal_vertical"
    );
  }

  // pointedenum/pointlessenum
  TeX!(
    r#"""
\def\pl@pointxxxenum{%
  \def\theenumi{\arabic{enumi}}%
  \def\theenumii{\theenumi.\arabic{enumii}}%
  \def\theenumiii{\theenumii.\arabic{enumiii}}%
  \def\theenumiv{\theenumiii.\arabic{enumiv}}%
  \def\p@enumi{}%
  \def\p@enumii{}%
  \def\p@enumiii{}%
  \def\p@enumiv{}}
\def\pl@pointedenum{%
  \def\labelenumi{\theenumi.}%
  \def\labelenumii{\theenumii.}%
  \def\labelenumiii{\theenumiii.}%
  \def\labelenumiv{\theenumiv.}}
\def\pl@pointlessenum{%
  \def\labelenumi{\theenumi}%
  \def\labelenumii{\theenumii}%
  \def\labelenumiii{\theenumiii}%
  \def\labelenumiv{\theenumiv}}
\def\pointedenum{\pl@pointxxxenum\pl@pointedenum}
\def\pointlessenum{\pl@pointxxxenum\pl@pointlessenum}
\if@plpointedenum\pointedenum\fi
\if@plpointlessenum\pointlessenum\fi
"""#
  );
});

/// The label of the itemization opened at the current depth is empty (paralist.sty:380-399: the blank lists'
/// `\list{}{…}` and `\let\@itemlabel\@empty`).
fn blank_item_label() -> Result<()> {
  let level = match lookup_register_quiet("\\@itemdepth") {
    Some(RegisterValue::Number(n)) => n.0.max(0) as i32,
    _ => 0,
  };
  def_macro(
    T_CS!(s!("\\labelitem{}", roman_aux(level))),
    None,
    Tokens::new(vec![]),
    None,
  )?;
  Ok(())
}
