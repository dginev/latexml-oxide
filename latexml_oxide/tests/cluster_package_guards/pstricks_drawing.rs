//! pstricks drawing objects (Perl pstricks_support.sty.ltxml:661-843): each
//! `\psline`/`\psframe`/`\pscircle`/… is a `DefPSConstructor` that draws an
//! `ltx:line`/`ltx:rect`/`ltx:circle`/… into the `{pspicture}`, with the line
//! and fill parameters the raw `\psset` keeps. They were gobbled as no-ops
//! ("DVI-only"), leaving every picture empty (Gemini round 12 P1).
use latexml::util::test::{assert_element, rng_error_count};

use super::perfect_kernel_batch46::{convert, error_count, warning_count};

/// The P1 repro: an arrowed line, a frame, a circle with a local
/// `linewidth`, a polygon, a dot and a line with a bare-number `linewidth`.
/// Perl draws the same objects with the same attributes, except that it
/// writes the line/polygon/dot points and the circle radius with `ptValue`
/// into the px picture (points `0,0 56.91,28.45`, `r="14.23"`: 72.27% of
/// where pdflatex puts them, off the px frame) and loses the `{->}` arrows
/// (its `psTerminators` hook reads the stomach as the whatsit, :495).
/// Witnesses: egpeirce-doc, srdp-mathematik, egameps, lsc, dspTricksManual.
#[test]
fn pstricks_objects_draw_into_the_picture() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/graphics-tikz/pstricks_drawing_objects.tex");
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "picture",
    &[],
    r##"<picture fill="none" height="85.36pt" stroke="none" unitlength="28.45pt" width="113.81pt" xml:id="p1.pic1">
      <line arrowlength="4.34" fill="none" points="0,0 78.74,39.37" stroke="black" stroke-width="0.8" terminators="-&gt;"/>
      <rect fill="none" height="39.37" stroke="black" stroke-width="0.8" width="39.37" x="0" y="0"/>
      <circle fill="none" r="19.69" stroke="black" stroke-width="1" x="39.37" y="39.37"/>
      <polygon fill="none" points="0,0 39.37,0 39.37,39.37" stroke="black" stroke-width="0.8"/>
      <dots dotsize="2" dotstyle="*" fill="black" points="78.74,78.74"/>
      <line fill="none" points="0,39.37 39.37,78.74" stroke="#FF0000" stroke-width="1.42"/>
    </picture>"##,
  );
  if let Some(invalid) = rng_error_count(&xml) {
    assert_eq!(invalid, 0, "schema-invalid:\n{xml}");
  }
}

/// `\psset` is the raw pst-xkey one, so `linewidth=0.05` runs `\pssetlength
/// \pslinewidth{0.05}` (pstricks.tex:1078). The raw `\pssetlength` (:668,
/// :790) reads a bare number in `\psunit`, as pdflatex and Perl's own
/// `PSDimension` key do (0.05cm); the `\setlength` alias Perl keeps for its
/// user-level `\pssetlength` (:650) made it 0.05pt with two warnings.
/// `\psaddtolength` twice in a row, as lsc.sty does, keeps its unit the
/// second time: it is the raw pstricks.tex one too (:673-677), whose
/// `\advance` ends with `\afterassignment\pstunit@off`, fired since batch
/// 56is. `latex` prints the same 1.42271pt, 5.69046pt and -41.67911pt.
#[test]
fn psset_lengths_read_bare_numbers_in_psunit() {
  let tex = "\\documentclass{article}\n\\usepackage{pstricks}\n\\begin{document}\n\\psset{linewidth=0.05,linearc=0.2}\n\\newdimen\\y \\y=1pt \\psaddtolength{\\y}{-0.75}\\psaddtolength{\\y}{-0.75}\nW=\\the\\pslinewidth;A=\\the\\pslinearc;Y=\\the\\y.\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "p",
    &[],
    "<p>W=1.42271pt;A=5.69046pt;Y=-41.67911pt.</p>",
  );
}

