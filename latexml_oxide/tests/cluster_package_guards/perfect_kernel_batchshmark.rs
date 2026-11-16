//! Red/green guards for SHMARK (brief SHARED, DG's rulings 1-3 of 2026-11-16; FIDELITY_CHARTER §4, the mark rule): a
//! creator's note carries `mark`, the symbol it prints, and its text never opens with a copy of it; a footnote mark on a
//! name is a full copy of the note it marks; an affiliation's linking mark is no note. The XML keeps a copy per carrier
//! (Perl's `relocateAnnotations`, Base_Utility.pool.ltxml:895-901); the XSLT prints a shared note once. Code paths: the
//! marks a title block prints for its notes (`\lx@title@thanks`, `\lx@title@footnote`, the emitter's `shown_mark` and
//! `requested_mark`, the reading's `rehearse_title_first`, acmart's `\authornote`), the mark rule's pass
//! (`insert::mark_creator_notes`), the dropped linking marks (`insert::drop_linking_marks`) and the Link step
//! (`insert::link_marked_notes`). Witnesses 2502.19103, 1911.13299, 2312.10807, 2407.07614, 2606.00397, 1607.07280.
use latexml::util::test::{convert_with_setup_then, error_count, warning_count};

use super::perfect_kernel_batch84b::{creators, text};

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $name,
      ".tex"
    ))
  };
}

/// The notes of `tex`'s creators, converted in process with `warnings` Warnings and no Error: per creator its
/// personname's text, then each `ltx:note` it holds as `<role>[<mark>]=<text>` (its tags left out), in order.
#[track_caller]
pub(super) fn marked_notes(tex: &str, warnings: usize) -> Vec<Vec<String>> {
  let (log, xml, ()) = convert_with_setup_then(tex, Some("ar5iv.sty"), || {}, |_| {});
  assert_eq!(
    (error_count(&log), warning_count(&log)),
    (0, warnings),
    "{log}"
  );
  xml
    .match_indices("<creator ")
    .map(|(start, _)| {
      let end = start + xml[start..].find("</creator>").expect("closed creator");
      let creator = &xml[start..end];
      let mut out = Vec::new();
      if let Some(at) = creator.find("<personname>") {
        let close = at
          + creator[at..]
            .find("</personname>")
            .expect("closed personname");
        out.push(text(&creator[at + "<personname>".len()..close]));
      }
      for (at, _) in creator.match_indices("<note ") {
        let open_end = at + creator[at..].find('>').expect("note tag");
        let attribute = |name: &str| {
          creator[at..open_end]
            .split(&format!(" {name}=\""))
            .nth(1)
            .and_then(|m| m.split('"').next())
            .unwrap_or("")
            .to_string()
        };
        let close = open_end + creator[open_end..].find("</note>").expect("closed note");
        let mut body = creator[open_end + 1..close].to_string();
        if let (Some(from), Some(to)) = (body.find("<tags>"), body.find("</tags>")) {
          body.replace_range(from..to + "</tags>".len(), "");
        }
        out.push(format!(
          "{}[{}]={}",
          attribute("role"),
          attribute("mark"),
          text(&body)
        ));
      }
      out
    })
    .collect()
}

fn row(items: &[&str]) -> Vec<String> { items.iter().map(ToString::to_string).collect() }

