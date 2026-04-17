//! Base_Deprecated.pool.ltxml — Deprecation aliases for old macro names
//! These redirect old \@FOO names to new \lx@foo names.
//! Perl: 227 lines, all simple DefMacro redirections.
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Simplified: skip deprecation warnings, just redirect.
  // Perl's \lx@DEPRECATE warns once then redirects; we just redirect.

  // Alignment
  DefMacro!("\\@start@alignment", "\\lx@start@alignment");
  DefMacro!("\\@finish@alignment", "\\lx@finish@alignment");
  DefMacro!("\\@close@alignment", "\\lx@close@alignment");
  DefMacro!("\\if@in@alignment", "\\if@in@lx@alignment ");
  DefMacro!("\\@alignment@hline", "\\lx@alignment@hline");
  DefMacro!("\\@alignment@newline", "\\lx@alignment@newline");
  DefMacro!("\\@alignment@newline@noskip", "\\lx@alignment@newline@noskip");
  DefMacro!("\\@alignment@newline@marker", "\\lx@alignment@newline@marker");
  DefMacro!("\\@alignment@newline@markertall", "\\lx@alignment@newline@markertall");
  DefMacro!("\\@alignment@column", "\\lx@alignment@column");
  DefMacro!("\\@alignment@ncolumns", "\\lx@alignment@ncolumns");
  DefMacro!("\\@alignment@bindings", "\\lx@alignment@bindings");
  DefMacro!("\\@row@before", "\\lx@alignment@row@before");
  DefMacro!("\\@row@after", "\\lx@alignment@row@after");
  DefMacro!("\\@column@before", "\\lx@alignment@column@before");
  DefMacro!("\\@column@after", "\\lx@alignment@column@after");
  DefMacro!("\\@tabular@begin@heading", "\\lx@alignment@begin@heading");
  DefMacro!("\\@tabular@end@heading", "\\lx@alignment@end@heading");
  DefMacro!("\\@multicolumn", "\\lx@alignment@multicolumn");

  // Core commands
  DefMacro!("\\@ERROR", "\\lx@ERROR");
  DefMacro!("\\@@eqno", "\\lx@eqno");
  DefMacro!("\\LTX@nonumber", "\\lx@equation@nonumber");
  DefMacro!("\\LTX@newpage", "\\lx@newpage");
  DefMacro!("\\normal@par", "\\lx@normal@par");
  DefMacro!("\\inner@par", "\\lx@normal@par");

  // Group/align hidden commands
  DefMacro!("\\hidden@bgroup", "\\lx@hidden@bgroup");
  DefMacro!("\\hidden@egroup", "\\lx@hidden@egroup");
  DefMacro!("\\right@hidden@egroup", "\\lx@hidden@egroup@right");
  DefMacro!("\\hidden@align", "\\lx@hidden@align");
  DefMacro!("\\hidden@noalign", "\\lx@hidden@noalign");
  DefMacro!("\\hidden@cr", "\\lx@hidden@cr");
  DefMacro!("\\hidden@crc", "\\lx@hidden@crcr");

  // Dollar mode
  DefMacro!("\\@dollar@in@normalmode", "\\lx@dollar@default");
  DefMacro!("\\@dollar@in@textmode", "\\lx@dollar@default");
  // Perl L115-118: \lx@-prefixed deprecation aliases for the old \@-names
  DefMacro!("\\lx@dollar@in@normalmode", "\\lx@dollar@default");
  DefMacro!("\\lx@dollar@in@textmode", "\\lx@dollar@default");
  // Perl L29: \lx@DEPRECATE was the warn-once helper. In Rust we just no-op
  // since all deprecation chains have already been rewritten in-place above.
  DefMacro!("\\lx@DEPRECATE{}{}", None);

  // Math decorations
  DefMacro!("\\math@underline", "\\lx@math@underline");
  DefMacro!("\\text@underline", "\\lx@text@underline");
  DefMacro!("\\math@overleftarrow", "\\lx@math@overleftarrow");
  DefMacro!("\\math@overrightarrow", "\\lx@math@overrightarrow");

  // Math mode transitions
  DefMacro!("\\@@BEGININLINEMATH", "\\lx@begin@inline@math");
  DefMacro!("\\@@ENDINLINEMATH", "\\lx@end@inline@math");
  DefMacro!("\\@@BEGINDISPLAYMATH", "\\lx@begin@display@math");
  DefMacro!("\\@@ENDDISPLAYMATH", "\\lx@end@display@math");
  DefMacro!("\\@@BEGININLINETEXT", "\\lx@begin@inmath@text");
  DefMacro!("\\@@ENDINLINETEXT", "\\lx@end@inmath@text");

  // Script operators
  DefMacro!("\\@@FLOATINGSUBSCRIPT", "\\lx@floating@subscript");
  DefMacro!("\\@@FLOATINGSUPERSCRIPT", "\\lx@floating@superscript");
  DefMacro!("\\@@POSTSUBSCRIPT", "\\lx@post@subscript");
  DefMacro!("\\@@POSTSUPERSCRIPT", "\\lx@post@superscript");
  DefMacro!("\\@ASSERT@MEANING", "\\lx@assert@meaning");

  // XMath wrappers
  DefMacro!("\\FCN{}", "\\lx@wrap[role=FUNCTION]{#1}");
  DefMacro!("\\ROLE{}{}", "\\lx@wrap[role={#1}]{#2}");
  DefMacro!("\\@SYMBOL{}", "\\lx@wrap[role=ID]{#1}");
  DefMacro!("\\@WRAP{}", "\\lx@wrap[]{#1}");
  DefMacro!("\\@PADDED", "\\lx@padded");
  DefMacro!("\\DUAL", "\\lx@dual");
  DefMacro!("\\@XMArg", "\\lx@xmarg");
  DefMacro!("\\@XMRef", "\\lx@xmref");
  DefMacro!("\\@APPLYFUNCTION", "\\lx@ApplyFunction");
  DefMacro!("\\@INVISIBLETIMES", "\\lx@InvisibleTimes");
  DefMacro!("\\@INVISIBLECOMMA", "\\lx@InvisibleComma");
  DefMacro!("\\@INVISIBLEPLUS", "\\lx@InvisiblePlus");

  // Dashes
  DefMacro!("\\@@endash", "\\lx@endash");
  DefMacro!("\\@@emdash", "\\lx@emdash");

  // Line formatting
  DefMacro!("\\ltx@leftline", "\\lx@leftline");
  DefMacro!("\\ltx@rightline", "\\lx@rightline");
  DefMacro!("\\ltx@centerline", "\\lx@centerline");

  // Input
  DefMacro!("\\ltx@input", "\\lx@latex@input");

  // Math operators (deprecated @-prefixed names)
  DefMacro!("\\@dollar@in@mathmode", "\\lx@dollar@in@mathmode");
  DefMacro!("\\@CSYMBOL{}", "\\lx@symbol[meaning={#1}]{}");
  DefMacro!("\\@MAYBEAPPLY{}{}", "\\ifx.#2.#1\\else\\lx@apply{#1}{#2}\\fi");
  DefMacro!("\\@TOKEN{}", "\\lx@symbol[name={#1}]{}");
  DefMacro!("\\@SUPERSCRIPT{}{}", "\\ifx.#2.#1\\else\\lx@superscript[]{#1}{#2}\\fi");
  DefMacro!("\\@SUBSCRIPT{}{}", "\\ifx.#2.#1\\else\\lx@subscript[]{#1}{#2}\\fi");
  // Perl L169: \@APPLY is deprecated application wrapper
  DefMacro!("\\@APPLY{}", "\\lx@apply[]{#1}{}");
});
