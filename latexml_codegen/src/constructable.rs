use once_cell::sync::Lazy;
use regex::{Captures, Regex};

use proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;

use latexml_core::util::text::*;

// We recognize several special operators:
//  #      #number|name      accesses an argument to or property of the whatsit
//  ? ( )  ?test(if)(else)   a conditional, test
//  & ,    &func(arg,...)    replaces by result of function call
//  < >    <qname attr...>   generates xml tag
//  ^      ^ pattern         floats to where pattern would be allowed
// Each of these can be used literally, if DOUBLED (ie. ## )
// (except ^ ??? ^^ means something special!)
#[rustfmt::skip]
macro_rules! QNAME_RE_STR(() => (r"((?:\p{Ll}|\p{Lu}|\p{Lo}|\p{Lt}|\p{Nl}|_|:)(?:\p{Ll}|\p{Lu}|\p{Lo}|\p{Lt}|\p{Nl}|_|:|\p{M}|\p{Lm}|\p{Nd}|\.|-)*)"));
macro_rules! PI_RE_STR(() => (concat!(r"^\s*<\?",QNAME_RE_STR!())));
macro_rules! KEY_RE_STR (() => (concat!(r"^",QNAME_RE_STR!(),r"\s*=\s*")));
macro_rules! VALUE_RE_STR (() => (r"(#[\w]+|&[\w:]*\()"));
macro_rules! LEAD_VALUE_RE_STR (() => (concat!(r"^",VALUE_RE_STR!())));
macro_rules! COND_RE_STR (() => (concat!(r"[?]", VALUE_RE_STR!())));
macro_rules! LEAD_COND_RE_STR (() => (concat!(r"^",COND_RE_STR!())));
macro_rules! OPEN_TAG_RE_STR (() => (concat!(r"\s*<", QNAME_RE_STR!())));
macro_rules! CLOSE_TAG_RE_STR (() => (concat!(r"</", QNAME_RE_STR!(),r"\s*>")));
macro_rules! QNAME_KEY_RE_STR (() => (concat!(r"^", QNAME_RE_STR!(),r"\s*=\s*")));
macro_rules! LEAD_OPEN_TAG_RE_STR (() => (concat!(r"^", OPEN_TAG_RE_STR!())));
macro_rules! LEAD_CLOSE_TAG_RE_STR (() => (concat!(r"^", CLOSE_TAG_RE_STR!())));

macro_rules! SPECIALS (() => (r"\#\?&\\"));
// Quoted special characters (or semi-special)
// Includes: \X where X is a special, ##, \&amp;, &amp;,
// and \word (backslash + letters = TeX CS in text position, treated as literal text).
macro_rules! QUOTED_SPECIALS (
    () => (concat!(r"\\\\\\[",SPECIALS!(),
                   r"]|\\#\\#|\\&amp;|&amp;",
                   r"|\\[a-zA-Z@]+"  // \textbf, \textit etc. — literal text insertion
                   ))); // or special cases: doubled #, \&amp;, &amp;
macro_rules! LEAD_QUOTED_RE_STR (() => (
  concat!(r"^((",QUOTED_SPECIALS!(),r"|[^",SPECIALS!(),"'\"])+)")));
macro_rules! LEAD_RANDOM_TEXT_RE_STR (() => (
  concat!(r"^((",QUOTED_SPECIALS!(),r"|[^",SPECIALS!(),r"<])+)")));

// These recognize the beginnings of value expressions, conditionals, ..
// Attempt to follow XML Spec, Appendix B
static LEAD_VALUE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(LEAD_VALUE_RE_STR!()).unwrap());
static LEAD_COND_RE: Lazy<Regex> = Lazy::new(|| Regex::new(LEAD_COND_RE_STR!()).unwrap());
static LEAD_QMARK: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[?]").unwrap());
static LEAD_OPEN_TAG_RE: Lazy<Regex> = Lazy::new(|| Regex::new(LEAD_OPEN_TAG_RE_STR!()).unwrap());
static LEAD_CLOSE_TAG_RE: Lazy<Regex> = Lazy::new(|| Regex::new(LEAD_CLOSE_TAG_RE_STR!()).unwrap());
static LEAD_QUOTED_RE: Lazy<Regex> = Lazy::new(|| Regex::new(LEAD_QUOTED_RE_STR!()).unwrap());
static LEAD_RANDOM_TEXT_RE: Lazy<Regex> =
  Lazy::new(|| Regex::new(LEAD_RANDOM_TEXT_RE_STR!()).unwrap());
