//! `\href` inside `\edef`/`\xdef` must not infinite-loop.
//!
//! Root cause (2110.10227): LaTeXML defines `\href` as an expandable macro
//! whose body re-emits `\href` itself (for the `\lx@hyper@url@` constructor's
//! reversion argument). In a partial-expansion context (`\edef`/`\xdef`) that
//! re-emitted `\href` is expanded again and again — an unbounded expansion
//! loop. ems-journal.sty's `\Emsaffil` → `\build@ffil` does
//! `\xdef\ems@temp{… \href{mailto:…}{\mbox{…}} …}`, so raw-loading the class
//! (INCLUDE_STYLES=true, as ar5iv does) drove the loop to a
//! `Fatal:Timeout:PushbackLimit` / `Fatal:Stomach:Recursion`.
//!
//! Fix: mark `\href` `protected => true`. In real hyperref `\href` is a robust
//! command (`\DeclareRobustCommand`/`\protected`), so `\edef` leaves the
//! literal `\href{…}{…}` untouched. At top-level digestion (`fully_expand`)
//! protected macros still expand, so normal `\href` is unchanged. Perl LaTeXML
//! omits the flag and *hangs* on this input — this is a surpass-Perl
//! robustness win that is also faithful to real-TeX semantics.
//!
//! Dump-independent: the hyperref binding (and `\href`) is compiled in.
use latexml::converter::Converter;
use latexml_core::common::{Config, OutputFormat};

#[test]
fn href_inside_xdef_does_not_loop() {
  let _ = latexml_core::util::logger::init(log::LevelFilter::Warn);
  let cfg = Config {
    format: OutputFormat::HTML5,
    ..Config::default()
  };
  let mut c = Converter::from_config(cfg);
  c.initialize_session().expect("initialize");
  let r = c.convert("tests/cluster_regressions/href_edef_loop.tex".to_string());

  // The loop manifested as a fatal recursion/timeout abort with no result.
  assert!(
    r.result.is_some(),
    "conversion produced no result — the \\href-in-\\xdef expansion loop \
     likely re-triggered (status_code={})",
    r.status_code
  );
  assert!(
    !r.log.contains("PushbackLimit") && !r.log.contains("Infinite digestion loop"),
    "detected an infinite-expansion / infinite-digestion fatal in the log — \
     `\\href` is expanding inside `\\xdef` again (it must be protected)"
  );
  // status_code 3 == fatal; the protected `\href` keeps this well below.
  assert!(
    r.status_code < 3,
    "conversion hit a fatal (status_code={}) — expected a clean run",
    r.status_code
  );
}
