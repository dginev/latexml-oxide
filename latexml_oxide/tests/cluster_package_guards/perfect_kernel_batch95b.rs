//! Guards for batch 95b (FM S2b: the observer writes IEEEtran, acmart, authblk and the MNRAS family, each family's own
//! declaration `written`; OXIDIZED_DESIGN_DIVERGENCES #837, #942). Each guard converts its repro under the shadow
//! switch in process and with the binary, eager and `--streaming` (`perfect_kernel_batch92a::assert_reading`), and
//! asserts the shadow reading's tuples, the XML's creators with every contact, the writer's outcome, and no Error or
//! Warning; the family guards also convert in the default mode (the fleet's), which writes the same creators. Every
//! repro compiles clean with pdflatex (TL 2025; the mn2e repro with 0810.2791's mn2e.cls, which TeX Live lacks).

use latexml::util::test::{convert_with_setup_then, error_count, warning_count};
use latexml_engine::base_utilities::set_fm_shadow_override;

use super::{perfect_kernel_batch84b::creators, perfect_kernel_batch92a::assert_reading};

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/class-bindings/",
      $path
    ))
  };
}

/// The XML's creators of `tex` converted in the default mode (no shadow switch: the fleet's), with no Error or Warning.
#[track_caller]
fn default_mode_creators(tex: &str) -> Vec<Vec<String>> {
  let (log, xml, ()) = convert_with_setup_then(
    tex,
    Some("ar5iv.sty"),
    || set_fm_shadow_override(None),
    |_| (),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  creators(&xml)
}

/// The family guards: the reading written (the writer's outcome `written`), and the default mode writing the same.
#[track_caller]
fn assert_written(tex: &str, xml: &[&[&str]], shadow: &[&str]) {
  let (written, _, _, writer) = assert_reading(tex, Some(xml), shadow);
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
  assert_written(
    repro!("ieeetran_entry_addresses_written.tex"),
    &[
      &["Ann Able 1", "affiliation=Univ X", "email=ann.able@x.org"],
      &["Bob Baker 1", "affiliation=Univ X", "email=bbaker@x.org"],
      &["Cid Cole 2", "affiliation=Univ Y", "email=cid@y.org"],
    ],
    &[
      "C Ann Able [1] | line:names",
      "  a Univ X | api:leading-mark",
      "  c email ann.able@x.org | api:annotate:new owner:entry-address owner:spelled",
      "C Bob Baker [1] | line:names sep:comma",
      "  a Univ X | api:leading-mark",
      "  c email bbaker@x.org | api:annotate:new owner:entry-address owner:spelled",
      "C Cid Cole [2] | line:names sep:word",
      "  a Univ Y | api:leading-mark",
      "  c email cid@y.org | api:annotate:new owner:entry-address owner:spelled",
    ],
  );
}

/// An address taken out of an entry with one carrier, spelling no one, is that carrier's (`owner:only-carrier`,
/// FIDELITY_CHARTER §4's chain) and written: `owner:entry-address` is a change tag (`resolver::CHANGE_TAGS`, as OD #938
/// names it), so the writer writes every owner the take-out decided, not only a spelled one (0707.1008's
/// `\IEEEauthorblockA{ETH Zurich\\ … Email: lapidoth@isi.ee.ethz.ch}`; 36 IEEEtran census papers). Distinct from
/// `ieeetran_is_written`: the owner by the entry, not by spelling.
#[test]
fn an_entry_address_of_its_only_carrier_is_written() {
  assert_written(
    repro!("ieeetran_entry_address_of_its_only_carrier.tex"),
    &[
      &["Ann Able", "affiliation=Univ X", "email=office@x.org"],
      &["Bob Baker", "affiliation=Univ Y", "email=bob@y.org"],
    ],
    &[
      "C Ann Able [] | line:names",
      "  a Univ X | api:annotate:new ruling3:address-out",
      "  c email office@x.org | api:annotate:new owner:entry-address owner:only-carrier",
      "C Bob Baker [] | line:names",
      "  a Univ Y | api:annotate:new ruling3:address-out",
      "  c email bob@y.org | api:annotate:new owner:entry-address owner:spelled",
    ],
  );
}

/// acmart is written under its own conventions (`acmart_cls.rs` `FRONTMATTER_CONVENTIONS`, no longer `undeclared`): an
/// `\author` list (acmart.cls:1620-1621) whose `\affiliation` prints one marked line per institution, each its marked
/// authors', and an `\email` group written out to its spellers (2311.10779; 2010.12245's shape, `\institution{$^1$…,
/// $^2$…}`, stays declined: its pieces are no balanced run of the address part's wrapper, SYNC). Today's gave every
/// creator both lines and the last both addresses.
#[test]
fn acmart_is_written() {
  assert_written(
    repro!("acmart_author_list_written.tex"),
    &[
      &["Ann Able 1", "affiliation=1 Univ X", "email=ann.able@x.org"],
      &["Bob Baker 2", "affiliation=2 Univ Y"],
      &["Cid Cole 1", "affiliation=1 Univ X", "email=cid.cole@x.org"],
    ],
    &[
      "C Ann Able [1] | line:names",
      "  a Univ X | api:leading-mark api:split-by-marks",
      "  c email ann.able@x.org | api:annotate:last ruling3:written-out owner:spelled",
      "C Bob Baker [2] | line:names sep:comma",
      "  a Univ Y | api:leading-mark api:split-by-marks",
      "C Cid Cole [1] | line:names sep:word",
      "  a Univ X | api:leading-mark api:split-by-marks",
      "  c email cid.cole@x.org | api:annotate:last ruling3:written-out owner:spelled",
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
      &[
        "Bob Baker*",
        "affiliation=Univ Y",
        "affiliation=Equal contribution.",
      ],
      &[
        "Cid Cole*",
        "affiliation=Univ X",
        "affiliation=Equal contribution.",
      ],
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
}

/// The writer never takes an item that prints, past its first ink, a mark its creator requests (`plan::later_marks`,
/// read as the comparator reads marks: an `ltx:sup` or a formula that is one): the reading labels a line by its leading
/// mark alone where a later symbol is glued to the word before it (`City$^\ddagger$Lab Y`, which opens no entry, as
/// `ECT$^*$`), so taking the line from the `‡` name lost that name's own affiliation (2311.10290's Hongyang Chen,
/// 2405.00230's Maximilian Schiffer under an unlabelled authblk `\affil` printing `\textsuperscript{2}`). Declined:
/// today's, each creator the whole line. Distinct from the family guards: the writer's fail-safe, not a reading.
#[test]
fn an_item_printing_a_later_mark_is_not_taken() {
  let (_, _, _, writer) = assert_reading(
    repro!("ieeetran_entry_with_a_later_mark_keeps_its_carriers.tex"),
    Some(&[
      &["Ann Able †", "affiliation=† Univ X, City ‡ Lab Y, Town"],
      &["Bob Baker †", "affiliation=† Univ X, City ‡ Lab Y, Town"],
      &["Cid Cole ‡", "affiliation=† Univ X, City ‡ Lab Y, Town"],
    ]),
    &[
      "C Ann Able [†] | line:names",
      "  a Univ X, City Lab Y, Town | api:leading-mark",
      "C Bob Baker [†] | line:names sep:comma",
      "  a Univ X, City Lab Y, Town | api:leading-mark",
      "C Cid Cole [‡] | line:names sep:word",
    ],
  );
  assert!(
    writer
      .as_str()
      .is_some_and(|w| w.starts_with("declined: Affiliation univxcitylabytown")),
    "{writer}"
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
