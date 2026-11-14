//! Generates `docs/SUPPORTED_PACKAGES.md` from the two binding tables, `latexml_package::BINDINGS`
//! and this crate's `BINDINGS` (the single sources of truth `dispatch` reads), and checks it is
//! current. Regenerate with:
//!
//! ```sh
//! UPDATE_SUPPORTED_PACKAGES=1 cargo test -p latexml_contrib --lib supported_packages_doc_is_up_to_date
//! ```
use std::{
  collections::{HashMap, HashSet},
  path::{Path, PathBuf},
};

/// One row of a `BINDINGS` table as written in its `lib.rs`: the name, the extension and the
/// loader's path (`package::amsmath_sty::load_definitions`).
struct Entry {
  name:   String,
  ext:    String,
  loader: String,
}

impl Entry {
  /// The loader's module (`package::amsmath_sty`), whose file holds the binding's doc comment.
  fn module(&self) -> &str {
    &self.loader[..self.loader.rfind("::").expect("a `module::fn` loader")]
  }
}

/// The index of the `)` closing the `(` at `open`, skipping string literals.
fn balanced_close(text: &str, open: usize) -> Option<usize> {
  let (mut depth, mut in_str, mut escaped) = (0usize, false, false);
  for (at, c) in text[open..].char_indices() {
    match (in_str, c) {
      (true, _) if escaped => escaped = false,
      (true, '\\') => escaped = true,
      (true, '"') | (false, '"') => in_str = !in_str,
      (false, '(') => depth += 1,
      (false, ')') => {
        depth -= 1;
        if depth == 0 {
          return Some(open + at);
        }
      },
      _ => {},
    }
  }
  None
}

