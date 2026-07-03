//! TeX Box
//!
//! Core TeX Implementation for LaTeXML

use latexml_core::common::numeric_ops::round_to;

use crate::prelude::*;

/// Perl: hackVBoxAttachment($box, $valign)
/// Sets vattach on the box, with special handling for \halign alignment objects.
///
/// For \halign inside \vbox/\vtop, vattach must be set on the alignment's XML
/// attributes (so it becomes an attribute on `<tabular>`), NOT on the box property
/// (which would end up on `<para>` via insertBlock).
///
/// Perl's List() simplification returns single-item vertical lists unwrapped,
/// so $box->getProperty('alignment') finds the alignment directly. In Rust,
/// Lists are not simplified, so we must walk into children to find it.
fn hack_vbox_attachment(whatsit: &mut Whatsit, valign: &'static str) {
  if let Some(content_box) = whatsit.get_arg_mut(2)
    && !set_halign_vattach(content_box, valign)
  {
    // No \halign alignment found — set vattach as property on the box
    content_box.set_property("vattach", valign);
  }
}

/// Walk into a Digested tree to find a \halign Whatsit with an 'alignment' property.
/// If found, set vattach on the alignment's XML attributes and return true.
/// Returns false if no \halign alignment was found.
fn set_halign_vattach(digested: &Digested, valign: &str) -> bool {
  match digested.data() {
    DigestedData::Whatsit(w) => {
      let w_ref = w.borrow();
      if w_ref.get_property("alignment").is_some() {
        // Check if this whatsit's definition is \halign
        let def = w_ref.get_definition();
        let is_halign = *def.get_cs_name() == *"\\halign";
        if is_halign {
          // Get the alignment property value and set vattach on it
          if let Some(Cow::Borrowed(Stored::Digested(alignment_dig))) =
            w_ref.get_property("alignment")
            && let DigestedData::Alignment(ref alignment_cell) = *alignment_dig.data()
          {
            alignment_cell
              .borrow_mut()
              .get_xml_attributes_mut()
              .insert(String::from("vattach"), String::from(valign));
            return true;
          }
        }
        // Has alignment but not \halign (e.g. tabular) — don't set vattach
        return false;
      }
      false
    },
    DigestedData::List(list_cell) => {
      // Walk children to find a \halign
      let children = list_cell.borrow().unlist();
      for child in children.iter() {
        if set_halign_vattach(child, valign) {
          return true;
        }
      }
      false
    },
    _ => false,
  }
}

/// Helper to get (width, height, depth) from a Digested node_box without mutable borrows.
/// This avoids RefCell conflicts when called during the absorption pipeline.
/// For simple TBox/Whatsit: reads cached_width/height/depth properties.
/// For Lists: sums child box widths and takes max height/depth.
fn fobj_get_size(digested: &Digested) -> (Dimension, Dimension, Dimension) {
  fn read_dims(d: &Digested) -> (Dimension, Dimension, Dimension) {
    d.with_properties(|props| {
      let w = match props.get("cached_width").or_else(|| props.get("width")) {
        Some(Stored::Dimension(d)) => *d,
        _ => Dimension::default(),
      };
      let h = match props.get("cached_height").or_else(|| props.get("height")) {
        Some(Stored::Dimension(d)) => *d,
        _ => Dimension::default(),
      };
      let d = match props.get("cached_depth").or_else(|| props.get("depth")) {
        Some(Stored::Dimension(d)) => *d,
        _ => Dimension::default(),
      };
      (w, h, d)
    })
  }
  // First try: read cached dimensions
  let dims = read_dims(digested);
  if dims.0.value_of() != 0 || dims.1.value_of() != 0 || dims.2.value_of() != 0 {
    return dims;
  }
  // If zero dims: for Lists, sum children's dimensions
  if let DigestedData::List(list_cell) = digested.data()
    && let Ok(list) = list_cell.try_borrow()
  {
    let mut total_w: i64 = 0;
    let mut max_h: i64 = 0;
    let mut max_d: i64 = 0;
    for child in &list.boxes {
      let (cw, ch, cd) = fobj_get_size(child);
      total_w += cw.value_of();
      max_h = max_h.max(ch.value_of());
      max_d = max_d.max(cd.value_of());
    }
    return (
      Dimension::new(total_w),
      Dimension::new(max_h),
      Dimension::new(max_d),
    );
  }
  dims
}

/// Perl: collapseSVGGroup (TeX_Box.pool.ltxml L199-271)
/// Collapse/remove/unwrap unneeded svg:g elements to reduce tree depth.
fn collapse_svg_group(document: &mut Document, node: &mut Node) -> Result<()> {
  use latexml_core::common::xml::element_nodes;

  // Collapsible SVG group attributes (Perl L193-197)
  const COLLAPSIBLE: &[&str] = &[
    "fill",
    "fill-rule",
    "fill-opacity",
    "stroke",
    "stroke-width",
    "stroke-linecap",
    "stroke-linejoin",
    "stroke-miterlimit",
    "stroke-dasharray",
    "stroke-dashoffset",
    "stroke-opacity",
    "color",
  ];

  // Record public (non-internal) attributes on this node
  let nodeattr: HashMap<String, String> = node
    .get_attributes()
    .into_iter()
    .filter(|(k, _)| !k.starts_with('_'))
    .collect();
  // Perl L208: skip if clip-path is set
  if nodeattr.contains_key("clip-path") {
    return Ok(());
  }

  let is_svg_g = |n: &Node| -> bool { document::with_node_qname(n, |q| q == "svg:g") };

  // Step 1: Remove empty svg:g children (Perl L211-214)
  let mut children = element_nodes(node);
  let mut nempty = 0;
  for c in &children {
    if is_svg_g(c) && element_nodes(c).is_empty() {
      document.remove_node(c.clone());
      nempty += 1;
    }
  }
  if nempty > 0 {
    children = element_nodes(node);
  }

  // Step 2: Pop leading children that completely mask parent attributes (Perl L218-228)
  // If a leading child svg:g has collapsible attributes covering ALL of parent's attributes,
  // the child "masks" the parent — move it before parent in the DOM.
  let nodeattr_count = nodeattr.len();
  let mut npopped = 0;
  while !children.is_empty() && is_svg_g(&children[0]) {
    let c = &children[0];
    let mut nmasked = 0;
    for (key, _val) in c.get_attributes() {
      if !key.starts_with('_') && COLLAPSIBLE.contains(&key.as_str()) && nodeattr.contains_key(&key)
      {
        nmasked += 1;
      }
    }
    if nmasked != nodeattr_count {
      break; // child doesn't completely mask parent
    }
    // Move child before node in parent
    let mut child = children.remove(0);
    node.add_prev_sibling(&mut child)?;
    npopped += 1;
  }

  // Step 3: Push trailing children that completely mask parent attributes (Perl L230-237)
  let mut npushed = 0;
  while !children.is_empty() && is_svg_g(children.last().unwrap()) {
    let c = children.last().unwrap();
    let mut nmasked = 0;
    for (key, _val) in c.get_attributes() {
      if !key.starts_with('_') && COLLAPSIBLE.contains(&key.as_str()) && nodeattr.contains_key(&key)
      {
        nmasked += 1;
      }
    }
    if nmasked != nodeattr_count {
      break; // child doesn't completely mask parent
    }
    // Move child after node in parent
    let mut child = children.pop().unwrap();
    node.add_next_sibling(&mut child)?;
    npushed += 1;
  }
  if npopped > 0 || npushed > 0 {
    children = element_nodes(node);
  }

  // Step 4: Remove redundant svg:g children (same attributes as parent, Perl L239-250)
  let mut nredundant = 0;
  for c in &children {
    if is_svg_g(c) {
      let mut is_same = true;
      for (key, val) in c.get_attributes() {
        if key.starts_with('_') {
          continue;
        }
        // Perl L245-246: different value OR key is 'transform' → not redundant
        if val != *nodeattr.get(&key).unwrap_or(&String::new()) || key == "transform" {
          is_same = false;
          break;
        }
      }
      if is_same {
        document.unwrap_nodes(c.clone())?;
        nredundant += 1;
      }
    }
  }
  if nredundant > 0 {
    children = element_nodes(node);
  }

  // Step 5: Merge single svg:g child into parent (Perl L254-270)
  if children.len() == 1 && is_svg_g(&children[0]) {
    let c = &children[0];
    let mut av: HashMap<String, String> = HashMap::default();
    let mut mergeable = true;
    for (key, val) in c.get_attributes() {
      if key.starts_with('_') || COLLAPSIBLE.contains(&key.as_str()) {
        av.insert(key, val);
      } else if key == "transform" {
        // Compose transforms: parent's transform + child's transform
        let composed = if let Some(parent_t) = nodeattr.get("transform") {
          format!("{} {}", parent_t, val)
        } else {
          val
        };
        av.insert(key, composed);
      } else {
        mergeable = false;
        break;
      }
    }
    if mergeable {
      for (key, val) in &av {
        node.set_attribute(key, val)?;
      }
      document.unwrap_nodes(children[0].clone())?;
    }
  }

  Ok(())
}

