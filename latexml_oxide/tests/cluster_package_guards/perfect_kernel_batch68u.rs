//! Red/green guards for batch 68u: the rawclasses lever's remaining regression roots (study
//! agent_reports/2026-10-15_rc_rawcls_regressions.md). R4: the undefined-token error stub installs past a lock, so a
//! locked name a document `\let` to an undefined one errs once instead of looping to TooManyErrors (2006.07968,
//! 1509.06375; OXIDIZED_DESIGN_DIVERGENCES #641). R6: `\|` is LaTeX's protected macro over the math atom, so
//! beilstein.cls:264's `\expandafter\DeclareRobustCommand\expandafter\|\expandafter{\|}` makes a robust `\|`
//! instead of a self-loop (2604.06890; #642). R8: the kernel leaves `secnumdepth` 0, as latex.ltx does, so letter.cls's
//! sections are unnumbered (0907.4214). A class setter replay runs no frontmatter fallback, so paper.cls's replayed
//! `\abstract` no longer emits the title and authors in the preamble (2012.12760, 2207.06852).
//!
//! Every repro is checked against pdflatex (its header holds the oracle).
use super::perfect_kernel_batch57::{RAW, assert_elements, assert_elements_with};

/// macro-state/locked_name_let_to_undefined_stub_loop: `\let\cite\parencite` with `\parencite` undefined clears the
/// locked `\cite`; its use is one "not defined" Error and the text after it is typeset, as pdflatex's one "Undefined
/// control sequence" ("Text x more text."). Before 68u the lock refused the stub: 101 Errors and Fatal.
#[test]
fn a_locked_name_let_to_an_undefined_one_errs_once() {
  assert_elements_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/macro-state/locked_name_let_to_undefined_stub_loop.tex"
    ),
    RAW,
    (1, 0),
    &["Error:undefined:\\cite The token T_CS[\\cite] is not defined."],
    &[(
      "para",
      "p1",
      r#"<para xml:id="p1"><p>Text <ERROR class="undefined">\cite</ERROR>x more text.</p></para>"#,
    )],
  );
}

/// The same recovery for a locked binding without a braced interface, as pittetd.cls:313's
/// `\AtBeginDocument{\let\maketitle\HyOrg@maketitle}` cleared it before 67q gave hyperref `\HyOrg@maketitle`
/// (1509.06375): one Error, and the paragraph after `\maketitle` is typeset (pdflatex: one "Undefined control
/// sequence", then "After.").
#[test]
fn a_locked_maketitle_let_to_an_undefined_one_errs_once() {
  assert_elements_with(
    "\\documentclass{article}\n\\title{T}\n\\AtBeginDocument{\\let\\maketitle\\nosuchmaketitle}\n\
     \\begin{document}\n\\maketitle\nAfter.\n\\end{document}\n",
    RAW,
    (1, 0),
    &["Error:undefined:\\maketitle The token T_CS[\\maketitle] is not defined."],
    &[(
      "para",
      "p1",
      r#"<para xml:id="p1"><ERROR class="undefined">\maketitle</ERROR><p>After.</p></para>"#,
    )],
  );
}

/// expansion-primitives/robust_double_bar_over_its_expansion_beilstein: the robust `\|` over `\|`'s one-step
/// expansion typesets ‖ as before, `\left\|…\right\|` and `\|…\|` both a norm with OPEN/CLOSE ‖ (pdflatex: "‖x‖ and
/// ‖y‖", 0 errors). Before 68u: `Fatal:Timeout:Recursion`.
#[test]
fn a_robust_double_bar_over_its_expansion_typesets_it() {
  assert_elements(
    include_str!(
      "../../../tools/perfect_kernel/repros/expansion-primitives/robust_double_bar_over_its_expansion_beilstein.tex"
    ),
    RAW,
    (0, 0),
    &[
      (
        "Math",
        "p1.m1",
        r#"<Math mode="inline" tex="\left\|x\right\|" text="norm@(x)" xml:id="p1.m1"><XMath><XMDual><XMApp><XMTok meaning="norm"/><XMRef idref="p1.m1.1"/></XMApp><XMWrap><XMTok name="||" role="OPEN" stretchy="true">‖</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="p1.m1.1">x</XMTok><XMTok name="||" role="CLOSE" stretchy="true">‖</XMTok></XMWrap></XMDual></XMath></Math>"#,
      ),
      (
        "Math",
        "p1.m2",
        r#"<Math mode="inline" tex="\|y\|" text="norm@(y)" xml:id="p1.m2"><XMath><XMDual><XMApp><XMTok meaning="norm"/><XMRef idref="p1.m2.1"/></XMApp><XMWrap><XMTok name="||" role="OPEN">‖</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="p1.m2.1">y</XMTok><XMTok name="||" role="CLOSE">‖</XMTok></XMWrap></XMDual></XMath></Math>"#,
      ),
    ],
  );
}

