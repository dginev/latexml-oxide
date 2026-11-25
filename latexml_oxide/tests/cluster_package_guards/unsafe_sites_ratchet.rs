//! Issue #937 Phase 0: the `unsafe` sites of every `latexml_*` crate, shipped and test code alike, may only fall. A
//! site is an `unsafe {` block, an `unsafe impl` or an `unsafe fn`, read from the lexed source (comments and literals
//! are no code); its ledger is `docs/release/SAFETY.md`. A phase that removes sites lowers its file's pin here.
use std::{collections::BTreeMap, fs};

use regex::Regex;

use crate::perfect_kernel_batche0::{assert_debts, lex_rust_with_tests, repo_path, rust_files};

/// The sites per file (issue #937's inventory, `rg -n 'unsafe\s*\{|unsafe impl|unsafe fn' latexml_* -g '*.rs'`, comment
/// lines excluded: 106 at its Phase 0 base; Phase 1's first Unix syscalls to `rustix`/`std` took it to 80).
const UNSAFE_SITES: &[(&str, usize)] = &[
  ("latexml_contrib/src/script_bindings/engine.rs", 1),
  ("latexml_contrib/src/script_bindings/mod.rs", 3),
  ("latexml_contrib/src/script_bindings/tests.rs", 3),
  ("latexml_core/src/common/arena.rs", 1),
  ("latexml_core/src/common/store.rs", 2),
  ("latexml_core/src/runtime_bindings_reentrancy_model.rs", 1),
  ("latexml_core/src/state.rs", 1),
  ("latexml_core/src/util/logger.rs", 2),
  ("latexml_core/src/util/pathname.rs", 1),
  ("latexml_core/src/watchdog.rs", 18),
  ("latexml_oxide/bin/cortex_worker/exit_report.rs", 4),
  ("latexml_oxide/bin/cortex_worker.rs", 4),
  ("latexml_oxide/bin/latexml_oxide.rs", 2),
  ("latexml_oxide/src/core_interface.rs", 3),
  ("latexml_oxide/src/heap_trim.rs", 2),
  ("latexml_oxide/src/identity.rs", 2),
  ("latexml_oxide/src/lsp_server/unix.rs", 1),
  ("latexml_oxide/src/post.rs", 1),
  ("latexml_oxide/src/streaming_restart.rs", 1),
  ("latexml_oxide/src/util/test.rs", 5),
  ("latexml_oxide/tests/112_fragment_yield.rs", 1),
  ("latexml_oxide/tests/117_post_2gib_handoff.rs", 4),
  ("latexml_oxide/tests/118_streaming_split_parity.rs", 7),
  (
    "latexml_oxide/tests/cluster_package_guards/perfect_kernel_batch106t.rs",
    2,
  ),
  ("latexml_oxide/tests/cortex_worker_exit_reports.rs", 3),
  ("latexml_post/src/graphics_cache.rs", 1),
  ("latexml_post/src/graphics.rs", 1),
  ("latexml_post/src/test_env.rs", 3),
  ("latexml_post/src/xslt.rs", 3),
];

#[test]
fn unsafe_sites_only_fall() {
  const WHY: &str = "A new `unsafe` needs a safe API to be missing (`rustix`, `std`): use the safe one, or record the \
                     site and its contract in docs/release/SAFETY.md (issue #937).";
  // Every `unsafe` keyword in code (comments and literals are lexed away): blocks, `impl`, `fn`, `trait`,
  // `unsafe extern "C" { … }` (whose `safe fn` items need no block at the call) and `#[unsafe(…)]` attributes.
  let site = Regex::new(r"\bunsafe\b").expect("the regex");
  let mut counts = BTreeMap::new();
  let mut crates: Vec<String> = fs::read_dir(repo_path(""))
    .expect("the repository")
    .filter_map(|entry| {
      let name = entry.ok()?.file_name().to_string_lossy().into_owned();
      (name.starts_with("latexml_") && repo_path(&name).is_dir()).then_some(name)
    })
    .collect();
  crates.sort();
  for krate in crates {
    for file in rust_files(&krate) {
      let text = fs::read_to_string(repo_path(&file)).unwrap_or_else(|e| panic!("{file}: {e}"));
      let n = site.find_iter(&lex_rust_with_tests(&text).code).count();
      if n > 0 {
        counts.insert(file, n);
      }
    }
  }
  assert_debts("the unsafe sites", UNSAFE_SITES, &counts, WHY);
}
