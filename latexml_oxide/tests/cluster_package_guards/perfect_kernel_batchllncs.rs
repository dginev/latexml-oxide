//! Red/green guards for LLNCSMARKS (MARKBIND's rule, DG 2026-11-16: a class binding computes the glyph its class
//! prints): llncs.cls's own `\@fnsymbol` (⋆ ⋆⋆ ⋆⋆⋆ † ‡ § ¶ ‖ ∗∗ †† ‡‡, llncs.cls:889-892). The title block's marks are
//! rows of `perfect_kernel_batchfmmarks` (`fmmarks_llncs_title_marks_are_the_class_symbols`,
//! `fmmarks_llncs_title_note_counts_first`); here the body's `\fnsymbol` and the footnote counter around the block.
use latexml::util::test::{
  convert_files_with, convert_with_setup_then, error_count, warning_count,
};

use super::{perfect_kernel_batch84b::text, perfect_kernel_batchfmmarks::creator_lines};

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $name,
      ".tex"
    ))
  };
}

/// The body's `\fnsymbol` is the class's, in text and in math, and its notes count from 1 after the title's two
/// `\thanks` (one institute: llncs.cls:879 `\fnnstart` 0, :1003): pdflatex "Body1 and ⋆, ⋆, a⋆; ∥; ∗∗." The text ⋆
/// in a box is text, as in the body; the ninth is ASCII "**" (`\textasteriskcentered`, as the kernel's and Perl's).
/// Distinct from the fmmarks rows (the title block's marks through the title hook): `\fnsymbol` outside the title and
/// the counter after it.
#[test]
fn body_fnsymbol_is_the_class_s() {
  let tex = r"\documentclass{llncs}
\title{A Title}
\author{Ann Able\thanks{Ann note.} \and Bob Baker\thanks{Bob note.}}
\institute{Lab A}
\begin{document}
\maketitle
Body\footnote{Body note.} and \fnsymbol{footnote}, $\fnsymbol{footnote}$, $a^{\mbox{\fnsymbol{footnote}}}$;
\setcounter{footnote}{8}\fnsymbol{footnote}; \setcounter{footnote}{9}\fnsymbol{footnote}.
\end{document}
";
  let (log, xml, ()) = convert_with_setup_then(tex, Some("ar5iv.sty"), || {}, |_| {});
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let body = xml.find("Body<note").expect("the body's paragraph");
  let start = xml[..body].rfind("<p").expect("its start");
  let para = &xml[start..start + xml[start..].find("</p>").expect("closed paragraph")];
  assert!(
    para.contains(r#"<note mark="1" role="footnote" xml:id="footnote1">"#),
    "{para}"
  );
  assert!(
    para.contains(r#"<XMTok name="star" role="MULOP">⋆</XMTok>"#),
    "{para}"
  );
  assert_eq!(
    text(para),
    "Body 1 1 footnote 1 Body note. and ⋆, ⋆ , a ⋆ ; ‖; **.",
    "{para}"
  );
}

/// llncs's `\maketitle` sets the footnote counter to 0 before its block (llncs.cls:932) and after it (:1003, `\fnnstart`
/// 0 under one institute): a note before `\maketitle` leaves the title's marks ⋆ ⋆⋆ and the body's next note 1, as
/// pdflatex prints "Pre1", "Ann Able⋆ and Bob Baker⋆⋆", "Body1" (MARKBIND's binary: the names unmarked, the body's 2).
/// Distinct from the marks' rows: the counter around the block (`\lx@maketitle@begin`, `\lx@maketitle@body`).
#[test]
fn the_title_counts_its_notes_from_zero() {
  let tex = r"\documentclass{llncs}
\title{A Title}
\author{Ann Able\thanks{Ann note.} \and Bob Baker\thanks{Bob note.}}
\institute{Lab A}
\begin{document}
Pre\footnote{Pre note.}
\maketitle
Body\footnote{Body note.}.
\end{document}
";
  let (log, xml, ()) = convert_with_setup_then(tex, Some("ar5iv.sty"), || {}, |_| {});
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let marks = xml
    .match_indices("<note ")
    .map(|(at, _)| {
      let tag = &xml[at..at + xml[at..].find('>').expect("start tag")];
      let mark = tag
        .split(" mark=\"")
        .nth(1)
        .and_then(|m| m.split('"').next());
      let body = &xml[at..at + xml[at..].find("</note>").expect("closed note")];
      format!(
        "{}={}",
        mark.unwrap_or("-"),
        text(body.split("</tags>").last().unwrap_or(body))
      )
    })
    .collect::<Vec<_>>();
  assert_eq!(marks, [
    "1=Pre note.",
    "⋆=Ann note.",
    "⋆⋆=Bob note.",
    "1=Body note."
  ]);
}

/// The body's note marks of `xml`, in order.
fn body_marks(xml: &str) -> Vec<String> {
  let body = &xml[xml.find("Body<note").expect("the body's note")..];
  body
    .match_indices("<note mark=\"")
    .map(|(at, open)| {
      body[at + open.len()..]
        .split('"')
        .next()
        .unwrap_or("")
        .to_string()
    })
    .collect()
}

/// llncs.cls:1023-1027 prints `\@subtitle` between the title and the names, so its `\thanks` steps the counter ahead of
/// theirs ("Sub⋆", "Ann Able⋆⋆"), which the title block's rehearsal does not count: a subtitle with a note, before or
/// after the `\author`, withdraws the title marks (the names' thanks unmarked, not the mis-linked ⋆ ⋆⋆ of LLNCSMARKS,
/// revLLNCS B1), and one with none keeps them; the subtitle's own note is written unmarked, its text whole. Distinct: the
/// `\subtitle` path of `\lx@llncs@title@scan`. The oracle's ⋆⋆ ⋆⋆⋆ are the RED row
/// `fmmarks_llncs_subtitle_note_counts_first_red`.
#[test]
fn a_note_in_the_subtitle_withdraws_the_title_marks() {
  let after = r"\documentclass{llncs}
\title{A Title}
\author{Ann Able\thanks{Ann note.} \and Bob Baker\thanks{Bob note.}}
\subtitle{Sub\thanks{Sub note.}}
\institute{Lab A}
\begin{document}
\maketitle
Body\footnote{Body note.}.
\end{document}
";
  let plain = after.replace(r"\subtitle{Sub\thanks{Sub note.}}", r"\subtitle{Sub}");
  let withdrawn = [
    "Ann Able ; thanks[]=Ann note. | affiliation=Lab A",
    "Bob Baker ; thanks[]=Bob note. | affiliation=Lab A",
  ];
  let kept = [
    "Ann Able ; thanks[⋆]=Ann note. | affiliation=Lab A",
    "Bob Baker ; thanks[⋆⋆]=Bob note. | affiliation=Lab A",
  ];
  let sub_note = r#"<pubnote role="thanks">Sub note.</pubnote>"#;
  for (tex, creators, notes) in [
    (
      repro!("fmmarks_llncs_subtitle_note_counts_first_red"),
      withdrawn,
      1,
    ),
    (after, withdrawn, 1),
    (plain.as_str(), kept, 0),
  ] {
    let (log, xml, ()) = convert_with_setup_then(tex, Some("ar5iv.sty"), || {}, |_| {});
    assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
    assert_eq!(creator_lines(&xml), creators, "{tex}");
    assert_eq!(body_marks(&xml), ["1"], "{tex}");
    assert_eq!(xml.matches("<pubnote").count(), notes, "{xml}");
    assert_eq!(xml.matches(sub_note).count(), notes, "{xml}");
  }
}

/// llncs.cls:1028-1029's counting pass runs `\@author` with `\thanks` gobbled, so a step of the footnote counter
/// outside a `\thanks` runs twice and shifts the names' marks: the binding's scan reads it by meaning (a `\let` copy of
/// `\footnotemark`; a `\let` copy of `\thanks`, which the pass does not gobble) and reads the document's macros in their
/// bodies (`\newcommand`, `\DeclareRobustCommand`'s inner `\fm␣`), and withdraws the marks: no glyph on a name (the
/// mark-only note's lost glyph a Warning), where LLNCSMARKS wrote ⋆ ⋆⋆ (revLLNCS B2; pdflatex ⋆⋆ ⋆⋆⋆). A macro that
/// steps nothing keeps them. Distinct: `steps_footnotes`'s meaning and body paths; the bare forms are
/// `fmmarks_llncs_counting_pass_steps_twice_red` and perfect_kernel_batchcr7's.
#[test]
fn a_names_macro_that_steps_withdraws_the_title_marks() {
  let shape = |preamble: &str, names: &str| {
    format!(
      "\\documentclass{{llncs}}\n{preamble}\n\\title{{A Title}}\n\\author{{{names}}}\n\\institute{{Lab A}}
\\begin{{document}}\n\\maketitle\nBody\\footnote{{Body note.}}.\n\\end{{document}}\n"
    )
  };
  let ann_thanks = r"Ann Able\thanks{Ann note.} \and ";
  let mark_withdrawn = [
    "Ann Able ; thanks[]=Ann note. | affiliation=Lab A",
    "Bob Baker | affiliation=Lab A",
  ];
  let cases: [(String, usize, Vec<&str>); 5] = [
    (
      repro!("fmmarks_llncs_names_macro_steps_twice_red").to_string(),
      1,
      mark_withdrawn.to_vec(),
    ),
    (
      shape(
        r"\let\fm\footnotemark",
        &format!(r"{ann_thanks}Bob Baker\protect\fm"),
      ),
      1,
      mark_withdrawn.to_vec(),
    ),
    (
      shape(
        r"\DeclareRobustCommand\fm{\footnotemark}",
        &format!(r"{ann_thanks}Bob Baker\fm"),
      ),
      1,
      mark_withdrawn.to_vec(),
    ),
    (
      shape(
        r"\let\th\thanks",
        r"Ann Able\th{Ann note.} \and Bob Baker\thanks{Bob note.}",
      ),
      0,
      vec![
        "Ann Able ; thanks[]=Ann note. | affiliation=Lab A",
        "Bob Baker ; thanks[]=Bob note. | affiliation=Lab A",
      ],
    ),
    (
      shape(
        r"\newcommand\corr{\textsuperscript{c}}\DeclareRobustCommand\eq{\textsuperscript{e}}",
        r"Ann Able\corr\thanks{Ann note.} \and Bob Baker\eq\thanks{Bob note.}",
      ),
      0,
      vec![
        "Ann Able ; footnotemark[c]= ; thanks[⋆]=Ann note. | affiliation=Lab A",
        "Bob Baker ; footnotemark[e]= ; thanks[⋆⋆]=Bob note. | affiliation=Lab A",
      ],
    ),
  ];
  for (tex, warnings, creators) in cases {
    let (log, xml, ()) = convert_with_setup_then(&tex, Some("ar5iv.sty"), || {}, |_| {});
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, warnings),
      "{tex}\n{log}"
    );
    assert_eq!(creator_lines(&xml), creators, "{tex}");
    assert_eq!(body_marks(&xml), ["1"], "{tex}");
  }
}

