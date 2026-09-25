use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: epstopdf.sty.ltxml
  // Nothing to do here!
  DefMacro!("\\epstopdfsetup{}", None);
  // epstopdf.sty:150 `\RequirePackage{grfext}`: its `\AppendGraphicsExtensions`/
  // `\PrependGraphicsExtensions`/`\RemoveGraphicsExtensions` edit
  // `\Gin@extensions` (grfext.sty:153-237), which a document may call once
  // epstopdf is loaded. Perl's stub leaves them undefined; they surfaced once
  // PDF output made `\ifpdf\usepackage{epstopdf}\PrependGraphicsExtensions{.svg}\fi`
  // run (arXiv 2606.05709, sweep of run 322). Our graphics lookup does not
  // read the list, so loading the real package raw is enough.
  InputDefinitions!("grfext", noltxml => true, extension => Some(Cow::Borrowed("sty")));
  // `\epstopdfDeclareGraphicsRule{ext}{type}{ext-out}{cmd}` (epstopdf-base.sty)
  // registers an EPS→<other> conversion command for graphics inclusion.
  // We delegate graphics format conversion to mutool/gs at the post-
  // processing graphics phase, so author-supplied conversion rules
  // have no effect on our HTML output. No-op stub mirrors Perl's
  // "nothing to do" philosophy for the whole package.
  // Witness: 2 papers in R06-R09 emitting `Error:undefined:
  // \epstopdfDeclareGraphicsRule`.
  DefMacro!("\\epstopdfDeclareGraphicsRule{}{}{}{}", None);
  // `\epstopdfcall{cmd}` is the lower-level shell-out wrapper used by
  // \epstopdfDeclareGraphicsRule. Same rationale: no-op.
  DefMacro!("\\epstopdfcall{}", None);
  // `\OutputFile` is `\edef`-set inside `\epstopdfDeclareGraphicsRule`
  // / `\epstopdfcall` to the generated converted-file name. Since we
  // stub those as no-ops, `\OutputFile` is never set. Subsequent
  // references inside `\Gin@rule@` etc. then hit `Error:undefined`
  // (~11 papers in R-stages). Initialize as empty so it never errors;
  // its value would be the converted filename but we don't actually
  // include via this path.
  DefMacro!("\\OutputFile", None);
});
