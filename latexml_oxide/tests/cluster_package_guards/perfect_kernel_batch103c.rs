//! Red/green guards for batch 103c: a legend line read as an affiliation is its carriers' note (FIDELITY_CHARTER §4,
//! author annotations are notes; census129 cluster 6; OXIDIZED_DESIGN_DIVERGENCES #961). Code paths: the Kind step for
//! a legend where every route that makes an affiliation meets, once the frontmatter is placed and the observer's
//! writing done (frontmatter/insert.rs `kind_legend_contacts`, its reader `entry_legend_kind`); and `classify_thanks`'s
//! project roles and the work's circumstances (`project_role`, `opens_with_work_done`), which the frontmatter
//! observer's legend reading (`is_legend_text`) shares. Each guard converts its repro under
//! `tools/perfect_kernel/repros/sectioning-frontmatter/` from the binary, eager and `--streaming`, and asserts the
//! frontmatter it makes whole.
use latexml::util::test::{convert_with_setup_then, error_count, warning_count};

use super::{perfect_kernel_batch84b::text, perfect_kernel_batch99b::converted};

/// The frontmatter notes of `tex`'s creators, converted in process (clean): per creator its personname's text, then
/// each `ltx:note` it holds as `note[<mark>]=<text>`, in order — the view a guard's contacts view (`creators`) leaves
/// out (review rev103c S3: `perfect_kernel_batch92b`, `perfect_kernel_batch95b`).
#[track_caller]
pub(super) fn creator_notes(tex: &str) -> Vec<Vec<String>> {
  let (log, xml, ()) = convert_with_setup_then(tex, Some("ar5iv.sty"), || {}, |_| {});
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  xml
    .match_indices("<creator ")
    .map(|(start, _)| {
      let end = start + xml[start..].find("</creator>").expect("closed creator");
      let creator = &xml[start..end];
      let mut out = Vec::new();
      if let Some(at) = creator.find("<personname>") {
        let close = at
          + creator[at..]
            .find("</personname>")
            .expect("closed personname");
        out.push(text(&creator[at + "<personname>".len()..close]));
      }
      for (at, _) in creator.match_indices("<note ") {
        let open_end = at + creator[at..].find('>').expect("note tag");
        let mark = creator[at..open_end]
          .split("mark=\"")
          .nth(1)
          .and_then(|m| m.split('"').next())
          .unwrap_or("");
        let close = open_end + creator[open_end..].find("</note>").expect("closed note");
        out.push(format!(
          "note[{mark}]={}",
          text(&creator[open_end + 1..close])
        ));
      }
      out
    })
    .collect()
}

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
/// label it was placed by where that prints (authblk prints its label before the line; IEEEtran's number shows in the
/// names, review rev103c2 B-A): authblk's `\affil[$\dagger$]{These authors contributed equally.}` and
/// `\affil[*]{Corresponding author: bob@a.edu}` (its address still Bob Baker's e-mail contact; 2409.05386, 2603.21906,
/// wlscirep's 1611.06677) and IEEEtran's `\IEEEauthorblockA{\IEEEauthorrefmark{3}These authors contributed equally}` (a
/// number its mark, as the names show it; 2410.12184). Each had been an affiliation contact of the same carriers. An
/// unlabelled `\affil[ ]{Corresponding author: Ann Able (ann@a.edu)}` prints no mark and stays the affiliation it was,
/// as int130a writes it (a note shows its text only behind its mark; review rev103c B1, main-session ruling
/// 2026-11-04), the address Ann Able's as it spells her (2412.14916, 2305.02090).
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
    r#"<title>T</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Univ A, Town</contact> <contact name="Email: " role="email">ann@a.edu</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Univ A, Town</contact> <contact name="Affiliation: " role="affiliation">Corresponding author: Ann Able</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#
  );
  // (a labelled legend no author requests sits on a nameless creator, which carries for every author: the name it
  // spells is an author's, review rev103c S2; 2609.08556, 1507.04774, 2503.10710)
  assert_eq!(
    converted(
      repro!("legend_kind_authblk_unrequested_label_names_an_author"),
      0
    ),
    r#"<title>T</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Univ A, Town</contact> <contact name="Email: " role="email">ann@a.edu</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Univ A, Town</contact> </creator> <creator role="author"> <note class="ltx_note_frontmatter ltx_thanks_correspondence" mark="†" role="thanks" xml:id="id1">Corresponding author: Ann Able</note> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#
  );
  // (a legend under several labels is each carrier's note under the mark its own name prints: authblk's
  // `\affil[1,2]{These authors contributed equally.}` is Ann Able's "1" and Bob Baker's "2", review rev103c2 B-A;
  // 2301.13111)
  assert_eq!(
    converted(
      repro!("legend_kind_authblk_multilabel_legend_marks_each_carrier"),
      0
    ),
    r#"<title>T</title> <creator role="author"> <personname>Ann Able</personname> <note class="ltx_note_frontmatter ltx_thanks_contribution" mark="1" role="thanks" xml:id="id1">These authors contributed equally.</note> <contact name="Affiliation: " role="affiliation">Univ A, Town</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <note class="ltx_note_frontmatter ltx_thanks_contribution" mark="2" role="thanks" xml:id="id2">These authors contributed equally.</note> <contact name="Affiliation: " role="affiliation">Univ A, Town</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#
  );
  assert_eq!(
    converted(repro!("legend_kind_ieee_numbered_legend"), 0),
    r#"<title>T</title> <creator role="author"> <personname>Ann Able<sup xml:id="id1">*</sup><sup xml:id="id2">‡</sup></personname> <contact name="Affiliation: " role="affiliation">Univ A, Town</contact> <note class="ltx_note_frontmatter ltx_thanks_contribution" mark="‡" role="thanks" xml:id="id3">These authors contributed equally</note> </creator> <creator before=" " role="author"> <personname>Bob Baker<sup xml:id="id4">†</sup><sup xml:id="id5">‡</sup></personname> <contact name="Affiliation: " role="affiliation">Univ B, City</contact> <note class="ltx_note_frontmatter ltx_thanks_contribution" mark="‡" role="thanks" xml:id="id6">These authors contributed equally</note> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#
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
}

