//! Red/green guards for SHMARK (brief SHARED, DG's rulings 1-3 of 2026-11-16; FIDELITY_CHARTER §4, the mark rule): a
//! creator's note carries `mark`, the symbol it prints, and its text never opens with a copy of it; a footnote mark on a
//! name is a full copy of the note it marks; an affiliation's linking mark is no note. The XML keeps a copy per carrier
//! (Perl's `relocateAnnotations`, Base_Utility.pool.ltxml:895-901); the XSLT prints a shared note once. Code paths: the
//! marks a title block prints for its notes (`\lx@title@thanks`, `\lx@title@footnote`, the emitter's `shown_mark` and
//! `requested_mark`, the reading's `rehearse_title_first`, acmart's `\authornote`), the mark rule's pass
//! (`insert::mark_creator_notes`), the dropped linking marks (`insert::drop_linking_marks`) and the Link step
//! (`insert::link_marked_notes`). Witnesses 2502.19103, 1911.13299, 2312.10807, 2407.07614, 2606.00397, 1607.07280.
//! MARKSAFE (DG 2026-11-16, "a wrong glyph is worse than none"): the class's marks only under its own `\maketitle`, a
//! name's mark the class sets apart none, an author block's `\thefootnote` its name's, a mark alone its glyph in `mark`
//! only, a printed label no entry answers counted. Witnesses 1807.02791, 2607.15705, 2606.00446.
use latexml::util::test::{convert_with_setup_then, error_count, kpse_has, warning_count};

use super::{
  perfect_kernel_batch84b::{creators, text},
  perfect_kernel_batchfmmarks::{page_lines, page_of},
};

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
  notes_in(&converted(tex, warnings))
}

/// `tex` converted in process, with `warnings` Warnings and no Error.
#[track_caller]
fn converted(tex: &str, warnings: usize) -> String {
  let (log, xml, ()) = convert_with_setup_then(tex, Some("ar5iv.sty"), || {}, |_| {});
  assert_eq!(
    (error_count(&log), warning_count(&log)),
    (0, warnings),
    "{log}"
  );
  xml
}

/// The notes of `xml`'s creators, as [`marked_notes`] lists them.
pub(super) fn notes_in(xml: &str) -> Vec<Vec<String>> {
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
        // (a note of no content, `<note …/>`: a mark whose glyph is unknown (MARKSAFE), or a mark-only note,
        // `<note role="footnotemark" mark="1"/>` (PURE's re-homed name marks))
        let mut body = if creator[..open_end].ends_with('/') {
          String::new()
        } else {
          let close = open_end + creator[open_end..].find("</note>").expect("closed note");
          creator[open_end + 1..close].to_string()
        };
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
/// carrier's copy alike; a symbol that is the text's own stays (review revSHMARK S2). Distinct from the Link step: the
/// note's own mark, not a mark on another name.
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
  // (an unbound class's hand-typed legend, the generic path alone: scrartcl read raw, and an unknown class through
  // OmniBus, its Warning the missing binding's — the `$^*$` its carriers' mark, out of its text. The `\thanks` is still
  // unmarked there though scrartcl prints ∗: no binding declares its title marks, the behaviour test is open, OD #1053)
  // (OmniBus's `1` no contact of the name answers: a mark-only note after the name, which holds the name alone — PURE,
  // OD #1055; its `∗` the legend's)
  for (class, warnings, unanswered) in [
    ("scrartcl", 0, None),
    ("zzunknownclass", 1, Some("footnotemark[1]=")),
  ] {
    let tex = format!(
      r"\documentclass{{{class}}}
\title{{T}}
\author{{Ann Able$^{{1,*}}$\thanks{{Funded by X.}} \and Bob Baker$^{{1,*}}$\\ $^1$Univ A, Town\\ $^*$Equal contribution}}
\date{{}}
\begin{{document}}
\maketitle
Text.
\end{{document}}"
    );
    let named = |name: &str, notes: &[&str]| {
      let mut row = row(&[name]);
      row.extend(unanswered.map(ToString::to_string));
      row.extend(notes.iter().map(ToString::to_string));
      row
    };
    assert_eq!(
      marked_notes(&tex, warnings),
      [
        named("Ann Able", &[
          "thanks[]=Funded by X.",
          "thanks[∗]=Equal contribution"
        ]),
        named("Bob Baker", &["thanks[∗]=Equal contribution"]),
      ],
      "{class}"
    );
  }
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
  // (a leading symbol that is the note's words, not a copy of its mark: a formula `-` follows, a bare `*` the class's ∗
  // is not, a superscript of no footnote symbol and no label its name prints — review revSHMARK S2; the glyph stays)
  let tex = repro!("thanks_leading_symbol_text_is_no_mark_copy");
  let bare = tex.replace("$^*$-algebras are her field.", "* marks the starred items.");
  for (tex, ann) in [
    (tex, "thanks[*]=∗ -algebras are her field."),
    (bare.as_str(), "thanks[*]=* marks the starred items."),
  ] {
    assert_eq!(marked_notes(tex, 0), [
      row(&["Ann Able", ann]),
      row(&["Bob Baker", "thanks[†]=3 He detectors were funded by X."]),
    ]);
  }
  assert_eq!(
    marked_notes(repro!("ieee_thanks_leading_label_is_no_mark"), 0),
    [
      row(&["Ann Able", "thanks[]=3 He detectors were funded by X."]),
      row(&["Bob Baker", "thanks[]=* marks the starred items below."]),
    ]
  );
}

