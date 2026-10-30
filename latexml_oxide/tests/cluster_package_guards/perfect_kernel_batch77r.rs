//! Red/green guards for batch 77r: run-367 Error→Fatal regressions of classes and the kernel (the main session's
//! triage C3, C6, C8-C13, C15). Each paper's whole document was lost to a Fatal or a crash; the guards assert no Fatal,
//! the exact diagnostics, and the authored content in its element.
use latexml::util::test::{assert_element, convert_files_with};

use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch57::{RAW, assert_elements, assert_elements_with},
};

/// A repro whose side file is a leading `filecontents*` block: the document without the block and the file, so the
/// conversion writes nothing (the file is staged beside the document).
fn split_filecontents(tex: &str) -> (String, String, String) {
  let begin = tex
    .find("\\begin{filecontents*}[overwrite]{")
    .expect("a filecontents* block");
  let name_start = begin + "\\begin{filecontents*}[overwrite]{".len();
  let name_end = name_start + tex[name_start..].find('}').expect("its file name");
  let end_marker = "\\end{filecontents*}\n";
  let end = name_end + tex[name_end..].find(end_marker).expect("its end");
  let content = tex[name_end + 2..end].to_string();
  let document = format!("{}{}", &tex[..begin], &tex[end + end_marker.len()..]);
  (document, tex[name_start..name_end].to_string(), content)
}

/// Convert a repro with a `filecontents*` side file under `preload`.
fn convert_with_side_file(tex: &str, preload: &str) -> (String, String) {
  let (document, name, content) = split_filecontents(tex);
  convert_files_with(&document, &[(&name, &content)], Some(preload))
}

/// C3: revtex4-2 never loads hyperref (revtex4-2.cls:476 only tests for it); the support binding preloads it (Perl
/// revtex4_support.sty.ltxml:23) and now marks it unloaded, so the document's own `\usepackage{hyperref}` restores
/// the `\hypersetup` phfnote's `hyperrefdefs=defer` deferred (phfnote.sty:857-861) — which otherwise re-added its
/// `\AtBeginDocument` code for ever (KNOWN_PERL_ERRORS #997; 2207.13707, 2403.04828, 2508.03993, 2508.03994,
/// 2601.20155). Repro class-bindings/revtex_preloaded_hyperref_loads_again.
#[test]
fn revtex_preloaded_hyperref_loads_again() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/revtex_preloaded_hyperref_loads_again.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[(
    "para",
    "p1",
    r##"<para class="ltx_noindent" xml:id="p1"><p>Text.</p></para>"##,
  )]);
}

/// C3's second load of the hyperref binding saves the kernel's `\refstepcounter` once: saved again it was hyperref's
/// own, `\H@refstepcounter` calling itself under cleveref (2207.13707, 1704.06321, 1204.1969). The heading tag is REVTeX's
/// "I." (aps4-2.rtx:288 `\@seccntformat`, 78d; KNOWN_PERL_ERRORS #1027). Repro
/// class-bindings/revtex_hyperref_loaded_twice_cleveref.
#[test]
fn revtex_hyperref_loaded_twice_keeps_cleveref() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/revtex_hyperref_loaded_twice_cleveref.tex"
  );
  let xml = assert_elements(tex, RAW, (0, 0), &[(
    "para",
    "S1.p1",
    r##"<para xml:id="S1.p1"><p>Text <ref labelref="LABEL:s" show="creftype~creflabel"/>.</p></para>"##,
  )]);
  assert_element(
    &xml,
    "tags",
    &[],
    "<tags><tag>I.</tag><tag role=\"autoref\">section\u{a0}I</tag><tag role=\"creftype\">Section</tag><tag \
     role=\"creftypecap\">Section</tag><tag role=\"creftypeplural\">Sections</tag><tag \
     role=\"creftypepluralcap\">Sections</tag><tag role=\"refnum\">I</tag><tag role=\"typerefnum\">§I</tag></tags>",
  );
}

