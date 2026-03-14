use crate::prelude::*;
//**********************************************************************
// See amsldoc
//
// Currently only a random collection of things I (Bruce) need for DLMF chapters.
// Eventually, go through the doc and implement it all.
//**********************************************************************

// DG:
// TODO: Most of this binding is not ported yet.

LoadDefinitions!({
  Let!("\\@xp", "\\expandafter");
  Let!("\\@nx", "\\noexpand");
  // sub-packages:
  RequirePackage!("amsbsy");
  RequirePackage!("amstext");
  RequirePackage!("amsopn");

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
});
