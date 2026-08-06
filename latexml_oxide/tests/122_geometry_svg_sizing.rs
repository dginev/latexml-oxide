//! Regression test: the `geometry` package's page layout must size **measured
//! SVG graphics** (tcolorbox / tikz / pgf pictures) so their aspect ratio
//! matches the PDF, while leaving the **reflowable HTML flow** untouched
//! (OXIDIZED_DESIGN #99).
//!
//! LaTeXML (Perl and, faithfully, Rust) ignores `geometry` for the HTML body —
//! page geometry is meaningless for reflowable output. But a measured picture
//! that reads `\linewidth` (e.g. `\begin{tcolorbox}[width=0.5\linewidth]`) is
//! emitted as a *fixed-size* SVG, so ignoring geometry there makes the box
//! `0.5 x 345pt` (the article default) instead of the `0.5 x 472pt` the PDF uses
//! (letterpaper minus fairmeta's 2.5cm margins). The narrow interior over-wraps
//! the content, doubling the box height and pushing text through the border
//! (arXiv:2605.29955, Figure 1).
//!
//! The fix injects the geometry-computed text width into `\linewidth` **only
//! inside SVG-producing pictures**, never into the main HTML flow. These tests
//! pin the contract in-process (via `convert_fixture`, no spawned binary):
//!   1. the tcolorbox SVG widens to the geometry `\linewidth` (was 345-based),
//!   2. a sibling `\rule{0.5\linewidth}` stays at the 345-based class default
//!      (proof the HTML flow is NOT rescaled),
//!   3. the SVG foreignObject `--ltx-fo-*` em sizing is preserved,
//!   4. a `width=\linewidth` box nested in a reduced-width minipage keeps the
//!      reduced width (the guard compares the class-default column, not the live
//!      minipage-reduced `\textwidth`).
//!
//! Conditional: needs the kernel dump (so tcolorbox's expl3 loads cleanly) AND
//! tcolorbox installed in the host TeX tree; self-skips otherwise.

use latexml::util::test::{convert_fixture, dump_available, kpse_has};

/// Pull the first `viewBox="minx miny W H"` width (the 3rd number) out of the
/// serialized markup — the tcolorbox root `<svg>` is the first picture.
fn first_viewbox_width(html: &str) -> f64 {
  let at = html.find("viewBox=\"").expect("no viewBox in output");
  let vb = &html[at + "viewBox=\"".len()..];
  let vb = &vb[..vb.find('"').expect("unterminated viewBox")];
  vb.split_whitespace()
    .nth(2)
    .and_then(|s| s.parse::<f64>().ok())
    .expect("could not parse viewBox width")
}

/// Pull the first `--ltx-fo-width:<N>em` value out of the markup.
fn fo_width_em(html: &str) -> f64 {
  let at = html
    .find("--ltx-fo-width:")
    .expect("no --ltx-fo-width in SVG output");
  let rest = &html[at + "--ltx-fo-width:".len()..];
  let end = rest.find("em").expect("--ltx-fo-width not in em");
  rest[..end]
    .parse::<f64>()
    .expect("could not parse --ltx-fo-width")
}

/// True when this environment can raw-load tcolorbox on the kernel dump.
fn can_run() -> bool { dump_available() && kpse_has("tcolorbox.sty") }

const TEX: &str = "literal:\\documentclass{article}\n\
  \\usepackage[left=2.5cm,right=2.5cm,top=2.5cm,bottom=2.5cm]{geometry}\n\
  \\usepackage[most]{tcolorbox}\n\
  \\begin{document}\n\
  \\noindent\\rule{0.5\\linewidth}{2pt}\n\
  \n\
  \\begin{tcolorbox}[width=0.5\\linewidth]\n\
  Some tcolorbox content here.\n\
  \\end{tcolorbox}\n\
  \\end{document}\n";

#[test]
fn geometry_sizes_svg_but_not_html_flow() {
  if !can_run() {
    eprintln!("skipping: needs kernel dump + tcolorbox.sty in the host TeX tree");
    return;
  }
  let r = convert_fixture(TEX);
  let html = r.result.expect("conversion produced no result");

  // (1) The tcolorbox SVG is sized from the geometry \linewidth (0.5 x 472pt ->
  // ~326.6px), NOT the 345pt article default (~238.7px). letterpaper (8.5in =
  // 614.295pt) minus left+right (2 x 2.5cm = 142.264pt) -> \textwidth 472.03pt;
  // 0.5\linewidth = 236.02pt, emitted in px (x DPI/72.27 = x100/72.27) as ~326.6.
  let vb_w = first_viewbox_width(&html);
  assert!(
    (315.0..=340.0).contains(&vb_w),
    "tcolorbox SVG viewBox width {vb_w} is not geometry-sized (~326.6); \
     a 345pt-default build emits ~238.7. geometry text width was not applied \
     to the SVG picture.\n{html}",
  );

  // (2) The HTML flow is untouched: a sibling \rule{0.5\linewidth} keeps the
  // 345-based class default (0.5 x 345 = 172.5pt). Geometry must NOT rescale
  // the reflowable HTML body.
  assert!(
    html.contains("width=\"172.5pt\""),
    "sibling \\rule{{0.5\\linewidth}} should stay at the 345-based default \
     172.5pt (HTML flow must not be rescaled by geometry).\n{html}",
  );

  // (3) SVG box-content emulation intact: the foreignObject still carries an
  // em-valued --ltx-fo-width, now widened by the geometry linewidth (~20.5em
  // vs the 345-default ~14.1em).
  let fo_em = fo_width_em(&html);
  assert!(
    (18.0..=22.0).contains(&fo_em),
    "SVG foreignObject --ltx-fo-width {fo_em}em is not geometry-widened \
     (~20.5em expected; 345-default ~14.1em). SVG content sizing regressed.\n{html}",
  );
}

// A `width=\linewidth` tcolorbox nested in a reduced-width minipage must keep
// the REDUCED width, not be clobbered to the full geometry width. A minipage
// sets \textwidth=\linewidth locally, so the SVG-scope guard must compare
// against the class-default column width, not the live \textwidth.
const TEX_NESTED: &str = "literal:\\documentclass{article}\n\
  \\usepackage[left=2.5cm,right=2.5cm]{geometry}\n\
  \\usepackage[most]{tcolorbox}\n\
  \\begin{document}\n\
  \\begin{minipage}{0.4\\linewidth}\n\
  \\begin{tcolorbox}[width=\\linewidth]x\\end{tcolorbox}\n\
  \\end{minipage}\n\
  \\end{document}\n";

#[test]
fn geometry_does_not_clobber_reduced_linewidth() {
  if !can_run() {
    eprintln!("skipping: needs kernel dump + tcolorbox.sty in the host TeX tree");
    return;
  }
  let r = convert_fixture(TEX_NESTED);
  let html = r.result.expect("conversion produced no result");

  // 0.4 x 345pt = 138pt -> ~190.9px. If the guard mis-fired against the live
  // (minipage-reduced) \textwidth it would jump to the full geometry width
  // (~653px). Must stay in the reduced band.
  let vb_w = first_viewbox_width(&html);
  assert!(
    (170.0..=215.0).contains(&vb_w),
    "nested tcolorbox width {vb_w} was clobbered by geometry; a reduced \
     \\linewidth (minipage) must be preserved (expected ~190.9px).\n{html}",
  );
}
