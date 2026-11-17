//! MARKKEY round 1 (brief 2026-11-16_brief_MARKKEY; DG's rulings of 2026-11-16): frontmatter marks keyed by the footnote
//! number TeX pairs them by, note identity across per-carrier copies, and the page's mark rules. Code paths, one test each:
//! a block's `\footnotetext[n]` given to the names whose `\footnotemark[n]` shows n (`resolver::answer_note_texts`,
//! `Event::number`) and a name's `\footnotemark[n]` filled with the block's note of number n (`insert::link_marked_notes`,
//! `_fnvalue`) — OD #1058; a mark glued past a name's comma measured from the comma (`resolver::mark_ledger`, COMMAMARK);
//! IEEEtran's unmarked `\thanks` kept at the block's level (`resolver::keep_note`); one note's copies marked as one
//! (`relocate::one_note` at `relocate_annotations`, the kernel's `relocate_footnote` and the Link step), and on the page
//! a shared unmarked note under a generated `[a]`, an unlinked name mark shown only where its glyph recurs
//! (`LaTeXML-meta-xhtml.xsl`) — OD #1059. Rows assert the pdflatex oracle in the FMRG suite's two layers
//! (`perfect_kernel_batchfmmarks::Row`). Witnesses 2407.04435, 1111.1979, 2006.10886, 1510.02728, 1108.5155,
//! 2508.05506, 2012.13912, 1503.07953.
use latexml::util::test::convert_with_setup_then;

use super::perfect_kernel_batchfmmarks::{Row, assert_green};

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $name,
      ".tex"
    ))
  };
}

/// Each copy of a note that names another as its first (`idref`): `(its creator's name, the name of the creator holding
/// the note its idref names)`, in document order.
fn copies(tex: &str) -> Vec<(String, String)> {
  let (_, xml, ()) = convert_with_setup_then(tex, Some("ar5iv.sty"), || {}, |_| {});
  let attribute = |tag: &str, name: &str| {
    tag
      .split(&format!(" {name}=\""))
      .nth(1)
      .and_then(|value| value.split('"').next())
      .map(ToString::to_string)
  };
  // (per creator its name, then each note's start tag)
  let creators: Vec<(String, Vec<&str>)> = xml
    .match_indices("<creator ")
    .map(|(start, _)| {
      let creator = &xml[start..start + xml[start..].find("</creator>").expect("closed creator")];
      let name = creator.find("<personname>").map_or(String::new(), |at| {
        let from = at + "<personname>".len();
        creator[from..from + creator[from..].find("</personname>").expect("closed")].to_string()
      });
      let notes = creator
        .match_indices("<note ")
        .map(|(at, _)| &creator[at..at + creator[at..].find('>').expect("start tag")])
        .collect();
      (name, notes)
    })
    .collect();
  let holder = |id: &str| {
    creators
      .iter()
      .find(|(_, notes)| {
        notes
          .iter()
          .any(|tag| attribute(tag, "xml:id").as_deref() == Some(id))
      })
      .map_or_else(|| "(no element)".to_string(), |(name, _)| name.clone())
  };
  creators
    .iter()
    .flat_map(|(name, notes)| {
      notes
        .iter()
        .filter_map(|tag| attribute(tag, "idref"))
        .map(|id| (name.clone(), holder(&id)))
    })
    .collect()
}

fn pairs(rows: &[(&str, &str)]) -> Vec<(String, String)> {
  rows
    .iter()
    .map(|(a, b)| ((*a).to_string(), (*b).to_string()))
    .collect()
}

