//! Red/green guards for batch 86b: title pages whose words were lost because two templates had no binding. pos.sty (the
//! PoS template, v2.1/v2.2, under `article`) keeps its title page in stores only its own `\maketitle` prints, which the
//! kernel refuses, as it refuses its `\RenewDocumentCommand\author{som}`; rmaa.cls appends its `\altaffiltext`
//! affiliations to `\@thanks`, which the kernel's `\maketitle` empties, and stores its Spanish abstract and keywords
//! for its own `\maketitle` too. The bindings `pos_sty.rs` and `rmaa_cls.rs` make each the frontmatter
//! (OXIDIZED_DESIGN_DIVERGENCES #904, KNOWN_PERL_ERRORS #1198). Witnesses 2409.20413, 2512.14592, 2602.22379,
//! 2107.12913 (pos.sty), 0903.4819, astro-ph0208329, 1301.7157 (rmaa). Each guard converts its repro under
//! `tools/perfect_kernel/repros/sectioning-frontmatter/` (a shortened copy of pos.sty, a minimal stand-in of rmaa.cls,
//! written by `{filecontents*}`) and asserts no Error or Warning, every creator's name and contacts in order, and the
//! other title-page elements (abstracts, keywords, notes, the toc title): start tag and text. One guard per distinct
//! path: pos.sty's sole speaker (its note from `\speakerText`, the unlabelled contacts, `\onbehalf*`, `\notes`,
//! `\dedicated`), its marked list (labels, a copy's own speaker text, an `\emailAdd` inside an `\author`, what no name
//! owns on the block; and a speaker after a labelled author with one more note, whose "Speaker" note is labelled so
//! the kernel's one-note-per-labelled-author rule cannot hand it to the first), rmaa's marks (a mark list, a label on
//! several authors, `\addkeyword`, `\shorttitle`), and rmaa's unmarked `\altaffiltext` (with `\affil` and
//! `\keywords`).
use latexml::util::test::convert_with;

use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch84b::{creators, text},
};

const PRELOAD: &str = "ar5iv.sty";

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $path
    ))
  };
}

/// The `<abstract>`, `<keywords>`, `<note>`, `<pubnote>` and `<toctitle>` elements of the title page (before the
/// first `<para>` or `<section>`), in document order: the start tag without its `xml:id`, and the text (markup aside,
/// whitespace collapsed).
fn title_page_elements(xml: &str) -> Vec<(String, String)> {
  let end = ["<para ", "<section "]
    .iter()
    .filter_map(|tag| xml.find(tag))
    .min()
    .unwrap_or(xml.len());
  let page = &xml[..end];
  let mut found = Vec::new();
  for name in ["abstract", "keywords", "note", "pubnote", "toctitle"] {
    let open = format!("<{name}");
    let close = format!("</{name}>");
    for (start, _) in page
      .match_indices(&open)
      .filter(|(start, _)| matches!(page.as_bytes().get(start + open.len()), Some(b' ' | b'>')))
    {
      let tag_end = start + page[start..].find('>').expect("start tag closes");
      let tag = page[start..=tag_end].to_string();
      let tag = match tag.find(" xml:id=\"") {
        Some(at) => {
          let value_end =
            at + " xml:id=\"".len() + tag[at + " xml:id=\"".len()..].find('"').expect("id closes");
          format!("{}{}", &tag[..at], &tag[value_end + 1..])
        },
        None => tag,
      };
      let body_end = tag_end + page[tag_end..].find(&close).expect("element closes");
      found.push((start, tag, text(&page[tag_end + 1..body_end])));
    }
  }
  found.sort_by_key(|(start, ..)| *start);
  found
    .into_iter()
    .map(|(_, tag, body)| (tag, body))
    .collect()
}

/// Converts the repro, checks it converts clean, and compares its creators (name, then each contact as `role=text`) and
/// its other title-page elements.
#[track_caller]
fn assert_title_page(tex: &str, expected_creators: &[&[&str]], expected_elements: &[(&str, &str)]) {
  let (log, xml) = convert_with(tex, Some(PRELOAD));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let want: Vec<Vec<String>> = expected_creators
    .iter()
    .map(|c| c.iter().map(|s| (*s).to_string()).collect())
    .collect();
  assert_eq!(creators(&xml), want, "{xml}");
  let found = title_page_elements(&xml);
  let found: Vec<(&str, &str)> = found
    .iter()
    .map(|(tag, body)| (tag.as_str(), body.as_str()))
    .collect();
  assert_eq!(found, expected_elements, "{xml}");
}

/// pos.sty's sole speaker (2409.20413): `\author*{Javier M. Duarte}` is the author, the star its "Speaker" note
/// (`\speakerText`, the copy's footnote under the mark, labelled, so after the unlabelled contacts given beside it);
/// the `\onbehalf*` line, the unlabelled affiliations and the e-mail are the author's; `\notes` and `\dedicated` notes
/// of the paper; `\FullConference` a conference note. Red (int116a): `<personname>*</personname>` and the name as its
/// affiliation, the rest lost.
#[test]
fn pos_sty_speaker_author_contacts() {
  assert_title_page(
    repro!("pos_sty_speaker_author_contacts.tex"),
    &[&[
      "Javier M. Duarte",
      "note=on behalf of the ALICE Collaboration",
      "affiliation=Department of Physics",
      "affiliation=University of California San Diego",
      "note=Speaker",
      "email=jduarte@ucsd.edu",
    ]],
    &[
      (
        "<abstract inlist=\"toc\" name=\"Abstract\">",
        "Machine learning is growing.",
      ),
      (
        "<pubnote role=\"note\">",
        "Talk given on behalf of the authors.",
      ),
      (
        "<pubnote role=\"dedication\">",
        "To the memory of a colleague.",
      ),
      ("<note role=\"conference\">", "ICHEP2024"),
    ],
  );
}

