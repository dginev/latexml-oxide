# backend-persona — root-cause notes

## ifpdf_relet_relax_restored_by_graphicx_jinst (RED, 62k)
A paper's `\let\ifpdf\relax` is undone in TeX by graphicx's pdftex driver: pdftex.def:677-700 loads epstopdf-base at
`\begin{document}`, which (epstopdf-base.sty:181) loads pdftexcmds (:185), whose iftex (:269+) re-lets `\ifpdf` to
`\iftrue`. The graphics binding runs no driver, so `\ifpdf` stays `\relax` and JINST's `\name` (JINST.cls:328-334, in
every `\label`) leaves a stray `\fi`, which closes the `\ifx` of the caption hack (sect09.rs `\@hack@caption@`) — a
Fatal. RUST-ONLY exposure (OmniBus never ran JINST's `\label`). Fix direction: replay the driver's begin-document
loads in graphics_sty.rs (risk MED).