/// Controls: an institution named with a legend's words stays an affiliation — "Facebook (work done while at JHU/APL)"
/// (1702.02684: the work's circumstances not opening it), "Center for Equal Opportunity", "Department of Equal
/// Contribution Studies, Univ C" and "Project Lead Institute, Univ D" (`reads_as_affiliation`), and "Equal contribution,
/// Google DeepMind", a company's name shaped as a person's that no carrier has (`spells_a_carrier`, review rev103c S2).
/// The same names on the kernel `\author` block's symbol-led lines, which the line route and the frontmatter observer
/// read (`$^{\dagger}$Project Lead Institute, Univ D`, `$^{\ddagger}$Core Contributor Network, Paris`, `$^{*}$Senior
/// Author Program, Acme Corp`): each its carrier's affiliation, as int130a writes it (`classify_thanks` reads a role only
/// where every piece of the text is a legend, the gate of every reader of the kind; review rev103c S1).
#[test]
fn institutions_named_with_legend_words_stay_affiliations() {
  assert_eq!(
    converted(
      repro!("legend_kind_institutions_named_with_role_words_stay"),
      0
    ),
    r#"<title>T</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Facebook (work done while at JHU/APL)</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Center for Equal Opportunity</contact> </creator> <creator before=" " role="author"> <personname>Cat Cole</personname> <contact name="Affiliation: " role="affiliation">Department of Equal Contribution Studies, Univ C</contact> </creator> <creator before=" " role="author"> <personname>Dan Dee</personname> <contact name="Affiliation: " role="affiliation">Project Lead Institute, Univ D</contact> </creator> <creator before=" " role="author"> <personname>Eve Eyre</personname> <contact name="Affiliation: " role="affiliation">Equal contribution, Google DeepMind</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#
  );
}

/// A legend whose label prints no mark stays the affiliation it is, as int130a writes it (a note's text shows only
/// behind its mark, and a label no name prints is a mark the page never shows; review rev103c2 B-A): icml's
/// `\icmlaffiliation{intern}{Work done during an internship}`, a key icml prints as the affiliation's number
/// (2106.09776), and svjour3's `\institute{… \and B. Baker (corresponding author, \email{…}) \and …}`, labelled by its
/// place, a number neither svjour3 nor svjour_blank prints (1507.04774).
#[test]
fn a_legend_whose_label_prints_no_mark_stays_an_affiliation() {
  assert_eq!(
    converted(repro!("legend_kind_icml_keyed_legend_stays"), 0),
    r#"<title>T</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Univ A, Town</contact> <contact name="Affiliation: " role="affiliation">Work done during an internship</contact> <contact name="Correspondence to: " role="email">ann@a.org</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Univ A, Town</contact> </creator> <note role="affiliationnotice" xml:id="id1">Equal contribution</note> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#
  );
}