/// The marks a title block prints for its notes: article's `\@fnsymbol` on each `\thanks`, an author's `\footnote` the
/// `\thanks` it is there (article.cls:241), counted after the title's own `\thanks` (article.cls:203-214 sets the title
/// first: the reading's `rehearse_title_first`), so a `\footnotemark[1]` beside a name is the title's note — no
/// creator's, it stays the mark it is; under `titlepage` the body's arabic marks, which the observer does not read: no
/// mark, nor under a class whose `\maketitle` prints none (IEEEtran), nor on a REVTeX 4.1/4.2 name's `\footnote`, set
/// in the class's own letters (2301.00220), nor on a REVTeX 4.0 one given no number, at a counter the reading does not
/// count (review revSHMARKb). Distinct from the Link step's guard: the mark each note is written with.
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
  // (REVTeX 4.1/4.2 set a name's `\footnote` in their minipage letters, or drop it under revtex4-2's `reprint`, not in
  // the title's symbols: Perl's note contact, no mark — 2301.00220, review revSHMARK B1; REVTeX 4.0's given
  // `\footnote[1]` ∗ is `an_affiliations_linking_mark_is_no_note`'s)
  let preprint = repro!("revtex42_name_footnote_is_its_note_contact");
  let reprint = preprint.replace("[aps,prl,preprint]", "[aps,prl,reprint]");
  for tex in [preprint, reprint.as_str()] {
    let xml = converted(tex, 0);
    assert_eq!(notes_in(&xml), [
      row(&["Ann Able"]),
      row(&["Bob Baker"]),
      row(&["Cy Cole"])
    ]);
    assert_eq!(creators(&xml), [
      row(&[
        "Ann Able",
        "thanks=Equal contribution.",
        "affiliation=Univ A"
      ]),
      row(&["Bob Baker", "note=Visiting Univ C.", "affiliation=Univ B"]),
      row(&["Cy Cole", "email=cy@b.edu", "affiliation=Univ B"]),
    ]);
  }
  // (REVTeX 4.0 prints an unnumbered one at a counter its `\thanks`, `\email` and `\altaffiliation` step too, which the
  // reading does not count: Perl's note contact, no mark, after either or neither, one or two of them — review
  // revSHMARKb; a given number prints its own symbol, marked so: 1607.07280)
  let r40 = repro!("revtex40_name_footnote_unnumbered_is_its_note_contact");
  let thanks = r"\thanks{Equal contribution.}";
  let ann = |contact: &str| {
    let mut ann = vec!["Ann Able"];
    if !contact.is_empty() {
      ann.push(contact);
    }
    ann.push("affiliation=Univ A");
    row(&ann)
  };
  let bob = row(&["Bob Baker", "note=Visiting Univ C.", "affiliation=Univ B"]);
  for (tex, first) in [
    (r40.to_string(), "thanks=Equal contribution."),
    (r40.replace(thanks, r"\email{ann@a.edu}"), "email=ann@a.edu"),
    (r40.replace(thanks, ""), ""),
  ] {
    let xml = converted(&tex, 0);
    assert_eq!(notes_in(&xml), [row(&["Ann Able"]), row(&["Bob Baker"])]);
    assert_eq!(creators(&xml), [ann(first), bob.clone()]);
  }
  let xml = converted(
    &r40
      .replace(thanks, "")
      .replace(r"Univ C.}", r"Univ C.}\footnote{Second.}"),
    0,
  );
  assert_eq!(notes_in(&xml), [row(&["Ann Able"]), row(&["Bob Baker"])]);
  assert_eq!(creators(&xml), [
    ann(""),
    row(&[
      "Bob Baker",
      "note=Visiting Univ C.",
      "note=Second.",
      "affiliation=Univ B"
    ])
  ]);
  assert_eq!(
    marked_notes(
      &r40.replace(r"\footnote{Visiting", r"\footnote[2]{Visiting"),
      0
    ),
    [
      row(&["Ann Able"]),
      row(&["Bob Baker", "thanks[†]=Visiting Univ C."])
    ]
  );
  let xml = converted(repro!("revtex41_name_footnote_is_its_note_contact"), 0);
  assert_eq!(notes_in(&xml), [row(&["Ann Able"]), row(&["Bob Baker"])]);
  assert_eq!(creators(&xml), [
    row(&["Ann Able", "note=ann@a.edu", "affiliation=Univ A, Town"]),
    row(&[
      "Bob Baker",
      "note=Corresponding author: bob@b.edu",
      "affiliation=Univ A, Town"
    ]),
  ]);
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
/// 1911.13299's `\ \footnotemark[N]` marks, which SHGIVEN carries back to the name before (DG ruling 4), are copies too:
/// Ann's †, Bob's and Dan's ‡, no Warning (pin updated at the merge: alone this branch lost them with 1 Warning).
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
    marked_notes(repro!("author_thanks_shared_by_footnotemark_marks_kept"), 0),
    [
      row(&[
        "Ann Able",
        "thanks[†]=Univ A",
        "thanks[*]=Equal contribution."
      ]),
      row(&[
        "Bob Baker",
        "thanks[*]=Equal contribution.",
        "thanks[‡]=Univ B"
      ]),
      row(&["Cat Cole", "thanks[†]=Univ A", "thanks[‡]=Univ B"]),
      row(&["Dan Dorn", "thanks[‡]=Univ B"]),
    ]
  );
  // (a mark no note answers stays a mark alone, its glyph in `mark` only, never its text as well: 2607.15705's
  // `\authornotemark[2]`, pdflatex "Bob Baker∗†"; MARKSAFE)
  let xml = converted(
    repro!("fmmarks_acmart_authornotemark_without_its_note_red"),
    0,
  );
  assert_eq!(notes_in(&xml), [
    row(&[
      "Ann Able",
      "thanks[*]=Ann Able and Bob Baker contributed equally."
    ]),
    row(&[
      "Bob Baker",
      "thanks[*]=Ann Able and Bob Baker contributed equally.",
      "footnotemark[†]="
    ]),
    row(&["Cat Cole"]),
  ]);
  assert_eq!(
    xml
      .matches(r#"<note mark="†" role="footnotemark" xml:id="id6"/>"#)
      .count(),
    1,
    "{xml}"
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
      "mark=*",
      "affiliation=Univ A, Town",
      "affiliation=Lab B, City"
    ]),
    row(&["Bob Baker", "mark=†", "affiliation=Univ C, Village"]),
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

