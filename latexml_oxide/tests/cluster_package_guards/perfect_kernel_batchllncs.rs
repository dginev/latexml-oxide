//! Red/green guards for LLNCSMARKS (MARKBIND's rule, DG 2026-11-16: a class binding computes the glyph its class
//! prints): llncs.cls's own `\@fnsymbol` (⋆ ⋆⋆ ⋆⋆⋆ † ‡ § ¶ ‖ ∗∗ †† ‡‡, llncs.cls:889-892). The title block's marks are
//! rows of `perfect_kernel_batchfmmarks` (`fmmarks_llncs_title_marks_are_the_class_symbols`,
//! `fmmarks_llncs_title_note_counts_first`); here the body's `\fnsymbol` and the footnote counter around the block.
use latexml::util::test::{convert_with_setup_then, error_count, warning_count};

use super::perfect_kernel_batch84b::text;

/// The body's `\fnsymbol` is the class's, in text and in math, and its notes count from 1 after the title's two
/// `\thanks` (one institute: llncs.cls:879 `\fnnstart` 0, :1003): pdflatex "Body1 and ⋆, ⋆; ∥." Distinct from the
/// fmmarks rows (the title block's marks through the title hook): `\fnsymbol` outside the title and the counter after it.
#[test]
fn body_fnsymbol_is_the_class_s() {
  let tex = r"\documentclass{llncs}
\title{A Title}
\author{Ann Able\thanks{Ann note.} \and Bob Baker\thanks{Bob note.}}
\institute{Lab A}
\begin{document}
\maketitle
Body\footnote{Body note.} and \fnsymbol{footnote}, $\fnsymbol{footnote}$; \setcounter{footnote}{8}\fnsymbol{footnote}.
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
    "Body 1 1 footnote 1 Body note. and ⋆, ⋆ ; ‖.",
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
