/// latexmlpost_oxide — Post-process LaTeXML XML output.
///
/// Usage: latexmlpost_oxide [--pmml] [--keepXMath] [--noscan] [--nocrossref]
///        [--dest output.xml] input.xml

use latexml_post::document::{PostDocument, PostDocumentOptions};
use latexml_post::mathml::MathML;
use latexml_post::processor::Processor;
use latexml_post::Post;

fn main() {
  let args: Vec<String> = std::env::args().collect();

  let mut input_path = None;
  let mut dest_path = None;
  let mut pmml = false;
  let mut keep_xmath = false;

  let mut i = 1;
  while i < args.len() {
    match args[i].as_str() {
      "--pmml" => pmml = true,
      "--keepXMath" | "--xmath" => keep_xmath = true,
      "--noscan" | "--nocrossref" => {} // accepted but no-op for now
      "--dest" => {
        i += 1;
        if i < args.len() {
          dest_path = Some(args[i].clone());
        }
      }
      arg if !arg.starts_with('-') => input_path = Some(arg.to_string()),
      other => eprintln!("Warning: unknown option '{}'", other),
    }
    i += 1;
  }

  let input_path = input_path.unwrap_or_else(|| {
    eprintln!("Usage: latexmlpost_oxide [--pmml] [--keepXMath] [--dest output.xml] input.xml");
    std::process::exit(1);
  });

  // Default to pmml if nothing specified
  if !pmml {
    pmml = true;
  }

  let input = std::fs::read_to_string(&input_path)
    .unwrap_or_else(|e| {
      eprintln!("Failed to read {}: {}", input_path, e);
      std::process::exit(1);
    });

  let doc = PostDocument::new_from_string(&input, PostDocumentOptions::default())
    .unwrap_or_else(|e| {
      eprintln!("Failed to parse {}: {}", input_path, e);
      std::process::exit(1);
    });

  let mut post = Post::new();
  let mut processors: Vec<Box<dyn Processor>> = Vec::new();

  if pmml {
    let mathml = MathML::new_presentation().with_keep_xmath(keep_xmath);
    processors.push(Box::new(mathml));
  }

  let results = post.process_chain(doc, &mut processors)
    .unwrap_or_else(|e| {
      eprintln!("Post-processing failed: {}", e);
      std::process::exit(1);
    });

  let output = results[0].to_xml_string();

  if let Some(dest) = dest_path {
    std::fs::write(&dest, &output).unwrap_or_else(|e| {
      eprintln!("Failed to write {}: {}", dest, e);
      std::process::exit(1);
    });
    eprintln!("Wrote {}", dest);
  } else {
    print!("{}", output);
  }
}
