use crate::prelude::*;
//**********************************************************************
// See amsldoc
//
// Currently only a random collection of things I (Bruce) need for DLMF chapters.
// Eventually, go through the doc and implement it all.
//**********************************************************************

// DG:
// TODO: Most of this binding is not ported yet.

/// Perl: amsAlignmentBindings($template, %properties) — amsmath.sty.ltxml lines 107-120
/// Simple alignment bindings for ams environments (no equation rearrangement)
fn ams_alignment_bindings(template: Template, xml_attributes: HashMap<String, String>) {
  use crate::engine::tex_tables::alignment_bindings;
  let properties = SymHashMap::default();
  // Perl: my $cur_jot = LookupDimension('\jot');
  // TODO: handle \jot rowsep
  alignment_bindings(template, String::from("math"), properties, xml_attributes);
  state::let_i(
    &T_CS!("\\\\"),
    &T_CS!("\\lx@alignment@newline@noskip"),
    None,
  );
}

/// Perl: amsRearrangeableBindings — creates equationgroup/equation/_Capture_ alignment
/// amsmath.sty.ltxml lines 121-147
fn ams_rearrangeable_bindings(
  template: Template,
  xml_attributes: HashMap<String, String>,
) -> Result<()> {
  let properties = SymHashMap::default();
  // Create alignment with equationgroup/equation/_Capture_ hooks
  let alignment = Alignment::new(AlignmentConfig {
    template: Some(template),
    open_container: Rc::new(move |document, mut props| {
      if let Ok(id_props) = ref_step_id("@equationgroup") {
        if let Some(id) = id_props.get("id") {
          props.insert(String::from("xml:id"), id.to_string());
        }
      }
      // Merge xml_attributes into props
      // (attributes passed at creation time)
      document
        .open_element("ltx:equationgroup", Some(props), None)
        .map(Option::Some)
    }),
    close_container: Rc::new(|document| document.close_element("ltx:equationgroup")),
    open_row: Rc::new(|document, mut props| {
      // Read equation tags from state
      if let Some(Stored::HashStored(eq_props)) =
        state::remove_value("EQUATIONROW_PROPS")
      {
        if let Some(id) = eq_props.get("id") {
          props.insert(String::from("xml:id"), Stored::from(id.to_string()));
        }
      }
      // Extract tags (Digested) before converting to string props
      let tags_digested = props.remove("tags");
      let str_props: HashMap<String, String> = props.into_iter()
        .map(|(k, v)| (k, v.to_string()))
        .collect();
      document
        .open_element("ltx:equation", Some(str_props), None)?;
      // If we have digested tags, absorb them into the opened element
      if let Some(Stored::Digested(d)) = tags_digested {
        document.absorb(&d, None)?;
      }
      Ok(())
    }),
    close_row: Rc::new(|document| document.close_element("ltx:equation")),
    open_column: Rc::new(|document, props| {
      document
        .open_element("ltx:_Capture_", Some(props), None)
        .map(Option::Some)
    }),
    close_column: Rc::new(|document| document.close_element("ltx:_Capture_")),
    is_math: true,
    properties,
    xml_attributes,
  });
  assign_alignment(alignment, None);
  state::let_i(&T_MATH!(), &T_CS!("\\lx@dollar@in@mathmode"), None);
  state::let_i(
    &T_CS!("\\\\"),
    &T_CS!("\\lx@alignment@newline@noskip"),
    None,
  );
  state::let_i(
    &T_CS!("\\lx@alignment@row@before"),
    &T_CS!("\\eqnarray@row@before"),
    None,
  );
  state::let_i(
    &T_CS!("\\lx@alignment@row@after"),
    &T_CS!("\\eqnarray@row@after"),
    None,
  );
  // Perl: Let('\intertext', '\@ams@intertext');
  state::let_i(&T_CS!("\\intertext"), &T_CS!("\\@ams@intertext"), None);
  // Redirect \label to the noalign version (matching Perl eqnarray behavior).
  // In Perl, \hfil makes cells with only \label non-skippable, so the \label
  // constructor runs during beAbsorbed. In Rust, \hfil doesn't contribute width,
  // so such cells are skippable and the constructor is never invoked.
  // By routing \label through \lx@hidden@noalign, the label is processed at the
  // row level (equation element), ensuring labels= is always set.
  state::let_i(
    &T_CS!("\\lx@eqnarray@save@label"),
    &T_CS!("\\label"),
    None,
  );
  state::let_i(
    &T_CS!("\\label"),
    &T_CS!("\\lx@eqnarray@label"),
    None,
  );
  Ok(())
}

/// Perl: \@ams@gather@bindings — single centered column
fn ams_gather_bindings() -> Result<()> {
  use latexml_core::alignment::cell::Cell;
  use latexml_core::alignment::template::TemplateConfig;

  let col = Cell {
    before: Some(Tokens::new(vec![
      T_CS!("\\hfil"), T_MATH!(), T_CS!("\\displaystyle"),
    ])),
    after: Some(Tokens::new(vec![T_MATH!(), T_CS!("\\hfil")])),
    empty: true,
    ..Cell::default()
  };
  let template = Template::new(TemplateConfig {
    columns: Some(vec![col]),
    ..TemplateConfig::default()
  });
  let mut attrs = HashMap::default();
  attrs.insert(String::from("class"), String::from("ltx_eqn_gather"));
  ams_rearrangeable_bindings(template, attrs)
}

/// Perl: \@ams@align@bindings — repeated pairs of columns
fn ams_align_bindings() -> Result<()> {
  use latexml_core::alignment::cell::Cell;
  use latexml_core::alignment::template::TemplateConfig;

  let col1 = Cell {
    before: Some(Tokens::new(vec![
      T_CS!("\\hfil"), T_MATH!(), T_CS!("\\displaystyle"),
    ])),
    after: Some(Tokens::new(vec![T_MATH!()])),
    empty: true,
    ..Cell::default()
  };
  let col2 = Cell {
    before: Some(Tokens::new(vec![
      T_MATH!(), T_CS!("\\displaystyle"),
    ])),
    after: Some(Tokens::new(vec![T_MATH!(), T_CS!("\\hfil")])),
    empty: true,
    ..Cell::default()
  };
  let template = Template::new(TemplateConfig {
    repeated: vec![col1, col2],
    ..TemplateConfig::default()
  });
  let mut attrs = HashMap::default();
  attrs.insert(String::from("class"), String::from("ltx_eqn_align"));
  attrs.insert(String::from("colsep"), String::from("0pt"));
  ams_rearrangeable_bindings(template, attrs)
}

/// Perl: \@ams@aligned@bindings — for aligned/alignedat/split within math
fn ams_aligned_bindings() -> Result<()> {
  use latexml_core::alignment::cell::Cell;
  use latexml_core::alignment::template::TemplateConfig;

  let col1 = Cell {
    before: Some(Tokens::new(vec![
      T_CS!("\\hfil"), T_CS!("\\displaystyle"),
    ])),
    empty: true,
    ..Cell::default()
  };
  let col2 = Cell {
    before: Some(Tokens::new(vec![T_CS!("\\displaystyle")])),
    after: Some(Tokens::new(vec![T_CS!("\\hfil")])),
    empty: true,
    ..Cell::default()
  };
  let template = Template::new(TemplateConfig {
    repeated: vec![col1, col2],
    ..TemplateConfig::default()
  });
  let mut attrs = HashMap::default();
  attrs.insert(String::from("name"), String::from("aligned"));
  attrs.insert(String::from("colsep"), String::from("0pt"));
  ams_alignment_bindings(template, attrs);
  // Perl: DefMacro('\lx@alignment@row@before', '');
  // Perl: DefMacro('\lx@alignment@row@after',  '');
  def_macro(T_CS!("\\lx@alignment@row@before"), None, Tokens!(), None)?;
  def_macro(T_CS!("\\lx@alignment@row@after"), None, Tokens!(), None)?;
  Ok(())
}

