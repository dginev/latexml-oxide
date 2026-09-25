//! (1) A document-position `\input{<name>.sty}` with no binding reads the
//! raw file as CONTENT under the current catcodes — real TeX's `\input`
//! (batch 7; re-read on every `\input`, batch 52). Perl (Package.pm:2289-2302)
//! instead loads it as definitions (`@`=letter, `[cat:11]`, text
//! suppressed) and SKIPS a second `\input`/`\DocInput` of the same file —
//! which is how it dodges doc.sty's `\CharacterTable` self-check and also
//! why it never typesets a `.sty`'s documentation body (frankenstein bundle;
//! the content route runs those bodies and still errors — PLANS P66).
//! (2) biblatex's `style=`/`bibstyle=`/`citestyle=` options load the raw
//! `.bbx`/`.cbx` style files (biblatex.sty L2256/L11428), whose
//! `\newtoggle`s etc. were undefined corpus-wide (windycity,
//! biblatex-ext/-fiwi/-sbl).

#[test]
fn document_body_sty_input_is_content_catcodes() {
  let (_stderr, xml) = super::convert_files(
    "\\documentclass{article}\n\\begin{document}\n\
       \\input{vguardcat.sty}\n[cat:\\guardcat]\n\\end{document}\n",
    &[("vguardcat.sty", "\\edef\\guardcat{\\the\\catcode`\\@}\n")],
  );
  assert!(
    xml.contains("[cat:12]"),
    "document-body \\input{{x.sty}} must read at current catcodes (@=12), got:\n{xml}"
  );
}

#[test]
fn biblatex_style_option_loads_bbx() {
  let (stderr, xml) = super::convert_files(
    "\\documentclass{article}\n\
       \\usepackage[style=lxguardstyle]{biblatex}\n\
       \\begin{document}\n\
       \\iftoggle{lxguardtoggle}{[BBX-LOADED]}{[BBX-FALSE]}\n\
       \\end{document}\n",
    &[(
      "lxguardstyle.bbx",
      "\\newtoggle{lxguardtoggle}\\toggletrue{lxguardtoggle}\n\
         \\DeclareBibliographyOption[boolean]{lxguardopt}[true]{}\n",
    )],
  );
  assert!(
    !stderr.contains("Error:"),
    "style-file load must digest cleanly:\n{stderr}"
  );
  assert!(
    xml.contains("[BBX-LOADED]"),
    ".bbx toggle not allocated — style file not loaded:\n{xml}"
  );
}
