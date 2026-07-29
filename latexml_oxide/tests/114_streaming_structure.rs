//! Forced-streaming corpus sweep — the wide version of `113_streaming_core`.
//! Every fixture converts twice (eager, and streaming with an aggressive
//! 3-box budget); the XML must be byte-identical and the error counts equal.
//! One suite per test binary — see `streaming_sweep/mod.rs` for why.

mod streaming_sweep;
use streaming_sweep::sweep_dir;

#[test]
fn streaming_matches_eager_on_structure() { sweep_dir("tests/structure"); }
