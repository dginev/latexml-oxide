//! Red/green guard for batch 77h: in the title-page deposit (`\lx@deposit@maketitle`, sect05.rs) a rerouted store the
//! paper never set and the class gives no value before its own `\maketitle` is `\relax` (`\lx@captured@stores`,
//! frontmatter_stores.rs), so its undefined store no longer drops the whole deposit and the title-page text beside it
//! (silent-loss inventory 23b: an.cls:532's `\ch@ckobl{institute}{…}`), while a class testing `\@ifundefined` still sees
//! it unset. The guard asserts the three measures: no Error, the authored text present once, its title-block line kept.
use latexml::util::test::convert_files_with;

use super::perfect_kernel_batch46::{error_count, warning_count};

/// An an.cls paper without `\institute` keeps its title block's "Key words" line; the class's own `\maketitle` default
/// for the unset affiliation is its affiliation, as pdflatex prints it (and as before).
#[test]
fn an_without_institute_keeps_the_title_block() {
  let (log, xml) = convert_files_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/class_notes/an_noinst/an_without_institute_keeps_the_title_block.tex"
    ),
    &[(
      "an.cls",
      include_str!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/class_notes/an_noinst/an.cls"
      ),
    )],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let xml = xml.split_whitespace().collect::<Vec<_>>().join(" ");
  assert_eq!(xml.matches("lensing – quasars").count(), 1, "{xml}");
  assert!(
    xml.contains(
      r#"<p xml:id="p1.1"><text font="bold" xml:id="p1.1.1">Key words</text> lensing – quasars</p>"#
    ),
    "{xml}"
  );
  assert_eq!(
    xml.matches(r#"<contact name="Affiliation: " role="affiliation">Address(es) of author(s) should be given</contact>"#).count(),
    1,
    "{xml}"
  );
  assert!(xml.contains("<title>A Title</title>"), "{xml}");
}
