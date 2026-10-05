//! Stub for bmvc2k.cls (BMVC British Machine Vision Conference).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  // bmvc2k.cls:149 loads xcolor (`\textcolor` was undefined, 2609.06007). A paper's later `\usepackage[table]{xcolor}`
  // still takes its options: a repeat load applies the options the first lacked (content.rs
  // `apply_new_options_on_reload`, witness 2605.00310).
  RequirePackage!("xcolor");
  RequirePackage!("hyperref");
  RequirePackage!("graphicx");

  // bmvc2k frontmatter (L167+) — preserve author content.
  def_macro_noop("\\bmvaOneDot")?;
  DefMacro!("\\bmvaHangBox{}", "#1");
  // bmvc2k.cls:254 sets T1, whose slot 95 is the underscore in every family.
  RequirePackage!("fontenc", options => vec![s!("T1")]);
  // \addauthor{name}{email}{institution-id}: the name is the author, and the mail/homepage is set as a URL in sans
  // (`\DeclareUrlCommand\bmvaUrl{\urlstyle{sf}}`, bmvc2k.cls:161, 376) — read verbatim, its `_`s are not math
  // (2609.06007).
  DefMacro!(
    "\\addauthor{} Semiverbatim {}",
    "\\author{#1}\\lx@add@email{\\sffamily #2}"
  );
  DefMacro!(
    "\\addinstitution{}",
    "\\@add@frontmatter{ltx:note}[role=affiliation]{#1}"
  );
});
