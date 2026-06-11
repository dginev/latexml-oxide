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
//!                       [--ns prefix=URI ...] [--no-latexml-defaults]
//!                       [--no-skip-svg] [--no-skip-aria] [--no-skip-xhtml]
//! ```

use std::{
  path::{Path, PathBuf},
  process,
};

use clap::Parser;
use latexml_core::common::relaxng::{
  Relaxng,
  scan::scan_external,
  simplify::simplify_top,
  tex::{Options as TexOptions, document_modules},
};

/// Use mimalloc for parity with the other oxide binaries.
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[derive(Parser, Debug)]
#[command(about = "Generate LaTeX manual.tex documentation from a RelaxNG schema.")]
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
  no_skip_svg:   bool,
  /// Don't skip the ARIA schema branch (default: skip).
  #[arg(long)]
  no_skip_aria:  bool,
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

  /// Pre-register a namespace prefix → URI binding before scanning.
  /// Repeatable. Useful when a `.rnc` declares `default namespace =
  /// "..."` without a prefix: trang strips the binding to the default
  /// `<grammar ns="..."/>`, leaving no `xmlns:` for the scanner to
  /// pick up, so element names render as the synthesized fallback
  /// `namespaceN:foo`. `--ns tei=http://www.tei-c.org/ns/1.0` makes
  /// them render as `tei:foo` instead.
  #[arg(long = "ns", value_name = "PREFIX=URI", value_parser = parse_ns_arg)]
  ns: Vec<(String, String)>,

  /// Skip the built-in LaTeXML namespace defaults (`xml`, `ltx`,
  /// `svg`, `xlink`, `m`, `xhtml`). Use when documenting a non-LaTeXML
  /// schema; pair with `--ns` to declare your own conventions.
  #[arg(long)]
  no_latexml_defaults: bool,
}

