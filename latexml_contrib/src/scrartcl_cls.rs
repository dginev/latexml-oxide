//! Stub for KOMA-Script `scrartcl` class.
//!
//! Article variant of scrbook (KOMA's typographically refined article).
//! We don't replay KOMA's typographic engine — fall back to OmniBus and
//! stub the most common configuration macros so author preamble doesn't
//! trip undefined-macro errors. Same pattern as scrbook_cls.
use latexml_package::prelude::*;


LoadDefinitions!({
  Warn!(
    "missing_file",
    "scrartcl.cls",
    "scrartcl.cls is only minimally stubbed and will not be interpreted raw."
  );
  LoadClass!("OmniBus");
  // Real scrartcl.cls pulls in the KOMA dependency chain (scrkbase, tocbasic,
  // scrlayer-scrpage, bookmark, typearea, xpatch, scrlogo, auxhook), which
  // transitively loads `iftex` — so `\ifpdf`/`\ifpdftex`/`\ifluatex`/… are
  // defined for author preamble doing engine/driver detection
  // (`\ifpdf \DeclareGraphicsExtensions{.eps,.pdf,…} \else …`). Perl ships no
  // scrartcl binding and raw-loads the .cls, picking up iftex that way (its
  // dependency-scan loads iftex.sty.ltxml). Our OmniBus stub intercepts the
  // class, so without this `\ifpdf` is undefined where Perl is clean. Mirror
  // the real class's dependency. Witness 1802.07175.
  RequirePackage!("iftex");
  // KOMA configuration knobs — layout/typography only, no body content.
  def_macro_noop("\\setkomafont{}{}")?;
  def_macro_noop("\\addtokomafont{}{}")?;
  def_macro_noop("\\setcapindent{}")?;
  def_macro_noop("\\deffootnote[]{}{}{}")?;
  def_macro_noop("\\deffootnotemark{}")?;
  // \KOMAoptions{key=val,...} — runtime option setter, no body content.
  def_macro_noop("\\KOMAoptions{}")?;
  // KOMA page-style commands `\headmark` / `\pagemark`. Defined by
  // KOMA classes (or scrlayer-scrpage) for use in custom page-style
  // declarations as the current chapter/section mark / page number.
  // Since we OmniBus-fall back for scrartcl rather than reading the
  // .cls, these stay undefined and any author preamble using
  // `\usepackage{scrlayer-scrpage}` or scrpage2 page-style hooks
  // triggers `Error:undefined`. Stub as no-ops — running heads/feet
  // are typesetting-only concerns; our HTML output doesn't render
  // them. Witness: 11 papers in R-stages for each.
  def_macro_noop("\\headmark")?;
  def_macro_noop("\\pagemark")?;
  // \subject{}, \dictum{}, \uppertitleback{}, \lowertitleback{},
  // \publishers{} — KOMA frontmatter pieces that DO carry author
  // content. Preserve as ltx:note frontmatter so the text reaches the
  // XML (rather than silently gobbling).
  DefMacro!("\\subject{}",
    "\\@add@frontmatter{ltx:note}[role=subject]{#1}");
  DefMacro!("\\dictum[]{}",
    "\\@add@frontmatter{ltx:note}[role=dictum]{#2}");
  DefMacro!("\\publishers{}",
    "\\@add@frontmatter{ltx:note}[role=publishers]{#1}");
});
