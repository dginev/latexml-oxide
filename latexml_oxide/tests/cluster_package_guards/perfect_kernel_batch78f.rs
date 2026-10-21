//! Red/green guards for batch 78f (quality batch 9): e-mail lists a class prints apart from the names placed on the
//! authors they belong to — jheppub's `\emailAdd` (and jcappub's and jinstpub's, which ride it) and iopart's `\ead`:
//! with the author an address spells, else the n-th author's while the list is the authors' in order, else the
//! group's only author's or the author block's (`\lx@add@contact@placed[owner=sequence]`; OXIDIZED_DESIGN_DIVERGENCES
//! #805, #806); and an `\inst` mark after a comma on the name before it, as llncs's and aa's `\unskip` sets it
//! (#807). Each guard converts its repro under `tools/perfect_kernel/repros/sectioning-frontmatter/` with the ar5iv
//! preload and asserts no Error or Warning and every creator whole, against pdflatex. Witnesses 2311.12551 (iopart),
//! 2401.16957 (aa), 2407.18115, 2011.05971, 2212.09776 (jheppub), 1804.07528, 1806.05015, 2407.07862 (jcappub).
use latexml::util::test::convert_with;

use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch77d::{creator, creators, squash},
};

/// A `sectioning-frontmatter` repro's squashed XML, the conversion free of errors and warnings.
macro_rules! convert_repro {
  ($path:literal) => {{
    let (log, xml) = convert_with(
      include_str!(concat!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
        $path
      )),
      Some("ar5iv.sty"),
    );
    assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
    squash(&xml)
  }};
}

