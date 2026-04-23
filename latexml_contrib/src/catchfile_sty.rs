use latexml_package::prelude::*;

LoadDefinitions!({
  // Perl: ar5iv-bindings/catchfile.sty.ltxml — DefMacro with Perl closure
  // that Input()s a file and DefMacroI()s its contents into the target CS.
  // The Rust port minimally stubs both control sequences so documents
  // that call `\CatchFileDef\target{path}{options}` don't hit undefined-CS
  // on `\CatchFileDef` itself. The target CS remains undefined (file I/O
  // deferred) — a faithful implementation would need Input() + dynamic
  // def_macro(target, contents) inside the DefPrimitive closure, which
  // is awkward because the file path is typically runtime-unavailable
  // in test fixtures.
  //
  // DP-audit kind flip (Perl DefMacro → Rust DefPrimitive) is a
  // deliberate structural adaptation: a no-op stub is more naturally
  // expressed as DefPrimitive-None than as DefMacro-empty-body because
  // the Perl body is a closure (not a template expansion). WISDOM #44
  // in reverse — gullet-level no-op vs stomach-level no-op; both
  // observationally equivalent when the body is inert.
  Warn!(
    "missing_file",
    "catchfile.sty",
    "catchfile.sty is only minimally stubbed and will not be interpreted raw."
  );
  DefPrimitive!("\\CatchFileDef DefToken {}{}", None);
  DefPrimitive!("\\CatchFileEdef DefToken {}{}", None);
});
