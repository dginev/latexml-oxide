//! Red/green guards for batch 64zf, diagnostics honesty stage 1 (user directive 2026-10-11): a problem of Error severity
//! is reported as an Error with Perl's `category:object`, the recovery kept. Where TeX errs and Perl is silent, a
//! recovery that yields good markup is a Warning. Each guard pins the exact diagnostic counts and a whole element.
//! Repros under `tools/perfect_kernel/repros/`.
use latexml::util::test::{assert_element, xml_element};

use super::perfect_kernel_batch46::{error_count, warning_count};

/// The log and XML of `tex` converted under ar5iv.
fn convert_ar5iv(tex: &str) -> (String, String) {
  latexml::util::test::convert_with(tex, Some("ar5iv.sty"))
}

/// The log lines carrying the diagnostic `kind` (`Error:unexpected:foo`, `Info:misdefined:utf-8`).
fn lines_with<'a>(log: &'a str, kind: &str) -> Vec<&'a str> {
  log.lines().filter(|l| l.contains(kind)).collect()
}

/// The core `xml` through `LaTeXML-html5.xsl` in-process, its source files in `source_directory` (the post stage reads
/// the `.bib` there), with the post stage's log.
fn post_logged(xml: &str, source_directory: &str) -> (String, String) {
  latexml_core::util::logger::bind_log();
  let destination = format!("{source_directory}/guard.html");
  let opts = latexml::post::PostOptions {
    pmml:                      true,
    cmml:                      false,
    keep_xmath:                false,
    stylesheet:                Some("resources/XSLT/LaTeXML-html5.xsl"),
    destination:               Some(&destination),
    source_directory:          Some(source_directory),
    site_directory:            None,
    search_paths:              &[],
    nodefaultresources:        true,
    css_files:                 &[],
    js_files:                  &[],
    noinvisibletimes:          false,
    plane1:                    true,
    hackplane1:                false,
    mathtex:                   false,
    url_style:                 latexml_post::crossref::UrlStyle::File,
    navigationtoc:             None,
    schemadocs:                false,
    split:                     false,
    split_xpath:               None,
    split_naming:              None,
    xslt_parameters:           &[],
    graphics_svg_threshold_kb: 0,
    graphicimages:             false,
    timestamp:                 None,
    icon:                      None,
    whatsout:                  latexml_post::extract::Whatsout::default(),
  };
  let out = latexml::post::run_post_processing(xml, &opts);
  (out, latexml_core::util::logger::flush_log())
}

/// A1/A2: the `.bib` reader reports Perl's two Errors — the malformed entry `b` (`Error:expected:}`, Pre/BibTeX.pm:248,
/// resynced and dropped as bibtex drops it) and the undefined macro `apj` (`Error:unexpected:apj`, BibTeX.pm:307-310,
/// its name kept). `month = jan` resolves through plain.bst's own `MACRO {jan}` (read by kpathsea), no error. Was: one
/// Warning, and nothing for `apj`. Repro index-bib/bib_parse_errors_are_reported.
#[test]
fn bib_parse_errors_are_reported() {
  let (log, _, (html, post_log)) = latexml::util::test::convert_bytes_files_then(
    include_bytes!(
      "../../../tools/perfect_kernel/repros/index-bib/bib_parse_errors_are_reported.tex"
    ),
    &[(
      "bib_parse_errors_are_reported.bib",
      include_bytes!(
        "../../../tools/perfect_kernel/repros/index-bib/bib_parse_errors_are_reported.bib"
      ),
    )],
    Some("ar5iv.sty"),
    |xml, source_dir| post_logged(xml, source_dir.to_str().expect("a UTF-8 path")),
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  assert_eq!(error_count(&post_log), 2, "{post_log}");
  assert_eq!(warning_count(&post_log), 2, "{post_log}");
  // Each Error is located at the `.bib`'s own line (Perl: `at String; line 0 col 1`).
  assert_eq!(
    lines_with(&post_log, "\tat bib_parse_errors_are_reported; line "),
    vec![
      "\tat bib_parse_errors_are_reported; line 10",
      "\tat bib_parse_errors_are_reported; line 4"
    ],
    "{post_log}"
  );
  assert_eq!(
    lines_with(&post_log, "Error:expected:}").len(),
    1,
    "{post_log}"
  );
  assert_eq!(
    lines_with(&post_log, "Error:unexpected:apj").len(),
    1,
    "{post_log}"
  );
  assert!(!post_log.contains("Error:unexpected:jan"), "{post_log}");
  assert_eq!(
    xml_element(&html, "li", &["id=\"bib.bib1\""]).unwrap_or_default(),
    "<li class=\"ltx_bibitem ltx_bib_article\" id=\"bib.bib1\"><span class=\"ltx_tag ltx_bib_key ltx_role_refnum ltx_tag_bibitem\">[1]</span><span class=\"ltx_bibblock\"><span class=\"ltx_text ltx_bib_author\">A. Author</span><span class=\"ltx_text ltx_bib_year\"> (2001)</span></span><span class=\"ltx_bibblock\"><span class=\"ltx_text ltx_bib_title\">Title a</span>.</span><span class=\"ltx_bibblock\"><span class=\"ltx_text ltx_bib_journal\">apj</span>.</span><span class=\"ltx_bibblock ltx_bib_cited\">Cited by: <a class=\"ltx_ref\" href=\"#p1.1\" title=\"\">p1.1</a>.</span></li>",
    "{html}"
  );
  assert_eq!(
    xml_element(&html, "li", &["id=\"bib.bib2\""]).unwrap_or_default(),
    "<li class=\"ltx_bibitem ltx_bib_article\" id=\"bib.bib2\"><span class=\"ltx_tag ltx_bib_key ltx_role_refnum ltx_tag_bibitem\">[2]</span><span class=\"ltx_bibblock\"><span class=\"ltx_text ltx_bib_author\">C. Author</span><span class=\"ltx_text ltx_bib_year\"> (2003)</span></span><span class=\"ltx_bibblock\"><span class=\"ltx_text ltx_bib_title\">Title c</span>.</span><span class=\"ltx_bibblock ltx_bib_cited\">Cited by: <a class=\"ltx_ref\" href=\"#p1.1\" title=\"\">p1.1</a>.</span></li>",
    "{html}"
  );
  assert!(
    xml_element(&html, "li", &["id=\"bib.bib3\""]).is_none(),
    "{html}"
  );
}

/// A3: an unknown color model is Perl's `Error:unexpected:<model>` "Unrecognized color model" (Color.pm:59-61); the text
/// keeps the current color. Was: black, silently. Repro
/// fonts-nfss/color_unknown_model_is_reported.
#[test]
fn color_unknown_model_is_reported() {
  let (log, xml) = convert_ar5iv(include_str!(
    "../../../tools/perfect_kernel/repros/fonts-nfss/color_unknown_model_is_reported.tex"
  ));
  assert_eq!(error_count(&log), 1, "{log}");
  assert_eq!(
    lines_with(&log, "Error:unexpected:foo Unrecognized color model 'foo'").len(),
    1,
    "{log}"
  );
  assert_eq!(warning_count(&log), 0, "{log}");
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    "<p xml:id=\"p1.1\">x z</p>",
  );
}

