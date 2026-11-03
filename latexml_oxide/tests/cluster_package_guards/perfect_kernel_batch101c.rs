//! Red/green guard for batch 101c: the re-part's fail-safe for a person in a row an item joins
//! (`observer/writer/repart.rs` `joined_rows`, batch 100c review rev100c B1, OXIDIZED_DESIGN_DIVERGENCES #959) exempts
//! the first row of an item a mark labels — the line the mark makes the entry, as the continued-lines check already
//! did. Its words may name a person ("Hellebore Capital Management") and the item is still the affiliation: 1507.08137's
//! four authors were one creator and a `merged_creators` Error since 100c r2. The fail-safe's own guard,
//! `perfect_kernel_batch100c::a_person_in_a_row_an_item_joins_is_never_a_contact` (2609.37429), stays: a person in a
//! later row still declines.
use super::{
  convert_with,
  perfect_kernel_batch46::{error_count, warning_count},
};

/// A marked IEEEtran conference block (1507.08137's shape): names with `{\small $~^{\#*}$}` marks, then `$^{\#}$\,…`
/// and `$^{*}$\,…` affiliations over two lines each under an `\itshape`. Each author holds the affiliations its marks
/// request, whole, and no affiliation keeps a `∗` (red before: one creator, the names and the `#` line merged, 1 Error).
#[test]
fn a_marked_entrys_first_row_that_reads_as_a_name_stays_the_entry() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/marked_affiliation_first_row_reads_as_a_name.tex"
    ),
    None,
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let xml = xml.split_whitespace().collect::<Vec<_>>().join(" ");
  let start = xml.find("<creator").expect("a creator");
  let end = xml.rfind("</creator>").expect("a closed creator") + "</creator>".len();
  let hcm = r#"<contact name="Affiliation: " role="affiliation">Hellebore Capital Management<break/>63 Avenue des Champs, Paris</contact>"#;
  let lix = r#"<contact name="Affiliation: " role="affiliation">Laboratoire d’Informatique<break/>LIX, Palaiseau</contact>"#;
  assert_eq!(
    &xml[start..end],
    format!(
      r#"<creator role="author"> <personname>Ann Able</personname> {hcm} {lix} </creator> <creator role="author"> <personname>Bob Baker</personname> {hcm} </creator> <creator role="author"> <personname>Cat Cole</personname> {lix} </creator>"#
    )
  );
}
