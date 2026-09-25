//! Token-level kernel reads that follow tex.web (batch 56jj). A primitive reads
//! its tokens with `get_token`, which at `normal` scanner status crosses the end
//! of an `\input` file (§362): `\expandafter`, `\string`, `\meaning`, `\let`,
//! `\futurelet`, `\afterassignment`, `\aftergroup` and `\ifx` as a file's last
//! token take their tokens from the input after it
//! (`gullet::read_primitive_token`), and a definition's name skips the spaces
//! there (§1215 `get_r_token`, `gullet::read_redefinable_token`). Inside a
//! definition the file end is the definition's runaway, met by the primitive. A
//! token-list assignment takes an implicit left brace and skips `\relax`
//! (§1226, `gullet::read_tokens_value`). `\message`, `\errmessage`, `\mark`,
//! `\marks` and `\special` read their text with `scan_toks(false, true)`: the
//! `{` is found expanding and a file that ends inside the text is a runaway
//! (§1279, §1101, §1354; `XGeneralText`). `\meaning` names the expandable
//! primitives. Text and error counts are pdflatex's; repros in
//! `tools/perfect_kernel/repros/expansion-primitives/`.
use latexml::util::test::assert_element;

use super::perfect_kernel_batch46::{convert, error_count, warning_count};

/// The `<para xml:id="p{n}">` holding one `<p>` with `text`.
fn assert_para(xml: &str, n: usize, text: &str) {
  assert_element(
    xml,
    "para",
    &[&format!(r#"xml:id="p{n}""#)],
    &format!(r#"<para xml:id="p{n}"><p>{text}</p></para>"#),
  );
}

/// Convert a clean repro: pdflatex has no error and no warning on it.
fn convert_clean(tex: &str) -> String {
  let (log, xml) = convert(tex, true);
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  xml
}

/// A file ending in `\expandafter\x` expands the first token after it (§368):
/// `\x` gets `F` of `\foo`'s `FOO` (pdflatex "A [F]OO"; Rust gave `\x` a
/// `\relax`, "A []FOO").
#[test]
fn expandafter_crosses_a_file_end() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/input_end_expandafter.tex"
  ));
  assert_para(&xml, 1, "A [F]OO");
}

/// `\string` at a file's end strings the first token after it (§471;
/// pdflatex "B [\foo]", OT1's backslash a quote; Rust strung `\relax`).
#[test]
fn string_crosses_a_file_end() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/input_end_string.tex"
  ));
  assert_para(&xml, 1, "B [\u{201C}foo]");
}

/// `\meaning` at a file's end shows the meaning of the first token after it
/// (§471; pdflatex "C [macro:->FOO]", OT1's `>` an inverted question mark).
#[test]
fn meaning_crosses_a_file_end() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/input_end_meaning.tex"
  ));
  assert_para(&xml, 1, "C [macro:-\u{BF}FOO]");
}

/// `\let\y=` and a bare `\let` as a file's last tokens take the name and the
/// value from the input after it, and a space before or after the `=` may be
/// implicit (§1221): `\let\c\@sptoken=\b` lets `\c` to `\b` (pdflatex "D
/// [FOO]", "E [FOO]", "[macro:->B]"; Rust 1 error, `\c` let to `=`).
#[test]
fn let_crosses_a_file_end() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/input_end_let.tex"
  ));
  assert_para(&xml, 1, "D [FOO]");
  assert_para(&xml, 2, "E [FOO]");
  assert_para(&xml, 3, "[macro:-\u{BF}B]");
}

/// `\futurelet\z` as a file's last tokens lets `\z` to the second token after
/// the file (pdflatex "F FOO[FOO]"; Rust "F FOO[]").
#[test]
fn futurelet_crosses_a_file_end() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/input_end_futurelet.tex"
  ));
  assert_para(&xml, 1, "F FOO[FOO]");
}

/// `\afterassignment` at a file's end saves `\foo` from after the file, run
/// after the `\def` that redefines it (§1268; pdflatex "G BAR", Rust "G FOO").
#[test]
fn afterassignment_crosses_a_file_end() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/input_end_afterassignment.tex"
  ));
  assert_para(&xml, 1, "G BAR");
}

