use latexml_package::prelude::*;

LoadDefinitions!({
  RequirePackage!("ifpdf");
  RawTeX!("\\RequirePackage[utf8]{inputenc}");
  RequirePackage!("CJK");
  RequirePackage!("fontenc");
  // CJKutf8.sty:707-708 provides hyperref's `\pdfstringdefPreHook` when
  // hyperref is absent; zhnumber.sty:63-64 appends to it at
  // `package/CJKutf8/after` with `\g@addto@macro` (now latex.ltx's real
  // macro, which requires its target to exist).
  RawTeX!(r"\ifx\pdfstringdefPreHook\undefined\def\pdfstringdefPreHook{}\fi");
});
