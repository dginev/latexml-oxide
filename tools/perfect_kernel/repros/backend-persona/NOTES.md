# backend-persona — root-cause notes

## ifpdf_relet_relax_restored_by_graphicx_jinst (RED 62k → GREEN 62p)
A paper's `\let\ifpdf\relax` is undone in TeX by graphicx's pdftex driver: pdftex.def:681-701 loads epstopdf-base at
`\begin{document}` (:693), which requires pdftexcmds when `\@curroptions` is not empty (epstopdf-base.sty:151-182,
the require at :181), whose iftex (iftex.sty:268-291) defines `\ifpdf` afresh. The graphics binding ran no driver, so
`\ifpdf` stayed `\relax` and JINST's `\name` (JINST.cls:328-334, in every `\label`) left a stray `\fi`, which closed
the `\ifx` of the caption hack (sect09.rs `\@hack@caption@`) — a Fatal. RUST-ONLY exposure (OmniBus never ran
JINST's `\label`).
GREEN since 62p (KNOWN_PERL_ERRORS #487): graphics_sty.rs registers the driver's begin-document load under its own
conditions (driver, `\DoNotLoadEpstopdf`, `\@curroptions`), reduced to `\RequirePackage{pdftexcmds}`; `\ProcessOptions`
and the package loader keep `\@curroptions` as latex.ltx does. Guards
`perfect_kernel_batch61::{graphics_pdftex_driver_chain_restores_ifpdf, the_pdftex_driver_chain_follows_the_last_options}`.

## included_eps_beside_stray_pdf, pstricks_load_is_dvi (65f)
The K6 DVI cue read the tree's file inventory (any PDF/PNG/JPEG → PDF): 1012.0815's IEEEtran manual PDF beside its
all-EPS figures, 1002.1751's stale PDFs beside pstricks. RUST-ONLY (64h). GREEN since 65f (OD #537):
`converter::figure_output_cue` resolves the figures the source includes and reads a pstricks load; the tree walk is the
fallback. Guard `perfect_kernel_batch65f::included_figures_choose_the_output_mode`.

## epstopdf_conversion_beside_eps_is_dvi (65f round 3)
EPS figures beside epstopdf's `-eps-converted-to.pdf` conversions, or loading epstopdf, were compiled by arXiv with
latex+dvips (Producer "dvips + GPL Ghostscript": 1806.00093, 1806.00188, 1806.00370, 1804.02991, 2104.01314, 1209.6096,
2210.10273, 1701.08490). Round 2 took them as PDF cues (2 of 13 right); round 3 restores round 1's rule (OD #537): every
included figure resolved PostScript-only is DVI. Guard `perfect_kernel_batch65f::included_figures_choose_the_output_mode`.
