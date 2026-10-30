//! Guards for batch 92a (frontmatter refactor stage 7 groundwork, retiring the old reader for llncs and aa,
//! `docs/parity/FRONTMATTER_DIGESTED_REFACTOR.md`; OXIDIZED_DESIGN_DIVERGENCES #837, #913): three observer readings the
//! move of llncs and aa onto the observer needs, each read from what TeX digested. A mark whose script holds `\ref`s
//! printing nothing at digestion is one Mark carrying every key, each reference an item of its own whatever prints
//! between them (`\inst{\ref{a},\ref{b}}`, `\inst{\ref{a}\ref{b}}`, `observer::events::script_mark`; the ref list
//! split by `resolver::mark_labels`), in a written family too; the e-mail calls a publication note makes (`ltx:pubnote`,
//! A&A's `\offprints`) reach the ledger when a binding's meaning makes them, placed by where the binding prints them —
//! in the note (the class's `\email`, `Policy::email_in_place`: llncs, aa, svjour) by FIDELITY_CHARTER §4's chain
//! exactly, the note's name, when one live creator's whole name, the corresponding author's designation; routed
//! (amsart, REVTeX) the current author's whole own call —, a link in the note or a call of the paper's own `\email`
//! staying its text (`ledger::email_meaning`, `Resolver::answer_document_notes`); a frontmatter clear removes the API calls and the author blocks queued before
//! it (`ledger::record_clear`, llncs's `\author` given twice). Each guard converts its repros under the shadow switch
//! in process and with the binary, eager and `--streaming`, and asserts every creator of the XML whole (today's output,
//! which the reading agrees with, or the written family's output; a sibling's reading only where today's XML is
//! the old reader's), the shadow reading's tuples whole, and no Error or Warning; the binary's two modes must give the
//! same XML creators and the same reading. Every repro compiles clean with pdflatex (TL 2025; the aa repros with
//! 1707.06238's aa.cls, which TeX Live lacks).
use std::{path::Path, process::Command};

use latexml::util::test::{convert_with_setup_then, error_count, warning_count};
use latexml_engine::base_utilities::{fm_shadow_json, set_fm_shadow_override};
use serde_json::Value;

use super::{
  perfect_kernel_batch84b::{creators, text},
  perfect_kernel_batchfms2a2::tuples,
};

const PRELOAD: &str = "ar5iv.sty";

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/class-bindings/",
      $path
    ))
  };
}

/// The XML's creators and the shadow reading's tuples of one conversion, which must be clean, then its publication
/// notes' text (`ltx:pubnote`, in order) and the writer's outcome (`"writer"`, null for a family not written).
type Reading = (Vec<Vec<String>>, Vec<String>, Vec<String>, Value);

#[track_caller]
fn reading_of(log: &str, xml: &str, json: &str) -> Reading {
  assert_eq!((error_count(log), warning_count(log)), (0, 0), "{log}");
  let json: Value = serde_json::from_str(json).unwrap_or_else(|e| panic!("{e}: {json}"));
  let notes = xml
    .match_indices("<pubnote")
    .map(|(at, _)| {
      let end = xml[at..].find("</pubnote>").map_or(xml.len(), |e| at + e);
      // (the note's content, its own tag aside)
      let start = xml[at..end].find('>').map_or(end, |g| at + g + 1);
      text(&xml[start..end])
    })
    .collect();
  (
    creators(xml),
    tuples(&json["resolved"]),
    notes,
    json["writer"].clone(),
  )
}

/// `tex` converted in process under the shadow switch.
#[track_caller]
fn in_process(tex: &str) -> Reading {
  let (log, xml, json) = convert_with_setup_then(
    tex,
    Some(PRELOAD),
    || set_fm_shadow_override(Some(true)),
    |_| fm_shadow_json(),
  );
  reading_of(&log, &xml, &json)
}

