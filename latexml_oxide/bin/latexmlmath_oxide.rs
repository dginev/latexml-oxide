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
  -q, --quiet                       Only warnings and errors; no progress/lexeme notes
  -h, --help                        Print this help

Passing both --pmml and --cmml emits parallel markup: Content MathML annotated
into the Presentation primary, in one <m:semantics>.

Examples:
  latexmlmath_oxide '1+1=2'
  latexmlmath_oxide 'a^2+b^2=c^2'
  latexmlmath_oxide --cmml '\\frac{a}{b}+c'

Convert whole documents with `latexml_oxide` instead.";

const MATHML_NS: &str = "http://www.w3.org/1998/Math/MathML";
const LATEXML_NS: &str = "http://dlmf.nist.gov/LaTeXML";

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
/// NOT reparented into a fresh document the way Perl does, despite that also buying
/// Perl's indented `toString(1)`: `Document::set_root_element` maps to
/// `xmlDocSetRootElement`, which does NOT call `xmlSetTreeDoc`, so the subtree's `->doc`
/// pointers keep referencing the old document and dropping either one frees nodes owned
/// by the other — `free(): invalid pointer`, SIGABRT, on every run. XML::LibXML's
/// `setDocumentElement` adopts the node; libxml-rs exposes no equivalent. So we
/// serialize in place, which costs only Perl's indentation (`node_to_string` hardcodes
/// libxml2 format=0). The markup and namespaces are identical.
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
  println!("{}", doc.node_to_string(node));
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
  let mut argv: Vec<String> = env::args().skip(1).collect();

  // Parse flags
  let mut pmml_flag = argv
    .iter()
    .any(|a| a == "--pmml" || a == "--presentationmathml");
  let cmml_flag = argv.iter().any(|a| a == "--cmml" || a == "--contentmathml");
  let xmath_flag = argv.iter().any(|a| a == "--xmath" || a == "--XMath");
  let quiet_flag = argv.iter().any(|a| a == "--quiet" || a == "-q");
  argv.retain(|a| {
    ![
      "--pmml",
      "--presentationmathml",
      "--cmml",
      "--contentmathml",
      "--xmath",
      "--XMath",
      "--quiet",
      "-q",
    ]
    .contains(&a.as_str())
  });
  // Perl L169: `$pmml = '-' unless (defined $mathimage || ... || $xmath || $unimath)` —
  // with no conversion option, presentation MathML on stdout is the default.
  if !pmml_flag && !cmml_flag && !xmath_flag {
    pmml_flag = true;
  }

  let log_level = if quiet_flag {
    log::LevelFilter::Warn
  } else {
    log::LevelFilter::Info
  };
  latexml_core::util::logger::init(log_level).ok();

  // `--help` before anything else: without it the flag falls through as the formula
  // and gets typeset, landing the user in TeX's interactive error prompt.
  if argv.iter().any(|a| a == "--help" || a == "-h") {
    println!("{USAGE}");
    process::exit(0);
  }

  let source = match argv.first() {
    Some(s) => s.clone(),
    None => {
      eprintln!("{USAGE}");
      process::exit(1);
    },
  };

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
      xmath
        .get_parent()
        .unwrap()
        .set_attribute("text", &text_form(&xml_tree, &doc))
        .unwrap();

      if pmml_flag || cmml_flag {
        // Post-process with MathML
        let xml_str = doc.get_document().to_string_with_options(SaveOptions {
          format: true,
          ..SaveOptions::default()
        });
        use latexml_post::{
          document::{PostDocument, PostDocumentOptions},
          processor::Processor,
        };
        let post_doc = PostDocument::new_from_string(&xml_str, PostDocumentOptions::default())
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
            eprintln!("MathML post-processing failed: {}", e);
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
    },
    _ => {
      Warn!("math", "parse", "Grammar did not recognize expression.");
      process::exit(1);
    },
  }
  Ok(())
}
