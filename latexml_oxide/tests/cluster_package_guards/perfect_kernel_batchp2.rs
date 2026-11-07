//! The native answer of l3fp's doors (`latexml_engine/src/l3fp_native`, OXIDIZED_DESIGN_DIVERGENCES #977): each fixture converted
//! with the native path on and off (`l3fp_native::set_native_override`) gives byte-identical XML and the same Errors, Warnings
//! and Fatals, whatever it does, with the native path proven to have answered (`native_counts`: raw matches pdflatex too, so
//! nothing else would tell); what the arithmetic must print is held against pdflatex's own answers (`kit/l3fp_oracle.sh` at
//! 100,000 operations per family; a slice of it here).
use latexml_engine::l3fp_native::{native_counts, set_check_override, set_native_override};

use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch84b::text,
};

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/expl3/",
      $name,
      ".tex"
    ))
  };
}

const PRELOAD: &str = "[rawstyles,rawclasses]latexml.sty";

/// A conversion: its log, its XML, and how many doors the native path answered (`None` when it was not installed).
fn convert(tex: &str, native: bool, check: bool) -> (String, String, Option<u64>) {
  let (log, xml, counts) = convert_counted(tex, native, check);
  (log, xml, counts.map(|counts| counts.0))
}

/// A conversion: its log, its XML, and how many doors the native path answered and how many it left to the macro.
fn convert_counted(tex: &str, native: bool, check: bool) -> (String, String, Option<(u64, u64)>) {
  latexml::util::test::convert_with_setup_then(
    tex,
    Some(PRELOAD),
    move || {
      set_native_override(Some(native));
      set_check_override(Some(check));
    },
    |_| native_counts(),
  )
}

/// As many doors answered natively. On a TeX tree whose l3fp the module does not port (its `\ExplLoaderFileDate`, CI's distro
/// tree) nothing is installed: the fixtures still hold raw against pdflatex's text, and only the count is moot. Under the
/// year this module was ported from, `the_install_takes_on_the_ported_year` says it is not moot.
fn assert_answered(answered: Option<u64>, count: u64, what: &str) {
  if let Some(answered) = answered {
    assert_eq!(answered, count, "the native path: {what}");
  }
}

fn fatal_count(log: &str) -> usize { log.lines().filter(|l| l.starts_with("Fatal:")).count() }

/// The XML with each run of white space as one space, as the printed values are compared.
fn squeezed(xml: &str) -> String { xml.split_whitespace().collect::<Vec<_>>().join(" ") }

/// Native on and off give the same document and the same report, which is returned with the XML and how many doors the
/// native path answered; off, it is not installed.
fn both_ways(tex: &str) -> (String, String, Option<u64>) {
  let (log, xml, counts) = both_ways_counted(tex);
  (log, xml, counts.map(|counts| counts.0))
}

/// [`both_ways`], with how many doors the native path left to the macro as well.
fn both_ways_counted(tex: &str) -> (String, String, Option<(u64, u64)>) {
  let (log_on, on, answered) = convert_counted(tex, true, false);
  let (log_off, off, answered_off) = convert_counted(tex, false, false);
  assert_eq!(answered_off, None, "the switch");
  assert_eq!(
    error_count(&log_on),
    error_count(&log_off),
    "native: {log_on}\nraw: {log_off}"
  );
  assert_eq!(
    warning_count(&log_on),
    warning_count(&log_off),
    "native: {log_on}\nraw: {log_off}"
  );
  assert_eq!(
    fatal_count(&log_on),
    fatal_count(&log_off),
    "native: {log_on}\nraw: {log_off}"
  );
  assert_eq!(on, off, "native and raw must give byte-identical XML");
  (log_on, on, answered)
}

/// The switch on, the l3fp of the TeX tree this module ports takes the native call: else every count here is moot. By the tree's
/// own `\ExplLoaderFileDate`, whatever the tree is. A year not ported runs the macros, and the guards hold them alone.
#[test]
fn the_install_takes_on_the_ported_year() {
  let tex = r"\documentclass{article}
\ExplSyntaxOn
\edef\expldate{\ExplLoaderFileDate}
\ExplSyntaxOff
\begin{document}
[\expldate]
\end{document}
";
  let (_, xml, answered) = convert(tex, true, false);
  if xml.contains("[2025-11-06]") || xml.contains("[2026-01-19]") {
    assert_eq!(answered, Some(0), "installed, and nothing to answer: {xml}");
  } else {
    assert_eq!(answered, None, "another year keeps its macros: {xml}");
  }
}

