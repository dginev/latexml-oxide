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
  // KOMA configuration knobs — layout/typography only, no body content.
  def_macro_noop("\\setkomafont{}{}")?;
  def_macro_noop("\\addtokomafont{}{}")?;
  def_macro_noop("\\setcapindent{}")?;
  def_macro_noop("\\deffootnote[]{}{}{}")?;
  def_macro_noop("\\deffootnotemark{}")?;
  // \KOMAoptions{key=val,...} — runtime option setter, no body content.
  def_macro_noop("\\KOMAoptions{}")?;
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
