//! Red/green guards for batch 78o (quality batch 12): a footnote mark and its text pair by the number they stand for
//! (`[<n>]` is `\thefootnote` at `<n>`, article's title block marks in `\@fnsymbol`); algorithm2e's `noline` blocks
//! give back their indentation; a `\subcaption` in a minipage makes the minipage its sub-figure; acmart's theorems
//! yield to an author's own; amsmath's alignment dimens are defined. Each guard converts its repro under
//! `tools/perfect_kernel/repros/` (or a pdflatex-checked probe) and asserts no Error or Warning and the authored
//! content in whole elements, against pdflatex.
use latexml::util::test::{convert_with, normalize_markup, xml_element};

use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch57::{assert_elements, assert_elements_with},
};

const PRELOAD: &str = "ar5iv.sty";

/// Every `<tag …>` element of `xml` in document order, whole (`xml_element`), against `expected`, after 0 errors and
/// 0 warnings: the creators of a title block have no id to pick one by.
#[track_caller]
fn assert_each(tex: &str, tag: &str, expected: &[&str]) -> String {
  let (log, xml) = convert_with(tex, Some(PRELOAD));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let mut found = Vec::new();
  let mut rest = xml.as_str();
  while let Some(element) = xml_element(rest, tag, &[]) {
    found.push(element);
    let open = rest.find(&format!("<{tag}")).unwrap();
    rest = &rest[open + 1..];
  }
  let expected: Vec<String> = expected.iter().map(|e| normalize_markup(e)).collect();
  assert_eq!(found, expected, "{xml}");
  xml
}

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!("../../../tools/perfect_kernel/repros/", $path))
  };
}

/// latex.ltx:17678-17699: `\footnotetext[1]` marks its note `\thefootnote` at counter 1 — "∗" under footmisc's
/// `symbol` — so it pairs with the `\footnotemark` ("∗") it answers, and `\footnote[2]` is "†". Red (int51a): the text
/// marked "1" stood apart from its mark, the `[2]` note "2". KNOWN_PERL_ERRORS #1060; witness 2507.05750. Repro
/// package-bindings/footmisc_symbol_numbered_footnotetext_pairs_with_its_mark.
#[test]
fn footmisc_symbol_numbered_footnotetext_pairs_with_its_mark() {
  assert_elements(
    repro!("package-bindings/footmisc_symbol_numbered_footnotetext_pairs_with_its_mark.tex"),
    PRELOAD,
    (0, 0),
    &[(
      "p",
      "p1.1",
      r#"<p xml:id="p1.1">Ann Able<note mark="*" role="footnote" xml:id="footnote1"><tags><tag>*</tag><tag role="refnum">*</tag><tag role="typerefnum">footnote *</tag></tags>Work done at home.</note> and Bob Baker<note mark="†" role="footnote" xml:id="footnotex1"><tags><tag>†</tag><tag role="refnum">†</tag><tag role="typerefnum">footnote †</tag></tags>Second.</note>.</p>"#,
    )],
  );
}

/// article.cls:203-223: the title block marks its notes in `\@fnsymbol` and the counter is reset after it, so
/// `\author{Ann\footnotemark[1]}` is "∗" and pairs with `\footnotetext[1]` given after `\renewcommand\thefootnote
/// {\fnsymbol{footnote}}` (the idiom of 2412.20504, 0903.0014, 2006.14582), and the body's first note is "1". Red
/// (int51a): "1"/"1" and the body note "1" only because nothing stepped; the 78k draft unpaired them. Repro
/// sectioning-frontmatter/article_title_block_marks_in_fnsymbol.
#[test]
fn article_title_block_marks_in_fnsymbol() {
  let xml = assert_each(
    repro!("sectioning-frontmatter/article_title_block_marks_in_fnsymbol.tex"),
    "creator",
    &[
      r#"<creator role="author"><personname>Ann<note mark="*" role="footnote" xml:id="footnotex1"><tags><tag>*</tag><tag role="refnum">*</tag><tag role="typerefnum">footnote *</tag></tags>Equal.</note></personname></creator>"#,
      r#"<creator before="  " role="author"><personname>Bob<note mark="†" role="footnote" xml:id="footnotex2"><tags><tag>†</tag><tag role="refnum">†</tag><tag role="typerefnum">footnote †</tag></tags>Intern.</note></personname></creator>"#,
    ],
  );
  latexml::util::test::assert_element(
    &xml,
    "note",
    &["xml:id=\"footnote1\""],
    r#"<note mark="1" role="footnote" xml:id="footnote1"><tags><tag>1</tag><tag role="refnum">1</tag><tag role="typerefnum">footnote 1</tag></tags>first</note>"#,
  );
}

/// report.cls sets a title page by default, whose `\maketitle` (:175-200) keeps `\thefootnote` arabic: the author's
/// `\footnotemark[1]` is "1" and pairs with `\footnotetext[1]`; under `notitlepage` (:204-213) the mark is "∗" and the
/// text "1", which pdflatex prints apart; they pair by the number they stand for (`relocate_footnote`'s `_fnvalue`).
/// pdflatex 0 errors: "Ann1" / "1 Equal."; "Ann∗" / "1 Equal.".
#[test]
fn report_title_page_marks_stay_arabic() {
  let tex = r"\documentclass{report}
\title{T}
\author{Ann\footnotemark[1]}
\begin{document}
\maketitle
\footnotetext[1]{Equal.}
Body\footnote{first}.
\end{document}";
  assert_each(tex, "creator", &[
    r#"<creator role="author"><personname>Ann<note mark="1" role="footnote" xml:id="Ch0.footnotex1"><tags><tag>1</tag><tag role="refnum">1</tag><tag role="typerefnum">footnote 1</tag></tags>Equal.</note></personname></creator>"#,
  ]);
  let tex = tex.replace("{report}", "[notitlepage]{report}");
  assert_each(&tex, "creator", &[
    r#"<creator role="author"><personname>Ann<note mark="*" role="footnote" xml:id="Ch0.footnotex1"><tags><tag>*</tag><tag role="refnum">*</tag><tag role="typerefnum">footnote *</tag></tags>Equal.</note></personname></creator>"#,
  ]);
}

