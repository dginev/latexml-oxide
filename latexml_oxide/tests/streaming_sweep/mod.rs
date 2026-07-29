//! Shared driver for the forced-streaming corpus sweep
//! (`114_streaming_*.rs` — one test BINARY per fixture suite, deliberately).
//!
//! One process cannot sweep every suite: each conversion leaves ~24 MB of
//! unreclaimable libxml2 parser-dictionary residue (see
//! `reset_thread_engine`'s docs — the engine itself IS freed), so ~500
//! conversions accrete ~12 GB of dead RSS and the process-wide fuse then
//! fails whichever conversions are in flight with phantom
//! `Timeout/MemoryBudget` fatals (the cargo-test-rss-fuse gotcha, at sweep
//! scale). Splitting per suite keeps each process a few GB under the cap.
#![allow(dead_code)]

use latexml::converter::Converter;
use latexml_core::common::{Config, OutputFormat};

/// One conversion in a fresh thread. Returns (xml, error_count).
pub fn convert(source: &str, streaming: Option<usize>) -> (String, usize) {
  let source = source.to_string();
  std::thread::Builder::new()
    .stack_size(64 * 1024 * 1024)
    .spawn(move || {
      latexml::util::test::init_test_rss_cap();
      let _ = latexml_core::util::logger::init(log::LevelFilter::Warn);
      let cfg = Config {
        format: OutputFormat::XML,
        streaming,
        // The contrib dispatcher the binaries install (see cluster/mod.rs):
        // without it, contrib-provided bindings resolve to nothing here while
        // working in production, and a fixture needing one degrades — in
        // MODE-DEPENDENT shapes, which reads as a phantom streaming
        // divergence (witness omnibus_natbib_bbl_sideload: 101 errors, an
        // empty eager doc, a partial streamed one).
        extra_bindings_dispatch: Some(std::rc::Rc::new(latexml_contrib::dispatch)),
        ..Config::default()
      };
      let mut c = Converter::from_config(cfg);
      c.initialize_session().expect("initialize");
      let r = c.convert(source.clone());
      let outcome = (
        r.result.unwrap_or_default(),
        latexml::util::test::error_count(&r.log),
      );
      // Free this thread's engine before it exits: `#[thread_local]` statics
      // run no destructors, so WITHOUT this every conversion leaks its engine
      // (~110 MB) and a few hundred sweep conversions trip the process-wide
      // RSS fuse — surfacing as phantom `Timeout/MemoryBudget` fatals in
      // whichever conversions are in flight (the cargo-test-rss-fuse gotcha).
      latexml_core::reset_thread_engine();
      outcome
    })
    .expect("spawn conversion thread")
    .join()
    .expect("conversion thread panicked")
}

pub fn sweep_dir(dir: &str) {
  let mut fixtures: Vec<_> = std::fs::read_dir(dir)
    .unwrap_or_else(|e| panic!("cannot read {dir}: {e}"))
    .filter_map(|e| e.ok().map(|e| e.path()))
    .filter(|p| p.extension().is_some_and(|x| x == "tex"))
    .collect();
  fixtures.sort();
  assert!(
    !fixtures.is_empty(),
    "{dir}: no fixtures found — wrong path?"
  );
  let mut divergent: Vec<String> = Vec::new();
  for path in fixtures {
    let src = path.to_string_lossy().into_owned();
    let (eager_xml, eager_errs) = convert(&src, None);
    let (streamed_xml, streamed_errs) = convert(&src, Some(3));
    if eager_errs != streamed_errs {
      divergent.push(format!(
        "{src}: error count diverges (eager {eager_errs}, streamed {streamed_errs})"
      ));
      continue;
    }
    if eager_xml != streamed_xml {
      let byte = eager_xml
        .bytes()
        .zip(streamed_xml.bytes())
        .position(|(a, b)| a != b)
        .unwrap_or_else(|| eager_xml.len().min(streamed_xml.len()));
      let lo = byte.saturating_sub(120);
      divergent.push(format!(
        "{src}: output diverges at byte {byte}\n  eager   …{}…\n  streamed…{}…",
        eager_xml[lo..(byte + 120).min(eager_xml.len())].replace('\n', "\\n"),
        streamed_xml[lo..(byte + 120).min(streamed_xml.len())].replace('\n', "\\n"),
      ));
    }
  }
  assert!(
    divergent.is_empty(),
    "{dir}: {} fixture(s) diverge under streaming:\n{}",
    divergent.len(),
    divergent.join("\n")
  );
}
