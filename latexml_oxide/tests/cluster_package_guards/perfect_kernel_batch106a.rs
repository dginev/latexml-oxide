//! Red/green guards for batch 106a: a line of addresses inside an affiliation-kind entry leaves the entry, its addresses
//! the entry's e-mails (FIDELITY_CHARTER §4; census132 cluster ADDR-a; OXIDIZED_DESIGN_DIVERGENCES #966). Code path:
//! the Kind step for an entry's address line at the flush, where every route that makes the entry meets
//! (frontmatter/insert.rs `kind_address_lines`, its reader `address_line_cut` over `resolver::address_runs`), before the
//! Link step places the e-mails (`link_entry_addresses`). Each guard converts its repro under
//! `tools/perfect_kernel/repros/sectioning-frontmatter/` from the binary, eager and `--streaming`, and asserts the
//! frontmatter it makes whole.
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

/// A whole line of addresses behind an e-mail label, the label the e-mails' name: IEEEtran's blockA line `E-mail:
/// ann.able@gmail.com, \{bbaker, ccole\}@b.edu, and dan@d.org` (a braced group, a connective "and"; 2302.02638), whose
/// unspelled `dan@` is the block's; and a REVTeX author block's last entry, `$^*$These authors contributed equally. \\
/// Email: …` (2207.11193). Each line had stayed inside the affiliation of every holder.
#[test]
fn a_labelled_line_of_addresses_leaves_its_entry() {
  assert_eq!(
    converted(repro!("addr_line_ieee_labelled_list_with_and"), 0),
    r#"<title>T</title> <creator role="author"> <personname>Ann Able<sup xml:id="id1">1</sup></personname> <contact name="Affiliation: " role="affiliation">Univ A, Town</contact> <contact name="E-mail: " role="email">ann.able@gmail.com</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker<sup xml:id="id2">2</sup></personname> <contact name="Affiliation: " role="affiliation"><text font="italic" xml:id="id3">Dept B, Univ B</text>, City</contact> <contact name="E-mail: " role="email">bbaker@b.edu</contact> </creator> <creator before=" " role="author"> <personname>Cat Cole<sup xml:id="id4">2</sup></personname> <contact name="Affiliation: " role="affiliation"><text font="italic" xml:id="id5">Dept B, Univ B</text>, City</contact> <contact name="E-mail: " role="email">ccole@b.edu</contact> </creator> <creator role="author"> <contact name="E-mail: " role="email">dan@d.org</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#
  );
  assert_eq!(
    converted(repro!("addr_line_revtex_author_block_email_line"), 0),
    r#"<title>T</title> <creator role="author"> <personname>Ann Able<sup xml:id="id1">∗</sup></personname> <contact name="Affiliation: " role="affiliation">Univ A, Town</contact> <contact name="Email: " role="email">ann.able@a.edu</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker<sup xml:id="id2">∗</sup></personname> <contact name="Affiliation: " role="affiliation">Univ A, Town</contact> <contact name="Email: " role="email">bob.baker@a.edu</contact> </creator> <creator before=" " role="author"> <personname>Cat Cole</personname> <contact name="Affiliation: " role="affiliation">Univ A, Town</contact> <contact name="Affiliation: " role="affiliation">Univ B, City<break/><sup xml:id="id3">*</sup>These authors contributed equally. </contact> </creator> <abstract inlist="toc" name="Abstract" xml:id="abstract1"> <p xml:id="abstract1.1">Abs.</p> </abstract> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#
  );
}

/// authblk's line after `\authorcr`, which the token-level Kind step does not part: `Email: {\tt\{aable, bbaker\}@a.edu}`,
/// each address its spelled author's, in the typewriter it printed in (2111.01631, 2309.05601).
#[test]
fn an_address_line_after_authorcr_leaves_the_affil() {
  assert_eq!(
    converted(repro!("addr_line_authblk_authorcr_email"), 0),
    r#"<title>T</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Univ A, Town</contact> <contact name="Email: " role="email"><text font="typewriter" xml:id="id1">aable@a.edu</text></contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Univ A, Town</contact> <contact name="Email: " role="email"><text font="typewriter" xml:id="id2">bbaker@a.edu</text></contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#
  );
}

/// A marked line of a contact's words before an address — the mark an unparsed formula (`$^{\ast}$`), the address a
/// `\href{mailto:…}` link — leaves the entry, the words the e-mail's name as printed: "Corresponding author: "
/// (2609.21085).
#[test]
fn a_marked_contact_label_line_leaves_the_affil() {
  assert_eq!(
    converted(repro!("addr_line_authblk_corresponding_author_mark"), 0),
    r#"<title>T</title> <creator role="author"> <personname>Ann Able<sup xml:id="id1"><text font="italic" xml:id="id1.1">∗</text></sup></personname> <contact name="Affiliation: " role="affiliation">Univ A, Town</contact> <contact name="Corresponding author: " role="email"><text font="typewriter" xml:id="id2">ann.able@a.edu</text></contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Univ A, Town</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#
  );
}

