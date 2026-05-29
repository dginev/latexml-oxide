//! Stub for sn-jnl.cls (Springer Nature journal class).
use latexml_package::prelude::*;


LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  RequirePackage!("amssymb");
  // Do NOT eager-load xcolor (Perl ships no sn-jnl binding → OmniBus, no
  // preload). A preloaded xcolor makes a later `\usepackage[table]{xcolor}`
  // a no-op → colortbl/array never load → array `m{}`/`b{}` columns are
  // "Unrecognized tabular template" → "Extra alignment tab". The document
  // loads xcolor with its own options; `\color`/`\definecolor` stay
  // available via hyperref→color. See ifacconf_cls.rs / SYNC_STATUS.
  RequirePackage!("hyperref");
  RequirePackage!("graphicx");
  // Real sn-jnl.cls loads geometry for page setup — papers commonly
  // call \\geometry{margin=2cm} without an explicit usepackage.
  // Witness 2503.06846.
  RequirePackage!("geometry");
  // sn-jnl.cls L615-618 raw-loads algorithm + algorithmicx +
  // algpseudocode so papers can use `\begin{algorithm}` and
  // `\begin{algorithmic}` (along with `\State`/`\If`/`\For`/`\While`)
  // without an explicit `\usepackage{...}`. Witness 2201.08889.
  RequirePackage!("algorithm");
  RequirePackage!("algorithmicx");
  RequirePackage!("algpseudocode");

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
  // sn-jnl.cls L599-606: \orgdiv / \orgname / \orgaddress / \street /
  // \postcode / \city / \state / \country — affiliation-element helpers
  // that pass through their argument as inline text. The paper-bundled
  // class file defines all of them as `\newcommand{\foo}[1]{#1}` (plain
  // pass-through). Our raw-load path doesn't always invoke them, so
  // bind explicitly. Without these stubs, papers using the standard
  // sn-jnl `\affil*[1]{\orgdiv{...}, \orgname{...}, \orgaddress{...}}`
  // pattern report undefined CS cascade. Witness 2311.09249, 2311.08387.
  DefMacro!("\\orgdiv{}",     "#1");
  DefMacro!("\\orgname{}",    "#1");
  DefMacro!("\\orgaddress{}", "#1");
  DefMacro!("\\street{}",     "#1");
  DefMacro!("\\postcode{}",   "#1");
  DefMacro!("\\city{}",       "#1");
  DefMacro!("\\state{}",      "#1");
  DefMacro!("\\country{}",    "#1");
  // sn-jnl.cls defines \botrule as a bottom-rule table separator
  // (similar shape to \toprule / \midrule from booktabs). Authors use
  // it inside \begin{tabular}...\end{tabular} for Springer-Nature
  // bottom rules. Map to \hline so the table still renders.
  // Witness 2402.17342.
  Let!("\\botrule", "\\hline");

  // Frontmatter envs — internal_vertical mode for multi-paragraph
  // bodies (declarations especially carries author prose with \par
  // separators). Without explicit mode, restricted_horizontal default
  // trips Endgroup mismatch on \par-containing bodies.
  DefEnvironment!("{abstract}", "<ltx:abstract>#body</ltx:abstract>",
    mode => "internal_vertical");
  // Real sn-jnl.cls defines `\abstract{...}` as a *macro* form (not the
  // standard env). Without this stub, our environment binding's
  // auto-created `\abstract` token (which expects a matching
  // `\endabstract`) eats every subsequent `\title`/`\author`/`\maketitle`
  // /`\section` into the still-open `<ltx:abstract>`, producing a
  // cascade of `Error:malformed:ltx:* isn't allowed in <ltx:abstract>`.
  // Forward to the env so the body is wrapped *and* properly closed.
  // Witness 2306.11901.
  DefMacro!("\\abstract{}",
    "\\begin{abstract}#1\\end{abstract}");
  DefEnvironment!("{declarations}", "<ltx:acknowledgements name='declarations'>#body</ltx:acknowledgements>",
    mode => "internal_vertical");
  DefEnvironment!("{appendices}", "<ltx:appendix>#body</ltx:appendix>",
    mode => "internal_vertical");
});
