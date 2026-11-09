//! Guards for batch 87a: each binding declares its frontmatter conventions at load (`declare_frontmatter_conventions`,
//! user ruling 2026-10-28 "in each binding"; OXIDIZED_DESIGN_DIVERGENCES #837), and the observer reads them from the
//! state, as it read the name-keyed policy table before — the table is gone. Every probe compiles clean with pdflatex
//! (TL 2025) and converts with no Error or Warning.
use latexml::util::test::{convert_with_setup_then, error_count, warning_count};
use latexml_engine::base_utilities::fm_reading_json;
use serde_json::Value;

/// Each author block of `tex`'s conversion: the binding its conventions are keyed on, and the
/// conventions it ran under (by name), as the dump records them.
fn conventions(tex: &str) -> Vec<(String, String)> {
  let (log, _, json) =
    convert_with_setup_then(tex, Some("ar5iv.sty"), || {}, |_| fm_reading_json());
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

/// 87a r2 (review S1), the live half (the census's `DECLARED` list went with `ab_audit fmshadow`, DEL4): every
/// `declare_frontmatter_conventions(&FRONTMATTER_CONVENTIONS)` call in `latexml_package/src/package/*.rs` and
/// `latexml_contrib/src/*.rs` is of that form, and its constant's `binding:` is a registered binding's name and
/// extension, the entry whose loader is the declaring file's own. A copied `binding: "jheppub.sty"` would key the family
/// on another's file, so `policy_for` would find no declaration for it and give it the kernel's conventions in live
/// conversions, in silence.
#[test]
fn each_declared_binding_is_the_declaring_files_own_registry_entry() {
  use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
  };
  let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
    .parent()
    .expect("workspace");
  let call = "declare_frontmatter_conventions(&FRONTMATTER_CONVENTIONS)";
  let binding_of = |text: &str| -> String {
    let start = text
      .find("pub const FRONTMATTER_CONVENTIONS: FmConventions = FmConventions {")
      .expect("the declared constant");
    let body = &text[start..start + text[start..].find("\n};").expect("constant end")];
    let at = body.find("\n  binding: \"").expect("its `binding:`");
    let value = &body[at + "\n  binding: \"".len()..];
    value[..value.find('"').expect("quote")].to_string()
  };
  let entry =
    regex::Regex::new(r#"\(\s*"([^"]+)",\s*"([^"]+)",\s*(?:\w+::)*(\w+)::load_definitions,?\s*\)"#)
      .expect("regex");
  // (each binding module's registry names, `name.ext`, from the crates' `BINDINGS` tables)
  let mut registered: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
  for lib in ["latexml_package/src/lib.rs", "latexml_contrib/src/lib.rs"] {
    let text = fs::read_to_string(workspace.join(lib)).expect("lib.rs");
    for c in entry.captures_iter(&text) {
      registered
        .entry(c[3].to_string())
        .or_default()
        .insert(format!("{}.{}", &c[1], &c[2]));
    }
  }
  let mut scanned: BTreeSet<String> = BTreeSet::new();
  for dir in ["latexml_package/src/package", "latexml_contrib/src"] {
    for file in fs::read_dir(workspace.join(dir)).expect("dir").flatten() {
      let path = file.path();
      if path.extension().is_none_or(|e| e != "rs") {
        continue;
      }
      let text = fs::read_to_string(&path).expect("source");
      let stem = path
        .file_stem()
        .expect("stem")
        .to_string_lossy()
        .into_owned();
      let code: Vec<&str> = text
        .lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect();
      let calls = code
        .iter()
        .filter(|l| l.contains("declare_frontmatter_conventions("))
        .count();
      if calls == 0 {
        continue;
      }
      assert_eq!(
        code.iter().filter(|l| l.contains(call)).count(),
        calls,
        "{stem}: a declaration of another form"
      );
      let binding = binding_of(&text);
      assert!(
        registered
          .get(&stem)
          .is_some_and(|names| names.contains(&binding)),
        "{stem}: `binding: \"{binding}\"` is not its registry entry ({:?})",
        registered.get(&stem)
      );
      assert!(
        scanned.insert(binding.clone()),
        "`{binding}` declared twice"
      );
    }
  }
  // (the 19 declaring bindings at DEL4: a scan that finds fewer has lost its reading, not the declarations)
  assert!(
    scanned.len() >= 19,
    "only {} declarations found: {scanned:?}",
    scanned.len()
  );
  // (and each is a binding the crates register under that name and extension, read from the tables themselves)
  let names: BTreeSet<String> = latexml_package::BINDINGS
    .iter()
    .map(|(name, ext, _)| format!("{name}.{ext}"))
    .chain(
      latexml_contrib::BINDINGS
        .iter()
        .map(|(name, ext, _)| format!("{name}.{ext}")),
    )
    .collect();
  for binding in &scanned {
    assert!(names.contains(binding), "{binding}");
  }
}