LoadDefinitions!({
  Let!("\\@xp", "\\expandafter");
  Let!("\\@nx", "\\noexpand");
  // sub-packages:
  RequirePackage!("amsbsy");
  RequirePackage!("amstext");
  RequirePackage!("amsopn");

  //======================================================================
  // Perl: amsmath.sty.ltxml lines 769-812
  // Matrix environments
  DefMacro!("\\lx@ams@matrix {}",
    "\\lx@gen@matrix@bindings{#1}\\lx@ams@cr@binding\\lx@ams@matrix@{#1}\\lx@begin@alignment");
  DefMacro!("\\lx@end@ams@matrix",
    "\\lx@end@alignment\\lx@end@gen@matrix");

  DefMacro!("\\matrix",    "\\lx@ams@matrix{name=matrix,datameaning=matrix}");
  DefMacro!("\\endmatrix", "\\lx@end@ams@matrix");
  DefMacro!("\\pmatrix", "\\lx@ams@matrix{name=pmatrix,datameaning=matrix,left=\\lx@left(,right=\\lx@right)}");
  DefMacro!("\\endpmatrix", "\\lx@end@ams@matrix");
  DefMacro!("\\bmatrix", "\\lx@ams@matrix{name=bmatrix,datameaning=matrix,left=\\lx@left[,right=\\lx@right]}");
  DefMacro!("\\endbmatrix", "\\lx@end@ams@matrix");
  DefMacro!("\\Bmatrix", "\\lx@ams@matrix{name=Bmatrix,datameaning=matrix,left=\\lx@left\\{,right=\\lx@right\\}}");
  DefMacro!("\\endBmatrix", "\\lx@end@ams@matrix");
  DefMacro!("\\vmatrix", "\\lx@ams@matrix{name=vmatrix,delimitermeaning=determinant,datameaning=matrix,left=\\lx@left|,right=\\lx@right|}");
  DefMacro!("\\endvmatrix", "\\lx@end@ams@matrix");
  DefMacro!("\\Vmatrix", "\\lx@ams@matrix{name=Vmatrix,delimitermeaning=norm,datameaning=matrix,left=\\lx@left\\|,right=\\lx@right\\|}");
  DefMacro!("\\endVmatrix", "\\lx@end@ams@matrix");
  // Perl has a typo: "atameaning" instead of "datameaning". Match Perl to avoid XMDual wrapping.
  DefMacro!("\\smallmatrix", "\\lx@ams@matrix{name=smallmatrix,atameaning=matrix,style=\\scriptsize}");
  DefMacro!("\\endsmallmatrix", "\\lx@end@ams@matrix");
  DefMacro!("\\matrix@check{}", None);

  //======================================================================
  // Perl: amsmath.sty.ltxml lines 687-721 — cases environments
  DefMacro!("\\lx@ams@cases{}",
    "\\lx@gen@cases@bindings{#1}\\lx@ams@cr@binding\\lx@ams@cases@{#1}\\lx@begin@alignment");
  DefMacro!("\\lx@end@ams@cases",
    "\\lx@hidden@cr{}\\lx@end@alignment\\lx@end@gen@cases");

  DefMacro!("\\cases",    "\\lx@ams@cases{name=cases,meaning=cases,left=\\lx@left\\{}");
  DefMacro!("\\endcases", "\\lx@end@ams@cases");

  //======================================================================
  // Section 4.2 Math spacing commands
  // \, == \thinspace
  // \: == \medspace
  // \; == \thickspace
  // \quad
  // \qquad
  // \! == \negthinspace
  // \negmedspace
  // \negthickspace
  // I think only these are new

  // DefConstructorI('\thinspace', undef,
  //   "?#isMath(<ltx:XMHint name='thinspace' width='#width'/>)(\x{2009})",
  //   properties => { isSpace => 1, width => sub { LookupValue('\thinmuskip'); } });
  // DefConstructorI('\negthinspace', undef,
  //   "?#isMath(<ltx:XMHint name='negthinspace' width='#width'/>)()",
  //   properties => { isSpace => 1, width => sub { LookupValue('\thinmuskip')->negate; } });
  DefConstructor!(
    "\\medspace",
    "?#isMath(<ltx:XMHint name='medspace'/>)()"
  );
  DefConstructor!(
    "\\negmedspace",
    "?#isMath(<ltx:XMHint name='negmedspace'/>)()"
  );
  DefConstructor!(
    "\\thickspace",
    "?#isMath(<ltx:XMHint name='thickspace'/>)(\u{2004})"
  );
  DefConstructor!(
    "\\negthickspace",
    "?#isMath(<ltx:XMHint name='negthickspace'/>)()"
  );

  // DefConstructor('\mspace{MuDimension}', "<ltx:XMHint name='mspace' width='#1'/>");

  //======================================================================
  // Section 4.3 Dots
  DefMath!("\\dotsc", "\u{2026}", role => "ID", alias => "\\dotsc");
  DefMath!("\\dotsb", "\u{22EF}", role => "ID", alias => "\\dotsb");
  DefMath!("\\dotsm", "\u{22EF}", role => "ID", alias => "\\dotsm");
  DefMath!("\\dotsi", "\u{22EF}", role => "ID", alias => "\\dotsi");
  DefMath!("\\dotso", "\u{2026}", role => "ID", alias => "\\dotso");

  DefMacro!("\\DOTSB", None);
  DefMacro!("\\DOTSI", None);
  DefMacro!("\\DOTSX", None);
  Let!("\\hdots", "\\lx@ldots");

  DefMacro!("\\hdotsfor Number", r"\hdots");

  // Perl amsmath L848-860: Smart \dots — peek at following token's role.
  // If ADDOP/BINOP/MULOP/RELOP → use ⋯ (cdots), else → … (ldots)
  // Read the next token, digest it, check its role, then put it back.
  def_primitive(
    T_CS!("\\lx@math@dots"),
    None,
    Some(PrimitiveBody::Closure(Rc::new(|_args: Vec<ArgWrap>| {
      // Read and digest the next box (like Perl's Digested parameter)
      let mut after_boxes = Vec::new();
      while let Some(tok) = gullet::read_x_token(Some(false), false, None)? {
        after_boxes = stomach::invoke_token(&tok)?;
        if !after_boxes.is_empty() {
          break;
        }
      }
      let after = after_boxes.first();
      let role = after
        .and_then(|d| d.get_property("role"))
        .map(|r| r.to_string())
        .unwrap_or_default();
      let is_binop = matches!(role.as_str(), "ADDOP" | "BINOP" | "MULOP" | "RELOP");
      let ch = if is_binop { "\u{22EF}" } else { "\u{2026}" };
      let font = lookup_font().unwrap()
        .merge(fontmap!(family => "serif", series => "medium", shape => "upright"))
        .specialize(ch);
      let tbox = Tbox::new(
        arena::pin(ch), Some(Rc::new(font)), None, Tokens!(T_CS!("\\dots")),
        stored_map!("mode" => "math", "name" => "dots", "role" => "ID", "isMath" => true)
      );
      let mut result: Vec<Digested> = vec![Digested::from(tbox)];
      result.extend(after_boxes); // put back the digested token
      Ok(result)
    }))),
    PrimitiveOptions { scope: Some(Scope::Global), ..PrimitiveOptions::default() },
  )?;
  DefMacro!("\\dots", r"\ifmmode\lx@math@dots\else\lx@ldots\fi", scope => Some(Scope::Global));

  //======================================================================
  // Section 4.9 Extensible arrows
  // Perl: amsmath.sty.ltxml lines 921-950
  DefConstructor!(
    "\\lx@long@arrow DefToken {} OptionalInScriptStyle InScriptStyle",
    r###"?#3(<ltx:XMApp role='ARROW'><ltx:XMWrap role='UNDERACCENT'>#3</ltx:XMWrap><ltx:XMApp role='ARROW'><ltx:XMWrap role='OVERACCENT'>#4</ltx:XMWrap>#2</ltx:XMApp></ltx:XMApp>)(<ltx:XMApp role='ARROW'><ltx:XMWrap role='OVERACCENT'>#4</ltx:XMWrap>#2</ltx:XMApp>)"###,
    reversion => sub[_whatsit, args] {
      // Perl: ($cs, ($under ? (T_OTHER('['), Revert($under), T_OTHER(']')) : ()), T_BEGIN, Revert($over), T_END)
      let cs_rev = match &args[0] { Some(inner) => inner.revert()?, None => Tokens!() };
      let under_rev = match &args[2] { Some(inner) => inner.revert()?, None => Tokens!() };
      let over_rev = match &args[3] { Some(inner) => inner.revert()?, None => Tokens!() };
      let mut tks = Vec::new();
      tks.extend(cs_rev.unlist());
      if !under_rev.is_empty() {
        tks.push(T_OTHER!("["));
        tks.extend(under_rev.unlist());
        tks.push(T_OTHER!("]"));
      }
      tks.push(T_BEGIN!());
      tks.extend(over_rev.unlist());
      tks.push(T_END!());
      Ok(Tokens::new(tks))
    }
  );
  DefMacro!("\\xrightarrow", "\\lx@long@arrow{\\xrightarrow}{\\lx@stretchy@rightarrow}");
  DefMacro!("\\xleftarrow", "\\lx@long@arrow{\\xleftarrow}{\\lx@stretchy@leftarrow}");
  DefMath!("\\lx@stretchy@leftarrow", "\u{2190}",
    role => "ARROW", stretchy => true, alias => "\\leftarrow");
  DefMath!("\\lx@stretchy@rightarrow", "\u{2192}",
    role => "ARROW", stretchy => true, alias => "\\rightarrow");

  //======================================================================
  // Section 4.10 Over and under arrows
  // Perl: amsmath.sty.ltxml lines 906-915
  DefMath!("\\underrightarrow{}", "\u{2192}",
    operator_role => "UNDERACCENT", operator_stretchy => true);
  DefMath!("\\underleftarrow{}", "\u{2190}",
    operator_role => "UNDERACCENT", operator_stretchy => true);
  DefMath!("\\overleftrightarrow{}", "\u{2194}",
    operator_role => "OVERACCENT", operator_stretchy => true);
  DefMath!("\\underleftrightarrow{}", "\u{2194}",
    operator_role => "UNDERACCENT", operator_stretchy => true);
  // Perl: amsmath.sty.ltxml Section 4.10 — Affixing symbols to other symbols
  DefConstructor!(
    "\\overset InScriptStyle {}",
    r###"<ltx:XMApp><ltx:XMWrap role='OVERACCENT'>#1</ltx:XMWrap><ltx:XMArg>#2</ltx:XMArg></ltx:XMApp>"###
  );
  DefConstructor!(
    "\\underset InScriptStyle {}",
    r###"<ltx:XMApp><ltx:XMWrap role='UNDERACCENT'>#1</ltx:XMWrap><ltx:XMArg>#2</ltx:XMArg></ltx:XMApp>"###
  );
  DefConstructor!(
    "\\overunderset InScriptStyle InScriptStyle {}",
    r###"<ltx:XMApp><ltx:XMWrap role='OVERACCENT'>#1</ltx:XMWrap><ltx:XMApp><ltx:XMWrap role='UNDERACCENT'>#2</ltx:XMWrap><ltx:XMArg>#3</ltx:XMArg></ltx:XMApp></ltx:XMApp>"###
  );

  //======================================================================
  // Section 4.11 Fractions and related commands

  // Section 4.11.1 The \frac, \dfrac, and \tfrac commands
  DefConstructor!(
    "\\tfrac ScriptStyle ScriptStyle",
    r###"<ltx:XMApp><ltx:XMTok meaning='divide' role='FRACOP' mathstyle='text'/><ltx:XMArg>#1</ltx:XMArg><ltx:XMArg>#2</ltx:XMArg></ltx:XMApp>"###
  );
  DefConstructor!(
    "\\dfrac TextStyle TextStyle",
    r###"<ltx:XMApp><ltx:XMTok meaning='divide' role='FRACOP' mathstyle='display'/><ltx:XMArg>#1</ltx:XMArg><ltx:XMArg>#2</ltx:XMArg></ltx:XMApp>"###
  );

  //======================================================================
  // Section 4.11.2 The \binom, \dbinom, and \tbinom commands
  DefMath!("\\binom{}{}", r"{\left({{#1}\atop{#2}}\right)}", meaning => "binomial");
  DefMath!("\\tbinom{}{}", r"{\textstyle\left({{#1}\atop{#2}}\right)}", meaning => "binomial");
  DefMath!("\\dbinom{}{}", r"{\displaystyle\left({{#1}\atop{#2}}\right)}", meaning => "binomial");

  //======================================================================
  // Section 4.11.3 The \genfrac command
  // Perl: amsmath.sty.ltxml lines 1016-1094
  // \genfrac{open}{close}{thickness}{style}{numerator}{denominator}
  DefMacro!("\\genfrac{}{}{}{}{}{}",
    r"\lx@genfrac{\if.#1.\else\lx@left#1\fi}{\if.#2.\else\lx@right#2\fi}{#3}{#4}{#5}{#6}");
  DefMacro!("\\lx@genfrac{}{}{}{}{}{}",
    r"\if @#3@\if.#4.\lx@@genfrac{#1}{#2}{#5}{#6}\else\lx@@genfrac{#1}{#2}[#4]{#5}{#6}\fi\else\if.#4.\lx@@genfrac{#1}[#3]{#2}{#5}{#6}\else\lx@@genfrac{#1}[#3]{#2}[#4]{#5}{#6}\fi\fi");

  // Perl: DefConstructor('\lx@@genfrac{}[Dimension]{}[Number]', ...)
  // NOTE: Perl reads numer/denom manually in afterDigest with MergeFont in scope.
  // We take 4 formal args; numer/denom are read manually in afterDigest.
  DefConstructor!(
    "\\lx@@genfrac {} [Dimension] {} [Number]",
    r###"?#needXMDual(<ltx:XMDual><ltx:XMApp><ltx:XMRef _xmkey='#xmkey0'/><ltx:XMRef _xmkey='#xmkey1'/><ltx:XMRef _xmkey='#xmkey2'/></ltx:XMApp><ltx:XMWrap>#open)()<ltx:XMApp><ltx:XMTok _xmkey='#xmkey0' role='#role' meaning='#meaning' mathstyle='#mathstyle' thickness='#thickness'/><ltx:XMArg _xmkey='#xmkey1'>#top</ltx:XMArg><ltx:XMArg _xmkey='#xmkey2'>#bottom</ltx:XMArg></ltx:XMApp>?#needXMDual(#close</ltx:XMWrap></ltx:XMDual>)(<ltx:XMApp><ltx:XMTok role='#role' meaning='#meaning' mathstyle='#mathstyle' thickness='#thickness'/><ltx:XMArg>#top</ltx:XMArg><ltx:XMArg>#bottom</ltx:XMArg></ltx:XMApp>)"###,
    alias => "\\genfrac",
    after_digest => sub[whatsit] {
      // Clone args upfront to avoid borrow conflicts with set_property
      let open = whatsit.get_arg(1).cloned();
      let thickness = whatsit.get_arg(2).cloned();
      let close = whatsit.get_arg(3).cloned();
      let stylecode_str = whatsit.get_arg(4).map(|a| a.to_attribute());

      let stylecode: Option<i64> = stylecode_str.as_ref().and_then(|s| s.parse::<i64>().ok());
      let mathstyle = match stylecode {
        None => {
          // Perl: LookupValue('font')->getMathstyle
          state::lookup_font()
            .and_then(|f| f.mathstyle.as_ref().map(|ms| ms.to_string()))
            .unwrap_or_default()
        },
        Some(0) => "display".to_string(),
        Some(1) => "text".to_string(),
        Some(2) => "script".to_string(),
        _ => "scriptscript".to_string(),
      };

      // Perl: $stomach->bgroup; MergeFont(mathstyle => $mathstyle); MergeFont(fraction => 1);
      // Read and digest numer/denom with font changes in scope
      bgroup();
      merge_font(Font { mathstyle: Some(Cow::Owned(mathstyle.clone())), ..Font::default() });
      merge_font(Font { fraction: Some(true), ..Font::default() });
      let numer_tokens = read_arg(ExpansionLevel::Full)?;
      let numer = digest(numer_tokens.clone())?;
      let denom_tokens = read_arg(ExpansionLevel::Full)?;
      let denom = digest(denom_tokens.clone())?;
      egroup()?;

      // thickness=0pt means no rule line (like \atop), so meaning is empty
      let thickness_str = thickness.as_ref().map(|t| t.to_attribute()).unwrap_or_default();
      let meaning = if thickness_str == "0.0pt" || thickness_str == "0pt" {
        String::new()
      } else {
        "divide".to_string()
      };

      let has_open = open.as_ref().is_some_and(|o| !o.to_string().trim().is_empty());
      let has_close = close.as_ref().is_some_and(|c| !c.to_string().trim().is_empty());

      if has_open || has_close {
        whatsit.set_property("needXMDual", "1");
        whatsit.set_property("xmkey0", get_xmarg_id()?);
        whatsit.set_property("xmkey1", get_xmarg_id()?);
        whatsit.set_property("xmkey2", get_xmarg_id()?);
      }
      if has_open {
        if let Some(ref o) = open { whatsit.set_property("open", o.clone()); }
      }
      if has_close {
        if let Some(ref c) = close { whatsit.set_property("close", c.clone()); }
      }
      whatsit.set_property("role", "FRACOP");
      if !meaning.is_empty() {
        whatsit.set_property("meaning", meaning);
      }
      if !mathstyle.is_empty() {
        whatsit.set_property("mathstyle", mathstyle);
      }
      if !thickness_str.is_empty() {
        whatsit.set_property("thickness", thickness_str);
      }
      whatsit.set_property("top", numer);
      whatsit.set_property("bottom", denom);

      // Build custom reversion: \genfrac{open_char}{close_char}{thickness}{style}{numer}{denom}
      // Perl: $open->getArg(1) to unwrap \lx@left whatsit, getting raw delimiter
      let mut rev_tokens: Vec<Token> = vec![T_CS!("\\genfrac"), T_BEGIN!()];
      // Extract raw delimiter from open arg (unwrap \lx@left whatsit)
      // Perl: $open = $open->getArg(1) if ref $open eq 'Whatsit'
      if let Some(ref o) = open {
        let reverted = o.revert()?;
        // Filter out CS tokens (\left, \lx@left) to keep just the delimiter char
        for t in reverted.unlist() {
          let cc = t.get_catcode();
          if cc != Catcode::CS && cc != Catcode::ESCAPE { rev_tokens.push(t); }
        }
      }
      rev_tokens.push(T_END!());
      rev_tokens.push(T_BEGIN!());
      if let Some(ref c) = close {
        let reverted = c.revert()?;
        for t in reverted.unlist() {
          let cc = t.get_catcode();
          if cc != Catcode::CS && cc != Catcode::ESCAPE { rev_tokens.push(t); }
        }
      }
      rev_tokens.push(T_END!());
      rev_tokens.push(T_BEGIN!());
      if let Some(ref th) = thickness {
        rev_tokens.extend(th.revert()?.unlist());
      }
      rev_tokens.push(T_END!());
      rev_tokens.push(T_BEGIN!());
      if let Some(sc) = whatsit.get_arg(4) {
        rev_tokens.extend(sc.revert()?.unlist());
      }
      rev_tokens.push(T_END!());
      rev_tokens.push(T_BEGIN!());
      rev_tokens.extend(numer_tokens.unlist());
      rev_tokens.push(T_END!());
      rev_tokens.push(T_BEGIN!());
      rev_tokens.extend(denom_tokens.unlist());
      rev_tokens.push(T_END!());
      whatsit.set_property("reversion", Stored::Tokens(Tokens::new(rev_tokens)));

      Ok(Vec::new())
    }
  );

  //======================================================================
  // Section 4.14.2 Vertical bar notations
  DefMath!("\\lvert", "|", role => "OPEN",  stretchy => false);
  DefMath!("\\lVert", "\u{2225}", role => "OPEN",  stretchy => false);
  DefMath!("\\rvert", "|", role => "CLOSE", stretchy => false);
  DefMath!("\\rVert", "\u{2225}", role => "CLOSE", stretchy => false);

  // Perl: amsmath.sty.ltxml line 85
  Let!("\\notag", "\\nonumber");

  // Perl: amsmath.sty.ltxml lines 87-91
  DefMacro!(
    "\\tag OptionalMatch:* {}",
    "\\lx@equation@settag{\\ifx#1*\\let\\fnum@equation\\relax\\fi\\expandafter\\def\\expandafter\\theequation\\expandafter{#2}\\lx@make@tags{equation}}"
  );

  // Perl: amsmath.sty.ltxml line 100
  DefConstructor!(
    "\\@ams@intertext{}",
    "<ltx:p class='ltx_intertext'>#1</ltx:p>",
    mode => "internal_vertical"
  );

  //======================================================================
  // Perl: amsmath.sty.ltxml lines 153-161
  DefPrimitive!("\\lx@ams@cr@binding", {
    state::let_i(
      &T_CS!("\\\\"),
      &T_CS!("\\lx@alignment@newline@noskip"),
      None,
    );
  });

  //======================================================================
  // Section 3.5 Equation groups without alignment (gather)
  // Perl: amsmath.sty.ltxml lines 382-415

  DefPrimitive!("\\@ams@gather@bindings", {
    ams_gather_bindings()?;
  });

  DefConstructor!("\\@@amsgather SkipSpaces DigestedBody",
    "#1",
    before_digest => { bgroup(); },
    after_construct => sub[document, _whatsit] {
      if let Some(mut last) = document.get_node().get_last_child() {
        rearrange_ams_gather(document, &mut last)?;
      }
    });
  DefPrimitive!("\\end@amsgather", { egroup()?; });

  DefMacro!("\\gather",
    "\\ifmmode\\let\\endgather\\endgathered\\gathered\\else\
     \\lx@hidden@bgroup\\@ams@gather@bindings\\@@amsgather\
     \\@equationgroup@numbering{numbered=1,postset=1,grouped=1,aligned=1}\
     \\lx@begin@alignment\\fi");
  DefMacro!("\\endgather",
    "\\lx@hidden@cr{}\\lx@end@alignment\\end@amsgather\\lx@hidden@egroup");
  DefMacro!("\\csname gather*\\endcsname",
    "\\ifmmode\\expandafter\\let\\csname endgather*\\endcsname\\endgathered\\gathered\\else\
     \\lx@hidden@bgroup\\@ams@gather@bindings\\@@amsgather\
     \\@equationgroup@numbering{numbered=0,postset=1,grouped=1,aligned=1}\
     \\lx@begin@alignment\\fi");
  DefMacro!("\\csname endgather*\\endcsname",
    "\\lx@hidden@cr{}\\lx@end@alignment\\end@amsgather\\lx@hidden@egroup");

  //======================================================================
  // Section 3.6 Equation groups with mutual alignment (align)
  // Perl: amsmath.sty.ltxml lines 443-551

  DefPrimitive!("\\@ams@align@bindings", {
    ams_align_bindings()?;
  });

  DefConstructor!("\\@@amsalign SkipSpaces DigestedBody",
    "#1",
    before_digest => { bgroup(); },
    after_construct => sub[document, _whatsit] {
      if let Some(mut last) = document.get_node().get_last_child() {
        rearrange_ams_align(document, &mut last)?;
      }
    });
  DefPrimitive!("\\end@amsalign", { egroup()?; });

  DefMacro!("\\align",
    "\\ifmmode\\let\\endalign\\endaligned\\aligned\\else\
     \\lx@hidden@bgroup\\@ams@align@bindings\\@@amsalign\
     \\@equationgroup@numbering{numbered=1,postset=1,grouped=1,aligned=1}\
     \\lx@begin@alignment\\fi",
    locked => true);
  DefMacro!("\\endalign",
    "\\lx@hidden@cr{}\\lx@end@alignment\\end@amsalign\\lx@hidden@egroup",
    locked => true);
  DefMacro!("\\csname align*\\endcsname",
    "\\ifmmode\\expandafter\\let\\csname endalign*\\endcsname\\endaligned\\aligned\\else\
     \\lx@hidden@bgroup\\@ams@align@bindings\\@@amsalign\
     \\@equationgroup@numbering{numbered=0,postset=1,grouped=1,aligned=1}\
     \\lx@begin@alignment\\fi",
    locked => true);
  DefMacro!("\\csname endalign*\\endcsname",
    "\\lx@hidden@cr{}\\lx@end@alignment\\end@amsalign\\lx@hidden@egroup",
    locked => true);

  // flalign — same as align for now (Perl treats it identically)
  DefMacro!("\\flalign",
    "\\ifmmode\\let\\endfalign\\endaligned\\aligned\\else\
     \\lx@hidden@bgroup\\@ams@align@bindings\\@@amsalign\
     \\@equationgroup@numbering{numbered=1,postset=1,grouped=1,aligned=1}\
     \\lx@begin@alignment\\fi");
  DefMacro!("\\endflalign",
    "\\lx@hidden@cr{}\\lx@end@alignment\\end@amsalign\\lx@hidden@egroup");
  DefMacro!("\\csname flalign*\\endcsname",
    "\\ifmmode\\expandafter\\let\\csname endfalign*\\endcsname\\endaligned\\aligned\\else\
     \\lx@hidden@bgroup\\@ams@align@bindings\\@@amsalign\
     \\@equationgroup@numbering{numbered=0,postset=1,grouped=1,aligned=1}\
     \\lx@begin@alignment\\fi");
  DefMacro!("\\csname endflalign*\\endcsname",
    "\\lx@hidden@cr{}\\lx@end@alignment\\end@amsalign\\lx@hidden@egroup");

  // alignat — same as align (ignores number-of-pairs arg)
  DefMacro!("\\alignat{}",
    "\\ifmmode\\let\\endalignat\\endalignedat\\alignedat{#1}\\else\
     \\lx@hidden@bgroup\\@ams@align@bindings\\@@amsalign\
     \\@equationgroup@numbering{numbered=1,postset=1,grouped=1,aligned=1}\
     \\lx@begin@alignment\\fi");
  DefMacro!("\\endalignat",
    "\\lx@hidden@cr{}\\lx@end@alignment\\end@amsalign\\lx@hidden@egroup");
  DefMacro!("\\csname alignat*\\endcsname{}",
    "\\ifmmode\\expandafter\\let\\csname endalignat*\\endcsname\\endalignedat\\alignedat{#1}\\else\
     \\lx@hidden@bgroup\\@ams@align@bindings\\@@amsalign\
     \\@equationgroup@numbering{numbered=0,postset=1,grouped=1,aligned=1}\
     \\lx@begin@alignment\\fi");
  DefMacro!("\\csname endalignat*\\endcsname",
    "\\lx@hidden@cr{}\\lx@end@alignment\\end@amsalign\\lx@hidden@egroup");

  //======================================================================
  // Section 3.3 Split equations without alignment (multline)
  // Perl: amsmath.sty.ltxml lines 240-310

  // Perl: \@ams@multirow@bindings — sets up single-column alignment template for multline
  DefPrimitive!("\\@ams@multirow@bindings RequiredKeyVals:multirow", sub[(kv)] {
    use latexml_core::alignment::cell::Cell;
    use latexml_core::alignment::template::TemplateConfig;
    let mut attrs: HashMap<String, String> = HashMap::default();
    if let Some(name_arg) = kv.get_value("name") {
      let name = name_arg.to_attribute();
      attrs.insert(String::from("name"), name);
    }
    // Single-column template: \hfil \displaystyle before
    let col1 = Cell {
      before: Some(Tokens::new(vec![T_CS!("\\hfil"), T_CS!("\\displaystyle")])),
      empty: true,
      ..Cell::default()
    };
    let template = Template::new(TemplateConfig {
      repeated: vec![col1],
      ..TemplateConfig::default()
    });
    ams_alignment_bindings(template, attrs);
  });

  DefConstructor!("\\@@multline DigestedBody",
    "<ltx:equation xml:id='#id'>#tags\
     <ltx:Math mode='display'><ltx:XMath>#1</ltx:XMath></ltx:Math>\
     </ltx:equation>",
    mode => "display_math",
    properties => { ref_step_counter("equation", false) },
    before_digest => { bgroup(); },
    after_digest => sub[whatsit] {
      whatsit.set_property("MULTIROW_ALIGNMENT_RULE_0", Stored::from("left"));
      whatsit.set_property("MULTIROW_ALIGNMENT_RULE_LAST", Stored::from("right"));
      // Perl: setBody(getArg(1)->unlist, undef) — sets body for tex= generation
      if let Some(arg) = whatsit.get_arg(1) {
        let mut body = arg.unlist();
        body.push(Digested::default()); // sentinel for trailer (popped by set_body)
        whatsit.set_body(body);
      }
    },
    reversion => "\\begin{multline}#1\\end{multline}",
    after_construct => sub[document, whatsit] {
      // Perl: lastChild->lastChild->lastChild->lastChild
      // equation > Math > XMath > XMArray
      let node = document.get_node();
      if let Some(array) = node.get_last_child()
        .and_then(|n| n.get_last_child())
        .and_then(|n| n.get_last_child())
        .and_then(|n| n.get_last_child())
      {
        let align_rule = get_multirow_alignment_rule(whatsit);
        rearrange_ams_multirow(document, array, &align_rule)?;
      }
    });
  DefConstructor!("\\@@multlinestar DigestedBody",
    "<ltx:equation>\
     <ltx:Math mode='display'><ltx:XMath>#body</ltx:XMath></ltx:Math>\
     </ltx:equation>",
    mode => "display_math",
    before_digest => { bgroup(); },
    after_digest => sub[whatsit] {
      whatsit.set_property("MULTIROW_ALIGNMENT_RULE_0", Stored::from("left"));
      whatsit.set_property("MULTIROW_ALIGNMENT_RULE_LAST", Stored::from("right"));
      if let Some(arg) = whatsit.get_arg(1) {
        let mut body = arg.unlist();
        body.push(Digested::default());
        whatsit.set_body(body);
      }
    },
    reversion => "\\begin{multline*}#1\\end{multline*}",
    after_construct => sub[document, whatsit] {
      let node = document.get_node();
      if let Some(array) = node.get_last_child()
        .and_then(|n| n.get_last_child())
        .and_then(|n| n.get_last_child())
        .and_then(|n| n.get_last_child())
      {
        let align_rule = get_multirow_alignment_rule(whatsit);
        rearrange_ams_multirow(document, array, &align_rule)?;
      }
    });
  DefPrimitive!("\\@end@multline", { egroup()?; });

  DefMacro!("\\multline",
    "\\ifmmode\\lx@hidden@bgroup\\@ams@multirow@bindings{name=multline}\\@@AmS@multline\\lx@begin@alignment\
     \\else\\lx@hidden@bgroup\\@ams@multirow@bindings{name=multline}\\@@multline\\lx@begin@alignment\\fi");
  DefMacro!("\\endmultline",
    "\\lx@hidden@cr{}\\lx@end@alignment\\@end@multline\\lx@hidden@egroup");
  DefMacro!("\\csname multline*\\endcsname",
    "\\lx@hidden@bgroup\\@ams@multirow@bindings{name=multline}\\@@multlinestar\\lx@begin@alignment");
  DefMacro!("\\csname endmultline*\\endcsname",
    "\\lx@hidden@cr{}\\lx@end@alignment\\@end@multline\\lx@hidden@egroup");
  // AmSTeX version (inside math)
  DefConstructor!("\\@@AmS@multline DigestedBody",
    "#body",
    mode => "display_math",
    before_digest => { bgroup(); },
    after_digest => sub[whatsit] {
      whatsit.set_property("MULTIROW_ALIGNMENT_RULE_0", Stored::from("left"));
      whatsit.set_property("MULTIROW_ALIGNMENT_RULE_LAST", Stored::from("right"));
      if let Some(arg) = whatsit.get_arg(1) {
        let mut body = arg.unlist();
        body.push(Digested::default());
        whatsit.set_body(body);
      }
    },
    reversion => "\\multline#1\\endmultline",
    after_construct => sub[document, whatsit] {
      // Perl: lastChild (directly XMArray since template is #body)
      if let Some(last) = document.get_node().get_last_child() {
        let align_rule = get_multirow_alignment_rule(whatsit);
        rearrange_ams_multirow(document, last, &align_rule)?;
      }
    });

  //======================================================================
  // Section 3.4 Split equations with alignment (split)
  // Perl: amsmath.sty.ltxml lines 333-378

  DefPrimitive!("\\@ams@aligned@bindings", {
    ams_aligned_bindings()?;
  });

  DefMacro!("\\split",
    "\\lx@hidden@bgroup\\@ams@aligned@bindings\\@@split\\lx@begin@alignment");
  DefMacro!("\\endsplit",
    "\\lx@hidden@cr{}\\lx@end@alignment\\@end@split\\lx@hidden@egroup");
  DefPrimitive!("\\@end@split", { egroup()?; });
  DefConstructor!("\\@@split DigestedBody",
    "#1",
    before_digest => { bgroup(); },
    reversion => "\\begin{split}#1\\end{split}",
    after_construct => sub[document, _whatsit] {
      if let Some(last) = document.get_node().get_last_child() {
        rearrange_ams_split(document, last)?;
      }
    });

  //======================================================================
  // Section 3.7 Alignment building blocks (gathered, aligned, alignedat)
  // Perl: amsmath.sty.ltxml lines 570-676

  DefMacro!("\\gathered[]",
    "\\lx@hidden@bgroup\\@@gathered\\lx@begin@alignment");
  DefMacro!("\\endgathered",
    "\\lx@hidden@cr{}\\lx@end@alignment\\@end@gathered\\lx@hidden@egroup");
  DefPrimitive!("\\@end@gathered", { egroup()?; });
  DefConstructor!("\\@@gathered DigestedBody",
    "#1",
    before_digest => { bgroup(); },
    reversion => "\\begin{gathered}#1\\end{gathered}");

  DefMacro!("\\aligned[]",
    "\\lx@hidden@bgroup\\@ams@aligned@bindings\\@@amsaligned\\lx@begin@alignment",
    locked => true);
  DefMacro!("\\endaligned",
    "\\lx@hidden@cr{}\\lx@end@alignment\\@end@amsaligned\\lx@hidden@egroup",
    locked => true);
  DefMacro!("\\alignedat{} []",
    "\\lx@hidden@bgroup\\@ams@aligned@bindings\\@@amsaligned\\lx@begin@alignment",
    locked => true);
  DefMacro!("\\endalignedat",
    "\\lx@hidden@cr{}\\lx@end@alignment\\@end@amsaligned\\lx@hidden@egroup",
    locked => true);
  DefPrimitive!("\\@end@amsaligned", { egroup()?; });
  DefConstructor!("\\@@amsaligned DigestedBody",
    "#1",
    before_digest => { bgroup(); },
    reversion => "\\begin{aligned} #1\\end{aligned}");

  //======================================================================
  // Perl: amsmath.sty.ltxml lines 1170-1175 — subarray/substack
  DefMacro!("\\substack{}", "\\begin{subarray}{c}#1\\end{subarray}");
  DefMacro!("\\subarray{}",
    "\\lx@ams@matrix{name=subarray,style=\\scriptsize,datameaning=list,rowsep=0pt,alignment=#1,alignment-required=true}");
  DefMacro!("\\endsubarray", "\\lx@end@ams@matrix");

  //======================================================================
  // subequations environment
  DefMacro!("\\subequations", "\\lx@equationgroup@subnumbering@begin");
  DefMacro!("\\endsubequations", "\\lx@equationgroup@subnumbering@end");

  DefMacro!("\\DOTSB", None);
  DefMacro!("\\DOTSI", None);
  DefMacro!("\\DOTSX", None);

  //======================================================================
  // Section 7.2 \sideset command
  // Perl: amsmath.sty.ltxml L1183-1234
  DefConstructor!("\\sideset{}{}{}", sub[document, args, props] {
    sideset_construct(document, args, props)?;
  },
  properties => {
    Ok(stored_map!("scriptlevel" => stomach::get_script_level()))
  });

  //======================================================================
  // Section 3.11.1 \numberwithin
  // Perl: amsmath.sty.ltxml line 741
  DefPrimitive!("\\numberwithin[]{}{}", sub[(format, counter, within)] {
    let format_str = if format.as_ref().is_none_or(|f| f.is_empty()) {
      s!("\\arabic")
    } else {
      format.unwrap().to_string()
    };
    let counter_str = counter.unwrap().to_string();
    let within_str = within.unwrap().to_string();
    new_counter(&counter_str, &within_str, None)?;
    let the_body = s!("\\csname the{within_str}\\endcsname.{format_str}{{{counter_str}}}");
    let expansion_tokens = latexml_core::mouth::tokenize(&the_body);
    def_macro(
      T_CS!(s!("\\the{counter_str}")),
      None,
      expansion_tokens,
      Some(ExpandableOptions { scope: Some(Scope::Global), ..Default::default() }),
    )?;
  });

  // Section 3.11.2 Cross references to equation numbers
  DefConstructor!("\\eqref Semiverbatim",
    "(<ltx:ref labelref='#label' _force_font='true'/>)",
    mode => "restricted_horizontal",
    properties => sub[args] {
      unpack_opt_ref!(args => label_opt);
      let label = label_opt.as_ref().unwrap().to_string();
      Ok(stored_map!("label" => Stored::String(arena::pin(clean_label(&label, None)))))
  });
  DefMacro!("\\thetag{}", "{\\rm #1}");

  // Perl: amsmath.sty.ltxml L882-896
  DefMacro!("\\boxed{}", "\\ifmmode\\boxed@math{#1}\\else\\boxed@text{#1}\\fi");
  DefConstructor!("\\boxed@math{}",
    "<ltx:XMArg enclose='box'>#1</ltx:XMArg>",
    alias => "\\boxed");
  DefConstructor!("\\boxed@text{}",
    "<ltx:Math mode='display' framed='rectangle'><ltx:XMath>#1</ltx:XMath></ltx:Math>",
    mode => "math",
    bounded => true,
    before_digest => { Let!("\\\\", "\\lx@newline"); },
    alias => "\\boxed");

  // Perl: amsmath.sty.ltxml L899-900
  DefMath!("\\implies", "\u{27F9}", role => "ARROW", meaning => "implies");
  DefMath!("\\impliedby", "\u{27F8}", role => "ARROW", meaning => "implied-by");

  // Perl: amsmath.sty.ltxml L1155 — \And for multi-author
  DefMath!("\\And", "&", role => "ADDOP", meaning => "and");

  // Perl: amsmath.sty.ltxml L1154-1157 — modular arithmetic
  // \bmod and \pmod are "already in LaTeX" (plain.rs) — do NOT redefine here
  DefMath!("\\mod", "mod", role => "MODIFIEROP", meaning => "modulo");
  DefMath!("\\pod{}", "(#1)", role => "MODIFIER", meaning => "modulo");

  // Perl: amsmath.sty.ltxml L1243-1250 — multiple integrals
  DefMath!("\\iint", "\u{222C}", role => "INTOP", meaning => "double-integral",
    mathstyle => "\\displaystyle");
  DefMath!("\\iiint", "\u{222D}", role => "INTOP", meaning => "triple-integral",
    mathstyle => "\\displaystyle");
  DefMath!("\\iiiint", "\u{2A0C}", role => "INTOP", meaning => "quadruple-integral",
    mathstyle => "\\displaystyle");
  DefMath!("\\idotsint", "\u{222B}\u{22EF}\u{222B}", role => "INTOP",
    meaning => "multiple-integral", mathstyle => "\\displaystyle");

  // Perl: amsmath.sty.ltxml L1283-1293 — italic Greek capitals
  DefMath!("\\varGamma", "\u{0393}", font => { shape => "italic" });
  DefMath!("\\varDelta", "\u{0394}", font => { shape => "italic" });
  DefMath!("\\varTheta", "\u{0398}", font => { shape => "italic" });
  DefMath!("\\varLambda", "\u{039B}", font => { shape => "italic" });
  DefMath!("\\varXi", "\u{039E}", font => { shape => "italic" });
  DefMath!("\\varPi", "\u{03A0}", font => { shape => "italic" });
  DefMath!("\\varSigma", "\u{03A3}", font => { shape => "italic" });
  DefMath!("\\varUpsilon", "\u{03A5}", font => { shape => "italic" });
  DefMath!("\\varPhi", "\u{03A6}", font => { shape => "italic" });
  DefMath!("\\varPsi", "\u{03A8}", font => { shape => "italic" });
  DefMath!("\\varOmega", "\u{03A9}", font => { shape => "italic" });

  // Perl: amsmath.sty.ltxml L1311-1319 — misc stubs
  DefMacro!("\\mintagsep", None);
  DefMacro!("\\minalignsep", "10pt");
  DefMacro!("\\primfrac{}", None);
  DefMacro!("\\shoveleft{}", "#1");
  DefMacro!("\\shoveright{}", "#1");
});

