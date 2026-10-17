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
    include_str!("../../../tools/perfect_kernel/repros/package-bindings/natbib_citeref_bibitem.tex"),
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
    include_str!("../../../tools/perfect_kernel/repros/package-bindings/keytheorems_style_named_proof.tex"),
    AR5IV,
    (0, 0),
    &[("para", "p1", r##"<para xml:id="p1"><p xml:id="p1.1">Text.</p></para>"##)],
  );
}

/// package-bindings/amsthm_proof_title_gobbler: the proof title ends `\@addpunct{.}` as amsthm.sty:435-437's does,
/// so a document's two-argument `\nopoint` at its end takes `\@addpunct` and `{.}` — the title is "Proof" without
/// its period, as pdflatex prints it — and never the title group's `}` (2503.02416; KNOWN_PERL_ERRORS #920).
#[test]
fn amsthm_proof_title_ends_with_addpunct() {
  assert_elements(
    include_str!("../../../tools/perfect_kernel/repros/package-bindings/amsthm_proof_title_gobbler.tex"),
    AR5IV,
    (0, 0),
    &[(
      "proof",
      "id1",
      r##"<proof xml:id="id1"><title class="ltx_runin" font="italic">Proof </title><para xml:id="p1"><p xml:id="p1.1"><text xml:id="p1.1.1">of the lemma. Text. ∎</text></p></para></proof>"##,
    )],
  );
}