/// C6: a deluxetable begun inside an open one is LaTeX's float in a float, "Not in outer par mode" (latex.ltx
/// `\@xfloat`'s `\@parmoderr`); the inner one keeps the outer's bindings — saving the outer's `\label` again as the
/// original made `\label` call itself (Fatal IfLimit; 1704.06321, 1204.1969). Repro
/// class-bindings/emulateapj_deluxetable_in_deluxetable.
#[test]
fn emulateapj_deluxetable_in_deluxetable_is_not_in_outer_par_mode() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/emulateapj_deluxetable_in_deluxetable.tex"
  );
  let xml = assert_elements_with(tex, RAW, (1, 0), &["Not in outer par mode"], &[]);
  // both tables' heads and rows, the inner's first (RED part: the outer's caption is the inner's, as in int32a since
  // 68y; SYNC_STATUS)
  assert_element(
    &xml,
    "table",
    &[r#"xml:id="S0.T1""#],
    r#"<table class="ltx_panels_gallery" inlist="lot" panelcols="1" panelrows="2" xml:id="S0.T1"><tags><tag>Table 1</tag><tag role="autoref">Table 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Table 1</tag></tags><toccaption><tag close=" ">1</tag>Inner caption</toccaption><caption><tag>Table 1</tag>Inner caption</caption><tabular class="ltx_figure_panel"><thead><tr><td align="center" border="tt" thead="column">IA</td><td align="center" border="tt" thead="column">IB</td></tr></thead><tbody><tr><td align="center" border="b t">ione</td><td align="center" border="b t">itwo</td></tr></tbody></tabular><break class="ltx_break"/><tabular class="ltx_figure_panel"><thead><tr><td align="center" border="tt" thead="column">OA</td><td align="center" border="tt" thead="column">OB</td></tr></thead><tbody><tr><td align="center" border="b t">oone</td><td align="center" border="b t">otwo</td></tr></tbody></tabular></table>"#,
  );
}

/// C8: a unit named by its own character code (atlaspackage.sty:225-227 `\DeclareSIUnit\%{\char`\%}`) is the
/// character: TeX reads the operand after a backquote unexpanded (tex.web §442). Resolving it as the unit recursed
/// until the stack overflowed (exit 134; 2504.05919). Repro package-bindings/siunitx_unit_named_by_its_own_char_code.
#[test]
fn siunitx_unit_named_by_its_own_char_code() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/package-bindings/siunitx_unit_named_by_its_own_char_code.tex"
  );
  // the unit's own `number-unit-product={}`: "60%", no space (siunitx.sty:6684-6694)
  let xml = assert_elements(tex, RAW, (0, 0), &[]);
  assert_element(
    &xml,
    "Math",
    &[r#"xml:id="p1.m1""#],
    r#"<Math mode="inline" tex="60\mathrm{\char 37\relax}" text="60 * percent" xml:id="p1.m1"><XMath><XMApp><XMTok meaning="times" role="MULOP"/><XMTok meaning="60" role="NUMBER">60</XMTok><XMTok class="ltx_unit" meaning="percent" role="ID">%</XMTok></XMApp></XMath></Math>"#,
  );
}

/// C8 (r2): a unit presentation keeps the space that ends a control word before a letter: `\char`\a b` is "ab"
/// (gluing it read `\char`\ab`, "a"), `\textmu m` is µm, not an undefined `\textmum` (2504.05919's shape). Repro
/// package-bindings/siunitx_unit_presentation_keeps_control_words.
#[test]
fn siunitx_unit_presentation_keeps_control_words() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/package-bindings/siunitx_unit_presentation_keeps_control_words.tex"
  );
  let xml = assert_elements(tex, RAW, (0, 0), &[]);
  assert_element(
    &xml,
    "Math",
    &[r#"xml:id="p1.m1""#],
    r#"<Math mode="inline" tex="1\text{\,}\mathrm{\char 97\relax b}" text="1 * ab" xml:id="p1.m1"><XMath><XMApp><XMText meaning="times" role="MULOP" xml:id="p1.m1.1"> </XMText><XMTok meaning="1" role="NUMBER">1</XMTok><XMTok class="ltx_unit" meaning="ab" role="ID">ab</XMTok></XMApp></XMath></Math>"#,
  );
  assert_element(
    &xml,
    "Math",
    &[r#"xml:id="p1.m2""#],
    r#"<Math mode="inline" tex="2\text{\,}\mathrm{\textmu m}" text="2 * um" xml:id="p1.m2"><XMath><XMApp><XMText meaning="times" role="MULOP" xml:id="p1.m2.1"> </XMText><XMTok meaning="2" role="NUMBER">2</XMTok><XMApp class="ltx_unit" meaning="um" role="ID" xml:id="p1.m2.3"><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok role="UNKNOWN">µ</XMTok><XMTok role="UNKNOWN">m</XMTok></XMApp></XMApp></XMath></Math>"#,
  );
  assert_element(
    &xml,
    "Math",
    &[r#"xml:id="p1.m3""#],
    r#"<Math mode="inline" tex="\mathrm{\textmu m}" text="um" xml:id="p1.m3"><XMath><XMApp class="ltx_unit" meaning="um" role="ID"><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok role="UNKNOWN">µ</XMTok><XMTok role="UNKNOWN">m</XMTok></XMApp></XMath></Math>"#,
  );
}

/// C9: a refused `{abstract}` begin code calling a macro whose delimiter does not occur later in the code reads the
/// environment's body, which a replay does not have: it is not replayed (wds10.sty:527-539 `\pickup#1\end#2`;
/// 0910.4848: 101 errors and a Fatal). Repro sectioning-frontmatter/refused_begin_code_with_delimited_reader.
#[test]
fn refused_begin_code_with_delimited_reader_is_not_replayed() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/refused_begin_code_with_delimited_reader.tex"
  );
  let (stderr, xml) = convert_with_side_file(tex, RAW);
  assert_eq!(
    (error_count(&stderr), warning_count(&stderr)),
    (0, 0),
    "{stderr}"
  );
  assert_element(
    &xml,
    "abstract",
    &[],
    r##"<abstract inlist="toc" name="Abstract" xml:id="abstract1"><p>Abstract text.</p></abstract>"##,
  );
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r##"<para xml:id="p1"><p>Body.</p></para>"##,
  );
}

