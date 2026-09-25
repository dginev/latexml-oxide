use std::{path::Path, process::Command};

const EXPL3_TEX: &str = r"\documentclass{article}
\usepackage{expl3}
\begin{document}
degraded-body-text
\end{document}
";

fn convert_nodump() -> (bool, String) {
  let bin = env!("CARGO_BIN_EXE_latexml_oxide");
  assert!(Path::new(bin).is_file(), "binary not staged at {bin}");
  let workdir = tempfile::tempdir().expect("create tempdir");
  std::fs::write(workdir.path().join("e.tex"), EXPL3_TEX).expect("write e.tex");
  let output = Command::new(bin)
    // `--timeout 900`: the unoptimized test-build raw expl3 load runs ~2 min,
    // far over the 60 s CLI default; 900 s stays well under nextest's 20 min
    // terminate-after so a genuine hang still surfaces.
    .args([
      "e.tex",
      "--dest",
      "e.xml",
      "--nocomments",
      "--timeout",
      "900",
    ])
    .env("LATEXML_NODUMP", "1")
    .current_dir(workdir.path())
    .output()
    .expect("spawn latexml_oxide");
  let stderr = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
  let xml = std::fs::read_to_string(workdir.path().join("e.xml")).unwrap_or_default();
  (
    output.status.success(),
    format!("{stderr}\n<<<XML>>>\n{xml}"),
  )
}

#[test]
fn expl3_converts_healthily_without_a_dump() {
  let (ok, out) = convert_nodump();
  assert!(
    ok,
    "degraded no-dump expl3 conversion exited non-zero:\n{out}"
  );
  assert!(
    !out.contains("fatal error"),
    "degraded no-dump conversion leaked a fatal (benign expl3 cascade):\n{out}"
  );
  assert!(
    out.contains("degraded-body-text"),
    "degraded no-dump conversion dropped the body:\n{out}"
  );
}