/// `tex` converted by the binary under the census switch (`LATEXML_FM_SHADOW=1`, `<dest>.fmshadow.json`),
/// `--streaming=true` when `streaming`.
#[track_caller]
fn with_binary(tex: &str, streaming: bool) -> Reading {
  let bin = env!("CARGO_BIN_EXE_latexml_oxide");
  assert!(Path::new(bin).is_file(), "binary not staged at {bin}");
  let workdir = tempfile::tempdir().expect("create tempdir");
  std::fs::write(workdir.path().join("t.tex"), tex).expect("write t.tex");
  let output = Command::new(bin)
    .args([
      "t.tex",
      "--dest",
      "t.xml",
      "--nocomments",
      "--timeout=110",
      "--preload=ar5iv.sty",
      if streaming {
        "--streaming=true"
      } else {
        "--streaming=false"
      },
    ])
    .env("LATEXML_FM_SHADOW", "1")
    .current_dir(workdir.path())
    .output()
    .expect("spawn latexml_oxide");
  let log = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
  let xml = std::fs::read_to_string(workdir.path().join("t.xml")).expect("t.xml");
  let json =
    std::fs::read_to_string(workdir.path().join("t.xml.fmshadow.json")).expect("the shadow JSON");
  reading_of(&log, &xml, &json)
}

/// Asserts `tex`'s XML creators are `xml` (when given; else only the same in every mode) and its shadow tuples
/// `shadow`, in process and from the binary, eager and `--streaming` (its notes and the writer's outcome the same in
/// every mode).
#[track_caller]
fn assert_reading(tex: &str, xml: Option<&[&[&str]]>, shadow: &[&str]) -> Reading {
  let shadow: Vec<String> = shadow.iter().map(ToString::to_string).collect();
  let (xml_in_process, shadow_in_process, notes, writer) = in_process(tex);
  if let Some(xml) = xml {
    let xml: Vec<Vec<String>> = xml
      .iter()
      .map(|c| c.iter().map(ToString::to_string).collect())
      .collect();
    assert_eq!(xml_in_process, xml, "in process");
  }
  assert_eq!(shadow_in_process, shadow, "in process");
  let want = (xml_in_process, shadow, notes, writer);
  assert_eq!(with_binary(tex, false), want, "the binary, eager");
  assert_eq!(with_binary(tex, true), want, "the binary, --streaming");
  want
}

