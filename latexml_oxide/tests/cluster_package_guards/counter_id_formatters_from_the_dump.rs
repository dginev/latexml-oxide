//! Batch 56dh: every counter defines its `\the<ctr>@ID` formatter, and the
//! format dump carries it. Perl's bootstrap makes `\@definecounter` the
//! LOCKED macro `\newcounter` (latex_bootstrap.pool.ltxml:54), so latex.ltx's
//! raw `\def\@definecounter` is refused and every kernel counter goes through
//! `NewCounter`, which always defines `\the<ctr>@ID` (Package.pm:695-703) —
//! as a Token body the dump can serialize. The Rust bootstrap held an
//! unlocked `Let!` (overwritten by the raw definition) and `new_counter` built
//! the formatters as closures (opaque to the dump), so under a class with no
//! binding loaded raw (ptptex, the manptp manual) `\theequation@ID` was
//! undefined at runtime and `subequations` children got the degenerate ids
//! `.1`/`X.1`, colliding across groups; Perl gives `equation1`/`equation1.1`.

/// The formatter is a macro (never `CODE`), the group and its equations
/// carry Perl's ids, and no id degenerates to the `X.` prefix.
#[test]
fn rawclass_subequations_carry_counter_ids() {
  if !latexml::util::test::kpse_has("ptptex.cls") {
    return;
  }
  let tex = std::fs::read_to_string("tests/cluster_regressions/subequations_rawclass_ids.tex")
    .expect("fixture");
  let (stderr, xml) = super::convert_with(&tex, Some("[rawstyles,rawclasses]latexml.sty"));
  assert_eq!(super::error_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "p",
    &[],
    r##"<p>F:macro:-¿equation“csname @equation@ID“endcsname .</p>"##,
  );
  let ids: Vec<&str> = xml
    .split('<')
    .filter(|tag| tag.starts_with("equation ") || tag.starts_with("equationgroup "))
    .filter_map(|tag| {
      tag
        .split("xml:id=\"")
        .nth(1)
        .and_then(|r| r.split('"').next())
    })
    .collect();
  assert_eq!(
    ids,
    [
      "equation1",
      "equation1.1",
      "section1.EGx1",
      "equation1.2",
      "equation1.3"
    ],
    "the equation ids:\n{xml}"
  );
  assert!(
    !xml.contains("xml:id=\"X."),
    "a degenerate `.n` id survived:\n{xml}"
  );
}
