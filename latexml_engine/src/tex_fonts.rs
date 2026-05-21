//! TeX Fonts
//!
//! Core TeX Implementation for LaTeXML

use crate::prelude::*;

LoadDefinitions!({
  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // Fonts Family of primitive control sequences
  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%

  //======================================================================
  // Font declaration
  //----------------------------------------------------------------------
  // \font             iq loads information about a font into TeX's memory.
  // \fontname         c  returns the system file name for a font.
  // \fontdimen        iq holds font parameters.
  // \nullfont         iq is a predefined font with no characters.

  // # 2nd arg is <font> = <fontdef token> | \font | <family member>
  // #  <family member> = <font range><4bit number>
  // #  <font range> = \textfont | \scriptfont | \scriptscriptfont
  // Perl: FontDef param type — for \textfont/\scriptfont/\scriptscriptfont, reads family
  // number and looks up the stored font CS token.  For \font, returns current_FontDef.
  DefParameterType!(FontToken, sub[_inner, _extra] {
    let token = gullet::read_token()?.unwrap();
    if let Some(font_type) = token.with_str(|ts| {
      if ts.starts_with("\\textfont") && ts == "\\textfont" { Some("textfont") }
      else if ts.starts_with("\\scriptscriptfont") && ts == "\\scriptscriptfont" { Some("scriptscriptfont") }
      else if ts.starts_with("\\scriptfont") && ts == "\\scriptfont" { Some("scriptfont") }
      else { None }
    }) {
      // Perl: $token = LookupValue($type . 'font_' . $fam->valueOf).
      // with_value avoids the Stored envelope clone; Token is Copy.
      let fam = gullet::read_number()?.value_of();
      let key = s!("{font_type}_{fam}");
      state::with_value(&key, |v| match v {
        Some(Stored::Token(t)) => *t,
        _ => token,
      })
    } else if token.with_str(|ts| ts == "\\font") {
      // Perl: $token = LookupValue('current_FontDef') || T_CS('\lx@default@font')
      state::with_value("current_FontDef", |v| match v {
        Some(Stored::Token(t)) => *t,
        _ => T_CS!("\\lx@default@font"),
      })
    } else {
      token
    }
  });

  // Perl TeX_Fonts.pool.ltxml L77-78: default skew/hyphen registers
  // before \font.
  DefRegister!("\\defaultskewchar", Number!(-1));
  DefRegister!("\\defaulthyphenchar", Number!(45)); // ord('-') = 45

  // Perl: \font SkipSpaces Token SkipSpaces SkipMatch:= SkipSpaces TeXFileName
  // (TeX_Fonts.pool.ltxml:82). Token does NOT auto-skip spaces in Rust gullet
  // (parity with Perl readToken), so the leading SkipSpaces is the
  // Perl-faithful guard for inputs like `\font  \foo  =  cmr10`.
  DefPrimitive!("\\font SkipSpaces Token SkipSpaces SkipMatch:= SkipSpaces TeXFileName",
  sub[(cs, name_arg)] {
    let name = name_arg.to_string();
    // Read optional "at <dimen>" or "scaled <number>" — Perl: TeX_Fonts.pool.ltxml L88-94
    //   if    ($gullet->readKeyword('at'))     { $at = $gullet->readDimension; }
    //   elsif ($gullet->readKeyword('scaled')) { $scaled = $gullet->readNumber/1000; }
    // The `elsif` is load-bearing: only one branch fires. Without it, an
    // `at <dim> scaled <n>` input would have BOTH consumed instead of leaving
    // `scaled <n>` in the stream as Perl does.
    let mut at_pt = None;
    let mut at_sp = None;  // raw sp value, needed for shared-key precision
    let mut scaled = None;
    if gullet::read_keyword(&["at"])?.is_some() {
      let d = gullet::read_dimension()?;
      at_sp = Some(d.value_of());            // exact sp count, lossless
      at_pt = Some(d.pt_value(None));        // ≈2-decimal pt for display
    } else if gullet::read_keyword(&["scaled"])?.is_some() {
      scaled = Some(gullet::read_number()?.value_of() as f64 / 1000.0);
    }
    let props_opt = if let Some(mut props) = font::decode_fontname(&name, at_pt, scaled) {
      props.name = Some(Cow::Owned(name.clone()));
      Some(props)
    } else { // Failed?
      let message = s!("Unrecognized font name {:?} Font switch macro {:?}
      will have no effect", name, cs.stringify());
      Info!("unexpected", name, message);
      None
    };
    gullet::skip_spaces()?;
    let cs_str = cs.to_string();
    if let Some(ref props) = props_opt {
      AssignValue!(&s!("fontinfo_{cs_str}"), props.clone());
    }
    // When `scaled <N>` was used, derive `at_sp` from the resolved size
    // so two `\font` declarations resolving to the same effective size
    // (e.g. `at 5pt` vs `scaled 500` of cmr10) share the same shared
    // key — without this, hyphenchar/fontdimen writes through one are
    // invisible from the other. plainfonts_test L42 catches this.
    if at_sp.is_none() {
      if let Some(sz_pt) = props_opt.as_ref().and_then(|p| p.size) {
        at_sp = Some((sz_pt * 65536.0).round() as i64);
      }
    }
    // Perl: $key = 'fontinfo_' . $name; $key .= " at " . ToString($at) if $at;
    // Shared font key: fonts with same name+size share hyphenchar/skewchar.
    // Precision matters here — expl3's intarray font-hack creates a
    // distinct `\font \foo = cmr10 at <N> sp` for each intarray, where
    // <N> is a unique integer (the intarray table index). Rounding to
    // `.1pt` (or any pt-decimal precision) collapses all these to
    // `0.0pt` and breaks intarray storage because all "different"
    // intarrays then share the same fontdimen backing. Use the raw
    // sp integer (lossless) in the key so e.g. `at 23 sp` and `at 24 sp`
    // are guaranteed distinct. Keep the pt-formatted form for `\meaning`
    // display, but separate the two concerns.
    // `at_str_opt` is used only for the `\meaning` display ("at 5.0pt").
    // The shared key uses `at_sp` (lossless sp count) for uniqueness.
    let at_str_opt = if let Some(at_val) = at_pt {
      Some(s!("{at_val:.1}pt"))
    } else if let Some(_sc) = scaled {
      props_opt.as_ref().and_then(|p| p.size).map(|sz| s!("{sz:.1}pt"))
    } else {
      None
    };
    // Compose the shared key using exact sp value (or `at_str_opt` for
    // the `scaled` branch which has no sp). Two `\font` calls with
    // different sp sizes produce different shared keys, so their
    // fontdimen/hyphenchar/skewchar state stays independent.
    let shared_key = if let Some(sp) = at_sp {
      s!("fontinfo_{name} at {sp}sp")
    } else if let Some(ref at_str) = at_str_opt {
      s!("fontinfo_{name} at {at_str}")
    } else {
      s!("fontinfo_{name}")
    };
    // Store CS → shared key mapping
    state::assign_value(
      &s!("font_shared_key_{cs_str}"),
      Stored::String(arena::pin(&shared_key)),
      None,
    );
    // Store explicit "at" value for \meaning (Perl: $$fontinfo{at} = ToString($at))
    if let Some(ref at_str) = at_str_opt {
      state::assign_value(
        &s!("fontinfo_at_{cs_str}"),
        Stored::String(arena::pin(at_str)),
        None,
      );
    }
    // Perl: only initialize hyphenchar/skewchar if this font key hasn't been seen before
    // (shared fontinfo means second \font with same name+size reuses existing values)
    let hc_key = s!("hyphenchar_{shared_key}");
    if !state::has_value(&hc_key) {
      let default_hyphen = lookup_int("\\defaulthyphenchar");
      state::assign_value(
        &hc_key,
        Stored::Number(Number::new(default_hyphen as i64)),
        Some(Scope::Global),
      );
      let default_skew = lookup_int("\\defaultskewchar");
      state::assign_value(
        &s!("skewchar_{shared_key}"),
        Stored::Number(Number::new(default_skew as i64)),
        Some(Scope::Global),
      );
    }
    // Perl: installDefinition(FontDef->new($cs, $key))
    //   FontDef.pm L42: assignValue(current_FontDef => $$self{cs}, 'local')
    // Perl's State::installDefinition with $scope undef → assign_internal
    // defaults to local-with-\global-prefix-promotion (State.pm L152).
    // Rust DefPrimitive! defaults match. TODO (SYNC_STATUS): rewrite this
    // primitive to a strict Perl-faithful translation — see Work Plan.
    let is_global = state::get_prefix("global");
    let cs_for_fontdef = cs;
    let font_id_key = arena::pin(s!("fontinfo_{cs_str}").as_str());
    // SYNC_STATUS Cluster C: do NOT bypass the lock here. If the target CS
    // is locked (e.g. `\abstract`, `\title`), `\font\<cs>=<file>` is a
    // documented no-op — state::install_definition emits an
    // Info!("ignore", "<cs>:locked", ...) and skips. Witnesses are ~46
    // pre-2000 plain-TeX-style papers that misuse `\font\abstract=cmr8`
    // expecting a font-switch group; both Perl and Rust LaTeXML treat
    // `\font` on a locked primitive as a no-op (SHARED-FAILURE on the
    // downstream `{\abstract ...}` mode-switch). The author's source
    // violates a LaTeX convention (shadowing a class-provided macro;
    // should have used `\newfont` which does an `\@ifundefined` check).
    // User directive 2026-05-19: "\font on a locked primitive shouldn't
    // work" — do not surpass Perl here.
    DefPrimitive!(cs, None, None, font => props_opt,
      before_digest => sub {
        AssignValue!("current_FontDef", cs_for_fontdef, None);
      }
    );
    // Tag the just-installed primitive with its fontinfo lookup key — this
    // is the Rust `font_id` (Perl `LaTeXML::Core::Definition::FontDef::fontID`)
    // that lets dump_writer emit `FD\t<font_id>` (Perl `dump_primitive` path
    // L383-389) instead of the generic Primitive serialization. Without the
    // tag, the writer falls into the `PA\t<self_cs>` self-alias path which
    // dump_reader skips, leaving the CS undefined post-dump.
    if let Some(Stored::Primitive(p)) = state::lookup_meaning(&cs) {
      let mut p_owned: latexml_core::definition::primitive::Primitive = (*p).clone();
      p_owned.font_id = Some(font_id_key);
      state::assign_meaning(&cs, Stored::Primitive(std::rc::Rc::new(p_owned)),
        if is_global { Some(Scope::Global) } else { None });
    }
    if is_global {
      if let Some(meaning) = state::lookup_meaning(&cs) {
        state::assign_meaning(&cs, meaning, Some(Scope::Global));
      }
    }
  });

  // Perl: DefMacro('\fontname FontDef', sub { Explode($fontinfo && $$fontinfo{name}
  //       || "fontname not available") })
  // Perl's FontDef param type: for \font, looks up current_FontDef, falls back to \lx@default@font
  DefMacro!("\\fontname FontToken", sub[args] {
    let token = args.into_iter().next().unwrap().expected_token();
    let cs_str = token.to_string();
    // Determine which font CS to look up
    let lookup_cs = if cs_str == "\\font" {
      // Current font — look up current_FontDef, fallback to \lx@default@font.
      // with_value avoids the Stored envelope clone; Token is Copy.
      state::with_value("current_FontDef", |v| match v {
        Some(Stored::Token(t)) => t.to_string(),
        _ => s!("\\lx@default@font"),
      })
    } else {
      cs_str
    };
    let key = s!("fontinfo_{}", lookup_cs);
    // Same pattern for the font-name read — we only need the owned name
    // String, never the Rc<Font>.
    let name = state::with_value(&key, |v| match v {
      Some(Stored::Font(f)) => f.name.as_ref().map(|n| n.to_string()),
      _ => None,
    });
    Tokens::new(Explode!(name.unwrap_or_else(|| s!("fontname not available"))))
  });
  DefRegister!("\\fontdimen Number FontToken", Dimension::new(0),
    getter => sub[args] {
      let p = args.remove(0).expect_number().value_of();
      let font_token = args.remove(0).expected_token();
      let cs_str = font_token.to_string();
      // Per-font fontdimen<p> override. Resolve to the canonical
      // font identity via the token's Primitive `font_id` (which is
      // shared across `\let` aliases — mirrors Perl's FontDef object
      // sharing on `\let`). Without this indirection, `\let \fb = \fa`
      // followed by `\fontdimen 1 \fb = 42pt` would store under
      // `font_shared_key_\fb` while `\the\fontdimen 1 \fb` reads
      // `font_shared_key_\fa`, producing 0pt.
      //
      // Witness: tests/structure/glossary.tex `\Gls{cabbage}` → "Cabbage"
      // needs expl3's `c__codepoint_uppercase_index_intarray` populated,
      // and the c__ alias is created via `\cs_gset_eq:cc { c__... }
      // { g__... }` (=`\let`).
      let canonical_cs = state::lookup_meaning(&font_token)
        .and_then(|m| if let Stored::Primitive(p) = m { p.font_id }
                      else { None })
        .map(|fid| {
          let s = arena::with(fid, |x| x.to_string());
          s.strip_prefix("fontinfo_").unwrap_or(&s).to_string()
        })
        .unwrap_or_else(|| cs_str.clone());
      let fd_key = state::with_value(&s!("font_shared_key_{canonical_cs}"), |v| match v {
        Some(Stored::String(s)) => arena::with(*s, |sk| s!("fontdimen_{sk}_{p}")),
        _ => s!("fontdimen_{canonical_cs}_{p}"),
      });
      let stored_val = state::with_value(&fd_key, |v| match v {
        Some(Stored::Dimension(d)) => Some(*d),
        Some(Stored::Number(n)) => Some(Dimension::new(n.value_of())),
        _ => None,
      });
      if let Some(d) = stored_val { return Some(d.into()); }
      // Fall-through: hard-coded cmr10-like defaults for indices that
      // user code commonly reads (\fontdimen2..22) when no explicit write
      // has happened. Preserves the prior behaviour for math layout code.
      match p {
        2 => Dimension::from_str("0.5em").ok()?,    // interword space
        5 => Dimension::from_str("1ex").ok()?,      // x-height
        6 => Dimension::from_str("1em").ok()?,      // quad width
        8 => Dimension::from_str("0.677em").ok()?,  // num1: numerator shift (display)
        9 => Dimension::from_str("0.394em").ok()?,  // num2: numerator shift (text)
        10 => Dimension::from_str("0.444em").ok()?, // num3
        11 => Dimension::from_str("0.686em").ok()?, // denom1: denominator shift (display)
        12 => Dimension::from_str("0.345em").ok()?, // denom2: denominator shift (text)
        22 => Dimension::from_str("0.25em").ok()?,  // math axis height (cmsy10: 2.5pt at 10pt)
        _ => Dimension::new(0)
      }
    },
    setter => sub[value, _scope, args] {
      let p = args.remove(0).expect_number().value_of();
      let font_token = args.remove(0).expected_token();
      let cs_str = font_token.to_string();
      // Resolve to canonical font identity via Primitive.font_id —
      // matches the getter so `\let`-aliased fonts share storage.
      let canonical_cs = state::lookup_meaning(&font_token)
        .and_then(|m| if let Stored::Primitive(p) = m { p.font_id }
                      else { None })
        .map(|fid| {
          let s = arena::with(fid, |x| x.to_string());
          s.strip_prefix("fontinfo_").unwrap_or(&s).to_string()
        })
        .unwrap_or_else(|| cs_str.clone());
      let fd_key = state::with_value(&s!("font_shared_key_{canonical_cs}"), |v| match v {
        Some(Stored::String(s)) => arena::with(*s, |sk| s!("fontdimen_{sk}_{p}")),
        _ => s!("fontdimen_{canonical_cs}_{p}"),
      });
      state::assign_value(
        &fd_key,
        Stored::Dimension(value.into()),
        Some(Scope::Global),
      );
    }
  );
  // \defaultskewchar / \defaulthyphenchar moved up to mirror Perl
  // L77-78 (right before \font).

  // Not sure what this should be...
  DefPrimitive!("\\nullfont", None, font => {family => "nullfont"});

  //======================================================================
  // Italic correction
  //----------------------------------------------------------------------
  // / (italic corr.)  c  inserts an italic correction.
  DefPrimitive!("\\/", {
    Tbox::new(
      pin!(""),
      None,
      None,
      Tokens!(T_CS!("\\/")),
      stored_map!("isSpace" => true, "name" => "italiccorr", "width" => Dimension::default()),
    )
  });
  DefPrimitive!("\\lx@fontencoding{}", sub[(encoding)] {
    let encoding = Expand!(encoding).to_string();
    if load_font_map(&encoding).is_some() {
      MergeFont!(encoding => encoding);
    } else {
      Info!("missing_font_encoding", encoding,
        "Couldn't find font encoding, falling back to OT1");
      // Default to OT1 encoding if no map found
      MergeFont!(encoding => "OT1");
    }
    Ok(Vec::new())
  });
  // Used for SemiVerbatim text
  DeclareFontMap!("ASCII", mixrc![
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    ' ', '!', '"', '#', '$', '%', '&', '\'', '(', ')', '*', '+', ',', '-', '.', '/', '0', '1', '2',
    '3', '4', '5', '6', '7', '8', '9', ':', ';', '<', '=', '>', '?', '@', 'A', 'B', 'C', 'D', 'E',
    'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X',
    'Y', 'Z', '[', '\\', ']', '^', '_', '`', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k',
    'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '{', '|', '}', '~',
    None
  ]);

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
  DeclareFontMap!("OT1", mixrc![
    '\u{0393}', '\u{0394}', '\u{0398}', '\u{039B}', '\u{039E}', '\u{03A0}', '\u{03A3}', '\u{03A5}',
    '\u{03A6}', '\u{03A8}', '\u{03A9}', '\u{FB00}', '\u{FB01}', '\u{FB02}', '\u{FB03}', '\u{FB04}',
    '\u{0131}', '\u{0237}', '\u{0060}', '\u{00B4}', '\u{02C7}', '\u{02D8}', '\u{00AF}', '\u{02DA}',
    '\u{00B8}', '\u{00DF}', '\u{00E6}', '\u{0153}', '\u{00F8}', '\u{00C6}', '\u{0152}', '\u{00D8}',
    '\u{0335}', '!', '\u{201D}', '#', '$', '%', '&', '\u{2019}', '(', ')', '*', '+', ',', '-', '.',
    '/', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', ':', ';', '\u{00A1}', '=', '\u{00BF}',
    '?', '@', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q',
    'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '[', '\u{201C}', ']', '\u{02C6}', '\u{02D9}',
    '\u{2018}', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p',
    'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '\u{2013}', '\u{2014}', '\u{02DD}',
    '\u{02DC}', '\u{00A8}'
  ]); // TODO: do we really need '\u{00A0}'\x{0335} as a single entry?

  DeclareFontMap!(
    "OT1",
    mixrc![
      '\u{0393}', '\u{0394}', '\u{0398}', '\u{039B}', '\u{039E}', '\u{03A0}', '\u{03A3}',
      '\u{03A5}', '\u{03A6}', '\u{03A8}', '\u{03A9}', '\u{2191}', '\u{2193}', '\'', '\u{00A1}',
      '\u{00BF}', '\u{0131}', '\u{0237}', '\u{0060}', '\u{00B4}', '\u{02C7}', '\u{02D8}',
      '\u{00AF}', '\u{02DA}', '\u{00B8}', '\u{00DF}', '\u{00E6}', '\u{0153}', '\u{00F8}',
      '\u{00C6}', '\u{152}', '\u{00D8}', '\u{2423}', '!', '"', '#', '$', '%', '&', '\u{2019}', '(',
      ')', '*', '+', ',', '-', '.', '/', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', ':',
      ';', '<', '=', '>', '?', '@', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L',
      'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '[', '\\', ']', '^',
      '_', '\u{2018}', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o',
      'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '{', '|', '}', '~', '\u{00A8}'
    ],
    "typewriter"
  );
  #[rustfmt::skip]
  DeclareFontMap!(
    "OML",
    mixrc![
      // \Gamma     \Delta      \Theta      \Lambda      \Xi         \Pi         \Sigma \Upsilon
      '\u{0393}', '\u{0394}', '\u{0398}', '\u{039B}', '\u{039E}', '\u{03A0}', '\u{03A3}',
      '\u{03A5}',
      // \Phi       \Psi        \Omega      alpha        beta gamma       delta       epsilon
      '\u{03A6}', '\u{03A8}', '\u{03A9}', '\u{03B1}', '\u{03B2}', '\u{03B3}', '\u{03B4}',
      '\u{03F5}', // zeta       eta         theta iota         kappa      lambda       mu nu
      '\u{03B6}', '\u{03B7}', '\u{03B8}', '\u{03B9}', '\u{03BA}', '\u{03BB}', '\u{03BC}',
      '\u{03BD}', // xi         pi          rho         sigma       tau         upsilon     phi chi
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
      '0', '1', '2', '3', '4', '5', '6',
      '7', /* 8          9           .           ,             <           /          >
            * star */
      '8', '9', '.', ',', '\u{003C}', '\u{002F}', '\u{003E}',
      '\u{22C6}', /* partial    A           B           C             D           E          F
                   * G */
      '\u{2202}', 'A', 'B', 'C', 'D', 'E', 'F',
      'G', // H          I           J           K             L           M          N           O
      'H', 'I', 'J', 'K', 'L', 'M', 'N',
      'O', // P          Q           R           S             T           U          V           W
      'P', 'Q', 'R', 'S', 'T', 'U', 'V',
      'W', /* X          Y           Z           flat          natural     sharp      smile
            * frown */
      'X', 'Y', 'Z', '\u{266D}', '\u{266E}', '\u{266F}', '\u{2323}',
      '\u{2322}', /* ell        a           b           c             d           e          f
                   * g */
      '\u{2113}', 'a', 'b', 'c', 'd', 'e', 'f',
      'g', // h          i           j           k             l           m          n           o
      'h', 'i', 'j', 'k', 'l', 'm', 'n',
      'o', // p          q           r           s             t           u          v           w
      'p', 'q', 'r', 's', 't', 'u', 'v',
      'w', // x          y           z           dotless i    dotless j    weier-p    arrow
      // acc.inv.breve
      'x', 'y', 'z', '\u{0131}', 'j', '\u{2118}', '\u{2192}', '\u{0361}'
    ]
  ); // Perl: '\u{00A0}' . '\u{0361}' — two-char string, we use U+0361 only

  #[rustfmt::skip]
  DeclareFontMap!(
    "OMS",
    mixrc![
    // minus     dot         times       ast          divide      diamond    plus-minus minus-plus
    '-',        '\u{22C5}', '\u{00D7}', '\u{2217}', '\u{00F7}', '\u{22C4}', '\u{00B1}', '\u{2213}',
    // oplus      ominus      otimes      oslash       odot        bigcirc circ        bullet
    '\u{2295}', '\u{2296}', '\u{2297}', '\u{2298}', '\u{2299}', '\u{25CB}', '\u{2218}', '\u{2219}',
    // asymp      equiv       subseteq    supseteq leq         geq         preceq      succeq
    '\u{224D}', '\u{2261}', '\u{2286}', '\u{2287}', '\u{2264}', '\u{2265}', '\u{2AAF}', '\u{2AB0}',
    // sim        approx      subset      supset       ll          gg   prec        succ
    '\u{223C}', '\u{2248}', '\u{2282}', '\u{2283}', '\u{226A}', '\u{226B}', '\u{227A}', '\u{227B}',
    // leftarrow  rightarrow  uparrow     downarrow    leftrightar nearrow     searrow     simeq
    '\u{2190}', '\u{2192}', '\u{2191}', '\u{2193}', '\u{2194}', '\u{2197}', '\u{2198}', '\u{2243}',
    // Leftarrow  Rightarrow  Uparrow Downarrow    Leftrightar nwarrow     swarrow propto
    '\u{21D0}', '\u{21D2}', '\u{21D1}', '\u{21D3}', '\u{21D4}', '\u{2196}', '\u{2199}', '\u{221D}',
    // prime      infty       in          ni           bigtri.up   bigtri.dn   slash       mapsto
    '\u{2032}', '\u{221E}', '\u{2208}', '\u{220B}', '\u{25B3}', '\u{25BD}', '/', '\u{21A6}',
    // forall     exists      not         emptyset  Re          Im          top         bot
    '\u{2200}', '\u{2203}', '\u{00AC}', '\u{2205}', '\u{211C}', '\u{2111}', '\u{22A4}', '\u{22A5}',
    // aleph      cal A       cal B       cal    C        cal D       cal E       cal F  cal G
    '\u{2135}', '\u{1D49C}', '\u{212C}', '\u{1D49E}', '\u{1D49F}', '\u{2130}', '\u{2131}', '\u{1D4A2}',
    // cal H      cal I       cal J       cal K        cal L      cal M       cal N       cal O
    '\u{210B}', '\u{2110}', '\u{1D4A5}', '\u{1D4A6}', '\u{2112}', '\u{2133}', '\u{1D4A9}', '\u{1D4AA}',
    // cal P      cal Q       cal R cal S        cal T       cal U       cal V   cal W
    '\u{1D4AB}','\u{1D4AC}','\u{211B}','\u{1D4AE}','\u{1D4AF}','\u{1D4B0}','\u{1D4B1}','\u{1D4B2}',
    // cal X      cal Y       cal Z       cup          cap       uplus       wedge       vee
    '\u{1D4B3}','\u{1D4B4}','\u{1D4B5}','\u{222A}','\u{2229}','\u{228C}','\u{2227}','\u{2228}',
    // vdash      dashv       lfloor    rfloor       lceil       rceil       lbrace       rbrace
    '\u{22A2}', '\u{22A3}', '\u{230A}', '\u{230B}', '\u{2308}', '\u{2309}',  '{',         '}',
    // langle     rangle       |          \|  updownarrow Updownarrow backslash   wr
    '\u{27E8}', '\u{27E9}', '|', '\u{2225}', '\u{2195}', '\u{21D5}', '\u{005C}', '\u{2240}',
    // surd       amalg       nabla      int          sqcup      sqcap      sqsubseteq  sqsupseteq
    '\u{221A}', '\u{2210}', '\u{2207}', '\u{222B}', '\u{2294}', '\u{2293}', '\u{2291}', '\u{2292}',
    // section    dagger      ddagger     para         clubsuit  diam.suit   heartsuit  spadesuit
    '\u{00A7}', '\u{2020}', '\u{2021}', '\u{00B6}', '\u{2663}', '\u{2662}', '\u{2661}', '\u{2660}'
    ]
  );

  let cal_font = Font {
    family: Some(std::borrow::Cow::Borrowed("caligraphic")),
    ..Default::default()
  };
  latexml_core::state::assign_value(
    "OMS_uppercase_mathstyle",
    latexml_core::state::Stored::Font(std::rc::Rc::new(cal_font)),
    Some(latexml_core::state::Scope::Global),
  );

  #[rustfmt::skip]
  DeclareFontMap!(
    "OMX",
    mixrc![
      // (          )           [           ]             lfloor      rfloor      lceil rceil
      '(', ')', '[', ']', '\u{230A}', '\u{230B}', '\u{2308}',
      '\u{2309}', /* lbrace      rbrace      langle      rangle        |           ||          /
                   * \ */
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
      // ointctr  ointctr  odot(big) odot(big) oplus(big) oplus(big) otimes(big) otimes(big)
      '\u{2294}', '\u{222E}', '\u{222E}', '\u{2A00}', '\u{2A00}', '\u{2A01}', '\u{2A01}',
      // (Perl: 2A00=N-ARY CIRCLED DOT, 2A01=N-ARY CIRCLED PLUS, 2A02=N-ARY CIRCLED TIMES)
      '\u{2A02}', '\u{2A02}', '\u{2211}', '\u{220F}', '\u{222B}', '\u{22C3}', '\u{22C2}',
      // uplus(big)    wedge(big)  vee(big)
      '\u{2A04}', '\u{22C0}', '\u{22C1}', '\u{2211}', '\u{220F}', '\u{222B}', '\u{22C3}',
      '\u{22C2}', '\u{2A04}', '\u{22C0}', '\u{22C1}', '\u{2210}', '\u{2210}', '\u{005E}',
      '\u{005E}', '\u{005E}', '\u{007E}', '\u{007E}', '\u{007E}', '[', ']', '\u{230A}', '\u{230B}',
      '\u{2308}', '\u{2309}', '{', '}',
      // [missing rad frags]     double arrow ext.
      '\u{23B7}', '\u{23B7}', '\u{23B7}', '\u{23B7}', '\u{23B7}', None, None,
      None, //                        [missing tips for horizontal curly braces]
      '\u{2191}', '\u{2193}', None, None, None, None, '\u{21D1}', '\u{21D3}'
    ]
  );

  // Perl: Digest('\font\lx@default@font=cmr10');
  Digest!("\\font\\lx@default@font=cmr10")?;

  //======================================================================
  // Perl: TeX_Fonts.pool.ltxml L335-365 — TeX ligatures
  // Note: applied in reverse order of definition (latest defined applied first!)
  // Note also, these are only applied in text content, not in attributes!

  DefPrimitive!("\\@@endash", {
    Tbox::new(
      arena::pin_static("\u{2013}"),
      None,
      None,
      Tokens!(T_CS!("\\@@endash")),
      SymHashMap::default(),
    );
  });
  DefPrimitive!("\\@@emdash", {
    Tbox::new(
      arena::pin_static("\u{2014}"),
      None,
      None,
      Tokens!(T_CS!("\\@@emdash")),
      SymHashMap::default(),
    );
  });

  // EN DASH (NOTE: With digits before & aft => \N{FIGURE DASH})
  DefLigature!(r"--", "\u{2013}", fontTest => sub[arg] { non_typewriter(arg) });
  // EM DASH
  DefLigature!(r"---", "\u{2014}", fontTest => sub[arg] { non_typewriter(arg) });

  // Ligatures for doubled single left & right quotes to convert to double quotes
  DefLigature!("\u{2018}\u{2018}", "\u{201C}",
    fontTest => sub[arg] { non_typewriter_t1(arg) }); // double left quote
  DefLigature!("\u{2019}\u{2019}", "\u{201D}",
    fontTest => sub[arg] { non_typewriter_t1(arg) }); // double right quote
  DefLigature!("[?]\u{2018}", "\u{00BF}",
    fontTest => sub[arg] { non_typewriter_t1(arg) }); // ? backquote
  DefLigature!("!\u{2018}", "\u{00A1}",
    fontTest => sub[arg] { non_typewriter_t1(arg) }); // ! backquote

  // Perl: DefLigature(qr{\.\.\.}, "\x{2026}", fontTest => \&nonTypewriter);
  DefLigature!(r"[.][.][.]", "\u{2026}",
    fontTest => sub[arg] { non_typewriter(arg) }); // ldots
});

// Perl: TeX_Fonts.pool.ltxml L338-344
fn non_typewriter(font: &Font) -> bool {
  font.get_family().unwrap_or(&Cow::Borrowed("")) != "typewriter"
}

fn non_typewriter_t1(font: &Font) -> bool {
  non_typewriter(font)
    && matches!(
      font
        .get_encoding()
        .unwrap_or(&Cow::Borrowed("OT1"))
        .as_ref(),
      "OT1" | "T1"
    )
}
