//! Batch 56hi: the post stage keeps glossary phrases as NODES (Perl
//! Scan.pm:442, MakeIndex.pm:477-480, CrossRef.pm:906-925) and fills an
//! empty `ltx:glossaryref` from its entry (CrossRef.pm:469-476).
use crate::cluster::convert_and_post_clean;

/// acronym.sty's `\ac`/`\acl`/`\acs` emit an EMPTY `<ltx:glossaryref
/// show=…>`; CrossRef fills it with the entry's `phrase:<show>`. The port
/// never called `generateGlossaryRefTitle`, so every acronym rendered as its
/// key with `ltx_missing` ("NN (NN)" for "Neural Network (NN)") and an empty
/// tooltip (it read `phrase:description`; acronym's role is `definition`).
#[test]
fn acronym_refs_show_their_phrase() {
  let xml = convert_and_post_clean("tests/cluster_regressions/acronym_glossaryref_phrase.tex");
  assert!(!xml.contains("ltx_missing"), "{xml}");
  latexml::util::test::assert_element(
    &xml,
    "glossaryref",
    &[r#"show="long""#],
    r##"<glossaryref idref="id1" inlist="acronym" key="NN" show="long" title="Neural Network"><text class="ltx_glossary_long">Neural Network</text></glossaryref>"##,
  );
  latexml::util::test::assert_element(
    &xml,
    "glossaryref",
    &[r#"show="short""#],
    r##"<glossaryref idref="id1" inlist="acronym" key="NN" show="short" title="Neural Network"><text class="ltx_glossary_short">NN</text></glossaryref>"##,
  );
}

/// glossaries' `\newglossaryentry` fields are the DIGESTED values (Perl
/// glossaries.sty.ltxml:96 inserts `$value` after KeyVals::beDigested): a
/// `$\alpha$` name is math and an `\emph` description emphasis. The binding
/// absorbed each value's string, so the definition, the glossary list and the
/// `\gls` tooltip carried the TeX source (`angle \emph{in} radians`).
#[test]
fn glossaries_fields_keep_their_markup() {
  let xml = convert_and_post_clean("tests/cluster_regressions/glossaries_field_markup.tex");
  assert!(!xml.contains("\\emph"), "{xml}");
  latexml::util::test::assert_element(
    &xml,
    "glossaryentry",
    &[],
    r##"<glossaryentry fragid="glo.main.al" key="al" lists="main" xml:id="glo.main.al"><glossaryphrase key="al" role="label"><Math mode="inline" tex="\alpha" text="alpha" xml:id="m1a"><XMath><XMTok font="italic" name="alpha" role="UNKNOWN">α</XMTok></XMath></Math></glossaryphrase><glossaryphrase role="definition">angle <emph font="italic">in</emph> radians</glossaryphrase></glossaryentry>"##,
  );
}

/// Convert `tex` as `t.tex` to HTML in one process under the raw preload (the glossary list is built by post).
/// Returns (ANSI-stripped stderr, HTML).
fn convert_html_raw(tex: &str) -> (String, String) {
  let bin = env!("CARGO_BIN_EXE_latexml_oxide");
  let workdir = tempfile::tempdir().expect("create tempdir");
  std::fs::write(workdir.path().join("t.tex"), tex).expect("write t.tex");
  let output = std::process::Command::new(bin)
    .args([
      "t.tex",
      "--dest",
      "t.html",
      "--nocomments",
      "--timeout=110",
      "--preload=[rawstyles,rawclasses]latexml.sty",
    ])
    .current_dir(workdir.path())
    .output()
    .expect("spawn latexml_oxide");
  let stderr = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
  let html = std::fs::read_to_string(workdir.path().join("t.html")).unwrap_or_default();
  (stderr, html)
}

/// 60i: the display keys a package adds with `\glsaddkey` (glosmathtools' `descseclang`) become phrases, set after
/// the description as classed text, and its `\glsaddstoragekey` data keys (`dot`) do not; the parent of a listed
/// entry is listed, its children after it, classed by depth (makeglossaries' `\subglossentry`) — `mu` (sort 12) under
/// `greek` (sort 2) though it sorts before it. pdflatex + makeglossaries: "Latin symbols (Symboles latins)" / "d
/// diameter (diametre)" / "m mass (masse)" / "Greek symbols (Symboles grecs)" / "µ viscosity (viscosite)" / "Vectors
/// (Vecteurs)" / "v velocity (vitesse)". Repro index-bib/glossary_user_keys_and_parents.
#[test]
fn glossary_user_keys_and_parents() {
  if !latexml::util::test::kpse_has("glosmathtools.sty") {
    return;
  }
  let (stderr, html) = convert_html_raw(include_str!(
    "../../../tools/perfect_kernel/repros/index-bib/glossary_user_keys_and_parents.tex"
  ));
  assert_eq!(latexml::util::test::error_count(&stderr), 0, "{stderr}");
  assert_eq!(stderr.matches("Warning:").count(), 0, "{stderr}");
  let entry = |key: &str, level: &str, label: &str, description: &str, seclang: &str| {
    format!(
      r#"<dt class="ltx_glossaryentry{level} ltx_list_main" id="glo.main.{key}">{label}</dt><dd>{description} <span class="ltx_text ltx_glossary_descseclang">{seclang}</span></dd>"#
    )
  };
  let math = |tex: &str, mi: &str| {
    format!(r#"<math alttext="{tex}" class="ltx_Math" display="inline"><mi>{mi}</mi></math>"#)
  };
  let child = " ltx_glossary_level_1";
  let expected = [
    entry("latin", "", "latin", "Latin symbols", "Symboles latins"),
    entry("d", child, &math("d", "d"), "diameter", "diametre"),
    entry("m", child, &math("m", "m"), "mass", "masse"),
    entry("greek", "", "greek", "Greek symbols", "Symboles grecs"),
    entry("mu", child, &math("\\mu", "μ"), "viscosity", "viscosite"),
    entry("vectors", "", "vectors", "Vectors", "Vecteurs"),
    entry("v", child, &math("v", "v"), "velocity", "vitesse"),
  ]
  .concat();
  latexml::util::test::assert_element(
    &html,
    "dl",
    &[r#"class="ltx_glossarylist""#],
    &format!(r#"<dl class="ltx_glossarylist">{expected}</dl>"#),
  );
  assert!(
    !html.contains("ltx_glossary_dot"),
    "a storage key is printed:\n{html}"
  );
}

/// 60i r2: a glossary sorts by its entries' sort strings, as makeindex: `$\alpha $` before `$x$` (`\` before `x`),
/// symbols before letters — under OT1, where the digested sort phrase was the font's `“` `”` and sorted after every
/// letter — and without glossaries' makeindex escapes (`a+e` before `a|d`, escaped `a"|d`). pdflatex +
/// makeglossaries: α, x, ape, apd, b, q”x. Repro index-bib/glossary_sort_key_is_its_string.
#[test]
fn glossary_sort_key_is_its_string() {
  let (stderr, html) = convert_html_raw(include_str!(
    "../../../tools/perfect_kernel/repros/index-bib/glossary_sort_key_is_its_string.tex"
  ));
  assert_eq!(latexml::util::test::error_count(&stderr), 0, "{stderr}");
  assert_eq!(stderr.matches("Warning:").count(), 0, "{stderr}");
  let math = |tex: &str, mi: &str| {
    format!(r#"<math alttext="{tex}" class="ltx_Math" display="inline"><mi>{mi}</mi></math>"#)
  };
  let entry = |key: &str, label: &str, description: &str| {
    format!(
      r#"<dt class="ltx_glossaryentry ltx_list_main" id="glo.main.{key}">{label}</dt><dd>{description}</dd>"#
    )
  };
  let expected = [
    entry("alpha", &math("\\alpha", "α"), "angle"),
    entry("x", &math("x", "x"), "position"),
    entry("ape", "ape", "plus"),
    entry("apd", "apd", "bar"),
    entry("b", "b", "breadth"),
    entry("q", "q”x", "quoted"),
  ]
  .concat();
  latexml::util::test::assert_element(
    &html,
    "dl",
    &[r#"class="ltx_glossarylist""#],
    &format!(r#"<dl class="ltx_glossarylist">{expected}</dl>"#),
  );
}
