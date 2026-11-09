//! Red/green guards for batch 107f (census138 ADDR-b and EMAILSPELL, 107c's open items; Bundle 5;
//! OXIDIZED_DESIGN_DIVERGENCES #966, #970, #805 amended, #972). E-mail ownership: (1) a name spells an address only
//! where the one boundary predicate places its key — a word, a word's prefix or suffix, between the name's own
//! initials; a weak spelling (a long key between other letters, a joined key ending a word it does not open) never
//! decides between names (`email_owners::LocalPart::standing`, `spelled_owners`) —, and an author's own call keeps an
//! address only a key of its own name is a whole word of or opens (`email_owners::own_call_keeps`, both
//! `respell_emails`; round 2, review rev107f B1 and finding 3); (2) a restated supplement's e-mail is no
//! upfront pile (`insert::distribute_upfront_contacts`); (3) a list in math braces is the list it prints
//! (`email_owners::math_braces_as_printed`); (4) a contact printing a list of addresses for several people is parted
//! into each address's e-mail on its owner — the name printed beside it, else the placement order
//! (`address_lists::distribute_address_lists`); (5) a split list whose tail was placed before its head is joined
//! (`insert::join_split_address_contacts`); (6) a role tag after an address is its owner's correspondence note
//! (`insert::role_tag`); (7) a held correspondence note keeps the e-mail's label (`insert::write_role_notes`); (8) a sole
//! author's orphan line is hers beside a block creator (`insert::link_entry_addresses`, user ruling 2026-11-05 "Sole
//! author") — unless its mark is in her own mark series (`insert::mark_series`, round 2, review rev107f finding 4); (9)
//! a shared note's addresses are cut out of it in place, its other nodes kept (`address_lists::cut_printed`, round 2,
//! review rev107f B2). Each guard converts its repro under `tools/perfect_kernel/repros/sectioning-frontmatter/` from the binary,
//! eager and `--streaming`, with 0 errors and 0 warnings, and asserts the frontmatter whole.
use super::perfect_kernel_batch99b::converted;

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $name,
      ".tex"
    ))
  };
}

