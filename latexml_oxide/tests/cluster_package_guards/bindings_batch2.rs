//! Guards for the second bindings batch of 2026-11-13 (issues #787, #773): one table-driven test per issue, each repro
//! converted once in-process and checked whole, with exact Error/Warning counts.

use latexml::util::test::{convert_with, error_count, kpse_has, warning_count};

macro_rules! repro {
  ($dir:literal, $path:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/",
      $dir,
      "/",
      $path
    ))
  };
}

/// Converts `tex` (no preload) and asserts it converts with no Error and no Warning; returns the XML.
#[track_caller]
fn clean(tex: &str) -> String {
  let (log, xml) = convert_with(tex, None);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  xml
}

/// Every element `<tag …>…</tag>` of `xml`, whole, in document order.
fn elements(xml: &str, tag: &str) -> Vec<String> {
  let (open, close) = (format!("<{tag}"), format!("</{tag}>"));
  let mut found = Vec::new();
  let mut rest = xml;
  while let Some(at) = rest.find(&open) {
    let tail = &rest[at..];
    let end = tail.find(&close).map_or(tail.len(), |e| e + close.len());
    found.push(tail[..end].to_string());
    rest = &tail[end..];
  }
  found
}

/// Issue #787: caption's `justification=` is the caption's alignment class (`\centering`'s, `\raggedright`'s,
/// `\raggedleft`'s) and `skip=`/`belowskip=` its margins on the sides caption.sty:251-258 puts them: a longtable's
/// `[longtable]` options (and not the next table's), a bottom caption by auto position, a `position=top` starred one.
/// Red on base 6df4632fb1: every `<caption>` without class or cssstyle.
#[test]
fn caption_justification_and_skip() {
  if !kpse_has("caption.sty") || !kpse_has("longtable.sty") {
    return;
  }
  let rows: [(&str, &[&str]); 2] = [
    (
      repro!("captions-floats", "caption_justification_and_skip.tex"),
      &[
        "<caption class=\"ltx_centering\" cssstyle=\"margin-bottom:20.0pt\"><tag close=\": \">Table 1</tag>Results</caption>",
        "<caption class=\"ltx_centering\"><tag close=\": \">Table 2</tag>After</caption>",
      ],
    ),
    (
      repro!(
        "captions-floats",
        "caption_justification_and_skip_sides.tex"
      ),
      &[
        "<caption class=\"ltx_align_left\" cssstyle=\"margin-top:4.0pt\"><tag close=\": \">Figure 1</tag>Bottom figure</caption>",
        "<caption class=\"ltx_align_right\" cssstyle=\"margin-top:6.0pt\"><tag close=\": \">Table 1</tag>Top table</caption>",
        "<caption class=\"ltx_centering\" cssstyle=\"margin-bottom:4.0pt\">Starred</caption>",
      ],
    ),
  ];
  for (tex, expected) in rows {
    let xml = clean(tex);
    assert_eq!(elements(&xml, "caption"), expected, "{xml}");
  }
}
