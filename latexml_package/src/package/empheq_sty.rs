use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // TODO: Styling.
  // This is an initial binding that will allow the package to function,
  // but does not apply any of the styling to emphasized equations.
  RequirePackage!("amsmath");
  RequirePackage!("mathtools");

  DefMacro!("\\empheqset{}", None);

  // Just pass on to the ams environment
  // Usually \begin{empheq}{amsenv} ==> \begin{amsenv}
  // but note specialcase: \begin{empheq}{alignat=2} ==> \begin{alignat}{2} !!!
  DefMacro!("\\empheq[]{}",
    "\\empheqset{#1}\\lx@empheq#2==\\end");

  RawTeX!("\\def\\lx@empheq #1=#2=#3\\end\
    {\\expandafter\\let\\expandafter\\endempheq\\csname end#1\\endcsname\
    \\if.#2.\\csname#1\\endcsname\
    \\else\\csname#1\\endcsname{#2}\\fi}");
});
