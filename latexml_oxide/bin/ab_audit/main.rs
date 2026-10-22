//! P0: one audited A/B comparator, and the per-document audit it is built from.
//!
//! * `ab_audit run --bin-a A --expect-a SHA --bin-b B --expect-b SHA --papers LIST --out DIR` converts
//!   every paper with both binaries (refusing a binary whose stamped revision is not the expected
//!   commit; `--accept-stamp <stamp>` replays a historical one, printed in the report) and compares them.
//! * `ab_audit compare A_ROOT B_ROOT -o PREFIX` compares two existing output trees.
//! * `ab_audit audit ROOT -o PREFIX` audits one tree (the seed of the end-of-queue fidelity audit).
//! * `ab_audit fmshadow ROOT -o PREFIX` compares the frontmatter observer's shadow resolution
//!   (`out.xml.fmshadow.json`, `LATEXML_FM_SHADOW=1`) with each paper's XML (`fmshadow.rs`).
//!
//! Every report is `<prefix>.tsv` (one row per paper: tier, flags, then the columns), `<prefix>.summary.txt`
//! (tallies, then the rows ranked regressions first) and `<prefix>.details/<key>.txt` per flagged paper.
//! It fails toward flagging: a missing or unparsable XML, a log without its status, a timeout is a
//! regression row. Exit code: 0 when every row is a change or the same, 1 when any row is a regression or
//! needs review (a reason), 2 refused or bad usage.

mod canon;
mod diff;
mod facts;
mod fmshadow;
mod front;
mod report;
mod runner;
mod tree;

use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
  name = "ab_audit",
  about = "Audited A/B comparison of latexml_oxide conversions"
)]
struct Cli {
  #[command(subcommand)]
  command: Command,
}

#[derive(Subcommand)]
enum Command {
  /// Convert a paper list with two binaries, then compare.
  Run(Box<RunCmd>),
  /// Compare two output trees.
  Compare {
    a_root: PathBuf,
    b_root: PathBuf,
    /// Report prefix: writes PREFIX.tsv, PREFIX.summary.txt and PREFIX.details/.
    #[arg(short = 'o', long, default_value = "ab_compare")]
    out:    PathBuf,
    /// Only these keys (one per line); default: every document directory in either tree.
    #[arg(long)]
    keys:   Option<PathBuf>,
    #[arg(short = 'j', long, default_value_t = 8)]
    jobs:   usize,
  },
  /// Compare the frontmatter observer's shadow resolution with each paper's XML (FM S2a.2).
  Fmshadow {
    root:       PathBuf,
    #[arg(short = 'o', long, default_value = "fmshadow")]
    out:        PathBuf,
    #[arg(long)]
    keys:       Option<PathBuf>,
    #[arg(short = 'j', long, default_value_t = 8)]
    jobs:       usize,
    /// Resolve again from the dump's events with this binary's resolver, not the dump's own `resolved`.
    #[arg(long)]
    re_resolve: bool,
  },
  /// Audit one output tree.
  Audit {
    root: PathBuf,
    #[arg(short = 'o', long, default_value = "ab_audit")]
    out:  PathBuf,
    #[arg(long)]
    keys: Option<PathBuf>,
    #[arg(short = 'j', long, default_value_t = 8)]
    jobs: usize,
  },
}

/// `run`'s arguments (boxed: the largest subcommand by far).
#[derive(clap::Args)]
struct RunCmd {
  #[arg(long)]
  bin_a:             PathBuf,
  /// The commit binary A must be stamped with (a short or full SHA).
  #[arg(long)]
  expect_a:          String,
  #[arg(long)]
  bin_b:             PathBuf,
  #[arg(long)]
  expect_b:          String,
  /// Accept a binary stamped with exactly this revision although it is not the expected one (a
  /// historical replay). Printed in the report header. Repeatable.
  #[arg(long)]
  accept_stamp:      Vec<String>,
  /// One paper per line: an arXiv id (resolved under --corpus) or a .zip path.
  #[arg(long)]
  papers:            PathBuf,
  #[arg(long)]
  out:               PathBuf,
  #[arg(short = 'j', long, default_value_t = 8)]
  jobs:              usize,
  /// The `taskset -c` core list.
  #[arg(long, default_value = "64-127")]
  cores:             String,
  /// The converter's own `--timeout`.
  #[arg(long, default_value_t = 240)]
  timeout:           u32,
  /// The wall-clock kill (`timeout <wall>`).
  #[arg(long, default_value_t = 300)]
  wall:              u32,
  /// `ulimit -v`, in KiB.
  #[arg(long, default_value_t = 8_912_896)]
  memory_kb:         u64,
  #[arg(long, default_value = "/data/arxmliv")]
  corpus:            PathBuf,
  /// Put first on PATH (kpsewhich, pdflatex).
  #[arg(long, default_value = "/usr/local/texlive/2025/bin/x86_64-linux")]
  texlive_bin:       PathBuf,
  /// LATEXML_DUMP_DIR for both binaries (default: inherited).
  #[arg(long)]
  dump_dir:          Option<PathBuf>,
  /// Allow A and B to be byte-identical binaries (a noise baseline).
  #[arg(long)]
  allow_same_binary: bool,
  /// Allow A and B to carry one revision stamp although their expected commits differ (a replay of
  /// binaries built in a target directory whose stamp never moved). Printed in the report header.
  #[arg(long)]
  accept_same_stamp: bool,
  /// The git checkout that resolves the expected commits (a binary older than its commit is refused).
  #[arg(long, default_value = ".")]
  repo:              PathBuf,
}

fn main() {
  let cli = Cli::parse();
  let result = match cli.command {
    Command::Run(run) => {
      let RunCmd {
        bin_a,
        expect_a,
        bin_b,
        expect_b,
        accept_stamp,
        papers,
        out,
        jobs,
        cores,
        timeout,
        wall,
        memory_kb,
        corpus,
        texlive_bin,
        dump_dir,
        allow_same_binary,
        accept_same_stamp,
        repo,
      } = *run;
      runner::run(&runner::RunArgs {
        bins: [bin_a, bin_b],
        expect: [expect_a, expect_b],
        accept: accept_stamp,
        papers,
        out,
        jobs,
        cores,
        timeout,
        wall,
        memory_kb,
        corpus,
        texlive_bin,
        dump_dir,
        allow_same_binary,
        accept_same_stamp,
        repo,
      })
    },
    Command::Compare {
      a_root,
      b_root,
      out,
      keys,
      jobs,
    } => report::run_compare(
      &a_root,
      &b_root,
      keys.as_deref(),
      &report::Output { prefix: out },
      jobs,
      &[],
    )
    .map(i32::from),
    Command::Fmshadow {
      root,
      out,
      keys,
      jobs,
      re_resolve,
    } => fmshadow::run(&root, keys.as_deref(), &out, jobs, re_resolve),
    Command::Audit { root, out, keys, jobs } => report::run_audit(
      &root,
      keys.as_deref(),
      &report::Output { prefix: out },
      jobs,
    )
    .map(i32::from),
  };
  match result {
    Ok(code) => std::process::exit(code),
    Err(e) => {
      eprintln!("ab_audit: {e}");
      std::process::exit(2);
    },
  }
}
