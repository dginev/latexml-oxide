//! turing.sty — Turing machine simulation
//! Perl: turing.sty.ltxml — 222 lines
//! Simulates Turing machines with tape, states, and transition rules.
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl turing.sty.ltxml L18-33: pre-RawTeX block defining \turingrules,
  // the four state flags, and a provides-package announcement.
  RawTeX!(r#"\NeedsTeXFormat{LaTeX2e}[1995/12/01]
\ProvidesPackage{turing}[2006/02/17]
\message{This is Turing-Tex.}

\def\turingrules#1{
  \xdef\rules{#1,(,,,,),}
}

\newif\ifnorulegiven
\newif\ifnorulefound
\newif\iftmfull
\newif\ifnextvalue
"#);

  // Perl L35-36: tape symbols.
  DefMacro!("\\newworld", "=");
  DefMacro!("\\blank",    "-");

  // Perl L37-39: \newtm SkipMatch:< Until:, Until:> SkipMatch:( Until:)
  DefMacro!(
    "\\newtm SkipMatch:< Until:, Until:> SkipMatch:( Until:)",
    "\\xdef\\tm{(=,)(#1)(#3,=,)}\\xdef\\stopstate{#2}\\xdef\\stopreached{no}"
  );

  // Perl L40-45: \findstate SkipMatch:( Until:, Until:) SkipMatch:( Until:)
  //   SkipMatch:( Until:, Until:, Until:) SkipMatch:;
  DefMacro!(
    "\\findstate SkipMatch:( Until:, Until:) SkipMatch:( Until:) \
     SkipMatch:( Until:, Until:, Until:) SkipMatch:;",
    "\\xdef\\farleft{#2}\\xdef\\left{#1}\\xdef\\state{#3}\
     \\xdef\\value{#4}\\xdef\\right{#5}\\xdef\\farright{#6}"
  );

  // Perl L47-54: \findrule — drives the rule-lookup loop.
  DefMacro!(
    "\\findrule",
    "\\norulegivenfalse\\norulefoundtrue\
     \\xdef\\remrules{\\rules}\
     \\loop\\expandafter\\findr\\remrules;\\ifnorulefound\\repeat"
  );

  // Perl L56-72: \findr with the same SkipMatch-heavy signature.
  DefMacro!(
    "\\findr SkipMatch:( Until:, Until:, Until:, Until:, Until:) \
     SkipMatch:, Until:;",
    "\\edef\\stpe{#1}\\edef\\stpv{#2}\
     \\ifx\\state\\stpe\\ifx\\value\\stpv\
       \\norulefoundfalse\\xdef\\newstate{#3}\\xdef\\newvalue{#4}\
       \\xdef\\direction{#5}\
     \\fi\\fi\
     \\xdef\\remrules{#6}\
     \\ifx\\empty\\remrules\\norulefoundfalse\\norulegiventrue\\fi"
  );

  // Perl L74-108: \nextstep assembles a step and dispatches by direction.
  DefMacro!(
    "\\nextstep",
    "\\expandafter\\findstate\\tm;\\findrule\
     \\ifx\\newstate\\stopstate\\xdef\\stopreached{yes}\\fi\
     \\ifnorulegiven\
       \\ifx\\state\\stopstate\
         \\message{Turing machine reached stop state.}\
       \\else\
         \\message{Rule not found for (state,value) (\\state,\\value)}\
       \\fi\
     \\else\
       \\def\\leftm{L}\\def\\rightm{R}\\def\\middlem{H}\
       \\if\\direction\\leftm\
         \\if\\left\\newworld\
           \\xdef\\tm{(=,\\farleft)(\\newstate)(-,\\newvalue,\\right,\\farright)}\
         \\else\
           \\xdef\\tm{(\\farleft)(\\newstate)(\\left,\\newvalue,\\right,\\farright)}\
         \\fi\
       \\else\\if\\direction\\rightm\
         \\if\\right\\newworld\
           \\xdef\\tm{(\\newvalue,\\left,\\farleft)(\\newstate)(-,=,\\farright)}\
         \\else\
           \\xdef\\tm{(\\newvalue,\\left,\\farleft)(\\newstate)(\\right,\\farright)}\
         \\fi\
       \\else\\if\\direction\\middlem\
         \\xdef\\tm{(\\left,\\farleft)(\\newstate)(\\newvalue,\\right,\\farright)}\
       \\fi\\fi\\fi\
     \\fi"
  );

  // Perl L110-125: the two display DefConstructors. Perl uses
  // `framed => 'underline'` and `'rectangle'` attributes, which produce
  // <ltx:text framed='underline'>…</ltx:text> / `'rectangle'` wrappers.
  // Perl's afterDigest substitutes `\hbox{ }` when the first arg is
  // empty so the frame rectangle/underline has minimum width.
  DefConstructor!(
    "\\spec {}",
    "<ltx:text framed='underline'>#1</ltx:text>",
    enter_horizontal => true,
    after_digest => sub[whatsit] {
      let arg_empty = whatsit.get_arg(1)
        .map(|a| a.to_string().is_empty())
        .unwrap_or(true);
      if arg_empty {
        let hbox = stomach::digest(mouth::tokenize_internal("\\hbox{ }"))?;
        whatsit.set_args(vec![Some(hbox)]);
      }
      Ok(Vec::new())
    }
  );
  DefConstructor!(
    "\\speca {}",
    "<ltx:text framed='rectangle'>#1 </ltx:text>",
    enter_horizontal => true,
    after_digest => sub[whatsit] {
      let arg_empty = whatsit.get_arg(1)
        .map(|a| a.to_string().is_empty())
        .unwrap_or(true);
      if arg_empty {
        let hbox = stomach::digest(mouth::tokenize_internal("\\hbox{ }"))?;
        whatsit.set_args(vec![Some(hbox)]);
      }
      Ok(Vec::new())
    }
  );

  // Perl L127-218: the post-RawTeX block defining \showtm, \mkleft,
  // \mkright, \stepandshow, \runtm, \loopstep. Verbatim port.
  RawTeX!(r#"\def\showtm{
  \xdef\result{}

  \expandafter\findstate\tm;

  \tmfulltrue
  \edef\remtm{\left,\farleft}
  \loop
    \expandafter\mkleft\remtm;
  \iftmfull
  \repeat

  \if\value\blank
    \xdef\result{\result\speca{}\ }
  \else
    \xdef\result{\result\speca{\value}\ }
  \fi

  \tmfulltrue
  \edef\remtm{\right,\farright}
  \loop
    \expandafter\mkright\remtm;
  \iftmfull
  \repeat

  \mbox{}\result\hskip3em(\state)
}

\def\mkleft#1,#2;{
  \edef\next{#1}
  \if\next\newworld
    \xdef\result{\spec{}\ \result}
  \else\if\next\blank
    \xdef\result{\result\spec{}\ }
  \else
    \xdef\result{\spec{#1}\ \result}
  \fi\fi
  \edef\remtm{#2}
  \ifx\empty\remtm
    \tmfullfalse
  \fi
}

\def\mkright#1,#2;{
  \edef\next{#1}
  \if\next\newworld
    \xdef\result{\result\spec{}\ }
  \else\if\next\blank
    \xdef\result{\result\spec{}\ }
  \else
    \xdef\result{\result\spec{#1}\ }
  \fi\fi
  \edef\remtm{#2}
  \ifx\empty\remtm
    \tmfullfalse
  \fi
}

\def\stepandshow#1{%
\newcount\tmit
\tmit=#1
\loop
   {\nextstep\showtm}
   \advance\tmit by -1
   \ifnum\tmit>0
\repeat
}

\def\runtm{%
\loop
   {\nextstep\showtm}
   \def\next{no}
   \ifx\next\stopreached
\repeat
}

\def\loopstep#1{%
\newcount\tmit
\tmit=#1
\loop
   {\nextstep}
   \advance\tmit by -1
   \ifnum\tmit>0
\repeat
}
"#);
});
