//! The `<?latexml class=…?>` / `<?latexml package=…?>` PIs that a `--preload`
//! contributes to the document.
//!
//! Perl `Core.pm` L268-277 rewrites the preload spec IN PLACE with `s///`
//! before it becomes an attribute value: the leading `[…]` option bracket and a
//! `.cls`/`.sty` suffix are stripped off, and the bracket's contents come back
//! as a separate `options` attribute. The Rust port used `Regex::replace_all`,
//! which *returns* the rewritten string instead of mutating its input, and
//! discarded every result — so nothing was ever stripped and the options
//! attribute was never emitted (`SYNC_STATUS.md` R2, second divergence; the
//! entry recorded only the `.cls` half of it).
//!
//! Every expectation below was ground-truthed against Perl LaTeXML 0.8.8 on the
//! same input, not just read off `Core.pm`.

use std::{path::Path, process::Command};

/// No `\documentclass` — a preloaded class is the point of the exercise, and
/// this is the exact input the Perl comparison was run on.
const DOC: &str = "\\begin{document}\nHello.\n\\end{document}\n";

fn preload_pi(spec: &str) -> String {
  let bin = env!("CARGO_BIN_EXE_latexml_oxide");
  assert!(Path::new(bin).is_file(), "binary not staged at {bin}");
  let workdir = tempfile::tempdir().expect("create tempdir");
  std::fs::write(workdir.path().join("p.tex"), DOC).expect("write p.tex");
  let output = Command::new(bin)
    .args(["p.tex", "--dest", "p.xml", "--nocomments"])
    .arg(format!("--preload={spec}"))
    .current_dir(workdir.path())
    .output()
    .expect("spawn latexml_oxide");
  let xml = std::fs::read_to_string(workdir.path().join("p.xml")).unwrap_or_else(|e| {
    let stderr = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
    panic!("no output for --preload={spec}: {e}\n{stderr}");
  });
  xml
    .lines()
    .find(|l| l.starts_with("<?latexml class=") || l.starts_with("<?latexml package="))
    .unwrap_or("<<no class/package PI>>")
    .to_string()
}

#[test]
fn preload_spec_is_stripped_to_its_bare_name_in_the_pi() {
  // (spec, expected PI) — verbatim Perl LaTeXML 0.8.8 output.
  let cases = [
    ("article.cls", "<?latexml class=\"article\"?>"),
    (
      "[twocolumn,11pt]article.cls",
      "<?latexml class=\"article\" options=\"twocolumn,11pt\"?>",
    ),
    (
      "[dvipsnames]color.sty",
      "<?latexml package=\"color\" options=\"dvipsnames\"?>",
    ),
    ("color.sty", "<?latexml package=\"color\"?>"),
    // An empty bracket is FALSY in Perl, so it contributes no attribute.
    ("[]color.sty", "<?latexml package=\"color\"?>"),
    // Only the two literal package suffixes are stripped — anything else stays
    // attached. This is why the loop cannot reuse `parse_preload_spec`, which
    // splits on the last `.` and would emit `package="mystyle"` here.
    ("mystyle.tex", "<?latexml package=\"mystyle.tex\"?>"),
  ];
  for (spec, expected) in cases {
    assert_eq!(preload_pi(spec), expected, "--preload={spec}");
  }
}
