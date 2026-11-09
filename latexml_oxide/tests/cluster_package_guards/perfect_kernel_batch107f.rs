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
use super::{perfect_kernel_batch99b::converted, perfect_kernel_batcht1::red_at_two};

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

/// A supplement's restated `\institute` puts a second copy of an unlabelled `\email` beside the first on the last
/// author; the copy is no part of an upfront pile of contacts, so the first is not handed to Lv Tang (2009.10916).
/// Distinct: `distribute_upfront_contacts` without the late calls.
#[test]
fn a_restated_supplement_email_is_no_upfront_pile() {
  if red_at_two(
    r#"a false content-check Warning (libraboli@tencent.com placed more often than printed)"#,
  ) {
    return;
  }
  assert_eq!(
    converted(repro!("llncs_supplement_restated_email_not_copied"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Lv Tang</personname> <contact name="Affiliation: " role="affiliation">Univ X, Town</contact> </creator> <creator before=" " role="author"> <personname>Bo Li</personname> <note class="ltx_note_frontmatter ltx_thanks_correspondence" role="thanks" xml:id="id1">Correspondence should be addressed to Bo Li.</note> <contact name="Affiliation: " role="affiliation">Univ Y, City </contact> <contact name="E-mail " role="email"><text font="typewriter" xml:id="id2">luckybird1994@gmail.com</text></contact> <contact name="E-mail " role="email"><text font="typewriter" xml:id="id3">libraboli@tencent.com</text></contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text. Supplement Lv Tang Bo LiCorrespondence should be addressed to Bo Li. More.</p> <pagination role="newpage"/> </para>"##
  );
}

/// llncs's `\email{$\{$aable,bbaker$\}$@x.org}` and `\email{$\lbrace$cat.cole, ddee$\rbrace$@y.org}` print the
/// text-brace lists: an address each, on the author it spells, no nameless creator (1106.2694, 2308.00146, 1602.04934).
/// Distinct: `math_braces_as_printed` in `\lx@add@email`.
#[test]
fn a_math_brace_list_is_the_list_it_prints() {
  if red_at_two(r#"writes the 4 brace-list addresses as plain text, 4 false Warnings"#) {
    return;
  }
  assert_eq!(
    converted(repro!("email_list_llncs_math_braces"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Univ X, Town</contact> <contact name="E-mail " role="email"><text font="typewriter" xml:id="id1">aable@x.org</text></contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Univ X, Town</contact> <contact name="E-mail " role="email"><text font="typewriter" xml:id="id2">bbaker@x.org</text></contact> </creator> <creator before=" " role="author"> <personname>Cat Cole</personname> <contact name="Affiliation: " role="affiliation">Univ Y, City</contact> <contact name="E-mail " role="email"><text font="typewriter" xml:id="id3">cat.cole@y.org</text></contact> </creator> <creator before=" " role="author"> <personname>Dan Dee</personname> <contact name="Affiliation: " role="affiliation">Univ Y, City</contact> <contact name="E-mail " role="email"><text font="typewriter" xml:id="id4">ddee@y.org</text></contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
}

/// A contact printing addresses for several people is parted, each address its owner's e-mail: by the author's name
/// printed beside it (REVTeX's `\email[Corresponding author]{: zx81@x.org (Ann Able); qq7@y.org (Bob Baker)}`,
/// 2009.14655; elsarticle's `Ann Able (zx81@x.org) and Cat Cole (qq7@y.org)`, 2507.07920), else by the placement order
/// among the carriers of a shared corresponding-author note (`cole.c@` spelled, `q77@` by elimination, 2302.03296; a
/// REVTeX `\thanks` list whose unplaced addresses go to the block creator, 2508.07950); the note's other words stay its
/// carriers'. Distinct: `distribute_address_lists`.
#[test]
fn an_address_list_on_a_contact_is_distributed_not_copied() {
  if red_at_two(r#"loses "Ann Able Bob Baker", a content-check Warning"#) {
    return;
  }
  assert_eq!(
    converted(repro!("email_list_revtex_names_after_addresses"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Univ X, Town</contact> <note class="ltx_note_frontmatter ltx_thanks_correspondence" role="thanks" xml:id="id1">Corresponding author</note> <contact name="Email: " role="email">zx81@x.org</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <note class="ltx_note_frontmatter ltx_thanks_correspondence" role="thanks" xml:id="id2">Corresponding author</note> <contact name="Email: " role="email">qq7@y.org</contact> <contact name="Affiliation: " role="affiliation">Univ X, Town</contact> </creator> <creator before=" " role="author"> <personname>Cat Cole</personname> <contact name="Affiliation: " role="affiliation">Univ Y, City</contact> </creator> <abstract inlist="toc" name="Abstract" xml:id="abstract1"> <p xml:id="abstract1.1">A.</p> </abstract> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
  assert_eq!(
    converted(repro!("email_list_elsarticle_names_before_addresses"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Corresponding author: " role="correspondent">Corresponding Authors:</contact> <contact name="Email: " role="email">zx81@x.org</contact> <contact name="Address: " role="address">Univ X, Town</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Address: " role="address">Univ X, Town</contact> </creator> <creator before=" " role="author"> <personname>Cat Cole</personname> <contact name="Corresponding author: " role="correspondent">Corresponding Authors:</contact> <contact name="Email: " role="email">qq7@y.org</contact> <contact name="Address: " role="address">Univ Y, City</contact> </creator> <abstract inlist="toc" name="Abstract" xml:id="abstract1"> <p xml:id="abstract1.1">A.</p> </abstract> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
  assert_eq!(
    converted(repro!("email_list_elsarticle_shared_cortext"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Address: " role="address">Univ X, Town</contact> </creator> <creator before=" " role="author"> <personname>Cat Cole</personname> <contact name="Corresponding author: " role="correspondent">Corresponding authors:</contact> <contact name="Email: " role="email">cole.c@x.org</contact> <contact name="Address: " role="address">Univ X, Town</contact> </creator> <creator before=" " role="author"> <personname>Dan Dee</personname> <contact name="Corresponding author: " role="correspondent">Corresponding authors:</contact> <contact name="Email: " role="email">q77@y.org</contact> <contact name="Address: " role="address">Univ Y, City</contact> </creator> <abstract inlist="toc" name="Abstract" xml:id="abstract1"> <p xml:id="abstract1.1">A.</p> </abstract> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
  assert_eq!(
    converted(repro!("email_list_revtex_shared_thanks"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Univ X, Town</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Thanks: " role="thanks">Corresponding Authors</contact> <contact name="Email: " role="email">bbaker@x.org</contact> <contact name="Affiliation: " role="affiliation">Univ X, Town</contact> </creator> <creator before=" " role="author"> <personname>Cat Cole</personname> <contact name="Thanks: " role="thanks">Corresponding Authors</contact> <contact name="Affiliation: " role="affiliation">Univ Y, City</contact> </creator> <creator role="author"> <contact name="Email: " role="email">zz9@y.org</contact> <contact name="Email: " role="email">q1@z.org</contact> </creator> <abstract inlist="toc" name="Abstract" xml:id="abstract1"> <p xml:id="abstract1.1">A.</p> </abstract> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
}

/// An IEEE blockA nested in another for a list's last line is placed before it; the tail placed just before its head is
/// joined to it and the list read whole (2410.15681). Distinct: `join_split_address_contacts`' backward tail.
#[test]
fn a_split_list_whose_tail_was_placed_first_is_joined() {
  if red_at_two(r#"writes cat.cole@x.org from "cat.cole}@x.org", 2 content-check Warnings"#) {
    return;
  }
  assert_eq!(
    converted(repro!("email_list_ieee_nested_blocka_tail_first"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Univ X, Town</contact> <contact name="Email: " role="email">ann.able@x.org</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Univ X, Town</contact> <contact name="Email: " role="email">bob.baker@x.org</contact> </creator> <creator before=" " role="author"> <personname>Cat Cole</personname> <contact name="Affiliation: " role="affiliation">Univ X, Town</contact> <contact name="Email: " role="email">cat.cole@x.org</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
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
  if red_at_two(r#"writes the role-tagged address line as plain text, a false Warning"#) {
    return;
  }
  assert_eq!(
    converted(repro!("email_role_tag_marked_after_list"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Univ X, Town.</contact> <contact name="Email: " role="email"><text font="typewriter" fontsize="90%" xml:id="id1">aable@x.edu</text></contact> </creator> <creator before=" " role="author"> <personname>Bob Baker<sup xml:id="id2">*</sup></personname> <contact name="Affiliation: " role="affiliation">Univ Y, City.</contact> <note class="ltx_note_frontmatter ltx_thanks_correspondence" mark="*" role="thanks" xml:id="id3"><sup xml:id="id3.1">*</sup>corresponding author</note> <contact name="Email: " role="email"><text font="typewriter" fontsize="90%" xml:id="id4">bakerb@y.edu</text></contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
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

/// A sole author's orphan `* Corresponding author, E-mail: qz@a.org` line is hers, whether or not another orphan line
/// (`$^{b}$ Univ B, City`) makes a block creator beside her (user ruling 2026-11-05, "Sole author"; rev107c2 finding
/// 2). Distinct: the only-author step's skip, now for a nameless creator holding names alone (1102.4967).
#[test]
fn a_sole_authors_orphan_line_is_hers_beside_a_block_creator() {
  if red_at_two(r#"writes the orphan line as plain text, 2 false Warnings"#) {
    return;
  }
  assert_eq!(
    converted(repro!("addr_line_sole_author_orphan_star"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation"><text fontsize="90%" xml:id="id1">Univ A, Town,<break/>Country.</text></contact> <note class="ltx_note_frontmatter ltx_thanks_correspondence" mark="*" role="thanks" xml:id="id2"><sup xml:id="id2.1">*</sup>Corresponding author</note> <contact name="E-mail: " role="email"><text fontsize="90%" xml:id="id3">qz@a.org</text></contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
  assert_eq!(
    converted(repro!("addr_line_sole_author_orphan_star_with_block"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation"><text fontsize="90%" xml:id="id1">Univ A, Town,<break/>Country.</text></contact> <note class="ltx_note_frontmatter ltx_thanks_correspondence" mark="*" role="thanks" xml:id="id2"><sup xml:id="id2.1">*</sup>Corresponding author</note> <contact name="E-mail: " role="email"><text fontsize="90%" xml:id="id3">qz@a.org</text></contact> </creator> <creator role="author"> <contact name="Affiliation: " role="affiliation"><text fontsize="90%" xml:id="id4"> Univ B, City</text></contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
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

/// A sole author's orphan e-mail line labelled in her own mark series (`$^{b}$ E-mail: lab@b.org` beside
/// `Ann Able$^{a}$`) is the block creator's, as its `$^{b}$ Univ B, City` sibling is; a `*` line beside her stays hers
/// (`a_sole_authors_orphan_line_is_hers_beside_a_block_creator`). FIDELITY_CHARTER "Sole author", user 2026-11-05;
/// review rev107f finding 4. Distinct: `insert::mark_series` in the only-author step.
#[test]
fn a_sole_authors_own_series_orphan_line_is_the_blocks() {
  if red_at_two(r#"gives the block's orphan email line to the sole author"#) {
    return;
  }
  assert_eq!(
    converted(
      repro!("addr_line_sole_author_unanswered_mark_email_is_blocks"),
      0
    ),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation"><text fontsize="90%" xml:id="id1">Univ A, Town.</text></contact> </creator> <creator role="author"> <contact name="Affiliation: " role="affiliation"><text fontsize="90%" xml:id="id2"> Univ B, City</text></contact> <contact name="E-mail: " role="email"><text fontsize="90%" xml:id="id3">lab@b.org</text></contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
}
