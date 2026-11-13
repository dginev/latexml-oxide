//! Guards for UNC: losses no Warning counted. The kernel `\thanks` notes printing no word are rows of
//! `perfect_kernel_batchhg1::reds_keep_their_counts_and_right_parts` (each row its content-check Warning); here the
//! IEEEtran grade outside the author fields (counted, and its safe side) and the vmsta 2018 e-mails (kept).

use std::borrow::Cow;

use super::perfect_kernel_batchhg1::{Creators, Row, assert_rows};

/// A repro of the catalog (`tools/perfect_kernel/repros/<path>.tex`), its name the row's.
macro_rules! repro {
  ($path:literal) => {
    (
      $path,
      Cow::Borrowed(include_str!(concat!(
        "../../../tools/perfect_kernel/repros/",
        $path,
        ".tex"
      ))),
    )
  };
}

/// Each row as pdflatex prints it (the repro headers name the oracle), one per code path:
/// - under `conference` the class locks `\IEEEmembership` out (IEEEtran.cls:6270) and pdflatex prints no grade, in the
///   names, the biography heading or the body: nothing counted, no membership contact (the journal case, each dropped
///   grade counted: `perfect_kernel_batch63::ieee_membership_outside_the_names_stays_dropped`);
/// - vmsta's 2018+ `\author[a]{…\ead[label=e1]{…}}` (1805.02511, 1810.09681, 2001.02487): each e-mail its author's
///   contact, read in the rehearsal (vmsta_cls.rs `\ead`).
#[test]
fn uncounted_losses_are_counted_or_kept() {
  let journal = repro!("sectioning-frontmatter/ieee_membership_outside_the_names_stays_dropped");
  let conference = journal
    .1
    .replace("[journal]{IEEEtran}", "[conference]{IEEEtran}");
  assert_ne!(conference, journal.1);
  assert_rows(&[
    Row {
      repro:    (
        "ieee_membership_outside_the_names_locked_out_in_conference",
        Cow::Owned(conference),
      ),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname>Wenming Li</personname></creator>"#,
        r#"<creator before="  " role="author"><personname>Xiaochun Ye</personname></creator>"#,
        r#"<creator before="  " role="author"><personname>Dongrui Fan</personname></creator>"#,
        r#"<creator before="  " role="author"><personname>Yuan Xie</personname></creator>"#,
      ]),
    },
    Row {
      repro:    repro!("class-bindings/vmsta_2018/vmsta_2018_emails_and_addresses_are_contacts"),
      files:    &[(
        "vmsta.cls",
        include_str!("../../../tools/perfect_kernel/repros/class-bindings/vmsta_2018/vmsta.cls"),
      )],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname>Yuliya Mishura</personname><contact name="Email: " role="email">myus@univ.kiev.ua</contact><contact name="Affiliation: " role="affiliation">Univ A</contact></creator>"#,
        r#"<creator before="  " role="author"><personname>Ivan Voronov</personname><contact name="Email: " role="email">ivan@univ.kiev.ua</contact><contact name="Affiliation: " role="affiliation">Univ B</contact></creator>"#,
      ]),
    },
  ]);
}
