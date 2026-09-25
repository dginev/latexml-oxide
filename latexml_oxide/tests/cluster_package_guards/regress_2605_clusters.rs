use super::perfect_kernel_batch46::{convert_with, error_count, warning_count};

/// hyperref: batch 55b added a `\hyper@makecurrent` noop, which makes
/// pgfplots take its PDF-anchor branch and call `\hyper@anchorstart`/
/// `\hyper@anchorend` — undefined until the companion NoHyper-shape defs were
/// added (hyperref.sty:6152-6154). A pgfplots crossref `\label` under
/// hyperref must not error, and the plot must still render (~39 papers).
#[test]
fn hyperref_pgfplots_label_anchor_no_error() {
  let tex = r"\documentclass{article}
\usepackage{hyperref}\usepackage{pgfplots}\pgfplotsset{compat=1.18}
\begin{document}
\begin{tikzpicture}\begin{axis}
\addplot coordinates {(0,0) (1,1)}; \label{p1}
\end{axis}\end{tikzpicture}\ref{p1}
\end{document}
";
  let (stderr, xml) = convert_with(tex, Some("ar5iv.sty"));
  assert!(!stderr.contains("hyper@anchorstart"), "{stderr}");
  assert!(!stderr.contains("hyper@anchorend"), "{stderr}");
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("<svg:svg"),
    "the pgfplots axis must still render\n{xml}"
  );
}