use latexml_core::document;

/// Extract the alignment rule from whatsit properties.
/// Perl stores as hash {0 => 'left', -1 => 'right', default => ...}
/// Rust stores as individual properties: MULTIROW_ALIGNMENT_RULE_0, MULTIROW_ALIGNMENT_RULE_LAST, etc.
fn get_multirow_alignment_rule(whatsit: &Whatsit) -> Vec<(String, String)> {
  let mut rules = Vec::new();
  if let Some(val) = whatsit.get_property("MULTIROW_ALIGNMENT_RULE_0") {
    if let Stored::String(s) = &*val {
      rules.push(("0".to_string(), arena::to_string(*s)));
    }
  }
  if let Some(val) = whatsit.get_property("MULTIROW_ALIGNMENT_RULE_LAST") {
    if let Stored::String(s) = &*val {
      rules.push(("last".to_string(), arena::to_string(*s)));
    }
  }
  rules
}

/// Perl: extractXMArrayCells (amsmath.sty.ltxml L165-197)
/// Extracts all math content from XMArray/XMRow/XMCell hierarchy, flattened.
/// Strips leading/trailing XMHint, deduplicates operators at row boundaries.
fn extract_xm_array_cells(array: &Node) -> Vec<Node> {
  use latexml_core::common::xml::element_nodes;
  let xmhint_sym = arena::pin_static("ltx:XMHint");
  let xmtok_sym = arena::pin_static("ltx:XMTok");
  let mut contents: Vec<Node> = Vec::new();
  let rows = element_nodes(array);
  for row in rows.iter() {
    let cells = element_nodes(row);
    for cell in cells.iter() {
      // XMCell should contain content directly (or via XMArg in some cases)
      let cell_children = element_nodes(cell);
      if cell_children.is_empty() {
        continue;
      }
      // Check if first child is an XMArg wrapper
      let arg_nodes: Vec<Node> = {
        let first = &cell_children[0];
        let qname = document::get_node_qname(first);
        if qname == arena::pin_static("ltx:XMArg") {
          element_nodes(first)
        } else {
          cell_children
        }
      };

      let mut nodes = arg_nodes;
      if nodes.is_empty() {
        continue;
      }

      // Strip leading & trailing XMHint nodes
      if document::get_node_qname(&nodes[0]) == xmhint_sym {
        nodes.remove(0);
      }
      if !nodes.is_empty() && document::get_node_qname(nodes.last().unwrap()) == xmhint_sym {
        nodes.pop();
      }

      // Deduplicate operators at row boundaries
      if let Some(prev) = contents.last() {
        if let Some(next) = nodes.first() {
          let prev_qname = document::get_node_qname(prev);
          let next_qname = document::get_node_qname(next);
          if prev_qname == xmtok_sym && next_qname == xmtok_sym {
            let prev_role = prev.get_attribute("role").unwrap_or_default();
            let next_role = next.get_attribute("role").unwrap_or_default();
            let prev_meaning = prev.get_attribute("meaning").unwrap_or_default();
            let next_meaning = next.get_attribute("meaning").unwrap_or_default();
            if prev_role == next_role
              && prev_meaning == next_meaning
              && matches!(prev_role.as_str(), "ADDOP" | "MULOP" | "RELOP")
            {
              contents.pop(); // Remove duplicate
            }
          }
        }
      }
      contents.extend(nodes);
    }
  }
  contents
}