/// aa class A of the llncs and aa gate study (1707.06238, 1302.1442; 52 papers): `\inst{\ref{ia},\ref{ib}}` prints "1, 2"
/// once the references resolve, but nothing at digestion but the comma, which the lexer read as the separator between
/// two marks: the shadow gave Ann Able no institute and Univ X to the block. Now one Mark `ref:ia,ref:ib`, each key a
/// request — the reading agrees with today's (and Perl's) XML; an owner found by a key says so (`api:ref-label`, ruling
/// 1's change tag: 1302.1442's entry 1, keyed after its `\email` line, which today's left on a nameless creator), one
/// found by the entry's number does not. Its sibling, the reading only: a space after the comma (1707.06238's
/// `\inst{\ref{inst5}, \ref{inst20}}`), a number and a key in one mark (`\inst{2,\ref{ic}}`: Univ Y by its number, Univ Z
/// by its key) and a formula's script holding two keys (`$^{\ref{ia},\ref{ic}}$`, the third script site) read as
/// pdflatex prints them, "Bob Baker2, 3", "Cid Cole1,3". Today's XML is not asserted there: its old reader keeps Bob
/// Baker's `2` in his name and loses Univ Y (Perl loses Univ Y too), and keeps Cid Cole's formula in his name with no
/// institute (Perl too).
#[test]
fn a_mark_of_refs_requests_every_key() {
  assert_reading(
    repro!("aa_inst_ref_list_is_one_mark.tex"),
    Some(&[
      &["Ann Able", "affiliation=Univ X", "affiliation=Univ Y"],
      &["Bob Baker", "affiliation=Univ Y"],
    ]),
    &[
      "C Ann Able [ref:ia,ref:ib] | line:names",
      "  a Univ X | api:labelseq api:ref-label",
      "  a Univ Y | api:labelseq api:ref-label",
      "C Bob Baker [ref:ib] | line:names",
      "  a Univ Y | api:labelseq api:ref-label",
    ],
  );
  assert_reading(repro!("aa_ref_marks_mixed_and_in_scripts.tex"), None, &[
    "C Ann Able [ref:ia,ref:ib] | line:names",
    "  a Univ X | api:labelseq api:ref-label",
    "  a Univ Y | api:labelseq api:ref-label",
    "C Bob Baker [2,ref:ic] | line:names",
    "  a Univ Y | api:labelseq",
    "  a Univ Z | api:labelseq api:ref-label",
    "C Cid Cole [ref:ia,ref:ic] | line:names",
    "  a Univ X | api:labelseq api:ref-label",
    "  a Univ Z | api:labelseq api:ref-label",
  ]);
  // (references touching, no comma between them — `\ref{ia}\ref{ib}`, `\ref{ia}\fnmsep\ref{ic}`, a range
  // `\ref{ia}--\ref{ic}` read by its ends, `\ref{ib};\ref{ic}`, a number glued to a key `1\ref{ib}` —: each key its
  // own request, what prints between them none; the keys were one label written back to back, `ref:iaref:ib`, which
  // matched no institute (review rev92a_r1 B3). Today's XML, every institute on a nameless creator, is not asserted.)
  assert_reading(repro!("aa_inst_refs_touching_are_each_a_key.tex"), None, &[
    "C Ann Able [ref:ia,ref:ib] | line:names",
    "  a Univ X | api:labelseq api:ref-label",
    "  a Univ Y | api:labelseq api:ref-label",
    "C Bob Baker [ref:ia,ref:ic] | line:names",
    "  a Univ X | api:labelseq api:ref-label",
    "  a Univ Z | api:labelseq api:ref-label",
    "C Cid Cole [ref:ia,ref:ic] | line:names",
    "  a Univ X | api:labelseq api:ref-label",
    "  a Univ Z | api:labelseq api:ref-label",
    "C Dee Dunn [ref:ib,ref:ic] | line:names",
    "  a Univ Y | api:labelseq api:ref-label",
    "  a Univ Z | api:labelseq api:ref-label",
    "C Eve Eng [1,ref:ib] | line:names",
    "  a Univ X | api:labelseq",
    "  a Univ Y | api:labelseq api:ref-label",
  ]);
  // (an ORCID link inside the mark — a paper's `\orcidicon`, `\href{https://orcid.org/…}{…tikzpicture…}`, after a key
  // and after a number — is an event of its own after the mark, the name's ORCID, as `\orcidlink` is: its logo's
  // path data was the marks' labels, its digits parted onto every institute, 2407.07678, 2605.02769, 2608.18287)
  assert_reading(repro!("aa_inst_orcid_link_is_no_mark_text.tex"), None, &[
    "C Ann Able [ref:ia] | line:names",
    "  a Univ X | api:labelseq api:ref-label",
    "  c orcid 0000-0001-2345-6789 | owner:adjacent",
    "C Bob Baker [2] | line:names",
    "  a Univ Y | api:labelseq",
    "  c orcid 0000-0002-3456-7890 | owner:adjacent",
  ]);
}

