//! The format-dump gate (CLAUDE.md parity rule 4; release-dumps.yml runs each
//! init under `LATEXML_INIT_DEBUG=1` and fails on any `Error:`/`Fatal:`):
//! `--init=plain.tex` and `--init=latex.ltx` complete with ZERO errors. Six
//! `\errmessage`s always fired in the latex init and surfaced once
//! `\errmessage` became an Error (batch 56g, KPE #195):
//! - latex.ltx:98-101 (ltdirchk) "LaTeX must be made using an initex with no
//!   format preloaded" — `{` was at catcode 1, where initex has 12 (tex.web §232);
//!   `ini_tex.rs` now reads the format with both braces at 12;
//! - fontmath.ltx:423 "Command `\sqrtsign' already defined" — `\radical` was
//!   undefined (Perl TeX_Math.pool.ltxml:28), so `\DeclareMathRadical`'s
//!   `\let\sqrtsign\radical` + `\meaning` test failed; `tex_math.rs` defines it,
//!   building the radical through the private `\lx@radical@sqrt`;
//! - latex.ltx:19582-19585 "Control sequence \CurrentFile… already defined" ×4 —
//!   Base defined the `\CurrentFile` family (RUST-ONLY); latex.ltx owns it now.
//!
//! The latex init takes ~23 s in the `test` profile (~28 s in `ci`, ~160 s at
//! opt-level 0 throughout). Repros:
//! `tools/perfect_kernel/repros/singletons/radical_{is_a_primitive_radical,let_sqrt}.tex`,
//! `tools/perfect_kernel/repros/loader/current_file_*.tex`,
//! `tools/perfect_kernel/repros/loader/nodump_latex_branch.tex`.
use std::{path::Path, process::Command};

use latexml::util::test::assert_element;

use super::perfect_kernel_batch46::{convert, error_count, warning_count};

/// release-dumps.yml's gate: `grep -acE '^(Error|Fatal):'` over the init log.
fn init_error_count(log: &str) -> usize {
  log
    .lines()
    .filter(|l| l.starts_with("Error:") || l.starts_with("Fatal:"))
    .count()
}

/// A catcode record for `{` or `}` in a dump (`C<TAB>{<TAB>CC<TAB>n`): the
/// format is read with both at 12, so any record would mean they leaked.
fn brace_catcode_records(dump: &str) -> Vec<&str> {
  dump
    .lines()
    .filter(|l| l.starts_with("C\t{\t") || l.starts_with("C\t}\t"))
    .collect()
}

/// Run `latexml_oxide --init=<init>` in a tempdir, as release-dumps.yml does
/// (`LATEXML_INIT_DEBUG=1` keeps every diagnostic visible), writing the dump to
/// `--dest`. Returns (ANSI-stripped stderr+stdout, dump text).
fn run_init(init: &str) -> (String, String) {
  let bin = env!("CARGO_BIN_EXE_latexml_oxide");
  assert!(Path::new(bin).is_file(), "binary not staged at {bin}");
  let workdir = tempfile::tempdir().expect("create tempdir");
  let dest = workdir.path().join("format.dump.txt");
  let output = Command::new(bin)
    .arg(format!("--init={init}"))
    .arg(format!("--dest={}", dest.display()))
    .env("LATEXML_INIT_DEBUG", "1")
    .current_dir(workdir.path())
    .output()
    .expect("spawn latexml_oxide --init");
  assert!(
    output.status.success(),
    "--init={init} exited {:?}",
    output.status
  );
  let log = format!(
    "{}{}",
    String::from_utf8_lossy(&output.stderr),
    String::from_utf8_lossy(&output.stdout)
  )
  .replace('\u{1b}', "");
  let dump = std::fs::read_to_string(&dest).unwrap_or_default();
  (log, dump)
}

