//! Guards for the bindings batch of 2026-11-13 (issues #772, #760, #801, #787, #773, #779): one test per issue, each a
//! binding's own mechanism, its repro converted once in-process and checked whole, with exact Error/Warning counts.

use latexml::util::test::{
  assert_element, convert_files_with, convert_with, error_count, kpse_has, warning_count,
};

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

/// Issue #772: `\import@path` is import.sty's own macro, set by `\import`/`\subimport` and restored after the input
/// (import.sty:83, :93), empty outside (:142), and emptied by `\import`/`\inputfrom` before it is saved (:53-58), so
/// a nested one restores it to empty. Red: `Error:undefined:\import@path`; the nested rows `[n/]` (revW2).
#[test]
fn import_path_is_the_package_macro() {
  if !kpse_has("import.sty") {
    return;
  }
  for (tex, paragraphs) in [
    (
      repro!("package-bindings", "import_path_is_the_package_macro.tex"),
      &["<p>[][a/][a/b/]\n[a/]\n[]</p>"][..],
    ),
    (
      repro!(
        "package-bindings",
        "import_nested_import_empties_the_path.tex"
      ),
      &["<p>[][n/][c/]\n[]\n[]</p>", "<p>[][n/][c/]\n[]\n[]</p>"][..],
    ),
  ] {
    let xml = clean(tex);
    for p in paragraphs.iter() {
      assert_element(&xml, "ltx:p", &[], p);
    }
    assert_eq!(xml.matches("<p>").count(), paragraphs.len(), "{xml}");
  }
}

/// Issue #779: a standard class's size and paper options set `\textwidth` as its size file does in one column
/// (size1x.clo:108-126) and `\@ptsize`; pdflatex's values. Red: 345.0pt and 0 for every row. Under 2.09 compatibility
/// bk1x.clo:105's 4.5in/5in (revW2; red 345.0pt, 390.0pt). The options are those the class was given (`\opt@article.cls`):
/// a class passing its options to article reaches them, its own `\textwidth` after it standing (llncs.cls:55); a raw
/// class's bare `\LoadClass{article}` does not (revW2b; red 345.0pt and 0 for elsarticle, llncs, `myc`, `wo`).
#[test]
fn standard_class_size_option_sets_textwidth() {
  let tex = repro!(
    "class-bindings",
    "standard_class_size_option_sets_textwidth.tex"
  );
  let classes = [
    (
      "mycls.cls",
      "\\ProvidesClass{mycls}\n\\LoadClass{article}\n",
    ),
    (
      "myc.cls",
      "\\ProvidesClass{myc}\n\\DeclareOption*{\\PassOptionsToClass{\\CurrentOption}{article}}\n\\ProcessOptions\n\\LoadClass{article}\n",
    ),
    (
      "wo.cls",
      "\\ProvidesClass{wo}\n\\LoadClassWithOptions{article}\n",
    ),
  ];
  for (class, width, ptsize) in [
    (r"\documentclass[12pt]{article}", "390.0pt", "2"),
    (r"\documentclass{article}", "345.0pt", "0"),
    (r"\documentclass[11pt]{report}", "360.0pt", "1"),
    (r"\documentclass[12pt,a5paper]{book}", "276.0pt", "2"),
    (r"\documentstyle{book}", "325.215pt", "0"),
    (r"\documentstyle[12pt]{book}", "361.34999pt", "2"),
    (r"\documentclass[12pt]{mycls}", "345.0pt", "0"),
    (r"\documentclass[12pt]{myc}", "390.0pt", "2"),
    (r"\documentclass[11pt,a5paper]{myc}", "276.0pt", "1"),
    (r"\documentclass[12pt]{wo}", "390.0pt", "2"),
    (r"\documentclass[12pt]{elsarticle}", "390.0pt", "2"),
    (r"\documentclass[12pt]{llncs}", "347.12354pt", "2"),
  ] {
    // A class that is not a standard one is read under ar5iv.sty (a raw class otherwise OmniBus), which also gives the
    // paragraphs ids.
    let preload = !["{article}", "{report}", "{book}"]
      .iter()
      .any(|c| class.ends_with(c));
    let (log, xml) = convert_files_with(
      &tex.replace(r"\documentclass[12pt]{article}", class),
      &classes,
      preload.then_some("ar5iv.sty"),
    );
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "{class}: {log}"
    );
    let id = if preload { " xml:id=\"p1.1\"" } else { "" };
    assert_element(
      &xml,
      "ltx:p",
      &[],
      &format!("<p{id}>TEXTWIDTH = {width}, PTSIZE = {ptsize}</p>"),
    );
  }
}