// QName (element tags, attribute names);  Could this also allow expressions?
static QNAME_KEY_RE: Lazy<Regex> = Lazy::new(|| Regex::new(QNAME_KEY_RE_STR!()).unwrap());
static FLOAT_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(\^+)\s*").unwrap());
static PI_RE: Lazy<Regex> = Lazy::new(|| Regex::new(PI_RE_STR!()).unwrap());
static PI_CLOSE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\s*\?>").unwrap());
static KEY_RE: Lazy<Regex> = Lazy::new(|| Regex::new(KEY_RE_STR!()).unwrap());
static FN_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^&([\w:]*)\(").unwrap());
static LEAD_CPAREN_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\s*\)").unwrap());
static LEAD_KV_SEP: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\s*,\s*").unwrap());
static ARG_HOLE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^#(\d+)").unwrap());
static PROP_HOLE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\#([\w_-]+)").unwrap());
static ESCAPED_OP: Lazy<Regex> = Lazy::new(|| Regex::new(r"\\[#\?(&,<>\\%]").unwrap());

pub fn compile_replacement(input: DeriveInput) -> TokenStream {
  let replacement = crate::attr_name_value_str(&input.attrs[0], "replacement");

  let compiled_replacement_closure: proc_macro2::TokenStream = if replacement.is_empty() {
    quote!(None)
  } else {
    // Following the original LaTeXML Compiler, we'll mutate this string in place,
    // cloning for safety. since this is all happening in Rust's compilation
    // step, the clone causes no major overhead. If we refactor away the
    // mutable borrows we do for in-place modification, we can avoid a lot of the
    // cloning, and stay conservative in memory. For now it shouldn't matter.

    // println!("Compiling: \n{:?}", &replacement);
    let mut operations = Vec::new();

    operations.extend(compile_replacement_tokens(replacement));

    // println!(
    //   "Into: \n{}",
    //   operations
    //     .iter()
    //     .map(|x| x.to_string())
    //     .collect::<Vec<_>>()
    //     .join("\n")
    // );

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

fn compile_replacement_tokens(mut replacement: String) -> Vec<proc_macro2::TokenStream> {
  // NO TRIM!
  // one replacement has "<ltx:break/>\n" with the "\n" being tested for.
  // let trimmed = replacement.trim();
  // if trimmed != replacement {
  //   replacement = trimmed.to_string();
  // }
  let mut floats: String = String::new();
  let mut has_floats: bool = false;
  let float_res = FLOAT_RE.replace(&replacement, |refs: &Captures| -> String {
    floats = refs.get(1).map_or("", |m| m.as_str()).trim().to_string();
    has_floats = true;
    String::new()
  });
  if has_floats {
    replacement = float_res.to_string();
  }
  let mut operations = Vec::new();
  let mut _iter_count = 0u32;

  while !replacement.is_empty() {
    _iter_count += 1;
    if _iter_count > 1000 {
      panic!(
        "compile_replacement_tokens: infinite loop detected after 1000 iterations. Remaining template: {:?}",
        &replacement[..replacement.len().min(200)]
      );
    }
    let mut current_tag = String::new();

    // ?test(ifclause)(elseclause)
    if LEAD_COND_RE.is_match(&replacement) {
      let (bool_branch, if_branch, else_branch) = parse_conditional(&mut replacement);
      let if_branch_compiled = compile_replacement_tokens(if_branch);
      let else_branch_compiled = compile_replacement_tokens(else_branch);

      operations.push(quote!(
        if #bool_branch {
          #(#if_branch_compiled)*
        } else {
          #(#else_branch_compiled)*
        }
      ));
      continue;
    }

    // Processing instruction: <?name a=v ...?>
    let mut is_match = false;
    let pi_result = PI_RE.replace(&replacement, |refs: &Captures| -> String {
      current_tag = refs.get(1).map_or("", |m| m.as_str()).to_string();
      is_match = true;
      String::new()
    });

    if is_match {
      replacement = pi_result.to_string();
      // println!("-- matched a PI ");
      // this is annoying since we want translate_avpairs to mutate the replacement
      // string in place, but also want it to run after the replacement...
      // makes `current_tag` in particular look very misplaced
      let av = translate_avpairs(&mut replacement);
      if av.is_empty() {
        operations.push(quote!(
          document.insert_pi(#current_tag, None)?;
        ));
      } else {
        operations.push(quote!(
          let mut av_props : HashMap<String, String> = HashMap::default();
          #(#av)*
          document.insert_pi(#current_tag, Some(av_props))?;
        ));
      }

      let mut pi_closed = false;
      replacement = PI_CLOSE_RE
        .replace(&replacement, |_: &Captures| -> String {
          pi_closed = true;
          String::new()
        })
        .to_string();

      if !pi_closed {
        panic!("Missing '?>' at '{replacement:?}'\n");
      }
      continue;
    }

    // Open tag: <name a=v ...> or .../> (for empty element)
    replacement = LEAD_OPEN_TAG_RE
      .replace(&replacement, |refs: &Captures| -> String {
        is_match = true;
        current_tag = refs.get(1).map_or("", |m| m.as_str()).to_string();
        // println!("-- open tag {:?}", current_tag);
        String::new()
      })
      .to_string();

    // handle open tag
    if is_match {
      let av = translate_avpairs(&mut replacement);
      if has_floats {
        let float_type = floats.len();
        if float_type == 1 {
          operations.push(quote!(savenode = document.float_to_element(#current_tag, false)?;));
        } else if float_type == 2 {
          operations.push(quote!(savenode = document.float_to_element(#current_tag, true)?;));
        }
        has_floats = false;
        floats = String::new();
      }
      // Fonts require a bit too much boilerplate at the moment, due to having
      // different directives (code vs asset) and trying to avoid deep cloning.
      if av.is_empty() {
        operations.push(quote!(
          document.open_element(#current_tag, None, None)?;
        ));
      } else {
        operations.push(quote!(
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
            document.open_element(#current_tag, Some(av_props), Some(&this_font))?;
          } else {
            document.open_element(#current_tag, Some(av_props), None)?;
          }
        ));
      }
      // Empty element?
      if replacement.starts_with('/') {
        operations.push(quote!(document.close_element(#current_tag)?;));
        replacement.remove(0);
      }
      if replacement.starts_with('>') {
        replacement.remove(0);
      } else {
        panic!("Missing '>' at '{replacement:?}'");
      }
      continue;
    }

    // Close tag: </name>
    let lead_close_tag_result =
      LEAD_CLOSE_TAG_RE.replace(&replacement, |refs: &Captures| -> String {
        is_match = true;
        current_tag = refs.get(1).map_or("", |m| m.as_str()).to_string();
        // println!("-- close tag {:?}", current_tag);
        // handle close tag
        operations.push(quote!(document.close_element(#current_tag)?;));
        String::new()
      });
    if is_match {
      replacement = lead_close_tag_result.to_string();
      continue;
    }

    // Substitutable value: argument, property...
    if LEAD_VALUE_RE.is_match(&replacement) {
      let to_absorb = translate_value("", false, &mut replacement);
      // TODO: What is the cleanest interface for dealing with Alignment?
      // when Stored, we wrap with Rc<RefCell<_>> which we can't unwrap without cloning
      // for the Digested variant.
      // (which is Digested(Rc<DigestedData<RefCell<Alignment>>>))
      // We could *either* use the Stored::Digested form *always*, or instead
      // try to store a RefCell without "Rc" and use it without ever cloning out...
      // for now, handle as a special case:

      // CONTINUE: Actually yes
      // let us switch to using Digested() consistently. Then check using:
      // if let Some(alignment) = some_stored.as_alignment() { ... }
      //
      operations.push(quote!(
        if let Some(ref stored_digested) = #to_absorb {
          let digested_opt : Option<Digested> = stored_digested.into();
          if let Some(ref digested) = digested_opt {
            document.absorb(digested, None)?;
          }
        }
      ));
      continue;
    }

    // Attribute: a='v' at the top level — typically prefixed by `^` to
    // float to an ancestor accepting that attribute. Perl: Compiler.pm
    // L137-148. Without `^`, falls back to setting the attribute on the
    // current node (matches Perl's `$$self{node}` semantics).
    let mut attr_key = String::new();
    let qname_key_result = QNAME_KEY_RE.replace(&replacement, |refs: &Captures| -> String {
      is_match = true;
      attr_key = refs.get(1).map_or("", |m| m.as_str()).to_string();
      String::new()
    });
    if is_match {
      replacement = qname_key_result.to_string();
      let val = translate_string(&mut replacement);
      let key_str = attr_key.clone();
      if has_floats {
        operations.push(quote!(
          {
            let val_str: String = #val;
            savenode = document.float_to_attribute(#key_str);
            let mut node = document.get_node().clone();
            document.set_attribute(&mut node, #key_str, &val_str)?;
            if let Some(ref sn) = savenode { document.set_node(sn); }
          }
        ));
        has_floats = false;
        floats = String::new();
      } else {
        operations.push(quote!(
          {
            let val_str: String = #val;
            let mut node = document.get_node().clone();
            document.set_attribute(&mut node, #key_str, &val_str)?;
          }
        ));
      }
      continue;
    }

    // Else random text
    let mut has_random_text = false;
    let lead_random_text_result =
      LEAD_RANDOM_TEXT_RE.replace(&replacement, |refs: &Captures| -> String {
        if let Some(text_match) = refs.get(1) {
          let escaped_match = &slashify(&unquote(text_match.as_str()));
          operations.push(quote!(
            document.absorb_string(#escaped_match, props)?;
          ));
        }
        has_random_text = true;
        String::new()
      });
    if has_random_text {
      replacement = lead_random_text_result.to_string();
    }
  }

  operations
}

// Parse a delimited string from the constructor (in $_),
// for example, an attribute value.  Can contain substitutions (above), as if
// interpolated. The result is a string, or undef if no quotes are found.
// NOTE: UNLESS there is ONLY one substituted value, then return the value
// object. This is (hopefully) temporary to handle font objects as attributes.
// The DOM holds the font objects, rather than strings,
// to resolve relative fonts on output.
fn translate_string(text: &mut String) -> proc_macro2::TokenStream {
  // println!("-- ts before: {:?}", text);
  let mut values: Vec<proc_macro2::TokenStream> = Vec::new();
  trim_start_in_place(&mut *text);
  if text.starts_with('\'') || text.starts_with('"') {
    let quote = text.remove(0);
    while !text.is_empty() && !text.starts_with(quote) {
      if LEAD_COND_RE.is_match(text) {
        // inline conditional; branches should be values
        let (bool_branch, mut if_branch, mut else_branch) = parse_conditional(text);
        let if_branch_translated = translate_value("", false, &mut if_branch);
        let else_branch_translated = translate_value("", false, &mut else_branch);
        let op = quote!(
          if #bool_branch {
            #if_branch_translated
          } else {
            #else_branch_translated
          }
        );
        values.push(op);
      } else if LEAD_VALUE_RE.is_match(text) {
        values.push(translate_value(&quote.to_string(), false, text));
      } else {
        let mut is_quoted_match = false;
        let mut quoted_match = String::new();
        *text = LEAD_QUOTED_RE
          .replace(text, |refs: &Captures| -> String {
            quoted_match = refs.get(1).map_or("", |m| m.as_str()).to_string();
            is_quoted_match = true;
            String::new()
          })
          .to_string();
        if is_quoted_match {
          let escaped_match = &slashify(&unquote(&quoted_match));
          values.push(quote!(#escaped_match));
        } else {
          panic!("Unrecognized at '{text:?}'\n");
        }
      }
    }
  }
  // drop last quote
  if !text.is_empty() {
    text.remove(0);
  }

  let token_values = values
    .iter()
    .map(|v| {
      let v_str = v.to_string();
      if v_str.starts_with('\'') || v_str.starts_with('"') {
        quote!(#v.to_string())
      } else if v_str.ends_with(". to_string ( ) ") {
        quote!(#v)
      } else {
        quote!(match #v {
          Some(ref val) => val.to_attribute(),
          None => String::new()
        })
      }
    })
    .collect::<Vec<_>>();
  quote!([#(#token_values),*].join(""))
}

fn translate_avpairs(text: &mut String) -> Vec<proc_macro2::TokenStream> {
  // Parse a set of attribute value pairs from a constructor pattern,
  // substituting argument and property values from the whatsit.
  let mut avs: Vec<proc_macro2::TokenStream> = Vec::new();
  trim_start_in_place(&mut *text);
  while !text.is_empty() {
    let mut is_match = false;
    let mut key = String::new();
    if LEAD_COND_RE.is_match(text) {
      is_match = true;
      let (bool_branch, mut if_branch, mut else_branch) = parse_conditional(text);
      let if_branch_translated = translate_avpairs(&mut if_branch);
      let else_branch_translated = translate_avpairs(&mut else_branch);
      let op = quote!(
        if #bool_branch {
          #(#if_branch_translated)*
        } else {
          #(#else_branch_translated)*
        }
      );
      avs.push(op);
      // elsif (/^%$VALUE_RE/) {    # Hash?  Assume the value can be turned into
      // a hash!       s/^%//;                  # Eat the "%"
      //       push(@avs, '%{' . translate_value() . '}'); }
    }
    if !is_match {
      *text = KEY_RE
        .replace(text, |refs: &Captures| -> String {
          key = refs.get(1).map_or("", |m| m.as_str()).to_string();
          is_match = true;
          String::new()
        })
        .to_string();
      if is_match {
        let val = translate_string(text);
        if key == "font" {
          // we handle font in a special case
          avs.push(quote!(();));
        } else {
          avs.push(quote!(av_props.insert(#key.to_string(), #val);))
        };
      }
    }
    if !is_match {
      break;
    }
    trim_start_in_place(&mut *text);
  }
  avs
}

/// Parse a substitutable value from the constructor (in $_)
/// Recognizes the #1, #prop, and also &function(args,...)
/// Note: signals an error if no recognizable value was found!
fn translate_value(
  exclude_chars: &str,
  for_test: bool,
  text: &mut String,
) -> proc_macro2::TokenStream {
  let mut val = quote!("");
  let mut is_match = false;
  let mut fcn = String::new();
  // Recognize a function call, w/args
  *text = FN_RE
    .replace(text, |refs: &Captures| -> String {
      refs.get(1).map_or("", |m| m.as_str()).clone_into(&mut fcn);
      is_match = true;
      String::new()
    })
    .to_string();
  if is_match {
    let mut args = Vec::new();
    while !LEAD_CPAREN_RE.is_match(text) {
      let ttl = text.trim_start();
      let quoted_follows = ttl.starts_with('\'') || ttl.starts_with('\"');

      let arg = if quoted_follows {
        translate_string(text)
      } else {
        translate_value(",)", for_test, text)
      };
      args.push(arg);
      let mut intermediate_kv = false;
      *text = LEAD_KV_SEP
        .replace(text, |_: &Captures| {
          intermediate_kv = true;
          String::new()
        })
        .to_string();
      if !intermediate_kv {
        break;
      }
    }
    trim_start_in_place(&mut *text);
    if text.starts_with(')') {
      text.remove(0);
    } else {
      panic!("Missing ')' in &$fcn(...) at '{text:?}'\n");
    }
    // println!("text after translate_value: {:?}", text);
    let fcn_ident = proc_macro2::Ident::new(&fcn, proc_macro2::Span::call_site());
    val = quote!(#fcn_ident( #(#args),* ));
  }

  if !is_match {
    // Recognize an explicit #1 for whatsit args
    *text = ARG_HOLE_RE
      .replace(text, |arg_refs: &Captures| {
        is_match = true;
        let n = arg_refs.get(1).map_or("", |m| m.as_str()).to_string();
        let n_int = n.parse::<i32>().unwrap_or(-1);
        if !(1..=9).contains(&n_int) {
          panic!("Illegal argument number {n_int:?} at '{text:?}'\n");
        } else {
          // index starts at 0
          // if we need the argument for a test such as `?#1(yes)(no)`
          // make sure we yield an Option<T> instead of T.
          let n_lit_usize = n_int as usize;
          let n_usize: usize = (n_int - 1) as usize;
          val = if for_test {
            quote!(if args.len() < #n_lit_usize { &None } else { &args[#n_usize] })
          } else {
            quote!(&args[#n_usize])
          }
        }
        String::new()
      })
      .to_string();
  }
  if !is_match {
    *text = PROP_HOLE_RE
      .replace(text, |prop_refs: &Captures| {
        let prop_name = prop_refs.get(1).map_or("", |m| m.as_str()).to_owned();
        is_match = true;
        // Recognize #prop for whatsit properties
        val = quote!(props.get(#prop_name));

        String::new()
      })
      .to_string();
  }
  if !is_match {
    // Build the exclusion regex
    let mut exclusion_str: String =
      concat!(r"^((?:", QUOTED_SPECIALS!(), r"|[^", SPECIALS!()).to_owned();
    exclusion_str = exclusion_str + exclude_chars + r"])+)";
    let exclusion_re: Regex = Regex::new(&exclusion_str).unwrap();

    *text = exclusion_re
      .replace(text, |quoted_refs: &Captures| {
        is_match = true;
        let quoted = quoted_refs.get(1).map_or("", |m| m.as_str()).to_owned();
        let normalized_val = &slashify(&unquote(&quoted));
        val = quote!(#normalized_val);
        String::new()
      })
      .to_string();
  }
  if !is_match {
    panic!("Missing value at '{text:?}'\n");
  }
  val
}

fn parse_conditional(text: &mut String) -> (proc_macro2::TokenStream, String, String) {
  // Remove leading "?"
  *text = LEAD_QMARK
    .replace(text, |_: &Captures| String::new())
    .to_string();
  let translated_bool = translate_value("(", true, text);
  // Note/TODO: This is a direct redo of latexml's "ToString(v) ? () : ()" approach
  //   for testing the boolean branch
  //   we could make it more performant by defining a simple trait `.to_bool`
  //   and implementing it for all Core objects that may be passed in as arguments
  //   currently Stored::Bool(false) will serialize to "false" so we need an extra check...
  let bool_branch = quote!(  match #translated_bool { None => false, Some(ref v) => {
    let v_str = v.to_string();
    !v_str.is_empty() && v_str != "false" }});
  let if_branch_opt = extract_bracketed(text, Some(&Delimiter::Parenthesis));
  if let Some(if_branch) = if_branch_opt {
    let else_branch = extract_bracketed(text, Some(&Delimiter::Parenthesis)).unwrap_or_default();
    // println!("-- cond after with else: {:?}", text);
    (bool_branch, if_branch, else_branch)
  } else {
    // println!("-- cond after malformed: {:?}", text);
    (bool_branch, String::new(), String::new())
  }
}

fn slashify(text: &str) -> String { text.replace('\\', "\\\\") }
fn unquote(text: &str) -> String {
  ESCAPED_OP
    .replace_all(text, |escaped_refs: &Captures| -> String {
      escaped_refs.get(1).map_or("", |m| m.as_str()).to_string()
    })
    .replace("##", "#")
    .replace("&amp;", "&")
}
