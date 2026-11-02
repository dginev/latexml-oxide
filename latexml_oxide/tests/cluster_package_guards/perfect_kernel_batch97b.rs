//! Red/green guards for batch 97b: a take-out removes what it read — the item's value and the label it took — and
//! nothing else TeX prints (OXIDIZED_DESIGN_DIVERGENCES #944). aa's `\mail` is labelled as the paper's copy of aa.cls
//! prints it (aa_support_sty.rs), and the writer's take-out of an entry it empties keeps the entry's own label
//! (writer/plan.rs `holder_label`) or gives the entry back when no contact would carry it (writer/mod.rs
//! `give_back_entries`); a K11 store's label is read before its printed occurrence, not inside an argument TeX does not
//! typeset (frontmatter_stores.rs `in_unprinted_argument`, `argument_of`), and leads a note's text; a names line split
//! by its marks keeps the separators it prints in `before` (names_split.rs `with_printed_separators`), and a piece
//! that is an address and no name is no marked name. Each guard converts its repro under
//! `tools/perfect_kernel/repros/sectioning-frontmatter/` from the binary, eager and `--streaming`, asserts the same XML
//! both ways, the exact Error and Warning counts, and the frontmatter whole.
use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch89a::cli_conversion,
};

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $name,
      ".tex"
    ))
  };
}

/// The XML with every whitespace run collapsed to one space.
fn squash(xml: &str) -> String { xml.split_whitespace().collect::<Vec<_>>().join(" ") }

/// Converts `tex` from the binary eager and `--streaming`, checks each has `errors` Errors and no Warning and both give
/// the same XML, and returns the log and the document from its `<title>` to its end, squashed.
#[track_caller]
fn both_ways(tex: &str, errors: usize) -> (String, String) {
  let (log, eager) = cli_conversion(tex, false);
  assert_eq!(
    (error_count(&log), warning_count(&log)),
    (errors, 0),
    "{log}"
  );
  let (streamed_log, streamed) = cli_conversion(tex, true);
  assert_eq!(
    (error_count(&streamed_log), warning_count(&streamed_log)),
    (errors, 0),
    "{streamed_log}"
  );
  assert_eq!(eager, streamed);
  let xml = squash(&eager);
  let start = xml.find("<title>").unwrap_or(0);
  let end = xml[start..]
    .find("</document>")
    .map_or(xml.len(), |end| start + end);
  (log, xml[start..end].trim_end().to_string())
}

/// aa's `\mail{x}` under a copy of aa.cls that prints it ("Correspondence to: x", v4.03-v7.0) is one e-mail contact
/// named by the class's label, where 95c's take-out of the address entry named it "Email: " and printed
/// "Correspondence" nowhere (the writer keeps the emptied entry's own label, `holder_label`), the label's italic on no
/// item after it (`\mailname`'s group); under a copy that only
/// stores it (v8 on, which prints neither), the binding gives it no label (`name={}`: no `\mailname`, no italic) and the
/// contact has the class's e-mail default, as before. Witnesses 1005.3784, 0805.1001, 1102.1556 (printing); 2007.14739,
/// 2012.11607, 2606.29508 (storing).
#[test]
fn aa_mail_is_labelled_as_its_copy_prints_it() {
  let (_, xml) = both_ways(repro!("aa_mail_printing_copy_keeps_its_label"), 0);
  assert_eq!(
    xml,
    r#"<title>T</title> <creator role="author"> <personname>A. Smith</personname> <contact name="Affiliation: " role="affiliation">Univ. X, Y</contact> <contact name="Correspondence to " role="email">smith@x.org</contact> </creator> <date role="creation">Received 2010</date> <para xml:id="p1"> <p xml:id="p1.1">Body.</p> </para>"#
  );
  let (_, xml) = both_ways(repro!("aa_mail_storing_copy_has_no_label"), 0);
  assert_eq!(
    xml,
    r#"<title>T</title> <creator role="author"> <personname>A. Smith</personname> <contact name="Affiliation: " role="affiliation">Univ. X, Y</contact> <contact name="Email: " role="email">smith@x.org</contact> </creator> <date role="creation">Received 2010</date> <para xml:id="p1"> <p xml:id="p1.1">Body.</p> </para>"#
  );
}

/// The `\mail` entry whose address the institute prints too (`\email`): its take-out would make no contact, the address
/// already one, so its label would be printed nowhere — the writer gives the entry back whole, beside the e-mail
/// contact, as int123a held them (`give_back_entries`). Witnesses 1005.3784, astro-ph0107243. Since 99a r2 the
/// institute goes before the `\mail` contact queued after it (R8: `relocate::at_queue_position`, OD #954; it had
/// followed it); pdflatex prints the "Correspondence to" footnote after the institute's e-mail too, which the placement
/// by role does not reach: `\mail` is declared the footer's (`_zone=footer`, 99a r3), but print zones reorder only a
/// relocated entry, and both contacts here are attached when digested (SYNC_STATUS row 99a). Since 100b (OD #956) the
/// `\mail` entry, an address alone, is kinded at the API as an e-mail named by the class's label: the same contact, its
/// role `email`, "Correspondence to" kept.
#[test]
fn aa_mail_entry_whose_address_is_a_contact_is_given_back() {
  let (_, xml) = both_ways(repro!("aa_mail_repeated_address_stays_labelled"), 0);
  assert_eq!(
    xml,
    r#"<title>T</title> <creator role="author"> <personname>A. Smith</personname> <contact name="Affiliation: " role="affiliation">Univ. X, Y</contact> <contact name="Correspondence to " role="email">smith@x.org</contact> <contact name="Email: " role="email"><text font="typewriter" xml:id="id1">smith@x.org</text></contact> </creator> <date role="creation">Received 2010</date> <para xml:id="p1"> <p xml:id="p1.1">Body.</p> </para>"#
  );
}

