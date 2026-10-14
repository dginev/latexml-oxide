//! Red/green guards for batch 67b: the kluwer.cls binding (Kluwer Academic Publishers' class, in TeX Live), which
//! OmniBus alone left without `{article}`, `{opening}`, `\surname`, `\newdisplay` (run 349: ~395 papers).
use super::perfect_kernel_batch46::{error_count, warning_count};

fn convert_counted(tex: &str) -> (String, String) {
  latexml::util::test::convert_with(tex, Some("ar5iv.sty"))
}

/// The paper is `{article}`, its title page `{opening}` whose end makes the title (kluwer.cls:707-738); an
/// `\institute` belongs to the authors given since the previous one (:766-769). astro-ph0001398, astro-ph0001217.
#[test]
fn kluwer_opening_article() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/kluwer_opening_article.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let creators: Vec<&str> = xml
    .split("<creator ")
    .skip(1)
    .filter(|c| c.starts_with("role=\"author\"") || c.contains(" role=\"author\">"))
    .map(|c| c.split("</creator>").next().unwrap_or(""))
    .collect();
  assert_eq!(creators.len(), 3, "{xml}");
  assert!(
    creators[0].contains("<personname>Ann Able</personname>")
      && creators[0].contains(">Inst A</contact>"),
    "{xml}"
  );
  for c in &creators[1..] {
    assert!(
      c.contains(">Inst B</contact>") && !c.contains("Inst A"),
      "{xml}"
    );
  }
  assert!(xml.contains("<personname>Bob Baker</personname>"), "{xml}");
  assert!(xml.contains("<personname>Cat Cole</personname>"), "{xml}");
  assert!(xml.contains("<title>A Title</title>"), "{xml}");
  assert!(xml.contains(r#"<keywords name="Keywords:"#), "{xml}");
  assert!(xml.contains(r#"class="ltx_theorem_guess""#), "{xml}");
  assert!(xml.contains("A proof."), "{xml}");
}
