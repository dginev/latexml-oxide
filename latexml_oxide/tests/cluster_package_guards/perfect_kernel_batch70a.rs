//! Red/green guards for batch 70a: binding shapes from the rc_struct4 study (`agent_reports/2026-10-17_rc_struct4.md`)
//! — a binding loads what its real package loads (biblatex: url, not hyperref; minted: fvextra; physics: xparse), a
//! TeX source given as a graphic is read as TeX (gincltex's `.tex` rule, svg's `\includeinkscape{x.pdf_tex}`), and a
//! binding's private helpers carry `\lx@` names, never one the real package leaves free (natbib's `\@@lbibitem`,
//! amsthm's `\th@proof`).
//!
//! Every repro is checked against pdflatex (its header holds the oracle).
use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch57::assert_elements,
};

/// The arXiv configuration's preload.
const AR5IV: &str = "ar5iv.sty";

/// package-bindings/biblatex_cleveref_before_biblatex: biblatex.sty:38-44 loads url, never hyperref, so a cleveref
/// loaded before biblatex passes its begin-document check (cleveref.sty:2370-2376) and `\cref` names the section;
/// `\url` is url.sty's (2108.12939, 2403.13692, 2501.07466, 2401.13438, 2211.04287, 2307.04997, 2210.03396,
/// 2112.07048, 2508.02914, 2112.15046, 2106.15390, 1811.07125; KNOWN_PERL_ERRORS #915).
#[test]
fn biblatex_loads_url_not_hyperref() {
  assert_elements(
    include_str!(
      "../../../tools/perfect_kernel/repros/package-bindings/biblatex_cleveref_before_biblatex.tex"
    ),
    AR5IV,
    (0, 0),
    &[(
      "para",
      "S1.p1",
      r##"<para xml:id="S1.p1"><p xml:id="S1.p1.1">See <ref labelref="LABEL:s" show="creftype~creflabel"/>. At <ref class="ltx_nolink ltx_url" font="typewriter" href="https://example.org/a_b">https://example.org/a_b</ref>. NOHYPERREF</p></para>"##,
    )],
  );
}

/// stub-dependencies/minted_loads_fvextra: minted.sty:32 loads fvextra, so `Verbatim`'s `breaklines` and
/// `breakanywhere` keys exist and the line is kept (2512.12360; KNOWN_PERL_ERRORS #916).
#[test]
fn minted_loads_fvextra() {
  assert_elements(
    include_str!("../../../tools/perfect_kernel/repros/stub-dependencies/minted_loads_fvextra.tex"),
    AR5IV,
    (0, 0),
    &[(
      "para",
      "p1",
      r##"<para xml:id="p1"><p xml:id="p1.1"><text class="ltx_verbatim" font="typewriter" fontsize="80%" width="345.0pt" xml:id="p1.1.1"><inline-logical-block class="ltx_parbox" vattach="top" width="345.0pt" xml:id="p1.1.1.1"><para class="ltx_noindent" xml:id="p1.p1"><p xml:id="p1.p1.1">some long verbatim line</p></para></inline-logical-block></text></p></para>"##,
    )],
  );
}

/// stub-dependencies/physics_loads_xparse: physics.sty:15 loads xparse, whose deprecated `g` argument type a
/// document's `\DeclareDocumentCommand\DD{ o g }` uses; `\DD{x}` is d x as pdflatex prints it (2603.20157;
/// KNOWN_PERL_ERRORS #917).
#[test]
fn physics_loads_xparse() {
  assert_elements(
    include_str!("../../../tools/perfect_kernel/repros/stub-dependencies/physics_loads_xparse.tex"),
    AR5IV,
    (0, 0),
    &[(
      "XMApp",
      "p1.m1.1.1",
      r##"<XMApp xml:id="p1.m1.1.1"><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" role="UNKNOWN">d</XMTok><XMTok font="italic" role="UNKNOWN">x</XMTok></XMApp>"##,
    )],
  );
}

/// package-bindings/natbib_citeref_bibitem: the natbib binding's entry constructor is `\lx@NAT@lbibitem`, so
/// citeref.sty:96's own `\@@lbibitem[#1]#2#3\par` no longer takes it over: the entry is one bibitem with its text
/// (2309.05436; KNOWN_PERL_ERRORS #918).
#[test]
fn natbib_entry_constructor_is_binding_private() {
  assert_elements(
    include_str!(
      "../../../tools/perfect_kernel/repros/package-bindings/natbib_citeref_bibitem.tex"
    ),
    AR5IV,
    (0, 0),
    &[(
      "bibitem",
      "bib.bib1",
      r##"<bibitem key="a" xml:id="bib.bib1"><tags><tag role="number">1</tag><tag role="year">2017</tag><tag role="authors">Batselier et al., </tag><tag role="fullauthors">Batselier et al., 2017</tag><tag role="refnum">Batselier et al.,  (2017)</tag><tag role="key">a</tag></tags><bibblock>Batselier, K. (2017).</bibblock></bibitem>"##,
    )],
  );
}

