//! french.ldf / frenchb.ldf — French language support for babel
//! Perl: french.ldf.ltxml + frenchb.ldf.ltxml (~35 lines each)
//!
//! Provides: French superscript commands, ordinals, guillemets,
//! degree symbol, number formatting delegation.
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: InputDefinitions('french', type => 'ldf', noltxml => 1)
  // We skip raw loading (it fails on babel 3.x \SetString commands)
  // and provide the essential definitions directly.

  // French superscript (Perl french.ldf.ltxml L24-26)
  DefConstructor!("\\up{}", "<ltx:sup>#1</ltx:sup>", enter_horizontal => true);
  DefConstructor!("\\fup{}", "<ltx:sup>#1</ltx:sup>", enter_horizontal => true);
  DefConstructor!("\\FB@up@fake{}", "<ltx:sup>#1</ltx:sup>", enter_horizontal => true);

  // Ordinal suffixes (from raw frenchb.ldf)
  DefMacro!("\\ier", "\\up{er}");
  DefMacro!("\\iers", "\\up{ers}");
  DefMacro!("\\iere", "\\up{re}");
  DefMacro!("\\ieres", "\\up{res}");
  DefMacro!("\\ieme", "\\up{e}");
  DefMacro!("\\iemes", "\\up{es}");

  // French enumeration (from raw frenchb.ldf)
  DefMacro!("\\FrenchEnumerate{}", "#1\\up{o}");
  DefMacro!("\\FrenchPopularEnumerate{}", "#1\\up{o})");
  DefMacro!("\\primo", "1\\up{o}");
  DefMacro!("\\secundo", "2\\up{o}");
  DefMacro!("\\tertio", "3\\up{o}");
  DefMacro!("\\quarto", "4\\up{o}");
  DefMacro!("\\fprimo)", "1\\up{o})");
  DefMacro!("\\fsecundo)", "2\\up{o})");
  DefMacro!("\\ftertio)", "3\\up{o})");
  DefMacro!("\\fquarto)", "4\\up{o})");

  // \No, \no, \Nos, \nos — French abbreviations for "Numéro"
  DefMacro!("\\No", "N\\up{o}");
  DefMacro!("\\no", "n\\up{o}");
  DefMacro!("\\Nos", "N\\up{os}");
  DefMacro!("\\nos", "n\\up{os}");

  // \bsc — small caps (from raw frenchb.ldf)
  DefMacro!("\\bsc{}", "{\\scshape #1}");

  // French quotes: \og and \fg (guillemets)
  DefMacro!("\\og", "\u{00AB}\u{00A0}");
  DefMacro!("\\fg", "\u{00A0}\u{00BB}");

  // Symbols (Perl french.ldf.ltxml L32-35, AtBeginDocument)
  DefMacro!("\\degre", "\\textdegree ");
  DefMacro!("\\degres", "\\hbox to 0.3em{\\degre}");
  DefMacro!("\\tild", "\\textasciitilde ");
  DefMacro!("\\circonflexe", "\\textasciicircum ");
  DefMacro!("\\at", "@");
  DefMacro!("\\boi", "\\textbackslash ");

  // \nombre — delegates to numprint if loaded (Perl french.ldf.ltxml L30)
  DefMacro!("\\nombre{}", "\\numprint{#1}");

  Let!("\\xspace", "\\relax");
});
