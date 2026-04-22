//! pstricks_support.sty — PSTricks drawing support (DVI-only)
//! Perl: pstricks_support.sty.ltxml — 1057 lines
//! Full PSTricks graphics system with coordinate transforms, custom parameter
//! types (PSCoord, PSDimension, PSAngle), and DefPSConstructor meta-definition.
//! DVI-only: all graphics commands produce no output in LaTeXML.
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // PSTricks is DVI-only. The raw pstricks.sty is loaded by pstricks_sty.rs.
  // This support file provides the infrastructure that the raw TeX needs.
  // Since PSTricks graphics are not rendered in LaTeXML (they need a DVI backend),
  // we stub the key macros and environments.

  // Core coordinate/dimension readers — Perl L30-120 (complex Perl closures)
  // Stubbed: ReadPSDimension, ReadPSCoord, ReadPSAngle

  // Transform management — Perl L130-200
  DefMacro!("\\pst@object{}", "#1");
  DefMacro!("\\use@par", "");
  DefMacro!("\\addto@par{}", "");
  DefMacro!("\\psset{}", "");
  DefMacro!("\\psset@special{}", "");

  // Graphics parameters — Perl L200-350
  DefRegister!("\\pslinewidth" => Dimension!("0.8pt"));
  DefRegister!("\\psunit" => Dimension!("1cm"));
  DefRegister!("\\psxunit" => Dimension!("1cm"));
  DefRegister!("\\psyunit" => Dimension!("1cm"));
  DefRegister!("\\pst@dima" => Dimension::new(0));
  DefRegister!("\\pst@dimb" => Dimension::new(0));

  // Core drawing environments — Perl L400-500
  DefEnvironment!("{pspicture}[][]", "#body");
  DefEnvironment!("{pspicture*}[][]", "#body");

  // Line/shape constructors — Perl L500-700
  // All drawing commands are no-ops (DVI-only)
  DefMacro!("\\psline[]", "");
  DefMacro!("\\pspolygon[]", "");
  DefMacro!("\\psframe[]", "");
  DefMacro!("\\pscircle[]", "");
  DefMacro!("\\psellipse[]", "");
  DefMacro!("\\psarc[]", "");
  DefMacro!("\\pswedge[]", "");
  DefMacro!("\\psbezier[]", "");
  DefMacro!("\\pscurve[]", "");
  DefMacro!("\\psecurve[]", "");
  DefMacro!("\\psccurve[]", "");
  DefMacro!("\\parabola[]", "");

  // Placement — Perl L700-900
  DefMacro!("\\rput OptionalMatch:* [][]{}{}",  "#4");
  DefMacro!("\\uput[]{}{}",  "#3");
  DefMacro!("\\multirput[]{}{}{}{}",  "");

  // Grid and axes — Perl L900-1000
  DefMacro!("\\psgrid[]", "");
  DefMacro!("\\psaxes[]", "");

  // Custom object and clip — Perl L1000-1057
  DefMacro!("\\pscustom[]", "");
  DefMacro!("\\psclip[]", "");
  DefMacro!("\\endpsclip", "");

  // Arrow tips
  DefMacro!("\\psoverlay{}", "");
  DefMacro!("\\pst@getangle{}", "");
  DefMacro!("\\pst@number{}", "");
  DefMacro!("\\pst@coor", "");

  // Perl pstricks_support.sty.ltxml L1042-1055: color shorthands. pstricks
  // re-binds these CSes (usually provided by color.sty / xcolor.sty as the
  // named colors) so that `\blue`, `\red`, etc. in figure/node text resolve
  // to a `\color{…}` call. Arxiv 1107.3732 uses `\node[…]{\blue{\small …}}`
  // inside `\tikzpicture`; without these, `\blue` is undefined and errors.
  // Extra length registers — Perl L411-419.
  DefRegister!("\\psframesep" => Dimension!("3pt"));
  DefRegister!("\\pslabelsep" => Dimension!("5pt"));
  DefRegister!("\\psdotsize"  => Dimension!("2pt"));
  DefRegister!("\\psrunit"    => Dimension!("1cm"));

  // Color definition shorthands — Perl L570-573.
  DefMacro!("\\newgray{}{}",      "\\definecolor{#1}{gray}{#2}");
  DefMacro!("\\newrgbcolor{}{}",  "\\definecolor{#1}{rgb}{#2}");
  DefMacro!("\\newhsbcolor{}{}",  "\\definecolor{#1}{hsb}{#2}");
  DefMacro!("\\newcmykcolor{}{}", "\\definecolor{#1}{cmyk}{#2}");

  // Length helpers — Perl L650-651: Let to \setlength / \addtolength.
  Let!("\\pssetlength",   "\\setlength");
  Let!("\\psaddtolength", "\\addtolength");

  // Coordinate-mode no-ops — Perl L1037-1039. No effect in LaTeXML.
  DefMacro!("\\SpecialCoor", "");
  DefMacro!("\\NormalCoor",  "");
  DefMacro!("\\PSTricksOff", "");

  // Rotation constructors — Perl L1002-1006. Produce <ltx:g> wrappers
  // with SVG-style rotate() transforms.
  DefConstructor!(
    "\\rotateleft{}",
    "<ltx:g transform='rotate(90)'>#1</ltx:g>"
  );
  DefConstructor!(
    "\\rotateright{}",
    "<ltx:g transform='rotate(-90)'>#1</ltx:g>"
  );
  DefConstructor!(
    "\\rotatedown{}",
    "<ltx:g transform='rotate(180)'>#1</ltx:g>"
  );

  DefMacro!("\\black", "\\color{black}");
  DefMacro!("\\darkgray", "\\color{darkgray}");
  DefMacro!("\\gray", "\\color{gray}");
  DefMacro!("\\lightgray", "\\color{lightgray}");
  DefMacro!("\\white", "\\color{white}");
  DefMacro!("\\blue", "\\color{blue}");
  DefMacro!("\\red", "\\color{red}");
  DefMacro!("\\green", "\\color{green}");
  DefMacro!("\\yellow", "\\color{yellow}");
  DefMacro!("\\magenta", "\\color{magenta}");
  DefMacro!("\\cyan", "\\color{cyan}");
});