/// A footnote mark answers its note by the number it shows, as TeX pairs them (latex.ltx `\@xfootnotemark`,
/// `\@xfootnotenext`) and Perl's `relocateFootnote` fills them: a `\footnotetext[n]` written inside `\author` is a copy
/// on each name whose `\footnotemark[n]` shows n, marked with the glyph that mark prints (2407.04435's lni block, every
/// text on the last name before; Cat's mark after `\textsuperscript{,}`, lost before with a Warning, is the comma's name's);
/// a `\footnotemark[1]` beside another name's `\thanks` is filled with it by number where no binding knows the glyph
/// scrartcl prints, the shared unmarked note printed once under a generated `[a]` (the advisor's ADVGEN I-3 probe:
/// Bob kept a mark-only "1", no copy); a `\footnotemark[2]` naming the date's `\thanks` by number is that note's mark,
/// shown though the date's publication note prints none (`ltx_mark_titlenote`; revMARKKEY1 B2: hidden before).
/// Distinct paths: the reading's footnote-text pairing, the Link step's number pairing and its title-note number.
#[test]
fn a_numbered_mark_answers_its_note_by_number() {
  assert_green(&[
    Row {
      repro:      "markkey_footnotetext_in_the_block_answers_its_names_by_number",
      tex:        repro!("markkey_footnotetext_in_the_block_answers_its_names_by_number"),
      warnings:   0,
      creators:   &[
        "Ann Able ; thanks[*]=Univ A, Town A",
        "Bob Baker ; thanks[*]=Univ A, Town A",
        "Cat Cole ; thanks[*]=Univ A, Town A ; thanks[†]=Univ B, Town B",
        "Dan Dorn ; thanks[‡]=Univ C, Town C",
      ],
      moves:      "",
      page:       &[
        "Ann Able ; *→note",
        "Bob Baker ; *→note",
        "Cat Cole ; *→note ; † plain",
        "Dan Dorn ; ‡ plain",
        "margin: Univ A, Town A ; Univ B, Town B ; Univ C, Town C",
      ],
      page_moves: "",
    },
    Row {
      repro:      "markkey_footnotemark_names_the_dates_thanks",
      tex:        repro!("markkey_footnotemark_names_the_dates_thanks"),
      warnings:   0,
      creators:   &[
        "Ann Able ; footnotemark[†]=",
        "Bob Baker ; thanks[*]=Univ B.",
      ],
      moves:      "",
      page:       &[
        "Ann Able ; † plain",
        "Bob Baker ; * plain",
        "margin: Univ B.",
      ],
      page_moves: "",
    },
    Row {
      repro:      "markkey_footnotemark_shares_a_thanks_by_number_unbound_class",
      tex:        repro!("markkey_footnotemark_shares_a_thanks_by_number_unbound_class"),
      warnings:   0,
      creators:   &[
        "Ann Able ; thanks[]=Equal contribution.",
        "Bob Baker ; thanks[]=Equal contribution.",
      ],
      moves:      "",
      page:       &[
        "Ann Able ; [a]→note",
        "Bob Baker ; [a]→note",
        "margin: Equal contribution.",
      ],
      page_moves: "",
    },
  ]);
  assert_eq!(
    copies(repro!(
      "markkey_footnotetext_in_the_block_answers_its_names_by_number"
    )),
    pairs(&[("Bob Baker", "Ann Able"), ("Cat Cole", "Ann Able")])
  );
  assert_eq!(
    copies(repro!(
      "markkey_footnotemark_shares_a_thanks_by_number_unbound_class"
    )),
    pairs(&[("Bob Baker", "Ann Able")])
  );
}

/// A mark glued past a name's comma and labels is measured from the comma's ink, the nearer name by the glue printed
/// (DG 2026-11-16, COMMAMARK): 1111.1979's `Igor Pikovski,$^{1, 2 }$\footnotemark $\, \,$ Michael`, every mark its left
/// name's (DG from the PDF), the body's `\footnotetext[n]` filling Igor's and Kim's marks. Before: no name's, both lost
/// with a Warning. A mark set off from the comma by a space stays undecided (llncs's `Ann Able,\n\inst{1}…\thanks{…}`,
/// `perfect_kernel_batch78f`).
#[test]
fn a_mark_after_a_names_comma_is_the_nearer_names() {
  assert_green(&[Row {
    repro:      "markkey_mark_after_a_names_comma_and_labels_is_its",
    tex:        repro!("markkey_mark_after_a_names_comma_and_labels_is_its"),
    warnings:   0,
    creators:   &[
      "Igor Pikovski ; footnote[*]=Electronic address: igor@x.org | affiliation=Univ A ; affiliation=Univ B",
      "Michael R. Vanner | affiliation=Univ A ; affiliation=Univ B",
      "Markus Aspelmeyer | affiliation=Univ A ; affiliation=Univ B",
      "M. S. Kim ; footnote[†]=Electronic address: kim@x.org | affiliation=Univ C",
      "Caslav Brukner | affiliation=Univ B ; affiliation=Univ D.",
    ],
    moves:      "",
    page:       &[
      "Igor Pikovski | under: Electronic address: igor@x.org",
      "Michael R. Vanner",
      "Markus Aspelmeyer",
      "M. S. Kim | under: Electronic address: kim@x.org",
      "Caslav Brukner",
    ],
    page_moves: "",
  }]);
}