/// `\|` expands one step as in LaTeX (fontmath.ltx:467: a protected macro), and the atom it reaches is the one
/// `\Vert`, `\bigl\|` and an infix `\|` always gave: VERTBAR `∥` named `||`, reverting as `\|` (the XML is 67int15's
/// byte for byte; only `\meaning` changed, as in pdflatex: `\protected macro:->\delimiter "026B30D `).
#[test]
fn double_bar_is_a_protected_macro_over_the_atom() {
  let xml = assert_elements(
    "\\documentclass{article}\n\\begin{document}\n\\texttt{\\meaning\\|}\n$a \\| b$ $\\bigl\\| c \\bigr\\|$\n\\end{document}\n",
    RAW,
    (0, 0),
    &[
      (
        "Math",
        "p1.m1",
        r#"<Math mode="inline" tex="a\|b" text="conditional@(a, b)" xml:id="p1.m1"><XMath><XMApp><XMTok meaning="conditional" name="||" role="MODIFIEROP">∥</XMTok><XMTok font="italic" role="UNKNOWN">a</XMTok><XMTok font="italic" role="UNKNOWN">b</XMTok></XMApp></XMath></Math>"#,
      ),
      (
        "Math",
        "p1.m2",
        r#"<Math mode="inline" tex="\bigl\|c\bigr\|" text="norm@(c)" xml:id="p1.m2"><XMath><XMDual><XMApp><XMTok meaning="norm"/><XMRef idref="p1.m2.1"/></XMApp><XMWrap><XMTok fontsize="120%" name="||" role="OPEN">‖</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="p1.m2.1">c</XMTok><XMTok fontsize="120%" name="||" role="CLOSE">‖</XMTok></XMWrap></XMDual></XMath></Math>"#,
      ),
    ],
  );
  assert!(
    xml.contains(r#"<text font="typewriter">\protected macro:-&gt;\lx@math@Vert </text>"#),
    "{xml}"
  );
}

/// sectioning-frontmatter/letter_section_is_unnumbered: letter.cls sets no `secnumdepth`, so the kernel's `\section`
/// and `\subsection` are unnumbered and step no counter letter.cls lacks (Perl `--includestyles`: the same, 0 errors;
/// pdflatex: both undefined in letter.cls). Before 68u: `Error:undefined:\thesubsection` in the numbered subsection.
#[test]
fn a_letter_section_is_unnumbered() {
  assert_elements(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/letter_section_is_unnumbered.tex"
    ),
    RAW,
    (0, 0),
    &[(
      "section",
      "sectionx1",
      r#"<section inlist="toc" xml:id="sectionx1"><title>Intro</title><para class="ltx_noindent" xml:id="sectionx1.p1"><p>Text.</p></para><subsection inlist="toc" xml:id="subsectionx1"><title>Sub</title><para class="ltx_noindent" xml:id="subsectionx1.p1"><p>More.</p></para></subsection></section>"#,
    )],
  );
}

/// sectioning-frontmatter/class_setter_replay_runs_no_frontmatter_fallback_jheppub: the replay of paper.cls's refused
/// `\abstract` for jheppub's `\abstract{…}` runs a `\subsubsection*`, whose frontmatter fallback is inert in the replay,
/// so `\maketitle` makes the title and the author (pdflatex prints both). Before 68u both were emitted in the preamble
/// and lost (2012.12760, 2207.06852).
#[test]
fn a_class_setter_replay_runs_no_frontmatter_fallback() {
  let xml = assert_elements(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/class_setter_replay_runs_no_frontmatter_fallback_jheppub.tex"
    ),
    RAW,
    (0, 0),
    &[],
  );
  latexml::util::test::assert_element(&xml, "title", &[], "<title>The Title</title>");
  latexml::util::test::assert_element(
    &xml,
    "creator",
    &[r#"role="author""#],
    r#"<creator role="author"><personname>Ann Author</personname></creator>"#,
  );
}
