//! Red/green guards for batch 88b: two pieces of frontmatter binding-API data.
//!
//! - ijcai's `\emails` list (OXIDIZED_DESIGN_DIVERGENCES #908, KNOWN_PERL_ERRORS #1200): the line ijcai19.sty:127-130
//!   prints under the affiliations, attributing none, which Perl's ijcai.sty.ltxml:50-53 split at every comma and gave
//!   the n-th piece to the n-th author — a grouped `\{aable, ccole\}@x.edu` cut into `{aable` and `ccole}@x.edu`, every
//!   piece after it one author on (2105.02711, 1905.13566, 2308.08563; 2401.03955's ttm.sty). It is now a class's
//!   address list (`owner=sequence`, as jheppub's `\emailAdd`), fixed once in the engine's `\lx@ijcai@*` machinery so
//!   the binding's `\author` and a raw derivative's marker branch share it: the author an address spells, else the
//!   n-th author while the list is one address per author in their order, else the author block; a grouped address
//!   written out, one address each (`\lx@split@addresses`); the section read line by line (`\\`, a further `\emails`;
//!   no break inside a printed group), a line with no address a note.
//! - cas's `\address[label]{…}[keys]` (2011.00505): labelled as cas-common.sty:1216 labels it, as `\affiliation` is,
//!   where OmniBus's `\address[]{}` dropped the label; the trailing `stm/address` keys of both commands are no text;
//!   every entry labelled by the letter cas's entry counter prints (a letter label steps it, a number N sets it, a
//!   `foot` entry leaves it), an author's number N the N-th letter, its letter label the letter that entry prints
//!   (2207.12381). A letter label whose entry comes later is a deferred reference (`\lx@label@ref`, filled from cas's
//!   counter), read at the flush by the digestion and by the queue's readers before it (the observer's, `place.rs`'s);
//!   a label written twice keeps its first entry, as `\NewLabel` read from the `.aux` does. `\corref{label}` prints
//!   nothing (2303.00352).
//!
//! One guard per path: the raw derivative's marker branch with a grouped address placed by spelling (eager and
//! `--streaming`), the binding's `\author` with a list in no author order (the unspelled addresses on the block, eager
//! and `--streaming`), the section's lines (a `\\`, a second `\emails`, a line with no address, a `\\` inside a
//! printed group; eager and `--streaming`), cas-sc's numeric labels with and without an `[orcid]` (eager and
//! `--streaming`), cas-dc's letter labels, a two-label author and the trailing keys (a `foot` entry among them), and
//! the entry counter (numeric marks naming letter entries, letters written out of order, word labels read by the
//! letter they print, a number entry before a letter one), the deferred labels read by the shadow (word labels,
//! cas-dc letter marks before their entries, one entry before and one after with a corresponding-author legend, a
//! label written twice with an author between its entries and with both after the authors), and `\corref`'s label
//! (inside the name and after the `\author`). Not
//! red/green: the binding's `\author` with a list in author order gives Perl's answer on the base too — a
//! no-regression check of the order step. Each repro is under `tools/perfect_kernel/repros/class-bindings/`,
//! pdflatex-checked.
use std::{path::Path, process::Command};

use latexml::util::test::convert_with_setup_then;
use latexml_engine::base_utilities::{fm_shadow_json, set_fm_shadow_override};
use serde_json::Value;

