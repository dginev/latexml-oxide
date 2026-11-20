#[macro_use]
extern crate latexml_core;
use std::{env, process};

use libxml::tree::SaveOptions;

/// Use mimalloc to avoid glibc arena contention in multi-process workloads.
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use latexml::util::preset::{lex_single_tex_formula, new_test_engine};
use latexml_core::{common::error::Result, state};
use latexml_math_parser::*;

const USAGE: &str = "\
latexmlmath_oxide — convert a single TeX formula (a LaTeXML reimplementation in Rust)

Usage: latexmlmath_oxide [OPTIONS] '<formula>'

Emits the math element itself — no document wrapper. With no conversion option,
the default is Presentation MathML on stdout (Perl `latexmlmath` parity).

Options:
      --pmml, --presentationmathml  Presentation MathML (the default)
      --cmml, --contentmathml       Content MathML
      --xmath, --XMath              LaTeXML's internal <XMath> parse tree
      --unicodemath[=FILE]          UnicodeMath, a plain Unicode string (printed last;
                                    written to FILE instead, `-` is stdout)
  -q, --quiet                       Only warnings and errors; no progress/lexeme notes
  -V, --version                     Print version
  -h, --help                        Print this help

Passing both --pmml and --cmml emits parallel markup: Content MathML annotated
into the Presentation primary, in one <m:semantics>.

Examples:
  latexmlmath_oxide '1+1=2'
  latexmlmath_oxide 'a^2+b^2=c^2'
  latexmlmath_oxide --cmml '\\frac{a}{b}+c'
  latexmlmath_oxide --unicodemath '\\alpha_i\\leq x^2'

Convert whole documents with `latexml_oxide` instead.";

const MATHML_NS: &str = "http://www.w3.org/1998/Math/MathML";
const LATEXML_NS: &str = "http://dlmf.nist.gov/LaTeXML";

/// Perl `outputText` (bin/latexmlmath L283-300): `-` prints the string and a newline
/// on stdout; a file gets the string alone, its directory created first.
fn write_unicodemath(string: &str, dest: Option<&str>) -> Result<()> {
  match dest {
    None | Some("-") => println!("{string}"),
    Some(path) => {
      let path = std::path::Path::new(path);
      let written = match path.parent().filter(|d| !d.as_os_str().is_empty()) {
        Some(dir) => std::fs::create_dir_all(dir).and_then(|()| std::fs::write(path, string)),
        None => std::fs::write(path, string),
      };
      if let Err(e) = written {
        Error!(
          "latexmlmath",
          "output",
          format!("couldn't write UnicodeMath to {}", path.display()),
          e
        );
        process::exit(1);
      }
    },
  }
  Ok(())
}

/// Perl `finalize` (`Core/Document.pm` L452, `$node->removeAttribute($n) if $n =~ /^_/`)
/// drops every `_`-prefixed helper attribute — the ones holding fonts, source boxes, id
/// bookkeeping — before anything is output. `--xmath` reads the digested document
/// directly, upstream of our equivalent sweep (`latexml_core::document` L599), so run it
/// here or `_font="4568…"` leaks into the output where Perl shows none.
fn strip_internal_attrs(node: &mut libxml::tree::Node) {
  let internal: Vec<String> = node
    .get_attributes()
    .keys()
    .filter(|n| n.starts_with('_'))
    .cloned()
    .collect();
  for name in internal {
    let _ = node.remove_attribute(&name);
  }
  for mut child in node.get_child_elements() {
    strip_internal_attrs(&mut child);
  }
}