/// The mark of `\ref`s in the families the observer writes (review rev92a_r1 S1, B3): revtex's
/// `\author{Ann Able$^{\ref{a1},\ref{a2}}$}` with `\affiliation{\label{a1}…}`, where 90a left University X on a
/// nameless creator and Ann Able without it (Perl loses both, 3 warnings; pdflatex prints both under the shared author
/// group), and amsart's `\textsuperscript{\ref{a1},\ref{a2}}` and `\textsuperscript{\ref{a2}\ref{a3}}` (the keys
/// touching, which 92a r1 read as one label: Bob Baker lost both addresses). Each key a request, the writer writes
/// every address to the names requesting it. (amsart's `\label` in `\address` steps no counter, so the page prints no
/// mark: the key is read as written, fms2a2's `ref:` reading.)
#[test]
fn a_written_family_reads_every_key_of_a_mark_of_refs() {
  assert_reading(
    repro!("revtex_ref_list_mark_is_every_key.tex"),
    Some(&[
      &[
        "Ann Able ,",
        "affiliation=University X, Town",
        "affiliation=University Y, City",
      ],
      &["Bob Baker", "affiliation=University Y, City"],
    ]),
    &[
      "C Ann Able [ref:a1,ref:a2] | line:names",
      "  a University X, Town | api:ref-label",
      "  a University Y, City | api:ref-label",
      "C Bob Baker [ref:a2] | line:names",
      "  a University Y, City | api:ref-label",
    ],
  );
  assert_reading(
    repro!("amsart_ref_marks_listed_and_touching.tex"),
    Some(&[
      &[
        "Ann Able ,",
        "email=ann@x.org",
        "address=University X, Town",
        "address=University Y, City",
      ],
      &[
        "Bob Baker",
        "email=bob@y.org",
        "address=University Y, City",
        "address=University Z, Village",
      ],
      &[
        "Cid Cole",
        "email=cid@z.org",
        "address=University Z, Village",
      ],
    ]),
    &[
      "C Ann Able [ref:a1,ref:a2] | line:names",
      "  a University X, Town | api:ref-label",
      "  a University Y, City | api:ref-label",
      "  c email ann@x.org | api:annotate:last",
      "C Bob Baker [ref:a2,ref:a3] | line:names",
      "  a University Y, City | api:ref-label",
      "  a University Z, Village | api:ref-label",
      "  c email bob@y.org | api:annotate:last",
      "C Cid Cole [ref:a3] | line:names",
      "  a University Z, Village | api:ref-label",
      "  c email cid@z.org | api:annotate:last",
    ],
  );
}

/// aa class B of the llncs and aa gate study (0711.1152, 0802.0707; 42 papers): `\offprints{B. Baker, \email{…}}` is a
/// publication note (`\lx@add@pubnote`, `ltx:pubnote`), which the ledger never recorded, so its e-mail, on Bob Baker
/// in today's XML (and Perl's), was missing from the reading. Now recorded, its e-mail call placed by
/// FIDELITY_CHARTER §4's chain exactly: the author it spells. Its siblings, distinct steps of the chain: the name the
/// note prints before an address spelling no one designates the corresponding author (main-session decision on 92a r1
/// question a: "Send offprint requests to: O. Kochukhov", 0903.3512 — today's XML, and Perl's, the last author's by
/// position); an address whose note names no author ("the editors") is the block's — today's nameless creator after
/// the authors (Perl loses it) —, though Ann Able's own e-mail leaves Bob Baker the one author without one: the chain
/// has no elimination (review rev92a_r1 B2, probe p12; 92a r1 gave it to Bob Baker).
#[test]
fn a_publication_note_email_is_placed_by_the_chain() {
  assert_reading(
    repro!("aa_offprints_email_is_the_authors.tex"),
    Some(&[&["Ann Able", "affiliation=Univ X"], &[
      "Bob Baker",
      "email=bob.baker@y.org",
      "affiliation=Univ Y",
    ]]),
    &[
      "C Ann Able [1] | line:names",
      "  a Univ X | api:labelseq",
      "C Bob Baker [2] | line:names",
      "  a Univ Y | api:labelseq",
      "  c email bob.baker@y.org | api:document-note owner:spelled ruling3:one-per-print",
    ],
  );
  assert_reading(
    repro!("aa_offprints_name_is_the_corresponding_author.tex"),
    None,
    &[
      "C O. Kochukhov [1] | line:names",
      "  a Univ X | api:labelseq",
      "  c email oleg@x.se | api:document-note owner:corresponding api:note-name ruling3:one-per-print",
      "C D. Shulyak [2] | line:names",
      "  a Univ Y | api:labelseq",
    ],
  );
  assert_reading(
    repro!("aa_offprints_unspelled_email_is_the_blocks.tex"),
    Some(&[
      &["Ann Able", "affiliation=Univ X", "email=ann@x.org"],
      &["Bob Baker", "affiliation=Univ Y"],
      &["email=office@y.org"],
    ]),
    &[
      "C Ann Able [1] | line:names",
      "  a Univ X | api:labelseq",
      "  c email ann@x.org | api:labelseq api:inner-contact",
      "C Bob Baker [2] | line:names",
      "  a Univ Y | api:labelseq",
      "B email office@y.org | api:document-note owner:block ruling3:one-per-print",
    ],
  );
}