/// A mark and a text printing no common mark pair by the counter value they stand for: article's title marks
/// `\footnotemark[2]` "†", the `\footnotetext[2]` after `\maketitle` prints "2" (2505.14146); pdflatex prints them
/// apart ("A†", notes "1 First." and "2 Second."), the author pairs them. `[1]` answers no mark and stays a text.
#[test]
fn title_mark_pairs_with_its_text_by_number() {
  let xml = assert_each(
    r"\documentclass{article}
\title{T}
\author{A\footnotemark[2]}
\begin{document}
\maketitle
\footnotetext[1]{First.}
\footnotetext[2]{Second.}
Body.
\end{document}",
    "creator",
    &[
      r#"<creator role="author"><personname>A<note mark="†" role="footnote" xml:id="footnotex1"><tags><tag>†</tag><tag role="refnum">†</tag><tag role="typerefnum">footnote †</tag></tags>Second.</note></personname></creator>"#,
    ],
  );
  latexml::util::test::assert_element(
    &xml,
    "note",
    &["role=\"footnotetext\""],
    r#"<note mark="1" role="footnotetext" xml:id="footnotex2">First.</note>"#,
  );
}

/// latex.ltx:17678-17682 reads `[<n>]` as `\c@footnote #1\relax`: `\footnotemark[\value{footnote}]` is the counter's
/// value, as `[\arabic{footnote}]` (pdflatex 0 errors: "Body11 and 1 1"). The notes read `[<n>]` undigested
/// (`OptionalUndigested`). Red (78o r1, int52a, Perl): 2 warnings "Missing number, treated as zero", the
/// `[\value{footnote}]` marks "". Review probe rev78o f8. Repro
/// package-bindings/footnotemark_value_footnote_is_the_counter.
#[test]
fn footnotemark_value_footnote_is_the_counter() {
  let mark = |id: &str| {
    format!(
      r#"<note mark="1" role="footnotemark" xml:id="{id}"><tags><tag>1</tag><tag role="refnum">1</tag><tag role="typerefnum">footnote 1</tag></tags></note>"#
    )
  };
  let expected = format!(
    r#"<p xml:id="p1.1">Body<note mark="1" role="footnote" xml:id="footnote1"><tags><tag>1</tag><tag role="refnum">1</tag><tag role="typerefnum">footnote 1</tag></tags>bodyone</note>{} and {}
{}.</p>"#,
    mark("footnotex1"),
    mark("footnotex2"),
    mark("footnotex3")
  );
  assert_elements(
    repro!("package-bindings/footnotemark_value_footnote_is_the_counter.tex"),
    PRELOAD,
    (0, 0),
    &[("p", "p1.1", expected.as_str())],
  );
}

/// The title block's `\thanks` and `\footnote` step the footnote counter, as article's `\thanks` is `\footnotemark` and
/// its text (article.cls:241 `\let\footnote\thanks`, latex.ltx:17215), so `\footnotemark[\arabic{footnote}]` after
/// one is its "∗" and after the second "†" (pdflatex 0 errors: "A∗ B∗ C† D†", the body's "1"). Red (78o r1): B's and D's
/// marks "" — nothing stepped, `\@fnsymbol` of 0. Witnesses 2201.11183 ("Zhenan Fan∗1, Huang Fang∗2"), 2111.13260
/// ("Suhani Vora1 *, Noha Radwan1 *"). KNOWN_PERL_ERRORS #1060. Repro
/// sectioning-frontmatter/article_title_notes_step_the_footnote_counter.
#[test]
fn article_title_notes_step_the_footnote_counter() {
  let xml = assert_each(
    repro!("sectioning-frontmatter/article_title_notes_step_the_footnote_counter.tex"),
    "creator",
    &[
      r#"<creator role="author"><personname>A</personname><contact name="Note: " role="note">fone</contact></creator>"#,
      r#"<creator before="  " role="author"><personname>B<note mark="*" role="footnotemark" xml:id="footnotex1"><tags><tag>*</tag><tag role="refnum">*</tag><tag role="typerefnum">footnote *</tag></tags></note></personname></creator>"#,
      r#"<creator before="  " role="author"><personname>C</personname><note class="ltx_note_frontmatter ltx_thanks_note" role="thanks" xml:id="id1">tthree</note></creator>"#,
      r#"<creator before="  " role="author"><personname>D<note mark="†" role="footnotemark" xml:id="footnotex2"><tags><tag>†</tag><tag role="refnum">†</tag><tag role="typerefnum">footnote †</tag></tags></note></personname></creator>"#,
    ],
  );
  latexml::util::test::assert_element(
    &xml,
    "note",
    &["xml:id=\"footnote1\""],
    r#"<note mark="1" role="footnote" xml:id="footnote1"><tags><tag>1</tag><tag role="refnum">1</tag><tag role="typerefnum">footnote 1</tag></tags>bodyone</note>"#,
  );
}

