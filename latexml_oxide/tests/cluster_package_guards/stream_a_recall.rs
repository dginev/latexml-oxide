//! Batch 56jt: recall defects from stream A (sweep #124), each a document part
//! pdflatex prints and the converted page lost. The guards read the HTML page,
//! the artifact whose words the recall audit counts.
use std::process::Command;

use latexml::util::test::assert_element;

use super::perfect_kernel_batch46::{error_count, warning_count};

const RAW: &str = "--preload=[rawstyles,rawclasses]latexml.sty";

/// Convert `t.tex` (plus side files) to an HTML page with the raw preload.
/// Returns (ANSI-stripped stderr, HTML).
fn convert_html(tex: &str, files: &[(&str, &str)]) -> (String, String) {
  convert_to(tex, files, "t.html")
}

/// Convert `t.tex` (plus side files) to core XML with the raw preload, for what
/// the page does not show (a location-only reference prints nothing).
/// Returns (ANSI-stripped stderr, XML).
fn convert_xml(tex: &str, files: &[(&str, &str)]) -> (String, String) {
  convert_to(tex, files, "t.xml")
}

fn convert_to(tex: &str, files: &[(&str, &str)], dest: &str) -> (String, String) {
  let bin = env!("CARGO_BIN_EXE_latexml_oxide");
  let workdir = tempfile::tempdir().expect("create tempdir");
  std::fs::write(workdir.path().join("t.tex"), tex).expect("write t.tex");
  for (name, content) in files {
    std::fs::write(workdir.path().join(name), content).expect("write side file");
  }
  let output = Command::new(bin)
    .args([
      "t.tex",
      "--dest",
      dest,
      "--nocomments",
      "--timeout=110",
      RAW,
    ])
    .current_dir(workdir.path())
    .output()
    .expect("spawn latexml_oxide");
  let stderr = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
  let out = std::fs::read_to_string(workdir.path().join(dest)).unwrap_or_default();
  (stderr, out)
}

const GLSADD: &str =
  include_str!("../../../tools/perfect_kernel/repros/index/glsadd_entries_listed.tex");
const GLSADD_EXTRA: &str =
  include_str!("../../../tools/perfect_kernel/repros/index/glsadd_glossaries_extra.tex");

