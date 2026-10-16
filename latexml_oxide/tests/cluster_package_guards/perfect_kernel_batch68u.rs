//! Red/green guards for batch 68u: the rawclasses lever's remaining regression roots (study
//! agent_reports/2026-10-15_rc_rawcls_regressions.md). R4: the undefined-token error stub installs past a lock, so a
//! locked name a document `\let` to an undefined one errs once instead of looping to TooManyErrors (2006.07968,
//! 1509.06375; OXIDIZED_DESIGN_DIVERGENCES #641). R6: `\|` is LaTeX's protected macro over the math atom, so
//! beilstein.cls:264's `\expandafter\DeclareRobustCommand\expandafter\|\expandafter{\|}` makes a robust `\|`
//! instead of a self-loop (2604.06890; #642). R8: the kernel leaves `secnumdepth` 0, as latex.ltx does, so letter.cls's
//! sections are unnumbered (0907.4214). A class setter replay runs no frontmatter fallback, so paper.cls's replayed
//! `\abstract` no longer emits the title and authors in the preamble (2012.12760, 2207.06852).
//!
//!
//! Round 2 — a raw class's frontmatter setters reach the frontmatter by what their names mean (K11; OXIDIZED_DESIGN
//! #643): rule (a), a table setter storing its argument in a macro only the class's title page reads (apa6
//! `\affiliation` → `\@affil`, `\authornote` → `\@acks`; 1710.08171, 1705.02935, 2003.11668); a redefined kernel
//! `\date` with side definitions (tufte-common.def:608-614; 2603.01440); a `{keywords}` collecting its body for the
//! title page (cclayout.sty:71-73; 1905.07821); and the beilstein and lni author blocks as bindings (2203.09951,
//! 2604.06890, 2608.21389).
//!
//! Round 3 — the review's should-fixes: apa6/apa's numbered author forms (`\twoauthors` … `\sixaffiliations`,
//! apa6.cls:745-777) pair the n-th name with the n-th affiliation as the class's title page does (2102.04789,
//! 1710.04173); a locked name's error stub is local, so a binding cleared in a group is back after it
//! (OXIDIZED_DESIGN_DIVERGENCES #641); a class declaring another format (`\NeedsTeXFormat{pLaTeX2e}`) is not read raw
//! (1203.0762); beilstein's `\author[marks][e-mail]{name}` (beilstein.cls:627).
//!
//! Every repro is checked against pdflatex (its header holds the oracle).
use latexml::util::test::{assert_element, normalize_markup, xml_element};

use super::{
  perfect_kernel_batch46::{convert_files_with, error_count, warning_count},
  perfect_kernel_batch57::{RAW, assert_elements, assert_elements_with},
};

/// The frontmatter block: from the title to the first paragraph, whole.
fn front_block(xml: &str) -> String {
  let start = xml.find("<title").expect("a title");
  let end = start + xml[start..].find("<para").expect("a paragraph");
  normalize_markup(&xml[start..end])
}

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
  assert_element(&xml, "title", &[], "<title>The Title</title>");
  assert_element(
    &xml,
    "creator",
    &[r#"role="author""#],
    r#"<creator role="author"><personname>Ann Author</personname></creator>"#,
  );
}

/// sectioning-frontmatter/raw_class_setter_store_read_only_by_title_page_apa6, rule (a): apa6.cls:760 `\affiliation`
/// stores `\@affil` and :779 `\authornote` `\@acks`, which only its `\maketitle` and `\check@author` read, so the
/// affiliation is the author's and the author note a frontmatter note (pdflatex: "Some University", "Author Note /
/// Correspondence to …"). Before 68u r2 both were lost under the raw class.
#[test]
fn a_raw_class_store_only_its_title_page_reads_is_the_frontmatter() {
  let xml = assert_elements(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/raw_class_setter_store_read_only_by_title_page_apa6.tex"
    ),
    RAW,
    (0, 0),
    &[],
  );
  assert_eq!(
    front_block(&xml),
    normalize_markup(
      r#"<title>The Title</title><toctitle>Short</toctitle><creator role="author"><personname>Ann Author</personname><contact name="Affiliation: " role="affiliation">Some University</contact></creator><abstract inlist="toc" name="Abstract" xml:id="abstract1"><p>Abs.</p></abstract><keywords name="Keywords: ">alpha, beta</keywords><note role="authornote">Correspondence to Ann Author, Some University.</note>"#
    ),
    "{xml}"
  );
}

