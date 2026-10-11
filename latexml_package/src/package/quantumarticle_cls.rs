use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: quantumarticle.cls.ltxml — Quantum Journal
  // See https://github.com/quantum-journal/quantum-journal

  load_class("article", Vec::new(), Tokens!())?;
  // quantumarticle.cls:197-205: a REVTeX journal option turns on its REVTeX compatibility. That provides the REVTeX
  // front-matter commands, which it does not print (:1462-1471, at `\begin{document}`, so a document's own stays its
  // own), and the journal abbreviations of REVTeX's `.rtx` files (:1429-1460). What `\pacs` and `\preprint` discard is
  // kept as metadata only, not displayed: each `\pacs`'s codes as RDFa `dcterms:subject` (every call, beside the
  // author's own `pdfkeywords`), `\preprint`'s report number as RDFa `dcterms:identifier`. Witnesses 1705.03968,
  // 1709.01098 (`\pacs`), 2006.05524, 2212.01355 (`\preprint`).
  for option in ["aps", "pra", "prb", "pre", "prl", "prx", "aip"] {
    DeclareOption!(option, { AssignValue!("quantumarticle_revtex_compat" => true); });
  }
  ProcessOptions!();
  if lookup_bool("quantumarticle_revtex_compat") {
    RawTeX!(r"\providecommand{\ao}{Appl.\  Opt.}
\providecommand{\ap}{Appl.\  Phys.}
\providecommand{\apl}{Appl.\ Phys.\ Lett.}
\providecommand{\apj}{Astrophys.\ J.}
\providecommand{\bell}{Bell Syst.\ Tech.\ J.}
\providecommand{\jqe}{IEEE J.\ Quantum Electron.}
\providecommand{\assp}{IEEE Trans.\ Acoust.\ Speech Signal Process.}
\providecommand{\aprop}{IEEE Trans.\ Antennas Propag.}
\providecommand{\mtt}{IEEE Trans.\ Microwave Theory Tech.}
\providecommand{\iovs}{Invest.\ Ophthalmol.\ Vis.\ Sci.}
\providecommand{\jcp}{J.\ Chem.\ Phys.}
\providecommand{\jmo}{J.\ Mod.\ Opt.}
\providecommand{\josa}{J.\ Opt.\ Soc.\ Am.}
\providecommand{\josaa}{J.\ Opt.\ Soc.\ Am.\ A}
\providecommand{\josab}{J.\ Opt.\ Soc.\ Am.\ B}
\providecommand{\jpp}{J.\ Phys.\ (Paris)}
\providecommand{\nat}{Nature (London)}
\providecommand{\oc}{Opt.\ Commun.}
\providecommand{\ol}{Opt.\ Lett.}
\providecommand{\pl}{Phys.\ Lett.}
\providecommand{\pra}{Phys.\ Rev.\ A}
\providecommand{\prb}{Phys.\ Rev.\ B}
\providecommand{\prc}{Phys.\ Rev.\ C}
\providecommand{\prd}{Phys.\ Rev.\ D}
\providecommand{\pre}{Phys.\ Rev.\ E}
\providecommand{\prl}{Phys.\ Rev.\ Lett.}
\providecommand{\rmp}{Rev.\ Mod.\ Phys.}
\providecommand{\pspie}{Proc.\ Soc.\ Photo-Opt.\ Instrum.\ Eng.}
\providecommand{\sjqe}{Sov.\ J.\ Quantum Electron.}
\providecommand{\vr}{Vision Res.}
\AtBeginDocument{\providecommand{\pacs}[1]{\lx@add@frontmatter{ltx:rdf}[property=dcterms:subject]{#1}}%
\providecommand{\preprint}[1]{\lx@add@frontmatter{ltx:rdf}[property=dcterms:identifier]{#1}}\providecommand{\volumeyear}[1]{}%
\providecommand{\volumenumber}[1]{}\providecommand{\issuenumber}[1]{}\providecommand{\eid}[1]{}%
\providecommand{\startpage}[1]{}\def\endpage#1{\gdef\@endpage{#1}}}");
    // :1426-1429, 1474-1495: the compatibility passes natbib `numbers,sort&compress` (a document's own
    // `\usepackage{natbib}` gets them too) and at the end of the preamble loads it, unless the document loaded natbib or
    // biblatex itself, providing REVTeX's `\textcite` and `\onlinecite` on it (1705.03968). Its `plainnat` default for a
    // `\bibliography` with no `\bibliographystyle` only names the `.bst` BibTeX runs.
    pass_options("natbib", "sty", vec!["numbers".to_string(), "sort&compress".to_string()])?;
    DefPrimitive!("\\lx@quantumarticle@natbib", {
      if !lookup_bool("natbib.sty_loaded") && !lookup_bool("biblatex.sty_loaded") {
        RequirePackage!("natbib");
      }
    });
    RawTeX!(r"\AtBeginDocument{\lx@quantumarticle@natbib
\providecommand\textcite[1]{\citet{#1}}\providecommand\onlinecite[1]{\cite{#1}}}");
  }

  // quantumarticle.cls:1392-1394: hyperref, loaded at `\begin{document}` (after the compatibility's natbib).
  DefPrimitive!("\\lx@quantumarticle@hyperref", {
    if !lookup_bool("hyperref.sty_loaded") {
      RequirePackage!("hyperref");
    }
  });
  RawTeX!(r"\AtBeginDocument{\lx@quantumarticle@hyperref}");
  RequirePackage!("bbm");
  // Pre-load with [dvipsnames, table] so user xcolor calls don't
  // silently option-clash. Quantum papers commonly use \cellcolor.
  RequirePackage!("xcolor", options => vec!["dvipsnames".to_string(), "table".to_string()]);

  // Note: This seems similar to revtex4; should we just require revtex4_support ?
  // \author[labels]{name}   One \author per author
  // If label given, the corresponding affiliation from \affil is attached
  // otherwise, \author should be followed by \affiliation  (Perl PR #2767)
  DefMacro!("\\author[]{}",    "\\lx@add@creator[role=author,annotations={#1}]{#2}");
  DefMacro!("\\affiliation{}", "\\lx@add@contact[annotate=new,role=affiliation]{#1}");
  DefMacro!("\\affil OptionalSemiverbatim {}",
    "\\lx@add@contact[role=affiliation,annotate={\\ifx.#1.new\\else 1\\fi},label={#1}]{#2}");
  // \address provides address for previous \author
  DefMacro!("\\address{}", "\\lx@add@contact[annotate=new,role=address]{#1}");
  // These add contacts to most recent author
  // The optional arguments here are a sort of prefix to the footnote (NOT label!)
  // quantumarticle.cls:1220-1225 `\href{mailto:#2}{#2}`, `\href{#2}{#2}`: in the current font (no corpus `_`
  // witness)
  DefMacro!("\\email[] Semiverbatim",    "\\lx@add@contact[role=email,name={#1}]{\\lx@address@textfont{#2}}");
  DefMacro!("\\homepage[] Semiverbatim", "\\lx@add@contact[role=url,name={#1}]{\\lx@address@textfont{#2}}");
  DefMacro!("\\thanks[]{}",              "\\lx@add@contact[role=thanks,name={#1}]{#2}");
  DefMacro!("\\orcid[]{}",               "\\lx@add@orcid[name={#1}]{#2}");

  DefMacro!("\\collaboration{}",    "\\author{#1}");
  DefMacro!("\\altaffiliation[]{}", "\\lx@add@contact[annotate=new,role=affiliation,name={#1}]{#2}");

  RawTeX!("\\DeclareRobustCommand\\openone{\\mathbbm{1}}");
  RawTeX!("\\definecolor{quantumviolet}{HTML}{53257F}");
  RawTeX!("\\definecolor{quantumgray}{HTML}{555555}");
  RawTeX!("\\DeclareRobustCommand{\\Quantum}{Quantum}");

  RawTeX!("\\newenvironment{widetext}{}{}");

  // \onecolumngrid / \twocolumngrid — REVTeX column-switching primitives
  // that quantumarticle.cls L348-349 wraps. Defined via raw cls L4448 in
  // REVTeX but our quantumarticle binding skips that load. No visual
  // effect in HTML/XML; stub as no-op.
  // Witness 2406.00091: `\onecolumngrid \section*{APPENDIX}`.
  def_macro_noop("\\onecolumngrid")?;
  def_macro_noop("\\twocolumngrid")?;
  // quantumarticle.cls L1412-1414: \keywords{x} stores in \@keywords.
  // Render as classification block to preserve the metadata.
  DefMacro!("\\keywords{}",
    "\\lx@add@classification[scheme=keywords]{#1}");
  // some elsearticle style commands?  (Perl PR #2767)
  DefMacro!("\\ead[]{}", "\\email{#1}"); // Strictly if #1=url, should be homepage.

  // quantumarticle is REVTeX-based and inherits the REVTeX
  // \acknowledgments / \acknowledgements bare-command pattern
  // (NO \begin/\end). Driver: 2512.01858 (`\acknowledgments` on its
  // own line with no explicit close, body followed by
  // `\bibliography{...}`). Mirror revtex4_support_sty.rs:79-98 so
  // the bare form opens <ltx:acknowledgements> and the bibliography
  // / end of document auto-closes it. The tolerant \endacknowledgments
  // closes only if the element is still open (covers cases where
  // user did write \begin{acknowledgments}...\end{acknowledgments}
  // and our opener+auto-close already wrapped the body).
  // ltx:acknowledgements Tag (autoClose + inlist=toc) is global — set in
  // latex_constructs.rs (arXiv-fork 23771504 removed the binding-local copies).
  DefConstructor!("\\acknowledgments", "<ltx:acknowledgements name='#name'>",
    properties => { Ok(stored_map!("name" => digest_in_group(T_CS!("\\acknowledgmentsname"))?)) });
  DefConstructor!("\\endacknowledgments", sub[document, _whatsit, _props] {
    let cur = document.get_node().clone();
    let has_open = document.findnode("ancestor-or-self::ltx:acknowledgements", Some(&cur)).is_some();
    if has_open {
      document.close_element("ltx:acknowledgements")?;
    }
  });
  DefMacro!("\\acknowledgmentsname", "Acknowledgements");
  Let!("\\acknowledgements", "\\acknowledgments");
  Let!("\\endacknowledgements", "\\endacknowledgments");
  // quantumarticle.cls:1266-1279: at `\begin{document}` the class loads tikz when installed (`\IfFileExists`, :1266),
  // for its title-page logo; tikz brings graphicx (pgfcore.sty:10), which papers then use without loading it
  // (`\includegraphics` undefined, 2110.03913). The xstring it loads there too (:1270, :933) serves only its own
  // date check and title URL code, which this binding does not run. Perl's binding (quantumarticle.cls.ltxml) loads
  // neither.
  RawTeX!(r"\AtBeginDocument{\IfFileExists{tikz.sty}{\RequirePackage{tikz}\usetikzlibrary{calc}}{}}");
  // quantumarticle.cls:22 `\def\quantumarticleversion{6.1}` (quantum-template).
  DefMacro!("\\quantumarticleversion", "6.1");
});
