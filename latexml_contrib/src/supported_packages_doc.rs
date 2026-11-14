//! Generates `docs/SUPPORTED_PACKAGES.md` from the two binding tables, `latexml_package::BINDINGS`
//! and this crate's `BINDINGS` (the single sources of truth `dispatch` reads), and checks it is
//! current. Regenerate with:
//!
//! ```sh
//! UPDATE_SUPPORTED_PACKAGES=1 cargo test -p latexml_contrib --lib supported_packages_doc_is_up_to_date
//! ```
use std::{
  collections::HashMap,
  path::{Path, PathBuf},
};

/// One row of a `BINDINGS` table as written in its `lib.rs`: the name, the extension and the
/// loader's module path (`package::amsmath_sty`, `keytheorems_sty`).
struct Entry {
  name:   String,
  ext:    String,
  module: String,
}

/// The `BINDINGS` table of a `lib.rs`, read from its source text: the loaders are fn pointers, so
/// the module a row names (whose file holds the binding's doc comment) is read where it is written.
fn table_entries(lib_rs: &str) -> Vec<Entry> {
  let start = lib_rs
    .find("pub const BINDINGS")
    .expect("a `pub const BINDINGS` table");
  let body_start = start + lib_rs[start..].find("= &[").expect("the table's `= &[`") + 4;
  let body_end = body_start + lib_rs[body_start..].find("\n];").expect("the table's `];`");
  let body: String = lib_rs[body_start..body_end]
    .lines()
    .map(|line| line.split("//").next().unwrap_or(""))
    .collect::<Vec<_>>()
    .join(" ");
  let mut entries = Vec::new();
  let mut rest = body.as_str();
  while let Some(open) = rest.find('(') {
    let close = open + rest[open..].find(')').expect("a closing `)` for each row");
    let fields: Vec<&str> = rest[open + 1..close].split(',').map(str::trim).collect();
    let unquote = |s: &str| s.trim_matches('"').to_string();
    let loader = fields[2];
    entries.push(Entry {
      name:   unquote(fields[0]),
      ext:    unquote(fields[1]),
      module: loader[..loader.rfind("::").expect("a `module::fn` loader")].to_string(),
    });
    rest = &rest[close + 1..];
  }
  entries
}

/// The source file of a loader's module, `<src>/<a>/<b>.rs` or `<src>/<a>/<b>/mod.rs`; an inline
/// module (`osa_journal_sty::opex3_sty`) is read in the file of its nearest file-backed parent.
fn module_file(src: &Path, module: &str) -> Option<PathBuf> {
  let segments: Vec<&str> = module.split("::").collect();
  (1..=segments.len()).rev().find_map(|len| {
    let rel: PathBuf = segments[..len].iter().collect();
    let file = src.join(&rel).with_extension("rs");
    let dir = src.join(&rel).join("mod.rs");
    [file, dir].into_iter().find(|path| path.is_file())
  })
}

/// What a binding's source says about itself: its module doc's first line naming it a stub or a
/// partial binding, and whether it reads its own TeX file raw (`InputDefinitions!("<name>"…noltxml`).
fn describe(source: &str, name: &str) -> (Option<String>, bool) {
  // The module doc, one paragraph per entry, lines joined.
  let doc: Vec<&str> = source
    .lines()
    .take_while(|line| line.trim_start().starts_with("//!") || line.trim().is_empty())
    .map(|line| line.trim_start().trim_start_matches("//!").trim())
    .collect();
  let note = doc
    .split(|line| line.is_empty())
    .map(|paragraph| paragraph.join(" "))
    .find_map(|paragraph| {
      paragraph
        .split_inclusive(". ")
        .find(|sentence| {
          let lower = sentence.to_lowercase();
          lower.contains("stub") || lower.contains("partial")
        })
        .map(str::to_string)
    })
    .map(|sentence| {
      // A long sentence is cut at a word before 200 characters; the source keeps it whole.
      let sentence = sentence.trim();
      let cut = match sentence.char_indices().nth(200) {
        Some((at, _)) => format!("{} …", &sentence[..sentence[..at].rfind(' ').unwrap_or(at)]),
        None => sentence.to_string(),
      };
      cut.replace('|', "\\|")
    });
  let call = format!("InputDefinitions!(\"{name}\"");
  let raw = source.match_indices(&call).any(|(at, _)| {
    let args = &source[at..];
    args[..args.find(')').unwrap_or(args.len())].contains("noltxml")
  });
  (note, raw)
}

const PREAMBLE: &str = "# Supported packages and classes

**Generated** from the binding tables — `latexml_package::BINDINGS` (`latexml_package/src/lib.rs`) and
`latexml_contrib::BINDINGS` (`latexml_contrib/src/lib.rs`), the lists `dispatch` reads — and checked on every test
run (`supported_packages_doc_is_up_to_date`); regenerate with
`UPDATE_SUPPORTED_PACKAGES=1 cargo test -p latexml_contrib --lib supported_packages_doc_is_up_to_date`.

- **ported** — a compiled binding in `latexml_package`, the tree that ports Perl LaTeXML's `.ltxml` bindings (a few
  of its rows are Rust-only support files or version aliases).
- **contributed** — a compiled binding in `latexml_contrib`, beyond Perl LaTeXML's set.
- **raw + overrides** — the binding reads the package's own TeX file raw (`InputDefinitions!(\"<name>\", noltxml …)`)
  and defines only what carries semantics around it.
