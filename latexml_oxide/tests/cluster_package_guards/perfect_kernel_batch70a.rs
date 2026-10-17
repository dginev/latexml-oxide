//! Red/green guards for batch 70a: binding shapes from the rc_struct4 study (`agent_reports/2026-10-17_rc_struct4.md`)
//! — a binding loads what its real package loads (biblatex: url, not hyperref; minted: fvextra; physics: xparse), a
//! TeX source given as a graphic is read as TeX (gincltex's `.tex` rule, svg's `\includeinkscape{x.pdf_tex}`), and a
//! binding's private helpers carry `\lx@` names, never one the real package leaves free (natbib's `\@@lbibitem`,
//! amsthm's `\th@proof`).
//!
//! Every repro is checked against pdflatex (its header holds the oracle).
use super::perfect_kernel_batch57::assert_elements;

/// The arXiv configuration's preload.
const AR5IV: &str = "ar5iv.sty";

/// package-bindings/biblatex_cleveref_before_biblatex: biblatex.sty:38-44 loads url, never hyperref, so a cleveref
/// loaded before biblatex passes its begin-document check (cleveref.sty:2370-2376) and `\cref` names the section;
/// `\url` is url.sty's (2108.12939, 2403.13692, 2501.07466, 2401.13438, 2211.04287, 2307.04997, 2210.03396,
/// 2112.07048, 2508.02914, 2112.15046, 2106.15390, 1811.07125; KNOWN_PERL_ERRORS #915).
#[test]
fn biblatex_loads_url_not_hyperref() {
  assert_elements(
    include_str!("../../../tools/perfect_kernel/repros/package-bindings/biblatex_cleveref_before_biblatex.tex"),
    AR5IV,
    (0, 0),
    &[(
      "para",
      "S1.p1",
      r##"<para xml:id="S1.p1"><p xml:id="S1.p1.1">See <ref labelref="LABEL:s" show="creftype~refnum"/>. At <ref class="ltx_nolink ltx_url" font="typewriter" href="https://example.org/a_b">https://example.org/a_b</ref>. NOHYPERREF</p></para>"##,
    )],
  );
}