/// Convert `tex` with the CLI binary in a tempdir, with no preload (so a document without `\documentclass` stays plain
/// TeX), the given `--timeout` and extra child-process environment: a process of its own for the `LATEXML_NODUMP`
/// caller, as the dump-or-raw `LoadFormat` branch is chosen once per process (a process-once `Lazy`, plain_dump.rs,
/// latex.rs). Returns (exit success, ANSI-stripped stderr, XML).
fn convert_cli(tex: &str, timeout: u32, env: &[(&str, &str)]) -> (bool, String, String) {
  let bin = env!("CARGO_BIN_EXE_latexml_oxide");
  let workdir = tempfile::tempdir().expect("create tempdir");
  std::fs::write(workdir.path().join("t.tex"), tex).expect("write t.tex");
  let output = Command::new(bin)
    .args(["t.tex", "--dest", "t.xml", "--nocomments"])
    .arg(format!("--timeout={timeout}"))
    .envs(env.iter().copied())
    .current_dir(workdir.path())
    .output()
    .expect("spawn latexml_oxide");
  let stderr = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
  let xml = std::fs::read_to_string(workdir.path().join("t.xml")).unwrap_or_default();
  (output.status.success(), stderr, xml)
}

fn has_record(dump: &str, record: &str) -> bool { dump.lines().any(|line| line == record) }

/// The latex init: zero errors, and the dump carries latex.ltx's own state —
/// `\sqrtsign` as `\DeclareMathRadical` builds it (latex.ltx:13683, what
/// pdflatex's `\meaning` shows), expl3's `\tex_radical:D` alias, and the four
/// `\CurrentFile` token lists empty (Perl blib latex_dump.pool.ltxml:2716-2719)
/// — and no brace catcode, which latex.ltx:102-103 restores.
#[test]
fn latex_ltx_init_has_zero_errors() {
  let (log, dump) = run_init("latex.ltx");
  assert_eq!(init_error_count(&log), 0, "{log}");
  assert!(!dump.is_empty(), "no dump written\n{log}");
  assert_eq!(brace_catcode_records(&dump), Vec::<&str>::new());
  for record in [
    "M\t\\sqrtsign\tE\t\\sqrtsign\t0\t\t16:\\radical,12:\",12:2,12:7,12:0,12:3,12:7,12:0,16:\\relax\t\t",
    "M\t\\tex_radical:D\tPA\t\\radical",
    "M\t\\CurrentFile\tE\t\\CurrentFile\t0\t\t\t\t",
    "M\t\\CurrentFilePath\tE\t\\CurrentFilePath\t0\t\t\t\t",
    "M\t\\CurrentFileUsed\tE\t\\CurrentFileUsed\t0\t\t\t\t",
    "M\t\\CurrentFilePathUsed\tE\t\\CurrentFilePathUsed\t0\t\t\t\t",
  ] {
    assert!(has_record(&dump, record), "dump lacks record {record:?}");
  }
}

/// The plain init: zero errors, no brace catcode (plain.tex:11-12 restores
/// them), and plain.tex's `\def\sqrt{\radical"270370 }` dumped as written.
#[test]
fn plain_tex_init_has_zero_errors() {
  let (log, dump) = run_init("plain.tex");
  assert_eq!(init_error_count(&log), 0, "{log}");
  assert_eq!(brace_catcode_records(&dump), Vec::<&str>::new());
  assert!(
    has_record(
      &dump,
      "M\t\\sqrt\tE\t\\sqrt\t0\t\t16:\\radical,12:\",12:2,12:7,12:0,12:3,12:7,12:0,10: \t\t"
    ),
    "dump lacks plain.tex's \\sqrt"
  );
}

/// `\radical` is the primitive (pdflatex `\meaning` = `\radical`), and a radical
/// takes its math field — braced, or a single token after `\relax` (tex.web
/// §1151 `scan_math`) — as a square root; `\sqrtsign` is latex.ltx's macro.
/// RED: `undefined:\radical`, and `\ERROR " 270370 x` in the math.
#[test]
fn radical_is_a_primitive_radical() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/singletons/radical_is_a_primitive_radical.tex"
  );
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "text",
    &["font=\"typewriter\""],
    r#"<text font="typewriter">[\radical][macro:-&gt;\radical "270370\relax ]</text>"#,
  );
  assert_element(
    &xml,
    "XMath",
    &[],
    r#"<XMath>
      <XMApp>
        <XMTok meaning="plus" role="ADDOP">+</XMTok>
        <XMApp>
          <XMTok meaning="square-root"/>
          <XMTok font="italic" role="UNKNOWN">x</XMTok>
        </XMApp>
        <XMApp>
          <XMTok meaning="square-root"/>
          <XMTok font="italic" role="UNKNOWN">y</XMTok>
        </XMApp>
        <XMApp>
          <XMTok meaning="square-root"/>
          <XMTok font="italic" role="UNKNOWN">z</XMTok>
        </XMApp>
      </XMApp>
    </XMath>"#,
  );
}

