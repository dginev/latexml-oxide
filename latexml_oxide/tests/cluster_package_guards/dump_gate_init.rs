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

/// Convert `tex` with the CLI binary in a tempdir, with no preload (so a
/// document without `\documentclass` stays plain TeX), the given `--timeout`
/// and extra child-process environment. Returns (exit success, ANSI-stripped
/// stderr, XML).
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
  let (_, stderr, xml) = convert_cli(tex, 110, &[]);
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
