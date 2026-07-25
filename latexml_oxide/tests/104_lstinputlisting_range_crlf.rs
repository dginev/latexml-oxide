//! Regression tests: `\lstinputlisting` over an externally-read source file —
//! a truncating line range, and CRLF line terminators.
//!
//! Witness: arXiv 2412.04705 (arXiv/html_feedback#6735, "Wrong code snippet in
//! html display"), whose `\inputpython` wraps
//! `\lstinputlisting[firstline=32,lastline=35,...]` over CRLF Python sources.
//! Both defects are shared with Perl LaTeXML; see OXIDIZED_DESIGN #68 / #69 and
//! `KNOWN_PERL_ERRORS.md`.
//!
//! 1. **Truncating range** (`listings_sty.rs` "Remove trailing empty lines").
//!    `lastline=N` on a file with MORE than N lines cut the generated token
//!    vector at `emptyfrom`, discarding `}` tokens that closed groups opened
//!    BEFORE the cut — measured discarded tail on the witness:
//!    `["\@lst@startline", "{", "}", "}", "}", "}", "\@lst@endline"]`, three of
//!    them closers. The listing body was emitted with unclosed groups, so
//!    `\@@listings@block` read its arguments off the end of the DOCUMENT and
//!    everything after the listing was swallowed.
//!
//! 2. **CRLF** (`listings_read_raw_file`). Every end-of-line test in the
//!    listings processor is written against `\n`; a `\r` before it defeats them,
//!    so a line comment never terminates and its STYLE (not its class — the
//!    `ltx_lst_comment` wrapper does close) bleeds over every following line.
//!    pdflatex on the witness renders only the `#` line in comment green
//!    (9 green vs 69 black glyph groups); both LaTeXML engines painted the whole
//!    snippet green.
//!
//! Binary-driven (fresh process) so the listing file is read from disk.

use std::{path::Path, process::Command};

/// CRLF on purpose — this is half of what is under test.
const DATA_PY: &str = "# a comment line\r\nvalue = 1\r\nother = 2\r\nlast = 3\r\n";

fn convert(tex: &str, data: &str) -> (String, String) {
  let bin = env!("CARGO_BIN_EXE_latexml_oxide");
  assert!(Path::new(bin).is_file(), "binary not staged at {bin}");

  let workdir = tempfile::tempdir().expect("create tempdir");
  std::fs::write(workdir.path().join("data.py"), data).expect("write data.py");
  std::fs::write(workdir.path().join("t.tex"), tex).expect("write t.tex");

  let output = Command::new(bin)
    .args(["t.tex", "--dest", "t.xml", "--nocomments"])
    .current_dir(workdir.path())
    .output()
    .expect("spawn latexml_oxide");

  let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
  assert!(
    output.status.success(),
    "binary exited {:?}\nstderr:\n{stderr}",
    output.status.code(),
  );
  let xml = std::fs::read_to_string(workdir.path().join("t.xml")).expect("read t.xml");
  (xml, stderr)
}

/// Collect the text of each `<listingline>`, in order.
fn listing_lines(xml: &str) -> Vec<String> {
  let mut out = Vec::new();
  for chunk in xml.split("<listingline").skip(1) {
    let Some(end) = chunk.find("</listingline>") else {
      continue;
    };
    out.push(chunk[..end].to_string());
  }
  out
}

fn strip_tags(fragment: &str) -> String {
  let mut text = String::new();
  let mut in_tag = false;
  for ch in fragment.chars() {
    match ch {
      '<' => in_tag = true,
      '>' => in_tag = false,
      c if !in_tag => text.push(c),
      _ => {},
    }
  }
  text
}

#[test]
fn lastline_shorter_than_file_does_not_swallow_the_document() {
  // `lastline=3` over a 4-line file: the truncation path is exercised.
  let tex = "\\documentclass{article}\n\
    \\usepackage{listings}\n\
    \\begin{document}\n\
    \\lstinputlisting[lastline=3]{data.py}\n\
    Text after the listing.\n\
    \\end{document}\n";
  let (xml, stderr) = convert(tex, DATA_PY);

  assert!(
    !stderr.contains("Error:") && !stderr.contains("Fatal:"),
    "truncating lastline should convert cleanly, stderr had:\n{stderr}",
  );
  // The document continues after the listing — the unbalanced body used to make
  // `\@@listings@block` read its arguments to EOF, losing everything after it.
  assert!(
    xml.contains("Text after the listing"),
    "content after the listing was swallowed:\n{xml}",
  );
  let lines = listing_lines(&xml);
  assert_eq!(
    lines.len(),
    3,
    "expected exactly lines 1..3 of the file, got {}:\n{xml}",
    lines.len()
  );
  assert!(
    strip_tags(&lines[2]).contains("other = 2"),
    "third listing line should be the file's line 3:\n{}",
    strip_tags(&lines[2])
  );
  assert!(
    !xml.contains("last = 3"),
    "line 4 is past lastline=3 and must not appear:\n{xml}",
  );
}

#[test]
fn crlf_line_comment_style_does_not_bleed_past_its_line() {
  // `\r\n` terminators: only the `#` line is a comment. The class wrapper always
  // closed correctly; it is the STYLE that used to leak, so assert on `font`.
  let tex = "\\documentclass{article}\n\
    \\usepackage{listings}\n\
    \\lstdefinestyle{s}{morecomment=[l]{\\#},commentstyle=\\itshape}\n\
    \\begin{document}\n\
    \\lstinputlisting[style=s]{data.py}\n\
    \\end{document}\n";
  let (xml, stderr) = convert(tex, DATA_PY);

  assert!(
    !stderr.contains("Error:") && !stderr.contains("Fatal:"),
    "CRLF listing should convert cleanly, stderr had:\n{stderr}",
  );
  let lines = listing_lines(&xml);
  assert_eq!(lines.len(), 4, "expected all 4 file lines:\n{xml}");

  assert!(
    lines[0].contains("font=\"italic\""),
    "the comment line should carry the commentstyle:\n{}",
    lines[0]
  );
  for (i, line) in lines.iter().enumerate().skip(1) {
    assert!(
      !line.contains("font=\"italic\""),
      "line {} is code, but the comment style bled into it:\n{line}",
      i + 1
    );
  }
}