/// `\let\sqrt\sqrtsign` (mathfixs.sty:139's shape): the lock on `\sqrt` does not
/// stop a `\let`, so a `\radical` building through `\sqrt` looped `\sqrt` →
/// `\sqrtsign` → `\radical` → `\sqrt` to `Fatal:Timeout`. It builds through
/// `\lx@radical@sqrt`; a `\mathchar` field carries its number (tex.web §1151).
/// pdflatex: three square roots, 0 errors. Tip: `undefined:\radical`.
#[test]
fn radical_survives_a_let_sqrt() {
  let tex = include_str!("../../../tools/perfect_kernel/repros/singletons/radical_let_sqrt.tex");
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "XMath",
    &[],
    r#"<XMath>
      <XMApp>
        <XMTok meaning="plus" role="ADDOP">+</XMTok>
        <XMApp>
          <XMTok meaning="square-root"/>
          <XMTok font="italic" role="UNKNOWN">x</XMTok>
        </XMApp>
        <XMApp>
          <XMTok meaning="square-root"/>
          <XMTok font="italic">x</XMTok>
        </XMApp>
        <XMApp>
          <XMTok meaning="square-root"/>
          <XMTok font="italic" role="UNKNOWN">z</XMTok>
        </XMApp>
      </XMApp>
    </XMath>"#,
  );
}

/// `\CurrentFile` is a LaTeX kernel token list: undefined in plain TeX (pdftex
/// `[undefined]`, Perl alike). RED: `[macro:-¿]` — Base defined it.
#[test]
fn current_file_comes_from_latex_not_plain() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/loader/current_file_is_latex_not_plain.tex");
  let (stderr, xml) = latexml::util::test::convert_with(tex, None);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(&xml, "p", &[], "<p>[undefined]</p>");
}

/// `\CurrentFile` and `\CurrentFilePathUsed` empty, and `\CurrentFile` equal to
/// `\CurrentFileUsed`: what pdflatex typesets for the repro's `\texttt` line.
const CURRENT_FILE_EMPTY: &str = r#"<text font="typewriter">[macro:-&gt;][macro:-&gt;]
same</text>"#;

/// Under LaTeX the family is defined and empty (pdflatex `[macro:->]`, `same`):
/// from the dump, and — without one — from `latex.rs`'s fallback, made before
/// `latex_constructs` (witnesses 2204.03209, 2205.10749, 2311.06870). The NODUMP
/// half is `nodump_latex_branch_converts_healthily`, which shares one raw kernel
/// load with the other NODUMP guards.
#[test]
fn current_file_is_defined_empty_under_latex() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/loader/current_file_defined_by_latex.tex");
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(&xml, "text", &["font=\"typewriter\""], CURRENT_FILE_EMPTY);
}