/// The marks a title block prints are its class's only where the class's own `\maketitle` runs
/// (`latex_constructs::class_maketitle_in_force`; DG 2026-11-16, "a wrong glyph is worse than none"): a document's own
/// prints marks unknown here (`\let\@makefnmark\relax`: none; `\def\thefootnote{\arabic…}`: 1, 2), so neither a
/// `\thanks` note nor a name's kernel `\footnotemark` carries one; a redefinition calling the class's by meaning
/// (`\let\oldmt\maketitle`) keeps them. titling's `\thanksmarkseries{arabic}` is its binding's declared title marks
/// ("Ann Able1"). Distinct from the mark rule: whether the class's marks hold, not where a note's mark goes.
#[test]
fn marks_only_where_the_classs_maketitle_runs() {
  assert_eq!(
    marked_notes(repro!("maketitle_redefined_prints_no_marks_red"), 0),
    [
      row(&["Ann Able", "thanks[]=First note."]),
      row(&["Bob Baker", "thanks[]=Second note."]),
    ]
  );
  // (pdflatex: "Ann Able Bob Baker", the footnote "1 Shared." — the mark this document's `\maketitle` sets aside)
  assert_eq!(
    marked_notes(
      r"\documentclass{article}
\makeatletter\renewcommand\maketitle{\begingroup\let\@makefnmark\relax\begin{center}\begin{tabular}[t]{c}\@author\end{tabular}\end{center}\endgroup}\makeatother
\title{A Title}
\author{Ann Able\footnotemark[1] \and Bob Baker\footnotemark[1]}
\begin{document}
\maketitle
\footnotetext[1]{Shared.}
Body.
\end{document}",
      0
    ),
    [
      row(&["Ann Able", "footnote[]=Shared."]),
      row(&["Bob Baker", "footnote[]=Shared."]),
    ]
  );
  // (pdflatex: "Ann Able∗ Bob Baker†")
  assert_eq!(
    marked_notes(
      r"\documentclass{article}
\makeatletter\let\oldmt\maketitle\renewcommand\maketitle{\oldmt\thispagestyle{empty}}\makeatother
\title{A Title}
\author{Ann Able\thanks{First note.} \and Bob Baker\thanks{Second note.}}
\begin{document}
\maketitle
Body.
\end{document}",
      0
    ),
    [
      row(&["Ann Able", "thanks[*]=First note."]),
      row(&["Bob Baker", "thanks[†]=Second note."]),
    ]
  );
  assert_eq!(
    marked_notes(repro!("titling_thanksmarkseries_marks_red"), 0),
    [
      row(&["Ann Able", "thanks[1]=First note."]),
      row(&["Bob Baker", "thanks[2]=Second note."]),
    ]
  );
}

