//! Guards for batch 95b (FM S2b: the observer writes IEEEtran, acmart, authblk and the MNRAS family, each family's own
//! declaration `written`; OXIDIZED_DESIGN_DIVERGENCES #837, #942). Each guard converts its repro in process and with the binary, eager and `--streaming` (`perfect_kernel_batch92a::assert_reading`), and
//! asserts the reading's tuples, the XML's creators with every contact, the writer's outcome, and no Error or
//! Warning; the family guards also convert in the default mode (the fleet's), which writes the same creators. Every
//! repro compiles clean with pdflatex (TL 2025; the mn2e repro with 0810.2791's mn2e.cls, which TeX Live lacks).

use latexml::util::test::{convert_with_setup_then, error_count, warning_count};

use super::{perfect_kernel_batch84b::creators, perfect_kernel_batch92a::assert_reading};

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/class-bindings/",
      $path
    ))
  };
}

/// The XML's creators of `tex` converted in the default mode (the fleet's), with no Error or Warning.
#[track_caller]
pub(super) fn default_mode_creators(tex: &str) -> Vec<Vec<String>> {
  let (log, xml, ()) = convert_with_setup_then(tex, Some("ar5iv.sty"), || {}, |_| ());
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  creators(&xml)
}

/// The family guards: the reading written (the writer's outcome `written`), and the default mode writing the same.
#[track_caller]
pub(super) fn assert_written(tex: &str, xml: &[&[&str]], tuples: &[&str]) {
  let (written, _, _, writer) = assert_reading(tex, Some(xml), tuples);
  assert_eq!(writer, "written", "the writer's outcome");
  assert_eq!(default_mode_creators(tex), written, "the default mode");
}

/// IEEEtran is written (`ieeetran_cls.rs` `written: true`): an `\IEEEauthorblockA` set under the names with an e-mail
/// line printing several addresses — each address an e-mail of the author it spells, cut out of the entry it printed
/// in (brief 95c's take-out, OD #938), each entry its marked authors' (2202.02533, 2401.16729, 1303.4266, 2405.16358;
/// the scorer's 18 IEEEtran `email_of_other` creators). Before, IEEEtran's `written: false` kept today's: every creator
/// holding the whole line as an affiliation.
#[test]
fn ieeetran_is_written() {
  // (since 100b, OD #956, today's kinds the e-mail line itself at the API and places it by the chain the reading runs:
  // the reading reads the entry's e-mails, `api:inner-contact owner:inner-email`, and the writer has nothing to write)
  let tex = repro!("ieeetran_entry_addresses_written.tex");
  let (..) = assert_reading(
    tex,
    Some(&[
      &["Ann Able 1", "affiliation=Univ X", "email=ann.able@x.org"],
      &["Bob Baker 1", "affiliation=Univ X", "email=bbaker@x.org"],
      &["Cid Cole 2", "affiliation=Univ Y", "email=cid@y.org"],
    ]),
    &[
      "C Ann Able [1] | line:names",
      "  a Univ X | api:leading-mark",
      "  c email ann.able@x.org | api:annotate:new api:inner-contact owner:inner-email owner:spelled",
      "C Bob Baker [1] | line:names sep:comma",
      "  a Univ X | api:leading-mark",
      "  c email bbaker@x.org | api:annotate:new api:inner-contact owner:inner-email owner:spelled",
      "C Cid Cole [2] | line:names sep:word",
      "  a Univ Y | api:leading-mark",
      "  c email cid@y.org | api:annotate:new api:inner-contact owner:inner-email owner:spelled",
    ],
  );
}

/// An address taken out of an entry with one carrier, spelling no one, is that carrier's (`owner:only-carrier`,
/// FIDELITY_CHARTER §4's chain) and written: `owner:entry-address` is a change tag (`resolver::CHANGE_TAGS`, as OD #938
/// names it), so the writer writes every owner the take-out decided, not only a spelled one (0707.1008's
/// `\IEEEauthorblockA{ETH Zurich\\ … Email: lapidoth@isi.ee.ethz.ch}`; 36 IEEEtran census papers). Distinct from
/// `ieeetran_is_written`: the owner by the entry, not by spelling. Since 100b (OD #956) the author block's Kind step
/// takes the e-mail line out of the entry at the API, so the reading reads it as the entry's e-mail
/// (`api:inner-contact owner:inner-email`, the chain both engines run) and today's holds it already: the writer has
/// nothing to write (`same`), the owners unchanged.
#[test]
fn an_entry_address_of_its_only_carrier_is_written() {
  let tex = repro!("ieeetran_entry_address_of_its_only_carrier.tex");
  let (..) = assert_reading(
    tex,
    Some(&[
      &["Ann Able", "affiliation=Univ X", "email=office@x.org"],
      &["Bob Baker", "affiliation=Univ Y", "email=bob@y.org"],
    ]),
    &[
      "C Ann Able [] | line:names",
      "  a Univ X | api:annotate:new",
      "  c email office@x.org | api:annotate:new api:inner-contact owner:inner-email owner:only-carrier",
      "C Bob Baker [] | line:names",
      "  a Univ Y | api:annotate:new",
      "  c email bob@y.org | api:annotate:new api:inner-contact owner:inner-email owner:spelled",
    ],
  );
}