/// Exponent limits, signed zeros, ties in the reader and in `+ - * /`, values next to multiples of pi/4, huge and tiny
/// angles, 16-digit round-offs: pdflatex's text. Under the self-check, every native answer is compared with the raw macro's.
#[test]
fn the_hard_values_are_pdflatexs() {
  let (log, xml, answered) = both_ways(repro!("l3fp_hard_cases"));
  let xml = squeezed(&xml);
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  assert_answered(answered, 237, "the fixture");
  for printed in [
    "Zeros: 0; -0; -0; 0; 0; -0; -0; 0; 0; 0; -0; 0; -0; 0; -0; -0; 0; 0;",
    "Infinities: inf; -inf; inf; -inf; -inf; 0; inf; 0; inf; inf; 1.570796326794897; 0; 0.7853981633974483;",
    "Limits: inf; -inf; 1e-9999; 9.999999999999999e9999; 9e9999; 1e-10000; 1e-10000; 1e9999; 4.270176462662952e9999;",
    "Ties: 1; 1.000000000000002; 1.000000000000001; 1e16; 9999999999999998; 1; 1.000000000000002; 1; 0.9999999999999998;",
    "2.384626433832795e-16; -4.76925286766559e-16; 1; -3.807686783083602e-16; -2626266436731868; -0.3723761236612767;",
  ] {
    assert!(xml.contains(printed), "{printed}\n{xml}");
  }
  let (log, _, checked) = convert(repro!("l3fp_hard_cases"), true, true);
  assert_eq!(error_count(&log), 0, "the self-check: {log}");
  assert_eq!(checked, answered, "the self-check saw every answer");
}

/// A seeded slice of the oracle's expressions in every family, against what pdflatex printed.
#[test]
fn a_seeded_slice_matches_pdflatex() {
  let slice = include_str!("../../../tools/perfect_kernel/repros/expl3/l3fp_oracle_slice.tsv");
  let rows: Vec<(&str, &str)> = slice
    .lines()
    .filter(|l| !l.starts_with('#'))
    .map(|l| {
      let mut f = l.split('\t');
      f.next();
      (f.next().unwrap(), f.next().unwrap())
    })
    .collect();
  let mut tex = String::from(
    r"\documentclass{article}
\ExplSyntaxOn
\fp_trap:nn { invalid_operation } { none }
\fp_trap:nn { division_by_zero } { none }
\fp_trap:nn { overflow } { none }
\fp_trap:nn { underflow } { none }
\fp_new:N \oraclefp \fp_set:Nn \oraclefp { 1.25 }
\dim_new:N \oracledim \dim_set:Nn \oracledim { 12.5pt }
\int_new:N \oracleint \int_set:Nn \oracleint { -37 }
\cs_new:Npn \oraclemacro { 25 }
\cs_new_eq:NN \oraclee \c_e_fp
\cs_new_eq:NN \oraclepi \c_pi_fp
\cs_new_protected:Npn \T #1 { [ \fp_to_tl:n {#1} ] ~ }
\ExplSyntaxOff
\begin{document}
",
  );
  for (expr, _) in &rows {
    tex.push_str(&format!("\\T{{{expr}}}\n"));
  }
  tex.push_str("\\end{document}\n");
  let (log, xml, answered) = convert(&tex, true, false);
  assert_eq!(error_count(&log), 0, "{log}");
  assert_answered(answered, 1024, "of 1,050 rows");
  let printed: Vec<String> = xml
    .split('[')
    .skip(1)
    .map(|v| {
      v.split(']')
        .next()
        .unwrap()
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect()
    })
    .collect();
  assert_eq!(printed.len(), rows.len(), "{xml}");
  for ((expr, want), got) in rows.iter().zip(&printed) {
    assert_eq!(got, want, "{expr}");
  }
}