/// biblatex-ieee: `style=ieee`/`ieee-comp` raw-loaded `ieee.cbx`, which
/// `\patchcmd`s bibmacros our `\newbibmacro` noop never defines, so
/// biblatex-ieee raised its own "Failed to update citation style" (~53
/// papers, error not warning). The IEEE styles are now in NATIVE_STYLES
/// (skipped, native pipeline renders the bibliography).
#[test]
fn biblatex_ieee_comp_style_no_error() {
  let tex = r"\documentclass{article}
\usepackage[backend=biber,style=ieee-comp]{biblatex}
\begin{filecontents}{\jobname.bib}
@article{a, author={A. Author}, title={T}, journal={J}, year={2020}}
\end{filecontents}
\addbibresource{\jobname.bib}
\begin{document}
Text~\cite{a}.
\printbibliography
\end{document}
";
  let (stderr, xml) = convert_with(tex, Some("ar5iv.sty"));
  assert!(
    !stderr.contains("Failed to update citation style"),
    "{stderr}"
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  // The bibliography itself needs biber's .bbl (LaTeXML does not run biber),
  // so a self-contained fixture renders no bibitem; the regression was the
  // spurious biblatex-ieee errors, and the document must still complete.
  assert!(
    xml.contains("</document>"),
    "the document must complete\n{xml}"
  );
}

/// jmlr-family conference classes (colt/midl/hld) are OmniBus-skipped, so
/// their `\Xauthor` wrapper was undefined and the jmlr `\addr
/// Until:\lx@jmlr@endaddr` scan ran to EOF (`Fatal:Mouth:EoF`, ~18 papers
/// with NO output). The class bindings now define the wrapper, routing
/// through the structured-author path that lays the sentinel and bounds the
/// scan. The class file need not be on disk (registry dispatch).
#[test]
fn jmlr_conference_author_bounds_the_addr_scan() {
  let tex = r"\documentclass{midl}
\midlauthor{\Name{Alice Smith} \Email{a@x.edu}\\ \addr University A}
\title{T}
\begin{document}
\maketitle
Body.
\end{document}
";
  let (stderr, xml) = convert_with(tex, Some("ar5iv.sty"));
  assert_eq!(stderr.matches("Fatal:").count(), 0, "{stderr}");
  assert!(!stderr.contains("lx@jmlr@endaddr"), "{stderr}");
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains(r#"role="affiliation""#),
    "the \\addr block must land as a bounded affiliation, not eat the doc\n{xml}"
  );
}

/// tipa: the binding raw-loads tipa.sty; on a trimmed host (no tipa.sty)
/// `\textipa` was undefined (~12 papers). An idempotent native fallback
/// keeps `\textipa` defined either way. Smoke test (host has tipa.sty, so
/// the raw path also defines it): `\textipa{...}` must not error.
#[test]
fn tipa_textipa_is_defined() {
  let tex = r"\documentclass{article}
\usepackage{tipa}
\begin{document}
\textipa{/S/}
\end{document}
";
  let (stderr, _xml) = convert_with(tex, Some("ar5iv.sty"));
  assert!(!stderr.contains("undefined:\\textipa"), "{stderr}");
  assert_eq!(error_count(&stderr), 0, "{stderr}");
}

/// PMLR sibling proceedings classes bundled by authors — l4dc2026, neus2025 —
/// are byte-identical to colt202x (`\LoadClass[pmlr]{jmlr}` +
/// `\coltauthor`→`\author`) but were unregistered, so OmniBus left
/// `\coltauthor` undefined and the downstream jmlr `\addr` scan ran to EOF
/// (`Fatal:Mouth:EoF`, no output). Registry dispatch to colt2024_cls now
/// defines `\coltauthor`; the class file need not be on disk.
#[test]
fn l4dc_neus_pmlr_classes_define_coltauthor() {
  for cls in ["l4dc2026", "neus2025"] {
    let tex = format!(
      r"\documentclass{{{cls}}}
\coltauthor{{\Name{{Alice Smith}} \Email{{a@x.edu}}\\ \addr University A}}
\title{{T}}
\begin{{document}}
\maketitle
Body.
\end{{document}}
"
    );
    let (stderr, xml) = convert_with(&tex, Some("ar5iv.sty"));
    assert!(
      !stderr.contains("undefined:\\coltauthor"),
      "{cls}: \\coltauthor must be defined\n{stderr}"
    );
    assert_eq!(stderr.matches("Fatal:").count(), 0, "{cls}: {stderr}");
    assert_eq!(error_count(&stderr), 0, "{cls}: {stderr}");
    assert!(
      xml.contains(r#"role="affiliation""#),
      "{cls}: the \\addr block must land as a bounded affiliation\n{xml}"
    );
    assert!(
      xml.contains("Alice Smith"),
      "{cls}: the author name must reach the output\n{xml}"
    );
  }
}

/// `\g@addto@macro\normalsize{...}` (the common display-skip idiom, ~6
/// article papers, e.g. 2605.04771) appended to the `\normalsize` font-switch
/// *primitive*. The former raw-`\def` binding `\xdef`'d the primitive token
/// verbatim into `\gdef\normalsize{\normalsize ...}` — a self-reference that
/// tripped `recursion:\normalsize`. Binding `\g@addto@macro` as a
/// non-expandable `DefPrimitive` routed through `AddToMacro!` restores Perl's
/// expandability guard (Package.pm:2534): appending to a non-expandable
/// target warns and ignores, matching Perl's exact output (0 errors + one
/// `unexpected:\normalsize` warning).
#[test]
fn g_addto_macro_on_normalsize_primitive_warns_not_recurses() {
  let tex = r"\documentclass[12pt]{article}
\makeatletter
\g@addto@macro\normalsize{\setlength\abovedisplayskip{1mm}}
\makeatother
\begin{document}
Hello \normalsize world.
\end{document}
";
  let (stderr, xml) = convert_with(tex, Some("ar5iv.sty"));
  assert!(
    !stderr.contains("recursion:\\normalsize"),
    "\\normalsize must not self-recurse\n{stderr}"
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(stderr.matches("Fatal:").count(), 0, "{stderr}");
  // Perl emits exactly this warning (the expandability guard fired, so the
  // primitive was left intact rather than silently mangled).
  assert!(
    stderr.contains("is not an expandable control sequence"),
    "the append to the \\normalsize primitive must warn-and-ignore\n{stderr}"
  );
  assert!(
    xml.contains("world"),
    "the body after \\normalsize must survive\n{xml}"
  );
}

/// `{bibunit}` (used directly and by apxproof's deferred appendix bodies) had
/// no vertical bound mode, so `$$…$$` inside it was not recognized as display
/// math: the body degraded to text and a subscript errored `unexpected:_`,
/// losing the equation's semantic markup (witness 2605.02787, 5 errors).
/// Giving `{bibunit}` `mode => "internal_vertical"` (the `{center}` precedent)
/// restores display math. A semantic-markup fix, not just an error suppression.
#[test]
fn bibunit_keeps_display_math_semantic() {
  let tex = r"\documentclass{article}
\usepackage{bibunits}
\begin{document}
\begin{bibunit}
$$ X_{i} = a $$
\end{bibunit}
\end{document}
";
  let (stderr, xml) = convert_with(tex, Some("ar5iv.sty"));
  assert!(!stderr.contains("unexpected:_"), "{stderr}");
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  // The `$$…$$` must open a SEMANTIC display-math element (not `<p>` text),
  // and `X_{i}` must parse as a real subscript — the exact degradation the
  // mode fix repairs (without it the `_` errors and no subscript forms).
  assert!(
    xml.contains(r#"<Math mode="display""#),
    "the $$…$$ inside bibunit must open a display <Math> element, not text\n{xml}"
  );
  assert!(
    xml.contains(r#"role="SUBSCRIPTOP""#),
    "X_{{i}} must parse as a real subscript, not degrade to text/error\n{xml}"
  );
}

/// amsmath alignment (`flalign*`/`align`) builds a `<MathFork>` per cell; a
/// cell whose content is a trivial "not really math" node (frege's
/// `\rule`-based `\Fcontent`, an `inline-block`, or multiple text runs) used
/// to unwrap straight under `<MathFork>`, which the schema forbids (MathFork
/// model = (Math|text),MathBranch*) — schema-invalid XML in ~14 logic/proof
/// manuals (frege, principia, natded, …). The surgical guard keeps the
/// `<Math>` wrapper for the invalid shapes while still unwrapping the valid
/// single-`<text>` cell. A semantic-markup (schema-validity) fix, batch 56eg.
#[test]
fn markedasmath_rule_cell_stays_wrapped_under_mathfork() {
  let tex = r"\documentclass{article}
\usepackage{amsmath}
\begin{document}
\begin{flalign*}
&\mbox{a} & &\rule[3.8pt]{20pt}{0.5pt}\hskip3pt & &\mbox{b}\\
\end{flalign*}
\end{document}
";
  let (stderr, xml) = convert_with(tex, Some("ar5iv.sty"));
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(stderr.matches("Fatal:").count(), 0, "{stderr}");
  // Whitespace-insensitive view of the element sequence.
  let compact = xml.split_whitespace().collect::<Vec<_>>().join(" ");
  // The `\rule` cell must NOT be a bare <MathFork> child (the schema
  // violation): its <Math> wrapper is kept instead.
  assert!(
    !compact.contains("<MathFork> <rule"),
    "a bare <rule> directly under <MathFork> is schema-invalid; keep the <Math> wrapper\n{xml}"
  );
  // The `\rule` cell keeps its <Math> wrapper (a valid MathFork first child).
  assert!(
    compact.contains("<MathFork> <Math"),
    "the rule cell must keep its <Math> wrapper as a valid MathFork first child\n{xml}"
  );
  // Surgical, not over-applied: the valid text-only cells STILL unwrap to a
  // first-child <text> (Perl's behavior, which IS valid there) rather than
  // also getting wrapped.
  assert!(
    compact.contains("<MathFork> <text class=\"ltx_markedasmath\""),
    "the valid single-text cell must still unwrap to a <text> first child\n{xml}"
  );
}

/// Companion boundary case (batch 56eg over-broad-guard fix; mathtools_test
/// regression): an EMPTY `<MathFork>` main branch — a leading `&&` empty
/// aligned column in alignat — must be UNWRAPPED to nothing so the empty fork
/// is pruned downstream (Perl / pre-56eg behavior), NOT kept as a spurious
/// `<MathFork><Math text="absent">…<td/><td/>`. `cleanup_math_unwrap_valid_
/// under_mathfork` now treats an empty main branch as safe to unwrap.
#[test]
fn empty_mathfork_main_branch_is_pruned_not_kept() {
  let tex = r"\documentclass{article}
\usepackage{amsmath}
\begin{document}
\begin{alignat}{2}
&& \framebox[1.5cm]{1} &= \framebox[3cm]{2}
\end{alignat}
\end{document}
";
  let (stderr, xml) = convert_with(tex, Some("ar5iv.sty"));
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  // No spurious empty fork: an `absent`-token Math main branch over empty cells.
  assert!(
    !xml.contains("text=\"absent\""),
    "an empty aligned column must not survive as a spurious <Math text=\"absent\"> MathFork\n{xml}"
  );
}

/// algorithm2e `{procedure}`/`{function}`: the raw environment lets `\@caption`
/// be `\algocf@caption@proc#1[#2]#3` (algorithm2e.sty:2402), and our `\caption`
/// passes no `[short]`, so the `[` scan ran to the end of the document
/// (arXiv 2605.00743, 2605.06384: `Fatal:Mouth:EoF`). Bound like `{algorithm}`,
/// the caption is algorithm2e's own: "Procedure Name(args)", unnumbered, the
/// name as the refnum, `\Name` declared as a function keyword.
#[test]
fn algorithm2e_procedure_caption_is_bound() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/captions-floats/algorithm2e_procedure_caption.tex"
  );
  let (stderr, xml) = convert_with(tex, None);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("<caption><text font=\"bold\">Procedure</text>\u{a0}SetMiniDisk(S, R)</caption>"),
    "{xml}"
  );
  assert!(
    xml.contains("<caption><text font=\"bold\">Function</text>\u{a0}Empty</caption>"),
    "{xml}"
  );
  assert!(
    xml.contains(
      "<float class=\"ltx_algorithm\" framed=\"top\" labels=\"LABEL:p:a\" xml:id=\"algorithmx1\">
    <tags>
      <tag role=\"refnum\">SetMiniDisk</tag>
    </tags>
    <toccaption>Procedure\u{a0}SetMiniDisk</toccaption>"
    ),
    "{xml}"
  );
  assert!(
    xml.contains("We call <text font=\"typewriter\">SetMiniDisk(<emph font=\"serif italic\">a</emph>)</text> in Procedure\u{a0}<ref labelref=\"LABEL:p:a\"/>."),
    "{xml}"
  );
  // {algorithm} keeps its numbered caption.
  assert!(xml.contains(r#"<tag role="refnum">1</tag>"#), "{xml}");
}

/// `\subimport` runs the imported file ungrouped, as import.sty:65-92 does: its
/// `\newcommand`s and the packages it loads (whose loaded-flags are global)
/// outlive the import. The grouped binding popped them (arXiv 2605.20598:
/// hyperref → etoolbox loaded in a subimport, biblatex's `\newbool` then
/// undefined, `Fatal:TooManyErrors`).
#[test]
fn subimport_keeps_definitions() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/loader/subimport_keeps_definitions.tex");
  let (stderr, xml) = convert_with(tex, None);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<p>MACRO Y</p>"), "{xml}");
}

/// jmlr's `\addr` scans to the author sentinel only inside the structured
/// author block; elsewhere it is jmlr.cls:344's empty macro. Under an undefined
/// class wrapper (`\coltauthor`, class-body in arXiv 2605.25859) it ran to the
/// end of the document (`Fatal:Mouth:EoF`).
#[test]
fn jmlr_addr_outside_the_author_block_is_empty() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/jmlr_addr_outside_author_block.tex"
  );
  let (stderr, xml) = convert_with(tex, None);
  assert_eq!(stderr.matches("Fatal:").count(), 0, "{stderr}");
  // The undefined `\coltauthor` only (Perl: the same).
  assert_eq!(error_count(&stderr), 1, "{stderr}");
  assert!(stderr.contains("undefined:\\coltauthor"), "{stderr}");
  assert!(xml.contains("<personname>Ido Nachum</personname>"), "{xml}");
  assert!(xml.contains("<p>University of Haifa</p>"), "{xml}");
  assert!(xml.contains("<p>Body.</p>"), "{xml}");
}

/// listings.sty:1742 `\let\lst@ifdisplaystyle\iffalse`: a style that tests it
/// (arXiv 2605.12091's `basicstyle=…\lst@ifdisplaystyle\scriptsize\else\fi`)
/// raised three errors per listing, Fatal on the paper (Perl the same).
#[test]
fn lst_ifdisplaystyle_is_false() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/parameter-conditional/lst_ifdisplaystyle_in_basicstyle.tex"
  );
  let (stderr, xml) = convert_with(tex, None);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains(r#"<text class="ltx_lst_identifier" font="typewriter">hello</text><text class="ltx_lst_space" font="typewriter"> </text><text class="ltx_lst_identifier" font="typewriter">world</text></listingline>"#),
    "{xml}"
  );
}

