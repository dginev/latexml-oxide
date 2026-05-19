use crate::engine::latex_constructs::*;
use crate::prelude::*;

/// DEP-18 helper for empty-body `DefMacro!("\\cs[opt-spec]", "")` stubs.
fn def_macro_noop(proto: &str) -> Result<()> {
  let (cs_tok, params) = parse_prototype(proto, true)?;
  let body = mouth::tokenize_internal("");
  def_macro(cs_tok, params, ExpansionBody::Tokens(body), None)?;
  Ok(())
}


#[rustfmt::skip]
LoadDefinitions!({
  // Perl: thmtools.sty.ltxml

  // Dependencies: raw TeX sub-packages (thm-kv.sty, thm-restate.sty) expect
  // kvsetkeys and keyval to be loaded. The raw thmtools.sty loads these via
  // thm-kv.sty, but our binding replaces raw thmtools.sty so the chain breaks.
  RequirePackage!("keyval");
  RequirePackage!("kvsetkeys");

  // Internal registers and macros needed by thm-restate.sty and thm-kv.sty
  // which load as raw TeX and expect these from thmtools internals.
  DefRegister!("\\thmt@toks" => RegisterValue::Tokens(Tokens!()));
  DefMacro!("\\thmt@thmuse@families", "thm@track@keys");
  def_macro_noop("\\thmt@mkignoringkeyhandler{}")?;
  def_macro_noop("\\thmt@thmuse@iskvtrue")?;

  // Set savable theorem parameters
  set_savable_theorem_parameters(vec![
    "\\thm@bodyfont", "\\thm@headfont", "\\thm@notefont",
    "\\thm@bodyfont", "\\thm@headpunct",
    "\\thm@styling", "\\thm@headstyling",
  ]);

  // \declaretheorem [keyvals] {name}
  // Perl: DefPrimitive('\declaretheorem OptionalKeyVals {}', sub { ... })
  DefPrimitive!("\\declaretheorem OptionalKeyVals {}", sub[(kv, thmset)] {
    let name = thmset.to_string();

    // Activate any requested style
    let style_str = kv.as_ref()
      .and_then(|k| k.get_value("style"))
      .map(|v| v.to_string())
      .unwrap_or_default();
    if !style_str.is_empty() {
      use_theorem_style(&style_str);
    } else {
      use_theorem_style("plain");
    }

    // Apply thmtools_style: save headfont/headpunct/notefont/bodyfont/headformat
    {
      let mut saved: Vec<(String, Stored)> = Vec::new();
      if let Some(headfont) = kv.as_ref().and_then(|k| k.get_value("headfont")) {
        saved.push(("\\thm@headfont".into(), Stored::Tokens(headfont.revert().unwrap_or_default())));
      }
      if let Some(headpunct) = kv.as_ref().and_then(|k| k.get_value("headpunct")) {
        saved.push(("\\thm@headpunct".into(), Stored::Tokens(headpunct.revert().unwrap_or_default())));
      }
      if let Some(notefont) = kv.as_ref().and_then(|k| k.get_value("notefont")) {
        saved.push(("\\thm@notefont".into(), Stored::Tokens(notefont.revert().unwrap_or_default())));
      }
      if let Some(bodyfont) = kv.as_ref().and_then(|k| k.get_value("bodyfont")) {
        saved.push(("\\thm@bodyfont".into(), Stored::Tokens(bodyfont.revert().unwrap_or_default())));
      }
      if let Some(headformat) = kv.as_ref().and_then(|k| k.get_value("headformat")) {
        let swap = headformat.eq_text("swapnumber");
        saved.push(("thm@swap".into(), Stored::Bool(swap)));
      }
      if !saved.is_empty() {
        save_theorem_style(&name, saved);
        use_theorem_style(&name);
      }
    }

    // Read title/name/heading for theorem type
    let type_tokens: Option<Tokens> = kv.as_ref()
      .and_then(|k| {
        k.get_value("title")
          .or_else(|| k.get_value("name"))
          .or_else(|| k.get_value("heading"))
      })
      .map(|v| v.revert().unwrap_or_default());

    // Read numbered key
    let numbered_str = kv.as_ref()
      .and_then(|k| k.get_value("numbered"))
      .map(|v| v.to_string())
      .unwrap_or_default();
    let flag = if numbered_str == "no" { Some(Tokens!()) } else { None };

    // Read sibling/numberlike/sharenumber
    let other: Option<Tokens> = kv.as_ref()
      .and_then(|k| {
        k.get_value("sibling")
          .or_else(|| k.get_value("numberlike"))
          .or_else(|| k.get_value("sharenumber"))
      })
      .map(|v| v.revert().unwrap_or_default());

    // Read parent/numberwithin/within
    let within: Option<Tokens> = kv.as_ref()
      .and_then(|k| {
        k.get_value("parent")
          .or_else(|| k.get_value("numberwithin"))
          .or_else(|| k.get_value("within"))
      })
      .map(|v| v.revert().unwrap_or_default());

    // Handle refname / Refname
    if let Some(refname) = kv.as_ref().and_then(|k| k.get_value("refname")) {
      let refname_str = refname.to_string();
      let parts: Vec<&str> = refname_str.splitn(2, ',').collect();
      let singular = parts.first().map(|s| s.trim()).unwrap_or("");
      let plural = parts.get(1).map(|s| s.trim()).unwrap_or(singular);
      def_macro(T_CS!(s!("\\{name}autorefname")), None,
        Tokens::new(ExplodeText!(singular)), None)?;
      def_macro(T_CS!(s!("\\cref@{name}@name")), None,
        Tokens::new(ExplodeText!(singular)), None)?;
      def_macro(T_CS!(s!("\\cref@{name}@name@plural")), None,
        Tokens::new(ExplodeText!(plural)), None)?;
    }
    if let Some(refname) = kv.as_ref().and_then(|k| k.get_value("Refname")) {
      let refname_str = refname.to_string();
      let parts: Vec<&str> = refname_str.splitn(2, ',').collect();
      let singular = parts.first().map(|s| s.trim()).unwrap_or("");
      let plural = parts.get(1).map(|s| s.trim()).unwrap_or(singular);
      def_macro(T_CS!(s!("\\Cref@{name}@name")), None,
        Tokens::new(ExplodeText!(singular)), None)?;
      def_macro(T_CS!(s!("\\Cref@{name}@name@plural")), None,
        Tokens::new(ExplodeText!(plural)), None)?;
    }

    define_new_theorem(flag, thmset, other, type_tokens, within)?;
  });

  // \declaretheoremstyle [keyvals] {name}
  // Perl: DefPrimitive('\declaretheoremstyle OptionalKeyVals {}', sub { ... })
  DefPrimitive!("\\declaretheoremstyle OptionalKeyVals {}", sub[(kv, name)] {
    let name_str = name.to_string();
    let mut saved: Vec<(String, Stored)> = Vec::new();

    if let Some(headfont) = kv.as_ref().and_then(|k| k.get_value("headfont")) {
      saved.push(("\\thm@headfont".into(), Stored::Tokens(headfont.revert().unwrap_or_default())));
    }
    if let Some(headpunct) = kv.as_ref().and_then(|k| k.get_value("headpunct")) {
      saved.push(("\\thm@headpunct".into(), Stored::Tokens(headpunct.revert().unwrap_or_default())));
    }
    if let Some(notefont) = kv.as_ref().and_then(|k| k.get_value("notefont")) {
      saved.push(("\\thm@notefont".into(), Stored::Tokens(notefont.revert().unwrap_or_default())));
    }
    if let Some(bodyfont) = kv.as_ref().and_then(|k| k.get_value("bodyfont")) {
      saved.push(("\\thm@bodyfont".into(), Stored::Tokens(bodyfont.revert().unwrap_or_default())));
    }
    if let Some(headformat) = kv.as_ref().and_then(|k| k.get_value("headformat")) {
      let swap = headformat.eq_text("swapnumber");
      saved.push(("thm@swap".into(), Stored::Bool(swap)));
    }

    save_theorem_style(&name_str, saved);
  });

  // \listtheoremname
  DefMacro!("\\listtheoremname", "List of Theorems");

  // \listoftheorems
  // Perl: DefConstructor('\listoftheorems OptionalKeyVals', ...)
  // Simplified: the Perl version reads ignoreall/show keyvals for filtering
  DefConstructor!("\\listoftheorems[]",
    "<ltx:TOC lists='#lists' scope='global'><ltx:title>#name</ltx:title></ltx:TOC>",
    properties => sub[_args] {
      let title = Stored::from(Digest!(T_CS!("\\listtheoremname"))?);
      let lists = String::from("thm");
      Ok(stored_map!("name" => title, "lists" => lists))
    });
});