/// A tuple, nan, rand, an unknown word, every flag and message, a primitive, a conditional or `\csname` in the argument, an
/// active character: the raw macro's result and its Errors, as many as it reports, with no effect of the native attempt
/// (a `\csname` is run once, by the raw macro).
#[test]
fn what_it_declines_is_the_raw_macros() {
  let (log, xml, answered) = both_ways(repro!("l3fp_fallback_shapes"));
  let xml = squeezed(&xml);
  assert_eq!(error_count(&log), 37, "{log}");
  assert_answered(answered, 33, "the fixture");
  assert!(xml.contains("relax."), "{xml}");
  for row in [
    "Tuples: nan; 6; (1, 2); (4, 6); (2, 4); (); (1,); nan;",
    "Nan: nan; nan; nan; nan; nan; 0; 1;",
    "Flags: inf; nan; nan; nan; nan; -inf; nan; nan; nan; nan; inf; inf; -inf; nan; nan; inf; nan; inf; inf; 1e-10001; inf; 0; inf; 0; 0; 0; 0; nan; nan; nan;",
    "Words: 0; 1; 3; 4; 4; inf; inf; 0; 0;",
    "Tokens: 7; 700; 1.5; 150; 2; 0; 7; 7; nan; nan; 12; 3; nan; 7; 3; 5; 1; 21; 3; 3; 6.283185307179586; 6.283185307179586; 12; 14; 4; 5; 12; nan; 2;",
  ] {
    assert!(xml.contains(row), "{row}\n{xml}");
  }
}

/// Digits run on across a macro, a space from an expansion stops one being expanded, words and operators are read across
/// macros, registers take an exponent, the converters `\dim_ratio:nn`, `\dim_to_fp:n`, `\fp_eval:n`, `\fp_use:N`, `\the`
/// and `\number` read a value, `\csname` of a known name; a document's `\number` is a macro like any other.
#[test]
fn it_expands_when_the_raw_parser_does() {
  let (log, xml, answered) = both_ways(repro!("l3fp_expansion_order"));
  let xml = squeezed(&xml);
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  assert_answered(answered, 119, "the fixture");
  for row in [
    "Digits: 12; 23; 23; 123; 1000000; 14; 42; 52; 52; 152; 2.3; 2.3; 0.2; 100; 100; 100; 100; 100; 0.01; 100; 2000; 23; 2;",
    "Registers: 3; -3; 300; 0.03; 1.5; -1.5; 150; 3; 16383.99998474121;",
    "Converters: 1; 71.13188171386719; 1.5; 2; 0; -144.5399780273438; 2; 0.9999999999999999; 1000; 0.001; -0; 1e20; 1e16; 123.456;",
    "Names: 5; 6; 4; 4; 5.271897055612996e-4; 1.5;",
  ] {
    assert!(xml.contains(row), "{row}\n{xml}");
  }
}

/// The expansion is a list for `invoke_expansion`: a printed `\@title` holding an fp expression is lifted, once, as with the
/// raw macro (the lesson of pk/P1's `\expandafter`).
#[test]
fn an_expression_in_a_printed_title_stays_a_field() {
  let (log, xml, answered) = both_ways(repro!("l3fp_in_a_title"));
  assert_eq!(error_count(&log), 0, "{log}");
  assert_answered(answered, 3, "the fixture");
  assert!(xml.contains("<title>Value 2.5 of it</title>"), "{xml}");
  assert_eq!(xml.matches("2.5 of it").count(), 1, "{xml}");
}

/// Nothing is reported before an answer: a dimension the engine's reader would call too large is the raw macro's to read, so
/// `Dimension too large` comes once (native: twice, as the first-round cap of 2000 of any unit passed 2000in), and below
/// `\maxdimen` in each unit the native path answers.
#[test]
fn a_dimension_too_large_is_reported_once() {
  let (log, xml, answered) = both_ways(repro!("l3fp_reported_once"));
  let xml = squeezed(&xml);
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 3, "{log}");
  assert_answered(answered, 8, "the fixture");
  for row in [
    "Below: 0; 15827.12998962402; 15588; 16189.61810302734; 16215.2255859375; 16058.99624633789; 16049.05963134766; 16037.28953552246; 16383.99998474121;",
    "Over: nan; inf; (16383.99998474121, 32767.99996948242);",
  ] {
    assert!(xml.contains(row), "{row}\n{xml}");
  }
}

