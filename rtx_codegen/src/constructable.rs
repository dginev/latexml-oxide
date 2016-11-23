use syn;
use quote;
use regex::{Captures, Regex};
use util::{get_options_from_input, get_option};
// impl Constructor {
//   pub fn compile_replacement(&self) -> Option<ReplacementClosure> {
//     if self.replacement.is_empty() {
//       return None;
//     }

//     let cs = self.get_cs();
//     let name = NONW_RE.replace_all(&self.get_cs_name(), "");
//     let nargs = self.get_num_args();

//     let mut floats: Option<String> = None;
//     let replacement = FLOAT_RE.replace(&self.replacement, |caps: &Captures| {
//       floats = match caps.at(1) { // Grab float marker.
//         None => None,
//         Some(subs) => Some(subs.to_owned()),
//       };
//       String::new()
//     });

//     // println_stderr!("-- Preparing translation closure for: \n{:?}\n",
//     //                 replacement);
//     Some(Arc::new(|document, args, props, state| {
//       let mut savenode: Option<Node> = None;
//       TranslateConstructor!(replacement, floats, savenode);
//       match savenode {
//         None => {}
//         Some(savenode) => document.set_node(savenode),
//       };
//       return;
//     }))
//   }
// }

macro_rules! QNAME_RE_STR(
  () => (r"((?:\p{Ll}|\p{Lu}|\p{Lo}|\p{Lt}|\p{Nl}|_|:)(?:\p{Ll}|\p{Lu}|\p{Lo}|\p{Lt}|\p{Nl}|_|:|\p{M}|\p{Lm}|\p{Nd}|\.|-)*)")
);

macro_rules! PI_RE_STR(
  () => (concat!(r"^\s*<\?",QNAME_RE_STR!()))
);

macro_rules! KEY_RE_STR (
    () => (concat!(r"^",QNAME_RE_STR!(),r"\s*=\s*"))
);

macro_rules! VALUE_RE_STR (
    () => (r"(\#|&[\w:]*\()")
);

macro_rules! LEAD_VALUE_RE_STR (
    () => (concat!("^",VALUE_RE_STR!()))
);
macro_rules! SPECIALS (
     () => ("#?&\\")
);
// Quoted special characters (or semi-special)
macro_rules! QUOTED_SPECIALS (
    () => (concat!("\\\\\\#|\\\\\\?|\\\\\\(|\\\\\\)|\\\\\\&|\\\\\\,|\\\\\\<|\\\\\\>|\\\\\\\\|\\\\\\%",
                   // or special cases: doubled #, &amp;
                   "|\\#\\#|\\&amp;"))
);

lazy_static! {
  static ref NARGS : i32 = 0;
  static ref VALUE_RE : Regex = Regex::new(VALUE_RE_STR!()).unwrap();
  static ref LEAD_VALUE_RE : Regex = Regex::new(LEAD_VALUE_RE_STR!()).unwrap();
//   static ref COND_RE : Regex = Regex::new(r"\?(\#|\&[\w\:]*\()").unwrap();
// // Attempt to follow XML Spec, Appendix B
//   static ref QNAME_RE : Regex = Regex::new(QNAME_STR!()).unwrap();
//   static ref TEXT_RE : Regex = Regex::new(r"(.[^\#<\?\)\&\,]*)").unwrap();
//   static ref NONW_RE : Regex = Regex::new(r"\W").unwrap();
  static ref FLOAT_RE : Regex = Regex::new(r"^(\^+)\s*").unwrap();
  static ref PI_RE : Regex = Regex::new(PI_RE_STR!()).unwrap();
  static ref KEY_RE : Regex = Regex::new(KEY_RE_STR!()).unwrap();
  static ref FN_RE : Regex = Regex::new(r"^&([\w:]*)\(").unwrap();
  static ref LEAD_CPAREN_RE : Regex = Regex::new(r"^\s*\)").unwrap();
  static ref LEAD_KV_SEP : Regex = Regex::new(r"^\s*\,\s*").unwrap();
  static ref ARG_HOLE_RE : Regex = Regex::new(r"^#(\d+)").unwrap();
  static ref ESCAPED_OP : Regex = Regex::new(r"\\(\#|\?|\(|\)|\&|\,|\<|\>|\\|%)").unwrap();
}

