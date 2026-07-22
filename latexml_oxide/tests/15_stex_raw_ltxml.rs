//! Raw-loading a style package under `--includestyles` must (a) never read a
//! Perl `.ltxml` binding as TeX, and (b) restore the `standalone → currfile →
//! filehook` dependency chain so the package-file hooks exist.
//!
//! Origin: raw `stex.sty` (sTeX 3.x) under the ar5iv config. stex ships BOTH
//! `stex.sty` (real TeX) and `stex.sty.ltxml` (a Perl LaTeXML binding) in TeX
//! Live, and `stex.sty` uses `\AtEndOfPackageFile` (filehook, reached via
//! standalone → currfile) and `\define@key` (xkeyval). Two bugs surfaced:
//!   1. `find_file` returned `stex.sty.ltxml` (kpsewhich lists it) ahead of the
//!      raw `stex.sty`, and the raw-loader tokenized the Perl source as TeX
//!      (`$out =~ s/^\s+//;` → "Script ^…", `\DefMacroI`/`\stex@backend`
//!      undefined). latexml-oxide can never read a `.ltxml`; binding availability
//!      is decided by the dispatcher, not a `.ltxml` on disk.
//!   2. the simplified `standalone` binding dropped standalone.sty's unconditional
//!      `\RequirePackage{xkeyval}` / `\RequirePackage{currfile}` (→ filehook), so
//!      `\AtEndOfPackageFile` / `\define@key` were undefined.

use std::{path::Path, process::Command};

fn strip_ansi(s: &str) -> String {
  // Drop CSI sequences `\x1b[ … m` so `Error:`/`Fatal:` counting is reliable.
  let mut out = String::with_capacity(s.len());
  let mut chars = s.chars().peekable();
  while let Some(c) = chars.next() {
    if c == '\x1b' {
      // Skip until the final byte of the CSI sequence (a letter).
      for d in chars.by_ref() {
        if d.is_ascii_alphabetic() {
          break;
        }
      }
    } else {
      out.push(c);
    }
  }
  out
}

fn convert(work: &Path, doc: &str) -> String {
  std::fs::write(work.join("doc.tex"), doc).expect("write doc.tex");
  let out = Command::new(env!("CARGO_BIN_EXE_latexml_oxide"))
    .args(["--includestyles", "--dest", "doc.xml", "doc.tex"])
    .current_dir(work)
    .output()
    .expect("spawn latexml_oxide");
  strip_ansi(&String::from_utf8_lossy(&out.stderr))
}

fn error_count(log: &str) -> usize {
  log
    .lines()
    .filter(|l| l.starts_with("Error:") || l.starts_with("Fatal:"))
    .count()
}

fn kpsewhich_has(name: &str) -> bool {
  Command::new("kpsewhich")
    .arg(name)
    .output()
    .map(|o| o.status.success() && !o.stdout.is_empty())
    .unwrap_or(false)
}

/// Self-contained (all bindings are compiled in — no TeX Live package needed):
/// under `--includestyles`, `\usepackage{standalone}` must pull in the
/// `xkeyval` + `currfile → filehook` chain so `\AtEndOfPackageFile` is defined.
#[test]
fn standalone_under_includestyles_provides_filehook_hooks() {
  let work = tempfile::tempdir().expect("tempdir");
  let log = convert(
    work.path(),
    "\\documentclass{article}\n\
     \\usepackage{standalone}\n\
     \\AtEndOfPackageFile{graphicx}{\\typeout{DEFERRED}}\n\
     \\begin{document}\nHello.\n\\end{document}\n",
  );
  assert!(
    !log.contains("AtEndOfPackageFile") && !log.contains("define@key"),
    "standalone under --includestyles must define the filehook/xkeyval hooks \
     (standalone → currfile → filehook, standalone → xkeyval); log:\n{log}"
  );
  assert_eq!(
    error_count(&log),
    0,
    "expected a clean conversion; log:\n{log}"
  );
}

/// The real witness: raw `stex.sty` must load — never its Perl `stex.sty.ltxml`
/// — and convert cleanly. Skipped where TeX Live lacks stex.
#[test]
fn raw_stex_sty_loads_not_the_perl_ltxml() {
  if !kpsewhich_has("stex.sty") || !kpsewhich_has("stex.sty.ltxml") {
    eprintln!("stex.sty / stex.sty.ltxml not in TeX Live — skipping");
    return;
  }
  let work = tempfile::tempdir().expect("tempdir");
  let log = convert(
    work.path(),
    "\\documentclass{article}\n\\usepackage{stex}\n\
     \\begin{document}\nHello sTeX.\n\\end{document}\n",
  );
  // The Perl binding must never be read as TeX.
  assert!(
    !log.contains("stex.sty.ltxml"),
    "the Perl stex.sty.ltxml must never be read (latexml-oxide can't read .ltxml); log:\n{log}"
  );
  assert!(
    !log.contains("DefMacroI") && !log.contains("stex@backend"),
    "Perl-syntax-as-TeX errors present — the .ltxml was misread; log:\n{log}"
  );
  // And the raw load (stex → standalone → currfile → filehook, xkeyval) is clean.
  assert_eq!(
    error_count(&log),
    0,
    "raw stex.sty must convert with 0 errors; log:\n{log}"
  );
}
