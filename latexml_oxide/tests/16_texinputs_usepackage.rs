//! GitHub #345: `\usepackage{X}` must find a runtime `X.sty.rhai` binding placed
//! in a texmf tree on `$TEXINPUTS` — the same way `\input{file}` already resolves
//! files there — without needing an explicit `--path`.
//!
//! The `.rhai` discovery (`converter.rs::rhai_dispatch`) searched the local
//! search paths ONLY (`--path` + the source dir) and skipped kpsewhich, which is
//! what honours `$TEXINPUTS`. kpsewhich locates a `.sty.rhai` on TEXINPUTS just
//! fine (the extension is irrelevant to a `//` recursive search), so consulting
//! it closes the `\input`-works-but-`\usepackage`-doesn't asymmetry the reporter
//! hit.
//!
//! The TeX-tree probe is the **last** tier of the binding chain, not the first:
//! a `.rhai` beside your document is an *override*, one that merely sits in a
//! texmf tree only *fills a gap*. The two `..._shadow_...` tests below pin both
//! halves of that split — see `converter.rs::install_binding_dispatch`.

mod common;

use std::{path::Path, process::Command};

use common::strip_ansi;

/// Deliberately not a real CTAN package name: the fixture must be absent from
/// every host texmf tree, or the "no binding" leg would resolve a real `.sty`.
const PKG: &str = "lxonowrap";

/// Marker text a loaded `.rhai` emits, so "did this binding run?" is a single
/// unambiguous substring rather than an inference from the package name.
fn binding(marker: &str) -> String {
  format!("DefEnvironment(\"{{{PKG}}}\", \"<ltx:block class='{marker}'>#body</ltx:block>\");\n")
}

const DOC: &str = "\\documentclass[12pt]{book}\n\
                   \\usepackage{lxonowrap}\n\
                   \\begin{document}\n\
                   \\begin{lxonowrap}wrapped text\\end{lxonowrap}\n\
                   \\end{document}\n";

/// Convert `index.tex` in `dir`, returning (ANSI-free log, output HTML).
fn convert(dir: &Path, texinputs: Option<&str>) -> (String, String) {
  let mut cmd = Command::new(env!("CARGO_BIN_EXE_latexml_oxide"));
  cmd
    .args(["--dest", "index.html", "index.tex"])
    .current_dir(dir);
  match texinputs {
    Some(paths) => cmd.env("TEXINPUTS", paths),
    // Set it to just `.` rather than leaving it unset: an ambient `$TEXINPUTS`
    // from the developer's shell must not be what decides the negative test.
    None => cmd.env("TEXINPUTS", "."),
  };
  let out = cmd.output().expect("spawn latexml_oxide");
  let html = std::fs::read_to_string(dir.join("index.html")).unwrap_or_default();
  (strip_ansi(&String::from_utf8_lossy(&out.stderr)), html)
}

/// A texmf tree holding `<name>` with `content`, plus a `doc/` dir holding
/// `index.tex`. Returns (tempdir, doc path, `$TEXINPUTS` value reaching it).
fn fixture(
  name: &str,
  content: &str,
  tex: &str,
) -> (tempfile::TempDir, std::path::PathBuf, String) {
  let work = tempfile::tempdir().expect("tempdir");
  // Buried a few levels deep, so only the recursive `//` search reaches it.
  let styles = work.path().join("texmf/tex/latex/mystyles");
  std::fs::create_dir_all(&styles).unwrap();
  std::fs::write(styles.join(name), content).unwrap();
  let doc = work.path().join("doc");
  std::fs::create_dir_all(&doc).unwrap();
  std::fs::write(doc.join("index.tex"), tex).unwrap();
  let texinputs = format!(".:{}//:", work.path().join("texmf").display());
  (work, doc, texinputs)
}