/// `\aftergroup` at a file's end saves `\foo` from after the file for the
/// group's end (§1271; pdflatex "H XFOO", Rust "H FOOX").
#[test]
fn aftergroup_crosses_a_file_end() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/input_end_aftergroup.tex"
  ));
  assert_para(&xml, 1, "H XFOO");
}

/// `\ifx\foo` as a file's last tokens compares `\foo` with the first token
/// after the file (§507; pdflatex "H YES", Rust 1 error and "H FOOYES").
#[test]
fn ifx_crosses_a_file_end() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/input_end_ifx.tex"
  ));
  assert_para(&xml, 1, "H YES");
}

/// Inside `\edef\y{`, a file ending in `\ifx\foo` or `\ifdefined` compares
/// with the first token after the file: tex.web §507 reads `\ifx`'s tokens
/// at `normal` status (eTeX's `\ifdefined` too), so the definition's runaway
/// does not fire (pdflatex 0 errors, "I [macro:->YES]", "J [macro:->YES]";
/// before, the inserted `}` was `\ifx`'s second token, `\y` = `NO`).
#[test]
fn ifx_reads_at_normal_status_in_a_definition() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/input_end_ifx_in_edef.tex"
  ));
  assert_para(&xml, 1, "I [macro:-\u{BF}YES]");
  assert_para(&xml, 2, "J [macro:-\u{BF}YES]");
}

/// A bare `\let` or `\def` as a file's last token names the first non-space
/// token after the file (§1215 `get_r_token`), after an `\input` file and a
/// `\scantokens` pseudo-file alike (pdflatex "E [FOO]", "F [ZZ]", "G [FOO]";
/// Rust made the parent's space the name, or nothing).
#[test]
fn a_definition_name_crosses_a_file_end() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/input_end_redefinable_name.tex"
  ));
  assert_para(&xml, 1, "E [FOO]");
  assert_para(&xml, 2, "F [ZZ]");
  assert_para(&xml, 3, "G [FOO]");
}

/// Inside `\edef\x{`, a file ending under `\expandafter` is the definition's
/// runaway (§338), and TeX's inserted `}` ends the body, so `\x` is empty and
/// the parent's `\foo` is expanded after it; `\string` reads at normal status
/// (§471) and strings the parent's `\foo` into `\y` (pdflatex 1 error, "A
/// FOO[macro:->]", "B [macro:->\foo]"; Rust 3 errors, `\relax` in both).
#[test]
fn an_edef_body_primitive_at_a_file_end() {
  let (log, xml) = convert(
    include_str!(
      "../../../tools/perfect_kernel/repros/expansion-primitives/input_end_edef_primitives.tex"
    ),
    true,
  );
  assert_eq!(error_count(&log), 1, "{log}");
  assert_eq!(
    log
      .matches("Error:expected:} File ended while scanning definition")
      .count(),
    1,
    "{log}"
  );
  assert_eq!(warning_count(&log), 0, "{log}");
  assert_para(&xml, 1, "A FOO[macro:-\u{BF}]");
  assert_para(&xml, 2, "B  [macro:-\u{BF}\u{201C}foo]");
}

/// `\meaning` of `\expandafter`, `\string`, `\meaning`, `\noexpand`, and of a
/// `\let` alias, is the primitive's name (§296), while a redefined `\string`
/// is a macro again (pdflatex; Rust and Perl printed `macro:…->CODE(<address>)`).
#[test]
fn meaning_names_an_expandable_primitive() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/meaning_of_expandable_primitive.tex"
  ));
  // OT1 prints `\` as a left quote and `>` as an inverted question mark.
  let (q, gt) = ('\u{201C}', '\u{BF}');
  assert_para(
    &xml,
    1,
    &format!(
      "[{q}expandafter] [{q}string] [{q}meaning] [{q}noexpand] [{q}let] [{q}expandafter] \
       [macro:-{gt}S]"
    ),
  );
}

/// `\toks0=\bgroup abc}`: the left brace of a token-list assignment may be
/// implicit, `\relax` before it is skipped, a macro is expanded to find it, a
/// register gives its value, and `\toks8=` as a file's last tokens takes the
/// text after the file (§1226-1227; pdflatex 0 errors, "[abc] [def] [ghi]
/// [xyz] [ghi] I [jkl]"; Rust 1 error, `\bgroup` and `\relax` stored).
#[test]
fn a_token_list_assignment_takes_an_implicit_brace() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/toks_implicit_left_brace.tex"
  ));
  assert_para(&xml, 1, "[abc] [def] [ghi] [xyz] [ghi] I [jkl]");
}

