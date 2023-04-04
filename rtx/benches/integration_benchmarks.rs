#[macro_use]
extern crate criterion;

use criterion::Criterion;
use rtx::core::DigestionAPI;
use rtx_core::{Core, CoreOptions};

/// Convenience function for iteratively executing rtx under a Bencher
fn benchmark_texfile(c: &mut Criterion, tex_path: &'static str) {
  let mut latexml = Core::new(CoreOptions {
    verbosity: Some(-2),
    search_paths: None,
    preload: None,
    include_comments: Some(false),
    ..CoreOptions::default()
  });
  c.bench_function(tex_path, move |b| {
    b.iter(|| latexml.convert_file(tex_path.to_string()).unwrap())
  });
}

fn bench_primes(c: &mut Criterion) { benchmark_texfile(c, "tests/digestion/primes.tex"); }

fn bench_big_equality(c: &mut Criterion) { benchmark_texfile(c, "tests/tokenize/equality.tex"); }

criterion_group!(benches, bench_primes, bench_big_equality);
criterion_main!(benches);