/// A `\renewcommand\thefootnote` in an author block holds in its cell of the `\@author` tabular alone, which `\and`, `\\`
/// and `&` end (latex.ltx:17219-17222), as in TeX: the class's `\maketitle` sets `\thefootnote` before `\@author` expands,
/// the redefinition after it. Both readers of the marks keep it there: the flush, which runs a block's declarations ahead of
/// its names (`emit::declarations` keys each to its cell's name, `latex_constructs::title_cell_declared`: 1807.02791's
/// `\arabic` before its one name, pdflatex "Ann Able 1 2"; one in Bob's cell renumbers Bob alone, "Ann Able∗ Bob Baker2
/// Cy Cole‡"), and the rehearsal, whose `\thanks` marks end with the cell (`rehearsal::end_of_cell`: "Ann Able1 Bob
/// Baker†"; past Bob's `\\` and `\and`, "Cat Cole‡"). One in the preamble the class's `\maketitle` overrides ("∗", "†").
/// Distinct from the class's gate: which `\thefootnote` a field's marks are made by (review revMARKSAFE S1).
#[test]
fn a_thefootnote_the_author_block_sets_holds_for_its_name() {
  assert_eq!(
    marked_notes(
      repro!("fmmarks_single_name_marks_after_newline_thin_spaces_red"),
      0
    ),
    [row(&["Ann Able", "footnotemark[1]=", "footnotemark[2]="])]
  );
  let block = |preamble: &str, author: &str| {
    format!(
      r"\documentclass{{article}}
{preamble}
\title{{T}}
\author{{{author}Ann Able\footnotemark[1] \and Bob Baker\footnotemark[2]}}
\date{{}}
\begin{{document}}
\maketitle
Text.
\end{{document}}"
    )
  };
  assert_eq!(
    marked_notes(
      &block("", r"\renewcommand{\thefootnote}{\arabic{footnote}}"),
      0
    ),
    [
      row(&["Ann Able", "footnotemark[1]="]),
      row(&["Bob Baker", "footnotemark[†]="]),
    ]
  );
  assert_eq!(
    marked_notes(
      &block(r"\renewcommand{\thefootnote}{\arabic{footnote}}", ""),
      0
    ),
    [
      row(&["Ann Able", "footnotemark[*]="]),
      row(&["Bob Baker", "footnotemark[†]="]),
    ]
  );
  assert_eq!(
    marked_notes(repro!("marksafe_thefootnote_set_in_a_cell_holds_there"), 0),
    [
      row(&["Ann Able", "footnotemark[*]="]),
      row(&["Bob Baker", "footnotemark[2]="]),
      row(&["Cy Cole", "footnotemark[‡]="]),
    ]
  );
  assert_eq!(
    marked_notes(repro!("marksafe_thefootnote_set_in_a_cell_thanks"), 0),
    [
      row(&["Ann Able", "thanks[1]=Note A."]),
      row(&["Bob Baker", "thanks[†]=Note B."]),
    ]
  );
  // (Univ B's `\footnotemark[1]`, "∗" on its line, is a mark the emitter writes on no name: counted)
  assert_eq!(
    marked_notes(
      repro!("marksafe_thefootnote_set_in_a_cell_ends_at_its_line"),
      1
    ),
    [
      row(&["Ann Able", "thanks[*]=Note A."]),
      row(&["Bob Baker", "thanks[2]=Note B."]),
      row(&["Cat Cole", "thanks[‡]=Note C."]),
    ]
  );
  // (one in a line's cell, which holds no name, renumbers no name: pdflatex "Ann Able∗ Bob Baker†")
  assert_eq!(
    marked_notes(
      r"\documentclass{article}
\title{T}
\author{Ann Able\footnotemark[1] \\ \renewcommand{\thefootnote}{\arabic{footnote}}Univ A \and Bob Baker\footnotemark[2]}
\date{}
\begin{document}
\maketitle
Body.
\end{document}",
      0
    ),
    [
      row(&["Ann Able", "footnotemark[*]="]),
      row(&["Bob Baker", "footnotemark[†]="]),
    ]
  );
}

