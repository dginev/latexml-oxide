//! Red/green guards for batch 107c (census138 ADDR-a and LEGEND; Bundle 5; OXIDIZED_DESIGN_DIVERGENCES #966, #961 and
//! #969, amended). Two code paths at the frontmatter flush, where every route that makes an entry meets. (1) The Kind
//! step for an entry's address line (`insert::kind_address_lines`, its cut `address_line_cut` over
//! `resolver::address_runs`): an unlabelled tail of addresses after a `;`, a comma or a full stop, a label of contact
//! words and "to", the lines a font wrapper holds, and a list's other printed forms (square brackets, a spaced `@`,
//! pieces joined to the tail after the brace) — each address leaves the affiliation for the Link step's placement
//! (FIDELITY_CHARTER §4). (2) The Kind steps for a legend, a link and a dedication (`insert::kind_legend_contacts` with
//! `notes::classify_thanks`' role phrases, `insert::kind_link_and_grade_contacts`, `insert::kind_dedication_lines`): a
//! marked legend is its carriers' note, a link line (labelled or not, alone or ending an entry) a URL or ORCID contact,
//! a dedication the paper's pubnote. Each guard converts its repro under
//! `tools/perfect_kernel/repros/sectioning-frontmatter/` from the binary, eager and `--streaming`, with 0 errors and 0
//! warnings, and asserts the frontmatter whole. Round 2 (review rev107c): the Link step's only-author step and glued
//! marks, the correspondence note a label's role phrase makes (`insert::write_role_notes`), a marked dedication line its
//! carriers' note, a dedication's end, and the notes' and first e-mail line's print order.
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

