use crate::prelude::*;

// Minimal xcolor.sty — loads color.sty and adds xcolor-specific stubs.
// Full xcolor port (color expressions, model extensions, etc.) is TODO.

LoadDefinitions!({
  // xcolor loads color.sty first
  // Perl: RequirePackage('color');
  RequirePackage!("color");

  // Conditionals (all default to false)
  DefConditional!("\\ifglobalcolors", { false });
  DefConditional!("\\ifdefinecolors", { false });
  DefConditional!("\\ifconvertcolorsD", { false });
  DefConditional!("\\ifconvertcolorsU", { false });
  DefConditional!("\\ifblendcolors", { false });
  DefConditional!("\\ifmaskcolors", { false });
  DefConditional!("\\ifxglobal@", { false });

  // Driver macros
  DefMacro!("\\GetGinDriver", None);
  DefMacro!("\\GinDriver", "LaTeXML");

  // Tracing
  DefRegister!("\\tracingcolors", Number!(0));
  DefMacro!("\\XC@tracing", "0");

  // Color model ranges (xcolor extends color with additional models)
  DefMacro!("\\rangeRGB", "255");
  DefMacro!("\\rangeHsb", "360");
  DefMacro!("\\rangeHSB", "240");
  DefMacro!("\\rangeGray", "15");
  DefMacro!("\\adjustUCRBG", "1,1,1,1");
  DefMacro!("\\paperquality", "1");

  // Color name prefix
  DefMacro!("\\colornameprefix", "XC@");

  // Model selection (stubs)
  DefMacro!("\\selectcolormodel{}", None);
  DefMacro!("\\substitutecolormodel{}{}", None);

  // xcolor's \definecolor delegates to color.sty's version for now
  // Perl: DefMacro('\definecolor[]{}{}{}', '\XC@definecolor[#1]{#2}[\colornameprefix]{#3}{#4}');
  // For now, ignore optional first arg and delegate to color.sty's \definecolor
  // (xcolor's 4-arg form has optional model_list prefix)

  // Color set definitions (stubs)
  DefMacro!("\\definecolorset[]{}{}{}{}", None);
  DefMacro!("\\providecolorset[]{}{}{}{}", None);
  DefMacro!("\\definecolors{}", None);
  DefMacro!("\\providecolors{}", None);

  // \colorlet (stub — just defines a color alias)
  DefMacro!("\\colorlet[]{}{}", None);

  // Blend colors (stub)
  DefMacro!("\\blendcolors{}", None);

  // \rowcolors (table coloring, stub)
  DefMacro!("\\rowcolors[]{}{}{}", None);
  DefMacro!("\\showrowcolors", None);
  DefMacro!("\\hiderowcolors", None);

  // xcolor-specific macros used by other packages
  DefMacro!("\\XC@mcolor", None);

  // \set@color — xcolor version
  // Perl: looks up color_. and applies it
  DefMacro!("\\set@color", None);

  // Ignorable options
  for option in &[
    "monochrome", "natural", "rgb", "cmy", "cmyk", "hsb",
    "gray", "RGB", "HTML", "HSB", "Gray", "wave",
    "luatex", "pdftex", "xetex", "dvips",
    "dvipdfm", "dvipdfmx", "hyperref", "fixpdftex",
    "pst", "nopst", "fixinclude", "hideerrors",
    "showerrors", "kernelfbox", "xcdraw",
    "prologue", "epilogue", "noprologue",
  ] {
    DeclareOption!(option, None);
  }
  // Options that load name definitions
  for option in &["dvipsnames", "dvipsnames*"] {
    DeclareOption!(*option, {
      InputDefinitions!("dvipsnam", extension => Some(Cow::Borrowed("def")));
    });
  }
  DeclareOption!("svgnames", None);
  DeclareOption!("svgnames*", None);
  DeclareOption!("x11names", None);
  DeclareOption!("x11names*", None);
  DeclareOption!("table", None);

  ProcessOptions!();
});