/// C9 (r2): only a refused begin code's own calls are judged — the tokens outside braces, not a definition's operands:
/// `\strip@prefix` inside an `\edef` body (its `>` comes from `\meaning`), `\pickup` as `\let`'s value or inside a
/// `\renewcommand` body. The code is replayed, so its `\let\emph\textbf` makes "emph" bold, as in pdflatex. Repro
/// sectioning-frontmatter/refused_begin_code_definition_operands.
#[test]
fn refused_begin_code_definition_operands_are_no_calls() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/refused_begin_code_definition_operands.tex"
  );
  let (stderr, xml) = convert_with_side_file(tex, RAW);
  assert_eq!(
    (error_count(&stderr), warning_count(&stderr)),
    (0, 0),
    "{stderr}"
  );
  assert_element(
    &xml,
    "abstract",
    &[],
    r#"<abstract inlist="toc" name="Abstract" xml:id="abstract1"><p>Abstract <text font="bold">emph</text> text.</p></abstract>"#,
  );
}

/// C10: an autoload a `\bibitem` label calls loads its package before the entry is read — crckapb's `\citeauthoryear`
/// is natbib's (OXIDIZED_DESIGN_DIVERGENCES #731), whose parser, loaded inside the tags, read `\the@bibitem` as its
/// own number for ever (astro-ph0007279: Fatal Stomach:Recursion). Repro index-bib/bibitem_label_calls_an_autoload.
#[test]
fn bibitem_label_autoload_loads_before_the_entry() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/index-bib/bibitem_label_calls_an_autoload.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[(
    "bibitem",
    "bib.bib1",
    "<bibitem key=\"ZQS88\" xml:id=\"bib.bib1\"><tags><tag role=\"number\">1</tag><tag role=\"year\">1988</tag><tag \
     role=\"authors\">Zurek</tag><tag role=\"refnum\">Zurek (1988)</tag><tag role=\"key\">ZQS88</tag></tags><bibblock> \
     Text.\n</bibblock></bibitem>",
  )]);
}

/// C11 (partly RED): a nine-argument `\p@equation` reads past the equation's reference forms (8 errors, pdflatex 0:
/// TeX applies it only where a `\ref` prints the label, SYNC_STATUS), but the second equation no longer loops: the
/// first one's digest left a group open, and with it the math label of 73b, which was saved as its own original
/// (`\@ensuremath` for ever; 1902.09345). Repro parameter-conditional/p_equation_taking_nine_arguments.
#[test]
fn p_equation_taking_nine_arguments_does_not_loop() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/parameter-conditional/p_equation_taking_nine_arguments.tex"
  );
  let (stderr, xml) = latexml::util::test::convert_with(tex, Some(RAW));
  assert!(!stderr.lines().any(|l| l.starts_with("Fatal:")), "{stderr}");
  assert_eq!(
    (error_count(&stderr), warning_count(&stderr)),
    (8, 1),
    "{stderr}"
  );
  assert_element(
    &xml,
    "equation",
    &[r#"xml:id="S0.E2""#],
    r#"<equation xml:id="S0.E2"><tags><tag>(2)</tag></tags><text class="ltx_markedasmath">y</text></equation>"#,
  );
}

/// C12: `\errmessage` shows `\errhelp` unexpanded (tex.web §1283 `token_show(err_help)`): t4enc.def:36-39's `\m`
/// puts `\m #1` in its help, which, expanded, called `\m` again for ever (2405.01972: Fatal PushbackLimit). t4enc.def's
/// own "Non existent modified letter" for a letter it has no slot for is shown with the help as written; `\m{y}` itself
/// prints ƴ since the T4 map (77u, `perfect_kernel_batch77u::t4_modified_letter_prints`). Repro
/// fonts-nfss/t4_modified_letter_help_is_not_expanded.
#[test]
fn errmessage_help_is_not_expanded() {
  assert_elements_with(
    r"\documentclass{article}
\usepackage[T4,T1]{fontenc}
\begin{document}
Text {\fontencoding{T4}\selectfont\m{q}} here.
\end{document}",
    RAW,
    (1, 0),
    &[r"t4enc.def: Non existent modified letter: The modified letter \mq does not exist in the"],
    &[(
      "para",
      "p1",
      r#"<para xml:id="p1"><p>Text here.</p></para>"#,
    )],
  );
}

