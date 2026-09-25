//! Guards for the raw-interpretation preload technique
//! (`--preload=[rawstyles,rawclasses]latexml.sty`, the perfect-kernel
//! protocol — `docs/perfect_kernel/README.md`). User directive 2026-08-31:
//!
//! 1. A compiled `.rs` binding ALWAYS takes precedence, raw mode included —
//!    `rawstyles`/`rawclasses` never demote an existing binding.
//! 2. For a class with NO binding, `rawclasses` raw-loads the `.cls`
//!    through the TeX engine and the OmniBus fallback must NOT fire.
//!
//! Exercised with workdir-local class files, so the test needs no host
//! texmf packages: `daj` (contrib-bound name whose binding is a compiled
//! article-based definition set — `scrartcl` served here until batch 53
//! made its binding a raw shim that itself `\input`s the `.cls`) for
//! precedence, `pkzzz` (no binding anywhere) for the no-OmniBus raw load.

fn convert(class: &str, cls_body: &str, preload: Option<&str>) -> (String, String) {
  let cls_name = format!("{class}.cls");
  let tex = format!(
    "\\documentclass{{{class}}}\n\\begin{{document}}\n\
       \\ifdefined\\rawmarker\\rawmarker\\else NOMARKER\\fi\n\\end{{document}}\n"
  );
  let (stderr, xml) = super::convert_files_with(&tex, &[(&cls_name, cls_body)], preload);
  (xml, stderr)
}

const RAW_CLS: &str = "\\ProvidesClass{whatever}\n\
    \\LoadClass{article}\n\
    \\newcommand{\\rawmarker}{RAWCLSLOADED}\n";

/// Directive 1: the contrib `daj` binding wins even under rawclasses —
/// the local raw `.cls`'s marker must NOT appear.
#[test]
fn contrib_binding_keeps_precedence_under_rawclasses() {
  let (xml, _stderr) = convert("daj", RAW_CLS, Some("[rawstyles,rawclasses]latexml.sty"));
  assert!(
    xml.contains("NOMARKER") && !xml.contains("RAWCLSLOADED"),
    "compiled daj binding must outrank the raw .cls under rawclasses:\n{xml}",
  );
}

/// Directive 2: a bindingless class raw-loads under rawclasses; OmniBus
/// stays out of the conversion entirely.
#[test]
fn bindingless_class_raw_loads_without_omnibus() {
  let (xml, stderr) = convert("pkzzz", RAW_CLS, Some("[rawstyles,rawclasses]latexml.sty"));
  assert!(
    xml.contains("RAWCLSLOADED"),
    "bindingless pkzzz.cls should raw-load under rawclasses:\n{xml}\nstderr:\n{stderr}",
  );
  assert!(
    !stderr.contains("OmniBus"),
    "OmniBus must not fire for a raw-loaded bindingless class:\n{stderr}",
  );
}

/// Control: without rawclasses the same bindingless class falls back to
/// OmniBus (the pre-existing default behavior, unchanged by the mission).
#[test]
fn bindingless_class_defaults_to_omnibus_without_rawclasses() {
  let (xml, stderr) = convert("pkzzz", RAW_CLS, None);
  assert!(
    xml.contains("NOMARKER") && stderr.contains("OmniBus"),
    "default mode should keep the OmniBus fallback for unknown classes:\n{stderr}",
  );
}
