//! Red/green guards for batch 103c: a legend line read as an affiliation is its carriers' note (FIDELITY_CHARTER §4,
//! author annotations are notes; census129 cluster 6; OXIDIZED_DESIGN_DIVERGENCES #961). Code paths: the Kind step for
//! a legend where every route that makes an affiliation meets, once the frontmatter is placed and the observer's
//! writing done (frontmatter/insert.rs `kind_legend_contacts`, its reader `entry_legend_kind`); and `classify_thanks`'s
//! project roles and the work's circumstances (`project_role`, `opens_with_work_done`), which the frontmatter
//! observer's legend reading (`is_legend_text`) shares. Each guard converts its repro under
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

/// An affiliation a binding or an author block placed whose whole text is a legend is its carriers' note, its mark the
/// label it was placed by: authblk's `\affil[$\dagger$]{These authors contributed equally.}` and `\affil[*]{Corresponding
/// author: bob@a.edu}` (its address still Bob Baker's e-mail contact; 2409.05386, 2603.21906, wlscirep's 1611.06677), an
/// unlabelled `\affil[ ]{Corresponding author: Ann Able (ann@a.edu)}` (no mark, the address Ann Able's as it spells her;
/// 2412.14916, 2305.02090), and IEEEtran's `\IEEEauthorblockA{\IEEEauthorrefmark{3}These authors contributed equally}`
/// (a number its mark; 2410.12184). Each had been an affiliation contact of the same carriers.
#[test]
fn an_affiliation_that_is_a_legend_is_its_carriers_note() {
  assert_eq!(
    converted(repro!("legend_kind_authblk_labelled_legends"), 0),
    r#"<title>T</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Univ A, Town</contact> <note class="ltx_note_frontmatter ltx_thanks_contribution" mark="†" role="thanks" xml:id="id1">These authors contributed equally.</note> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Univ A, Town</contact> <note class="ltx_note_frontmatter ltx_thanks_contribution" mark="†" role="thanks" xml:id="id2">These authors contributed equally.</note> <note class="ltx_note_frontmatter ltx_thanks_correspondence" mark="*" role="thanks" xml:id="id3">Corresponding author</note> <contact name="Email: " role="email">bob@a.edu</contact> </creator> <creator before=" " role="author"> <personname>Cat Cole</personname> <contact name="Affiliation: " role="affiliation">Univ B, City</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#
  );
  assert_eq!(
    converted(
      repro!("legend_kind_authblk_unlabelled_corresponding_author"),
      0
    ),
    r#"<title>T</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Univ A, Town</contact> <contact name="Email: " role="email">ann@a.edu</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Univ A, Town</contact> <note class="ltx_note_frontmatter ltx_thanks_correspondence" mark="" role="thanks" xml:id="id1">Corresponding author: Ann Able</note> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#
  );
  assert_eq!(
    converted(repro!("legend_kind_ieee_numbered_legend"), 0),
    r#"<title>T</title> <creator role="author"> <personname>Ann Able<sup xml:id="id1">1</sup><sup xml:id="id2">3</sup></personname> <contact name="Affiliation: " role="affiliation">Univ A, Town</contact> <note class="ltx_note_frontmatter ltx_thanks_contribution" mark="3" role="thanks" xml:id="id3">These authors contributed equally</note> </creator> <creator before=" " role="author"> <personname>Bob Baker<sup xml:id="id4">2</sup><sup xml:id="id5">3</sup></personname> <contact name="Affiliation: " role="affiliation">Univ B, City</contact> <note class="ltx_note_frontmatter ltx_thanks_contribution" mark="3" role="thanks" xml:id="id6">These authors contributed equally</note> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#
  );
}

/// A project role or a joint first authorship is a contribution, read as a phrase of whole words (`classify_thanks`'s
/// `project_role`): the kernel `\author` block's `$^{\dagger}$Project leader.` is Cat Cole's note (2609.19719,
/// 2609.18514; SYNC row 97c (d)), llncs's `\textsuperscript{$\dagger$}Joint first authors.` the note of the two `†`
/// names beside the `*` name's correspondence note (2608.22772, census129 repro L).
#[test]
fn a_project_role_or_joint_first_authorship_is_a_contribution_note() {
  assert_eq!(
    converted(repro!("legend_kind_article_author_block_project_lead"), 0),
    r#"<title>T</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Univ A, Town</contact> <note class="ltx_note_frontmatter ltx_thanks_contribution" mark="∗" role="thanks" xml:id="id1"><sup xml:id="id1.1"><text font="italic" xml:id="id1.1.1">∗</text></sup>Equal contribution.</note> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Univ A, Town</contact> <note class="ltx_note_frontmatter ltx_thanks_contribution" mark="∗" role="thanks" xml:id="id2"><sup xml:id="id2.1"><text font="italic" xml:id="id2.1.1">∗</text></sup>Equal contribution.</note> </creator> <creator before=" " role="author"> <personname>Cat Cole</personname> <contact name="Affiliation: " role="affiliation">Univ B, City</contact> <note class="ltx_note_frontmatter ltx_thanks_contribution" mark="†" role="thanks" xml:id="id3"><sup xml:id="id3.1"><text font="italic" xml:id="id3.1.1">†</text></sup>Project leader.</note> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#
  );
  assert_eq!(
    converted(repro!("legend_kind_llncs_joint_first_authors"), 0),
    r#"<title>T</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Univ A, Town</contact> <note class="ltx_note_frontmatter ltx_thanks_contribution" mark="†" role="thanks" xml:id="id1"><sup xml:id="id1.1"><Math mode="inline" tex="\dagger" text="dagger" xml:id="m3"> <XMath xml:id="m3.1"> <XMTok name="dagger" role="MULOP">†</XMTok> </XMath> </Math></sup>Joint first authors.</note> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Univ A, Town</contact> <note class="ltx_note_frontmatter ltx_thanks_contribution" mark="†" role="thanks" xml:id="id2"><sup xml:id="id2.1"><Math mode="inline" tex="\dagger" text="dagger" xml:id="m4"> <XMath xml:id="m4.1"> <XMTok name="dagger" role="MULOP">†</XMTok> </XMath> </Math></sup>Joint first authors.</note> </creator> <creator before=" " role="author"> <personname>Cat Cole</personname> <contact name="Affiliation: " role="affiliation">Univ A, Town</contact> <note class="ltx_note_frontmatter ltx_thanks_correspondence" mark="*" role="thanks" xml:id="id3"><sup xml:id="id3.1">*</sup>Corresponding author.</note> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#
  );
}

/// Controls: an institution named with a legend's words stays an affiliation — "Facebook (work done while at JHU/APL)"
/// (1702.02684: the work's circumstances not opening it), "Center for Equal Opportunity", "Department of Equal
/// Contribution Studies, Univ C" and "Project Lead Institute, Univ D" (`reads_as_affiliation`).
#[test]
fn institutions_named_with_legend_words_stay_affiliations() {
  assert_eq!(
    converted(
      repro!("legend_kind_institutions_named_with_role_words_stay"),
      0
    ),
    r#"<title>T</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Facebook (work done while at JHU/APL)</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Center for Equal Opportunity</contact> </creator> <creator before=" " role="author"> <personname>Cat Cole</personname> <contact name="Affiliation: " role="affiliation">Department of Equal Contribution Studies, Univ C</contact> </creator> <creator before=" " role="author"> <personname>Dan Dee</personname> <contact name="Affiliation: " role="affiliation">Project Lead Institute, Univ D</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#
  );
}
