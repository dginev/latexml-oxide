use crate::prelude::*;
use latexml_core::common::color::Color;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: pgf.sty.ltxml (54 lines)
  DefMacro!("\\pgfsysdriver", "pgfsys-latexml.def");
  // Pre-announce the driver binding so find_file discovers it.
  // Perl's FindFile finds pgfsys-latexml.def.ltxml on disk; in Rust, the binding
  // exists only in the dispatcher, so we flag it for find_file.
  state::assign_value("pgfsys-latexml.def_binding_available", true, Some(Scope::Global));
  // IMPORTANT: Let pgfutil@IfFileExists BEFORE loading raw pgf.
  // Raw TeX pgfutil-common.tex defines \pgfutil@IfFileExists using \openin (disk only).
  // Perl overrides it with \IfFileExists which uses FindFile (checks bindings too).
  // We must do the same before pgfsys.code.tex tries to find the driver file.
  Let!("\\pgfutil@IfFileExists", "\\IfFileExists");
  InputDefinitions!("pgf", noltxml => true, extension => Some(Cow::Borrowed("sty")));

  // Perl L35-38: pgfsetcolor integration — merge font color from pgfstrokecolor
  Let!("\\pgfsetcolor@orig", "\\pgfsetcolor");
  DefMacro!("\\pgfsetcolor{}", "\\pgfsetcolor@orig{#1}\\lxSVG@set@color");
  DefPrimitive!("\\lxSVG@set@color", {
    // Perl: MergeFont(color => LookupValue('color_pgfstrokecolor'));
    if let Some(Stored::String(color_str)) = state::lookup_value("color_pgfstrokecolor") {
      let cs = arena::to_string(color_str);
      if let Some(color) = Color::from_stored(&cs) {
        MergeFont!(color => color);
      }
    }
  });

  // Perl L41-43: XC@mcolor integration
  RawTeX!("\\ifx\\XC@mcolor\\relax\\let\\XC@mcolor\\@empty\\fi");
  AddToMacro!("\\XC@mcolor", "\\pgfsetcolor{.}");

  // Stub for tikz externalize library: \beginpgfgraphicnamed{name}...\endpgfgraphicnamed
  // In LaTeX, this checks if the graphic should be externalized. We just process inline.
  def_macro_noop("\\beginpgfgraphicnamed{}")?;
  def_macro_noop("\\endpgfgraphicnamed")?;

  // Perl L46-48: wrap pgfpicture/endpgfpicture with lxSVG@picture
  at_begin_document(TokenizeInternal!(
    r"\expandafter\def\expandafter\pgfpicture\expandafter{\expandafter\lxSVG@picture\pgfpicture}\expandafter\def\expandafter\endpgfpicture\expandafter{\endpgfpicture\endlxSVG@picture}"
  ))?;
});
