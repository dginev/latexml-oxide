//! The case changer (`\MakeUppercase`, `\MakeLowercase`, `\MakeTitlecase`)
//! honours l3text's case-change declarations, checking each CS before it
//! expands it: the exclusion list, `\DeclareCaseChangeEquivalent` (stored in
//! `\l__text_case_<cs>_tl`, expl3-code.tex:36952-36956), `\CaseSwitch`, and the
//! letter-like table from `\@uclclist` (expl3-code.tex:37926-37988). textalpha
//! declares `\CaseSwitch` equivalents for the breathings (textalpha.sty:213-214)
//! and extends `\@uclclist` with the Greek letter commands
//! (greek-fontenc.def:332-406). Expected output is lualatex's.
//! Witnesses: greek-fontenc char-list, textalpha-doc (lualatex).
use latexml::util::test::assert_element;

use super::perfect_kernel_batch46::{
  convert, convert_files, convert_with, error_count, warning_count,
};

const LUATEX: &str = "[rawstyles,rawclasses,luatex]latexml.sty";

const GREEK_ACCENT: &str = include_str!(
  "../../../tools/perfect_kernel/repros/unicode-catcodes/case_change_greek_accent_char.tex"
);

const LETTERLIKE: &str = include_str!(
  "../../../tools/perfect_kernel/repros/unicode-catcodes/case_change_letterlike_table.tex"
);

/// `\>` is replaced by its `\CaseSwitch` equivalent (upper: `\LGR@hiatus`,
/// which drops the breathing) instead of being expanded into
/// `\add@unicode@accent{"0313}{\`}`, whose accent took `\char` as its argument
/// ("Missing number", `”0313` printed); `\accpsili` maps to `\LGR@hiatus`
/// through the letter-like table.
#[test]
fn declared_equivalent_is_substituted_before_expansion() {
  let (stderr, xml) = convert_with(GREEK_ACCENT, Some(LUATEX));
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(&xml, "p", &[], "<p>[Ὰ] [X]</p>");
}

/// The letter-like table maps `\textalpha`↔`\textAlpha` and `\aa`→`\AA`,
/// first-wins (`\textEpsilon`→`\textepsilon`); an equivalent is found after a
/// macro expands to it; `\CaseSwitch` in title case keeps the rest of the word;
/// `\label` keeps its key.
#[test]
fn letterlike_table_and_title_case_switch() {
  let (stderr, xml) = convert_with(LETTERLIKE, Some(LUATEX));
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  // `\>α` is α + U+0313 COMBINING COMMA ABOVE (the psili accent, unnormalized).
  let psili_alpha = "\u{3b1}\u{313}";
  assert_element(
    &xml,
    "document",
    &[],
    &format!(
      r#"<document xmlns="http://dlmf.nist.gov/LaTeXML" labels="LABEL:ab" xml:id="id1">
  <resource src="LaTeXML.css" type="text/css"/>
  <resource src="ltx-article.css" type="text/css"/>
  <para xml:id="p1">
    <p>[Α] [α] [ε]
[Å] [Αβ]
[Ὰ] [{psili_alpha}] [{psili_alpha}]
[A]
[ABC <emph font="italic">DEF</emph> GHI] [abc <emph font="italic">def</emph>]</p>
  </para>
</document>"#
    ),
  );
}

/// Control: ordinary text, a robust command, math and the kernel's
/// `\@uclclist` letters are cased as before (pdflatex's output).
#[test]
fn plain_text_case_change_unchanged() {
  let (stderr, xml) = convert(
    r"\documentclass{article}
\begin{document}
[\MakeUppercase{abc \emph{def} ghi $x$ \ae\ss}] [\MakeLowercase{ABC \textbf{DEF} \OE}] [\MakeTitlecase{abc def}]
\end{document}
",
    true,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "p",
    &[],
    r#"<p>[ABC <emph font="italic">DEF</emph> GHI <Math mode="inline" tex="x" text="x" xml:id="p1.m1">
        <XMath>
          <XMTok font="italic" role="UNKNOWN">x</XMTok>
        </XMath>
      </Math> ÆSS] [abc <text font="bold">def</text> œ] [Abc def]</p>"#,
  );
}

/// The changer reads each command unexpanded, and crosses the end of an input
/// level into the rest of its argument as `read_x_token` does: a `\scantokens`
/// pseudo-file or a TeX-form `\input` file ending inside the argument must not
/// end the case change (and leave its mouth open, dropping what follows).
/// pdflatex's l3text cannot do this at all ("File ended while scanning use of
/// `\__text_expand_loop:w`"); the tip and Perl case the whole argument.
#[test]
fn case_change_crosses_input_level_ends() {
  let (stderr, xml) = convert_files(
    r"\documentclass{article}
\begin{document}
\MakeUppercase{\scantokens{abc} def} ghi

\MakeUppercase{\input title rest} after
\end{document}
",
    &[("title.tex", "titleword more\n")],
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(&xml, "p", &[], "<p>ABC  DEF ghi</p>");
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p2""#],
    r#"<para xml:id="p2">
  <p>TITLEWORD MORE REST after</p>
</para>"#,
  );
}

/// An excluded command keeps everything before its braced argument in place,
/// case-changed (l3text `\__text_change_case_exclude:nnnNw`): natbib's
/// `\cite*{k}` keeps its star and `\cite[see][p.~5]{k}` both optional
/// arguments, and the key is never cased. pdflatex: "Knuth (1984)" and
/// "(SEE Knuth, 1984, P. 5)".
#[test]
fn excluded_command_keeps_what_precedes_its_argument() {
  let (stderr, xml) = convert(
    r"\documentclass{article}
\usepackage{natbib}
\begin{document}
A \MakeUppercase{\cite*{k}} B \MakeUppercase{\cite[see][p.~5]{k}}

\begin{thebibliography}{1}
\bibitem[Knuth(1984)]{k} D. Knuth. The TeXbook.
\end{thebibliography}
\end{document}
",
    true,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  // `p.~5`: the tie is U+00A0.
  let tie = '\u{a0}';
  assert_element(
    &xml,
    "p",
    &[],
    &format!(
      r#"<p>A <cite class="ltx_citemacro_cite"><bibref bibrefs="k" separator=";" show="FullAuthors Phrase1YearPhrase2" yyseparator=",">
          <bibrefphrase>(</bibrefphrase>
          <bibrefphrase>)</bibrefphrase>
        </bibref></cite> B <cite class="ltx_citemacro_cite">(SEE <bibref bibrefs="k" separator=";" show="AuthorsPhrase1Year" yyseparator=",">
          <bibrefphrase>, </bibrefphrase>
        </bibref>, P.{tie}5)</cite></p>"#
    ),
  );
}