/// authblk is written (`authblk_sty.rs` `written: true`): an `\affil[1]{… Emails: ann@x.org; cole@x.org}` holding its
/// authors' addresses — each its speller's, cut out (2511.07020, 2008.07734, 2110.01283, 2008.13643). Before, each
/// carrier of entry 1 held both addresses inside it.
#[test]
fn authblk_is_written() {
  assert_written(
    repro!("authblk_entry_addresses_written.tex"),
    &[
      &[
        "Ann Able",
        "affiliation=Univ X, Croatia.",
        "email=ann@x.org",
      ],
      &["Bob Baker", "affiliation=Univ Y.", "email=bob@y.org"],
      &[
        "Cid Cole",
        "affiliation=Univ X, Croatia.",
        "email=cole@x.org",
      ],
    ],
    &[
      "C Ann Able [1] | line:names",
      "  a Univ X, Croatia. | api:label ruling1:request ruling3:address-out",
      "  c email ann@x.org | api:label ruling1:request owner:entry-address owner:spelled",
      "C Bob Baker [2] | line:names",
      "  a Univ Y. | api:label ruling1:request ruling3:address-out",
      "  c email bob@y.org | api:label ruling1:request owner:entry-address owner:spelled",
      "C Cid Cole [1] | line:names",
      "  a Univ X, Croatia. | api:label ruling1:request ruling3:address-out",
      "  c email cole@x.org | api:label ruling1:request owner:entry-address owner:spelled",
    ],
  );
}

/// A call's label as its source spells it (authblk's `\author[…]` keyval, a backslash in it) is what it spells before
/// its first command (`resolver::parted_label`): `\author[3\authfn{2}]{Raul Ramos-Pollán}` (2405.16051) is `3` — parted
/// whole, the command's letters were labels and its argument's `2` a request of `\affil[1,2,4]`, written as a second,
/// wrong affiliation; `\author[1\thanks{…}]` (2202.05736, 2410.10922) is `1`, as the parting read it before. The
/// starred entry is the starred names' (`\affil[*]{…}` under `\author[a]{Harini Veeraraghavan*}`, 2605.12855):
/// FIDELITY_CHARTER §4 ruling 2's symbol line, "affiliation or legend alike" (a legend note of it lost
/// `Corresponding author, …` addresses on seven sample papers, 95b r1).
#[test]
fn a_label_spelled_with_a_command_is_read_to_it() {
  assert_written(
    repro!("authblk_label_spelled_with_a_command.tex"),
    &[
      &["Ann Able", "affiliation=Univ X"],
      // (the starred legend each starred name's note since 103c, which this view of contacts does not list — the
      // notes' view below pins it; the reading's tuples still give it to both: `insert::kind_legend_contacts`, OD #961)
      &["Bob Baker*", "affiliation=Univ Y"],
      &["Cid Cole*", "affiliation=Univ X"],
      &["Dee Dunn", "affiliation=Univ Z"],
    ],
    &[
      "C Ann Able [1] | line:names",
      "  a Univ X | api:label ruling1:request",
      "C Bob Baker [2,*] | line:names",
      "  a Univ Y | api:label ruling1:request",
      "  a Equal contribution. | api:label ruling1:request",
      "C Cid Cole [1,*] | line:names",
      "  a Univ X | api:label ruling1:request",
      "  a Equal contribution. | api:label ruling1:request",
      "C Dee Dunn [3] | line:names mark:parted",
      "  a Univ Z | api:label ruling1:parted",
    ],
  );
  // (where the starred legend goes: each starred name's note, marked `*` — review rev103c S3)
  assert_eq!(
    super::perfect_kernel_batch103c::creator_notes(repro!(
      "authblk_label_spelled_with_a_command.tex"
    )),
    [
      vec!["Ann Able"],
      vec!["Bob Baker*", "note[*]=Equal contribution."],
      vec!["Cid Cole*", "note[*]=Equal contribution."],
      vec!["Dee Dunn"],
    ]
  );
}

