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

/// kluwer.cls defines its theorem and display environments only under the `thms` option (:124-125, :2411-2432):
/// without it a paper's own `\case`, `{rem}` and `\Note` are its own; with it `{thm}` and `{case}` exist.
#[test]
fn kluwer_displays_only_under_thms() {
  let own = "\\documentclass{kluwer}\n\\newcommand{\\case}[1]{CASE<#1>}\n\\newenvironment{rem}{My remark: }{}\n\
             \\newcommand{\\Note}{NOTE-MACRO}\n\\begin{document}\n\\begin{article}\n\\case{one} and \\Note.\n\
             \\begin{rem}Body of rem.\\end{rem}\n\\end{article}\n\\end{document}\n";
  let (log, xml) = convert_counted(own);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(!xml.contains("<theorem"), "{xml}");
  assert!(
    xml.contains("<p xml:id=\"p1.1\">CASE¡one¿ and NOTE-MACRO.\nMy remark: Body of rem.</p>"),
    "{xml}"
  );
  let thms = "\\documentclass[thms]{kluwer}\n\\begin{document}\n\\begin{article}\n\\begin{thm}A theorem.\\end{thm}\n\
              \\begin{defn}A definition.\\end{defn}\n\\begin{case}A case.\\end{case}\n\\end{article}\n\\end{document}\n";
  let (log, xml) = convert_counted(thms);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(xml.contains(r#"class="ltx_theorem_thm""#), "{xml}");
  assert!(xml.contains(r#"class="ltx_theorem_case""#), "{xml}");
  // `\rmtheorem{defn}` (:2424): the body is upright, a theorem's italic.
  assert!(
    xml.contains(r#"<p xml:id="Thmthm1.p1.1"><text font="italic" xml:id="Thmthm1.p1.1.1">A theorem.</text></p>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<p xml:id="Thmthm2.p1.1">A definition.</p>"#),
    "{xml}"
  );
}