/// IEEEtran's `\maketitle` empties `\thefootnote` and `\footnotemark` (IEEEtran.cls:4756-4758): a `\thanks` prints no
/// mark and names no author, so an unmarked `\thanks` is the paper's note, kept at the block's level — here the title's
/// publication note — not under the name it follows, where Perl and the base hung it and it read as that author's
/// affiliation (DG 2026-11-16; 1510.02728). A divergence from Perl's placement.
#[test]
fn an_ieee_unmarked_thanks_is_the_blocks() {
  let tex = repro!("markkey_ieee_unmarked_thanks_is_the_blocks");
  assert_green(&[Row {
    repro: "markkey_ieee_unmarked_thanks_is_the_blocks",
    tex,
    warnings: 0,
    creators: &[
      "Ann Able | membership=Member, IEEE",
      "Bob Baker | membership=Fellow, IEEE",
    ],
    moves: "",
    page: &["Ann Able", "Bob Baker"],
    page_moves: "",
  }]);
  let (_, xml, ()) = convert_with_setup_then(tex, Some("ar5iv.sty"), || {}, |_| {});
  assert!(
    xml.contains(
      "<pubnote role=\"thanks\">Manuscript received May 1, 2026. This work was supported by the NSF.</pubnote>"
    ),
    "{xml}"
  );
}

/// No dangling mark (DG 2026-11-16, 2006.10886, and its document-wide refinement): a name's mark that links to no note
/// shows only where its glyph prints anywhere else in the document — Ann's ∗, the body footnote's mark too —, and is
/// hidden where it prints once, over its name alone (Bob's †). A printed run counts per piece: Ann's `$^{1,2}$` shows,
/// the body's footnote marks 1 and 2 printing its pieces (1304.5040), Bob's 3 hides. A section's number is no mark
/// (revMARKKEY1 S1: three numbered sections showed Ann's 1 and Bob's 3). The XML keeps every mark-only note.
#[test]
fn an_unlinked_mark_shows_only_where_its_glyph_recurs() {
  assert_green(&[Row {
    repro:      "markkey_unlinked_mark_run_shows_where_a_piece_recurs",
    tex:        repro!("markkey_unlinked_mark_run_shows_where_a_piece_recurs"),
    warnings:   0,
    creators:   &[
      "Ann Able ; footnotemark[1,2]=",
      "Bob Baker ; footnotemark[3]=",
    ],
    moves:      "",
    page:       &["Ann Able ; 1,2 plain", "Bob Baker"],
    page_moves: "",
  }]);
  assert_green(&[Row {
    repro:      "markkey_section_numbers_are_no_name_marks",
    tex:        repro!("markkey_section_numbers_are_no_name_marks"),
    warnings:   0,
    creators:   &[
      "Ann Able ; footnotemark[1]=",
      "Bob Baker ; footnotemark[3]=",
    ],
    moves:      "",
    page:       &["Ann Able", "Bob Baker"],
    page_moves: "",
  }]);
  assert_green(&[Row {
    repro:      "markkey_unlinked_name_mark_shows_only_where_its_glyph_recurs",
    tex:        repro!("markkey_unlinked_name_mark_shows_only_where_its_glyph_recurs"),
    warnings:   0,
    creators:   &[
      "Ann Able ; footnotemark[*]=",
      "Bob Baker ; footnotemark[†]=",
    ],
    moves:      "",
    page:       &["Ann Able ; * plain", "Bob Baker"],
    page_moves: "",
  }]);
}