/// The degraded `LoadFormat('latex')` branch (no dump: bootstrap → base →
/// constructs, CLAUDE.md parity rule 1). Every NODUMP conversion raw-loads
/// latex.ltx and expl3-code.tex (~24 s in `test`, several times that at
/// opt-level 0), so the three NODUMP guards share this ONE conversion, with a
/// timeout far over the 60 s CLI default (which is calibrated for the dump path)
/// and under nextest's 20 min terminate-after, so a genuine hang still surfaces:
/// - issue #651 (witness: a bare `\usepackage{fvextra}` reported "Conversion
///   failed: 1 fatal error"): the symptom — an expl3-using document converts,
///   exits 0 and keeps its body. Not the `expl3_sty.rs` mechanism that fixed it
///   (scoping out the raw-load-only expl3-code.tex cascade, L33074-33180):
///   `latex.rs`'s degraded branch has already raw-loaded expl3-code.tex, so
///   `\tex_let:D` is defined and `expl3.sty` skips its own re-load;
/// - issue #719 (witness: user MWE): under `\parindent=0pt` the FIRST paragraph
///   is `ltx_noindent`. The first landing keyed the stamp on a one-shot that a
///   begin-document `\par` consumed first on this branch only; the stamp is now
///   structural (first `ltx:para` of its parent). Dump half:
///   `06_cluster_regressions::cluster_first_para_noindent_719`;
/// - the `\CurrentFile` family is defined and empty from `latex.rs`'s fallback
///   (dump half: `current_file_is_defined_empty_under_latex`), and the branch
///   warns once, for its `recursion:LaTeX.pool` re-entrance.
#[test]
fn nodump_latex_branch_converts_healthily() {
  let tex = include_str!("../../../tools/perfect_kernel/repros/loader/nodump_latex_branch.tex");
  let (ok, stderr, xml) = convert_cli(tex, 900, &[("LATEXML_NODUMP", "1")]);
  assert!(ok, "the NODUMP conversion exited non-zero:\n{stderr}");
  assert!(!stderr.contains("fatal error"), "{stderr}");
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 1, "{stderr}");
  assert!(stderr.contains("Warning:recursion:LaTeX.pool"), "{stderr}");
  assert_element(
    &xml,
    "para",
    &["xml:id=\"p1\""],
    r#"<para class="ltx_noindent" xml:id="p1"><p>First line</p></para>"#,
  );
  assert_element(
    &xml,
    "para",
    &["xml:id=\"p2\""],
    r#"<para class="ltx_noindent" xml:id="p2"><p>second lines</p></para>"#,
  );
  assert_element(
    &xml,
    "para",
    &["xml:id=\"p3\""],
    &format!(r#"<para class="ltx_noindent" xml:id="p3"><p>{CURRENT_FILE_EMPTY}</p></para>"#),
  );
  assert_element(
    &xml,
    "para",
    &["xml:id=\"p4\""],
    r#"<para class="ltx_noindent" xml:id="p4"><p>degraded-body-text</p></para>"#,
  );
}

/// The dump a TeX tree older than this one would have built: its recorded sources are another
/// tree's files, and its L3 kernel is dated 2000-01-01 (`\c__kernel_expl_date_tl`, the date
/// expl3.sty:64-78 checks a format's against its own).
fn older_tree_dump(dump: &str) -> String {
  let older_date = "12:2,12:0,12:0,12:0,12:-,12:0,12:1,12:-,12:0,12:1";
  dump
    .lines()
    .map(|line| {
      if let Some(source) = line.strip_prefix("# source\t") {
        let name = source.split('\t').next().unwrap_or_default();
        format!("# source\t{name}\t1\t00000000\t/older/texlive/{name}")
      } else if line.starts_with("M\t\\c__kernel_expl_date_tl\t") {
        line
          .split('\t')
          .map(|field| {
            if field.starts_with("12:") && field.contains("12:-") {
              older_date
            } else {
              field
            }
          })
          .collect::<Vec<_>>()
          .join("\t")
      } else {
        line.to_string()
      }
    })
    .collect::<Vec<_>>()
    .join("\n")
}

/// Convert `tex` with the CLI binary in a tempdir that also holds `local_files` (a document's own
/// files), with the given child-process environment. Returns (exit success, ANSI-stripped stderr,
/// XML). A child process, not the internal convert API: the dump locations and the format cache are
/// read once per process, and building a format changes the process's working directory.
fn convert_cli_beside(
  tex: &str,
  local_files: &[(&str, &str)],
  env: &[(&str, String)],
) -> (bool, String, String) {
  let bin = env!("CARGO_BIN_EXE_latexml_oxide");
  let workdir = tempfile::tempdir().expect("create tempdir");
  std::fs::write(workdir.path().join("t.tex"), tex).expect("write t.tex");
  for (name, content) in local_files {
    std::fs::write(workdir.path().join(name), content).expect("write a local file");
  }
  let output = Command::new(bin)
    .args(["t.tex", "--dest", "t.xml", "--nocomments", "--timeout=600"])
    .envs(env.iter().map(|(key, value)| (*key, value.as_str())))
    .current_dir(workdir.path())
    .output()
    .expect("spawn latexml_oxide");
  let stderr = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
  let xml = std::fs::read_to_string(workdir.path().join("t.xml")).unwrap_or_default();
  (output.status.success(), stderr, xml)
}

