//! fourier.sty — Utopia text/math fonts (presentational). fourier.sty:61
//! loads fourier-orns (`\lefthand`, `\decoone` … pgfornament's tikzrput
//! manual); the symbol set is what documents reach.
use crate::prelude::*;
#[rustfmt::skip]
LoadDefinitions!({
  // fourier.sty:61 `\RequirePackage[noOTF]{fourier-orns}`: no binding, so a missing-file warning
  // and the ornaments undefined. Its raw file draws each ornament as a slot of the `futs` font,
  // which has no glyph map here (font.rs), so a raw load would print the slot characters ("1" for
  // `\warning`) — invented text; RED `fonts-nfss/fourier_orns_ornaments_are_their_glyphs`.
  RequirePackage!("fourier-orns");
  // fourier.sty:300-303: `\math@bb` is the fourier-bb blackboard alphabet, and `\mathbb` becomes
  // it at `\begin{document}`, over amsfonts' and over a preamble
  // `\renewcommand{\mathbb}{\varmathbb}` (2406.06884; fourier has no `\varmathbb`).
  DefConstructor!("\\math@bb{}", "#1", bounded => true, require_math => true, alias => "\\mathbb",
    font => { family => "blackboard", series => "medium", shape => "upright" });
  RawTeX!(r"\AtBeginDocument{\let\mathbb\math@bb}");
});