/// A repeat `\usepackage[dvipsnames]{xcolor}` loads the name set (xcolor.sty:
/// 171-193), so tikz finds `Maroon` (arXiv 2605.28926: `/tikz/Maroon` unknown
/// key per use, Fatal; Perl the same).
#[test]
fn xcolor_reload_loads_dvipsnames() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/graphics-tikz/xcolor_reload_dvipsnames_tikz.tex"
  );
  let (stderr, xml) = convert_with(tex, None);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(xml.contains(r##"fill="#AD1737""##), "{xml}");
  assert!(xml.contains(r##"color="#AD0000">M</text>"##), "{xml}");
}

/// xcolor.sty `\XC@edef`: an active `!` stands for itself in a colour
/// expression (arXiv 2605.30133: `\catcode`!=13\def!{\itshape}` tables turned
/// `red!100.0!black` into font switches, Fatal; Perl the same).
#[test]
fn xcolor_expression_ignores_an_active_bang() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/unicode-catcodes/xcolor_active_bang_in_expression.tex"
  );
  let (stderr, xml) = convert_with(tex, None);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains(
      r##"<td align="left"><text color="#FF0000">HELLO</text> <text font="italic">x</text></td>"##
    ),
    "{xml}"
  );
}

/// A package an autoload trigger loads inside a group outlives the group, as
/// its global loaded-flag and lock do: natbib's `\citep` (and its citation
/// style) survived only until the `}` (arXiv 2605.08349, 2605.10423,
/// 2605.14513, 2605.04028: `undefined:\citep` ×100, Fatal).
#[test]
fn autoloaded_package_outlives_the_group() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/loader/autoload_in_group_survives_the_group.tex"
  );
  let (stderr, xml) = convert_with(tex, None);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  // Only the missing class (OmniBus fallback).
  assert_eq!(warning_count(&stderr), 1, "{stderr}");
  assert_eq!(
    xml.matches(r#"<cite class="ltx_citemacro_citep">"#).count(),
    2,
    "{xml}"
  );
  assert_eq!(
    xml.matches(r#"<cite class="ltx_citemacro_citet">"#).count(),
    1,
    "{xml}"
  );
  // One citation style throughout: natbib's author-year, set by the load.
  assert_eq!(
    xml.matches(r#"show="AuthorsPhrase1Year""#).count(),
    2,
    "{xml}"
  );
  // The group's italic stays in the group.
  assert_eq!(xml.matches(r#"font="italic""#).count(), 1, "{xml}");
}

fn kpsewhich_has(name: &str) -> bool {
  std::process::Command::new("kpsewhich")
    .arg(name)
    .output()
    .map(|o| o.status.success() && !o.stdout.is_empty())
    .unwrap_or(false)
}

/// tex.web §577 `scan_font_ident` expands: `\fontdimen8 \ifx#1\displaystyle
/// \textfont\else…\fi 3` selects `\textfont` (arXiv 2605.21425's `\mathpalette`
/// underline macro: the unexpanded read took `\ifx` as the font, 1000 orphaned
/// `\else`/`\fi`, Fatal; Perl the same).
#[test]
fn fontdimen_font_identifier_is_expanded() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/fontdimen_font_ident_expands.tex"
  );
  let (stderr, xml) = convert_with(tex, None);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains(r#"tex="\underline{\hbox{$\textstyle\underline{\sigma}$}}""#),
    "{xml}"
  );
}

/// `\texttt` & co. close their text branch with `\expandafter\egroup\fi`, as
/// latex.ltx's `\DeclareTextFontCommand` does: seqsplit's `\futurelet` scanner
/// peeked a `}` character and looped (arXiv 2605.04530, `Fatal:Timeout:IfLimit`).
#[test]
fn text_font_command_ends_with_egroup() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/text_font_command_ends_with_egroup.tex"
  );
  let (stderr, xml) = convert_with(tex, Some("ar5iv.sty"));
  assert_eq!(stderr.matches("Fatal:").count(), 0, "{stderr}");
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  // seqsplit splits `\texttt` from its argument, so the text is roman in
  // pdflatex too (CMR10 only).
  assert!(
    xml.contains(r#">infra_sweep.py and <text font="bold""#),
    "{xml}"
  );
}

/// glossaries entry labels are keys, never digested: an underscore label
/// raised "_ can only appear in math mode" per entry (arXiv 2605.01773: 122
/// labels, Fatal; Perl the same).
#[test]
fn glossaries_underscore_label_is_a_key() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/index-bib/glossaries_underscore_label.tex");
  let (stderr, xml) = convert_with(tex, None);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains(r#"<glossarydefinition inlist="main" key="beat_frequency">"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<glossaryref inlist="main" key="beat_frequency">"#),
    "{xml}"
  );
}

