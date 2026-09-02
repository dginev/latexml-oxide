//! delarray.sty — array delimiter package.
//!
//! delarray.sty (D. Carlisle, 1991-1994, tools bundle) lets `array` take a
//! delimiter pair around its column spec: `\begin{array}({cc})…\end{array}`,
//! `\begin{array}[t]\{{lL}.`, `|{cc}|`. It hooks the kernel entry chain
//! (delarray.sty:43-58): `\@tabarray` → `\@@array[pos]` peeks with
//! `\@ifnextchar\bgroup` — a brace means the plain `\@array[pos]{cols}`,
//! anything else is `\@del@array[pos]<left>{cols}<right>`, which sets
//! `\left<left>` … `\right<right>` around the array (`c` position; `t`/`b`
//! lower a `\vcenter`ed box by the same visual amount).
//!
//! Perl LaTeXML has no `delarray.sty.ltxml`; both engines replace `\array`
//! with their own `\array[]{}` (Perl latex_constructs.pool.ltxml:3755, Rust
//! latex_constructs.rs `\array[]{}`) that reads the template as `#2` and never
//! routes through `\@tabarray`, so the raw hook is dead code and the delimiter
//! `(` is read as the column spec — every `&` then reports "Extra alignment
//! tab" (memoir manual: 33 errors from memoir.cls:5468's
//! `\RequirePackage{delarray}`; SHARED with Perl). The binding re-enters our
//! `\array` machinery with the same peek and wraps the alignment in
//! `\left…\right` (the `t`/`b` lowering is presentational).
//!
//! Earlier witnesses (stub era, raw `\@@array` clobbering ours): canvas-3
//! 0809.4328, 0810.2088, 0810.2091, 0811.2514, 0811.4484, 0812.1967,
//! 0901.2107, 0901.3167.
use crate::prelude::*;

LoadDefinitions!({
  RequirePackage!("array");
  // delarray.sty:43-44 — `\@tabarray`/`\@@array[pos]` peek for a brace.
  DefMacro!(
    "\\array",
    r"\@ifnextchar[{\lx@delarray@pos}{\lx@delarray@pos[c]}"
  );
  DefMacro!(
    "\\lx@delarray@pos[]",
    r"\@ifnextchar\bgroup{\let\@arrayright\relax\lx@delarray@plain[#1]}{\lx@delarray@del[#1]}"
  );
  DefMacro!(
    "\\lx@delarray@plain[]{}",
    r"\@array@bindings[#1]{#2}\@@array[#1]{#2}\lx@begin@alignment"
  );
  // delarray.sty:45-58 `\@del@array[pos]<left>{cols}<right>`.
  DefMacro!(
    "\\lx@delarray@del[] Token {} Token",
    r"\def\@arrayright{\right#4}\left#2\lx@delarray@plain[#1]{#3}"
  );
  DefMacro!(
    "\\endarray",
    None,
    r"\lx@end@alignment\@end@array\@arrayright"
  );
  DefMacro!("\\@arrayright", None, r"\relax");
});
