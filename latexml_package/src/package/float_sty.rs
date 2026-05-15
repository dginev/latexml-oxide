use crate::prelude::*;
use latexml_core::document::Document;

LoadDefinitions!({
  // Choose the current float style (plain, plaintop, boxed, ruled)
  DefMacro!("\\float@style", None, "plain");
  DefMacro!("\\floatstyle{}", "\\def\\float@style{#1}");
  // \restylefloat{style} — ignore
  DefMacro!("\\restylefloat OptionalMatch:* {}", "");
  // \floatplacement{style}{placement} — ignore
  DefMacro!("\\floatplacement{}{}", "");
  // \listof{type}{title} — ignore
  DefMacro!("\\listof{}{}", "");
  // \floatname{type}{name}
  DefMacro!("\\floatname{}{}", "\\@namedef{lx@name@#1}{#2}");

  // \float@endH — close marker for `[H]` placement floats (float.sty
  // L103). Real def does box-placement layout (`\@endfloatbox\vskip
  // \intextsep \box\@currbox \vskip\intextsep`); purely visual for
  // PDF output. In XML/HTML the figure/table just closes via its
  // environment-end. Stub as no-op so unrendered raw-loads don't
  // emit "undefined". Witness: arXiv:2506.12112 / .15928 / .19294
  // (`\begin{figure}[H] ... \end{figure}` chain). Companion stubs
  // `\float@end`, `\float@dblend` follow the same pattern.
  DefMacro!("\\float@endH", "");
  DefMacro!("\\float@end", "");
  DefMacro!("\\float@dblend", "");

  // Perl: DefPrimitive('\newfloat{}{}{}[]', sub { ... })
  // Creates a new float environment with counter, title format, etc.
  DefPrimitive!("\\newfloat{}{}{}[]", sub[(ftype, _placement, auxext, within)] {
    let ftype = ftype.to_string();
    let auxext = auxext.to_string();
    let within = within.map(|t| t.to_string()).unwrap_or_default();

    // Set \lx@name@ to type name if not already defined (float.sty only, not newfloat)
    let name_cs_str = s!("\\lx@name@{ftype}");
    let name_tok = T_CS!(name_cs_str);
    if !has_meaning(&name_tok) {
      def_macro(name_tok, None, Tokens::new(ExplodeText!(ftype)), None)?;
    }

    // Get current float style for format@title
    let style = stomach::digest(T_CS!("\\float@style"))
      .map(|d| d.to_string())
      .unwrap_or_else(|_| "plain".to_string());
    let isplain = style.starts_with("plain");

    // \format@title@type{} (float.sty only, not newfloat)
    let format_cs = s!("\\format@title@{ftype}");
    let format_body = if isplain {
      s!("\\lx@tag[][: ]{{\\lx@fnum@@{{{ftype}}}}} #1")
    } else {
      s!("\\lx@tag[][ ]{{\\lx@fnum@@{{{ftype}}}}} #1")
    };
    let format_cs_tok = T_CS!(format_cs);
    let format_paramlist = parse_parameters("{}", &format_cs_tok, true)?;
    def_macro(format_cs_tok, format_paramlist,
      mouth::tokenize_internal(&format_body), None)?;

    define_float_environment(&ftype, &auxext, &within)?;
  });
});

/// Shared helper: creates a float environment with counter, formatting macros,
/// and DefEnvironmentI. Used by both float.sty \newfloat and newfloat.sty
/// \DeclareFloatingEnvironment.
pub fn define_float_environment(ftype: &str, auxext: &str, within: &str) -> Result<()> {
  // Get current float style. `\float@style` is only defined by float.sty;
  // newfloat.sty is independent of float.sty and Perl's newfloat.sty.ltxml
  // never reads `\float@style` (it hardcodes the default layout). To keep
  // the shared helper usable from both call sites without a spurious
  // `undefined:\float@style` error when newfloat is loaded alone, probe
  // the definition first and fall back to "plain" silently when absent.
  let style = if lookup_definition(&T_CS!("\\float@style"))?.is_some() {
    stomach::digest(T_CS!("\\float@style"))
      .map(|d| d.to_string())
      .unwrap_or_else(|_| "plain".to_string())
  } else {
    "plain".to_string()
  };

  // NewCounter($type, $within)
  new_counter(ftype, within, None)?;

  // DefMacroI('\the'.$type, ...) if $within
  if !within.is_empty() {
    let the_cs = s!("\\the{ftype}");
    let the_body = s!("\\the{within}.\\arabic{{{ftype}}}");
    def_macro(
      T_CS!(the_cs),
      None,
      mouth::tokenize_internal(&the_body),
      None,
    )?;
  }

  // DefMacroI('\fnum@font@'.$type, ...)
  let isplain = style.starts_with("plain");
  let fnum_cs = s!("\\fnum@font@{ftype}");
  let fnum_body = if isplain { "\\rmfamily" } else { "\\bfseries" };
  def_macro(
    T_CS!(fnum_cs),
    None,
    mouth::tokenize_internal(fnum_body),
    None,
  )?;

  // DefMacroI('\ext@'.$type, ..., $auxext)
  let ext_cs = s!("\\ext@{ftype}");
  def_macro(T_CS!(ext_cs), None, Tokens::new(ExplodeText!(auxext)), None)?;

  // Create the float environment and starred variant
  let class = s!("ltx_float_{ftype}");
  create_float_env(ftype, &class, &style)?;
  let starred_name = s!("{ftype}*");
  create_float_env(&starred_name, &class, &style)?;

  Ok(())
}