/// A contact of `role`, named as the frontmatter names it.
fn contact(role: &str, text: &str) -> String {
  let name = match role {
    "affiliation" => "Affiliation: ",
    "email" => "Email: ",
    "note" => "Note: ",
    _ => unreachable!(),
  };
  format!(r#"<contact name="{name}" role="{role}">{text}</contact>"#)
}

/// An e-mail contact set in the class's typewriter address font.
fn email(id: &str, address: &str) -> String {
  contact(
    "email",
    &format!(r#"<text font="typewriter" xml:id="{id}">{address}</text>"#),
  )
}

/// The author block's own creator (no name), holding `tail`.
fn block(tail: &str) -> String { format!(r#"<creator role="author"> {tail} </creator>"#) }

/// jheppub's `\emailAdd` list in author order — no address spells another author than the one at its place and
/// bbaker@ spells its own — keeps Perl's rule, the n-th call's addresses the n-th author's, also where no name spells
/// them (zq1@, wy7@); an `\emailAdd{a, b}` call is an address each, here both Cat Cole's. Red (int47a): "wy7@y.org,
/// cat2@y.org" one contact. Witnesses 1303.3040, 2011.02499, 2212.01400, 1710.06069. Repro
/// sectioning-frontmatter/jheppub_emailadd_in_author_order.
#[test]
fn jheppub_emailadd_in_author_order() {
  let xml = convert_repro!("jheppub_emailadd_in_author_order.tex");
  let univ_a = contact("affiliation", "Univ A");
  assert_eq!(creators(&xml), [
    creator(
      true,
      "Ann Able",
      &format!("{univ_a} {}", email("id1", "zq1@x.org"))
    ),
    creator(
      false,
      "Bob Baker",
      &format!("{univ_a} {}", email("id2", "bbaker@x.org"))
    ),
    creator(
      false,
      "Cat Cole",
      &format!(
        "{} {} {}",
        contact("affiliation", "Univ B"),
        email("id3", "wy7@y.org"),
        email("id4", "cat2@y.org")
      )
    ),
  ]);
}

/// One jheppub `\emailAdd{a, b, c}` listing every author's address: by the call the list is out of order (bbaker@ at
/// the first call's place), by the address it is the authors' in order, each address the n-th author's (zq1@, wy7@
/// spelling no one). Red (int47a, Perl): one contact of the three addresses on Ann Able. Witnesses 2406.03543,
/// 2505.23122, 2201.12390. Repro sectioning-frontmatter/jheppub_emailadd_one_call_lists_the_authors.
#[test]
fn jheppub_emailadd_one_call_lists_the_authors() {
  let xml = convert_repro!("jheppub_emailadd_one_call_lists_the_authors.tex");
  let univ_a = contact("affiliation", "Univ A");
  assert_eq!(creators(&xml), [
    creator(
      true,
      "Ann Able",
      &format!("{univ_a} {}", email("id1", "zq1@x.org"))
    ),
    creator(
      false,
      "Bob Baker",
      &format!("{univ_a} {}", email("id2", "bbaker@x.org"))
    ),
    creator(
      false,
      "Cat Cole",
      &format!(
        "{} {}",
        contact("affiliation", "Univ B"),
        email("id3", "wy7@y.org")
      )
    ),
  ]);
}

/// jheppub's `\emailAdd` list not in author order (bbaker@ first, Bob Baker author 2) is no list the n-th rule reads:
/// bbaker@ goes to Bob Baker, whose name it spells, and zq1@, spelling no one, to the author block, as the PDF's
/// shared "E-mail:" line leaves it. Red (int47a, Perl): bbaker@ on Ann Able, zq1@ on Bob Baker. Witnesses 2407.18115,
/// 2011.05971, 2212.09776; jcappub 1804.07528, 1806.05015, 2407.07862. Repro
/// sectioning-frontmatter/jheppub_emailadd_out_of_order_by_spelling.
#[test]
fn jheppub_emailadd_out_of_order_by_spelling() {
  let xml = convert_repro!("jheppub_emailadd_out_of_order_by_spelling.tex");
  let univ_a = contact("affiliation", "Univ A");
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &univ_a),
    creator(
      false,
      "Bob Baker",
      &format!("{univ_a} {}", email("id1", "bbaker@x.org"))
    ),
    block(&email("id2", "zq1@x.org")),
  ]);
}

/// iopart's `\ead` prints its address under the addresses, no name's (iopart.cls:354-355): it goes to B. Maier, whose
/// name it spells. Red (int47a, Perl's `\lx@add@email`): on C. Cole, the last author. Witness 2311.12551
/// (benedikt.maier@cern.ch on T. K. Årrestad). Repro sectioning-frontmatter/iopart_ead_goes_to_the_author_it_spells.
#[test]
fn iopart_ead_goes_to_the_author_it_spells() {
  let xml = convert_repro!("iopart_ead_goes_to_the_author_it_spells.tex");
  let kit = contact("affiliation", " KIT, Karlsruhe");
  let icl = contact("affiliation", " ICL, London");
  assert_eq!(creators(&xml), [
    creator(true, "A. Bal", &kit),
    creator(
      false,
      "B. Maier",
      &format!("{kit} {icl} {}", email("id1", "benedikt.maier@cern.ch"))
    ),
    creator(false, "C. Cole", &icl),
  ]);
}

/// An `\inst` after a comma stands on the name before it (llncs.cls:894-896 `\unskip$^{#1}$`, `\unskip$^,$`), with
/// the space after the comma or without: Ann Able keeps her institutes and her note, Bob Baker his, no nameless
/// creator. Red (int47a): Ann bare, Bob with Ann's note and Inst A, Inst B, Inst C on a nameless creator. Witness
/// 2401.16957 (aa). Repro sectioning-frontmatter/llncs_comma_before_inst_is_the_name_before.
#[test]
fn llncs_comma_before_inst_is_the_name_before() {
  let xml = convert_repro!("llncs_comma_before_inst_is_the_name_before.tex");
  let inst = |name: &str| contact("affiliation", name);
  assert_eq!(creators(&xml), [
    creator(
      true,
      "Ann Able",
      &format!(
        r#"<note class="ltx_note_frontmatter ltx_thanks_note" role="thanks" xml:id="id1">Email: ann@x.org</note> {} {}"#,
        inst("Inst A"),
        inst("Inst B")
      )
    ),
    creator(
      false,
      "Bob Baker",
      &format!("{} {}", inst("Inst B"), inst("Inst C"))
    ),
    creator(false, "Cat Cole", &inst("Inst A")),
  ]);
}

/// An iopart `\ead` listing several addresses — each in its own `\mailto` (iopart.cls:364, the typewriter font), or a
/// shared domain `\{a, b\}@x` — is an address each, each placed as one `\ead` is: the author it spells, else the author
/// block (four addresses for three authors are no list in author order). Red (int47a, Perl): every address on A. Able,
/// the last author, beside a "," contact. Witnesses 0811.0981, 2607.04241, 2111.00753. Repro
/// sectioning-frontmatter/iopart_ead_lists_are_an_address_each.
#[test]
fn iopart_ead_lists_are_an_address_each() {
  let xml = convert_repro!("iopart_ead_lists_are_an_address_each.tex");
  let univ_c = contact("affiliation", " Univ C");
  assert_eq!(creators(&xml), [
    creator(
      true,
      "E. Mortsell",
      &format!(
        "{} {}",
        contact("affiliation", " Univ S"),
        email("id1", "mortsell@s.se")
      )
    ),
    creator(
      false,
      "C. Clarkson",
      &format!("{univ_c} {}", email("id2", "chris.clarkson@c.za"))
    ),
    creator(
      false,
      "A. Able",
      &format!("{univ_c} {}", email("id3", "aable@c.za"))
    ),
    block(&email("id4", "xq9@c.za")),
  ]);
}

// Round r2 (the rev78f review and the user rulings of 2026-10-21): the iopart `\ead` under its `\address`'s mark, the
// iopart author groups, the one corresponding author of an address left over, the address lists inside an author and
// in markup, an `\ead{a and b}` list, and the `\inst` comma rule by `\inst`'s meaning (OXIDIZED_DESIGN_DIVERGENCES #805,
// #806, #807, #808, #809).

/// An `\ead` prints as an address block right after the `\address` before it (iopart.cls:363): an address spelling no
/// one under `$^1$`, which Ann Able alone requests, is hers; able@ under `$^3$` (Cat Cole's) is Ann Able's by its
/// spelling; wy5@ under `$^2$`, which two authors request, the author block's. Red (78f r1): zq77@ on the block too.
/// Witnesses 2407.14305, 2311.07087 (an `\ead` between the addresses); 1111.5264; not 1209.0173, 0910.0753, 1111.3344,
/// 1911.06133 (every `\ead` after the last address). Repro sectioning-frontmatter/iopart_ead_under_its_address_mark.
#[test]
fn iopart_ead_under_its_address_mark() {
  let xml = convert_repro!("iopart_ead_under_its_address_mark.tex");
  let univ_b = contact("affiliation", " Univ B");
  assert_eq!(creators(&xml), [
    creator(
      true,
      "Ann Able",
      &format!(
        "{} {} {}",
        contact("affiliation", " Univ A"),
        email("id1", "zq77@x.org"),
        email("id2", "able@z.org")
      )
    ),
    creator(
      false,
      "Bob Baker",
      &format!("{univ_b} {}", email("id3", "bbaker@y.org"))
    ),
    creator(
      false,
      "Cat Cole",
      &format!("{univ_b} {}", contact("affiliation", " Univ C"))
    ),
    block(&email("id4", "wy5@y.org")),
  ]);
}

/// In a document whose `\ead`s all follow its addresses ("after the postal address(es)", as IOP's guidelines ask), the
/// last address's mark is the `\ead`'s by position only: zq77@, spelling no one, stays the author block's. Red (the
/// address-mark rule alone): Bob Baker's. Witnesses 0910.0753, 1111.3344, 1911.06133 (the mark's author wrong there),
/// 1209.0173. Repro sectioning-frontmatter/iopart_ead_after_all_addresses_takes_no_mark.
#[test]
fn iopart_ead_after_all_addresses_takes_no_mark() {
  let xml = convert_repro!("iopart_ead_after_all_addresses_takes_no_mark.tex");
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &contact("affiliation", " Univ A")),
    creator(false, "Bob Baker", &contact("affiliation", " Univ B")),
    block(&email("id1", "zq77@x.org")),
  ]);
}

