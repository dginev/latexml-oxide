//! OXIDIZED_DESIGN #166: `\@filelist` entries carry kernel catcodes —
//! alphabetic chars as LETTER (`\string@makeletter`, latex.ltx L1784) —
//! so source-level delimited parses over the list match. hep-font.sty's
//! `\def\hepfont@get@class#1.cls#2\relax` + `\expandafter…\@filelist`
//! idiom got an empty #1 under all-OTHER tokens, and the mis-split
//! desynced conditional bookkeeping (13-bundle `expected:\fi` cluster).

const TEX: &str = "\\documentclass{article}\n\
    \\makeatletter\n\
    \\def\\get#1.cls#2\\relax{\\def\\res{#1}}\n\
    \\expandafter\\get\\@filelist\\relax\n\
    \\makeatother\n\
    \\begin{document}\n\
    res=[\\res]\n\
    \\end{document}\n";

#[test]
fn delimited_parse_of_filelist_matches() {
  let (_stderr, xml) = super::convert(TEX, false);
  // #1 = everything before the first ".cls" — must contain the class name,
  // not be empty.
  assert!(
    xml.contains("res=[") && xml.contains("article]"),
    "delimited .cls parse over \\@filelist must capture the prefix:\n{xml}",
  );
}