/// A publication note in a written family (review rev92a_r1 B1): amsart's `\thanks` is an `ltx:pubnote`. A link it
/// prints is the note's text, as today's output and TeX keep it, not an e-mail call — `\href{mailto:carl@z.org}{…}`
/// unspelled, and `\href{mailto:bob.baker@y.org}{…}` after "Correspondence to Bob Baker": 92a r1 read each as an
/// e-mail and placed it (by elimination, by spelling) where today's output has none, so the writer declined the paper
/// and both addresses fell to a nameless creator. Now the written output is 90a's. The e-mail call the note makes,
/// `\thanks{Correspondence: \email{carl@z.org}}`, is read: amsart's `\email` is not printed in place (amsart.cls:538
/// stores it for the address block, `Policy::email_in_place`), so the binding's routing is the evidence — the current
/// author's own call (FIDELITY_CHARTER §4, #420), Bob Baker's, as today's XML has it and pdflatex prints it (after
/// University Y, City). The reading agreeing there, the writer writes each address to the name requesting it (92a r2
/// read the call as the block's and declined: both addresses on a nameless creator).
#[test]
fn a_publication_note_reads_only_its_email_calls() {
  let written: &[&[&str]] = &[
    &["Ann Able", "email=ann@x.org", "address=University X, Town"],
    &["Bob Baker", "address=University Y, City"],
  ];
  let reading = [
    "C Ann Able [ref:a1] | line:names",
    "  a University X, Town | api:ref-label",
    "  c email ann@x.org | api:annotate:last",
    "C Bob Baker [ref:a2] | line:names",
    "  a University Y, City | api:ref-label",
  ];
  // (each note's text whole, the link's address in it as pdflatex prints it — review rev92a_r5 S1)
  let (.., notes, _) = assert_reading(
    repro!("amsart_thanks_link_stays_the_notes.tex"),
    Some(written),
    &reading,
  );
  assert_eq!(notes, ["Correspondence to carl@z.org"]);
  let (.., notes, _) = assert_reading(
    repro!("amsart_thanks_spelled_link_stays_the_notes.tex"),
    Some(written),
    &reading,
  );
  assert_eq!(notes, ["Correspondence to Bob Baker, bob.baker@y.org"]);
  assert_reading(
    repro!("amsart_thanks_email_call_is_read.tex"),
    Some(AMSART_CARL_ON_BOB),
    AMSART_CARL_ON_BOB_READING,
  );
}

/// amsart's written output when carl@z.org is Bob Baker's own call: each address on the name requesting it.
const AMSART_CARL_ON_BOB: &[&[&str]] = &[
  &["Ann Able", "email=ann@x.org", "address=University X, Town"],
  &[
    "Bob Baker",
    "email=carl@z.org",
    "address=University Y, City",
  ],
];
const AMSART_CARL_ON_BOB_READING: &[&str] = &[
  "C Ann Able [ref:a1] | line:names",
  "  a University X, Town | api:ref-label",
  "  c email ann@x.org | api:annotate:last",
  "C Bob Baker [ref:a2] | line:names",
  "  a University Y, City | api:ref-label",
  "  c email carl@z.org | api:document-note api:annotate:last owner:own-call",
];