/// The mark rule's pass (`insert::mark_creator_notes`): a note's leading formula mark is its mark where its class prints
/// none (IEEEtran's `\thanks{$^{*}$Equal contribution}`, 2312.10807), a legend line's `$^*$` repeating the mark the
/// emitter gave it is taken out (2407.07614, 2105.14239), and a bare `*` its names show is the emitter's
/// `requested_mark`, taken out of the text (ieeeconf's `\thanks{*Authors to whom correspondence …}`, 2606.00397). Each
/// carrier's copy alike. Distinct from the Link step: the note's own mark, not a mark on another name.
#[test]
fn a_notes_mark_is_never_in_its_text() {
  assert_eq!(
    marked_notes(repro!("ieee_thanks_symbol_legend_note_mark_lifted"), 0),
    [
      row(&["Ann Able", "thanks[∗]=Equal contribution"]),
      row(&["Bob Baker", "thanks[∗]=Equal contribution"]),
      row(&["Cat Cole", "thanks[†]=Corresponding author: Cat Cole"]),
    ]
  );
  assert_eq!(
    marked_notes(
      repro!("author_block_symbol_legend_note_mark_not_in_its_text"),
      0
    ),
    [
      row(&["Ann Able", "thanks[∗]=Equal contribution"]),
      row(&["Bob Baker", "thanks[∗]=Equal contribution"]),
      row(&["Cat Cole"]),
    ]
  );
  assert_eq!(
    marked_notes(repro!("thanks_bare_symbol_is_the_notes_mark"), 0),
    [
      row(&["Ann Able"]),
      row(&["Bob Baker", "thanks[*]=Corresponding author."]),
    ]
  );
  // (ieeeconf's `\maketitle` prints no automatic mark, ieeeconf.cls:3592 `\def\thefootnote{}`: a plain `\thanks` has no
  // mark, the hand-typed `*` its name shows is one — review revSHRENDb)
  assert_eq!(
    marked_notes(
      r"\documentclass[letterpaper, 10 pt, conference]{ieeeconf}
\title{T}
\author{Ann Able$^{1}$ and Bob Baker$^{1}$*%
\thanks{This work was funded by X.}%
\thanks{$^{1}$Ann Able and Bob Baker are with Univ A, Town.}%
\thanks{*Corresponding author.}}
\begin{document}
\maketitle
Text.
\end{document}",
      0
    ),
    [
      row(&["Ann Able"]),
      row(&[
        "Bob Baker",
        "thanks[]=This work was funded by X.",
        "thanks[*]=Corresponding author."
      ]),
    ]
  );
}

/// The marks a title block prints for its notes: article's `\@fnsymbol` on each `\thanks`, an author's `\footnote` the
/// `\thanks` it is there (article.cls:241), counted after the title's own `\thanks` (article.cls:203-214 sets the title
/// first: the reading's `rehearse_title_first`), so a `\footnotemark[1]` beside a name is the title's note — no
/// creator's, it stays the mark it is; under `titlepage` the body's arabic marks, which the observer does not read: no
/// mark, nor under a class whose `\maketitle` prints none (IEEEtran). Distinct from the Link step's guard: the mark
/// each note is written with.
#[test]
fn a_thanks_note_carries_the_mark_its_title_prints() {
  assert_eq!(
    marked_notes(repro!("author_footnote_is_the_marked_thanks_note"), 0),
    [row(&["Ann Able", "thanks[*]=Univ X"]), row(&["Bob Baker"])]
  );
  assert_eq!(
    marked_notes(repro!("author_thanks_mark_after_a_title_thanks"), 0),
    [
      row(&["Ann Able", "thanks[†]=Note X."]),
      row(&["Bob Baker", "footnotemark[*]="]),
    ]
  );
  assert_eq!(
    marked_notes(
      r"\documentclass[titlepage]{article}
\title{T}\author{Ann Able\thanks{Note X.} \and Bob Baker\thanks{Note Y.}}
\begin{document}\maketitle Text.\end{document}",
      0
    ),
    [
      row(&["Ann Able", "thanks[]=Note X."]),
      row(&["Bob Baker", "thanks[]=Note Y."]),
    ]
  );
  // (IEEEtran's `\maketitle` prints none, IEEEtran.cls:4758 `\let\@makefnmark\relax`: no mark — review revSHRENDb)
  assert_eq!(
    marked_notes(
      r"\documentclass[journal]{IEEEtran}
\begin{document}
\title{T}
\author{Ann Able and Bob Baker%
\thanks{This work was funded by X.}}
\maketitle
Text.
\end{document}",
      0
    ),
    [
      row(&["Ann Able"]),
      row(&["Bob Baker", "thanks[]=This work was funded by X."]),
    ]
  );
}