/// package-bindings/keytheorems_style_named_proof: amsthm.sty defines no `\th@proof`, so keytheorems'
/// `\newkeytheoremstyle{proof}` is a new style; the binding's proof head style is `\lx@amsthm@proof@style`
/// (2512.04638; KNOWN_PERL_ERRORS #919).
#[test]
fn amsthm_proof_style_is_binding_private() {
  assert_elements(
    include_str!(
      "../../../tools/perfect_kernel/repros/package-bindings/keytheorems_style_named_proof.tex"
    ),
    AR5IV,
    (0, 0),
    &[(
      "para",
      "p1",
      r##"<para xml:id="p1"><p xml:id="p1.1">Text.</p></para>"##,
    )],
  );
}

/// package-bindings/amsthm_proof_title_gobbler: the proof title ends `\@addpunct{.}` as amsthm.sty:435-437's does,
/// so a document's two-argument `\nopoint` at its end takes `\@addpunct` and `{.}` — the title is "Proof" without
/// its period, as pdflatex prints it — and never the title group's `}` (2503.02416; KNOWN_PERL_ERRORS #920).
#[test]
fn amsthm_proof_title_ends_with_addpunct() {
  assert_elements(
    include_str!(
      "../../../tools/perfect_kernel/repros/package-bindings/amsthm_proof_title_gobbler.tex"
    ),
    AR5IV,
    (0, 0),
    &[(
      "proof",
      "id1",
      r##"<proof xml:id="id1"><title class="ltx_runin" font="italic">Proof </title><para xml:id="p1"><p xml:id="p1.1"><text xml:id="p1.1.1">of the lemma. Text. ∎</text></p></para></proof>"##,
    )],
  );
}