/// A4: `\end{document}` with a conditional still open warns as Perl does (latex_constructs.pool.ltxml:374-377), naming
/// the conditional on a tab-indented detail line (Perl's format for a message's extra lines); TeX logs "(\end occurred
/// when \iftrue on line 13 was incomplete)". The report is 65c's (`\lx@finalize@document`); this pins it. Repro
/// parameter-conditional/conditional_open_at_end_document.
#[test]
fn conditional_open_at_end_document() {
  let (log, xml) = convert_ar5iv(include_str!(
    "../../../tools/perfect_kernel/repros/parameter-conditional/conditional_open_at_end_document.tex"
  ));
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 1, "{log}");
  assert_eq!(
    lines_with(&log, "Warning:unexpected:\\end{document}").len(),
    1,
    "{log}"
  );
  assert_eq!(
    lines_with(&log, "Conditional \\iftrue started"),
    vec!["\tConditional \\iftrue started t; line 13 col 1 - line 13 col 8"],
    "{log}"
  );
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    "<p xml:id=\"p1.1\">Text.\nMore.</p>",
  );
}

/// A5: a line that is not UTF-8, with no inputenc declaring another encoding, is Perl's `Info:misdefined`
/// (Core/Mouth/file.pm:72-77), its bytes kept as Latin-1 text (OXIDIZED_DESIGN_DIVERGENCES #481). Was: silent. Repro
/// unicode-catcodes/invalid_utf8_byte_without_inputenc.
#[test]
fn invalid_utf8_byte_without_inputenc_is_an_info() {
  let (log, xml, ()) = latexml::util::test::convert_bytes_files_then(
    include_bytes!(
      "../../../tools/perfect_kernel/repros/unicode-catcodes/invalid_utf8_byte_without_inputenc.tex"
    ),
    &[],
    Some("ar5iv.sty"),
    |_, _| (),
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  assert_eq!(lines_with(&log, "Info:misdefined:utf-8").len(), 1, "{log}");
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    "<p xml:id=\"p1.1\">It\u{92}s a test\u{e9}.</p>",
  );
}

/// The same bytes under a declared Latin-1 inputenc are that encoding's text: no Info.
#[test]
fn latin1_inputenc_bytes_are_no_misdefined_info() {
  let (log, xml, ()) = latexml::util::test::convert_bytes_files_then(
    b"\\documentclass{article}\n\\usepackage[latin1]{inputenc}\n\\begin{document}\nA test\xe9.\n\\end{document}\n",
    &[],
    Some("ar5iv.sty"),
    |_, _| (),
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  assert!(lines_with(&log, "Info:misdefined").is_empty(), "{log}");
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    "<p xml:id=\"p1.1\">A test\u{e9}.</p>",
  );
}

/// Gullet overflow: an integer constant past 2^31 − 1 is TeX's "Number too big" (tex.web §445) and a dimension
/// reaching 2^30 sp is "Dimension too large" (§460), clamped as TeX clamps, reported as Warnings (the severity of
/// Perl's scan recoveries, Gullet.pm:904-1089); the clamp is +max_dimen, as §460 negates only for the scan's own sign.
/// Was: -200000000000.0pt, silently. Repro
/// expansion-primitives/dimension_overflow_max_dimen.
#[test]
fn number_and_dimension_overflow_clamp_and_warn() {
  let (log, xml) = convert_ar5iv(include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/dimension_overflow_max_dimen.tex"
  ));
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 2, "{log}");
  assert_eq!(
    lines_with(&log, "Warning:expected:<number> Number too big.").len(),
    1,
    "{log}"
  );
  assert_eq!(
    lines_with(&log, "Warning:expected:<dimen> Dimension too large.").len(),
    1,
    "{log}"
  );
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    "<p xml:id=\"p1.1\">B=[16383.99998pt]</p>",
  );
}

/// A count assignment takes TeX's `infinity` for a constant past it, and a hexadecimal one too (§445).
#[test]
fn integer_constant_overflow_is_infinity() {
  let (log, xml) = convert_ar5iv(
    "\\documentclass{article}\n\\begin{document}\n\\count255=99999999999 A=\\the\\count255;\n\\count255=\"FFFFFFFFF B=\\the\\count255.\n\\end{document}\n",
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 2, "{log}");
  assert_eq!(
    lines_with(&log, "Warning:expected:<number> Number too big.").len(),
    2,
    "{log}"
  );
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    "<p xml:id=\"p1.1\">A=2147483647; B=2147483647.</p>",
  );
}

/// babel's "not a shorthand" Error (babel.def:1558, errbabel.def:93-97) for `^` in `\shorthandoff*{~^}`, as Perl's raw
/// babel reports it; `~` is still restored and `^` stays a superscript. Repro babel-lang/shorthandoff_star_reads_the_star.
#[test]
fn shorthandoff_reports_a_character_that_is_no_shorthand() {
  let (log, xml) = convert_ar5iv(include_str!(
    "../../../tools/perfect_kernel/repros/babel-lang/shorthandoff_star_reads_the_star.tex"
  ));
  assert_eq!(error_count(&log), 1, "{log}");
  assert_eq!(
    lines_with(
      &log,
      "Error:latex:(babel) Package babel Error: I can't switch '^' on or off--not a shorthand"
    )
    .len(),
    1,
    "{log}"
  );
  assert_eq!(warning_count(&log), 0, "{log}");
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    "<p xml:id=\"p1.1\">XY a:b\nc\u{a0}d <Math mode=\"inline\" tex=\"x^{2}\" text=\"x ^ 2\" xml:id=\"p1.m1\"><XMath xml:id=\"p1.m1.1\"><XMApp xml:id=\"p1.m1.1.1\"><XMTok role=\"SUPERSCRIPTOP\" scriptpos=\"post1\"/><XMTok font=\"italic\" role=\"UNKNOWN\">x</XMTok><XMTok fontsize=\"70%\" meaning=\"2\" role=\"NUMBER\">2</XMTok></XMApp></XMath></Math></p>",
  );
}

/// `{\doublespace text}`: setspace.sty:542-547's `\begingroup` meets the `}` — TeX's "Extra }, or forgotten
/// \endgroup", one Error (`\lx@setspace@check`); the text is kept. Was: silent no-ops. Repro
/// boxes-groups/bare_doublespace_closed_by_brace (witness 2310.08233).
#[test]
fn bare_doublespace_closed_by_brace_is_an_error() {
  let (log, xml) = convert_ar5iv(include_str!(
    "../../../tools/perfect_kernel/repros/boxes-groups/bare_doublespace_closed_by_brace.tex"
  ));
  assert_eq!(error_count(&log), 1, "{log}");
  assert_eq!(
    lines_with(
      &log,
      "Error:unexpected:} Extra }, or forgotten \\endgroup: \\doublespace's"
    )
    .len(),
    1,
    "{log}"
  );
  assert_eq!(warning_count(&log), 0, "{log}");
  assert!(!log.contains("Anonymous String"), "{log}");
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p2.1\""],
    "<p xml:id=\"p2.1\">Text in group. After.</p>",
  );
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p3.1\""],
    "<p xml:id=\"p3.1\">Closing text.</p>",
  );
}