/// pos.sty's marked list (2602.22379, 2512.14592): `\author*[a,b]`, `\author[a]`, `\author[c]` each have the
/// affiliations their labels name; the speaker's note is the text the paper's copy prints (a v2.1 copy's "Presenter",
/// 2107.12913); an `\emailAdd` inside an `\author` is that author's; the `\onbehalf` line is the author's it follows;
/// the affiliation no author marks and the address no name spells are the author block's. Red (int116a): one creator
/// "*", the other names paragraphs, every contact lost. The variant: the speaker second, after a labelled author, and
/// one more note (`\onbehalf`): two unlabelled notes for two labelled authors were dealt out one each, in order, so Ann
/// Lee had the "Speaker" note and Bob Kim, the speaker, the on-behalf line (red on 86b r1); the speaker's note is
/// labelled `lx@pos@speaker`, which only a starred author requests.
#[test]
fn pos_sty_marked_authors() {
  assert_title_page(
    repro!("pos_sty_marked_authors.tex"),
    &[
      &[
        "Christian Zimmermann",
        "note=Presenter",
        "affiliation=University of Kentucky",
        "affiliation=Berkeley Lab",
      ],
      &["Terrence Draper", "affiliation=University of Kentucky"],
      &[
        "Jian Liang",
        "email=liang@scnu.edu.cn",
        "note=for the Lattice Collaboration d",
        "affiliation=South China Normal University",
      ],
      &[
        "affiliation=The collaboration office",
        "email=chrzim@lbl.gov",
      ],
    ],
    &[
      (
        "<abstract inlist=\"toc\" name=\"Abstract\">",
        "The hadronic tensor.",
      ),
      ("<note role=\"conference\">", "LATTICE2025"),
    ],
  );
  assert_title_page(
    repro!("pos_sty_marked_speaker_second.tex"),
    &[&["Ann Lee", "affiliation=Inst A"], &[
      "Bob Kim",
      "note=for the XYZ Collaboration",
      "note=Speaker",
      "affiliation=Inst B",
    ]],
    &[
      ("<abstract inlist=\"toc\" name=\"Abstract\">", "Abs."),
      ("<note role=\"conference\">", "Conf X"),
    ],
  );
}

/// rmaa's marks (0903.4819, astro-ph0208329): `\altaffilmark{1}`, `\altaffilmark{1,2}` and `\altaffilmark{2}` give each
/// author the `\altaffiltext` affiliations its marks list, one label on two authors; the Spanish abstract before the
/// English, as the title page prints them, whatever the order given; the `\addkeyword`s one keyword list; the running
/// head's `\shorttitle` the toc title, as OmniBus's (Perl keeps it, 0903.4819). Red (int116a): the affiliations, the
/// Spanish abstract, the keywords and the short title lost, the marks shown on the names.
#[test]
fn rmaa_altaffiltext_marks() {
  assert_title_page(
    repro!("rmaa_altaffiltext_marks.tex"),
    &[
      &[
        "R.E. Mennickent",
        "altaffiliation=Universidad de Concepción, Chile.",
      ],
      &[
        "Z. Kołaczkowski",
        "altaffiliation=Universidad de Concepción, Chile.",
        "altaffiliation=Uniwersytet Wrocławski, Poland.",
      ],
      &["L. Silva", "altaffiliation=Uniwersytet Wrocławski, Poland."],
    ],
    &[
      ("<toctitle>", "Double Periodic Variables"),
      (
        "<abstract inlist=\"toc\" name=\"Resumen\">",
        "En este artículo.",
      ),
      (
        "<abstract inlist=\"toc\" name=\"Abstract\">",
        "In this paper.",
      ),
      (
        "<keywords name=\"Key Words:\">",
        "Binaries: Eclipsing — Stars: Early-type",
      ),
    ],
  );
}

/// rmaa's unmarked `\altaffiltext{1}` (1301.7157): no author requests it, and the class prints it as footnote 1 all the
/// same, so it is the author block's; an `\affil` inside the `\author` the affiliation of the authors before it
/// (astro-ph0105072); `\keywords` the whole list. Red (int116a): the `\altaffiltext`, the Spanish abstract and the
/// keywords lost, each `\affil` in the name before it.
#[test]
fn rmaa_altaffiltext_unmarked() {
  assert_title_page(
    repro!("rmaa_altaffiltext_unmarked.tex"),
    &[
      &["H. Aceves", "affiliation=Universidad de Sonora, México."],
      &["F.J. Tamayo", "affiliation=Universidad de Sonora, México."],
      &["A. Third", "affiliation=Third place."],
      &["altaffiliation=Instituto de Astronomía, UNAM. Ensenada, México."],
    ],
    &[
      ("<abstract inlist=\"toc\" name=\"Resumen\">", "Se estudia."),
      ("<abstract inlist=\"toc\" name=\"Abstract\">", "We study."),
      (
        "<keywords name=\"Key Words:\">",
        "methods: numerical – dark matter",
      ),
    ],
  );
}
