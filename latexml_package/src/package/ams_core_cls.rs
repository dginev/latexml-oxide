use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: ams_core.cls.ltxml — common support for amsart, amsproc, amsbook

  //======================================================================
  // Document structure.

  // None of the options are vital, I think; deferred.
  // [though loading an unwanted amsfonts (noamsfonts) could be an issue]
  for option in [
    "a4paper", "letterpaper", "landscape", "portrait",
    "oneside", "twoside", "draft", "final", "e-only",
    "titlepage", "notitlepage",
    "openright", "openany", "onecolumn", "twocolumn",
    "nomath", "noamsfonts", "psamsfonts",
    "centertags", "tbtags",
    "8pt", "9pt", "10pt", "11pt", "12pt",
    "makeidx",
  ].iter() {
    DeclareOption!(*option, None);
  }
  // amsart.cls:159-162: `leqno` and `reqno` also go to amsmath, whose own
  // switch sets the tags (amsmath.sty:53-55; the binding resets `ltx_leqno`
  // as it loads, DIVERGENCES #336). `leqno` is the default (amsart.cls:350).
  DeclareOption!("leqno", sub {
    AssignMapping!("DOCUMENT_CLASSES", "ltx_leqno" => true);
    Digest!("\\PassOptionsToPackage{leqno}{amsmath}")?;
  });
  DeclareOption!("reqno", sub {
    assign_mapping("DOCUMENT_CLASSES", "ltx_leqno", None::<bool>);
    Digest!("\\PassOptionsToPackage{reqno}{amsmath}")?;
  });
  DeclareOption!("fleqn", sub { AssignMapping!("DOCUMENT_CLASSES", "ltx_fleqn" => true); });
  execute_options(&["leqno"])?; // Default is left!

  ProcessOptions!();

  // I think all options are (non)handled above, so don't need to pass any.
  load_class("article", Vec::new(), Tokens!())?;
  RequirePackage!("ams_support");
  ams_support_sty::amsart_author_storage()?;
  ams_support_sty::amsart_uppercase_nonmath()?;
  // amsart.cls:518-520 (amsproc.cls alike): `\\enddoc@text`, run `\\AtEndDocument`, sets the translators and the
  // addresses after the body (`\\@settranslators`, `\\@setaddresses`, :524-577, ported raw), and a derived class
  // queues its own end matter there (resphilosophica.cls:94 `\\AddtoEndMatter`: its `{notes}` collection, :432). The
  // binding's `\\address`, `\\translator` and `\\thanks` go to the frontmatter, so amsart's accumulators (:505, :571)
  // stay empty and the hook sets only what a class queued or filled itself (smfart.cls:420-422 appends to
  // `\\addresses`). Perl defines no `\\enddoc@text` (ams_core.cls.ltxml): the queued end matter was lost
  // (KNOWN_PERL_ERRORS #443). Repro sectioning-frontmatter/amsart_end_matter_is_set.
  RawTeX!(r"\let\thankses\@empty\let\@translators\@empty
\def\enddoc@text{\ifx\@empty\@translators \else\@settranslators\fi
  \ifx\@empty\addresses \else\@setaddresses\fi}
\AtEndDocument{\enddoc@text}
\def\@setaddresses{\par
  \nobreak \begingroup
\footnotesize
  \def\author##1{\nobreak\addvspace\bigskipamount}%
  \def\\{\unskip, \ignorespaces}%
  \interlinepenalty\@M
  \def\address##1##2{\begingroup
    \par\addvspace\bigskipamount\indent
    \@ifnotempty{##1}{(\ignorespaces##1\unskip) }%
    {\scshape\ignorespaces##2}\par\endgroup}%
  \def\curraddr##1##2{\begingroup
    \@ifnotempty{##2}{\nobreak\indent\curraddrname
      \@ifnotempty{##1}{, \ignorespaces##1\unskip}\/:\space
      ##2\par}\endgroup}%
  \def\email##1##2{\begingroup
    \@ifnotempty{##2}{\nobreak\indent\emailaddrname
      \@ifnotempty{##1}{, \ignorespaces##1\unskip}\/:\space
      \ttfamily##2\par}\endgroup}%
  \def\urladdr##1##2{\begingroup
    \def~{\char`\~}%
    \@ifnotempty{##2}{\nobreak\indent\urladdrname
      \@ifnotempty{##1}{, \ignorespaces##1\unskip}\/:\space
      \ttfamily##2\par}\endgroup}%
  \addresses
  \endgroup
}
\def\@settranslators{\par\begingroup
  \addvspace{6\p@\@plus9\p@}%
  \hbox to\columnwidth{\hss\normalfont\normalsize
    \translname{ }%
    \andify\@translators \uppercasenonmath\@translators
    \@translators}
  \endgroup
}");
});