pub fn compile_replacement(input: syn::MacroInput) -> quote::Tokens {
  fn bug() -> ! {
      panic!("This is a bug. Please open a Github issue \
             with your DefConstructor invocation");
  }
  let options = get_options_from_input(&input.attrs, bug);
  let replacement_opt = options.map(|o| get_option(&o, "replacement", bug));
  let compiled_replacement_closure = match replacement_opt {
    None => quote!(None),
    Some(replacement) => {
      let mut operations = vec![quote!(println_stderr!("-- replacement mock executed. \n args: {:?}", args);)];

      // Following the original LaTeXML Compiler, we'll mutate this string in place, cloning for safety.
      // since this is all happening in Rust's compilation step, the clone causes no major overhead.
      // If we refactor away the mutable borrows we do for in-place modification, we can avoid a lot of the
      // cloning, and stay conservative in memory. For now it shouldn't matter.
      println!("Compiling: {:?}", &replacement);
      let mut replacement = replacement.to_owned();
      let mut floats: String = String::new();
      let mut has_floats: bool = false;
      replacement = FLOAT_RE.replace_all(&replacement, |refs: &Captures| -> String {
        floats = refs.at(1).unwrap_or("").to_owned();
        has_floats = true;
        String::new()
      });

      while !replacement.is_empty() {
        let mut consumed = String::new();
        // TODO: Is there a better way to write code conditional on triggered .replace?
        let mut is_match = false;
        let mut pi_tag = String::new();

        // Processing instruction: <?name a=v ...?>
        replacement = PI_RE.replace(&replacement, |refs: &Captures| -> String {
          pi_tag = refs.at(1).unwrap_or("").to_owned();
          let (_, match_end) = refs.pos(1).unwrap();
          consumed = replacement[0..match_end].to_owned();
          is_match = true;
          String::new()
        });
        if is_match {
          // println_stderr!("-- matched a PI ");
          // this is annoying since we want translate_avpairs to mutate the replacement string in place,
          // but also want it to run after the replacement... makes `pi_tag` in particular look very misplaced
          let av = translate_avpairs(&mut replacement);
          operations.push(quote!(doc.insert_pi(#pi_tag,vec![#(av),*]); ));
        }

        // Close tag: </name>
        if !is_match && replacement.starts_with("</") {
          // println_stderr!("-- close tag");
          // consumed = "</".to_owned();
        }
        // Open tag: <name a=v ...> or .../> (for empty element)
        if !is_match && replacement.starts_with("<") {
          // println_stderr!("-- open tag");
          // consumed = "<".to_owned();
        }
        // Substitutable value: argument, property...
        if !is_match && replacement.starts_with("#") {
          // println_stderr!("-- argument hole");
          // consumed = "#".to_owned();
        }
        // Attribute: a=v; assigns in current node? [May conflict with random replacement!?!]
        if !is_match && replacement.find("=").is_some() {
          // println_stderr!("-- Attribute");
          consumed = replacement[0..1 + replacement.find("=").unwrap()].to_owned();
        }
        // Else random text
        else {
          // println_stderr!("-- random text");
          consumed = replacement[0..1].to_owned();
        }
        // println!("consumed: {:?}", consumed);
        replacement = replacement[consumed.len()..].to_owned();
      }
      println!("Body operations: \n{:?}",
        operations.iter().map(|x| x.to_string()).collect::<Vec<_>>().join("\n"));

      quote!(
        Some(Arc::new(
        |doc: &mut Document, args: &Vec<TBox>, _props: &HashMap<String, String>, _state: &mut State| {
          #(operations)*
        }))
      )
    }
  };


  // We have to jump an extra hoop, since we are forcing the struct-derive mechanism. Once the new procedural macro scheme lands, this begs to be refactored.
  quote!(
    impl _Dummy {
      fn replacement() -> Option<ReplacementClosure> {
        #compiled_replacement_closure
      }
    })
}


// Parse a delimited string from the constructor (in $_),
// for example, an attribute value.  Can contain substitutions (above), as if interpolated.
// The result is a string, or undef if no quotes are found.
// NOTE: UNLESS there is ONLY one substituted value, then return the value object.
// This is (hopefully) temporary to handle font objects as attributes.
// The DOM holds the font objects, rather than strings,
// to resolve relative fonts on output.
fn translate_string(mut text : &mut String) -> quote::Tokens {
  let mut values : Vec<quote::Tokens> = Vec::new();
  *text = text.trim_left().to_owned();
  if text.starts_with('\'') || text.starts_with('"') {
    let quote = text.remove(0);
    while !text.is_empty() && !text.starts_with(quote) {
      // if (/^$COND_RE/o) {    # inline conditional; branches should be values
      //   my ($bool, $if, $else) = parse_conditional();
      //   my $code = "($bool ?";
      //   { local $_ = $if; $code .= translate_value(); }
      //   $code .= ":";
      //   if ($else) { local $_ = $else; $code .= translate_value(); }
      //   else       { $code .= "''"; }
      //   $code .= ")";
      //   push(@values, $code); }
      if LEAD_VALUE_RE.is_match(text) {
        values.push(translate_value(&quote.to_string(), &mut text));
      }
      // else if (s/^((?:$QUOTED_SPECIALS|[^\Q$SPECIALS$quote\E])+)//s) {
      //   push(@values, "'" . slashify!(unquote!($1)) . "'"); }
      else {
        panic!("Unrecognized at '{:?}'\n", text);
      }
    }
  }

  let token_values = values.iter().map(|v| if v.to_string().starts_with('\'') {quote!(#v)} else {quote!(&#v.to_string())})
    .collect::<Vec<_>>();
  quote!(#(token_values)+*)
}


fn translate_avpairs(mut text: &mut String) -> Vec<quote::Tokens> {
  // Parse a set of attribute value pairs from a constructor pattern,
  // substituting argument and property values from the whatsit.
  let mut avs : Vec<quote::Tokens> = Vec::new();
  *text = text.trim_left().to_owned();
  while !text.is_empty() {
    *text = text.trim_left().to_owned();
    let mut is_match = false;
    let mut key = String::new();
  //     if (/^$COND_RE/o) {
  //       my ($bool, $if, $else) = parse_conditional();
  //       my $code = "($bool ? (";
  //       { local $_ = $if; $code .= translate_avpairs(); }
  //       $code .= ") : (";
  //       { local $_ = $else; $code .= translate_avpairs() if $else; }
  //       $code .= "))";
  //       push(@avs, $code); }
  //     elsif (/^%$VALUE_RE/) {    # Hash?  Assume the value can be turned into a hash!
  //       s/^%//;                  # Eat the "%"
  //       push(@avs, '%{' . translate_value() . '}'); }
    if !is_match {
      *text = KEY_RE.replace(text, |refs: &Captures| -> String {
        key = refs.at(1).unwrap_or("").to_owned();
        is_match = true;
        String::new()
      });
      if is_match {
        let val = translate_string(&mut text);
        avs.push(quote!(#key));
        avs.push(val);
      }
    }
    if !is_match {
      break
    }
  }
  avs
}

/// Parse a substitutable value from the constructor (in $_)
/// Recognizes the #1, #prop, and also &function(args,...)
/// Note: signals an error if no recognizable value was found!
fn translate_value(exclude_chars : &str, mut text : &mut String) -> quote::Tokens {
  let mut val = quote!("");
  let mut is_match = false;
  let mut fcn = String::new();
  // Recognize a function call, w/args
  *text = FN_RE.replace(text, |refs: &Captures| -> String {
    fcn  = refs.at(1).unwrap_or("").to_owned();
    is_match = true;
    String::new()
  });
  if is_match {
    let mut args = Vec::new();
    while ! LEAD_CPAREN_RE.is_match(text) {
      let quoted_follows;
      { let ttl = text.trim_left(); // need an immutable borrow of text, so wrapping in a block
        quoted_follows = ttl.starts_with("\'") || ttl.starts_with("\"");
      }
      let arg = if quoted_follows {
        translate_string(&mut text)
      } else {
        translate_value(",)", &mut text)
      };
      args.push(arg);
      let mut intermediate_kv = false;
      *text = LEAD_KV_SEP.replace(text, |_: &Captures| {
        intermediate_kv = true;
        String::new()
      });
      if !intermediate_kv {
        break;
      }
    }
    *text = text.trim_left().to_owned();
    if text.starts_with(")") {
      text.remove(0);
    } else {
      panic!("Missing ')' in &$fcn(...) at '{:?}'\n",text);
    }
    val = quote!(#fcn( #(args),* ));
  }

  if !is_match {
    // Recognize an explicit #1 for whatsit args
    *text = ARG_HOLE_RE.replace(text, |arg_refs: &Captures| {
      is_match = true;
      let n = arg_refs.at(1).unwrap_or("").to_owned();
      let n_int = n.parse::<i32>().unwrap_or(-1);
      if n_int < 1 {//|| (n_int > *NARGS) {
        panic!("Illegal argument number {:?} at '{:?}'\n", n_int, text);
      } else {
        let n_usize : usize = (n_int - 1) as usize; // index starts at 0
        val = quote!(args[#n_usize])
      }
      String::new()
    });
  }
  // TODO:
  // elsif (s/^\#([\w\-_]+)//) { $value = "\$prop{'$1'}"; }    # Recognize #prop for whatsit properties
  if !is_match {
    // Build the exclusion regex
    let mut exclusion_str : String = concat!(r"^((?:",QUOTED_SPECIALS!(),r"|[^\Q", SPECIALS!()).to_owned();
    exclusion_str = exclusion_str + exclude_chars + r"\E])+)";
    let exclusion_re : Regex = Regex::new(&exclusion_str).unwrap();

    exclusion_re.replace(text, |quoted_refs: &Captures| {
      is_match = true;
      let quoted = quoted_refs.at(1).unwrap_or("").to_owned();
      let normalized_val = &slashify(&unquote(&quoted));
      val = quote!(#normalized_val);
    String::new()
    });
  }
  if !is_match {
    panic!("Missing value at '{:?}'\n", text);
  }
  val
}

fn slashify(text: &str) -> String {
  text.replace("\\", "\\\\")
}
fn unquote(text: &str) -> String {
  ESCAPED_OP.replace_all(text, |escaped_refs: &Captures| -> String {
    escaped_refs.at(1).unwrap_or("").to_owned()
  }).replace("##","#")
    .replace("&amp;","&")
}
