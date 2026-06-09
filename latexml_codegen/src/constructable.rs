//! Compile-time consumer of the shared XML-replacement template AST (#171).
//!
//! The template grammar, its parser, and the runtime interpreter all live in
//! `latexml_core::binding::def::replacement`. This file is the **compile-time**
//! consumer: it parses a constructor's `"<ltx:…>"` replacement into a
//! `Vec<ReplacementOp>` (via the *same* `winnow` parser the runtime uses) and
//! lowers each op to the `quote!` `Document` calls that previously came from the
//! hand-rolled regex-strip state machine. Sharing one parser/AST means the
//! compile-time and runtime template paths cannot drift.
//!
//! The emitted code is byte-for-byte equivalent to the previous compiler's
//! output (same `Document` operations, same `savenode` discipline), so the
//! generated native constructors are unchanged.

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::quote;
use syn::DeriveInput;

use latexml_core::binding::def::replacement::{
  parse_replacement, slashify, AttrPair, AttrPart, AttrValue, FloatKind, FuncArg, ReplacementOp,
  Value,
};

pub fn compile_replacement(input: DeriveInput) -> TokenStream {
  let replacement = crate::attr_name_value_str(&input.attrs[0], "replacement");

  let compiled_replacement_closure: TokenStream2 = if replacement.is_empty() {
    quote!(None)
  } else {
    let ops = parse_replacement(&replacement).unwrap_or_else(|e| {
      panic!("compile_replacement: failed to parse template {replacement:?}: {e}")
    });
    let operations = emit_ops(&ops);

    quote!(
    Some(Rc::new(
    |document: &mut Document,
      #[allow(unused_variables)]args: &Vec<Option<Digested>>,
      #[allow(unused_variables)]props: &SymHashMap<Stored>| {
      #[allow(unused_assignments,unused_mut)]
      let mut savenode : Option<Node> = None;

      #(#operations)*

      if let Some(snode) = savenode {
        document.set_node(&snode);
      }
      Ok(())
    })))
  };
  // We have to jump an extra hoop, since we are forcing the struct-derive
  // mechanism. Once the new procedural macro scheme lands, this begs to be
  // refactored.
  quote!(
    macro_rules! this_replacement {
      () => {#compiled_replacement_closure}
    }
  )
  .into()
}

/// Lower a parsed op-list to a sequence of `quote!` statements.
fn emit_ops(ops: &[ReplacementOp]) -> Vec<TokenStream2> { ops.iter().map(emit_op).collect() }