/// A line whose words come before an e-mail label keeps its words: the Science template's `$^\ast$To whom
/// correspondence should be addressed; E-mail: ann.able@a.edu` run on into the last affiliation entry gives up its label
/// and address alone (1111.6932, 1808.02718, astro-ph0306103).
#[test]
fn an_email_label_after_a_legends_words_leaves_with_its_address() {
  assert_eq!(
    converted(repro!("addr_line_article_block_legend_line_tail"), 0),
    r#"<title>T</title> <creator role="author"> <personname>Ann Able<sup xml:id="id1">∗</sup></personname> <contact name="Affiliation: " role="affiliation">Univ A, Dept A,<break/>Town A</contact> <contact name="E-mail: " role="email">ann.able@a.edu</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Univ A, Dept A,<break/>Town A</contact> <contact name="Affiliation: " role="affiliation">Univ B, Dept B<break/>City B<break/><sup xml:id="id2">∗</sup>To whom correspondence should be addressed</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#
  );
}

/// The fail-safe: an address beside words with no label (`see data@a.org for data`), a run tags follow (`ann.able@a.edu
/// (Ann)`, 2512.10975) and a template, bare or labelled (`\{firstname.lastname\}@a.edu`, 2009.04283; `Contact:
/// forename.surname@a.edu`, 2410.09681, review rev106a S2), are no lines of addresses; the entry stays as it printed,
/// every author's.
#[test]
fn unlabelled_tagged_and_template_lines_stay() {
  assert_eq!(
    converted(repro!("addr_line_ieee_unlabelled_and_tagged_lines_stay"), 0),
    r#"<title>T</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Univ A, Town, see data@a.org for data<break/>ann.able@a.edu (Ann), bbaker@a.edu (Bob)<break/>{firstname.lastname}@a.edu<break/>Contact: forename.surname@a.edu</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Univ A, Town, see data@a.org for data<break/>ann.able@a.edu (Ann), bbaker@a.edu (Bob)<break/>{firstname.lastname}@a.edu<break/>Contact: forename.surname@a.edu</contact> </creator> <creator before=" " role="author"> <personname>Cat Cole</personname> <contact name="Affiliation: " role="affiliation">Univ A, Town, see data@a.org for data<break/>ann.able@a.edu (Ann), bbaker@a.edu (Bob)<break/>{firstname.lastname}@a.edu<break/>Contact: forename.surname@a.edu</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#
  );
}

/// Marks win (user ruling R1; review rev106a B1): the mark leading a line is the key of its unspelled addresses — the
/// authors whose names answer it own them, never the only-carrier or order step. gr-qc0207058's REVTeX block `$**$
/// email: cx@x.org, c\_dee@y.org`: `cx@` is C. Dee's, whose name carries `∗∗` (an asterisk run one key, every form one:
/// user ruling 2026-11-02), not Ann Bee's of `∗a),b)`; and authblk's numbers before each address, `Emails: $^1$zz@x.org,
/// $^2$qq@y.org` in Carl Dee's entry alone: `zz@` is Ann Bee's, whose name requests `1` (1706.04764, 2505.11901).
#[test]
fn a_line_mark_gives_its_addresses_to_the_names_answering_it() {
  assert_eq!(
    converted(repro!("addr_line_revtex_mark_led_lines"), 0),
    r#"<title>T</title> <creator role="author"> <personname>Ann Bee<sup xml:id="id1"><text font="italic" xml:id="id1.1">∗a),b)</text></sup></personname> <contact name="Affiliation: " role="affiliation">a) Univ X, West Bengal,</contact> <contact name="Affiliation: " role="affiliation">India, Pin.-741235</contact> <contact name="Affiliation: " role="affiliation">b) IUCAA, Pune 411 007, India</contact> <contact name="email: " role="email">abee@x.org</contact> </creator> <creator before=" " role="author"> <personname>C. Dee<sup xml:id="id2"><text font="italic" xml:id="id2.1">∗∗a)</text></sup></personname> <contact name="Affiliation: " role="affiliation">a) Univ X, West Bengal,</contact> <contact name="Affiliation: " role="affiliation">India, Pin.-741235</contact> <contact name="Affiliation: " role="affiliation">b) IUCAA, Pune 411 007, India</contact> <contact name="email: " role="email">cx@x.org</contact> <contact name="email: " role="email">c_dee@y.org</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Hello.</p> </para>"#
  );
  assert_eq!(
    converted(repro!("addr_line_authblk_numbered_marks"), 0),
    r#"<title>T</title> <creator role="author"> <personname>Ann Bee</personname> <contact name="Affiliation: " role="affiliation">Univ X</contact> <contact name="Emails: " role="email">zz@x.org</contact> </creator> <creator before=" " role="author"> <personname>Carl Dee</personname> <contact name="Affiliation: " role="affiliation">Univ Y</contact> <contact name="Emails: " role="email">qq@y.org</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Hello.</p> </para>"#
  );
}

/// A line's mark no name answers stays printed (review rev106a S1): authblk's `\affil[1]{Univ X\authorcr
/// $^\dagger$Email: zq@x.org, ab@x.org}` under two names carrying no `†` — `ab@` its spelled Ann Bee's, `zq@` the
/// block's (an unanswered mark, so neither carrier's), the `†` before it.
#[test]
fn an_unanswered_line_mark_stays_printed_before_its_address() {
  assert_eq!(
    converted(repro!("addr_line_authblk_unanswered_mark_shown"), 0),
    r#"<title>T</title> <creator role="author"> <personname>Ann Bee</personname> <contact name="Affiliation: " role="affiliation">Univ X</contact> <contact name="Email: " role="email">ab@x.org</contact> </creator> <creator before=" " role="author"> <personname>Carl Dee</personname> <contact name="Affiliation: " role="affiliation">Univ X</contact> </creator> <creator role="author"> <contact name="Email: " role="email"><sup xml:id="id1">†</sup>zq@x.org</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Hello.</p> </para>"#
  );
}