/// Perl: rearrangeAMSSplit (amsmath.sty.ltxml L364-373)
/// Wraps XMArray in XMDual(XMWrap(refs), XMArray).
/// The XMWrap content is a flat list of all cells, which the math parser
/// will then parse as a regular expression.
fn rearrange_ams_split(document: &mut Document, mut array: Node) -> Result<()> {
  let array_qname = arena::to_string(document::get_node_qname(&array));
  if !array_qname.ends_with("XMArray") {
    return Ok(());
  }
  let mut cells = extract_xm_array_cells(&array);
  if cells.is_empty() {
    return Ok(());
  }

  // Ensure all content nodes have xml:ids, and collect XMRef idrefs
  let xmhint_sym = arena::pin_static("ltx:XMHint");
  let mut ref_ids: Vec<String> = Vec::new();
  for node in cells.iter_mut() {
    let qname = document::get_node_qname(node);
    if qname == xmhint_sym {
      continue; // Ephemeral, skip
    }
    // Generate xml:id if needed
    if !node.has_attribute_ns("id", "http://www.w3.org/XML/1998/namespace") {
      document.generate_id(node, "")?;
    }
    if let Some(id) = node
      .get_attribute_ns("id", "http://www.w3.org/XML/1998/namespace")
      .or_else(|| node.get_attribute("xml:id"))
    {
      ref_ids.push(id);
    }
  }

  // Build XMDual in-place:
  // 1. Get parent of XMArray
  // 2. Create XMDual at parent, before array
  // 3. Create XMWrap inside XMDual with XMRef children
  // 4. Move array into XMDual
  if let Some(mut parent) = array.get_parent() {
    let mut xm_dual = document.open_element_at(&mut parent, "ltx:XMDual", None, None)?;
    // Move XMDual before the array
    array.add_prev_sibling(&mut xm_dual).ok();

    // Create XMWrap inside XMDual
    let mut wrap_attrs: HashMap<String, String> = HashMap::default();
    wrap_attrs.insert("rule".to_string(), "Anything,".to_string());
    let mut xm_wrap =
      document.open_element_at(&mut xm_dual, "ltx:XMWrap", Some(wrap_attrs), None)?;
    // Add XMRef children
    for id in &ref_ids {
      let mut ref_attrs: HashMap<String, String> = HashMap::default();
      ref_attrs.insert("idref".to_string(), id.clone());
      let mut xm_ref =
        document.open_element_at(&mut xm_wrap, "ltx:XMRef", Some(ref_attrs), None)?;
      document.close_element_at(&mut xm_ref)?;
    }
    document.close_element_at(&mut xm_wrap)?;

    // Move XMArray into XMDual
    array.unlink_node();
    xm_dual.add_child(&mut array)?;

    document.close_element_at(&mut xm_dual)?;
  }

  Ok(())
}

