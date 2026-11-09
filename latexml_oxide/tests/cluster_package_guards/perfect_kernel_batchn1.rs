//! Guards for N1 (`docs/parity/FRONTMATTER_SINGLE_ENGINE_PLAN.md`, "The safety net"; OXIDIZED_DESIGN_DIVERGENCES #980): the emitter's checks of the
//! reading, each from digested facts, with the switch on. An e-mail's label in any language by its place on its line
//! (`an_email_label_in_any_language_is_written_as_printed`); the separator check (`a_role_heading_in_a_name_is_counted`) and the marks check
//! (`a_mark_reaching_no_contact_is_counted`), each counted in its Warning. N1r2 (review revN1): a kept note cuts out its written address by
//! position, the note of the author whose line it is (`a_note_cuts_out_its_written_address`); a lost name fails the names, the block written whole
//! (`a_lost_name_writes_the_block_whole`). N1r3 (review revN1r2): no reading outlives its conversion
//! (`no_reading_outlives_its_conversion`); a lost name ranked against the written names spells its own address
//! (`a_lost_name_spells_its_own_address`); a line's other inks go with it to its address's author (`a_note_cuts_out_its_written_address`), a list's
//! local parts to none (`a_list_part_goes_to_no_line_author`).

use latexml::util::test::{error_count, warning_count};
use regex::Regex;

use super::{perfect_kernel_batch84b::creators, perfect_kernel_batchz4::run};

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $name,
      ".tex"
    ))
  };
}

/// The e-mail contacts of `xml`, in order: each `name` and text.
fn emails(xml: &str) -> Vec<(String, String)> {
  let contact =
    Regex::new(r#"<contact\b[^>]*\bname="([^"]*)"[^>]*\brole="email"[^>]*>(.*?)</contact>"#)
      .expect("regex");
  contact
    .captures_iter(xml)
    .map(|c| (c[1].replace('\u{a0}', " "), c[2].to_string()))
    .collect()
}

/// An e-mail's label in any language is what its line prints before the address's run, one word or two and a colon, written as printed: no word
/// lost, no note doubling the address (review revZ4br2's `Correo electr\'onico:`, `Courriel :`); with either switch.
#[test]
fn an_email_label_in_any_language_is_written_as_printed() {
  let template = repro!("emit_an_email_label_in_any_language");
  let printed = "Correo electr\\'onico: ann@a.org";
  for (label, name) in [
    (printed, "Correo electrónico: "),
    ("Courriel : ann@a.org", "Courriel : "),
  ] {
    let tex = template.replace(printed, label);
    let (log, xml) = run(&tex);
    assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
    assert_eq!(
      emails(&xml),
      [(name.to_string(), "ann@a.org".to_string())],
      "{label}"
    );
    assert!(!xml.contains("role=\"note\""), "{xml}");
  }
}

/// The separator check: a name holding a label before a colon and the words after it, a role's heading read into it, is counted, the reading kept
/// (2205.07715's revtex `\author{Editor: …}`, hep-ph0311131's `{\bf Authors:} F.~Arleo$^a$, …`); with either switch.
#[test]
fn a_role_heading_in_a_name_is_counted() {
  let block = repro!("emit_net_a_role_heading_in_a_name");
  let revtex = "\\documentclass{revtex4-1}\n\\begin{document}\n\\title{T}\n\\author{Editor: Ann Able}\n\\affiliation{Univ A}\n\\author{Bob \
                Baker}\n\\affiliation{Univ B}\n\\maketitle\nBody.\n\\end{document}\n";
  for (tex, warnings, heading) in [
    (block, 2, "Authors: | Ann Able"),
    (revtex, 1, "Editor: | Ann Able"),
  ] {
    let (log, xml) = run(tex);
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, warnings),
      "{log}"
    );
    assert!(
      log.contains(&format!(
        "safety net: a separator its author block prints between names parts no creators ({heading}); they are counted"
      )),
      "{log}"
    );
    let first = heading.replace(" |", "");
    assert_eq!(creators(&xml), [
      vec![first, "affiliation=Univ A".to_string()],
      vec!["Bob Baker".to_string(), "affiliation=Univ B".to_string()],
    ]);
  }
}