/// `\doublespace … \enddoublespace` as a switch pair, and a switch left open at the document level (TeX: a log line
/// only), give no diagnostic.
#[test]
fn doublespace_switch_pair_is_clean() {
  let (log, xml) = convert_ar5iv(
    "\\documentclass{article}\\usepackage{setspace}\n\\begin{document}\n\\begin{quote}\\doublespace\nText.\n\\enddoublespace\\end{quote}\n\\onehalfspace\nAfter.\n\\end{document}\n",
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  assert_element(
    &xml,
    "para",
    &["xml:id=\"p1\""],
    "<para xml:id=\"p1\"><quote xml:id=\"p1.1\"><p xml:id=\"p1.1.1\">Text.</p></quote><p xml:id=\"p1.2\">After.</p></para>",
  );
}

/// `\begin{quote}\onehalfspace Q.\end{quote}` twice: each `\end{quote}` meets the switch's `\begingroup` (pdflatex
/// errs at `\end{document}`), an Error each, and every word is kept as two sibling quotes. A real `\begingroup` there
/// now keeps every word too (65c, `perfect_kernel_batch65c::unclosed_nested_environments_keep_their_text`) but nests the
/// second quote inside the first; the checked no-op keeps the author's sibling structure (content and semantics ahead
/// of PDF fidelity). Review probe ss/two.tex.
#[test]
fn onehalfspace_in_quotes_keeps_every_word() {
  let (log, xml) = convert_ar5iv(
    "\\documentclass{article}\n\\usepackage{setspace}\n\\begin{document}\nBefore.\n\\begin{quote}\\onehalfspace Q1.\\end{quote}\nMiddle.\n\\begin{quote}\\onehalfspace Q2.\\end{quote}\nAfter.\n\\end{document}\n",
  );
  assert_eq!(error_count(&log), 2, "{log}");
  assert_eq!(
    lines_with(
      &log,
      "Error:unexpected:\\end{quote} \\end{quote} closes \\onehalfspace's \\begingroup"
    )
    .len(),
    2,
    "{log}"
  );
  assert_eq!(warning_count(&log), 0, "{log}");
  assert!(!log.contains("Anonymous String"), "{log}");
  assert_element(
    &xml,
    "para",
    &["xml:id=\"p1\""],
    "<para xml:id=\"p1\"><p xml:id=\"p1.1\">Before.</p><quote xml:id=\"p1.2\"><p xml:id=\"p1.2.1\">Q1.</p></quote><p xml:id=\"p1.3\">Middle.</p><quote xml:id=\"p1.4\"><p xml:id=\"p1.4.1\">Q2.</p></quote><p xml:id=\"p1.5\">After.</p></para>",
  );
}

/// The switch errs where pdflatex errs (review 2 probes ss1, ss2): an `\item`'s list ending with it open, a tabular
/// cell, a `\footnote` (read from a token list, so the report names no position), each one Error at its closer.
#[test]
fn setspace_switch_reports_where_tex_errs() {
  let (log, xml) = convert_ar5iv(
    "\\documentclass{article}\n\\usepackage{setspace}\n\\begin{document}\n\\begin{itemize}\\item Delta \\doublespace four. \\end{itemize}\n\\begin{tabular}{c} \\doublespace Gamma \\\\ three \\end{tabular}\n\\footnote{\\doublespace Delta four.}\n\\end{document}\n",
  );
  assert_eq!(error_count(&log), 3, "{log}");
  assert_eq!(
    lines_with(
      &log,
      "Error:unexpected:\\end{itemize} \\end{itemize} closes \\doublespace's \\begingroup"
    )
    .len(),
    1,
    "{log}"
  );
  assert_eq!(
    lines_with(&log, "Error:unexpected:} Extra }, or forgotten \\endgroup").len(),
    2,
    "{log}"
  );
  assert_eq!(warning_count(&log), 0, "{log}");
  assert!(!log.contains("Anonymous String"), "{log}");
  assert_element(
    &xml,
    "item",
    &["xml:id=\"S0.I1.i1\""],
    "<item xml:id=\"S0.I1.i1\"><tags><tag>•</tag><tag role=\"typerefnum\">1st item</tag></tags><para xml:id=\"S0.I1.i1.p1\"><p xml:id=\"S0.I1.i1.p1.1\">Delta four.</p></para></item>",
  );
}

/// No report where pdflatex has none (review 2 probes ss1, ss5, ss6, ss7): a switch in the preamble, in
/// `\AtBeginDocument`, at the document level after a float, inside a plain `\begingroup … \endgroup` (TeX: a log line
/// at `\end`; Perl errs, KNOWN_PERL_ERRORS #602), in `\newenvironment{dsp}{\doublespace}{\enddoublespace}`, and the `{doublespace}`
/// environment in a quote.
#[test]
fn setspace_switch_is_quiet_where_tex_is() {
  let (log, xml) = convert_ar5iv(
    "\\documentclass{article}\n\\usepackage{setspace}\n\\doublespace\n\\AtBeginDocument{\\onehalfspace}\n\\newenvironment{dsp}{\\doublespace}{\\enddoublespace}\n\\begin{document}\n\\begingroup\\onehalfspace Beta two.\\endgroup\nGamma three.\n\\begin{dsp}Alpha one.\\end{dsp}\n\\begin{quote}\\begin{doublespace}Delta.\\end{doublespace}\\end{quote}\n\\onehalfspace\nEpsilon.\n\\end{document}\n",
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  assert_element(
    &xml,
    "para",
    &["xml:id=\"p1\""],
    "<para xml:id=\"p1\"><p xml:id=\"p1.1\">Beta two.Gamma three. Alpha one.</p><quote xml:id=\"p1.2\"><p xml:id=\"p1.2.1\">Delta.</p></quote><p xml:id=\"p1.3\">Epsilon.</p></para>",
  );
}

/// `\doublespace` in `{abstract}`: the abstract's end meets its `\begingroup` (pdflatex: "\begin{abstract} ended by
/// \end{document}"), one Error; the abstract keeps its text. Repro boxes-groups/abstract_doublespace_group_end.
#[test]
fn abstract_doublespace_group_end_is_an_error() {
  let (log, xml) = convert_ar5iv(include_str!(
    "../../../tools/perfect_kernel/repros/boxes-groups/abstract_doublespace_group_end.tex"
  ));
  assert_eq!(error_count(&log), 1, "{log}");
  // The abstract is a captured body (`\lx@add@frontmatter@until`), whose frame no `\begin{abstract}` names: the
  // brace-group wording.
  assert_eq!(
    lines_with(
      &log,
      "Error:unexpected:} Extra }, or forgotten \\endgroup: \\doublespace's"
    )
    .len(),
    1,
    "{log}"
  );
  assert_eq!(warning_count(&log), 0, "{log}");
  assert!(!log.contains("Anonymous String"), "{log}");
  assert_element(
    &xml,
    "abstract",
    &["xml:id=\"abstract1\""],
    "<abstract inlist=\"toc\" name=\"Abstract\" xml:id=\"abstract1\"><p xml:id=\"abstract1.1\">An abstract.</p></abstract>",
  );
}

/// Without the `writefile` aspect, `\lst@BeginWriteFile`/`\lst@EndWriteFile` are undefined, as in listings (lstmisc.sty:30);
/// the environment's end code still runs. Repro boxes-groups/lstenv_writefile_hbox_pst-exa-doc.
#[test]
fn lst_writefile_commands_need_their_aspect() {
  let (log, xml) = convert_ar5iv(include_str!(
    "../../../tools/perfect_kernel/repros/boxes-groups/lstenv_writefile_hbox_pst-exa-doc.tex"
  ));
  assert_eq!(error_count(&log), 2, "{log}");
  assert_eq!(
    lines_with(&log, "Error:undefined:\\lst@BeginWriteFile").len(),
    1,
    "{log}"
  );
  assert_eq!(
    lines_with(&log, "Error:undefined:\\lst@EndWriteFile").len(),
    1,
    "{log}"
  );
  assert_eq!(warning_count(&log), 0, "{log}");
  assert_element(
    &xml,
    "para",
    &["xml:id=\"p1\""],
    "<para xml:id=\"p1\"><p xml:id=\"p1.1\">RESULTMARK</p></para>",
  );
}

/// With the aspect loaded (`\lstloadaspects{writefile}`) the commands exist: no diagnostic. Repro
/// boxes-groups/lstenv_writefile_aspect_loaded.
#[test]
fn lst_writefile_aspect_loaded_is_clean() {
  let (log, xml) = latexml::util::test::convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/boxes-groups/lstenv_writefile_aspect_loaded.tex"
    ),
    Some("[rawstyles,rawclasses,luatex]latexml.sty"),
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  assert_element(
    &xml,
    "para",
    &["xml:id=\"p1\""],
    "<para xml:id=\"p1\"><p>RESULTMARK</p></para>",
  );
}

/// A booktabs width that is no <dimen> reports Perl's `[Dimension]` Warnings (Gullet.pm:972), judged on its text so
/// nothing reaches the next row; both rules stay. Repro expansion-primitives/braced_length_tail_between_rows.
#[test]
fn booktabs_width_that_is_no_dimension_warns() {
  let (log, xml) = convert_ar5iv(include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/braced_length_tail_between_rows.tex"
  ));
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 2, "{log}");
  assert_eq!(
    lines_with(
      &log,
      "Warning:expected:<number> Missing number (Dimension), treated as zero."
    )
    .len(),
    2,
    "{log}"
  );
  assert_element(
    &xml,
    "td",
    &["xml:id=\"p1.1.2.1\""],
    "<td align=\"left\" border=\"t\" xml:id=\"p1.1.2.1\">e</td>",
  );
}

