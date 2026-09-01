//! Stub for aomart.cls (Annals of Mathematics).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  RequirePackage!("amssymb");
  // Eager xcolor preload removed for Perl parity: it makes a later document
  // xcolor[table] load a no-op, so colortbl/array never load and array m{}/b{}
  // columns break (Unrecognized tabular template -> Extra alignment tab). The
  // document loads xcolor itself; color/definecolor stay via hyperref->color.
  // See ifacconf_cls.rs and SYNC_STATUS (eager-xcolor cluster).
  RequirePackage!("hyperref");
  RequirePackage!("fancyhdr");

  // Author metadata (aomart.cls L222+) — preserve as ltx:note
  // frontmatter so author-supplied values reach the XML output.
  // Name parts emit inline.
  DefMacro!("\\givenname{}", "#1");
  DefMacro!("\\surname{}", "#1");
  DefMacro!(
    "\\subject{}{}{}",
    "\\@add@frontmatter{ltx:classification}[scheme=#1]{#3}"
  );
  DefMacro!(
    "\\published{}",
    "\\@add@frontmatter{ltx:note}[role=published]{#1}"
  );
  DefMacro!(
    "\\publishedonline{}",
    "\\@add@frontmatter{ltx:note}[role=published-online]{#1}"
  );
  DefMacro!(
    "\\publicationyear{}",
    "\\@add@frontmatter{ltx:note}[role=year]{#1}"
  );
  DefMacro!(
    "\\volumenumber{}",
    "\\@add@frontmatter{ltx:note}[role=volume]{#1}"
  );
  DefMacro!(
    "\\issuenumber{}",
    "\\@add@frontmatter{ltx:note}[role=issue]{#1}"
  );
  DefMacro!(
    "\\papernumber{}",
    "\\@add@frontmatter{ltx:note}[role=papernumber]{#1}"
  );
  DefMacro!(
    "\\startpage{}",
    "\\@add@frontmatter{ltx:note}[role=startpage]{#1}"
  );
  DefMacro!(
    "\\endpage{}",
    "\\@add@frontmatter{ltx:note}[role=endpage]{#1}"
  );
  DefMacro!(
    "\\doinumber{}",
    "\\@add@frontmatter{ltx:note}[role=doi]{#1}"
  );
  DefMacro!("\\mrnumber{}", "\\@add@frontmatter{ltx:note}[role=mr]{#1}");
  DefMacro!(
    "\\zblnumber{}",
    "\\@add@frontmatter{ltx:note}[role=zbl]{#1}"
  );
  DefMacro!(
    "\\arxivnumber{}",
    "\\@add@frontmatter{ltx:note}[role=arxiv]{#1}"
  );
  DefMacro!(
    "\\version{}",
    "\\@add@frontmatter{ltx:note}[role=version]{#1}"
  );
  DefMacro!(
    "\\copyrightnote{}",
    "\\@add@frontmatter{ltx:note}[role=copyright]{#1}"
  );
  DefMacro!("\\formatdate{}", "#1");

  // Manuscript-review surface (aomart.cls L712-744): outside manuscript
  // mode the real class makes these no-ops or plain passthroughs —
  // `\HSelect`/`\ECSelect` only `\gdef` under `\if@aom@manuscript@mode`,
  // `\Highlight` falls through to its content. aomsample witnesses.
  def_macro_noop("\\HSelect[]{}")?;
  def_macro_noop("\\ECSelect[]{}")?;
  DefMacro!("\\Highlight[]{}", "#2");
  def_macro_noop("\\EditorialComment[]{}")?;
  // Review-metadata gobbles (aomart.cls L430-433 `\let\proposed\@gobble` …).
  def_macro_noop("\\proposed{}")?;
  def_macro_noop("\\seconded{}")?;
  def_macro_noop("\\corresponding{}")?;
  // AMS-style contact fields (aomart.cls L306, L340).
  DefMacro!(
    "\\urladdr[]{}",
    "\\@add@frontmatter{ltx:note}[role=urladdr]{#2}"
  );
  DefMacro!(
    "\\contrib[]{}",
    "\\@add@frontmatter{ltx:note}[role=contributor]{#2}"
  );
  // Cross-ref sugar (aomart.cls L745-750): `\fullref{Theorem}{lbl}` →
  // "Theorem~\ref{lbl}" (hyperref'd in the class; \ref suffices here).
  DefMacro!("\\fullref{}{}", "#1~\\ref{#2}");
  DefMacro!("\\pfullref{}{}", "#1~(\\ref{#2})");
  DefMacro!("\\bfullref{}{}", "#1~[\\ref{#2}]");
  DefMacro!("\\eqfullref{}{}", "#1~\\ref{#2}");
  DefMacro!("\\fullpageref[]{}", "#1~\\pageref{#2}");
  // `\funding[text]{sponsor}{grantid}` (aomart.cls L778): records sponsor
  // metadata; the optional is display text printed in place.
  DefMacro!(
    "\\funding[]{}{}",
    "#1\\@add@frontmatter{ltx:note}[role=funding]{#2 #3}"
  );
});
