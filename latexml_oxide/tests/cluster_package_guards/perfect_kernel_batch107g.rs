//! Red/green guards for batch 107g, the Bundle 5 fix round (brief 107g; OXIDIZED_DESIGN_DIVERGENCES #970, #966, #897 "107b",
//! #972 as amended): the real papers worse than int138a in the bundle's 28k A/B, by mechanism, each a narrow rule: (1) an
//! own call moves only to a name whose full-strength score is strictly higher (`email_owners::own_call_keeps`); (2) a
//! four-letter surname inside a word is a weak spelling (`LocalPart::standing`); (3) a list of addresses a mark leads, its
//! siblings or its line naming others, is no carriers' (`insert::link_entry_addresses`' `mark_lists`); (4) a tie and a
//! space are one word space only inside one name (`Lexer::ties_the_gap`); (5) a wrapper under the names over plain lines
//! stays one entry (`separators::wrapper_holds_names_or_marks`). Each guard converts its repros under
//! `tools/perfect_kernel/repros/sectioning-frontmatter/` from the binary, eager and `--streaming`, with 0 errors and 0
//! warnings, and asserts the frontmatter whole.
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

/// An own call moves only to a strictly better full spelling, whichever part of it the holder's key is: a shared surname
/// (0902.4311, 0907.3301, 1008.3764), the head of a longer spelling (rev107f2 finding 1, twice), a misread name's word
/// (2107.06981); a tie keeps it (2511.18493). Distinct: `own_call_keeps`, read by both `respell_emails`.
#[test]
fn an_own_call_moves_only_to_a_strictly_better_full_spelling() {
  assert_eq!(
    converted(
      repro!("email_own_call_longer_spelling_of_a_shared_surname_moves"),
      0
    ),
    r##"<title>T</title> <creator role="author"> <personname>Dongsu Kim</personname> <contact name="Email address, Dongsu Kim : " role="email"><text font="typewriter" xml:id="id1">dongsu.kim@x.kr</text></contact> </creator> <creator before=" and " role="author"> <personname>Jang Soo Kim</personname> <contact name="Address: " role="address">Dept, KAIST</contact> <contact name="Email address, Jang Soo Kim : " role="email"><text font="typewriter" xml:id="id2">jskim@x.kr</text></contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Body.</p> </para>"##
  );
  assert_eq!(
    converted(
      repro!("email_own_call_opening_key_of_longer_spelling_moves"),
      0
    ),
    r##"<title>T</title> <creator role="author"> <personname>Angela Rossi</personname> <contact name="Affiliation: " role="affiliation">Univ X, Town</contact> <contact name="Email: " role="email">angela.rossi@y.edu</contact> </creator> <creator before=" " role="author"> <personname>Lay Kee Ang</personname> <contact name="Affiliation: " role="affiliation">Univ Y, City</contact> </creator> <abstract inlist="toc" name="Abstract" xml:id="abstract1"> <p xml:id="abstract1.1">A.</p> </abstract> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
  assert_eq!(
    converted(
      repro!("email_own_call_opening_key_of_longer_spelling_moves_twin"),
      0
    ),
    r##"<title>T</title> <creator role="author"> <personname>Joe Bennett</personname> <contact name="Affiliation: " role="affiliation">Univ X, Town</contact> <contact name="Email: " role="email">bennett@y.edu</contact> </creator> <creator before=" " role="author"> <personname>Ben Smith</personname> <contact name="Affiliation: " role="affiliation">Univ Y, City</contact> </creator> <abstract inlist="toc" name="Abstract" xml:id="abstract1"> <p xml:id="abstract1.1">A.</p> </abstract> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
}

/// A list a mark leads is no carriers' where a sibling spells a non-carrier (2412.13509) or the line names one (2102.08716,
/// spelled addresses out of the list's order, so the unspelled stay on the block as on base: rev108a3 B3); siblings spelling carriers leave the free carrier; a lone address stays every
/// carrier's (user ruling 2026-10-30; batch93a, batch100b pin it, this file's fixture for it now reads by numbers). Distinct: `link_entry_addresses`' `marked` and `list_placements`' `mark_lists`.
/// (Since 108a r2 `\textsuperscript{*1}` asks for 1 and shows its star, so the first and third shapes' entries are cut by
/// their marks, each name its own, and `bmfu@`, under no entry of Bob Fu's, is the block's, as `\textsuperscript{1}` read on
/// int139c: 2412.13509's `hmfu@`, open in SYNC 108a.)
#[test]
fn a_list_a_mark_leads_is_no_carriers_where_it_spells_others() {
  assert_eq!(
    converted(
      repro!("email_list_mark_line_siblings_spell_others_is_no_carriers"),
      0
    ),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able<sup xml:id="id1">*</sup><sup xml:id="id2">1</sup></personname> <contact name="Affiliation: " role="affiliation"><text class="ltx_affiliation_institution" xml:id="id3"><sup xml:id="id3.1">1</sup>Univ A</text></contact> <contact name="Email: " role="email">aable@a.edu</contact> </creator> <creator before=", " role="author"> <personname>Dan Dorn<sup xml:id="id4">*</sup><sup xml:id="id5">2</sup></personname> <contact name="Affiliation: " role="affiliation"><text class="ltx_affiliation_institution" xml:id="id6"><sup xml:id="id6.1">2</sup>Univ B</text></contact> <contact name="Email: " role="email">ddorn@b.edu</contact> </creator> <creator before=", " role="author"> <personname>Bob Fu<sup xml:id="id7">1</sup></personname> <contact name="Affiliation: " role="affiliation"><text class="ltx_affiliation_institution" xml:id="id8"><sup xml:id="id8.1">1</sup>Univ A</text></contact> </creator> <creator before=" and " role="author"> <personname>Cat Cole<sup xml:id="id9">†</sup><sup xml:id="id10">1</sup></personname> <contact name="Affiliation: " role="affiliation"><text class="ltx_affiliation_institution" xml:id="id11"><sup xml:id="id11.1">1</sup>Univ A</text></contact> <contact name="Email: " role="email">ccole@a.edu</contact> </creator> <creator role="author"> <contact name="Email: " role="email">bmfu@a.edu</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Body.</p> </para>"##
  );
}

/// A paper repeating its author block for a supplement copies no address onto the last creator (2307.07324, 1801.07635,
/// 1810.03867, 2510.13035, 2001.07542, 2608.31160; the bundle int139b A/B). Since RB1 REVTeX's `\maketitle` switches the
/// second block off from its `\author` on (revtex4_cls.rs `switch_off_author_block_after_maketitle`;
/// revtex4-2.cls:2670-2684), so its names no longer print as body text; its `\title` does, as int159b (RB1c: authored
/// text kept, though pdflatex prints none of it), and this REVTeX repro no longer reaches `respell_emails`' `held` branch (a copy the owner already holds dropped), which a subfile's block
/// (2307.07324, 2401.15572) and other classes' (1810.03867, bmvc2k) still reach.
#[test]
fn a_repeated_author_block_copies_no_address_onto_the_last_author() {
  assert_eq!(
    converted(repro!("email_repeated_author_block_for_a_supplement"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Email: " role="email">ann.able@x.org</contact> <contact name="Affiliation: " role="affiliation">Univ X, Town</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Email: " role="email">bob.baker@y.org</contact> <contact name="Affiliation: " role="affiliation">Univ Y, City</contact> </creator> <abstract inlist="toc" name="Abstract" xml:id="abstract1"> <p xml:id="abstract1.1">A.</p> </abstract> <para xml:id="p1"> <p xml:id="p1.1">Text. Supplement More.</p> <pagination role="newpage"/> </para>"##
  );
}
