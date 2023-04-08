//**********************************************************************
// Other stuff
//**********************************************************************
// Some stuff that got missed in the appendices ?

use crate::package::*;
LoadDefinitions!(state, {
  RawTeX!(
    r###"
    \def\@namedef#1{\expandafter\def\csname #1\endcsname}
    \def\@nameuse#1{\csname #1\endcsname}
    \def\@cons#1#2{\begingroup\let\@elt\relax\xdef#1{#1\@elt #2}\endgroup}
    \def\@car#1#2\@nil{#1}
    \def\@cdr#1#2\@nil{#2}
    \def\@carcube#1#2#3#4\@nil{#1#2#3}
    \def\nfss@text#1{{\mbox{#1}}}
    \def\@sect#1#2#3#4#5#6[#7]#8{}
    "###
  );

  Let!("\\@begindocumenthook", "\\@empty");

  DefMacro!("\\@qend", { Tokens::new(Explode!("end")) });
  DefMacro!("\\@qrelax", { Tokens::new(Explode!("relax")) });
  DefMacro!("\\@spaces", r"\space\space\space\space");
  Let!("\\@sptoken", T_SPACE!());

  DefMacro!(
    "\\@uclclist",
    r"\oe\OE\o\O\ae\AE\dh\DH\dj\DJ\l\L\ng\NG\ss\SS\th\TH"
  );
  RawTeX!(
    r###"
  \DeclareRobustCommand{\MakeUppercase}[1]{{%
    \def\i{I}\def\j{J}%
    \def\reserved@a##1##2{\let##1##2\reserved@a}%
    \expandafter\reserved@a\@uclclist\reserved@b{\reserved@b\@gobble}%
    \let\UTF@two@octets@noexpand\@empty
    \let\UTF@three@octets@noexpand\@empty
    \let\UTF@four@octets@noexpand\@empty
    \protected@edef\reserved@a{\uppercase{#1}}%
    \reserved@a
  }}
  \DeclareRobustCommand{\MakeLowercase}[1]{{%
    \def\reserved@a##1##2{\let##2##1\reserved@a}%
    \expandafter\reserved@a\@uclclist\reserved@b{\reserved@b\@gobble}%
    \let\UTF@two@octets@noexpand\@empty
    \let\UTF@three@octets@noexpand\@empty
    \let\UTF@four@octets@noexpand\@empty
    \protected@edef\reserved@a{\lowercase{#1}}%
    \reserved@a
  }}
  \protected@edef\MakeUppercase#1{\MakeUppercase{#1}}
  \protected@edef\MakeLowercase#1{\MakeLowercase{#1}}
  "###
  );

  //======================================================================
  DefMacro!("\\@ehc", "I can't help");

  DefMacro!("\\@gobble{}", None);
  DefMacro!("\\@gobbletwo{}{}", None);
  DefMacro!("\\@gobblefour{}{}{}{}", None);
  DefMacro!("\\@firstofone{}",       sub[gullet, (first), state] { Ok(first) });
  Let!("\\@iden", "\\@firstofone");
  DefMacro!("\\@firstoftwo{}{}",     sub[gullet, (first,_second), state] { Ok(first) });
  DefMacro!("\\@secondoftwo{}{}",    sub[gullet, (_first, second), state] { Ok(second) });
  DefMacro!("\\@thirdofthree{}{}{}", sub[gullet, (_first,_second, third), state] { Ok(third) });
  DefMacro!("\\@expandtwoargs{}{}{}", sub[gullet, (first,second,third), state] {
    let mut tks = first.unlist();
    tks.push(T_BEGIN!());
    tks.append(&mut Expand!(second, gullet).unlist());
    tks.push(T_END.clone());
    tks.push(T_BEGIN!());
    tks.append(&mut Expand!(third, gullet).unlist());
    tks.push(T_END.clone());
    tks });

  DefMacro!("\\@makeother {}", sub[gullet,(arg),state] {
    let arg_str = arg.to_string();
    let mut arg_chars = arg_str.chars();
    let arg_c = match arg_chars.next() {
      Some('\\') => arg_chars.next().unwrap(),
      Some(other) => other,
      None => {
        Warn!("expected","character",gullet,state,"\\@makeother called on empty argument?");
        return Ok(Tokens!());
      }};
    state.assign_catcode(arg_c, Catcode::OTHER, Some(Scope::Local));
  });

  RawTeX!(
    r###"{\catcode`\^^M=13 \gdef\obeycr{\catcode`\^^M13 \def^^M{\\\relax}%
    \@gobblecr}%
    {\catcode`\^^M=13 \gdef\@gobblecr{\@ifnextchar
    \@gobble\ignorespaces}}%
    \gdef\restorecr{\catcode`\^^M5 }}"###
  );
  RawTeX!(
    r###"\begingroup
  \catcode`P=12
  \catcode`T=12
  \lowercase{
    \def\x{\def\rem@pt##1.##2PT{##1\ifnum##2>\z@.##2\fi}}}
  \expandafter\endgroup\x
  \def\strip@pt{\expandafter\rem@pt\the}
  \def\strip@prefix#1>{}
  \def\@sanitize{\@makeother\ \@makeother\\\@makeother\$\@makeother\&%
  \@makeother\#\@makeother\^\@makeother\_\@makeother\%\@makeother\~}
  \def \@onelevel@sanitize #1{%
    \edef #1{\expandafter\strip@prefix
            \meaning #1}%
  }
  \def\dospecials{\do\ \do\\\do\{\do\}\do\$\do\&%
    \do\#\do\^\do\_\do\%\do\~}"###
  );

  DefMacro!(
    "\\nfss@catcodes",
    r###"\makeatletter
    \catcode`\ 9%
    \catcode`\^^I9%
    \catcode`\^^M9%
    \catcode`\\\z@
    \catcode`\{\@ne
    \catcode`\}\tw@
    \catcode`\#6%
    \catcode`\^7%
    \catcode`\%14%
    \@makeother\<%
    \@makeother\>%
    \@makeother\*%
    \@makeother\.%
    \@makeother\-%
    \@makeother\/%
    \@makeother\[%
    \@makeother\]%
    \@makeother\`%
    \@makeother\'%
    \@makeother\"%
    "###
  );
  DefMacro!("\\ltx@hard@MessageBreak", None, "^^J");

  DefPrimitive!("\\@onlypreamble{}", sub[stomach,(arg),state] {
    only_preamble("\\@onlypreamble", stomach, state); }); // Don't bother enforcing this.
  DefPrimitive!("\\GenericError{}{}{}{}", sub[stomach,(arg1,arg2,arg3,arg4),state] {
    make_generic_message("\\GenericError", vec![arg2, arg3, arg4], "error", stomach, state)?;
  });
  DefPrimitive!("\\GenericWarning{}{}", sub[stomach,(arg1,arg2),state] {
    make_generic_message("\\GenericWarning", vec![arg1,arg2], "warn", stomach, state)?;
  });
  DefPrimitive!("\\GenericInfo{}{}", sub[stomach,(arg1,arg2),state] {
    make_generic_message("\\GenericInfo", vec![arg1,arg2], "info", stomach, state)?;
  });

  Let!("\\MessageBreak", "\\relax");
  RawTeX!(
    r###"
     \gdef\PackageError#1#2#3{%
       \GenericError{%
           (#1)\@spaces\@spaces\@spaces\@spaces
        }{%
           Package #1 Error: #2%
        }{%
           See the #1 package documentation for explanation.%
        }{#3}%
     }
     \def\PackageWarning#1#2{%
       \GenericWarning{%
           (#1)\@spaces\@spaces\@spaces\@spaces
        }{%
           Package #1 Warning: #2%
        }%
     }
     \def\PackageWarningNoLine#1#2{%
       \PackageWarning{#1}{#2\@gobble}}
     \def\PackageInfo#1#2{%
       \GenericInfo{%
           (#1) \@spaces\@spaces\@spaces
        }{%
           Package #1 Info: #2%
        }%
     }
     \def\ClassError#1#2#3{%
       \GenericError{%
           (#1) \space\@spaces\@spaces\@spaces
        }{%
           Class #1 Error: #2%
        }{%
           See the #1 class documentation for explanation.%
        }{#3}%
     }
     \def\ClassWarning#1#2{%
       \GenericWarning{%
           (#1) \space\@spaces\@spaces\@spaces
        }{%
           Class #1 Warning: #2%
        }%
     }
     \def\ClassWarningNoLine#1#2{%
       \ClassWarning{#1}{#2\@gobble}}
     \def\ClassInfo#1#2{%
       \GenericInfo{%
           (#1) \space\space\@spaces\@spaces
        }{%
           Class #1 Info: #2%
        }%
     }
     \def\@latex@error#1#2{%
       \GenericError{%
           \space\space\space\@spaces\@spaces\@spaces
        }{%
           LaTeX Error: #1%
        }{%
           See the LaTeX manual or LaTeX Companion for explanation.%
        }{#2}%
     }
     \def\@latex@warning#1{%
       \GenericWarning{%
           \space\space\space\@spaces\@spaces\@spaces
        }{%
           LaTeX Warning: #1%
        }%
     }
     \def\@latex@warning@no@line#1{%
       \@latex@warning{#1\@gobble}}
     \def\@latex@info#1{%
       \GenericInfo{%
           \@spaces\@spaces\@spaces
        }{%
           LaTeX Info: #1%
        }%
     }
     \def\@latex@info@no@line#1{%
       \@latex@info{#1\@gobble}}
     "###
  );
  DefPrimitive!("\\@setsize{}{}{}{}", None);
  Let!("\\@warning", "\\@latex@warning");
  Let!("\\@@warning", "\\@latex@warning@no@line");
  DefMacro!("\\G@refundefinedtrue", None);
  DefMacro!(
    "\\@nomath{}",
    r"\relax\ifmmode\@font@warning{Command \noexpand#1invalid in math mode}\fi"
  );
  DefMacro!(
    "\\@font@warning{}",
    r"\GenericWarning{(Font)\@spaces\@spaces\@spaces\space\space}{LaTeX Font Warning: #1}"
  );
  //======================================================================
  RawTeX!(
    r###"
    \chardef\@xxxii=32
    \mathchardef\@Mi=10001
    \mathchardef\@Mii=10002
    \mathchardef\@Miii=10003
    \mathchardef\@Miv=10004
    \def\@fontenc@load@list{\@elt{T1,OT1}}
  "###
  );

  DefMacro!("\\@vpt", "5");
  DefMacro!("\\@vipt", "6");
  DefMacro!("\\@viipt", "7");
  DefMacro!("\\@viiipt", "8");
  DefMacro!("\\@ixpt", "9");
  DefMacro!("\\@xpt", "10");
  DefMacro!("\\@xipt", "10.95");
  DefMacro!("\\@xiipt", "12");
  DefMacro!("\\@xivpt", "14.4");
  DefMacro!("\\@xviipt", "17.28");
  DefMacro!("\\@xxpt", "20.74");
  DefMacro!("\\@xxvpt", "24.88");

  DefMacro!("\\@tempa", None);
  DefMacro!("\\@tempb", None);
  DefMacro!("\\@tempc", None);
  DefMacro!("\\@gtempa", None);

  RawTeX!(
    r###"
    \long\def\loop#1\repeat{%
      \def\iterate{#1\relax\expandafter\iterate\fi}%
      \iterate%
      \let\iterate\relax}
    \newdimen\@ydim
    \let\@@hyph=\-
    \newbox\@arstrutbox
    \newbox\@begindvibox
    \newcount\@botnum
    \newdimen\@botroom
    \newcount\@chclass
    \newcount\@chnum
    \newdimen\@clnht
    \newdimen\@clnwd
    \newdimen\@colht
    \newcount\@colnum
    \newdimen\@colroom
    \newbox\@curfield
    \newbox\@curline
    \newcount\@currtype
    \newcount\@curtab
    \newcount\@curtabmar
    \newbox\@dashbox
    \newcount\@dashcnt
    \newdimen\@dashdim
    \newcount\@dbltopnum
    \newdimen\@dbltoproom
    \let\@dischyph=\-
    \newcount\@enumdepth
    \newcount\@floatpenalty
    \newdimen\@fpmin
    \newcount \@fpstype
    \newcount\@highpenalty
    \newcount\@hightab
    \newbox\@holdpg
    \newinsert \@kludgeins
    \newcount\@lastchclass
    \newbox\@leftcolumn
    \newbox\@linechar
    \newdimen\@linelen
    \newcount\@lowpenalty
    \newdimen\@maxdepth
    \newcount\@medpenalty
    \newdimen\@mparbottom \@mparbottom\z@
    \newinsert\@mpfootins
    \newcount\@mplistdepth
    \newcount\@multicnt
    \newcount\@nxttabmar
    \newbox\@outputbox
    \newdimen\@pagedp
    \newdimen\@pageht
    \newbox\@picbox
    \newdimen\@picht
    \newdimen \@reqcolroom
    \newskip\@rightskip \@rightskip \z@skip
    \newcount\@savsf
    \newdimen\@savsk
    \newcount\@secpenalty
    \def\@sqrt[#1]{\root #1\of}
    \newbox\@tabfbox
    \newcount\@tabpush
    \newdimen \@textfloatsheight
    \newdimen\@textmin
    \newcount\@topnum
    \newdimen\@toproom
    \newcount\@xarg
    \newdimen\@xdim
    \newcount\@yarg
    \newdimen\@ydim
    \newcount\@yyarg
    \newtoks\every@math@size
    \newif \if@fcolmade
    \newdimen\lower@bound
    \newcount\par@deathcycles
    \newdimen\upper@bound
    \newif\if@insert
    \newif\if@colmade
    \newif\if@specialpage   \@specialpagefalse
    \newif\if@firstcolumn   \@firstcolumntrue
    \newif\if@twocolumn     \@twocolumnfalse
    \newif\if@twoside       \@twosidefalse
    \newif\if@reversemargin \@reversemarginfalse
    \newif\if@mparswitch    \@mparswitchfalse
    \newcount\col@number    \@ne
    \newread\@inputcheck
    \newwrite\@unused
    \newwrite\@mainaux
    \newwrite\@partaux
    \let\@auxout=\@mainaux
    \openout\@mainaux\jobname.aux
    \newcount\@clubpenalty
    \@clubpenalty \clubpenalty
    \newif\if@filesw \@fileswtrue
    \newif\if@partsw \@partswfalse
    \def\@tempswafalse{\let\if@tempswa\iffalse}
    \def\@tempswatrue{\let\if@tempswa\iftrue}
    \let\if@tempswa\iffalse
    \newcount\@tempcnta
    \newcount\@tempcntb
    \newif\if@tempswa
    \newdimen\@tempdima
    \newdimen\@tempdimb
    \newdimen\@tempdimc
    \newbox\@tempboxa
    \newskip\@tempskipa
    \newskip\@tempskipb
    \newtoks\@temptokena
    \newskip\@flushglue \@flushglue = 0pt plus 1fil
    \newif\if@afterindent\@afterindenttrue
    \newbox\rootbox

    \newcount\@eqcnt
    \newcount\@eqpen
    \newif\if@eqnsw\@eqnswtrue
    \newskip\@centering
    \@centering = 0pt plus 1000pt
    \let\@eqnsel=\relax

     \long\def\@whilenum#1\do #2{\ifnum #1\relax #2\relax\@iwhilenum{#1\relax
          #2\relax}\fi}
     \long\def\@iwhilenum#1{\ifnum #1\expandafter\@iwhilenum
              \else\expandafter\@gobble\fi{#1}}
     \long\def\@whiledim#1\do #2{\ifdim #1\relax#2\@iwhiledim{#1\relax#2}\fi}
     \long\def\@iwhiledim#1{\ifdim #1\expandafter\@iwhiledim
             \else\expandafter\@gobble\fi{#1}}
     \long\def\@whilesw#1\fi#2{#1#2\@iwhilesw{#1#2}\fi\fi}
     \long\def\@iwhilesw#1\fi{#1\expandafter\@iwhilesw
              \else\@gobbletwo\fi{#1}\fi}
    \def\@nnil{\@nil}
    \def\@fornoop#1\@@#2#3{}
    \long\def\@for#1:=#2\do#3{%
      \expandafter\def\expandafter\@fortmp\expandafter{#2}%
      \ifx\@fortmp\@empty \else
        \expandafter\@forloop#2,\@nil,\@nil\@@#1{#3}\fi}
    \long\def\@forloop#1,#2,#3\@@#4#5{\def#4{#1}\ifx #4\@nnil \else
           #5\def#4{#2}\ifx #4\@nnil \else#5\@iforloop #3\@@#4{#5}\fi\fi}
    \long\def\@iforloop#1,#2\@@#3#4{\def#3{#1}\ifx #3\@nnil
           \expandafter\@fornoop \else
          #4\relax\expandafter\@iforloop\fi#2\@@#3{#4}}
    \def\@tfor#1:={\@tf@r#1 }
    \long\def\@tf@r#1#2\do#3{\def\@fortmp{#2}\ifx\@fortmp\space\else
        \@tforloop#2\@nil\@nil\@@#1{#3}\fi}
    \long\def\@tforloop#1#2\@@#3#4{\def#3{#1}\ifx #3\@nnil
           \expandafter\@fornoop \else
          #4\relax\expandafter\@tforloop\fi#2\@@#3{#4}}
    \long\def\@break@tfor#1\@@#2#3{\fi\fi}
    \def\remove@to@nnil#1\@nnil{}
    \def\remove@angles#1>{\set@simple@size@args}
    \def\remove@star#1*{#1}
    \def\@defaultunits{\afterassignment\remove@to@nnil}

    \newif\ifmath@fonts \math@fontstrue
    \newbox\@labels
    \newif\if@inlabel \@inlabelfalse
    \newif\if@newlist   \@newlistfalse
    \newif\if@noparitem \@noparitemfalse
    \newif\if@noparlist \@noparlistfalse
    \newif\if@noitemarg \@noitemargfalse
    \newif\if@nmbrlist  \@nmbrlistfalse

    \def\glb@settings{}%
    "###
  );

  DefMacro!("\\@height", None, "height");
  DefMacro!("\\@width", None, "width");
  DefMacro!("\\@depth", None, "depth");
  DefMacro!("\\@minus", None, "minus");
  DefMacro!("\\@plus", None, "plus");
  DefMacro!("\\hmode@bgroup", None, "\\leavevmode\\bgroup");

  DefMacro!(T_CS!("\\@backslashchar"), None, T_OTHER!("\\"));
  DefMacro!(T_CS!("\\@percentchar"), None, T_OTHER!("%"));
  DefMacro!(T_CS!("\\@charlb"), None, T_LETTER!("{"));
  DefMacro!(T_CS!("\\@charrb"), None, T_LETTER!("}"));
  // ======================================================================

  DefMacro!("\\check@mathfonts", None);
  DefMacro!("\\fontsize{}{}", None);
  // https://tex.stackexchange.com/questions/112492/setfontsize-vs-fontsize#112501
  DefMacro!("\\@setfontsize{}{}{}", "\\let\\@currsize#1");

  DefMacro!(T_CS!("\\@vpt"), None, T_OTHER!("5"));
  DefMacro!(T_CS!("\\@vipt"), None, T_OTHER!("6"));
  DefMacro!(T_CS!("\\@viipt"), None, T_OTHER!("7"));
  DefMacro!(T_CS!("\\@viiipt"), None, T_OTHER!("8"));
  DefMacro!(T_CS!("\\@ixpt"), None, T_OTHER!("9"));
  DefMacro!("\\@xpt", "10");
  DefMacro!("\\@xipt", "10.95");
  DefMacro!("\\@xiipt", "12");
  DefMacro!("\\@xivpt", "14.4");
  DefMacro!("\\@xviipt", "17.28");
  DefMacro!("\\@xxpt", "20.74");
  DefMacro!("\\@xxvpt", "24.88");
  DefMacro!("\\vpt", r"\edef\f@size{\@vpt}\rm");
  DefMacro!("\\vipt", r"\edef\f@size{\@vipt}\rm");
  DefMacro!("\\viipt", r"\edef\f@size{\@viipt}\rm");
  DefMacro!("\\viiipt", r"\edef\f@size{\@viiipt}\rm");
  DefMacro!("\\ixpt", r"\edef\f@size{\@ixpt}\rm");
  DefMacro!("\\xpt", r"\edef\f@size{\@xpt}\rm");
  DefMacro!("\\xipt", r"\edef\f@size{\@xipt}\rm");
  DefMacro!("\\xiipt", r"\edef\f@size{\@xiipt}\rm");
  DefMacro!("\\xivpt", r"\edef\f@size{\@xivpt}\rm");
  DefMacro!("\\xviipt", r"\edef\f@size{\@xviipt}\rm");
  DefMacro!("\\xxpt", r"\edef\f@size{\@xxpt}\rm");
  DefMacro!("\\xxvpt", r"\edef\f@size{\@xxvpt}\rm");

  DefMacro!("\\defaultscriptratio", None, ".7");
  DefMacro!("\\defaultscriptscriptratio", None, ".5");

  //======================================================================

  DefMacro!("\\loggingoutput", None);
  DefMacro!("\\loggingall", None);
  DefMacro!("\\tracingfonts", None);
  DefMacro!("\\showoverfull", None);
  DefMacro!("\\showoutput", None);
  DefMacro!("\\wlog{}", "");
});
