use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: mathtools.sty.ltxml
  // Options: fixamsmath, donotfixamsmathbugs, allowspaces, disallowspaces — all ignored
  for option in ["fixamsmath", "donotfixamsmathbugs", "allowspaces", "disallowspaces"].iter() {
    DeclareOption!(*option, None);
  }
  // Pass all other options to amsmath
  DeclareOption!(None, {
    Digest!("\\PassOptionsToPackage{\\CurrentOption}{amsmath}")?;
  });
  ProcessOptions!();

  RequirePackage!("keyval");
  RequirePackage!("calc");
  // TODO: add support for mhsetup
  // RequirePackage!("mhsetup");
  // Perl L43: RequirePackage('amsmath', withoptions => 1)
  require_package_with_options("amsmath")?;
  // Perl: AtBeginDocument(sub { RequirePackage('graphicx'); });
  at_begin_document(TokenizeInternal!(r"\RequirePackage{graphicx}"))?;

  //======================================================================
  // 3 — Macros
  //======================================================================

  // mt keyset for `\mathtoolsset`. Perl `mathtools.sty.ltxml` doesn't
  // call DefKeyVal for any of these — `\mathtoolsset` stashes whatever
  // pairs you hand it as `\@mt@mathtoolsset@<key>` macros and looks
  // them up via `\@mt@getmtoption`. With `21e730e71e`'s Info→Warn
  // promotion, mathtools-using papers emit a Warn per call. Rust-only
  // divergence: register the documented mathtools.dtx options.
  for key in [
    "showonlyrefs", "showmanualtags",
    "mathic", "centercolon", "prescript-arg-format",
    "prescript-sub-format", "prescript-sup-format",
    "smallmatrix-align", "smallmatrix-inner-space",
    "multlined-pos", "multlined-width",
    "shortvdotsadjustabove", "shortvdotsadjustbelow",
    "firstline-afterskip", "lastline-preskip",
    "centered-mhchem-above-below",
  ] {
    DefKeyVal!("mt", key, "");
  }

  // \mathtoolsset — stores keyval pairs as macros \@mt@mathtoolsset@<key>
  // Perl: DefPrimitive('\mathtoolsset RequiredKeyVals', sub { ... getPairs ... DefMacro })
  DefPrimitive!("\\mathtoolsset RequiredKeyVals:mt", sub[(kv)] {
    for (key, val) in kv.get_pairs() {
      let val_str = if val.is_empty() { "\\@mt@true".to_string() } else { val.to_string() };
      let cs_name = s!("\\@mt@mathtoolsset@{}", key);
      def_macro(T_CS!(&cs_name), None, Tokenize!(&val_str), None)?;
    }
  });

  // Lookup function for mathtoolsset
  DefMacro!("\\@mt@getmtoption{}",
    "\\ifcsname @mt@mathtoolsset@#1\\endcsname\
     \\expandafter\\let\\expandafter\\@mt@currentvalue\\csname @mt@mathtoolsset@#1\\endcsname\\else\
     \\let\\@mt@currentvalue\\relax\\fi\
     \\@mt@currentvalue");

  //======================================================================
  // 3.1
  //======================================================================

  // Perl mathtools.sty.ltxml has `enterHorizontal=>1`. The previous
  // comment claimed enter_horizontal "not supported in template form"
  // but that's no longer the case — DefConstructor! macro accepts the
  // flag with bare-template body (see cycle 91 xcolor \fcolorbox,
  // cycle 88 acmart affiliation fields, etc.). Match Perl on the flag.
  DefConstructor!("\\mathmbox{}", "#1", enter_horizontal => true);

  // \mathllap — zero-width math overlap (left): xoffset = -width
  DefConstructor!("\\mathllap[]{}",
    "<ltx:XMArg width='0pt' ?#xoffset(xoffset='#xoffset')>#2</ltx:XMArg>",
    after_digest => sub[whatsit] {
      if let Ok(Some(RegisterValue::Dimension(w))) = whatsit.get_width(None) {
        let neg = w.negate();
        whatsit.set_property("xoffset", Stored::String(pin(neg.to_attribute())));
      }
      whatsit.set_width(Stored::String(pin_static("0pt")));
    });
  // \mathrlap — zero-width math overlap (right): no xoffset needed
  DefConstructor!("\\mathrlap[]{}",
    "<ltx:XMArg width='0pt' ?#xoffset(xoffset='#xoffset')>#2</ltx:XMArg>",
    after_digest => sub[whatsit] {
      whatsit.set_width(Stored::String(pin_static("0pt")));
    });
  // \mathclap — zero-width math overlap (center): xoffset = -0.5 * width
  DefConstructor!("\\mathclap[]{}",
    "<ltx:XMArg width='0pt' ?#xoffset(xoffset='#xoffset')>#2</ltx:XMArg>",
    after_digest => sub[whatsit] {
      if let Ok(Some(RegisterValue::Dimension(w))) = whatsit.get_width(None) {
        let half_neg = w.multiply(Float::new_f64(-0.5));
        whatsit.set_property("xoffset", Stored::String(pin(half_neg.to_attribute())));
      }
      whatsit.set_width(Stored::String(pin_static("0pt")));
    });

  DefConstructor!("\\clap{}", "#1");
  DefConstructor!("\\mathmakebox[][]{}", "#3");
  // Ignoring cramped, for now
  DefConstructor!("\\cramped[]{}", "#2");

  // Same as \mathllap, etc (but also cramped!)
  Let!("\\crampedllap", "\\mathllap");
  Let!("\\crampedrlap", "\\mathrlap");
  Let!("\\crampedclap", "\\mathclap");

  // \smashoperator — destructures argument to recognize operators and scripts.
  // Perl: \smashoperator[align]{op_sub_sup} → destructure → \lx@@smashoperator{align}{op}{sub}{sup}
  // \smashoperator — passes operator+scripts through directly.
  // Perl L112-177: full decomposition with SUMOP structure and width zeroing.
  // Our math parser can't handle the SUMOP structure yet, so we use the simple
  // passthrough. The visual width "smashing" is cosmetic.
  DefMacro!("\\smashoperator[]{}", "#2");

  // \adjustlimits — Perl mathtools.sty.ltxml L180-199: a DefConstructor
  // building two `<ltx:XMApp>` SUBSCRIPTOP nodes directly. We MUST use the
  // constructor form (not a `#1_{#3}#4_{#6}` DefMacro): the DefMacro
  // re-emits `_` tokens, so when `\adjustlimits` is MISUSED with a single
  // operator (e.g. `\adjustlimits\sup_{x\in R} |\mbox{F}_{…}` — the macro
  // greedily grabs `| \mbox {F}` as the second op/sub) the trailing `_{…}`
  // collides with the re-emitted subscript → a digestion-time
  // `Error:unexpected:double-subscript`, where Perl (building the script
  // structure directly) only warns at the parser. Witness 2010.00165
  // (`\adjustlimits\sup_…`: Perl 0 err, Rust 3). The cosmetic
  // `depth='#limdepth'`/`height='#subheight'` alignment from Perl's
  // afterDigest is omitted (both math backends ignore it — WISDOM #44).
  DefConstructor!("\\adjustlimits {} DefToken InScriptStyle {} DefToken InScriptStyle",
    "<ltx:XMApp><ltx:XMTok role='SUBSCRIPTOP' scriptpos='mid'/><ltx:XMArg>#1</ltx:XMArg><ltx:XMArg>#3</ltx:XMArg></ltx:XMApp><ltx:XMApp><ltx:XMTok role='SUBSCRIPTOP' scriptpos='mid'/><ltx:XMArg>#4</ltx:XMArg><ltx:XMArg>#6</ltx:XMArg></ltx:XMApp>");

  DefConstructor!("\\SwapAboveDisplaySkip", "");

  //======================================================================
  // 3.2 — Tag forms
  //======================================================================

  // \newtagform{name}[style]{open}{close} — Perl: DefPrimitive calling defMTTagForm
  // Creates \fnum@equation@MT@{name}, \ref@equation@MT@{name}, \@MTStag@{name}
  DefPrimitive!("\\newtagform{} [] {}{}", sub[(name_arg, style_opt, open_arg, close_arg)] {
    use latexml_core::common::def_parser::parse_parameters;

    let name = name_arg.to_string();
    // Perl L231-234: skip redefinition with Error('ignore', ...).
    let mtstag_cs = T_CS!(&s!("\\@MTStag@{}", name));
    if !is_definable(&mtstag_cs) {
      Error!("ignore", mtstag_cs, "Ignoring redefinition (\\newtagform) of '{}'", name);
    } else {
    // Perl: $open->unlist, $close->unlist, $style->unlist — preserve CS tokens
    let open_toks: Vec<Token> = open_arg.unlist();
    let close_toks: Vec<Token> = close_arg.unlist();
    let style_toks: Vec<Token> = if let Some(s) = style_opt {
      s.unlist()
    } else { Vec::new() };

    // Define \fnum@equation@MT@{name} = {open [style] {\theequation} close}
    let mut fnum_body = vec![T_BEGIN!()];
    fnum_body.extend(open_toks.iter().cloned());
    fnum_body.extend(style_toks.iter().cloned());
    fnum_body.push(T_BEGIN!());
    fnum_body.push(T_CS!("\\theequation"));
    fnum_body.push(T_END!());
    fnum_body.extend(close_toks.iter().cloned());
    fnum_body.push(T_END!());
    let fnum_cs_name = s!("\\fnum@equation@MT@{}", name);
    def_macro(T_CS!(&fnum_cs_name), None, Tokens::new(fnum_body),
      Some(ExpandableOptions { scope: Some(Scope::Global), ..Default::default() }))?;

    // Define \ref@equation@MT@{name} Semiverbatim = {open [style] {\ref{#1}} close}
    let mut ref_body = vec![T_BEGIN!()];
    ref_body.extend(open_toks.iter().cloned());
    ref_body.extend(style_toks.iter().cloned());
    ref_body.push(T_BEGIN!());
    ref_body.push(T_CS!("\\ref"));
    ref_body.push(T_BEGIN!());
    ref_body.push(T_PARAM!());
    ref_body.push(T_OTHER!("1"));
    ref_body.push(T_END!());
    ref_body.push(T_END!());
    ref_body.extend(close_toks.iter().cloned());
    ref_body.push(T_END!());
    let ref_cs_name = s!("\\ref@equation@MT@{}", name);
    let ref_params = parse_parameters("Semiverbatim", &T_CS!(&ref_cs_name), true)?;
    def_macro(T_CS!(&ref_cs_name), ref_params, Tokens::new(ref_body),
      Some(ExpandableOptions { scope: Some(Scope::Global), ..Default::default() }))?;

    // Define \@MTStag@{name} as a primitive that redefines \fnum@equation and \eqref
    let fnum_cs_clone = fnum_cs_name;
    let ref_cs_clone = ref_cs_name;
    def_primitive(
      T_CS!(&s!("\\@MTStag@{}", name)),
      None,
      Some(PrimitiveBody::Closure(Rc::new(move |_args| {
        def_macro(T_CS!("\\fnum@equation"), None, Tokens!(T_CS!(&fnum_cs_clone)),
          Some(ExpandableOptions { scope: Some(Scope::Global), ..Default::default() }))?;
        let eqref_params = parse_parameters("Semiverbatim", &T_CS!("\\eqref"), true)?;
        def_macro(T_CS!("\\eqref"), eqref_params,
          Tokens::new(vec![T_CS!(&ref_cs_clone), T_BEGIN!(), T_PARAM!(), T_OTHER!("1"), T_END!()]),
          Some(ExpandableOptions { scope: Some(Scope::Global), ..Default::default() }))?;
        Ok(Vec::new())
      }))),
      PrimitiveOptions::default(),
    )?;
    }
  });

  // \renewtagform — same logic as \newtagform (Perl skips isDefinable check)
  DefPrimitive!("\\renewtagform{} [] {}{}", sub[(name_arg, style_opt, open_arg, close_arg)] {
    use latexml_core::common::def_parser::parse_parameters;
    let name = name_arg.to_string();
    let open_toks: Vec<Token> = open_arg.unlist();
    let close_toks: Vec<Token> = close_arg.unlist();
    let style_toks: Vec<Token> = if let Some(s) = style_opt {
      s.unlist()
    } else { Vec::new() };
    // Same body as \newtagform:
    let mut fnum_body = vec![T_BEGIN!()];
    fnum_body.extend(open_toks.iter().cloned());
    fnum_body.extend(style_toks.iter().cloned());
    fnum_body.push(T_BEGIN!());
    fnum_body.push(T_CS!("\\theequation"));
    fnum_body.push(T_END!());
    fnum_body.extend(close_toks.iter().cloned());
    fnum_body.push(T_END!());
    let fnum_cs_name = s!("\\fnum@equation@MT@{}", name);
    def_macro(T_CS!(&fnum_cs_name), None, Tokens::new(fnum_body),
      Some(ExpandableOptions { scope: Some(Scope::Global), ..Default::default() }))?;
    let mut ref_body = vec![T_BEGIN!()];
    ref_body.extend(open_toks.iter().cloned());
    ref_body.extend(style_toks.iter().cloned());
    ref_body.push(T_BEGIN!());
    ref_body.push(T_CS!("\\ref"));
    ref_body.push(T_BEGIN!());
    ref_body.push(T_PARAM!());
    ref_body.push(T_OTHER!("1"));
    ref_body.push(T_END!());
    ref_body.push(T_END!());
    ref_body.extend(close_toks.iter().cloned());
    ref_body.push(T_END!());
    let ref_cs_name = s!("\\ref@equation@MT@{}", name);
    let ref_params = parse_parameters("Semiverbatim", &T_CS!(&ref_cs_name), true)?;
    def_macro(T_CS!(&ref_cs_name), ref_params, Tokens::new(ref_body),
      Some(ExpandableOptions { scope: Some(Scope::Global), ..Default::default() }))?;
    let fnum_cs_clone = fnum_cs_name;
    let ref_cs_clone = ref_cs_name;
    def_primitive(T_CS!(&s!("\\@MTStag@{}", name)), None,
      Some(PrimitiveBody::Closure(Rc::new(move |_args| {
        def_macro(T_CS!("\\fnum@equation"), None, Tokens!(T_CS!(&fnum_cs_clone)),
          Some(ExpandableOptions { scope: Some(Scope::Global), ..Default::default() }))?;
        let eqref_params = parse_parameters("Semiverbatim", &T_CS!("\\eqref"), true)?;
        def_macro(T_CS!("\\eqref"), eqref_params,
          Tokens::new(vec![T_CS!(&ref_cs_clone), T_BEGIN!(), T_PARAM!(), T_OTHER!("1"), T_END!()]),
          Some(ExpandableOptions { scope: Some(Scope::Global), ..Default::default() }))?;
        Ok(Vec::new())
      }))), PrimitiveOptions::default())?;
  });

  DefMacro!("\\usetagform{}", "\\csname @MTStag@#1\\endcsname");

  // Initialize default tag form — creates \fnum@equation@MT@default and \@MTStag@default
  // These are needed for \usetagform{default} to work (switches back to standard format)
  RawTeX!("\\newtagform{default}{(}{)}");

  Let!("\\refeq", "\\ref");
  DefMacro!("\\noeqref{}", None);

  //======================================================================
  // 3.3 — Extensible arrows
  //======================================================================

  DefConstructor!("\\xleftrightarrow OptionalInScriptStyle InScriptStyle",
    "?#1(\
     <ltx:XMApp role='METARELOP'>\
     <ltx:XMWrap role='UNDERACCENT'>#1</ltx:XMWrap>\
     <ltx:XMApp>\
     <ltx:XMWrap role='OVERACCENT'>#2</ltx:XMWrap>\
     <ltx:XMTok name='xleftrightarrow' role='METARELOP' stretchy='true'>\u{2194}</ltx:XMTok>\
     </ltx:XMApp>\
     </ltx:XMApp>\
     )(\
     <ltx:XMApp role='METARELOP'>\
     <ltx:XMWrap role='OVERACCENT'>#2</ltx:XMWrap>\
     <ltx:XMTok name='xleftrightarrow' role='METARELOP' stretchy='true'>\u{2194}</ltx:XMTok>\
     </ltx:XMApp>\
     )");

  DefConstructor!("\\xLeftarrow OptionalInScriptStyle InScriptStyle",
    "?#1(\
     <ltx:XMApp role='ARROW'>\
     <ltx:XMWrap role='UNDERACCENT'>#1</ltx:XMWrap>\
     <ltx:XMApp>\
     <ltx:XMWrap role='OVERACCENT'>#2</ltx:XMWrap>\
     <ltx:XMTok name='xLeftarrow' role='ARROW' stretchy='true'>\u{21D0}</ltx:XMTok>\
     </ltx:XMApp>\
     </ltx:XMApp>\
     )(\
     <ltx:XMApp role='ARROW'>\
     <ltx:XMWrap role='OVERACCENT'>#2</ltx:XMWrap>\
     <ltx:XMTok name='xLeftarrow' role='ARROW' stretchy='true'>\u{21D0}</ltx:XMTok>\
     </ltx:XMApp>\
     )");

  DefConstructor!("\\xRightarrow OptionalInScriptStyle InScriptStyle",
    "?#1(\
     <ltx:XMApp role='ARROW'>\
     <ltx:XMWrap role='UNDERACCENT'>#1</ltx:XMWrap>\
     <ltx:XMApp>\
     <ltx:XMWrap role='OVERACCENT'>#2</ltx:XMWrap>\
     <ltx:XMTok name='xRightarrow' role='ARROW' stretchy='true'>\u{21D2}</ltx:XMTok>\
     </ltx:XMApp>\
     </ltx:XMApp>\
     )(\
     <ltx:XMApp role='ARROW'>\
     <ltx:XMWrap role='OVERACCENT'>#2</ltx:XMWrap>\
     <ltx:XMTok name='xRightarrow' role='ARROW' stretchy='true'>\u{21D2}</ltx:XMTok>\
     </ltx:XMApp>\
     )");

  DefConstructor!("\\xLeftrightarrow OptionalInScriptStyle InScriptStyle",
    "?#1(\
     <ltx:XMApp role='ARROW'>\
     <ltx:XMWrap role='UNDERACCENT'>#1</ltx:XMWrap>\
     <ltx:XMApp>\
     <ltx:XMWrap role='OVERACCENT'>#2</ltx:XMWrap>\
     <ltx:XMTok name='xLeftrightarrow' role='ARROW' stretchy='true'>\u{21D4}</ltx:XMTok>\
     </ltx:XMApp>\
     </ltx:XMApp>\
     )(\
     <ltx:XMApp role='ARROW'>\
     <ltx:XMWrap role='OVERACCENT'>#2</ltx:XMWrap>\
     <ltx:XMTok name='xLeftrightarrow' role='ARROW' stretchy='true'>\u{21D4}</ltx:XMTok>\
     </ltx:XMApp>\
     )");

  DefConstructor!("\\xhookleftarrow OptionalInScriptStyle InScriptStyle",
    "?#1(\
     <ltx:XMApp role='ARROW'>\
     <ltx:XMWrap role='UNDERACCENT'>#1</ltx:XMWrap>\
     <ltx:XMApp>\
     <ltx:XMWrap role='OVERACCENT'>#2</ltx:XMWrap>\
     <ltx:XMTok name='xhookleftarrow' role='ARROW' stretchy='true'>\u{21A9}</ltx:XMTok>\
     </ltx:XMApp>\
     </ltx:XMApp>\
     )(\
     <ltx:XMApp role='ARROW'>\
     <ltx:XMWrap role='OVERACCENT'>#2</ltx:XMWrap>\
     <ltx:XMTok name='xhookleftarrow' role='ARROW' stretchy='true'>\u{21A9}</ltx:XMTok>\
     </ltx:XMApp>\
     )");

  DefConstructor!("\\xhookrightarrow OptionalInScriptStyle InScriptStyle",
    "?#1(\
     <ltx:XMApp role='ARROW'>\
     <ltx:XMWrap role='UNDERACCENT'>#1</ltx:XMWrap>\
     <ltx:XMApp>\
     <ltx:XMWrap role='OVERACCENT'>#2</ltx:XMWrap>\
     <ltx:XMTok name='xhookrightarrow' role='ARROW' stretchy='true'>\u{21AA}</ltx:XMTok>\
     </ltx:XMApp>\
     </ltx:XMApp>\
     )(\
     <ltx:XMApp role='ARROW'>\
     <ltx:XMWrap role='OVERACCENT'>#2</ltx:XMWrap>\
     <ltx:XMTok name='xhookrightarrow' role='ARROW' stretchy='true'>\u{21AA}</ltx:XMTok>\
     </ltx:XMApp>\
     )");

  DefConstructor!("\\xmapsto OptionalInScriptStyle InScriptStyle",
    "?#1(\
     <ltx:XMApp role='ARROW'>\
     <ltx:XMWrap role='UNDERACCENT'>#1</ltx:XMWrap>\
     <ltx:XMApp>\
     <ltx:XMWrap role='OVERACCENT'>#2</ltx:XMWrap>\
     <ltx:XMTok name='xmapsto' role='ARROW' stretchy='true'>\u{21A6}</ltx:XMTok>\
     </ltx:XMApp>\
     </ltx:XMApp>\
     )(\
     <ltx:XMApp role='ARROW'>\
     <ltx:XMWrap role='OVERACCENT'>#2</ltx:XMWrap>\
     <ltx:XMTok name='xmapsto' role='ARROW' stretchy='true'>\u{21A6}</ltx:XMTok>\
     </ltx:XMApp>\
     )");

  DefConstructor!("\\xrightharpoondown OptionalInScriptStyle InScriptStyle",
    "?#1(\
     <ltx:XMApp role='ARROW'>\
     <ltx:XMWrap role='UNDERACCENT'>#1</ltx:XMWrap>\
     <ltx:XMApp>\
     <ltx:XMWrap role='OVERACCENT'>#2</ltx:XMWrap>\
     <ltx:XMTok name='xrightharpoondown' role='ARROW' stretchy='true'>\u{21C1}</ltx:XMTok>\
     </ltx:XMApp>\
     </ltx:XMApp>\
     )(\
     <ltx:XMApp role='ARROW'>\
     <ltx:XMWrap role='OVERACCENT'>#2</ltx:XMWrap>\
     <ltx:XMTok name='xrightharpoondown' role='ARROW' stretchy='true'>\u{21C1}</ltx:XMTok>\
     </ltx:XMApp>\
     )");

  DefConstructor!("\\xrightharpoonup OptionalInScriptStyle InScriptStyle",
    "?#1(\
     <ltx:XMApp role='ARROW'>\
     <ltx:XMWrap role='UNDERACCENT'>#1</ltx:XMWrap>\
     <ltx:XMApp>\
     <ltx:XMWrap role='OVERACCENT'>#2</ltx:XMWrap>\
     <ltx:XMTok name='xrightharpoonup' role='ARROW' stretchy='true'>\u{21C0}</ltx:XMTok>\
     </ltx:XMApp>\
     </ltx:XMApp>\
     )(\
     <ltx:XMApp role='ARROW'>\
     <ltx:XMWrap role='OVERACCENT'>#2</ltx:XMWrap>\
     <ltx:XMTok name='xrightharpoonup' role='ARROW' stretchy='true'>\u{21C0}</ltx:XMTok>\
     </ltx:XMApp>\
     )");

  DefConstructor!("\\xleftharpoondown OptionalInScriptStyle InScriptStyle",
    "?#1(\
     <ltx:XMApp role='ARROW'>\
     <ltx:XMWrap role='UNDERACCENT'>#1</ltx:XMWrap>\
     <ltx:XMApp>\
     <ltx:XMWrap role='OVERACCENT'>#2</ltx:XMWrap>\
     <ltx:XMTok name='xleftharpoondown' role='ARROW' stretchy='true'>\u{21BD}</ltx:XMTok>\
     </ltx:XMApp>\
     </ltx:XMApp>\
     )(\
     <ltx:XMApp role='ARROW'>\
     <ltx:XMWrap role='OVERACCENT'>#2</ltx:XMWrap>\
     <ltx:XMTok name='xleftharpoondown' role='ARROW' stretchy='true'>\u{21BD}</ltx:XMTok>\
     </ltx:XMApp>\
     )");

  DefConstructor!("\\xleftharpoonup OptionalInScriptStyle InScriptStyle",
    "?#1(\
     <ltx:XMApp role='ARROW'>\
     <ltx:XMWrap role='UNDERACCENT'>#1</ltx:XMWrap>\
     <ltx:XMApp>\
     <ltx:XMWrap role='OVERACCENT'>#2</ltx:XMWrap>\
     <ltx:XMTok name='xleftharpoonup' role='ARROW' stretchy='true'>\u{21BC}</ltx:XMTok>\
     </ltx:XMApp>\
     </ltx:XMApp>\
     )(\
     <ltx:XMApp role='ARROW'>\
     <ltx:XMWrap role='OVERACCENT'>#2</ltx:XMWrap>\
     <ltx:XMTok name='xleftharpoonup' role='ARROW' stretchy='true'>\u{21BC}</ltx:XMTok>\
     </ltx:XMApp>\
     )");

  DefConstructor!("\\xrightleftharpoons OptionalInScriptStyle InScriptStyle",
    "?#1(\
     <ltx:XMApp role='METARELOP'>\
     <ltx:XMWrap role='UNDERACCENT'>#1</ltx:XMWrap>\
     <ltx:XMApp>\
     <ltx:XMWrap role='OVERACCENT'>#2</ltx:XMWrap>\
     <ltx:XMTok name='xrightleftharpoons' role='METARELOP' stretchy='true'>\u{21CC}</ltx:XMTok>\
     </ltx:XMApp>\
     </ltx:XMApp>\
     )(\
     <ltx:XMApp role='METARELOP'>\
     <ltx:XMWrap role='OVERACCENT'>#2</ltx:XMWrap>\
     <ltx:XMTok name='xrightleftharpoons' role='METARELOP' stretchy='true'>\u{21CC}</ltx:XMTok>\
     </ltx:XMApp>\
     )");

  DefConstructor!("\\xleftrightharpoons OptionalInScriptStyle InScriptStyle",
    "?#1(\
     <ltx:XMApp role='METARELOP'>\
     <ltx:XMWrap role='UNDERACCENT'>#1</ltx:XMWrap>\
     <ltx:XMApp>\
     <ltx:XMWrap role='OVERACCENT'>#2</ltx:XMWrap>\
     <ltx:XMTok name='xleftrightharpoons' role='METARELOP' stretchy='true'>\u{21CB}</ltx:XMTok>\
     </ltx:XMApp>\
     </ltx:XMApp>\
     )(\
     <ltx:XMApp role='METARELOP'>\
     <ltx:XMWrap role='OVERACCENT'>#2</ltx:XMWrap>\
     <ltx:XMTok name='xleftrightharpoons' role='METARELOP' stretchy='true'>\u{21CB}</ltx:XMTok>\
     </ltx:XMApp>\
     )");

  // \overbracket / \underbracket — ignore optional rule thickness and bracket height args
  DefMacro!("\\overbracket[][][]{}",  "\\lx@mt@overbracket{#4}");
  DefMacro!("\\underbracket[][][]{}", "\\lx@mt@underbracket{#4}");
  DefMath!("\\lx@mt@overbracket{}", "\u{FE47}",
    operator_role => "OVERACCENT", scriptpos => "mid",
    alias => "\\overbracket");
  DefMath!("\\lx@mt@underbracket{}", "\u{FE48}",
    operator_role => "UNDERACCENT", scriptpos => "mid",
    alias => "\\underbracket");
  Let!("\\LaTeXunderbrace", "\\underbrace");
  Let!("\\LaTeXoverbrace", "\\overbrace");

  //======================================================================
  // 3.4 — Starred matrix environments
  //======================================================================

  DefMacro!("\\csname matrix*\\endcsname[]",
    "\\lx@ams@matrix{name=matrix,datameaning=matrix,alignment=#1}");
  DefMacro!("\\csname endmatrix*\\endcsname", "\\lx@end@ams@matrix");

  DefMacro!("\\csname pmatrix*\\endcsname[]",
    "\\lx@ams@matrix{name=pmatrix,datameaning=matrix,alignment=#1,left=\\lx@left(,right=\\lx@right)}");
  DefMacro!("\\csname endpmatrix*\\endcsname", "\\lx@end@ams@matrix");

  DefMacro!("\\csname bmatrix*\\endcsname[]",
    "\\lx@ams@matrix{name=bmatrix,datameaning=matrix,alignment=#1,left=\\lx@left[,right=\\lx@right]}");
  DefMacro!("\\csname endbmatrix*\\endcsname", "\\lx@end@ams@matrix");

  DefMacro!("\\csname Bmatrix*\\endcsname[]",
    "\\lx@ams@matrix{name=Bmatrix,datameaning=matrix,alignment=#1,left=\\lx@left\\{,right=\\lx@right\\}}");
  DefMacro!("\\csname endBmatrix*\\endcsname", "\\lx@end@ams@matrix");

  DefMacro!("\\csname vmatrix*\\endcsname[]",
    "\\lx@ams@matrix{name=vmatrix,delimitermeaning=determinant,datameaning=matrix,alignment=#1,left=\\lx@left|,right=\\lx@right|}");
  DefMacro!("\\csname endvmatrix*\\endcsname", "\\lx@end@ams@matrix");

  DefMacro!("\\csname Vmatrix*\\endcsname[]",
    "\\lx@ams@matrix{name=Vmatrix,delimitermeaning=norm,datameaning=matrix,alignment=#1,left=\\lx@left\\|,right=\\lx@right\\|}");
  DefMacro!("\\csname endVmatrix*\\endcsname", "\\lx@end@ams@matrix");

  // Perl L502-538: Starred small matrices — \@smallmatrix@star@tmp reads alignment
  // from optional arg or falls back to mathtoolsset smallmatrix-align option.
  // We use \ifx/#1/ pattern to only pass alignment when explicitly given.
  DefMacro!("\\csname smallmatrix*\\endcsname[]",
    "\\lx@ams@matrix{name=matrix,datameaning=matrix,style=\\scriptsize,\\ifx/#1/\\else alignment=#1,\\fi}");
  DefMacro!("\\csname endsmallmatrix*\\endcsname", "\\lx@end@ams@matrix");

  DefMacro!("\\csname psmallmatrix*\\endcsname[]",
    "\\lx@ams@matrix{name=pmatrix,datameaning=matrix,left=\\lx@left(,right=\\lx@right),style=\\scriptsize,\\ifx/#1/\\else alignment=#1,\\fi}");
  DefMacro!("\\csname endpsmallmatrix*\\endcsname", "\\lx@end@ams@matrix");

  DefMacro!("\\csname bsmallmatrix*\\endcsname[]",
    "\\lx@ams@matrix{name=bmatrix,datameaning=matrix,left=\\lx@left[,right=\\lx@right],style=\\scriptsize,\\ifx/#1/\\else alignment=#1,\\fi}");
  DefMacro!("\\csname endbsmallmatrix*\\endcsname", "\\lx@end@ams@matrix");

  DefMacro!("\\csname Bsmallmatrix*\\endcsname[]",
    "\\lx@ams@matrix{name=Bmatrix,datameaning=matrix,left=\\lx@left\\{,right=\\lx@right\\},style=\\scriptsize,\\ifx/#1/\\else alignment=#1,\\fi}");
  DefMacro!("\\csname endBsmallmatrix*\\endcsname", "\\lx@end@ams@matrix");

  DefMacro!("\\csname vsmallmatrix*\\endcsname[]",
    "\\lx@ams@matrix{name=vmatrix,delimitermeaning=determinant,datameaning=matrix,left=\\lx@left|,right=\\lx@right|,style=\\scriptsize,\\ifx/#1/\\else alignment=#1,\\fi}");
  DefMacro!("\\csname endvsmallmatrix*\\endcsname", "\\lx@end@ams@matrix");

  DefMacro!("\\csname Vsmallmatrix*\\endcsname[]",
    "\\lx@ams@matrix{name=Vmatrix,delimitermeaning=norm,datameaning=matrix,left=\\lx@left\\|,right=\\lx@right\\|,style=\\scriptsize,\\ifx/#1/\\else alignment=#1,\\fi}");
  DefMacro!("\\csname endVsmallmatrix*\\endcsname", "\\lx@end@ams@matrix");

  // Non-starred small matrices
  DefMacro!("\\psmallmatrix",
    "\\lx@ams@matrix{name=pmatrix,datameaning=matrix,left=\\lx@left(,right=\\lx@right),style=\\scriptsize}");
  DefMacro!("\\endpsmallmatrix", "\\lx@end@ams@matrix");

  DefMacro!("\\bsmallmatrix",
    "\\lx@ams@matrix{name=bmatrix,datameaning=matrix,left=\\lx@left[,right=\\lx@right],style=\\scriptsize}");
  DefMacro!("\\endbsmallmatrix", "\\lx@end@ams@matrix");

  DefMacro!("\\Bsmallmatrix",
    "\\lx@ams@matrix{name=Bmatrix,datameaning=matrix,left=\\lx@left\\{,right=\\lx@right\\},style=\\scriptsize}");
  DefMacro!("\\endBsmallmatrix", "\\lx@end@ams@matrix");

  DefMacro!("\\vsmallmatrix",
    "\\lx@ams@matrix{name=vmatrix,delimitermeaning=determinant,datameaning=matrix,left=\\lx@left|,right=\\lx@right|,style=\\scriptsize}");
  DefMacro!("\\endvsmallmatrix", "\\lx@end@ams@matrix");

  DefMacro!("\\Vsmallmatrix",
    "\\lx@ams@matrix{name=Vmatrix,delimitermeaning=norm,datameaning=matrix,left=\\lx@left\\|,right=\\lx@right\\|,style=\\scriptsize}");
  DefMacro!("\\endVsmallmatrix", "\\lx@end@ams@matrix");

  //======================================================================
  // {multlined} environment
  //======================================================================
  // Perl: DefConstructor('\@@multlined DigestedBody', "#1", ...)
  // DigestedBody absorbs the entire content until the matching end command.
  // Perl: afterDigest sets alignment rule {default=>'center', 0=>'left', -1=>'right'}
  // afterConstruct calls rearrangeAMSMultirow
  DefConstructor!("\\@@multlined DigestedBody",
    "#1",
    before_digest => { bgroup(); },
    after_digest => sub[whatsit] {
      whatsit.set_property("MULTIROW_ALIGNMENT_RULE_DEFAULT", Stored::from("center"));
      whatsit.set_property("MULTIROW_ALIGNMENT_RULE_0", Stored::from("left"));
      whatsit.set_property("MULTIROW_ALIGNMENT_RULE_LAST", Stored::from("right"));
    },
    after_construct => sub[document, whatsit] {
      if let Some(last) = document.get_node().get_last_child() {
        let align_rule = amsmath_sty::get_multirow_alignment_rule(whatsit);
        amsmath_sty::rearrange_ams_multirow(document, last, &align_rule)?;
      }
    },
    reversion => "\\begin{multlined}#1\\end{multlined}"
  );
  // Perl: \multlined[][] → \@multlined@tmp{name=multlined,...}\@@multlined\lx@begin@alignment
  // The \ifx/#1/ pattern: if #1 is empty, /==/ is true and vattach is omitted.
  DefMacro!("\\multlined[][]",
    "\\@ams@multirow@bindings{name=multlined,\\ifx/#1/\\else vattach=#1,\\fi\\ifx/#2/\\else width=#2,\\fi}\\@@multlined\\lx@begin@alignment");
  DefMacro!("\\endmultlined", "\\lx@end@alignment\\@end@multlined");
  DefPrimitive!("\\@end@multlined", { egroup()?; });

  // Perl L590-599: \@MT@shove stores alignment direction for current row.
  // Perl: LookupValue('Alignment')->currentRowNumber → sets MULTIROW_ALIGNMENT_RULE hash.
  // Currently passes content through; the alignment shifting is cosmetic
  // and requires deep integration with the alignment row tracking.
  DefMacro!("\\shoveright[]{}", "#2");
  DefMacro!("\\shoveleft[]{}", "#2");

  //======================================================================
  // Cases variants
  //======================================================================

  DefMacro!("\\dcases",
    "\\lx@ams@cases{name=dcases,meaning=cases,left=\\lx@left\\{,style=\\displaystyle,conditionmode=math}");
  DefMacro!("\\enddcases", "\\lx@end@ams@cases");

  DefMacro!("\\csname dcases*\\endcsname",
    "\\lx@ams@cases{name=dcases*,meaning=cases,left=\\lx@left\\{,style=\\displaystyle,conditionmode=text}");
  DefMacro!("\\csname enddcases*\\endcsname", "\\lx@end@ams@cases");

  DefMacro!("\\rcases",
    "\\lx@ams@cases{name=rcases,meaning=cases,right=\\lx@right\\},style=\\textstyle,conditionmode=math}");
  DefMacro!("\\endrcases", "\\lx@end@ams@cases");

  DefMacro!("\\csname rcases*\\endcsname",
    "\\lx@ams@cases{name=rcases*,meaning=cases,right=\\lx@right\\},style=\\textstyle,conditionmode=text}");
  DefMacro!("\\csname endrcases*\\endcsname", "\\lx@end@ams@cases");

  DefMacro!("\\drcases",
    "\\lx@ams@cases{name=drcases,meaning=cases,right=\\lx@right\\},style=\\displaystyle,conditionmode=math}");
  DefMacro!("\\enddrcases", "\\lx@end@ams@cases");

  DefMacro!("\\csname drcases*\\endcsname",
    "\\lx@ams@cases{name=drcases*,meaning=cases,right=\\lx@right\\},style=\\displaystyle,conditionmode=text}");
  DefMacro!("\\csname enddrcases*\\endcsname", "\\lx@end@ams@cases");

  DefMacro!("\\csname cases*\\endcsname",
    "\\lx@ams@cases{name=cases*,meaning=cases,left=\\lx@left\\{,style=\\textstyle,conditionmode=text}");
  DefMacro!("\\csname endcases*\\endcsname", "\\lx@end@ams@cases");

  // Perl mathtools.sty.ltxml L638 defines \MoveEqLeft as bare T_ALIGN —
  // i.e. the tokenizer-level `&` alignment tab. The string `"&"` below
  // tokenizes to the same alignment tab under the math catcode regime
  // where this macro is invoked. Perl's corresponding TODO note
  // (shift-the-equation layout) is inherited.
  DefMacro!("\\MoveEqLeft[]", "&");

  // Perl mathtools.sty.ltxml L641 also ships `\Aboxed` as a passthrough
  // `#1` and flags a proper-implementation TODO. The Rust binding matches
  // — any improvement belongs upstream (layout/frame rendering).
  DefMacro!("\\Aboxed{}", "#1");

  // Perl mathtools.sty.ltxml L644-645 defines both `\ArrowBetweenLines` and
  // its star form as empty tokens. The Rust `None` gives the same empty
  // expansion. Upstream Perl also flags the "make it do something" TODO.
  DefMacro!("\\ArrowBetweenLines[]", None);
  DefMacro!("\\csname ArrowBetweenLines*\\endcsname[]", None);
  DefMacro!("\\vdotswithin{}",
    "\\mathmakebox[\\widthof{\\ensuremath{{}#1{}}}][c]{\\vdots}");
  DefMacro!("\\shortvdotswithin{}",
    "\\MTFlushSpaceAbove & \\vdotswithin{#1} \\MTFlushSpaceBelow");
  DefMacro!("\\csname shortvdotswithin*\\endcsname{}",
    "\\MTFlushSpaceAbove \\vdotswithin{#1} & \\MTFlushSpaceBelow");
  DefMacro!("\\MTFlushSpaceAbove", None);
  DefMacro!("\\MTFlushSpaceBelow", "\\\\");

  //======================================================================
  // 3.5 — Short intertext
  //======================================================================
  Let!("\\shortintertext", "\\@ams@intertext");

  //======================================================================
  // 3.6 — Paired delimiters
  //======================================================================

  // \DeclarePairedDelimiter\cmd{left}{right}
  // Perl: creates \cmd with star/optional-size/plain variants via wrapper macros:
  //   \cmd*{x}      → \left<ldel> x \right<rdel>
  //   \cmd[\Big]{x}  → \Big<ldel> x \Big<rdel>
  //   \cmd{x}       → <ldel> x <rdel>
  DefPrimitive!("\\DeclarePairedDelimiter DefToken {}{}", sub[(cs, ldel, rdel)] {
    use latexml_core::definition::ExpansionBody;
    let cmd = cs.to_string();
    let cmd_name = cmd.trim_start_matches('\\');
    let ldel_toks: Vec<Token> = ldel.unlist();
    let rdel_toks: Vec<Token> = rdel.unlist();
    // Wrapper macros: #1#2#3 (identity by default, can be overridden)
    let star_wrapper_cs = s!("\\MT@delim@{}@star@wrapper", cmd_name);
    def_macro(T_CS!(&star_wrapper_cs),
      parse_parameters("{}{}{}", &T_CS!(&star_wrapper_cs), true)?,
      Tokenize!("#1#2#3"), None)?;
    let nostar_wrapper_cs = s!("\\MT@delim@{}@nostar@wrapper", cmd_name);
    def_macro(T_CS!(&nostar_wrapper_cs),
      parse_parameters("{}{}{}", &T_CS!(&nostar_wrapper_cs), true)?,
      Tokenize!("#1#2#3"), None)?;
    // Star variant: @star@wrapper{\left ldel}{#1}{\right rdel}
    let star_cs_name = s!("\\MT@delim@{}@star", cmd_name);
    let mut star_body_toks: Vec<Token> = vec![];
    star_body_toks.push(T_CS!(&star_wrapper_cs));
    star_body_toks.push(T_BEGIN!());
    star_body_toks.push(T_CS!("\\left"));
    star_body_toks.extend(ldel_toks.iter().cloned());
    star_body_toks.push(T_END!());
    star_body_toks.push(T_BEGIN!());
    star_body_toks.push(T_PARAM!()); star_body_toks.push(T_OTHER!("1"));
    star_body_toks.push(T_END!());
    star_body_toks.push(T_BEGIN!());
    star_body_toks.push(T_CS!("\\right"));
    star_body_toks.extend(rdel_toks.iter().cloned());
    star_body_toks.push(T_END!());
    def_macro(T_CS!(&star_cs_name),
      parse_parameters("{}", &T_CS!(&star_cs_name), true)?,
      ExpansionBody::Tokens(Tokens::new(star_body_toks)), None)?;
    // Nostar variant: @nostar@wrapper{#1 ldel}{#2}{#1 rdel}
    let nostar_cs_name = s!("\\MT@delim@{}@nostar", cmd_name);
    let mut nostar_body_toks: Vec<Token> = vec![T_CS!(&nostar_wrapper_cs)];
    nostar_body_toks.push(T_BEGIN!());
    nostar_body_toks.push(T_PARAM!()); nostar_body_toks.push(T_OTHER!("1"));
    nostar_body_toks.extend(ldel_toks.iter().cloned());
    nostar_body_toks.push(T_END!());
    nostar_body_toks.push(T_BEGIN!());
    nostar_body_toks.push(T_PARAM!()); nostar_body_toks.push(T_OTHER!("2"));
    nostar_body_toks.push(T_END!());
    nostar_body_toks.push(T_BEGIN!());
    nostar_body_toks.push(T_PARAM!()); nostar_body_toks.push(T_OTHER!("1"));
    nostar_body_toks.extend(rdel_toks.iter().cloned());
    nostar_body_toks.push(T_END!());
    def_macro(T_CS!(&nostar_cs_name),
      parse_parameters("[]{}", &T_CS!(&nostar_cs_name), true)?,
      ExpansionBody::Tokens(Tokens::new(nostar_body_toks)), None)?;
    // Main command: \@ifstar dispatches to star or nostar
    let dispatch_toks = Tokens::new(vec![
      T_CS!("\\@ifstar"),
      T_BEGIN!(), T_CS!(&star_cs_name), T_END!(),
      T_BEGIN!(), T_CS!(&nostar_cs_name), T_END!(),
    ]);
    def_macro(cs, None, dispatch_toks, None)?;
  });

  // \DeclarePairedDelimiterX\cmd[nargs]{left}{right}{body}
  // Perl: creates \cmd@inner with n args expanding to body + \cmd@after,
  // then \cmd with OptionalMatch:* [] dispatching to construct:
  //   star:  \left ldel \def\delimsize{\middle} \def\cmd@after{\right rdel} \cmd@inner
  //   [opt]: opt ldel \def\delimsize{opt} \def\cmd@after{opt rdel} \cmd@inner
  //   plain: ldel \def\delimsize{} \def\cmd@after{rdel} \cmd@inner
  DefPrimitive!("\\DeclarePairedDelimiterX DefToken [Number] {} {} {}", sub[(cs, nargs, ldel, rdel, body)] {
    use latexml_core::definition::ExpansionBody;
    let cmd = cs.to_string();
    let n = nargs.value_of() as usize;
    let ldel_toks: Vec<Token> = ldel.unlist();
    let rdel_toks: Vec<Token> = rdel.unlist();
    let body_toks: Vec<Token> = body.unlist();
    // Create \cmd@inner: n args, body = user_body + \cmd@after
    let inner_cs_name = s!("{}@inner", cmd);
    let after_cs_name = s!("{}@after", cmd);
    let param_spec: String = (0..n.max(1)).map(|_| "{}").collect();
    let mut inner_body_toks = body_toks;
    inner_body_toks.push(T_CS!(&after_cs_name));
    def_macro(T_CS!(&inner_cs_name),
      parse_parameters(&param_spec, &T_CS!(&inner_cs_name), true)?,
      ExpansionBody::Tokens(Tokens::new(inner_body_toks)), None)?;
    // Create main \cmd with OptionalMatch:* [] expansion closure.
    // Move inner_cs_name / after_cs_name / ldel_toks / rdel_toks
    // directly into the closure capture — none are used after this
    // point, so four setup-time clones are avoided.
    def_macro(cs, parse_parameters("OptionalMatch:* []", &cs, true)?,
      Some(ExpansionBody::Closure(Rc::new(move |args| {
        let star = &args[0]; // OptionalMatch:*
        let opt = &args[1];  // []
        let is_star = !star.is_empty();
        let has_opt = !opt.is_empty();
        let mut toks: Vec<Token> = vec![];
        // Prefix: \left (star), opt tokens (sized), or nothing (plain)
        if is_star {
          toks.push(T_CS!("\\left"));
        } else if has_opt {
          toks.extend_from_slice(&opt.unlist_cow());
        }
        // Left delimiter
        toks.extend(ldel_toks.iter().cloned());
        // \def\delimsize{...}
        toks.push(T_CS!("\\def"));
        toks.push(T_CS!("\\delimsize"));
        toks.push(T_BEGIN!());
        if has_opt {
          toks.extend_from_slice(&opt.unlist_cow());
        } else if is_star {
          toks.push(T_CS!("\\middle"));
        }
        toks.push(T_END!());
        // \def\cmd@after{... rdel}
        toks.push(T_CS!("\\def"));
        toks.push(T_CS!(&after_cs_name));
        toks.push(T_BEGIN!());
        if is_star {
          toks.push(T_CS!("\\right"));
        } else if has_opt {
          toks.extend_from_slice(&opt.unlist_cow());
        }
        toks.extend(rdel_toks.iter().cloned());
        toks.push(T_END!());
        // \cmd@inner
        toks.push(T_CS!(&inner_cs_name));
        Ok(Tokens::new(toks))
      }))), None)?;
  });

  // \DeclarePairedDelimiterXPP — most general form (with pre/post code)
  // Perl: same as X but with precode before delimiters and postcode after.
  DefPrimitive!("\\DeclarePairedDelimiterXPP DefToken [Number] {} {} {} {} {}", sub[(cs, nargs, pre, ldel, rdel, post, body)] {
    use latexml_core::definition::ExpansionBody;
    let cmd = cs.to_string();
    let n = nargs.value_of() as usize;
    let ldel_toks: Vec<Token> = ldel.unlist();
    let rdel_toks: Vec<Token> = rdel.unlist();
    let body_toks: Vec<Token> = body.unlist();
    let pre_toks: Vec<Token> = pre.unlist();
    let post_toks: Vec<Token> = post.unlist();
    // Create \cmd@inner: n args, body = user_body + \cmd@after + postcode
    let inner_cs_name = s!("{}@inner", cmd);
    let after_cs_name = s!("{}@after", cmd);
    let param_spec: String = (0..n.max(1)).map(|_| "{}").collect();
    let mut inner_body_toks = body_toks;
    inner_body_toks.push(T_CS!(&after_cs_name));
    inner_body_toks.extend(post_toks.iter().cloned());
    def_macro(T_CS!(&inner_cs_name),
      parse_parameters(&param_spec, &T_CS!(&inner_cs_name), true)?,
      ExpansionBody::Tokens(Tokens::new(inner_body_toks)), None)?;
    // Create main \cmd with OptionalMatch:* [] expansion closure.
    // Move the captured Vecs/Strings directly — no setup-time clones.
    def_macro(cs, parse_parameters("OptionalMatch:* []", &cs, true)?,
      Some(ExpansionBody::Closure(Rc::new(move |args| {
        let star = &args[0];
        let opt = &args[1];
        let is_star = !star.is_empty();
        let has_opt = !opt.is_empty();
        let mut toks: Vec<Token> = vec![];
        // Precode
        toks.extend(pre_toks.iter().cloned());
        // Prefix
        if is_star {
          toks.push(T_CS!("\\left"));
        } else if has_opt {
          toks.extend_from_slice(&opt.unlist_cow());
        }
        // Left delimiter
        toks.extend(ldel_toks.iter().cloned());
        // \def\delimsize{...}
        toks.push(T_CS!("\\def"));
        toks.push(T_CS!("\\delimsize"));
        toks.push(T_BEGIN!());
        if has_opt {
          toks.extend_from_slice(&opt.unlist_cow());
        } else if is_star {
          toks.push(T_CS!("\\middle"));
        }
        toks.push(T_END!());
        // \def\cmd@after{... rdel}
        toks.push(T_CS!("\\def"));
        toks.push(T_CS!(&after_cs_name));
        toks.push(T_BEGIN!());
        if is_star {
          toks.push(T_CS!("\\right"));
        } else if has_opt {
          toks.extend_from_slice(&opt.unlist_cow());
        }
        toks.extend(rdel_toks.iter().cloned());
        toks.push(T_END!());
        // \cmd@inner
        toks.push(T_CS!(&inner_cs_name));
        Ok(Tokens::new(toks))
      }))), None)?;
  });

  // \reDeclarePairedDelimiterInnerWrapper\cmd{star|nostar}{body}
  // Perl: redefines the @star@wrapper or @nostar@wrapper for \DeclarePairedDelimiter
  DefPrimitive!("\\reDeclarePairedDelimiterInnerWrapper DefToken {}{}", sub[(cs, nstar, body)] {
    use latexml_core::definition::ExpansionBody;
    let cmd = cs.to_string();
    let cmd_name = cmd.trim_start_matches('\\');
    let variant = nstar.to_string();
    let wrapper_cs = s!("\\MT@delim@{}@{}@wrapper", cmd_name, variant);
    let body_toks: Vec<Token> = body.unlist();
    def_macro(T_CS!(&wrapper_cs),
      parse_parameters("{}{}{}", &T_CS!(&wrapper_cs), true)?,
      ExpansionBody::Tokens(Tokens::new(body_toks)), None)?;
  });

  //======================================================================
  // 3.7 — Math-mode symbol definitions
  //======================================================================

  DefMath!("\\lparen", "(", role => "OPEN",  stretchy => false);
  DefMath!("\\rparen", ")", role => "CLOSE", stretchy => false);

  DefMath!("\\vcentcolon", None, ":", role => "RELOP");
  DefMath!("\\ordinarycolon", None, ":", role => "RELOP");

  DefMath!("\\dblcolon", "::", role => "RELOP");

  DefMath!("\\coloneqq",    "\u{2254}",   role => "RELOP");
  DefMath!("\\Coloneqq",    "\u{2A74}",   role => "RELOP");
  DefMath!("\\coloneq",     "\u{2254}",   role => "RELOP");
  DefMath!("\\Coloneq",     "\u{2A74}",   role => "RELOP");
  DefMath!("\\eqqcolon",    "\u{2255}",   role => "RELOP");
  DefMath!("\\Eqqcolon",    "=::",        role => "RELOP");
  DefMath!("\\eqcolon",     "\u{2255}",   role => "RELOP");
  DefMath!("\\Eqcolon",     "=::",        role => "RELOP");
  DefMath!("\\colonapprox", ":\u{2248}",  role => "RELOP");
  DefMath!("\\Colonapprox", "::\u{2248}", role => "RELOP");
  DefMath!("\\approxcolon", "\u{2248}:",  role => "RELOP");
  DefMath!("\\Approxcolon", "\u{2248}::", role => "RELOP");
  DefMath!("\\colonsim",    ":\u{223C}",  role => "RELOP");
  DefMath!("\\Colonsim",    "::\u{223C}", role => "RELOP");
  DefMath!("\\simcolon",    "\u{223C}:",  role => "RELOP");
  DefMath!("\\Simcolon",    "\u{223C}::", role => "RELOP");
  DefMath!("\\colondash",   ":-",         role => "RELOP");
  DefMath!("\\Colondash",   "::-",        role => "RELOP");
  DefMath!("\\dashcolon",   "-:",         role => "RELOP");
  DefMath!("\\Dashcolon",   "-::",        role => "RELOP");

  // Perl: UTF(0x2909) — RIGHTWARDS DOUBLE ARROW FROM BAR (approximation)
  DefMath!("\\nuparrow", None, "\u{2909}", role => "ARROW");
  // Perl: UTF(0x2908) — DOWNWARDS DOUBLE ARROW FROM BAR (approximation)
  DefMath!("\\ndownarrow", None, "\u{2908}", role => "ARROW");
  // Perl: UTF(0xD7) = × MULTIPLICATION SIGN
  // Perl: font => { size => 'Big' } where rationalizeFontSize('Big') = 1.6 * DEFSIZE(10) = 16.0pt
  DefMath!("\\bigtimes", None, "\u{00D7}", role => "MULOP", meaning => "times",
    font => { size => 16.0 },
    dynamic_scriptpos => true);

  //======================================================================
  // 4 — Extended features
  //======================================================================

  // 4.2 — Prescripts
  DefMacro!("\\prescript{}{}{}",
    "\\@ams@prescript{#1}{#2}{#3}{\
     {}^{\\@mt@getmtoption{prescript-sup-format}{#1}}\
     _{\\@mt@getmtoption{prescript-sub-format}{#2}}\
     {\\@mt@getmtoption{prescript-arg-format}{#3}}\
     }");
  // wrapper to get reversion
  DefConstructor!("\\@ams@prescript{}{}{}{}", "#4",
    reversion => "\\prescript{#1}{#2}{#3}");

  // 4.4 — Spread lines
  DefMacro!("\\csname spreadlines\\endcsname{}", "\\begingroup\\jot=#1\\relax");
  DefMacro!("\\csname endspreadlines\\endcsname", "\\endgroup");

  // 4.5 — lgathered / rgathered
  // TODO: @@lgathered/@@rgathered have complex afterDigest/afterConstruct — simplified
  DefMacro!("\\lgathered[]",
    "\\@ams@multirow@bindings{name=lgathered,vattach=#1}\\@@lgathered\\lx@begin@alignment");
  DefMacro!("\\endlgathered", "\\lx@end@alignment\\@end@gathered");

  DefMacro!("\\rgathered[]",
    "\\@ams@multirow@bindings{name=rgathered,vattach=#1}\\@@rgathered\\lx@begin@alignment");
  DefMacro!("\\endrgathered", "\\lx@end@alignment\\@end@gathered");

  // Perl: DefConstructor('\@@lgathered DigestedBody', ...)
  // Perl: afterDigest sets MULTIROW_ALIGNMENT_RULE { default => 'left' }
  // afterConstruct calls rearrangeAMSMultirow
  DefConstructor!("\\@@lgathered DigestedBody", "#1",
    before_digest => { bgroup(); },
    after_digest => sub[whatsit] {
      whatsit.set_property("MULTIROW_ALIGNMENT_RULE_DEFAULT", Stored::from("left"));
    },
    reversion => "\\begin{lgathered}#1\\end{lgathered}",
    after_construct => sub[document, whatsit] {
      if let Some(last) = document.get_node().get_last_child() {
        let align_rule = amsmath_sty::get_multirow_alignment_rule(whatsit);
        amsmath_sty::rearrange_ams_multirow(document, last, &align_rule)?;
      }
    });
  DefConstructor!("\\@@rgathered DigestedBody", "#1",
    before_digest => { bgroup(); },
    after_digest => sub[whatsit] {
      whatsit.set_property("MULTIROW_ALIGNMENT_RULE_DEFAULT", Stored::from("right"));
    },
    reversion => "\\begin{rgathered}#1\\end{rgathered}",
    after_construct => sub[document, whatsit] {
      if let Some(last) = document.get_node().get_last_child() {
        let align_rule = amsmath_sty::get_multirow_alignment_rule(whatsit);
        amsmath_sty::rearrange_ams_multirow(document, last, &align_rule)?;
      }
    });

  // \newgathered{name}{pre_line}{post_line}{after}
  // Creates \name and \endname environments for gathered-like displays.
  // Perl: DefMacro sub{} body that dynamically DefMacroI-installs runtime
  // macros. Rust DefPrimitive does the installs at stomach time.
  // WISDOM #44: NOT universally equivalent — safe here because
  // `\newgathered` is a user-facing preamble declaration, not something
  // that flows through `\edef`.
  // WISDOM #44 verified 2026-04-23: zero `\edef`/`\ifx`/`\expandafter`
  // uses of `\newgathered` across LaTeXML/lib + ar5iv-bindings.
  DefPrimitive!("\\newgathered{}{}{}{}", sub[(name, _pre, _post, _after)] {
    let env_name = name.to_string();
    // Create \name macro → begins gathered alignment
    // Build tokens manually to preserve @ in CS names
    let mut begin_toks = vec![
      T_CS!("\\@ams@multirow@bindings"),
      T_BEGIN!(),
    ];
    begin_toks.extend(ExplodeText!(&format!("name={env_name}")));
    begin_toks.extend(vec![
      T_END!(),
      T_CS!("\\@@newgathered@dummy"),
      T_CS!("\\lx@begin@alignment"),
    ]);
    let begin_cs = T_CS!(&format!("\\{env_name}"));
    def_macro(begin_cs, None, Tokens::new(begin_toks), None)?;
    // Create \endname macro → ends alignment
    let end_toks = Tokens::new(vec![
      T_CS!("\\lx@end@alignment"),
      T_CS!("\\@end@gathered"),
    ]);
    let end_cs = T_CS!(&format!("\\end{env_name}"));
    def_macro(end_cs, None, end_toks, None)?;
  });
  Let!("\\renewgathered", "\\newgathered");

  // \@@newgathered@dummy — gathered-like constructor with DigestedBody
  // Perl: afterConstruct extracts array cells and wraps in XMDual.
  // Simplified: just output the body as-is, like \@@gathered.
  DefConstructor!("\\@@newgathered@dummy DigestedBody",
    "#1",
    before_digest => { bgroup(); },
    reversion => "\\begin{gathered}#1\\end{gathered}",
    after_construct => sub[document, whatsit] {
      if let Some(last) = document.get_node().get_last_child() {
        let align_rule = amsmath_sty::get_multirow_alignment_rule(whatsit);
        amsmath_sty::rearrange_ams_multirow(document, last, &align_rule)?;
      }
    }
  );
  DefPrimitive!("\\@end@gathered", { egroup()?; });

  // 4.6 — Split fractions
  DefMacro!("\\splitfrac{}{}",
    "\\@ams@multirow@bindings{name=splitfrac}\\@@multlined\\lx@begin@alignment #1 \\\\\\\\ #2 \\lx@end@alignment\\@end@multline");
  DefMacro!("\\splitdfrac{}{}",
    "\\displaystyle\\@ams@multirow@bindings{name=splitdfrac}\\@@multlined\\lx@begin@alignment #1 \\\\\\\\ #2 \\lx@end@alignment\\@end@multline");
});