/// sectioning-frontmatter/maketitle_replay_gate_ifundefined_branch_apa6 (the round-1 RED repro): the jou-mode
/// affiliation reaches the author through rule (a), whatever the replay gate does with `\@journal` (pdflatex: "Some
/// University" under the author).
#[test]
fn an_apa6_journal_mode_affiliation_is_the_authors() {
  let xml = assert_elements(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/maketitle_replay_gate_ifundefined_branch_apa6.tex"
    ),
    RAW,
    (0, 0),
    &[],
  );
  assert_eq!(
    front_block(&xml),
    normalize_markup(
      r#"<title>The Title</title><creator role="author"><personname>Ann Author</personname><contact name="Affiliation: " role="affiliation">Some University</contact></creator><abstract inlist="toc" name="Abstract" xml:id="abstract1"><p>Abs.</p></abstract>"#
    ),
    "{xml}"
  );
}

/// The reader scan's other side (sweep 93's rule): a store another class macro reads stays the class's. `\institute`
/// stores `\@where`, which `\signature` reads as well as the title page, so it is not rerouted and the body's
/// `\signature` prints it; `\affiliation`'s `\@affil`, read only by `\maketitle`, is the author's (pdflatex: "The
/// Title / Ann Author / Some University / Some City", then "Signed at Some City.").
#[test]
fn a_raw_class_store_another_macro_reads_stays_the_classs() {
  let cls = "\\NeedsTeXFormat{LaTeX2e}\n\\ProvidesClass{storeread}\n\\LoadClass{article}\n\
             \\def\\affiliation#1{\\def\\@affil{#1}}\n\\def\\institute#1{\\gdef\\@where{#1}}\n\
             \\def\\signature{Signed at \\@where.}\n\
             \\def\\maketitle{\\begin{center}\\@title\\par\\@author\\par\\@affil\\par\\@where\\end{center}}\n";
  let tex = "\\documentclass{storeread}\n\\title{The Title}\n\\author{Ann Author}\n\\affiliation{Some University}\n\
             \\institute{Some City}\n\\begin{document}\n\\maketitle\n\\signature\n\\end{document}\n";
  let (stderr, xml) = convert_files_with(tex, &[("storeread.cls", cls)], Some(RAW));
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(
    stderr.contains(
      "Rerouted the raw class's title-page stores to the frontmatter API: \\affiliation\n"
    ),
    "{stderr}"
  );
  assert_element(
    &xml,
    "creator",
    &[r#"role="author""#],
    r#"<creator role="author"><personname>Ann Author</personname><contact name="Affiliation: " role="affiliation">Some University</contact></creator>"#,
  );
  assert!(xml.contains("Signed at Some City."), "{xml}");
}

/// sectioning-frontmatter/raw_class_date_with_side_definitions_tufte: tufte-common.def:608-614's `\date` stores
/// `\@date` and also defines `\thedate`; it keeps both and the date is the frontmatter's (pdflatex: "27 February
/// 2026"; its one Warning is bibentry's, as pdflatex's). Before 68u r2 the date was lost.
#[test]
fn a_raw_class_date_with_side_definitions_sets_the_date() {
  let xml = assert_elements_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/raw_class_date_with_side_definitions_tufte.tex"
    ),
    RAW,
    (0, 1),
    &["Package bibentry Warning: You have used \\nobibliography*"],
    &[],
  );
  assert_eq!(
    front_block(&xml),
    normalize_markup(
      r#"<title class="ltx_align_left">The Title</title><creator class="ltx_align_left" role="author"><personname>Ann Author</personname></creator><date class="ltx_align_left" role="creation">27 February 2026</date>"#
    ),
    "{xml}"
  );
}

/// sectioning-frontmatter/raw_class_collected_keywords_env_cc: cclayout.sty:71-73's `{keywords}` collects its body
/// for the title page only, so it is the frontmatter's keywords (pdflatex: "Keywords. alpha, beta"; its three
/// Warnings are the class's two and ccalgo's, as pdflatex's three). Before 68u r2 the keywords were lost.
#[test]
fn a_raw_class_keywords_environment_collecting_its_body_sets_the_keywords() {
  let xml = assert_elements_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/raw_class_collected_keywords_env_cc.tex"
    ),
    RAW,
    (0, 3),
    &[
      "Who is the contact author?",
      "Missing submission/received date.",
    ],
    &[],
  );
  assert_eq!(
    front_block(&xml),
    normalize_markup(
      r#"<title>The Title</title><creator role="author"><personname>Ann Author</personname><contact name="Affiliation: " role="affiliation">Some University</contact></creator><abstract inlist="toc" name="Abstract" xml:id="abstract1"><p>Abs.</p></abstract><keywords name="Keywords: ">alpha, beta</keywords>"#
    ),
    "{xml}"
  );
}

