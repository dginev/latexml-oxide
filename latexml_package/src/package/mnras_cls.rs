use latexml_core::definition::origin::{DefinitionOrigin, OriginGuard};

use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: mnras.cls.ltxml

  LoadClass!("mn");
  RequirePackage!("hyperref");
  // MNRAS papers often use \color{ForestGreen} / \color{NavyBlue}
  // from the dvipsnames palette AND \cellcolor (colortbl, loaded via
  // the `table` option). Pre-load with both options so the user's
  // later `\usepackage[table, xcdraw]{xcolor}` doesn't silently
  // option-clash and leave colortbl unloaded.
  // Witness 2509.13010 ("Can't find color named 'ForestGreen'") and
  // 2305.08788 ("undefined:\\cellcolor").
  RequirePackage!("xcolor", options => vec!["dvipsnames".to_string(), "table".to_string()]);
  // mnras.cls:930-940: the class's enumerate labels, (i), (a), (1), (A), and the references they make; :1312, 1339 its
  // bibliography heading, "REFERENCES" with or without natbib; :1333-1340, with natbib, its author-year punctuation
  // "(Draine 2011)" and `\bibname` (2609.09329, 2609.08700, which printed "1." and "(Draine, 2011)").
  RawTeX!(r"\def\labelenumi{\theenumi}\def\theenumi{(\roman{enumi})}
\def\labelenumii{\theenumii}\def\theenumii{(\alph{enumii})}\def\p@enumii{\theenumi}
\def\labelenumiii{\theenumiii}\def\theenumiii{(\arabic{enumiii})}\def\p@enumiii{\theenumi(\theenumii)}
\def\labelenumiv{\theenumiv}\def\theenumiv{(\Alph{enumiv})}\def\p@enumiv{\p@enumiii\theenumiii}
\renewcommand\refname{REFERENCES}");
  if lookup_int("@usenatbib") != 0 {
    RawTeX!(r"\bibpunct{(}{)}{;}{a}{}{,}\renewcommand\bibname{BIBLIOGRAPHY}");
  }
  // mnras.cls:1349-1354: with `usedcolumn`, the class's column types on dcolumn's `D` (mn2e_support loads dcolumn).
  if lookup_int("@usedcolumn") != 0 {
    RawTeX!(r"\newcolumntype{d}[1]{D{.}{.}{#1}}\newcolumntype{.}{D{.}{.}{-1}}\newcolumntype{,}{D{,}{,}{2}}");
  }

  // mnras.cls (TL 2025 v3.2, :1399-1479; every copy papers ship, v3.0 to v3.3, alike): the reference-list links mnras.bst
  // writes, `\doi[text]{10.foo}` (a link to the DOI, "doi:10.foo" or the text, its argument read with `\$&#^_%~`
  // other), `\eprint{prefix}{id}` (an arXiv or dblp link, else "prefix:id") and `\mniiiauthor{key}{first}{again}`
  // (the bibliography style's first-time-three-authors switch). Perl's mnras.cls.ltxml defines none of them, so every
  // `\doi` of an MNRAS reference list was undefined (37 papers: 1609.03154, 2008.09832, 2104.08053; `\eprint` 17:
  // 1709.09375, 1809.04269; `\mniiiauthor` 4: 1507.08002, 1703.04544, 1806.09815; KNOWN_PERL_ERRORS #742).
  RawTeX!(r"\def\@urlcharsother{\let\do\@makeother\do\\\do\$\do\&\do\#\do\^\do\_\do\%\do\~}
\def\doi{\begingroup\@urlcharsother\@ifnextchar[{\@doi}{\@doi[]}}
\def\@doi[#1]#2{\def\@tempa{#1}\ifx\@tempa\@empty\href{http://dx.doi.org/#2}{doi:#2}\else\href{http://dx.doi.org/#2}{#1}\fi
  \endgroup}
\def\eprint#1#2{\@eprint#1:#2::\@nil}
\def\@eprint@arXiv#1{\href{http://arxiv.org/abs/#1}{{\tt arXiv:#1}}}
\def\@eprint@dblp#1{\href{http://dblp.uni-trier.de/rec/bibtex/#1.xml}{dblp:#1}}
\def\@eprint#1:#2:#3:#4\@nil{\def\@tempa{#1}\def\@tempb{#2}\def\@tempc{#3}%
  \ifx\@tempc\@empty\let\@tempc\@tempb\let\@tempb\@tempa\fi
  \ifx\@tempb\@empty\def\@tempb{arXiv}\fi
  \@ifundefined{@eprint@\@tempb}{\@tempb:\@tempc}{\expandafter\expandafter\csname @eprint@\@tempb\endcsname\expandafter{\@tempc}}}
\def\mniiiauthor#1#2#3{\@ifundefined{mniiiauth@#1}{\global\expandafter\let\csname mniiiauth@#1\endcsname\null #2}{#3}}");
  RawTeX!(r"\newcommand\aap{A\&A}");                // Astronomy and Astrophysics
  RawTeX!(r"\let\astap=\aap");                       // alternative shortcut
  RawTeX!(r"\newcommand\aapr{A\&ARv}");              // Astronomy and Astrophysics Review
  RawTeX!(r"\newcommand\aaps{A\&AS}");               // Astronomy and Astrophysics Supplement Series
  RawTeX!(r"\newcommand\actaa{Acta Astron.}");       // Acta Astronomica
  RawTeX!(r"\newcommand\afz{Afz}");                  // Astrofizika
  RawTeX!(r"\newcommand\aj{AJ}");                    // Astronomical Journal
  RawTeX!(r"\newcommand\ao{Appl. Opt.}");            // Applied Optics
  RawTeX!(r"\let\applopt=\ao");                      // alternative shortcut
  RawTeX!(r"\newcommand\aplett{Astrophys.~Lett.}");  // Astrophysics Letters
  RawTeX!(r"\newcommand\apj{ApJ}");                  // Astrophysical Journal
  RawTeX!(r"\newcommand\apjl{ApJ}");                 // Astrophysical Journal, Letters
  RawTeX!(r"\let\apjlett=\apjl");                    // alternative shortcut
  RawTeX!(r"\newcommand\apjs{ApJS}");                // Astrophysical Journal, Supplement
  RawTeX!(r"\let\apjsupp=\apjs");                    // alternative shortcut
  RawTeX!(r"\newcommand\apss{Ap\&SS}");              // Astrophysics and Space Science
  RawTeX!(r"\newcommand\araa{ARA\&A}");              // Annual Review of Astronomy and Astrophysics
  RawTeX!(r"\newcommand\arep{Astron. Rep.}");        // Astronomy Reports
  RawTeX!(r"\newcommand\aspc{ASP Conf. Ser.}");      // ASP Conference Series
  RawTeX!(r"\newcommand\azh{Azh}");                  // Astronomicheskii Zhurnal
  RawTeX!(r"\newcommand\baas{BAAS}");                // Bulletin of the AAS
  RawTeX!(r"\newcommand\bac{Bull. Astron. Inst. Czechoslovakia}");
  RawTeX!(r"\newcommand\bain{Bull. Astron. Inst. Netherlands}");
  RawTeX!(r"\newcommand\caa{Chinese Astron. Astrophys.}");
  RawTeX!(r"\newcommand\cjaa{Chinese J.~Astron. Astrophys.}");
  RawTeX!(r"\newcommand\fcp{Fundamentals Cosmic Phys.}");
  RawTeX!(r"\newcommand\gca{Geochimica Cosmochimica Acta}");
  RawTeX!(r"\newcommand\grl{Geophys. Res. Lett.}");
  RawTeX!(r"\newcommand\iaucirc{IAU~Circ.}");
  RawTeX!(r"\newcommand\icarus{Icarus}");
  RawTeX!(r"\newcommand\japa{J.~Astrophys. Astron.}");
  RawTeX!(r"\newcommand\jcap{J.~Cosmology Astropart. Phys.}");
  RawTeX!(r"\newcommand\jcp{J.~Chem.~Phys.}");
  RawTeX!(r"\newcommand\jgr{J.~Geophys.~Res.}");
  RawTeX!(r"\newcommand\jqsrt{J.~Quant. Spectrosc. Radiative Transfer}");
  RawTeX!(r"\newcommand\jrasc{J.~R.~Astron. Soc. Canada}");
  RawTeX!(r"\newcommand\memras{Mem.~RAS}");
  RawTeX!(r"\newcommand\memsai{Mem. Soc. Astron. Italiana}");
  RawTeX!(r"\newcommand\mnassa{MNASSA}");
  RawTeX!(r"\newcommand\mnras{MNRAS}");
  RawTeX!(r"\newcommand\na{New~Astron.}");
  RawTeX!(r"\newcommand\nar{New~Astron.~Rev.}");
  RawTeX!(r"\newcommand\nat{Nature}");
  RawTeX!(r"\newcommand\nphysa{Nuclear Phys.~A}");
  RawTeX!(r"\newcommand\pra{Phys. Rev.~A}");
  RawTeX!(r"\newcommand\prb{Phys. Rev.~B}");
  RawTeX!(r"\newcommand\prc{Phys. Rev.~C}");
  RawTeX!(r"\newcommand\prd{Phys. Rev.~D}");
  RawTeX!(r"\newcommand\pre{Phys. Rev.~E}");
  RawTeX!(r"\newcommand\prl{Phys. Rev.~Lett.}");
  RawTeX!(r"\newcommand\pasa{Publ. Astron. Soc. Australia}");
  RawTeX!(r"\newcommand\pasp{PASP}");
  RawTeX!(r"\newcommand\pasj{PASJ}");
  RawTeX!(r"\newcommand\physrep{Phys.~Rep.}");
  RawTeX!(r"\newcommand\physscr{Phys.~Scr.}");
  RawTeX!(r"\newcommand\planss{Planet. Space~Sci.}");
  RawTeX!(r"\newcommand\procspie{Proc.~SPIE}");
  RawTeX!(r"\newcommand\rmxaa{Rev. Mex. Astron. Astrofis.}");
  RawTeX!(r"\newcommand\qjras{QJRAS}");
  RawTeX!(r"\newcommand\sci{Science}");
  RawTeX!(r"\newcommand\skytel{Sky \& Telesc.}");
  RawTeX!(r"\newcommand\solphys{Sol.~Phys.}");
  RawTeX!(r"\newcommand\sovast{Soviet~Ast.}");
  RawTeX!(r"\newcommand\ssr{Space Sci. Rev.}");
  RawTeX!(r"\newcommand\zap{Z.~Astrophys.}");
  // v3.0 (2015/05/22, 85 of the 99 copies papers ship) lists `\apspr`, which v3.2 (TeX Live's, :510) disables as a
  // journal that "does not appear to exist": provide-only, a paper's own definition wins.
  {
    let _guess = OriginGuard::new(DefinitionOrigin::Fallback);
    RawTeX!(r"\providecommand\apspr{Astrophys.~Space~Phys.~Res.}");
  }
});
