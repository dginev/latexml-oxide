use syn;
use quote;
// use regex::{Captures, Regex};
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

macro_rules! QNAME_STR(
  () => (r"((?:\p{Ll}|\p{Lu}|\p{Lo}|\p{Lt}|\p{Nl}|_|:)(?:\p{Ll}|\p{Lu}|\p{Lo}|\p{Lt}|\p{Nl}|_|:|\p{M}|\p{Lm}|\p{Nd}|\.|-)*)")
);

macro_rules! PI_STR(
  () => (concat!(r"^\s*<\?",QNAME_STR!()))
);

// lazy_static! {
//   static ref VALUE_RE : Regex = Regex::new(r"(\#|\&[\w\:]*\()").unwrap();
//   static ref COND_RE : Regex = Regex::new(r"\?(\#|\&[\w\:]*\()").unwrap();
// // Attempt to follow XML Spec, Appendix B
//   static ref QNAME_RE : Regex = Regex::new(QNAME_STR!()).unwrap();
//   static ref TEXT_RE : Regex = Regex::new(r"(.[^\#<\?\)\&\,]*)").unwrap();
//   static ref NONW_RE : Regex = Regex::new(r"\W").unwrap();
//   static ref FLOAT_RE : Regex = Regex::new(r"^(\^+)\s*").unwrap();
//   static ref PI_RE : Regex = Regex::new(PI_STR!()).unwrap();
// }

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
      let mut operations = Vec::new();
      println_stderr!("--- IR: {:?}\n", replacement);
      operations.push(quote!(println_stderr!("-- replacement mock executed.");));

      quote!(
        Some(Arc::new(|_doc: &mut Document, _args: &Vec<TBox>, _props: &HashMap<String, String>, _state: &mut State| {
          #(operations)*
        }))
      )
    }
  };
//   let mut replacement = &*replacement;
//   let input_replacement = replacement;
//   println_stderr!("replacement IN : {}", input_replacement);
//   let mut floats: String = String::new();
// let mut has_floats: bool = false;
//   FLOAT_RE.replace_all(replacement, |refs: &Captures| -> String {
//     floats = refs.at(1).unwrap_or("").to_owned();
//     has_floats = true;
//     String::new()
//   });

//   let mut consumed = "";
//   while !replacement.is_empty() {
//     let mut is_match = false;
//     // Processing instruction: <?name a=v ...?>
//     PI_RE.replace(replacement, |refs: &Captures| -> String {
//       let node_def = refs.at(1).unwrap_or("").to_owned();
//       let (match_start, match_end) = refs.pos(1).unwrap();
//       println_stderr!("-- PI between {:?} and {:?}", match_start, match_end);
//       consumed = &replacement[0..match_end];
//       String::new()
//     });

//     // Close tag: </name>
//     if !is_match && replacement.starts_with("</") {
//       println_stderr!("-- close tag");
//       consumed = "</";
//     }
//     // Open tag: <name a=v ...> or .../> (for empty element)
//     if !is_match && replacement.starts_with("<") {
//       println_stderr!("-- open tag");
//       consumed = "<";
//     }
//     // Substitutable value: argument, property...
//     if !is_match && replacement.starts_with("#") {
//       println_stderr!("-- argument hole");
//       consumed = "#";
//     }
//     // Attribute: a=v; assigns in current node? [May conflict with random replacement!?!]
//     if !is_match && replacement.find("=").is_some() {
//       println_stderr!("-- Attribute");
//       consumed = &replacement[0..1 + replacement.find("=").unwrap()];
//     }
//     // Else random text
//     else {
//       println_stderr!("-- random text");
//       consumed = &replacement[0..1];
//     }

//     replacement = &replacement[consumed.len()..];
//   }

  // We have to jump an extra hoop, since we are forcing the struct-derive mechanism. Once the new procedural macro scheme lands, this begs to be refactored.
  quote!(
    impl _Dummy {
      fn replacement() -> Option<ReplacementClosure> {
        #compiled_replacement_closure
      }
    })
}

fn translate_avpairs(text: &str) {
  // # Parse a set of attribute value pairs from a constructor pattern,
  // # substituting argument and property values from the whatsit.
  // sub translate_avpairs {
  //   my @avs = ();
  //   s|^\s*||;
  //   while ($_) {
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
  //     elsif (s|^$QNAME_RE\s*=\s*||o) {
  //       my ($key, $value) = ($1, translate_string());
  //       push(@avs, "'$key'=>$value"); }    # if defined $value; }
  //     else { last; }
  //     s|^\s*||; }
  //   return join(', ', @avs); }
  println_stderr!("AV pairs: {:?}", text);
  return;
}
