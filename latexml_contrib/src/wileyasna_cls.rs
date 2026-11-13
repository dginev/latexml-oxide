//! WileyASNA-v1.cls (Astronomische Nachrichten): a Wiley NJD class ([`crate::wileynjd_cls`]), and the class's own
//! bibliography package.
use latexml_package::prelude::*;

LoadDefinitions!({
  crate::wileynjd_cls::wiley_njd_family("WileyASNA-v1")?;
  // WileyASNA-v1.cls:2120 `\RequirePackage[natbibapa]{apacite}`: the apacite macros its `.bbl` entries use
  // (`\APACinsertmetastar`, `\APACrefYearMonthDay`, `{APACrefauthors}`, 2407.11648: 30 errors).
  RequirePackage!("apacite", options => vec![s!("natbibapa")]);
  // WileyASNA-v1.cls:1300-1301: the funding agency and grant number set where the text names them, bold and italic
  // ("acknowledge the \fundingAgency{Czech Science Foundation} Grant No. \fundingNumber{16-03564Y}", 1806.06754;
  // 1810.06326, 1812.03179), each also added to `\fundagenandnum`, which `\AtEndDocument` writes to the `.finfo` file
  // for the publisher (:1303-1307), not to the page. Undefined in 70 papers of run 353.
  RawTeX!(
    r"\def\fundagenandnum{}%
\def\fundingAgency#1{\textbf{#1}\g@addto@macro\fundagenandnum{#1,\ }}%
\def\fundingNumber#1{\textit{#1}\g@addto@macro\fundagenandnum{#1.\ }}"
  );
  // WileyASNA-v1.cls:2722-2788, "Author macros included from journals.tex": the journal abbreviations of the
  // astronomy reference lists (`\aaa`, 1909.01905, 2303.03859).
  RawTeX!(
    r"\def\aaa{A\&A}\def\aaas{A\&AS}\def\mnras{MNRAS}\def\aas{A\&AS}\def\aj{AJ}\def\apj{ApJ}\def\apjl{ApJ}%
\def\apjs{ApJS}\def\baas{BAAS}\def\pasp{PASP}\def\pasj{PASJ}\def\araa{ARA\&A}\def\an{AN}\def\apss{Ap\&SS}%
\def\aap{A\&A}\def\aaps{A\&AS}\let\astap=\aap\def\aapr{A\&A~Rev.}\def\actaa{Acta Astron.}\def\ao{Appl.~Opt.}%
\let\applopt\ao\let\apjlett\apjl\let\apjsupp\apjs\def\aplett{Astrophys.~Lett.}%
\def\apspr{Astrophys.~Space~Phys.~Res.}\def\azh{AZh}\def\bac{Bull. astr. Inst. Czechosl.}%
\def\bain{Bull.~Astron.~Inst.~Netherlands}\def\caa{Chinese Astron. Astrophys.}%
\def\cjaa{Chinese J. Astron. Astrophys.}\def\fcp{Fund.~Cosmic~Phys.}\def\gca{Geochim.~Cosmochim.~Acta}%
\def\grl{Geophys.~Res.~Lett.}\def\iaucirc{IAU~Circ.}\def\icarus{Icarus}\def\jcap{J. Cosmology Astropart. Phys.}%
\def\jcp{J.~Chem.~Phys.}\def\jgr{J.~Geophys.~Res.}\def\jqsrt{J.~Quant.~Spec.~Radiat.~Transf.}\def\jrasc{JRASC}%
\def\memras{MmRAS}\def\memsai{Mem.~Soc.~Astron.~Italiana}\def\na{New A}\def\nar{New A Rev.}\def\nat{Nature}%
\def\nphysa{Nucl.~Phys.~A}\def\pasa{PASA}\def\physrep{Phys.~Rep.}\def\physscr{Phys.~Scr}%
\def\planss{Planet.~Space~Sci.}\def\pra{Phys.~Rev.~A}\def\prb{Phys.~Rev.~B}\def\prc{Phys.~Rev.~C}%
\def\prd{Phys.~Rev.~D}\def\pre{Phys.~Rev.~E}\def\prl{Phys.~Rev.~Lett.}\def\procspie{Proc.~SPIE}%
\def\qjras{QJRAS}\def\rmxaa{Rev. Mexicana Astron. Astrofis.}\def\skytel{S\&T}\def\solphys{Sol.~Phys.}%
\def\sovast{Soviet~Ast.}\def\ssr{Space~Sci.~Rev.}\def\zap{ZAp}"
  );
});
