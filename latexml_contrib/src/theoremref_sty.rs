//! theoremref.sty binding (no Perl binding).
//!
//! theoremref names a theorem by its environment where it is referred to: `\thlabel{key}` writes
//! `\newlabel{key}{{\@currentlabel}{<theorem name>}}` to the `.aux` file (theoremref.sty:23-26) and `\thref{key,…}`
//! prints each name with its number, read from that record on the next run (:47-66). Raw-loaded, `\thlabel` labelled
//! nothing the document holds: every `\thref` printed "?THM?" and a missing label where pdflatex prints "Theorem 1.1"
//! (1211.4329). `\thlabel` labels the theorem as `\label` does, and `\thref` refers to it by the type and number its
//! tags give (`typerefnum`, ntheorem's `\thref`, ntheorem_sty.rs). theoremref's grouping of a run of references to one
//! environment under its plural ("Lemmas 5.7.7 and 2.1.5") is not made: each reference names its own.
use latexml_package::prelude::*;

LoadDefinitions!({
  // The options and `\thnameref`.
  InputDefinitions!("theoremref", noltxml => true, extension => Some(Cow::Borrowed("sty")));
  // The theorem's label. theoremref's own record names the environment by `\thmref@currname`, which its `\@thm`
  // patch sets and our theorem environments never run, so `\thnameref` (one census paper) still prints "?THM?".
  DefMacro!("\\thlabel{}", "\\label{#1}", locked => true);
  // `\thref{a,b,c}`: "A 1, B 2, and C 3", "A 1 and B 2" (`\thmref@flush` with each key of its own environment).
  DefMacro!("\\thref{}", sub[(keys)] {
    let keys = keys.to_string();
    let keys: Vec<&str> = keys.split(',').map(str::trim).filter(|key| !key.is_empty()).collect();
    let mut tokens = Vec::new();
    for (i, key) in keys.iter().enumerate() {
      if i > 0 {
        let separator = match (keys.len(), i + 1 == keys.len()) {
          (2, _) => " and ",
          (_, true) => ", and ",
          _ => ", ",
        };
        tokens.extend(ExplodeText!(separator));
      }
      tokens.extend([T_CS!("\\lx@thref"), T_BEGIN!()]);
      tokens.extend(ExplodeText!(*key));
      tokens.push(T_END!());
    }
    Ok(Tokens::new(tokens))
  }, locked => true);
  DefConstructor!("\\lx@thref Semiverbatim",
  "<ltx:ref labelref='#label' show='typerefnum' _force_font='true'/>",
  enter_horizontal => true,
  properties => sub[args] {
    let key = args[0].as_ref().map(|a| a.to_string()).unwrap_or_default();
    Ok(stored_map!("label" => clean_label(&key, None).into_owned()))
  });
});
