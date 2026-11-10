//! Guards for S5 (review revZ18c S5, revZ18d): the corresponding author is decided once, on the reading
//! (`Resolver::corresponding`, by a note or a legend), and the Link step of an entry's addresses
//! (`insert::link_entry_addresses`) carries that decision out through the creator's transient `_corresponding`, written by
//! the emitter and dropped at output. Each converts its repro and asserts every creator's name and contacts in order and
//! the exact Error and Warning counts.

use latexml::util::test::{error_count, warning_count};

use super::{perfect_kernel_batch84b::creators, perfect_kernel_batchz4::run};

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/class-bindings/",
      $path
    ))
  };
}

/// `tex`'s creators, converted clean, with no transient `_corresponding` left in the output and no unknown key reported.
fn clean_creators(tex: &str) -> Vec<Vec<String>> {
  let (log, xml) = run(tex);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(!xml.contains("_corresponding"), "{xml}");
  // (the key is declared: an unknown key's report changed the paragraphs after 1512.06325's title page)
  assert!(!log.contains("unknown KeyVals key"), "{log}");
  creators(&xml)
}

fn rows(rows: &[&[&str]]) -> Vec<Vec<String>> {
  rows
    .iter()
    .map(|row| row.iter().map(|cell| cell.to_string()).collect())
    .collect()
}

/// `\author{Ann Able\inst{1,2} \and Bob Baker\inst{1}}\institute{Univ X, Town\\ \email{zz9@x.edu} \and Corresponding
/// author}`: institute 2's legend marks Ann Able the corresponding author, so the one address left of the entry's two
/// carriers is hers, as a `\thanks{Corresponding author}` makes it (85b's `llncs_corresponding_author_takes_the_left_address`).
/// The same legend requested by Bob Baker alone makes it his; requested by both, no one is marked and it stays the block's.
/// Red (Z18d, int154u): zz9@ on the block — the Link step re-read the corresponding author from the DOM's notes alone.
#[test]
fn a_legend_corresponding_author_takes_the_left_address() {
  let tex = repro!("llncs_legend_corresponding_author_takes_the_left_address.tex");
  assert_eq!(
    clean_creators(tex),
    rows(&[
      &[
        "Ann Able",
        "affiliation=Univ X, Town",
        "email=zz9@x.edu",
        "affiliation=Corresponding author",
      ],
      &["Bob Baker", "affiliation=Univ X, Town"],
    ])
  );
  let bobs = tex.replace(
    r"\author{Ann Able\inst{1,2} \and Bob Baker\inst{1}}",
    r"\author{Ann Able\inst{1} \and Bob Baker\inst{1,2}}",
  );
  assert_ne!(bobs, tex);
  assert_eq!(
    clean_creators(&bobs),
    rows(&[&["Ann Able", "affiliation=Univ X, Town"], &[
      "Bob Baker",
      "affiliation=Univ X, Town",
      "email=zz9@x.edu",
      "affiliation=Corresponding author",
    ],])
  );
  let both = tex.replace(
    r"\author{Ann Able\inst{1,2} \and Bob Baker\inst{1}}",
    r"\author{Ann Able\inst{1,2} \and Bob Baker\inst{1,2}}",
  );
  assert_ne!(both, tex);
  assert_eq!(
    clean_creators(&both),
    rows(&[
      &[
        "Ann Able",
        "affiliation=Univ X, Town",
        "affiliation=Corresponding author",
      ],
      &[
        "Bob Baker",
        "affiliation=Univ X, Town",
        "affiliation=Corresponding author",
      ],
      &["email=zz9@x.edu"],
    ])
  );
}
