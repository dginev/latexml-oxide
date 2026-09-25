//! OXIDIZED_DESIGN #169 (surpass-Perl, user-approved 2026-08-31): the
//! raw-TeX loader must NOT clobber `\ver@<file>` with `\fmtversion` when
//! the file's own `\ProvidesPackage` already recorded its version — real
//! LaTeX keeps the declared string, and date guards (`\GetFileInfo`,
//! toptesi.cls L44-73's version comparison) read it. Perl shares the
//! clobber (Package.pm L2393). Witness cluster: all 12 toptesi manuals
//! abort with "the sty file you are using has a date of <empty>".

const STY: &str = "\\ProvidesPackage{vguard}[2001/01/01 v9.9 Version guard fixture]\n\
    \\endinput\n";
const TEX: &str = "\\documentclass{article}\n\
    \\usepackage{vguard}\n\
    \\begin{document}\n\
    V[\\expandafter\\meaning\\csname ver@vguard.sty\\endcsname]\n\
    \\end{document}\n";

#[test]
fn provides_package_version_not_clobbered() {
  let (_stderr, xml) =
    super::convert_files_with(TEX, &[("vguard.sty", STY)], Some("[rawstyles]latexml.sty"));
  assert!(
    xml.contains("2001/01/01 v9.9 Version guard fixture"),
    "\\ver@vguard.sty must keep the ProvidesPackage string:\n{xml}",
  );
}
