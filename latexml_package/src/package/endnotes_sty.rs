use crate::prelude::*;

LoadDefinitions!({
  NewCounter!("endnote");
  DefMacro!("\\theendnote", None, "\\arabic{endnote}");
  DefMacro!("\\endnotetyperefname", None, "\\lx@latin@name{endnote}");

  // \enotesize
  // \@makeentext to format the text of the endnote; not used (yet)!!!

  // endnotes.sty:271 `\theenmark` is `\@theenmark`, the mark of the note last made: `\endnote`, `\endnotemark` and
  // `\endnotetext` set it as the note's number when they make it (:274-282, :343-373 `\protected@xdef\@theenmark
  // {\theendnote}` after the step; `[<n>]` the number given), and `\theendnotes` as each note it lists (:400-405).
  // Perl's `\theenmark` is `\theendnote` (endnotes.sty.ltxml:29, "NOT correct"), the counter as it stands where it
  // is read: biblatex-chicago's endnote back-references (cmsendnotes.sty:164-204, which sets `\@theenmark` itself)
  // and a later `\theenmark` read the last number (cms-noteref-demo: every mark "18"). KNOWN_PERL_ERRORS #1013; guard
  // `perfect_kernel_batch77v::endnotes_theenmark_is_the_note_mark`.
  DefMacro!("\\theenmark", "\\@theenmark");
  // `\lx@enote@mark{<maker>}` `[<n>]`: the mark of the note `<maker>` is about to make, the number given, else the
  // number the counter steps to (`\lx@enote@textmark`: the counter's number, `\endnotetext` steps none). A number
  // given is formatted as the counter is (endnotes.sty:278-283, :351-357, :373-378 `\c@endnote=#1` in a group,
  // `\@theenmark` = `\theendnote`), and the note made with that mark: `\endnote[4]` under a `\roman` `\theendnote` is
  // "iv" at the note, in the list and as `\theenmark` (it was "4"). Guard
  // `perfect_kernel_batch77v::endnotes_given_number_is_formatted`.
  RawTeX!(
    r"\def\lx@enote@mark#1{\@ifnextchar[{\lx@enote@given{#1}}{\begingroup\advance\c@endnote\@ne
\protected@xdef\@theenmark{\theendnote}\endgroup#1}}
\def\lx@enote@textmark#1{\@ifnextchar[{\lx@enote@given{#1}}{\protected@xdef\@theenmark{\theendnote}#1}}
\def\lx@enote@given#1[#2]{\begingroup\c@endnote#2\relax\protected@xdef\@theenmark{\theendnote}\endgroup#1[\@theenmark]}"
  );
  DefMacro!(
    "\\makeenmark",
    r"\hbox{\textsuperscript{\normalfont\theenmark}}"
  );
  DefMacro!("\\fnum@endnote", "\\makeenmark");

  DefMacro!("\\ext@endnote", None, "ent");

  // endnotes.sty:309,323: every `\endnote`/`\endnotetext` first opens the
  // `\jobname.ent` file (`\if@enotesopen\else\@openenotes\fi`) and writes
  // the note to it; `\theendnotes` reads it back. The notes render inline
  // here (`\lx@note`), so nothing is written — but the file must still EXIST,
  // because a document that redefines `\theendnotes` raw and ends it with
  // `\input{\jobname.ent}` (latex-doc-ptr.sty:69-88) otherwise reports the
  // file missing (latex-doc-ptr, oracle pdflatex-clean; Perl endnotes.sty.ltxml
  // shares the shape and fails earlier on `\@enotes`). Keep the package's
  // invariant: the inline note also opens the (empty) file. Guard:
  // `perfect_kernel_batch56::endnote_opens_the_ent_file_for_a_raw_theendnotes`.
  DefMacro!(
    "\\endnote",
    "\\if@enotesopen\\else\\@openenotes\\fi\\lx@enote@mark{\\lx@note{endnote}}"
  );
  DefMacro!("\\endnotemark", "\\lx@enote@mark{\\lx@notemark{endnote}}");
  DefMacro!(
    "\\endnotetext",
    "\\if@enotesopen\\else\\@openenotes\\fi\\lx@enote@textmark{\\lx@notetext{endnote}}"
  );

  // \addtoendnotes{text} — appends author-typed text to the endnotes
  // list. Render as a `\\par` followed by the body so the prose
  // shows up in the output (content-preserving). The endnotes.sty
  // implementation writes the text out to the endnotes auxiliary
  // file; we don't replay that aux-file pipeline, but the text
  // belongs in the final document somehow.
  DefMacro!("\\addtoendnotes{}", "\\par #1");

  DefMacro!("\\notesname", "Notes");

  RawTeX!(
    r"
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
  "
  );

  // Note: NOT called \printendnotes!
  DefConstructor!(T_CS!("\\lx@theendnotes"), None,
    "<ltx:TOC lists='ent' scope='global' show='refnum > note'><ltx:title>#name</ltx:title></ltx:TOC>",
    properties => { stored_map!("name" => digest_in_group(T_CS!("\\notesname"))?) });

  RawTeX!(
    r"
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
  "
  );
});