/// class-bindings/beilstein_author_affiliation_numbers: each author carries the affiliations numbered after it and
/// those its marks name, the corresponding author its e-mail (pdflatex: "Ann Able1, 2 , Bob Baker∗1 and Carl Cole3").
/// Before 68u r2: an author named `*`, and no affiliation.
#[test]
fn beilstein_authors_carry_their_numbered_affiliations() {
  let xml = assert_elements(
    include_str!(
      "../../../tools/perfect_kernel/repros/class-bindings/beilstein_author_affiliation_numbers.tex"
    ),
    RAW,
    (0, 0),
    &[],
  );
  assert_eq!(
    front_block(&xml),
    normalize_markup(
      r#"<title>The Title</title><creator role="author"><personname>Ann Able</personname><contact name="Affiliation: " role="affiliation">First University</contact><contact name="Affiliation: " role="affiliation">Second Institute</contact></creator><creator before="  " role="author"><personname>Bob Baker</personname><contact name="Email: " role="email">bob@example.org</contact><contact name="Affiliation: " role="affiliation">First University</contact></creator><creator before="  " role="author"><personname>Carl Cole</personname><contact name="Affiliation: " role="affiliation">Third Lab</contact></creator><abstract inlist="toc" name="Abstract" xml:id="abstract1"><p>Abs.</p></abstract>"#
    ),
    "{xml}"
  );
}