/// Each iopart `\author` is a group whose `\address` and `\ead` follow it (iopart.cls:225-231): yy2@ is B. Maier's,
/// his group's only author; zq1@, under a group of two, the author block's. Red (78f r1): both on the block; int47a: the
/// last author's each. Witness 2311.17496. Repro sectioning-frontmatter/iopart_author_groups_own_their_ead.
#[test]
fn iopart_author_groups_own_their_ead() {
  let xml = convert_repro!("iopart_author_groups_own_their_ead.tex");
  assert_eq!(creators(&xml), [
    creator(true, "A. Bal", ""),
    creator(false, "C. Cole", &contact("affiliation", "KIT, Karlsruhe")),
    creator(
      false,
      "B. Maier",
      &format!(
        "{} {}",
        contact("affiliation", "ICL, London"),
        email("id1", "yy2@icl.uk")
      )
    ),
    block(&email("id2", "zq1@kit.edu")),
  ]);
}

/// `\ead{a@x and b@y}` is an address each, each on the author it spells. Red (78f r1, int47a): one contact on Bob
/// Baker. Witnesses 0710.2104, 1003.0073. Repro sectioning-frontmatter/iopart_ead_and_list_is_an_address_each.
#[test]
fn iopart_ead_and_list_is_an_address_each() {
  let xml = convert_repro!("iopart_ead_and_list_is_an_address_each.tex");
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &email("id1", "able@x.org")),
    creator(
      false,
      "Bob Baker",
      &format!(
        "{} {}",
        contact("affiliation", "Univ A, Town"),
        email("id2", "baker@x.org")
      )
    ),
  ]);
}

