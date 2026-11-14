use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: crop.sty.ltxml
  def_primitive_noop("\\crop []")?;
  def_primitive_noop("\\cropdef [] DefToken DefToken DefToken DefToken {}")?;
  // crop.sty:27-34 the stock size `\stockwidth`/`\stockheight` (the paper's, unless a class set them) and
  // :163 `\CROP@size{<width>}{<height>}`, which sets it — the page geometry, as the paper-size options
  // (:167ff) do. Classes built on crop call it: the OUP authoring template (`oup-authoring-template.cls`,
  // `\CROP@size{\croppaperwidth}{\croppaperheight}`; run 340: 137 first errors, witnesses 2508.14007,
  // 2406.14864, 2406.10778) and then `\renewcommand*`s the crop marks `\CROP@@ulc`, `\CROP@@urc`,
  // `\CROP@@llc`, `\CROP@@lrc` (crop.sty:288-331, drawing only: empty here).
  RawTeX!(r"\expandafter\ifx\csname stockwidth\endcsname\relax\newdimen\stockwidth\stockwidth\paperwidth\fi
\expandafter\ifx\csname stockheight\endcsname\relax\newdimen\stockheight\stockheight\paperheight\fi
\newcommand*\CROP@size[2]{\stockwidth#1 \stockheight#2 }
\newcommand*\CROP@@ulc{}\newcommand*\CROP@@urc{}\newcommand*\CROP@@llc{}\newcommand*\CROP@@lrc{}");
});
