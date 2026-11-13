use latexml_package::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // modernposter.cls: LaTeX poster template based on a0poster.
  // In print LaTeX, modernposter creates an overlay tikzpicture spanning the whole page
  // and positions column boxes via TikZ coordinate calculations relative to an overlay node `sep`.
  // In LaTeXML, TikZ overlay pictures do not provide semantic structure and fail with
  // "No shape named sep is known" (WITNESS: modernposter/demo).
  // We model modernposter after a0poster, loading a0poster (which loads article)
  // and defining `postercolumn`, `posterbox`, `doubleposterbox` as semantic XML blocks,
  // while preserving standard frontmatter (\title, \author, \email, \maketitle).

  // Class options
  DeclareOption!("helvet", "\\def\\@helvettrue{}");
  DeclareOption!(None, {});
  ProcessOptions!();

  LoadClass!("a0poster");

  RequirePackage!("xcolor");
  RequirePackage!("hyperref");

  // Modernposter theme colors and highlight macro
  RawTeX!(r"
    \definecolor{mDarkTeal}{HTML}{23373b}
    \definecolor{mDarkBrown}{HTML}{604c38}
    \definecolor{hlcol}{HTML}{FF8000}
    \newcommand{\highlight}[1]{\textcolor{hlcol}{#1}}
  ");

  // Dimension registers used by modernposter or document code
  RawTeX!(r"
    \newdimen\colheight
    \newdimen\colwidth
    \newdimen\coltextwidth
    \newdimen\colsep
    \newdimen\boxheight
    \newdimen\boxlinewidth
    \newcounter{numcols}
  ");

  // Fontawesome icon compatibility
  RawTeX!(r"
    \providecommand{\faicon}[1]{}
  ");

  // Frontmatter: \email attached to frontmatter
  DefMacro!("\\email{}", "\\@add@frontmatter{ltx:note}[role=email]{#1}");

  // Semantic container environments and constructors
  DefEnvironment!("{postercolumn}", "<ltx:block class='ltx_postercolumn'>#body</ltx:block>");

  DefConstructor!(
    "\\posterbox []{}{}",
    "<ltx:block class='ltx_posterbox'><ltx:p class='ltx_posterbox_title'><ltx:text font='bold'>#2</ltx:text></ltx:p>#3</ltx:block>"
  );

  DefConstructor!(
    "\\doubleposterbox []{}{}{}{}",
    "<ltx:block class='ltx_doubleposterbox'><ltx:block class='ltx_posterbox'><ltx:p class='ltx_posterbox_title'><ltx:text font='bold'>#2</ltx:text></ltx:p>#3</ltx:block><ltx:block class='ltx_posterbox'><ltx:p class='ltx_posterbox_title'><ltx:text font='bold'>#4</ltx:text></ltx:p>#5</ltx:block></ltx:block>"
  );
});
