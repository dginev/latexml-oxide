use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;

use lazy_static::lazy_static;
use regex::Regex;

use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, Lit, Meta};

use rtx_core::common::error::*;
use rtx_core::state::State;
use rtx_core::util::pathname;
use rtx_core::{fatal, s};

lazy_static! {
  static ref TAG_MODEL_LINE: Regex = Regex::new(r"^([^\{]+)\{(.*?)\}\((.*?)\)$").unwrap();
  static ref CLASS_MODEL_LINE: Regex = Regex::new(r"^([^:=]+):=(.*?)$").unwrap();
  static ref NAMESPACE_MODEL_LINE: Regex = Regex::new(r"^([^=]+)=(.*?)$").unwrap();
}

pub fn load_model(input: DeriveInput) -> Result<TokenStream> {
  let name: String = match input.attrs[0].parse_meta().unwrap() {
    Meta::NameValue(v) => match v.lit {
      Lit::Str(v) => v.value().to_string(),
      _ => panic!("only accepts #[name = \"filename\"] attribute syntax, mandatory double-quotes (Lit)"),
    },
    _ => panic!("only accepts #[name = \"filename\"] attribute syntax, mandatory double-quotes (parse_meta)"),
  };

  let pathname_opt = pathname::find(
    &name,
    pathname::PathnameFindOptions {
      paths: Some(vec![s!(".")]),
      types: Some(vec![s!("model")]),
      installation_subdir: Some(s!("resources/RelaxNG")),
    },
  );

  let path = match pathname_opt {
    Some(n) => n,
    None => panic!("Model {:?} not found, required to load a compiled model!", name),
  };

  let mut operations = Vec::new();
  // NOTE: Do something automatic about this too!?!
  // We'll need to generate namespace prefixes for all namespaces found in the doc!
  operations.push(quote!(
    model.register_document_namespace("", Some(s!("http://dlmf.nist.gov/LaTeXML")));
    model.schema = Some(Relaxng{ name: #name.to_owned(), ..Relaxng::default()});
  ));

  // note_begin(&(s!("Compiling .model file: {}", path)));
  let compiled_fh = File::open(path.clone())?;
  let compiled_reader = BufReader::new(&compiled_fh);
  for line_result in compiled_reader.lines() {
    if let Ok(line) = line_result {
      if let Some(caps) = TAG_MODEL_LINE.captures(&line) {
        let tag = caps.get(1).map_or("", |m| m.as_str()).to_string();
        let attr = caps.get(2).map_or("", |m| m.as_str()).to_string();
        let children = caps.get(3).map_or("", |m| m.as_str()).to_string();

        let attr_vec: Vec<String> = attr.split(',').map(ToString::to_string).collect();
        let child_vec: Vec<String> = children.split(',').map(ToString::to_string).collect();

        operations.push(quote!(
          model.add_tag_attribute(#tag, vec![#(#attr_vec),*]);
          model.add_tag_content(#tag, vec![#(#child_vec),*]);
        ));
      } else if let Some(caps) = CLASS_MODEL_LINE.captures(&line) {
        let classname = caps.get(1).map_or("", |m| m.as_str()).to_string();
        let elements = caps.get(2).map_or("", |m| m.as_str()).to_string();
        let elements_vec = elements.split(',').map(ToString::to_string).collect::<Vec<String>>();

        operations.push(quote!(
          model.set_schema_class(#classname,
            HashSet::from_iter(vec![#(#elements_vec),*].iter().map(ToString::to_string)));
        ));
      } else if let Some(caps) = NAMESPACE_MODEL_LINE.captures(&line) {
        let prefix = caps.get(1).map_or("", |m| m.as_str()).to_string();
        let namespace = caps.get(2).map_or("", |m| m.as_str()).to_string();
        operations.push(quote!(
          model.register_document_namespace(#prefix, Some(#namespace.to_owned()));
        ));
      } else {
        fatal!(Codegen, Malformed, s!(" Loaded model '{}' is malformatted at \"{}\"", path, line));
      }
    }
  }

  operations.push(quote!(return;));
  // note_end(&(s!("Compiling .model file: {}", path)));

  Ok(TokenStream::from(quote!(
    impl _ModelLoader {
      fn model(model : &mut Model) {
        #(#operations)*
      }
    }
  )))
}

pub fn load_indirect_model(input: DeriveInput) -> TokenStream {
  // Load the model as one would at runtime
  let name = quote!(#input).to_string();
  let mut state = State::default();
  state.model.set_relaxng_schema(name.to_string());
  state.model.load_schema(&[]);

  let indirect_model = state.compute_indirect_model();

  let mut operations = Vec::new();
  operations.push(quote!(let mut im : IndirectModel = HashMap::new();));
  for (key, sub_model) in indirect_model {
    for (sub_key, value) in sub_model {
      operations.push(quote!(im.entry(#key).or_insert_with(HashMap::new).entry(#sub_key).or_insert(#value)));
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
  .into()
}