/// `\mathcode` reads back the whole code: `\mathcode`\'` is "8000
/// (plain.tex:88). Its low byte, 0, sent babel's `\initiate@active@char{'}`
/// down the branch where the active `'` is itself, and `$x'$` under
/// czech/slovak looped (arXiv 2605.05181, 2605.16660, `Fatal:Timeout:IfLimit`).
#[test]
fn mathcode_reads_the_whole_code() {
  let tex = "\\documentclass{article}\n\\begin{document}\n\\the\\mathcode`\\' \\ifnum\\mathcode`\\'=\"8000 yes\\else no\\fi\n\\mathcode`\\z=32768 \\the\\mathcode`\\z.\n\\end{document}\n";
  let (stderr, xml) = convert_with(tex, None);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<p>32768yes32768.</p>"), "{xml}");
  if kpsewhich_has("slovak.ldf") {
    let tex = include_str!(
      "../../../tools/perfect_kernel/repros/expansion-primitives/mathcode_reads_the_whole_code.tex"
    );
    let (stderr, xml) = convert_with(tex, Some("ar5iv.sty"));
    assert_eq!(stderr.matches("Fatal:").count(), 0, "{stderr}");
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains(r#"tex="x^{\prime}""#), "{xml}");
  }
}

/// ieeetj.cls is IEEEtran V1.7a inline plus a numbered `\affil` store: bound
/// on IEEEtran (with inst_support's `\author`), not OmniBus, so IEEEtran's
/// `\ifCLASSOPTION…`, `{IEEEkeywords}` and `\IEEEPARstart` exist (arXiv
/// 2605.01773; 2405.01673 and 2603.04284 lost their keywords).
#[test]
fn ieeetj_is_ieeetran() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/ieeetj_is_ieeetran.tex"
  );
  let (stderr, xml) = convert_with(tex, None);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("<personname>Alice Smith<sup>1</sup></personname>"),
    "{xml}"
  );
  assert!(
    xml.contains("<personname>Bob Jones<sup>1</sup></personname>"),
    "{xml}"
  );
  assert!(
    xml
      .contains("<contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">University A</contact>"),
    "{xml}"
  );
  assert!(
    xml.contains("<keywords name=\"Index Terms:\u{a0}\">radar, navigation"),
    "{xml}"
  );
  assert!(xml.contains("<p>This is y.</p>"), "{xml}");
}

