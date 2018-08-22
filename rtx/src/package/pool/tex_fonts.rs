use package::*;

pub fn load_definitions(state: &mut State) -> Result<()> {
  SetupBindingMacros!(state);

  // Used for SemiVerbatim text
  DeclareFontMap!(
    "ASCII",
    mixvec![
      None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
      None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
      None, None, ' ', '!', '"', '#', '$', '%', '&', '\'', '(', ')', '*', '+', ',', '-', '.', '/',
      '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', ':', ';', '<', '=', '>', '?', '@', 'A',
      'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S',
      'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '[', '\\', ']', '^', '_', '`', 'a', 'b', 'c', 'd', 'e',
      'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w',
      'x', 'y', 'z', '{', '|', '}', '~', None
    ]
  ); 

  // Note that several entries are used for accents, and in practice will actually
  // be used in something like an m:mover; thus they needn't (shouldn't?) be "small"
  // There are also some questions about which choices are best
  // grave & acute accents (entry 0x12 & 0x13) (often typed using 0x60 & 0x27)
  //   are probably best using U+60(grave accent) & U+B4(acute accent)
  //   but could be U+2035 (reversed prime) & U+2032 (prime).  (particularly for math?)
  //   [we do use these for \prime, however!]
  //   or U+02CB (modifier letter grave accent) & U+02CA (modifier letter acute accent)
  // Similarly, hat & tilde (entries 0x5E & 0x7E)
  //   typed using ^ 0x5E circumflex accent) & ~ 0x7E  tilde
  //   are probably best just sticking with U+5E & U+7E
  //   but could be U+02C6 (modifier letter circumflex accent) U+02DC (small tilde)
  // [Note that generally we're using codepoints characterized as "modifier letter"
  // only when no other spacing point is available.]
  DeclareFontMap!(
    "OT1",
    mixvec![
      '\u{0393}', '\u{0394}', '\u{0398}', '\u{039B}', '\u{039E}', '\u{03A0}', '\u{03A3}',
      '\u{03A5}', '\u{03A6}', '\u{03A8}', '\u{03A9}', '\u{FB00}', '\u{FB01}', '\u{FB02}',
      '\u{FB03}', '\u{FB04}', '\u{0131}', '\u{0237}', '\u{0060}', '\u{00B4}', '\u{02C7}',
      '\u{02D8}', '\u{00AF}', '\u{02DA}', '\u{00B8}', '\u{00DF}', '\u{00E6}', '\u{0153}',
      '\u{00F8}', '\u{00C6}', '\u{0152}', '\u{00D8}', '\u{0335}', '!', '\u{201D}', '#', '$', '%',
      '&', '\u{2019}', '(', ')', '*', '+', ',', '-', '.', '/', '0', '1', '2', '3', '4', '5', '6',
      '7', '8', '9', ':', ';', '\u{00A1}', '=', '\u{00BF}', '?', '@', 'A', 'B', 'C', 'D', 'E', 'F',
      'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X',
      'Y', 'Z', '[', '\u{201C}', ']', '^', '\u{02D9}', '\u{2018}', 'a', 'b', 'c', 'd', 'e', 'f',
      'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x',
      'y', 'z', '\u{2013}', '\u{2014}', '\u{02DD}', '\u{007E}', '\u{00A8}'
    ]
  ); // TODO: do we really need '\u{00A0}'\x{0335} as a single entry?

  DeclareFontMap!(
    "OT1",
    mixvec![
      '\u{0393}', '\u{0394}', '\u{0398}', '\u{039B}', '\u{039E}', '\u{03A0}', '\u{03A3}',
      '\u{03A5}', '\u{03A6}', '\u{03A8}', '\u{03A9}', '\u{2191}', '\u{2193}', '\'', '\u{00A1}',
      '\u{00BF}', '\u{0131}', '\u{0237}', '\u{0060}', '\u{00B4}', '\u{02C7}', '\u{02D8}',
      '\u{00AF}', '\u{02DA}', '\u{00B8}', '\u{00DF}', '\u{00E6}', '\u{0153}', '\u{00F8}',
      '\u{00C6}', '\u{152}', '\u{00D8}', '\u{2423}', '!', '\'', '#', '$', '%', '&', '\u{2019}',
      '(', ')', '*', '+', ',', '-', '.', '/', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
      ':', ';', '<', '=', '>', '?', '@', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K',
      'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '[', '\\', ']',
      '^', '_', '\u{2018}', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n',
      'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '{', '|', '}', '~', '\u{00A8}'
    ],
    "typewriter"
  );

  DeclareFontMap!(
    "OML",
    mixvec![
      // \Gamma     \Delta      \Theta      \Lambda      \Xi         \Pi         \Sigma \Upsilon
      '\u{0393}', '\u{0394}', '\u{0398}', '\u{039B}', '\u{039E}', '\u{03A0}', '\u{03A3}',
      '\u{03A5}',
      // \Phi       \Psi        \Omega      alpha        beta gamma       delta       epsilon
      '\u{03A6}', '\u{03A8}', '\u{03A9}', '\u{03B1}', '\u{03B2}', '\u{03B3}', '\u{03B4}',
      '\u{03F5}',
      // zeta       eta         theta iota         kappa      lambda       mu nu
      '\u{03B6}', '\u{03B7}', '\u{03B8}', '\u{03B9}', '\u{03BA}', '\u{03BB}', '\u{03BC}',
      '\u{03BD}',
      // xi         pi          rho         sigma       tau         upsilon     phi chi
      '\u{03BE}', '\u{03C0}', '\u{03C1}', '\u{03C3}', '\u{03C4}', '\u{03C5}', '\u{03D5}',
      '\u{03C7}',
      // psi        omega       varepsilon  vartheta    varpi       varrho  varsigma    varphi
      '\u{03C8}', '\u{03C9}', '\u{03B5}', '\u{03D1}', '\u{03D6}', '\u{03F1}', '\u{03C2}',
      '\u{03C6}',
      // l.harp.up  l.harp.dn   r.harp.up   r.harp.dnlhook       rhook       rt.tri     lf.tri
      '\u{21BC}', '\u{21BD}', '\u{21C0}', '\u{21C1}', '\u{2E26}', '\u{2E27}', '\u{25B7}',
      '\u{25C1}',
      // old style numerals! (no separate codepoints ?)
      // 0          1           2           3             4           5          6           7
      '0', '1', '2', '3', '4', '5', '6', '7',
      // 8          9           .           ,             <           /          >           star
      '8', '9', '.', ',', '\u{003C}', '\u{002F}', '\u{003E}', '\u{22C6}',
      // partial    A           B           C             D           E          F           G
      '\u{2202}', 'A', 'B', 'C', 'D', 'E', 'F', 'G',
      // H          I           J           K             L           M          N           O
      'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O',
      // P          Q           R           S             T           U          V           W
      'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W',
      // X          Y           Z           flat          natural     sharp      smile       frown
      'X', 'Y', 'Z', '\u{266D}', '\u{266E}', '\u{266F}', '\u{2323}', '\u{2322}',
      // ell        a           b           c             d           e          f           g
      '\u{2113}', 'a', 'b', 'c', 'd', 'e', 'f', 'g',
      // h          i           j           k             l           m          n           o
      'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o',
      // p          q           r           s             t           u          v           w
      'p', 'q', 'r', 's', 't', 'u', 'v', 'w',
      // x          y           z           dotless i    dotless j    weier-p    arrow
      // acc.inv.breve
      'x', 'y', 'z', '\u{0131}', 'j', '\u{2118}', '\u{2192}', '\u{0311}'
    ]
  ); // TODO: '\u{00A0}' .'\u{0311}'

  DeclareFontMap!(
    "OMS",
    mixvec![
      // minus       dot         times       ast          divide      diamond    plus-minus
      // minus-plus
      '-',
      '\u{22C5}',
      '\u{00D7}',
      '\u{2217}',
      '\u{00F7}',
      '\u{22C4}',
      '\u{00B1}',
      '\u{2213}',
      // oplus      ominus      otimes      oslash       odot        bigcirc circ        bullet
      '\u{2295}',
      '\u{2296}',
      '\u{2297}',
      '\u{2298}',
      '\u{2299}',
      '\u{25CB}',
      '\u{2218}',
      '\u{2219}',
      // asymp      equiv       subseteq    supseteq leq         geq         preceq      succeq
      '\u{224D}',
      '\u{2261}',
      '\u{2286}',
      '\u{2287}',
      '\u{2264}',
      '\u{2265}',
      '\u{2AAF}',
      '\u{2AB0}',
      // sim        approx      subset      supset       ll          gg   prec        succ
      '\u{223C}',
      '\u{2248}',
      '\u{2282}',
      '\u{2283}',
      '\u{226A}',
      '\u{226B}',
      '\u{227A}',
      '\u{227B}',
      // leftarrow  rightarrow  uparrow     downarrow    leftrightar nearrow     searrow     simeq
      '\u{2190}',
      '\u{2192}',
      '\u{2191}',
      '\u{2193}',
      '\u{2194}',
      '\u{2197}',
      '\u{2198}',
      '\u{2243}',
      // Leftarrow  Rightarrow  Uparrow Downarrow    Leftrightar nwarrow     swarrow propto
      '\u{21D0}',
      '\u{21D2}',
      '\u{21D1}',
      '\u{21D3}',
      '\u{21D4}',
      '\u{2196}',
      '\u{2199}',
      '\u{221D}',
      // prime      infty       in          ni           bigtri.up   bigtri.dn   slash       mapsto
      '\u{2032}',
      '\u{221E}',
      '\u{2208}',
      '\u{220B}',
      '\u{25B3}',
      '\u{25BD}',
      '/',
      '\u{21A6}',
      // forall     exists      not         emptyset  Re          Im          top         bot
      '\u{2200}',
      '\u{2203}',
      '\u{00AC}',
      '\u{2205}',
      '\u{211C}',
      '\u{2111}',
      '\u{22A4}',
      '\u{22A5}',
      // aleph      cal A       cal B       cal    C        cal D       cal E       cal F  cal G
      '\u{2135}',
      '\u{1D49C}',
      '\u{212C}',
      '\u{1D49E}',
      '\u{1D49F}',
      '\u{2130}',
      '\u{2131}',
      '\u{1D4A2}',
      // cal H      cal I       cal J       cal K        cal L      cal M       cal N       cal O
      '\u{210B}',
      '\u{2110}',
      '\u{1D4A5}',
      '\u{1D4A6}',
      '\u{2112}',
      '\u{2133}',
      '\u{1D4A9}',
      '\u{1D4AA}',
      // cal P      cal Q       cal R cal S        cal T       cal U       cal V   cal W
      '\u{1D4AB}',
      '\u{1D4AC}',
      '\u{211B}',
      '\u{1D4AE}',
      '\u{1D4AF}',
      '\u{1D4B0}',
      '\u{1D4B1}',
      '\u{1D4B2}',
      // cal X      cal Y       cal Z       cup          cap       uplus       wedge       vee
      '\u{1D4B3}',
      '\u{1D4B4}',
      '\u{1D4B5}',
      '\u{222A}',
      '\u{2229}',
      '\u{228C}',
      '\u{2227}',
      '\u{2228}',
      // vdash      dashv       lfloor    rfloor       lceil       rceil       lbrace       rbrace
      '\u{22A2}',
      '\u{22A3}',
      '\u{230A}',
      '\u{230B}',
      '\u{2308}',
      '\u{2309}',
      '{',
      '}',
      // langle     rangle       |          \|           updownarrow Updownarrow backslash   wr
      '\u{27E8}',
      '\u{27E9}',
      '|',
      '\u{2225}',
      '\u{2195}',
      '\u{21D5}',
      '\u{005C}',
      '\u{2240}',
      // surd       amalg       nabla       int          sqcup      sqcap        sqsubseteq
      // sqsupseteq
      '\u{221A}',
      '\u{2210}',
      '\u{2207}',
      '\u{222B}',
      '\u{2294}',
      '\u{2293}',
      '\u{2291}',
      '\u{2292}',
      // section    dagger      ddagger     para         clubsuit       diam.suit   heartsuit
      // spadesuit
      '\u{00A7}',
      '\u{2020}',
      '\u{2021}',
      '\u{00B6}',
      '\u{2663}',
      '\u{2662}',
      '\u{2661}',
      '\u{2660}'
    ]
  );

  DeclareFontMap!(
    "OMX",
    mixvec![
      // (          )           [           ]             lfloor      rfloor      lceil rceil
      '(', ')', '[', ']', '\u{230A}', '\u{230B}', '\u{2308}', '\u{2309}',
      //lbrace      rbrace      langle      rangle        |           ||          /           \
      '{', '}', '\u{27E8}', '\u{27E9}', '|', '\u{2225}', '/', '\u{005C}', '(', ')', '(', ')', '[',
      ']', '\u{230A}', '\u{230B}', '\u{2308}', '\u{2309}', '{', '}', '\u{27E8}', '\u{27E9}', '/',
      '\u{005C}', '(', ')', '[', ']', '\u{230A}', '\u{230B}', '\u{2308}', '\u{2309}', '{', '}',
      '\u{27E8}', '\u{27E9}', '/', '\u{005C}', '/', '\u{005C}',
      // next two rows are just fragments
      // l.up.paren r.up.paren  l.up.brak   r.up.brak    l.bot.brak  r.bot.brak  l.brak.ext
      // r.brak.ext
      '\u{239B}', '\u{239E}', '\u{23A1}', '\u{23A4}', '\u{23A3}', '\u{23A6}', '\u{23A2}',
      '\u{23A5}', /* l.up.brace r.up.brace  l.bot.brace r.bot.brace  l.brace.mid r.brace.mid
                   * brace.ext  v.arrow.ext */
      '\u{23A7}', '\u{23AB}', '\u{23A9}', '\u{23AD}', '\u{23A8}', '\u{23AC}', '\u{23AA}',
      '\u{23D0}', // l.bot.paren r.bot.paren l.paren.ext     r.paren.ext
      '\u{239D}', '\u{23A0}', '\u{239C}', '\u{239F}', '\u{27E8}', '\u{27E9}', '\u{2294}',
      '\u{2294}', '\u{222E}', '\u{222E}', '\u{2299}', '\u{2299}', '\u{2295}', '\u{2295}',
      '\u{2297}', '\u{2297}', '\u{2211}', '\u{220F}', '\u{222B}', '\u{22C3}', '\u{22C2}',
      '\u{228C}', '\u{2227}', '\u{2228}', '\u{2211}', '\u{220F}', '\u{222B}', '\u{22C3}',
      '\u{22C2}', '\u{228C}', '\u{2227}', '\u{2228}', '\u{2210}', '\u{2210}', '\u{005E}',
      '\u{005E}', '\u{005E}', '\u{007E}', '\u{007E}', '\u{007E}', '[', ']', '\u{230A}', '\u{230B}',
      '\u{2308}', '\u{2309}', '{', '}',
      // [missing rad frags]     double arrow ext.
      '\u{23B7}', '\u{23B7}', '\u{23B7}', '\u{23B7}', '\u{23B7}', None, None, None,
      //                        [missing tips for horizontal curly braces]
      '\u{2191}', '\u{2193}', None, None, None, None, '\u{21D1}', '\u{21D3}'
    ]
  );

  DefPrimitive!("\\char Number", sub[stomach, args, p_state] {
    unpack_to_token!(args=>token);
    let number = token.to_number();
    let gullet = stomach.get_gullet_mut();
    let decoded = match font::decode(number.value_of() as u8, None, false, p_state) {
      None => String::new(),
      Some(c) => c.to_string()
    };
    let invoked = Invocation!(T_CS!("\\char"), vec![token], gullet, p_state)?;
    Tbox::new(
     decoded,
     None,
     None,
     invoked,
     HashMap::new(), 
     p_state).into()
  });

  // Almost like a register, but different...
  DefPrimitive!("\\chardef Token SkipMatch:= Number", sub[stomach, args, p_state] {
    unpack_to_token!(args => newcs, value);
    let csname = newcs.get_cs_name();
    let number = value.to_number();
    let chardef_value = value.clone();
    let internalcs = T_CS!(&format!("\\@chardef@{}", csname));
    DefPrimitiveII!(internalcs, None, sub[stomach,args,i_state] {
      let gullet = stomach.get_gullet_mut();
      let decoded = match font::decode(number.value_of() as u8, None, false, i_state) {
        None => String::new(),
        Some(c) => c.to_string()
      };
      Tbox::new(decoded, 
        None,
        None,
        Invocation!(T_CS!("\\char"), vec![value.clone()], gullet, i_state)?, 
        HashMap::new(),
        i_state).into()
    }, p_state); 

    p_state.install_definition(Register::new_chardef(newcs, Some(chardef_value.into()), Some(internalcs)), None);
    AfterAssignment!(p_state);
    Ok(vec![])
  });

  Ok(())
}
 