/// The Link step (`insert::link_marked_notes`): a mark alone on a name that a note of another creator carries becomes a
/// full copy of that note, as Perl's `relocateFootnote_aux` fills a mark (latex_constructs.pool.ltxml:518-527) — a
/// document's `\samethanks[1]` (2502.19103), acmart's `\authornotemark[1]` (acmart.cls:1733-1738, the binding's marked
/// `\authornote`), 1911.13299's `\footnotemark[1]`; the copy carries the note's mark, so the XSLT prints the note once.
/// 1911.13299's `\ \footnotemark[N]` marks, lost with 1 Warning here, are SHGIVEN's (DG ruling 4).
#[test]
fn a_mark_on_a_name_is_a_copy_of_the_note_it_marks() {
  assert_eq!(
    marked_notes(repro!("author_samethanks_is_a_copy_of_the_marked_note"), 0),
    [
      row(&["Ann Able", "thanks[*]=Equal Contribution."]),
      row(&["Bob Baker", "thanks[*]=Equal Contribution."]),
      row(&["Cat Cole"]),
    ]
  );
  assert_eq!(
    marked_notes(
      repro!("acmart_authornotemark_is_a_copy_of_the_marked_note"),
      0
    ),
    [
      row(&["Ann Able", "thanks[*]=Equal contribution."]),
      row(&["Bob Baker", "thanks[*]=Equal contribution."]),
      row(&["Cat Cole"]),
    ]
  );
  assert_eq!(
    marked_notes(repro!("author_thanks_shared_by_footnotemark_marks_kept"), 1),
    [
      row(&["Ann Able", "thanks[*]=Equal contribution."]),
      row(&["Bob Baker", "thanks[*]=Equal contribution."]),
      row(&["Cat Cole", "thanks[†]=Univ A", "thanks[‡]=Univ B"]),
      row(&["Dan Dorn"]),
    ]
  );
}

/// An affiliation's leading `\footnotemark[n]`, the mark the reading paired the entry by (`mark:footnote`), is no note
/// of its own where a note of its creator carries that mark (DG ruling 3, no institution marks; 1607.07280's REVTeX
/// `\affiliation{\footnotemark[1]Univ A}` under `Ann Able\footnote[1]{…}`, the name's `\footnote` its note marked as
/// REVTeX prints it) — nor where the name's own mark, filled by a `\footnotetext` after `\maketitle`, carries it.
#[test]
fn an_affiliations_linking_mark_is_no_note() {
  let tex = repro!("revtex_affiliation_footnotemark_link_is_no_note");
  assert_eq!(marked_notes(tex, 0), [
    row(&["Ann Able", "thanks[*]=E-mail: ann@a.org"]),
    row(&["Bob Baker", "thanks[†]=E-mail: bob@c.org"]),
  ]);
  // (each affiliation its own name's, as the reading paired it by the mark)
  let (_, xml, ()) = convert_with_setup_then(tex, Some("ar5iv.sty"), || {}, |_| {});
  assert_eq!(creators(&xml), [
    row(&[
      "Ann Able",
      "affiliation=Univ A, Town",
      "affiliation=Lab B, City"
    ]),
    row(&["Bob Baker", "affiliation=Univ C, Village"]),
  ]);
  assert_eq!(
    marked_notes(
      r"\documentclass{article}
\usepackage{authblk}
\title{T}
\author{Ann Able\footnotemark[1]}
\affil{\footnotemark[1]Univ X}
\begin{document}
\maketitle
\footnotetext[1]{Note A.}
Text.
\end{document}",
      0
    ),
    [row(&["Ann Able", "footnote[*]=Note A."])]
  );
}
