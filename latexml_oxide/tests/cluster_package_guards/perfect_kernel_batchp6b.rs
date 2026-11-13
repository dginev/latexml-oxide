//! P6b (Option C P6, batch 2): one collaboration vocabulary — the words a remark speaking for a group opens with
//! (`SPEAKS_FOR`) and the words naming the group (`COLLABORATION_WORDS`, `TEAM_WORDS`), read by one predicate
//! (`classify::collaborations::collaboration_remark`) wherever a names line reads a remark after a name.

use latexml::util::test::convert_with;

use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch61::creators_of,
};

/// Each row: a names line whose last name speaks for a group, the remark's spelling one the infix reading
/// (`name_before_remark`) lacked before the union — "representing" (repro
/// `author_names_representing_tail_is_a_name`), "Consortia", "teams": red on P6bbase (one creator, Ann Able, with
/// "Bob Baker <remark>" her affiliation), now two creators, each with the remark as their collaboration contact.
/// pdflatex 0 `!` lines each. The two Warnings are the emitter's plain-text fallback the "for" spelling prints on the
/// base too (`author_names_inline_remark_markup_fallback_red`, post-merge).
#[test]
fn a_remark_speaks_for_a_group_in_one_vocabulary() {
  let doc = |author: &str| {
    format!(
      "\\documentclass{{article}}\n\\begin{{document}}\n\\title{{T}}\n\\author{{{author}}}\n\\maketitle\nText.\n\
       \\end{{document}}\n"
    )
  };
  let two = |remark: &str| -> Vec<String> {
    [
      "<creator role=\"author\"><personname>Ann Able",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker",
    ]
    .iter()
    .map(|head| {
      format!("{head}</personname><contact role=\"collaboration\">{remark}</contact></creator>")
    })
    .collect()
  };
  let rows: Vec<(String, Vec<String>, (usize, usize))> = vec![
    (
      include_str!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_names_representing_tail_is_a_name.tex"
      )
      .into(),
      two("representing the XYZ Collaboration"),
      (0, 2),
    ),
    (doc("Ann Able and Bob Baker for the XYZ Consortia"), two("for the XYZ Consortia"), (0, 2)),
    (doc("Ann Able and Bob Baker on behalf of the XYZ teams"), two("on behalf of the XYZ teams"), (0, 2)),
  ];
  let mut differ = Vec::new();
  for (row, (tex, expected, counts)) in rows.iter().enumerate() {
    let (log, xml) = convert_with(tex, Some("ar5iv.sty"));
    let found = (creators_of(&xml), (error_count(&log), warning_count(&log)));
    if found != (expected.clone(), *counts) {
      differ.push(format!("row {row}: {found:#?}\n{log}"));
    }
  }
  assert!(differ.is_empty(), "{}", differ.join("\n\n"));
}