/// The take-out's holders are the live creators' after `resolve` deletes a replaced `\author` call's (`resolver::
/// resolve`, the holders remapped): IEEEtran's anonymous-review `\author{Author Names Omitted for Anonymous Review}`
/// before the real one (2602.12633) left the last holder past the end — the writer's give-back panicked (Fatal
/// `panic:caught`, the IEEEtran census's one new Fatal under the writer), and any other holder named the creator after
/// its own. Distinct from `ieeetran_is_written`: a replaced call before the block.
#[test]
fn entry_addresses_after_a_replaced_author_are_written() {
  assert_written(
    repro!("ieeetran_entry_addresses_after_a_replaced_author.tex"),
    &[
      &[
        "Ann Able",
        "affiliation=Ann Able and Bob Baker are with Univ X",
        "email=ann.able@x.org",
      ],
      &[
        "Bob Baker",
        "affiliation=Ann Able and Bob Baker are with Univ X",
        "email=bbaker@x.org",
      ],
      &[
        "Cid Cole",
        "affiliation=Cid Cole is with Univ Y",
        "email=cid@y.org",
      ],
    ],
    &[
      "C Ann Able [1] | line:names",
      "  a Ann Able and Bob Baker are with Univ X | note:adjacent note:legend ruling1:request ruling3:address-out",
      "  c email ann.able@x.org | note:adjacent note:legend ruling1:request owner:entry-address owner:spelled",
      "C Bob Baker [1] | line:names sep:comma",
      "  a Ann Able and Bob Baker are with Univ X | note:adjacent note:legend ruling1:request ruling3:address-out",
      "  c email bbaker@x.org | note:adjacent note:legend ruling1:request owner:entry-address owner:spelled",
      "C Cid Cole [2] | line:names sep:word",
      "  a Cid Cole is with Univ Y | note:adjacent note:legend ruling1:request ruling3:address-out",
      "  c email cid@y.org | note:adjacent note:legend ruling1:request owner:entry-address owner:spelled",
    ],
  );
}

/// The MNRAS family is written under its own conventions (`mn2e_support_sty.rs` `FRONTMATTER_CONVENTIONS`, the binding
/// owning `\author` for mn2e, mnras and mn): a marked address line printing its authors' addresses in parentheses
/// (`(able,baker@x.it)`, a list sharing its domain) — each address its speller's, cut out (0810.2791, 1001.5025,
/// astro-ph0309037). Before, Ann Able and Bob Baker each held both addresses inside Univ X.
#[test]
fn mn2e_is_written() {
  assert_written(
    repro!("mn2e_entry_addresses_written.tex"),
    &[
      &["Ann Able", "affiliation=Univ X, Italy", "email=able@x.it"],
      &[
        "Bob Baker",
        "affiliation=Univ X, Italy",
        "email=baker@x.it",
        "affiliation=Univ Y, Germany",
      ],
      &["Cid Cole", "affiliation=Univ Y, Germany"],
    ],
    &[
      "C Ann Able [1] | line:names",
      "  a Univ X, Italy | ruling1:request ruling3:address-out",
      "  c email able@x.it | ruling1:request owner:entry-address owner:spelled",
      "C Bob Baker [1,2] | line:names sep:comma",
      "  a Univ X, Italy | ruling1:request ruling3:address-out",
      "  a Univ Y, Germany | ruling1:request",
      "  c email baker@x.it | ruling1:request owner:entry-address owner:spelled",
      "C Cid Cole [2] | line:names sep:word",
      "  a Univ Y, Germany | ruling1:request",
    ],
  );
}

/// An entry's number is parted into digits only under single-digit numbering (`Resolver::part_glued_labels`, the entry
/// side, the mirror of the names' side guard of 1501.05343): MNRAS's `$^{15}$The Inter-University Institute …` under
/// names marked up to 18, none marked 15 (2207.06713), stays in the block as it prints — 95b parted it into `1` and `5`
/// and gave it to every name marked 1 or 5 (9 creators). IEEEtran's sibling (`\textsuperscript{15}`, review rev95b p3)
/// on the same path. Entries 1, 5 and 10 their names'. Distinct from `mn2e_is_written`: the label reading, not the
/// take-out.
#[test]
fn an_entry_number_no_name_shows_stays_in_the_block() {
  let tex = repro!("mnras_entry_number_of_no_name_stays_in_the_block.tex");
  let (..) = assert_reading(
    tex,
    Some(&[
      &["Ann Able", "affiliation=University One, Town"],
      &["Bob Baker", "affiliation=University Five, Town"],
      &["Cid Cole", "affiliation=University Ten, Town"],
      &["affiliation=University Fifteen, Town"],
    ]),
    &[
      "C Ann Able [1] | line:names mark:after-comma",
      "  a University One, Town | ruling1:request",
      "C Bob Baker [5] | line:names sep:comma mark:after-comma",
      "  a University Five, Town | ruling1:request",
      "C Cid Cole [10] | line:names sep:comma",
      "  a University Ten, Town | ruling1:request",
      "B affiliation University Fifteen, Town | ruling1:request label:unrequested owner:block",
    ],
  );
}
