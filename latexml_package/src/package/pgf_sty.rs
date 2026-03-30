use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: pgf.sty.ltxml (54 lines)
  DefMacro!("\\pgfsysdriver", "pgfsys-latexml.def");
  InputDefinitions!("pgf", noltxml => true, extension => Some(Cow::Borrowed("sty")));
  Let!("\\pgfutil@IfFileExists", "\\IfFileExists");

  // Perl L35-38: pgfsetcolor integration
  Let!("\\pgfsetcolor@orig", "\\pgfsetcolor");
  DefMacro!("\\pgfsetcolor{}", "\\pgfsetcolor@orig{#1}\\lxSVG@set@color");
  DefPrimitive!("\\lxSVG@set@color", {
    // Perl: MergeFont(color => LookupValue('color_pgfstrokecolor'));
    // TODO: integrate with font color system when needed
  });

  // Perl L41-43: XC@mcolor integration
  RawTeX!("\\ifx\\XC@mcolor\\relax\\let\\XC@mcolor\\@empty\\fi");
  AddToMacro!("\\XC@mcolor", "\\pgfsetcolor{.}");

  // Perl L46-48: wrap pgfpicture/endpgfpicture with lxSVG@picture
  RawTeX!("\\AtBeginDocument{\\expandafter\\def\\expandafter\\pgfpicture\\expandafter{\\expandafter\\lxSVG@picture\\pgfpicture}\\expandafter\\def\\expandafter\\endpgfpicture\\expandafter{\\endpgfpicture\\endlxSVG@picture}}");
});
