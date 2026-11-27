//! Guards for OMNT: the fnsymbol title block (`declare_fnsymbol_title_block`, OXIDIZED_DESIGN_DIVERGENCES #851) of the
//! binding that serves the document's class (`serves_document_class`): the class's own name, the request a versioned
//! fallback answered with it (siamltex1213, siamonline220329), or the base a raw class file loads and leaves the title
//! block alone (scrarticle.cls; `settle_title_block_after_class_load`, CR7's FNR1 table holds that row). One row per
//! binding path: KOMA without and with its title page, the two versioned SIAM fallbacks, dmtcs over OmniBus; OmniBus
//! itself and a raw class whose own `\maketitle` prints no marks are the negative controls.

use latexml::util::test::{convert_files_with_setup_then, error_count, warning_count};

use super::perfect_kernel_batchfmmarks::creator_lines;

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $name,
      ".tex"
    ))
  };
}

/// A repro's leading `filecontents*` class, as the conversion's own file, and the document after it.
fn shipped_class(tex: &'static str) -> (Vec<(String, &'static str)>, &'static str) {
  let close = "\\end{filecontents*}\n";
  match (
    tex.find("\\begin{filecontents*}[overwrite]{"),
    tex.find(close),
  ) {
    (Some(open), Some(end)) => {
      let head = &tex[open..];
      let name_start = head.find("]{").unwrap() + 2;
      let name_end = name_start + head[name_start..].find('}').unwrap();
      let body = &head[name_end + 2..end - open];
      (
        vec![(head[name_start..name_end].to_string(), body)],
        &tex[end + close.len()..],
      )
    },
    _ => (Vec::new(), tex),
  }
}

/// The mark of each body footnote of `xml` (a `role="footnote"` note outside the creators), in order.
fn body_footnote_marks(xml: &str) -> Vec<String> {
  let body = xml
    .rsplit_once("</creator>")
    .map_or(xml, |(_, after)| after);
  body
    .match_indices("<note ")
    .filter_map(|(at, _)| {
      let tag = &body[at..at + body[at..].find('>').unwrap()];
      tag.contains(r#"role="footnote""#).then(|| {
        tag
          .split(r#"mark=""#)
          .nth(1)
          .and_then(|value| value.split('"').next())
          .unwrap_or_default()
          .to_string()
      })
    })
    .collect()
}

/// A row: the repro, its Warnings, its creators ([`creator_lines`]), the body's footnote marks.
type Row = (
  &'static str,
  &'static str,
  usize,
  &'static [&'static str],
  &'static [&'static str],
);

/// latex.ltx:17215 `\thanks`; scrartcl.cls:2859-2862, scrreprt.cls:3080-3083, siamltex1213.cls:151-169,
/// siamonline220329.cls:508-528, dmtcs-episciences.cls:304-316 mark the title's notes `\@fnsymbol` and reset the counter
/// after `\maketitle`; pdflatex for each row in its repro header.
#[test]
fn a_binding_serving_the_document_class_marks_its_title_in_fnsymbol() {
  let rows: [Row; 7] = [
    (
      "koma_title_footnotemark_prints_fnsymbol",
      repro!("koma_title_footnotemark_prints_fnsymbol"),
      0,
      &[
        "Ann One ; thanks[*]=The University",
        "Bob Two ; thanks[*]=The University",
      ],
      &["1"],
    ),
    (
      "koma_titlepage_marks_in_fnsymbol",
      repro!("koma_titlepage_marks_in_fnsymbol"),
      0,
      &[
        "Ann One ; thanks[*]=The University",
        "Bob Two ; thanks[*]=The University",
      ],
      &["1"],
    ),
    (
      "siamltex_versioned_title_marks_in_fnsymbol",
      repro!("siamltex_versioned_title_marks_in_fnsymbol"),
      0,
      &[
        "Ann One ; footnote[‡]=Univ A.",
        "Bob Two ; footnote[‡]=Univ A.",
      ],
      &["1"],
    ),
    (
      "siamonline_versioned_title_marks_in_fnsymbol",
      repro!("siamonline_versioned_title_marks_in_fnsymbol"),
      0,
      &["Ann One ; thanks[*]=Univ A.", "Bob Two ; thanks[*]=Univ A."],
      &["1"],
    ),
    (
      "dmtcs_title_marks_in_fnsymbol",
      repro!("dmtcs_title_marks_in_fnsymbol"),
      0,
      &["Ann One ; thanks[*]=Univ A.", "Bob Two ; thanks[*]=Univ A."],
      &[],
    ),
    // (the negative controls: OmniBus fronting birkjour, a raw class's own unmarked `\maketitle`)
    (
      "omnibus_unknown_class_title_marks_unmarked",
      repro!("omnibus_unknown_class_title_marks_unmarked"),
      1,
      &["Ann One ; thanks[]=Univ A.", "Bob Two ; thanks[]=Univ A."],
      &["1"],
    ),
    (
      "raw_class_title_block_redefined_keeps_its_marks",
      repro!("raw_class_title_block_redefined_keeps_its_marks"),
      0,
      &[
        "Ann One ; thanks[]=The University",
        "Bob Two ; thanks[]=The University",
      ],
      &["1"],
    ),
  ];
  let mut wrong = Vec::new();
  for (name, tex, warnings, creators, body) in rows {
    let (files, doc) = shipped_class(tex);
    let files: Vec<(&str, &str)> = files
      .iter()
      .map(|(file, body)| (file.as_str(), *body))
      .collect();
    let (log, xml, ()) =
      convert_files_with_setup_then(doc, &files, Some("ar5iv.sty"), || {}, |_| ());
    let found = (
      error_count(&log),
      warning_count(&log),
      creator_lines(&xml),
      body_footnote_marks(&xml),
    );
    let oracle = (
      0,
      warnings,
      creators.iter().map(ToString::to_string).collect::<Vec<_>>(),
      body.iter().map(ToString::to_string).collect::<Vec<_>>(),
    );
    if found != oracle {
      wrong.push(format!("{name}\n  oracle: {oracle:?}\n  found:  {found:?}"));
    }
  }
  assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}
