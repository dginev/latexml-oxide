//! OXIDIZED_DESIGN #164 (class half): the kernel records the raw
//! `\documentclass` option text in `\@raw@classoptionslist` (latex.ltx
//! L18718, first class only). Modern babel reads exactly that list for
//! global language options (babel.sty L4199) — without it
//! `[french]{article}` + babel loads nil.ldf and `\og`/`\fg` are
//! undefined. Babel isn't needed to guard the record itself.

const TEX: &str = "\\documentclass[french,11pt]{article}\n\
    \\begin{document}\n\
    raw=[\\makeatletter\\@raw@classoptionslist\\makeatother]\n\
    \\end{document}\n";

#[test]
fn documentclass_options_recorded_raw() {
  let (_stderr, xml) = super::convert(TEX, false);
  assert!(
    xml.contains("raw=[french,11pt]"),
    "\\@raw@classoptionslist must carry the raw \\documentclass options:\n{xml}",
  );
}