/// The marks check: a name's mark whose label leads a line in its run of marks (`$^1$ $^2$ Univ A`), which no item of the name's creator carries, is
/// counted, the reading kept (1703.06483: the mark stays printed on Bob Baker's name); with either switch.
#[test]
fn a_mark_reaching_no_contact_is_counted() {
  let tex = repro!("emit_net_a_mark_reaches_no_contact");
  let (log, xml) = run(tex);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(
    log.contains("reaches no contact carrying its label, or a labelled contact no name's mark reaches (2); they are counted"),
    "{log}"
  );
  assert_eq!(creators(&xml), [
    vec!["Ann Able", "affiliation=Univ A, Town", "email=ann@a.org"],
    vec!["Bob Baker 2", "affiliation=Univ A, Town", "email=bob@b.org"],
    vec!["Cat Cole", "affiliation=Univ C, City", "email=cat@c.org"],
  ]);
}

/// Per creator of `xml`, the text of each of its notes (`role="note"`), in order.
pub(super) fn notes(xml: &str) -> Vec<Vec<String>> {
  let creator = Regex::new(r"(?s)<creator\b.*?</creator>").expect("regex");
  let note = Regex::new(r#"(?s)<note role="note"[^>]*>(.*?)</note>"#).expect("regex");
  let tag = Regex::new(r"<[^>]+>").expect("regex");
  creator
    .find_iter(xml)
    .map(|c| {
      note
        .captures_iter(c.as_str())
        .map(|n| tag.replace_all(&n[1], "").to_string())
        .collect()
    })
    .collect()
}

/// A guard row: the block, the creators, and each creator's notes.
type NoteRow = (
  &'static str,
  &'static [&'static [&'static str]],
  &'static [&'static [&'static str]],
);

/// The rows of [`a_note_cuts_out_its_written_address`]: the block, the creators the switch on writes, and each one's notes.
const NOTE_ROWS: &[NoteRow] = &[
  // (2110.03506: a name's word the address repeats, `K. Hobbs (kerianne.hobbs@…)`)
  (
    "Ann L. Able, Bob Baker, and Cat Cole \\\\ POC: A.\\ Able (ann.able@a.org)\\\\ Approved for public release.",
    &[
      &[
        "Ann L. Able",
        "email=ann.able@a.org",
        "affiliation=Approved for public release.",
      ],
      &["Bob Baker", "affiliation=Approved for public release."],
      &["Cat Cole", "affiliation=Approved for public release."],
    ],
    &[&["POC: A. Able"], &[], &[]],
  ),
  // (2203.10777's line under the affiliation it continues, `Institute of Things, Universität Dresden,\\ Dresden, Saxony 01062,
  // Germany (ann.able@…)`, is that affiliation's holders' since LO1, no note: `perfect_kernel_batchlo1::an_address_decides_only_its_own_ink`)
  // (an ink after the address on its line, `ext. 5`: the line's author's, review revN1r2's `rep/both.tex`)
  (
    "Ann L. Able, Bob Baker, and Cat Cole \\\\ POC: A.\\ Able (ann.able@a.org), ext.\\ 5\\\\ Approved for public release.",
    &[
      &[
        "Ann L. Able",
        "email=ann.able@a.org",
        "affiliation=Approved for public release.",
      ],
      &["Bob Baker", "affiliation=Approved for public release."],
      &["Cat Cole", "affiliation=Approved for public release."],
    ],
    &[&["POC: A. Able", "ext. 5"], &[], &[]],
  ),
  // (a label after the address, which no address run takes: the address's author's)
  (
    "Ann Able and Bob Baker \\\\ Univ A \\\\ ann.able@a.org (corresponding)",
    &[
      &["Ann Able", "affiliation=Univ A", "email=ann.able@a.org"],
      &["Bob Baker", "affiliation=Univ A"],
    ],
    &[&["corresponding"], &[]],
  ),
];

/// A kept note of ink the reading holds no place for is its lost words' own text, its written address runs cut out by position, not a range of words
/// out of a multiset (`POC: K. Hobbs (kerianne.hobbs`, 2110.03506; `Germany (martin.waltz@tu-dresden`, 2203.10777: review revN1); the note of the
/// author whose line it is by the reading's rule of an address; in one counted Warning, with either switch, off clean.
#[test]
fn a_note_cuts_out_its_written_address() {
  let template = repro!("emit_net_a_note_cuts_its_written_address");
  for &(block, expected, held) in NOTE_ROWS {
    let tex = template.replace(NOTE_ROWS[0].0, block);
    assert!(tex.contains(block));
    let (log, xml) = run(&tex);
    assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
    assert!(
      log.contains("they are kept as a note of the author block"),
      "{log}"
    );
    assert_eq!(creators(&xml), expected, "{block}");
    assert_eq!(notes(&xml), held, "{block}");
  }
}

/// A lost lettered Name of a block that is no author's line by its addresses fails the names whatever its line holds: the block is written whole, Perl's
/// whole-block creator, not a name kept as a note (JAIR's `\name Shaul Markovitch \email …\\`, 1109.6618; 1312.0286, 1111.6082, 1905.12960,
/// 1807.03929; a column of its own, 1805.08288, 2002.00084: review revN1); in one counted Warning, with either switch, off clean.
#[test]
fn a_lost_name_writes_the_block_whole() {
  let tex = repro!("emit_net_a_lost_name_writes_the_block_whole");
  let block = "Ann Able ann@a.org\\\\ Bob Baker bob@b.org\\\\ Univ A\\\\ Town, Country";
  let columns = "\\begin{minipage}{.4\\linewidth}\\centering Ann Able\\\\ ann.able@a.org\\end{minipage}\\begin{minipage}{.4\\linewidth}\\centering Bob \
                 Baker\\\\ bob.baker@b.org\\end{minipage}\\\\ Univ A";
  for (block, whole) in [
    (
      block,
      "Ann Able ann@a.org Bob Baker bob@b.org Univ A Town, Country",
    ),
    (
      columns,
      "Ann Able ann.able@a.org Bob Baker bob.baker@b.org Univ A",
    ),
  ] {
    let tex = tex.replace(
      "Ann Able ann@a.org\\\\ Bob Baker bob@b.org\\\\ Univ A\\\\ Town, Country",
      block,
    );
    assert!(tex.contains(block));
    let (log, xml) = run(&tex);
    assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
    assert!(
      log.contains("the block is written whole, as Perl's whole-block creator"),
      "{log}"
    );
    assert_eq!(creators(&xml), [vec![whole.to_string()]], "{block}");
    assert_eq!(notes(&xml), [Vec::<String>::new()], "{block}");
  }
}

/// `docs` converted in turn on one thread, as a persistent `cortex_worker` converts papers (no engine reset between them), with
/// the switch on and the frontmatter trace on (`--debug=frontmatter`): each one's frontmatter trace (its `FRONT` lines) and XML.
fn in_turn(docs: Vec<&'static str>) -> Vec<(Vec<String>, String)> {
  use std::rc::Rc;

  use latexml::converter::Converter;
  use latexml_core::common::{Config, OutputFormat};
  std::thread::Builder::new()
    .stack_size(256 * 1024 * 1024)
    .spawn(move || {
      let _ = latexml_core::util::logger::init(log::LevelFilter::Debug);
      latexml_core::common::error::enable_debug_feature("frontmatter");
      let runs = docs
        .into_iter()
        .map(|tex| {
          let opts = Config {
            format: OutputFormat::XML,
            include_comments: Some(false),
            preload: Some(vec!["ar5iv.sty".to_string()]),
            bindings_dispatch: Some(Rc::new(latexml_package::dispatch)),
            extra_bindings_dispatch: Some(Rc::new(latexml_contrib::dispatch)),
            ..Config::default()
          };
          let mut converter = Converter::from_config(opts.clone());
          converter.prepare_session(&opts).expect("a session");
          let response = converter.convert_content_with_provenance("t.tex", tex.to_string());
          let trace: Vec<String> = response
            .log
            .lines()
            .filter(|line| line.contains("FRONT"))
            .map(str::to_string)
            .collect();
          (trace, response.result.unwrap_or_default())
        })
        .collect();
      latexml_core::reset_thread_engine();
      runs
    })
    .expect("a conversion thread")
    .join()
    .unwrap_or_else(|panic| std::panic::resume_unwind(panic))
}

/// No reading of the emitter outlives its conversion: documents converted in turn on one thread, as a persistent
/// `cortex_worker` converts papers (`a.tex` twice, then `b.tex`, then `a.tex`), each write the frontmatter, and trace it, as
/// their solo conversions do. The reading is the ledger's (`Ledger::analysis`), dropped with it at a conversion's start
/// (review revN1r2: a thread-local had kept the last paper's reading, its tokens and fonts, into the next). No conversion
/// follows an engine reset on its thread (`latexml_core::reset_thread_engine`'s contract: its token constants are not
/// re-pinned), so none is made here.
#[test]
fn no_reading_outlives_its_conversion() {
  let a = repro!("emit_no_reading_outlives_its_conversion");
  let b: &'static str = Box::leak(
    a.replace("Paper A", "Paper B")
      .replace("\\section{Two} Text.\n", "")
      .into_boxed_str(),
  );
  assert_ne!(a, b);
  let solo_a = in_turn(vec![a]).remove(0);
  let solo_b = in_turn(vec![b]).remove(0);
  assert!(
    solo_a.0.iter().any(|line| line.contains("FRONT")) && solo_a.1.contains("Ann Able"),
    "{solo_a:?}"
  );
  let runs = in_turn(vec![a, a, b, a]);
  for ((trace, xml), solo) in runs.iter().zip([&solo_a, &solo_a, &solo_b, &solo_a]) {
    assert_eq!(trace, &solo.0);
    assert_eq!(xml, &solo.1);
  }
}

/// Whether a lost name fails the names ranks it against the written names in one owner decision: an address the lost name
/// spells best is its own (`philips@` Andy Philips's, though its prefix spells the written Philip Laird), so the block is
/// written whole, Perl's whole-block creator, as it is where no written name spells the address (`Andy Smith`, `smith@`):
/// JAIR's `\name Andy Philips \email philips@…`, 1908.02923 (review revN1r2), a raw style's rows (its `\@maketitle`
/// fixture); in one counted Warning, with either switch, off clean.
#[test]
fn a_lost_name_spells_its_own_address() {
  let template = repro!("emit_net_a_lost_name_spells_its_own_address");
  let style =
    include_str!("../../../tools/perfect_kernel/repros/sectioning-frontmatter/emit_jair_rows.sty");
  let row = "\\name Andy Philips \\email philips@ptolemy.arc.nasa.gov";
  for (name, address) in [
    ("Andy Philips", "philips@ptolemy.arc.nasa.gov"),
    ("Andy Smith", "smith@ptolemy.arc.nasa.gov"),
  ] {
    let tex = template.replace(row, &format!("\\name {name} \\email {address}"));
    assert!(tex.contains(name));
    let convert = || {
      let (log, xml, ()) = latexml::util::test::convert_files_with_setup_then(
        &tex,
        &[("emit_jair_rows.sty", style)],
        Some("ar5iv.sty"),
        move || {},
        |_| (),
      );
      (log, xml)
    };
    let (log, xml) = convert();
    assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
    assert!(
      log.contains(&format!(
        "prints ({name}); the block is written whole, as Perl's whole-block creator"
      )),
      "{log}"
    );
    let creators = creators(&xml);
    assert_eq!(creators.len(), 1, "{name}: {creators:?}");
    let whole = &creators[0][0];
    for printed in [
      "Steven Minton minton@ptolemy.arc.nasa.gov",
      &format!("{name} {address}"),
      "Philip Laird laird@ptolemy.arc.nasa.gov",
    ] {
      assert!(whole.contains(printed), "{printed}: {whole}");
    }
    assert_eq!(notes(&xml), [Vec::<String>::new()], "{name}");
  }
}

/// A list's local parts printed apart, sharing one domain and opening their line in a font group, are one run of addresses the address
/// reader reads (`address_runs`, in `Resolver::place_line_contacts` with the emitter's switch on), each part an author's by spelling
/// (`{\it pzou94, ozel, suresh@gwu.edu}` under three names, 2003.14069; 2102.08981, 1901.01753, 1708.00818, 2307.03645), each
/// named creator's element as it is. (Until LO1 the reading lost the parts the base writes as addresses, `aable@a.org`, kept as the
/// block's note in a Warning; repro `emit_a_font_group_of_local_parts_is_a_list`.) `=2` alone since LO2: the reading's
/// switch-on rules run with `LATEXML_FM_EMIT=2`, as its whole entries (`Resolver::emitting`, review revLO1 4); `=1` keeps the
/// switch-off reading, the parts its Warning.
#[test]
fn a_list_part_goes_to_no_line_author() {
  let template = repro!("emit_net_a_note_cuts_its_written_address");
  let tex = template.replace(
    NOTE_ROWS[0].0,
    "Ann Able \\qquad Bob Baker \\qquad Cat Cole \\\\ Univ A \\\\ {\\it aable, bbaker, cat@a.org}",
  );
  let (log, xml) = run(&tex);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(creators(&xml), [
    vec!["Ann Able", "affiliation=Univ A", "email=aable@a.org"],
    vec!["Bob Baker", "affiliation=Univ A", "email=bbaker@a.org"],
    vec!["Cat Cole", "affiliation=Univ A", "email=cat@a.org"],
  ]);
  assert_eq!(
    notes(&xml),
    [Vec::<String>::new(), Vec::new(), Vec::new()],
    "{xml}"
  );
}
