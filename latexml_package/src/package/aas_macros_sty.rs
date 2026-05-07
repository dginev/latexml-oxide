use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: aas_macros.sty.ltxml
  // AAS : American Astronomical Society
  // Subset of AAS macros from aasguide

  //======================================================================
  // 2.13.4 Abbreviations for Journal Names

  Let!("\\jnl@style", "\\rm");
  DefMacro!("\\ref@jnl{}", "{\\jnl@style#1}");

  DefMacro!("\\aj",    "\\ref@jnl{AJ}");              // Astronomical Journal
  DefMacro!("\\actaa", "\\ref@jnl{Acta Astron.}");    // Acta Astronomica
  DefMacro!("\\araa",  "\\ref@jnl{ARA\\&A}");          // Annual Review of Astron and Astrophys
  DefMacro!("\\apj",   "\\ref@jnl{ApJ}");             // Astrophysical Journal
  DefMacro!("\\apjl",  "\\ref@jnl{ApJ}");             // Astrophysical Journal, Letters
  DefMacro!("\\apjs",  "\\ref@jnl{ApJS}");            // Astrophysical Journal, Supplement
  DefMacro!("\\ao",    "\\ref@jnl{Appl.~Opt.}");      // Applied Optics
  DefMacro!("\\apss",  "\\ref@jnl{Ap\\&SS}");          // Astrophysics and Space Science
  DefMacro!("\\aap",   "\\ref@jnl{A\\&A}");            // Astronomy and Astrophysics
  DefMacro!("\\aapr",  "\\ref@jnl{A\\&A~Rev.}");       // Astronomy and Astrophysics Reviews
  DefMacro!("\\aaps",  "\\ref@jnl{A\\&AS}");           // Astronomy and Astrophysics, Supplement
  DefMacro!("\\azh",   "\\ref@jnl{AZh}");             // Astronomicheskii Zhurnal
  DefMacro!("\\baas",  "\\ref@jnl{BAAS}");            // Bulletin of the AAS
  DefMacro!("\\bac",   "\\ref@jnl{Bull. astr. Inst. Czechosl.}");
  DefMacro!("\\caa",   "\\ref@jnl{Chinese Astron. Astrophys.}");
  DefMacro!("\\cjaa",  "\\ref@jnl{Chinese J. Astron. Astrophys.}");
  DefMacro!("\\icarus","\\ref@jnl{Icarus}");
  DefMacro!("\\jcap",  "\\ref@jnl{J. Cosmology Astropart. Phys}");
  DefMacro!("\\jrasc", "\\ref@jnl{JRASC}");
  DefMacro!("\\memras","\\ref@jnl{MmRAS}");
  DefMacro!("\\mnras", "\\ref@jnl{MNRAS}");
  DefMacro!("\\na",    "\\ref@jnl{New A}");
  DefMacro!("\\nar",   "\\ref@jnl{New A Rev.}");
  DefMacro!("\\pra",   "\\ref@jnl{Phys.~Rev.~A}");
  DefMacro!("\\prb",   "\\ref@jnl{Phys.~Rev.~B}");
  DefMacro!("\\prc",   "\\ref@jnl{Phys.~Rev.~C}");
  DefMacro!("\\prd",   "\\ref@jnl{Phys.~Rev.~D}");
  DefMacro!("\\pre",   "\\ref@jnl{Phys.~Rev.~E}");
  DefMacro!("\\prl",   "\\ref@jnl{Phys.~Rev.~Lett.}");
  DefMacro!("\\pasa",  "\\ref@jnl{PASA}");
  DefMacro!("\\pasp",  "\\ref@jnl{PASP}");
  DefMacro!("\\pasj",  "\\ref@jnl{PASJ}");
  DefMacro!("\\qjras", "\\ref@jnl{QJRAS}");
  DefMacro!("\\rmxaa", "\\ref@jnl{Rev. Mexicana Astron. Astrofis.}");
  DefMacro!("\\skytel",  "\\ref@jnl{S\\&T}");
  DefMacro!("\\solphys", "\\ref@jnl{Sol.~Phys.}");
  DefMacro!("\\sovast",  "\\ref@jnl{Soviet~Ast.}");
  DefMacro!("\\ssr",     "\\ref@jnl{Space~Sci.~Rev.}");
  DefMacro!("\\zap",     "\\ref@jnl{ZAp}");
  DefMacro!("\\nat",     "\\ref@jnl{Nature}");
  DefMacro!("\\iaucirc", "\\ref@jnl{IAU~Circ.}");
  DefMacro!("\\aplett",  "\\ref@jnl{Astrophys.~Lett.}");
  DefMacro!("\\apspr",   "\\ref@jnl{Astrophys.~Space~Phys.~Res.}");
  DefMacro!("\\bain",    "\\ref@jnl{Bull.~Astron.~Inst.~Netherlands}");
  DefMacro!("\\fcp",     "\\ref@jnl{Fund.~Cosmic~Phys.}");
  DefMacro!("\\gca",     "\\ref@jnl{Geochim.~Cosmochim.~Acta}");
  DefMacro!("\\grl",     "\\ref@jnl{Geophys.~Res.~Lett.}");
  DefMacro!("\\jcp",     "\\ref@jnl{J.~Chem.~Phys.}");
  DefMacro!("\\jgr",     "\\ref@jnl{J.~Geophys.~Res.}");
  DefMacro!("\\jqsrt",   "\\ref@jnl{J.~Quant.~Spec.~Radiat.~Transf.}");
  DefMacro!("\\memsai",  "\\ref@jnl{Mem.~Soc.~Astron.~Italiana}");
  DefMacro!("\\nphysa",  "\\ref@jnl{Nucl.~Phys.~A}");
  DefMacro!("\\physrep", "\\ref@jnl{Phys.~Rep.}");
  DefMacro!("\\physscr", "\\ref@jnl{Phys.~Scr}");
  DefMacro!("\\planss",  "\\ref@jnl{Planet.~Space~Sci.}");
  DefMacro!("\\procspie","\\ref@jnl{Proc.~SPIE}");
  // aastex631.cls L1839: \newcommand\psj{\ref@jnl{PSJ}} — Planetary
  // Science Journal abbreviation. Driver: 2306.11151.
  DefMacro!("\\psj",     "\\ref@jnl{PSJ}");

  Let!("\\astap",   "\\aap");
  Let!("\\apjlett", "\\apjl");
  Let!("\\apjsupp", "\\apjs");
  Let!("\\applopt", "\\ao");
});
