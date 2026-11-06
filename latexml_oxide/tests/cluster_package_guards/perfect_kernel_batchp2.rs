//! The native `\__fp_parse:n` (`latexml_engine/src/l3fp_native`, OXIDIZED_DESIGN_DIVERGENCES #973): each fixture converted
//! with the native path on and off (`l3fp_native::set_native_override`) gives byte-identical XML and the same Errors and
//! Warnings, whatever it does; what the arithmetic must print is held against pdflatex's own answers
//! (`kit/l3fp_oracle.sh` at 100,000 operations per family; a slice of it here).
use latexml_engine::l3fp_native::{set_check_override, set_native_override};

use super::perfect_kernel_batch46::{error_count, warning_count};

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

fn convert(tex: &str, native: bool, check: bool) -> (String, String) {
  let (log, xml, ()) = latexml::util::test::convert_with_setup_then(
    tex,
    Some(PRELOAD),
    move || {
      set_native_override(Some(native));
      set_check_override(Some(check));
    },
    |_| (),
  );
  (log, xml)
}

/// The XML with each run of white space as one space, as the printed values are compared.
fn squeezed(xml: &str) -> String { xml.split_whitespace().collect::<Vec<_>>().join(" ") }

/// Native on and off give the same document and the same report, which is returned with the XML.
fn both_ways(tex: &str) -> (String, String) {
  let (log_on, on) = convert(tex, true, false);
  let (log_off, off) = convert(tex, false, false);
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
  assert_eq!(on, off, "native and raw must give byte-identical XML");
  (log_on, on)
}

/// Exponent limits, signed zeros, ties in the reader and in `+ - * /`, values next to multiples of pi/4, huge and tiny
/// angles, 16-digit round-offs: pdflatex's text. Under the self-check, every native answer is compared with the raw macro's.
#[test]
fn the_hard_values_are_pdflatexs() {
  let (log, xml) = both_ways(repro!("l3fp_hard_cases"));
  let xml = squeezed(&xml);
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  for printed in [
    "Zeros: 0; -0; -0; 0; 0; -0; -0; 0; 0; 0; -0; 0; -0; 0; -0; -0; 0; 0;",
    "Infinities: inf; -inf; inf; -inf; -inf; 0; inf; 0; inf; inf; 1.570796326794897; 0; 0.7853981633974483;",
    "Limits: inf; -inf; 1e-9999; 9.999999999999999e9999; 9e9999; 1e-10000; 1e-10000; 1e9999; 4.270176462662952e9999;",
    "Ties: 1; 1.000000000000002; 1.000000000000001; 1e16; 9999999999999998; 1; 1.000000000000002; 1; 0.9999999999999998;",
    "2.384626433832795e-16; -4.76925286766559e-16; 1; -3.807686783083602e-16; -2626266436731868; -0.3723761236612767;",
  ] {
    assert!(xml.contains(printed), "{printed}\n{xml}");
  }
  let (log, _) = convert(repro!("l3fp_hard_cases"), true, true);
  assert_eq!(error_count(&log), 0, "the self-check: {log}");
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
  let (log, xml) = convert(&tex, true, false);
  assert_eq!(error_count(&log), 0, "{log}");
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
  let (log, xml) = both_ways(repro!("l3fp_fallback_shapes"));
  let xml = squeezed(&xml);
  assert_eq!(error_count(&log), 37, "{log}");
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
  let (log, xml) = both_ways(repro!("l3fp_expansion_order"));
  let xml = squeezed(&xml);
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
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
  let (log, xml) = both_ways(repro!("l3fp_in_a_title"));
  assert_eq!(error_count(&log), 0, "{log}");
  assert!(xml.contains("<title>Value 2.5 of it</title>"), "{xml}");
  assert_eq!(xml.matches("2.5 of it").count(), 1, "{xml}");
}