/// Perl: rearrangeAMSMultirow (amsmath.sty.ltxml L286-307)
/// Like split, but also adjusts row alignment (first=left, last=right for multline).
fn rearrange_ams_multirow(
  document: &mut Document,
  array: Node,
  align_rules: &[(String, String)],
) -> Result<()> {
  use latexml_core::common::xml::element_nodes;
  let array_qname = arena::to_string(document::get_node_qname(&array));
  if !array_qname.ends_with("XMArray") {
    return Ok(());
  }
  // Apply alignment rules to rows
  let rows = element_nodes(&array);
  let num_rows = rows.len();
  for (key, align_val) in align_rules {
    let row_idx = if key == "last" {
      if num_rows > 0 { num_rows - 1 } else { continue; }
    } else if let Ok(idx) = key.parse::<usize>() {
      idx
    } else {
      continue;
    };
    if row_idx < num_rows {
      let row = &rows[row_idx];
      let cells = element_nodes(row);
      for mut cell in cells {
        cell.set_attribute("align", align_val).ok();
      }
    }
  }

  // Now do the same XMDual wrapping as split
  rearrange_ams_split(document, array)
}

/// Perl: rearrangeAMSGather (amsmath.sty.ltxml L400-415)
/// Each equation row consists of single equation. Pull math content up past _Capture_.
pub fn rearrange_ams_gather(
  document: &mut Document,
  equationgroup: &mut Node,
) -> Result<()> {
  let equations: Vec<Node> = document.findnodes("ltx:equation", Some(equationgroup));
  for mut equation in equations {
    let cells: Vec<Node> = document.findnodes("ltx:_Capture_", Some(&equation));
    if cells.is_empty() {
      continue;
    }
    let cell1_children: Vec<Node> = cells[0].get_child_elements();
    // Check if this equation is really an intertext
    if cells.len() == 1 && cell1_children.len() == 1 {
      let class = cell1_children[0].get_attribute("class").unwrap_or_default();
      if class.contains("ltx_intertext") {
        // Replace equation with the block
        let mut block = cell1_children[0].clone();
        block.unlink_node();
        equation.add_prev_sibling(&mut block).ok();
        equation.unlink_node();
        continue;
      }
    }
    if cells.len() == 1 && cell1_children.is_empty() {
      // Empty row — remove it
      equation.unlink_node();
      continue;
    }
    // Unwrap _Capture_ elements, set Math mode to display
    let children: Vec<Node> = equation.get_child_elements();
    for child in children {
      let qname = document::get_node_qname(&child);
      if qname == arena::pin_static("ltx:_Capture_") {
        document.unwrap_nodes(child)?;
      }
    }
    // Set mode='display' on Math elements
    let maths: Vec<Node> = document.findnodes("ltx:Math", Some(&equation));
    for mut math in maths {
      document.set_attribute(&mut math, "mode", "display")?;
    }
  }
  Ok(())
}