/// Where the class prints a publication note's e-mail call decides whether the note's name is evidence (review
/// rev92a_r2 B1): only a class whose `\email` prints in place (aa, llncs, svjour: `Policy::email_in_place`) prints the
/// address in the note after the name. amsart's `\thanks{Correspondence to Ann Able, \email{carl@z.org}}` prints
/// "Email address: carl@z.org" under Bob Baker's address — 92a r2 wrote it onto Ann Able by the note's name, against
/// today's and the page — and so does "B. Baker or A. Able": the call is the current author's own (FIDELITY_CHARTER §4,
/// #420), the written output today's e-mail with each address on the name requesting it. REVTeX's `\preprint{Contact
/// Ann Able, \email{carl@z.org}}` before the names prints the address nowhere: no author is current, the block's, as
/// today's (92a r2 wrote it onto Ann Able). A routed call is the current author's whole call (review rev92a_r3 S2):
/// `\thanks{Contact: \email{p@x.org, q@y.org}}` after Bob Baker prints both under his address, as the same call outside
/// a note is read — 92a r3 parted the two addresses by order, p@x.org onto Ann Able, and the writer declined. With no
/// live author current the whole call is the block's, one author's call never parted by order (review rev92a_r4 S1):
/// the same `\thanks` before every `\author` keeps both on the block, as today's, the writer "same" (92a r4 gave p@x.org
/// to Ann Able and q@y.org to Bob Baker by order, and the writer declined; the page prints both after Bob Baker's
/// address, deferred with `amsart_thanks_before_every_author_email`). Distinct from the `\thanks` row of
/// `a_publication_note_reads_only_its_email_calls`: a name in the note that the routed call must not follow, the
/// routed call with no current author (of one address and of two), and a call of several addresses.
#[test]
fn a_routed_note_email_is_the_current_authors_own_call() {
  assert_reading(
    repro!("amsart_thanks_named_email_is_the_current_authors.tex"),
    Some(AMSART_CARL_ON_BOB),
    AMSART_CARL_ON_BOB_READING,
  );
  assert_reading(
    repro!("amsart_thanks_or_names_email_is_the_current_authors.tex"),
    Some(AMSART_CARL_ON_BOB),
    AMSART_CARL_ON_BOB_READING,
  );
  assert_reading(
    repro!("amsart_thanks_routed_call_of_two_addresses.tex"),
    Some(&[&["Ann Able", "address=University X, Town"], &[
      "Bob Baker",
      "address=University Y, City",
      "email=p@x.org, q@y.org",
    ]]),
    &[
      "C Ann Able [] | line:names",
      "  a University X, Town | api:annotate:last",
      "C Bob Baker [] | line:names",
      "  a University Y, City | api:annotate:last",
      "  c email p@x.org, q@y.org | api:document-note api:annotate:last owner:own-call",
    ],
  );
  assert_reading(
    repro!("revtex_preprint_email_before_the_names_is_the_blocks.tex"),
    Some(&[
      &["Ann Able", "affiliation=University X, Town"],
      &["Bob Baker", "affiliation=University Y, City"],
      &["email=carl@z.org"],
    ]),
    &[
      "C Ann Able [ref:a1] | line:names",
      "  a University X, Town | api:ref-label",
      "C Bob Baker [ref:a2] | line:names",
      "  a University Y, City | api:ref-label",
      "B email carl@z.org | api:document-note owner:block",
    ],
  );
  let (.., writer) = assert_reading(
    repro!("amsart_thanks_before_every_author_two_addresses.tex"),
    Some(&[
      &["Ann Able", "address=University X, Town"],
      &["Bob Baker", "address=University Y, City"],
      &["email=p@x.org, q@y.org"],
    ]),
    &[
      "C Ann Able [] | line:names",
      "  a University X, Town | api:annotate:last",
      "C Bob Baker [] | line:names",
      "  a University Y, City | api:annotate:last",
      "B email p@x.org, q@y.org | api:document-note owner:block",
    ],
  );
  assert_eq!(writer, "same", "the reading agrees with today's");
}

