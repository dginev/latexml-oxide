//! Red/green guards for FM S2b r1 (OXIDIZED_DESIGN_DIVERGENCES #837 "S2b"; user frontmatter rulings 2026-10-17, M3
//! observer): for a family whose policy entry is `written` — the pilot, revtex4_support's `revtex` — the observer's
//! reading is written into the document once today's frontmatter is placed, where every difference from today's is
//! one of the writer's operations (`observer::writer`). Under the shadow switch only (`LATEXML_FM_SHADOW=1`): unset,
//! the XML is today's (the shadow-neutral guards of the families not switched stay byte-identical, batchfms2a2r7/r8).
//! Each guard converts with the switch on and off and asserts every creator whole.
use latexml::util::test::{convert_with_setup_then, error_count, warning_count};
use latexml_engine::base_utilities::set_fm_shadow_override;

/// Convert `tex` with the shadow switch `on`, asserting a clean conversion; its creators, each with the whitespace
/// between its tags removed.
fn creators(tex: &str, on: bool) -> Vec<String> {
  let (log, xml, ()) = convert_with_setup_then(
    tex,
    Some("ar5iv.sty"),
    move || set_fm_shadow_override(Some(on)),
    |_| (),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let mut found = Vec::new();
  let mut rest = xml.as_str();
  while let Some(start) = rest.find("<creator ") {
    let end =
      rest[start..].find("</creator>").expect("a closed creator") + start + "</creator>".len();
    let mut compact = String::new();
    let mut pending_space = false;
    for part in rest[start..end].split('\n') {
      let part = part.trim_start();
      if pending_space && !part.starts_with('<') {
        compact.push(' ');
      }
      compact.push_str(part);
      pending_space = !part.ends_with('>');
    }
    found.push(compact);
    rest = &rest[end..];
  }
  found
}

/// A footnote mark's tags as the revtex binding writes them: `n` as revtex's front matter prints it
/// (`\frontmatter@thefootnote`, the counter's symbols: 1 `*`, 2 `†`, KNOWN_PERL_ERRORS #1189).
fn mark_tags(n: &str, id: &str) -> String {
  format!(
    "<tags><tag>{n}</tag><tag role=\"autoref\">footnote\u{a0}{n}<text xml:id=\"{id}.1\"/></tag><tag role=\"refnum\">{n}</tag>\
     <tag role=\"typerefnum\">footnote {n}</tag></tags>"
  )
}

/// Operation 3 (owners by mark): REVTeX's `\affiliation` after a name list is every name's (revtex4-2.cls's grouping,
/// today's reading), but the affiliations open with the `\footnotemark` the names' `\footnote[n]` printed — in the PDF
/// Ann's mark 1 stands at Univ A and Univ B, Bob's 2 at Univ C. Written: each affiliation goes to the name showing
/// its mark, its mark kept as it prints (today's: all three both names', the group rule; before 80u, Ann Univ A alone,
/// Bob all three). Bob's Univ C is the copy today's numbered `footnotex3a`; Ann's, taken, gives it back its id
/// (`footnotex3`; red, 80u before `reclaim_ids`: `footnotex3a` alone, no `footnotex3` in the document). Shape
/// `perfect_kernel_batchfms2a2r7::FOOTNOTE_MARK`.
#[test]
fn footnote_marked_affiliations_go_to_their_names() {
  let tex = r"\documentclass[aps,pra,preprint]{revtex4-2}
\begin{document}
\title{T}
\author{Ann Able\footnote[1]{E-mail: ann.able@example.org} and Bob Baker\footnote[2]{E-mail: bob.baker@example.org}}
\affiliation{\footnotemark[1]Univ A, City}
\affiliation{\footnotemark[1]Univ B, Town}
\affiliation{\footnotemark[2]Univ C, Village}
\begin{abstract}
Abstract.
\end{abstract}
\maketitle
Text.
\end{document}";
  let affiliation = |n: &str, id: &str, text: &str| {
    format!(
      "<contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><note mark=\"{n}\" role=\"footnotemark\" xml:id=\"{id}\">{}\
       </note>{text}</contact>",
      mark_tags(n, id)
    )
  };
  assert_eq!(creators(tex, true), [
    format!(
      "<creator role=\"author\"><personname>Ann Able</personname><contact name=\"Note:\u{a0}\" role=\"note\">E-mail: \
       ann.able@example.org</contact>{}{}</creator>",
      affiliation("*", "footnotex1", "Univ A, City"),
      affiliation("*", "footnotex2", "Univ B, Town")
    ),
    format!(
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname><contact name=\"Note:\u{a0}\" \
       role=\"note\">E-mail: bob.baker@example.org</contact>{}</creator>",
      affiliation("†", "footnotex3", "Univ C, Village")
    ),
  ]);
  // (unset, today's reading: the writer does not run)
  let today = creators(tex, false);
  assert_eq!(today.len(), 2);
  assert!(
    today[1].contains("Univ A, City") && today[1].contains("Univ B, Town"),
    "{today:?}"
  );
}

/// Operations 1 and 3 (split by marks, a legend made a note): one `\affiliation{$^1$Univ A, City \\ $^\star${…}}` under
/// `Ann Able$^{1}$, Bob Baker$^{1,\star}$` is cut at its marks — Univ A both names', the starred legend a
/// correspondence note of Bob, who shows the star, its ink kept as styled label markup (the label-text ruling). Was:
/// the whole call, legend and all, both names' affiliation. Shape `perfect_kernel_batchfms2a2r8::API_LEGEND`.
#[test]
fn a_marked_affiliation_is_cut_at_its_marks() {
  let tex = r"\documentclass[aps,prl]{revtex4-2}
\begin{document}
\title{T}
\author{Ann Able$^{1}$, Bob Baker$^{1,\star}$}
\affiliation{$^1$Univ A, City \\ $^\star${corresponding author, e-mail: bob@x.org}}
\begin{abstract}
Abstract.
\end{abstract}
\maketitle
Text.
\end{document}";
  assert_eq!(creators(tex, true), [
    "<creator role=\"author\"><personname>Ann Able<sup xml:id=\"id1\"><text font=\"italic\" xml:id=\"id1.1\">1</text>\
     </sup></personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><sup xml:id=\"id2\">1</sup>Univ A, City\
     </contact></creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker<sup xml:id=\"id3\"><text font=\"italic\" \
     xml:id=\"id3.1\">1,⋆</text></sup></personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><sup \
     xml:id=\"id4\">1</sup>Univ A, City</contact><note class=\"ltx_note_frontmatter ltx_thanks_correspondence\" \
     role=\"thanks\" xml:id=\"id5\"><sup xml:id=\"id5.1\">⋆</sup>corresponding author, e-mail: bob@x.org</note>\
     </creator>",
  ]);
  let today = creators(tex, false);
  assert!(
    today.iter().all(|c| c.contains("corresponding author")),
    "{today:?}"
  );
}

/// The creator holding `name`, and whether it lists each of `has` and none of `lacks`.
fn holds(creators: &[String], name: &str, has: &[&str], lacks: &[&str]) {
  let creator = creators
    .iter()
    .find(|c| c.contains(name))
    .unwrap_or_else(|| panic!("{name}: {creators:#?}"));
  for text in has {
    assert!(creator.contains(text), "{name} lacks {text}: {creator}");
  }
  for text in lacks {
    assert!(!creator.contains(text), "{name} holds {text}: {creator}");
  }
}

/// FM S2b r2 (the census's worse 35): a mark glued of several symbols is the marks it spells — `$^{\dagger\natural
/// \sharp}$` under entries `†`, `♮`, `♯` (`parted_label`: the most known pieces before the fewest). Red (r1): only `†`
/// matched, the `♮` and `♯` affiliations left to the block. Witness 1004.1402.
#[test]
fn glued_symbol_marks_part_into_their_affiliations() {
  let tex = r"\documentclass[aps,prd]{revtex4-1}
\begin{document}
\title{T}
\author{David Lowe$^{\dagger}$ and Shubho Roy$^{\dagger\natural\sharp}$}
\affiliation{$^{\dagger}$Univ A, City}
\affiliation{$^{\natural}$Univ B, City}
\affiliation{$^{\sharp}$Univ C, City}
\maketitle
Text.
\end{document}
";
  let on = creators(tex, true);
  assert!(on.iter().all(|c| c.contains("<personname")), "{on:#?}");
  holds(&on, "Roy", &["Univ A", "Univ B", "Univ C"], &[]);
  holds(&on, "Lowe", &["Univ A"], &["Univ B", "Univ C"]);
}

/// FM S2b r2: a number with a letter glued to it requests the number, the letter a note's mark — `$^{1{\rm a}}$` under
/// entries `1`, `2`. Red (r1): `1a` matched no entry, Han and Hu kept no affiliation. Witness 1904.02582.
#[test]
fn a_number_with_a_glued_letter_requests_the_number() {
  let tex = r"\documentclass[aps,prb]{revtex4-1}
\begin{document}
\title{T}
\author{Ann Able$^{1}$, Bob Baker$^{1{\rm a}}$, Cat Cole$^{2{\rm a}}$}
\affiliation{$^{1}$Univ A, City}
\affiliation{$^{2}$Univ B, City}
\maketitle
Text.
\end{document}
";
  let on = creators(tex, true);
  holds(&on, "Bob Baker", &["Univ A"], &["Univ B"]);
  holds(&on, "Cat Cole", &["Univ B"], &["Univ A"]);
}

/// FM S2b r2: a mark after a name's comma is that name's when it stands nearer the comma, its formula's base and math
/// shifts counted — `Han,${}^1$ Vogt`: measured from the script, the empty base `{}` put the mark nearer the next name.
/// Red (r1): Han kept no affiliation. Witness 1603.00635.
#[test]
fn a_mark_after_the_comma_with_an_empty_base_is_the_name_before() {
  let tex = r"\documentclass[aps,pra]{revtex4-1}
\begin{document}
\title{T}
\author{Ann Able,${}^1$ Bob Baker,${}^{1,2}$ and Cat Cole${}^{1,3}$}
\affiliation{${}^1$Univ A, City}
\affiliation{${}^2$Univ B, City}
\affiliation{${}^3$Univ C, City}
\maketitle
Text.
\end{document}
";
  let on = creators(tex, true);
  holds(&on, "Ann Able", &["Univ A"], &["Univ B", "Univ C"]);
  holds(&on, "Bob Baker", &["Univ A", "Univ B"], &["Univ C"]);
  holds(&on, "Cat Cole", &["Univ A", "Univ C"], &["Univ B"]);
}

/// FM S2b r2: revtex marks its front matter's footnotes with symbols (revtex4.cls:5020-5022 `\frontmatter@thefootnote`,
/// `\@fnsymbol` of the counter; the binding declares it the title block's), each `\author`'s note stepping the counter:
/// the names show ∗, †, ‡, and an affiliation those symbols lead is their names' (ruling 2: a symbol line's carriers, a
/// note's printed mark among them). Red (r1): the rehearsal began every `\author` block at the unstepped counter, all
/// three notes showed `∗`, and no name carried a symbol: both affiliations went to the block. Witness 1608.06509.
#[test]
fn author_footnote_symbols_carry_their_affiliations() {
  let tex = r"\documentclass{revtex4}
\begin{document}
\title{T}
\author{Ann Able\footnote{ann@x.org}}
\author{Bob Baker\footnote{bob@x.org}}
\author{Cat Cole\footnote{cat@x.org}}
\affiliation{$^{\ast, \ddagger}$ Univ A, City}
\affiliation{$^{\dagger}$ Univ B, City}
\maketitle
Text.
\end{document}
";
  let on = creators(tex, true);
  assert!(on.iter().all(|c| c.contains("<personname")), "{on:#?}");
  holds(&on, "Ann Able", &["Univ A"], &["Univ B"]);
  holds(&on, "Bob Baker", &["Univ B"], &["Univ A"]);
  holds(&on, "Cat Cole", &["Univ A"], &["Univ B"]);
}

/// FM S2b r2: a note's printed mark labels a name only when the name shows no mark of its own — Shukla shows `†` and
/// his e-mail's footnote `∗` (revtex's first front-matter mark), Bera shows `†∗`: the `∗` affiliation is Bera's, Shukla
/// keeps his `†` one alone. Red (80t before the refinement): Shukla's note mark took Bera's `∗` affiliation. Witness
/// 2406.01694.
#[test]
fn a_named_mark_holder_keeps_its_note_mark_as_the_notes() {
  let tex = r"\documentclass[aps,prd]{revtex4-1}
\begin{document}
\title{T}
\author{Ann Bera$^{\dagger,\ast}$, Pat Shukla$^\dagger$ \footnote{E-mail: pat@x.org}}
\affiliation{$^\dagger$Univ P, City \\ $^\ast$Univ I, Town}
\maketitle
Text.
\end{document}
";
  let on = creators(tex, true);
  holds(&on, "Ann Bera", &["Univ P", "Univ I"], &[]);
  holds(&on, "Pat Shukla", &["Univ P"], &["Univ I"]);
}

/// FM S2b r3 (the off-vs-on census): a mark set in `\( … \)` is cut whole — the pieces of `\affiliation{\( ^{1}
/// \)Univ A \\ \( ^{2} \)Univ B}` open at the `\(`, as at a `$`. Red (80t): the cut took the script and left `\(` in
/// the piece before, so each piece re-digested `^{1} \)Univ A \\ \(`: "Script ^ can only appear in math mode" and
/// "Attempt to end mode math". Witnesses hep-ph0110378, 2307.12984 (`\({}^{a}\)`).
#[test]
fn a_mark_in_paren_math_is_cut_whole() {
  let tex = r"\documentclass[aps,prd]{revtex4-1}
\begin{document}
\title{T}
\author{Ann Able\( ^{1} \), Bob Baker\protect\( ^{1,2} \) and Cat Cole\({}^{2}\)}
\affiliation{\( ^{1} \)Univ A, City\\ \({}^{2}\)Univ B, Town}
\maketitle
Text.
\end{document}
";
  let on = creators(tex, true);
  holds(&on, "Ann Able", &["Univ A"], &["Univ B"]);
  holds(&on, "Bob Baker", &["Univ A", "Univ B"], &[]);
  holds(&on, "Cat Cole", &["Univ B"], &["Univ A"]);
}

/// FM S2b r3: a number range a name shows requests every number in it — `Ann Able$^{1-3}$` the entries 1, 2 and 3.
/// Red (80u): the range read as its ends, Ann kept Univ A and Univ C, not Univ B. Witness 2012.15817
/// (`Bauer$^{1-4}$`, `Wieschaus$^{2-4}$`).
#[test]
fn a_number_range_requests_every_number_in_it() {
  let tex = r"\documentclass[aps,prl]{revtex4-2}
\begin{document}
\title{T}
\author{Ann Able$^{1-3}$, Bob Baker$^{2}$ and Cat Cole$^{1,4}$}
\affiliation{$^1$Univ A, City\\ $^2$Univ B, Town\\ $^3$Univ C, Village\\ $^4$Univ D, Hamlet}
\maketitle
Text.
\end{document}
";
  let on = creators(tex, true);
  holds(&on, "Ann Able", &["Univ A", "Univ B", "Univ C"], &[
    "Univ D",
  ]);
  holds(&on, "Bob Baker", &["Univ B"], &[
    "Univ A", "Univ C", "Univ D",
  ]);
  holds(&on, "Cat Cole", &["Univ A", "Univ D"], &[
    "Univ B", "Univ C",
  ]);
}

/// FM S2b r3: the reading records the run of a call today's attaches at once, as `frontmatter_api.rs` does — revtex's
/// symbol-led `\affiliation{$^\flat$…}` —, so an unmarked line after it continues that run (the group rule): every name's.
/// Red (80u): the reading's run began afresh at the unmarked line and stopped at the names the marked lines reached,
/// the last name alone; today's gave every name the line, and the writer declined the paper. Witness 1303.7097.
#[test]
fn an_unmarked_line_after_symbol_led_ones_is_the_groups() {
  let tex = r"\documentclass[aps,prl]{revtex4-1}
\begin{document}
\title{T}
\author{Ann Able$^{*}$, Bob Baker$^{\flat}$, Cat Cole$^{*,\S}$}
\affiliation{$^*$ Univ A, City}
\affiliation{$^\flat$ Univ B, Town}
\affiliation{Council C, Village}
\affiliation{$^\S$ Univ D, Hamlet}
\maketitle
Text.
\end{document}
";
  let on = creators(tex, true);
  holds(&on, "Ann Able", &["Univ A", "Council C"], &[
    "Univ B", "Univ D",
  ]);
  holds(&on, "Bob Baker", &["Univ B", "Council C"], &[
    "Univ A", "Univ D",
  ]);
  holds(&on, "Cat Cole", &["Univ A", "Univ D", "Council C"], &[
    "Univ B",
  ]);
}