/// An `{lstlisting}` whose body was read ahead as a macro argument (`\parbox{3cm}{\begin{lstlisting}…}`) has its
/// `\end` already tokenized: no line holds it, the body is lost, and TeX runs out — an Error. Repro
/// kernel-alignment/pcol_lstlisting_parbox_CONTROL.
#[test]
fn lstlisting_read_ahead_as_an_argument_is_an_error() {
  let (log, xml) = convert_ar5iv(include_str!(
    "../../../tools/perfect_kernel/repros/kernel-alignment/pcol_lstlisting_parbox_CONTROL.tex"
  ));
  assert_eq!(error_count(&log), 1, "{log}");
  assert_eq!(
    lines_with(&log, "Error:expected:\\end{lstlisting}").len(),
    1,
    "{log}"
  );
  // The `\parbox` group the lost body leaves open, reported at `\end{document}`.
  assert_eq!(warning_count(&log), 1, "{log}");
  assert_eq!(
    lines_with(&log, "Warning:unexpected:\\end{document}").len(),
    1,
    "{log}"
  );
  assert_element(
    &xml,
    "inline-block",
    &["xml:id=\"p1.1.1\""],
    "<inline-block class=\"ltx_parbox\" vattach=\"middle\" width=\"85.4pt\" xml:id=\"p1.1.1\"><listing class=\"ltx_lstlisting\" dataencoding=\"base64\" datamimetype=\"text/plain\" xml:id=\"p1.1.1.1\"/></inline-block>",
  );
}

