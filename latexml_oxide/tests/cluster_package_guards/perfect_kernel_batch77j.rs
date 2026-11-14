//! Red/green guard for batch 77j: text that LaTeX only writes or uses as a name is not digested, and a raw expl3-syntax
//! load gives back the catcodes it found — sole-cause `unexpected:_` shapes of the projection samples
//! (agent_reports/sole_underscore_2026-10-19). The guards assert the three measures: no Error, the authored text present,
//! its element.
use latexml::util::test::convert_with;

use super::perfect_kernel_batch46::{error_count, warning_count};

fn squash(xml: &str) -> String { xml.split_whitespace().collect::<Vec<_>>().join(" ") }

/// `\addcontentsline`'s file and unit type are only written (`\@writefile{#1}`, `\contentsline{#2}`): a template's
/// leftover line with `_` in the type raises nothing (1204.2781, 2402.17181).
#[test]
fn addcontentsline_names_are_not_digested() {
  let (log, xml) = convert_with(
    "\\documentclass{amsart}\\begin{document}\\section{One}\n\\addcontentsline{file}{sec_unit}{entry}\nText.\\end{document}",
    None,
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(squash(&xml).contains("Text."), "{xml}");
  // the written type is the list the section joins; the entry text is only written, never typeset
  assert!(
    xml.contains(r#"<section inlist="toc file" xml:id="S1">"#),
    "{xml}"
  );
  assert!(!xml.contains("entry"), "{xml}");
}

/// An acronym's key is a name: `\acro{gcc_phat}` and `\ac{gcc_phat}` keep it whole (2003.05548, 2603.23723).
#[test]
fn acronym_keys_are_names() {
  let (log, xml) = convert_with(
    "\\documentclass{article}\\usepackage{acronym}\n\\begin{document}\\begin{acronym}\n\\acro{gcc_phat}[GCC-PHAT]{generalized \
     cross-correlation phase transform}\n\\end{acronym}Text \\ac{gcc_phat}.\\end{document}",
    None,
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    xml.contains(r#"<glossaryref inlist="acronym" key="gcc_phat" show="long"/>"#),
    "{xml}"
  );
  assert!(
    xml.contains("generalized cross-correlation phase transform"),
    "{xml}"
  );
}

/// A document's active `_` survives a raw xparse load (`\ExplSyntaxOff` restores what `\ExplSyntaxOn` saved,
/// expl3-code.tex:247-261; gb4e + tikzmark, 2408.09634).
#[test]
fn expl_syntax_load_restores_the_documents_catcodes() {
  let (log, xml) = convert_with(
    "\\documentclass{article}\\catcode`\\_=13 \\def_{U}\n\\usepackage{xparse}\\typeout{CAT=\\the\\catcode`\\_}\n\\begin{document}a_b\\end{document}",
    None,
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(log.contains("CAT=13"), "{log}");
  assert!(xml.contains(">aUb</p>"), "{xml}");
}

/// underscore.sty's `\AtBeginDocument` catcode runs in load order, so mathstyle's later one (catcode 12) wins
/// (2507.09194; pdflatex "a˙b" in OT1).
#[test]
fn underscore_catcode_hook_runs_in_load_order() {
  let (log, xml) = convert_with(
    "\\documentclass{article}\\usepackage{underscore}\\usepackage{mathstyle}\n\\begin{document}See a_b and $x_1$.\\end{document}",
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(xml.contains("See a˙b and"), "{xml}");
  // math keeps `_` a subscript
  assert!(xml.contains(r#"tex="x_{1}""#), "{xml}");
}

/// Code a begin-document hook adds to that hook runs at once, as LaTeX's one-time hooks do (`\UseOneTimeHook`,
/// latex.ltx:9512; sect02.rs `hook_use_once_tokens`): a nested `\AtBeginDocument`, and underscore.sty loaded from the
/// hook, whose own `\AtBeginDocument{\catcode`\_\active}` was dropped (77j review). pdflatex: "INNER and a_b.", 0 errors.
#[test]
fn begin_document_code_added_by_the_hook_runs() {
  let (log, xml) = convert_with(
    "\\documentclass{article}\n\\AtBeginDocument{\\AtBeginDocument{\\def\\inner{INNER}}\\usepackage{underscore}}\n\\begin{document}\n\\inner{} and a_b.\n\\end{document}",
    None,
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(squash(&xml).contains("INNER and a"), "{xml}");
  assert!(!xml.contains("<Math"), "{xml}");
}

/// A standalone sub-file's preamble runs for its definitions only; its begin-document code is not run: once the
/// document hooks are one-time hooks it would run in the middle of babel.sty (six undefined `\bbl@…`, 2405.01743).
/// pdflatex: "Main text. Sub text. After.", 0 errors.
#[test]
fn standalone_subpreamble_begin_document_code_is_not_run() {
  let (log, xml) = latexml::util::test::convert_files_with(
    "\\documentclass{article}\n\\usepackage{standalone}\n\\begin{document}\nMain text.\n\n\\input{pic}\n\nAfter.\n\\end{document}",
    &[(
      "pic.tex",
      "\\documentclass{standalone}\n\\usepackage[english]{babel}\n\\begin{document}\nSub text.\n\\end{document}",
    )],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  for text in ["Main text.", "Sub text.", "After."] {
    assert!(squash(&xml).contains(text), "{xml}");
  }
}

/// A theorem's `[within]` is a counter name (latex.ltx `\@xnthm`), read through `\csname` (1307.7314).
#[test]
fn theorem_within_is_a_counter_name() {
  let (log, xml) = convert_with(
    "\\documentclass{article}\\newcounter{todo_counter}\\newtheorem{t}{T}[todo_counter]\n\\begin{document}\\begin{t}x\\end{t}\\end{document}",
    None,
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(xml.contains("<tag>T 0.1</tag>"), "{xml}");
}

/// nomencl's sort prefix is only written for makeindex, kept as a string (2105.05016).
#[test]
fn nomenclature_prefix_is_a_string() {
  let (log, xml) = convert_with(
    "\\documentclass{article}\\usepackage{nomencl}\\makenomenclature\n\\begin{document}\\nomenclature[Enc_Aut]{$E$}{Energy}Text.\\printnomenclature\\end{document}",
    None,
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(xml.contains(r#"role="sort">Enc_Aut<Math"#), "{xml}");
  assert!(xml.contains("Energy"), "{xml}");
}

/// backref's citation hook `\Hy@backout` (backref.sty:457-482) is defined as backref defines it, so a document's
/// `\patchcmd{\Hy@backout}{Doc-Start}{\@currentHref}{}{\errmessage{…}}` succeeds (2405.13143, 2601.06492, 2004.14852).
#[test]
fn backref_citation_hook_is_patchable() {
  let (log, xml) = convert_with(
    "\\documentclass{article}\\usepackage[pagebackref]{hyperref}\\usepackage{etoolbox}\n\\makeatletter\
     \\patchcmd{\\Hy@backout}{Doc-Start}{\\@currentHref}{}{\\errmessage{I can't seem to patch backref}}\\makeatother\n\
     \\begin{document}\\section{A}Text \\cite{k1}.\n\\begin{thebibliography}{9}\\bibitem{k1} One.\\end{thebibliography}\\end{document}",
    None,
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(xml.contains("One."), "{xml}");
}