/// The objects follow the parameter state: `linestyle` (dashed, dotted,
/// none), `fillstyle=solid` with `fillcolor`, a starred object filled with the
/// line colour, the `arrows` default from `\psset`, `\psarcn`'s reversed
/// arrows (`{->}` → `{<-}`: the head at the arc's start, which is where
/// pdflatex's clockwise arc ends) and swapped angles, `framearc` rounding,
/// `\degrees`, a frame under a negative `yunit` (ffslides.cls:102-103; Perl's
/// `height` came out negative and SVG drew nothing), and `\psgrid`'s options
/// consumed (Perl printed `[subgriddiv=1](0,0)(2,2)` and swallowed the rest
/// of the picture into that text).
#[test]
fn pstricks_objects_follow_the_parameter_state() {
  let tex = r"\documentclass{article}
\usepackage{pstricks}
\begin{document}
\begin{pspicture}(0,0)(4,3)
\psgrid[subgriddiv=1](0,0)(2,2)
\psline[linestyle=dashed](0,0)(1,0)
\psline[linestyle=dotted,dotsep=2pt](0,1)(1,1)
\psline[linestyle=none](0,2)(1,2)
\pspolygon[fillstyle=solid,fillcolor=blue](0,0)(1,0)(0,1)
\pspolygon[fillstyle=vlines](0,0)(1,0)(0,1)
\psline*[linecolor=green](0,0)(1,0)(1,1)
\psset{arrows=<->}
\psbezier(0,0)(1,1)(2,1)(3,0)
\psset{arrows=-}
\psarcn{->}(1,1){1}{90}{0}
\degrees[400]\pswedge(1,1){1}{100}{200}\degrees
\psframe[framearc=0.3](0,0)(2,1)
\psframe[yunit=-1cm](0,0)(1,1)
\end{pspicture}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "picture",
    &[],
    r##"<picture fill="none" height="85.36pt" stroke="none" unitlength="28.45pt" width="113.81pt" xml:id="p1.pic1">
      <grid/>
      <line fill="none" points="0,0 39.37,0" stroke="black" stroke-dasharray="5,3" stroke-width="0.8"/>
      <line fill="none" points="0,39.37 39.37,39.37" stroke="black" stroke-dasharray="1,2" stroke-width="0.8"/>
      <line fill="none" points="0,78.74 39.37,78.74" stroke="none" stroke-width="0.8"/>
      <polygon fill="#0000FF" points="0,0 39.37,0 0,39.37" stroke="black" stroke-width="0.8"/>
      <polygon fill="none" points="0,0 39.37,0 0,39.37" stroke="black" stroke-width="0.8"/>
      <line fill="#00FF00" points="0,0 39.37,0 39.37,39.37" stroke="none"/>
      <bezier arrowlength="4.34" points="0,0 39.37,39.37 78.74,39.37 118.11,0" stroke="black" stroke-width="0.8" terminators="&lt;-&gt;"/>
      <arc angle1="0" angle2="90" arrowlength="4.34" fill="none" r="39.37" stroke="black" stroke-width="0.8" terminators="&lt;-" x="39.37" y="39.37"/>
      <wedge angle1="90" angle2="180" fill="none" r="39.37" stroke="black" stroke-width="0.8" x="39.37" y="39.37"/>
      <rect fill="none" height="39.37" rx="5.91" stroke="black" stroke-width="0.8" width="78.74" x="0" y="0"/>
      <rect fill="none" height="39.37" stroke="black" stroke-width="0.8" width="39.37" x="0" y="-39.37"/>
    </picture>"##,
  );
  if let Some(invalid) = rng_error_count(&xml) {
    assert_eq!(invalid, 0, "schema-invalid:\n{xml}");
  }
}

/// A coordinate pstricks resolves from the nodes or in PostScript (`(A)`,
/// `(A|B)`, `([angle=45,nodesep=1]A)`, `(!1 2)`) cannot be placed here: the
/// object draws nothing rather than at the origin (P1 drew `\pscircle(A)` at
/// (0,0), and the bracketed node split at its inner comma with two "Missing
/// number" warnings), while `\rput` still places its label at the origin.
/// Polar `(1;45)` and mixed `(1,2|3,4)` coordinates resolve as pstricks'
/// `\SpecialCoor` reads them (pstricks.tex:830-949); `\radians` stores a
/// float full circle (Perl `Float(6.28319)`), so 3.14159 is half a turn (P1
/// stored the integer 6).
/// Perl: 1 error and 7 warnings, the polar pair leaked as text.
#[test]
fn pstricks_coordinates_that_cannot_be_placed_draw_nothing() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/graphics-tikz/pstricks_coordinate_forms.tex"
  );
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "picture",
    &[],
    r##"<picture fill="none" height="85.36pt" stroke="none" unitlength="28.45pt" width="113.81pt" xml:id="p1.pic1">
      <g transform="translate(39.37,39.37)">
        <text>X</text>
      </g>
      <line fill="none" points="39.37,0 27.84,27.84" stroke="black" stroke-width="0.8"/>
      <line fill="none" points="39.37,157.48 78.74,78.74" stroke="black" stroke-width="0.8"/>
      <g transform="translate(0,0)">
        <text>L</text>
      </g>
      <wedge angle1="0" angle2="180" fill="none" r="39.37" stroke="black" stroke-width="0.8" x="0" y="0"/>
    </picture>"##,
  );
  if let Some(invalid) = rng_error_count(&xml) {
    assert_eq!(invalid, 0, "schema-invalid:\n{xml}");
  }
}