/// A raw-loaded file's `\endgroup` against an `\hbox` frame is Perl's one Error at that line (Stomach.pm:363-372), the
/// frame kept, so the file's own `\egroup`/`\endgroup` close what they opened. Was: popped silently, three Errors at
/// the file's end. Repro boxes-groups/raw_sty_endgroup_against_hbox.
#[test]
fn raw_sty_endgroup_against_hbox_is_one_error() {
  let (log, xml) = super::perfect_kernel_batch46::convert_files(
    include_str!(
      "../../../tools/perfect_kernel/repros/boxes-groups/raw_sty_endgroup_against_hbox.tex"
    ),
    &[(
      "raw_sty_endgroup_against_hbox.sty",
      include_str!(
        "../../../tools/perfect_kernel/repros/boxes-groups/raw_sty_endgroup_against_hbox.sty"
      ),
    )],
  );
  assert_eq!(error_count(&log), 1, "{log}");
  let error = lines_with(
    &log,
    "Error:unexpected:\\endgroup Attempt to close a group that switched to mode",
  );
  assert_eq!(error.len(), 1, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  assert!(
    log.contains("at raw_sty_endgroup_against_hbox; line 2"),
    "{log}"
  );
  assert_element(
    &xml,
    "para",
    &["xml:id=\"p1\""],
    "<para xml:id=\"p1\"><p>Hello</p></para>",
  );
}

/// A2 with REVTeX's implicit style: with no `\bibliographystyle`, REVTeX writes its society's `\@bibstyle`
/// (aps4-1.rtx:630 `apsrev\substyle@post`), so apsrev4-1.bst's `MACRO {prl}` defines `journal = prl`: no diagnostic,
/// and the journal is the style's (2603.22119's shape). A name no style defines is still an Error.
#[test]
fn revtex_implicit_bibstyle_defines_its_macros() {
  let tex = b"\\documentclass{revtex4-1}\n\\begin{document}\n\\cite{p,q}\n\\bibliography{refs}\n\\end{document}\n";
  let bib =
    b"@article{p, author = {P. Author}, title = {T}, journal = prl, volume = 1, year = 2001}\n\
@article{q, author = {Q. Author}, title = {U}, journal = nosuchjournal, volume = 2, year = 2002}\n";
  let (log, _, (html, post_log)) = latexml::util::test::convert_bytes_files_then(
    tex,
    &[("refs.bib", bib)],
    Some("ar5iv.sty"),
    |xml, source_dir| post_logged(xml, source_dir.to_str().expect("a UTF-8 path")),
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(error_count(&post_log), 1, "{post_log}");
  assert_eq!(
    lines_with(&post_log, "Error:unexpected:nosuchjournal").len(),
    1,
    "{post_log}"
  );
  assert!(!post_log.contains(":prl"), "{post_log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  assert_eq!(warning_count(&post_log), 0, "{post_log}");
  assert_eq!(
    xml_element(&html, "li", &["id=\"bib.bib1\""]).unwrap_or_default(),
    "<li class=\"ltx_bibitem ltx_bib_article\" id=\"bib.bib1\"><span class=\"ltx_tag ltx_bib_key ltx_role_refnum ltx_tag_bibitem\">[1]</span><span class=\"ltx_bibblock\"><span class=\"ltx_text ltx_bib_author\">P. Author</span><span class=\"ltx_text ltx_bib_year\"> (2001)</span></span><span class=\"ltx_bibblock\"><span class=\"ltx_text ltx_bib_title\">T</span>.</span><span class=\"ltx_bibblock\"><span class=\"ltx_text ltx_bib_journal\">Phys. Rev. Lett.</span> <span class=\"ltx_text ltx_bib_volume\">1</span>.</span><span class=\"ltx_bibblock ltx_bib_cited\">Cited by: <a class=\"ltx_ref\" href=\"#p1.1\" title=\"\">p1.1</a>.</span></li>",
    "{html}"
  );
}

/// listings drops the text after `\begin{lstlisting}` on its line, with its own warning (listings.sty:2142-2145);
/// options after a space are no dropped text (`\@ifnextchar[` skips the space; 1606.07935, pdflatex no warning).
#[test]
fn lstlisting_begin_line_text_is_dropped_with_a_warning() {
  let (log, xml) = convert_ar5iv(
    "\\documentclass{article}\\usepackage{listings}\n\\begin{document}\n\\begin{lstlisting} dropped\nx=1;\n\\end{lstlisting}\n\\begin{lstlisting} [frame=single]\ny=2;\n\\end{lstlisting}\n\\begin{lstlisting}% comment one\na=1;\n\\end{lstlisting}\n\\begin{lstlisting}[language=C] % comment two\nc=3;\n\\end{lstlisting}\n\\end{document}\n",
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 1, "{log}");
  assert_eq!(
    lines_with(
      &log,
      "Warning:latex:(Listings) Package Listings Warning: Text dropped after begin of listing"
    )
    .len(),
    1,
    "{log}"
  );
  // The first listing's text is "x=1;" alone; after a begin-line `%` comment the next line is kept (pdflatex drops
  // it with "Text dropped": content first), and a comment after the options is no dropped text.
  assert_element(
    &xml,
    "listing",
    &["xml:id=\"p1.1\""],
    "<listing class=\"ltx_lstlisting\" data=\"eD0xOw==\" dataencoding=\"base64\" datamimetype=\"text/plain\" xml:id=\"p1.1\"><listingline xml:id=\"lstnumberx1\"><text class=\"ltx_lst_identifier\" xml:id=\"lstnumberx1.1\">x</text>=1;</listingline></listing>",
  );
  assert_element(
    &xml,
    "listing",
    &["xml:id=\"p1.3\""],
    "<listing class=\"ltx_lstlisting\" data=\"YT0xOw==\" dataencoding=\"base64\" datamimetype=\"text/plain\" xml:id=\"p1.3\"><listingline xml:id=\"lstnumberx3\"><text class=\"ltx_lst_identifier\" xml:id=\"lstnumberx3.1\">a</text>=1;</listingline></listing>",
  );
  assert_element(
    &xml,
    "listing",
    &["xml:id=\"p1.4\""],
    "<listing class=\"ltx_lst_language_C ltx_lstlisting\" data=\"Yz0zOw==\" dataencoding=\"base64\" datamimetype=\"text/plain\" xml:id=\"p1.4\"><listingline xml:id=\"lstnumberx4\"><text class=\"ltx_lst_identifier\" xml:id=\"lstnumberx4.1\">c</text>=3;</listingline></listing>",
  );
  assert!(!xml.contains("dropped"), "{xml}");
}

/// A `.bst` value with TeX braces (`{"Comptes Rendus des S{\'e}ances"}`) is read to its closing quote, so the
/// macros after it stay defined: besjournals.bst's `andgeo` is "Annales de Geophysique", no Error.
#[test]
fn bst_macros_after_a_braced_value_resolve() {
  let tex = b"\\documentclass{article}\\usepackage{natbib}\n\\begin{document}\n\\cite{a}\n\\bibliographystyle{besjournals}\n\\bibliography{refs}\n\\end{document}\n";
  let bib = b"@article{a, author = {A. Author}, title = {T}, journal = andgeo, year = 2001}\n";
  let (log, _, (html, post_log)) = latexml::util::test::convert_bytes_files_then(
    tex,
    &[("refs.bib", bib)],
    Some("ar5iv.sty"),
    |xml, source_dir| post_logged(xml, source_dir.to_str().expect("a UTF-8 path")),
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(error_count(&post_log), 0, "{post_log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  assert_eq!(warning_count(&post_log), 0, "{post_log}");
  assert_eq!(
    xml_element(&html, "li", &["id=\"bib.bib1\""]).unwrap_or_default(),
    "<li class=\"ltx_bibitem ltx_bib_article\" id=\"bib.bib1\"><span class=\"ltx_tag ltx_bib_author-year ltx_role_refnum ltx_tag_bibitem\">Author (2001)</span><span class=\"ltx_bibblock\"><span class=\"ltx_text ltx_bib_author\">A. Author</span></span><span class=\"ltx_bibblock\"><span class=\"ltx_text ltx_bib_title\">T</span>.</span><span class=\"ltx_bibblock\"><span class=\"ltx_text ltx_bib_journal\">Annales de Geophysique</span>.</span><span class=\"ltx_bibblock ltx_bib_cited\">Cited by: <a class=\"ltx_ref\" href=\"#p1.1\" title=\"\">p1.1</a>.</span></li>",
    "{html}"
  );
}

/// Under xcolor a colour model is xcolor's: nicematrix's `\cellcolor[HTML]{…}` in a `\CodeBefore` is no unknown
/// model (review probe col/nm.tex; pdflatex 0 errors).
#[test]
fn xcolor_html_model_is_no_unknown_model() {
  let (log, xml) = convert_ar5iv(
    "\\documentclass{article}\n\\usepackage{xcolor}\n\\usepackage{nicematrix}\n\\begin{document}\n\\begin{NiceTabular}{cc}\n\\CodeBefore\n  \\cellcolor[HTML]{FFFF88}{1-1}\n  \\rowcolor[HTML]{EEEEEE}{2}\n\\Body\na & b \\\\\ne & f \\\\\n\\end{NiceTabular}\n\\end{document}\n",
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  assert_element(
    &xml,
    "tr",
    &["xml:id=\"p1.1.2\""],
    "<tr xml:id=\"p1.1.2\"><td align=\"center\" xml:id=\"p1.1.2.1\">e</td><td align=\"center\" xml:id=\"p1.1.2.2\">f</td></tr>",
  );
}

/// tabularray's booktabs library reads `\toprule[2pt,red]` as keys, not a width: no "Missing number" (review probe
/// bt/tblr.tex; pdflatex 0 errors).
#[test]
fn tabularray_booktabs_keys_are_no_width() {
  let (log, xml) = convert_ar5iv(
    "\\documentclass{article}\n\\usepackage{xcolor}\n\\usepackage{tabularray}\n\\UseTblrLibrary{booktabs}\n\\begin{document}\n\\begin{booktabs}{ll}\n\\toprule[2pt,red]\na & b \\\\\n\\midrule[blue]\nd & e \\\\\n\\bottomrule\n\\end{booktabs}\n\\end{document}\n",
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  assert_element(
    &xml,
    "tr",
    &["xml:id=\"p1.1.2\""],
    "<tr xml:id=\"p1.1.2\"><td align=\"left\" border=\"bb t\" xml:id=\"p1.1.2.1\">d</td><td align=\"left\" border=\"bb t\" xml:id=\"p1.1.2.2\">e</td></tr>",
  );
}

/// doi.sty's `\doi` reads a URL, so a DOI's `_` is no TeX error there: no `\doi` Warning (review probe doi/d1.tex).
#[test]
fn doi_sty_doi_is_no_warning() {
  let (log, xml) = convert_ar5iv(
    "\\documentclass{apa6}\n\\usepackage{doi}\n\\title{T}\\shorttitle{T}\\author{A}\\affiliation{B}\n\\begin{document}\n\\maketitle\nSee \\doi{10.1000/j_x.2002}.\n\\end{document}\n",
  );
  assert_eq!(error_count(&log), 0, "{log}");
  // apa6 has no binding and is read raw under the arXiv profile (rawclasses, 67v): no Warning; before 67v its one
  // Warning was the missing binding (OmniBus stood in).
  assert_eq!(warning_count(&log), 0, "{log}");
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    // doi.sty's own `\doitext` "doi:" (pdflatex: "See doi:10.1000/j_x.2002."), now that apa6 is read raw
    "<p xml:id=\"p1.1\">See doi:<ref class=\"ltx_href\" href=\"https://doi.org/10.1000/j_x.2002\">10.1000/j_x.2002</ref>.</p>",
  );
}

/// A booktabs width with a number and no unit is TeX's "Illegal unit of measure (pt inserted)" (Gullet.pm:969), with
/// the document's location; a register, or a factor and a unit, is no diagnostic (review probe bt/bt.tex).
#[test]
fn booktabs_width_shapes() {
  let (log, xml) = convert_ar5iv(
    "\\documentclass{article}\n\\usepackage{booktabs}\n\\begin{document}\n\\begin{tabular}{ll}\n\\toprule[\\heavyrulewidth]\na & b \\\\\n\\midrule[0.5\\lightrulewidth]\nc & d \\\\\n\\cmidrule[\\cmidrulewidth](lr){1-2}\ne & f \\\\\n\\midrule[1PT]\ng & h \\\\\n\\bottomrule[ 1.2Em]\n\\end{tabular}\n\\begin{tabular}{ll}\n\\toprule[2]\na & b \\\\\n\\bottomrule[thick]\n\\end{tabular}\n\\end{document}\n",
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 2, "{log}");
  assert_eq!(
    lines_with(
      &log,
      "Warning:expected:<unit> Illegal unit of measure (pt inserted)."
    )
    .len(),
    1,
    "{log}"
  );
  assert_eq!(
    lines_with(
      &log,
      "Warning:expected:<number> Missing number (Dimension), treated as zero."
    )
    .len(),
    1,
    "{log}"
  );
  assert!(!log.contains("Anonymous String"), "{log}");
  assert_element(
    &xml,
    "tr",
    &["xml:id=\"p1.1.4\""],
    "<tr xml:id=\"p1.1.4\"><td align=\"left\" border=\"bb t\" xml:id=\"p1.1.4.1\">g</td><td align=\"left\" border=\"bb t\" xml:id=\"p1.1.4.2\">h</td></tr>",
  );
}

/// One report per group, however many switches it holds (review 3 probes q5, q6): LaTeX gives one error for the
/// one environment left open, so three switches in `{center}` are one Error, and 120 in `{quote}` are one, not the
/// too-many-errors Fatal. The report names the innermost switch.
#[test]
fn setspace_switches_in_one_group_are_one_error() {
  let (log, xml) = convert_ar5iv(
    "\\documentclass{article}\n\\usepackage{setspace}\n\\begin{document}\n\\begin{center}\n\\doublespace One. \\onehalfspace Two. \\doublespace Three.\n\\end{center}\nAfter.\n\\end{document}\n",
  );
  assert_eq!(error_count(&log), 1, "{log}");
  assert_eq!(
    lines_with(
      &log,
      "Error:unexpected:\\end{center} \\end{center} closes \\doublespace's \\begingroup"
    )
    .len(),
    1,
    "{log}"
  );
  assert_eq!(warning_count(&log), 0, "{log}");
  assert_element(
    &xml,
    "para",
    &["xml:id=\"p1\""],
    "<para xml:id=\"p1\"><p align=\"center\" xml:id=\"p1.1\">One. Two. Three.</p></para>",
  );
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p2.1\""],
    "<p xml:id=\"p2.1\">After.</p>",
  );
  let many = format!(
    "\\documentclass{{article}}\n\\usepackage{{setspace}}\n\\begin{{document}}\n\\begin{{quote}}\n{}Q.\n\\end{{quote}}\n\\end{{document}}\n",
    "\\doublespace ".repeat(120)
  );
  let (log, xml) = convert_ar5iv(&many);
  assert_eq!(error_count(&log), 1, "{log}");
  assert!(!log.contains("Fatal:"), "{log}");
  assert_element(
    &xml,
    "quote",
    &["xml:id=\"p1.1\""],
    "<quote xml:id=\"p1.1\"><p xml:id=\"p1.1.1\">Q.</p></quote>",
  );
}

/// A switch still open as a plain `\begingroup` ends re-arms on the enclosing group (stomach.rs `pop_stack_frame`
/// honours the `\aftergroup` a group-end hook queues, as Perl's popStackFrame does): inside `{quote}` it is the quote's
/// one error (review 3 probe q2: pdflatex 1, Perl 2), while at the document level it stays quiet (ss7).
#[test]
fn setspace_switch_in_a_plain_group_rearms_on_the_enclosing_group() {
  let (log, xml) = convert_ar5iv(
    "\\documentclass{article}\n\\usepackage{setspace}\n\\begin{document}\nAlpha.\n\\begin{quote}\n\\begingroup\\onehalfspace Quoted text.\\endgroup\n\\end{quote}\nOmega.\n\\end{document}\n",
  );
  assert_eq!(error_count(&log), 1, "{log}");
  assert_eq!(
    lines_with(
      &log,
      "Error:unexpected:\\end{quote} \\end{quote} closes \\onehalfspace's \\begingroup"
    )
    .len(),
    1,
    "{log}"
  );
  assert_eq!(warning_count(&log), 0, "{log}");
  assert!(!log.contains("Anonymous String"), "{log}");
  assert_element(
    &xml,
    "quote",
    &["xml:id=\"p1.2\""],
    "<quote xml:id=\"p1.2\"><p xml:id=\"p1.2.1\">Quoted text.</p></quote>",
  );
  // The author's `\aftergroup` survives the group-end hook's own `\aftergroup` push (the re-arm emits no text, so
  // the order of the two is not observable, in TeX either): "A.Xa." and one Error.
  let (log, xml) = convert_ar5iv(
    "\\documentclass{article}\\usepackage{setspace}\\def\\X{Xa.}\n\\begin{document}\n\\begin{quote}\\begingroup\\aftergroup\\X\\doublespace A.\\endgroup\\end{quote}\nD.\n\\end{document}\n",
  );
  assert_eq!(error_count(&log), 1, "{log}");
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1.1\""],
    "<p xml:id=\"p1.1.1\">A.Xa.</p>",
  );
}

/// An environment whose end closes a box — a float's `\@endfloatbox`, `{minipage}` — is TeX's "Extra }" there
/// (review 3 probes q4, q7).
#[test]
fn setspace_switch_in_a_box_environment_is_extra_brace() {
  let (log, xml) = convert_ar5iv(
    "\\documentclass{article}\n\\usepackage{setspace}\n\\begin{document}\n\\begin{minipage}{3cm}\\doublespace Mini.\\end{minipage}\n\\begin{figure}\n\\onehalfspace Figure text.\n\\caption{Cap}\n\\end{figure}\n\\end{document}\n",
  );
  assert_eq!(error_count(&log), 2, "{log}");
  assert_eq!(
    lines_with(&log, "Error:unexpected:} Extra }, or forgotten \\endgroup").len(),
    2,
    "{log}"
  );
  assert_eq!(warning_count(&log), 0, "{log}");
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1.1.1\""],
    "<p xml:id=\"p1.1.1.1\">Mini.</p>",
  );
  assert_element(
    &xml,
    "p",
    &["xml:id=\"S0.F1.1\""],
    "<p xml:id=\"S0.F1.1\">Figure text.</p>",
  );
}

