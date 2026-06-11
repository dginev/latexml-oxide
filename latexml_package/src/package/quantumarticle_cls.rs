use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: quantumarticle.cls.ltxml — Quantum Journal
  // See https://github.com/quantum-journal/quantum-journal

  load_class("article", Vec::new(), Tokens!())?;
  ProcessOptions!();

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
  DefMacro!("\\email[] Semiverbatim",    "\\lx@add@contact[role=email,name={#1}]{#2}");
  DefMacro!("\\homepage[] Semiverbatim", "\\lx@add@contact[role=url,name={#1}]{#2}");
  DefMacro!("\\thanks[]{}",              "\\lx@add@contact[role=thanks,name={#1}]{#2}");
  DefMacro!("\\orcid[]{}",               "\\lx@add@contact[role=orcid,name={#1}]{#2}");

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
  Tag!("ltx:acknowledgements", auto_close => true);
  DefConstructor!("\\acknowledgments", "<ltx:acknowledgements name='#name'>",
    properties => { Ok(stored_map!("name" => digest(T_CS!("\\acknowledgmentsname"))?)) });
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
});