/// An `\ead` spelling no one, under an address all three authors request, goes to the one author marked as the
/// corresponding author: Bob Baker's `*`, whose legend says so (user ruling 2026-10-21). Red (78f r1): the author
/// block's. Witnesses (iopart) 2504.15015 (`Bo Yang$^{1,2,*}$` under `\address{$^*$ Author to whom any correspondence
/// should be addressed.}`), 2205.14019, 2205.09074; the jheppub-family papers of the rule are the `\emailAdd` guards'
/// (1901.08773, 2510.11030 by a note; 1802.03677 a note two authors carry; 2308.06532 an address naming
/// correspondence). Repro sectioning-frontmatter/iopart_ead_corresponding_author_legend.
#[test]
fn iopart_ead_corresponding_author_legend() {
  let xml = convert_repro!("iopart_ead_corresponding_author_legend.tex");
  let univ_a = contact("affiliation", " Univ A, Town");
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &univ_a),
    creator(
      false,
      "Bob Baker",
      &format!(
        "{univ_a} {} {}",
        email("id1", "zq77@x.org"),
        contact(
          "affiliation",
          " Author to whom any correspondence should be addressed."
        )
      )
    ),
    creator(false, "Cat Cole", &univ_a),
  ]);
}

/// The one `\emailAdd` address left over (zq77@; ccole@ is Cat Cole's by its spelling) goes to the one author a
/// `\note{Corresponding author.}` marks, Bob Baker, no one else carrying its number (user ruling 2026-10-21). Red (78f
/// r1): the author block's; int47a: Ann Able's. Witnesses 1901.08773, 2510.11030, 2211.07892; not 1802.03677, whose
/// note number two authors carry (`jheppub_note_number_carried_by_two_stays_the_blocks`). Repro
/// sectioning-frontmatter/jheppub_emailadd_corresponding_author_note.
#[test]
fn jheppub_emailadd_corresponding_author_note() {
  let xml = convert_repro!("jheppub_emailadd_corresponding_author_note.tex");
  let univ_a = contact("affiliation", "Univ A");
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &univ_a),
    creator(
      false,
      "Bob Baker",
      &format!(
        r#"<contact name="Note: " role="note">Corresponding author.</contact> {univ_a} {}"#,
        email("id1", "zq77@x.org")
      )
    ),
    creator(
      false,
      "Cat Cole",
      &format!(
        "{} {}",
        contact("affiliation", "Univ B"),
        email("id2", "ccole@y.org")
      )
    ),
  ]);
}

/// The one `\emailAdd` address goes to the one author whose mark's legend names the corresponding author (`\author[a,*]`
/// under `\affiliation[*]{Corresponding author}`): Cat Cole (user ruling 2026-10-21). Red (78f r1): the author block's;
/// int47a: Ann Able's. Witness 1304.1216 (the legend as an affiliation, `\affiliation[f]{\textcolor{blue}{Corresponding
/// authors}}`, carried by two; its addresses each spelled); 2308.06532's `*` has no legend (its guard is
/// `jheppub_emailadd_address_naming_correspondence_goes_to_the_star`). Repro
/// sectioning-frontmatter/jheppub_emailadd_corresponding_author_star_legend.
#[test]
fn jheppub_emailadd_corresponding_author_star_legend() {
  let xml = convert_repro!("jheppub_emailadd_corresponding_author_star_legend.tex");
  let univ_a = contact("affiliation", "Univ A");
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &univ_a),
    creator(false, "Bob Baker", &univ_a),
    creator(
      false,
      "Cat Cole",
      &format!(
        "{univ_a} {} {}",
        contact("affiliation", "Corresponding author"),
        email("id1", "zq77@x.org")
      )
    ),
  ]);
}