/// Options for [`framed_properties`] (Perl `framedProperties(%options)`,
/// TeX_Box.pool.ltxml #2829).
#[derive(Default)]
pub struct FramedOptions {
  /// the frame kind (default "rectangle")
  pub frame:           Option<String>,
  /// the margin separation, e.g. `margin => '\fboxsep'`
  pub margin:          Option<String>,
  /// the rule thickness
  pub rule:            Option<String>,
  /// the rule color (default, the current font color)
  pub color:           Option<String>,
  /// the background color (default, NONE)
  pub backgroundcolor: Option<String>,
}

/// Compute the properties required for a framed something.
///
/// Port of Perl `framedProperties` (TeX_Box.pool.ltxml L69-89, #2829):
/// consistent `framed`/`framecolor`/`backgroundcolor`/`cssstyle` attributes,
/// plus `padtop`/`padbottom`/`padleft`/`padright` Dimension properties
/// (margin + rule per side) that feed the whatsit size computation.
/// `cssstyle` carries `padding:` whenever a margin is given, and
/// `border-width:` only when the rule differs from the 0.4pt default.
pub fn framed_properties(options: FramedOptions) -> SymHashMap<Stored> {
  // Perl `$options{margin} && LookupDimension(...)`: an absent OR empty
  // option is falsy and skips the lookup entirely.
  let sep = options
    .margin
    .as_deref()
    .filter(|m| !m.is_empty())
    .and_then(|m| lookup_dimension_cs(m, false));
  let th = options
    .rule
    .as_deref()
    .filter(|r| !r.is_empty())
    .and_then(|r| lookup_dimension_cs(r, false));
  let pad = match (sep, th) {
    (Some(s), Some(t)) => Some(Dimension::new(s.value_of() + t.value_of())),
    (s, t) => s.or(t),
  };
  let th_pt = th.map(|t| t.to_attribute());
  let mut style_parts: Vec<String> = Vec::new();
  if let Some(s) = sep {
    style_parts.push(s!("padding:{}", s.to_attribute()));
  }
  if let (Some(t), Some(tp)) = (th, th_pt.as_deref())
    && tp != "0.4pt"
  {
    style_parts.push(s!("border-width:{}", t.to_attribute()));
  }
  let style = style_parts.join(";");

  let mut props: SymHashMap<Stored> = SymHashMap::default();
  props.insert(
    "framed",
    Stored::from(options.frame.unwrap_or_else(|| "rectangle".to_string())),
  );
  // Perl `LookupValue('font')->getColor` always yields a color (default
  // Black), so framecolor is always present.
  let framecolor = options
    .color
    .or_else(|| lookup_font().and_then(|f| f.get_color().map(|c| c.to_attribute())))
    .unwrap_or_else(|| s!("#000000"));
  props.insert("framecolor", Stored::from(framecolor));
  if let Some(bg) = options.backgroundcolor {
    props.insert("backgroundcolor", Stored::from(bg));
  }
  if !style.is_empty() {
    props.insert("cssstyle", Stored::from(style));
  }
  if let Some(p) = pad {
    props.insert("padtop", Stored::Dimension(p));
    props.insert("padbottom", Stored::Dimension(p));
    props.insert("padleft", Stored::Dimension(p));
    props.insert("padright", Stored::Dimension(p));
  }
  props
}