/// Print `node` alone, with `href` as its DEFAULT namespace — `<math xmlns="…">`, the
/// way Perl `latexmlmath` does it, not `<m:math>`.
///
/// Perl's `outputXML` reparents the node into a fresh document, then
/// `setNamespaceDeclPrefix($oldprefix, undef)` rewrites the one namespace *declaration*
/// so every element referencing it drops the prefix at once. Serializing in place
/// instead emits `<m:math>` with NO `xmlns:m` — the declaration lived on the
/// `<document>` root we discard — which is not well-formed standalone (`xmllint`:
/// "Namespace prefix m on math is not defined"). libxml-rs exposes no prefix mutator,
/// so the equivalent is: declare the default namespace on the extracted root and
/// re-point every element already in that namespace at it.
///
/// Indented like Perl's `toString(1)` via a serialize → re-parse → serialize round trip,
/// NOT by reparenting into a fresh document. Reparenting is what Perl does and would be
/// the direct route, but `Document::set_root_element` is `xmlDocSetRootElement`, which
/// does NOT call `xmlSetTreeDoc`: the subtree keeps pointing at the old document, and
/// dropping either one frees the other's nodes — `free(): invalid pointer`, SIGABRT on
/// every run (measured). XML::LibXML's `setDocumentElement` adopts the node; libxml-rs
/// exposes no equivalent. The round trip is safe because the namespace fix above makes
/// the serialized node well-formed standalone; `no_blanks` drops the whitespace text
/// nodes carrying the node's ORIGINAL nesting depth, so it re-indents from column 0
/// instead of inheriting `--xmath`'s ten-space lead-in.
fn print_node_defaulted(doc: &libxml::tree::Document, node: &mut libxml::tree::Node, href: &str) {
  use libxml::tree::Namespace;
  if let Ok(default_ns) = Namespace::new("", href, node) {
    fn repoint(node: &mut libxml::tree::Node, ns: &Namespace, href: &str) {
      if node.get_namespace().is_some_and(|n| n.get_href() == href) {
        let _ = node.set_namespace(ns);
      }
      for mut child in node.get_child_elements() {
        repoint(&mut child, ns, href);
      }
    }
    repoint(node, &default_ns, href);
  }
  let raw = doc.node_to_string(node);
  let pretty = libxml::parser::Parser::default()
    .parse_string_with_options(&raw, libxml::parser::ParserOptions {
      no_blanks: true,
      ..libxml::parser::ParserOptions::default()
    })
    .map(|reparsed| {
      reparsed.to_string_with_options(SaveOptions {
        format: true,
        no_declaration: true, // Perl serializes the NODE — no <?xml …?> preamble
        ..SaveOptions::default()
      })
    })
    // Never lose the output to a formatting nicety: fall back to the flat form.
    .unwrap_or(raw);
  print!("{pretty}");
}

fn main() -> Result<()> {
  // 256 MB stack — see cortex_worker.rs for rationale (#17).
  std::thread::Builder::new()
    .stack_size(256 * 1024 * 1024)
    .spawn(|| real_main().map_err(|e| e.to_string()))
    .expect("spawn worker thread")
    .join()
    .expect("worker thread panicked")
    .map_err(|s| s.into())
}