/// A name's mark the class sets apart, which no reading knows, is written with none (`latex_constructs::
/// drop_unknown_title_mark`): REVTeX 4.1/4.2 set a name's `\footnotemark` in their minipage letters ("Ann Ablea",
/// revtex4-2.cls:2555-2557, `lx_title_footnote_own_marks`), bare or given its number, its text after `\maketitle` paired
/// by the counter. Distinct from the class's gate: the class's `\maketitle` runs, its name marks are another series.
#[test]
fn a_names_mark_revtex_sets_apart_is_unknown() {
  // (its note never placed, nothing names it: one counted Warning, `insert::count_unknown_marks`)
  let xml = converted(
    repro!("revtex_name_footnotemark_leaves_the_marks_unknown"),
    1,
  );
  assert_eq!(notes_in(&xml), [
    row(&["Ann Able", "footnotemark[]="]),
    row(&["Bob Baker"]),
    row(&["Cy Cole"]),
  ]);
  // (and on the page no glyph: not the XSLT's `†`, revMARKSAFE B1 — the unmarked mark-only note shows nothing)
  assert_eq!(
    page_lines(&page_of(&xml)),
    (row(&["Ann Able", "Bob Baker", "Cy Cole"]), Vec::new())
  );
  assert_eq!(
    xml
      .matches(r#"<note role="footnotemark" xml:id="footnote1"/>"#)
      .count(),
    1,
    "{xml}"
  );
  // (pdflatex: "Ann Ablea", the footnote "1 Note A.")
  assert_eq!(
    marked_notes(
      r"\documentclass[aps,prl,preprint]{revtex4-2}
\begin{document}
\title{A Title}
\author{Ann Able\footnotemark[1]}
\affiliation{Univ A}
\author{Bob Baker}
\affiliation{Univ B}
\begin{abstract}Abs.\end{abstract}
\maketitle
\footnotetext[1]{Note A.}
Body.
\end{document}",
      0
    ),
    [
      row(&["Ann Able", "footnote[]=Note A."]),
      row(&["Bob Baker"])
    ]
  );
}

/// A mark whose glyph is unknown here prints none, in the XML or on the page (DG 2026-11-16: a wrong or invented glyph is
/// worse than none; the generic engine never invents one): under an unbound class (scrartcl's own `\maketitle` prints
/// `\fnsymbol`, `latex_constructs::title_marks_unknown`) a name's `\thanks` is written unmarked and the XSLT shows its text
/// under the name with no `†` (`LaTeXML-meta-xhtml.xsl`'s unmarked creator note, OD #990; revMARKSAFE B1); a name's
/// `\footnotemark[1]` is no "1" (Perl's number key, the advisor's I-3) — its note no number key names yet, a counted
/// Warning, or its `\footnotetext[1]` paired by the number, an Info (`insert::count_unknown_marks`). Distinct from the
/// class's gate: an unbound class's marks, and what the page shows of a note with none.
#[test]
fn a_mark_whose_glyph_is_unknown_prints_none() {
  let case = |tex: &str, warnings: usize| {
    let (log, xml, ()) = convert_with_setup_then(tex, Some("ar5iv.sty"), || {}, |_| {});
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, warnings),
      "{log}"
    );
    let html = page_of(&xml);
    let block = &html[html.find("ltx_authors").expect("an author block")..];
    assert!(!block.contains("ltx_note_mark"), "{html}");
    (log, notes_in(&xml), page_lines(&html))
  };
  let (log, notes, page) = case(repro!("marksafe_unbound_class_thanks_print_no_glyph"), 0);
  assert_eq!(notes, [
    row(&["Ann Able", "thanks[]=Note A."]),
    row(&["Bob Baker", "thanks[]=Note B."]),
  ]);
  assert_eq!(
    page,
    (
      row(&["Ann Able | under: Note A.", "Bob Baker | under: Note B."]),
      Vec::new()
    )
  );
  assert!(
    log.contains(
      "Info:frontmatter:unknown_mark 2 author note(s) written with no mark, the mark the title prints being unknown \
       here: each under its author"
    ),
    "{log}"
  );
  let (log, notes, page) = case(
    repro!("marksafe_unbound_class_footnotemark_given_number_unknown"),
    1,
  );
  assert!(
    log.contains(
      "Warning:frontmatter:unanswered_mark_lost A footnote mark on an author's name whose note cannot be identified, \
       its glyph unknown here, is written with none"
    ),
    "{log}"
  );
  assert_eq!(notes, [
    row(&["Ann Able", "thanks[]=Equal contribution."]),
    row(&["Bob Baker", "footnotemark[]="]),
  ]);
  assert_eq!(
    page,
    (
      row(&["Ann Able | under: Equal contribution.", "Bob Baker"]),
      Vec::new()
    )
  );
  let (_, notes, page) = case(
    repro!("marksafe_unbound_class_footnotemark_answered_by_its_text"),
    0,
  );
  assert_eq!(notes, [
    row(&["Ann Able", "footnote[]=Note A."]),
    row(&["Bob Baker", "footnote[]=Note B."]),
  ]);
  assert_eq!(
    page,
    (
      row(&["Ann Able | under: Note A.", "Bob Baker | under: Note B."]),
      Vec::new()
    )
  );
  // (one `\footnotetext[1]` two names' marks share is one note, counted once; the flag a declared key, no KeyVals
  // report: review revMARKSAFEb N1, N2)
  let (log, notes, page) = case(repro!("marksafe_unbound_class_shared_text_counted_once"), 0);
  assert!(
    log.contains(
      "Info:frontmatter:unknown_mark 2 author note(s) written with no mark, the mark the title prints being unknown \
       here: each under its author"
    ),
    "{log}"
  );
  assert!(!log.contains("KeyVals key '_"), "{log}");
  assert_eq!(notes, [
    row(&["Ann Able", "footnote[]=Equal contribution."]),
    row(&["Bob Baker", "footnote[]=Equal contribution."]),
    row(&["Cy Cole", "footnote[]=Cy note."]),
  ]);
  assert_eq!(
    page,
    (
      row(&[
        "Ann Able | under: Equal contribution.",
        "Bob Baker | under: Equal contribution.",
        "Cy Cole | under: Cy note."
      ]),
      Vec::new()
    )
  );
}