fn create_float_env(name: &str, class: &str, style: &str) -> Result<()> {
  use crate::engine::latex_constructs::{after_float, before_float_ex};

  let class_val = class.to_string();
  // Extract the base type for before_float (remove trailing *)
  let base_type = name.trim_end_matches('*').to_string();
  // Perl float.sty.ltxml L70: starred variant calls beforeFloat with
  // `double => 1` so \hsize gets \textwidth (spans both columns) rather
  // than \columnwidth (single column). The detect-by-name-suffix mirrors
  // the DefEnvironmentI("$type*", ...) branch in Perl's \newfloat.
  let is_double = name.ends_with('*');
  let style_str = style.to_string();

  let replacement: ReplacementClosure = Rc::new({
    let class_val = class_val.clone();
    move |document: &mut Document,
          args: &Vec<Option<Digested>>,
          props: &arena::SymHashMap<Stored>| {
      let mut av: HashMap<String, String> = HashMap::default();
      if let Some(stored) = props.get("id") {
        av.insert("xml:id".into(), stored.to_string());
      }
      if let Some(stored) = props.get("inlist") {
        let inlist_str = stored.to_string();
        if !inlist_str.is_empty() {
          av.insert("inlist".into(), inlist_str);
        }
      }
      // ?#1(placement='#1') — placement from optional arg
      if let Some(Some(ref arg1)) = args.first() {
        let placement = arg1.to_string();
        if !placement.is_empty() {
          av.insert("placement".into(), placement);
        }
      }
      av.insert("class".into(), class_val.clone());
      document.open_element("ltx:float", Some(av), None)?;
      // #tags
      if let Some(stored) = props.get("tags") {
        let digested_opt: Option<Digested> = stored.into();
        if let Some(ref digested) = digested_opt {
          document.absorb(digested, None)?;
        }
      }
      // #body
      if let Some(stored) = props.get("body") {
        let digested_opt: Option<Digested> = stored.into();
        if let Some(ref digested) = digested_opt {
          document.absorb(digested, None)?;
        }
      }
      document.close_element("ltx:float")?;
      Ok(())
    }
  });

  let env_cs = T_CS!(s!("\\begin{{{name}}}"));
  let paramlist = parse_parameters("[]", &env_cs, true)?;

  let mut options = ConstructorOptions {
    mode: Some("internal_vertical".into()),
    ..Default::default()
  };

  // before_digest: beforeFloat($type [, double => 1])
  let bt = base_type.clone();
  let before_closure: BeforeDigestClosure = Rc::new(move || {
    before_float_ex(&bt, None, is_double);
    Ok(Vec::new())
  });
  options.before_digest.push(before_closure);

  // after_digest: afterFloat($whatsit)
  let after_closure: DigestionClosure = Rc::new(move |whatsit: &mut Whatsit| {
    after_float(whatsit);
    Ok(Vec::new())
  });
  options.after_digest.push(after_closure);

  // after_construct: addFloatFrames
  let style_for_construct = style_str.clone();
  let after_construct_closure: ConstructionClosure =
    Rc::new(move |document: &mut Document, _whatsit: &Whatsit| {
      add_float_frames(document, &style_for_construct)?;
      Ok(())
    });
  options.after_construct.push(after_construct_closure);

  options.properties = Rc::new(|_| Ok(stored_map!("layout" => "vertical")));

  def_environment(name.to_string(), paramlist, Some(replacement), options);
  Ok(())
}

/// Perl: addFloatFrames (float.sty.ltxml L76-85)
pub fn add_float_frames(document: &mut Document, style: &str) -> Result<()> {
  let caption_qname = arena::pin_static("ltx:caption");
  let toccaption_qname = arena::pin_static("ltx:toccaption");
  let node = document.get_node();
  if let Some(float_node) = node.get_last_child() {
    match style {
      "ruled" => {
        let mut float_mut = float_node.clone();
        document.set_attribute(&mut float_mut, "framed", "top")?;
        // inner frame: topbottom on first non-caption child
        for child in float_node.get_child_elements() {
          let qname = latexml_core::document::get_node_qname(&child);
          if qname != caption_qname && qname != toccaption_qname {
            let mut child_mut = child;
            document.set_attribute(&mut child_mut, "framed", "topbottom")?;
            break;
          }
        }
      },
      "boxed" => {
        // inner frame: rectangle on first non-caption child
        for child in float_node.get_child_elements() {
          let qname = latexml_core::document::get_node_qname(&child);
          if qname != caption_qname && qname != toccaption_qname {
            let mut child_mut = child;
            document.set_attribute(&mut child_mut, "framed", "rectangle")?;
            break;
          }
        }
      },
      _ => {}, // plain, plaintop — no framing
    }
  }
  Ok(())
}
