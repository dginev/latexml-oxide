use crate::prelude::*;

/// DEP-18 helper for empty-body `DefMacro!("\\cs[opt-spec]", "")` stubs.
fn def_macro_noop(proto: &str) -> Result<()> {
  let (cs_tok, params) = parse_prototype(proto, true)?;
  let body = mouth::tokenize_internal("");
  def_macro(cs_tok, params, ExpansionBody::Tokens(body), None)?;
  Ok(())
}


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

  // svgpath — code callback that pushes onto GRAPHICSPATHS.
  // Perl: DefKeyVal('Gin', 'svgpath', '', '', code => sub {
  //   my $root = $STATE->lookupValue('SOURCEDIRECTORY') || '';
  //   my $path = pathname_absolute(pathname_canonical(ToString($_[1])), $root);
  //   PushValue(GRAPHICSPATHS => $path); });
  // BLOCKER: Rust keyval::define doesn't dispatch the `code` field on set,
  // so per-\includegraphics `svgpath=X` invocations don't trigger the
  // GRAPHICSPATHS push. As a partial fix, parse the package-options form
  // (`\usepackage[svgpath=X]{svg}` / `\RequirePackage[svgpath=X]{svg}`)
  // at load time so at least the common preamble-level case works.
  DefKeyVal!("Gin", "svgpath",  "");
  if let Some(opts) = state::lookup_vecdeque("opt@svg.sty") {
    for opt in opts.iter() {
      let opt_str = opt.to_string();
      if let Some(val) = opt_str.strip_prefix("svgpath=") {
        let canonical = latexml_core::util::pathname::canonical(val.trim());
        let absolute = latexml_core::util::pathname::absolute(&canonical);
        // PushValue appends to back of the VecDeque (Perl PushValue semantics).
        let _ = state::push_value(
          "GRAPHICSPATHS",
          Stored::String(arena::pin(&absolute)),
        );
      }
    }
  }

  def_macro_noop("\\lx@svg@options")?;
  DefMacro!("\\setsvg{}", "\\gdef\\lx@svg@options{#1}");

  // Note that various sizing & rescaling are not yet supported by Post::Graphics
  DefMacro!("\\includesvg[]{}", "\\includegraphics[\\lx@svg@options,#1]{#2}");
});