/// Perl: rearrangeAMSAlign (amsmath.sty.ltxml L460-473)
/// Each equation row consists of pairs (LHS, =RHS); group accordingly.
pub fn rearrange_ams_align(
  document: &mut Document,
  equationgroup: &mut Node,
) -> Result<()> {
  use crate::engine::base_xmath::equationgroup_join_cols;
  let equations: Vec<Node> = document.findnodes("ltx:equation", Some(equationgroup));
  for mut equation in equations {
    let cells: Vec<Node> = document.findnodes("ltx:_Capture_", Some(&equation));
    if cells.is_empty() {
      continue;
    }
    let cell1_children: Vec<Node> = cells[0].get_child_elements();
    // Check if this equation is really an intertext
    if cells.len() == 1 && cell1_children.len() == 1 {
      let class = cell1_children[0].get_attribute("class").unwrap_or_default();
      if class.contains("ltx_intertext") {
        let mut block = cell1_children[0].clone();
        block.unlink_node();
        equation.add_prev_sibling(&mut block).ok();
        equation.unlink_node();
        continue;
      }
    }
    if cells.len() == 1 && cell1_children.is_empty() {
      equation.unlink_node();
      continue;
    }
    // Group every 2 columns into a MathFork
    equationgroup_join_cols(document, 2, &mut equation)?;
  }
  Ok(())
}