/// Two authors marked as corresponding leave the address to the author block (user ruling 2026-10-21). Witnesses
/// 1304.1216 (two authors under "Corresponding authors"), 2606.26596 (two under `\note[*]{Corresponding authors.}`);
/// 2111.00753's two stars have no legend and mark no one. Repro
/// sectioning-frontmatter/jheppub_emailadd_two_marked_authors_stay_the_blocks.
#[test]
fn jheppub_emailadd_two_marked_authors_stay_the_blocks() {
  let xml = convert_repro!("jheppub_emailadd_two_marked_authors_stay_the_blocks.tex");
  let univ_a = contact("affiliation", "Univ A");
  let legend = contact("affiliation", "Corresponding authors");
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &format!("{univ_a} {legend}")),
    creator(false, "Bob Baker", &univ_a),
    creator(false, "Cat Cole", &format!("{univ_a} {legend}")),
    block(&email("id1", "zq77@x.org")),
  ]);
}

/// An `\emailAdd` inside an `\author` is that author's e-mail, one contact. Red (78f r1): a second, empty e-mail
/// contact beside it. Repro sectioning-frontmatter/jheppub_emailadd_in_author_is_the_authors (rev78f probe j13).
#[test]
fn jheppub_emailadd_in_author_is_the_authors() {
  let xml = convert_repro!("jheppub_emailadd_in_author_is_the_authors.tex");
  let univ_a = contact("affiliation", "Univ A");
  assert_eq!(creators(&xml), [
    creator(
      true,
      "Ann Able",
      &format!("{} {univ_a}", email("id1", "zq1@x.org"))
    ),
    creator(
      false,
      "Bob Baker",
      &format!("{univ_a} {}", email("id2", "bbaker@x.org"))
    ),
  ]);
}

/// An `\emailAdd` list in markup is one contact, its markup kept, not written out address by address. Red (78f r1): two
/// contacts, the `\mbox` dropped. Repro sectioning-frontmatter/jheppub_emailadd_markup_list_kept_whole (rev78f probe
/// j06).
#[test]
fn jheppub_emailadd_markup_list_kept_whole() {
  let xml = convert_repro!("jheppub_emailadd_markup_list_kept_whole.tex");
  let univ_a = contact("affiliation", "Univ A");
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &univ_a),
    creator(false, "Bob Baker", &univ_a),
    block(&email("id1", "zq1@x.org, bbaker@x.org")),
  ]);
}

/// Names that open with their `\inst` marks keep each its own. Red (78f r1): Inst A and Inst B on Ann Able. Repro
/// sectioning-frontmatter/llncs_inst_leading_marks_keep_their_names (rev78f probe c01).
#[test]
fn llncs_inst_leading_marks_keep_their_names() {
  let xml = convert_repro!("llncs_inst_leading_marks_keep_their_names.tex");
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &contact("affiliation", "Inst A")),
    creator(false, "Bob Baker", &contact("affiliation", "Inst B")),
    creator(false, "Cat Cole", &contact("affiliation", "Inst C")),
  ]);
}

/// A document's own `\inst` that takes back no space (`\newcommand{\inst}[1]{$^{#1}$}`) gets no comma rule: Bob Baker
/// keeps his 2. Red (78f r1): "1,2" on Ann Able. Repro sectioning-frontmatter/inst_without_unskip_keeps_the_next_name
/// (rev78f probes c09, c08).
#[test]
fn inst_without_unskip_keeps_the_next_name() {
  let xml = convert_repro!("inst_without_unskip_keeps_the_next_name.tex");
  assert_eq!(creators(&xml), [
    creator(true, r#"Ann Able<sup xml:id="id1">1</sup>"#, ""),
    creator(false, r#"Bob Baker<sup xml:id="id2">2</sup>"#, ""),
  ]);
}

/// A `*` with no legend anywhere says nothing of correspondence: the one `\emailAdd` address left over stays the author
/// block's (user ruling 2026-10-21: a "Corresponding author" note, or a mark whose legend says so). Red (78f r2): Cat
/// Cole's. Witnesses 2111.00753 (iopart, two legend-less stars), 2602.07867. Repro
/// sectioning-frontmatter/jheppub_emailadd_star_without_legend_stays_the_blocks (rev78f2 probe s1).
#[test]
fn jheppub_emailadd_star_without_legend_stays_the_blocks() {
  let xml = convert_repro!("jheppub_emailadd_star_without_legend_stays_the_blocks.tex");
  let univ_a = contact("affiliation", "Univ A");
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &univ_a),
    creator(false, "Bob Baker", &univ_a),
    creator(false, "Cat Cole", &univ_a),
    block(&email("id1", "zq77@x.org")),
  ]);
}

