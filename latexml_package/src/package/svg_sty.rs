use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: svg.sty.ltxml
  RequirePackage!("graphicx");
  RequirePackage!("subfig");
  RequirePackage!("xcolor");
  RequirePackage!("transparent");
  RequirePackage!("import");

  // Since we've already arranged for graphicx to accept svg, we're pretty much done.
  // There are some new options...
  DefKeyVal!("Gin", "pdf",      "", "true");
  DefKeyVal!("Gin", "eps",      "", "true");
  DefKeyVal!("Gin", "png",      "", "true");
  DefKeyVal!("Gin", "clean",    "", "true");
  DefKeyVal!("Gin", "exclude",  "", "true");
  DefKeyVal!("Gin", "pretex",   "", "true");
  DefKeyVal!("Gin", "postex",   "");
  DefKeyVal!("Gin", "preamble", "");
  DefKeyVal!("Gin", "end",      "");
  DefKeyVal!("Gin", "inkscape", "");
  DefKeyVal!("Gin", "pdflatex", "");
  DefKeyVal!("Gin", "pdftops",  "");
  DefKeyVal!("Gin", "convert",  "");

  // svgpath — code callback that pushes onto GRAPHICSPATHS
  // Perl: DefKeyVal('Gin', 'svgpath', '', '', code => sub { ... });
  // For now, register the key; the code callback is not yet ported.
  DefKeyVal!("Gin", "svgpath",  "");

  DefMacro!("\\lx@svg@options", "");
  DefMacro!("\\setsvg{}", "\\gdef\\lx@svg@options{#1}");

  // Note that various sizing & rescaling are not yet supported by Post::Graphics
  DefMacro!("\\includesvg[]{}", "\\includegraphics[\\lx@svg@options,#1]{#2}");
});
