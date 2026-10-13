//! Stub for spie.cls (SPIE conference proceedings).
use latexml_core::definition::origin::{DefinitionOrigin, OriginGuard};
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  // Pre-load xcolor with [dvipsnames, table] so a paper's later
  // `\usepackage[table]{xcolor}` doesn't silently option-clash and
  // leave colortbl unloaded → `\cellcolor` undefined. spie.cls itself
  // does NOT load xcolor (only `\LoadClassWithOptions{article}`), so
  // in Perl the user's `[table]{xcolor}` is the first load and colortbl
  // comes in via the `table` option; matching that outcome here. Same
  // anti-clash pattern as svproc_cls / mnras_cls / quantumarticle_cls.
  // Witness: 1807.04749 ("undefined:\\cellcolor").
  RequirePackage!("xcolor", options => vec!["dvipsnames".to_string(), "table".to_string()]);
  RequirePackage!("hyperref");
  // spie.cls L92: `\RequirePackage[superscript]{cite}[2003/11/04]` — SPIE uses
  // the cite package for superscript numeric citations. cite.sty defines the
  // no-brackets `\citen`/`\citenum`/`\citeonline` aliases SPIE templates use
  // directly. Our OmniBus base autoloads natbib's `\citenum` but NOT cite's
  // `\citen`, so without this `\citen` is undefined where Perl (raw-loads
  // spie.cls → cite) is clean. Faithful (spie.cls genuinely loads cite — not an
  // eager preload). Witness 1808.10428 (`{spie}` + `\citen`).
  RequirePackage!("cite", options => vec!["superscript".to_string()]);

  // spie.cls L107: \authorinfo{...} for author footnote — preserve.
  DefMacro!(
    "\\authorinfo{}",
    "\\@add@frontmatter{ltx:note}[role=authorinfo]{#1}"
  );
  def_macro_noop("\\skiplinehalf")?;
  DefMacro!("\\supit{}", "\\textsuperscript{#1}");
  // spieman.cls, SPIE's journal manuscript class, which the loader gives this binding ("Can't find binding for class
  // spieman (using spie)"; 87 of 99 sampled copies are its v1.10, 1703.03292's): `\linkable` (:82), an address or URL
  // set blue and underlined but not linked (run 340: 640 first errors; 2604.17554, 2604.05004); the 2006 v1.00 copies'
  // `\supscr`/`\supscrsm`, raised small text (1408.0407). spie.cls defines neither, so they are guesses a paper's own
  // definition replaces (`DefinitionOrigin::Fallback`).
  {
    let _guess = OriginGuard::new(DefinitionOrigin::Fallback);
    RawTeX!(
      r"\newcommand{\linkable}[1]{\textcolor{blue}{\underline{#1}}}
\def\supscr#1{\raisebox{0.8ex}{\small #1}\hspace{0.05em}}
\def\supscrsm#1{\raisebox{0.8ex}{\footnotesize #1}\hspace{0.05em}}"
    );
  }
});