/// Plain-TeX `\input pstricks` loads pstricks_support and not pstricks.sty:
/// `{pspicture}` is defined there, as in Perl (:527-563), so the picture is
/// sized (P1: the raw `\pspicture` ran and the line auto-opened an unsized
/// `ltx:picture`). A `\psline` in running text, which pdflatex draws over
/// the text in no box of its own, draws nothing (P1: an unsized picture that
/// swallowed " here.").
#[test]
fn plain_input_pstricks_sizes_its_pictures() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/graphics-tikz/pstricks_input_plain.tex");
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "para",
    &[],
    r##"<para xml:id="p1">
    <picture fill="none" height="28.45pt" stroke="none" unitlength="28.45pt" width="56.91pt" xml:id="p1.pic1">
      <line fill="none" points="0,0 39.37,39.37" stroke="#00FF00" stroke-width="0.8"/>
      <line fill="none" points="0,0 78.74,39.37" stroke="#404040" stroke-width="0.8"/>
    </picture>
    <p>A line here.
</p>
  </para>"##,
  );
}

/// `\newpsobject` keeps its keys as tokens and the objects read the value
/// macros expanded (egameps.sty:62-63 `linecolor=\@branchcolor,
/// linestyle=\@branchstyle,…`: Perl 1 error, 2 warnings, black lines of width
/// 0); a colour in any xcolor model (`[HTML]{FF8000}`: Perl and P1 black) or
/// a user-defined name; `\psarcn{->}` reversed to `<-`; the reflected
/// `\parabola` point rounded (P1 `66.78999999999999`). In an alignment cell
/// the object's `\begingroup` leaves the cells intact (Perl's `{`…
/// `Digest(T_END)` left its brace in the ledger: dspTricksManual 0 → 17
/// errors), inside a `{pspicture}` it draws, bare it draws nothing.
#[test]
fn pstricks_objects_read_macro_keys_colour_models_and_cells() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/graphics-tikz/pstricks_object_parameters.tex"
  );
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "picture",
    &[],
    r##"<picture fill="none" height="56.91pt" stroke="none" unitlength="28.45pt" width="85.36pt" xml:id="p1.pic1">
      <line fill="none" points="0,0 39.37,39.37" stroke="#0000FF" stroke-dasharray="5,3" stroke-width="1"/>
      <line fill="none" points="0,39.37 39.37,0" stroke="#FF0000" stroke-dasharray="5,3" stroke-width="1"/>
      <line fill="none" points="0,0 78.74,0" stroke="#FF8000" stroke-width="0.8"/>
      <line fill="none" points="0,39.37 78.74,39.37" stroke="#123456" stroke-width="0.8"/>
      <arc angle1="0" angle2="90" arrowlength="4.34" fill="none" r="39.37" stroke="black" stroke-width="0.8" terminators="&lt;-" x="39.37" y="39.37"/>
      <bezier points="11.81,0 39.3,78.74 66.79,0" stroke="black" stroke-width="0.8"/>
    </picture>"##,
  );
  assert_element(
    &xml,
    "tabular",
    &[],
    r##"<tabular vattach="middle">
      <tbody>
        <tr>
          <td align="center"><picture fill="none" height="28.45pt" stroke="none" unitlength="28.45pt" width="28.45pt" xml:id="p2.pic1">
              <line fill="none" points="0,0 39.37,39.37" stroke="black" stroke-width="0.8"/>
            </picture></td>
          <td align="center">B</td>
        </tr>
        <tr>
          <td align="center">C</td>
          <td align="center">D</td>
        </tr>
      </tbody>
    </tabular>"##,
  );
  if let Some(invalid) = rng_error_count(&xml) {
    assert_eq!(invalid, 0, "schema-invalid:\n{xml}");
  }
}