/// The title's, the authors' and the date's `\thanks` step the counter in turn, so the bare `\footnotemark` after B's
/// is the fourth note, "§" (pdflatex 0 errors: "T∗", "A† B‡§", "D¶", the body's "1"). Red (78o r1): "∗"; int52a and
/// Perl "1" and the body's note "2". Repro sectioning-frontmatter/article_title_thanks_in_every_field_step_in_turn.
#[test]
fn article_title_thanks_in_every_field_step_in_turn() {
  let xml = assert_each(
    repro!("sectioning-frontmatter/article_title_thanks_in_every_field_step_in_turn.tex"),
    "creator",
    &[
      r#"<creator role="author"><personname>A</personname><note class="ltx_note_frontmatter ltx_thanks_note" role="thanks" xml:id="id1">ta</note></creator>"#,
      r#"<creator before="  " role="author"><personname>B<note mark="§" role="footnotemark" xml:id="footnote4"><tags><tag>§</tag><tag role="refnum">§</tag><tag role="typerefnum">footnote §</tag></tags></note></personname><note class="ltx_note_frontmatter ltx_thanks_note" role="thanks" xml:id="id2">tb</note></creator>"#,
    ],
  );
  latexml::util::test::assert_element(
    &xml,
    "note",
    &["xml:id=\"footnote1\""],
    r#"<note mark="1" role="footnote" xml:id="footnote1"><tags><tag>1</tag><tag role="refnum">1</tag><tag role="typerefnum">footnote 1</tag></tags>bodyone</note>"#,
  );
}

/// A mark and a text pair by the counter value they stand for only across the title block (`title_block_partner`):
/// a `\footnotetext[1]` under `\fnsymbol` ("∗ Equal contribution") is no answer to a table's `\footnotemark` "1",
/// which the `\footnotetext` after the tabular answers (pdflatex 0 errors: "x1", notes "∗ Equal contribution", "1 table
/// note", "2 bodytwo"). Red (78o r1, int52a, Perl): "Equal contribution" in the table's mark, "table note" a text
/// apart. Review probe rev78o f2. Repro package-bindings/footnotetext_by_number_pairs_only_across_the_title_block.
#[test]
fn footnotetext_by_number_pairs_only_across_the_title_block() {
  assert_elements(
    repro!("package-bindings/footnotetext_by_number_pairs_only_across_the_title_block.tex"),
    PRELOAD,
    (0, 0),
    &[
      (
        "note",
        "footnotex1",
        r#"<note mark="*" role="footnotetext" xml:id="footnotex1">Equal contribution</note>"#,
      ),
      (
        "td",
        "p1.1.1.1",
        r#"<td align="center" xml:id="p1.1.1.1">x<note mark="1" role="footnote" xml:id="footnote1"><tags><tag>1</tag><tag role="refnum">1</tag><tag role="typerefnum">footnote 1</tag></tags>table note</note></td>"#,
      ),
    ],
  );
}

/// Two notes that print no mark — `{\renewcommand{\thefootnote}{}\footnotemark\footnotetext{…}}`, the unmarked
/// "equal contribution" note — and stand for the same counter value are one note, as a mark and a text printing the
/// same mark are (`title_block_partner`): pdflatex 0 errors, "Body2", notes " Equal contribution." (unmarked) and
/// "2 bodyone". Witness 2606.03847 (paired by 78o r1, apart in int52a and Perl: an empty mark matches none).
#[test]
fn unmarked_footnotemark_pairs_with_its_unmarked_text() {
  assert_elements(
    r"\documentclass{article}
\title{T}
\author{A \and B}
\begin{document}
\maketitle
{\renewcommand{\thefootnote}{}\footnotemark\footnotetext{Equal contribution.}}
Body\footnote{bodyone}.
\end{document}",
    PRELOAD,
    (0, 0),
    &[(
      "note",
      "footnote1",
      r#"<note role="footnote" xml:id="footnote1"><tags><tag role="typerefnum">footnote</tag></tags>Equal contribution.</note>"#,
    )],
  );
}

/// The title's own `\thefootnote` is one hook (`\lx@title@thefootnote`, `latex_constructs::TITLE_THEFOOTNOTE`): a
/// binding declaring it `\@alph\c@footnote` changes the marks the title block prints and the frontmatter observer's Mark
/// labels alike — Bob's bare `\footnotemark` after Ann's `\thanks` is "b" in both, Cat's `\footnotemark[5]` "e"
/// (OXIDIZED_DESIGN_DIVERGENCES #837, #851; the observer's `TITLE_FOOTNOTE_HOOK`). The hook is a binding's, so
/// pdflatex has no say on the letters; under article's own it prints "Ann∗ Bob† Cat¶" (0 errors).
#[test]
fn declared_title_thefootnote_prints_the_marks_the_observer_labels() {
  let tex = "\\documentclass{article}\n\\makeatletter\\def\\lx@title@thefootnote{\\@alph\\c@footnote}\\makeatother\n\\begin{document}\n\\title{T}\n\\author{Ann Able\\thanks{A.} \\and Bob Baker\\footnotemark \\and Cat Cole\\footnotemark[5]}\n\\maketitle\nBody.\n\\end{document}\n";
  let (log, xml, json) = super::perfect_kernel_batchfms2a1::shadow(tex, Some(PRELOAD));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(super::perfect_kernel_batchfms2a1::blocks(&json), [[
    "N:Ann Able",
    "T:A.",
    "S:and",
    "N:Bob Baker",
    "M:b",
    "S:and",
    "N:Cat Cole",
    "M:e"
  ]]);
  latexml::util::test::assert_element(
    &xml,
    "note",
    &["xml:id=\"footnote2\""],
    r#"<note mark="b" role="footnotemark" xml:id="footnote2"><tags><tag>b</tag><tag role="refnum">b</tag><tag role="typerefnum">footnote b</tag></tags></note>"#,
  );
  latexml::util::test::assert_element(
    &xml,
    "note",
    &["xml:id=\"footnotex1\""],
    r#"<note mark="e" role="footnotemark" xml:id="footnotex1"><tags><tag>e</tag><tag role="refnum">e</tag><tag role="typerefnum">footnote e</tag></tags></note>"#,
  );
}

