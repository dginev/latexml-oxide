//! Regression guard for issue #304's mechanism: file resolution must survive a
//! process that cannot resolve a `kpsewhich` executable.
//!
//! Before kpathsea 0.3.4, `Kpaths::new()` returned `Err` whenever `kpsewhich`
//! was unresolvable *from this process* — absent from its PATH (as opposed to
//! the user's interactive shell), a stale `KPSEWHICH`, not executable, or a
//! `kpsewhich.exe` beside a Linux binary under WSL — and `select_kpaths()`
//! discarded that error with `.ok()?`. The linked libkpathsea was then never
//! initialized, so EVERY lookup returned `None` while embedded bindings and
//! dumps kept the conversion working. The only symptom was `Can't find TeX
//! file X`, indistinguishable from a genuinely absent file.
//!
//! Runs the binary as a subprocess, because the backend is chosen once per
//! process and cannot be re-selected from inside a test.

use std::{fs, process::Command};

/// `\input`s a file reachable ONLY through kpathsea's `TEXINPUTS` handling —
/// not via `--path`, which is resolved Rust-side and would bypass the backend
/// entirely (exactly how the reporter's workaround masked the problem).
const MAIN_TEX: &str = "\\documentclass{article}\n\
                        \\input{lxo_probe_304}\n\
                        \\begin{document}\n\
                        \\lxoprobe\n\
                        \\end{document}\n";

/// Requires a host TeX installation, which is optional for latexml-oxide, so
/// this is `ignore`d rather than failed where none exists — and `ignore` rather
/// than an early `return`, so the skip is visible in the test summary instead
/// of reporting green while asserting nothing. It further assumes a linked
/// libkpathsea (the default wherever `libkpathsea` is present at build time);
/// on a subprocess-only build the deliberately-broken `KPSEWHICH` below leaves
/// no backend at all, which is a different scenario than the one guarded here.
#[test]
#[cfg_attr(
  not(building_with_texlive),
  ignore = "requires a TeX Live installation"
)]
fn texinputs_resolves_without_a_resolvable_kpsewhich() {
  let bin = env!("CARGO_BIN_EXE_latexml_oxide");
  let dir = std::env::temp_dir().join(format!("lxo_kpse_backend_{}", std::process::id()));
  let include = dir.join("include");
  fs::create_dir_all(&include).unwrap();
  fs::write(
    include.join("lxo_probe_304.tex"),
    "\\newcommand\\lxoprobe{PROBE-OK}\n",
  )
  .unwrap();
  fs::write(dir.join("main.tex"), MAIN_TEX).unwrap();

  let sep = if cfg!(windows) { ';' } else { ':' };
  let out = Command::new(bin)
    .current_dir(&dir)
    .arg("--destination=out.xml")
    .arg("main.tex")
    .env("TEXINPUTS", format!("{}{sep}", include.display()))
    // No `kpsewhich` reachable: the condition that used to disable the linked
    // libkpathsea outright. `KPSEWHICH` is honored ahead of PATH by the
    // kpathsea crate, so pointing it at a nonexistent file is enough, and
    // unlike clearing PATH it stays portable.
    .env("KPSEWHICH", "/nonexistent/definitely-not-kpsewhich")
    .output()
    .expect("failed to run latexml_oxide");

  let log = String::from_utf8_lossy(&out.stderr).into_owned();
  let produced = fs::read_to_string(dir.join("out.xml")).unwrap_or_default();
  let _ = fs::remove_dir_all(&dir);

  assert!(
    !log.contains("Error:missing_file:lxo_probe_304"),
    "the TEXINPUTS-only include must resolve with no usable kpsewhich; log:\n{log}"
  );
  assert!(
    produced.contains("PROBE-OK"),
    "the included macro must have been expanded; log:\n{log}"
  );
  // The backend line is what makes a future report self-diagnosing: whatever
  // the outcome, the log must say which resolver was in play.
  assert!(
    log.contains("kpathsea:backend"),
    "every conversion log must record the resolved kpathsea backend; log:\n{log}"
  );
}