- *alias of* — the row loads the same binding as the one named (a version or a renamed copy).
- A note quotes the binding's own module doc where it calls itself a stub or a partial binding.

**Anything not listed.** A `<name>.sty.rhai` / `<name>.cls.rhai` runtime binding beside the document (or under
`--path`) is read first and overrides any row here; one on the TeX tree fills a name no row has
([`parity/RHAI_BINDINGS_GUIDE.md`](parity/RHAI_BINDINGS_GUIDE.md)). Otherwise a package with no binding is read raw
from TeX Live only when raw styles are allowed (`INCLUDE_STYLES`: `--includestyles`, or `latexml.sty`'s `rawstyles`,
`localrawstyles` for the document's own directory; `--preload=ar5iv.sty` turns on `rawstyles` and `rawclasses`); when
they are not, it is reported missing, its dependencies on bound packages still loaded. A class with no binding is read
raw under `INCLUDE_CLASSES` (`rawclasses`, `localrawclasses`, `--includestyles`), else it falls back to OmniBus. A
compiled binding is never displaced by raw mode.
";

/// Render the page.
fn generate(repo: &Path) -> String {
  let tables = [
    (
      "ported",
      repo.join("latexml_package/src"),
      latexml_package::BINDINGS
        .iter()
        .map(|(n, e, _)| (*n, *e))
        .collect::<Vec<_>>(),
    ),
    (
      "contributed",
      repo.join("latexml_contrib/src"),
      crate::BINDINGS.iter().map(|(n, e, _)| (*n, *e)).collect(),
    ),
  ];
  // (kind, name, support, note), in table order within each table.
  let mut rows: Vec<(String, String, String, String)> = Vec::new();
  for (support, src, live) in &tables {
    let lib_rs = std::fs::read_to_string(src.join("lib.rs")).expect("the crate's lib.rs");
    let entries = table_entries(&lib_rs);
    let parsed: Vec<(&str, &str)> = entries
      .iter()
      .map(|e| (e.name.as_str(), e.ext.as_str()))
      .collect();
    assert_eq!(
      &parsed, live,
      "the {support} BINDINGS table as read from its source differs from the compiled one"
    );
    // The engine's modules live in latexml_engine (`engine::` re-exports it).
    let engine_src = repo.join("latexml_engine/src");
    let mut first_of: HashMap<&str, String> = HashMap::new();
    for entry in &entries {
      let (base, module) = match entry.module.strip_prefix("engine::") {
        Some(rest) => (&engine_src, rest),
        None => (src, entry.module.as_str()),
      };
      let file = module_file(base, module)
        .unwrap_or_else(|| panic!("no source file for the binding module `{}`", entry.module));
      let source = std::fs::read_to_string(&file).expect("the binding's source");
      let (note, raw) = describe(&source, &entry.name);
      let label = format!("{}.{}", entry.name, entry.ext);
      let mut notes = Vec::new();
      match first_of.get(entry.module.as_str()) {
        Some(first) => notes.push(format!("*alias of* `{first}`")),
        None => {
          first_of.insert(&entry.module, label.clone());
        },
      }
      if let Some(note) = note {
        notes.push(note);
      }
      let support = if raw {
        format!("{support}, raw + overrides")
      } else {
        support.to_string()
      };
      rows.push((
        entry.ext.clone(),
        entry.name.clone(),
        support,
        notes.join("; "),
      ));
    }
  }
  let mut out = String::from(PREAMBLE);
  for (title, kinds) in [
    ("Classes", &["cls"][..]),
    ("Packages", &["sty"][..]),
    (
      "Other binding files (engine pools, encodings, language and font-map files, TeX code files)",
      &[][..],
    ),
  ] {
    let mut section: Vec<&(String, String, String, String)> = rows
      .iter()
      .filter(|(ext, ..)| {
        if kinds.is_empty() {
          ext != "cls" && ext != "sty"
        } else {
          kinds.contains(&ext.as_str())
        }
      })
      .collect();
    section.sort_by_key(|(ext, name, ..)| (name.to_lowercase(), ext.clone()));
    out.push_str(&format!(
      "\n## {title} ({})\n\n| Name | Kind | Support | Note |\n|---|---|---|---|\n",
      section.len()
    ));
    for (ext, name, support, note) in section {
      out.push_str(&format!("| `{name}` | `.{ext}` | {support} | {note} |\n"));
    }
  }
  out
}

#[test]
fn supported_packages_doc_is_up_to_date() {
  let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
    .parent()
    .expect("the workspace root");
  let generated = generate(repo);
  let path = repo.join("docs/SUPPORTED_PACKAGES.md");
  let on_disk = std::fs::read_to_string(&path).unwrap_or_default();
  if generated != on_disk {
    if std::env::var("UPDATE_SUPPORTED_PACKAGES").is_ok() {
      std::fs::write(&path, generated).expect("write docs/SUPPORTED_PACKAGES.md");
      return;
    }
    let added: Vec<&str> = generated
      .lines()
      .filter(|l| !on_disk.lines().any(|o| o == *l))
      .collect();
    let removed: Vec<&str> = on_disk
      .lines()
      .filter(|l| !generated.lines().any(|g| g == *l))
      .collect();
    panic!(
      "docs/SUPPORTED_PACKAGES.md is stale against the binding tables.\n  added: {added:?}\n  \
       removed: {removed:?}\nRegenerate with: UPDATE_SUPPORTED_PACKAGES=1 cargo test -p \
       latexml_contrib --lib supported_packages_doc_is_up_to_date"
    );
  }
}
