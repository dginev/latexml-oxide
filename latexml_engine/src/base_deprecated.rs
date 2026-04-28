//! Base_Deprecated.pool.ltxml — Deprecation aliases for old macro names.
//!
//! These redirect old `\@FOO` names to new `\lx@foo` names. Perl uses
//! `\lx@DEPRECATE` to emit a one-time warning then redirect; Rust
//! currently elides the warning and just redirects (the destination
//! macro is the live name).
//!
//! Order mirrors Perl `Base_Deprecated.pool.ltxml` L29-220 1:1.
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl L29: \lx@DEPRECATE was the warn-once helper. In Rust we just
  // no-op since all deprecation chains have already been rewritten
  // in-place to redirect to the new \lx@-prefixed name.
  DefMacro!("\\lx@DEPRECATE{}{}", None);

  // Perl L33-47
  DefMacro!("\\@ERROR", "\\lx@ERROR");
  DefMacro!("\\@@eqno", "\\lx@eqno");
  DefMacro!("\\LTX@nonumber", "\\lx@equation@nonumber");
  DefMacro!("\\LTX@newpage", "\\lx@newpage");
  DefMacro!("\\normal@par", "\\lx@normal@par");
  DefMacro!("\\inner@par", "\\lx@normal@par");

  // Perl L50-54
  DefMacro!("\\hidden@bgroup", "\\lx@hidden@bgroup");
  DefMacro!("\\hidden@egroup", "\\lx@hidden@egroup");
  DefMacro!("\\right@hidden@egroup", "\\lx@hidden@egroup@right");

  // Perl L57-63 (note: Perl L63 has typo `crc` for the macro NAME
  // even though the body redirects to `\lx@hidden@crcr`; Rust mirrors
  // that — the deprecated CS is `\hidden@crc`).
  DefMacro!("\\hidden@align", "\\lx@hidden@align");
  DefMacro!("\\hidden@noalign", "\\lx@hidden@noalign");
  DefMacro!("\\hidden@cr", "\\lx@hidden@cr");
  DefMacro!("\\hidden@crc", "\\lx@hidden@crcr");

  // Perl L65-72
  DefMacro!("\\@start@alignment", "\\lx@start@alignment");
  DefMacro!("\\@finish@alignment", "\\lx@finish@alignment");
  DefMacro!("\\@close@alignment", "\\lx@close@alignment");
  DefMacro!("\\if@in@alignment", "\\if@in@lx@alignment ");

  // Perl L75-90
  DefMacro!("\\@alignment@hline", "\\lx@alignment@hline");
  DefMacro!("\\@alignment@newline", "\\lx@alignment@newline");
  DefMacro!("\\@alignment@newline@noskip", "\\lx@alignment@newline@noskip");
  DefMacro!("\\@alignment@newline@marker", "\\lx@alignment@newline@marker");
  DefMacro!("\\@alignment@newline@markertall", "\\lx@alignment@newline@markertall");
  DefMacro!("\\@alignment@column", "\\lx@alignment@column");
  DefMacro!("\\@alignment@ncolumns", "\\lx@alignment@ncolumns");
  DefMacro!("\\@alignment@bindings", "\\lx@alignment@bindings");

  // Perl L92-99
  DefMacro!("\\@row@before", "\\lx@alignment@row@before");
  DefMacro!("\\@row@after", "\\lx@alignment@row@after");
  DefMacro!("\\@column@before", "\\lx@alignment@column@before");
  DefMacro!("\\@column@after", "\\lx@alignment@column@after");

  // Perl L101-104
  DefMacro!("\\@tabular@begin@heading", "\\lx@alignment@begin@heading");
  DefMacro!("\\@tabular@end@heading", "\\lx@alignment@end@heading");

  // Perl L106-107
  DefMacro!("\\@multicolumn", "\\lx@alignment@multicolumn");

  // Perl L109-118
  DefMacro!("\\@dollar@in@normalmode", "\\lx@dollar@default");
  DefMacro!("\\@dollar@in@mathmode", "\\lx@dollar@in@mathmode");
  DefMacro!("\\@dollar@in@textmode", "\\lx@dollar@default");
  DefMacro!("\\lx@dollar@in@normalmode", "\\lx@dollar@default");
  DefMacro!("\\lx@dollar@in@textmode", "\\lx@dollar@default");

  // Perl L120-127
  DefMacro!("\\math@underline", "\\lx@math@underline");
  DefMacro!("\\text@underline", "\\lx@text@underline");
  DefMacro!("\\math@overleftarrow", "\\lx@math@overleftarrow");
  DefMacro!("\\math@overrightarrow", "\\lx@math@overrightarrow");

  // Perl L129-136
  DefMacro!("\\@@BEGININLINEMATH", "\\lx@begin@inline@math");
  DefMacro!("\\@@ENDINLINEMATH", "\\lx@end@inline@math");
  DefMacro!("\\@@BEGINDISPLAYMATH", "\\lx@begin@display@math");
  DefMacro!("\\@@ENDDISPLAYMATH", "\\lx@end@display@math");

  // Perl L138-141
  DefMacro!("\\@@BEGININLINETEXT", "\\lx@begin@inmath@text");
  DefMacro!("\\@@ENDINLINETEXT", "\\lx@end@inmath@text");

  // Perl L143-150
  DefMacro!("\\@@FLOATINGSUBSCRIPT", "\\lx@floating@subscript");
  DefMacro!("\\@@FLOATINGSUPERSCRIPT", "\\lx@floating@superscript");
  DefMacro!("\\@@POSTSUBSCRIPT", "\\lx@post@subscript");
  DefMacro!("\\@@POSTSUPERSCRIPT", "\\lx@post@superscript");

  // Perl L152-153
  DefMacro!("\\@ASSERT@MEANING", "\\lx@assert@meaning");

  // Perl L161-180 — older deprecations (XMath wrappers)
  DefMacro!("\\FCN{}", "\\lx@wrap[role=FUNCTION]{#1}");
  DefMacro!("\\ROLE{}{}", "\\lx@wrap[role={#1}]{#2}");
  DefMacro!("\\@SYMBOL{}", "\\lx@wrap[role=ID]{#1}");
  DefMacro!("\\@CSYMBOL{}", "\\lx@symbol[meaning={#1}]{}");
  DefMacro!("\\@APPLY{}", "\\lx@apply[]{#1}{}");
  DefMacro!("\\@MAYBEAPPLY{}{}", "\\ifx.#2.#1\\else\\lx@apply{#1}{#2}\\fi");
  DefMacro!("\\@WRAP{}", "\\lx@wrap[]{#1}");
  DefMacro!("\\@TOKEN{}", "\\lx@symbol[name={#1}]{}");
  DefMacro!("\\@SUPERSCRIPT{}{}", "\\ifx.#2.#1\\else\\lx@superscript[]{#1}{#2}\\fi");
  DefMacro!("\\@SUBSCRIPT{}{}", "\\ifx.#2.#1\\else\\lx@subscript[]{#1}{#2}\\fi");

  // Perl L190-205 — XMath padded/dual/arg/ref + invisible operators
  DefMacro!("\\@PADDED", "\\lx@padded");
  DefMacro!("\\DUAL", "\\lx@dual");
  DefMacro!("\\@XMArg", "\\lx@xmarg");
  DefMacro!("\\@XMRef", "\\lx@xmref");
  DefMacro!("\\@APPLYFUNCTION", "\\lx@ApplyFunction");
  DefMacro!("\\@INVISIBLETIMES", "\\lx@InvisibleTimes");
  DefMacro!("\\@INVISIBLECOMMA", "\\lx@InvisibleComma");
  DefMacro!("\\@INVISIBLEPLUS", "\\lx@InvisiblePlus");

  // Perl L207-210
  DefMacro!("\\@@endash", "\\lx@endash");
  DefMacro!("\\@@emdash", "\\lx@emdash");

  // Perl L212-217
  DefMacro!("\\ltx@leftline", "\\lx@leftline");
  DefMacro!("\\ltx@rightline", "\\lx@rightline");
  DefMacro!("\\ltx@centerline", "\\lx@centerline");

  // Perl L219-220
  DefMacro!("\\ltx@input", "\\lx@latex@input");
});
