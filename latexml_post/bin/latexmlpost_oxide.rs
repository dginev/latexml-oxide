//! latexmlpost_oxide — Post-process LaTeXML XML output.
//!
//! Usage: latexmlpost_oxide [--pmml] [--keepXMath] [--noscan] [--nocrossref]
//!        [--stylesheet path.xsl] [--whatsout TYPE] [--dest output.xml] input.xml

use latexml_post::Post;
use latexml_post::document::{PostDocument, PostDocumentOptions};
use latexml_post::extract::{Whatsout, serialize_whatsout};
use latexml_post::mathml::MathML;
use latexml_post::processor::Processor;
use latexml_post::xslt::XSLT;
use rustc_hash::FxHashMap as HashMap;

fn main() {
  let args: Vec<String> = std::env::args().collect();

  let mut input_path = None;
  let mut dest_path = None;
  let mut pmml = false;
  let mut keep_xmath = false;
  let mut stylesheet = None;
  let mut whatsout: Whatsout = Whatsout::default();

  let mut i = 1;
  while i < args.len() {
    match args[i].as_str() {
      "--pmml" => pmml = true,
      "--keepXMath" | "--xmath" => keep_xmath = true,
      "--noscan" | "--nocrossref" => {}, // accepted but no-op for now
      "--stylesheet" => {
        i += 1;
        if i < args.len() {
          stylesheet = Some(args[i].clone());
        }
      },
      "--dest" => {
        i += 1;
        if i < args.len() {
          dest_path = Some(args[i].clone());
        }
      },
      "--whatsout" => {
        i += 1;
        if i < args.len() {
          match Whatsout::from_cli(&args[i]) {
            Some(w) => whatsout = w,
            None => eprintln!("Warning: unknown --whatsout '{}', falling back to 'document'", args[i]),
          }
        }
      },
      arg if !arg.starts_with('-') => input_path = Some(arg.to_string()),
      other => eprintln!("Warning: unknown option '{}'", other),
    }
    i += 1;
  }

  let input_path = input_path.unwrap_or_else(|| {
    eprintln!(
      "Usage: latexmlpost_oxide [--pmml] [--keepXMath] [--stylesheet path.xsl] \
       [--whatsout document|fragment|math] [--dest output] input.xml"
    );
    std::process::exit(1);
  });

  // Default to pmml if nothing specified
  if !pmml && stylesheet.is_none() {
    pmml = true;
  }

  let input = std::fs::read_to_string(&input_path).unwrap_or_else(|e| {
    eprintln!("Failed to read {}: {}", input_path, e);
    std::process::exit(1);
  });

  let doc =
    PostDocument::new_from_string(&input, PostDocumentOptions::default()).unwrap_or_else(|e| {
      eprintln!("Failed to parse {}: {}", input_path, e);
      std::process::exit(1);
    });

  let mut post = Post::new();
  let mut processors: Vec<Box<dyn Processor>> = Vec::new();

  if pmml {
    let mathml = MathML::new_presentation().with_keep_xmath(keep_xmath);
    processors.push(Box::new(mathml));
  }

  if let Some(ref xsl_path) = stylesheet {
    let searchpaths = vec!["resources/XSLT".to_string(), ".".to_string()];
    let xslt = XSLT::new(xsl_path, HashMap::default(), false, None, searchpaths).unwrap_or_else(|e| {
      eprintln!("Failed to create XSLT processor: {}", e);
      std::process::exit(1);
    });
    processors.push(Box::new(xslt));
  }

  let results = post
    .process_chain(vec![doc], &mut processors)
    .unwrap_or_else(|e| {
      eprintln!("Post-processing failed: {}", e);
      std::process::exit(1);
    });

  // Apply --whatsout (Perl `LaTeXML::Util::Pack::whatsout`). `Document`
  // (default) is a no-op passthrough; `Fragment` / `Math` extract the
  // matching subtree via `latexml_post::extract::serialize_whatsout`.
  let output = serialize_whatsout(&results[0], whatsout);

  // Route through the shared `latexml_post::writer::write_output`
  // (Perl `LaTeXML::Post::Writer` analog) so all post-processing
  // binaries share one destination-handling implementation.
  if let Err(e) = latexml_post::writer::write_output(&output, dest_path.as_deref()) {
    eprintln!(
      "Failed to write {}: {}",
      dest_path.as_deref().unwrap_or("<stdout>"),
      e
    );
    std::process::exit(1);
  }
  if let Some(dest) = dest_path.as_deref() {
    eprintln!("Wrote {}", dest);
  }
}