/// The verbatim lines `x=…` of `xml`, in order.
fn verbatim_lines(xml: &str) -> Vec<&str> {
  xml
    .match_indices("x=")
    .map(|(at, _)| &xml[at..at + xml[at..].find(['<', '\n']).unwrap_or(0)])
    .collect()
}

/// Issue #801: under upquote, `{verbatim}` and `{verbatim*}` print ' and ` upright as `\@noligs` makes them, like
/// fancyvrb's Verbatim and `\verb` (pdflatex: all four upright). Without upquote they stay the typewriter font's curly
/// quotes, as pdflatex prints them (`\verb` keeps its ASCII, OXIDIZED_DESIGN #144). Red: the last two curly.
#[test]
fn verbatim_quotes_follow_upquote() {
  if !kpse_has("upquote.sty") || !kpse_has("fancyvrb.sty") {
    return;
  }
  let tex = repro!("package-bindings", "verbatim_quotes_follow_upquote.tex");
  let xml = clean(tex);
  assert_eq!(
    verbatim_lines(&xml),
    ["x='s' `b`", "x='s' `b`", "x='s' `b`", "x='s'\u{2423}`b`"],
    "{xml}"
  );
  let xml = clean(&tex.replace("\\usepackage{upquote}\n", ""));
  assert_eq!(
    verbatim_lines(&xml),
    [
      "x=\u{2019}s\u{2019} \u{2018}b\u{2018}",
      "x='s' `b`",
      "x=\u{2019}s\u{2019} \u{2018}b\u{2018}",
      "x=\u{2019}s\u{2019}\u{2423}\u{2018}b\u{2018}"
    ],
    "{xml}"
  );
}

/// Issue #760: etoc's `\localtableofcontents` lists the enclosing section's subsections, to `\etocsetnexttocdepth`'s
/// depth, `\tableofcontents` the document's sections, `\invisiblelocaltableofcontents` nothing (pdflatex, two runs).
/// Red: 2 Errors (both commands undefined), one table.
#[test]
fn etoc_local_table_lists_the_unit() {
  if !kpse_has("etoc.sty") {
    return;
  }
  let tex = repro!("package-bindings", "etoc_local_table_lists_the_unit.tex");
  let xml = clean(tex);
  assert_element(
    &xml,
    "ltx:TOC",
    &["scope=\"S1\""],
    "<TOC lists=\"toc\" scope=\"S1\" select=\"ltx:part | ltx:chapter | ltx:section | ltx:subsection | ltx:appendix | \
     ltx:index | ltx:bibliography\"><title>Contents</title></TOC>",
  );
  assert_eq!(xml.matches("<TOC ").count(), 2, "{xml}");
  let html = latexml::util::test::convert_to_html_page(tex).expect("HTML");
  let tables: Vec<String> = html
    .split("<nav class=\"ltx_TOC")
    .skip(1)
    .map(|nav| {
      let nav = &nav[nav.find('>').map_or(0, |at| at + 1)..nav.find("</nav>").unwrap_or(nav.len())];
      let mut text = String::new();
      let mut in_tag = false;
      for c in nav.chars() {
        match c {
          '<' => in_tag = true,
          '>' => {
            in_tag = false;
            text.push(' ');
          },
          c if !in_tag => text.push(c),
          _ => {},
        }
      }
      text.split_whitespace().collect::<Vec<_>>().join(" ")
    })
    .collect();
  assert_eq!(
    tables,
    [
      "Contents 1 First section 2 Second section",
      "Contents 1.1 First subsection 1.2 Second subsection"
    ],
    "{html}"
  );
  // `\etocsettocstyle`'s heading is not typeset (RED `etoc_settocstyle_heading_red.tex`): each table's loss is counted
  // by a Warning naming it (revW2b; red: 0 Warnings).
  let (log, _) = convert_with(
    repro!("package-bindings", "etoc_settocstyle_heading_red.tex"),
    None,
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 2), "{log}");
  for heading in [r"\section*{Overview}", r"\subsection*{In this section}"] {
    assert!(
      log.contains(&format!("its heading is lost: {heading}")),
      "{log}"
    );
  }
}
