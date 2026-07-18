//! Binding for fairmeta.cls (FAIR / Meta pre-print class).
//!
//! fairmeta.cls builds its frontmatter interface
//! (\author/\affiliation/\contribution/\metadata/\correspondence/\abstract) on
//! an \addtolist[5] accumulator in the class BODY. Since an unknown .cls body is
//! NOT raw-loaded (OmniBus extracts dependencies only), every one of those
//! commands is Error:undefined without a binding — and the class's
//! \RequirePackage{nicematrix} etc. never take effect either. Route the
//! user-facing frontmatter through \@add@frontmatter so title/authors/
//! affiliations/metadata/abstract reach the XML, and pull in the real
//! dependency packages the class relies on.
//!
//! Witnesses: 2412.06264 (ar5iv #520), 2509.24704 (#567), 2511.16624 (#576).
//! (selfevolagent.cls, 2508.07407/#556, is a near-identical sibling class.)
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  // Dependency packages the class \RequirePackage's and the paper body relies
  // on (those with bindings / user-facing commands). Purely-cosmetic layout
  // packages (geometry, microtype, setspace, parskip, titlesec, …) are omitted
  // — they are visual no-ops in LaTeXML and OmniBus degrades gracefully.
  RequirePackage!("amsmath");
  RequirePackage!("amssymb");
  RequirePackage!("graphicx");
  RequirePackage!("xcolor");
  RequirePackage!("booktabs");
  RequirePackage!("multirow");
  RequirePackage!("bm");
  RequirePackage!("etoolbox");
  RequirePackage!("caption");
  RequirePackage!("hyperref");
  RequirePackage!("cleveref");
  RequirePackage!("natbib");
  RequirePackage!("nicematrix");
  // Faithful to the class's `\RequirePackage[most]{tcolorbox}` (fairmeta.cls
  // L42). PassOptions BEFORE the require (Perl idiom, mirrors ar5iv.sty.ltxml)
  // so tcolorbox.sty's own \ProcessOptions sees `most` at raw-load time and
  // \tcbuselibrary{most} loads the enhanced/breakable/skins keys.
  pass_options("tcolorbox", "sty", vec![s!("most")])?;
  RequirePackage!("tcolorbox");

  // \geometry{...} — page-geometry hint; visual-only, gobble the arg.
  def_macro_noop("\\geometry{}")?;

  // Class palette (\color{metafg} is used by the abstract box).
  Digest!("\\definecolor{metablue}{HTML}{E2EFEF}")?;
  Digest!("\\definecolor{metafg}{HTML}{1C2B33}")?;
  Digest!("\\definecolor{metabg}{HTML}{EFF6F6}")?;

  // Frontmatter. Each fairmeta helper takes an optional leading mark
  // (\author[1]{...}, \metadata[Label]{...}); route the content to the shared
  // \@add@frontmatter sink so it lands in <ltx:document> frontmatter. The
  // accumulator lists (\authorlist, …) become no-ops — the sink accumulates.
  def_macro_noop("\\authorlist")?;
  def_macro_noop("\\affiliationlist")?;
  def_macro_noop("\\contributionlist")?;
  def_macro_noop("\\metadatalist")?;

  // \author[mark]{name} — accumulate as a document creator. Route to the
  // internal \lx@add@author sink (NOT \author — that would re-match this macro
  // and recurse); the leading affiliation mark #1 is dropped.
  DefMacro!("\\author[]{}", "\\lx@add@author{#2}");
  // \affiliation[mark]{text}
  DefMacro!(
    "\\affiliation[]{}",
    "\\@add@frontmatter{ltx:note}[role=affiliation]{#2}"
  );
  // \contribution[mark]{text}
  DefMacro!(
    "\\contribution[]{}",
    "\\@add@frontmatter{ltx:note}[role=contribution]{#2}"
  );
  // \metadata[label]{value} — the label becomes the note role.
  DefMacro!(
    "\\metadata[]{}",
    "\\@add@frontmatter{ltx:note}[role=#1]{#2}"
  );
  // \correspondence{text} (class: \metadata[Correspondence]{...}).
  DefMacro!(
    "\\correspondence{}",
    "\\@add@frontmatter{ltx:note}[role=correspondence]{#1}"
  );
  // \date{text} (class: \metadata[Date]{...}; the \faCalendar icon is dropped).
  DefMacro!("\\date{}", "\\@add@frontmatter{ltx:note}[role=date]{#1}");
  // \abstract{text} (class stores it in \abstractlist; feed the abstract sink).
  DefMacro!("\\abstract{}", "\\lx@add@abstract{#1}");
  // \email{addr} — kept verbatim from the class.
  DefMacro!("\\email{}", "\\href{mailto:#1}{\\texttt{#1}}");

  // \beginappendix — the class's stand-in for \appendix.
  DefMacro!("\\beginappendix", "\\appendix");
  // \nm{...} — no-op text wrapper (\newcommand{\nm}[1]{#1}).
  DefMacro!("\\nm{}", "#1");
});
