use crate::prelude::*;

LoadDefinitions!({
  NewCounter!("endnote");
  DefMacro!("\\theendnote", None, "\\arabic{endnote}");
  DefMacro!("\\endnotetyperefname", None, "endnote");

  // \theenmark  Should be assigned to the mark, by \endnote,\endnotemark !

  // \enotesize
  // \@makeentext to format the text of the endnote; not used (yet)!!!

  // This is NOT correct; it should be edef"d after the counter is stepped...
  DefMacro!("\\theenmark", "\\theendnote");
  DefMacro!(
    "\\makeenmark",
    r"\hbox{\textsuperscript{\normalfont\theenmark}}"
  );
  DefMacro!("\\fnum@endnote", "\\makeenmark");

  DefMacro!("\\ext@endnote", None, "ent");

  DefMacro!("\\endnote", "\\lx@note{endnote}");
  DefMacro!("\\endnotemark", "\\lx@notemark{endnote}");
  DefMacro!("\\endnotetext", "\\lx@notetext{endnote}");

  // \addtoendnotes{text} — appends author-typed text to the endnotes
  // list. Render as a `\\par` followed by the body so the prose
  // shows up in the output (content-preserving). The endnotes.sty
  // implementation writes the text out to the endnotes auxiliary
  // file; we don't replay that aux-file pipeline, but the text
  // belongs in the final document somehow.
  DefMacro!("\\addtoendnotes{}", "\\par #1");

  DefMacro!("\\notesname", "Notes");

  RawTeX!(r"
    \newwrite\@enotes
    \newif\if@enotesopen \global\@enotesopenfalse
    \newif\if@haveenotes \global\@haveenotesfalse
    \let\@doanenote=0
    \let\@endanenote=0
    \def\enotesize{\footnotesize}
    \def\enoteheading{\section*{\notesname\@mkboth{\MakeUppercase{\notesname}}{\MakeUppercase{\notesname}}}\mbox{}\par\vskip-\baselineskip}
    \def\enoteformat{\rightskip\z@ \leftskip\z@ \parindent=1.8em \leavevmode\llap{\makeenmark}}
    \def\@openenotes{\immediate\openout\@enotes=\jobname.ent\relax\global\@enotesopentrue}
    \newdimen\endnotesep
    \def\@theenmark{\theendnote}
    \def\@makeenmark{\hbox{\@textsuperscript{\normalfont\@theenmark}}}
    \def\@endnotemark{\leavevmode\ifhmode\edef\@x@sf{\the\spacefactor}\nobreak\fi\makeenmark\ifhmode\spacefactor\@x@sf\fi\relax}
    \long\def\@endnotetext#1{%
      \global\@haveenotestrue
      \if@enotesopen\else\@openenotes\fi
      \immediate\write\@enotes{\@doanenote{\@theenmark}}%
      \begingroup
        \def\next{#1}%
        \newlinechar='40
        \immediate\write\@enotes{\meaning\next}%
      \endgroup
      \immediate\write\@enotes{\@endanenote}}
  ");

  // Note: NOT called \printendnotes!
  DefConstructor!(T_CS!("\\lx@theendnotes"), None,
    "<ltx:TOC lists='ent' scope='global' show='refnum > note'><ltx:title>#name</ltx:title></ltx:TOC>",
    properties => { stored_map!("name" => digest(T_CS!("\\notesname"))?) });

  RawTeX!(r"
    \def\theendnotes{%
      \immediate\closeout\@enotes \global\@enotesopenfalse
      \if@haveenotes
        \begingroup
          \makeatletter
          \edef\@tempa{`\string >}%
          \ifnum\catcode\@tempa=12
            \let\@ResetGT\relax
          \else
            \edef\@ResetGT{\noexpand\catcode\@tempa=\the\catcode\@tempa}%
            \@makeother\>%
          \fi
          \def\@doanenote##1##2>{\def\@theenmark{##1}\par\begingroup
              \@ResetGT
              \edef\@currentlabel{\csname p@endnote\endcsname\@theenmark}%
              \enoteformat}
          \def\@endanenote{\par\endgroup}%
          \enoteheading
          \enotesize
          \InputIfFileExists{\jobname.ent}{}{%
             \PackageWarning{endnotes}{No endnotes found (file \jobname.ent does not exist)\MessageBreak}%
          }%
        \endgroup
      \else
        \lx@theendnotes
      \fi
    }
  ");
});
