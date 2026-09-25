//! `\noexpand` reads its token under normal scanner status (tex.web §367), so
//! the end of the input level it meets closes without a runaway (§362, §336)
//! and the read goes on in the enclosing input. `\everyeof{\noexpand}` is the
//! idiom that lets an `\edef`, `\message` or x-expansion swallow a whole file
//! (catchfile.sty:251-261 `\CatchFileEdef`, l3build regression-test.tex:101,
//! morewrites.sty:465, biblatex.sty:1253). A definition WITHOUT that idiom
//! still runs off the file's end as a runaway (§338). Batch 56ix; repros in
//! `tools/perfect_kernel/repros/expansion-primitives/`.
use latexml::util::test::assert_element;

use super::perfect_kernel_batch46::{convert, convert_files, error_count, warning_count};

const INPUT_END_TXT: &str = include_str!(
  "../../../tools/perfect_kernel/repros/expansion-primitives/everyeof_noexpand_input_end.txt"
);

/// `\everyeof{\noexpand}\edef\x{\@@input f }`: the `\noexpand` at the file's
/// end reads the parent's `}`, so the body is the whole file (pdflatex
/// `[AFOOB ]`, 0 errors; Rust and Perl had 2 errors).
#[test]
fn noexpand_crosses_a_file_end_in_an_edef() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/everyeof_noexpand_input_end.tex"
  );
  let (stderr, xml) = convert_files(tex, &[("everyeof_noexpand_input_end.txt", INPUT_END_TXT)]);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(&xml, "p", &[], "<p>[AFOOB ]</p>");
}

/// The control: without `\everyeof{\noexpand}` the same `\edef` is a runaway
/// in pdflatex too ("File ended while scanning definition of \x"), and so it
/// stays here.
#[test]
fn a_definition_still_runs_off_a_file_end() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/everyeof_absent_input_runaway.tex"
  );
  let (stderr, _xml) = convert_files(tex, &[("everyeof_noexpand_input_end.txt", INPUT_END_TXT)]);
  assert_eq!(error_count(&stderr), 2, "{stderr}");
  assert!(
    stderr.contains("Error:expected:} Gullet->readBalanced ran out of input"),
    "{stderr}"
  );
}

/// At the end of a `\scantokens` pseudo-file the `\noexpand` reads the next
/// token of the enclosing input and leaves it unexpanded: `\bar` stays `\bar`
/// (pdflatex `[same]`). Rust expanded it to `BAR`, silently.
#[test]
fn noexpand_crosses_a_scantokens_end() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/everyeof_noexpand_scantokens_end.tex"
  );
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(&xml, "p", &[], "<p>[same]</p>");
}

/// `\message{\@@input f }` under `\everyeof{\noexpand}` (l3build
/// regression-test.tex:101): the message scan crosses the file's end.
#[test]
fn noexpand_crosses_a_file_end_in_a_message() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/everyeof_noexpand_message.tex"
  );
  let txt = include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/everyeof_noexpand_message.txt"
  );
  let (stderr, xml) = convert_files(tex, &[("everyeof_noexpand_message.txt", txt)]);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(&xml, "p", &[], "<p>done</p>");
}

/// expl3's form: `\tex_everyeof:D{\exp_not:N}` around an x-expanded
/// `\tex_input:D` (morewrites.sty:465). pdflatex `[L1FOOL2]`.
#[test]
fn expl3_exp_not_crosses_a_file_end() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/everyeof_expl3_input_x.tex"
  );
  let txt = include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/everyeof_expl3_input_x.txt"
  );
  let (stderr, xml) = convert_files(tex, &[("everyeof_expl3_input_x.txt", txt)]);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(&xml, "p", &[], "<p>[L1FOOL2]</p>");
}

/// `\everyeof` is inserted only when a file really ends, not after
/// `\endinput` (eTeX; pdflatex `[X]`), and the rest of the `\endinput` line is
/// still read (tex.web §362: `X\endinput Z⏎Y` gives `XZ `, pdflatex).
#[test]
fn endinput_does_not_insert_everyeof() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/everyeof_after_endinput.tex"
  );
  let txt =
    include_str!("../../../tools/perfect_kernel/repros/expansion-primitives/everyeof_endinput.txt");
  let (stderr, xml) = convert_files(tex, &[("everyeof_endinput.txt", txt)]);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(&xml, "p", &[], "<p>[X]</p>");
  let (stderr, xml) = convert_files(tex, &[("everyeof_endinput.txt", "X\\endinput Z\nY")]);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_element(&xml, "p", &[], "<p>[XZ ]</p>");
}

/// catchfile's `\CatchFileEdef` (catchfile.sty:251-261) is that idiom: with
/// `\noexpand` crossing the file's end it reads the file itself, not a string
/// copy, so a Latin-1 byte decodes under `inputenc[latin1]` as in
/// `\CatchFileDef` (pdflatex `[café][café ]`; the string stand-in gave
/// `[café][caf�]`). The side file is written as bytes: it is not UTF-8.
#[test]
fn catchfile_edef_reads_the_file_bytes() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/singletons/catchfile_edef_latin1.tex");
  let bin = env!("CARGO_BIN_EXE_latexml_oxide");
  let workdir = tempfile::tempdir().expect("create tempdir");
  std::fs::write(workdir.path().join("t.tex"), tex).expect("write t.tex");
  std::fs::write(
    workdir.path().join("catchfile_edef_latin1.txt"),
    include_bytes!("../../../tools/perfect_kernel/repros/singletons/catchfile_edef_latin1.txt"),
  )
  .expect("write the Latin-1 file");
  let output = std::process::Command::new(bin)
    .args([
      "t.tex",
      "--dest",
      "t.xml",
      "--nocomments",
      "--timeout=110",
      "--preload=[rawstyles,rawclasses]latexml.sty",
    ])
    .current_dir(workdir.path())
    .output()
    .expect("spawn latexml_oxide");
  let stderr = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
  let xml = std::fs::read_to_string(workdir.path().join("t.xml")).unwrap_or_default();
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(&xml, "p", &[], "<p>[café][café ]</p>");
}