/// REGRESSION 2026-10-05 (2609 release): the worker's dumps were built from `/usr/local/texlive/2025`
/// (L3 kernel 2025-11-06) while it read packages from `/usr/share/texlive` (2026-01-19), and every
/// paper loading `expl3.sty` ended as Error — "Mismatched LaTeX support files detected" and
/// "Cannot run piped system commands" (expl3.sty:64-78) — 12,144 of 38,624. A dump records the files
/// it was built from (`# source` lines, ini_tex.rs) and one this TeX tree does not have them for is
/// passed over (`dump_paths::dump_matches_tree`). Here an older tree's dumps, forged from the dumps
/// built on whatever TeX Live runs the test, are the only ones the conversion may use
/// (`LATEXML_DUMP_DIR_ONLY`, no embedded dumps): on the old engine they are loaded and expl3 refuses
/// the format; now both formats are built for this tree into the cache (`latexml::format_dumps`),
/// recording this tree's sources, and loaded from there. The document ships its own
/// `expl3-code.tex`, which neither the tree's stamps nor the build may read (kpathsea searches `.`
/// first). With the dumps that do match, the same document loads them and builds nothing, as before
/// (62s11). Witnesses 2607.28725 (mnras + xparse), 2609.40175.
#[test]
fn a_dump_built_from_another_tree_is_not_used() {
  let (log, latex_dump) = run_init("latex.ltx");
  assert_eq!(init_error_count(&log), 0, "{log}");
  assert!(
    latex_dump
      .lines()
      .any(|line| line.starts_with("# source\tlatex.ltx\t")),
    "the dump records no sources"
  );
  let (log, plain_dump) = run_init("plain.tex");
  assert_eq!(init_error_count(&log), 0, "{log}");
  let year = latexml_engine::dump_paths::detect_ambient_texlive_year().unwrap_or(2000);
  let older = older_tree_dump(&latex_dump);
  let date_line = |dump: &str| {
    dump
      .lines()
      .find(|line| line.starts_with("M\t\\c__kernel_expl_date_tl\t"))
      .map(str::to_string)
  };
  assert!(
    date_line(&latex_dump).is_some(),
    "the dump has no L3 kernel date"
  );
  assert_ne!(
    date_line(&older),
    date_line(&latex_dump),
    "the L3 kernel date was not rewritten"
  );
  let forged = tempfile::tempdir().expect("create tempdir");
  std::fs::write(forged.path().join(format!("latex.{year}.dump.txt")), older)
    .expect("write latex dump");
  std::fs::write(
    forged.path().join(format!("plain.{year}.dump.txt")),
    older_tree_dump(&plain_dump),
  )
  .expect("write plain dump");
  let cache = tempfile::tempdir().expect("create tempdir");
  let tex =
    "\\documentclass{article}\n\\usepackage{expl3}\n\\begin{document}\nText.\n\\end{document}\n";
  let own_copy = [(
    "expl3-code.tex",
    "\\errmessage{the document's own expl3-code.tex was read}\n",
  )];
  let forced = [
    ("LATEXML_DUMP_DIR", forged.path().display().to_string()),
    ("LATEXML_DUMP_DIR_ONLY", String::from("1")),
    ("LATEXML_NO_EMBEDDED_DUMP", String::from("1")),
    ("LATEXML_FORMAT_CACHE", cache.path().display().to_string()),
  ];
  let (ok, stderr, xml) = convert_cli_beside(tex, &own_copy, &forced);
  assert!(ok, "the conversion exited non-zero:\n{stderr}");
  assert!(
    !stderr.contains("Mismatched LaTeX support files"),
    "{stderr}"
  );
  assert!(!stderr.contains("own expl3-code.tex was read"), "{stderr}");
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    stderr.contains(&format!(
      "Info:dump:other_tree latex dump {}/latex.{year}.dump.txt was built from another TeX tree: latex.ltx (built from /older/texlive/latex.ltx;",
      forged.path().display()
    )),
    "{stderr}"
  );
  for kind in ["plain", "latex"] {
    assert!(
      stderr.contains(&format!(
        "[format_dumps] no {kind} dump matches this TeX tree; building one into"
      )),
      "{stderr}"
    );
  }
  let built_into: Vec<_> = std::fs::read_dir(cache.path())
    .expect("read the cache")
    .flatten()
    .map(|entry| entry.path())
    .collect();
  assert_eq!(built_into.len(), 1, "{built_into:?}");
  // The built dumps record this tree's sources, what the loaders compare (the discriminating check
  // that the build read the tree's files, not the document's, is the own `expl3-code.tex` above).
  for kind in ["plain", "latex"] {
    let dump = std::fs::read_to_string(built_into[0].join(format!("{kind}.{year}.dump.txt")))
      .expect("read a built dump");
    let sources = latexml_engine::dump_paths::runtime_source_stamps(kind).expect("this tree");
    assert_eq!(
      latexml_engine::dump_paths::recorded_source_stamps(&dump),
      sources.to_vec()
    );
  }
  let loaded_from_cache = stderr.lines().any(|line| {
    line.starts_with("Info:dump_reader:loaded")
      && line.contains(&cache.path().display().to_string())
      && line.contains(&format!("latex.{year}.dump.txt"))
  });
  assert!(loaded_from_cache, "{stderr}");
  assert_element(
    &xml,
    "para",
    &["xml:id=\"p1\""],
    r#"<para xml:id="p1"><p>Text.</p></para>"#,
  );
  // A cached dump another user could have written is not loaded: it is built again — here after a
  // build that died with its process, which is tried once more and, succeeding, forgotten.
  let key = &built_into[0];
  let plain_cached = key.join(format!("plain.{year}.dump.txt"));
  let attempts = key.join(".attempts.plain");
  let failed = key.join(".failed.plain");
  #[cfg(unix)]
  {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(&plain_cached, std::fs::Permissions::from_mode(0o666))
      .expect("make the cached dump writable by all");
    std::fs::write(&attempts, "1\tthe plain format build died with its process")
      .expect("record a dead build");
    let (ok, stderr, _) = convert_cli_beside(tex, &own_copy, &forced);
    assert!(ok, "the conversion exited non-zero:\n{stderr}");
    assert!(
      stderr.contains("[format_dumps] no plain dump matches this TeX tree; building one into"),
      "{stderr}"
    );
    assert!(!stderr.contains("[format_dumps] no latex dump"), "{stderr}");
    let mode = std::fs::metadata(&plain_cached)
      .expect("the rebuilt dump")
      .permissions()
      .mode();
    assert_eq!(mode & 0o777, 0o644);
    assert!(
      !attempts.exists(),
      "a successful build keeps its failed attempts"
    );
  }
  // A build that fails (here a `plain.tex` ahead of the tree's on TEXINPUTS raises an error) is
  // counted, not marked; the second failure in a row marks the format failed, with its reason, and
  // the format loads from the engine's definitions, which each conversion's log says.
  std::fs::remove_file(&plain_cached).expect("remove the cached plain dump");
  let shadow = tempfile::tempdir().expect("create tempdir");
  std::fs::write(
    shadow.path().join("plain.tex"),
    // (initex reads a format with the braces at 12, plain.tex:11-12)
    "\\catcode`\\{=1 \\catcode`\\}=2 \\errmessage{a plain.tex that fails}\n",
  )
  .expect("write a failing plain.tex");
  let mut failing = forced.to_vec();
  failing.push(("TEXINPUTS", format!("{}:", shadow.path().display())));
  let plain_tex = "Text.\n\\bye\n";
  let failure = "building the plain format logged 1 error(s)";
  let (ok, stderr, _) = convert_cli_beside(plain_tex, &[], &failing);
  assert!(ok, "the conversion exited non-zero:\n{stderr}");
  assert!(stderr.contains("building one into"), "{stderr}");
  assert_eq!(
    std::fs::read_to_string(&attempts).expect("the counted failure"),
    format!("1\t{failure}")
  );
  assert!(!failed.exists(), "one failed build marks the format failed");
  let (ok, stderr, xml) = convert_cli_beside(plain_tex, &[], &failing);
  assert!(ok, "the conversion exited non-zero:\n{stderr}");
  assert_eq!(
    std::fs::read_to_string(&failed).expect("the failed mark"),
    failure
  );
  let reported = format!(
    "no dump matches this TeX tree and none could be built ({failure}; remove {} to retry)",
    failed.display()
  );
  assert!(
    stderr.contains(&format!(
      "[format_dumps] {reported}: the format loads from the engine's own definitions instead of a dump"
    )),
    "{stderr}"
  );
  assert!(
    stderr.contains(&format!("Warning:dump:build_failed No format dump matches this TeX tree and none could be built ({reported})")),
    "{stderr}"
  );
  assert_element(&xml, "para", &[], "<para><p>Text.</p></para>");
  assert!(!attempts.exists(), "the mark keeps the count beside it");
  assert_eq!(warning_count(&stderr), 1, "{stderr}");
  // The failing build's own `\errmessage`, on the build thread: not the conversion's.
  assert_eq!(error_count(&stderr), 1, "{stderr}");
  // Removing the mark, as it says, builds the format again.
  std::fs::remove_file(&failed).expect("remove the failed mark");
  let (ok, stderr, _) = convert_cli_beside(plain_tex, &[], &forced);
  assert!(ok, "the conversion exited non-zero:\n{stderr}");
  assert!(stderr.contains("building one into"), "{stderr}");
  assert!(!stderr.contains("Warning:dump:build_failed"), "{stderr}");
  assert!(plain_cached.exists(), "the plain dump was not rebuilt");
  // Builds that died with their process twice mark the format without a third build (counted
  // before each build).
  std::fs::remove_file(&plain_cached).expect("remove the cached plain dump");
  std::fs::write(&attempts, "2\tthe plain format build died with its process")
    .expect("record two dead builds");
  let (ok, stderr, _) = convert_cli_beside(plain_tex, &[], &forced);
  assert!(ok, "the conversion exited non-zero:\n{stderr}");
  assert!(!stderr.contains("building one into"), "{stderr}");
  assert_eq!(
    std::fs::read_to_string(&failed).expect("the failed mark"),
    "the plain format build died with its process"
  );
  assert!(!attempts.exists(), "the mark keeps the count beside it");
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 1, "{stderr}");
  // A mark lapses after a day: the format is built again, and succeeding, forgets its failures.
  let two_days_ago =
    std::time::SystemTime::now() - std::time::Duration::from_secs(2 * 24 * 60 * 60);
  std::fs::File::options()
    .write(true)
    .open(&failed)
    .and_then(|file| file.set_modified(two_days_ago))
    .expect("age the failed mark");
  let (ok, stderr, _) = convert_cli_beside(plain_tex, &[], &forced);
  assert!(ok, "the conversion exited non-zero:\n{stderr}");
  assert!(stderr.contains("building one into"), "{stderr}");
  assert!(!stderr.contains("Warning:dump:build_failed"), "{stderr}");
  assert!(
    !failed.exists() && !attempts.exists(),
    "a lapsed mark is kept"
  );
  assert!(plain_cached.exists(), "the plain dump was not rebuilt");

  let matching = tempfile::tempdir().expect("create tempdir");
  std::fs::write(
    matching.path().join(format!("latex.{year}.dump.txt")),
    &latex_dump,
  )
  .expect("write latex dump");
  std::fs::write(
    matching.path().join(format!("plain.{year}.dump.txt")),
    &plain_dump,
  )
  .expect("write plain dump");
  let unused_cache = tempfile::tempdir().expect("create tempdir");
  let (ok, stderr, xml) = convert_cli_beside(tex, &own_copy, &[
    ("LATEXML_DUMP_DIR", matching.path().display().to_string()),
    ("LATEXML_DUMP_DIR_ONLY", String::from("1")),
    ("LATEXML_NO_EMBEDDED_DUMP", String::from("1")),
    (
      "LATEXML_FORMAT_CACHE",
      unused_cache.path().display().to_string(),
    ),
  ]);
  assert!(ok, "the conversion exited non-zero:\n{stderr}");
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("[format_dumps]"), "{stderr}");
  let loaded_from_matching = stderr.lines().any(|line| {
    line.starts_with("Info:dump_reader:loaded")
      && line.contains(&format!(
        "{}/latex.{year}.dump.txt",
        matching.path().display()
      ))
  });
  assert!(loaded_from_matching, "{stderr}");
  assert_eq!(
    std::fs::read_dir(unused_cache.path())
      .expect("read the cache")
      .count(),
    0
  );
  assert_element(
    &xml,
    "para",
    &["xml:id=\"p1\""],
    r#"<para xml:id="p1"><p>Text.</p></para>"#,
  );
}