/// `\SpecialCoor`/`\NormalCoor` are the raw pstricks.tex modes (:746-806),
/// which switch the raw `\pssetlength`: pst-poly.tex:68-71 sets `\NormalCoor`
/// around `\pssetlength` of an empty length, which the `\SpecialCoor` form
/// (`\special@length#1#2\@nil#3`, :1001) cannot take. As no-ops (Perl
/// :1037-1038, sound beside Perl's own `\pssetlength` primitive) they left it
/// reading past its `\@nil` (hexgame 0 → 1 error). The polar coordinates
/// pst-poly then draws, a dimen register radius and a `\the\count<macro>`
/// angle, are read from the argument tokens: 0.5cm at 44.5 and 30 degrees.
#[test]
fn pstricks_coordinate_modes_are_the_raw_ones() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/graphics-tikz/pstricks_coordinate_modes.tex"
  );
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "p",
    &[],
    r##"<p>D=28.45274pt.
<picture fill="none" height="113.81pt" origin-x="-56.91pt" origin-y="-56.91pt" stroke="none" unitlength="28.45pt" width="113.81pt" xml:id="p1.pic1">
        <g transform="translate(78.74,78.74)">
          <line fill="none" points="14.04,13.8 0,39.37" stroke="black" stroke-width="0.8"/>
          <line fill="none" points="17.05,9.84 0,39.37" stroke="black" stroke-width="0.8"/>
        </g>
      </picture></p>"##,
  );
}

/// hexgame.sty's board (`\PstHexagon`, pst-poly) converts without the
/// "\@nil undefined" error. The 7 warnings are the `\multirput[]{}{}{}{}`
/// stub reading `(x,y)(dx,dy)` as brace arguments (10 before P1; Perl
/// defines `\multirput` with `PSCoord PSCoord`, :926-932).
#[test]
fn hexgame_board_converts() {
  let tex = "\\documentclass{article}\n\\usepackage{hexgame}\n\\begin{document}\n\\begin{hexgame}{3}\n\\colorhex{2}{playerone}\n\\labelhex{5}{7}\n\\end{hexgame}\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 7, "{stderr}");
  assert_element(
    &xml,
    "rect",
    &[r##"fill="#CC0000""##, r#"x="0""#],
    r##"<rect fill="#CC0000" height="119.33" stroke="black" stroke-width="0.8" width="196.85" x="0" y="119.33"/>"##,
  );
}

/// An object's `[fillcolor=##1]` inside a `\newcommand` in an `\afterpage`,
/// used in `tabularx` cells (sesamath-doc-fr:444-465): the raw `\psset`
/// resolves the colour at set time, as pstricks.tex:1203 does, once per
/// cell. (sesamath-doc-fr itself loads `sesamath-doc.sty`, absent from TeX
/// Live, so `\afterpage` is undefined, `##1` stays a parameter token and
/// every `\C` logs "Can't find color named '#1'"; `latex` logs "Undefined
/// color `##1'" there too.)
#[test]
fn pstricks_macro_colour_keys_in_tabularx_cells() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/graphics-tikz/pstricks_macro_colour_keys_tabularx.tex"
  );
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "tr",
    &[],
    r##"<tr>
          <td align="left"><inline-block vattach="top">
              <picture fill="none" height="42.68pt" stroke="none" unitlength="28.45pt" width="56.91pt" xml:id="p1.pic1">
                <rect fill="#008080" height="39.37" stroke="black" stroke-width="0.8" width="78.74" x="0" y="19.69"/>
                <g transform="translate(39.37,0)">
                  <text>A1: x</text>
                </g>
              </picture>
            </inline-block></td>
          <td align="left"><inline-block vattach="top">
              <picture fill="none" height="42.68pt" stroke="none" unitlength="28.45pt" width="56.91pt" xml:id="p1.pic2">
                <rect fill="#FF0000" height="39.37" stroke="black" stroke-width="0.8" width="78.74" x="0" y="19.69"/>
                <g transform="translate(39.37,0)">
                  <text>red: y</text>
                </g>
              </picture>
            </inline-block></td>
        </tr>"##,
  );
}