/// iopart's `$^{1,*}$` with no `\address{$^*$ …}` legend marks no corresponding author: the `\ead` under an address all
/// three authors request stays the author block's. Red (78f r2): Bob Baker's. Witness 2111.00753. Repro
/// sectioning-frontmatter/iopart_ead_star_without_legend_stays_the_blocks (rev78f2 probe i3).
#[test]
fn iopart_ead_star_without_legend_stays_the_blocks() {
  let xml = convert_repro!("iopart_ead_star_without_legend_stays_the_blocks.tex");
  let univ_a = contact("affiliation", " Univ A, Town");
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &univ_a),
    creator(false, r#"Bob Baker<sup xml:id="id1">∗</sup>"#, &univ_a),
    creator(false, "Cat Cole", &univ_a),
    block(&email("id2", "zq77@x.org")),
  ]);
}

/// The one address left over that says itself it is the corresponding author's, with exactly one author carrying a `*`
/// no legend explains, is that author's: Cat Cole. Witness 2308.06532 (`\author[1,*]{Zhaoyi Xu}`,
/// `\emailAdd{zyxu@gzu.edu.cn(Corresponding author)}`). Repro
/// sectioning-frontmatter/jheppub_emailadd_address_naming_correspondence_goes_to_the_star (rev78f2 probe s6).
#[test]
fn jheppub_emailadd_address_naming_correspondence_goes_to_the_star() {
  let xml = convert_repro!("jheppub_emailadd_address_naming_correspondence_goes_to_the_star.tex");
  let univ_a = contact("affiliation", "Univ A");
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &univ_a),
    creator(false, "Bob Baker", &univ_a),
    creator(
      false,
      "Cat Cole",
      &format!(
        "{univ_a} {}",
        email("id1", "zq77@x.org (Corresponding author)")
      )
    ),
  ]);
}

/// jheppub's `\note[*]{Corresponding author.}` (jheppub.sty:84-89 `\newcommand\note[2][]`) in Bob Baker's `\author` is
/// his note, his name whole, and marks him the corresponding author: the one `\emailAdd` address left over is his. Red
/// (78f r2, int47a, Perl's one-argument `\note`): the name "Bob Baker*]Corresponding author" and a note "[". Witnesses
/// 2606.26596, 1510.03434, 2404.00867. Repro sectioning-frontmatter/jheppub_note_mark_corresponding_author (rev78f2
/// probe s7).
#[test]
fn jheppub_note_mark_corresponding_author() {
  let xml = convert_repro!("jheppub_note_mark_corresponding_author.tex");
  let univ_a = contact("affiliation", "Univ A");
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &univ_a),
    creator(
      false,
      "Bob Baker",
      &format!(
        "{} {univ_a} {}",
        contact("note", "Corresponding author."),
        email("id1", "zq77@x.org")
      )
    ),
    creator(false, "Cat Cole", &univ_a),
  ]);
}

/// A `\note[*]{Equal contribution.}` is its `*`'s legend and says nothing of correspondence: Ann Able keeps her name
/// and the note, the one `\emailAdd` address left over stays the author block's. Red (78f r2, int47a): the name "Ann
/// Able,*]Equal contribution", a note "[", zq77@ hers. Witness 1510.03434. Repro
/// sectioning-frontmatter/jheppub_note_mark_equal_contribution_stays_the_blocks (rev78f2 probe s3).
#[test]
fn jheppub_note_mark_equal_contribution_stays_the_blocks() {
  let xml = convert_repro!("jheppub_note_mark_equal_contribution_stays_the_blocks.tex");
  let univ_a = contact("affiliation", "Univ A");
  assert_eq!(creators(&xml), [
    creator(
      true,
      "Ann Able",
      &format!("{} {univ_a}", contact("note", "Equal contribution."))
    ),
    creator(false, "Bob Baker", &univ_a),
    creator(false, "Cat Cole", &univ_a),
    block(&email("id1", "zq77@x.org")),
  ]);
}