/// One rule for every l3fp name the parser knows or the macros' run depends on: its meaning changed, however locally, and the
/// macros answer from then on. Each row a document of its own, since the rule lasts: what it sets up, what it prints as
/// pdflatex does, its Errors and Fatals. `1+1` before it is the one native answer. A word as a variable (in a group too, the
/// macro's for good) or as a function, by `\cs_set_eq:NN`, undefined; an operator; an arithmetic macro; a constant; the door;
/// the mark.
#[test]
fn a_changed_name_of_l3fp_is_left_to_the_macros() {
  const MARKED: &str = r"\fp_clear_variable:n{pi}\fp_set_variable:nn{pi}{3}\T{pi+1}";
  let template = repro!("l3fp_changed_names");
  assert!(template.contains(MARKED));
  for (row, printed, errors, fatals) in [
    (MARKED, "[2] [4]", 0, 0),
    (
      r"\group_begin:\fp_clear_variable:n{pi}\fp_set_variable:nn{pi}{3}\T{pi+1}\group_end:\T{pi+1}",
      "[2] [4] [4.141592653589793]",
      0,
      0,
    ),
    (
      r"\fp_new_function:n{cos}\fp_set_function:nnn{cos}{x}{x+1}\T{cos(1)}",
      "[2] [2]",
      1,
      0,
    ),
    (
      r"\cs_set_eq:NN\__fp_parse_word_true:N\__fp_parse_word_false:N\T{true}",
      "[2] [0]",
      0,
      0,
    ),
    (
      r"\cs_undefine:N\__fp_parse_word_pi:N\T{pi}",
      "[2] [nan]",
      1,
      0,
    ),
    (
      r"\cs_set_eq:cc{__fp_parse_infix_+:N}{__fp_parse_infix_-:N}\T{3+1}",
      "[2] [2]",
      0,
      0,
    ),
    (
      r"\cs_set_eq:NN\__fp_sin_o:w\__fp_cos_o:w\T{sin(0)}",
      "[2] [1]",
      0,
      0,
    ),
    (r"\tl_set:Nn\c_pi_fp{\c_one_fp}\T{pi}", "[2] [1]", 0, 0),
    (
      r"\cs_set_eq:NN\orig:n\__fp_parse:n\cs_set:Npn\__fp_parse:n#1{\orig:n{2*(#1)}}\T{\__fp_parse:n{3}}",
      "[2] [12]",
      0,
      0,
    ),
    (r"\cs_set:Npn\s__fp{7}\T{\c_one_fp}", "", 1, 1),
  ] {
    let (log, xml, answered) = both_ways(&template.replace(MARKED, row));
    assert_answered(answered, 1, row);
    assert_eq!(error_count(&log), errors, "{row}: {log}");
    assert_eq!(fatal_count(&log), fatals, "{row}: {log}");
    assert!(squeezed(&xml).contains(printed), "{row}: {printed}\n{xml}");
  }
  // What l3fp assigns as it runs, a word a document adds and a copy onto itself change nothing the parser knows.
  for (row, printed, native) in [
    (r"\fp_trap:nn{invalid_operation}{none}\T{2+2}", "[2] [4]", 2),
    (
      r"\fp_new_variable:n{xx}\fp_set_variable:nn{xx}{5}\T{2+2}\T{xx+1}",
      "[2] [4] [6]",
      4,
    ),
    (r"\cs_gset_eq:NN\s__fp\s__fp\T{2+2}", "[2] [4]", 2),
  ] {
    let (log, xml, answered) = both_ways(&template.replace(MARKED, row));
    assert_answered(answered, native, row);
    assert_eq!(error_count(&log), 0, "{row}: {log}");
    assert!(squeezed(&xml).contains(printed), "{row}: {printed}\n{xml}");
  }
}

