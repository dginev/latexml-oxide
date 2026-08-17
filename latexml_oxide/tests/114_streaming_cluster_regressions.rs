//! Forced-streaming corpus sweep — the wide version of `113_streaming_core`.
//! Every fixture converts twice (eager, and streaming with an aggressive
//! 3-box budget); the XML must be byte-identical and the error counts equal.
//! One suite per test binary — see `streaming_sweep/mod.rs` for why.
//!
//! `cluster_regressions` (174 fixtures) accretes ~8.3 GB of libxml2 residue in one
//! process — at the RSS fuse. It is swept in TWO shards across two binaries (this is
//! shard 0; `_b` is shard 1) so each process stays well under the cap.

mod streaming_sweep;
use streaming_sweep::sweep_dir_shard;

#[test]
fn streaming_matches_eager_on_cluster_regressions_shard0() {
  sweep_dir_shard("tests/cluster_regressions", 0, 2);
}