/// Note owners are a set: two authors whose notes name them corresponding and a third under a "Corresponding author"
/// legend are three marked authors, so the address stays the author block's. Red (78f r2): Cat Cole's. Repro
/// sectioning-frontmatter/jheppub_emailadd_note_owners_are_a_set (rev78f2 probe s5).
#[test]
fn jheppub_emailadd_note_owners_are_a_set() {
  let xml = convert_repro!("jheppub_emailadd_note_owners_are_a_set.tex");
  let univ_a = contact("affiliation", "Univ A");
  let noted = format!("{} {univ_a}", contact("note", "Corresponding author."));
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &noted),
    creator(false, "Bob Baker", &noted),
    creator(
      false,
      "Cat Cole",
      &format!(
        "{univ_a} {}",
        contact("affiliation", "Corresponding author")
      )
    ),
    block(&email("id1", "zq77@x.org")),
  ]);
}

/// A jheppub `\note{…}` prints its text under the next footnote number, beside no name (jheppub.sty:84-89): the authors
/// writing that number on their names (`\author[a,1]`) are the ones it is about. Two carry the 1 of "Corresponding
/// author.", so zq77@ stays the author block's; ccole@ is Cat Cole's by its spelling. Red (78f r2): zq77@ Ann Able's.
/// Witness 1802.03677 (`gear8mike@`, the 1 on M.V. Smirnov and J.J. Ling). Repro
/// sectioning-frontmatter/jheppub_note_number_carried_by_two_stays_the_blocks (rev78f2 probe s4).
#[test]
fn jheppub_note_number_carried_by_two_stays_the_blocks() {
  let xml = convert_repro!("jheppub_note_number_carried_by_two_stays_the_blocks.tex");
  let univ_a = contact("affiliation", "Univ A");
  assert_eq!(creators(&xml), [
    creator(
      true,
      "Ann Able",
      &format!("{} {univ_a}", contact("note", "Corresponding author."))
    ),
    creator(false, "Bob Baker", &univ_a),
    creator(
      false,
      "Cat Cole",
      &format!("{univ_a} {}", email("id1", "ccole@y.org"))
    ),
    block(&email("id2", "zq77@x.org")),
  ]);
}

/// An `\ead` belongs to the author group it follows (iopart.cls:225-231): zq1@, under A. Bal and C. Cole, is not B.
/// Maier's, though he alone is marked corresponding in the next group; it stays the author block's. Red (78f r2): B.
/// Maier's. Witness 2311.17496. Repro sectioning-frontmatter/iopart_ead_corresponding_author_of_another_group (rev78f2
/// probe i1).
#[test]
fn iopart_ead_corresponding_author_of_another_group() {
  let xml = convert_repro!("iopart_ead_corresponding_author_of_another_group.tex");
  assert_eq!(creators(&xml), [
    creator(true, "A. Bal", ""),
    creator(false, "C. Cole", &contact("affiliation", "KIT, Karlsruhe")),
    creator(
      false,
      r#"B. Maier<sup xml:id="id1">*</sup>"#,
      &format!(
        "{} {}",
        contact("affiliation", "ICL, London"),
        contact(
          "affiliation",
          r#"<sup xml:id="id2">∗</sup> Author to whom any correspondence should be addressed."#
        )
      )
    ),
    block(&email("id3", "zq1@kit.edu")),
  ]);
}

/// An `\ead` printed under `$^2$ Univ B` (iopart.cls:363) is one of the authors requesting 2: Cat Cole, the one marked
/// corresponding, requests 1 and `*`, so the address stays the author block's. Red (78f r2): Cat Cole's. Witnesses
/// 2407.14305, 2311.07087. Repro sectioning-frontmatter/iopart_ead_corresponding_author_not_under_its_mark (rev78f2
/// probe i2).
#[test]
fn iopart_ead_corresponding_author_not_under_its_mark() {
  let xml = convert_repro!("iopart_ead_corresponding_author_not_under_its_mark.tex");
  let univ_b = contact("affiliation", " Univ B, City");
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &univ_b),
    creator(false, "Bob Baker", &univ_b),
    creator(
      false,
      "Cat Cole",
      &format!(
        "{} {}",
        contact("affiliation", " Univ A, Town"),
        contact(
          "affiliation",
          " Author to whom any correspondence should be addressed."
        )
      )
    ),
    block(&email("id1", "zq77@x.org")),
  ]);
}

