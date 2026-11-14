//! Raw packages poke graphicx/graphics INTERNALS our bindings reimplement
//! around (`wisdom_latexml_reimpl_internal_name_mismatch` shape): hvfloat
//! calls `\Gin@boolkey{true}{iso}` (hvfloat.sty L411) which drives the
//! `\newif\ifGin@iso` from graphics.sty L579 via graphicx.sty L137's
//! two-arg csname dispatcher. Sweep-11 cluster: `\Gin@boolkey` 34 docs +
//! `\Gin@draftfalse` 9 (bohr, pagelayout, …). The binding must carry the
//! real internal names, faithfully ported from the sources.

const TEX: &str = "\\documentclass{article}\n\
    \\usepackage{graphicx}\n\
    \\makeatletter\n\
    \\begin{document}\n\
    \\Gin@boolkey{true}{iso}\\ifGin@iso ISOK\\else ISNO\\fi\n\
    \\Gin@boolkey{}{clip}\\ifGin@clip CLOK\\else CLNO\\fi\n\
    \\Gin@draftfalse\\ifGin@draft DRNO\\else DROK\\fi\n\
    \\end{document}\n";

#[test]
fn gin_internal_names_defined() {
  let (stderr, xml) = super::convert(TEX, false);
  assert!(
    !stderr.contains("Error:"),
    "Gin@ internals must digest cleanly:\n{stderr}"
  );
  assert!(
    xml.contains("ISOK") && xml.contains("CLOK") && xml.contains("DROK"),
    "boolkey must flip the real newifs (empty #1 = true per graphicx.sty L137):\n{xml}",
  );
}
