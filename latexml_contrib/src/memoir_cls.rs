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
  InputDefinitions!("memoir", noltxml => true, extension => Some(Cow::Borrowed("cls")));
});
