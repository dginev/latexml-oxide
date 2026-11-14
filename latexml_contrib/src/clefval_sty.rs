//! clefval.sty — key/value pairs: `\TheKey{key}{value}` defines, `\TheValue{key}` prints.
//!
//! `\TheKey` only writes `\newkey{key}{value}` to the `.aux` (clefval.sty `\TheKey`, `\@protected@write`), and
//! `\V@key` is defined when the NEXT run reads it back at `\begin{document}` (`\newkey` = `\@newk@ey V`, a
//! `\global\@namedef`); a single pass prints "[?? key ??]" with a warning, in Perl too (clefval/example-utf8 recall
//! 78.8 %). Here `\TheKey` also defines the value at once, as the read-back would, so a value used after its key
//! prints; one used before it still waits for a second run, and a key defined twice gives each value from its
//! definition on, where the read-back gives the last everywhere (OXIDIZED_DESIGN_DIVERGENCES #425). Repro
//! singletons/clefval_value_after_its_key; guard `perfect_kernel_batch60::clefval_value_after_its_key`.
use latexml_package::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  InputDefinitions!("clefval", noltxml => true, extension => Some(Cow::Borrowed("sty")));
  RawTeX!(r"\let\lx@clefval@TheKey\TheKey");
  // A key defined twice warns as `\@newk@ey` does when it reads the `.aux` back. The spaces on both sides of
  // `\TheKey` stay, as in TeX: its `\@esphack` meets the group's `}`, not the space (`Charlie \TheKey{b}{B} delta` is
  // 60.03pt in pdflatex, two interword glues).
  // The value is expanded as the `.aux` write expands it (`\@protected@write`, an `\edef` under
  // `\@unexpandable@protect`): `\protected@edef`.
  RawTeX!(r"\def\TheKey#1#2{\@ifundefined{V@#1}{}{\@latex@warning@no@line{Key `#1' multiply defined}}%
    \protected@edef\lx@clefval@value{#2}\global\expandafter\let\csname V@#1\endcsname\lx@clefval@value
    \lx@clefval@TheKey{#1}{#2}}");
});