/// `\toks0=x`: "Missing { inserted" (§403), one error; the token is the value,
/// Perl's recovery, where TeX absorbs up to the next `}` (pdflatex 4 errors, an
/// emergency stop at the file's end).
#[test]
fn a_token_list_without_a_brace() {
  let (log, xml) = convert(
    include_str!(
      "../../../tools/perfect_kernel/repros/expansion-primitives/toks_missing_left_brace.tex"
    ),
    true,
  );
  assert_eq!(error_count(&log), 1, "{log}");
  assert!(log.contains("Error:expected:{ Missing { inserted"), "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  assert_para(&xml, 1, "[x]");
}

/// `\message\expandafter{\foo}` logs `hi` (tex.web §1279 `scan_toks(false,
/// true)` finds the `{` expanding); `\errmessage`, `\special` and `\mark` read
/// alike, as does eTeX's `\marks`, and none of them typesets its text
/// (pdflatex 1 error, the `\errmessage`, and "AB C D E F G"; Rust typeset `hi`
/// four times).
#[test]
fn message_finds_its_brace_expanding() {
  let (log, xml) = convert(
    include_str!("../../../tools/perfect_kernel/repros/expansion-primitives/message_scan_toks.tex"),
    true,
  );
  assert_eq!(error_count(&log), 1, "{log}");
  assert!(log.contains("Error:errmessage:\\errmessage hi"), "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  for line in ["hi", "frhyphex"] {
    assert!(
      log.lines().any(|l| l == line),
      "no \\message line {line:?}:\n{log}"
    );
  }
  assert_para(&xml, 1, "AB C D E F G");
}

/// `\message{\the\toks0 \p}` logs `\foo \p `: the register's tokens are not
/// expanded further and a `\protected` macro is kept (§477-478; pdflatex's log
/// line; Rust logged `FP`).
#[test]
fn message_keeps_the_and_protected() {
  let (log, xml) = convert(
    include_str!(
      "../../../tools/perfect_kernel/repros/expansion-primitives/message_the_protected.tex"
    ),
    true,
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  assert!(log.lines().any(|l| l == "\\foo \\p "), "{log}");
  assert_para(&xml, 1, "A");
}

/// A file that ends inside `\message{abc` is a runaway of the text, which ends
/// at the file's end (§338-339): pdflatex's one error, "File ended while
/// scanning text of \message", and "A B" (Rust reported Perl's
/// "readBalanced ran out of input").
#[test]
fn a_file_ending_inside_a_message() {
  let (log, xml) = convert(
    include_str!("../../../tools/perfect_kernel/repros/expansion-primitives/message_file_end.tex"),
    true,
  );
  assert_eq!(error_count(&log), 1, "{log}");
  assert!(
    log.contains("Error:expected:} File ended while scanning text of \\message"),
    "{log}"
  );
  assert_eq!(warning_count(&log), 0, "{log}");
  assert!(log.lines().any(|l| l.trim_end() == "abc"), "{log}");
  assert_para(&xml, 1, "A B");
}

/// An unbraced `\mark x` or `\message y` is an error, one each as in pdflatex
/// ("Missing { inserted"). The recovery is Perl's: the token is put back and
/// the text is empty, so `x` and `y` are typeset, where TeX absorbs them and
/// the group's `}` into the text (pdflatex "AB", "CD"). Rust took the one
/// token as a `{}` argument, silently.
#[test]
fn an_unbraced_mark_is_an_error() {
  let (log, xml) = convert(
    include_str!("../../../tools/perfect_kernel/repros/expansion-primitives/mark_unbraced.tex"),
    true,
  );
  assert_eq!(error_count(&log), 2, "{log}");
  assert_eq!(
    log.matches("Error:expected:{ Expected opening '{'").count(),
    2,
    "{log}"
  );
  assert_eq!(warning_count(&log), 0, "{log}");
  assert_para(&xml, 1, "AxB");
  assert_para(&xml, 2, "CyD");
}
