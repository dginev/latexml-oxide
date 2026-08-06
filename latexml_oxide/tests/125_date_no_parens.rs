//! The title-page date must render WITHOUT surrounding parentheses — full
//! pipeline (runs XSLT).
//!
//! arXiv html_feedback #1934 (arXiv:2408.08811v1): the title-page date showed as
//! `(August 1, 2024)`. LaTeXML's `dates` XSLT template
//! (`LaTeXML-structure-xhtml.xsl`) historically wrapped every date div in
//! `(...)` — a convention with no pdflatex counterpart (no LaTeX puts parens
//! around `\date`, titlepage or not). Removed for PDF fidelity, a surpass-Perl
//! divergence (OXIDIZED_DESIGN #102; same-host Perl still parenthesizes).
//!
//! The parens are added at the XSLT stage, so the in-process `Converter`
//! (`06_cluster_regressions.rs`) — which stops at Core XML — cannot see them;
//! this drives the binary end-to-end, like `07_xslt_seclev_levels.rs`.

use std::{path::Path, process::Command};

fn run(cwd: &Path, args: &[&str]) -> std::process::Output {
  Command::new(env!("CARGO_BIN_EXE_latexml_oxide"))
    .args(args)
    .current_dir(cwd)
    .output()
    .expect("spawn latexml_oxide")
}

const TEX: &str = "\\documentclass{article}\n\
   \\title{A Title}\n\
   \\author{An Author}\n\
   \\date{August 1, 2024}\n\
   \\begin{document}\\maketitle\\end{document}\n";

#[test]
fn date_renders_without_surrounding_parens() {
  let work = tempfile::tempdir().expect("tempdir");
  std::fs::write(work.path().join("d.tex"), TEX).unwrap();
  let out = run(work.path(), &["d.tex", "--dest", "d.html"]);
  assert!(
    out.status.success(),
    "conversion failed (status {:?}):\n{}",
    out.status.code(),
    String::from_utf8_lossy(&out.stderr)
  );
  let html = std::fs::read_to_string(work.path().join("d.html")).expect("read d.html");

  // Isolate the dates div.
  let at = html
    .find("ltx_dates")
    .expect("no ltx_dates div in output — the date was lost");
  let tail = &html[at..];
  let end = tail.find("</div>").expect("unterminated ltx_dates div");
  let dates = &tail[..end];

  // The author's date is preserved…
  assert!(
    dates.contains("August 1, 2024"),
    "the date content was lost:\n{dates}"
  );
  // …but WITHOUT the LaTeXML-ism parentheses that no PDF shows.
  assert!(
    !dates.contains('(') && !dates.contains(')'),
    "the date is still wrapped in parentheses — the `dates` XSLT template's \
     `(`/`)` were not removed:\n{dates}"
  );
}
