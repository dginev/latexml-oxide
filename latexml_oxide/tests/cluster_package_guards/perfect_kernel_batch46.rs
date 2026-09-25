//! Red/green guards for perfect-kernel batch 46 (PLANS P27/P28/P29/P23/P24).
//! Each test is the minimal reproduction distilled during triage; the
//! doc-comment names the ORIGINAL corpus witness (TeX Live doc corpus,
//! `bundle/doc`) whose larger conversion was vetted separately.
use std::{path::Path, process::Command, rc::Rc};

use latexml::converter::Converter;
use latexml_core::common::{Config, OutputFormat};

/// Convert an inline snippet in a tempdir; `raw` selects the perfect-kernel
/// preload, otherwise the default (arXiv) configuration. Returns
/// (ANSI-stripped stderr, XML string).
pub(crate) fn convert(tex: &str, raw: bool) -> (String, String) {
  convert_with(
    tex,
    if raw {
      Some("[rawstyles,rawclasses]latexml.sty")
    } else {
      None
    },
  )
}

/// `convert` with the raw preload plus extra CLI arguments (`--streaming`,
/// `--max-memory=N`, …).
pub(crate) fn convert_args(tex: &str, extra: &[&str]) -> (String, String) {
  let bin = env!("CARGO_BIN_EXE_latexml_oxide");
  assert!(Path::new(bin).is_file(), "binary not staged at {bin}");
  let workdir = tempfile::tempdir().expect("create tempdir");
  std::fs::write(workdir.path().join("t.tex"), tex).expect("write t.tex");
  let mut args = vec![
    "t.tex",
    "--dest",
    "t.xml",
    "--nocomments",
    "--timeout=110",
    "--preload=[rawstyles,rawclasses]latexml.sty",
  ];
  args.extend_from_slice(extra);
  let output = Command::new(bin)
    .args(&args)
    .current_dir(workdir.path())
    .output()
    .expect("spawn latexml_oxide");
  let stderr = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
  let xml = std::fs::read_to_string(workdir.path().join("t.xml")).unwrap_or_default();
  (stderr, xml)
}

pub(crate) fn convert_files_with(
  tex: &str,
  files: &[(&str, &str)],
  preload: Option<&str>,
) -> (String, String) {
  let workdir = tempfile::tempdir().expect("create tempdir");
  for (name, content) in files {
    let path = workdir.path().join(name);
    if let Some(parent) = path.parent() {
      let _ = std::fs::create_dir_all(parent);
    }
    std::fs::write(path, content).expect("write file");
  }
  let tex = tex.to_string();
  let search_path = workdir.path().to_string_lossy().into_owned();
  let preload = preload.map(|p| vec![p.to_string()]);
  std::thread::Builder::new()
    .stack_size(256 * 1024 * 1024)
    .spawn(move || {
      let _ = latexml_core::util::logger::init(log::LevelFilter::Info);
      let opts = Config {
        format: OutputFormat::XML,
        include_comments: Some(false),
        preload,
        search_paths: Some(vec![search_path]),
        bindings_dispatch: Some(Rc::new(latexml_package::dispatch)),
        extra_bindings_dispatch: Some(Rc::new(latexml_contrib::dispatch)),
        ..Config::default()
      };
      let mut converter = Converter::from_config(opts.clone());
      if let Err(e) = converter.prepare_session(&opts) {
        return (format!("Error:prepare_session:{e}"), String::new());
      }
      let resp = converter.convert_content_with_provenance("t.tex", tex);
      latexml_core::reset_thread_engine();
      (resp.log, resp.result.unwrap_or_default())
    })
    .expect("spawn test worker")
    .join()
    .expect("test worker panicked")
}

/// Like `convert_args` with the raw preload, after writing `files`
/// (`(name, content)`) into the work directory — for repros that need a
/// package, class or data file beside the document.
pub(crate) fn convert_files(tex: &str, files: &[(&str, &str)]) -> (String, String) {
  convert_files_with(tex, files, Some("[rawstyles,rawclasses]latexml.sty"))
}