/// The glyphs printed elsewhere that a name's unlinked mark is shown by (revMARKKEY1 B1; OD #1059, `f:printed-glyph`):
/// a body footnote opening with a formula of the glyph alone, its tags left out (2508.05506's `\blfootnote{$\dagger$
/// …}`); a formula of the glyph alone inside a note, past its opening (2012.13912's acmart `\authornote{… \\ $\dagger$
/// … is the corresponding author}`); and a note with no known mark on the name, whose glyph it may be (1503.07953's
/// `\altaffiltext{*}{Corresponding author}`, its label's glyph not kept); a legend printed bracketed (2302.11378's
/// `$^{(2)}$` legends in `\date`); and a piece of another name's mark run (2110.12052). Each was hidden before. RED: a
/// label on a name holding a contact its block's lines gave it, the label consumed (1302.0490's `E.Mail :$^1$sidd…`;
/// row R MARKGLYPHLOST: the page cannot tell it from a mark printed once beside an author line).
#[test]
fn an_unlinked_mark_shows_where_its_glyph_prints() {
  assert_green(&[
    Row {
      repro:      "markkey_blfootnote_dagger_legend_shows_its_names_mark",
      tex:        repro!("markkey_blfootnote_dagger_legend_shows_its_names_mark"),
      warnings:   0,
      creators:   &["Ann Able", "Bob Baker ; footnotemark[†]="],
      moves:      "",
      page:       &["Ann Able", "Bob Baker ; † plain"],
      page_moves: "",
    },
    Row {
      repro:      "markkey_authornote_mid_text_legend_shows_its_names_mark",
      tex:        repro!("markkey_authornote_mid_text_legend_shows_its_names_mark"),
      warnings:   0,
      creators:   &[
        "Ann Able ; footnotemark[*]= | affiliation=Univ A , Land",
        "Bob Baker ; footnotemark[*]= | affiliation=Univ A , Land",
        "Cat Cole ; footnotemark[†]= ; thanks[*]=Ann Able and Bob Baker contributed equally. † Cat Cole is the \
         corresponding author. | affiliation=Univ A , Land",
      ],
      moves:      "",
      page:       &[
        "Ann Able ; *→note",
        "Bob Baker ; *→note",
        "Cat Cole ; † plain ; *→note",
        "margin: Ann Able and Bob Baker contributed equally. † Cat Cole is the corresponding author.",
      ],
      page_moves: "",
    },
    Row {
      repro:      "markkey_altaffiltext_star_shows_its_names_mark",
      tex:        repro!("markkey_altaffiltext_star_shows_its_names_mark"),
      warnings:   0,
      creators:   &[
        "Ann Able | affiliation=Univ A, Town A",
        "Bob Baker ; footnotemark[*]= ; thanks[]=Corresponding author | affiliation=Univ A, Town A ; \
         email=bob@a.edu",
      ],
      moves:      "",
      page:       &[
        "Ann Able",
        "Bob Baker ; * plain | under: Corresponding author",
      ],
      page_moves: "",
    },
    Row {
      repro:      "markkey_bracketed_legend_shows_its_names_mark",
      tex:        repro!("markkey_bracketed_legend_shows_its_names_mark"),
      warnings:   0,
      creators:   &[
        "Ann Able ; footnotemark[1]=",
        "Bob Baker ; footnotemark[2]=",
      ],
      moves:      "",
      page:       &["Ann Able ; 1 plain", "Bob Baker ; 2 plain"],
      page_moves: "",
    },
    Row {
      repro:      "markkey_mark_in_another_names_run_shows",
      tex:        repro!("markkey_mark_in_another_names_run_shows"),
      warnings:   0,
      creators:   &[
        "Ann Able ; footnotemark[1,3]=",
        "Bob Baker ; footnotemark[1]=",
      ],
      moves:      "",
      page:       &["Ann Able ; 1,3 plain", "Bob Baker ; 1 plain"],
      page_moves: "",
    },
    Row {
      repro:      "markkey_author_line_label_shows_its_names_mark",
      tex:        repro!("markkey_author_line_label_shows_its_names_mark"),
      warnings:   2,
      creators:   &[
        "Ann Able ; footnotemark[1]= | affiliation=Department of Physics Univ A, Town A ; email=ann@a.edu",
        "Bob Baker ; footnotemark[2]= | affiliation=Department of Physics Univ A, Town A ; email=bob@a.edu",
        "Cat Cole ; footnotemark[3]= | affiliation=Department of Physics Univ A, Town A ; email=cat@a.edu",
      ],
      moves:      "NEITHER (the e-mail line's label E.Mail kept in the affiliation, the line written as plain text with 2 Warnings; pre-existing)",
      page:       &[
        "Ann Able ; 1 plain",
        "Bob Baker ; 2 plain",
        "Cat Cole ; 3 plain",
      ],
      page_moves: "row R MARKGLYPHLOST (a legend's glyph the XML drops: the author-line contact's consumed label)",
    },
  ]);
}