/// class-bindings/lni_author_email_orcid_affil: `\author[mark]{name}{e-mail}{ORCID}` is authblk's author with its
/// e-mail and ORCID, `\affil[mark]` authblk's (pdflatex: the footnotes "1 First University, Street 1, Town,
/// ann@example.org, …" and "2 Second Institute, Town, bob@example.org"). Before 68u r2 no affiliation, e-mail or ORCID.
#[test]
fn lni_authors_carry_email_orcid_and_affiliation() {
  let xml = assert_elements(
    include_str!(
      "../../../tools/perfect_kernel/repros/class-bindings/lni_author_email_orcid_affil.tex"
    ),
    RAW,
    (0, 0),
    &[],
  );
  assert_eq!(xml.matches("<creator").count(), 2, "{xml}");
  assert_element(
    &xml,
    "creator",
    &[r#"before="  ""#],
    r#"<creator before="  " role="author"><personname>Bob Baker</personname><contact name="Email: " role="email">bob@example.org</contact><contact name="Affiliation: " role="affiliation">Second Institute<break/>Town</contact></creator>"#,
  );
  let ann = xml_element(&xml, "creator", &[r#"role="author""#]).expect("Ann Able");
  for part in [
    r#"<personname>Ann Able</personname><contact name="Email: " role="email">ann@example.org</contact><contact role="orcid"><ref class="ltx_orcid" href="https://orcid.org/0000-0002-1825-0097" title="ORCID 0000-0002-1825-0097">"#,
    r#"<contact name="Affiliation: " role="affiliation">First University<break/>Street 1<break/>Town</contact></creator>"#,
  ] {
    assert!(ann.contains(part), "{ann}");
  }
}

/// class-bindings/apa6_numbered_author_affiliation_forms: `\threeauthors` and `\threeaffiliations` are three creators,
/// each with the affiliation of its number (pdflatex: "Ann Author / First University / Bob Builder / Second Institute
/// / Cy Coder / Third College"). Before 68u r3 no creator at all (2102.04789, 1710.04173).
#[test]
fn apa6_numbered_authors_carry_the_affiliation_of_their_number() {
  let xml = assert_elements(
    include_str!(
      "../../../tools/perfect_kernel/repros/class-bindings/apa6_numbered_author_affiliation_forms.tex"
    ),
    RAW,
    (0, 0),
    &[],
  );
  assert_eq!(
    front_block(&xml),
    normalize_markup(
      r#"<title>The Title</title><toctitle>Short</toctitle><creator role="author"><personname>Ann Author</personname><contact name="Affiliation: " role="affiliation">First University</contact></creator><creator before="  " role="author"><personname>Bob Builder</personname><contact name="Affiliation: " role="affiliation">Second Institute</contact></creator><creator before="  " role="author"><personname>Cy Coder</personname><contact name="Affiliation: " role="affiliation">Third College</contact></creator><abstract inlist="toc" name="Abstract" xml:id="abstract1"><p>Abs.</p></abstract>"#
    ),
    "{xml}"
  );
}

/// A name slot holding a group of names (1401.2054 `\twoauthors{Zhiyong Zhang, Kaifeng Jiang, Haiyan Liu}{In-Sue Oh}`):
/// each name is a creator, all carrying the slot's affiliation (pdflatex: "Ann Author, Bob Builder and Cy Coder /
/// First University / Dee Doe / Second Institute").
#[test]
fn apa6_numbered_author_slot_with_several_names_shares_its_affiliation() {
  let xml = assert_elements(
    "\\documentclass[man]{apa6}\n\\title{The Title}\n\\shorttitle{Short}\n\
     \\twoauthors{Ann Author, Bob Builder and Cy Coder}{Dee Doe}\n\\twoaffiliations{First University}{Second Institute}\n\
     \\abstract{Abs.}\n\\begin{document}\n\\maketitle\nText.\n\\end{document}\n",
    RAW,
    (0, 0),
    &[],
  );
  assert_eq!(
    front_block(&xml),
    normalize_markup(
      r#"<title>The Title</title><toctitle>Short</toctitle><creator role="author"><personname>Ann Author</personname><contact name="Affiliation: " role="affiliation">First University</contact></creator><creator before="  " role="author"><personname>Bob Builder</personname><contact name="Affiliation: " role="affiliation">First University</contact></creator><creator before="  " role="author"><personname>Cy Coder</personname><contact name="Affiliation: " role="affiliation">First University</contact></creator><creator before="  " role="author"><personname>Dee Doe</personname><contact name="Affiliation: " role="affiliation">Second Institute</contact></creator><abstract inlist="toc" name="Abstract" xml:id="abstract1"><p>Abs.</p></abstract>"#
    ),
    "{xml}"
  );
}

/// apa.cls (:427-459, the same forms): once `\@authorTwo` is defined the numbered names and affiliations are the
/// title page's, whatever `\author` and `\affiliation` said (2011.06172's placeholder `\author{Author}`
/// `\affiliation{Affiliation}`; pdflatex: "Ann Author / First University / Bob Builder / Second Institute", and the
/// class's "Using BibTeX with apacite" warning).
#[test]
fn apa_numbered_authors_supersede_the_one_author_forms() {
  let xml = assert_elements_with(
    "\\documentclass[man]{apa}\n\\title{The Title}\n\\author{Placeholder}\n\\affiliation{Placeholder Affil}\n\
     \\twoauthors{Ann Author}{Bob Builder}\n\\twoaffiliations{First University}{Second Institute}\n\
     \\abstract{Abs.}\n\\begin{document}\n\\maketitle\nText.\n\\end{document}\n",
    RAW,
    (0, 1),
    &["Class apa Warning: Using BibTeX with apacite for citations and references"],
    &[],
  );
  assert_eq!(
    front_block(&xml),
    normalize_markup(
      r#"<title>The Title</title><creator role="author"><personname>Ann Author</personname><contact name="Affiliation: " role="affiliation">First University</contact></creator><creator before="  " role="author"><personname>Bob Builder</personname><contact name="Affiliation: " role="affiliation">Second Institute</contact></creator><abstract inlist="toc" name="Abstract" xml:id="abstract1"><p>Abs.</p></abstract>"#
    ),
    "{xml}"
  );
}

/// Without numbered names apa6's title page sets `\@author` over `\@affil`, which `\twoaffiliations` set to its
/// first affiliation (apa6.cls:762; 1605.09288, 1609.04156): the names the kernel's `\author` split carry it, the
/// second affiliation is printed nowhere (pdflatex: "Ann Author, Bob Builder / 1. First University", the class's
/// "More affiliations than authors defined").
#[test]
fn apa6_one_author_line_with_numbered_affiliations_carries_the_first() {
  let xml = assert_elements(
    "\\documentclass[jou]{apa6}\n\\title{The Title}\n\\shorttitle{Short}\n\\author{Ann Author, Bob Builder}\n\
     \\twoaffiliations{1. First University}{2. Second Institute}\n\\abstract{Abs.}\n\\begin{document}\n\\maketitle\n\
     Text.\n\\end{document}\n",
    RAW,
    (0, 0),
    &[],
  );
  assert_eq!(
    front_block(&xml),
    normalize_markup(
      r#"<title>The Title</title><toctitle>Short</toctitle><creator role="author"><personname>Ann Author</personname><contact name="Affiliation: " role="affiliation">1. First University</contact></creator><creator before="  " role="author"><personname>Bob Builder</personname><contact name="Affiliation: " role="affiliation">1. First University</contact></creator><abstract inlist="toc" name="Abstract" xml:id="abstract1"><p>Abs.</p></abstract>"#
    ),
    "{xml}"
  );
}

/// macro-state/locked_name_cleared_in_a_group_is_back_after_it: the error stub for a locked `\cite` a group's `\let`
/// cleared ends with the group, so the second `\cite` is the binding's citation (pdflatex: one "Undefined control
/// sequence", then "Xa Y[2]"). Before 68u r3 the stub was global and the second `\cite` an ERROR too.
#[test]
fn a_locked_name_cleared_in_a_group_is_back_after_it() {
  assert_elements_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/macro-state/locked_name_cleared_in_a_group_is_back_after_it.tex"
    ),
    RAW,
    (1, 0),
    &["Error:undefined:\\cite The token T_CS[\\cite] is not defined."],
    &[(
      "para",
      "p1",
      r#"<para xml:id="p1"><p>X<ERROR class="undefined">\cite</ERROR>a Y<cite class="ltx_citemacro_cite">[<bibref bibrefs="b" separator="," yyseparator=","/>]</cite></p></para>"#,
    )],
  );
}

/// The same for a locked `\section` cleared between `\begingroup` and `\endgroup`: the section after the group is
/// the kernel's (pdflatex: "A", one "Undefined control sequence", then the section "1 B").
#[test]
fn a_locked_section_cleared_in_a_group_is_back_after_it() {
  assert_elements_with(
    "\\documentclass{article}\n\\begin{document}\n\\begingroup\\let\\section\\nonexist\\section{A}\\endgroup\n\
     \\section{B}\nText.\n\\end{document}\n",
    RAW,
    (1, 0),
    &["Error:undefined:\\section The token T_CS[\\section] is not defined."],
    &[(
      "section",
      "S1",
      r#"<section inlist="toc" xml:id="S1"><tags><tag>1</tag><tag role="refnum">1</tag><tag role="typerefnum">§1</tag></tags><title><tag close=" ">1</tag>B</title><para xml:id="S1.p1"><p>Text.</p></para></section>"#,
    )],
  );
}

/// loader/class_for_another_format_not_read_raw_jarticle: jarticle.cls declares `\NeedsTeXFormat{pLaTeX2e}`
/// (:16), which LaTeX does not process further (latex.ltx:18663-18674), so the rawclasses lever does not read it raw;
/// it falls to OmniBus with its one missing-binding warning, and the text is kept (pdflatex: the format error, then
/// "Hello world from a pLaTeX class."). Before 68u r3: 5 errors and Fatal:Misdefined, no text (1203.0762).
#[test]
fn a_class_for_another_format_is_not_read_raw() {
  assert_elements_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/loader/class_for_another_format_not_read_raw_jarticle.tex"
    ),
    RAW,
    (0, 1),
    &["Warning:missing_file:jarticle Can't find binding for class jarticle (using OmniBus)"],
    &[(
      "para",
      "p1",
      r#"<para xml:id="p1"><p>Hello world from a pLaTeX class.</p></para>"#,
    )],
  );
}