/// A document's wrapper of its class's `\maketitle` by meaning (`\let\oldmt\maketitle \renewcommand\maketitle{\oldmt}`)
/// runs the class's title page, which pdflatex prints as the plain document's: the output, the page and the log's counts
/// are the plain document's, under a class binding (mnras, IEEEtran, llncs). Only the class
/// file's own drop of `\maketitle` makes its marks unknown (`\maketitle:redefined@class`, `state::note_redefined`;
/// review revMARKSAFEb B1). Distinct from the class's gate: who redefined `\maketitle`, not which class.
#[test]
fn a_wrapper_of_the_class_maketitle_is_neutral() {
  let wrapper = "\\let\\oldmt\\maketitle\n\\renewcommand\\maketitle{\\oldmt}\n";
  for wrapped in [
    repro!("marksafe_maketitle_wrapper_mnras_is_neutral"),
    repro!("marksafe_maketitle_wrapper_ieeetran_is_neutral"),
    repro!("marksafe_maketitle_wrapper_llncs_is_neutral"),
  ] {
    assert!(wrapped.contains(wrapper), "{wrapped}");
    let plain = wrapped.replace(wrapper, "");
    let convert = |tex: &str| {
      let (log, xml, ()) = convert_with_setup_then(tex, Some("ar5iv.sty"), || {}, |_| {});
      assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
      assert!(!log.contains("unknown_mark"), "{log}");
      let page = page_lines(&page_of(&xml));
      (notes_in(&xml), page, xml)
    };
    let (plain_notes, plain_page, plain_xml) = convert(&plain);
    let (notes, page, xml) = convert(wrapped);
    assert_eq!((&notes, &page), (&plain_notes, &plain_page), "{wrapped}");
    // (the whole document alike)
    assert_eq!(xml, plain_xml);
    if wrapped.contains("{mnras}") {
      // (mnras's own `\@fnsymbol`, ⋆ †: the `\footnotemark[n]` and the body's `\footnotetext[n]` one note; MARKBIND)
      assert_eq!(plain_notes, [
        row(&["Ann Able", "footnote[⋆]=Ann note."]),
        row(&["Bob Baker", "footnote[†]=Bob note."])
      ]);
    }
    if wrapped.contains("{llncs}") {
      // (llncs's own `\@fnsymbol`, ⋆ ⋆⋆, llncs.cls:889, :935: llncs_cls.rs's title hook; LLNCSMARKS)
      assert_eq!(plain_notes, [
        row(&["Ann Able", "thanks[⋆]=Ann note."]),
        row(&["Bob Baker", "thanks[⋆⋆]=Bob note."])
      ]);
    }
    if wrapped.contains("IEEEtran") {
      // (Ann's `\footnote` IEEEtran's contact, Perl's; Bob's mark kept, no Warning)
      assert_eq!(notes, [
        row(&["Ann Able"]),
        row(&["Bob Baker", "footnotemark[1]="])
      ]);
    }
  }
}