/// graphics-tikz/gincltex_tex_graphic: gincltex.sty:43's `\DeclareGraphicsRule{.tex}{tex}{.tex}{}` is kept
/// (graphics.sty:150-154) and `\includegraphics[width=3cm]{x.tex}` follows it as graphics.sty's `\Gin@setfile` does:
/// gincltex's reader boxes the file (`\Gread@tex`), its include scales the box to the asked width (`\Ginclude@tex`,
/// `\resizebox*`). The TikZ picture is in the figure, 3cm (85.4pt) wide; no `.tex` goes to the image converter
/// (1604.05572, 2511.17298, 2305.04329, 2408.06999, 2305.16841).
#[test]
fn gincltex_tex_graphic_is_read_as_tex() {
  let (log, xml) = latexml::util::test::convert_files_with(
    include_str!("../../../tools/perfect_kernel/repros/graphics-tikz/gincltex_tex_graphic.tex"),
    &[(
      "gincltex_tz.tex",
      include_str!("../../../tools/perfect_kernel/repros/graphics-tikz/gincltex_tz.tex"),
    )],
    Some(AR5IV),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(!xml.contains("<graphics "), "{xml}");
  assert!(
    xml.contains(
      r#"<inline-block depth="0.0pt" height="65.2pt" width="85.4pt" xscale="1.84152433962028""#
    ),
    "{xml}"
  );
  latexml::util::test::assert_element(
    &xml,
    "picture",
    &[r#"xml:id="S0.F1.pic1""#],
    r##"<picture height="48.99" width="59.53" xml:id="S0.F1.pic1"><svg:svg height="48.99" overflow="visible" version="1.1" viewBox="0 0 59.53 48.99" width="59.53"><svg:g fill="#000000" stroke="#000000" stroke-width="0.4pt" transform="translate(0,48.99) matrix(1 0 0 -1 0 0) translate(0.28,0) translate(0,0.28)"><svg:path d="M 0 0 L 39.37 39.37" style="fill:none"/><svg:g fill="#000000" stroke="#000000" transform="matrix(1.0 0.0 0.0 1.0 44.26 34.64)"><svg:foreignObject height="9.46" overflow="visible" style="--ltx-fo-width:0.75em;--ltx-fo-height:0.68em;--ltx-fo-depth:0em;font-size:10pt;" transform="matrix(1 0 0 -1 0 9.46)" width="10.38"><text xml:id="S0.F1.pic1.1">A</text></svg:foreignObject></svg:g></svg:g></svg:svg></picture>"##,
  );
}

/// graphics-tikz/includeinkscape_overlay: svg's `\includeinkscape` inputs the Inkscape export's LaTeX overlay
/// (svg.sty:1018-1060), named by itself (`box.pdf_tex`) or by the export (`box.pdf`), at the asked width (the overlay's
/// `\svgwidth`); the overlay's `\includegraphics{box.pdf}` includes svg's own file, `inkscape/box.pdf`
/// (svg.sty:1348-1394). Each figure is the picture with its label (2402.04829, 2310.03515).
#[test]
fn includeinkscape_inputs_the_overlay() {
  let (log, xml, ()) = latexml::util::test::convert_bytes_files_then(
    include_bytes!(
      "../../../tools/perfect_kernel/repros/graphics-tikz/includeinkscape_overlay.tex"
    ),
    &[
      (
        "inkscape/box.pdf_tex",
        include_bytes!("../../../tools/perfect_kernel/repros/graphics-tikz/inkscape/box.pdf_tex"),
      ),
      (
        "inkscape/box.pdf",
        include_bytes!("../../../tools/perfect_kernel/repros/graphics-tikz/inkscape/box.pdf"),
      ),
    ],
    Some(AR5IV),
    |_, _| (),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  latexml::util::test::assert_element(
    &xml,
    "picture",
    &[r#"xml:id="S0.F1.pic1""#],
    r##"<picture fill="none" height="56.9pt" stroke="none" unitlength="113.8pt" width="113.8pt" xml:id="S0.F1.pic1"><g innerdepth="0.0pt" innerheight="161.0pt" innerwidth="113.8pt" transform="translate(0,0)"><graphics candidates="inkscape/box.pdf" graphic="inkscape/box.pdf" options="width=113.81102pt,page=1,keepaspectratio=true" xml:id="S0.F1.pic1.g1"/></g><g innerdepth="1.9pt" innerheight="6.9pt" innerwidth="58.1pt" transform="translate(15.75,31.5)"><g class="makebox" innerdepth="1.9pt" innerheight="6.9pt" innerwidth="58.1pt" transform="translate(0,-12.3)"><text xml:id="S0.F1.pic1.1">Overlay label</text></g></g></picture>"##,
  );
  latexml::util::test::assert_element(
    &xml,
    "graphics",
    &[r#"xml:id="S0.F2.pic1.g1""#],
    r##"<graphics candidates="inkscape/box.pdf" graphic="inkscape/box.pdf" options="width=57.11336pt,page=1,keepaspectratio=true" xml:id="S0.F2.pic1.g1"/>"##,
  );
  assert_eq!(xml.matches("Overlay label").count(), 2, "{xml}");
}

/// graphics-tikz/gnuplot_lua_tikz_driver_mode: gnuplot-lua-tikz-common.tex:17-32 finds this driver in its table
/// (`\gnuplot@select@driver@pgfsys-latexml.def`, pgfsys_latexml_def.rs) — `html`, pgfsys-tex4ht.def's mode, the
/// other driver writing SVG — so the plot style loads without its "driver not supported" `\errmessage` and picks
/// `\gp@rawimage@html` (:86; 2501.11366; OXIDIZED_DESIGN_DIVERGENCES #702).
#[test]
fn gnuplot_lua_tikz_driver_mode() {
  let (log, xml) = latexml::util::test::convert_files_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/graphics-tikz/gnuplot_lua_tikz_driver_mode.tex"
    ),
    &[(
      "gnuplot-lua-tikz-excerpt.sty",
      include_str!(
        "../../../tools/perfect_kernel/repros/graphics-tikz/gnuplot-lua-tikz-excerpt.sty"
      ),
    )],
    Some(AR5IV),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r##"<para xml:id="p1"><p xml:id="p1.1">Mode html: HTML</p></para>"##,
  );
}

/// graphics-tikz/standalone_tikzscale_tex_graphic: standalone loads gincltex (standalone.sty:1012-1013), tikzscale's
/// `\includegraphics` hands a non-TikZ file on as `{\tikzscale@fileName}` (tikzscale.sty:183; the name is expanded
/// as the constructor's Semiverbatim is) and epsfig leaves graphics.sty's `\DeclareGraphicsRule` alone: the `.tex` is
/// the TikZ picture, 3cm wide (2408.06999, 2511.17298, 2305.16841, 2410.05869, 2305.04329).
#[test]
fn standalone_loads_gincltex() {
  let (log, xml) = latexml::util::test::convert_files_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/graphics-tikz/standalone_tikzscale_tex_graphic.tex"
    ),
    &[(
      "gincltex_tz.tex",
      include_str!("../../../tools/perfect_kernel/repros/graphics-tikz/gincltex_tz.tex"),
    )],
    Some(AR5IV),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(!xml.contains("<graphics "), "{xml}");
  assert!(
    xml.contains(
      r#"<inline-block depth="0.0pt" height="65.2pt" width="85.4pt" xscale="1.84152433962028""#
    ),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<svg:path d="M 0 0 L 39.37 39.37" style="fill:none"/>"#),
    "{xml}"
  );
}
