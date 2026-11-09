//! Guards for NE7 (`agent_reports/2026-11-09_brief_NE7.md`): a copied TikZ picture keeps each SVG element's own `id`.
//! `Document::append_clone` copies an SVG `id` as Perl's `appendClone` does (Document.pm:1943-1954: only `xml:id` is
//! remapped) and drops the `_svgid` mark, so the copy's `svg:svg` close renames the ids it repeats with their
//! references (`record_svg_ids`). `append_clone_aux` remapped the plain `id` into an `xml:id`, which the svg schema
//! forbids: tikz-shields-doc (a TeX Live manual, `\githubBadge` in `\author`) had 8 jing lines at `=2`, whose writer
//! copies each contact and note into its creator (`repart.rs` `write_items`).

use latexml::util::test::{
  convert_files_with_setup_then, error_count, rng_error_count, warning_count,
};
use latexml_engine::base_utilities::set_fm_shadow_override;

use super::perfect_kernel_batch84b::creators;

/// Convert `tex`: the log and the XML.
fn run(tex: &str, preload: Option<&str>) -> (String, String) {
  let (log, xml, ()) = convert_files_with_setup_then(
    tex,
    &[],
    preload,
    move || {
      set_fm_shadow_override(None);
    },
    |_| (),
  );
  (log, xml)
}

/// The SVG `id`s of `xml`, sorted; panics on a repeated one, an SVG `xml:id`, a leaked `_svgid` mark, or a
/// `url(#…)`/`xlink:href="#…"` reference to no id.
fn svg_ids(xml: &str, context: &str) -> Vec<String> {
  assert!(
    !xml.contains("_svgid"),
    "{context}: the bookkeeping mark leaked:\n{xml}"
  );
  assert!(
    !xml.contains("xml:id=\"pgf"),
    "{context}: an SVG id written as xml:id:\n{xml}"
  );
  let mut ids: Vec<String> = xml
    .split(" id=\"")
    .skip(1)
    .map(|rest| rest[..rest.find('"').unwrap()].to_string())
    .collect();
  ids.sort_unstable();
  let mut unique = ids.clone();
  unique.dedup();
  assert_eq!(ids, unique, "{context}: a repeated SVG id:\n{xml}");
  for (open, close) in [("url(#", ")"), ("xlink:href=\"#", "\"")] {
    for rest in xml.split(open).skip(1) {
      let target = &rest[..rest.find(close).unwrap()];
      assert!(
        ids.iter().any(|id| id == target),
        "{context}: a reference to no id: #{target}\n{xml}"
      );
    }
  }
  ids
}

/// A TikZ badge (a shading and a clip, each an SVG id with its `url(#…)` reference) as an author's URL contact and in
/// another author's `\thanks`: off leaves the digested pictures in place; `=2` copies each into its creator, and the
/// copy carried `xml:id="pgfcp1"`, `xml:id="pgfsh2"` … (6 jing lines). Now both are schema-valid, the copy's ids
/// renamed with their references. Repro sectioning-frontmatter/ne7_tikz_badges_in_contacts_keep_svg_ids.
#[test]
fn tikz_badges_in_frontmatter_contacts_keep_svg_ids() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/ne7_tikz_badges_in_contacts_keep_svg_ids.tex"
  );
  let suffix = "a";
  let context = "=2";
  let (log, xml) = run(tex, Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  if let Some(lines) = rng_error_count(&xml) {
    assert_eq!(lines, 0, "jing:\n{xml}");
  }
  let expected: Vec<String> = ["pgfcp1", "pgfcp3", "pgfcp4", "pgfcp6", "pgfsh2", "pgfsh5"]
    .iter()
    .map(|id| format!("{id}{suffix}"))
    .collect();
  let ids = svg_ids(&xml, context);
  assert_eq!(ids, expected, "{context}:\n{xml}");
  for id in &ids {
    assert_eq!(
      xml.matches(&format!("url(#{id})")).count(),
      1,
      "{id} referenced once\n{xml}"
    );
  }
  assert_eq!(
    creators(&xml),
    [vec!["Ann Able".to_string(), "url=GH".to_string()], vec![
      "Bob Baker".to_string()
    ]],
    "{context}"
  );
  for (holder, href) in [
    (
      "<contact name=\"URL:\u{a0}\" role=\"url\">",
      "https://example.org/a",
    ),
    (
      r#"<note class="ltx_note_frontmatter ltx_thanks_note" role="thanks" xml:id="id1">"#,
      "https://example.org/b",
    ),
  ] {
    let at = xml
      .find(holder)
      .unwrap_or_else(|| panic!("no {holder}\n{xml}"));
    assert!(
      xml[at + holder.len()..].trim_start().starts_with(&format!(
        r#"<ref class="ltx_href" href="{href}"><picture height="19.69" width="47.24" "#
      )),
      "the badge's picture in {holder}\n{xml}"
    );
  }
}

/// The alignment split's MathFork copy of an eqnarray cell holding a `\tikz`, inside a picture: the copy kept its
/// `xlink:href="#pgfpath2"` while its ids became generated `xml:id`s, and the outer picture's walk renamed the cell
/// original, so the reference dangled (57cl.1 re-review, probe q8eqnode). The copy now keeps its SVG ids and its own
/// close renames them with their references. A distinct consumer of the same copy (math, nested pictures, `use`
/// references). Repro graphics-tikz/eqnarray_copy_in_a_picture_keeps_its_references.
#[test]
fn an_eqnarray_copy_in_a_picture_keeps_its_references() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/graphics-tikz/eqnarray_copy_in_a_picture_keeps_its_references.tex"
  );
  let (log, xml) = run(tex, None);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  if let Some(lines) = rng_error_count(&xml) {
    assert_eq!(lines, 0, "jing:\n{xml}");
  }
  assert_eq!(
    svg_ids(&xml, "eqnarray"),
    [
      "pgfcp1",
      "pgfcp1a",
      "pgfcp1b",
      "pgfcp1c",
      "pgfcp2",
      "pgfcp2a",
      "pgfpath1",
      "pgfpath1a",
      "pgfpath1b",
      "pgfpath1c",
      "pgfpath2",
      "pgfpath2a"
    ],
    "{xml}"
  );
}
