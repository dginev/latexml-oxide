use crate::prelude::*;

LoadDefinitions!({
  // Perl: csquotes.sty.ltxml
  // Load the raw TeX style file first
  InputDefinitions!("csquotes", noltxml => true, extension => Some(Cow::Borrowed("sty")));

  // Ensure @ is catcode letter for all our internal macro definitions
  RawTeX!(r#"\makeatletter"#);

  // Provide default quote style macros — csquotes.def normally defines these
  // via DeclareQuoteStyle, but the complex TeX initialization may fail.
  // Define English-style defaults that csq@setstyle will override at \begin{document}.
  RawTeX!(
    r#"\def\csq@thequote@oinit{}%
\def\csq@thequote@oopen{\textquotedblleft}%
\def\csq@thequote@oclose{\textquotedblright}%
\def\csq@thequote@iinit{}%
\def\csq@thequote@iopen{\textquoteleft}%
\def\csq@thequote@iclose{\textquoteright}%
\let\csq@kernchar@i\relax"#
  );

  // Compatibility fixes: unicode check workaround
  RawTeX!(
    r#"\def\csq@ifutfchar#1{%
  \ifundef\@inpenc@undefined
    {\@secondoftwo}
    {\csq@ifutfenc}%
  {\csq@ifsingle{#1}
     {\ifnum`#1<128\relax
        \expandafter\@secondoftwo
      \else
        \expandafter\@secondoftwo
      \fi}
     {\@firstoftwo}}
  {\csq@ifsingle{#1}
     {\@secondoftwo}
     {\csq@err@char
      \@gobbletwo}}}"#
  );

  // work around \cl@@ckpt not being defined by LaTeXML
  RawTeX!(
    r#"\let\blockquote@prehook\relax
\newcommand*{\blockquote@prehook}{%
  \def\@elt##1{\global\value{##1}\the\value{##1}\relax}%
  \edef\csq@tempa{\@elt{page}\@elt{footnote}}%
  \let\@elt\relax
  \@fileswfalse}"#
  );

  //======================================================================
  // Typesettable quotes — override internal csquotes macros to inject ltxml markers
  //======================================================================

  // Override \csq@qopen to inject ltxml markers
  RawTeX!(
    r#"\def\csq@qopen{%
  \ifnum\csq@qlevel>\csq@maxlvl
    \csq@mismatch{%
      Level \number\csq@qlevel\space quote invalid at this point.
      The maximum level is \number\csq@maxlvl}%
  \else
    \csq@resetstyle
    \csq@init
    \csq@addkern@open
    \ifodd\csq@qlevel
      \let\csq@kernchar@i\csq@thequote@oopen
      \ltxml@oqmark@open\csq@thequote@oopen
    \else
      \let\csq@kernchar@i\csq@thequote@iopen
      \ltxml@iqmark@open\csq@thequote@iopen
    \fi
    \csq@setmarker@open
    \expandafter\csq@fixkern
  \fi}"#
  );

  // Override \csq@iqmark
  RawTeX!(
    r#"\protected\def\csq@iqmark{%
  \csq@bqgroup
  \ifnum\csq@qlevel>\@ne
    \csq@mismatch{%
      Level 2 quote invalid at this point.
      The current level is \number\csq@qlevel}%
    \advance\csq@qlevel\@ne
    \let\csq@iqmark\csq@eqerror
  \else
    \csq@qlevel\tw@
    \let\csq@iqmark{\csq@qclose\ltxml@iqmark@close}
    \ltxml@iqmark@open\expandafter\csq@qopen
  \fi}"#
  );

  // Simplify closing mark
  RawTeX!(r#"\def\csq@qclose@i{\csq@qclose@ii{}}"#);

  // Override \csq@qclose@ii to inject ltxml markers
  RawTeX!(
    r#"\def\csq@qclose@ii#1{%
  \ifdim\lastkern=\csq@omitmarker
    #1\csq@eqgroup
  \else
    \csq@addkern@close
    \ifodd\csq@qlevel
      \csq@thequote@oclose
      \ltxml@oqmark@close
      \let\csq@kernchar@i\csq@thequote@oclose
    \else
      \csq@thequote@iclose
      \ltxml@iqmark@close
      \let\csq@kernchar@i\csq@thequote@iclose
    \fi
    \ifnum\csq@qlevel>\@ne
      \csq@setmarker@close
    \fi
    \ifblank{#1}{}{\expandafter#1}%
    \expandafter\csq@eqgroup
      \expandafter\def
      \expandafter\csq@kernchar@i
      \expandafter{\csq@kernchar@i}%
    \expandafter\csq@fixkern
  \fi}"#
  );

  // Debug: check csquotes state at end of preamble

  // Constructors for quote markers
  DefConstructor!(
    "\\ltxml@oqmark@open",
    "<ltx:text class='ltx_inline-quote ltx_outerquote' _noautoclose='1'>"
  );
  DefConstructor!("\\ltxml@oqmark@close", "</ltx:text>");

  DefConstructor!(
    "\\ltxml@iqmark@open",
    "<ltx:text class='ltx_inline-quote ltx_innerquote' _noautoclose='1'>"
  );
  DefConstructor!("\\ltxml@iqmark@close", "</ltx:text>");

  //======================================================================
  // Normal Citation (\mkcitation)
  //======================================================================
  RawTeX!(
    r#"\long\def\csq@getcargs@ii#1#2[#3]{%
  #1{\ltxml@mkcitation}{#2}{#3}}"#
  );

  RawTeX!(
    r#"\let\MakeBlockQuote\relax
\newrobustcmd*{\MakeBlockQuote}[3]{%
  \csq@addbspecial{#1}{#2}{#3}{\csq@bquote{}{}{\ltxml@mkcitation}}}"#
  );

  RawTeX!(
    r#"\let\MakeForeignBlockQuote\relax
\newrobustcmd*{\MakeForeignBlockQuote}[4]{%
  \csq@addbspecial{#2}{#3}{#4}%
    {\csq@bquote{\csq@lang{#1}}{\csq@endlang}{\ltxml@mkcitation}}}"#
  );

  RawTeX!(
    r#"\let\MakeHyphenBlockQuote\relax
\newrobustcmd*{\MakeHyphenBlockQuote}[4]{%
  \csq@addbspecial{#2}{#3}{#4}%
    {\csq@bquote{\csq@hyph{#1}}{\csq@endhyph}{\ltxml@mkcitation}}}"#
  );

  RawTeX!(
    r#"\let\MakeHybridBlockQuote\relax
\newrobustcmd*{\MakeHybridBlockQuote}[4]{%
  \csq@addbspecial{#2}{#3}{#4}%
    {\csq@bquote
       {\iftoggle{csq@block}{\csq@lang}{\csq@hyph}{#1}}
       {\iftoggle{csq@block}{\csq@endlang}{\csq@endhyph}}
       {\ltxml@mkcitation}}}"#
  );

  DefMacro!(
    "\\ltxml@mkcitation{}",
    "\\ltxml@citation@open\\mkcitation{#1}\\ltxml@citation@close"
  );
  DefConstructor!(
    "\\ltxml@citation@open",
    "<ltx:text class='ltx_citation' _noautoclose='1'>"
  );
  DefConstructor!("\\ltxml@citation@close", "</ltx:text>");

  //======================================================================
  // Integrated Citation (\mkccitation)
  //======================================================================
  RawTeX!(
    r#"\long\def\csq@getccargs@iii#1#2#3[#4]{%
  #1{\ltxml@mkccitation}{\csq@cite#2{#3}}{#4}}"#
  );

  DefMacro!(
    "\\ltxml@mkccitation{}",
    "\\ltxml@ccitation@open\\mkccitation{#1}\\ltxml@ccitation@close"
  );
  DefConstructor!(
    "\\ltxml@ccitation@open",
    "<ltx:text class='ltx_ccitation' _noautoclose='1'>"
  );
  DefConstructor!("\\ltxml@ccitation@close", "</ltx:text>");

  //======================================================================
  // Text Quotes (\mktextquote)
  //======================================================================
  RawTeX!(
    r#"\long\def\csq@tquote@i#1#2#3#4#5#6#7#8#9{%
  \begingroup
  \csq@setsfcodes
  \edef\csq@tempa{%
    \unexpanded{%
      \ltxml@mktextquote
      {#3}%
      {#7}%
      {\csq@qclose@i{#2}}%
      {#6}{#8}}%
    {\ifblank{#5}
       {}
       {\unexpanded{\csq@switchlang{#4{#5}}}}}}%
  \csq@bqgroup#1\csq@tempa#9%
  \endgroup}"#
  );

  DefMacro!(
    "\\ltxml@mktextquote{}{}{}{}{}{}",
    "\\ltxml@mktextquote@open\\mktextquote{#1}{#2}{#3}{#4}{#5}{#6}\\ltxml@mktextquote@close"
  );
  DefConstructor!(
    "\\ltxml@mktextquote@open",
    "<ltx:text class='ltx_textquote' _noautoclose='1'>"
  );
  DefConstructor!("\\ltxml@mktextquote@close", "</ltx:text>");

  //======================================================================
  // Block Quotes (\mkblockquote)
  //======================================================================
  RawTeX!(
    r#"\long\def\csq@bquote@iii#1#2#3#4#5#6#7#8{%
  \ltxml@mkblockquote@open\begin{\csq@blockenvironment}%
  \toggletrue{csq@block}%
  \csq@setsfcodes
  \edef\csq@tempa{%
    \unexpanded{%
      \mkblockquote
      {\ltxml@mkblockquote@aopen#6\ltxml@mkblockquote@aclose}%
      {#5}{#7}}%
    {\ifblank{#4}
       {}
       {\unexpanded{\csq@switchlang{#3{#4}}}}}}%
  #1\csq@tempa#8#2%
  \end{\csq@blockenvironment}\ltxml@mkblockquote@close}"#
  );

  // Always use a proper block-quote environment
  RawTeX!(r#"\let\csq@bquote@ii\csq@bquote@iii"#);

  // update the 'quote' environment to be <ltx:quote class="ltx_blockquote">
  DefMacro!(
    "\\ltxml@mkblockquote@open",
    "\\begingroup\\ltxml@mkblockquote@open@i"
  );
  DefPrimitive!("\\ltxml@mkblockquote@open@i", sub [_args] {
    DefEnvironment!("{quote}",
      "<ltx:quote class='ltx_blockquote'>#body</ltx:quote>",
      mode => "internal_vertical");
  });
  DefMacro!("\\ltxml@mkblockquote@close", "\\endgroup");

  DefConstructor!(
    "\\ltxml@mkblockquote@aopen",
    "<ltx:text class='ltx_inline-quote' _noautoclose='1'>"
  );
  DefConstructor!("\\ltxml@mkblockquote@aclose", "</ltx:text>");

  //======================================================================
  // Display Quote (\mkbegdispquote, \mkenddispquote)
  //======================================================================
  RawTeX!(
    r#"\def\csq@bdquote#1#2#3#4#5{%
  \ltxml@blockenvironment@open\csuse{\csq@blockenvironment}%
  \toggletrue{csq@block}%
  \csq@setsfcodes
  #1\ifblank{#4}
    {\def\csq@tempb{\ltxml@mkenddispquote{#5}{}#2}%
     \ltxml@mkbegdispquote{#5}{}}
    {\def\csq@tempb{\ltxml@mkenddispquote{#5}{\csq@switchlang{#3{#4}}}#2}%
     \ltxml@mkbegdispquote{#5}{\csq@switchlang{#3{#4}}}}%
  \ignorespaces}"#
  );

  RawTeX!(
    r#"\def\csq@edquote{%
  \unspace\csq@tempb
  \csuse{end\csq@blockenvironment}\ltxml@blockenvironment@close}"#
  );

  // update the 'quote' environment to be <ltx:quote class="ltx_displayquote">
  DefMacro!(
    "\\ltxml@blockenvironment@open",
    "\\begingroup\\ltxml@blockenvironment@open@i"
  );
  DefPrimitive!("\\ltxml@blockenvironment@open@i", sub [_args] {
    DefEnvironment!("{quote}",
      "<ltx:quote class='ltx_displayquote'>#body</ltx:quote>",
      mode => "internal_vertical");
  });
  DefMacro!("\\ltxml@blockenvironment@close", "\\endgroup");

  DefMacro!(
    "\\ltxml@mkbegdispquote{}{}",
    "\\ltxml@mkbegdispquote@aopen\\mkbegdispquote{#1}{#2}"
  );
  DefConstructor!(
    "\\ltxml@mkbegdispquote@aopen",
    "<ltx:text class='ltx_inline-quote' _noautoclose='1'>"
  );

  DefMacro!(
    "\\ltxml@mkenddispquote{}{}",
    "\\mkenddispquote{#1}{#2}\\ltxml@mkenddispquote@aclose"
  );
  DefConstructor!("\\ltxml@mkenddispquote@aclose", "</ltx:text>");

  // Restore @ catcode
  RawTeX!(r#"\makeatother"#);
});
