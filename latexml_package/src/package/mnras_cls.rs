use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: mnras.cls.ltxml

  LoadClass!("mn");
  RequirePackage!("hyperref");
  // MNRAS papers often use \color{ForestGreen} / \color{NavyBlue}
  // from the dvipsnames palette but never explicitly load xcolor.
  // Eager-load xcolor[dvipsnames] so these named colors resolve.
  // Witness 2509.13010 ("Can't find color named 'ForestGreen'").
  RequirePackage!("xcolor", options => vec!["dvipsnames".to_string()]);

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
});
