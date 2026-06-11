//! mathenv.sty — TWO different packages ship under this name; the binding
//! raw-loads the self-contained one and no-ops the mdwtools one.
//!
//! * **Bosisio's "Extended math environments"** (Francesco Bosisio, 1997) — self-contained, defines
//!   `\newenvironment{EqSystem}`/`{Equation}` and an active-`&` system-of-equations machinery. NOT
//!   in TeX Live (papers ship it locally). Perl LaTeXML has no `mathenv.sty.ltxml`, so under the
//!   ar5iv gate (`INCLUDE_STYLES=true`) it RAW-LOADS this file and converts cleanly (witness
//!   1710.07800 → 92 errors here without the raw-load, 0 with it).
//!
//! * **Mark Wooding's mdwtools `mathenv`** — opens with `\RequirePackage{mdwtab}` (a full `tabular`
//!   rewrite). Our `mdwtab` stub can't supply its `\tab@*` internals, so raw-loading Wooding's
//!   mathenv cascades (`\tab@setstate` / `\tab@preamble` / … undefined → ~42 errors). A no-op stub
//!   avoids that (witness 0910.3293). The principled fix for THIS variant is a working `mdwtab`
//!   port; until then we keep the stub for it.
//!
//! Distinguish the two by the `mdwtab` requirement (Bosisio's file never
//! mentions it; Wooding's does `\RequirePackage{mdwtab}`): raw-load the
//! self-contained Bosisio variant like Perl, no-op the mdwtools one. Both then
//! match Perl. (Task #273 — prefer raw-load over stubs; here the stub is kept
//! ONLY for the variant whose dependency we can't yet raw-load.)
use crate::prelude::*;

LoadDefinitions!({
  let requires_mdwtab = find_file(
    "mathenv",
    Some(FindFileOptions {
      ext_type: Some(Cow::Borrowed("sty")),
      ..Default::default()
    }),
  )
  .and_then(|p| std::fs::read_to_string(&p).ok())
  .is_some_and(|content| content.contains("mdwtab"));
  if !requires_mdwtab {
    // Bosisio's self-contained mathenv → raw-load it, matching Perl.
    InputDefinitions!("mathenv", noltxml => true, extension => Some(Cow::Borrowed("sty")));
  }
  // else: Wooding's mdwtools mathenv → no-op (avoid the broken-mdwtab cascade).
});