/// The box test is the frame, not a name list: an environment whose frame sets `\hsize` boxes its body, as
/// subcaption's `{subfigure}`, float.sty's `{algorithm}` and wrapfig's `{wrapfigure}` do — each TeX's "Extra }"
/// (review 4 probes r10, r5, r5b).
#[test]
fn setspace_box_environments_are_known_by_their_frame() {
  for (tex, (id, kept)) in [
    (
      "\\documentclass{article}\\usepackage{setspace,subcaption}\n\\begin{document}\n\\begin{figure}\\begin{subfigure}{.4\\textwidth}\\doublespace Sub text.\\caption{a}\\end{subfigure}\\caption{b}\\end{figure}\nD.\n\\end{document}\n",
      ("S0.F1.sf1.2", "<p xml:id=\"S0.F1.sf1.2\">Sub text.</p>"),
    ),
    (
      "\\documentclass{article}\\usepackage{setspace,algorithm}\n\\begin{document}\n\\begin{algorithm}\\doublespace Alg text.\\caption{c}\\end{algorithm}\nD.\n\\end{document}\n",
      (
        "algorithm1.2",
        "<p framed=\"topbottom\" xml:id=\"algorithm1.2\">Alg text.</p>",
      ),
    ),
    (
      "\\documentclass{article}\\usepackage{setspace,graphicx,wrapfig}\n\\begin{document}\n\\begin{wrapfigure}{r}{3cm}\\doublespace Wrap text.\\end{wrapfigure}\nD. more text here.\n\\end{document}\n",
      ("fig1.1", "<p xml:id=\"fig1.1\">Wrap text.</p>"),
    ),
  ] {
    let (log, xml) = convert_ar5iv(tex);
    assert_eq!(error_count(&log), 1, "{log}");
    assert_eq!(
      lines_with(&log, "Error:unexpected:} Extra }, or forgotten \\endgroup").len(),
      1,
      "{log}"
    );
    assert_element(&xml, "p", &[&format!("xml:id=\"{id}\"")], kept);
  }
}

