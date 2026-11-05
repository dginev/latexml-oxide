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
/// (2107.06981); a tie keeps it (2511.18493); one printed under both stays on both (2502.15891). Distinct: `own_call_keeps`, read by both `respell_emails`.
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
  assert_eq!(
    converted(repro!("email_own_call_misread_name_word_gives_way"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Filippo Neri</personname> <contact name="Email: " role="email">filippo.neri.email@gmail.com</contact> </creator> <creator before=" " role="author"> <personname>DIETI</personname> </creator> <creator before=" " role="author"> <personname>University of Naples</personname> </creator> <creator before=" " role="author"> <personname>Italy. email</personname> </creator> <para xml:id="p1"> <p xml:id="p1.1">Body.</p> </para>"##
  );
  assert_eq!(
    converted(repro!("email_own_call_printed_under_both_stays_on_both"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Deborah Oliveira</personname> <contact name="Address: " role="address">IMPA, Rio</contact> <contact name="Email address: " role="email"><text font="typewriter" xml:id="id1">deborah.oliveira@impa.br</text></contact> </creator> <creator before=" and " role="author"> <personname>Roberto Oliveira</personname> <contact name="Address: " role="address">IMPA, Rio</contact> <contact name="Email address: " role="email"><text font="typewriter" xml:id="id2">deborah.oliveira@impa.br</text></contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Body.</p> </para>"##
  );
  assert_eq!(
    converted(repro!("email_own_call_tie_keeps_the_call"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Anh-Minh Phan</personname> <contact name="Affiliation: " role="affiliation">Univ X, Town</contact> </creator> <creator before=" " role="author"> <personname>Nhat Ho</personname> <contact name="Email: " role="email">minhnnhat@y.edu</contact> <contact name="Affiliation: " role="affiliation">Univ Y, City</contact> </creator> <abstract inlist="toc" name="Abstract" xml:id="abstract1"> <p xml:id="abstract1.1">A.</p> </abstract> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
}

/// A four-letter surname inside a word spells weakly: it owns an address no other name spells, after two letters at most,
/// whatever follows (2208.05114's `ylwanguestc@`), and set after the last author (2004.06253's `tpengqy@`); it moves no other
/// own call (2411.01273's `yhsheng@`). Distinct: `LocalPart::standing`'s threshold and `own_call_keeps`' `last`.
#[test]
fn a_weak_four_letter_surname_owns_only_an_unclaimed_address() {
  assert_eq!(
    converted(
      repro!("email_weak_four_letter_surname_inside_a_word_after_the_last_author"),
      0
    ),
    r##"<title>T</title> <creator role="author"> <personname>Q. Y. Peng</personname> <contact name="Affiliation: " role="affiliation">Univ X, Town</contact> <contact name="Email: " role="email">tpengqy@x.cn</contact> </creator> <creator before=" " role="author"> <personname>Y. J. Shang</personname> <contact name="Affiliation: " role="affiliation">Univ X, Town</contact> </creator> <abstract inlist="toc" name="Abstract" xml:id="abstract1"> <p xml:id="abstract1.1">A.</p> </abstract> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
  assert_eq!(
    converted(
      repro!("email_weak_four_letter_surname_opens_after_two_letters"),
      0
    ),
    r##"<title>T</title> <creator role="author"> <personname>Zhen Liu</personname> <contact name="Affiliation: " role="affiliation">Univ A, City </contact> <contact name="E-mail " role="email"><text font="typewriter" xml:id="id1">liuzhen03@a.com</text></contact> </creator> <creator before=" " role="author"> <personname>Yinglong Wang</personname> <contact name="Affiliation: " role="affiliation">Univ B, City </contact> <contact name="E-mail " role="email"><text font="typewriter" xml:id="id2">ylwanguestc@gmail.com</text></contact> </creator> <creator before=" " role="author"> <personname>Bing Zeng</personname> <contact name="Affiliation: " role="affiliation">Univ C, City </contact> <contact name="E-mail " role="email"><text font="typewriter" xml:id="id3">zengbing@c.cn</text></contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Body.</p> </para>"##
  );
  assert_eq!(
    converted(
      repro!("email_own_call_weak_spelling_of_another_name_stays"),
      0
    ),
    r##"<title>T</title> <creator role="author"> <personname>Husheng Yu</personname> <contact name="Email: " role="email">yhsheng@x.cn</contact> <contact name="Affiliation: " role="affiliation">Univ X, Town</contact> </creator> <creator before=" " role="author"> <personname>Xiangmin Shen</personname> <contact name="Email: " role="email">xshen@y.edu</contact> <contact name="Affiliation: " role="affiliation">Univ Y, City</contact> </creator> <abstract inlist="toc" name="Abstract" xml:id="abstract1"> <p xml:id="abstract1.1">A.</p> </abstract> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
}

/// A list a mark leads is one per person where a sibling spells a non-carrier (2412.13509) or the line names one (2102.08716,
/// each address on one author by elimination); siblings spelling carriers leave the free carrier; a lone address stays every
/// carrier's (user ruling 2026-10-30). Distinct: `link_entry_addresses`' `marked` and `list_placements`' `mark_lists`.
#[test]
fn a_list_a_mark_leads_is_no_carriers_where_it_spells_others() {
  assert_eq!(
    converted(
      repro!("email_list_mark_line_siblings_spell_others_is_no_carriers"),
      0
    ),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able<sup xml:id="id1">*1</sup></personname> <contact name="Affiliation: " role="affiliation"><text class="ltx_affiliation_institution" xml:id="id2"><sup xml:id="id2.1">1</sup>Univ A, <sup xml:id="id2.2">2</sup>Univ B</text></contact> <contact name="Email: " role="email">aable@a.edu</contact> </creator> <creator before=", " role="author"> <personname>Dan Dorn<sup xml:id="id3">*2</sup></personname> <contact name="Affiliation: " role="affiliation"><text class="ltx_affiliation_institution" xml:id="id4"><sup xml:id="id4.1">1</sup>Univ A, <sup xml:id="id4.2">2</sup>Univ B</text></contact> <contact name="Email: " role="email">ddorn@b.edu</contact> </creator> <creator before=", " role="author"> <personname>Bob Fu<sup xml:id="id5">1</sup></personname> <contact name="Affiliation: " role="affiliation"><text class="ltx_affiliation_institution" xml:id="id6"><sup xml:id="id6.1">1</sup>Univ A, <sup xml:id="id6.2">2</sup>Univ B</text></contact> <contact name="Email: " role="email">bmfu@a.edu</contact> </creator> <creator before=" and " role="author"> <personname>Cat Cole<sup xml:id="id7">†1</sup></personname> <contact name="Affiliation: " role="affiliation"><text class="ltx_affiliation_institution" xml:id="id8"><sup xml:id="id8.1">1</sup>Univ A, <sup xml:id="id8.2">2</sup>Univ B</text></contact> <contact name="Email: " role="email">ccole@a.edu</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Body.</p> </para>"##
  );
  assert_eq!(
    converted(
      repro!("email_list_mark_line_names_non_carriers_is_no_carriers"),
      0
    ),
    r##"<title>T</title> <creator role="author"> <personname>David Macedo*</personname> <contact name="Emails: " role="email"><text font="typewriter" fontsize="90%" xml:id="id1">dlm@cin.ufpe.br</text></contact> </creator> <creator before=" " role="author"> <personname>Pedro Dreyer*</personname> <contact name="Emails: " role="email"><text font="typewriter" fontsize="90%" xml:id="id2">phdl@cin.ufpe.br</text></contact> </creator> <creator before=" " role="author"> <personname>Teresa Ludermir</personname> <contact name="Emails: " role="email"><text font="typewriter" fontsize="90%" xml:id="id3">tbl@cin.ufpe.br</text></contact> </creator> <creator before=" " role="author"> <personname>Cleber Zanchettin</personname> <contact name="Emails: " role="email"><text font="typewriter" fontsize="90%" xml:id="id4">cz@cin.ufpe.br</text></contact> </creator> <creator role="author"> <contact name="Affiliation: " role="affiliation">*equal contributions. David Macedo, Pedro Dreyer, Teresa Ludermir, and Cleber Zanchettin are with the Centro de Informatica, Universidade Federal, Recife, Brasil.</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Body.</p> </para>"##
  );
  assert_eq!(
    converted(
      repro!("email_list_mark_line_siblings_spell_carriers_leaves_the_free_one"),
      0
    ),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able<sup xml:id="id1">*1</sup></personname> <contact name="Affiliation: " role="affiliation"><text class="ltx_affiliation_institution" xml:id="id2"><sup xml:id="id2.1">1</sup>Univ A, <sup xml:id="id2.2">2</sup>Univ B</text></contact> <contact name="Email: " role="email">aable@a.edu</contact> </creator> <creator before=", " role="author"> <personname>Dan Dorn<sup xml:id="id3">*2</sup></personname> <contact name="Affiliation: " role="affiliation"><text class="ltx_affiliation_institution" xml:id="id4"><sup xml:id="id4.1">1</sup>Univ A, <sup xml:id="id4.2">2</sup>Univ B</text></contact> </creator> <creator before=", " role="author"> <personname>Bob Fu<sup xml:id="id5">1</sup></personname> <contact name="Affiliation: " role="affiliation"><text class="ltx_affiliation_institution" xml:id="id6"><sup xml:id="id6.1">1</sup>Univ A, <sup xml:id="id6.2">2</sup>Univ B</text></contact> <contact name="Email: " role="email">bmfu@a.edu</contact> </creator> <creator before=" and " role="author"> <personname>Cat Cole<sup xml:id="id7">†1</sup></personname> <contact name="Affiliation: " role="affiliation"><text class="ltx_affiliation_institution" xml:id="id8"><sup xml:id="id8.1">1</sup>Univ A, <sup xml:id="id8.2">2</sup>Univ B</text></contact> <contact name="Email: " role="email">ccole@a.edu</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Body.</p> </para>"##
  );
  assert_eq!(
    converted(
      repro!("email_lone_address_mark_line_stays_every_carriers"),
      0
    ),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able<sup xml:id="id1">*1</sup></personname> <contact name="Affiliation: " role="affiliation"><text class="ltx_affiliation_institution" xml:id="id2"><sup xml:id="id2.1">1</sup>Univ A, <sup xml:id="id2.2">2</sup>Univ B</text></contact> <contact name="Email: " role="email">lab@a.edu</contact> </creator> <creator before=", " role="author"> <personname>Dan Dorn<sup xml:id="id3">*2</sup></personname> <contact name="Affiliation: " role="affiliation"><text class="ltx_affiliation_institution" xml:id="id4"><sup xml:id="id4.1">1</sup>Univ A, <sup xml:id="id4.2">2</sup>Univ B</text></contact> </creator> <creator before=" and " role="author"> <personname>Cat Cole<sup xml:id="id5">†1</sup></personname> <contact name="Affiliation: " role="affiliation"><text class="ltx_affiliation_institution" xml:id="id6"><sup xml:id="id6.1">1</sup>Univ A, <sup xml:id="id6.2">2</sup>Univ B</text></contact> <contact name="Email: " role="email">lab@a.edu</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Body.</p> </para>"##
  );
}

/// A tie and a space part whole names (1805.06045's IEEE block, `Ann Able ~Bob Baker`), and stay one word space inside one
/// name (1202.2048's `N.~B. ~Li`, 0802.4349's `V.G.~ Kagramanova`). Distinct: `Lexer::ties_the_gap`.
#[test]
fn a_tie_and_a_space_are_one_word_space_only_inside_a_name() {
  assert_eq!(
    converted(repro!("ieee_tie_and_space_between_whole_names_part"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able<sup xml:id="id1">∗</sup></personname> </creator> <creator role="author"> <personname>Bob Baker<sup xml:id="id2">∗</sup></personname> </creator> <creator role="author"> <personname>Cat Cole</personname> </creator> <creator role="author"> <personname>Dan Dee</personname> </creator> <creator role="author"> <personname>Eve Ewing</personname> <note class="ltx_note_frontmatter ltx_thanks_note" role="thanks" xml:id="id3">Thanks note.</note> </creator> <para xml:id="p1"> <p xml:id="p1.1">Body text.</p> </para>"##
  );
  assert_eq!(
    converted(repro!("author_space_tie_between_two_names"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able</personname> </creator> <creator role="author"> <personname>Bob Baker</personname> </creator> <para xml:id="p1"> <p xml:id="p1.1">Body.</p> </para>"##
  );
  assert_eq!(
    converted(
      repro!("author_tie_after_initials_then_space_is_one_name"),
      0
    ),
    r##"<title>T</title> <creator role="author"> <personname>N. B. Li</personname> </creator> <creator before=" " role="author"> <personname>V.G. Kagramanova</personname> </creator> <creator before=" " role="author"> <personname>Lei Li</personname> </creator> <para xml:id="p1"> <p xml:id="p1.1">Body.</p> </para>"##
  );
}

/// A wrapper under the names over lines with no names and no marks stays one entry: a `\parbox` (2509.02924), a group
/// (2404.17162), beside a legend in a group, a note as unwrapped (2405.20895). Distinct: `wrapper_holds_names_or_marks`.
#[test]
fn a_wrapper_over_plain_lines_stays_one_entry() {
  assert_eq!(
    converted(repro!("author_wrapper_over_plain_lines_stays_one_entry"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation"><inline-block class="ltx_parbox" vattach="middle" width="101.2pt" xml:id="id1"> <p align="center" xml:id="id1.1"><text fontsize="70%" xml:id="id1.1.1">Media Arts &amp; Technology <break/>UC Santa Barbara</text></p> </inline-block></contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation"><inline-block class="ltx_parbox" vattach="middle" width="101.2pt" xml:id="id2"> <p align="center" xml:id="id2.1"><text fontsize="70%" xml:id="id2.1.1">Media Arts &amp; Technology <break/>UC Santa Barbara</text></p> </inline-block></contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Body.</p> </para>"##
  );
  assert_eq!(
    converted(repro!("author_group_over_plain_lines_stays_one_entry"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation"><text fontsize="90%" xml:id="id1">Department of Mathematics,<break/>Central University,<break/>Thiruvarur 610005,<break/>India.</text></contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation"><text fontsize="90%" xml:id="id2">Department of Mathematics,<break/>Central University,<break/>Thiruvarur 610005,<break/>India.</text></contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Body.</p> </para>"##
  );
  assert_eq!(
    converted(repro!("author_group_legend_line_is_a_note_as_unwrapped"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">1. Department of Statistics, <break/>Faculty of Sciences, Utrecht University</contact> <note class="ltx_note_frontmatter ltx_thanks_correspondence" mark="∗" role="thanks" xml:id="id1">*Corresponding author</note> <contact name="Email: " role="email"><text font="typewriter" xml:id="id2">ann@x.org</text></contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">1. Department of Statistics, <break/>Faculty of Sciences, Utrecht University</contact> <contact name="Email: " role="email"><text font="typewriter" xml:id="id3">bob@x.org</text></contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Body.</p> </para>"##
  );
}

/// An address starts where its font does, the word glued before it staying with the entry's text (2203.06585, review
/// rev107g), on the kernel's `kind_address_lines`; the observer's route is `addr_line_glued_word_before_a_font_observer_route_red`.
/// Distinct: no other guard has a font boundary inside an address's token.
#[test]
fn an_address_starts_at_its_font() {
  assert_eq!(
    converted(repro!("addr_line_glued_word_before_a_font_of_its_own"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Ann Able and Bob Baker are with Univ A, Town, China.</contact> <contact name="Email: " role="email"><text font="typewriter" fontsize="80%" xml:id="id1">aable@a.edu</text></contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Ann Able and Bob Baker are with Univ A, Town, China.</contact> <contact name="Email: " role="email"><text font="typewriter" fontsize="80%" xml:id="id2">bbaker@a.edu</text></contact> </creator> <creator before=" " role="author"> <personname>Cy Coe</personname> <contact name="Affiliation: " role="affiliation">Cy Coe, corresponding author, is with Univ B, City, China.</contact> <contact name="Email: " role="email"><text font="typewriter" fontsize="80%" xml:id="id3">ccoe@b.edu</text></contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
}
