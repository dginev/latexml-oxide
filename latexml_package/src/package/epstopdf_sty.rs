use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: epstopdf.sty.ltxml
  // Nothing to do here!
  // Beyond Perl's stub: epstopdf-base.sty:218-248 the `prepend` option (`\ETE@prependtrue`), which its load acts on
  // below, and `append`, the default; the rest (`update`, `outdir=…`, `suffix=…`) drive a conversion the
  // post-processor does in its own way.
  RawTeX!(r"\newif\ifETE@prepend");
  DeclareOption!("prepend", "\\ETE@prependtrue");
  DeclareOption!("append", "\\ETE@prependfalse");
  DeclareOption!(None, None);
  ProcessOptions!();
  // epstopdf-base.sty:509-515 (`\epstopdfsetup` is `\setkeys{ETE}`): after the load, `prepend` is
  // `\PrependGraphicsExtensions{.eps}` and `append` `\AppendGraphicsExtensions{.eps}` — `\epstopdfsetup{prepend}`
  // makes `fig` beside `fig.png` and `fig.eps` the eps, as pdflatex converts it (review rev79a2 finding 4, probe gd).
  // The other keys mean nothing here.
  DefMacro!("\\epstopdfsetup{}", sub[(keys)] {
    let mut code = String::new();
    for key in keys.to_string().split(',') {
      match key.split('=').next().unwrap_or_default().trim() {
        "prepend" => code.push_str(r"\PrependGraphicsExtensions{.eps}"),
        "append" => code.push_str(r"\AppendGraphicsExtensions{.eps}"),
        _ => {},
      }
    }
    Ok(TokenizeInternal!(TeXString::assembled(code)))
  });
  // epstopdf.sty:150 `\RequirePackage{grfext}`: its `\AppendGraphicsExtensions`/
  // `\PrependGraphicsExtensions`/`\RemoveGraphicsExtensions` edit
  // `\Gin@extensions` (grfext.sty:153-237), which a document may call once
  // epstopdf is loaded. Perl's stub leaves them undefined; they surfaced once
  // PDF output made `\ifpdf\usepackage{epstopdf}\PrependGraphicsExtensions{.svg}\fi`
  // run (arXiv 2606.05709, sweep of run 322). Loaded raw, they edit the list
  // graphics_sty.rs seeds with pdftex.def's, which orders an image's
  // candidates (util/image.rs `declared_graphics_extensions`; 1711.06128's
  // `\AppendGraphicsExtensions{.gif}`).
  InputDefinitions!("grfext", noltxml => true, extension => Some(Cow::Borrowed("sty")));
  // epstopdf-base.sty:498-505 `\AppendGraphicsExtensions{.eps}`: `.eps` last, after the extensions the document
  // declared before (graphics_sty.rs `\lx@graphics@eps@last`); with the `prepend` option
  // `\PrependGraphicsExtensions{.eps}`, first (pdflatex `\usepackage[prepend]{epstopdf}` takes the eps beside the png,
  // review rev79a2 finding 4, probe ga). The driver's begin-document load of epstopdf-base (pdftex.def:681-701) does
  // not undo either: epstopdf is loaded.
  RawTeX!(
    r"\ifx\lx@graphics@eps@last\@undefined\else
      \ifETE@prepend\PrependGraphicsExtensions{.eps}\else\lx@graphics@eps@last\fi
    \fi"
  );
  // epstopdf.sty:149-152 (and epstopdf-base.sty:178-181) also load infwarerr, kvoptions and pdftexcmds, whose
  // iftex gives the document `\ifpdf` (witnesses 2410.22009, 2305.04598, 2602.17971: `\usepackage{epstopdf}`,
  // then `\ifpdf` in the preamble; Perl's stub loads none of them).
  RequirePackage!("infwarerr");
  RequirePackage!("kvoptions");
  RequirePackage!("pdftexcmds");
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
