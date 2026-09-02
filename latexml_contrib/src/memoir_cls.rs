use latexml_package::prelude::*;

// memoir.cls is raw-interpreted through the engine (tlp/czjphys precedent).
//
// The former stub (`LoadClass!("book")` + ~40 page-geometry no-ops, kept at
// git history e5a46e1443^) hid the real class, whose command surface is
// enormous — `\onelineskip` (memoir.cls L62), `{vplace}` (L11305),
// `\cftbeforechapterskip` (L7429), `\HUGE`, `\setsecnumdepth`,
// `\chapterstyle`, `\xpretocmd`, `\makeoddhead`, the output-stream family
// (L10965-11063, content-bearing) … — so 22 of 24 oracle-clean memoir manuals
// in the perfect-kernel corpus errored on `undefined:\<memoir-CS>`
// (witnesses: titlepages 4→0, dlfltxbmarkup 3→0, memexsupp, the dlfltxb*
// family, biblatex-oxref oxalph/oxnum/oxyear-doc). The real class raw-loads
// with zero errors and yields the correct <chapter>/<section> structure, so
// the complete class beats the stub (policy: complete support over stubs).
// Keeping a binding — rather than deleting the file — makes memoir raw-load
// under BOTH `[rawclasses]` and the default (arXiv) configuration, where a
// bindingless class would otherwise fall to the OmniBus article base.
// Perl LaTeXML ships no memoir.cls.ltxml.
LoadDefinitions!({
  // memoir.cls:8811 redefines ONLY `\endminipage` (the classic latex.ltx box
  // closer plus minipage-footnote flushing), never `\minipage`. Our minipage
  // is a native constructor pair (latex_constructs.rs `\minipage`/`\endminipage`;
  // Perl latex_constructs.pool.ltxml:4771) whose begin sets no `\@mpargs`, so
  // the raw closer would `\egroup` the native mode frame and hand the still-live
  // dump `\@iiiparbox` (latex.ltx:16309) an undefined `\@mpargs` — its `Until:[`
  // scan then swallows the NEXT `[…]` in the document (tcolorbox captures the
  // closer via `\let\endtcb@lrbox=\endminipage`, tcolorbox.sty:1118 — witness
  // biblatex-oxref/oxalph-doc: 983× `\csname bm@bicolor ,colframe = …` + Fatal
  // TooManyErrors; Perl has no `\@iiiparbox` at all and merely errors). Keep
  // the native pair paired: save the closer around the raw class load and
  // restore it. Guard: `perfect_kernel_batch54::memoir_keeps_native_endminipage`.
  Let!("\\lx@memoir@saved@endminipage", "\\endminipage");
  InputDefinitions!("memoir", noltxml => true, extension => Some(Cow::Borrowed("cls")));
  Let!("\\endminipage", "\\lx@memoir@saved@endminipage");
});
