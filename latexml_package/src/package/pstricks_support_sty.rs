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

  // Transform management — Perl L130-200. `\psset` / `\@@@ackscale` are
  // Perl DefConstructor / DefPrimitive — the Rust stubs are DefMacro
  // drops because the full body needs PSDim*/PSAngle/PSOrigin parameter
  // types that aren't ported (see WISDOM #41 — TeXDelimiter / Pair /
  // PSDim are all structural parameter-type gaps). DP-audit flags both
  // entries; safe until the parameter types land (no \edef site
  // observes these CSes; pstricks use is stomach-time invocation).
  //
  // Intentional divergence (WISDOM #44 class: DVI-only + parameter-type
  // blocker): pstricks is DVI-only — the tracker (_psActiveTransform /
  // ackTransform) lives only in PSTricks post-processing and is not
  // ported to Rust. Both the \psset DefConstructor → DefMacro flip
  // (L28) and the \@@@ackscale DefPrimitive → DefMacro flip (L181) are
  // no-op arg-consumers whose observable behavior under an
  // HTML/MathML backend is identical to a constructor body of "".
  DefMacro!("\\pst@object{}", "#1");
  DefMacro!("\\use@par", "");
  DefMacro!("\\addto@par{}", "");
  DefMacro!("\\psset{}", "");
  DefMacro!("\\psset@special{}", "");

  // Perl pstricks_support.sty.ltxml L580-606: register 22 pstricks keyvals
  // covering dot/arrow sizes, line styling, frame/arc/label spacing, and
  // coordinate units. Perl types PSDimFloat / PSAngle / PSDimension /
  // PSDimDim / PSOrigin / PSRegisterDimension / Float aren't registered
  // Rust types; register with the untyped placeholder ("") since this
  // binding is DVI-only and no consumer coerces these values. Author code
  // that tests `\@ifundefined{KV@pstricks@dotsize@default}` now sees the
  // Perl-equivalent answer.
  for key in ["dotsize", "tbarsize", "dotangle",
              "arrowsize", "arrowlength", "arrowinset",
              "dotsep", "dash", "linewidth", "linearc", "framearc",
              "origin", "framesep", "labelsep", "doublesep",
              "arcsep", "arcsepA", "arcsepB",
              "unit", "xunit", "yunit", "runit"] {
    DefKeyVal!("pstricks", key, "");
  }

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
  // Perl pstricks_support.sty.ltxml L996:
  //   DefEnvironment('{psclip} {}',
  //     '<ltx:clip> <ltx:clippath> #1 </ltx:clippath> #body </ltx:clip>');
  // Prior Rust stubbed both \psclip and \endpsclip to empty DefMacro,
  // losing the <ltx:clip>/<ltx:clippath> wrapping structure entirely.
  // Port as DefEnvironment matching Perl's signature and template.
  // (ltx:clip + ltx:clippath are declared in picture.rnc schema.)
  DefEnvironment!("{psclip}{}",
    "<ltx:clip><ltx:clippath>#1</ltx:clippath>#body</ltx:clip>");

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

  // Angle units — Perl L653-654. Sets `\degrees` state to 360 (default)
  // or to \degrees{angle}-provided value; \radians flips to 2π. The
  // angle value is stored via AssignValue and consulted by pstricks's
  // coord readers. Taken as-is from Perl.
  // `[Float]` would map to a non-typed Tokens under the current prelude;
  // Perl uses the raw string value as-is, so keep the argument as
  // Optional tokens and parse at use time.
  DefPrimitive!("\\degrees []", sub[(angle)] {
    let v = angle
      .as_ref()
      .and_then(|t| t.to_string().trim().parse::<f64>().ok())
      .unwrap_or(360.0);
    AssignValue!("\\degrees" => Stored::from(v), None);
  });
  DefPrimitive!("\\radians", {
    AssignValue!("\\degrees" => Stored::from(std::f64::consts::TAU), None);
  });

  // Coordinate-mode no-ops — Perl L1037-1039. No effect in LaTeXML.
  DefMacro!("\\SpecialCoor", "");
  DefMacro!("\\NormalCoor",  "");
  DefMacro!("\\PSTricksOff", "");

  // Rotation constructors — Perl pstricks_support.sty.ltxml L1002-1006.
  // Produce <ltx:g> SVG-rotate wrappers. `bounded => 1` matches Perl —
  // restricts the body's group scope so subsequent text outside the
  // rotate doesn't inherit the wrapper's font/color side effects.
  // (Perl additionally calls `ackTransform('rotate(...)')` in
  // beforeDigest to compose the rotation onto the
  // `_psActiveTransform` state used downstream by ps-coordinate math
  // — that piece needs the Transform/_psActiveTransform infrastructure
  // which doesn't yet exist on the Rust side; deferred. See
  // pstricks_support.sty.ltxml `sub ackTransform`.)
  DefConstructor!(
    "\\rotateleft{}",
    "<ltx:g transform='rotate(90)'>#1</ltx:g>",
    bounded => true
  );
  DefConstructor!(
    "\\rotateright{}",
    "<ltx:g transform='rotate(-90)'>#1</ltx:g>",
    bounded => true
  );
  DefConstructor!(
    "\\rotatedown{}",
    "<ltx:g transform='rotate(180)'>#1</ltx:g>",
    bounded => true
  );

  // Perl pstricks_support.sty.ltxml L1009-1012: \@@@ackscale and the
  // \scalebox / \@@scalebox / \@@@scalebox trio that threads a scale
  // transform through the PSTricks _psActiveTransform tracker. The
  // tracker (ackTransform) isn't ported to Rust — it lives only in
  // PSTricks post-processing which Rust doesn't implement. Ship
  // \@@@ackscale as a no-op consume-the-arg stub so documents invoking
  // `\scalebox{0.5}{body}` via PSTricks don't hit undefined-CS.
  // `\scalebox` itself is provided by graphics_sty (standard LaTeX form);
  // not overriding it here keeps the tested scalebox golden intact.
  // Intentional DefPrimitive → DefMacro (WISDOM #44, same DVI-only
  // blocker as L18-31 umbrella).
  DefMacro!("\\@@@ackscale{}", "");

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

  // Rotation wrappers — Perl pstricks_support.sty.ltxml L1002-1008.
  // Perl runs `ackTransform(...)` in before_digest to track the
  // transform for PSTricks post-processing (which Rust doesn't do —
  // pstricks is DVI-only here, documented at file-top WISDOM #44
  // class). Drop the before_digest and emit the <ltx:g> element
  // directly; visual result is identical under HTML/MathML backend.
  DefConstructor!("\\rotateleft{}",  "<ltx:g transform='rotate(90)'>#1</ltx:g>",
    bounded => true);
  DefConstructor!("\\rotateright{}", "<ltx:g transform='rotate(-90)'>#1</ltx:g>",
    bounded => true);
  DefConstructor!("\\rotatedown{}",  "<ltx:g transform='rotate(180)'>#1</ltx:g>",
    bounded => true);
  // Note: `\scalebox` is left to graphics.sty's fully-featured
  // DefConstructor — pstricks's Perl override only swaps in its own
  // post-processing tracker (`\@@scalebox` → `\@@@scalebox`) which is
  // the DVI-only path we already omit above via the `\@@@ackscale`
  // no-op stub.
});