/// A later `\author` replaces the block before it (llncs.cls `\gdef\@author`), but the frontmatter's reading still
/// rehearses the replaced block and carries its footnote steps into the printed one (row R AUTHRESET, the engine's:
/// pdflatex "Ann Able⋆ and Bob Baker⋆⋆", LLNCSb ⋆⋆ ⋆⋆⋆ for one replaced `\thanks`, ⋆⋆⋆ † for a `\thanks` and a bare
/// `\footnotemark`): the binding reads the replaced names at `\maketitle` and withdraws its marks where they step, the
/// names' thanks unmarked (revLLNCSb item 1); a replaced template that steps nothing keeps them. Distinct: the
/// replaced names' path of `\lx@llncs@title@scan`, read with `\thanks` (the rehearsal steps it).
#[test]
fn a_replaced_author_withdraws_the_title_marks() {
  let template = repro!("fmmarks_llncs_replaced_author_steps_red").replace(
    r"\author{Ann Able\thanks{Ann note.}}",
    r"\author{First Author\inst{1}}",
  );
  assert!(template.contains("First Author"), "{template}");
  let withdrawn = [
    "Ann Able ; thanks[]=Ann note. | affiliation=Lab A",
    "Bob Baker ; thanks[]=Bob note. | affiliation=Lab A",
  ];
  let kept = [
    "Ann Able ; thanks[⋆]=Ann note. | affiliation=Lab A",
    "Bob Baker ; thanks[⋆⋆]=Bob note. | affiliation=Lab A",
  ];
  for (tex, creators) in [
    (repro!("fmmarks_llncs_replaced_author_steps_red"), withdrawn),
    (repro!("fmmarks_llncs_replaced_author_mark_red"), withdrawn),
    (template.as_str(), kept),
  ] {
    let (log, xml, ()) = convert_with_setup_then(tex, Some("ar5iv.sty"), || {}, |_| {});
    assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
    assert_eq!(creator_lines(&xml), creators, "{tex}");
    assert_eq!(body_marks(&xml), ["1"], "{tex}");
  }
}