fn parse_ns_arg(arg: &str) -> Result<(String, String), String> {
  let (prefix, uri) = arg
    .split_once('=')
    .ok_or_else(|| format!("expected `prefix=URI`, got `{}`", arg))?;
  if prefix.is_empty() {
    return Err("prefix may not be empty".into());
  }
  if uri.is_empty() {
    return Err("URI may not be empty".into());
  }
  Ok((prefix.to_string(), uri.to_string()))
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
  if !cli.no_latexml_defaults {
    rng.with_latexml_defaults();
  }
  for (prefix, uri) in &cli.ns {
    rng.register_namespace(prefix, uri);
  }

  let raw = match scan_external(
    &mut rng,
    schema_path
      .file_name()
      .and_then(|f| f.to_str())
      .unwrap_or(""),
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

  // Auto-elide the schema's primary namespace prefix from rendered
  // display names — `default namespace = "http://…"` in the master
  // RNG resolves to a prefix in `document_namespaces`; that prefix
  // becomes contextually obvious for every element / attribute name
  // on the site, so we drop it (`xhtml:div` → `div`, `ltx:para` →
  // `para`, `m:math` → `math`).
  rng.auto_strip_primary_namespace();

  let tex_opts = TexOptions {
    skip_svg:   !cli.no_skip_svg,
    skip_aria:  !cli.no_skip_aria,
    skip_xhtml: !cli.no_skip_xhtml,
  };
  let mut docs = document_modules(&rng, tex_opts);
  if cli.module_abstract {
    docs = hoist_document_abstract(&docs);
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
/// Scan a TeX group starting at the byte offset of its opening brace;
/// return the contents and the offset just past the closing brace.
/// Needed because doc-args routinely carry nested `\code{...}` groups,
/// which a `[^}]*` regex would truncate at the first inner brace.
fn read_tex_group(tex: &str, open: usize) -> Option<(&str, usize)> {
  let bytes = tex.as_bytes();
  if bytes.get(open) != Some(&b'{') {
    return None;
  }
  let mut depth = 0usize;
  for (i, &b) in bytes.iter().enumerate().skip(open) {
    match b {
      b'{' => depth += 1,
      b'}' => {
        depth -= 1;
        if depth == 0 {
          return Some((&tex[open + 1..i], i + 1));
        }
      },
      _ => {},
    }
  }
  None
}

/// The driver module is the first `\begin{schemamodule}` in the
/// emitted TeX, and the `## comments` at the head of the driver file
/// are the schema's own front-page overview. Left in place they would
/// be sectioned away with the rest of the module when the postprocessor
/// splits at `\section` — landing on the module page while the
/// document's `index.html` shows only the title and table of contents.
/// Hoist them: when the driver's first patterndef carries a
/// multi-paragraph doc-arg, move the whole narrative to document level
/// (before the first `\begin{schemamodule}`) as `\documentabstract`,
/// leaving the patterndef's doc empty. Single-paragraph docs stay put —
/// those are per-pattern commentary (e.g. `Inline.model` in
/// LaTeXML.rnc), not a front-page narrative.
fn hoist_document_abstract(tex: &str) -> String {
  let Some(module_start) = tex.find("\\begin{schemamodule}") else {
    return tex.to_owned();
  };
  let Some(pdef_rel) = tex[module_start..].find("\\patterndef{") else {
    return tex.to_owned();
  };
  let pdef_start = module_start + pdef_rel;
  // Don't reach into a later module: the patterndef must belong to
  // the driver module block.
  if let Some(next_module_rel) = tex[module_start + 1..].find("\\begin{schemamodule}")
    && module_start + 1 + next_module_rel < pdef_start
  {
    return tex.to_owned();
  }
  let name_open = pdef_start + "\\patterndef".len();
  let Some((_name, doc_open)) = read_tex_group(tex, name_open) else {
    return tex.to_owned();
  };
  let Some((doc, doc_end)) = read_tex_group(tex, doc_open) else {
    return tex.to_owned();
  };
  let paragraphs: Vec<&str> = doc
    .split("\n\n")
    .map(str::trim)
    .filter(|p| !p.is_empty())
    .collect();
  if paragraphs.len() < 2 {
    return tex.to_owned();
  }
  // All paragraphs but the last form the document narrative; the
  // final paragraph is the first define's own documentation and
  // stays on the patterndef (mirroring the module-lift convention,
  // where trailing paragraphs document the define they precede).
  let (narrative, pattern_doc) = paragraphs.split_at(paragraphs.len() - 1);
  let mut out = String::with_capacity(tex.len() + 32);
  out.push_str(&tex[..module_start]);
  out.push_str("\\documentabstract{");
  out.push_str(&narrative.join("\n\n"));
  out.push_str("}\n");
  out.push_str(&tex[module_start..doc_open]);
  out.push('{');
  out.push_str(pattern_doc[0]);
  out.push('}');
  out.push_str(&tex[doc_end..]);
  out
}

/// Mirrors the Perl regex in `tools/genschema`'s post-processing step:
/// the `## comments` at the head of each RNC file land — via trang's
/// `<a:documentation>` and `RelaxNG.pm`'s `doc` op — as the doc-arg of
/// whichever `<define>` happens to come first in the module. This lift
/// rewrites the emitted block so the doc reads as a module-level
/// narrative rather than as documentation attached to one specific
/// pattern.
fn lift_module_abstract(tex: &str) -> String {
  // Unified per-module rendering — no kind subsections; defs flow
  // into a single `description` env after the optional preamble.
  // Structure:
  //
  //   \begin{schemamodule}{NAME}
  //   \par\noindent\textit{Includes:} …            (optional preamble)
  //   \begin{description}
  //   \patterndef{…}{DOC}{…}        ← only this is liftable
  //
  // Promote the first patterndef's DOC up to the module-section
  // level (above the description-list opener) so it renders as a
  // per-module narrative aside, not as part of one specific pattern.
  // The patterndef's doc-arg then becomes empty.
  //
  // **Why only patterndef.** A `## doc` placed *before* a define in
  // the .rnc lands inside `<define>` (sibling of `<element>`) — that
  // shape is what produces a `\patterndef` with a doc-arg, and the
  // author's intent is typically a module-level narrative.
  // A `## doc` placed *inside* `define = ## … element …` lands
  // inside `<element>` instead, the simplify singleton-shortcut
  // hoists the element out of the define, and `\elementdef` ends up
  // carrying that doc. Those docs are element-specific (e.g. "The
  // document root." on `\elementdef{document}` in LaTeXML-structure)
  // — lifting them would silently steal the per-element commentary.
  let re = regex::Regex::new(concat!(
    r"(\\begin\{schemamodule\}\{[^}]+\}\n)",
    r"((?:\\par\\noindent[^\n]*\n)*)",
    r"(\\begin\{description\}\n)",
    r"\\patterndef\{([^}]+)\}\{([^}]*)\}\{",
  ))
  .expect("static regex compiles");
  re.replace_all(tex, |caps: &regex::Captures| {
    let module_head = &caps[1];
    let preamble = &caps[2];
    let desc_open = &caps[3];
    let pname = &caps[4];
    // Match Perl's `if ($doc =~ /\S/)` — promote whenever any
    // non-whitespace exists, but DON'T trim the doc itself: the
    // trailing newline emitted by `Pattern::Doc` is part of the
    // canonical schema.tex shape.
    let doc = &caps[5];
    // Split the doc-arg into paragraphs (`extract_docs` separates
    // adjacent `<a:documentation>` blocks with `\n\n`). The lift
    // rule:
    //
    // * 0 paragraphs: nothing to do.
    // * 1 paragraph: leave it on the patterndef. A single `## doc` on a `<define>` is per-pattern
    //   documentation (e.g. "Combined model for inline content." annotates `Inline.model` in
    //   LaTeXML.rnc) — lifting it would steal the per-pattern commentary and present it as a module
    //   narrative.
    // * ≥ 2 paragraphs: lift the FIRST as the module narrative, keep the rest on the patterndef.
    //   The convention: a `## comment` block at the top of a file (separated from the next block by
    //   a blank line) is the module narrative; subsequent blocks document the first define.
    let paragraphs: Vec<&str> = doc
      .split("\n\n")
      .map(str::trim)
      .filter(|p| !p.is_empty())
      .collect();
    if paragraphs.len() >= 2 {
      let module_para = paragraphs[0];
      let pattern_doc = paragraphs[1..].join("\n\n");
      format!(
        "{}{}\\moduleabstract{{{}}}\n{}\\patterndef{{{}}}{{{}\n}}{{",
        module_head, preamble, module_para, desc_open, pname, pattern_doc
      )
    } else {
      format!(
        "{}{}{}\\patterndef{{{}}}{{{}}}{{",
        module_head, preamble, desc_open, pname, doc
      )
    }
  })
  .into_owned()
}
