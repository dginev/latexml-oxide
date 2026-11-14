//! Guards for ST3: a document converted after others on one thread, as a persistent `cortex_worker` converts its papers
//! (a `Converter` per paper, no engine reset between them), is the document converted alone.

use latexml::util::test::{GUARD_JOBNAME, convert_on_one_thread, error_count, warning_count};

/// A file of `tools/perfect_kernel/repros/`, by its path there.
macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!("../../../tools/perfect_kernel/repros/", $path))
  };
}

/// A document: its source, and the files beside it.
struct Doc {
  tex:   String,
  files: Vec<(&'static str, String)>,
}

fn doc(tex: &str) -> Doc {
  Doc {
    tex:   tex.to_string(),
    files: Vec::new(),
  }
}

/// `docs` written in `dir`, the last as `<GUARD_JOBNAME>.tex`, each earlier one as `d<i>.tex`, with their files: the
/// sources' paths.
fn write_in(dir: &std::path::Path, docs: &[&Doc]) -> Vec<String> {
  let last = docs.len() - 1;
  docs
    .iter()
    .enumerate()
    .map(|(i, doc)| {
      for (name, content) in &doc.files {
        std::fs::write(dir.join(name), content).expect("a file beside the document");
      }
      let name = if i == last {
        format!("{GUARD_JOBNAME}.tex")
      } else {
        format!("d{i}.tex")
      };
      let path = dir.join(name);
      std::fs::write(&path, &doc.tex).expect("the document");
      path.to_string_lossy().into_owned()
    })
    .collect()
}

/// The log's Error, Warning and Fatal lines and its unknown-key Infos, without their locators (which name the
/// document's directory).
fn diagnostics(log: &str) -> Vec<&str> {
  log
    .lines()
    .filter(|line| {
      ["Error:", "Warning:", "Fatal:", "Info:undefined:"]
        .iter()
        .any(|kind| line.starts_with(kind))
    })
    .collect()
}

/// A row: the documents converted before and what each of their logs counts, the document, whether they share its
/// directory, its (Error, Warning) counts and a line its log carries that many times.
struct Row {
  name:          &'static str,
  before:        Vec<Doc>,
  /// Each earlier document's (Error, Warning, times the row's line) counts, its own conversion's.
  before_counts: (usize, usize, Option<usize>),
  document:      Doc,
  one_dir:       bool,
  counts:        (usize, usize),
  line:          Option<(&'static str, usize)>,
}

/// After the documents before it on one thread, each with its own counts, the document has its counts, its diagnostics
/// and its XML byte for byte as alone.
#[test]
fn a_document_after_another_on_one_thread_is_the_document_alone() {
  let keyval = repro!("macro-state/unknown_keyval_key_warns_in_each_document.tex");
  let subjclass = repro!("macro-state/unknown_frontmatter_key_noted_in_each_document.tex");
  let listing = repro!("macro-state/listing_body_file_named_in_each_document.tex");
  let chain = |extra: &str| Doc {
    tex:   repro!(
      "stand-in-copies/text_defaults/chain/copy_newcommand_through_a_chained_base_class_errs.tex"
    )
    .to_string(),
    files: vec![
      (
        "cimart.cls",
        repro!("stand-in-copies/text_defaults/chain/cimart.cls").replace(
          "\\newcommand\\refname{Works Cited}\n\\newcommand\\figurename{Fig}\n",
          "\\newcommand\\bibname{Works}\n",
        ),
      ),
      (
        "mybase.cls",
        format!(
          "{}{extra}",
          repro!("stand-in-copies/text_defaults/chain/mybase.cls")
        ),
      ),
    ],
  };
  let loadclass = || Doc {
    tex:   repro!(
      "stand-in-copies/text_defaults/loadclass/copy_newcommand_of_a_base_class_name_errs.tex"
    )
    .to_string(),
    files: vec![(
      "cimart.cls",
      repro!("stand-in-copies/text_defaults/loadclass/cimart.cls").to_string(),
    )],
  };
  let rows = [
    Row {
      name:          "unknown KeyVals key, Warning",
      before:        vec![doc(keyval)],
      before_counts: (0, 1, Some(1)),
      document:      doc(keyval),
      one_dir:       false,
      counts:        (0, 1),
      line:          Some(("unknown KeyVals key 'detect-inline-weight'", 1)),
    },
    Row {
      name:          "unknown 'Frontmatter' key, Info",
      before:        vec![doc(subjclass)],
      before_counts: (0, 0, Some(1)),
      document:      doc(subjclass),
      one_dir:       false,
      counts:        (0, 0),
      line:          Some(("unknown KeyVals key 'scheme'", 1)),
    },
    Row {
      name:          "90a elsarticle row after its predecessor",
      before:        vec![doc(repro!(
        "sectioning-frontmatter/revtex_affiliation_requested_in_formula_marks_part_one_line.tex"
      ))],
      before_counts: (0, 0, None),
      document:      doc(repro!(
        "sectioning-frontmatter/elsarticle_address_requested_in_formula_marks_part_one_line.tex"
      )),
      one_dir:       false,
      counts:        (0, 0),
      line:          None,
    },
    Row {
      name:          "class load record (a base class of the same name)",
      before:        vec![chain("\\def\\bibname{Literature}\n")],
      before_counts: (1, 0, Some(1)),
      document:      chain(""),
      one_dir:       false,
      counts:        (0, 0),
      line:          Some(("Command \\bibname already defined", 0)),
    },
    Row {
      name:          "raw read record (one directory)",
      before:        vec![loadclass()],
      before_counts: (2, 0, Some(1)),
      document:      loadclass(),
      one_dir:       true,
      counts:        (2, 0),
      line:          Some(("Command \\refname already defined", 1)),
    },
    Row {
      name:          "a listing body's file, numbered per conversion",
      before:        vec![doc(listing)],
      before_counts: (0, 0, Some(1)),
      document:      doc(listing),
      one_dir:       false,
      counts:        (0, 0),
      line:          Some(("lx-LTXexample-1.tmp", 1)),
    },
  ];
  let mut failures = Vec::new();
  for row in rows {
    let Row {
      name,
      before,
      before_counts,
      document,
      one_dir,
      counts,
      line,
    } = row;
    let alone_dir = tempfile::tempdir().expect("a document directory");
    let alone =
      convert_on_one_thread(write_in(alone_dir.path(), &[&document]), Some("ar5iv.sty")).remove(0);
    let docs: Vec<&Doc> = before.iter().chain([&document]).collect();
    let dirs: Vec<_> = docs
      .iter()
      .map(|_| tempfile::tempdir().expect("a document directory"))
      .collect();
    let sources = if one_dir {
      write_in(dirs[0].path(), &docs)
    } else {
      docs
        .iter()
        .zip(&dirs)
        .flat_map(|(doc, dir)| write_in(dir.path(), &[doc]))
        .collect()
    };
    let mut runs = convert_on_one_thread(sources, Some("ar5iv.sty"));
    let after = runs.pop().expect("the document");
    let found = |log: &str| {
      let times = line.map(|(line, _)| log.matches(line).count());
      (error_count(log), warning_count(log), times)
    };
    for (i, (log, _)) in runs.iter().enumerate() {
      if found(log) != before_counts {
        failures.push(format!(
          "{name}: document {i} before, counts {:?}, want {before_counts:?}\n{log}",
          found(log)
        ));
      }
    }
    let want = (counts.0, counts.1, line.map(|(_, times)| times));
    if found(&alone.0) != want || found(&after.0) != want {
      failures.push(format!(
        "{name}: counts alone {:?}, after {:?}, want {want:?}\n{}",
        found(&alone.0),
        found(&after.0),
        after.0
      ));
    }
    if diagnostics(&alone.0) != diagnostics(&after.0) {
      failures.push(format!(
        "{name}: diagnostics alone {:#?}\nafter {:#?}",
        diagnostics(&alone.0),
        diagnostics(&after.0)
      ));
    }
    if alone.1.is_empty() || alone.1 != after.1 {
      failures.push(format!(
        "{name}: XML alone\n{}\nafter\n{}",
        alone.1, after.1
      ));
    }
  }
  assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}