pub(crate) fn convert_with_budget(
  tex: &str,
  preload: Option<&str>,
  _secs: u32,
) -> (String, String) {
  convert_with(tex, preload)
}

pub(crate) fn convert_with(tex: &str, preload: Option<&str>) -> (String, String) {
  let tex = tex.to_string();
  let preload = preload.map(String::from);
  std::thread::Builder::new()
    .stack_size(256 * 1024 * 1024)
    .spawn(move || {
      let _ = latexml_core::util::logger::init(log::LevelFilter::Info);
      let mut preloads = vec![];
      if let Some(p) = preload {
        preloads.push(p);
      }
      let opts = Config {
        format: OutputFormat::XML,
        include_comments: Some(false),
        preload: if preloads.is_empty() {
          None
        } else {
          Some(preloads)
        },
        bindings_dispatch: Some(Rc::new(latexml_package::dispatch)),
        extra_bindings_dispatch: Some(Rc::new(latexml_contrib::dispatch)),
        ..Config::default()
      };
      let mut converter = Converter::from_config(opts.clone());
      if let Err(e) = converter.prepare_session(&opts) {
        return (format!("Error:prepare_session:{e}"), String::new());
      }
      let resp = converter.convert_content_with_provenance("t.tex", tex);
      latexml_core::reset_thread_engine();
      (resp.log, resp.result.unwrap_or_default())
    })
    .expect("spawn test worker")
    .join()
    .expect("test worker panicked")
}

/// Count of lines carrying a `Warning:<class>:` diagnostic ANYWHERE in the line
/// (WISDOM 85: a diagnostic can follow other output on the same line).
pub(crate) fn warning_count(stderr: &str) -> usize {
  let re = regex::Regex::new(r"Warning:[A-Za-z_]+:").unwrap();
  stderr.lines().filter(|l| re.is_match(l)).count()
}

pub(crate) fn error_count(stderr: &str) -> usize {
  // Any `Error:`/`Fatal:` diagnostic, anywhere in the line (WISDOM 85), whatever its
  // category spells: `Error:I/O:` and `Error:<char>:` count too (a `[A-Za-z_]` category
  // missed them).
  let re = regex::Regex::new(r"(Error|Fatal):[^:\s]+:").unwrap();
  stderr.lines().filter(|l| re.is_match(l)).count()
}

const MEMOIR: &str = r"\documentclass{memoir}
\begin{document}
\chapter{C}
\onelineskip
\section{S}
Body.
\end{document}
";

/// P27: memoir.cls is raw-interpreted through the engine (the binding is a
/// raw-load shim, tlp/czjphys precedent). RED: the former stub hid the real
/// class — `\onelineskip` and every memoir-only macro undefined. Witnesses:
/// titlepages/titlepages (4→0), dlfltxb/dlfltxbmarkup (3→0), memexsupp.
/// Both preload modes must agree, since the binding is what makes the
/// class raw-load under the default arXiv configuration too.
#[test]
fn memoir_raw_loads_in_both_modes() {
  for raw in [true, false] {
    let (stderr, xml) = convert(MEMOIR, raw);
    assert_eq!(error_count(&stderr), 0, "raw={raw}:\n{stderr}");
    assert!(
      xml.contains("<chapter") && xml.contains("<section"),
      "raw={raw}:\n{xml}"
    );
  }
}