/// A title block's mark numbered from the counter pairs by value only in the title block (`same_count`, `_fncounted`):
/// article's `\maketitle` sets the counter back to 0 (article.cls:190, :223), so C's
/// `\footnotemark[\value{footnote}]` after two `\thanks` is the title's 2, "†", and the body's `\footnotetext[2]` is
/// another note (pdflatex 0 errors: "A∗ B† C†", notes "∗ aa", "† bb", "2 Body text two"). Red (78o r2): "Body text
/// two" moved into C's mark. A literal `[2]` still pairs (`title_mark_pairs_with_its_text_by_number`, 2505.14146).
/// Review probe rev78o2 art2. Repro package-bindings/title_counter_mark_keeps_to_the_title_block.
#[test]
fn title_counter_mark_keeps_to_the_title_block() {
  let xml = assert_each(
    repro!("package-bindings/title_counter_mark_keeps_to_the_title_block.tex"),
    "creator",
    &[
      r#"<creator role="author"><personname>A</personname><note class="ltx_note_frontmatter ltx_thanks_note" role="thanks" xml:id="id1">aa</note></creator>"#,
      r#"<creator before="  " role="author"><personname>B</personname><note class="ltx_note_frontmatter ltx_thanks_note" role="thanks" xml:id="id2">bb</note></creator>"#,
      r#"<creator before="  " role="author"><personname>C<note mark="†" role="footnotemark" xml:id="footnotex1"><tags><tag>†</tag><tag role="refnum">†</tag><tag role="typerefnum">footnote †</tag></tags></note></personname></creator>"#,
    ],
  );
  latexml::util::test::assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    r#"<p xml:id="p1.1">Body text.
<note mark="2" role="footnotetext" xml:id="footnotex2">Body text two</note></p>"#,
  );
}

/// And by the printed mark (Perl's pairing) not between two notes numbered from the counter on the two sides of the
/// reset: report's title page counts its notes from the counter and sets it back after `\maketitle` (report.cls:196,
/// :229), so B's bare `\footnotemark` "2" after A's `\thanks` has no text, and the body's bare `\footnotetext` "2"
/// answers the body's own `\footnotemark` (pdflatex 0 errors: "A1 B2",
/// "Body1 x2 y.", notes "1 one", "2 two"). Red (78o r2): "two" copied into B's mark too. Review probe rev78o2 ttp3.
/// Repro package-bindings/report_title_bare_footnotemark_keeps_to_the_title_block.
#[test]
fn report_title_bare_footnotemark_keeps_to_the_title_block() {
  let xml = assert_each(
    repro!("package-bindings/report_title_bare_footnotemark_keeps_to_the_title_block.tex"),
    "creator",
    &[
      r#"<creator role="author"><personname>A</personname><note class="ltx_note_frontmatter ltx_thanks_note" role="thanks" xml:id="id1">aa</note></creator>"#,
      r#"<creator before="  " role="author"><personname>B<note mark="2" role="footnotemark" xml:id="Ch0.footnote2"><tags><tag>2</tag><tag role="refnum">2</tag><tag role="typerefnum">footnote 2</tag></tags></note></personname></creator>"#,
    ],
  );
  latexml::util::test::assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    r#"<p xml:id="p1.1">Body<note mark="1" role="footnote" xml:id="Ch0.footnote1"><tags><tag>1</tag><tag role="refnum">1</tag><tag role="typerefnum">footnote 1</tag></tags>one</note> x<note mark="2" role="footnote" xml:id="Ch0.footnote2a"><tags><tag>2</tag><tag role="refnum">2</tag><tag role="typerefnum">footnote 2</tag></tags>two</note> y.</p>"#,
  );
}

/// Across the reset, a title's mark numbered from the counter and a body text that names its number print one note
/// when they print one mark (`same_count`): article's bare `\footnotemark` "∗" and the body's `\footnotetext[1]` under
/// `\fnsymbol` "∗ Equal contribution." (pdflatex 0 errors: "A∗ B", notes "∗ Equal contribution.", "1 one"). Witnesses
/// 2507.05750 (footmisc `symbol`), 1405.7962 (a title's `\footnotemark`); red in the first cut of 78o r3, which kept
/// every counted title mark to the title block.
#[test]
fn title_bare_footnotemark_pairs_with_a_numbered_body_text() {
  assert_each(
    r"\documentclass{article}
\title{T}
\author{A\footnotemark \and B}
\begin{document}
\maketitle
{\renewcommand{\thefootnote}{\fnsymbol{footnote}}\footnotetext[1]{Equal contribution.}}
Body\footnote{one}.
\end{document}",
    "creator",
    &[
      r#"<creator role="author"><personname>A<note mark="*" role="footnote" xml:id="footnote1"><tags><tag>*</tag><tag role="refnum">*</tag><tag role="typerefnum">footnote *</tag></tags>Equal contribution.</note></personname></creator>"#,
      r#"<creator before="  " role="author"><personname>B</personname></creator>"#,
    ],
  );
}

/// A `[<n>]` of a counter of its own (`\thealtnum`) is not the footnote counter's, so the title's mark and the body's
/// text naming it pair across the reset (pdflatex 0 errors: "T∗", notes "∗ Based on observations.", "1 one"). Witness
/// astro-ph9712060 (aaspp4, `\footnotemark[\thealtnum]` and `\footnotetext[\thealtnum]`).
#[test]
fn title_mark_of_another_counter_pairs_with_its_body_text() {
  assert_elements(
    r"\documentclass{article}
\newcounter{altnum}\setcounter{altnum}{1}
\title{T\footnotemark[\thealtnum]}
\author{A}
\begin{document}
\maketitle
\renewcommand{\thefootnote}{\fnsymbol{footnote}}
\footnotetext[\thealtnum]{Based on observations.}
\renewcommand{\thefootnote}{\arabic{footnote}}
Body\footnote{one}.
\end{document}",
    PRELOAD,
    (0, 0),
    &[(
      "note",
      "footnotex1",
      r#"<note mark="*" role="footnote" xml:id="footnotex1"><tags><tag>*</tag><tag role="refnum">*</tag><tag role="typerefnum">footnote *</tag></tags>Based on observations.</note>"#,
    )],
  );
}