/// One `\footnotetext[1]` the kernel's `relocate_footnote` fills into two names' `\footnotemark[1]` (Perl's
/// `relocateFootnote`, cloning the text into each mark) is one note: the second copy's `idref` names the first
/// (1108.5155, 1404.2124's shape). Distinct from the reading's copies: the kernel's clone site.
#[test]
fn a_note_several_names_carry_is_one_note() {
  let tex = repro!("markkey_body_footnotetext_two_names_share_is_one_note");
  assert_green(&[Row {
    repro: "markkey_body_footnotetext_two_names_share_is_one_note",
    tex,
    warnings: 0,
    creators: &[
      "Ann Able ; footnote[*]=Univ A, Town A",
      "Bob Baker ; footnote[*]=Univ A, Town A",
      "Cat Cole ; footnote[†]=Univ B, Town B",
    ],
    moves: "",
    page: &[
      "Ann Able ; *→note",
      "Bob Baker ; *→note",
      "Cat Cole ; † plain",
      "margin: Univ A, Town A ; Univ B, Town B",
    ],
    page_moves: "",
  }]);
  assert_eq!(copies(tex), pairs(&[("Bob Baker", "Ann Able")]));
  // (the copy first in document order holds the id, the paired note later: revMARKKEY1 NIT, Ann's copy named Bob's)
  assert_eq!(
    copies(repro!("markkey_mark_before_its_note_is_the_first_copy")),
    pairs(&[("Bob Baker", "Ann Able")])
  );
}

/// The source files of `dir` (relative to the workspace), recursively, with their text up to a `#[cfg(test)]` and no line
/// comments.
fn code_of(dir: &str) -> Vec<(String, String)> {
  fn walk(path: &std::path::Path, out: &mut Vec<(String, String)>) {
    let mut entries: Vec<_> = std::fs::read_dir(path)
      .expect("a source directory")
      .flatten()
      .map(|e| e.path())
      .collect();
    entries.sort();
    for entry in entries {
      if entry.is_dir() {
        walk(&entry, out);
      } else if entry.extension().is_some_and(|x| x == "rs") {
        let text = std::fs::read_to_string(&entry).expect("a source file");
        let text = text.split("#[cfg(test)]").next().unwrap_or_default();
        let code: Vec<&str> = text
          .lines()
          .filter(|l| !l.trim_start().starts_with("//"))
          .collect();
        out.push((entry.display().to_string(), code.join("\n")));
      }
    }
  }
  let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
    .join("..")
    .join(dir);
  let mut out = Vec::new();
  walk(&root, &mut out);
  out
}