use super::{
  perfect_kernel_batch46::{convert_with, error_count, warning_count},
  perfect_kernel_batch84b::creators,
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

/// Converts `tex` in process, checks it converts with no Error or Warning, and returns its creators.
#[track_caller]
fn eager_creators(tex: &str) -> Vec<Vec<String>> {
  let (log, xml) = convert_with(tex, Some(PRELOAD));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  creators(&xml)
}

/// Converts `tex` with the binary, `--streaming=true` when `streaming`, checks it converts with no Error or Warning,
/// and returns its creators.
#[track_caller]
fn cli_creators(tex: &str, streaming: bool) -> Vec<Vec<String>> {
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
    .current_dir(workdir.path())
    .output()
    .expect("spawn latexml_oxide");
  let log = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  creators(&std::fs::read_to_string(workdir.path().join("t.xml")).expect("t.xml"))
}

/// Asserts `tex`'s creators are `expected`, in process and from the binary eager and `--streaming`.
#[track_caller]
fn assert_creators_on_both_builds(tex: &str, expected: &[&[&str]]) {
  let expected: Vec<Vec<String>> = expected
    .iter()
    .map(|c| c.iter().map(ToString::to_string).collect())
    .collect();
  assert_eq!(eager_creators(tex), expected);
  assert_eq!(cli_creators(tex, false), expected, "the binary, eager");
  assert_eq!(cli_creators(tex, true), expected, "the binary, --streaming");
}

/// 1905.13566's shape, on a raw IJCAI style's `\maketitle` (the `\lx@add@authors` marker branch): the grouped
/// `\{aable, ccole\}@x.edu` is written out, `aable@x.edu` and `ccole@x.edu`, each on the author it spells, and
/// `bob.baker@y.org` on Bob Baker — where the comma split gave Able `{aable`, Baker `ccole}@x.edu`, Cole Baker's.
#[test]
fn ijcai_grouped_email_list_placed_by_spelling() {
  assert_creators_on_both_builds(repro!("ijcai_emails_grouped_address_by_spelling.tex"), &[
    &["Ann Able", "affiliation=Univ X", "email=aable@x.edu"],
    &["Bob Baker", "affiliation=Univ Y", "email=bob.baker@y.org"],
    &["Cid Cole", "affiliation=Univ X", "email=ccole@x.edu"],
  ]);
}

/// The ijcai binding's `\author`: a list of one address per author, none spelling another, is in author order (the
/// charter's consistent-order evidence) — the n-th on the n-th author, Perl's answer. A no-regression check, not
/// red/green: the base gives the same. (The one unmarked `\affiliations` line is the first author's, Perl's
/// `labelseq=author`.)
#[test]
fn ijcai_binding_email_list_in_author_order_keeps_perl_order() {
  let got = eager_creators(repro!("ijcai_binding_emails_in_author_order.tex"));
  assert_eq!(got, [
    vec!["Ann Able", "affiliation=Univ X", "email=x1@a.org"],
    vec!["Bob Baker", "email=y2@b.org"],
    vec!["Cid Cole", "email=z3@c.org"],
  ]);
}

/// The ijcai binding's `\author` with a list in no author order: `bob.baker@y.org` first, on Bob Baker by its spelling;
/// `x1@a.org` and `z3@c.org` spell no one and the list is no author-order list, so they stay on the author block (a
/// creator with no name), not on Ann Able and Cid Cole by their place.
#[test]
fn ijcai_binding_email_list_out_of_order_leaves_the_unspelled_on_the_block() {
  assert_creators_on_both_builds(
    repro!("ijcai_emails_out_of_order_stay_on_the_block.tex"),
    &[
      &["Ann Able", "affiliation=Univ X"],
      &["Bob Baker", "email=bob.baker@y.org"],
      &["Cid Cole"],
      &["email=x1@a.org", "email=z3@c.org"],
    ],
  );
}

/// The section read line by line (2002.11711's `…\\zelei.liu@…`, 2609.15094's second `\emails`): a `\\` and a further
/// `\emails` start a line of the title block, so `im@bb.org\\ \{cid\_cole, ddodd\}@z.org` is three addresses and the
/// list four, one per author in their order — im@ Bob Baker's by its place, the others by their spelling; the group is
/// written out with its underscore (2304.11989's `scs$\_$lzg`, which printed the OT1 dot accent); the line with no
/// address is no place of the list and no e-mail: a note on the author block (2609.15094's `$^{*}$Corresponding
/// author` under two starred names), as jheppub's `\note` files its "Corresponding author.". A `\\` inside a printed
/// group parts no line: `\{aable, bbaker,\\ ccole\}@x.edu` is one group of three addresses (review probe p1), and so
/// is `\textbraceleft … \textbraceright` (review of r2, probe i7: Bob Baker held `{aable, bbaker,`).
#[test]
fn ijcai_email_section_lines_part_the_addresses() {
  assert_creators_on_both_builds(repro!("ijcai_emails_lines_and_sections.tex"), &[
    &["Ann Able", "affiliation=Univ X", "email=ann.able@x.org"],
    &["Bob Baker", "email=im@bb.org"],
    &["Cid Cole", "email=cid_cole@z.org"],
    &["Dan Dodd *", "email=ddodd@z.org"],
    &["note=∗ Corresponding author"],
  ]);
  let broken = repro!("ijcai_emails_group_broken_over_lines.tex");
  let written_out: &[&[&str]] = &[
    &["Ann Able", "affiliation=Univ X", "email=aable@x.edu"],
    &["Bob Baker", "email=bbaker@x.edu"],
    &["Cid Cole", "email=ccole@x.edu"],
  ];
  assert_creators_on_both_builds(broken, written_out);
  let text_braces = broken
    .replace("\\{aable", "\\textbraceleft aable")
    .replace("ccole\\}@", "ccole\\textbraceright @");
  assert_eq!(
    (
      broken.matches("\\{aable").count(),
      broken.matches("ccole\\}@").count()
    ),
    (1, 1)
  );
  assert_eq!(eager_creators(&text_braces), written_out);
}

/// 2011.00505's shape (cas-sc): `\address[1]{Univ X}` and `\address[2]{Univ Y}` go to the authors carrying those
/// labels, each an affiliation as cas-common.sty files it — where both went to the last author once an `[orcid=…]`
/// broke the positional reading — and the same without the ORCID.
#[test]
fn cas_address_label_places_it_with_and_without_orcid() {
  let tex = repro!("cas_address_label_keeps_its_author.tex");
  assert_creators_on_both_builds(tex, &[&["Ann Able", "orcid=", "affiliation=Univ X"], &[
    "Bob Baker",
    "affiliation=Univ Y",
  ]]);
  let without = tex.replace("[orcid=0000-0003-0696-521X]", "");
  assert_ne!(without, tex);
  assert_eq!(eager_creators(&without), [
    vec!["Ann Able", "affiliation=Univ X"],
    vec!["Bob Baker", "affiliation=Univ Y"],
  ]);
}

/// cas-dc: letter labels (`\stmLabel`led, cas-common.sty:1230-1236), an author carrying two, an `\affiliation` and two
/// `\address`es of one label; the trailing `[type=alt]` and `[foot=true]` are the `stm/address` keys, typeset nowhere.
/// The `foot` entry, Univ Y, leaves the counter (Univ Z is "b") and is printed at the title's foot under the letter the
/// last main entry printed (`\@currentlabel`, cas-common.sty:1310), Univ Z's "b": it goes with Univ Z.
#[test]
fn cas_dc_address_letter_labels_and_trailing_keys() {
  let (log, xml) = convert_with(
    repro!("cas_dc_address_letter_labels_and_keys.tex"),
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    !xml.contains("type=alt") && !xml.contains("foot=true"),
    "{xml}"
  );
  assert_eq!(creators(&xml), [
    vec!["Ann Able", "affiliation=Univ X, Paris"],
    vec![
      "Bob Baker",
      "affiliation=Univ X, Paris",
      "affiliation=Univ Y",
      "affiliation=Univ Z"
    ],
    vec!["Cid Cole", "affiliation=Univ Y", "affiliation=Univ Z"],
  ]);
}

/// cas's entry counter (`\g_stm_aff_int`, cas-common.sty:1228-1242): a letter-labelled entry steps it and prints its
/// letter, a number entry sets it; an author's number N is the N-th letter (`\processAffNum`), its letter label the
/// letter that entry prints (`\stmRef`, `\lx@label@ref` until the entries are read). Each entry is labelled by the
/// letter it prints (`cas_labelled_entry`, `cas_mark_labels`):
/// - 2207.12381's shape: `\author[1,2]` under `\address[a]…\address[b]` requests both, where the labels were compared
///   as written and the addresses went to no one;
/// - letters written out of order (review probe p7): `\address[b]{Univ X}\address[a]{Univ Y}` prints "a Univ X", "b
///   Univ Y", Univ X `\author[1]`'s, where r1 read each by its written letter and swapped them;
/// - word labels out of order under number marks and a word mark: `\address[aff2]{Univ X}\address[aff1]{Univ Y}`
///   prints "a", "b", so `\author[1]` has Univ X and `\author[2]` and `\author[aff1]` Univ Y;
/// - a number entry before a letter one: `\address[1]{Univ X}` prints "a", the next `\affiliation[a]` steps the
///   counter to "b", which `\author[a]` and `\author[2]` both request.
#[test]
fn cas_entries_are_labelled_by_the_letter_their_counter_prints() {
  assert_eq!(
    eager_creators(repro!("cas_numeric_marks_name_letter_entries.tex")),
    [
      vec!["Ann Able", "affiliation=Univ X", "affiliation=Univ Y"],
      vec!["Bob Baker", "affiliation=Univ Z"],
    ]
  );
  assert_eq!(
    eager_creators(repro!("cas_letter_entries_print_by_the_counter.tex")),
    [vec!["Ann Able", "affiliation=Univ X"], vec![
      "Bob Baker",
      "affiliation=Univ Y"
    ],]
  );
  assert_eq!(
    eager_creators(repro!("cas_word_labels_out_of_order.tex")),
    [
      vec!["Ann Able", "affiliation=Univ X"],
      vec!["Bob Baker", "affiliation=Univ Y"],
      vec!["Cid Cole", "affiliation=Univ Y"],
    ]
  );
  assert_eq!(
    eager_creators(repro!("cas_mixed_number_and_letter_entries.tex")),
    [
      vec!["Ann Able", "affiliation=Univ Y"],
      vec!["Bob Baker", "affiliation=Univ Y"],
      vec!["Cid Cole", "affiliation=Univ X"],
    ]
  );
}

/// Converts `tex` in process under the shadow switch (`LATEXML_FM_SHADOW=1`), checks it converts with no Error or
/// Warning, and returns its XML, the shadow's reading of its creators — each creator's name and the affiliations it
/// gives it, in order (`<name> | <affiliation>; …`), then its contacts (`| <role>=<text>; …`) and its notes (`| notes:
/// <text>; …`) when it gives any, then the block's affiliations (`block | …`) when it holds any — and the dump.
#[track_caller]
fn xml_and_shadow(tex: &str) -> (String, Vec<String>, Value) {
  let (log, xml, json) = convert_with_setup_then(
    tex,
    Some(PRELOAD),
    || set_fm_shadow_override(Some(true)),
    |_| fm_shadow_json(),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let dump: Value = super::perfect_kernel_batchfms2a1::parsed(&json);
  let texts = |list: &Value| -> String {
    list
      .as_array()
      .into_iter()
      .flatten()
      .map(|a| a["text"].as_str().unwrap_or("").to_string())
      .collect::<Vec<_>>()
      .join("; ")
  };
  let resolved = &dump["resolved"];
  let mut shadow: Vec<String> = resolved["creators"]
    .as_array()
    .expect("creators")
    .iter()
    .map(|c| {
      let mut row = format!(
        "{} | {}",
        c["name"].as_str().unwrap_or(""),
        texts(&c["affiliations"])
      );
      let contacts: Vec<String> = c["contacts"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|a| {
          format!(
            "{}={}",
            a["role"].as_str().unwrap_or(""),
            a["text"].as_str().unwrap_or("")
          )
        })
        .collect();
      if !contacts.is_empty() {
        row.push_str(&format!(" | {}", contacts.join("; ")));
      }
      let notes = texts(&c["notes"]);
      if !notes.is_empty() {
        row.push_str(&format!(" | notes: {notes}"));
      }
      row
    })
    .collect();
  let block = texts(&resolved["block"]);
  if !block.is_empty() {
    shadow.push(format!("block | {block}"));
  }
  (xml, shadow, dump)
}

/// A deferred label is read by the queue's readers too (review of 88b r2, finding 1): an author's letter label whose
/// entry comes after it is `\lx@label@ref{…}` until the flush (`cas_mark_labels`), and the readers of the queued
/// keyvals before the digestion — the observer's (`LATEXML_FM_SHADOW=1`) and `place.rs`'s — read the letter it stands
/// for (`resolve_label_refs`), as the digestion expands it; one known when the `\author` is read is written as the
/// letter. Red on r2, whose shadow read the raw reference as single-character requests (2303.00352, 2603.22658: every
/// affiliation gone; 2504.19452, 2509.10496: every author every one):
/// - word labels after the authors (`\author[aff1]` before `\address[aff1]`): Cid Cole has Univ Y;
/// - cas-dc letter marks before their entries, written out of order (review probe q10): Ann Able's `[a]` prints "b",
///   Univ X;
/// - one entry before the authors and one after, with cas's corresponding-author legend (`\cormark`, `\cortext`):
///   Bob Baker's `[b]` written as "a" (Univ Y) when his `\author` is read, Ann Able's `[a]` at the flush; Ann Able's
///   ORCID (its id in the link) and e-mail hers in the XML and the shadow; the legend is a note on the title block, as
///   cas's `\cormark` marks no one in the binding, and in the shadow no creator's;
/// - a label written twice (review of 88b r3, finding 1), the second entry between the authors or both after them:
///   both authors have the first entry, the second is the block's, as pdflatex prints it.
#[test]
fn cas_deferred_labels_are_read_before_digestion() {
  let (xml, shadow, _) = xml_and_shadow(repro!("cas_word_labels_out_of_order.tex"));
  let written = creators(&xml);
  assert_eq!(written, [
    vec!["Ann Able", "affiliation=Univ X"],
    vec!["Bob Baker", "affiliation=Univ Y"],
    vec!["Cid Cole", "affiliation=Univ Y"],
  ]);
  assert_eq!(shadow, [
    "Ann Able | Univ X",
    "Bob Baker | Univ Y",
    "Cid Cole | Univ Y"
  ]);
  let (xml, shadow, _) = xml_and_shadow(repro!("cas_dc_letter_marks_before_their_entries.tex"));
  let written = creators(&xml);
  assert_eq!(written, [vec!["Ann Able", "affiliation=Univ X"], vec![
    "Bob Baker",
    "affiliation=Univ Y"
  ],]);
  assert_eq!(shadow, ["Ann Able | Univ X", "Bob Baker | Univ Y"]);
  let (xml, shadow, dump) =
    xml_and_shadow(repro!("cas_corresponding_mark_on_a_deferred_label.tex"));
  assert!(
    xml.contains("<note role=\"corresponding\" xml:id=\"id2\">Corresponding author</note>"),
    "{xml}"
  );
  let written = creators(&xml);
  assert_eq!(written, [
    vec![
      "Ann Able",
      "orcid=",
      "email=aable@x.edu",
      "affiliation=Univ X"
    ],
    vec!["Bob Baker", "affiliation=Univ Y"],
  ]);
  let ann = &xml[xml.find("<personname>Ann Able").expect("Ann Able")..];
  assert!(
    ann[..ann.find("</creator>").expect("closed creator")].contains(
      "<contact role=\"orcid\"><ref class=\"ltx_orcid\" href=\"https://orcid.org/0000-0002-1825-0097\" \
       title=\"ORCID 0000-0002-1825-0097\">"
    ),
    "{xml}"
  );
  assert_eq!(shadow, [
    "Ann Able | Univ X | orcid=0000-0002-1825-0097; email=aable@x.edu",
    "Bob Baker | Univ Y"
  ]);
  // (the legend is the title block's note in the shadow too: one entry, given to no creator, which the rows show)
  let legends: Vec<&Value> = dump["entries"]
    .as_array()
    .expect("entries")
    .iter()
    .filter(|e| e["tag"] == "ltx:note" && e["role"] == "corresponding")
    .collect();
  assert_eq!(legends.len(), 1, "{dump}");
  assert_eq!(legends[0]["inside"], false, "{dump}");
  assert_eq!(
    legends[0]["events"][0]["text"], "Corresponding author",
    "{dump}"
  );
  // A label written twice keeps its first entry (review of 88b r3, finding 1): `\NewLabel` defines `X@a:` globally
  // only when it is free and LaTeX reads the `.aux` in a group, so both `\author[a]` print the first entry's letter
  // and Univ Y, the second entry, is no one's; the same whether the second entry comes between the authors (Bob
  // Baker's `[a]` written as the letter when his `\author` is read) or both after them (deferred, read at the flush).
  for tex in [
    repro!("cas_first_entry_under_a_label_keeps_it.tex"),
    repro!("cas_first_entry_under_a_deferred_label_keeps_it.tex"),
  ] {
    let (xml, shadow, _) = xml_and_shadow(tex);
    assert_eq!(creators(&xml), [
      vec!["Ann Able", "affiliation=Univ X"],
      vec!["Bob Baker", "affiliation=Univ X"],
      vec!["affiliation=Univ Y"],
    ]);
    assert_eq!(shadow, [
      "Ann Able | Univ X",
      "Bob Baker | Univ X",
      "block | Univ Y"
    ]);
  }
}

/// cas's `\corref{label}` (cas-common.sty:224, a mandatory argument) prints nothing: the label of the `\cortext` it
/// marks is no text, inside the name (2303.00352's `\author[label4]{Enrique García-Macías\corref{cor1}}`, which read
/// "García-Macíascor1") or after the `\author` (a paragraph `cor2`), where the binding's `\corref[]` took no
/// argument. Distinct from the label rows: a mark command's argument, not an entry's label. Both legends stay the
/// title block's notes (the `\corref`/`\cortext` link is not built).
#[test]
fn cas_corref_label_is_no_text() {
  let (log, xml) = convert_with(repro!("cas_corref_label_is_no_text.tex"), Some(PRELOAD));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(creators(&xml), [
    vec!["Ann Able", "affiliation=Univ X"],
    vec!["Bob Baker", "affiliation=Univ X"],
  ]);
  assert!(
    !xml.contains("cor1") && !xml.contains("cor2") && !xml.contains("<para"),
    "{xml}"
  );
  assert!(
    xml.contains("<note role=\"corresponding\" xml:id=\"id1\">Corresponding author</note>")
      && xml
        .contains("<note role=\"corresponding\" xml:id=\"id2\">Second corresponding author</note>"),
    "{xml}"
  );
}
