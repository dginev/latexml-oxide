use latexml_core::document::Document;

use crate::prelude::*;

LoadDefinitions!({
  // Choose the current float style (plain, plaintop, boxed, ruled)
  DefMacro!("\\float@style", None, "plain");
  DefMacro!("\\floatstyle{}", "\\def\\float@style{#1}");
  // \restylefloat{style} — ignore
  def_macro_noop("\\restylefloat OptionalMatch:* {}")?;
  // \floatplacement{style}{placement} — ignore
  def_macro_noop("\\floatplacement{}{}")?;
  // \listof{type}{title} — ignore
  def_macro_noop("\\listof{}{}")?;
  // \floatname{type}{name}. LaTeXML reimplements float.sty with its own
  // `\lx@name@<type>` internal (Perl float.sty.ltxml L36), but real float.sty
  // (L34) names it `\fname@<type>` — and documents poke at that real internal
  // directly (e.g. the popular `breakablealgorithm` recipe:
  // `\textbf{\fname@algorithm~\thealgorithm}`). Define BOTH so such recipes
  // compile instead of leaking a raw `\fname@algorithm`. Surpass over Perl,
  // which defines only `\lx@name@` and errors here. OXIDIZED_DESIGN #150,
  // KNOWN_PERL_ERRORS #107. Witness arXiv 2408.07803 (html_feedback #1998).
  DefMacro!(
    "\\floatname{}{}",
    "\\@namedef{lx@name@#1}{#2}\\@namedef{fname@#1}{#2}"
  );

  // \float@endH — close marker for `[H]` placement floats (float.sty
  // L103). Real def does box-placement layout (`\@endfloatbox\vskip
  // \intextsep \box\@currbox \vskip\intextsep`); purely visual for
  // PDF output. In XML/HTML the figure/table just closes via its
  // environment-end. Stub as no-op so unrendered raw-loads don't
  // emit "undefined". Witness: arXiv:2506.12112 / .15928 / .19294
  // (`\begin{figure}[H] ... \end{figure}` chain). Companion stubs
  // `\float@end`, `\float@dblend` follow the same pattern.
  def_macro_noop("\\float@endH")?;
  def_macro_noop("\\float@end")?;
  def_macro_noop("\\float@dblend")?;

  // Perl: DefPrimitive('\newfloat{}{}{}[]', sub { ... })
  // Creates a new float environment with counter, title format, etc.
  DefPrimitive!("\\newfloat{}{}{}[]", sub[(ftype, _placement, auxext, within)] {
    let ftype = ftype.to_string();
    let auxext = auxext.to_string();
    let within = within.map(|t| t.to_string()).unwrap_or_default();

    // Default the float's caption name to its type if not already set (real
    // float.sty L59: `\@ifundefined{fname@#1}{\floatname{#1}{#1}}`). We keep
    // LaTeXML's `\lx@name@<type>` internal AND the real float.sty `\fname@<type>`
    // (see \floatname above) so both our machinery and documents referencing
    // `\fname@<type>` resolve. float.sty only, not newfloat.
    for prefix in ["\\lx@name@", "\\fname@"] {
      let name_tok = T_CS!(s!("{prefix}{ftype}"));
      if !has_meaning(&name_tok) {
        def_macro(name_tok, None, Tokens::new(ExplodeText!(ftype)), None)?;
      }
    }

    // Get current float style for format@title
    let style = digest(T_CS!("\\float@style"))
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
      mouth::tokenize_internal(TeXString::assembled(format_body)), None)?;

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
    digest(T_CS!("\\float@style"))
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
      mouth::tokenize_internal(TeXString::assembled(the_body)),
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

  let replacement: ReplacementClosure = Rc::new(
    move |document: &mut Document, args: &Vec<Option<Digested>>, props: &SymHashMap<Stored>| {
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
      if let Some(Some(arg1)) = args.first() {
        let placement = arg1.to_string();
        if !placement.is_empty() {
          av.insert("placement".into(), placement);
        }
      }
      av.insert("class".into(), class_val.clone());
      // `^^` float-up (Perl float.sty.ltxml:56 has none; KPE #190): a custom
      // float opened inside a Block container (quote, list item — bashful's
      // `program` floats) escapes to the enclosing `ltx:para`, as the kernel
      // floats do (sect09.rs). Guard:
      // `perfect_kernel_batch54::floats_escape_block_containers`.
      let savenode = document.float_to_element("ltx:float", true)?;
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
      if let Some(sn) = savenode {
        document.set_node(&sn);
      }
      Ok(())
    },
  );

  let env_cs = T_CS!(s!("\\begin{{{name}}}"));
  let paramlist = parse_parameters("[]", &env_cs, true)?;

  let mut options = ConstructorOptions {
    mode: Some("internal_vertical".into()),
    ..Default::default()
  };

  // before_digest: beforeFloat($type [, double => 1])
  let bt = base_type;
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
  let style_for_construct = style_str;
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
  let caption_qname = pin!("ltx:caption");
  let toccaption_qname = pin!("ltx:toccaption");
  let tags_qname = pin!("ltx:tags");
  // The inner frame must land on the float's BODY element (listing/graphics/…),
  // never on its `<ltx:tags>` metadata sibling. Perl's filter only skips
  // caption/toccaption (float.sty.ltxml L82) — but a RefStepCounter'd float
  // emits `<tags>` as its FIRST child, and `<tags>` (LaTeXML-block.rnc:325,
  // `element tags { tag+ }`) carries NO attributes, so `framed` on it is
  // silently schema-dropped and no frame is ever drawn. Both engines lose the
  // box; skipping `<tags>` too puts the frame on the real body.
  // OXIDIZED_DESIGN #149 (surpass), KNOWN_PERL_ERRORS #106.
  let is_body = |qname| qname != caption_qname && qname != toccaption_qname && qname != tags_qname;
  let node = document.get_node();
  if let Some(float_node) = node.get_last_child() {
    // Frame styles: outer on the float, inner on the body. Ports Perl's
    // %float_outerframe / %float_innerframe (float.sty.ltxml L37-38) plus the
    // algorithm2e ruled family (top rule on the float, topbottom on the body).
    let (outer, inner): (Option<&str>, Option<&str>) = match style {
      "ruled" => (Some("top"), Some("topbottom")),
      "boxed" => (None, Some("rectangle")),
      _ => (None, None), // plain, plaintop — no framing
    };
    if let Some(outer) = outer {
      let mut float_mut = float_node.clone();
      document.set_attribute(&mut float_mut, "framed", outer)?;
    }
    if let Some(inner) = inner {
      for child in float_node.get_child_elements() {
        if is_body(document::get_node_qname(&child)) {
          let mut child_mut = child;
          document.set_attribute(&mut child_mut, "framed", inner)?;
          break;
        }
      }
    }
  }
  Ok(())
}

/// Move a float's `<ltx:caption>`/`<ltx:toccaption>` to BEFORE its body element, so
/// the caption renders at the TOP of the frame. algorithm2e's ruled family
/// (`ruled`/`algoruled`/`tworuled`/`plainruled`/`boxruled`) draws the caption at the
/// top: real algorithm2e.sty sets `\@algocf@capt@ruled`=`top` (L2530) /
/// `\@algocf@capt@boxruled`=`above` (L2540), and `\algocf@makethealgo` (L2589+) lays
/// the caption out before the body. LaTeXML emits the caption last (standard float
/// order = bottom), and so does Perl LaTeXML — so this is a surpass over Perl to match
/// the pdflatex golden. OXIDIZED_DESIGN #153. DOM order determines the XSLT render
/// position, so a pre-body caption renders above the listing. The float content model
/// (`LaTeXML-para.rnc:196`, `(tags? | … | Block.model | Caption.class)*`) is an
/// order-free choice, so the moved caption stays schema-valid.
pub fn reposition_caption_top(document: &mut Document) -> Result<()> {
  let caption_qname = pin!("ltx:caption");
  let toccaption_qname = pin!("ltx:toccaption");
  let tags_qname = pin!("ltx:tags");
  let node = document.get_node();
  let Some(float_node) = node.get_last_child() else {
    return Ok(());
  };
  let is_body = |qname| qname != caption_qname && qname != toccaption_qname && qname != tags_qname;
  // The first body element (the listing) is the reference point; captions move ahead
  // of it, preserving their relative order (toccaption then caption).
  let children = float_node.get_child_elements();
  let Some(mut body) = children
    .iter()
    .find(|c| is_body(document::get_node_qname(c)))
    .cloned()
  else {
    return Ok(());
  };
  for child in children {
    let qname = document::get_node_qname(&child);
    if qname == caption_qname || qname == toccaption_qname {
      let mut child_mut = child;
      body.add_prev_sibling(&mut child_mut)?;
    }
  }
  Ok(())
}
