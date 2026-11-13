//! Surpass (user-approved shape, 2026-09-17): frontespizio's default `write`
//! mode typesets the title page in a SECOND pdflatex run on a generated
//! `\jobname-frn.tex` and re-includes it as a graphic — out of scope like
//! shell-escape, so raw-loaded (and in Perl) the environment yielded an
//! empty `<titlepage>` and all six shipped manuals lost their whole content
//! (S3 0/27). The binding forces the package's own inline route
//! (`[nowrite,infront]`, `\preparefrontpage<shape>`); the whole
//! `<titlepage>` of the standard shape is pinned. The `Matricola` row's
//! `\\[\frontcandidatesep]` = `\\[3ex]` (frontespizio.sty:296, :304) is 3ex of
//! the cell's `\frontsmallfont` (`\fontsize{9}{11}`, :293), where `\@xtabularcr`
//! evaluates it (latex.ltx:16590-16602): 12.0pt since batch 56jr reads it as the
//! `\\` expands (pdflatex's cmr9 ex gives 11.6pt); 15.99994pt before, in the
//! row-end marker's later font.

#[test]
fn standard_shape_title_page_is_typeset_inline() {
  if !latexml::util::test::kpse_has("frontespizio.sty") {
    return;
  }
  let tex = "\\documentclass[a4paper,titlepage]{book}\n\\usepackage{frontespizio}\n\\begin{document}\n\\begin{frontespizio}\n\\Universita{Padova}\n\\Facolta{Scienze Matematiche, Fisiche e Naturali}\n\\Corso[Laurea]{Matematica}\n\\Titoletto{Tesi di laurea}\n\\Titolo{Equivalenze fra categorie di moduli}\n\\Candidato[145822]{Enrico Gregorio}\n\\Relatore{Ch.mo Prof.~Adalberto Orsatti}\n\\Annoaccademico{1999-2000}\n\\end{frontespizio}\nBODY\n\\end{document}\n";
  let (stderr, xml) = super::convert(tex, true);
  assert_eq!(super::error_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "titlepage",
    &[],
    r##"<titlepage><p><text font="bold" fontsize="140%">Università degli Studi di Padova</text></p><p><rule height="1px" width="100%"/><text fontsize="120%">FACOLTÀ DI SCIENZE MATEMATICHE, FISICHE E NATURALI<break/>Corso di Laurea in Matematica</text></p><p><text font="smallcaps">Tesi di laurea</text></p><p><text font="bold" fontsize="170%">Equivalenze fra categorie di moduli</text></p><p><tabular vattach="middle"><tbody><tr><td align="left" class="ltx_nopad_r"><tabular vattach="top"><tr><td align="left" class="ltx_nopad_r">Candidato:</td></tr><tr><td align="left" class="ltx_nopad_r"><text font="bold" fontsize="120%">Enrico Gregorio</text></td></tr><tr><td align="left" class="ltx_nopad_r" cssstyle="padding-bottom: 12.0pt"><text font="bold" fontsize="90%">Matricola 145822</text></td></tr></tabular></td><td align="left" class="ltx_nopad_l ltx_nopad_r"><tabular vattach="top"><tr><td align="left" class="ltx_nopad_r">Relatore:</td></tr><tr><td align="left" class="ltx_nopad_r"><text font="bold" fontsize="120%">Ch.mo Prof. Adalberto Orsatti</text></td></tr></tabular></td></tr></tbody></tabular></p><p><rule height="1px" width="100%"/><text font="bold" fontsize="120%">Anno Accademico 1999-2000</text></p></titlepage>"##,
  );
}
/// `Preambolo*` (preamble material for the external title-page document,
/// frontespizio.sty:186-187) is executed inline: its `\newcommand` is live
/// for `\Titolo` (examplec's `\compring`), its package loaders and that
/// document's `\geometry` page layout (toptesi-example-con-frontespizio;
/// undefined inline, its argument leaked as body text) are gobbled.
#[test]
fn preambolo_material_is_executed_inline() {
  if !latexml::util::test::kpse_has("frontespizio.sty") {
    return;
  }
  let tex = "\\documentclass[a4paper,titlepage]{book}\n\\usepackage{frontespizio}\n\\begin{document}\n\\begin{frontespizio}\n\\begin{Preambolo*}\n  \\usepackage{fourier}\n  \\geometry{a4paper, left=35mm, right=35mm}\n  \\newcommand{\\compring}{anelli compatti}\n\\end{Preambolo*}\n\\Universita{Bologna}\n\\Dipartimento{Matematica}\n\\Corso[Dottorato di Ricerca]{Matematica}\n\\Titolo{Sugli \\compring}\n\\Candidato{Nome Cognome}\n\\Relatore{Prof.~Relatore}\n\\Annoaccademico{2000-2001}\n\\end{frontespizio}\nBODY\n\\end{document}\n";
  let (stderr, xml) = super::convert(tex, true);
  assert_eq!(super::error_count(&stderr), 0, "{stderr}");
  assert!(
    !xml.contains("a4paper, left"),
    "the external document's \\geometry argument leaked:\n{xml}"
  );
  latexml::util::test::assert_element(
    &xml,
    "titlepage",
    &[],
    r##"<titlepage><p><text font="bold" fontsize="140%">Università degli Studi di Bologna</text></p><p><rule height="1px" width="100%"/><text fontsize="120%">DIPARTIMENTO DI MATEMATICA<break/>Corso di Dottorato di Ricerca in Matematica</text></p><p><text font="bold" fontsize="170%">Sugli anelli compatti</text></p><p><tabular vattach="middle"><tbody><tr><td align="left" class="ltx_nopad_r"><tabular vattach="top"><tr><td align="left" class="ltx_nopad_r">Candidato:</td></tr><tr><td align="left" class="ltx_nopad_r"><text font="bold" fontsize="120%">Nome Cognome</text></td></tr></tabular></td><td align="left" class="ltx_nopad_l ltx_nopad_r"><tabular vattach="top"><tr><td align="left" class="ltx_nopad_r">Relatore:</td></tr><tr><td align="left" class="ltx_nopad_r"><text font="bold" fontsize="120%">Prof. Relatore</text></td></tr></tabular></td></tr></tbody></tabular></p><p><rule height="1px" width="100%"/><text font="bold" fontsize="120%">Anno Accademico 2000-2001</text></p></titlepage>"##,
  );
}
