use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: xypic.tex.ltxml — the `\input xypic` entry point.
  //
  //   InputDefinitions('xy', type => 'tex');
  //   RawTeX('\xyoption{v2}');
  //
  // CRITICAL (Perl-faithful split): this loads the xy *tex* binding
  // overlay (= our `xy_sty.rs`, registered for `("xy","tex")`) but does
  // NOT `RequirePackage('xy')`, so the xy *package* (`xy.sty`) is left
  // UN-marked. A subsequent `\usepackage[all]{xy}` therefore still runs
  // its option processing (loading the curve/arrow/... feature files),
  // rather than being short-circuited by the "already loaded" option-clash
  // early-stop in `input_definitions`.
  //
  // Why this matters: a curved arrow `\ar@/^1pc/[u]` digests through
  // `\crvi` (xycurve.tex L69), which only defines when the `curve` feature
  // loads via `\xyoption{all}` (or `{curve}`). When `\input xypic` mis-marks
  // `xy.sty` loaded (the previous `RequirePackage('xy')` behaviour), the
  // document's own `\usepackage[all]{xy}` got dropped, curve never loaded,
  // and `\crvi` stayed undefined. Witness 2011.01105 (`\input xypic` +
  // `\usepackage[all]{xy}` + `\ar@/^1pc/`, Perl-clean); also 2311.05789
  // (`\xygraph` from `[all,tips]`).
  //
  // `\usepackage{xypic}` (the *.sty* entry) keeps Perl `xypic.sty.ltxml`'s
  // `RequirePackage('xy', options => ['v2'])` semantics in `xypic_sty.rs`.
  InputDefinitions!("xy", extension => Some(Cow::Borrowed("tex")));
  Digest!("\\xyoption{v2}")?;
});
