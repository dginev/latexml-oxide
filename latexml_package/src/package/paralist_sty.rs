use crate::prelude::*;

LoadDefinitions!({
  // Package options via TeX conditionals
  TeX!(r#"""
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
"""#);

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
  DefRegister!("\\plpaarsep", Dimension::new(0));

  DefMacro!("\\setdefaultleftmargin{}{}{}{}{}{}", "");

  // Enumerations
  DefMacro!("\\setdefaultenum{}{}{}{}", "");

  DefEnvironment!("{inparaenum} OptionalUndigested",
    "<ltx:inline-enumerate xml:id='#id'>#body</ltx:inline-enumerate>",
    properties => sub[_args] {
      begin_itemize("inline@enumerate", Some("enum"), BeginItemizeOptions::default())
    },
    mode => "internal_vertical"
  );
  DefEnvironment!("{compactenum} OptionalUndigested",
    "<ltx:enumerate xml:id='#id'>#body</ltx:enumerate>",
    properties => sub[_args] { BeginItemize!("enumerate", "enum") },
    mode => "internal_vertical"
  );
  DefEnvironment!("{asparaenum} OptionalUndigested",
    "<ltx:enumerate xml:id='#id'>#body</ltx:enumerate>",
    properties => sub[_args] { BeginItemize!("enumerate", "enum") },
    mode => "internal_vertical"
  );

  // Itemizations
  DefMacro!("\\setdefaultitem{}{}{}{}", "");

  DefEnvironment!("{inparaitem} OptionalUndigested",
    "<ltx:inline-itemize xml:id='#id'>#body</ltx:inline-itemize>",
    properties => sub[_args] {
      begin_itemize("inline@itemize", Some("@item"), BeginItemizeOptions::default())
    },
    mode => "internal_vertical"
  );
  DefEnvironment!("{compactitem} OptionalUndigested",
    "<ltx:itemize xml:id='#id'>#body</ltx:itemize>",
    properties => sub[_args] { BeginItemize!("itemize", "@item") },
    mode => "internal_vertical"
  );
  DefEnvironment!("{asparaitem} OptionalUndigested",
    "<ltx:itemize xml:id='#id'>#body</ltx:itemize>",
    properties => sub[_args] { BeginItemize!("itemize", "@item") },
    mode => "internal_vertical"
  );

  // Descriptions
  DefEnvironment!("{inparadesc}",
    "<ltx:inline-description xml:id='#id'>#body</ltx:inline-description>",
    properties => sub[_args] {
      begin_itemize("inline@description", Some("@desc"), BeginItemizeOptions::default())
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

  // pointedenum/pointlessenum
  TeX!(r#"""
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
"""#);
});
