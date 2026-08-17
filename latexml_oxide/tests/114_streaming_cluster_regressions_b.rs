//! Forced-streaming corpus sweep for `cluster_regressions`, shard 1 of 2.
//! See `114_streaming_cluster_regressions.rs` (shard 0) and `streaming_sweep/mod.rs`:
//! the 174-fixture suite is split across two binaries so neither process accretes
//! enough libxml2 residue to trip the RSS fuse.

mod streaming_sweep;
use streaming_sweep::sweep_dir_shard;

#[test]
fn streaming_matches_eager_on_cluster_regressions_shard1() {
  sweep_dir_shard("tests/cluster_regressions", 1, 2);
}
