/// Engine preset helpers used by both the production
/// `latexmlmath_oxide` binary and by the test harness. Always
/// compiled — no test-only dependencies.
pub mod preset;

/// Integration-test harness. Gated behind the `test-utils` feature
/// (default on) so the distribution build (`cargo build
/// --no-default-features --profile maxperf --bin latexml_oxide`)
/// drops `glob` / `phf` from the dependency graph (audit DEP-02).
#[cfg(feature = "test-utils")]
#[macro_use]
pub mod test;
