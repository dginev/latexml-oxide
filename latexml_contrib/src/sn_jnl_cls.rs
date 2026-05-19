//! Stub for sn-jnl.cls (Springer Nature journal class).
use latexml_package::prelude::*;


LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  RequirePackage!("amssymb");
  RequirePackage!("xcolor");
  RequirePackage!("hyperref");
  RequirePackage!("graphicx");
  // Real sn-jnl.cls loads geometry for page setup — papers commonly
  // call \\geometry{margin=2cm} without an explicit usepackage.
  // Witness 2503.06846.
  RequirePackage!("geometry");

  // sn-jnl frontmatter — gobble layout-only / preserve author text.
  DefMacro!("\\bmhead{}", "\\subsubsection*{#1}");
  DefMacro!("\\bmsection{}", "\\section*{#1}");
  // \sectiontitle{text} carries an author-typed section title used in
  // sn-jnl's TOC/running-head pipeline. Preserve as ltx:note rather
  // than silently dropping the words. Content-preserving.
  DefMacro!("\\sectiontitle{}",
    "\\@add@frontmatter{ltx:note}[role=sectiontitle]{#1}");
  // \headtype{...} / \extralength{...} are layout knobs (no author body).
  def_macro_noop("\\headtype{}")?;
  def_macro_noop("\\extralength{}")?;
  // \theHfigure / \theHtable are hyperref H-counter overrides (no body).
  def_macro_noop("\\theHfigure{}")?;
  def_macro_noop("\\theHtable{}")?;

  // Author-block — preserve author-supplied affiliation / equalcont /
  // presentaddress content as ltx:note frontmatter.
  //
  // sn-jnl.cls defines `\author` to dispatch on `\@ifstar` to either
  // `\@@corrauthor` (corresponding author) or `\@@author` (regular).
  // We collapse both to the core `\author{#name}` semantics — we don't
  // distinguish corresponding from regular in the output.
  //
  // CAVEAT (root-cause for 2306.11901): our DefMacro prototype parser
  // treats the `*` immediately after `\author` as a Token parameter
  // (literal star), NOT as a CS suffix. So a naive
  //   `DefMacro!("\\author*[]{}", "\\author{#2}")`
  // overrides `\author` to *require* a literal star and then the body
  // `\author{#2}` calls itself with the empty optional `#2`, recursing
  // forever on `\author{X}` (which has no star and no [opt]). Use
  // `OptionalMatch:*` so the star is optional, save the core `\author`
  // first, and forward to it with the full `[opt]{name}` shape so the
  // body never re-enters this stub.
  state::let_i(&T_CS!("\\lx@sn@core@author"), &T_CS!("\\author"), None);
  DefMacro!("\\author OptionalMatch:* []{}",
    "\\lx@sn@core@author[#2]{#3}");
  DefMacro!("\\affil OptionalMatch:* []{}",
    "\\@add@frontmatter{ltx:note}[role=affiliation]{#3}");
  DefMacro!("\\equalcont{}",
    "\\@add@frontmatter{ltx:note}[role=equal-contributors]{#1}");
  DefMacro!("\\presentaddress{}",
    "\\@add@frontmatter{ltx:note}[role=present-address]{#1}");
  // Name part helpers (first-name, surname) — emit inline.
  DefMacro!("\\fnm{}", "#1");
  DefMacro!("\\sur{}", "#1");

  // Frontmatter envs — internal_vertical mode for multi-paragraph
  // bodies (declarations especially carries author prose with \par
  // separators). Without explicit mode, restricted_horizontal default
  // trips Endgroup mismatch on \par-containing bodies.
  DefEnvironment!("{abstract}", "<ltx:abstract>#body</ltx:abstract>",
    mode => "internal_vertical");
  DefEnvironment!("{declarations}", "<ltx:acknowledgements name='declarations'>#body</ltx:acknowledgements>",
    mode => "internal_vertical");
  DefEnvironment!("{appendices}", "<ltx:appendix>#body</ltx:appendix>",
    mode => "internal_vertical");
});