/// A section's title is not the title block (`in_title_block`: the document's title, a creator, the date): a bare
/// `\footnotemark` in `\section{Results…}` and the bare `\footnotetext` after it are one body note (pdflatex 0
/// errors: "Results2", note "2 In a preprint."). Witness 1907.00457 (left apart in the first cut of 78o r3).
#[test]
fn section_title_footnotemark_pairs_with_its_text() {
  assert_elements(
    r"\documentclass{article}
\title{T}
\author{A\thanks{aa}}
\begin{document}
\maketitle
Body\footnote{one}.
\section{Results\protect\footnotemark}
\footnotetext{In a preprint.}
Text.
\end{document}",
    PRELOAD,
    (0, 0),
    &[(
      "note",
      "footnote2",
      r#"<note mark="2" role="footnote" xml:id="footnote2"><tags><tag>2</tag><tag role="refnum">2</tag><tag role="typerefnum">footnote 2</tag></tags>In a preprint.</note>"#,
    )],
  );
}

/// A `[<n>]` is read for a number once per note, its diagnostics held (`given_footnote_number`): the mark's digestion
/// reports an error of its expansion, once, as pdflatex does (1 error, "You can't use `\relax' after \the"; "H0", the
/// counter as TeX recovers). Red (78o r2): the same error twice (three times with `\the\c@footnote` under `@` other,
/// review probe rev78o2 val). Review probe rev78o2 valerr. Repro package-bindings/footnotemark_number_error_reported_once.
#[test]
fn footnotemark_number_error_reported_once() {
  assert_elements_with(
    repro!("package-bindings/footnotemark_number_error_reported_once.tex"),
    PRELOAD,
    (1, 0),
    &["Error:unexpected:\\the\\relax You can't use \\relax after \\the"],
    &[(
      "p",
      "p1.1",
      r#"<p xml:id="p1.1">A<note mark="1" role="footnote" xml:id="footnote1"><tags><tag>1</tag><tag role="refnum">1</tag><tag role="typerefnum">footnote 1</tag></tags>one</note>.
H<note mark="0" role="footnotemark" xml:id="footnotex1"><tags><tag>0</tag><tag role="refnum">0</tag><tag role="typerefnum">footnote 0</tag></tags></note> X.</p>"#,
    )],
  );
}

/// The same for a `\footnotetext[<n>]` (pdflatex 1 error, the text "0 tt"). Red (78o r2): 2 errors. Review probe
/// rev78o2 valerr2.
#[test]
fn footnotetext_number_error_reported_once() {
  assert_elements_with(
    r"\documentclass{article}
\begin{document}
A\footnote{one}.
H\footnotetext[\the\relax]{tt} X.
\end{document}",
    PRELOAD,
    (1, 0),
    &["Error:unexpected:\\the\\relax You can't use \\relax after \\the"],
    &[(
      "note",
      "footnotex1",
      r#"<note mark="0" role="footnotetext" xml:id="footnotex1">tt</note>"#,
    )],
  );
}

/// algorithm2e.sty:1851-1860: a `noline` block's body is a box of its own, so the line after the block is at the
/// block's level: only "a;" and "c;" are indented. Red (int51a, Perl): "end", "b;", "body" one level deep, "c;", "d;"
/// two. KNOWN_PERL_ERRORS #1062. Repro listings-algorithms/algorithm2e_noline_block_pops_its_indentation.
#[test]
fn algorithm2e_noline_block_pops_its_indentation() {
  assert_elements(
    repro!("listings-algorithms/algorithm2e_noline_block_pops_its_indentation.tex"),
    PRELOAD,
    (0, 0),
    &[(
      "listing",
      "tab1.1",
      r#"<listing class="ltx_lst_numbers_left" xml:id="tab1.1"><listingline xml:id="tab1.1.1"><text font="bold" xml:id="tab1.1.1.1">loop</text></listingline><listingline xml:id="tab1.1.2">  a;</listingline><listingline xml:id="tab1.1.3"><text font="bold" xml:id="tab1.1.3.1">end</text></listingline><listingline xml:id="tab1.1.4">b;</listingline><listingline xml:id="tab1.1.5"><text font="bold" xml:id="tab1.1.5.1">body</text></listingline><listingline xml:id="tab1.1.6">  c;</listingline><listingline xml:id="tab1.1.7">d;</listingline></listing>"#,
    )],
  );
}

/// `\SetAlgoNoLine` makes `\algocf@group` (an `\eIf` branch, a `\While` body) `\algocf@Noline` too
/// (algorithm2e.sty:1901-1905): each branch indents its body and gives the level back. pdflatex: "if c then", "a;"
/// indented, "else", "b;" indented, "end", "d;", "while w do", "e;" indented, "end", "f;". Red (int51a): every line
/// after the first branch one level deeper.
#[test]
fn algorithm2e_setalgonoline_groups_pop_their_indentation() {
  assert_elements(
    r"\documentclass{article}
\usepackage{algorithm2e}
\begin{document}\SetAlgoNoLine
\begin{algorithm}
\eIf{c}{a\;}{b\;}
d\;
\While{w}{e\;}
f\;
\end{algorithm}
\end{document}",
    PRELOAD,
    (0, 0),
    &[(
      "listing",
      "tab1.1",
      r#"<listing class="ltx_lst_numbers_left" xml:id="tab1.1"><listingline xml:id="tab1.1.1"><text font="bold" xml:id="tab1.1.1.1">if</text> <emph font="italic" xml:id="tab1.1.1.2">c</emph> <text font="bold" xml:id="tab1.1.1.3">then</text></listingline><listingline xml:id="tab1.1.2">  a;</listingline><listingline xml:id="tab1.1.3"><text font="bold" xml:id="tab1.1.3.1">else</text></listingline><listingline xml:id="tab1.1.4">  b;</listingline><listingline xml:id="tab1.1.5"><text font="bold" xml:id="tab1.1.5.1">end</text></listingline><listingline xml:id="tab1.1.6">d;</listingline><listingline xml:id="tab1.1.7"><text font="bold" xml:id="tab1.1.7.1">while</text> <emph font="italic" xml:id="tab1.1.7.2">w</emph> <text font="bold" xml:id="tab1.1.7.3">do</text></listingline><listingline xml:id="tab1.1.8">  e;</listingline><listingline xml:id="tab1.1.9"><text font="bold" xml:id="tab1.1.9.1">end</text></listingline><listingline xml:id="tab1.1.10">f;</listingline></listing>"#,
    )],
  );
}

