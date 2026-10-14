//! Red/green guards for batch 66j: 2605/2606 sandbox regressions from the run-341/342 root-causer reports
//! (`agent_reports/2026-10-13_sbx2605_regressions.md` C7/C11/C12, `…_sbx2606_regressions.md` N6).
use super::perfect_kernel_batch46::{error_count, warning_count};

fn convert_counted(tex: &str) -> (String, String) {
  latexml::util::test::convert_with(tex, Some("ar5iv.sty"))
}

/// latex.ltx:18587-18592: an option no `\DeclareOption` declared runs its `\ds@<option>` when defined (2605.19951
/// gji.cls, 2606.12015 biom.cls; KNOWN_PERL_ERRORS #657).
#[test]
fn ds_option_handler_without_declareoption() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/loader/ds_option_handler_without_declareoption.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(xml.contains("Extra: yes."), "{xml}");
}

/// revtex4-2's `\@AF@join` for a class that re-dresses `\@email` over it (JASA.cls; 2605.18495, 2605.23416).
#[test]
fn revtex_af_join_from_redressed_email() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/revtex_af_join_from_redressed_email.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let flat = xml.split_whitespace().collect::<Vec<_>>().join(" ");
  assert!(
    flat.contains(
      r#"<personname>Ann Author</personname> <contact name="Thanks: " role="thanks"><ref class="ltx_href" href="mailto:ann@example.org">ann@example.org</ref></contact> </creator>"#
    ),
    "{xml}"
  );
}

/// mathcolor.ltx:20-29 adds no group around its math, so a `\left` and `\right` in two calls pair (2605.11792).
#[test]
fn mathcolor_adds_no_group() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/fonts-nfss/mathcolor_adds_no_group.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    xml.contains(r##"<XMTok color="#FF0000" role="OPEN" stretchy="true">[</XMTok>"##),
    "{xml}"
  );
  assert!(
    xml.contains(r##"<XMTok color="#FF0000" role="CLOSE" stretchy="true">]</XMTok>"##),
    "{xml}"
  );
  // The color is restored after each call: x between them is not red.
  assert!(
    xml.contains(r#"<XMTok font="italic" role="UNKNOWN" xml:id="S0.E1.m1.4">x</XMTok>"#),
    "{xml}"
  );
}

/// The loader's expl3 scope-exit cleanup fires on expl3's own flag, not on `_` being a letter: etextools restores its
/// own catcodes (2606.06323).
#[test]
fn expl_exit_cleanup_not_for_a_package_own_underscore() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/loader/expl_exit_cleanup_not_for_a_package_own_underscore.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(xml.contains("SLASH=12"), "{xml}");
}

/// A revtex4 class has processed its options before its binding defines REVTeX 3's `\ds@draft` & co., so they are let
/// to `\relax`: a later package's undeclared `[draft]` reaches its own `\DeclareOption*` (review 66j #2).
#[test]
fn revtex_option_handlers_retired() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/loader/revtex_option_handlers_retired.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(xml.contains("Option: draft."), "{xml}");
}
