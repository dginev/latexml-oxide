use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: quantumarticle.cls.ltxml — Quantum Journal
  // See https://github.com/quantum-journal/quantum-journal

  load_class("article", Vec::new(), Tokens!())?;
  ProcessOptions!();

  RequirePackage!("bbm");
  RequirePackage!("inst_support");
  RequirePackage!("xcolor");

  DefConstructor!("\\@@@email{}{}", "^ <ltx:contact role='#2'>#1</ltx:contact>");
  DefMacro!("\\email{}", "\\@add@to@frontmatter{ltx:creator}{\\@@@email{#1}{email}}");
  DefConstructor!("\\@@@address{}", "^ <ltx:contact role='address'>#1</ltx:contact>");
  DefConstructor!("\\@@@affiliation{}", "^ <ltx:contact role='affiliation'>#1</ltx:contact>");
  DefConstructor!("\\@@@homepage{}", "^ <ltx:contact role='homepage'>#1</ltx:contact>");
  DefMacro!("\\address[]{}", "\\@add@to@frontmatter{ltx:creator}{\\@@@address{#2}}");
  DefMacro!("\\affiliation{}", "\\@add@to@frontmatter{ltx:creator}{\\@@@affiliation{#1}}");
  DefMacro!("\\homepage{}", "\\@add@to@frontmatter{ltx:creator}{\\@@@homepage{#1}}");
  DefMacro!("\\orcid{}", "");

  RawTeX!("\\DeclareRobustCommand\\openone{\\mathbbm{1}}");
  RawTeX!("\\definecolor{quantumviolet}{HTML}{53257F}");
  RawTeX!("\\definecolor{quantumgray}{HTML}{555555}");
  RawTeX!("\\DeclareRobustCommand{\\Quantum}{Quantum}");

  RawTeX!("\\newenvironment{acknowledgements}{\\section*{Acknowledgements}}{}");
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
  DefMacro!("\\keywords{}", "");

  DefEnvironment!("{acknowledgments}", "<ltx:acknowledgements>#body</ltx:acknowledgements>");
});
