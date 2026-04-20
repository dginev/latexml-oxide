//! JHEP.cls — Journal of High Energy Physics document class
//! Perl: JHEP.cls.ltxml — 314 lines (mostly journal abbreviation macros)
use crate::engine::latex_constructs::{after_float, before_float};
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl L26-35: Class options
  DeclareOption!("proceedings", {});
  DeclareOption!("published", {});
  DeclareOption!("hyper", {});
  DeclareOption!("nohyper", {});
  DeclareOption!("notoc", {});
  ProcessOptions!();
  load_class("article", Vec::new(), Tokens!())?;
  RequirePackage!("amssymb");
  // The raw JHEP.cls loads hyperref, giving authors \href / \url / \hypertarget
  // without an explicit \usepackage{hyperref}. Perl's JHEP.cls.ltxml omits
  // this, but the real-world paper corpus (e.g. arxiv 1010.4240 via PoS →
  // JHEP) depends on hyperref being active. Load it here so the arxiv sandbox
  // resolves \href without an Error:undefined cascade.
  RequirePackage!("hyperref");

  // Perl L40-58: Frontmatter
  DefMacro!("\\speaker{}", "\\@add@frontmatter{ltx:creator}[role=speaker]{\\@personname{#1}}");
  DefConstructor!("\\@@@abstract{}", "^ <ltx:abstract name='#name'>#1</ltx:abstract>",
    properties => { stored_map!("name" => Stored::from("Abstract")) }
  );
  DefMacro!("\\abstract{}", "\\@add@to@frontmatter{ltx:abstract}{\\@@@abstract{#1}}");
  DefConstructor!("\\@@@email{}", "^ <ltx:contact role='email'>#1</ltx:contact>");
  DefMacro!("\\email Semiverbatim", "\\@add@to@frontmatter{ltx:creator}{\\@@@email{#1}}");
  DefMacro!("\\received{}", "\\@add@frontmatter{ltx:date}[role=received]{#1}");
  DefMacro!("\\revised{}", "\\@add@frontmatter{ltx:date}[role=revised]{#1}");
  DefMacro!("\\accepted{}", "\\@add@frontmatter{ltx:date}[role=accepted]{#1}");
  DefMacro!("\\JHEPcopydate{}", "\\@add@frontmatter{ltx:date}[role=copydate]{#1}");
  DefMacro!("\\dedicated{}", "\\@add@frontmatter{ltx:note}[role=dedicated]{#1}");
  DefMacro!("\\conference{}", "\\@add@frontmatter{ltx:note}[role=conference]{#1}");
  DefMacro!("\\preprint{}", "\\@add@frontmatter{ltx:note}[role=preprint]{#1}");
  DefMacro!("\\keywords{}", "\\@add@frontmatter{ltx:keywords}{#1}");

  // Perl L61-64: Acknowledgements environment
  DefConstructor!("\\acknowledgments", "<ltx:acknowledgements name='Acknowledgments'>");
  DefConstructor!("\\endacknowledgments", "</ltx:acknowledgements>");

  // Perl L67-76: Misc macros
  DefMacro!("\\hash", "\\#");
  DefMacro!("\\secstyle", "\\bfseries");
  DefMacro!("\\militarytime", "\\time");
  Let!("\\textref", "\\ref");
  DefMacro!("\\tocsecs", "");
  DefMacro!("\\logo", "JHEP");
  DefMacro!("\\JHEP{}", "");
  DefMacro!("\\PrHEP{}", "");
  DefMacro!("\\Proof", "\\emph{Proof.}\\ ");

  // Perl L80-83: Figure/table macros (map to environments)
  // Perl wraps into `{floatingfigure}` / `{floatingtable}` so the nested
  // `\caption` sees a proper `\@captype`. Previously Rust expanded to bare
  // `#2` which dumped the caption into text-mode and triggered
  // `Error:unexpected:\caption (outside any known float)`.
  DefMacro!("\\FIGURE[]{}", "\\begin{floatingfigure}[#1]#2\\end{floatingfigure}");
  DefMacro!("\\TABLE[]{}",  "\\begin{floatingtable}[#1]#2\\end{floatingtable}");
  DefMacro!("\\EPSFIGURE[]{}{}", "\\begin{floatingfigure}[#1]\\epsfig{file=#2}\\caption{#3}\\end{floatingfigure}");
  DefMacro!("\\TABULAR[]{}{}{}",
    "\\begin{floatingtable}[#1]\\begin{tabular}{#2}#3\\end{tabular}\\caption{#4}\\end{floatingtable}");

  // Perl JHEP.cls.ltxml L85-89: \DOUBLEFIGURE[pos]{img1}{img2}{cap1}{cap2}
  DefMacro!("\\DOUBLEFIGURE[]{}{}{}{}",
    "\\begin{figure}[#1]\
     \\begin{@half@doublefigure}\\epsfig{file=#2}\\caption{#4}\\end{@half@doublefigure}\
     \\begin{@half@doublefigure}\\epsfig{file=#3}\\caption{#5}\\end{@half@doublefigure}\
     \\end{figure}");
  DefEnvironment!("{@half@doublefigure}",
    "<ltx:figure xml:id='#id' inlist='#inlist' width='0.45%'>#body</ltx:figure>#tags",
    before_digest => { before_float("figure", None); },
    after_digest  => sub[whatsit] { after_float(whatsit); },
    mode => "internal_vertical");

  // Perl JHEP.cls.ltxml L96-100: \DOUBLETABLE[pos]{tab1}{tab2}{cap1}{cap2}
  DefMacro!("\\DOUBLETABLE[]{}{}{}{}",
    "\\begin{table}[#1]\
     \\begin{@half@doubletable}#2\\caption{#4}\\end{@half@doubletable}\
     \\begin{@half@doubletable}#3\\caption{#5}\\end{@half@doubletable}\
     \\end{table}");
  DefEnvironment!("{@half@doubletable}",
    "<ltx:table xml:id='#id' inlist='#inlist' width='0.45%'>#body</ltx:table>#tags",
    before_digest => { before_float("table", None); },
    after_digest  => sub[whatsit] { after_float(whatsit); },
    mode => "internal_vertical");

  // Perl JHEP.cls.ltxml L109-117: JHEP-specific {floatingfigure} without
  // the `{Dimension}` width arg that the standalone floatfig package uses.
  DefEnvironment!("{floatingfigure}[]",
    "<ltx:figure xml:id='#id' inlist='#inlist' float='#float'>#tags#body</ltx:figure>",
    before_digest => { before_float("figure", None); },
    after_digest  => sub[whatsit] { after_float(whatsit); },
    properties    => sub[args] {
      let pos = args[0].as_ref().map(|a| a.to_string()).unwrap_or_default();
      let float = if pos.starts_with('v') || pos.starts_with('r') { "right" } else { "left" };
      Ok(stored_map!("float" => float))
    },
    mode => "internal_vertical");
  DefEnvironment!("{floatingtable}[]",
    "<ltx:table xml:id='#id' inlist='#inlist' float='#float'>#tags#body</ltx:table>",
    before_digest => { before_float("table", None); },
    after_digest  => sub[whatsit] { after_float(whatsit); },
    properties    => sub[args] {
      let pos = args[0].as_ref().map(|a| a.to_string()).unwrap_or_default();
      let float = if pos.starts_with('v') || pos.starts_with('r') { "right" } else { "left" };
      Ok(stored_map!("float" => float))
    },
    mode => "internal_vertical");

  // Perl L133-137: Hyperref stubs
  DefMacro!("\\JHEPspecialurl Semiverbatim", "");
  DefMacro!("\\base Semiverbatim", "");
  DefMacro!("\\name Semiverbatim", "");

  // Perl L143: SPIRES URL generator
  DefMacro!("\\@spires{}", "\\href{http://www-spires.slac.stanford.edu/spires/find/hep/www?j=#1}");

  // Journal abbreviation macros with SPIRES links — Perl L145-240
  DefMacro!("\\ap{}{}{}",    "\\@spires{APNYA\\%2C#1\\%2C#3}{{\\it Ann.\\ Phys.\\ (NY) }{\\bf #1} (#2) #3}");
  DefMacro!("\\cqg{}{}{}",   "\\@spires{CQGRD\\%2C#1\\%2C#3}{{\\it Class.\\ and Quant.\\ Grav.\\ }{\\bf #1} (#2) #3}");
  // Perl JHEP.cls.ltxml L169 — Computer Physics Communications journal alias
  DefMacro!("\\cpc{}{}{}",   "\\@spires{CPHCB\\%2C#1\\%2C#3}{{\\it Comput.\\ Phys.\\ Commun.\\ }{\\bf #1} (#2) #3}");
  DefMacro!("\\cmp{}{}{}",   "\\@spires{CMPHA\\%2C#1\\%2C#3}{{\\it Commun.\\ Math.\\ Phys.\\ }{\\bf #1} (#2) #3}");
  DefMacro!("\\epjc{}{}{}",  "\\@spires{EPHJA\\%2CC#1\\%2C#3}{{\\it Eur.\\ Phys.\\ J. }{\\bf C #1} (#2) #3}");
  DefMacro!("\\grg{}{}{}",   "\\@spires{GRGVA\\%2C#1\\%2C#3}{{\\it Gen.\\ Rel.\\ Grav.\\ }{\\bf #1} (#2) #3}");
  DefMacro!("\\ijmpa{}{}{}", "\\@spires{IMPAE\\%2CA#1\\%2C#3}{{\\it Int.\\ J.\\ Mod.\\ Phys.\\ }{\\bf A #1} (#2) #3}");
  DefMacro!("\\ijmpb{}{}{}", "\\@spires{IMPAE\\%2CB#1\\%2C#3}{{\\it Int.\\ J.\\ Mod.\\ Phys.\\ }{\\bf B #1} (#2) #3}");
  DefMacro!("\\jhep{}{}{}",  "\\href{http://jhep.sissa.it/stdsearch?paper=#1\\%28#2\\%29#3}{{\\it J. High Energy Phys.\\ }{\\bf #1} (#2) #3}");
  DefMacro!("\\jmp{}{}{}",   "\\@spires{JMAPA\\%2C#1\\%2C#3}{{\\it J.\\ Math.\\ Phys.\\ }{\\bf #1} (#2) #3}");
  DefMacro!("\\mpla{}{}{}",  "\\@spires{MPLAE\\%2CA#1\\%2C#3}{{\\it Mod.\\ Phys.\\ Lett.\\ }{\\bf A #1} (#2) #3}");
  DefMacro!("\\nature{}{}{}","\\@spires{NATUA\\%2C#1\\%2C#3}{{\\it Nature }{\\bf #1} (#2) #3}");
  DefMacro!("\\npa{}{}{}",   "\\@spires{NUPHA\\%2CA#1\\%2C#3}{{\\it Nucl.\\ Phys.\\ }{\\bf A #1} (#2) #3}");
  DefMacro!("\\npb{}{}{}",   "\\@spires{NUPHA\\%2CB#1\\%2C#3}{{\\it Nucl.\\ Phys.\\ }{\\bf B #1} (#2) #3}");
  DefMacro!("\\plb{}{}{}",   "\\@spires{PHLTA\\%2CB#1\\%2C#3}{{\\it Phys.\\ Lett.\\ }{\\bf B #1} (#2) #3}");
  DefMacro!("\\prd{}{}{}",   "\\@spires{PHRVA\\%2CD#1\\%2C#3}{{\\it Phys.\\ Rev.\\ }{\\bf D #1} (#2) #3}");
  DefMacro!("\\prep{}{}{}",  "\\@spires{PRPLC\\%2C#1\\%2C#3}{{\\it Phys.\\ Rept.\\ }{\\bf #1} (#2) #3}");
  DefMacro!("\\prl{}{}{}",   "\\@spires{PRLTA\\%2C#1\\%2C#3}{{\\it Phys.\\ Rev.\\ Lett.\\ }{\\bf #1} (#2) #3}");
  DefMacro!("\\rmp{}{}{}",   "\\@spires{RMPHA\\%2C#1\\%2C#3}{{\\it Rev.\\ Mod.\\ Phys.\\ }{\\bf #1} (#2) #3}");
  DefMacro!("\\ptp{}{}{}",   "\\@spires{PTPKA\\%2C#1\\%2C#3}{{\\it Prog.\\ Theor.\\ Phys.\\ }{\\bf #1} (#2) #3}");
  DefMacro!("\\newjournal{}{}{}{}{}", "\\@spires{#2\\%2C#3\\%2C#5}{{\\it #1 }{\\bf #3} (#4) #5}");
  DefMacro!("\\ibid{}{}{}",  "{\\it ibid.\\ }{\\bf #1} (#2) #3");

  // arXiv category links — Perl L244-261
  DefMacro!("\\hepth{}",   "\\href{http://xxx.lanl.gov/abs/hep-th/#1}{\\tt hep-th/#1}");
  DefMacro!("\\hepph{}",   "\\href{http://xxx.lanl.gov/abs/hep-ph/#1}{\\tt hep-ph/#1}");
  DefMacro!("\\heplat{}",  "\\href{http://xxx.lanl.gov/abs/hep-lat/#1}{\\tt hep-lat/#1}");
  DefMacro!("\\hepex{}",   "\\href{http://xxx.lanl.gov/abs/hep-ex/#1}{\\tt hep-ex/#1}");
  DefMacro!("\\nuclth{}",  "\\href{http://xxx.lanl.gov/abs/nucl-th/#1}{\\tt nucl-th/#1}");
  DefMacro!("\\nuclex{}",  "\\href{http://xxx.lanl.gov/abs/nucl-ex/#1}{\\tt nucl-ex/#1}");
  DefMacro!("\\grqc{}",    "\\href{http://xxx.lanl.gov/abs/gr-qc/#1}{\\tt gr-qc/#1}");
  DefMacro!("\\astroph{}", "\\href{http://xxx.lanl.gov/abs/astro-ph/#1}{\\tt astro-ph/#1}");
  DefMacro!("\\condmat{}", "\\href{http://xxx.lanl.gov/abs/cond-mat/#1}{\\tt cond-mat/#1}");
  DefMacro!("\\quantph{}", "\\href{http://xxx.lanl.gov/abs/quant-ph/#1}{\\tt quant-ph/#1}");
  DefMacro!("\\Math{}{}", "\\href{http://xxx.lanl.gov/abs/math.#1/#2}{\\tt math.#1/#2}");

  // Conditionals — Perl L267-291
  TeX!(r"
  \newif\if@preprint\@preprinttrue
  \newif\if@draft\@draftfalse
  \newif\if@hyper\@hypertrue
  \newif\if@proc\@procfalse
  \newif\if@author\@authorfalse
  \newif\if@abstract\@abstractfalse
  \newif\if@keywords\@keywordsfalse
  \newif\if@todotoc\@todotocfalse
  \newif\if@rece\@recefalse
  \newif\if@revi\@revifalse
  \newif\if@acce\@accefalse
  \newif\if@conf\@conffalse
  \newif\if@speaker\@speakerfalse
  ");

  // Perl L293-308: Names
  DefMacro!("\\acknowlname", "Acknowledgments");
  DefMacro!("\\receivedname", "Received:");
  DefMacro!("\\revisedname", "Revised:");
  DefMacro!("\\acceptedname", "Accepted:");
  DefMacro!("\\keywordsname", "Keywords:");
  DefMacro!("\\abstractname", "Abstract:");
  DefMacro!("\\JHEP@todaysname", "");
  DefMacro!("\\preprintname", "PREPRINT");
  DefMacro!("\\appendixname", "Appendix");
  DefMacro!("\\contentsname", "Contents");
  DefMacro!("\\listfigurename", "List of figures");
  DefMacro!("\\listtablename", "List of tables");
  DefMacro!("\\refname", "References");
  DefMacro!("\\figurename", "Figure");
  DefMacro!("\\tablename", "Table");
  DefMacro!("\\partname", "Part");
});