/// The mark components may only shrink (ADVARCH, 2026-11-16; MARKKEY K5 retires the glyph joins): the call sites of
/// `mark_form(` (the glyph key every glyph-equality join reads), the `_mark…`/`_refmark…` attribute names, the number
/// keys' `_fn…` attribute names, the state keys of marks and notes (`lx_…mark…`, `lx_…note…`), and the vectors of the
/// reading's `RCreator` and `Resolver` whose name says `mark`, or `note`, `thanks` or `fn_` (the number keys' stores,
/// MARKKEY round 1: `numbered_notes`, `block_thanks`). A raised count is a new join or a new store: the batch states why
/// and the review reads it; a fallen one lowers its pin.
#[test]
fn the_mark_components_only_shrink() {
  const MARK_FORM_CALLS: usize = 30;
  const MARK_ATTRIBUTES: &[&str] = &[
    "_mark",
    "_markfromcounter",
    "_markintext",
    "_markpiece",
    "_markread",
    "_marks",
    "_markshown",
    "_markunknown",
    "_refmark",
    "_refmarkshown",
  ];
  const FOOTNOTE_NUMBER_ATTRIBUTES: &[&str] = &[
    "_fncounted",
    "_fngiven",
    "_fnmark",
    "_fnpart",
    "_fntext",
    "_fnvalue",
  ];
  const MARK_STATE_KEYS: &[&str] = &[
    "lx_acm_author_notes",
    "lx_acm_title_notes",
    "lx_address_mark",
    "lx_affiliation_mark_shown",
    "lx_affiliation_mark_shown_with_names",
    "lx_fm_footnote_marks",
    "lx_fm_list_note_",
    "lx_fm_list_notes",
    "lx_fm_mark_led_names_counted",
    "lx_frontmatter_entry_mark",
    "lx_frontmatter_requested_marks",
    "lx_late_abstract_mark_pending",
    "lx_late_abstract_marks",
    "lx_store_label_noted_",
    "lx_title_counts_footnotes",
    "lx_title_footnote_given_marks_only",
    "lx_title_footnote_own_marks",
    "lx_title_note_values",
  ];
  const RCREATOR_MARK_VECTORS: &[&str] = &["whole_marks", "notes"];
  const RESOLVER_MARK_VECTORS: &[&str] = &[
    "title_fn_marks",
    "block_markers",
    "note_marks",
    "by_mark",
    "marked_entries",
    "thenote_requests",
    "note_links",
    "note_legends",
    "fn_requests",
    "note_texts",
    "numbered_notes",
    "block_thanks",
  ];
  let sources: Vec<(String, String)> = code_of("latexml_engine/src")
    .into_iter()
    .chain(code_of("latexml_package/src"))
    .collect();
  let calls: usize = sources
    .iter()
    .map(|(_, code)| code.matches("mark_form(").count() - code.matches("fn mark_form(").count())
    .sum();
  assert!(
    calls <= MARK_FORM_CALLS,
    "`mark_form(` call sites rose: {calls} > {MARK_FORM_CALLS} (a new glyph-equality join)"
  );
  let mut attributes: Vec<String> = sources
    .iter()
    .flat_map(|(_, code)| code.split("\"_").skip(1))
    .map(|rest| {
      rest
        .chars()
        .take_while(|c| c.is_ascii_lowercase() || *c == '_')
        .collect::<String>()
    })
    .filter(|name| name.starts_with("mark") || name.starts_with("refmark"))
    .map(|name| format!("_{name}"))
    .collect();
  attributes.sort();
  attributes.dedup();
  let new: Vec<&String> = attributes
    .iter()
    .filter(|a| !MARK_ATTRIBUTES.contains(&a.as_str()))
    .collect();
  assert!(new.is_empty(), "new `_mark…` attributes: {new:?}");
  // (the number keys' attributes, `"_fn…"`, and the state keys of marks and notes, `"lx_…"`)
  let names = |prefix: &str, keep: &dyn Fn(&str) -> bool| -> Vec<String> {
    let quoted = format!("\"{prefix}");
    let mut names: Vec<String> = sources
      .iter()
      .flat_map(|(_, code)| code.split(quoted.as_str()).skip(1))
      .map(|rest| {
        rest
          .chars()
          .take_while(|c| c.is_ascii_lowercase() || *c == '_')
          .collect::<String>()
      })
      .filter(|name| !name.is_empty() && keep(name))
      .map(|name| format!("{prefix}{name}"))
      .collect();
    names.sort();
    names.dedup();
    names
  };
  let new: Vec<String> = names("_fn", &|_| true)
    .into_iter()
    .filter(|a| !FOOTNOTE_NUMBER_ATTRIBUTES.contains(&a.as_str()))
    .collect();
  assert!(new.is_empty(), "new `_fn…` attributes: {new:?}");
  let new: Vec<String> = names("lx_", &|name| {
    name.contains("mark") || name.contains("note")
  })
  .into_iter()
  .filter(|key| !MARK_STATE_KEYS.contains(&key.as_str()))
  .collect();
  assert!(new.is_empty(), "new state keys of marks or notes: {new:?}");
  let resolver = std::fs::read_to_string(
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
      .join("../latexml_engine/src/base_utilities/frontmatter/observer/resolver/mod.rs"),
  )
  .expect("the resolver");
  let mark_vectors = |name: &str| -> Vec<String> {
    let start = resolver
      .find(&format!("struct {name} {{"))
      .expect("the struct");
    let body = &resolver[start..start + resolver[start..].find("\n}").expect("its end")];
    body
      .lines()
      .filter_map(|line| {
        let (field, ty) = line.trim().trim_start_matches("pub ").split_once(':')?;
        let field = field.trim();
        (["mark", "note", "thanks", "fn_"]
          .iter()
          .any(|word| field.contains(word))
          && ty.contains("Vec"))
        .then(|| field.to_string())
      })
      .collect()
  };
  for (name, pinned) in [
    ("RCreator", RCREATOR_MARK_VECTORS),
    ("Resolver", RESOLVER_MARK_VECTORS),
  ] {
    let new: Vec<String> = mark_vectors(name)
      .into_iter()
      .filter(|field| !pinned.contains(&field.as_str()))
      .collect();
    assert!(new.is_empty(), "new mark vectors on `{name}`: {new:?}");
  }
}
