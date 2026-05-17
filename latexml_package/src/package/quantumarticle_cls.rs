use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: quantumarticle.cls.ltxml — Quantum Journal
  // See https://github.com/quantum-journal/quantum-journal

  load_class("article", Vec::new(), Tokens!())?;
  ProcessOptions!();

  RequirePackage!("bbm");
  RequirePackage!("inst_support");
  // Pre-load with [dvipsnames, table] so user xcolor calls don't
  // silently option-clash. Quantum papers commonly use \cellcolor.
  RequirePackage!("xcolor", options => vec!["dvipsnames".to_string(), "table".to_string()]);

  DefConstructor!("\\@@@email{}{}", "^ <ltx:contact role='#2'>#1</ltx:contact>");
  DefMacro!("\\email{}", "\\@add@to@frontmatter{ltx:creator}{\\@@@email{#1}{email}}");
  DefConstructor!("\\@@@address{}", "^ <ltx:contact role='address'>#1</ltx:contact>");
  DefConstructor!("\\@@@affiliation{}", "^ <ltx:contact role='affiliation'>#1</ltx:contact>");
  DefConstructor!("\\@@@homepage{}", "^ <ltx:contact role='homepage'>#1</ltx:contact>");
  DefMacro!("\\address[]{}", "\\@add@to@frontmatter{ltx:creator}{\\@@@address{#2}}");
  DefMacro!("\\affiliation{}", "\\@add@to@frontmatter{ltx:creator}{\\@@@affiliation{#1}}");
  DefMacro!("\\homepage{}", "\\@add@to@frontmatter{ltx:creator}{\\@@@homepage{#1}}");
  // Preserve ORCID as ltx:note frontmatter (content: the iD string
  // identifies the author and downstream JATS/HTML may surface it).
  DefMacro!("\\orcid{}",
    "\\@add@frontmatter{ltx:note}[role=orcid]{#1}");

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
  DefMacro!("\\onecolumngrid", "");
  DefMacro!("\\twocolumngrid", "");
  // quantumarticle.cls L1412-1414: \keywords{x} stores in \@keywords.
  // Render as classification block to preserve the metadata.
  DefMacro!("\\keywords{}",
    "\\@add@frontmatter{ltx:classification}[scheme=keywords]{#1}");
  // \ead is the elsart-style email-address macro often inherited by
  // quantumarticle users from journal templates. Preserve as note.
  // Witness 2406.10832.
  DefMacro!("\\ead[]{}",
    "\\@add@frontmatter{ltx:note}[role=email]{#2}");

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
    properties => { Ok(stored_map!("name" => stomach::digest(T_CS!("\\acknowledgmentsname"))?)) });
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