/// A `\note[*]{Corresponding author.}` after Bob Baker's `\author[*\,a]` is his note and the legend of his `*`: the
/// one address left over is his, and the thin space parts his labels, so Univ A is his too. Red (78f r2, int47a): no
/// Univ A on him, a note "[". Witness 2404.00867 (`\author[*\,b,c]{Paul J. Steinhardt,}`). Repro
/// sectioning-frontmatter/jheppub_note_mark_apart_from_the_names.
#[test]
fn jheppub_note_mark_apart_from_the_names() {
  let xml = convert_repro!("jheppub_note_mark_apart_from_the_names.tex");
  let univ_a = contact("affiliation", "Univ A");
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &univ_a),
    creator(
      false,
      "Bob Baker",
      &format!(
        "{} {univ_a} {}",
        contact("note", "Corresponding author."),
        email("id1", "zq77@x.org")
      )
    ),
    creator(false, "Cat Cole", &univ_a),
  ]);
}

/// A note names the corresponding author by a phrase that says so, not by the word alone: "the AdS/CFT
/// correspondence" marks no one, so the address left over stays the author block's. Red (78f r2): Bob Baker's. Repro
/// sectioning-frontmatter/jheppub_note_on_the_adscft_correspondence_names_no_one.
#[test]
fn jheppub_note_on_the_adscft_correspondence_names_no_one() {
  let xml = convert_repro!("jheppub_note_on_the_adscft_correspondence_names_no_one.tex");
  let univ_a = contact("affiliation", "Univ A");
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &univ_a),
    creator(
      false,
      "Bob Baker",
      &format!(
        "{} {} {univ_a}",
        contact("note", "On leave to study the AdS/CFT correspondence."),
        contact("note", "Correspondence between gauge and string theory.")
      )
    ),
    creator(false, "Cat Cole", &univ_a),
    block(&email("id1", "zq77@x.org")),
  ]);
}

/// "For" names the corresponding author only right before the word ("Author for correspondence"): "a test for the
/// correspondence" marks no one, so the address left over stays the author block's. Red (78f r3): Bob Baker's. Repro
/// sectioning-frontmatter/jheppub_note_on_a_test_for_the_correspondence_names_no_one.
#[test]
fn jheppub_note_on_a_test_for_the_correspondence_names_no_one() {
  let xml = convert_repro!("jheppub_note_on_a_test_for_the_correspondence_names_no_one.tex");
  let univ_a = contact("affiliation", "Univ A");
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &univ_a),
    creator(
      false,
      "Bob Baker",
      &format!(
        "{} {univ_a}",
        contact("note", "Supported by a test for the correspondence.")
      )
    ),
    creator(false, "Cat Cole", &univ_a),
    block(&email("id1", "zq77@x.org")),
  ]);
}

/// A `\note`'s counter number that an affiliation is labelled with too is, on a name, that name's request for the
/// affiliation (FIDELITY_CHARTER §4), not a mark of the note: Bob Baker's `\author[1]` asks for Univ A, so Ann Able,
/// whose name holds "Corresponding author.", is the one marked and zq77@ is hers. Red (78f r3): the author block's.
/// Witnesses 2005.05695, 2005.14143, 2405.09762, 1403.3797, 2101.00866. Repro
/// sectioning-frontmatter/jheppub_note_counter_number_under_an_affiliation_label (rev78f3 probe a6).
#[test]
fn jheppub_note_counter_number_under_an_affiliation_label() {
  let xml = convert_repro!("jheppub_note_counter_number_under_an_affiliation_label.tex");
  let univ_a = contact("affiliation", "Univ A");
  assert_eq!(creators(&xml), [
    creator(
      true,
      "Ann Able",
      &format!(
        "{} {univ_a} {}",
        contact("note", "Corresponding author."),
        email("id1", "zq77@x.org")
      )
    ),
    creator(false, "Bob Baker", &univ_a),
    creator(
      false,
      "Cat Cole",
      &format!(
        "{} {}",
        contact("affiliation", "Univ B"),
        email("id2", "ccole@y.org")
      )
    ),
  ]);
}