/// The doors and the operand macro stay the format's: `\meaning`, prefixes, replacement text, a one-step expansion; what the
/// macro's run leaves is what the answer leaves; a braced argument reads as the macro reads it; `\__fp_parse_o:n`'s callers
/// are answered; a `\__fp_parse:n` inside an expression (`\fp_add:Nn`, `\fp_abs:n`) is read as the macro's run would.
#[test]
fn the_doors_stay_the_formats_macros() {
  let (log, xml, answered) = both_ways(repro!("l3fp_doors"));
  let xml = squeezed(&xml);
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  assert_answered(answered, 25, "the fixture");
  let door = r"\exp:w \exp_after:wN \__fp_parse_after:ww \exp:w \__fp_parse_operand:Nw \c__fp_prec_end_int \__fp_parse_expand:w";
  let marks = r"\s__fp_expr_mark \__fp_parse_infix_end:N \s__fp_expr_stop";
  let two = r"\s__fp \__fp_chk:w 10{1}{2000}{0000}{0000}{0000}\__fp_sep:";
  for row in [
    format!(r"Meaning: \long macro:#1-&gt;{door} #1{marks} \exp_end:"),
    r"Operand: \long macro:#1-&gt;\exp_end_continue_f:w \exp_after:wN \__fp_parse_continue:NwN \exp_after:wN #1\exp:w \exp_end_continue_f:w \exp_after:wN \__fp_parse_one:Nw \exp_after:wN #1\exp:w".to_string(),
    format!(r"Replacement: {door} #1{marks} {{\exp_end_continue_f:w \__fp_exp_after_any_f:nw {{\exp_after:wN \exp_stop_f: }}}}"),
    r"Prefix: \long ; \long ; #1; same".to_string(),
    format!(r"Onestep: {door} 1+1{marks} \exp_end:"),
    format!(r"Oargs: {door} 1{marks} \exp_end:"),
    format!(r"Inner: \exp_end_continue_f:w \exp_after:wN \__fp_parse_continue:NwN \exp_after:wN \c__fp_prec_end_int \exp:w \exp_end_continue_f:w \exp_after:wN \__fp_parse_one:Nw \exp_after:wN \c__fp_prec_end_int \exp:w \__fp_parse_expand:w 1{marks}"),
    format!(r"Leaves: {two} @\__fp_parse_infix_end:N \s__fp_expr_stop"),
    format!(r"Whole: {two} "),
    "Braces: 2; 1; 2; 12; 1; 2; 2;".to_string(),
    "Step: [1][2][3]; 3; 7".to_string(),
  ] {
    assert!(xml.contains(&row), "{row}\n{xml}");
  }
  // the mark in the argument, no `\s__fp_expr_stop` after it: no door of the macro's
  let (log, _, answered) = both_ways(&repro!("l3fp_changed_names").replace(
    r"\fp_clear_variable:n{pi}\fp_set_variable:nn{pi}{3}\T{pi+1}",
    r"\T{1+1\s__fp_expr_mark\__fp_parse_infix_end:N 3}",
  ));
  assert_answered(answered, 1, "the fixture");
  assert_eq!(fatal_count(&log), 1, "{log}");
}

/// Only a canonical stored form is a value to the native path: a short block's arithmetic, a form handed on as it stands.
#[test]
fn a_stored_form_not_canonical_is_the_macros() {
  let (log, xml, answered) = both_ways(repro!("l3fp_stored_forms"));
  let xml = squeezed(&xml);
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  assert_answered(answered, 1, "the canonical row");
  let form = |blocks: &str| format!(r"\s__fp \__fp_chk:w 10{blocks}\__fp_sep: ;");
  for row in [
    format!(
      "Short: {} {}",
      form("{1}{1001}{0000}{0000}{0000}"),
      form("{1}{1000}{0}{0}{0}")
    ),
    format!("Long: {}", form("{1}{2000}{0000}{0000}{0000}")),
    format!(
      "Exponent: {} {}",
      form("{1}{1000}{0000}{0000}{0000}"),
      form("{0}{1000}{0000}{0000}{0000}")
    ),
    format!("Canonical: {}", form("{1}{2000}{0000}{0000}{0000}")),
  ] {
    assert!(xml.contains(&row), "{row}\n{xml}");
  }
}

/// An expression too deep is the macro's, whatever it then does, and the native path never recurses past `parse::MAX_DEPTH`:
/// so the macro's Fatal stays its own (`1^1^…` and a function's 2,000 arguments, which native answered; `(` at 1,000,000
/// aborted), and a long flat expression stays native, in a loop.
#[test]
fn a_deep_expression_is_the_macros() {
  let doc = |expression: String| {
    format!(
      "\\documentclass{{article}}\n\\ExplSyntaxOn\n\\cs_new_protected:Npn \\T #1 {{ [ \\fp_to_tl:n {{#1}} ] ~ }}\n\\ExplSyntaxOff\n\\begin{{document}}\n\\ExplSyntaxOn\n\\T{{{expression}}}\n\\ExplSyntaxOff\n\\end{{document}}\n"
    )
  };
  let nested =
    |open: &str, n: usize, close: &str| format!("{}1{}", open.repeat(n), close.repeat(n));
  for (expression, printed, fatals, native) in [
    (nested("(", 150, ")"), "[1]", 0, 1),
    (nested("(", 300, ")"), "[1]", 0, 0),
    (nested("-", 500, ""), "[1]", 0, 0),
    (nested("abs(", 300, ")"), "[1]", 0, 0),
    (["1"; 2000].join("^"), "", 1, 0),
    (nested("(", 20000, ")"), "", 1, 0),
    (format!("max({})", ["1"; 2000].join(",")), "", 1, 0),
    (format!("max({})", ["1"; 100].join(",")), "[1]", 0, 1),
    (["1"; 2000].join("+"), "[2000]", 0, 1),
  ] {
    let (log, xml, answered) = both_ways(&doc(expression.clone()));
    let shape = &expression[..expression.len().min(12)];
    assert_answered(answered, native, shape);
    assert_eq!(fatal_count(&log), fatals, "{shape}…: {log}");
    assert!(
      squeezed(&xml).contains(printed),
      "{shape}…: {printed}\n{xml}"
    );
  }
}