/// Perl: \sideset constructor body (amsmath.sty.ltxml L1183-1225)
fn sideset_construct(
  document: &mut Document,
  args: &[Option<Digested>],
  props: &SymHashMap<Stored>,
) -> Result<()> {
  use crate::engine::tex_scripts::is_script;
  use latexml_core::token::Catcode;

  let pre = args.first().and_then(|a| a.as_ref());
  let post = args.get(1).and_then(|a| a.as_ref());
  let base = args.get(2).and_then(|a| a.as_ref());

  // Perl: Insert non-scripts from pre as XMWrap BEFORE the base
  if let Some(pre_arg) = pre {
    for item in pre_arg.unlist() {
      if is_script(&item).is_none() && !item.is_empty().unwrap_or(true) {
        document.open_element("ltx:XMWrap", None, None)?;
        document.absorb(&item, None)?;
        document.close_element("ltx:XMWrap")?;
      }
    }
  }

  // Insert the base in XMArg
  document.open_element("ltx:XMArg", None, None)?;
  if let Some(b) = base {
    document.absorb(b, None)?;
  }
  let node_opt = document.close_element("ltx:XMArg")?;
  let mut node = node_opt.unwrap_or_else(|| document.get_node().clone());

  // Get scriptpos prefix from base
  let opx = node.get_first_element_child()
    .and_then(|ch| ch.get_attribute("scriptpos"))
    .map(|sp| {
      let prefix: String = sp.chars().take_while(|c| !c.is_ascii_digit()).collect();
      if prefix.is_empty() { "post".to_string() } else { prefix }
    })
    .unwrap_or_else(|| "post".to_string());

  let level0 = props.get("scriptlevel")
    .map(|v| v.to_string().parse::<usize>().unwrap_or(0))
    .unwrap_or(0);
  let mut level = level0;

  // Process pre-scripts in reverse
  if let Some(pre_arg) = pre {
    let items: Vec<_> = pre_arg.unlist().into_iter().rev().collect();
    for item in items {
      if let Some(scriptop) = is_script(&item) {
        let y = if scriptop.1 == Catcode::SUPER { "SUPERSCRIPTOP" } else { "SUBSCRIPTOP" };
        node = sideset_wrap_impl(document, node, "pre", y, level, &item)?;
        if scriptop.0 == "FLOATING" {
          level += 1;
        }
      }
    }
  }

  // Process post-scripts; save non-scripts for insertion after
  let mut after: Vec<Digested> = Vec::new();
  if let Some(post_arg) = post {
    for item in post_arg.unlist() {
      if let Some(scriptop) = is_script(&item) {
        if scriptop.0 == "FLOATING" {
          level += 1;
        }
        let y = if scriptop.1 == Catcode::SUPER { "SUPERSCRIPTOP" } else { "SUBSCRIPTOP" };
        node = sideset_wrap_impl(document, node, "post", y, level, &item)?;
      } else if !item.is_empty().unwrap_or(true) {
        after.push(item);
      }
    }
  }

  // Set scriptpos on the final node
  if !opx.is_empty() {
    document.set_attribute(&mut node, "scriptpos", &format!("{opx}{level0}"))?;
  }

  // Perl: Insert non-script garbage from post AFTER the sideset structure
  for nonscript in &after {
    document.open_element("ltx:XMWrap", None, None)?;
    document.absorb(nonscript, None)?;
    document.close_element("ltx:XMWrap")?;
  }
  Ok(())
}

