use syn;
use quote;
use regex::Regex;
use util::{get_option, get_options_from_input};

use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;
use rtx_core::common::error::*;
use rtx_core::util::pathname;
use rtx_core::state::State;

lazy_static! {
  static ref TAG_MODEL_LINE : Regex = Regex::new(r"^([^\{]+)\{(.*?)\}\((.*?)\)$").unwrap();
  static ref CLASS_MODEL_LINE : Regex = Regex::new(r"^([^:=]+):=(.*?)$").unwrap();
  static ref NAMESPACE_MODEL_LINE : Regex = Regex::new(r"^([^=]+)=(.*?)$").unwrap();
}

pub fn load_model(input: syn::MacroInput) -> Result<quote::Tokens> {
  fn bug() -> ! {
    panic!(
      "This is a bug. Please open a Github issue \
       with your load_model invocation"
    );
  }
  let options = get_options_from_input("load_model_options", &input.attrs, bug);
  let name_opt = options.as_ref().map(|o| get_option(&o, "name", bug));
  let name = match name_opt {
    Some(n) => n,
    None => panic!("Model name is required to load a compiled model!"),
  };

  let pathname_opt = pathname::find(
    &name,
    pathname::FindOptions {
      paths: Some(vec![".".to_owned()]),
      types: Some(vec!["model".to_string()]),
      installation_subdir: Some("resources/RelaxNG".to_owned()),
    },
  );

  let path = match pathname_opt {
    Some(n) => n,
    None => panic!("Model not found, required to load a compiled model!"),
  };

  let mut operations = Vec::new();
  // NOTE: Do something automatic about this too!?!
  // We'll need to generate namespace prefixes for all namespaces found in the doc!
  operations.push(quote!(
    model.register_document_namespace("", Some("http://dlmf.nist.gov/LaTeXML".to_owned()));
    model.schema = Some(Relaxng{ name: #name.to_owned(), ..Relaxng::default()});
  ));

  // note_begin(&(format!("Compiling .model file: {}", path)));
  let compiled_fh = try!(File::open(path.clone()));
  let compiled_reader = BufReader::new(&compiled_fh);
  for line_result in compiled_reader.lines() {
    if let Ok(line) = line_result {
      if let Some(caps) = TAG_MODEL_LINE.captures(&line) {
        let tag = caps.get(1).map_or("", |m| m.as_str()).to_string();
        let attr = caps.get(2).map_or("", |m| m.as_str()).to_string();
        let children = caps.get(3).map_or("", |m| m.as_str()).to_string();

        let attr_vec: Vec<String> = attr.split(",").map(|t| t.to_string()).collect();
        let child_vec: Vec<String> = children.split(",").map(|t| t.to_string()).collect();

        operations.push(quote!(
          model.add_tag_attribute(#tag, vec![#(#attr_vec),*]);
          model.add_tag_content(#tag, vec![#(#child_vec),*]);
        ));
      } else if let Some(caps) = CLASS_MODEL_LINE.captures(&line) {
        let classname = caps.get(1).map_or("", |m| m.as_str()).to_string();
        let elements = caps.get(2).map_or("", |m| m.as_str()).to_string();
        let elements_vec = elements
          .split(",")
          .map(|t| t.to_string())
          .collect::<Vec<String>>();

        operations.push(quote!(
          model.set_schema_class(#classname,
            HashSet::from_iter(vec![#(#elements_vec),*].iter().map(|t| t.to_string())));
        ));
      } else if let Some(caps) = NAMESPACE_MODEL_LINE.captures(&line) {
        let prefix = caps.get(1).map_or("", |m| m.as_str()).to_string();
        let namespace = caps.get(2).map_or("", |m| m.as_str()).to_string();
        operations.push(quote!(
          model.register_document_namespace(#prefix, Some(#namespace.to_owned()));
        ));
      } else {
        fatal!(
          Codegen,
          Malformed,
          format!(
            " Loaded model '{:?}' is malformatted at \"{:?}\"",
            path, line
          )
        );
      }
    }
  }

  operations.push(quote!(return;));
  // note_end(&(format!("Compiling .model file: {}", path)));

  Ok(quote!(
    impl _ModelLoader {
      fn model(model : &mut Model) {
        #(#operations)*
      }
    }
  ))
}

pub fn load_indirect_model(input: syn::MacroInput) -> quote::Tokens {
  // Load the model as one would at runtime
  fn bug() -> ! {
    panic!(
      "This is a bug. Please open a Github issue \
       with your load_model invocation"
    );
  }
  let options = get_options_from_input("load_indirect_model_options", &input.attrs, bug);
  let name_opt = options.as_ref().map(|o| get_option(&o, "name", bug));
  let name = match name_opt {
    Some(n) => n,
    None => panic!("Model name is required to load a compiled model!"),
  };

  let mut state = State::default();
  state.model.set_relaxng_schema(name.to_string());
  state.model.load_schema(None);

  let indirect_model = state.compute_indirect_model();

  let mut operations = Vec::new();
  operations.push(quote!(let mut im : IndirectModel = HashMap::new();));
  for (key, sub_model) in indirect_model {
    for (sub_key, value) in sub_model {
      operations.push(
        quote!(im.entry(#key).or_insert_with(HashMap::new).entry(#sub_key).or_insert(#value)),
      );
    }
  }
  operations.push(quote!(return im));

  quote!(
    impl _ModelLoader {
      fn indirect_model() -> IndirectModel {
        #(#operations)*
      }
    }
  )
}
