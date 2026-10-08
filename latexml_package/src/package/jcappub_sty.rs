//! jcappub.sty — JCAP's SISSA/IOP publication style: jheppub's frontmatter (the same accumulating
//! `\author[affil]{name}` + `\affiliation` + `\emailAdd` API; html_feedback #6884, 2404.03569) and its own journal
//! abbreviations. Neither Perl nor Rust had a jcappub binding.
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  jheppub_sty::load_definitions()?;
  // jcappub.sty:68
  DefMacro!("\\jname", "JCAP");
  // jcappub.sty:72-146: the journal abbreviations of reference lists (`\jcap`, `\apj`, `\prl`, …), each in
  // `\jnl@style`; 2404.02153 (13 undefined).
  RawTeX!(r"\let\jnl@style=\rmfamily
\def\ref@jnl#1{{\jnl@style#1}}
\newcommand\aj{\ref@jnl{AJ}}
\newcommand\psj{\ref@jnl{PSJ}}
\newcommand\araa{\ref@jnl{ARA\&A}}
\newcommand\apj{\ref@jnl{ApJ}}
\newcommand\apjl{\ref@jnl{ApJL}}
\newcommand\apjs{\ref@jnl{ApJS}}
\newcommand\ao{\ref@jnl{ApOpt}}
\newcommand\apss{\ref@jnl{Ap\&SS}}
\newcommand\aap{\ref@jnl{A\&A}}
\newcommand\aapr{\ref@jnl{A\&A~Rv}}
\newcommand\aaps{\ref@jnl{A\&AS}}
\newcommand\azh{\ref@jnl{AZh}}
\newcommand\baas{\ref@jnl{BAAS}}
\newcommand\icarus{\ref@jnl{Icarus}}
\newcommand\jaavso{\ref@jnl{JAAVSO}}
\newcommand\jrasc{\ref@jnl{JRASC}}
\newcommand\memras{\ref@jnl{MmRAS}}
\newcommand\mnras{\ref@jnl{MNRAS}}
\newcommand\pra{\ref@jnl{PhRvA}}
\newcommand\prb{\ref@jnl{PhRvB}}
\newcommand\prc{\ref@jnl{PhRvC}}
\newcommand\prd{\ref@jnl{PhRvD}}
\newcommand\pre{\ref@jnl{PhRvE}}
\newcommand\prl{\ref@jnl{PhRvL}}
\newcommand\pasp{\ref@jnl{PASP}}
\newcommand\pasj{\ref@jnl{PASJ}}
\newcommand\qjras{\ref@jnl{QJRAS}}
\newcommand\skytel{\ref@jnl{S\&T}}
\newcommand\solphys{\ref@jnl{SoPh}}
\newcommand\sovast{\ref@jnl{Soviet~Ast.}}
\newcommand\ssr{\ref@jnl{SSRv}}
\newcommand\zap{\ref@jnl{ZA}}
\newcommand\nat{\ref@jnl{Nature}}
\newcommand\iaucirc{\ref@jnl{IAUC}}
\newcommand\aplett{\ref@jnl{Astrophys.~Lett.}}
\newcommand\apspr{\ref@jnl{Astrophys.~Space~Phys.~Res.}}
\newcommand\bain{\ref@jnl{BAN}}
\newcommand\fcp{\ref@jnl{FCPh}}
\newcommand\gca{\ref@jnl{GeoCoA}}
\newcommand\grl{\ref@jnl{Geophys.~Res.~Lett.}}
\newcommand\jcp{\ref@jnl{JChPh}}
\newcommand\jgr{\ref@jnl{J.~Geophys.~Res.}}
\newcommand\jqsrt{\ref@jnl{JQSRT}}
\newcommand\memsai{\ref@jnl{MmSAI}}
\newcommand\nphysa{\ref@jnl{NuPhA}}
\newcommand\physrep{\ref@jnl{PhR}}
\newcommand\physscr{\ref@jnl{PhyS}}
\newcommand\planss{\ref@jnl{Planet.~Space~Sci.}}
\newcommand\procspie{\ref@jnl{Proc.~SPIE}}
\newcommand\actaa{\ref@jnl{AcA}}
\newcommand\caa{\ref@jnl{ChA\&A}}
\newcommand\cjaa{\ref@jnl{ChJA\&A}}
\newcommand\jcap{\ref@jnl{JCAP}}
\newcommand\na{\ref@jnl{NewA}}
\newcommand\nar{\ref@jnl{NewAR}}
\newcommand\pasa{\ref@jnl{PASA}}
\newcommand\rmxaa{\ref@jnl{RMxAA}}
\newcommand\maps{\ref@jnl{M\&PS}}
\newcommand\aas{\ref@jnl{AAS Meeting Abstracts}}
\newcommand\dps{\ref@jnl{AAS/DPS Meeting Abstracts}}
\let\astap=\aap
\let\apjlett=\apjl
\let\apjsupp=\apjs
\let\applopt=\ao");
});
