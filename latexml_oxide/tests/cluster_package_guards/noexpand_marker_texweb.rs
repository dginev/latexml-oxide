//! Batch 56dd: the `\special_relax` no-expand family is tex.web §358's
//! marker — cmd `relax`, chr `no_expand_flag`, `cur_cs` = the shadowed
//! control sequence — made a persistent token. Thirteen probes pin every
//! observable against pdflatex: `\ifx` against `\relax` is false
//! (`no_expand_flag` ≠ 256) and between two markers true, `\edef`/`\let`
//! capture the plain token (§478/§1221), a marker matches its delimiter
//! (§392), `\if`/`\ifcat` see an active char (§506), and the two printing
//! primitives show `\relax` (§266) and the shadowed name (§472) — also for
//! a `\let` alias, which §1221 stores as the `relax` variant — where Perl
//! prints its internal `\special_relax` (divergence #238).

/// Every `<p>` of the probe, in pdflatex's words.
#[test]
fn fourteen_probes_match_pdflatex() {
  let tex = "\\documentclass{article}\n\\def\\foo{x}\\def\\bar{y}\\def~{z}\n\\begin{document}\n\\noindent\n\
c1:\\expandafter\\ifx\\noexpand\\foo\\relax T\\else F\\fi.\n\n\
c2:\\expandafter\\ifx\\noexpand\\undefinedcs\\relax T\\else F\\fi.\n\n\
c3:\\edef\\r{\\noexpand\\foo}\\meaning\\r.\n\n\
c4:\\expandafter\\meaning\\noexpand\\foo.\n\n\
c5:\\expandafter\\string\\noexpand\\foo.\n\n\
c6:\\def\\m#1\\foo{[#1]}\\expandafter\\m\\expandafter a\\noexpand\\foo.\n\n\
c7:\\expandafter\\let\\expandafter\\x\\noexpand\\foo \\ifx\\x\\relax T\\else F\\fi.\n\n\
c8:\\expandafter\\let\\expandafter\\y\\noexpand\\bar \\ifx\\x\\y T\\else F\\fi.\n\n\
c9:\\edef\\s{\\noexpand\\foo\\noexpand\\bar}\\meaning\\s.\n\n\
c10:\\if\\noexpand~\\noexpand~T\\else F\\fi.\n\n\
c11:\\ifcat\\noexpand~aT\\else F\\fi.\n\n\
c12:\\edef\\t{\\x}\\meaning\\t.\n\n\
c13:\\expandafter\\ifx\\noexpand\\foo\\noexpand\\bar T\\else F\\fi.\n\n\
c14:\\meaning\\x.\n\
\\end{document}\n";
  let (stderr, xml) = super::convert(tex, true);
  assert_eq!(super::error_count(&stderr), 0, "{stderr}");
  // `\` and `>` in text are OT1's glyphs (`“`, `¿`), as pdftotext reads
  // them off pdflatex's page.
  let expected = [
    "c1:F.",
    "c2:F.",
    "c3:macro:-¿“foo .",
    "c4:“relax.",
    "c5:“foo.",
    "c6:[a].",
    "c7:F.",
    "c8:T.",
    "c9:macro:-¿“foo “bar .",
    "c10:T.",
    "c11:F.",
    "c12:macro:-¿“x .",
    "c13:F.",
    "c14:“relax.",
  ];
  let paras: Vec<&str> = xml
    .split("<p>")
    .skip(1)
    .filter_map(|rest| rest.split_once("</p>").map(|(body, _)| body))
    .collect();
  assert_eq!(paras, expected, "the probe's paragraphs:\n{xml}");
}