/// C13: a second `\documentstyle` raises LaTeX's "Two \documentclass or \documentstyle commands" and loads no class
/// (latex.ltx:18617-18624, :18944-18947); applying latex209 and the class again made `\normalsize` and
/// `\original@normalsize` call each other (astro-ph9812142). Repro
/// sectioning-frontmatter/documentstyle_twice_loads_one_class.
#[test]
fn documentstyle_twice_loads_one_class() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/documentstyle_twice_loads_one_class.tex"
  );
  assert_elements_with(
    tex,
    RAW,
    (1, 0),
    &["Two \\documentclass or \\documentstyle commands"],
    &[(
      "para",
      "p1",
      r##"<para xml:id="p1"><p>Hello <text font="bold">world</text>. <text fontsize="90%">small</text></p></para>"##,
    )],
  );
}

/// C13 (r2, main-session ruling 2026-10-19): a first class declaration no binding and no file answered (the OmniBus
/// fallback) does not count, so the real `\documentstyle{aa}` after it loads (aa's `\offprints` note), and
/// latex209.def's definitions apply once (applied again, `\normalsize` and `\original@normalsize` called each other):
/// astro-ph9812142's line 1 `#\documentstyle[psfig,epsf]{/disk/3/roud/home/Texstuff/aa}`. Repro
/// sectioning-frontmatter/documentstyle_after_a_missing_class.
#[test]
fn documentstyle_after_a_missing_class_loads_the_class() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/documentstyle_after_a_missing_class.tex"
  );
  let xml = assert_elements_with(
    tex,
    RAW,
    (0, 1),
    &["Can't find binding or file for '/nowhere/aa.sty'"],
    &[(
      "para",
      "p1",
      r##"<para xml:id="p1"><p>Hello <text font="bold">world</text>. <text fontsize="90%">small</text></p></para>"##,
    )],
  );
  assert_element(
    &xml,
    "pubnote",
    &[],
    // (the class's label, aa.cls:337-340 "Send offprint requests to:", not the role's "Preprint: ": batch 92e, OD #915)
    r#"<pubnote name="Send offprint requests to: " role="preprint">A. Name</pubnote>"#,
  );
}

/// C13 (r2): the same after a `\documentclass` no binding and no file answered — OmniBus, then aa.
#[test]
fn documentstyle_after_a_missing_documentclass_loads_the_class() {
  let tex = "\\documentclass{nowhereclass}\n\\documentstyle{aa}\n\\offprints{A. Name}\n\\begin{document}\n\\title{T}\\author{A}\n\\maketitle\nHello.\n\\end{document}\n";
  let xml = assert_elements_with(
    tex,
    RAW,
    (0, 1),
    &["Can't find binding for class nowhereclass (using OmniBus)"],
    &[("para", "p1", r##"<para xml:id="p1"><p>Hello.</p></para>"##)],
  );
  assert_element(
    &xml,
    "pubnote",
    &[],
    // (the class's label, aa.cls:337-340 "Send offprint requests to:", not the role's "Preprint: ": batch 92e, OD #915)
    r#"<pubnote name="Send offprint requests to: " role="preprint">A. Name</pubnote>"#,
  );
}

/// C15 (user ruling 2026-10-19): a runaway limit (here the pushback) inside a speculative title-page run is discarded
/// with the run, `SpeculativeRun::Aborted`, its box lists and conditionals unwound: the class's `\MakeUppercase` of a
/// title holding its non-robust `\small` loops in pdflatex too, and the real `\maketitle` never runs the code
/// (astro-ph0203021, VisionStyle.cls). Repro sectioning-frontmatter/speculative_title_run_runaway_is_discarded.
#[test]
fn speculative_title_run_runaway_is_discarded() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/speculative_title_run_runaway_is_discarded.tex"
  );
  // (a lower pushback limit than the default 5,000,000, for the test's sake: the loop is the same)
  let (stderr, xml) = convert_with_side_file(
    tex,
    "[rawstyles,rawclasses,pushbacklimit=200000]latexml.sty",
  );
  assert_eq!(
    (error_count(&stderr), warning_count(&stderr)),
    (0, 0),
    "{stderr}"
  );
  assert_element(
    &xml,
    "title",
    &[],
    r#"<title fontsize="90%"><text fontsize="111%">X </text>Y</title>"#,
  );
  assert_element(
    &xml,
    "creator",
    &[],
    r#"<creator role="author"><personname>A</personname><contact name="Affiliation: " role="affiliation">B</contact></creator>"#,
  );
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r#"<para xml:id="p1"><p>Text</p></para>"#,
  );
}