/// A nested environment left open is ended by the enclosing environment's `\end`, which the report names (review 4
/// probe r14: pdflatex "\begin{itemize} on input line 5 ended by \end{quote}").
#[test]
fn setspace_report_names_the_enclosing_end() {
  let (log, _) = convert_ar5iv(
    "\\documentclass{article}\n\\usepackage{setspace}\n\\begin{document}\n\\begin{quote}\n\\begin{itemize}\\item \\doublespace x\n\\end{itemize}\ny\n\\end{quote}\nBody.\n\\end{document}\n",
  );
  assert_eq!(error_count(&log), 1, "{log}");
  assert_eq!(
    lines_with(&log, "(LaTeX: \\begin{itemize} … ended by \\end{quote})").len(),
    1,
    "{log}"
  );
}

/// A switch in a `\section` title reports like every other error in a title: twice, once for the heading and once
/// for the toc digestion (s3: pdflatex 1, ours 2). That double report is general, shared with Perl, and filed as RED
/// sectioning-frontmatter/section_title_errors_reported_twice; setspace does not special-case it. Distinct switches
/// read from one token list are distinct errors: a `\footnote` with two (review 5 probe p17: pdflatex 2 "Extra }") is
/// two, a title with two is four (p11; pdflatex 3).
#[test]
fn setspace_switch_in_a_section_title_reports_like_any_title_error() {
  let (log, _) = convert_ar5iv(
    "\\documentclass{article}\n\\usepackage{setspace}\n\\begin{document}\n\\section{\\doublespace T}\nZ.\n\\end{document}\n",
  );
  assert_eq!(error_count(&log), 2, "{log}");
  assert_eq!(lines_with(&log, "\tat t; line 4").len(), 2, "{log}");
  assert!(!log.contains("Anonymous String"), "{log}");
  let (log, xml) = convert_ar5iv(
    "\\documentclass{article}\\usepackage{setspace}\n\\begin{document}\nText.\\footnote{{\\doublespace a} {\\doublespace b}} More.\n\\end{document}\n",
  );
  assert_eq!(error_count(&log), 2, "{log}");
  assert_eq!(
    lines_with(&log, "Error:unexpected:} Extra }, or forgotten \\endgroup").len(),
    2,
    "{log}"
  );
  assert_element(
    &xml,
    "note",
    &["xml:id=\"footnote1\""],
    "<note mark=\"1\" role=\"footnote\" xml:id=\"footnote1\"><tags><tag>1</tag><tag role=\"refnum\">1</tag><tag role=\"typerefnum\">footnote 1</tag></tags>a b</note>",
  );
  let (log, _) = convert_ar5iv(
    "\\documentclass{article}\\usepackage{setspace}\n\\begin{document}\n\\section{{\\doublespace a} {\\doublespace b}}\nD.\n\\end{document}\n",
  );
  assert_eq!(error_count(&log), 4, "{log}");
}

/// An `\end…` closes the innermost switch of its own group; one met in a nested group closes nothing, and the switch's
/// group still reports — erring toward flagging. Each shape is one Error:
/// - r9 `\begingroup\enddoublespace\endgroup` (pdflatex 0: a false positive, RED
///   boxes-groups/doublespace_end_in_a_nested_plain_group; h5 two nested plain groups and h9 likewise);
/// - p9 a brace group, p10 an environment, h1 a plain group in a brace group, h2 in an environment, h3 an
///   `\endonehalfspace` in a plain group after two switches, h10 the switch closed twice (pdflatex 2, 2, 2, 2, 1, 2);
/// - h4 two switches and one `\endonehalfspace` in the quote itself (pdflatex 1): the outer `\doublespace` stays open
///   and names the report.
#[test]
fn setspace_end_closes_only_a_switch_of_its_own_group() {
  for (shape, name, kept) in [
    (
      "\\doublespace A. \\begingroup\\enddoublespace\\endgroup C.",
      "doublespace",
      "<p xml:id=\"p1.1.1\">A. C.</p>",
    ),
    (
      "\\doublespace A. \\begingroup\\begingroup\\enddoublespace\\endgroup\\endgroup C.",
      "doublespace",
      "<p xml:id=\"p1.1.1\">A. C.</p>",
    ),
    (
      "\\begingroup\\doublespace A.\\endgroup \\begingroup\\enddoublespace\\endgroup C.",
      "doublespace",
      "<p xml:id=\"p1.1.1\">A.C.</p>",
    ),
    (
      "\\doublespace A. {\\enddoublespace} C.",
      "doublespace",
      "<p xml:id=\"p1.1.1\">A.  C.</p>",
    ),
    (
      "\\doublespace A. \\begin{center}\\enddoublespace\\end{center} C.",
      "doublespace",
      "<p xml:id=\"p1.1.1\">A.</p><p xml:id=\"p1.1.2\">C.</p>",
    ),
    (
      "\\doublespace A. {\\begingroup\\enddoublespace\\endgroup} C.",
      "doublespace",
      "<p xml:id=\"p1.1.1\">A.  C.</p>",
    ),
    (
      "\\doublespace A. \\begin{center}\\begingroup\\enddoublespace\\endgroup\\end{center} C.",
      "doublespace",
      "<p xml:id=\"p1.1.1\">A.</p><p xml:id=\"p1.1.2\">C.</p>",
    ),
    (
      "\\doublespace\\onehalfspace A. \\begingroup\\endonehalfspace\\endgroup C.",
      "onehalfspace",
      "<p xml:id=\"p1.1.1\">A. C.</p>",
    ),
    (
      "\\doublespace A. \\begingroup\\enddoublespace\\endgroup B. \\begingroup\\enddoublespace\\endgroup C.",
      "doublespace",
      "<p xml:id=\"p1.1.1\">A. B. C.</p>",
    ),
    (
      "\\doublespace\\onehalfspace A. \\endonehalfspace C.",
      "doublespace",
      "<p xml:id=\"p1.1.1\">A. C.</p>",
    ),
  ] {
    let (log, xml) = convert_ar5iv(&format!(
      "\\documentclass{{article}}\\usepackage{{setspace}}\n\\begin{{document}}\n\\begin{{quote}}{shape}\\end{{quote}}\nD.\n\\end{{document}}\n"
    ));
    assert_eq!(error_count(&log), 1, "{shape}\n{log}");
    assert_eq!(warning_count(&log), 0, "{shape}\n{log}");
    assert_eq!(
      lines_with(
        &log,
        &format!("Error:unexpected:\\end{{quote}} \\end{{quote}} closes \\{name}'s \\begingroup")
      )
      .len(),
      1,
      "{shape}\n{log}"
    );
    assert_element(
      &xml,
      "quote",
      &["xml:id=\"p1.1\""],
      &format!("<quote xml:id=\"p1.1\">{kept}</quote>"),
    );
    assert_element(&xml, "p", &["xml:id=\"p1.2\""], "<p xml:id=\"p1.2\">D.</p>");
  }
}

