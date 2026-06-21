use std::{
  fs::File,
  io::{BufRead, BufReader},
};

use latexml_core::{common::error::*, fatal, s, util::pathname};
use once_cell::sync::Lazy;
use proc_macro::TokenStream;
use quote::quote;
use regex::Regex;
use syn::DeriveInput;

static TAG_MODEL_LINE: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"^([^\{]+)\{(.*?)\}\((.*?)\)$").unwrap());
// Mirrors Perl Model.pm L149: `m/^([^:=]+):=\(?([^)]*?)\)?$/` — the
// `\(?…\)?` pair strips the surrounding parens from
// `classname:=(elt1,elt2,...)` so the elements split cleanly.
static CLASS_MODEL_LINE: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"^([^:=]+):=\(?([^)]*?)\)?$").unwrap());
static NAMESPACE_MODEL_LINE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^([^=]+)=(.*?)$").unwrap());

pub fn load_model(input: DeriveInput) -> Result<TokenStream> {
  let name = crate::attr_name_value_str(&input.attrs[0], "name");

  let pathname_opt = pathname::find(&name, pathname::PathnameFindOptions {
    paths:               Some(vec![s!(".")]),
    extensions:          Some(vec![s!("model")]),
    installation_subdir: Some(s!("resources/RelaxNG")),
  });

  let path = match pathname_opt {
    Some(n) => n,
    None => panic!("Model {name:?} not found, required to load a compiled model!"),
  };

  let mut operations = Vec::new();
  // NOTE: Do something automatic about this too!?!
  // We'll need to generate namespace prefixes for all namespaces found in the doc!
  operations.push(quote!(
    model::register_document_namespace("", Some("http://dlmf.nist.gov/LaTeXML"));
    model::set_schema(Relaxng{ name: #name.to_owned(), ..Relaxng::default()});
  ));

  // note_begin(&(s!("Compiling .model file: {}", path)));
  let compiled_fh = File::open(path.clone())?;
  let compiled_reader = BufReader::new(&compiled_fh);
  for line in compiled_reader.lines().map_while(std::result::Result::ok) {
    if let Some(caps) = TAG_MODEL_LINE.captures(&line) {
      let tag = caps.get(1).map_or("", |m| m.as_str()).to_string();
      let attr = caps.get(2).map_or("", |m| m.as_str()).to_string();
      let children = caps.get(3).map_or("", |m| m.as_str()).to_string();

      let attr_vec: Vec<String> = attr.split(',').map(ToString::to_string).collect();
      let child_vec: Vec<String> = children.split(',').map(ToString::to_string).collect();

      operations.push(quote!(
        model::add_tag_attribute(#tag, vec![#(#attr_vec),*]);
        model::add_tag_content(#tag, vec![#(#child_vec),*]);
      ));
    } else if let Some(caps) = CLASS_MODEL_LINE.captures(&line) {
      let classname = caps.get(1).map_or("", |m| m.as_str()).to_string();
      let elements = caps.get(2).map_or("", |m| m.as_str()).to_string();
      let elements_vec = elements
        .split(',')
        .map(ToString::to_string)
        .collect::<Vec<String>>();

      operations.push(quote!(
        model::set_schema_class(#classname,
          rustc_hash::FxHashSet::from_iter(vec![#(#elements_vec),*].into_iter()
          .map(latexml_core::common::arena::pin_static)));
      ));
    } else if let Some(caps) = NAMESPACE_MODEL_LINE.captures(&line) {
      let prefix = caps.get(1).map_or("", |m| m.as_str());
      let namespace = caps.get(2).map_or("", |m| m.as_str());
      operations.push(quote!(
        model::register_document_namespace(#prefix, Some(#namespace));
      ));
    } else {
      fatal!(
        Codegen,
        Malformed,
        s!(" Loaded model '{}' is malformatted at \"{}\"", path, line)
      );
    }
  }

  // note_end(&(s!("Compiling .model file: {}", path)));

  Ok(TokenStream::from(quote!(
    impl _ModelLoader {
      fn build_model() {
        #(#operations)*
      }
    }
  )))
}