/// A tab mark the macro's run would read at `align_state` 0 ends the cell (tex.web §342), with what follows: the Errors and the
/// value are the macro's, so the native path leaves it the door, by the gullet's own count, the operand, the infix, an
/// argument and a `\csname` alike. Where the count is not 0 nothing is left: the control has a `{…}`-wrapped cell, an
/// `\edef`'s body, an argument's braces, no alignment at all. `\group_align_safe_begin:` is in the count: on the door of
/// 2026-01-19 the rows are native, and only native = raw is asserted there.
#[test]
fn a_tab_mark_that_ends_a_cell_is_the_macros() {
  let date = r"\documentclass{article}
\begin{document}
[\ExplLoaderFileDate]
\end{document}
";
  let (_, xml, _) = convert(date, true, false);
  let unguarded_door = xml.contains("[2025-11-06]");
  const MARKED: &str = r"x\FT{&1} \\";
  let siblings = repro!("l3fp_tab_mark_siblings");
  assert!(siblings.contains(MARKED));
  let row = |row: &str| siblings.replace(MARKED, row);
  for (what, tex, errors, warnings, native, left, printed) in [
    (
      "a conditional",
      repro!("l3fp_tab_mark_conditional").to_string(),
      3,
      0,
      5,
      2,
      "yes yes b xyes b",
    ),
    (
      "the value",
      repro!("l3fp_tab_mark_value").to_string(),
      6,
      0,
      8,
      3,
      "a [0] c d [3] [0] y b c [0] x",
    ),
    (
      "a macro's, and an argument's",
      repro!("l3fp_tab_mark_from_a_macro").to_string(),
      8,
      0,
      9,
      3,
      "x[0] x[0] x[2] x[0]",
    ),
    ("an operand", row(MARKED), 3, 0, 3, 1, "x[0]"),
    (
      "another character's",
      row(r"\char_set_catcode_alignment:N \| x\FT{1||0} \\"),
      3,
      0,
      3,
      1,
      "x[0]",
    ),
    (
      "in a csname",
      row(r"x\FT{\csname known&\endcsname + 1} \\"),
      0,
      0,
      0,
      1,
      "x[3]",
    ),
    (
      "in a dimexpr",
      row(r"\char_set_catcode_alignment:N \( x\FT{\dim_to_fp:n {(1pt)}} \\"),
      0,
      1,
      0,
      1,
      "x[0]",
    ),
    // (the review's implicit column ends, each native and uncounted on P2r3: `x[2]`, E=0)
    (
      "a `\\let` copy in an argument",
      row(r"x\FT{\one\amp + 1} \\"),
      3,
      0,
      2,
      1,
      "x[nan]",
    ),
    (
      "`\\cr` in an argument",
      row(r"x\FT{\one\cr + 1} \\"),
      3,
      0,
      3,
      1,
      "x[1]",
    ),
    (
      "`\\span` in an argument",
      row(r"x\FT{\one\span + 1} \\"),
      3,
      0,
      3,
      1,
      "x[1]",
    ),
    (
      "in an argument's braces, at a stray close",
      row(r"x\iffalse}\fi\FT{\one{&} + 1}\iffalse{\fi \\"),
      0,
      0,
      0,
      1,
      "x[2]",
    ),
  ] {
    let (log, xml, counts) = both_ways_counted(&tex);
    if unguarded_door {
      assert_eq!(counts, Some((native, left)), "{what}: {log}");
      assert_eq!(error_count(&log), errors, "{what}: {log}");
      assert_eq!(warning_count(&log), warnings, "{what}: {log}");
      assert!(text(&xml).contains(printed), "{what}: {printed}\n{xml}");
    }
  }
  let (log, xml, counts) = both_ways_counted(repro!("l3fp_tab_mark_stays_native"));
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  if let Some(counts) = counts {
    assert_eq!(counts, (9, 0), "the control: {log}");
  }
  assert!(
    text(&xml).contains("[1][1][1][2] [1] xy x1 [2] [1] x"),
    "{xml}"
  );
}
