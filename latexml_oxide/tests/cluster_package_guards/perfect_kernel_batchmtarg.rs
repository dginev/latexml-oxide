//! Guards for MTARG: an `\@maketitle` that takes an argument (iaus.cls:824-868, 0801.1806). The title-page deposit
//! wrapped it as `{…\@maketitle}`, so it read the closing brace as its argument and left the group open at
//! `\end{document}`; the replay of the class's `\maketitle` relaxed it, and its argument (`\@maketitle{n}`) was
//! typeset as a stray "n". Now the deposit skips it and the replay leaves it live (sect05.rs `takes_parameters`).

use latexml::util::test::{error_count, warning_count};

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $name
    ))
  };
}

/// `tex` (and `files`) converted by the binary in a directory of their own to `dest` (`t.xml` or `t.html`), as a
/// paper is: (log, output).
fn convert_to(tex: &str, files: &[File], dest: &str) -> (String, String) {
  let workdir = tempfile::tempdir().expect("create tempdir");
  std::fs::write(workdir.path().join("t.tex"), tex).expect("write t.tex");
  for (name, content) in files {
    std::fs::write(workdir.path().join(name), content).expect("write a file");
  }
  let output = std::process::Command::new(env!("CARGO_BIN_EXE_latexml_oxide"))
    .args([
      "t.tex",
      "--dest",
      dest,
      "--nocomments",
      "--timeout=110",
      "--preload=ar5iv.sty",
    ])
    .current_dir(workdir.path())
    .output()
    .expect("spawn latexml_oxide");
  let log = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
  let out = std::fs::read_to_string(workdir.path().join(dest))
    .unwrap_or_else(|_| panic!("no {dest}: {log}"));
  (log, out)
}

/// A side file: (name, content).
type File<'a> = (&'a str, &'a str);

/// Each shape, as XML and as HTML: no error and no Warning (none of "open groups"), the title once, and the body's one
/// paragraph exactly "Hi" (no stray "n"). Rows: a class whose own `\maketitle` calls `\@maketitle{n}` (replayed, the
/// argument supplied by the class); a document's `\def\@maketitle#1{}` under article's kernel `\maketitle` (no replay).
#[test]
fn an_argument_taking_maketitle_closes_its_group() {
  let class = [("mtarg.cls", repro!("mtarg.cls"))];
  let rows: [(&str, &str, &[File]); 2] = [
    (
      "class",
      repro!("maketitle_argument_class_title_page_closes_its_group.tex"),
      &class,
    ),
    (
      "document",
      repro!("maketitle_argument_document_definition_closes_its_group.tex"),
      &[],
    ),
  ];
  for (name, tex, files) in rows {
    let (log, xml) = convert_to(tex, files, "t.xml");
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "{name}: {log}"
    );
    assert_eq!(xml.matches("<title").count(), 1, "{name}: {xml}");
    assert!(xml.contains("<title>T</title>"), "{name}: {xml}");
    assert_eq!(xml.matches("<p ").count(), 1, "{name}: {xml}");
    assert!(xml.contains(r#"<p xml:id="p1.1">Hi</p>"#), "{name}: {xml}");
    assert!(!xml.contains("<titlepage"), "{name}: {xml}");

    let (log, html) = convert_to(tex, files, "t.html");
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "{name}: {log}"
    );
    assert_eq!(
      html
        .matches(r#"<h1 class="ltx_title ltx_title_document">T</h1>"#)
        .count(),
      1,
      "{name}: {html}"
    );
    assert_eq!(html.matches("<p ").count(), 1, "{name}: {html}");
    assert!(
      html.contains(r#"<p id="p1.1" class="ltx_p">Hi</p>"#),
      "{name}: {html}"
    );
  }
}
