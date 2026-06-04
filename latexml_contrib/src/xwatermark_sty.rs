//! Stub for xwatermark.sty — page-watermark package (visual-only).
//!
//! Watermarks have no XML/HTML output. Raw-loading xwatermark triggers
//! a transitive load of catoptions, which uses expl3-style macros our
//! engine doesn't fully emulate. The result is a 100-error cascade
//! that often OOMs (SIGKILL). Stub the public macros instead.
//!
//! Witness: 2202.02001, 2201.03132, 2202.12029, 2204.05441, 2204.12966
//! and ~20 sibling FATAL_137 (OOM-during-cascade) papers.
use latexml_package::prelude::*;

LoadDefinitions!({
  // xwatermark.sty L31/L52 `\RequirePackage{catoptions}` +
  // `\ifcsndefTF{ver@hyperref.sty}{}{\usepackage{hyperref}}` — loading
  // xwatermark pulls in hyperref (and catoptions). Perl has no xwatermark
  // binding: it raw-loads xwatermark.sty and gets hyperref → `\href`/`\url`
  // become available to the whole document (e.g. a `plainurl` .bbl with
  // `\href{doi}{...}` entries). This stub previously omitted the hyperref
  // dependency, so a paper that loads xwatermark but NOT hyperref directly
  // saw `\href` undefined when the bibliography rendered — and the entire
  // <ltx:bibliography> failed (witness 2001.03244: `\usepackage[printwatermark]
  // {xwatermark}` + `\bibliographystyle{plainurl}`; 1 error + empty bib → 0
  // errors + full bib). Mirror the real package's dependencies. (catoptions is
  // the safe Rust stub, not the cascade-prone raw load this stub was created
  // to avoid.)
  RequirePackage!("catoptions");
  RequirePackage!("hyperref");

  // No-op the public watermark API
  def_macro_noop("\\newwatermark{}")?;
  def_macro_noop("\\xnewwatermark[]{}")?;
  def_macro_noop("\\xnewwatermark*[]{}")?;
  def_macro_noop("\\removewatermark{}")?;
  def_macro_noop("\\DeclareWatermarkParser{}")?;
  // Options (printwatermark, draft, etc.) handled by DeclareOption no-ops
  for opt in ["printwatermark", "draft", "final", "noprintwatermark"].iter() {
    DeclareOption!(*opt, None);
  }
  DeclareOption!(None, {});
  ProcessOptions!();
});
