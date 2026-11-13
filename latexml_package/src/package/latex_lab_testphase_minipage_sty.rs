//! latex-lab-testphase-minipage.sty — the tagging sockets the module declares.
//!
//! `\DocumentMetadata{tagging=on}` loads `latex-lab-testphase-latest.sty`
//! (documentmetadata-support.ltx:72), which pulls in this module (:47); its
//! only surface other raw code consumes are the four sockets
//! (latex-lab-testphase-minipage.sty:47-50) — tagpdfdocu-patches.sty:146-149
//! `\AssignSocketPlug{tagsupport/minipage/before}{noop}` errored "Socket
//! undeclared!" (latex.ltx:7405) without them (tagpdf manual: 88 socket error
//! lines; SHARED, pdflatex clean). The `\@iiiminipage`/`\@iiiparbox` rewrites
//! that call the sockets are PDF-structure only and stay with our minipage.
//! Guard: `perfect_kernel_batch56::testphase_tagging_sockets_and_block_templates_are_declared`.
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  RawTeX!(r"\NewTaggingSocket{minipage/before}{0}
\NewTaggingSocket{minipage/after}{0}
\NewTaggingSocket{parbox/before}{0}
\NewTaggingSocket{parbox/after}{0}");
});