/// A source with its whole-line `//` comments dropped.
fn code_of(source: &str) -> String {
  source
    .lines()
    .filter(|line| !line.trim_start().starts_with("//"))
    .collect::<Vec<_>>()
    .join("\n")
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
    let close = balanced_close(rest, open).expect("a closing `)` for each row");
    let fields: Vec<&str> = rest[open + 1..close].split(',').map(str::trim).collect();
    let unquote = |s: &str| s.trim_matches('"').to_string();
    entries.push(Entry {
      name:   unquote(fields[0]),
      ext:    unquote(fields[1]),
      loader: fields[2].to_string(),
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

/// A source's raw loads, `InputDefinitions!(…)` and `input_definitions(…)` calls with `noltxml` on:
/// (first argument, all arguments unspaced). A variable bound by a local `let` to a literal is read
/// as the literal.
fn raw_load_targets(code: &str) -> Vec<(String, String)> {
  let mut targets = Vec::new();
  for call in ["InputDefinitions!(", "input_definitions("] {
    for (at, _) in code.match_indices(call) {
      let open = at + call.len() - 1;
      let Some(close) = balanced_close(code, open) else {
        continue;
      };
      let args: String = code[open + 1..close].split_whitespace().collect();
      if !args.contains("noltxml=>true") && !args.contains("noltxml:true") {
        continue;
      }
      let first = args.split(',').next().unwrap_or("").trim_start_matches('&');
      let literal = (!first.starts_with('"'))
        .then(|| {
          [
            format!("let {first} = \""),
            format!("let {first}: &str = \""),
          ]
          .iter()
          .find_map(|binding| code.find(binding.as_str()).map(|at| at + binding.len()))
          .and_then(|at| code[at..].split('"').next().map(|lit| format!("\"{lit}\"")))
        })
        .flatten();
      targets.push((literal.unwrap_or_else(|| first.to_string()), args.clone()));
    }
  }
  targets
}

/// Whether a binding reads its own TeX file raw: a raw load of `"<name>"`, or of a variable (the
/// name a shared loader is called with) as a `"<ext>"` file, in its module or in a
/// `<module>::<fn>("<name>"…)` it calls.
fn reads_raw(code: &str, file: &Path, name: &str, ext: &str) -> bool {
  let own = |targets: Vec<(String, String)>| {
    targets
      .iter()
      .any(|(first, args)| match first.strip_prefix('"') {
        // `"tikzlibraryexternal.code"` for `tikzlibraryexternal.code.tex`
        Some(literal) => format!("{name}.{ext}").to_lowercase().starts_with(&format!(
          "{}.",
          literal.trim_end_matches('"').to_lowercase()
        )),
        None => args.contains(&format!("\"{ext}\"")),
      })
  };
  if own(raw_load_targets(code)) {
    return true;
  }
  let call = format!("(\"{name}\"");
  code.match_indices(&call).any(|(at, _)| {
    let path: String = code[..at]
      .chars()
      .rev()
      .take_while(|c| c.is_alphanumeric() || *c == '_' || *c == ':')
      .collect::<Vec<_>>()
      .into_iter()
      .rev()
      .collect();
    let Some((module, _)) = path.rsplit_once("::") else {
      return false;
    };
    let dir = file.parent().expect("a module's directory");
    module_file(dir, module).is_some_and(|other| {
      std::fs::read_to_string(other).is_ok_and(|source| own(raw_load_targets(&code_of(&source))))
    })
  })
}

/// The module doc's sentence calling the binding itself a stub or a partial binding: its first
/// sentence, or one starting "Stub"/"No-op stub" — never a sentence about its history.
fn stub_note(source: &str) -> Option<String> {
  let doc: Vec<&str> = source
    .lines()
    .take_while(|line| line.trim_start().starts_with("//!") || line.trim().is_empty())
    .map(|line| line.trim_start().trim_start_matches("//!").trim())
    .collect();
  let words = |s: &str| -> Vec<String> {
    s.split(|c: char| !c.is_alphanumeric() && c != '-' && c != '_')
      .filter(|w| !w.is_empty())
      .map(str::to_lowercase)
      .collect()
  };
  let sentences: Vec<String> = doc
    .split(|line| line.is_empty())
    .flat_map(|paragraph| {
      let paragraph = paragraph.join(" ");
      paragraph
        .split_inclusive(". ")
        .map(str::to_string)
        .collect::<Vec<_>>()
    })
    .collect();
  sentences
    .iter()
    .enumerate()
    .find(|(at, sentence)| {
      let w = words(sentence);
      let starts_stub = w.first().is_some_and(|f| f == "stub")
        || (w.first().is_some_and(|f| f == "no-op") && w.get(1).is_some_and(|s| s == "stub"));
      let stub = w.iter().any(|x| x == "stub" || x == "stubs")
        || w.windows(2).any(|p| p[0] == "partial" && p[1] == "binding");
      let history = w.iter().any(|x| {
        [
          "former", "formerly", "earlier", "replaced", "replaces", "retired",
        ]
        .contains(&x.as_str())
          || x.starts_with("supersede")
      });
      (starts_stub || (*at == 0 && stub)) && !history
    })
    .map(|(_, sentence)| sentence.clone())
    .map(|sentence| {
      // A long sentence is cut at a word before 200 characters; the source keeps it whole.
      let sentence = sentence.trim();
      let cut = match sentence.char_indices().nth(200) {
        Some((at, _)) => format!("{} …", &sentence[..sentence[..at].rfind(' ').unwrap_or(at)]),
        None => sentence.to_string(),
      };
      cut.replace('|', "\\|")
    })
}

/// Perl LaTeXML's binding names (`<name>.<ext>` of every `.ltxml` under `lib/LaTeXML/{Package,Engine}`),
/// from the committed snapshot `docs/parity/perl_bindings.txt` — `LaTeXML/` is gitignored, so CI has
/// no Perl tree. Where the Perl tree is present (`<repo>/LaTeXML`, or `LATEXML_PERL_LIB` naming its
/// `lib/LaTeXML`), the snapshot is checked against it and `UPDATE_SUPPORTED_PACKAGES=1` rewrites it.
fn perl_bindings(repo: &Path) -> HashSet<String> {
  let snapshot_path = repo.join("docs/parity/perl_bindings.txt");
  let snapshot = std::fs::read_to_string(&snapshot_path).unwrap_or_default();
  let perl_lib = std::env::var("LATEXML_PERL_LIB")
    .map(PathBuf::from)
    .unwrap_or_else(|_| repo.join("LaTeXML/lib/LaTeXML"));
  if perl_lib.join("Package").is_dir() {
    let mut names: Vec<String> = ["Package", "Engine"]
      .iter()
      .flat_map(|dir| std::fs::read_dir(perl_lib.join(dir)).expect("a Perl binding directory"))
      .filter_map(|entry| {
        let file = entry.ok()?.file_name().into_string().ok()?;
        file.strip_suffix(".ltxml").map(str::to_string)
      })
      .collect();
    names.sort();
    let current = format!(
      "# Perl LaTeXML's bindings: every lib/LaTeXML/{{Package,Engine}}/*.ltxml, without `.ltxml`. Generated; refresh \
       with each upstream sync:\n# LATEXML_PERL_LIB=<LaTeXML>/lib/LaTeXML UPDATE_SUPPORTED_PACKAGES=1 cargo test -p \
       latexml_contrib --lib supported_packages_doc_is_up_to_date\n{}\n",
      names.join("\n")
    );
    if current != snapshot {
      assert!(
        std::env::var("UPDATE_SUPPORTED_PACKAGES").is_ok(),
        "docs/parity/perl_bindings.txt is stale against {}; regenerate with UPDATE_SUPPORTED_PACKAGES=1",
        perl_lib.display()
      );
      std::fs::write(&snapshot_path, &current).expect("write docs/parity/perl_bindings.txt");
    }
    return names.into_iter().collect();
  }
  snapshot
    .lines()
    .filter(|line| !line.starts_with('#') && !line.is_empty())
    .map(str::to_string)
    .collect()
}

/// Whether Perl LaTeXML loads a binding for `name.ext`: its own `.ltxml` (matched case-insensitively,
/// as `pathname_find` falls back to, Pathname.pm:381-391) or the generic one its `FindFile_fallback` strips a
/// version to (Package.pm:2191-2226: `aastex631` → `aastex`).
fn perl_has(perl: &HashSet<String>, name: &str, ext: &str) -> bool {
  let has = |n: &str| {
    let want = format!("{n}.{ext}").to_lowercase();
    perl.iter().any(|p| p.to_lowercase() == want)
  };
  has(name)
    || (matches!(ext, "sty" | "cls")
      && latexml_core::binding::content::version_stripped_name(name).is_some_and(|base| has(&base)))
}

const PREAMBLE: &str = "# Supported packages and classes

**Generated** from the binding tables — `latexml_package::BINDINGS` (`latexml_package/src/lib.rs`) and
`latexml_contrib::BINDINGS` (`latexml_contrib/src/lib.rs`), the lists `dispatch` reads — and checked on every test
run (`supported_packages_doc_is_up_to_date`); regenerate with
`UPDATE_SUPPORTED_PACKAGES=1 cargo test -p latexml_contrib --lib supported_packages_doc_is_up_to_date`.

- **ported** — a compiled binding in `latexml_package` for a package Perl LaTeXML binds too: Perl ships a `.ltxml`
  for the name, or for the generic name its versioned-name fallback strips it to (`docs/parity/perl_bindings.txt`,
  the snapshot of Perl's binding names, refreshed with each upstream sync).
- **beyond Perl** — a compiled binding in `latexml_package` for a name Perl LaTeXML has no binding for.
- **contributed** — a compiled binding in `latexml_contrib`, the contributed bindings beyond Perl LaTeXML's tree.
- **raw + overrides** — the binding reads the package's own TeX file raw (`InputDefinitions!(\"<name>\", noltxml …)`
  or `input_definitions(…, noltxml: true …)`) and defines only what carries semantics around it.
- *alias of* — the row runs the same loader as the one named (a version or a renamed copy).
- A note quotes the binding's own module doc where it calls itself a stub or a partial binding.

**Name matching.** A request matches a row by exact name first, then case-insensitively (`ieeetran` finds `IEEEtran.cls`). A
versioned name with no row of its own falls back to the generic row, as Perl's `FindFile_fallback` does
(`Package.pm:2191-2226`): version suffixes (`aastex701` → `aastex`, `neurips_2026` → `neurips`, `-arxiv`, `_final`,
`_v2`…) and the prefixes `rw`, `my`, `preprint` (`mysvjour3` → `svjour3`) are stripped.

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
  let perl = perl_bindings(repo);
  let tables = [
    (
      "latexml_package",
      repo.join("latexml_package/src"),
      latexml_package::BINDINGS
        .iter()
        .map(|(n, e, _)| (*n, *e))
        .collect::<Vec<_>>(),
    ),
    (
      "latexml_contrib",
      repo.join("latexml_contrib/src"),
      crate::BINDINGS.iter().map(|(n, e, _)| (*n, *e)).collect(),
    ),
  ];
  // (kind, name, support, note), in table order within each table.
  let mut rows: Vec<(String, String, String, String)> = Vec::new();
  // `dispatch` answers a name+extension with its first row (the two tables are disjoint).
  let mut seen: HashSet<(String, String)> = HashSet::new();
  let mut first_of: HashMap<String, (String, bool)> = HashMap::new();
  for (crate_name, src, live) in &tables {
    let lib_rs = std::fs::read_to_string(src.join("lib.rs")).expect("the crate's lib.rs");
    let entries = table_entries(&lib_rs);
    let parsed: Vec<(&str, &str)> = entries
      .iter()
      .map(|e| (e.name.as_str(), e.ext.as_str()))
      .collect();
    assert_eq!(
      &parsed, live,
      "the {crate_name} BINDINGS table as read from its source differs from the compiled one"
    );
    // The engine's modules live in latexml_engine (`engine::` re-exports it).
    let engine_src = repo.join("latexml_engine/src");
    for entry in &entries {
      if !seen.insert((entry.name.clone(), entry.ext.clone())) {
        continue;
      }
      let (base, module) = match entry.module().strip_prefix("engine::") {
        Some(rest) => (&engine_src, rest),
        None => (src, entry.module()),
      };
      let file = module_file(base, module)
        .unwrap_or_else(|| panic!("no source file for the binding module `{}`", entry.module()));
      let source = std::fs::read_to_string(&file).expect("the binding's source");
      let label = format!("{}.{}", entry.name, entry.ext);
      let mut notes = Vec::new();
      let loader = format!("{crate_name}::{}", entry.loader);
      // An alias runs its first row's loader, so it reads what that row reads.
      let raw = match first_of.get(&loader) {
        Some((first, raw)) => {
          notes.push(format!("*alias of* `{first}`"));
          *raw
        },
        None => {
          let raw = reads_raw(&code_of(&source), &file, &entry.name, &entry.ext);
          first_of.insert(loader, (label.clone(), raw));
          raw
        },
      };
      notes.extend(stub_note(&source));
      let mut support = match *crate_name {
        "latexml_contrib" => "contributed",
        _ if perl_has(&perl, &entry.name, &entry.ext) => "ported",
        _ => "beyond Perl",
      }
      .to_string();
      if raw {
        support.push_str(", raw + overrides");
      }
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
