//! filehook.sty — hooks for input files
//! Perl: filehook.sty.ltxml
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl filehook.sty.ltxml L18-20: this comparison can't reliably work
  // with latexml's subroutine macro definitions, so default it to *true*
  // to avoid needless warnings. Perl adds `locked => 1` — critical here
  // because the very next line (InputDefinitions) pulls in the raw
  // filehook.sty which itself defines \filehook@cmp; without the lock,
  // the raw-sty redefinition replaces our always-true stub.
  DefMacro!("\\filehook@cmp{}{}", "\\@firstoftwo", locked => true);

  // The raw filehook.sty:27 is a 1-line dispatcher:
  //   \@ifl@t@r\fmtversion{2020/10/01}{\RequirePackage{filehook-2020}}
  //                                   {\RequirePackage{filehook-2019}}
  // and ALL the hook internals (`\filehook@every@atbegin`,
  // `\filehook@prefixwarg`, `\filehook@appendwarg`, `\filehook@every@atend`
  // — used by raw currfile.sty:68-73 — plus the `\AtBeginOfPackageFile`
  // family) live in the versioned sub-file. Without this registration
  // the version-suffix fallback (Perl Package.pm:2118-2121, mirrored in
  // `content.rs` Step 3) strips `filehook-2020` → `filehook`, re-enters
  // this binding (`Warning:recursion`) and the sub-file is never loaded,
  // leaving every hook undefined. Perl shares that misfire
  // (`Info:fallback:filehook-2020`). Registering the sub-files as
  // INTERPRETABLE_SOURCES (Perl FindFile_aux L2107) makes the raw load
  // find them directly — the same shape xparse.sty.ltxml uses for its
  // `xparse-2018-04-12.sty` rollback file. Witnesses: TL doc corpus
  // currfile/currfile (`\ifcurrfilename` swallowed the document),
  // pythontex/pythontex (loads currfile); arXiv 2405.18977, 2406.01136,
  // 2406.01832 (pbalance, formerly served by no-op hook stubs here).
  AssignMapping!("INTERPRETABLE_SOURCES", "filehook-2020.sty" => 1);
  AssignMapping!("INTERPRETABLE_SOURCES", "filehook-2019.sty" => 1);

  InputDefinitions!("filehook", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