/// subcaption.sty:251: `\subcaption` is `\caption` under `\@captype` `sub<type>`; in a minipage of a figure the
/// minipage is the sub-figure (`begin_caption_box` takes the sub-type's caption), numbered "1a"/"1b" — the figure's
/// counter stepped ahead of its own caption, as a `{subfigure}` does — and each `\label` names its panel. Red (int51a,
/// Perl): both labels on the figure, the panels id-less ("1", "1"). Witness 2011.13897 (fidelity3 N4). Repro
/// captions-floats/subcaption_in_a_minipage_labels_its_panel.
#[test]
fn subcaption_in_a_minipage_labels_its_panel() {
  assert_elements(
    repro!("captions-floats/subcaption_in_a_minipage_labels_its_panel.tex"),
    PRELOAD,
    (0, 0),
    &[(
      "figure",
      "S0.F1",
      r#"<figure class="ltx_panels_gallery" inlist="lof" labels="LABEL:f" panelcols="2" panelfill="0.8" panelline="1" panelrows="1" xml:id="S0.F1"><tags><tag>Figure 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Figure 1</tag></tags><figure class="ltx_figure_panel ltx_minipage" inlist="lof" labels="LABEL:a" panelwidth="0.4" vattach="bottom" width="138.0pt" xml:id="S0.F1.sf1"><tags><tag><text fontsize="90%" xml:id="S0.F1.sf1.1">(a)</text></tag><tag role="refnum">1a</tag></tags><p xml:id="S0.F1.sf1.2">X</p><toccaption><tag close=" ">a</tag>One</toccaption><caption><tag close=" "><text fontsize="90%" xml:id="S0.F1.sf1.3">(a)</text></tag><text fontsize="90%" xml:id="S0.F1.sf1.4">One</text></caption></figure><figure class="ltx_figure_panel ltx_minipage" inlist="lof" labels="LABEL:b" panelwidth="0.4" vattach="bottom" width="138.0pt" xml:id="S0.F1.sf2"><tags><tag><text fontsize="90%" xml:id="S0.F1.sf2.1">(b)</text></tag><tag role="refnum">1b</tag></tags><p xml:id="S0.F1.sf2.2">Y</p><toccaption><tag close=" ">b</tag>Two</toccaption><caption><tag close=" "><text fontsize="90%" xml:id="S0.F1.sf2.3">(b)</text></tag><text fontsize="90%" xml:id="S0.F1.sf2.4">Two</text></caption></figure><toccaption><tag close=" ">1</tag>Both.</toccaption><caption><tag close=": ">Figure 1</tag>Both.</caption></figure>"#,
    )],
  );
}

/// In a `table` the minipage holding a `\subcaption` is an `ltx:table`, as a `{subtable}` is (2502.10377's minipage
/// sub-tables): pdflatex 0 errors, "(a) One", "(b) Two", "Table 1: Both.", "See 1a, 1b, 1.".
#[test]
fn subcaption_in_a_minipage_of_a_table_is_a_subtable() {
  assert_elements(
    r"\documentclass{article}
\usepackage{subcaption}
\begin{document}
\begin{table}
\begin{minipage}{.4\linewidth}\subcaption{One}\label{a}\begin{tabular}{c}x\end{tabular}\end{minipage}%
\begin{minipage}{.4\linewidth}\subcaption{Two}\label{b}\begin{tabular}{c}y\end{tabular}\end{minipage}
\caption{Both.}\label{t}
\end{table}
See \ref{a}, \ref{b}, \ref{t}.
\end{document}",
    PRELOAD,
    (0, 0),
    &[(
      "table",
      "S0.T1",
      r#"<table class="ltx_panels_gallery" inlist="lot" labels="LABEL:t" panelcols="2" panelfill="0.8" panelline="1" panelrows="1" xml:id="S0.T1"><tags><tag>Table 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Table 1</tag></tags><table class="ltx_figure_panel ltx_minipage" inlist="lot" labels="LABEL:a" panelwidth="0.4" vattach="middle" width="138.0pt" xml:id="S0.T1.st1"><tags><tag><text fontsize="90%" xml:id="S0.T1.st1.1">(a)</text></tag><tag role="refnum">1a</tag></tags><toccaption><tag close=" ">a</tag>One</toccaption><caption><tag close=" "><text fontsize="90%" xml:id="S0.T1.st1.2">(a)</text></tag><text fontsize="90%" xml:id="S0.T1.st1.3">One</text></caption><tabular vattach="middle" xml:id="S0.T1.st1.4"><tbody><tr xml:id="S0.T1.st1.4.1"><td align="center" xml:id="S0.T1.st1.4.1.1">x</td></tr></tbody></tabular></table><table class="ltx_figure_panel ltx_minipage" inlist="lot" labels="LABEL:b" panelwidth="0.4" vattach="middle" width="138.0pt" xml:id="S0.T1.st2"><tags><tag><text fontsize="90%" xml:id="S0.T1.st2.1">(b)</text></tag><tag role="refnum">1b</tag></tags><toccaption><tag close=" ">b</tag>Two</toccaption><caption><tag close=" "><text fontsize="90%" xml:id="S0.T1.st2.2">(b)</text></tag><text fontsize="90%" xml:id="S0.T1.st2.3">Two</text></caption><tabular vattach="middle" xml:id="S0.T1.st2.4"><tbody><tr xml:id="S0.T1.st2.4.1"><td align="center" xml:id="S0.T1.st2.4.1.1">y</td></tr></tbody></tabular></table><toccaption><tag close=" ">1</tag>Both.</toccaption><caption><tag close=": ">Table 1</tag>Both.</caption></table>"#,
    )],
  );
}

