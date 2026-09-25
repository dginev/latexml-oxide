//! OXIDIZED_DESIGN #164: the loader must record `\@raw@opt@<name>.<ext>` —
//! the ONLY thing the modern kernel's `\ProcessKeyOptions` reads
//! (latex.ltx L19398). Without it every ltkeys key-option package silently
//! drops its load-time options (Perl 0.8.8 shares the miss).

const STY: &str = "\\NeedsTeXFormat{LaTeX2e}\n\
    \\ProvidesPackage{pkoguard}\n\
    \\RequirePackage{expl3}\n\
    \\ExplSyntaxOn\n\
    \\keys_define:nn {pkoguard}\n\
    \x20 {\n\
    \x20   flag .bool_set:N = \\l_pkoguard_flag_bool ,\n\
    \x20   flag .default:n  = {true} ,\n\
    \x20 }\n\
    \\ProcessKeyOptions [pkoguard]\n\
    \\bool_if:NT \\l_pkoguard_flag_bool { \\def\\FLAGON{yes} }\n\
    \\ExplSyntaxOff\n";
const TEX: &str = "\\documentclass{article}\n\
    \\usepackage[flag]{pkoguard}\n\
    \\begin{document}\n\
    flag=\\ifdefined\\FLAGON ON\\else OFF\\fi\n\
    \\end{document}\n";

#[test]
fn key_option_reaches_process_key_options() {
  let (stderr, xml) = super::convert_files_with(
    TEX,
    &[("pkoguard.sty", STY)],
    Some("[rawstyles]latexml.sty"),
  );
  assert!(
    xml.contains("flag=ON"),
    "\\ProcessKeyOptions must see the [flag] load option:\n{xml}\n{stderr}",
  );
}
