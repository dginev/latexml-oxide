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
    "###
  );

  // DefMacro('\@ifdefinable DefToken {}', sub {
  //     my ($gullet, $token, $if) = @_;
  //     if (isDefinable($token)) {
  //       return $if->unlist }
  //     else {
  //       my ($slash, @s) = ExplodeText($token->toString);
  //       DefMacroI('\reserved@a', undef, Tokens(@s));
  //       return (T_CS('\@notdefinable')); } });

  // Let('\@@ifdefinable', '\@ifdefinable');

  // DefMacro('\@rc@ifdefinable DefToken {}', sub {
  //     my ($gullet, $token, $if) = @_;
  //     Let('\@ifdefinable', '\@@ifdefinable');
  //     return $if->unlist; });

  // DefMacroI('\@notdefinable', undef,
  //   '\@latex@error{%
  //    Command \@backslashchar\reserved@a\space
  //    already defined.
  //    Or name \@backslashchar\@qend... illegal,
  //    see p.192 of the manual}');

  DefMacro!("\\@qend", sub[_a,_b,_c] { Tokens::new(Explode!("end")) });
  DefMacro!("\\@qrelax", sub[_a,_b,_c] { Tokens::new(Explode!("relax")) } );
  DefMacro!("\\@spaces", r"\space\space\space\space");
  Let!("\\@sptoken", T_SPACE!());

  // DefMacroI('\@uclclist', undef, '\oe\OE\o\O\ae\AE\dh\DH\dj\DJ\l\L\ng\NG\ss\SS\th\TH');

  // DefMacro('\MakeUppercase{}', sub {
  //     my @t = LookupDefinition(T_CS('\@uclclist'))->getExpansion->unlist;
  //     my @x = (T_CS('\def'), T_CS('\i'), T_BEGIN, T_LETTER('I'), T_END,
  //       T_CS('\def'), T_CS('\j'), T_BEGIN, T_LETTER('J'), T_END);
  //     while (@t) { push(@x, T_CS('\let'), shift(@t), shift(@t)); }
  //     my $arg = Expand(Tokens(T_BEGIN, @x, $_[1]->unlist, T_END));
  //     (T_CS('\uppercase'), T_BEGIN, $arg->unlist, T_END); });

  // DefMacro('\MakeLowercase{}', sub {
  //     my @t = LookupDefinition(T_CS('\@uclclist'))->getExpansion->unlist;
  //     my @x = ();
  //     while (@t) { my $y = shift(@t); push(@x, T_CS('\let'), shift(@t), $y); }
  //     my $arg = Expand(Tokens(T_BEGIN, @x, $_[1]->unlist, T_END));
  //     (T_CS('\lowercase'), T_BEGIN, $arg->unlist, T_END); });

  // #======================================================================

  // DefMacroI('\@ehc', undef, "I can't help");

  DefMacro!(r"\@gobble{}", "");
  DefMacro!(r"\@gobbletwo{}{}", "");
  DefMacro!(r"\@gobblefour{}{}{}{}", "");
  DefMacro!(r"\@firstofone{}",       sub[gullet, args, state] { Ok(args[0].clone()) });
  Let!("\\@iden", "\\@firstofone");
  DefMacro!("\\@firstoftwo{}{}",     sub[gullet, args, state] { unpack!(args=>one,two); Ok(one) });
  DefMacro!("\\@secondoftwo{}{}",    sub[gullet, args, state] { unpack!(args=>one,two); Ok(two) });
  DefMacro!("\\@thirdofthree{}{}{}", sub[gullet, args, state] { unpack!(args=>one,two, three); Ok(three) });
  // DefMacro('\@expandtwoargs{}{}{}', sub {
  //     ($_[1]->unlist, T_BEGIN, Expand($_[2])->unlist, T_END, T_BEGIN, Expand($_[3])->unlist, T_END); });
  DefMacro!("\\@makeother{}", sub[gullet,args,state] {
    unpack_to_token!(args=>arg);
    let arg_c = arg.get_string().chars().next().unwrap();
    state.assign_catcode(arg_c, Catcode::OTHER, Some(Scope::Local));
  });

  // TODO: Stubs until we can deal with the rawtex fully
  DefMacro!("\\dospecials", "");
  //   RawTeX!(
  //     r###"
  //  {\catcode`\^^M=13 \gdef\obeycr{\catcode`\^^M13 \def^^M{\\\relax}%
  //      \@gobblecr}%
  //  {\catcode`\^^M=13 \gdef\@gobblecr{\@ifnextchar
  //  \@gobble\ignorespaces}}%
  //  \gdef\restorecr{\catcode`\^^M5 }}
  //  \begingroup
  //    \catcode`P=12
  //    \catcode`T=12
  //    \lowercase{
  //      \def\x{\def\rem@pt##1.##2PT{##1\ifnum##2>\z@.##2\fi}}}
  //    \expandafter\endgroup\x
  //  \def\strip@pt{\expandafter\rem@pt\the}
  //  \def\strip@prefix#1>{}
  //  \def\@sanitize{\@makeother\ \@makeother\\\@makeother\$\@makeother\&%
  //  \@makeother\#\@makeother\^\@makeother\_\@makeother\%\@makeother\~}
  //  \def \@onelevel@sanitize #1{%
  //    \edef #1{\expandafter\strip@prefix
  //             \meaning #1}%
  //  }
  //  \def\dospecials{\do\ \do\\\do\{\do\}\do\$\do\&%
  //    \do\#\do\^\do\_\do\%\do\~}
  // "###
  //   );

  // DefMacroI('\nfss@catcodes', undef, <<'EOMacro');
  //     \makeatletter
  //     \catcode`\ 9%
  //      \catcode`\^^I9%
  //      \catcode`\^^M9%
  //      \catcode`\\\z@
  //      \catcode`\{\@ne
  //      \catcode`\}\tw@
  //      \catcode`\#6%
  //      \catcode`\^7%
  //      \catcode`\%14%
  //    \@makeother\<%
  //    \@makeother\>%
  //    \@makeother\*%
  //    \@makeother\.%
  //    \@makeother\-%
  //    \@makeother\/%
  //    \@makeother\[%
  //    \@makeother\]%
  //    \@makeother\`%
  //    \@makeother\'%
  //    \@makeother\"%
  // EOMacro
  // DefMacroI('\ltx@hard@MessageBreak', undef, '^^J');

  // sub make_message {
  //   my ($cmd, @args) = @_;
  //   my $stomach = $STATE->getStomach;
  //   $stomach->bgroup;
  //   Let('\protect',      '\string');
  //   Let('\MessageBreak', '\ltx@hard@MessageBreak');    # tricky, we need Expand() to execute it
  //   my $message = join("", map { ToString(Expand($_, T_CS('\MessageBreak'))) } @args);
  //   $stomach->egroup;
  //   return ('latex', $cmd, $stomach, $message); }

  // DefPrimitive('\@onlypreamble{}', sub { onlyPreamble('\@onlypreamble'); }); # Don't bother enforcing this.
  // DefPrimitive('\GenericError{}{}{}{}', sub { Error(make_message('\GenericError', $_[2], $_[3], $_[4])); });
  // DefPrimitive('\GenericWarning{}{}', sub { Warn(make_message('\GenericWarning', $_[1], $_[2])); });
  // DefPrimitive('\GenericInfo{}{}',    sub { Info(make_message('\GenericInfo',    $_[1], $_[2])); });

  // Let('\MessageBreak', '\relax');
  //   RawTeX!(
  //     r###"
  //   \gdef\PackageError#1#2#3{%
  //     \GenericError{%
  //         (#1)\@spaces\@spaces\@spaces\@spaces
  //      }{%
  //         Package #1 Error: #2%
  //      }{%
  //         See the #1 package documentation for explanation.%
  //      }{#3}%
  //   }
  //   \def\PackageWarning#1#2{%
  //     \GenericWarning{%
  //         (#1)\@spaces\@spaces\@spaces\@spaces
  //      }{%
  //         Package #1 Warning: #2%
  //      }%
  //   }
  //   \def\PackageWarningNoLine#1#2{%
  //     \PackageWarning{#1}{#2\@gobble}}
  //   \def\PackageInfo#1#2{%
  //     \GenericInfo{%
  //         (#1) \@spaces\@spaces\@spaces
  //      }{%
  //         Package #1 Info: #2%
  //      }%
  //   }
  //   \def\ClassError#1#2#3{%
  //     \GenericError{%
  //         (#1) \space\@spaces\@spaces\@spaces
  //      }{%
  //         Class #1 Error: #2%
  //      }{%
  //         See the #1 class documentation for explanation.%
  //      }{#3}%
  //   }
  //   \def\ClassWarning#1#2{%
  //     \GenericWarning{%
  //         (#1) \space\@spaces\@spaces\@spaces
  //      }{%
  //         Class #1 Warning: #2%
  //      }%
  //   }
  //   \def\ClassWarningNoLine#1#2{%
  //     \ClassWarning{#1}{#2\@gobble}}
  //   \def\ClassInfo#1#2{%
  //     \GenericInfo{%
  //         (#1) \space\space\@spaces\@spaces
  //      }{%
  //         Class #1 Info: #2%
  //      }%
  //   }
  //   \def\@latex@error#1#2{%
  //     \GenericError{%
  //         \space\space\space\@spaces\@spaces\@spaces
  //      }{%
  //         LaTeX Error: #1%
  //      }{%
  //         See the LaTeX manual or LaTeX Companion for explanation.%
  //      }{#2}%
  //   }
  //   \def\@latex@warning#1{%
  //     \GenericWarning{%
  //         \space\space\space\@spaces\@spaces\@spaces
  //      }{%
  //         LaTeX Warning: #1%
  //      }%
  //   }
  //   \def\@latex@warning@no@line#1{%
  //     \@latex@warning{#1\@gobble}}
  //   \def\@latex@info#1{%
  //     \GenericInfo{%
  //         \@spaces\@spaces\@spaces
  //      }{%
  //         LaTeX Info: #1%
  //      }%
  //   }
  //   \def\@latex@info@no@line#1{%
  //     \@latex@info{#1\@gobble}}
  //   "###
  //   );

  // DefPrimitive('\@setsize{}{}{}{}', undef);
  // Let('\@warning',  '\@latex@warning');
  // Let('\@@warning', '\@latex@warning@no@line');

  // DefMacro('\G@refundefinedtrue', '');

  // DefMacro('\@nomath{}',
  //   '\relax\ifmmode\@font@warning{Command \noexpand#1invalid in math mode}\fi');
  // DefMacro('\@font@warning{}',
  //   '\GenericWarning{(Font)\@spaces\@spaces\@spaces\space\space}{LaTeX Font Warning: #1}');

  // #======================================================================

  RawTeX!(
    r###"
    \chardef\@xxxii=32
    \mathchardef\@Mi=10001
    \mathchardef\@Mii=10002
    \mathchardef\@Miii=10003
    \mathchardef\@Miv=10004
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
  DefMacro!("\\@tempa", "");
  DefMacro!("\\@tempb", "");
  DefMacro!("\\@tempc", "");
  DefMacro!("\\@gtempa", "");

  RawTeX!(
    r###"
    \long\def \loop #1\repeat{%
      \def\iterate{#1\relax  % Extra \relax
                   \expandafter\iterate\fi
                   }%
      \iterate
      \let\iterate\relax
    }
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

  // DefMacroI('\@height', undef, 'height');
  // DefMacroI('\@width',  undef, 'width');
  // DefMacroI('\@depth',  undef, 'depth');
  // DefMacroI('\@minus',  undef, 'minus');
  // DefMacroI('\@plus',   undef, 'plus');

  // DefMacroI('\hmode@bgroup', undef, '\leavevmode\bgroup');

  // DefMacroI('\@backslashchar', undef, T_OTHER('\\'));
  // DefMacroI('\@percentchar',   undef, T_OTHER('%'));
  // DefMacroI('\@charlb',        undef, T_LETTER('{'));
  // DefMacroI('\@charrb',        undef, T_LETTER('}'));
  // #======================================================================

  // DefMacroI('\check@mathfonts', undef, Tokens());
  // DefMacro('\fontsize{}{}', Tokens());
  // # https://tex.stackexchange.com/questions/112492/setfontsize-vs-fontsize#112501
  // DefMacro('\@setfontsize{}{}{}', Tokens());

  // DefMacroI('\defaultscriptratio',       undef, '.7');
  // DefMacroI('\defaultscriptscriptratio', undef, '.5');

  // #======================================================================
  // DefMacroI('\loggingoutput', undef, Tokens());
  // DefMacroI('\loggingall',    undef, Tokens());
  // DefMacroI('\tracingfonts',  undef, Tokens());
  // DefMacroI('\showoverfull',  undef, Tokens());
  // DefMacroI('\showoutput',    undef, Tokens());
  DefMacro!("\\wlog{}", "");
});