/// P28: nicematrix.sty / tabularray.sty ARE implemented — the stale
/// `missing_file` "not implemented and will not be interpreted raw"
/// warnings misreported every document using them.
#[test]
fn nicematrix_tabularray_no_stale_missing_file_warning() {
  let (stderr, _xml) = convert(
    r"\documentclass{article}
\usepackage{nicematrix,tabularray}
\begin{document}
\begin{NiceTabular}{cc} a & b \\ \end{NiceTabular}
\begin{tblr}{cc} a & b \\ \end{tblr}
\end{document}
",
    true,
  );
  assert!(
    !stderr.contains("missing_file:nicematrix") && !stderr.contains("missing_file:tabularray"),
    "stale missing_file warning is back:\n{stderr}"
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
}

/// P29: `\index` expands its entry before splitting on `@`/`!`/`|`, as
/// real `\@wrindex` writes it via `\protected@write` (latex.ltx:17720),
/// and a sort key holding sanitized specials is a plain makeindex string.
/// RED: macro-built entries never met their `@` (tcolorbox documentation
/// library `\kvtcb@doc@sortindex\idx@actual…`), so the sort key was
/// digested as text and every `_` in it errored; a literal `a_b@…` key
/// errored too and rendered `˙`. Witness: tagpdf/tagpdf (113→21).
#[test]
fn index_entry_expands_and_keys_sanitized_specials() {
  let (stderr, xml) = convert(
    r"\documentclass{article}
\makeindex
\begin{document}
\def\key{x_y}\def\show{\texttt{x\_y}}
A\index{a_b@\texttt{a\_b}}
B\index{\key @\show}
D\index{plain}\index{p|see{plain}}
\end{document}
",
    true,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  for key in ["key=\"a_b\"", "key=\"x_y\"", "key=\"plain\""] {
    assert!(xml.contains(key), "missing {key}:\n{xml}");
  }
  assert!(!xml.contains('˙'), "sort key rendered through OT1:\n{xml}");
}

/// P29 witness shape: tcolorbox `docCommand{tag_if_active:TF}` writes
/// `\index{\kvtcb@doc@sortindex\idx@actual\tcbIndexPrintComC{…}}`
/// (tcbdocumentation.code.tex:495). RED: 4 `Script _` errors per entry.
#[test]
fn tcolorbox_doccommand_index_key_expands() {
  let (stderr, xml) = convert(
    r"\documentclass{article}
\usepackage[documentation]{tcolorbox}
\begin{document}
\begin{docCommand}{tag_if_active:TF}{}\end{docCommand}
Text.
\end{document}
",
    true,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("key=\"tag_if_active:TF\""), "{xml}");
}

/// P23: `NiceTabularX{width}[opts]{colspec}[opts]` (nicematrix.sty:3788)
/// is a tabularx — its `X` columns need the tabularx column engine. RED:
/// the reduction to `\tabular` dropped every X cell ("Unrecognized tabular
/// template X" + "Extra alignment tab"). Witness: nicematrix/nicematrix
/// `\begin{NiceTabularX}{\linewidth}{l||*{\LastDay}{X}}`.
#[test]
fn nicetabularx_is_a_tabularx() {
  let (stderr, xml) = convert(
    r"\documentclass{article}
\usepackage{nicematrix}
\newcommand\LastDay{3}
\begin{document}
\begin{NiceTabularX}{\linewidth}{l||*{\LastDay}{X}}[hvlines]
a & b & c & d \\
\end{NiceTabularX}
\begin{NiceTabular*}{\linewidth}[hvlines]{cc}
e & f \\
\end{NiceTabular*}
\end{document}
",
    true,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    !stderr.contains("Unrecognized tabular template"),
    "{stderr}"
  );
  assert_eq!(xml.matches("<td").count(), 6, "{xml}");
}

/// P24: tabularray's template API — `\DeclareTblrTemplate` (:5673; the
/// bound `\DefTblrTemplate` is only its alias), `\UseTblrTemplate`,
/// `\MapTblrRemarks`, `\InsertTblrRemarkTag`. Witness: tabularray-abnt.
#[test]
fn tabularray_template_api_defined() {
  let (stderr, _xml) = convert(
    r"\documentclass{article}
\usepackage{tabularray}
\DeclareTblrTemplate{remark-tag}{x}{\InsertTblrRemarkTag}
\SetTblrTemplate{remark-tag}{x}
\begin{document}
\UseTblrTemplate{remark-tag}{x}\MapTblrRemarks{\InsertTblrRemarkTag}
\begin{tblr}{cc} a & b \\ \end{tblr}
\end{document}
",
    true,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
}