/// llncs.cls:1003 sets the footnote counter after the title block to `\fnnstart`, the institutes `\institutename`
/// counted at its `\and`s when two or more (:877-881): under three (one in a group) the body's notes are 4 and 5, as
/// pdflatex prints them (1005.4552's three: its body notes 4 to 15; was 1 and 2). A copy of the class the paper ships
/// from before `\fnnstart` (v2.4-v2.10) sets 0 there: 1508.06013's v2.6 prints its body notes from 1 (here a copy
/// without it, `shipped_class_text`). Two pieces, the second holding an `\and` in a group, are three institutes too,
/// "Body4" (revLLNCSb item 3, was 3: the pieces counted where the list parts). Distinct: the counter after the block
/// (`\lx@llncs@institute@dispatch`'s `\fnnstart`), where the other guards' one institute gives 0.
#[test]
fn the_body_s_notes_count_after_the_institutes() {
  let tex = repro!("llncs_fnnstart_counts_the_institutes");
  let old_copy =
    "\\NeedsTeXFormat{LaTeX2e}\n\\ProvidesClass{llncs}[1999/04/27 v2.6]\n\\LoadClass{article}\n";
  let converted = |tex: &str| {
    let (log, xml, ()) = convert_with_setup_then(tex, Some("ar5iv.sty"), || {}, |_| {});
    (log, xml)
  };
  for ((log, xml), body) in [
    (converted(tex), ["4", "5"]),
    (
      convert_files_with(tex, &[("llncs.cls", old_copy)], Some("ar5iv.sty")),
      ["1", "2"],
    ),
    (
      converted(repro!("llncs_fnnstart_counts_nested_institutes")),
      ["4", "5"],
    ),
  ] {
    assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
    assert_eq!(creator_lines(&xml), [
      "Ann Able ; thanks[⋆]=Ann note. | affiliation=Lab A",
      "Bob Baker | affiliation=Lab B",
      "Cy Cole | affiliation=Lab C",
    ]);
    assert_eq!(body_marks(&xml), body);
  }
}