fn emit_op(op: &ReplacementOp) -> TokenStream2 {
  match op {
    ReplacementOp::OpenElement { qname, attrs, float, self_closing } => {
      emit_open(qname, attrs, *float, *self_closing)
    },
    ReplacementOp::CloseElement { qname } => quote!(document.close_element(#qname)?;),
    ReplacementOp::ProcessingInstruction { qname, attrs } => emit_pi(qname, attrs),
    ReplacementOp::AbsorbValue { value } => emit_absorb(value),
    ReplacementOp::SetAttribute { key, value, float } => emit_set_attribute(key, value, *float),
    ReplacementOp::Text { text } => {
      // Re-slashify the (already-`unquote`d) literal for embedding as a Rust
      // string literal — preserving the original compiler's output exactly.
      let escaped = slashify(text);
      quote!(document.absorb_string(#escaped, props)?;)
    },
    ReplacementOp::Conditional { test, then_ops, else_ops } => {
      let cond = emit_bool(test);
      let then_c = emit_ops(then_ops);
      let else_c = emit_ops(else_ops);
      quote!(
        if #cond {
          #(#then_c)*
        } else {
          #(#else_c)*
        }
      )
    },
  }
}

fn emit_open(
  qname: &str,
  attrs: &[AttrPair],
  float: Option<FloatKind>,
  self_closing: bool,
) -> TokenStream2 {
  let mut stmts: Vec<TokenStream2> = Vec::new();
  match float {
    Some(FloatKind::Single) => {
      stmts.push(quote!(savenode = document.float_to_element(#qname, false)?;))
    },
    Some(FloatKind::Double) => {
      stmts.push(quote!(savenode = document.float_to_element(#qname, true)?;))
    },
    None => {},
  }
  if attrs.is_empty() {
    stmts.push(quote!(document.open_element(#qname, None, None)?;));
  } else {
    let av = emit_avpairs(attrs);
    stmts.push(quote!(
      #[allow(unused_mut)]
      let mut av_props : HashMap<String, String> = HashMap::default();
      #(#av)*
      let this_font_opt = match props.get("font") {
        Some(Stored::Font(f)) => Some(Cow::Borrowed(&**f)),
        Some(Stored::FontDirective(FontDirective::Asset(fa))) => Some(Cow::Borrowed(&**fa)),
        Some(Stored::FontDirective(FontDirective::Closure(code))) =>
          Some(Cow::Owned(code(None)?)),
        _ => None
      };
      if let Some(this_font) = this_font_opt {
        document.open_element(#qname, Some(av_props), Some(&this_font))?;
      } else {
        document.open_element(#qname, Some(av_props), None)?;
      }
    ));
  }
  if self_closing {
    stmts.push(quote!(document.close_element(#qname)?;));
  }
  quote!(#(#stmts)*)
}

fn emit_pi(qname: &str, attrs: &[AttrPair]) -> TokenStream2 {
  if attrs.is_empty() {
    quote!(document.insert_pi(#qname, None)?;)
  } else {
    let av = emit_avpairs(attrs);
    quote!(
      let mut av_props : HashMap<String, String> = HashMap::default();
      #(#av)*
      document.insert_pi(#qname, Some(av_props))?;
    )
  }
}

fn emit_absorb(value: &Value) -> TokenStream2 {
  let to_absorb = emit_value(value, false);
  quote!(
    if let Some(ref stored_digested) = #to_absorb {
      let digested_opt : Option<Digested> = stored_digested.into();
      if let Some(ref digested) = digested_opt {
        document.absorb(digested, None)?;
      }
    }
  )
}

fn emit_set_attribute(key: &str, value: &AttrValue, float: bool) -> TokenStream2 {
  let val = emit_attr_value(value);
  if float {
    quote!(
      {
        let val_str: String = #val;
        savenode = document.float_to_attribute(#key);
        let mut node = document.get_node().clone();
        document.set_attribute(&mut node, #key, &val_str)?;
        if let Some(ref sn) = savenode { document.set_node(sn); }
      }
    )
  } else {
    quote!(
      {
        let val_str: String = #val;
        let mut node = document.get_node().clone();
        document.set_attribute(&mut node, #key, &val_str)?;
      }
    )
  }
}

/// Attribute-value pairs inside a `<tag …>` / `<?pi …?>` (Perl `translate_avpairs`).
fn emit_avpairs(attrs: &[AttrPair]) -> Vec<TokenStream2> {
  attrs
    .iter()
    .map(|a| match a {
      AttrPair::KeyValue { key, value } => {
        if key == "font" {
          // font is handled via the open-element font param, not av_props.
          quote!(();)
        } else {
          let val = emit_attr_value(value);
          quote!(av_props.insert(#key.to_string(), #val);)
        }
      },
      AttrPair::Conditional { test, then_attrs, else_attrs } => {
        let cond = emit_bool(test);
        let then_av = emit_avpairs(then_attrs);
        let else_av = emit_avpairs(else_attrs);
        quote!(
          if #cond {
            #(#then_av)*
          } else {
            #(#else_av)*
          }
        )
      },
    })
    .collect()
}

/// A quoted attribute value, lowered to a `[piece, …].join("")` expression
/// (Perl `translate_string`).
fn emit_attr_value(av: &AttrValue) -> TokenStream2 {
  let pieces: Vec<TokenStream2> = av
    .parts
    .iter()
    .map(|p| match p {
      AttrPart::Literal(s) => {
        let escaped = slashify(s);
        quote!(#escaped.to_string())
      },
      AttrPart::Value(v) => {
        let ve = emit_value(v, false);
        quote!(match #ve { Some(ref val) => val.to_attribute(), None => String::new() })
      },
      AttrPart::Conditional { test, then_val, else_val } => {
        let cond = emit_bool(test);
        let tv = emit_value(then_val, false);
        let ev = emit_value(else_val, false);
        quote!(match (if #cond { #tv } else { #ev }) {
          Some(ref val) => val.to_attribute(),
          None => String::new()
        })
      },
    })
    .collect();
  quote!([#(#pieces),*].join(""))
}

/// A substitutable value (Perl `translate_value`). `for_test` wraps `#n` in an
/// `Option` for boolean contexts (the conditional test).
fn emit_value(v: &Value, for_test: bool) -> TokenStream2 {
  match v {
    Value::Arg(n) => {
      let n1 = *n; // 1-based
      let n0 = n - 1; // 0-based
      if for_test {
        quote!(if args.len() < #n1 { &None } else { &args[#n0] })
      } else {
        quote!(&args[#n0])
      }
    },
    Value::Prop(name) => quote!(props.get(#name)),
    Value::Func { name, args } => {
      let fid = Ident::new(name, Span::call_site());
      let fargs = args.iter().map(|fa| emit_func_arg(fa, for_test));
      quote!(#fid( #(#fargs),* ))
    },
    Value::Literal(s) => {
      let escaped = slashify(s);
      quote!(#escaped)
    },
  }
}

fn emit_func_arg(fa: &FuncArg, for_test: bool) -> TokenStream2 {
  match fa {
    FuncArg::Value(v) => emit_value(v, for_test),
    FuncArg::Str(av) => emit_attr_value(av),
  }
}

/// The truth test of a conditional (Perl `parse_conditional`'s bool branch).
fn emit_bool(test: &Value) -> TokenStream2 {
  let tv = emit_value(test, true);
  quote!(match #tv {
    None => false,
    Some(ref v) => {
      let v_str = v.to_string();
      !v_str.is_empty() && v_str != "false"
    }
  })
}