/// orcidlink's emptiness tests carry no bare `&`: `\ifx&#1&` read a column end
/// at alignment brace level 0 (tex.web §342), so `\textbf{Name~\orcidlink{…}}`
/// in a `p{…}` cell — `\textbf` is `\bgroup…\egroup` since 56hv, which does not
/// raise that level — ended the cell early (arXiv 2605.21922, 7 errors).
#[test]
fn orcidlink_in_a_p_cell_keeps_the_cell() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/alignment/orcidlink_textbf_in_p_cell.tex");
  let (stderr, xml) = convert_with(tex, Some("ar5iv.sty"));
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains(
      "<text font=\"bold\" xml:id=\"p1.1.1.1.1.1.1\">Alice\u{a0}<ref class=\"ltx_orcid\" \
         href=\"https://orcid.org/0000-0002-7342-2090\""
    ),
    "{xml}"
  );
}

/// revtex4-2's `\close@column@grid` (cls:7424, `\onecolumngrid` when balancing)
/// is a layout no-op like `\onecolumngrid`; a paper's `\balancecolsandclearpage`
/// calls it (arXiv 2605.07942).
#[test]
fn revtex_close_column_grid_is_defined() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/revtex_close_column_grid.tex"
  );
  let (stderr, xml) = convert_with(tex, None);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<p>Text.\nMore.</p>"), "{xml}");
}