/// lni's binding reads the raw lni.cls, whose own `\maketitle` numbers the title's notes in arabic (lni.cls:475-477): the
/// binding declares those marks, kept in force over the class file's own drop (`\maketitle:redefined@class`), so
/// `\thanks` prints 1, 2 as pdflatex does (2407.04435; review revMARKSAFEb S2). Distinct from the wrapper's: a binding
/// computes its own class's glyph.
#[test]
fn lni_prints_its_own_arabic_title_marks() {
  if !kpse_has("lni.cls") {
    eprintln!("SKIP: lni.cls is not in this TeX Live");
    return;
  }
  let (log, xml, ()) = convert_with_setup_then(
    repro!("marksafe_lni_title_marks_are_arabic"),
    Some("ar5iv.sty"),
    || {},
    |_| {},
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(!log.contains("unknown_mark"), "{log}");
  assert_eq!(notes_in(&xml), [
    row(&["Ann Able", "thanks[1]=Note A."]),
    row(&["Bob Baker", "thanks[2]=Note B."]),
  ]);
}

/// A label authblk prints on a name that no `\affil` prints is a mark the frontmatter places nowhere, counted
/// (`authblk_sty::count_lost_labels`; FIDELITY_CHARTER: no silent loss): `\author[1,*,\S]` whose `\S` only the `\date`
/// line explains (2606.00446; pdflatex "Ann Able1,*,§"), one Warning; its answered `*` a note of each carrier. Both
/// sides are read as printed (`printed_label_pieces`), so a label a macro prints is the line's (`\author[\affA]`).
#[test]
fn a_class_label_no_entry_answers_is_counted() {
  let tex = repro!("fmmarks_authblk_double_star_label_and_date_note_red");
  let (log, xml, ()) = convert_with_setup_then(tex, Some("ar5iv.sty"), || {}, |_| {});
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(
    log.contains(
      "Warning:frontmatter:unanswered_mark_lost authblk prints 1 label(s) of the names no \\affil explains, \
       written on no name: §"
    ),
    "{log}"
  );
  assert_eq!(notes_in(&xml), [
    row(&[
      "Ann Able",
      "thanks[*]=These authors contributed equally to this work."
    ]),
    row(&[
      "Bob Baker",
      "thanks[*]=These authors contributed equally to this work."
    ]),
    row(&["Cat Cole"]),
  ]);
  assert_eq!(creators(&xml), [
    row(&["Ann Able", "affiliation=Univ A, Town", "mark=*"]),
    row(&["Bob Baker", "affiliation=Univ A, Town", "mark=*"]),
    row(&[
      "Cat Cole",
      "affiliation=Univ A, Town",
      "affiliation=Present address: Lab B, City"
    ]),
  ]);
  // (both sides compared as printed, `printed_label_pieces`: `\author[\affA]`, `\affA` 1, is `\affil[1]`'s; review
  // revMARKSAFE S3)
  let (log, xml, ()) = convert_with_setup_then(
    repro!("authblk_macro_label_is_its_printed_text"),
    Some("ar5iv.sty"),
    || {},
    |_| {},
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(creators(&xml), [row(&["Ann Able", "affiliation=Univ A"])]);
  // (a line's label read as authblk reads it, not verbatim: `\affil[$^1$]` prints 1, as `\author[$^{1,2}$]`'s pieces do,
  // 2106.15893; pdflatex "Ann Able1,2 Bob Baker⋆,1")
  let (log, _, ()) = convert_with_setup_then(
    r"\documentclass{article}
\usepackage{authblk}
\author[$^{1,2}$]{Ann Able}
\author[$^{\star,1}$]{Bob Baker}
\affil[$^1$]{Univ A}
\affil[$^2$]{Univ B}
\affil[$\star$]{corresponding author}
\title{T}
\date{}
\begin{document}
\maketitle
Text.
\end{document}",
    Some("ar5iv.sty"),
    || {},
    |_| {},
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
}