fn real_main() -> Result<()> {
  let raw_argv: Vec<String> = env::args().skip(1).collect();

  // Parse options left-to-right; `--` stops option parsing so formulas that
  // begin with `--` can still be passed positionally. Single-dash tokens other
  // than `-h`, `-q`, `-V` (such as `-x + 1` or `-1`) remain positional formula
  // text.
  let mut pmml_flag = false;
  let mut cmml_flag = false;
  let mut xmath_flag = false;
  // Perl `unicodemath=s` (bin/latexmlmath L57): `--unicodemath=FILE`, `-` for
  // stdout; the bare flag is stdout too. The last one given wins (GetOpt::Long).
  let mut unimath_dest: Option<String> = None;
  let mut quiet_flag = false;
  let mut show_help = false;
  let mut show_version = false;
  let mut positional: Vec<String> = Vec::new();
  let mut parsing_opts = true;

  for arg in raw_argv {
    if !parsing_opts {
      positional.push(arg);
      continue;
    }
    match arg.as_str() {
      "--" => parsing_opts = false,
      "-h" | "--help" => show_help = true,
      "-V" | "--version" => show_version = true,
      "--pmml" | "--presentationmathml" => pmml_flag = true,
      "--cmml" | "--contentmathml" => cmml_flag = true,
      "--xmath" | "--XMath" => xmath_flag = true,
      "--unicodemath" => unimath_dest = Some("-".to_string()),
      "-q" | "--quiet" => quiet_flag = true,
      _ if arg.starts_with("--unicodemath=") => {
        let dest = &arg["--unicodemath=".len()..];
        if dest.is_empty() {
          eprintln!(
            "latexmlmath_oxide: --unicodemath= requires a non-empty file path (or '-' for stdout)"
          );
          process::exit(2);
        }
        unimath_dest = Some(dest.to_string());
      },
      _ if arg.starts_with("--") => {
        eprintln!("latexmlmath_oxide: unknown option '{arg}' (try --help)");
        process::exit(2);
      },
      _ => positional.push(arg),
    }
  }

  // `--help` / `--version` before anything else: without it the flag would fall
  // through as a formula or fail on missing positional arguments.
  if show_help {
    println!("{USAGE}");
    process::exit(0);
  }
  if show_version {
    println!(
      "latexmlmath_oxide {}",
      latexml::identity::VERSION_WITH_REVISION
    );
    process::exit(0);
  }

  let unimath_flag = unimath_dest.is_some();
  // Perl L169: `$pmml = '-' unless (defined $mathimage || ... || $xmath || $unimath)` —
  // with no conversion option, presentation MathML on stdout is the default.
  if !pmml_flag && !cmml_flag && !xmath_flag && !unimath_flag {
    pmml_flag = true;
  }

  let log_level = if quiet_flag {
    log::LevelFilter::Warn
  } else {
    log::LevelFilter::Info
  };
  latexml_core::util::logger::init(log_level).ok();

  let source = match positional.first() {
    Some(s) => s.clone(),
    None => {
      eprintln!("{USAGE}");
      process::exit(1);
    },
  };

  // Identity banner (executable, version, git revision, exact start time) —
  // the same line `Converter::convert` logs for the document front-ends, emitted
  // here because latexmlmath drives its own digest path. `Note!` does the
  // log-always / stderr-gated split, so `--quiet` mutes it on the console
  // (`stderr_shows_info()` is false at Warn); latexmlmath binds no log buffer, so
  // there is no log file to keep it in either.
  Note!(latexml::identity::identity_banner());

  let mut core_engine = new_test_engine();
  let (lexemes, mut lex_nodes, xmath_opt, mut doc) =
    lex_single_tex_formula(&source, &mut core_engine);
  if lexemes.is_empty() {
    Error!("latexmlmath", "lex", "No lexemes produced from input");
    process::exit(1);
  }
  if !quiet_flag {
    eprintln!("lexemes: {lexemes:?}");
  }

  state::set_nomathparse_flag(false);
  let mut parser = MathParser::default();
  match parser.parse_lexemes(lexemes, &lex_nodes, &mut doc) {
    Ok(Some(parse_tree)) => {
      let mut xmath = xmath_opt.unwrap();
      for mut node in xmath.get_child_nodes() {
        node.unlink();
      }
      let xml_tree = parse_tree.into_xmath(&mut xmath, &mut lex_nodes, &mut doc)?;
      // `into_xmath` RETURNS the tree; it does not attach it. The main parser
      // path pairs it with `append_tree` (`parser.rs`, the reparent step) —
      // without that, a formula whose whole body is ONE top-level structure
      // (`\frac{1}{2}`, `\sqrt{2}`) serialized as an empty `<mrow/>`, because
      // the pre-parse children were unlinked just above and nothing replaced
      // them. Formulas with more than one top-level part appeared to work only
      // because their parts are pre-existing lexeme nodes still in the tree.
      if xml_tree.get_parent().is_none() {
        doc.append_tree(&mut xmath, vec![xml_tree.clone()])?;
      }
      xmath
        .get_parent()
        .unwrap()
        .set_attribute("text", &text_form(&xml_tree, &doc))
        .unwrap();

      use latexml_post::{
        document::{PostDocument, PostDocumentOptions},
        processor::Processor,
      };
      // Serialised once, before `--xmath` reparents the XMath out of `doc`: each
      // format post-processes its own copy (Perl `cloneDoc`, bin/latexmlmath L264-269).
      let xml_str = (pmml_flag || cmml_flag || unimath_flag).then(|| {
        doc.get_document().to_string_with_options(SaveOptions {
          format: true,
          ..SaveOptions::default()
        })
      });
      if let Some(xml_str) = xml_str.as_deref()
        && (pmml_flag || cmml_flag)
      {
        // Post-process with MathML
        let post_doc = PostDocument::new_from_string(xml_str, PostDocumentOptions::default())
          .expect("parse XML for MathML post-processing");
        let mut post = latexml_post::Post::new();
        let mut processors: Vec<Box<dyn Processor>> = Vec::new();
        // Parallel P+C: the Content processor is a parallel *secondary* of the
        // Presentation primary, merged into one <m:semantics>/<m:annotation-xml>
        // by combine_parallel — not an independent pass (which left content as
        // an orphan <apply> sibling of <m:semantics>). Mirrors latexml_oxide::post.
        if pmml_flag {
          let mut presentation = latexml_post::mathml::MathML::new_presentation();
          if cmml_flag {
            presentation = presentation.with_secondaries(vec![Box::new(
              latexml_post::mathml::MathML::new_content().secondary(),
            )]);
          }
          processors.push(Box::new(presentation));
        } else if cmml_flag {
          processors.push(Box::new(latexml_post::mathml::MathML::new_content()));
        }
        match post.process_chain(vec![post_doc], &mut processors) {
          Ok(results) => {
            // Perl emits the math element alone (`findnode('//m:math')`), not the
            // document around it. With parallel P+C the content markup is annotated
            // INTO the presentation <m:math>, so one node carries both.
            match results[0].findnodes("//m:math").into_iter().next() {
              Some(mut math) => {
                print_node_defaulted(results[0].get_document(), &mut math, MATHML_NS)
              },
              None => {
                Error!(
                  "latexmlmath",
                  "output",
                  "no <m:math> produced for the formula"
                );
                process::exit(1);
              },
            }
          },
          Err(e) => {
            latexml_core::common::error::emit_error(
              "latexmlmath",
              "postprocess",
              &format!("MathML post-processing failed: {e}"),
            );
            process::exit(1);
          },
        }
      }

      // Perl emits XMath after pmml/cmml (bin/latexmlmath L232-241), and the order is
      // load-bearing here: `print_node_defaulted` REPARENTS the node, so running this
      // first would tear <XMath> out of `doc` before the MathML path serialises it.
      // Read straight from the digested document — the MathML path above serialises +
      // re-parses, which re-materialises indentation whitespace at the node's original
      // depth and drags it into the extracted subtree.
      if xmath_flag {
        strip_internal_attrs(&mut xmath);
        print_node_defaulted(doc.get_document(), &mut xmath, LATEXML_NS);
      }

      // UnicodeMath comes last, as in Perl (bin/latexmlmath L244-248): the standalone
      // processor puts the string in `ltx:Math/ltx:text`, printed as plain text. Perl
      // prints that text node's XML serialisation, escaping `<` and `&`
      // (KNOWN_PERL_ERRORS #1290); the string itself is the output here.
      if let Some(xml_str) = xml_str.as_deref()
        && unimath_flag
      {
        let post_doc = PostDocument::new_from_string(xml_str, PostDocumentOptions::default())
          .expect("parse XML for UnicodeMath post-processing");
        let mut processors: Vec<Box<dyn Processor>> =
          vec![Box::new(latexml_post::unicode_math::UnicodeMath::new())];
        match latexml_post::Post::new().process_chain(vec![post_doc], &mut processors) {
          Ok(results) => match results[0].findnodes("//ltx:Math/ltx:text").first() {
            Some(text) => write_unicodemath(&text.get_content(), unimath_dest.as_deref())?,
            None => {
              Error!(
                "latexmlmath",
                "output",
                "no UnicodeMath produced for the formula"
              );
              process::exit(1);
            },
          },
          Err(e) => {
            latexml_core::common::error::emit_error(
              "latexmlmath",
              "postprocess",
              &format!("UnicodeMath post-processing failed: {e}"),
            );
            process::exit(1);
          },
        }
      }
    },
    _ => {
      Warn!("math", "parse", "Grammar did not recognize expression.");
      process::exit(1);
    },
  }
  // End-of-run verdict guard, as in the main CLI: a Fatal raised anywhere in
  // the conversion (and recovered into a result) must not exit 0 — the
  // framework side treats a fatal-status run as failed, and so must we.
  // Single-phase binary: the math post-conversions run in-process against the
  // same shared REPORT counter, so reading it here IS the complete
  // max-across-phases fold (there is no separate `PostOutcome` to fold in).
  let final_status_code = latexml_core::common::error::get_status_code();
  if final_status_code >= 3 {
    eprintln!(
      "{}",
      latexml_core::common::error::conversion_verdict(final_status_code)
    );
    process::exit(1);
  }
  Ok(())
}