/// beilstein.cls:627 `\@@@author`: `\author[marks][e-mail]{name}` is the author with its e-mail (the class lists it
/// with the corresponding author's, :649-651; pdflatex: "Ann Able1, 1 and Bob Baker∗2, 2"). Before 68u r3 the name was
/// `[` and the e-mail and name stray text.
#[test]
fn beilstein_author_with_marks_and_email_options() {
  let xml = assert_elements(
    "\\documentclass{beilstein}\n\\begin{document}\n\\title{The Title}\n\\author[1][ann@example.org]{Ann Able}\n\
     \\affiliation{First University}\n\\author*[2]{Bob Baker}{bob@example.org}\n\\affiliation{Second Institute}\n\
     \\maketitle\n\\begin{abstract}\nAbs.\n\\end{abstract}\nText.\n\\end{document}\n",
    RAW,
    (0, 0),
    &[],
  );
  assert_eq!(
    front_block(&xml),
    normalize_markup(
      r#"<title>The Title</title><creator role="author"><personname>Ann Able</personname><contact name="Email: " role="email">ann@example.org</contact><contact name="Affiliation: " role="affiliation">First University</contact></creator><creator before="  " role="author"><personname>Bob Baker</personname><contact name="Email: " role="email">bob@example.org</contact><contact name="Affiliation: " role="affiliation">Second Institute</contact></creator><abstract inlist="toc" name="Abstract" xml:id="abstract1"><p>Abs.</p></abstract>"#
    ),
    "{xml}"
  );
}
