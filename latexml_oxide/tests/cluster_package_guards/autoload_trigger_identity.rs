//! Batch-27 contract: `\@ifundefined{X}` must treat X as DEFINED when the
//! kernel dump gave it a real definition, even though X is also a
//! def_autoload trigger whose package never loaded. The `:autoload` flag
//! alone goes stale for `\ProvidesExplPackage`/`\ProvidesExplClass`/
//! `\ExplSyntaxOn` (triggers pre-dump AND real latex.ltx macros); the
//! reader now also requires the CS to still HOLD the trigger definition
//! (Rc identity snapshot). Witness: updatemarks-nums.sty's single-branch
//! `\@ifundefined{ProvidesExplPackage}{\RequirePackage{expl3}}` swallowed
//! the following `\ProvidesExplPackage` as the phantom second branch —
//! expl3 catcodes never enabled, 89-error cascade (21-doc `unexpected:_`
//! cluster; updatemarks 101→2).

#[test]
fn ifundefined_sees_dump_definition_over_stale_trigger() {
  let (stderr, _xml) = super::convert_files(
    "\\documentclass{article}\n\\usepackage{trigid}\n\\begin{document}\nx\n\\end{document}\n",
    &[(
      "trigid.sty",
      "\\@ifundefined{ProvidesExplPackage}{\\RequirePackage{expl3}}\n\
         \\ProvidesExplPackage{trigid}{2024/02/19}{v0.1}{x}\n\
         \\tl_new:N \\l__trigid_tmpa_tl\n\\ExplSyntaxOff\n",
    )],
  );
  assert!(
    !stderr.contains("Error:"),
    "stale autoload trigger masked the dump's \\ProvidesExplPackage:\n{stderr}"
  );
}