/// A plain `\begingroup` holding switches ends: its `\endgroup` closes the innermost switch's group and the plain group
/// stands open in its place, so every switch of the group re-arms on the enclosing one, outermost first (review 7
/// probes): k1 `\begingroup\doublespace\onehalfspace A.\endgroup \enddoublespace` leaves `\doublespace` open (pdflatex
/// 1); k3 closes both after the group, k2 one switch, k4 two nested plain groups (pdflatex 0 each). The spaces after
/// `\endgroup` and `\enddoublespace` are a control word's: "A.C.".
#[test]
fn setspace_plain_group_rearms_every_switch() {
  for (shape, errors, kept) in [
    (
      "\\begingroup\\doublespace\\onehalfspace A.\\endgroup \\enddoublespace C.",
      1,
      "<p xml:id=\"p1.1.1\">A.C.</p>",
    ),
    (
      "\\begingroup\\doublespace\\onehalfspace A.\\endgroup \\endonehalfspace\\enddoublespace C.",
      0,
      "<p xml:id=\"p1.1.1\">A.C.</p>",
    ),
    (
      "\\begingroup\\doublespace A.\\endgroup \\enddoublespace C.",
      0,
      "<p xml:id=\"p1.1.1\">A.C.</p>",
    ),
    (
      "\\begingroup\\begingroup\\doublespace A.\\endgroup\\endgroup \\enddoublespace C.",
      0,
      "<p xml:id=\"p1.1.1\">A.C.</p>",
    ),
  ] {
    let (log, xml) = convert_ar5iv(&format!(
      "\\documentclass{{article}}\\usepackage{{setspace}}\n\\begin{{document}}\n\\begin{{quote}}{shape}\\end{{quote}}\nD.\n\\end{{document}}\n"
    ));
    assert_eq!(error_count(&log), errors, "{shape}\n{log}");
    assert_eq!(warning_count(&log), 0, "{shape}\n{log}");
    assert_eq!(
      lines_with(
        &log,
        "Error:unexpected:\\end{quote} \\end{quote} closes \\doublespace's \\begingroup"
      )
      .len(),
      errors,
      "{shape}\n{log}"
    );
    assert_element(
      &xml,
      "quote",
      &["xml:id=\"p1.1\""],
      &format!("<quote xml:id=\"p1.1\">{kept}</quote>"),
    );
    assert_element(&xml, "p", &["xml:id=\"p1.2\""], "<p xml:id=\"p1.2\">D.</p>");
  }
}

/// A group abandoned at a bounded body's terminal (stomach.rs `abandon_stack_frame`, from
/// `until_terminal_inside_group`) discards its `\aftergroup` list, including what its group-end hooks queue: a switch
/// in a plain group inside an unbalanced `{abstract}` brace queues its re-arm as that group is abandoned at
/// `\end{abstract}`, and the re-arm does not run — were it run, the switch would re-arm on the enclosing `{quote}` /
/// `{minipage}` and report its end.
#[test]
fn abandoned_group_discards_hook_queued_aftergroup() {
  for (begin, end, id) in [
    ("quote}", "quote", "p1.1.1"),
    ("minipage}{5cm}", "minipage", "p1.1.1.1"),
  ] {
    let (log, xml) = convert_ar5iv(&format!(
      "\\documentclass{{article}}\\usepackage{{setspace}}\n\\begin{{document}}\n\\begin{{{begin}\n\\begin{{abstract}}{{\\begingroup\\doublespace Abs text.\\end{{abstract}}\nQ.\n\\end{{{end}}}\nD.\n\\end{{document}}\n"
    ));
    assert_eq!(error_count(&log), 0, "{log}");
    assert_eq!(warning_count(&log), 1, "{log}");
    assert_eq!(
      lines_with(&log, "inside 2 open group(s): closing them").len(),
      1,
      "{log}"
    );
    assert_element(
      &xml,
      "abstract",
      &["xml:id=\"abstract1\""],
      "<abstract inlist=\"toc\" name=\"Abstract\" xml:id=\"abstract1\"><p xml:id=\"abstract1.1\">Abs text.</p></abstract>",
    );
    assert_element(
      &xml,
      "p",
      &[&format!("xml:id=\"{id}\"")],
      &format!("<p xml:id=\"{id}\">Q.</p>"),
    );
  }
}

/// A `%` that is not a comment character (doc.sty's `\MakePercentIgnore`, catcode 9) is text on the begin line, and
/// listings drops it with its warning; the next line is the listing's first (pdflatex: 1 warning, "x=1;" kept).
#[test]
fn lstlisting_begin_line_percent_that_is_no_comment_is_dropped_text() {
  let (log, xml) = convert_ar5iv(
    "\\documentclass{article}\\usepackage{doc,listings}\n\\begin{document}\n\\MakePercentIgnore\n\\begin{lstlisting}% dropped\nx=1;\n\\end{lstlisting}\n\\MakePercentComment\nD.\n\\end{document}\n",
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 1, "{log}");
  assert_eq!(
    lines_with(
      &log,
      "Warning:latex:(Listings) Package Listings Warning: Text dropped after begin of listing"
    )
    .len(),
    1,
    "{log}"
  );
  assert_element(
    &xml,
    "listing",
    &["xml:id=\"p1.1\""],
    "<listing class=\"ltx_lstlisting\" data=\"eD0xOw==\" dataencoding=\"base64\" datamimetype=\"text/plain\" xml:id=\"p1.1\"><listingline xml:id=\"lstnumberx1\"><text class=\"ltx_lst_identifier\" xml:id=\"lstnumberx1.1\">x</text>=1;</listingline></listing>",
  );
}
