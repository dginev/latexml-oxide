//! MARKKEY round 1 (brief 2026-11-16_brief_MARKKEY; DG's rulings of 2026-11-16): frontmatter marks keyed by the footnote
//! number TeX pairs them by, note identity across per-carrier copies, and the page's mark rules. Code paths, one test each:
//! a block's `\footnotetext[n]` given to the names whose `\footnotemark[n]` shows n (`resolver::answer_note_texts`,
//! `Event::number`) and a name's `\footnotemark[n]` filled with the block's note of number n (`insert::link_marked_notes`,
//! `_fnvalue`) — OD #1058; a mark glued past a name's comma measured from the comma (`resolver::mark_ledger`, COMMAMARK);
//! IEEEtran's unmarked `\thanks` kept at the block's level (`resolver::keep_note`); one note's copies marked as one
//! (`relocate::one_note` at `relocate_annotations`, the kernel's `relocate_footnote` and the Link step), and on the page
//! a shared unmarked note under a generated `[a]`, an unlinked name mark shown only where its glyph recurs
//! (`LaTeXML-meta-xhtml.xsl`) — OD #1059. Rows assert the pdflatex oracle in the FMRG suite's two layers
//! (`perfect_kernel_batchfmmarks::Row`). Witnesses 2407.04435, 1111.1979, 2006.10886, 1510.02728, 1108.5155.
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
/// Bob kept a mark-only "1", no copy). Distinct paths: the reading's footnote-text pairing and the Link step's
/// number pairing.
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
        "Cat Cole ; *→note | under: Univ B, Town B",
        "Dan Dorn | under: Univ C, Town C",
        "margin: Univ A, Town A",
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

/// IEEEtran prints no `\thanks` mark and hangs every `\thanks` on the last author (IEEEtran.cls:4756-4758), so an unmarked
/// `\thanks` is the paper's note, kept at the block's level — the title's publication note — not under the last name,
/// where it read as that author's affiliation (DG 2026-11-16; 1510.02728). A divergence from Perl's placement.
#[test]
fn an_ieee_unmarked_thanks_is_the_blocks() {
  let tex = repro!("markkey_ieee_unmarked_thanks_is_the_blocks");
  assert_green(&[Row {
    repro:      "markkey_ieee_unmarked_thanks_is_the_blocks",
    tex,
    warnings:   0,
    creators:   &[
      "Ann Able | membership=Member, IEEE",
      "Bob Baker | membership=Fellow, IEEE",
    ],
    moves:      "",
    page:       &["Ann Able", "Bob Baker"],
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
/// hidden where it prints once, over its name alone (Bob's †). The XML keeps both mark-only notes.
#[test]
fn an_unlinked_mark_shows_only_where_its_glyph_recurs() {
  assert_green(&[Row {
    repro:      "markkey_unlinked_name_mark_shows_only_where_its_glyph_recurs",
    tex:        repro!("markkey_unlinked_name_mark_shows_only_where_its_glyph_recurs"),
    warnings:   0,
    creators:   &["Ann Able ; footnotemark[*]=", "Bob Baker ; footnotemark[†]="],
    moves:      "",
    page:       &["Ann Able ; * plain", "Bob Baker"],
    page_moves: "",
  }]);
}

/// One `\footnotetext[1]` the kernel's `relocate_footnote` fills into two names' `\footnotemark[1]` (Perl's
/// `relocateFootnote`, cloning the text into each mark) is one note: the second copy's `idref` names the first
/// (1108.5155, 1404.2124's shape). Distinct from the reading's copies: the kernel's clone site.
#[test]
fn a_note_several_names_carry_is_one_note() {
  let tex = repro!("markkey_body_footnotetext_two_names_share_is_one_note");
  assert_green(&[Row {
    repro:      "markkey_body_footnotetext_two_names_share_is_one_note",
    tex,
    warnings:   0,
    creators:   &[
      "Ann Able ; footnote[*]=Univ A, Town A",
      "Bob Baker ; footnote[*]=Univ A, Town A",
      "Cat Cole ; footnote[†]=Univ B, Town B",
    ],
    moves:      "",
    page:       &[
      "Ann Able ; *→note",
      "Bob Baker ; *→note",
      "Cat Cole | under: Univ B, Town B",
      "margin: Univ A, Town A",
    ],
    page_moves: "",
  }]);
  assert_eq!(copies(tex), pairs(&[("Bob Baker", "Ann Able")]));
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
  let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join(dir);
  let mut out = Vec::new();
  walk(&root, &mut out);
  out
}

/// The mark components may only shrink (ADVARCH, 2026-11-16; MARKKEY K5 retires the glyph joins): the call sites of
/// `mark_form(` (the glyph key every glyph-equality join reads), the `_mark…`/`_refmark…` attribute names, and the mark
/// vectors of the reading's `RCreator` and `Resolver` (fields whose name says `mark`). A raised count is a new join or a
/// new mark store: the batch states why and the review reads it; a fallen one lowers its pin.
#[test]
fn the_mark_components_only_shrink() {
  const MARK_FORM_CALLS: usize = 29;
  const MARK_ATTRIBUTES: &[&str] = &[
    "_mark",
    "_markfromcounter",
    "_markintext",
    "_markpiece",
    "_markread",
    "_marks",
    "_markshown",
    "_refmark",
    "_refmarkshown",
  ];
  const RCREATOR_MARK_VECTORS: &[&str] = &["whole_marks"];
  const RESOLVER_MARK_VECTORS: &[&str] = &[
    "title_fn_marks",
    "block_markers",
    "note_marks",
    "by_mark",
    "marked_entries",
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
        (field.contains("mark") && ty.contains("Vec")).then(|| field.trim().to_string())
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
