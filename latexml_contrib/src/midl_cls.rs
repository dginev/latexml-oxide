//! Stub for midl.cls (MIDL — Medical Imaging with Deep Learning), a jmlr
//! derivative. midl.cls does `\LoadClass[pmlr]{jmlr}` and defines its author wrapper
//! in the cls body: `\newcommand{\midlauthor}[1]{\author{#1}}` (midl.cls:57).
//! As an unbound class Rust takes the OmniBus fallback, which dep-scans the cls
//! (so jmlr loads) but does NOT execute the cls body, leaving `\midlauthor`
//! undefined; the undefined wrapper then fails to consume its `{...}`, so the
//! jmlr binding's `\addr Until:\lx@jmlr@endaddr` scanner (jmlr_cls.rs) runs to
//! EOF and aborts the whole document (`Fatal:Mouth:EoF`). Define the wrapper
//! (routing through jmlr's structured-author path, which lays the sentinel) and
//! the few body helpers so the author block is clean. Witness 2605.00538.
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("jmlr");
  DefMacro!("\\midlauthor{}", "\\author{#1}");
  // midl.cls body author helpers (midl.cls:50-57,106).
  DefMacro!("\\midljointauthortext{}", "\\nametag{\\thanks{#1}}");
  def_macro_noop("\\midlotherjointauthor")?;
  DefMacro!("\\midlacknowledgments{}", "\\acks{#1}");
  DefMacro!("\\orcid{}", "");
  DefMacro!("\\jmlrpages{}", ""); // jmlr.cls:559 is 1-arg; swallow it (no body leak)
});
