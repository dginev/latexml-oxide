//! RAWDEFW: the CLI raw-loads an unbound `.sty`/`.cls` by default; `--noincludestyles`, or latexml.sty's
//! `norawstyles,norawclasses`, restores Perl's default (OXIDIZED_DESIGN_DIVERGENCES #1080). The in-process harness keeps Perl's default (`Config::default`), so these
//! spawn the binary.

use std::{path::Path, process::Command};

const DIR: &str = concat!(
  env!("CARGO_MANIFEST_DIR"),
  "/../tools/perfect_kernel/repros/loader/cli_raw_default"
);
const MAIN: &str = "cli_default_raw_loads_unbound_package_and_class.tex";

/// Convert the repro (and its local `.sty`/`.cls`) in a fresh directory with `args`; returns (stderr, xml).
fn run(args: &[&str]) -> (String, String) {
  let work = tempfile::tempdir().expect("tempdir");
  for name in [MAIN, "pkzzzraw.sty", "pkzzzcls.cls"] {
    std::fs::copy(Path::new(DIR).join(name), work.path().join(name)).expect("stage the repro");
  }
  let out = Command::new(env!("CARGO_BIN_EXE_latexml_oxide"))
    .args([MAIN, "--dest", "out.xml", "--nocomments"])
    .args(args)
    .current_dir(work.path())
    .output()
    .expect("spawn latexml_oxide");
  let stderr = String::from_utf8_lossy(&out.stderr).to_string();
  assert!(
    out.status.success(),
    "exit {:?}:\n{stderr}",
    out.status.code()
  );
  let xml = std::fs::read_to_string(work.path().join("out.xml")).expect("read out.xml");
  (stderr, xml)
}

fn warnings(stderr: &str) -> Vec<&str> {
  stderr
    .lines()
    .filter(|l| l.starts_with("Warning:"))
    .collect()
}

/// The paragraph's text: the class and the package each say whether they were raw-loaded.
fn answer(xml: &str) -> &str {
  let start = xml.find("<p>").expect("a paragraph") + 3;
  &xml[start..start + xml[start..].find("</p>").expect("its end")]
}

/// Default (no flags): both raw-load, error- and warning-free, as pdflatex (its PDF: "RAWCLSLOADED RAWSTYLOADED").
/// The last of `--noincludestyles --includestyles` wins, as `--noplane1 --plane1` does.
#[test]
fn cli_default_raw_loads_unbound_package_and_class() {
  for args in [&[][..], &["--noincludestyles", "--includestyles"][..]] {
    let (stderr, xml) = run(args);
    assert_eq!(super::error_count(&stderr), 0, "{args:?}\n{stderr}");
    assert_eq!(warnings(&stderr), Vec::<&str>::new(), "{args:?}");
    assert_eq!(answer(&xml), "RAWCLSLOADED RAWSTYLOADED", "{args:?}\n{xml}");
    assert!(xml.contains("<?latexml class=\"pkzzzcls\"?>"), "{xml}");
  }
}

/// `--noincludestyles` (and the existing `--preload=[norawstyles,norawclasses]latexml.sty`) is Perl LaTeXML's
/// default: bindings only, the package skipped with a missing_file warning and the class served by OmniBus.
/// `--includestyles --noincludestyles`: the last one wins.
#[test]
fn cli_noincludestyles_restores_perls_binding_only_loading() {
  for args in [
    &["--noincludestyles"][..],
    &["--includestyles", "--noincludestyles"][..],
    &["--preload=[norawstyles,norawclasses]latexml.sty"][..],
  ] {
    let (stderr, xml) = run(args);
    assert_eq!(super::error_count(&stderr), 0, "{args:?}\n{stderr}");
    assert_eq!(
      warnings(&stderr),
      [
        "Warning:missing_file:pkzzzcls Can't find binding for class pkzzzcls (using OmniBus)",
        "Warning:missing_file:pkzzzraw Can't find binding or file for 'pkzzzraw.sty'. no dispatcher entry, and \
         raw TeX loading is off (latexml.sty's rawstyles turns it on).",
      ],
      "{args:?}"
    );
    assert_eq!(answer(&xml), "NOCLS NOSTY", "{args:?}\n{xml}");
  }
}

/// A latexml.sty limit option loads after the front end's ar5iv default and wins (Perl latexml.sty.ltxml:100-102,
/// `$LaTeXML::IF_LIMIT = int(…)`): `iflimit=50` stops a 100-conditional loop with the Fatal, `iflimit=0` lifts the
/// limit, Perl's default.
#[test]
fn latexml_sty_iflimit_overrides_the_front_end_default() {
  let tex = "\\documentclass{article}\n\\newcount\\n\n\\begin{document}\n\
    \\loop\\ifnum\\n<100 \\advance\\n by 1 \\repeat Done \\the\\n.\n\\end{document}\n";
  let work = tempfile::tempdir().expect("tempdir");
  std::fs::write(work.path().join("i.tex"), tex).expect("write i.tex");
  let run_with = |preload: &str| {
    let out = Command::new(env!("CARGO_BIN_EXE_latexml_oxide"))
      .args(["i.tex", "--dest", "i.xml", "--nocomments", preload])
      .current_dir(work.path())
      .output()
      .expect("spawn latexml_oxide");
    String::from_utf8_lossy(&out.stderr).to_string()
  };
  let stderr = run_with("--preload=[iflimit=50]latexml.sty");
  assert!(
    stderr.contains("Fatal:Timeout:IfLimit")
      && stderr.contains("Conditional limit of 50 exceeded, infinite loop?"),
    "{stderr}"
  );
  let stderr = run_with("--preload=[iflimit=0]latexml.sty");
  assert_eq!(super::error_count(&stderr), 0, "{stderr}");
  let xml = std::fs::read_to_string(work.path().join("i.xml")).expect("read i.xml");
  assert!(xml.contains("Done 100."), "{xml}");
}