/// A note's piece names a creator only when it is that creator's name, every word agreeing (review rev92a_r2 B2,
/// `Resolver::note_named`): 92a r2 matched a piece's last word alone, parted only at `,`, `&` and "and". aa's
/// `\offprints{J. Smith, \email{…}}` under Adam Smith names no author (J. is not Adam), "A. Able or B. Baker" names two,
/// "the editor of Bob Baker" no one: the block's each, today's XML unchanged (a nameless creator). Over the live authors
/// only (S1): llncs's `\author{Ann Able\inst{1}}` cleared by the second `\author{Ann Able\inst{1} \and Bob Baker…}`,
/// then `\thanks{Corresponding: A. Able, \email{carl@z.org}}` — "Corresponding" a piece naming no one — is Ann Able's,
/// as pdflatex prints it (92a r2: the cleared Ann Able a second match, the block's; today's XML Bob Baker's, the last
/// creator). A colon parts the pieces as a comma does (review rev92a_r3 S1): aa's `\offprints{A. Able: \email{…}}` and
/// llncs's `\thanks{Ann Able: corresponding author, \email{…}}` name Ann Able, as the page prints the address after her
/// name (92a r3 dropped the lead before the colon as a label: the block's); today's XML Bob Baker's.
#[test]
fn a_note_names_a_creator_only_by_their_whole_name() {
  for (repro, address) in [
    (
      repro!("aa_offprints_initial_mismatch_is_the_blocks.tex"),
      "js@mpia.de",
    ),
    (
      repro!("aa_offprints_two_names_is_the_blocks.tex"),
      "x@z.org",
    ),
    (
      repro!("aa_offprints_prose_name_is_the_blocks.tex"),
      "ed@z.org",
    ),
  ] {
    let second = if address == "js@mpia.de" {
      "Adam Smith"
    } else {
      "Bob Baker"
    };
    let email = format!("email={address}");
    let creator = format!("C {second} [2] | line:names");
    let block = format!("B email {address} | api:document-note owner:block ruling3:one-per-print");
    assert_reading(
      repro,
      Some(&[
        &["Ann Able", "affiliation=Univ X"],
        &[second, "affiliation=Univ Y"],
        &[&email],
      ]),
      &[
        "C Ann Able [1] | line:names",
        "  a Univ X | api:labelseq",
        &creator,
        "  a Univ Y | api:labelseq",
        &block,
      ],
    );
  }
  assert_reading(
    repro!("llncs_thanks_name_after_cleared_author.tex"),
    Some(&[&["Ann Able", "affiliation=Univ X"], &[
      "Bob Baker",
      "email=carl@z.org",
      "affiliation=Univ Y",
    ]]),
    &[
      "C Ann Able [1] | line:names",
      "  a Univ X | api:labelseq",
      "  c email carl@z.org | api:document-note owner:corresponding api:note-name ruling3:one-per-print",
      "C Bob Baker [2] | line:names",
      "  a Univ Y | api:labelseq",
    ],
  );
  for repro in [
    repro!("aa_offprints_colon_after_the_name.tex"),
    repro!("llncs_thanks_name_before_a_colon.tex"),
  ] {
    assert_reading(
      repro,
      Some(&[&["Ann Able", "affiliation=Univ X"], &[
        "Bob Baker",
        "email=carl@z.org",
        "affiliation=Univ Y",
      ]]),
      &[
        "C Ann Able [1] | line:names",
        "  a Univ X | api:labelseq",
        "  c email carl@z.org | api:document-note owner:corresponding api:note-name ruling3:one-per-print",
        "C Bob Baker [2] | line:names",
        "  a Univ Y | api:labelseq",
      ],
    );
  }
}