/// A `<pkg>.sty.rhai` under a `$TEXINPUTS` texmf tree (recursive `//`, no
/// `--path`) must be discovered and loaded by `\usepackage{<pkg>}`.
#[cfg_attr(
  not(building_with_texlive),
  ignore = "requires a TeX Live installation (kpsewhich resolves $TEXINPUTS)"
)]
#[test]
fn usepackage_finds_rhai_binding_on_texinputs() {
  let (_work, doc, texinputs) = fixture(&format!("{PKG}.sty.rhai"), &binding("lxo-loaded"), DOC);
  let (log, html) = convert(&doc, Some(&texinputs));

  assert!(
    !log.contains(&format!("missing_file:{PKG}")),
    "\\usepackage{{{PKG}}} must resolve {PKG}.sty.rhai via TEXINPUTS (no --path); log:\n{log}"
  );
  // The binding actually RAN: its constructor emitted the block. Asserting the
  // marker class (not the package name) is what makes this discriminating —
  // the undefined-environment fallback also prints the package name, into an
  // `ltx_ERROR` span, which is exactly what `..._is_undefined_without_texinputs`
  // pins below.
  assert!(
    html.contains("lxo-loaded") && html.contains("wrapped text"),
    "the {PKG} environment (from the TEXINPUTS .rhai) should render its block; html:\n{html}"
  );
  assert!(
    !html.contains("ltx_ERROR"),
    "a loaded binding leaves no error node in the document; html:\n{html}"
  );
}

/// The negative control for the assertions above: with the tree off
/// `$TEXINPUTS`, the very same document must FAIL, and must fail without
/// producing the marker. Without this, "html contains the package name" would
/// pass on a broken binary too — the undefined-environment recovery emits
/// `<span class="ltx_ERROR undefined">{lxonowrap}</span>`.
#[test]
fn package_is_undefined_without_texinputs() {
  let (_work, doc, _texinputs) = fixture(&format!("{PKG}.sty.rhai"), &binding("lxo-loaded"), DOC);
  let (log, html) = convert(&doc, None);

  assert!(
    log.contains(&format!("missing_file:{PKG}")),
    "with no TEXINPUTS there is nothing to find; log:\n{log}"
  );
  assert!(
    !html.contains("lxo-loaded") && html.contains("ltx_ERROR"),
    "the failing conversion must not produce the marker; html:\n{html}"
  );
}

/// Authority half of the tier split: a `.rhai` that merely sits on the TeX tree
/// FILLS A GAP — it must not displace a compiled binding of the same name.
/// Before the TeX-tree probe was demoted to the last tier, a stray
/// `amsmath.sty.rhai` anywhere on `$TEXINPUTS` replaced the whole compiled
/// `amsmath` binding, and `\begin{align}` became an undefined `\align`.
#[cfg_attr(
  not(building_with_texlive),
  ignore = "requires a TeX Live installation (kpsewhich resolves $TEXINPUTS)"
)]
#[test]
fn texmf_rhai_does_not_shadow_a_compiled_binding() {
  let (_work, doc, texinputs) = fixture(
    "amsmath.sty.rhai",
    "DefMacro(\"\\\\lxoprobe\", \"TEXMF-RHAI-WON\");\n",
    "\\documentclass{article}\n\
     \\usepackage{amsmath}\n\
     \\begin{document}\n\
     \\begin{align}a&=b\\end{align}\n\
     \\end{document}\n",
  );
  let (log, html) = convert(&doc, Some(&texinputs));

  assert!(
    !log.contains("undefined:\\align") && !log.contains("unexpected:&"),
    "the compiled amsmath binding must still win over a texmf .rhai; log:\n{log}"
  );
  assert!(
    !html.contains("TEXMF-RHAI-WON"),
    "the texmf .rhai must not have been loaded at all; html:\n{html}"
  );
}

/// Cost/authority half kept intact: a `.rhai` in the document's own directory
/// still overrides the compiled binding of the same name (the documented
/// tier-1 behaviour — `script_bindings_plan.md` §7).
#[test]
fn local_rhai_still_overrides_a_compiled_binding() {
  let work = tempfile::tempdir().expect("tempdir");
  let doc = work.path().join("doc");
  std::fs::create_dir_all(&doc).unwrap();
  std::fs::write(
    doc.join("amsmath.sty.rhai"),
    "DefMacro(\"\\\\lxoprobe\", \"LOCAL-RHAI-WON\");\n",
  )
  .unwrap();
  std::fs::write(
    doc.join("index.tex"),
    "\\documentclass{article}\n\
     \\usepackage{amsmath}\n\
     \\begin{document}\n\
     \\lxoprobe\n\
     \\end{document}\n",
  )
  .unwrap();
  let (_log, html) = convert(&doc, None);

  assert!(
    html.contains("LOCAL-RHAI-WON"),
    "a .rhai beside the document overrides the compiled binding; html:\n{html}"
  );
}
