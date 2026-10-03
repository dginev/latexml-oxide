//! clefval.sty — key/value pairs: `\TheKey{key}{value}` defines, `\TheValue{key}` prints.
//!
//! `\TheKey` only writes `\newkey{key}{value}` to the `.aux` (clefval.sty `\TheKey`, `\@protected@write`), and
//! `\V@key` is defined when the NEXT run reads it back at `\begin{document}` (`\newkey` = `\@newk@ey V`, a
//! `\global\@namedef`); a single pass prints "[?? key ??]" with a warning, in Perl too (clefval/example-utf8 recall
//! 78.8 %). Here `\TheKey` also defines the value at once, as the read-back would, so a value used after its key
//! prints; one used before it still waits for a second run (OXIDIZED_DESIGN_DIVERGENCES #425). Repro
//! singletons/clefval_value_after_its_key; guard `perfect_kernel_batch60::clefval_value_after_its_key`.
use latexml_package::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  InputDefinitions!("clefval", noltxml => true, extension => Some(Cow::Borrowed("sty")));
  RawTeX!(r"\let\lx@clefval@TheKey\TheKey");
  RawTeX!(r"\def\TheKey#1#2{\lx@clefval@TheKey{#1}{#2}\global\@namedef{V@#1}{#2}}");
});