/// The same for minipages in `\scalebox`es, and a `\ContinuedFloat`'s sub-captions go on lettering (c), (d) — the
/// float's counter is not stepped again. Repros captions-floats/subcaption_labels_in_scaled_panels_stay_on_their_panels
/// and captions-floats/continuedfloat_minipage_subcaption_continues_the_letters (RED since 64ze/65a and 57ad).
#[test]
fn subcaption_minipage_panels_in_boxes_and_continued_floats() {
  // Not `assert_elements`: a figure panel in the `\scalebox`'s `ltx:inline-block` is schema-invalid (2 jing lines),
  // in int51a as here (the `ltx:figure` the minipage became since 65a; the transform's own question).
  let (log, xml) = convert_with(
    repro!("captions-floats/subcaption_labels_in_scaled_panels_stay_on_their_panels.tex"),
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  latexml::util::test::assert_element(
    &xml,
    "figure",
    &["xml:id=\"S0.F1\""],
    r#"<figure class="ltx_panels_gallery" inlist="lof" panelcols="2" panelfill="0.72" panelline="1" panelrows="1" xml:id="S0.F1"><tags><tag>Figure 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Figure 1</tag></tags><inline-block align="center" class="ltx_figure_panel" depth="7.3pt" height="11.8pt" panelwidth="0.36" width="124.2pt" xml:id="S0.F1.1" xscale="0.9" xtranslate="-6.9pt" yscale="0.9" ytranslate="1.1pt"><figure class="ltx_minipage" inlist="lof" labels="LABEL:sa" vattach="middle" width="138.0pt" xml:id="S0.F1.sf1"><tags><tag><text fontsize="90%" xml:id="S0.F1.sf1.1">(a)</text></tag><tag role="refnum">1a</tag></tags><p xml:id="S0.F1.sf1.2">Panel A.</p><toccaption><tag close=" ">a</tag>Sub A</toccaption><caption><tag close=" "><text fontsize="90%" xml:id="S0.F1.sf1.3">(a)</text></tag><text fontsize="90%" xml:id="S0.F1.sf1.4">Sub A</text></caption></figure></inline-block><inline-block align="center" class="ltx_figure_panel" depth="7.3pt" height="11.8pt" panelwidth="0.36" width="124.2pt" xml:id="S0.F1.2" xscale="0.9" xtranslate="-6.9pt" yscale="0.9" ytranslate="1.1pt"><figure class="ltx_minipage" inlist="lof" labels="LABEL:sb" vattach="middle" width="138.0pt" xml:id="S0.F1.sf2"><tags><tag><text fontsize="90%" xml:id="S0.F1.sf2.1">(b)</text></tag><tag role="refnum">1b</tag></tags><p xml:id="S0.F1.sf2.2">Panel B.</p><toccaption><tag close=" ">b</tag>Sub B</toccaption><caption><tag close=" "><text fontsize="90%" xml:id="S0.F1.sf2.3">(b)</text></tag><text fontsize="90%" xml:id="S0.F1.sf2.4">Sub B</text></caption></figure></inline-block><toccaption class="ltx_centering"><tag close=" ">1</tag>Whole</toccaption><caption class="ltx_centering"><tag close=": ">Figure 1</tag>Whole</caption></figure>"#,
  );
  assert_elements(
    repro!("captions-floats/continuedfloat_minipage_subcaption_continues_the_letters.tex"),
    PRELOAD,
    (0, 0),
    &[
      (
        "figure",
        "S0.F1a",
        r#"<figure inlist="lof" xml:id="S0.F1a"><tags><tag>Figure 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Figure 1</tag></tags><figure class="ltx_minipage" inlist="lof" vattach="middle" width="138.0pt" xml:id="S0.F1a.sf3"><tags><tag><text fontsize="90%" xml:id="S0.F1a.sf3.1">(c)</text></tag><tag role="refnum">1c</tag></tags><rule height="28.5pt" width="28.5pt"/><toccaption><tag close=" ">c</tag>Three</toccaption><caption><tag close=" "><text fontsize="90%" xml:id="S0.F1a.sf3.2">(c)</text></tag><text fontsize="90%" xml:id="S0.F1a.sf3.3">Three</text></caption></figure><toccaption><tag close=" ">1</tag>Second part</toccaption><caption><tag close=": ">Figure 1</tag>Second part</caption></figure>"#,
      ),
      (
        "figure",
        "S0.F1b",
        r#"<figure inlist="lof" xml:id="S0.F1b"><tags><tag>Figure 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Figure 1</tag></tags><figure class="ltx_minipage" inlist="lof" vattach="middle" width="138.0pt" xml:id="S0.F1b.sf4"><tags><tag><text fontsize="90%" xml:id="S0.F1b.sf4.1">(d)</text></tag><tag role="refnum">1d</tag></tags><rule height="28.5pt" width="28.5pt"/><toccaption><tag close=" ">d</tag>Four</toccaption><caption><tag close=" "><text fontsize="90%" xml:id="S0.F1b.sf4.2">(d)</text></tag><text fontsize="90%" xml:id="S0.F1b.sf4.3">Four</text></caption></figure><toccaption><tag close=" ">1</tag>Third part</toccaption><caption><tag close=": ">Figure 1</tag>Third part</caption></figure>"#,
      ),
      (
        "figure",
        "S0.F2",
        r#"<figure inlist="lof" xml:id="S0.F2"><tags><tag>Figure 2</tag><tag role="refnum">2</tag><tag role="typerefnum">Figure 2</tag></tags><rule height="28.5pt" width="28.5pt"/><toccaption><tag close=" ">2</tag>Next</toccaption><caption><tag close=": ">Figure 2</tag>Next</caption></figure>"#,
      ),
    ],
  );
}

/// acmart.cls:3460-3489 makes its theorems `\AtEndPreamble` under `\@ifundefined`: an author's `\example` command
/// stays the author's. Red (int51a, Perl): a theorem opened in the cell, 6 errors. Witness 2510.04987. Repro
/// class-bindings/acmart_author_theorem_name_wins.
#[test]
fn acmart_author_theorem_name_wins() {
  assert_elements(
    repro!("class-bindings/acmart_author_theorem_name_wins.tex"),
    PRELOAD,
    (0, 0),
    &[(
      "td",
      "p1.1.1.2",
      r#"<td align="center" xml:id="p1.1.1.2"><text font="bold" xml:id="p1.1.1.2.1">ex1</text></td>"#,
    )],
  );
}

/// An author's `\newtheorem{theorem}{Theorem}` is numbered as the author says (not within sections), the class's
/// `{lemma}` on its counter: pdflatex "Theorem 1", "Theorem 2", "Lemma 3". Red (int51a): "Theorem 1" twice, "Lemma
/// 2". Witness 2306.01662. Repro class-bindings/acmart_author_newtheorem_numbers_as_the_author_says.
#[test]
fn acmart_author_newtheorem_numbers_as_the_author_says() {
  assert_elements(
    repro!("class-bindings/acmart_author_newtheorem_numbers_as_the_author_says.tex"),
    PRELOAD,
    (0, 0),
    &[(
      "section",
      "S1",
      r#"<section inlist="toc" xml:id="S1"><tags><tag>1</tag><tag role="autoref">section 1<text xml:id="S1.1"/></tag><tag role="refnum">1</tag><tag role="typerefnum">§1</tag></tags><title><tag close=". ">1</tag>S</title><toctitle><tag close=" ">1</tag>S</toctitle><theorem class="ltx_theorem_theorem" inlist="thm theorem:theorem" xml:id="Thmtheorem2"><tags><tag>Theorem 2</tag><tag role="autoref">Theorem 2<text xml:id="Thmtheorem2.1"/></tag><tag role="refnum">2</tag><tag role="typerefnum">Theorem 2</tag></tags><title class="ltx_runin" font="smallcaps">Theorem 2.</title><para xml:id="Thmtheorem2.p1"><p xml:id="Thmtheorem2.p1.1"><text font="italic" xml:id="Thmtheorem2.p1.1.1">B</text></p></para></theorem><theorem class="ltx_theorem_lemma" inlist="thm theorem:lemma" xml:id="Thmtheorem3"><tags><tag>Lemma 3</tag><tag role="autoref">Theorem 3<text xml:id="Thmtheorem3.1"/></tag><tag role="refnum">3</tag><tag role="typerefnum">Lemma 3</tag></tags><title class="ltx_runin" font="smallcaps">Lemma 3.</title><para xml:id="Thmtheorem3.p1"><p xml:id="Thmtheorem3.p1.1"><text font="italic" xml:id="Thmtheorem3.p1.1.1">C</text></p></para></theorem></section>"#,
    )],
  );
}

/// `acmthm=false` (acmart.cls:86-93) makes no theorems: `\theorem` stays undefined. pdflatex 0 errors: "undefined",
/// "Lemma 1. C".
#[test]
fn acmart_acmthm_false_makes_no_theorems() {
  assert_elements(
    r"\documentclass[sigconf,acmthm=false]{acmart}
\newtheorem{lemma}{Lemma}
\begin{document}
\title{T}\maketitle
\ifdefined\theorem defined\else undefined\fi
\begin{lemma}C\end{lemma}
\end{document}",
    PRELOAD,
    (0, 0),
    &[
      ("p", "p1.1", r#"<p xml:id="p1.1">undefined</p>"#),
      (
        "theorem",
        "Thmlemma1",
        r#"<theorem class="ltx_theorem_lemma" inlist="thm theorem:lemma" xml:id="Thmlemma1"><tags><tag>Lemma 1</tag><tag role="autoref">1<text xml:id="Thmlemma1.1"/></tag><tag role="refnum">1</tag><tag role="typerefnum">Lemma 1</tag></tags><title class="ltx_runin" font="smallcaps">Lemma 1.</title><para xml:id="Thmlemma1.p1"><p xml:id="Thmlemma1.p1.1"><text font="italic" xml:id="Thmlemma1.p1.1.1">C</text></p></para></theorem>"#,
      ),
    ],
  );
}

/// amsmath.sty:1309-1317 `\newdimen\alignsep@`, `\newdimen\totwidth@` (and the other alignment dimens), :1364
/// `\black@`: raw code reading them (0912.5438's shipped mathtools.sty v1.01a and fabntheorem.sty) finds them. Red
/// (int51a, Perl): 2 undefined. Repro package-bindings/amsmath_alignment_dimens_totwidth_alignsep_are_defined.
#[test]
fn amsmath_alignment_dimens_totwidth_alignsep_are_defined() {
  assert_elements(
    repro!("package-bindings/amsmath_alignment_dimens_totwidth_alignsep_are_defined.tex"),
    PRELOAD,
    (0, 0),
    &[("p", "p1.1", r#"<p xml:id="p1.1">Width 10.0pt.</p>"#)],
  );
}
