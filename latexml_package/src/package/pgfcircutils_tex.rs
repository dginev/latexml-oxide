use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: pgfcircutils.tex.ltxml (44 lines)
  // Custom \pgf@circ@stripdecimals that handles missing decimal points.
  // The Perl version handles the case where there is no dot by manually scanning.
  // We use a two-step TeX approach: first append a dot (ensuring one exists),
  // then split on the dot and keep only the integer part.
  RawTeX!(r"\def\pgf@circ@stripdecimals#1\pgf@nil{\lx@pgf@stripdecimals#1.\pgf@nil}");
  RawTeX!(r"\def\lx@pgf@stripdecimals#1.#2\pgf@nil{#1}");

  InputDefinitions!("pgfcircutils", noltxml => true, extension => Some(Cow::Borrowed("tex")));
});
