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
  // delarray.sty:43-44 — `\@tabarray`/`\@@array[pos]` peek for a brace. Outside math the array is its own formula,
  // as the kernel `\array` is (sect10.rs `\lx@array@text@open`; memoir.cls:5468 loads delarray), its `\right`
  // inside it.
  DefMacro!(
    "\\array",
    r"\@ifnextchar[{\lx@delarray@pos}{\lx@delarray@pos[c]}"
  );
  DefMacro!(
    "\\lx@delarray@pos[]",
    r"\lx@array@text@open{#1}\@ifnextchar\bgroup{\let\@arrayright\relax\lx@delarray@plain[#1]}{\lx@delarray@del[#1]}"
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
    r"\lx@end@alignment\@end@array\@arrayright\lx@array@text@close"
  );
  DefMacro!("\\@arrayright", None, r"\relax");
  // array.sty's `\tabular` enters the same `\@tabarray` (array.sty:291, :297-310 `\@tabular` … `\@tabarray`), so delarray's
  // peek delimits a `tabular` too: `\begin{tabular}[t]\{{l}.` (1304.5109, 1106.2670). The kernel's tabular reads its
  // `[pos]{cols}` in `\lx@@tabular@args` (sect10.rs, locked as there): a brace is the plain template; else the
  // delimiters go around the tabular in a formula, as array.sty's `$`-built tabular has them (`\@arrayleft` before
  // the `\vcenter`, `\@arrayright` after it), `\endtabular` closing both. A plain tabular forgets an enclosing
  // delimited one's right delimiter and formula. Perl has no delarray binding and its `\tabular` never reaches
  // `\@tabarray`, so the delimiter is read as the template (KNOWN_PERL_ERRORS #1096, OXIDIZED_DESIGN_DIVERGENCES #858).
  // Repro alignment-bindings/delarray_delimits_a_tabular.
  DefMacro!(
    "\\lx@@tabular@args[]",
    r"\@ifnextchar\bgroup{\let\@arrayright\relax\let\lx@array@text@close\relax\lx@delarray@tabular[#1]}{\lx@delarray@tabular@del[#1]}",
    locked => true
  );
  DefMacro!(
    "\\lx@delarray@tabular[]{}",
    r"\@tabular@bindings{#2}[vattach=#1]\@@tabular[#1]{#2}\lx@begin@alignment\@tabular@before"
  );
  DefMacro!(
    "\\lx@delarray@tabular@del[] Token {} Token",
    r"\def\@arrayright{\right#4}\lx@delarray@tabular@open\left#2\lx@delarray@tabular[#1]{#3}"
  );
  // array.sty:297-301: the tabular is set in math of its own (`\leavevmode\hbox\bgroup$`), at any position — no
  // `\vcenter` outside math, so not the array's "Missing $ inserted" (`\lx@array@text@open`).
  DefMacro!(
    "\\lx@delarray@tabular@open",
    r"\ifmmode\let\lx@array@text@close\relax\else\leavevmode\lx@begin@inline@math\let\lx@array@text@close\lx@end@inline@math\fi"
  );
  DefMacro!(
    "\\endtabular",
    r"\@tabular@after\lx@end@alignment\@end@tabular\lx@raw@array@close\@arrayright\lx@array@text@close",
    locked => true
  );
});
