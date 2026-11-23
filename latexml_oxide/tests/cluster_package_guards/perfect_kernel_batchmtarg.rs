//! Guards for MTARG: an `\@maketitle` that takes an argument (iaus.cls:824-868, 0801.1806). The title-page deposit
//! wrapped it as `{…\@maketitle}`, so it read the closing brace as its argument and left the group open at
//! `\end{document}`; the replay of the class's `\maketitle` relaxed it, and its argument (`\@maketitle{n}`) was
//! typeset as a stray "n". Now the replay that runs it leaves it live and the deposit leaves it to that replay; where
//! nothing replays it, or the replay is dropped, the deposit gives it an `{}` per mandatory argument (sect05.rs
//! `maketitle_left_to_replay`).

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

/// Each shape, as XML and as HTML: no error and no Warning (none of "open groups"), the title once, the title page's
/// own text (its italic subtitle) kept as a whole paragraph, and the body's paragraph exactly "Hi" (no stray "n").
/// Rows, each a distinct path of `\lx@deposit@maketitle` (sect05.rs `maketitle_left_to_replay`):
/// - "class": a class `\maketitle` calling `\@maketitle{n}`, replayed with `\@maketitle` live (the class supplies the
///   argument; the deposit is left to the replay);
/// - "document": a document's `\def\@maketitle#1{…}` under article's kernel `\maketitle` (no replay): deposited with
///   an `{}` argument;
/// - "optional": a document's `\renewcommand\@maketitle[1][]{…}` (no replay): deposited as it is;
/// - "dropped": the class shape whose replay raises an error (its title read as empty) and is dropped: the deposit is
///   made after it, with an `{}` argument.
#[test]
fn an_argument_taking_maketitle_closes_its_group() {
  let class = [("mtarg.cls", repro!("mtarg.cls"))];
  let class_err = [("mtargerr.cls", repro!("mtargerr.cls"))];
  // (name, source, side files, title, the subtitle paragraph's attributes before its id)
  let rows: [(&str, &str, &[File], &str, &str); 4] = [
    (
      "class",
      repro!("maketitle_argument_class_title_page_closes_its_group.tex"),
      &class,
      "T",
      "",
    ),
    (
      "document",
      repro!("maketitle_argument_document_definition_closes_its_group.tex"),
      &[],
      "The Title",
      r#"align="center" "#,
    ),
    (
      "optional",
      repro!("maketitle_optional_argument_document_definition_keeps_its_text.tex"),
      &[],
      "The Title",
      r#"align="center" "#,
    ),
    (
      "dropped",
      repro!("maketitle_argument_class_replay_dropped_deposits_its_text.tex"),
      &class_err,
      "T",
      "",
    ),
  ];
  for (name, tex, files, title, align) in rows {
    let (log, xml) = convert_to(tex, files, "t.xml");
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "{name}: {log}"
    );
    assert!(!log.contains("open groups"), "{name}: {log}");
    assert_eq!(xml.matches("<title").count(), 1, "{name}: {xml}");
    assert!(
      xml.contains(&format!("<title>{title}</title>")),
      "{name}: {xml}"
    );
    assert_eq!(xml.matches("<p ").count(), 2, "{name}: {xml}");
    assert!(
      xml.contains(&format!(
        r#"<p {align}xml:id="p1.1"><text font="italic" xml:id="p1.1.1">A Subtitle Written Here</text></p>"#
      )),
      "{name}: {xml}"
    );
    assert!(xml.contains(r#"<p xml:id="p2.1">Hi</p>"#), "{name}: {xml}");
    assert!(!xml.contains("<titlepage"), "{name}: {xml}");

    let (log, html) = convert_to(tex, files, "t.html");
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "{name}: {log}"
    );
    assert!(!log.contains("open groups"), "{name}: {log}");
    assert_eq!(
      html
        .matches(&format!(
          r#"<h1 class="ltx_title ltx_title_document">{title}</h1>"#
        ))
        .count(),
      1,
      "{name}: {html}"
    );
    let center = if align.is_empty() {
      ""
    } else {
      " ltx_align_center"
    };
    assert_eq!(html.matches("<p ").count(), 2, "{name}: {html}");
    assert!(
      html.contains(&format!(
        r#"<p id="p1.1" class="ltx_p{center}"><span id="p1.1.1" class="ltx_text ltx_font_italic">A Subtitle Written Here</span></p>"#
      )),
      "{name}: {html}"
    );
    assert!(
      html.contains(r#"<p id="p2.1" class="ltx_p">Hi</p>"#),
      "{name}: {html}"
    );
  }
}
