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
    "draft", "final", "e-only",
    "titlepage", "notitlepage",
    "openright", "openany", "onecolumn", "twocolumn",
    "nomath", "noamsfonts", "psamsfonts",
    "centertags", "tbtags",
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

  // amsart.cls:103-104/:350-351 (amsbook.cls, amsproc.cls:328-329 alike): two-sided by default
  // (KNOWN_PERL_ERRORS #469; Perl's options are no-ops).
  DeclareOption!("oneside", r"\@twosidefalse\@mparswitchfalse");
  DeclareOption!("twoside", r"\@twosidetrue\@mparswitchtrue");
  RawTeX!(r"\@twosidetrue\@mparswitchtrue");
  // The size options are the class's (ams_support_sty.rs `declare_ams_size_options`).
  ams_support_sty::declare_ams_size_options()?;
  ProcessOptions!();

  // I think all options are (non)handled above, so don't need to pass any.
  // The size option set `\@ptsize` (amsart.cls:258-296); the article binding resets it (article_cls.rs), the class
  // loading no base class.
  RawTeX!(r"\let\lx@ams@ptsize\@ptsize");
  LoadClass!("article");
  RawTeX!(r"\let\@ptsize\lx@ams@ptsize");
  RequirePackage!("ams_support");
  ams_support_sty::amsart_author_storage()?;
  ams_support_sty::amsart_uppercase_nonmath()?;
  ams_support_sty::ams_class_caption_format()?;
  ams_support_sty::amsart_class_internals()?;
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
  // amsart.cls:638, :808 (amsproc.cls:613, :746): `\maketitle` runs `\@setauthors` after the title, a store documents extend
  // (2607.28440, 2610.02789, 2610.02630, 2610.08257, 2610.25104) or copy (1202.1250, 2310.04922). The binding places the
  // names, contributors and addresses, so it starts empty and runs with none of them: what else it prints is a note.
  // amsart.cls:836 (amsproc.cls:773, amsbook.cls:615): `\@setthanks` prints the stored `\thankses`, empty here (the
  // binding's `\thanks` is a note), which a document's `\@setauthors` calls as the class's ends (2606.04416).
  // A class built on these (ijmart.cls:229) has its own, which runs nothing more: what the document added runs
  // (`\lx@ams@setauthors@class`, kept where each class's load ends).
  RawTeX!(r"\let\@setauthors\@empty
\def\@setthanks{\def\thanks##1{\par##1\@addpunct.}\thankses}
\AddToHook{class/after}{\global\let\lx@ams@setauthors@class\@setauthors}
\def\lx@ams@setauthors{\ifx\@empty\authors\else
  \lx@frontmatter@footnotes{\let\authors\@empty\let\contribs\@empty\let\@setaddresses\relax
    \lx@ams@setauthors@added}\fi}");
  // (`\@setauthors` past the class's, kept at its end; the whole of it where the document replaced it. ijmart.cls:240's
  // `\if\authors\@empty\relax\else` has no `\fi`, harmless only while `\authors` holds the names: ijmsample, TL doc)
  DefMacro!("\\lx@ams@setauthors@added", sub[_args] {
    let body = |cs: &str| match lookup_meaning(&T_CS!(cs)) {
      Some(Stored::Expandable(d)) if d.get_parameters().is_none_or(|p| p.get_parameters().is_empty()) => match d.get_expansion() {
        Some(ExpansionBody::Tokens(body)) => Some(body.unlist_ref().to_vec()),
        _ => None,
      },
      _ => None,
    };
    let current = body(r"\@setauthors");
    let class = body(r"\lx@ams@setauthors@class");
    Ok(match current.as_ref().zip(class.as_ref()).and_then(|(current, class)| current.strip_prefix(class.as_slice())) {
      Some(added) => Tokens::new(added.to_vec()),
      None => Tokens!(T_CS!(r"\@setauthors")),
    })
  });
  AddToMacro!("\\lx@maketitle@body", "\\lx@ams@setauthors");
});
