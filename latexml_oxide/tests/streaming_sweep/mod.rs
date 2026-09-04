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

/// A suite's binding dispatcher. A plain `fn` pointer, not an `Rc`, because it
/// crosses into the conversion thread — the `Rc` is built on the far side.
pub type DispatchFn = fn(&str) -> Option<latexml_core::common::error::Result<()>>;

/// One conversion in a fresh thread. Returns (xml, error_count).
pub fn convert(source: &str, streaming: Option<usize>) -> (String, usize) {
  convert_with(source, streaming, latexml_contrib::dispatch)
}

/// `convert`, with the suite's own binding dispatcher. A fixture whose
/// bindings are a TEST helper rather than a contrib entry (grouping's
/// `scopemacro.latexml`) degrades under the plain contrib dispatcher, and a
/// degraded fixture proves nothing about streaming.
pub fn convert_with(
  source: &str,
  streaming: Option<usize>,
  dispatch: DispatchFn,
) -> (String, usize) {
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
        extra_bindings_dispatch: Some(std::rc::Rc::new(dispatch)),
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

pub fn sweep_dir(dir: &str) { sweep_dir_with(dir, latexml_contrib::dispatch) }

pub fn sweep_dir_with(dir: &str, dispatch: DispatchFn) { sweep_dir_shard_with(dir, 0, 1, dispatch) }

/// As [`sweep_dir`], but sweeps only the fixtures at `index % nshards == shard`
/// (in sorted order). Splitting an over-large suite across several test BINARIES
/// keeps each process's cumulative libxml2 residue well under the RSS fuse (see the
/// module docs). `cluster_regressions` (174 fixtures ≈ 8.3 GB residue) needs this and
/// is swept in two shards; the smaller suites call [`sweep_dir`] (shard 0 of 1 = all).
pub fn sweep_dir_shard(dir: &str, shard: usize, nshards: usize) {
  sweep_dir_shard_with(dir, shard, nshards, latexml_contrib::dispatch)
}

/// Fixtures that should be excluded from the generic streaming sweep.
///
/// These fall into two categories:
/// 1. Intentional error fixtures designed to verify compiler error handling,
///    diagnostic recovery, or `makeError` markers in dedicated unit tests
///    (e.g., `INTENTIONALLY_FAILING` or `convert_expecting_errors`). Running
///    them here spams stderr with expected errors and tests nothing about
///    streaming equivalence.
/// 2. Fixtures requiring non-default CLI flags or custom preloads (such as
///    `--includestyles` or `ar5iv.sty`) not provided by the generic streaming
///    sweep runner, causing intentional undefined macros/classes if run plain.
const EXCLUDED_FIXTURES: &[&str] = &[
  // tests/structure: intentional error fixture for makeError (<ltx:ERROR class='undefined'>)
  "undefined_env.tex",
  // tests/digestion: deliberate malformed read content in exists.data (unbalanced brace)
  "io.tex",
  // tests/expansion: deliberate self-recursion error (\def\cs{\protect\cs})
  "protect_self_ref.tex",
  // tests/cluster_regressions: intentional EOF truncation error while scanning \url
  "url_eof_no_panic.tex",
  // tests/cluster_regressions: intentional unknown mathversion error
  "mathversion_unknown_version_errors.tex",
  // tests/cluster_regressions: tests class exclusion without rawclasses (\subdirclsmarker undefined)
  "subdir_cls_not_rawloaded.tex",
  // tests/cluster_regressions: requires --includestyles / ar5iv.sty preload for subdirdispatch
  "subdir_sty_not_shadowed.tex",
];

pub fn sweep_dir_shard_with(dir: &str, shard: usize, nshards: usize, dispatch: DispatchFn) {
  let mut fixtures: Vec<_> = std::fs::read_dir(dir)
    .unwrap_or_else(|e| panic!("cannot read {dir}: {e}"))
    .filter_map(|e| e.ok().map(|e| e.path()))
    .filter(|p| {
      p.extension().is_some_and(|x| x == "tex")
        && p
          .file_name()
          .and_then(|n| n.to_str())
          .is_none_or(|name| !EXCLUDED_FIXTURES.contains(&name))
    })
    .collect();
  fixtures.sort();
  assert!(
    !fixtures.is_empty(),
    "{dir}: no fixtures found — wrong path?"
  );
  let mut divergent: Vec<String> = Vec::new();
  for (i, path) in fixtures.into_iter().enumerate() {
    if i % nshards != shard {
      continue;
    }
    let src = path.to_string_lossy().into_owned();
    let (eager_xml, eager_errs) = convert_with(&src, None, dispatch);
    let (streamed_xml, streamed_errs) = convert_with(&src, Some(3), dispatch);
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