/// A note's e-mail call is read only when a binding's meaning makes it (review rev92a_r3 B1, `ledger::email_meaning`,
/// the meaning when the note is rehearsed, at the flush, `\maketitle`; review rev92a_r4 B1): a paper's own `\email`
/// prints wherever its definition prints, so its call stays the note's text, as today's XML keeps it and TeX prints it
/// — amsart's `\thanks{Correspondence to Ann Able, \email{carl@z.org}}` under `\renewcommand{\email}[1]{\texttt{#1}}`
/// prints "Correspondence to Ann Able, carl@z.org" in the footnote (1412.0748, 1809.01965: 92a r3 read it as Bob
/// Baker's routed call, so the writer declined both papers), and aa's `\offprints{A. Able, \email{…}}` "A. Able,
/// carl@z.org" in the note (92a r3 Bob Baker's, r2 Ann Able's): no creator holds the address, and the note holds it
/// whole (review rev92a_r4 S2: a lost note or address must fail). Each call by its own command's meaning: a note calling the
/// binding's `\email` and the paper's own `\emailAdd` gives the first to Bob Baker, the current author, and keeps the
/// second its text (the writer "same"; the base declined). Distinct rows: a routed class, a class printing in place,
/// and two commands of different meanings in one note.
#[test]
fn a_note_email_of_no_bindings_meaning_stays_the_notes_text() {
  let (.., notes, _) = assert_reading(
    repro!("amsart_thanks_document_email_stays_the_notes.tex"),
    Some(&[&["Ann Able", "address=University X, Town"], &[
      "Bob Baker",
      "address=University Y, City",
    ]]),
    &[
      "C Ann Able [] | line:names",
      "  a University X, Town | api:annotate:last",
      "C Bob Baker [] | line:names",
      "  a University Y, City | api:annotate:last",
    ],
  );
  // (the note's text whole, the address in it as pdflatex prints it)
  assert_eq!(notes, ["Correspondence to Ann Able, carl@z.org"]);
  let (.., notes, _) = assert_reading(
    repro!("aa_offprints_document_email_stays_the_notes.tex"),
    Some(&[&["Ann Able", "affiliation=Univ X"], &[
      "Bob Baker",
      "affiliation=Univ Y",
    ]]),
    &[
      "C Ann Able [1] | line:names",
      "  a Univ X | api:labelseq",
      "C Bob Baker [2] | line:names",
      "  a Univ Y | api:labelseq",
    ],
  );
  assert_eq!(notes, ["A. Able, carl@z.org"]);
  let (.., notes, _) = assert_reading(
    repro!("amsart_thanks_document_emailadd_beside_the_bindings_email.tex"),
    Some(&[&["Ann Able", "address=University X, Town"], &[
      "Bob Baker",
      "address=University Y, City",
      "email=carl@z.org",
    ]]),
    &[
      "C Ann Able [] | line:names",
      "  a University X, Town | api:annotate:last",
      "C Bob Baker [] | line:names",
      "  a University Y, City | api:annotate:last",
      "  c email carl@z.org | api:document-note api:annotate:last owner:own-call",
    ],
  );
  // (the binding's address gone to Bob Baker, the paper's own `\emailAdd` kept in the note — rev92a_r5 S1)
  assert_eq!(notes, ["Correspondence to ; at home bob@home.org"]);
}

/// llncs class N1 of the llncs and aa gate study (2104.04331, 1712.00617; 7 papers): an anonymous `\author` and
/// `\institute`, then the real ones. llncs's `\author` clears the authors (`\lx@clear@creators[role=author]`) and its
/// `\institute` the affiliations before queueing its own, but the ledger kept the cleared calls: the reading held
/// Anonymous Author, and Anonymous Institute numbered 1 — Ann Able's, Univ Y on the block. Now a clear removes the API
/// calls it dequeues. Its sibling (the author-block path): the first `\author` marked, an author block of its own, and
/// the second unmarked — the block's creators are cleared too, as a replacing call's.
#[test]
fn a_frontmatter_clear_removes_the_calls_before_it() {
  assert_reading(
    repro!("llncs_author_twice_keeps_the_last.tex"),
    Some(&[&["Ann Able", "affiliation=Univ X"], &[
      "Bob Baker",
      "affiliation=Univ Y",
    ]]),
    &[
      "C Ann Able [1] | line:names",
      "  a Univ X | api:labelseq",
      "C Bob Baker [2] | line:names",
      "  a Univ Y | api:labelseq",
    ],
  );
  assert_reading(
    repro!("llncs_marked_author_then_unmarked_keeps_the_last.tex"),
    Some(&[&["Cid Cole"], &["Dee Dunn"]]),
    &["C Cid Cole [] | api:creator", "C Dee Dunn [] | api:creator"],
  );
}