/// Perl: sidesetWrap (amsmath.sty.ltxml L1227-1234)
/// Uses Document's stack-based API (openElement/closeElement) matching Perl.
fn sideset_wrap_impl(
  document: &mut Document,
  inner: Node,
  x: &str,
  y: &str,
  level: usize,
  script: &Digested,
) -> Result<Node> {
  use latexml_core::digested::DigestedData;

  let scriptpos = format!("{x}{level}");
  // Perl: $document->openElement('ltx:XMApp')
  document.open_element("ltx:XMApp", None, None)?;
  // Perl: $document->insertElement('ltx:XMTok', undef, role => ..., scriptpos => ...)
  let mut tok_attrs: HashMap<String, String> = HashMap::default();
  tok_attrs.insert("role".to_string(), y.to_string());
  tok_attrs.insert("scriptpos".to_string(), scriptpos);
  document.open_element("ltx:XMTok", Some(tok_attrs), None)?;
  document.close_element("ltx:XMTok")?;
  // Perl: $new->appendChild($node) — move existing node into current element (XMApp)
  // Use append_tree to properly recreate the node tree (avoids libxml DOM corruption
  // from unlink_node + add_child on detached nodes).
  let inner_children = vec![inner.clone()];
  let mut current = document.get_node().clone();
  document.append_tree(&mut current, inner_children)?;
  document.remove_node(inner);
  // Perl: $document->insertElement('ltx:XMWrap', $script->getArg(1))
  document.open_element("ltx:XMWrap", None, None)?;
  if let DigestedData::Whatsit(ref w) = script.data() {
    if let Some(arg) = w.borrow().get_arg(1) {
      document.absorb(arg, None)?;
    }
  }
  document.close_element("ltx:XMWrap")?;
  // Perl: $document->closeElement('ltx:XMApp')
  let closed = document.close_element("ltx:XMApp")?;
  Ok(closed.unwrap_or_else(|| document.get_node().clone()))
}

