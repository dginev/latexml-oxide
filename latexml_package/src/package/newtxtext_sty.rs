use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: newtxtext.sty.ltxml — nothing to do for the fonts themselves
  // (txfonts is all math commands; see newtxmath.sty). But newtxtext.sty:20
  // `\RequirePackage{xpatch,xcolor}` is what re-enables ltcmd's legacy `g`
  // argument type (xpatch.sty:42 → xparse.sty:98-106) for a class that
  // declares `\NewDocumentCommand\entry{m g}` after loading newtxtext
  // (prtec.cls:316; PRTEC24-template). Keep the dependency.
  // Guard: `perfect_kernel_batch56::package_state_prtec_psfragx_knowledge`.
  RequirePackage!("xpatch");
  RequirePackage!("xcolor");
  // newtxtext.sty:22 `\RequirePackage{xstring,ifthen,scalefnt}`: heria.cls:389
  // uses xstring's `\IfEq` after loading newtxtext (:758). Perl's binding omits
  // these too (SHARED); pdflatex clean.
  RequirePackage!("xstring");
  RequirePackage!("ifthen");
  RequirePackage!("scalefnt");
});
