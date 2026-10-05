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