/// An unlabelled tail of addresses after a list's or a sentence's boundary leaves its affiliation (authblk's unkeyed
/// `\affil{Dept A, Univ A; aable@a.edu}`, `\affil{Univ B, City, bbaker@b.edu}`, `\affil{Univ C\authorcr Street 1,
/// Town. \textup{\small\{ccole,\,dd\}@c.edu}}`): each address its spelled author's e-mail, `dd@` the block creator's
/// (2602.07392, 1701.07541, 1903.04089, 1204.4622). Distinct: the cut's words-before branch with no label.
#[test]
fn an_address_tail_after_a_boundary_leaves_its_affiliation() {
  assert_eq!(
    converted(repro!("addr_tail_authblk_after_comma_and_stop"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Dept A, Univ A</contact> <contact name="Email: " role="email">aable@a.edu</contact> <contact name="Affiliation: " role="affiliation">Univ B, City</contact> <contact name="Affiliation: " role="affiliation">Univ C<break/>Street 1, Town.</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Dept A, Univ A</contact> <contact name="Affiliation: " role="affiliation">Univ B, City</contact> <contact name="Email: " role="email">bbaker@b.edu</contact> <contact name="Affiliation: " role="affiliation">Univ C<break/>Street 1, Town.</contact> </creator> <creator before=" " role="author"> <personname>Cat Cole</personname> <contact name="Affiliation: " role="affiliation">Dept A, Univ A</contact> <contact name="Affiliation: " role="affiliation">Univ B, City</contact> <contact name="Affiliation: " role="affiliation">Univ C<break/>Street 1, Town.</contact> <contact name="Email: " role="email"><text fontsize="90%" xml:id="id1">ccole@c.edu</text></contact> </creator> <creator role="author"> <contact name="Email: " role="email"><text fontsize="90%" xml:id="id2">dd@c.edu</text></contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
}

/// A list's other printed forms are read as the braced list is, so the line leaves the affiliation: IEEEtran's
/// `Email: [ann.able, bob.baker, cat.cole]@a.it` (2405.17925); `{\tt\small \{aable@link., bbaker@link., ccole@\}a.edu.cn}`
/// (2108.00873, 2605.01350); `\textsuperscript{}\{dan.dee, eve.eng\}.civ@x.mil` after a blockA, an empty superscript
/// printing nothing (1908.06816). Distinct: the list reader (`address_runs`, `domain_after_group`).
#[test]
fn a_list_in_its_other_printed_forms_leaves_its_affiliation() {
  assert_eq!(
    converted(repro!("addr_line_ieee_bracket_list"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Univ A, Town, Italy</contact> <contact name="Email: " role="email">ann.able@a.it</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Univ A, Town, Italy</contact> <contact name="Email: " role="email">bob.baker@a.it</contact> </creator> <creator before=" " role="author"> <personname>Cat Cole</personname> <contact name="Affiliation: " role="affiliation">Univ A, Town, Italy</contact> <contact name="Email: " role="email">cat.cole@a.it</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
  assert_eq!(
    converted(repro!("addr_line_group_domain_after_brace"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Univ A</contact> <contact name="Email: " role="email"><text font="typewriter" fontsize="90%" xml:id="id1">aable@link.a.edu.cn</text></contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Univ A</contact> <contact name="Email: " role="email"><text font="typewriter" fontsize="90%" xml:id="id2">bbaker@link.a.edu.cn</text></contact> </creator> <creator before=" " role="author"> <personname>Cat Cole</personname> <contact name="Affiliation: " role="affiliation">Univ C</contact> <contact name="Email: " role="email"><text font="typewriter" fontsize="90%" xml:id="id3">ccole@a.edu.cn</text></contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
  assert_eq!(
    converted(repro!("addr_line_ieee_shared_tail_list"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Dan Dee</personname> <contact name="Affiliation: " role="affiliation">Army Lab, Town</contact> <contact name="Email: " role="email">dan.dee.civ@x.mil</contact> </creator> <creator before=" " role="author"> <personname>Eve Eng</personname> <contact name="Affiliation: " role="affiliation">Army Lab, Town</contact> <contact name="Email: " role="email">eve.eng.civ@x.mil</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
}

/// The lines a font declaration wraps together are read as the entry's (`\scriptsize{$^1$Univ A, Town}\\
/// \scriptsize{\{aable, bbaker\} @ a.edu}\\ …`), and a list spaced round its `@` is a list: each address line leaves
/// the affiliation it is glued to (2104.11106, 1707.04775, 2603.02131). Distinct: `entry_lines` through a wrapper.
#[test]
fn address_lines_a_font_wrapper_holds_leave_their_affiliation() {
  assert_eq!(
    converted(repro!("addr_line_spaced_at_and_font_wrapped_lines"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation"><text fontsize="70%" xml:id="id1">Univ A, Town</text></contact> <contact name="Email: " role="email"><text fontsize="70%" xml:id="id2">aable@a.edu</text></contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation"><text fontsize="70%" xml:id="id3">Univ A, Town</text></contact> <contact name="Email: " role="email"><text fontsize="70%" xml:id="id4">bbaker@a.edu</text></contact> </creator> <creator before=" " role="author"> <personname>Cat Cole</personname> <contact name="Affiliation: " role="affiliation"><text fontsize="70%" xml:id="id5">Univ C, City</text></contact> <contact name="E-mail: " role="email"><text font="typewriter" xml:id="id6">ccole@c.edu</text></contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
}

/// Contact words and "to" before an address, no colon, are its label (`$^{*}$ Corresponding\ to\ bbaker@a.edu`): the
/// line leaves the affiliation, its e-mail the name's carrying `*` (2606.31381); and a correspondence phrase opening an
/// e-mail's label, by any route, is the owner's correspondence note before the e-mail, with the line's mark, the e-mail
/// keeping the label's rest or its role's name (review rev107c N4, the main session's Q6 reading of FIDELITY_CHARTER
/// §4). Distinct: the label reading (`address_line_cut`) and the note (`insert::write_role_notes`).
#[test]
fn a_correspondence_label_is_its_owners_note() {
  assert_eq!(
    converted(repro!("addr_line_corresponding_to"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Univ A,<break/>Town</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker<sup xml:id="id1">∗</sup></personname> <contact name="Affiliation: " role="affiliation">Univ A,<break/>Town</contact> <note class="ltx_note_frontmatter ltx_thanks_correspondence" mark="*" role="thanks" xml:id="id2"><sup xml:id="id2.1">*</sup>Corresponding to</note> <contact name="Email: " role="email">bbaker@a.edu</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
}

/// A paper of one author makes no block creator: an address line whose mark no name shows or requests
/// (`{\small * Corresponding author, E-mail: qz@a.org}` under `Ann Able$^{a}$*`) is the only author's — FIDELITY_CHARTER
/// §4's "the group's only author" before the block (1711.00192, review rev107c B1); and a mark glued to a name as text
/// (`Ann Able$^{a}$*`) is that name's when a line opens with it, so the same line under two names is Ann's. The role
/// phrase before the label is the owner's note (`write_role_notes`). A nameless creator holding names the reader did not
/// part makes it no one-author paper (1102.4967: the addresses stay there). Distinct: the Link step's only-author step and
/// its glued-mark reading (`link_entry_addresses`, `glued_name_marks`).
#[test]
fn a_sole_or_glued_marked_authors_address_is_never_the_blocks() {
  assert_eq!(
    converted(
      repro!("addr_line_one_named_author_beside_unparted_names"),
      0
    ),
    r##"<title>T</title> <creator role="author"> <personname>Cat Cole<sup xml:id="id1">‡</sup></personname> <contact name="Affiliation: " role="affiliation">Univ C, Village</contact> <contact name="Email: " role="email">ccole@c.edu</contact> </creator> <creator role="author"> <contact name="Affiliation: " role="affiliation">Ann Able, Bob Baker<sup xml:id="id2">†</sup> and</contact> <contact name="Affiliation: " role="affiliation">Univ A, Town</contact> <contact name="Affiliation: " role="affiliation">Univ B, City</contact> <contact name="Email: " role="email">aable@a.edu</contact> <contact name="Email: " role="email">bbaker@b.edu</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
  assert_eq!(
    converted(repro!("addr_line_sole_author_unanswered_mark"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able*</personname> <contact name="Affiliation: " role="affiliation"><text fontsize="90%" xml:id="id1">Univ A, Town,<break/>Country.</text></contact> <note class="ltx_note_frontmatter ltx_thanks_correspondence" mark="*" role="thanks" xml:id="id2"><sup xml:id="id2.1">*</sup>Corresponding author</note> <contact name="E-mail: " role="email"><text fontsize="90%" xml:id="id3">qz@a.org</text></contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
  assert_eq!(
    converted(repro!("addr_line_mark_glued_to_name"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able*</personname> <contact name="Affiliation: " role="affiliation"><text fontsize="90%" xml:id="id1">Univ A, Town,<break/>Country.</text></contact> <note class="ltx_note_frontmatter ltx_thanks_correspondence" mark="*" role="thanks" xml:id="id2"><sup xml:id="id2.1">*</sup>Corresponding author</note> <contact name="E-mail: " role="email"><text fontsize="90%" xml:id="id3">qz@a.org</text></contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation"><text fontsize="90%" xml:id="id4">Univ A, Town,<break/>Country.</text></contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
}

/// A marked legend's role phrase is read (`$^{*}$Joint co-authors, $^{**}$Joint senior co-authors`,
/// `\textsuperscript{\dag}First author`, `\textsuperscript{$\star$}Work performed during internship at Acme`): each
/// line its carriers' contribution note, never an affiliation (2403.05680, 2609.03429, 2503.04130). Distinct:
/// `classify_thanks`' role reading.
#[test]
fn marked_role_phrases_are_their_carriers_notes() {
  assert_eq!(
    converted(repro!("legend_kind_role_phrases"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Univ A</contact> <note class="ltx_note_frontmatter ltx_thanks_contribution" mark="∗" role="thanks" xml:id="id1"><sup xml:id="id1.1"><text font="italic" xml:id="id1.1.1">∗</text></sup>Joint co-authors</note> <note class="ltx_note_frontmatter ltx_thanks_contribution" mark="†" role="thanks" xml:id="id2"><sup xml:id="id2.1">†</sup>First author</note> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Univ A</contact> <note class="ltx_note_frontmatter ltx_thanks_contribution" mark="∗" role="thanks" xml:id="id3"><sup xml:id="id3.1"><text font="italic" xml:id="id3.1.1">∗</text></sup>Joint co-authors</note> </creator> <creator before=" " role="author"> <personname>Cat Cole</personname> <contact name="Affiliation: " role="affiliation">Univ B</contact> <note class="ltx_note_frontmatter ltx_thanks_contribution" mark="∗∗" role="thanks" xml:id="id4"><sup xml:id="id4.1"><text font="italic" xml:id="id4.1.1">∗∗</text></sup>Joint senior co-authors</note> </creator> <creator before=" " role="author"> <personname>Dan Dee</personname> <contact name="Affiliation: " role="affiliation">Univ B</contact> <note class="ltx_note_frontmatter ltx_thanks_contribution" mark="⋆" role="thanks" xml:id="id5"><sup xml:id="id5.1"><Math mode="inline" tex="\star" text="star" xml:id="m4"> <XMath xml:id="m4.1"> <XMTok name="star" role="MULOP">⋆</XMTok> </XMath> </Math></sup>Work performed during internship at Acme</note> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
}

/// A link line is a link contact, never an affiliation: one labelled `Project website:` ending a blockA's last entry is
/// its holder's, named by the label (2609.03591's shape inside an entry); one in a blockA of its own, every author
/// holding it, the block creator's, once (2609.03591, 2502.19820); an author's own block of `\url` lines one contact a
/// line, the `orcid.org` one an ORCID contact (2206.10577). Distinct: `kind_link_and_grade_contacts`' label and line
/// readings (`link_label`, `split_link_lines`).
#[test]
fn link_lines_are_link_contacts() {
  assert_eq!(
    converted(repro!("legend_kind_labelled_link_line"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able<sup xml:id="id1">1</sup></personname> <contact name="Affiliation: " role="affiliation">Univ A</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker<sup xml:id="id2">2</sup></personname> <contact name="Affiliation: " role="affiliation">Univ B</contact> <contact name="Project website: " role="url"><ref class="ltx_href" fontsize="90%" href="https://x.org/p">https://x.org/p</ref></contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
  assert_eq!(
    converted(repro!("legend_kind_labelled_link_line_is_the_blocks"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able<sup xml:id="id1">1</sup></personname> <contact name="Affiliation: " role="affiliation">Univ A</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker<sup xml:id="id2">2</sup></personname> <contact name="Affiliation: " role="affiliation">Univ B. </contact> </creator> <creator role="author"> <contact name="Project website: " role="url"><ref class="ltx_href" color="#FF0000" fontsize="90%" href="https://x.org/p">https://x.org/p</ref></contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
  assert_eq!(
    converted(repro!("legend_kind_link_lines_entry"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able<sup xml:id="id1">*</sup></personname> <contact name="Email: " role="email"><ref class="ltx_href" href="mailto:ann@a.org">ann@a.org</ref></contact> <contact role="orcid"><ref class="ltx_url" font="typewriter" href="https://orcid.org/0000-0002-1825-0097">https://orcid.org/0000-0002-1825-0097</ref></contact> <contact name="URL: " role="url"><ref class="ltx_url" font="typewriter" href="https://a.org/ann">https://a.org/ann</ref></contact> <contact name="Affiliation: " role="affiliation">Univ A</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker<sup xml:id="id2">†</sup></personname> <contact name="Email: " role="email"><ref class="ltx_href" href="mailto:bob@b.org">bob@b.org</ref></contact> <contact name="Affiliation: " role="affiliation">Univ B</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
}

/// A dedication set as an author block's last lines (`\centerline{This work is dedicated to Professor C. Dee}\\
/// \centerline{on the occasion of his birthday}`) is the paper's dedication pubnote, once, after the creators; the
/// affiliation it was glued to keeps its own line (0910.1386). Distinct: `kind_dedication_lines`.
///
/// It ends at the first later line that reads as something else ("Present address: …", "Supported by NSF grant 123"),
/// which stays where it was (review rev107c S2); its lines' breaks are one each (N3). A line led by a mark a name
/// requests is no dedication but its carriers' note, as "Deceased" is — the observer's legend reading
/// (`$^\dagger$In memory of Bob Baker, …`) and the flush's (`$^2$Dedicated to the memory of …`, `legend_mark`): review
/// rev107c S1.
#[test]
fn a_dedication_is_the_papers_pubnote() {
  assert_eq!(
    converted(repro!("legend_kind_dedication_ends_at_other_lines"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Univ A, Town</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Univ B, City<break/>Present address: Univ C, Village<break/>Supported by NSF grant 123</contact> </creator> <pubnote role="dedication">This work is dedicated to Professor C. Dee<break/>on the occasion of his birthday</pubnote> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
  assert_eq!(
    converted(repro!("legend_kind_marked_dedication_line"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Univ A, Town</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Univ A, Town</contact> <note class="ltx_note_frontmatter ltx_thanks_note" mark="†" role="thanks" xml:id="id1"><sup xml:id="id1.1">†</sup>In memory of Bob Baker, who passed away during this work</note> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
  assert_eq!(
    converted(repro!("legend_kind_marked_dedication_entry"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Univ A, Town</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Univ A, Town</contact> <note class="ltx_note_frontmatter ltx_thanks_note" mark="2" role="thanks" xml:id="id1">Dedicated to the memory of his teacher C. Dee</note> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
  assert_eq!(
    converted(repro!("legend_kind_dedication_line"), 0),
    r##"<title>T</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation"> Univ A</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation"> Univ B</contact> </creator> <pubnote role="dedication"><text class="ltx_align_center" xml:id="id1">This work is dedicated to Professor C. Dee</text><break/><text class="ltx_align_center" xml:id="id2">on the occasion of his birthday</text></pubnote> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"##
  );
}