/// `\glsaddall` writes every entry to the glossary file (glossaries.sty:5295,
/// through `\glsadd` :5280 and `\@@do@wrglossary` :6299), so makeindex lists
/// the entries no `\gls` names: Strength in the glossary, AC in the acronyms.
/// The location-only reference it leaves in the paragraph prints nothing.
#[test]
fn glsaddall_lists_every_entry() {
  let (stderr, html) = convert_html(GLSADD, &[]);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &html,
    "section",
    &[r#"id="glo.main""#],
    r#"<section class="ltx_glossary ltx_list_main" id="glo.main">
<h2 class="ltx_title ltx_title_glossary">Glossary</h2>
<dl class="ltx_glossarylist">
<dt class="ltx_glossaryentry ltx_list_main" id="glo.main.dex">Dexterity</dt>
<dd>Agility and accuracy</dd>
<dt class="ltx_glossaryentry ltx_list_main" id="glo.main.str">Strength</dt>
<dd>Physical power of a character</dd></dl>
</section>"#,
  );
  assert_element(
    &html,
    "section",
    &[r#"id="glo.acronym""#],
    r#"<section class="ltx_glossary ltx_list_acronym" id="glo.acronym">
<h2 class="ltx_title ltx_title_glossary">Acronyms</h2>
<dl class="ltx_glossarylist">
<dt class="ltx_glossaryentry ltx_list_acronym" id="glo.acronym.ac">AC</dt>
<dd>Armor Class</dd></dl>
</section>"#,
  );
  assert_element(
    &html,
    "div",
    &[r#"id="p1""#],
    r#"<div class="ltx_para" id="p1">
<p class="ltx_p">Text about <span class="ltx_glossaryref" title="Agility and accuracy">Dexterity</span>.
</p>
</div>"#,
  );
}

/// glossaries-extra replaces `\@gls@link` and `\glsadd` (glossaries-extra.sty
/// :3465, :3571-3604); the binding's wraps follow them, so the `\gls`-used
/// entry, the `\glsaddeach`-added one and the preamble `\glsadd` are listed.
#[test]
fn glossaries_extra_lists_used_and_added_entries() {
  let (stderr, html) = convert_html(GLSADD_EXTRA, &[]);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &html,
    "section",
    &[r#"id="glo.main""#],
    r#"<section class="ltx_glossary ltx_list_main" id="glo.main">
<h2 class="ltx_title ltx_title_glossary">Glossary</h2>
<dl class="ltx_glossarylist">
<dt class="ltx_glossaryentry ltx_list_main" id="glo.main.dex">Dexterity</dt>
<dd>Agility and accuracy</dd>
<dt class="ltx_glossaryentry ltx_list_main" id="glo.main.str">Strength</dt>
<dd>Physical power</dd>
<dt class="ltx_glossaryentry ltx_list_main" id="glo.main.wis">Wisdom</dt>
<dd>Common sense</dd></dl>
</section>"#,
  );
  // The preamble `\glsadd{wis}` is carried into the first paragraph, as a
  // reference that prints nothing (DIVERGENCES #320).
  assert_element(
    &html,
    "div",
    &[r#"id="p1""#],
    r#"<div class="ltx_para" id="p1">
<p class="ltx_p">Text about <span class="ltx_glossaryref" title="Agility and accuracy">Dexterity</span>. </p>
</div>"#,
  );
}

const GLSADD_PREAMBLE: &str =
  include_str!("../../../tools/perfect_kernel/repros/index/glsadd_preamble_first_paragraph.tex");

/// A preamble `\glsaddall`'s references ride into the first paragraph that
/// opens, the abstract's, instead of opening one at `\begin{document}`: the body
/// starts with `\maketitle`, so that paragraph was an empty `ltx:para` p1 and
/// "Opening words." became p2. Both entries are still listed.
#[test]
fn preamble_glsadd_opens_no_paragraph() {
  let (stderr, html) = convert_html(GLSADD_PREAMBLE, &[]);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &html,
    "div",
    &[r#"id="p1""#],
    r#"<div id="p1" class="ltx_para">
<p class="ltx_p">Opening words.</p>
</div>"#,
  );
  // No stray paragraph anywhere: the section holds its one paragraph and the
  // glossary, and the document level only p1.
  assert!(!html.contains(r#"id="p2""#), "{html}");
  assert_element(
    &html,
    "section",
    &[r#"id="S1""#],
    r#"<section id="S1" class="ltx_section">
<h2 class="ltx_title ltx_title_section"><span class="ltx_tag ltx_tag_section">1 </span>Intro</h2>
<div id="S1.p1" class="ltx_para">
<p class="ltx_p">Hello <span title="Agility" class="ltx_glossaryref">Dexterity</span>.</p>
</div>
<section id="glo.main" class="ltx_glossary ltx_list_main">
<h2 class="ltx_title ltx_title_glossary">Glossary</h2>
<dl class="ltx_glossarylist">
<dt id="glo.main.dex" class="ltx_glossaryentry ltx_list_main">Dexterity</dt>
<dd>Agility</dd>
<dt id="glo.main.str" class="ltx_glossaryentry ltx_list_main">Strength</dt>
<dd>Physical power</dd></dl>
</section>
</section>"#,
  );
  assert_element(
    &html,
    "div",
    &[r#"id="abstract1""#],
    r#"<div id="abstract1" class="ltx_abstract"><h6 class="ltx_title ltx_title_abstract">Abstract</h6>
<p class="ltx_p">Abstract text.</p>
</div>"#,
  );
  // The references rode the one-shot `\everypar` into the abstract's
  // paragraph, the first to open, not the `\end{document}` fallback.
  let (stderr, xml) = convert_xml(GLSADD_PREAMBLE, &[]);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "abstract",
    &[],
    r#"<abstract inlist="toc" name="Abstract" xml:id="abstract1">
<p><glossaryref inlist="main" key="str" show="none"/><glossaryref inlist="main" key="dex" show="none"/>Abstract text.</p>
</abstract>"#,
  );
}

const GLS_MATH: &str =
  include_str!("../../../tools/perfect_kernel/repros/index/gls_in_math_glossaries.tex");
const GLS_MATH_EXTRA: &str =
  include_str!("../../../tools/perfect_kernel/repros/index/gls_in_math_glossaries_extra.tex");

/// A `\gls` in math typesets the term alone: the formula's `alttext` is the
/// term's TeX and its MathML the term's `<mi>`, where the reference wrap was an
/// `XMText` atom (`\lx@glossaries@gls@link{main}{v}{…}` in `alttext`, an
/// `<mtext>` holding a nested `<math>`; Perl identical, KNOWN_PERL_ERRORS #276).
/// The location-only reference after the inline formula lists the entry.
#[test]
fn gls_in_math_typesets_the_term_alone() {
  // (document, glossary type, glossary title, inline alttext, display alttext)
  for (tex, list, title, inline_tex, display_tex) in [
    (
      GLS_MATH,
      "main",
      "Glossary",
      r"{{}}\mathbf{v}=\frac{d{{}}\mathbf{v}}{dt}",
      r"E={{}}\mathbf{v}^{2}",
    ),
    (
      GLS_MATH_EXTRA,
      "symbols",
      "Symbols",
      r"{{}}{\mathbf{v}}=\frac{d{{}}{\mathbf{v}}}{dt}",
      r"E={{}}{\mathbf{v}}^{2}",
    ),
  ] {
    let (stderr, html) = convert_html(tex, &[]);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert_eq!(warning_count(&stderr), 0, "{stderr}");
    assert_element(
      &html,
      "math",
      &[r#"id="p1.m1""#],
      &format!(
        r#"<math id="p1.m1" class="ltx_Math" alttext="{inline_tex}" display="inline"><mrow><mi>𝐯</mi><mo>=</mo><mfrac><mrow><mi>d</mi><mo>⁢</mo><mi>𝐯</mi></mrow><mrow><mi>d</mi><mo>⁢</mo><mi>t</mi></mrow></mfrac></mrow></math>"#
      ),
    );
    assert_element(
      &html,
      "math",
      &[r#"id="S0.E1.m1""#],
      &format!(
        r#"<math id="S0.E1.m1" class="ltx_Math" alttext="{display_tex}" display="block"><mrow><mi>E</mi><mo>=</mo><msup><mi>𝐯</mi><mn>2</mn></msup></mrow></math>"#
      ),
    );
    assert_element(
      &html,
      "section",
      &[&format!(r#"id="glo.{list}""#)],
      &format!(
        r#"<section id="glo.{list}" class="ltx_glossary ltx_list_{list}">
<h2 class="ltx_title ltx_title_glossary">{title}</h2>
<dl class="ltx_glossarylist">
<dt id="glo.{list}.v" class="ltx_glossaryentry ltx_list_{list}"><math class="ltx_Math" alttext="\mathbf{{v}}" display="inline"><mi>𝐯</mi></math></dt>
<dd>velocity</dd></dl>
</section>"#
      ),
    );
    // The location-only references follow the inline formula in its `p`.
    let (stderr, xml) = convert_xml(tex, &[]);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert_element(
      &xml,
      "p",
      &[],
      &format!(
        r#"<p>Velocity <Math mode="inline" tex="{inline_tex}" text="v = (d * v) / (d * t)" xml:id="p1.m1">
<XMath>
<XMApp>
<XMTok meaning="equals" role="RELOP">=</XMTok>
<XMTok font="bold" role="UNKNOWN">v</XMTok>
<XMApp>
<XMTok mathstyle="text" meaning="divide" role="FRACOP"/>
<XMApp>
<XMTok meaning="times" role="MULOP">⁢</XMTok>
<XMTok font="italic" fontsize="70%" role="UNKNOWN">d</XMTok>
<XMTok font="bold" fontsize="70%" role="UNKNOWN">v</XMTok>
</XMApp>
<XMApp>
<XMTok meaning="times" role="MULOP">⁢</XMTok>
<XMTok font="italic" fontsize="70%" role="UNKNOWN">d</XMTok>
<XMTok font="italic" fontsize="70%" role="UNKNOWN">t</XMTok>
</XMApp>
</XMApp>
</XMApp>
</XMath>
</Math><glossaryref inlist="{list}" key="v" show="none"/><glossaryref inlist="{list}" key="v" show="none"/> and</p>"#
      ),
    );
  }
}

/// In an alignment the cells' formulae and the whole-row formula of the
/// `MathFork` stay clean too: a display level admits no inline element, so no
/// location-only reference is made there, and neither `\gls` nor a `\glsadd`
/// inside a macro (`\vx`) leaves an `XMText` atom in any formula.
#[test]
fn gls_in_an_alignment_leaves_the_formulae_alone() {
  let tex = r"\documentclass{article}
\usepackage{amsmath}
\usepackage{glossaries}
\makeglossaries
\newglossaryentry{v}{name={\ensuremath{\mathbf{v}}},description={velocity}}
\newglossaryentry{vec}{name={vec},description={a vector}}
\newcommand\vx{\glsadd{vec}\mathbf{x}}
\begin{document}
\begin{align} E &= \gls{v}^2 \\ F &= \vx + 1 \end{align}
\printglossaries
\end{document}
";
  let (stderr, xml) = convert_xml(tex, &[]);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(!xml.contains("<XMText"), "{xml}");
  assert!(!xml.contains("<glossaryref"), "{xml}");
  assert_element(
    &xml,
    "Math",
    &[r#"xml:id="S0.E2.m3""#],
    r#"<Math tex="\displaystyle F={}\mathbf{x}+1" text="F = x + 1" xml:id="S0.E2.m3">
<XMath>
<XMApp>
<XMTok meaning="equals" role="RELOP">=</XMTok>
<XMTok font="italic" role="UNKNOWN">F</XMTok>
<XMApp>
<XMTok meaning="plus" role="ADDOP">+</XMTok>
<XMTok font="bold" role="UNKNOWN">x</XMTok>
<XMTok meaning="1" role="NUMBER">1</XMTok>
</XMApp>
</XMApp>
</XMath>
</Math>"#,
  );
}

const BIB_FILES: &str =
  include_str!("../../../tools/perfect_kernel/repros/index-bib/bib_files_of_each_bibliography.tex");
const BIB_FILES_BIB: &str =
  include_str!("../../../tools/perfect_kernel/repros/index-bib/bib_files_of_each_bibliography.bib");
const REFSECTION_GLOBAL: &str = include_str!(
  "../../../tools/perfect_kernel/repros/index-bib/biblatex_refsection_global_resources.tex"
);

/// Each bibliography reads its own `@files`: an inline `{thebibliography}`
/// before `\bibliography{…}` no longer leaves the real list empty (Perl
/// MakeBibliography.pm:101 reads the first bibliography's files for all).
#[test]
fn each_bibliography_reads_its_own_files() {
  let (stderr, html) = convert_html(BIB_FILES, &[(
    "bib_files_of_each_bibliography.bib",
    BIB_FILES_BIB,
  )]);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &html,
    "li",
    &[r#"id="biba.bib1""#],
    r##"<li class="ltx_bibitem ltx_bib_book" id="biba.bib1"><span class="ltx_tag ltx_bib_key ltx_role_refnum ltx_tag_bibitem">[1]</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_author">Donald Knuth</span><span class="ltx_text ltx_bib_year"> (1986)</span>
</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_title">Computers and typesetting</span>.
</span>
<span class="ltx_bibblock"> <span class="ltx_text ltx_bib_publisher">Addisonwesley</span>.
</span>
<span class="ltx_bibblock ltx_bib_cited">Cited by: <a class="ltx_ref" href="#p2" title="">p2</a>.
</span></li>"##,
  );
}

/// A refsection naming its own resources reads the `\addglobalbib` ones too
/// (biblatex.sty:10797-10801), so the second list holds the global entry and
/// its own; the first section's entry is not repeated there. Each section
/// `\nocite{*}`s its files, as pdflatex+biber lists them, so no key is cited
/// that the other list's files lack (the shared citation list, SYNC_STATUS).
#[test]
fn refsection_reads_the_global_resources() {
  let (stderr, html) = convert_html(REFSECTION_GLOBAL, &[]);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &html,
    "section",
    &[r#"id="biba""#],
    r##"<section class="ltx_bibliography" id="biba">
<h2 class="ltx_title ltx_title_bibliography">References</h2>
<ul class="ltx_biblist" id="biba.L1">
<li class="ltx_bibitem ltx_bib_book" id="biba.bib2"><span class="ltx_tag ltx_bib_key ltx_role_refnum ltx_tag_bibitem">[1]</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_author">Gamma Global</span><span class="ltx_text ltx_bib_year"> (2001)</span>
</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_title">Globalentry</span>.
</span>
<span class="ltx_bibblock"> <span class="ltx_text ltx_bib_publisher">Pub</span>.
</span></li>
<li class="ltx_bibitem ltx_bib_book" id="biba.bib1"><span class="ltx_tag ltx_bib_key ltx_role_refnum ltx_tag_bibitem">[2]</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_author">Bravo Second</span><span class="ltx_text ltx_bib_year"> (2003)</span>
</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_title">Secondentry</span>.
</span>
<span class="ltx_bibblock"> <span class="ltx_text ltx_bib_publisher">Pub</span>.
</span></li>
</ul>
</section>"##,
  );
}

const OTHER_LIST_ITEM: &str =
  include_str!("../../../tools/perfect_kernel/repros/index-bib/bib_other_list_item_is_missing.tex");

/// A key cited but absent from a bibliography's files is missing there, also
/// when another list, the supplement's `{thebibliography}`, holds its item
/// (Perl MakeBibliography.pm:342-343; arXiv 2605.06049). A pseudo-item built
/// from that item's record read "[2]" with no content in the `.bib` list and
/// took over the citation's link. The one warning is the faithful diagnostic
/// (bibtex: 'I didn't find a database entry for "supp"'), an exception to the
/// 0-warning fixture rule recorded in SYNC_STATUS.
#[test]
fn a_citation_of_another_lists_item_is_missing_here() {
  let (stderr, html) = convert_html(OTHER_LIST_ITEM, &[]);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 1, "{stderr}");
  assert!(
    stderr
      .lines()
      .any(|line| line == "Warning:bibliography:missing_keys Missing bibkeys: supp"),
    "{stderr}"
  );
  assert_element(
    &html,
    "section",
    &[r#"id="bib""#],
    r##"<section id="bib" class="ltx_bibliography">
<h2 class="ltx_title ltx_title_bibliography">References</h2>
<ul id="bib.L1" class="ltx_biblist">
<li id="bib.bib1" class="ltx_bibitem ltx_bib_book"><span class="ltx_tag ltx_bib_key ltx_role_refnum ltx_tag_bibitem">[1]</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_author">Donald Knuth</span><span class="ltx_text ltx_bib_year"> (1986)</span>
</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_title">Computers and typesetting</span>.
</span>
<span class="ltx_bibblock"> <span class="ltx_text ltx_bib_publisher">Addison-Wesley</span>.
</span>
<span class="ltx_bibblock ltx_bib_cited">Cited by: <a href="#p1" title="" class="ltx_ref">p1</a>.
</span></li>
</ul>
</section>"##,
  );
  assert_element(
    &html,
    "div",
    &[r#"id="Sx1.p1""#],
    r##"<div id="Sx1.p1" class="ltx_para">
<p class="ltx_p">Supplement <cite class="ltx_cite ltx_citemacro_cite">[<a href="#biba.bib1" title="" class="ltx_ref">1</a>]</cite>.</p>
</div>"##,
  );
}

const RECASE: &str = include_str!(
  "../../../tools/perfect_kernel/repros/index-bib/bib_title_recase_keeps_control_words.tex"
);
const RECASE_BIB: &str = include_str!(
  "../../../tools/perfect_kernel/repros/index-bib/bib_title_recase_keeps_control_words.bib"
);

/// The title re-case keeps a control word whose re-cased name is undefined:
/// `\LaTeXe` and `\TeX` print their logos, where `\latexe`/`\tex` were two
/// undefined-macro errors (lshort-german/l2kurz, once its bibliography was read).
/// (The logo's "X 2" gap is U+2002 EN SPACE, as `\LaTeXe` sets it.)
#[test]
fn bib_title_recase_keeps_undefined_control_words() {
  let (stderr, html) = convert_html(RECASE, &[(
    "bib_title_recase_keeps_control_words.bib",
    RECASE_BIB,
  )]);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &html,
    "li",
    &[r#"id="bib.bib1""#],
    r##"<li class="ltx_bibitem ltx_bib_misc" id="bib.bib1"><span class="ltx_tag ltx_bib_key ltx_role_refnum ltx_tag_bibitem">[2]</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_author">Keith Reckdahl</span><span class="ltx_text ltx_bib_year"> (2006)</span>
</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_title">Using imported graphics in <span class="ltx_text ltx_LaTeX_logo" style="letter-spacing:-0.2em; margin-right:0.1em;">L<span class="ltx_text" style="position:relative; bottom:0.4ex;font-variant:small-caps;;">a</span>T<span class="ltx_text" style="position:relative; bottom:-0.2ex;font-variant:small-caps;font-size:120%;">e</span>X 2<span class="ltx_text" style="position:relative; bottom:-0.3ex;font-style:italic;">ε</span></span></span>.
</span></li>"##,
  );
  assert_element(
    &html,
    "li",
    &[r#"id="bib.bib2""#],
    r##"<li class="ltx_bibitem ltx_bib_misc" id="bib.bib2"><span class="ltx_tag ltx_bib_key ltx_role_refnum ltx_tag_bibitem">[1]</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_author">Bernd Raichle</span><span class="ltx_text ltx_bib_year"> (1998)</span>
</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_title">German <span class="ltx_text ltx_TeX_logo" style="letter-spacing:-0.2em; margin-right:0.2em;">T<span class="ltx_text" style="position:relative; bottom:-0.2ex;font-variant:small-caps;font-size:120%;;">e</span>X</span> and Østergaard</span>.
</span></li>"##,
  );
}

const OPENOUT_TEX: &str = include_str!(
  "../../../tools/perfect_kernel/repros/string-mouth/openout_completes_tex_extension.tex"
);
const WRAPPED_FILECONTENTS: &str = include_str!(
  "../../../tools/perfect_kernel/repros/string-mouth/openout_wrapped_filecontents.tex"
);

/// `{filecontents*}{democode}` writes `democode.tex`, as `\openout` completes
/// an extension-less name (tex.web §1374): both existence tests see it,
/// `\input{democode.tex}` reads it, and `\verbatiminput{democode}` still finds
/// it by the bare name, as TeX's reads try `.tex` first (§537).
#[test]
fn openout_names_an_extensionless_file_tex() {
  let (stderr, html) = convert_html(OPENOUT_TEX, &[]);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &html,
    "div",
    &[r#"id="p1""#],
    r#"<div class="ltx_para" id="p1">
<p class="ltx_p">A:Yes.
B:Yes.
C:Alpha code.

D:</p><pre class="ltx_verbatim ltx_font_typewriter">Alpha code.
</pre>
</div>"#,
  );
}

/// latexdemo's `\PrintDemo` finds the `democode.tex` its `{DefineCode}` wrote
/// (latexdemo.sty:97-101, :159, :167), so each example prints its code and its
/// result (latex4wp). The fixture defines the two as latexdemo does, without its
/// mdframed frames and kvoptions keys, whose loads warn (a stub, and pdflatex's
/// own kvoptions-patch warning; `openout_latexdemo_printdemo.tex` loads the package).
#[test]
fn wrapped_filecontents_prints_the_code_and_its_result() {
  let (stderr, html) = convert_html(WRAPPED_FILECONTENTS, &[]);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &html,
    "figure",
    &[r#"id="tab1""#],
    r#"<figure id="tab1" class="ltx_float ltx_lstlisting">
<div class="ltx_listing ltx_lstlisting ltx_listing"><div class="ltx_listing_data"><a href="data:text/plain;base64,RGVsdGEgXGVtcGh7RWNob30=" download="democode">⬇</a></div>
<div id="lstnumberx1" class="ltx_listingline"><span class="ltx_text ltx_lst_identifier">Delta</span><span class="ltx_text ltx_lst_space"> </span>\<span class="ltx_text ltx_lst_identifier">emph</span>{<span class="ltx_text ltx_lst_identifier">Echo</span>}
</div>
</div>
</figure>"#,
  );
  assert_element(
    &html,
    "div",
    &[r#"id="p2""#],
    r#"<div id="p2" class="ltx_para">
<p class="ltx_p">Result: Delta <em class="ltx_emph ltx_font_italic">Echo</em></p>
</div>"#,
  );
  assert_element(
    &html,
    "div",
    &[r#"id="p1""#],
    r#"<div id="p1" class="ltx_para">
<p class="ltx_p">Code:</p>
</div>"#,
  );
  // The second `{DefineCode}` overwrites the file.
  assert_element(
    &html,
    "figure",
    &[r#"id="tab2""#],
    r#"<figure id="tab2" class="ltx_float ltx_lstlisting">
<div class="ltx_listing ltx_lstlisting ltx_listing"><div class="ltx_listing_data"><a href="data:text/plain;base64,Rm94dHJvdCBcdGV4dGJme0dvbGZ9" download="democode">⬇</a></div>
<div id="lstnumberx2" class="ltx_listingline"><span class="ltx_text ltx_lst_identifier">Foxtrot</span><span class="ltx_text ltx_lst_space"> </span>\<span class="ltx_text ltx_lst_identifier">textbf</span>{<span class="ltx_text ltx_lst_identifier">Golf</span>}
</div>
</div>
</figure>"#,
  );
  assert_element(
    &html,
    "div",
    &[r#"id="p4""#],
    r#"<div id="p4" class="ltx_para">
<p class="ltx_p">Result: Foxtrot <span class="ltx_text ltx_font_bold">Golf</span></p>
</div>"#,
  );
}

const BIB_FIELDS: &str =
  include_str!("../../../tools/perfect_kernel/repros/index-bib/bib_fields_separated.tex");
const BIB_FIELDS_BIB: &str =
  include_str!("../../../tools/perfect_kernel/repros/index-bib/bib_fields_separated.bib");

/// The fields of one formatting block are separated: the website's title and
/// type follow its name after a space, an editor follows the author after a
/// comma, an entry's notes are units of their own, and two fields read into
/// one element (`organization`, `institution`) are a list. Perl's "" rows
/// (MakeBibliography.pm:688, :708-787) ran them together, and LaTeXML.css adds
/// no separator between the `ltx_bib_*` spans.
#[test]
fn bibliography_fields_are_separated() {
  let (stderr, html) = convert_html(BIB_FIELDS, &[("bib_fields_separated.bib", BIB_FIELDS_BIB)]);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &html,
    "li",
    &[r#"class="ltx_bibitem ltx_bib_website""#],
    r##"<li class="ltx_bibitem ltx_bib_website" id="bib.bib1"><span class="ltx_tag ltx_bib_key ltx_role_refnum ltx_tag_bibitem">[3]</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_author">V. Jacobson</span><span class="ltx_text ltx_bib_year"> (1990)</span> <span class="ltx_text ltx_bib_title">Modified TCP Congestion Avoidance Algorithm</span> (Website)
</span>
<span class="ltx_bibblock">External Links: <span class="ltx_text ltx_bib_links"><a class="ltx_ref ltx_bib_external" href="https://example.org/tcp" title="">https://example.org/tcp</a></span>
</span>
<span class="ltx_bibblock ltx_bib_cited">Cited by: <a class="ltx_ref" href="#p1" title="">p1</a>.
</span></li>"##,
  );
  assert_element(
    &html,
    "li",
    &[r#"class="ltx_bibitem ltx_bib_video""#],
    r##"<li class="ltx_bibitem ltx_bib_video" id="bib.bib2"><span class="ltx_tag ltx_bib_key ltx_role_refnum ltx_tag_bibitem">[2]</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_author">John Cleese</span>, <span class="ltx_text ltx_bib_editor">Terry Gilliam (Ed.)</span><span class="ltx_text ltx_bib_year"> (2001)</span>
</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_title">Commentaries</span>.
</span>
<span class="ltx_bibblock"> <span class="ltx_text ltx_bib_publisher">Columbia</span>.
</span>
<span class="ltx_bibblock ltx_bib_cited">Cited by: <a class="ltx_ref" href="#p1" title="">p1</a>.
</span></li>"##,
  );
  assert_element(
    &html,
    "li",
    &[r#"class="ltx_bibitem ltx_bib_misc""#],
    r##"<li class="ltx_bibitem ltx_bib_misc" id="bib.bib3"><span class="ltx_tag ltx_bib_key ltx_role_refnum ltx_tag_bibitem">[1]</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_author">Mike Author</span><span class="ltx_text ltx_bib_year"> (2003)</span>
</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_title">Misc Thing</span>.
</span>
<span class="ltx_bibblock"> <span class="ltx_text ltx_bib_publisher">Oscarorg, Cobaltinstitution</span>.
</span>
<span class="ltx_bibblock">Note: <span class="ltx_text ltx_bib_note">Novemberhow. Limanote. Mikeaddendum</span>
</span>
<span class="ltx_bibblock ltx_bib_cited">Cited by: <a class="ltx_ref" href="#p1" title="">p1</a>.
</span></li>"##,
  );
}
