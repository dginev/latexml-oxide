use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: svg.sty.ltxml
  RequirePackage!("graphicx");
  // Not subfig: svg.sty loads iftex, scrbase, pdftexcmds, trimspaces, graphicx, shellesc (svg.sty:66-73)
  // and xcolor/transparent on demand (:337-352). Perl's `RequirePackage('subfig')` (svg.sty.ltxml:19)
  // put subfig beside subcaption, and which `\ContinuedFloat` was in force then depended on load
  // order (witness 2605.17685; KPE #322).
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
  // svg.sty:328-683 (`\DefineFamilyKey{SVG}`, `\FamilyBoolKey{SVG}{inkscapelatex}`): how Inkscape converts the SVG for
  // TeX; no effect on the graphic here, but options `\setsvg`/`\svgsetup` give every `\includesvg`.
  DefKeyVal!("Gin", "inkscapelatex",    "", "true");
  DefKeyVal!("Gin", "inkscapeversion",  "");
  DefKeyVal!("Gin", "inkscapeexe",      "");
  DefKeyVal!("Gin", "inkscapeopt",      "");
  DefKeyVal!("Gin", "inkscapeformat",   "");
  DefKeyVal!("Gin", "inkscapearea",     "");
  DefKeyVal!("Gin", "inkscapedpi",      "");
  DefKeyVal!("Gin", "inkscapedensity",  "");
  DefKeyVal!("Gin", "inkscapepath",     "");
  DefKeyVal!("Gin", "inkscapename",     "");
  DefKeyVal!("Gin", "svgextension",     "");
  DefKeyVal!("Gin", "extension",        "");
  DefKeyVal!("Gin", "ext",              "");
  DefKeyVal!("Gin", "apptex",           "");
  DefKeyVal!("Gin", "lastpage",         "", "true");
  DefKeyVal!("Gin", "distort",          "", "true");
  DefKeyVal!("Gin", "latex",            "", "true");
  DefKeyVal!("Gin", "tex",              "", "true");
  DefKeyVal!("Gin", "usexcolor",        "", "true");
  DefKeyVal!("Gin", "usetransparent",   "", "true");

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
  if let Some(opts) = lookup_vecdeque("opt@svg.sty") {
    for opt in opts.iter() {
      let opt_str = opt.to_string();
      if let Some(val) = opt_str.strip_prefix("svgpath=") {
        let absolute = svg_search_path(val.trim());
        // PushValue appends to back of the VecDeque (Perl PushValue semantics).
        let _ = push_value(
          "GRAPHICSPATHS",
          Stored::String(pin(&absolute)),
        );
      }
    }
  }

  def_macro_noop("\\lx@svg@options")?;
  // svg.sty:810-811: `\setsvg` and `\svgsetup` (2401.10458, 2402.15627, 2504.02263, 2505.19061, 2512.15659) add to
  // the options every `\includesvg` takes, as `\FamilyOptions` does.
  DefMacro!("\\setsvg{}",
    "\\xdef\\lx@svg@options{\\unexpanded\\expandafter{\\lx@svg@options},\\unexpanded{#1}}");
  DefMacro!("\\svgsetup{}",
    "\\xdef\\lx@svg@options{\\unexpanded\\expandafter{\\lx@svg@options},\\unexpanded{#1}}");
  // svg.sty:814-823 `\svgpath{{a/}{b/}}`, or a bare `\svgpath{a/}` it wraps in braces: where SVG files are found,
  // pushed onto GRAPHICSPATHS as the `svgpath=` option is (2402.15627 `\svgpath{{svg/}}`). The argument is read as
  // written, as `\graphicspath`'s is: a macro in it is not expanded.
  DefPrimitive!("\\svgpath{}", sub[(paths)] {
    let tokens = paths.unlist();
    let grouped = tokens.iter().find(|t| **t != T_SPACE!()).is_some_and(|t| t.get_catcode() == Catcode::BEGIN);
    let mut found: Vec<String> = Vec::new();
    if grouped {
      let (mut depth, mut current) = (0usize, String::new());
      for t in &tokens {
        match t.get_catcode() {
          Catcode::BEGIN => {
            if depth > 0 {
              current.push('{');
            }
            depth += 1;
          },
          Catcode::END => {
            depth = depth.saturating_sub(1);
            if depth == 0 {
              found.push(std::mem::take(&mut current));
            } else {
              current.push('}');
            }
          },
          _ if depth > 0 => current.push_str(&t.to_string()),
          _ => {},
        }
      }
    } else {
      found.push(Tokens::new(tokens).to_string());
    }
    for path in found.iter().map(|p| p.trim()).filter(|p| !p.is_empty()) {
      let _ = push_value("GRAPHICSPATHS", Stored::String(pin(svg_search_path(path))));
    }
  });

  // Note that various sizing & rescaling are not yet supported by Post::Graphics. The options given by `\setsvg` /
  // `\svgsetup` are expanded before `\includegraphics` reads them: its keys are read unexpanded (keyvals.rs, as TeX's
  // keyval does), and `\lx@svg@options` as a key was no option at all (2401.10458).
  DefMacro!("\\includesvg[]{}",
    "\\expandafter\\lx@svg@includegraphics\\expandafter{\\lx@svg@options}{#1}{#2}");
  DefMacro!("\\lx@svg@includegraphics{}{}{}", "\\includegraphics[#1,#2]{#3}");
  // svg.sty:876 `\includeinkscape`: an Inkscape export included as `\includesvg` includes it.
  DefMacro!("\\includeinkscape[]{}",
    "\\expandafter\\lx@svg@includegraphics\\expandafter{\\lx@svg@options}{#1}{#2}");
});

/// Where a path of the svg package's search path is: relative to the paper's source directory, as `\graphicspath`'s
/// paths are (Perl svg.sty.ltxml:42-44 `pathname_absolute(…, SOURCEDIRECTORY)`); the process's directory is no root.
fn svg_search_path(path: &str) -> String {
  let canonical = pathname::canonical(path.trim_matches('"'));
  let root = with_value("SOURCEDIRECTORY", |v| {
    v.map(|s| s.to_string()).unwrap_or_default()
  });
  if root.is_empty() || canonical.starts_with('/') {
    canonical
  } else {
    s!("{root}/{canonical}")
  }
}
