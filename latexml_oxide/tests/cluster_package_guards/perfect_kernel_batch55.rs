//! Red/green guards for perfect-kernel batch 55 (wave-5 root-causer
//! reports over the CJK and LuaTeX oracle-clean doc residuals: circledtext,
//! suanpan-l3, pascaltriangle, tikz-bagua, joinbox, jnuexam).
use super::perfect_kernel_batch46::{convert, error_count};

/// XeTeX/LuaTeX \Uchar <number> expands to a character token (Catcode 10 for space, 12 otherwise).
/// expl3 binds \tex_Uchar:D to \Uchar.
#[test]
fn uchar_primitive_and_expl3_alias() {
  let tex = r"\documentclass{article}
\usepackage{expl3}
\begin{document}
[\Uchar 65][\Uchar 32][\ExplSyntaxOn\tex_Uchar:D 66\ExplSyntaxOff]
\end{document}
";
  // `\Uchar` is a Unicode-engine primitive: only the luatex profile has it.
  let (stderr, xml) = super::perfect_kernel_batch46::convert_with(tex, Some("[luatex]latexml.sty"));
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("[A][ ][B]"), "{xml}");
}

/// xunicode.sty exports \UTFencname and macro declarations used by xunicode-addon.sty.
#[test]
fn xunicode_interface_for_addon() {
  let tex = r"\documentclass{article}
\usepackage{xunicode}
\DeclareUTFcharacter[\UTFencname]{x00A0}{\nobreakspace}
\ReloadXunicode
\begin{document}
UTF:\UTFencname
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("UTF:TU"), "{xml}");
}

/// \disable@package@load{pkg}{action} stores action in \@pkg-disable@pkg.ext and suppresses package load.
#[test]
fn kernel_disable_package_load() {
  let tex = r"\documentclass{article}
\makeatletter
\disable@package@load{nonexistentpkg}{\def\customdisabled{DISABLED}}
\makeatother
\usepackage{nonexistentpkg}
\begin{document}
[\customdisabled]
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("[DISABLED]"), "{xml}");
}

/// fontspec family setters and ctex-engine-luatex lua commands
#[test]
fn fontspec_family_setters_and_ctex_luatex() {
  let tex = r"\documentclass{article}
\usepackage{expl3}
\ExplSyntaxOn
\tl_new:N \l_test_family_tl
\fontspec_gset_family:Nnn \l_test_family_tl {} {SimSun}
\fontspec_set_family:Nnn \l_test_family_tl {} {SimHei}
\ctex_ltj_add_kyenc:n { EU1 }
\ctex_ltj_zero_globaldefs:
\ExplSyntaxOff
\begin{document}
CJK fontspec ok
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("CJK fontspec ok"), "{xml}");
}
