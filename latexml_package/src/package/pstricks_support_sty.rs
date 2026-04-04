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
});
