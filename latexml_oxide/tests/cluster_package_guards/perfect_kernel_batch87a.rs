//! Guards for batch 87a: each binding declares its frontmatter conventions at load (`declare_frontmatter_conventions`,
//! user ruling 2026-10-28 "in each binding"; OXIDIZED_DESIGN_DIVERGENCES #837), and the observer reads them from the
//! state, as it read the name-keyed policy table before — the table is gone. Every probe compiles clean with pdflatex
//! (TL 2025) and converts with no Error or Warning.
use latexml::util::test::{convert_with_setup_then, error_count, warning_count};
use latexml_engine::base_utilities::{fm_shadow_json, set_fm_shadow_override};
use serde_json::Value;

/// Each author block of `tex`'s conversion under the shadow switch: the binding its conventions are keyed on, and the
/// conventions it ran under (by name), as the dump records them.
fn conventions(tex: &str) -> Vec<(String, String)> {
  let (log, _, json) = convert_with_setup_then(
    tex,
    Some("ar5iv.sty"),
    || set_fm_shadow_override(Some(true)),
    |_| fm_shadow_json(),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let dump: Value = super::perfect_kernel_batchfms2a1::parsed(&json);
  dump["entries"]
    .as_array()
    .into_iter()
    .flatten()
    .filter(|e| e["kind"] == "author")
    .map(|e| {
      (
        e["binding"].as_str().unwrap_or("").to_string(),
        e["policy"].as_str().unwrap_or("").to_string(),
      )
    })
    .collect()
}

fn pair(binding: &str, name: &str) -> (String, String) { (binding.to_string(), name.to_string()) }

/// The three ways a block reaches a binding's declaration — the class's binding owning `\author` (llncs), a binding
/// the class's loads owning it (revtex4-1's revtex4_support), the class's binding under the kernel `\author`
/// (`ledger::policy_binding`: report) — and a family no binding declared for, the kernel's (article).
#[test]
fn a_bindings_declared_conventions_are_read_and_an_undeclared_family_is_the_kernels() {
  let llncs = "\\documentclass{llncs}\n\\begin{document}\n\\title{T}\n\\author{Ann Able\\inst{1} \\and Bob Baker\\inst{2}}\n\\institute{Univ A \\and Univ B}\n\\maketitle\nBody.\n\\end{document}\n";
  // (llncs's `\author` a block per `\and`-separated name)
  assert_eq!(conventions(llncs), [
    pair("llncs.cls", "llncs"),
    pair("llncs.cls", "llncs")
  ]);
  let revtex = "\\documentclass{revtex4-1}\n\\begin{document}\n\\title{T}\n\\author{Ann Able}\n\\affiliation{Univ A}\n\\begin{abstract}\nAbs.\n\\end{abstract}\n\\maketitle\nBody.\n\\end{document}\n";
  assert_eq!(conventions(revtex), [pair("revtex4_support.sty", "revtex")]);
  let report = "\\documentclass{report}\n\\begin{document}\n\\title{T}\n\\author{Ann Able\\thanks{Univ A}}\n\\maketitle\nBody.\n\\end{document}\n";
  assert_eq!(conventions(report), [pair("report.cls", "report")]);
  let article = "\\documentclass{article}\n\\begin{document}\n\\title{T}\n\\author{Ann Able\\thanks{Univ A}}\n\\maketitle\nBody.\n\\end{document}\n";
  assert_eq!(conventions(article), [pair("kernel", "kernel")]);
}
