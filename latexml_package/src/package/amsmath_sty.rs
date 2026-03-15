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
          props.insert(String::from("xml:id"), id.to_string());
        }
      }
      document
        .open_element("ltx:equation", Some(props), None)
        .and(Ok(()))
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
  let attrs = HashMap::default();
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
  DefMacro!("\\smallmatrix", "\\lx@ams@matrix{name=smallmatrix,datameaning=matrix,style=\\scriptsize}");
  DefMacro!("\\endsmallmatrix", "\\lx@end@ams@matrix");
  DefMacro!("\\matrix@check{}", None);

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

  //======================================================================
  // Section 4.9 Extensible arrows
  // Perl: amsmath.sty.ltxml lines 921-950
  DefConstructor!(
    "\\lx@long@arrow DefToken {} OptionalInScriptStyle InScriptStyle",
    r###"?#3(<ltx:XMApp role='ARROW'><ltx:XMWrap role='UNDERACCENT'>#3</ltx:XMWrap><ltx:XMApp role='ARROW'><ltx:XMWrap role='OVERACCENT'>#4</ltx:XMWrap>#2</ltx:XMApp></ltx:XMApp>)(<ltx:XMApp role='ARROW'><ltx:XMWrap role='OVERACCENT'>#4</ltx:XMWrap>#2</ltx:XMApp>)"###
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
  // (overset/underset already in LaTeX core via latex_ch7)
  // \overunderset is amsmath-specific
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

      let has_open = open.as_ref().map_or(false, |o| !o.to_string().trim().is_empty());
      let has_close = close.as_ref().map_or(false, |c| !c.to_string().trim().is_empty());

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
    before_digest => { bgroup(); });
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
    before_digest => { bgroup(); });
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

  DefConstructor!("\\@@multline DigestedBody",
    "<ltx:equation xml:id='#id'>#tags\
     <ltx:Math mode='display'><ltx:XMath>#1</ltx:XMath></ltx:Math>\
     </ltx:equation>",
    mode => "display_math",
    properties => { ref_step_counter("equation", false) },
    before_digest => { bgroup(); },
    reversion => "\\begin{multline}#1\\end{multline}");
  DefConstructor!("\\@@multlinestar DigestedBody",
    "<ltx:equation>\
     <ltx:Math mode='display'><ltx:XMath>#body</ltx:XMath></ltx:Math>\
     </ltx:equation>",
    mode => "display_math",
    before_digest => { bgroup(); },
    reversion => "\\begin{multline*}#1\\end{multline*}");
  DefPrimitive!("\\@end@multline", { egroup()?; });

  DefMacro!("\\multline",
    "\\ifmmode\\lx@hidden@bgroup\\@@AmS@multline\\lx@begin@alignment\
     \\else\\lx@hidden@bgroup\\@@multline\\fi");
  DefMacro!("\\endmultline",
    "\\@end@multline\\lx@hidden@egroup");
  DefMacro!("\\csname multline*\\endcsname",
    "\\lx@hidden@bgroup\\@@multlinestar");
  DefMacro!("\\csname endmultline*\\endcsname",
    "\\@end@multline\\lx@hidden@egroup");
  // AmSTeX version (inside math)
  DefConstructor!("\\@@AmS@multline DigestedBody",
    "#body",
    mode => "display_math",
    before_digest => { bgroup(); },
    reversion => "\\multline#1\\endmultline");

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
    reversion => "\\begin{split}#1\\end{split}");

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
  // Perl: amsmath.sty.ltxml lines 93-95
  DefMacro!("\\DOTSB", None);
  DefMacro!("\\DOTSI", None);
  DefMacro!("\\DOTSX", None);
});
