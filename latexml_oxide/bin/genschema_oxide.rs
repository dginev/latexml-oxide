//! `genschema_oxide` — RelaxNG → schema.tex generator.
//!
//! Native Rust replacement for the Perl `tools/genschema` driver.
//! Walks an RNG file with [`latexml_core::common::relaxng`] (port of
//! `LaTeXML::Common::Model::RelaxNG.pm`) and prints the resulting
//! `\schemamodule{}` / `\patterndef{}` / `\elementdef{}` /
//! `\attrdef{}` LaTeX manual.tex consumed by `latexmlman.sty`.
//!
//! Usage:
//!
//! ```text
//! genschema_oxide <RNG> [--output FILE] [--path DIR ...] [--module-abstract]
//!                       [--no-skip-svg] [--no-skip-aria] [--no-skip-xhtml]
//! ```

use clap::Parser;
use latexml_core::common::relaxng::{
  scan::scan_external,
  simplify::simplify_top,
  tex::{document_modules, Options as TexOptions},
  Relaxng,
};
use std::path::{Path, PathBuf};
use std::process;

/// Use mimalloc for parity with the other oxide binaries.
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[derive(Parser, Debug)]
#[command(
  about = "Generate LaTeX manual.tex documentation from a RelaxNG schema."
)]
struct Cli {
  /// Path to the .rng entry-point. Resolved against `--path` if not
  /// found at the bare path.
  schema: PathBuf,

  /// Output file. Defaults to stdout.
  #[arg(short, long, value_name = "FILE")]
  output: Option<PathBuf>,

  /// Search paths for `<include href="..."/>` resolution. Repeatable.
  #[arg(long = "path", value_name = "DIR")]
  paths: Vec<PathBuf>,

  /// Don't skip the SVG schema branch (default: skip).
  #[arg(long)]
  no_skip_svg: bool,
  /// Don't skip the ARIA schema branch (default: skip).
  #[arg(long)]
  no_skip_aria: bool,
  /// Don't skip the XHTML schema branch (default: skip).
  #[arg(long)]
  no_skip_xhtml: bool,

  /// Lift the first `\patterndef` doc-arg per `\schemamodule` block
  /// into a top-level `\moduleabstract{...}` macro. RNC `## comments`
  /// flow into doc annotations on the first `<define>` of each module
  /// (per the trang convention); this lift promotes them to module-
  /// level so the schema-doc renderer can show them as a per-module
  /// narrative aside rather than inline with one specific pattern.
  #[arg(long)]
  module_abstract: bool,
}

fn main() {
  let cli = Cli::parse();

  let schema_path = match resolve_schema_path(&cli.schema, &cli.paths) {
    Some(p) => p,
    None => {
      eprintln!(
        "genschema_oxide: schema file not found: {}",
        cli.schema.display()
      );
      process::exit(2);
    },
  };

  let mut search_paths: Vec<&Path> = Vec::with_capacity(cli.paths.len() + 1);
  let dir = schema_path.parent().unwrap_or_else(|| Path::new("."));
  search_paths.push(dir);
  for p in &cli.paths {
    search_paths.push(p);
  }

  let schema_name = schema_path
    .file_stem()
    .and_then(|s| s.to_str())
    .unwrap_or("schema")
    .to_string();
  let mut rng = Relaxng::new(schema_name);

  let raw = match scan_external(
    &mut rng,
    schema_path.file_name().and_then(|f| f.to_str()).unwrap_or(""),
    None,
    &search_paths,
  ) {
    Ok(p) => p,
    Err(e) => {
      eprintln!("genschema_oxide: scan failed: {}", e);
      process::exit(1);
    },
  };
  let _ = simplify_top(&mut rng, raw);

  let tex_opts = TexOptions {
    skip_svg:   !cli.no_skip_svg,
    skip_aria:  !cli.no_skip_aria,
    skip_xhtml: !cli.no_skip_xhtml,
  };
  let mut docs = document_modules(&rng, tex_opts);
  if cli.module_abstract {
    docs = lift_module_abstract(&docs);
  }

  match cli.output {
    Some(out) => {
      if let Err(e) = std::fs::write(&out, &docs) {
        eprintln!("genschema_oxide: write {}: {}", out.display(), e);
        process::exit(1);
      }
    },
    None => print!("{}", docs),
  }
}

fn resolve_schema_path(schema: &Path, paths: &[PathBuf]) -> Option<PathBuf> {
  if schema.is_file() {
    return Some(schema.to_path_buf());
  }
  for dir in paths {
    let cand = dir.join(schema);
    if cand.is_file() {
      return Some(cand);
    }
  }
  None
}

/// Promote the first `\patterndef`'s doc-arg of each `\schemamodule`
/// to a top-level `\moduleabstract{...}` macro.
///
/// Mirrors the Perl regex in `tools/genschema`'s post-processing step:
/// the `## comments` at the head of each RNC file land — via trang's
/// `<a:documentation>` and `RelaxNG.pm`'s `doc` op — as the doc-arg of
/// whichever `<define>` happens to come first in the module. This lift
/// rewrites the emitted block so the doc reads as a module-level
/// narrative rather than as documentation attached to one specific
/// pattern.
fn lift_module_abstract(tex: &str) -> String {
  let re = regex::Regex::new(
    r#"(\\begin\{schemamodule\}\{[^}]+\}\s*\n)\\patterndef\{([^}]+)\}\{([^}]*)\}\{"#,
  )
  .expect("static regex compiles");
  re.replace_all(tex, |caps: &regex::Captures| {
    let head = &caps[1];
    let pname = &caps[2];
    // Match Perl's `if ($doc =~ /\S/)` — promote whenever any
    // non-whitespace exists, but DON'T trim the doc itself: the
    // trailing newline emitted by `Pattern::Doc` is part of the
    // canonical schema.tex shape.
    let doc = &caps[3];
    let has_content = doc.chars().any(|c| !c.is_whitespace());
    if has_content {
      format!(
        "{}\\moduleabstract{{{}}}\n\\patterndef{{{}}}{{}}{{",
        head, doc, pname
      )
    } else {
      format!("{}\\patterndef{{{}}}{{}}{{", head, pname)
    }
  })
  .into_owned()
}
