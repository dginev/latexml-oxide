use latexml_package::prelude::*;

LoadDefinitions!({
  // catchfile.sty provides \CatchFileDef and \CatchFileEdef for reading file contents.
  // The Perl binding uses Input() and DefMacroI() which require runtime closures.
  // We stub both as no-ops for now — the files they try to read are typically unavailable.
  Warn!(
    "missing_file",
    "catchfile.sty",
    "catchfile.sty is only minimally stubbed and will not be interpreted raw."
  );
});