/// A surname spells an address only as a run of its local part's words: REVTeX's `\author{Lay Kee Ang}\email{ricky\_ang@…}`
/// is Ang's, not Shengyuan A. Yang's ("yang" across the `_`, 2604.02636), and Bo Li's `bangkok@` spells no Ang ("ang"
/// strictly inside the word), so each own call keeps its address. Distinct: the word-boundary rule of `spelled_owners`.
#[test]
fn an_address_spells_only_a_run_of_its_words() {
  assert_eq!(
    converted(repro!("email_spell_surname_inside_another_word"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Shengyuan A. Yang</personname> <contact name="Email: " role="email">shengyuan.yang@x.edu</contact> <contact name="Affiliation: " role="affiliation">Univ X, Town</contact> </creator> <creator before=" " role="author"> <personname>Lay Kee Ang</personname> <contact name="Email: " role="email">ricky_ang@y.edu</contact> <contact name="Affiliation: " role="affiliation">Univ Y, City</contact> </creator> <creator before=" " role="author"> <personname>Bo Li</personname> <contact name="Email: " role="email">bangkok@z.org</contact> <contact name="Affiliation: " role="affiliation">Univ Z, Place</contact> </creator> <abstract inlist="toc" name="Abstract" xml:id="abstract1"> <p xml:id="abstract1.1">A.</p> </abstract> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
}

/// An author's own call keeps the address its own name spells, though another author spells it better: amsart's
/// `\author{Yang Zhang}…\email{yang91@…}` beside Hengyun Yang (1707.09730, FIDELITY_CHARTER §4 #420). Distinct: the
/// holder's own spelling in both `respell_emails` (`name_spells`).
#[test]
fn an_own_call_keeps_an_address_its_name_spells() {
  assert_eq!(
    converted(repro!("email_spell_own_call_name_also_matches"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Hengyun Yang</personname> <contact name="Address: " role="address">Univ A, Town</contact> <contact name="Email address: " role="email"><text font="typewriter" xml:id="id1">hyyang@a.edu</text></contact> </creator> <creator before=" and " role="author"> <personname>Yang Zhang</personname> <contact name="Address: " role="address">Univ B, City</contact> <contact name="Email address: " role="email"><text font="typewriter" xml:id="id2">yang91@b.edu</text></contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
}

/// A role tag after an address (`Email: dkdee@y.org \textbf{(Corresponding Author)}`, 2503.14354; `aable@x.edu,
/// bakerb@y.edu (* corresponding author)`, 2210.14319) is the last address's owner's correspondence note, with its
/// mark; the e-mail's text the address alone. Distinct: `role_tag` in the Kind step for an address line.
#[test]
fn a_role_tag_after_an_address_is_its_owners_note() {
  assert_eq!(
    converted(repro!("email_role_tag_after_address_ieee"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able<sup xml:id="id1">*</sup></personname> <contact name="Affiliation: " role="affiliation">Univ X, Town</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker<sup xml:id="id2">*</sup></personname> <contact name="Affiliation: " role="affiliation">Univ X, Town</contact> </creator> <creator before=" " role="author"> <personname>Dan Kumar Dee<sup xml:id="id3">†</sup></personname> <contact name="Affiliation: " role="affiliation">Univ Y, City</contact> <note class="ltx_note_frontmatter ltx_thanks_correspondence" role="thanks" xml:id="id4">Corresponding Author</note> <contact name="Email: " role="email">dkdee@y.org</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
}

/// An author holding a correspondence `\thanks` keeps it, and a "Corresponding author: aable@a.edu" line's e-mail keeps
/// its label as printed (rev107c2 finding 1). Distinct: `write_role_notes`' note check before the label is cut.
#[test]
fn a_held_correspondence_note_keeps_the_email_label() {
  assert_eq!(
    converted(repro!("email_label_correspondence_beside_thanks_note"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able</personname> <note class="ltx_note_frontmatter ltx_thanks_correspondence" role="thanks" xml:id="id1">To whom correspondence should be addressed.</note> <contact name="Affiliation: " role="affiliation">Univ A, Town</contact> <contact name="Corresponding author: " role="email">aable@a.edu</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Univ A, Town</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
}

/// A joined spelling ending a word it does not open is weak: Yi Wang's "ywang" in `xywang@` never breaks the tie
/// "wang" makes with Xiaoyu Wang, so the elimination gives `xywang@` to Xiaoyu Wang once `yiwang@` is Yi Wang's
/// (review rev107f B1; 2510.24320's `xjhuang@`, which Jixuan Huang's "jhuang" took from Xuanjing Huang). Distinct: the
/// weak standing of `LocalPart::joined_standing` in `spelled_owners`' two rankings.
#[test]
fn a_joined_suffix_spelling_never_breaks_a_surname_tie() {
  assert_eq!(
    converted(repro!("email_spell_joined_suffix_breaks_surname_tie"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Yi Wang</personname> <contact name="Affiliation: " role="affiliation">Univ. A</contact> <contact name="Email: " role="email"><text font="typewriter" xml:id="id1">yiwang@a.edu</text></contact> </creator> <creator before=" " role="author"> <personname>Xiaoyu Wang</personname> <contact name="Affiliation: " role="affiliation">Univ. A</contact> <contact name="Email: " role="email"><text font="typewriter" xml:id="id2">xywang@a.edu</text></contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Body.</p> </para>"##
  );
}

/// An author's own call keeps an address another name spells better only where a key of the holder's name is a whole
/// word of the local part or opens one (FIDELITY_CHARTER §4 #420): Mark Anderson's "anderson" is the tail of
/// `sanderson@`, Peter Andersen's "anders" a truncation of `anderson@`, so each moves to the author it spells whole
/// (review rev107f finding 3). Distinct: `own_call_keeps` in both `respell_emails`.
#[test]
fn an_own_call_keeps_only_an_address_its_name_opens() {
  assert_eq!(
    converted(repro!("email_own_call_suffix_spelling_moves"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Lee Sanderson</personname> <contact name="Affiliation: " role="affiliation">Univ X, Town</contact> <contact name="Email: " role="email">sanderson@y.edu</contact> </creator> <creator before=" " role="author"> <personname>Mark Anderson</personname> <contact name="Affiliation: " role="affiliation">Univ Y, City</contact> </creator> <abstract inlist="toc" name="Abstract" xml:id="abstract1"> <p xml:id="abstract1.1">A.</p> </abstract> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
  assert_eq!(
    converted(repro!("email_own_call_truncated_surname_moves"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Maria Anderson</personname> <contact name="Affiliation: " role="affiliation">Univ X, Town</contact> <contact name="Email: " role="email">anderson@y.edu</contact> </creator> <creator before=" " role="author"> <personname>Peter Andersen</personname> <contact name="Affiliation: " role="affiliation">Univ Y, City</contact> </creator> <abstract inlist="toc" name="Abstract" xml:id="abstract1"> <p xml:id="abstract1.1">A.</p> </abstract> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
}

/// A shared corresponding-author note whose addresses are distributed keeps every other node in place — its words
/// ("and" and "or" outside the list), its math — only the address spans and the separators beside them cut (review
/// rev107f B2; 2311.15476's DOI link and break). Distinct: `address_lists::cut_printed`.
#[test]
fn a_shared_note_keeps_its_other_nodes_as_printed() {
  assert_eq!(
    converted(repro!("email_list_shared_note_keeps_prose_and_markup"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Address: " role="address">Univ X, Town</contact> </creator> <creator before=" " role="author"> <personname>Cat Cole</personname> <contact name="Corresponding author: " role="correspondent">Corresponding authors: Funded by NSF and DOE or ERC, grant <Math mode="inline" tex="N^{2}" text="N ^ 2" xml:id="m1a"> <XMath xml:id="m1a.1"> <XMApp xml:id="m1a.1.1"> <XMTok role="SUPERSCRIPTOP" scriptpos="post1"/> <XMTok font="italic" role="UNKNOWN">N</XMTok> <XMTok fontsize="70%" meaning="2" role="NUMBER">2</XMTok> </XMApp> </XMath> </Math>.</contact> <contact name="Email: " role="email">cole.c@x.org</contact> <contact name="Address: " role="address">Univ X, Town</contact> </creator> <creator before=" " role="author"> <personname>Dan Dee</personname> <contact name="Corresponding author: " role="correspondent">Corresponding authors: Funded by NSF and DOE or ERC, grant <Math mode="inline" tex="N^{2}" text="N ^ 2" xml:id="m1b"> <XMath xml:id="m1b.1"> <XMApp xml:id="m1b.1.1"> <XMTok role="SUPERSCRIPTOP" scriptpos="post1"/> <XMTok font="italic" role="UNKNOWN">N</XMTok> <XMTok fontsize="70%" meaning="2" role="NUMBER">2</XMTok> </XMApp> </XMath> </Math>.</contact> <contact name="Email: " role="email">q77@y.org</contact> <contact name="Address: " role="address">Univ Y, City</contact> </creator> <abstract inlist="toc" name="Abstract" xml:id="abstract1"> <p xml:id="abstract1.1">A.</p> </abstract> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
}