/// A raw package's e-mail store printed as `Correspondence: \href{mailto:\@x}{\@x}` in its emptiness test's `\else`
/// branch: the label read before the link's text — not its target, which TeX does not typeset — leads the note's text,
/// where the test's take-out had printed "Correspondence" nowhere; the same through `\textcolor{black}{…}` round the
/// link's text (2506.10896's acl2023.sty:186) and with the store printed bare. Without a label the note is the table's
/// unnamed one.
#[test]
fn note_store_label_is_read_before_its_printed_occurrence() {
  let tex = repro!("raw_package_note_label_before_a_link_names_it");
  let labelled = r#"<title>T</title> <creator role="author"> <personname>Ann Able</personname> </creator> <note role="email" xml:id="id1">Correspondence: ann@x.org</note> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#;
  assert_eq!(both_ways(tex, 0).1, labelled);
  let link = r"\href{mailto:\@corremail}{\@corremail}";
  let coloured = tex.replace(
    link,
    r"\href{mailto:\@corremail}{\textcolor{black}{\@corremail}}",
  );
  assert_ne!(coloured, tex);
  assert_eq!(both_ways(&coloured, 0).1, labelled);
  let bare = tex.replace(link, r"\@corremail");
  assert_ne!(bare, tex);
  assert_eq!(both_ways(&bare, 0).1, labelled);
  let unlabelled = tex.replace(r"\itshape Correspondence: \href", r"\itshape \href");
  assert_ne!(unlabelled, tex);
  assert_eq!(
    both_ways(&unlabelled, 0).1,
    labelled.replace("Correspondence: ann@x.org", "ann@x.org")
  );
}

/// A revtex names line split by its marks keeps the separators it prints: the second creator's `before` ", ", the
/// third's " and " — the binding's space had carried neither, the word "and" printed nowhere. Witness hep-ph9710228.
#[test]
fn marked_list_split_keeps_its_printed_separators() {
  let (_, xml) = both_ways(repro!("author_marked_list_keeps_its_printed_separators"), 0);
  assert_eq!(
    xml,
    r#"<title>T</title> <creator role="author"> <personname>J. P. B. C. de Melo<sup xml:id="id1"><text font="italic" xml:id="id1.1">a</text></sup></personname> <contact name="Affiliation: " role="affiliation">Univ A</contact> </creator> <creator before=", " role="author"> <personname>H. W. L. Naus<sup xml:id="id2"><text font="italic" xml:id="id2.1">b</text></sup></personname> <contact name="Affiliation: " role="affiliation">Univ B</contact> </creator> <creator before=" and " role="author"> <personname>T. Frederico<sup xml:id="id3"><text font="italic" xml:id="id3.1">c</text></sup></personname> <contact name="Affiliation: " role="affiliation">Univ C</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#
  );
}

/// A marked piece that is an address and no name is no marked name (review rev97a S1, synthetic): the line is no list by
/// its marks — not a creator `b.baker@two.edu`. Since batch 100c the observer's writer writes the block as the reading
/// reads it, two people and the address the second's by its spelling (OXIDIZED_DESIGN_DIVERGENCES #958, `Made::Email`),
/// no longer one creator holding the line with its two marked names reported (`merged_creators`, the fail-safe while the
/// re-part could make no e-mail item). A marked name with its address beside it is a name (imsart's `\ead`,
/// 2107.03920; emulateapj's `\authoremail`, astro-ph9812176, which the first reading of this rule merged): the list
/// splits.
#[test]
fn marked_address_piece_is_no_person() {
  let (_, xml) = both_ways(repro!("author_marked_email_piece_is_no_person"), 0);
  assert_eq!(
    xml,
    r#"<title>A Title</title> <creator role="author"> <personname>A. Able</personname> <contact name="Affiliation: " role="affiliation">Institute of One</contact> </creator> <creator role="author"> <personname>B. in der Baker<sup xml:id="id1">2,∗</sup></personname> <contact name="Email: " role="email">b.baker@two.edu</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Body text.</p> </para>"#
  );
  let tex = repro!("author_marked_email_piece_is_no_person");
  let beside = tex.replace(
    r"b.baker@two.edu$^{*}$",
    r"C.~Cole$^{1}$\thanks{c.cole@one.edu}",
  );
  assert_ne!(beside, tex);
  let (_, xml) = both_ways(&beside, 0);
  assert_eq!(
    xml,
    r#"<title>A Title</title> <creator role="author"> <personname>A. Able<sup xml:id="id1"><text font="italic" xml:id="id1.1">1</text></sup></personname> <contact name="Affiliation: " role="affiliation">Institute of One</contact> </creator> <creator before=", " role="author"> <personname>B. in der Baker<sup xml:id="id2"><text font="italic" xml:id="id2.1">2</text></sup></personname> </creator> <creator before=", " role="author"> <personname>C. Cole<sup xml:id="id3"><text font="italic" xml:id="id3.1">1</text></sup></personname> <note class="ltx_note_frontmatter ltx_thanks_note" role="thanks" xml:id="id4">c.cole@one.edu</note> <contact name="Affiliation: " role="affiliation">Institute of One</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Body text.</p> </para>"#
  );
}