LoadDefinitions!({
  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // Box Family of primitive control sequences
  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%

  //======================================================================
  // These define the handler for { } (or anything of catcode BEGIN, END)
  // Perl TeX_Box.pool.ltxml L32-47 (T_BEGIN, T_END handlers).

  // These are actually TeX primitives, but we treat them as a Whatsit so they
  // remain in the constructed tree.
  DefPrimitive!("{", {
    bgroup();
    let open = Tbox::new(
      pin!(""),
      None,
      None,
      Tokens!(T_BEGIN!()),
      stored_map!("isEmpty" => true),
    );
    let mode = Some(if lookup_bool_sym(pin!("IN_MATH")) {
      TexMode::Math
    } else {
      TexMode::Text
    });
    let body = digest_next_body(None)?;
    let mut boxes = vec![Digested::from(open)];
    boxes.extend(body);
    let mut font = None;
    for abox in boxes.iter().rev() {
      if let Some(boxfont) = abox.get_font()? {
        font = Some(boxfont.into_owned());
        break;
      }
    }
    let mut properties = SymHashMap::default();
    // Perl: List stores mode property from current TeX mode string.
    // Only set for vertical modes to enable vertical stacking in compute_size.
    // Not set for horizontal modes to avoid interfering with repack_horizontal's
    // mode detection logic which defaults to "horizontal" when mode property is None.
    let mode_str = lookup_string_from_sym(pin!("MODE"));
    if mode_str.ends_with("vertical") {
      properties.insert("mode", Stored::String(pin(&mode_str)));
    }
    // Perl: List() sets width => \hsize when mode eq 'horizontal' (NOT restricted_horizontal)
    if matches!(mode, Some(TexMode::Text))
      && mode_str == "horizontal"
      && let Some(hsize) = lookup_dimension("\\hsize")
    {
      properties.insert("width", Stored::Dimension(hsize));
    }
    List {
      boxes,
      mode,
      font,
      locator: None,
      properties,
    }
  });

  DefPrimitive!("}", {
    let f = LookupFont!();
    egroup()?;
    Tbox::new(
      pin!(""),
      f,
      None,
      Tokens!(T_END!()),
      stored_map!("isEmpty"=>true),
    )
  });

  // Perl TeX_Box.pool.ltxml L50-55: \lx@hidden@bgroup / \lx@hidden@egroup
  // — scoping without visible { } in reversion.
  DefConstructor!("\\lx@hidden@bgroup", "#body",
    before_digest => { bgroup(); },
    capture_body => true,
    reversion => sub[whatsit, _args] {
      match whatsit.get_body()? { Some(body) => {
        body.revert()
      } _ => { Ok(Tokens!()) }}
    }
  );
  DefConstructor!("\\lx@hidden@egroup", "",
    after_digest => sub[_whatsit] { egroup()?; },
    reversion => ""
  );

  // These are for those screwy cases where you need to create a group like box,
  // more than just bgroup, egroup,
  // BUT you DON'T want extra {, } showing up in any untex-ing.
  DefConstructor!("\\@hidden@bgroup", "#body",
    before_digest => { bgroup(); },
    capture_body => true,
    reversion=> sub[whatsit,_args] {
      match whatsit.get_body()? { Some(body) => {
        body.revert()
      } _ => { Ok(Tokens!()) }}
    }
  );
  DefConstructor!("\\@hidden@egroup", "",
    after_digest => { egroup()?; },
    reversion => ""
  );

  DefMacro!(
    "\\lx@nounicode {}",
    r"\ifmmode\lx@math@nounicode#1\else\lx@text@nounicode#1\fi"
  );

  // Perl TeX_Box.pool.ltxml L61-90 (#2829): `\lx@framed` takes keyvals to
  // specify framing parameters; framedProperties massages them into
  // consistent attributes + padding properties for size calculations.
  DefKeyVal!("framed", "margin", "Dimension");
  DefKeyVal!("framed", "rule", "Dimension");
  DefConstructor!(
    "\\lx@framed OptionalKeyVals:framed {}",
    "<ltx:text framed='#framed' framecolor='#framecolor' cssstyle='#cssstyle' \
  _noautoclose='1'>#2</ltx:text>",
    enter_horizontal => true,
    sizer => "#2",
    properties => sub[args] {
      let mut opts = FramedOptions::default();
      // The OptionalKeyVals arg arrives already digested in a properties
      // closure — read its data directly (be_digested would panic here).
      if let Some(kv) = args[0].as_ref()
        && let DigestedData::KeyVals(dkv) = kv.data()
      {
        // Perl passes the whole getHash through; framedProperties reads
        // margin/rule (declared Dimension keyvals) plus any of its other
        // recognized options.
        let hash = dkv.get_hash_digested();
        opts.margin = hash.get("margin").cloned();
        opts.rule = hash.get("rule").cloned();
        opts.frame = hash.get("frame").cloned();
        opts.color = hash.get("color").cloned();
        opts.backgroundcolor = hash.get("backgroundcolor").cloned();
      }
      Ok(framed_properties(opts))
    }
  );
  // Perl: enterHorizontal => 1
  DefConstructor!(
    "\\lx@hflipped{}",
    "<ltx:text class='ltx_hflipped' _noautoclose='1'>#1</ltx:text>",
    enter_horizontal => true
  );

  // Perl TeX_Box.pool.ltxml L69-74: \lx@overlay — overlay one glyph
  // on another (used by \accent fallback). Moved here from the
  // bottom of the LoadDefinitions block to mirror Perl's order.
  DefConstructor!("\\lx@overlay{}{}",
    "<ltx:text class='ltx_overlay' _noautoclose='1'>\
       <ltx:text class='ltx_overlay_base' _noautoclose='1'>#1</ltx:text>\
       <ltx:text class='ltx_overlay_over' _noautoclose='1'>#2</ltx:text>\
     </ltx:text>",
    enter_horizontal => true
  );

  // WARNING: These two definitions MUST be active. When they were commented out,
  // \lx@nounicode expanded to \lx@text@nounicode which was undefined, causing
  // an unbounded memory leak / infinite loop that OOM-killed tests.
  //
  // Perl: DefPrimitive('\lx@math@nounicode DefToken', sub {
  //   reportNoUnicode($cs);
  //   Box(ToString($cs), undef, undef, $cs, class => 'ltx_nounicode'); });
  DefPrimitive!("\\lx@math@nounicode DefToken", sub[(cs)] {
    // `cs.get_sym()` returns the already-interned SymStr for the token
    // text — avoids the `cs.to_string() + arena::pin` round-trip which
    // allocated a String just to re-intern the same bytes.
    let tbox = Tbox::new(
      cs.get_sym(),
      None,
      None,
      Tokens!(cs),
      stored_map!("class" => "ltx_nounicode"),
    );
    Ok(vec![Digested::from(tbox)])
  });
  // Perl: DefConstructor('\lx@text@nounicode DefToken',
  //   "<ltx:text _no_autoclose='true' class='ltx_nounicode'>#1</ltx:text>", ...);
  DefConstructor!(
    "\\lx@text@nounicode DefToken",
    "<ltx:text class='ltx_nounicode'>#1</ltx:text>"
  );

  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // Box creation commands
  // ----------------------------------------------------------------------
  // \hbox           c  constructs a box holding horizontal material.
  // \vbox           c  constructs a box holding vertical material.
  // \vtop           c  is an alternate way to construct a box holding vertical material.
  //
  // \everyhbox      pt holds tokens inserted at the start of every hbox.
  // \everyvbox      pt holds tokens inserted at the start of every vbox.
  // ======================================================================

  DefParameterType!(BoxSpecification, sub[_inner, _extra] {
    if let Some(key) = read_keyword(&["to", "spread"])? {
      Ok(Tokens!(T_OTHER!(key)))
    } else {
      Ok(Tokens!())
    }
  },
  // The predigest closure reads the dimension from the gullet and stores it as KeyVals.
  // This allows afterDigest to use GetKeyVal!(spec, "to") / GetKeyVal!(spec, "spread").
  // Perl: DefParameterType('BoxSpecification', sub { ... $keyvals->setValue($key, $dim); },
  //   reversion => sub { Tokens(Tokenize($key), Revert($dim)) },
  //   optional => 1, undigested => 1);
  predigest => sub[key] {
    if !key.is_empty() {
      let mut keyvals = KeyVals::new(
        KeyvalsConfig{skip_missing: keyvals::SkipMissing::All, ..KeyvalsConfig::default()});
      let dim = read_dimension()?;
      keyvals.set_value(&key.owned_tokens().unwrap().to_string(), dim.into(), false)?;
      keyvals.into()
    } else {
      Ok(None)
    }
  },
  // Perl: reversion => sub { Tokens(Tokenize('to'), Revert($to)) }
  // Produces "to128.0374pt" as letter tokens + dimension reversion tokens,
  // matching Perl's Tokens(Tokenize('to'), Revert($to)) format exactly.
  digested_reversion => sub[spec] {
    if let DigestedData::KeyVals(keyval) = spec.data() {
      if let Some(ArgWrap::Dimension(dim)) = keyval.get_value("to") {
        let mut tks = ExplodeText!("to");
        tks.extend(dim.revert()?.unlist());
        Ok(Tokens::new(tks))
      } else if let Some(ArgWrap::Dimension(dim)) = keyval.get_value("spread") {
        let mut tks = ExplodeText!("spread");
        tks.extend(dim.revert()?.unlist());
        Ok(Tokens::new(tks))
      } else {
        Ok(Tokens!())
      }
    } else {
      Ok(Tokens!())
    }
  },
  optional => true);

  DefRegister!("\\everyhbox", Tokens!());
  DefRegister!("\\everyvbox", Tokens!());

  // Perl TeX_Box.pool.ltxml L156-167: HBoxContents/VBoxContents both call
  // readBoxContents($gullet, $everybox, $mode) with the mode argument
  // hardcoded to 'restricted_horizontal' / 'internal_vertical'
  // respectively — independent of the current mode at invocation time.
  DefParameterType!(HBoxContents, sub[_inner, _extra] {
      read_box_contents(lookup_tokens("\\everyhbox")) },
    predigest => sub[arg] {
      predigest_box_contents_in_mode(arg, "restricted_horizontal") });
  DefParameterType!(VBoxContents, sub[_inner, _extra] {
    read_box_contents(lookup_tokens("\\everyvbox")) },
  predigest => sub[arg] {
    predigest_box_contents_in_mode(arg, "internal_vertical") },
  // The faithful vertical-box loop (`predigest_box_contents_in_mode`) rebuilds the
  // body as a fresh `List::new(boxes)`, whose `revert()` concatenates the boxes
  // WITHOUT the delimiting group braces (the old `invoke_token(T_BEGIN)` group
  // box carried them). Re-add the `{}` here so `\vbox{a}` reverts to `\vbox{a}`
  // (not `\vbox a`) — matching Perl. Mirrors the standard `{}`-arg `Plain`/
  // `DefPlain` reversion (base_parameter_types.rs).
  reversion => sub[arg, _inner, _extra] {
    let mut t: Vec<Token> = vec![T_BEGIN!()];
    t.extend(arg);
    t.push(T_END!());
    Ok(Tokens::new(t))
  });

  // This re-binds a number of important control sequences to their default text binding.
  // This is useful within common boxing or footnote macros that can appear within
  // alignments or special environments that have redefined many of these.
  AssignValue!("TEXT_MODE_BINDINGS"  => Stored::VecDequeStored(VecDeque::new()));
  AssignValue!("HTEXT_MODE_BINDINGS" => Stored::VecDequeStored(VecDeque::new()));
  AssignValue!("VTEXT_MODE_BINDINGS" => Stored::VecDequeStored(VecDeque::new()));
  push_value(
    "HTEXT_MODE_BINDINGS",
    Tokens!(T_MATH!(), T_CS!("\\lx@dollar@in@textmode")),
  )?;
  push_value(
    "VTEXT_MODE_BINDINGS",
    Tokens!(T_MATH!(), T_CS!("\\lx@dollar@in@normalmode")),
  )?;

  // Perl: collapseSVGGroup (TeX_Box.pool.ltxml L199-271)
  // Collapse/remove/unwrap unneeded svg:g's to reduce tree depth.
  Tag!("svg:g", after_close => sub[document, node] {
    collapse_svg_group(document, node)?;
  });

  DefConstructor!("\\hbox BoxSpecification HBoxContents", sub[document, args, props] {
    // "<ltx:text width='#width' _noautoclose='1'>#2</ltx:text>",
    unpack_opt_ref!(args => _spec_opt, contents_opt);
    // HBoxContents can legitimately be None: the predigest returns None when
    // the box body digested to zero boxes (a malformed \hbox after mode
    // damage — witness math-ph/0405041, LamsTeX `\list\item` +
    // `$$…\tag\label{…}$$`). Perl's template `#2` of undef renders nothing
    // and the conversion completes with errors; mirror that, don't panic.
    let contents_opt = contents_opt.as_ref();
    if contents_opt.is_none() {
      Info!("empty", "hbox",
        "\\hbox contents digested to nothing; emitting an empty box");
    }
    let current_opt = document.get_element();

    // Perl: $tag eq 'ltx:_CaptureBlock_' — detect if going into insertBlock
    let is_svg = if let Some(ref current) = current_opt {
      document::with_node_qname(current, |qname| qname.starts_with("svg:"))
    } else { false };
    let vmode = if let Some(ref current) = current_opt {
      document::with_node_qname(current, |qname| qname == "ltx:_CaptureBlock_")
    } else { false };
    // Perl: $inline = $document->canContain($current, '#PCDATA')
    let inline = if let Some(ref current) = current_opt {
      document::can_contain(current, "#PCDATA")
    } else { false };
    // Perl: $newtag = ($issvg ? 'svg:g' : ($vmode ? ($inline ? 'ltx:inline-block' : 'ltx:p')
    //                                              : 'ltx:text'))
    let newtag = if is_svg { "svg:g" }
      else if vmode { if inline { "ltx:inline-block" } else { "ltx:p" } }
      else { "ltx:text" };
    let width : String = if let Some(Stored::Dimension(w)) = props.get("width") {
      w.to_attribute()
    } else {
      String::new()
    };
    let node = document.open_element(newtag,
      Some(string_map!("_noautoclose" => "true", "width" => width)), None)?;
    if let Some(contents) = contents_opt {
      document.absorb(contents, None)?;
    }
    // Perl L318-321: cleanup auto-opened svg:g/svg:svg (only when NOT in SVG),
    // then always close the specific node we opened.
    if !is_svg {
      // `get_element()` returns None at the document root (e.g. a malformed
      // `\hbox` whose body over-closed up to the root). Perl's `getElement`
      // there yields the document node whose `hasAttribute('_beginscope')` is
      // false → its loop falls through to a no-op `maybeCloseElement` and stops.
      // Rust must NOT `.unwrap()` the None (FATAL_101 panic, witness 2312.10730):
      // treat "no current element" as "stop" — behaviour-equivalent to Perl.
      while document.get_element().is_some_and(|e| !e.has_attribute("_beginscope")) &&
        document.maybe_close_element("svg:g")?.is_some() {}
      document.maybe_close_element("svg:svg")?;
    }
    document.maybe_close_node(&node)?;
  },
  mode => "restricted_horizontal",
  bounded => true,
  sizer => "#2",
  // Perl TeX_Box.pool.ltxml L300-334: `\hbox` has NO beforeDigest. The
  // outer T_MATH binding (e.g. revtex3's `\lx@dollar@in@oldrevtex` set
  // by the {equation} env) MUST persist into the hbox body so that the
  // closing `$` inside `\hbox\bgroup ... $\egroup` can toggle back via
  // the state-aware switch. Earlier Rust called `reenter_text_mode(false)`
  // here, which rebound T_MATH to `\lx@dollar@in@textmode`, breaking the
  // revtex3 `$ in equation` toggle (8+ sandbox papers, ~300 errors).
  after_digest => sub[whatsit] {
    let width : Option<RegisterValue> = {
      let spec = whatsit.get_arg(1);
      if let Some(ArgWrap::Dimension(w)) = GetKeyVal!(spec, "to") {
        Some((*w).into())
      } else if let Some(ArgWrap::Dimension(s_num_ref)) = GetKeyVal!(spec, "spread") {
        // The contents arg (and its width) can be absent for a degenerate
        // \hbox (see the None-contents note in the constructor above) —
        // skip the spread adjustment rather than panic.
        let s_num = *s_num_ref;
        if let Some(tbox) = whatsit.get_arg_mut(2)
          && let Some(current_w) = tbox.get_width(None)? {
            Some(current_w.add(s_num))
          } else {
            None
          }
      } else {
        None
      }
    };
    if let Some(w) = width {
      whatsit.set_width(w);
    }
    // Perl: $whatsit->setProperty(content_box => $box)
    whatsit.set_property("content_box", whatsit.get_arg(2).cloned());
  });

  // Perl: Tag('svg:foreignObject', autoOpen => 1, autoClose => 1, afterClose => ...)
  // This enables automatic insertion of <svg:foreignObject> when non-SVG content
  // (like <ltx:text>, <ltx:Math>) appears inside <svg:g>.
  // The afterClose handler (Perl L337-388) cleans up empty foreignObjects,
  // converts text-only content to svg:text, and sets size attributes.
  Tag!("svg:foreignObject", auto_open => true, auto_close => true,
    after_close => sub[document, node, whatsit] {
      use latexml_core::common::xml::element_nodes;
      // Perl L341: my @fo = $node->childNodes
      let has_any_child = node.get_first_child().is_some();
      // Perl L342-344: Empty foreignObject → remove
      if !has_any_child {
        let n = node.clone();
        document.remove_node(n);
        return Ok(());
      }
      let children = element_nodes(node);
      // Perl L349-362: Single <p/> cleanup
      if children.len() == 1 {
        let child_qname = document::get_node_qname(&children[0]);
        if with(child_qname, |s| s == "ltx:p") {
          let p_children = element_nodes(&children[0]);
          if p_children.is_empty() {
            let n = node.clone();
            document.remove_node(n);
            return Ok(());
          }
          if p_children.len() == 1 {
            let inner_qname = document::get_node_qname(&p_children[0]);
            if with(inner_qname, |s| s == "ltx:picture" || s == "ltx:text") {
              let pic_children = element_nodes(&p_children[0]);
              if pic_children.len() == 1 {
                let svg_qname = document::get_node_qname(&pic_children[0]);
                if with(svg_qname, |s| s == "svg:svg") {
                  let replacement = pic_children[0].clone();
                  document.replace_tree(replacement, node.clone())?;
                  return Ok(());
                }
              }
            }
          }
        }
      }
      // Perl L363-388: Set size and transform on remaining foreignObjects
      let mut has_dims = false;
      if let Some(wh) = whatsit {
        // Perl L368: my ($w, $h, $d) = $whatsit->getSize;
        // For accumulated Lists (from appendNodeBox), read cached dimensions
        // or sum up child box dimensions. Avoids mutable borrows that conflict
        // with the absorption pipeline's active RefCell borrows.
        let dims = fobj_get_size(wh);
        let (mut w, h, d) = dims;
        // If the foreignObject wraps a block with explicit width (minipage/parbox),
        // use that width instead of the accumulated box widths.
        // Perl: the node_box IS the minipage whatsit with getSize returning the
        // specified width. Our appendNodeBox creates Lists that sum widths incorrectly.
        if w.value_of() != 0 {
          for child_el in &children {
            let child_qname = document::get_node_qname(child_el);
            let is_block = with(child_qname, |s|
              s == "ltx:inline-block" || s == "ltx:_CaptureBlock_");
            if is_block
              && let Some(width_attr) = child_el.get_attribute("width") {
                // Parse width from attribute (e.g. "28.5pt", "2.85em")
                let trimmed = width_attr.trim();
                if let Some(pt_str) = trimmed.strip_suffix("pt") {
                  if let Ok(val) = pt_str.parse::<f64>() {
                    w = Dimension::new((val * 65536.0) as i64);
                    break;
                  }
                } else if let Some(em_str) = trimmed.strip_suffix("em")
                  && let Ok(val) = em_str.parse::<f64>() {
                    // 1em = 10pt at default font size
                    let font_size = wh.get_font().ok().flatten()
                      .map(|f| f.get_em_width())
                      .unwrap_or((10.0 * 65536.0) as i64);
                    w = Dimension::new((val * font_size as f64) as i64);
                    break;
                  }
              }
          }
        }
        if w.value_of() != 0 || h.value_of() != 0 || d.value_of() != 0 {
          has_dims = true;
          let w_px = w.px_value(Some(2));
          let h_px = h.px_value(Some(2));
          // Perl L369: my $H = $h->add($d); — add in sp first, then convert to px
          let total_h_dim = Dimension::new(h.value_of() + d.value_of());
          let total_h = total_h_dim.px_value(Some(2));
          // Perl L378-382: width and height (total height = h + d)
          if !node.has_attribute("width") {
            document.set_attribute(node, "width", &s!("{}", w_px))?;
          }
          if !node.has_attribute("height") {
            document.set_attribute(node, "height", &s!("{}", total_h))?;
          }
          // Perl L381: transform flips y-axis, offset by height above baseline
          document.set_attribute(node, "transform",
            &s!("matrix(1 0 0 -1 0 {})", h_px))?;
          document.set_attribute(node, "overflow", "visible")?;
          // Perl L373-387: CSS custom properties in em units
          // Perl: emValue(undef, $font) = roundto($sp / $font->getEMWidth, undef)
          let em_width = wh.get_font().ok().flatten()
            .map(|f| f.get_em_width())
            .unwrap_or(0);
          let em_width = if em_width > 0 { em_width as f64 } else { 65536.0 * 10.0 };
          let w_em = w.value_f64() / em_width;
          let h_em = h.value_f64() / em_width;
          let d_em = d.value_f64() / em_width;
          // Perl: roundto(val, undef) uses epsilon-adjusted rounding, strips trailing zeros
          let fmt_em = |v: f64| {
            let r = round_to(v, None);
            if r == r.floor() { format!("{}", r as i64) }
            else { format!("{:.2}", r).trim_end_matches('0').to_string() }
          };
          document.set_attribute(node, "style",
            &s!("--ltx-fo-width:{}em;--ltx-fo-height:{}em;--ltx-fo-depth:{}em;",
              fmt_em(w_em), fmt_em(h_em), fmt_em(d_em)))?;
        }
      }
      if !has_dims && !node.has_attribute("overflow") {
        document.set_attribute(node, "overflow", "visible")?;
      }
    }
  );

  DefConstructor!("\\vbox BoxSpecification VBoxContents", sub[document, args, _props] {
      // None contents is unreachable today (the vertical-mode predigest always
      // builds a List), but guard like \hbox rather than panic if that changes.
      if let Some(contents) = args[1].as_ref() {
        // Perl: is_vbox property detects nested \vbox|\vtop — only inner one affects vattach
        if contents.get_property_bool("is_vbox") {
          document.absorb(contents, None)?;
        } else {
          insert_block(document, contents, string_map!("vattach" => "bottom"))?;
        }
      }
    },
    sizer       => "#2",
    properties  => { stored_map!("vattach" => "bottom") },
    // Perl #2798: \vbox is an inline block — internal_vertical but no leaveHorizontal.
    mode        => "inline_internal_vertical",
    after_digest => sub[whatsit] {
      // Perl: hackVBoxAttachment($box, 'bottom')
      hack_vbox_attachment(whatsit, "bottom");
      whatsit.set_property("content_box", whatsit.get_arg(2).cloned());
      whatsit.set_property("is_vbox", true);
      // Note: BoxSpecification 'to'/'spread' height not used in XML output
    }
  );

  DefConstructor!("\\vtop BoxSpecification VBoxContents", sub[document, args, _props] {
      // None contents is unreachable today — see the \vbox note above.
      if let Some(contents) = args[1].as_ref() {
        // Perl: is_vbox property detects nested \vbox|\vtop — only inner one affects vattach
        if contents.get_property_bool("is_vbox") {
          document.absorb(contents, None)?;
        } else {
          insert_block(document, contents, string_map!("vattach" => "top"))?;
        }
      }
    },
    sizer       => "#2",
    properties  => { stored_map!("vattach" => "top") },
    // Perl #2798: \vtop is an inline block — internal_vertical but no leaveHorizontal.
    mode        => "inline_internal_vertical",
    after_digest => sub[whatsit] {
      // Perl: hackVBoxAttachment($box, 'top')
      hack_vbox_attachment(whatsit, "top");
      whatsit.set_property("content_box", whatsit.get_arg(2).cloned());
      whatsit.set_property("is_vbox", true);
      // Note: BoxSpecification 'to'/'spread' height not used in XML output
    }
  );

  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // Commands to store and use boxes
  // ----------------------------------------------------------------------
  // \setbox         c  assigns an hbox, vbox, or vtop to a box register.
  // \dp             iq is the depth of a box.
  // \ht             iq is the height of a box.
  // \wd             iq is the width of a box.
  // \box            c  puts the box's contents in the current list and empties the box.
  // \copy           c  puts the box's contents in the current list but does not empty the box     .
  // \unhbox         c  puts unwrapped hbox contents in the current list and empties the box.
  // \unhcopy        c  puts unwrapped hbox contents in the current list but does not empty the box.
  // \unvbox         c  puts unwrapped vbox contents in the current list and empties the box.
  // \unvcopy        c  puts unwrapped vbox contents in the current list but does not empty the box.
  // \lastbox        c  is void or the last hbox or vbox on the current list.
  // ======================================================================

  DefPrimitive!("\\lastbox", {
    // Hopefully, the correct box got seen!
    pop_box_list().map(|b| vec![b]).unwrap_or_default()
  });

  DefPrimitive!("\\setbox Number SkipMatch:=", sub[(number)] {
    // If there is any afterAssignment tokens, move them over so BoxContents parameter will use them
    if let Some(after_token) = remove_value("afterAssignment") {
      assign_value("BeforeNextBox", after_token, None);
    }
    // Save global flag, since we're digesting to get the box content, which resets the flag!
    // Should afterDigest be responsible for resetting flags?
    let scope = if get_prefix("global") {
      Some(Scope::Global)
    } else {
      None
    };
    clear_prefixes(); // before invoke, below; we've saved the only relevant one (global)
    let mut rest = if let Some(xtoken) = read_x_token(None, false, None)? {
        invoke_token(&xtoken)?
    } else { Vec::new() };
    let stuff = if !rest.is_empty() {
      Stored::Digested(rest.remove(0))
    } else {
      Stored::None
    };
    assign_value(&format!("box{}", number.value_of()), stuff, scope);
    rest
  });

  // # <box dimension> = \ht | \wd | \dp
  // NOTE: \ht, \wd, \dp use checkout_value/checkin_value to extract the box from state
  // before computing dimensions. This avoids borrow conflicts when compute_size() needs
  // to access state (font metrics, etc.) during dimension computation.
  DefRegister!("\\ht Number", Dimension::new(0),
  getter => sub[args] {
    if args.is_empty() { return Some(RegisterValue::Dimension(Dimension::default())); }
    let n = args.remove(0).expect_number();
    let boxid = format!("box{}", n.value_of());
    let stuff = checkout_value(&boxid);
    let result = if let Some(Stored::Digested(ref thebox)) = stuff {
      thebox.get_height()
    } else {
      Some(RegisterValue::Dimension(Dimension::default()))
    };
    if let Some(thebox) = stuff {
      checkin_value(&boxid, thebox);
    }
    result
  },
  setter => sub[value,_scope,args] {
    let n = args.remove(0).expect_number();
    let boxkey = format!("box{}", n.value_of());
    with_value_mut(&boxkey, |val_opt|
    if let Some(Stored::Digested(thebox)) = val_opt {
      thebox.set_height(value);
    })});

  DefRegister!("\\wd Number", Dimension::default(),
  getter => sub[args] {
    if args.is_empty() { return Some(RegisterValue::Dimension(Dimension::default())); }
    let n = args.remove(0).expect_number();
    let boxid = format!("box{}", n.value_of());
    let mut stuff = checkout_value(&boxid);
    let result = {if let Some(Stored::Digested(ref mut thebox)) = stuff {
      match thebox.get_width(None) {
        Ok(v) => v,
        Err(e) => {
          let err = || {Error!("method", "get_width", format!("{e}")); Ok(()) };
          err().ok();
          None
        }
      }
    } else {
      Some(RegisterValue::Dimension(Dimension::default()))
    }};
    if let Some(thebox) = stuff {
      checkin_value(&boxid, thebox);
    }
    result
  },
  setter => sub[value,_scope,args] {
    let n = args.remove(0).expect_number();
    let boxkey = format!("box{}", n.value_of());
    with_value_mut(&boxkey, |val_opt|
    if let Some(Stored::Digested(thebox)) = val_opt {
      thebox.set_width(value);
    })});

  DefRegister!("\\dp Number", Dimension::new(0),
  getter => sub[args] {
    if args.is_empty() { return Some(RegisterValue::Dimension(Dimension::default())); }
    let n = args.remove(0).expect_number();
    let boxid = format!("box{}", n.value_of());
    let stuff = checkout_value(&boxid);
    let result = if let Some(Stored::Digested(ref thebox)) = stuff {
      thebox.get_depth()
    } else {
      Some(RegisterValue::Dimension(Dimension::default()))
    };
    if let Some(thebox) = stuff {
      checkin_value(&boxid, thebox);
    }
    result
  },
  setter => sub[value,_scope,args] {
    let n = args.remove(0).expect_number();
    let boxkey = format!("box{}", n.value_of());
    with_value_mut(&boxkey, |val_opt|
    if let Some(Stored::Digested(thebox)) = val_opt {
      thebox.set_depth(value);
    })
  });

  // Perl: \box does NOT call enterHorizontal (TeX_Box.pool.ltxml line 647)
  DefPrimitive!("\\box Number", sub[(number)] {
    let box_key = s!("box{}", number.value_of());
    match remove_value(&box_key) { Some(Stored::Digested(stuff)) => {
      Ok(vec![stuff])
    } _ => {
      Ok(Vec::new())
    }}
  });

  // Perl: \copy does NOT call enterHorizontal (TeX_Box.pool.ltxml line 653)
  DefPrimitive!("\\copy Number", sub[(number)] {
    let box_key = s!("box{}", number.value_of());
    match lookup_value(&box_key) { Some(Stored::Digested(stuff)) => {
      Ok(vec![stuff])
    } _ => {
      Ok(Vec::new())
    }}
  });

  // \unhbox<8bit>, \unhcopy<8bit>
  // Perl: $stomach->enterHorizontal (TeX_Box.pool.ltxml lines 663, 673)
  DefPrimitive!("\\unhbox Number", sub[(number)] {
    enter_horizontal();
    let box_key = s!("box{}", number.value_of());
    match remove_value(&box_key) { Some(Stored::Digested(stuff)) => {
      // Only unlist if box is horizontal (mode ends with "horizontal")
      let mode = stuff.get_property_string("mode");
      if mode.ends_with("horizontal") {
        Ok(stuff.unlist())
      } else {
        Ok(vec![stuff])
      }
    } _ => {
      Ok(Vec::new())
    }}
  });

  DefPrimitive!("\\unhcopy Number", sub[(number)] {
    enter_horizontal();
    let box_key = s!("box{}", number.value_of());
    match lookup_value(&box_key) { Some(Stored::Digested(stuff)) => {
      let mode = stuff.get_property_string("mode");
      if mode.ends_with("horizontal") {
        Ok(stuff.unlist())
      } else {
        Ok(vec![stuff])
      }
    } _ => {
      Ok(Vec::new())
    }}
  });

  // \unvbox<8bit>, \unvcopy<8bit>
  // Perl: $stomach->leaveHorizontal (TeX_Box.pool.ltxml lines 683, 693)
  DefPrimitive!("\\unvbox Number", sub[(number)] {
    leave_horizontal()?;
    let box_key = s!("box{}", number.value_of());
    match remove_value(&box_key) { Some(Stored::Digested(stuff)) => {
      // Only unlist if box is vertical (mode ends with "vertical")
      let mode = stuff.get_property_string("mode");
      if mode.ends_with("vertical") {
        Ok(stuff.unlist())
      } else {
        Ok(vec![stuff])
      }
    } _ => {
      Ok(Vec::new())
    }}
  });

  // Perl: $stomach->leaveHorizontal (TeX_Box.pool.ltxml line 693)
  DefPrimitive!("\\unvcopy Number", sub[(number)] {
    leave_horizontal()?;
    let box_key = s!("box{}", number.value_of());
    match lookup_value(&box_key) { Some(Stored::Digested(stuff)) => {
      let mode = stuff.get_property_string("mode");
      if mode.ends_with("vertical") {
        Ok(stuff.unlist())
      } else {
        Ok(vec![stuff])
      }
    } _ => {
      Ok(Vec::new())
    }}
  });

  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // Various box related parameters
  // ----------------------------------------------------------------------
  // \prevdepth      iq is the depth of the last box added to the current vertical list.
  // \boxmaxdepth    pd is the maximum possible depth of a vertical box.
  // \badness        iq is 0-10,000 and represents the badness of the glue settings
  //                    in the last constructed box.
  // \hbadness       pi is the badness above which bad hboxes are reported.
  // \vbadness       pi is the badness above which bad vboxes are reported.
  // \hfuzz          pd is the overrun allowed before overfull hboxes are reported.
  // \vfuzz          pd is the overrun allowed before overfull vboxes are reported.
  // \overfullrule   pd is the width of the rule appended to an overfull box.
  // ======================================================================
  DefRegister!("\\prevdepth", Dimension::new(0));
  DefRegister!("\\boxmaxdepth", Dimension!("16383.99999pt"));
  DefRegister!("\\hfuzz", Dimension!("0.1pt"));
  DefRegister!("\\vfuzz", Dimension!("0.1pt"));
  DefRegister!("\\overfullrule", Dimension!("5pt"));
  DefRegister!("\\badness", Number::new(0), readonly => true); // Perl: readonly
  DefRegister!("\\hbadness", Number!(1000)); // Perl: NOT readonly (writable threshold)
  DefRegister!("\\vbadness", Number!(1000));

  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // Rules and Leaders
  // ----------------------------------------------------------------------
  // \hrule          c  makes a rule box in vertical mode.
  // \vrule          c  makes a rule box in horizontal mode.
  // \cleaders       c  insert centered leaders.
  // \leaders        c  fill space using specified glue with a box or rule.
  // \xleaders       c  insert expanded leaders.
  // ======================================================================
  DefParameterType!(RuleSpecification, sub[_inner, _extra] {
    let mut keyvals = KeyVals::new(
      KeyvalsConfig{ skip_missing: keyvals::SkipMissing::All, .. KeyvalsConfig::default()});
    while let Some(key) = read_keyword(&["width", "height", "depth"])? {
      keyvals.set_value(&key, ArgWrap::Dimension(read_dimension()?), false)?;
    }
    keyvals
  },
  optional => true,
  predigest => sub[arg] {
    match arg {
      ArgWrap::KV(kv) => Ok(Some(Digested::from(*kv))),
      _ => Ok(arg.undigested()),
    }
  });

  // \hrule, \vrule are awkward in trying to deal with 3 cases
  //  * as rules within an alignment/table
  //  * as separating lines within text
  //  * as graphical lines within svg
  // and each has different requirements for size
  DefConstructor!("\\vrule RuleSpecification",
    "?#invisible()(?#isVerticalRule()(<ltx:rule ?#rheight(height='#rheight') ?#rdepth(depth='#rdepth')\
       ?#rwidth(width='#rwidth') ?#color(color='#color')/>))",
  after_digest => sub [whatsit] {
    // Extract dimensions from keyvals arg (Perl: TeX_Box.pool.ltxml L752-760)
    use latexml_core::digested::DigestedData;
    use latexml_core::definition::argument::ArgWrap;
    use latexml_core::common::dimension::Dimension;
    let arg1 = whatsit.get_arg(1);
    let (width, height, depth) = if let Some(d) = &arg1 {
      if let DigestedData::KeyVals(kv) = d.data() {
        let w = kv.get_value("width").and_then(|a| if let ArgWrap::Dimension(d) = a { Some(*d) } else { None });
        let h = kv.get_value("height").and_then(|a| if let ArgWrap::Dimension(d) = a { Some(*d) } else { None });
        let d = kv.get_value("depth").and_then(|a| if let ArgWrap::Dimension(d) = a { Some(*d) } else { None });
        (w, h, d)
      } else { (None, None, None) }
    } else { (None, None, None) };

    // Perl: $stomach->enterHorizontal
    enter_horizontal();

    // Perl: rwidth => $width, cwidth => $width || Dimension('0.4pt'), etc.
    // Use to_attribute() for 1-decimal-place formatting matching Perl's Dimension->toAttribute
    use latexml_core::common::numeric_ops::NumericOps;
    if let Some(w) = width { whatsit.set_property("rwidth", w.to_attribute()); }
    if let Some(h) = height { whatsit.set_property("rheight", h.to_attribute()); }
    if let Some(d) = depth { whatsit.set_property("rdepth", d.to_attribute()); }
    // Set computed sizes (Perl: cwidth/cheight/cdepth) as cached_width/cached_height/cached_depth
    // These are used by get_size() to determine box dimensions for alignment cell sizing.
    // Perl: cwidth => $width || Dimension('0.4pt')
    let default_width: Dimension = "0.4pt".parse().unwrap_or_default();
    let cwidth = width.unwrap_or(default_width);
    let cheight = height.unwrap_or_default();
    let cdepth = depth.unwrap_or_default();
    whatsit.set_property("cached_width", cwidth);
    whatsit.set_property("cached_height", cheight);
    whatsit.set_property("cached_depth", cdepth);

    let w_pt = width.map(|d| d.value_of() as f64 / 65536.0);
    let h_pt = height.map(|d| d.value_of() as f64 / 65536.0);

    match lookup_alignment() { Some(_alignment) => {
      // Perl: set isVerticalRule only if dimensions suggest a real rule
      let dominated_by_height = match (h_pt, w_pt) {
        (None, None) => true,
        (Some(h), None) => h > 20.0,
        (Some(h), Some(w)) => h > 3.0 * w,
        _ => false,
      };
      if dominated_by_height {
        whatsit.set_property("isVerticalRule", true);
      }
    } _ => if w_pt == Some(0.0) {
      whatsit.set_property("invisible", true);
    }}
    // Set color from current font (Perl: only if NOT black)
    if let Some(font) = lookup_font()
      && let Some(color) = font.color
        && color != common::color::BLACK {
          whatsit.set_property("color", color.to_attribute());
        }
    Ok(Vec::new())
  });

  DefConstructor!("\\hrule RuleSpecification",
    "?#isHorizontalRule()(<ltx:rule ?#rheight(height='#rheight') ?#rdepth(depth='#rdepth')\
       ?#rwidth(width='#rwidth') ?#color(color='#color')/>)",
  before_construct => sub[document, whatsit] {
    // Perl: maybeCloseElement('ltx:p') if rwidth is '100%'
    if whatsit.get_property_string("rwidth") == "100%" {
      document.maybe_close_element("ltx:p")?;
    }
  },
  after_digest => sub [whatsit] {
    use latexml_core::digested::DigestedData;
    use latexml_core::definition::argument::ArgWrap;
    use latexml_core::common::numeric_ops::NumericOps;
    let (width, height, depth) = if let Some(d) = whatsit.get_arg(1) {
      if let DigestedData::KeyVals(kv) = d.data() {
        let w = kv.get_value("width").and_then(|a| if let ArgWrap::Dimension(d) = a { Some(*d) } else { None });
        let h = kv.get_value("height").and_then(|a| if let ArgWrap::Dimension(d) = a { Some(*d) } else { None });
        let d = kv.get_value("depth").and_then(|a| if let ArgWrap::Dimension(d) = a { Some(*d) } else { None });
        (w, h, d)
      } else { (None, None, None) }
    } else { (None, None, None) };

    // Perl: $stomach->leaveHorizontal;
    leave_horizontal()?;
    let w_pt = width.map(|d| d.value_of() as f64 / 65536.0);
    let h_pt = height.map(|d| d.value_of() as f64 / 65536.0);

    // Perl: rwidth => $width || '100%', rheight => $height || '1px'
    whatsit.set_property("rwidth", width.map(|w| w.to_attribute()).unwrap_or_else(|| "100%".to_string()));
    whatsit.set_property("rheight", height.map(|h| h.to_attribute()).unwrap_or_else(|| "1px".to_string()));
    if let Some(d) = depth { whatsit.set_property("rdepth", d.to_attribute()); }
    // Set computed sizes for alignment cell sizing
    let cheight = height.unwrap_or_else(|| "1px".parse::<Dimension>().unwrap_or_default());
    let cdepth = depth.unwrap_or_default();
    whatsit.set_property("cached_height", cheight);
    whatsit.set_property("cached_depth", cdepth);
    // hrule defaults to full width — don't cache a specific width

    if let Some(_alignment) = lookup_alignment() {
      // Perl: set isHorizontalRule only if dimensions suggest a real rule
      let dominated_by_width = match (h_pt, w_pt) {
        (None, None) => true,
        (None, Some(w)) => w > 20.0,
        (Some(h), Some(w)) => w > 3.0 * h,
        _ => false,
      };
      if dominated_by_width {
        _alignment.alignment_cell().unwrap().borrow_mut()
          .add_line("t", Vec::new());
        whatsit.set_property("isHorizontalRule", true);
      }
    }
    // Outside alignment: isHorizontalRule is NOT set, so template outputs <ltx:rule>
    // Set color from current font (Perl: only if NOT black)
    if let Some(font) = lookup_font()
      && let Some(color) = font.color
        && color != common::color::BLACK {
          whatsit.set_property("color", color.to_attribute());
        }
    Ok(Vec::new())
  });

  // Various leaders — fill space with box or rule
  // Perl: DefConstructor('\leaders Digested Digested', sub { ... },
  //   bounded => 1, beforeDigest => sub { $STATE->assignValue(Alignment => undef); });
  DefConstructor!("\\leaders Digested Digested", sub [document, args] {
    let filler = &args[0];
    let _glue = &args[1];

    // Get the context element's box to check for explicit width
    let context = document.get_element();
    let cbox = context.as_ref().and_then(|n| document.get_node_box(n));
    let req_width: Option<Dimension> = cbox.as_ref().and_then(|b| {
      b.get_property("width").and_then(|s| {
        let dim_opt: Option<Dimension> = (&*s).into();
        dim_opt
      })
    });

    let mut noautoclose_attrs = HashMap::default();
    noautoclose_attrs.insert(String::from("_noautoclose"), String::from("1"));
    let mut container = document.open_element("ltx:text", Some(noautoclose_attrs), None)?;

    if let Some(filler_d) = filler {
      document.absorb(filler_d, None)?;
    }

    // Check if we should extend a rule to fill the requested width
    let mut unwrap = false;
    if let (Some(_), Some(rw)) = (&cbox, &req_width) {
      // Find the last child of the container — should be the absorbed rule
      if let Some(mut fnode) = container.get_last_child() {
        let qname = fnode.get_name();
        if qname == "rule" {
          // Extend the rule to fill the width
          document.set_attribute(&mut fnode, "width", &rw.to_attribute())?;
          document.add_class(&mut fnode, "ltx_filled_leader")?;
          unwrap = true;
        }
      }
    }
    if !unwrap {
      document.add_class(&mut container, "ltx_leader")?;
    }

    document.close_element("ltx:text")?;
    if unwrap {
      document.unwrap_nodes(container)?;
    }
  },
    bounded => true,
    before_digest => sub {
      // Hide alignment so that \hrule inside \leaders doesn't add border="t"
      // Perl: $STATE->assignValue(Alignment => undef);
      assign_value("Alignment", Stored::None, None);
    }
  );

  let_i(&T_CS!("\\cleaders"), &T_CS!("\\leaders"), None);
  let_i(&T_CS!("\\xleaders"), &T_CS!("\\leaders"), None);

  // \lx@overlay was here in the old Rust order; moved to its
  // Perl-mirrored position (TeX_Box.pool.ltxml L69) right after
  // \lx@hflipped to satisfy the file-order parity audit.
});

// Risky: I think this needs to be digested as a body to work like TeX (?)
// but parameter think's it's just parsing from gullet...
pub fn read_box_contents(everybox_opt: Option<Tokens>) -> Result<Tokens> {
  while let Some(t) = read_token()? {
    // Perl: $t->defined_as(T_BEGIN) — checks meaning, not catcode.
    // This catches both { (catcode BEGIN) and \bgroup (\let to T_BEGIN).
    if t.defined_as(&T_BEGIN!()) {
      break;
    } // Skip till { or \bgroup
  }
  // Now, insert some extra tokens, if any, possibly from \afterassignment
  match remove_value("BeforeNextBox") {
    Some(Stored::Tokens(tokens)) => unread(tokens),
    Some(Stored::Token(token)) => unread_one(token),
    None | Some(Stored::None) => {},
    Some(other) => log::warn!("afterAssignment should be a token, got: {}", other),
  };
  // AND, insert any extra tokens passed in, due to everyhbox or everyvbox
  if let Some(everybox) = everybox_opt {
    unread(everybox);
  }
  Ok(Tokens!())
}
