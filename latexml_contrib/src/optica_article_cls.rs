//! Stub for optica-article.cls (Optica/OSA journal class).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  // No eager plain xcolor (Perl parity: a later document xcolor[table] would be a no-op, so colortbl/array never load;
  // ifacconf_cls.rs, SYNC_STATUS eager-xcolor cluster): the class's own `xcolor[table]` (:32) comes from the scan below.
  RequirePackage!("hyperref");
  // optica-article.cls:25-282 loads what its papers' tables, symbols and figures need — array's `>{$}l<{$}` columns,
  // tabularx, multirow, newtxmath's `\gtrsim` (2609.00899, 2609.05706, 2609.06191, 2609.10235): the class file's own
  // dependency scan, as sn_jnl_cls.rs does, without the base class and soul, which is the `\else` arm of `\ifpdf`
  // (:46-54) pdflatex never takes — its `\hl`/`\st` would replace a paper's own. (`styles/#1`, :59, sits in a macro body
  // the scan skips anyway.)
  require_dependencies_except("optica-article", "cls", &["article", "soul", "styles/#1"]);

  // Optica-specific frontmatter / formatting.
  DefMacro!("\\authormark{}", "\\textsuperscript{#1}");
  DefMacro!(
    "\\bmsection{}",
    "\\par\\medskip\\noindent\\textbf{#1.}\\enspace"
  );
  DefMacro!("\\JournalTitle{}", "\\emph{#1}");
  // Bibliographic metadata — preserve author values.
  DefMacro!("\\Year{}", "\\@add@frontmatter{ltx:note}[role=year]{#1}");
  DefMacro!("\\Month{}", "\\@add@frontmatter{ltx:note}[role=month]{#1}");
  DefMacro!(
    "\\Volume{}",
    "\\@add@frontmatter{ltx:note}[role=volume]{#1}"
  );
  DefMacro!("\\Page{}", "\\@add@frontmatter{ltx:note}[role=page]{#1}");

  // {abstract*} environment.
  DefEnvironment!("{abstract*}", "<ltx:abstract>#body</ltx:abstract>");
});
