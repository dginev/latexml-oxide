use latexml_package::prelude::*;

/// CJKutf8.sty:41-84 `\CJK@XX#1#2` … `\CJK@XXXXp`: the octet readers behind
/// the active lead bytes reassemble the code point by byte arithmetic
/// (`\csname CJK@\number`#1\endcsname{`#2}{`#3}` → a subfont glyph); the
/// `p` variants are the same readers with a `\protect` interleaved
/// (CJKutf8.sty:57 `\ifx #2\protect`). Under the byte mouth (K10) the arguments
/// are the bytes; emit the character they encode (`inputenc_sty::utf8_octets`,
/// skipping the `\protect`).
fn cjk_octets(args: &[ArgWrap]) -> Tokens {
  let bytes: Vec<ArgWrap> = args
    .iter()
    .filter(|a| inputenc_sty::byte_of_arg(a).is_some())
    .cloned()
    .collect();
  match bytes.split_first() {
    Some((lead, rest)) => match inputenc_sty::byte_of_arg(lead) {
      Some(lead) => inputenc_sty::utf8_octets(lead, rest),
      None => Tokens::new(Vec::new()),
    },
    None => Tokens::new(Vec::new()),
  }
}

/// A byte handed over as `` `<byte> `` (CJK.sty:981-1012 `{`#2}`): the
/// character after the backquote.
fn quoted_byte(arg: &ArgWrap) -> Option<u8> {
  let ts = arg.clone().owned_tokens()?;
  let t = ts
    .unlist_ref()
    .iter()
    .rev()
    .find(|t| t.code != Catcode::CS)
    .cloned()?;
  u8::try_from(t.get_charcode()).ok()
}

/// CJK.sty:981-1012 / CJKutf8.sty:41-84 / cjkutf8-ko.sty:392-430: every
/// octet reader ends in `\csname CJK@\number`<lead>\endcsname{`b2}{`b3}` — the
/// per-lead-byte handlers the UTF8 encoding's binding files supply (they map
/// the bytes to a subfont plane and glyph). Bind that layer: whatever a
/// package's own `\CJK@XXX` looks like, the bytes funnel here and become the
/// character.
fn bind_cjk_lead_byte_handlers() -> Result<()> {
  for lead in 0xC2..=0xF4u8 {
    let cs = T_CS!(s!("\\CJK@{}", lead));
    if lead <= 0xDF {
      DefMacro!(cs, "{}", sub[args] { Ok(cjk_lead_octets(lead, &args)) });
    } else if lead <= 0xEF {
      DefMacro!(cs, "{}{}", sub[args] { Ok(cjk_lead_octets(lead, &args)) });
    } else {
      DefMacro!(cs, "{}{}{}", sub[args] { Ok(cjk_lead_octets(lead, &args)) });
    }
  }
  Ok(())
}

fn cjk_lead_octets(lead: u8, args: &[ArgWrap]) -> Tokens {
  let mut bytes = vec![lead];
  bytes.extend(args.iter().filter_map(quoted_byte));
  match str::from_utf8(&bytes).ok().and_then(|t| t.chars().next()) {
    Some(ch) if bytes.len() == args.len() + 1 => {
      Tokens!(Token {
        text: pin_char(ch),
        code: Catcode::OTHER,
        #[cfg(feature = "token-locators")]
        loc: 0,
      })
    },
    _ => Tokens::new(Vec::new()),
  }
}

fn bind_cjk_utf8_octets() -> Result<()> {
  bind_cjk_lead_byte_handlers()?;
  let cjk_at_at_at = T_CS!("\\CJK@@@");
  if lookup_definition(&cjk_at_at_at)?.is_none() {
    def_macro_noop("\\CJK@@@")?;
    def_macro_noop("\\CJK@X{}")?;
  }
  // Bindings outrank raw: a raw CJKutf8.sty/cjkutf8-ko.sty load in between
  // (kotex's `[cjk]` path) leaves CJKutf8.sty:41's `\csname CJK@\number`#1
  // \endcsname{`#2}{`#3}` in place, whose subfont handlers do not exist here
  // (the backquotes typeset as ‘). Re-bind the readers at every env start.
  DefMacro!("\\CJK@XX{}{}", sub[args] { Ok(cjk_octets(&args)) });
  DefMacro!("\\CJK@XXp{}{}", sub[args] { Ok(cjk_octets(&args)) });
  DefMacro!("\\CJK@XXX{}{}{}", sub[args] { Ok(cjk_octets(&args)) });
  DefMacro!("\\CJK@XXXp{}{}{}{}", sub[args] { Ok(cjk_octets(&args)) });
  DefMacro!("\\CJK@XXXX{}{}{}{}", sub[args] { Ok(cjk_octets(&args)) });
  DefMacro!("\\CJK@XXXXp{}{}{}{}{}", sub[args] { Ok(cjk_octets(&args)) });
  // CJK.sty:898-903 `\CJK@makeActive`: 0x80..0xFE active — under the byte
  // mouth the bytes really arrive, so the meanings below need the catcode.
  if lookup_bool("PDFTEX_BYTE_MOUTH") {
    for code in 0x80..=0xF4u8 {
      assign_catcode(code as char, Catcode::ACTIVE, Some(Scope::Global));
    }
  }

  for code in 0x80..=0xF4u8 {
    let ch = code as char;
    let act = T_ACTIVE!(ch);
    let expansion = if code <= 0xBF {
      Tokens!(
        T_CS!("\\CJK@@@"),
        T_CS!("\\ifx"),
        T_CS!("\\protect"),
        T_CS!("\\@typeset@protect"),
        T_CS!("\\string"),
        act,
        T_CS!("\\else"),
        T_CS!("\\noexpand"),
        act,
        T_CS!("\\fi")
      )
    } else if code <= 0xDF {
      Tokens!(
        T_CS!("\\CJK@@@"),
        T_CS!("\\ifx"),
        T_CS!("\\protect"),
        T_CS!("\\@typeset@protect"),
        T_CS!("\\expandafter"),
        T_CS!("\\expandafter"),
        T_CS!("\\expandafter"),
        T_CS!("\\CJK@XX"),
        T_CS!("\\expandafter"),
        T_CS!("\\string"),
        T_CS!("\\expandafter"),
        act,
        T_CS!("\\else"),
        T_CS!("\\noexpand"),
        act,
        T_CS!("\\fi")
      )
    } else if code <= 0xEF {
      Tokens!(
        T_CS!("\\CJK@@@"),
        T_CS!("\\ifx"),
        T_CS!("\\protect"),
        T_CS!("\\@typeset@protect"),
        T_CS!("\\expandafter"),
        T_CS!("\\expandafter"),
        T_CS!("\\expandafter"),
        T_CS!("\\CJK@XXX"),
        T_CS!("\\expandafter"),
        T_CS!("\\string"),
        T_CS!("\\expandafter"),
        act,
        T_CS!("\\else"),
        T_CS!("\\noexpand"),
        act,
        T_CS!("\\fi")
      )
    } else {
      Tokens!(
        T_CS!("\\CJK@@@"),
        T_CS!("\\ifx"),
        T_CS!("\\protect"),
        T_CS!("\\@typeset@protect"),
        T_CS!("\\expandafter"),
        T_CS!("\\expandafter"),
        T_CS!("\\expandafter"),
        T_CS!("\\CJK@XXXX"),
        T_CS!("\\expandafter"),
        T_CS!("\\string"),
        T_CS!("\\expandafter"),
        act,
        T_CS!("\\else"),
        T_CS!("\\noexpand"),
        act,
        T_CS!("\\fi")
      )
    };
    DefMacro!(act, None, expansion);
  }
  Ok(())
}

LoadDefinitions!({
  // KERNEL_CAPABILITIES K10: CJK is built on pdfTeX's byte model
  // (CJK.sty:896-968); run the rest of the document under the byte mouth.
  inputenc_sty::enable_pdftex_byte_mouth()?;
  // CJK.sty:76-84 `\CJK@input` (bxcjkjatype.sty:932 `\CJK@input{UTF8.bdg}`)
  // and the `\CJK@namedef` family it feeds (CJK.sty:925-975; `\CJK@active`
  // = `\relax` for LaTeX, CJK.sty:43): the active-byte meanings the raw
  // binding files install — the same meanings `bind_cjk_utf8_octets` gives.
  RawTeX!(
    r"\let\CJK@active\relax
\def\CJK@input#1{\makeatletter
  \edef\CJK@lesscatcode{\noexpand\catcode`< \the\catcode`<}\catcode`\< 12\relax
  \endlinechar \m@ne \input #1\relax \endlinechar `\^^M \CJK@lesscatcode \makeatother}
\def\CJK@namedef#1{\CJK@active\def#1{\CJK@@@\ifx\protect\@typeset@protect\string #1\else\noexpand #1\fi}}
\def\CJK@namepdef#1{\CJK@active\def#1{\CJK@@@\ifx\protect\@typeset@protect
  \expandafter\expandafter\expandafter\CJK@X\expandafter\string\expandafter#1\else\noexpand #1\fi}}
\def\CJK@nameppdef#1{\CJK@active\def#1{\CJK@@@\ifx\protect\@typeset@protect
  \expandafter\expandafter\expandafter\CJK@XX\expandafter\string\expandafter#1\else\noexpand #1\fi}}
\def\CJK@namepppdef#1{\CJK@active\def#1{\CJK@@@\ifx\protect\@typeset@protect
  \expandafter\expandafter\expandafter\CJK@XXX\expandafter\string\expandafter#1\else\noexpand #1\fi}}
\def\CJK@nameppppdef#1{\CJK@active\def#1{\CJK@@@\ifx\protect\@typeset@protect
  \expandafter\expandafter\expandafter\CJK@XXXX\expandafter\string\expandafter#1\else\noexpand #1\fi}}"
  );
  // ar5iv-bindings/bindings/CJK.sty.ltxml L17-24: CJK environment is a
  // transparent wrapper that passes body through. `leaveHorizontal` +
  // `internal_vertical` mode ensures paragraph breaks inside CJK blocks
  // feed the line-wrapping / height-estimation code correctly instead of
  // accumulating inside an implicit horizontal list.
  DefEnvironment!("{CJK}{}{}", "#body",
    before_digest => { leave_horizontal()?; },
    mode => "internal_vertical"
  );
  DefEnvironment!("{CJK*}{}{}", "#body",
    before_digest => { leave_horizontal()?; },
    mode => "internal_vertical"
  );
  // CJK.sty `\CJKfamily{fam}` selects the CJK font family — a font switch
  // with no XML meaning. (ar5iv-bindings CJK.sty.ltxml:24 expands it to
  // `#1`, which prints the family NAME as body text: kotex's
  // `\CJKfamily{nanummj}` leaked "nanummj" into every cjk-ko document.)
  def_macro_noop("\\CJKfamily{}")?;
  // CJK/xeCJK/ctex SURFACE macros absorbed (perfect-kernel sweep-16
  // `\CJKaddEncHook` = 14 bundles; einfart's minimalist chain pulls
  // CJKpunct/CJKspace raw even in non-CJK docs). Font selection is the
  // D6-fontspec shape (no XML meaning); hooks/encodings are engine
  // bookkeeping. Deep CJK typesetting (kanji classes, pTeX primitives)
  // stays catalogued in DIFFICULT_CASES — these absorbs only stop the
  // undefined-CS cascade.
  // CJK.sty:879/881 `\CJKspace`/`\CJKnospace` (the inter-CJK glue toggle;
  // cjk-ko-doc) and :477/:484 `\CJKencfamily[enc]{family}{shape}` /
  // `\CJKencshape` (per-encoding family selection; bxcjkjatype beamer sample)
  // — presentation only.
  def_macro_noop("\\CJKspace")?;
  def_macro_noop("\\CJKnospace")?;
  def_macro_noop("\\CJKencfamily[]{}{}")?;
  Let!("\\CJKencshape", "\\CJKencfamily");
  def_macro_noop("\\CJKaddEncHook{}{}")?;
  // CJK.sty:232 `\DeclareRobustCommand{\Unicode}[2]` typesets the character
  // at code point `#1*256+#2` (cjkutf8-ko.sty:65,134 `\dotemphchar` =
  // `\Unicode{"02}{"D9}` = U+02D9; witness cjk-ko/cjk-ko-doc `\dotemph`).
  // Our input is Unicode already, so insert the code point as a character
  // token (the texvc `\unicode` shape) — `\char` is font-encoding bound.
  DefPrimitive!("\\Unicode {Number} {Number}", sub[(hi, lo)] {
    let cp = (hi.value_of() as u32) * 256 + lo.value_of() as u32;
    if let Some(ch) = char::from_u32(cp) {
      unread(Tokens!(Token { text: pin_char(ch), code: Catcode::OTHER,
        #[cfg(feature = "token-locators")] loc: 0 }));
    }
  });
  // CJK.sty:915-1012, 1049-1075 + UTF8.bdg:
  // Faithfully implement CJK's active-byte decoder definitions and UTF8 binding.
  // Downstream packages (such as cjkutf8-ko.sty:150-154's "protect utf8 octets" loop)
  // expand active bytes 0x80..0xF4 via `\unexpanded\expandafter{~}`; without these
  // bindings, they hit inputenc's `\@inpenc@undefined` 117 times (cjk-ko-doc fatal).
  //
  // NOTE: Under the LuaTeX persona we bind the active-byte MEANINGS only and
  // leave catcodes 0x80..0xFE as Catcode::OTHER (from utf8_def.rs): input is
  // Unicode codepoints there, and setting catcode ACTIVE would cause accented Latin characters
  // (e.g. U+00E9 'é' in "café") to be intercepted as multi-byte CJK lead bytes.
  DefPrimitive!("\\CJK@loadBinding{}", sub[(binding)] {
    let b = binding.to_string();
    if b.trim().is_empty() || b.trim().eq_ignore_ascii_case("utf8") {
      bind_cjk_utf8_octets()?;
    }
    Ok(Vec::new())
  });
  DefPrimitive!("\\CJK@envStart{}{}{}", sub[(_font, enc, family)] {
    let enc_str = enc.to_string();
    if enc_str.trim().is_empty() || enc_str.trim().eq_ignore_ascii_case("utf8") {
      bind_cjk_utf8_octets()?;
    }
    digest(Tokens!(T_CS!("\\CJKfamily"), family))?;
    Ok(Vec::new())
  });
  def_macro_noop("\\CJKenc{}")?;
  def_macro_noop("\\CJKfontenc{}{}")?;
  def_macro_noop("\\CJK@envEnd")?;
  def_macro_noop("\\CJKtilde")?;
  def_macro_noop("\\nbs")?;
  def_macro_noop("\\setCJKmainfont[]{}[]")?;
  def_macro_noop("\\setCJKsansfont[]{}[]")?;
  def_macro_noop("\\setCJKmonofont[]{}[]")?;
  def_macro_noop("\\setCJKfamilyfont{}[]{}[]")?;
  def_macro_noop("\\newCJKfontfamily DefToken []{}[]")?;
  def_macro_noop("\\CJKsetecglue{}")?;
  def_macro_noop("\\punctstyle{}")?;
  // xeCJK expl3-layer surface raw fntef/underline code invokes directly
  // (XeTeX-engine territory — D9 out-of-scope; the noops keep pdfTeX-model
  // digestion progressing instead of looping on error stubs, fixdif-zh-cn).
  def_macro_noop("\\xeCJK_no_break:")?;
  def_macro_noop("\\xeCJK_allow_break:")?;
  def_macro_noop("\\CJKsymbol{}")?;
  def_macro_noop("\\CJKpunctsymbol{}")?;
  // ctex's pdfTeX layer requires CJKpunct (ctex-engine-pdftex.def:122), which
  // is loaded RAW and re-routes the six declared punctuation codepoints
  // (CJKpunct.sty:442-447: U+2018/2019/201C/201D/2014/2026) through
  // `\CJKpunct@utfasymbol` → `\CJK@punctchar{\CJK@uniPunct}{0}{"80}{byte}`
  // (:449-450) once `\punctstyle{quanjiao}` fires at `\begin{document}`
  // (:389, :372). Real CJK supplies `\CJK@uniPunct` from CJK.enc:291 and
  // `\CJK@punctchar` from a lazily-input `*.chr` glyph selector — neither is
  // ever loaded behind this binding (Perl's CJK.sty.ltxml omits them too:
  // SHARED, 18 ctex docs × 2 errors; jnuexam/jnuexam → 0). The Unicode
  // reduction below mirrors CJKpunct.sty:451-474 (`\CJKpunct@utfbsymbol`, the
  // `plain` style's own rendering of the same low bytes); the glyph spacing
  // quanjiao adds is an 8-bit-font concern with no Unicode-output meaning.
  RawTeX!(concat!(
    r"\xdef\CJK@uniPunct{30, fe, ff}",
    "\n",
    r"\def\CJK@punctchar#1#2#3#4{",
    r"\ifnum#4=148 \textemdash\else",
    r"\ifnum#4=166 \textellipsis\else",
    r"\ifnum#4=152 \textquoteleft\else",
    r"\ifnum#4=153 \textquoteright\else",
    r"\ifnum#4=156 \textquotedblleft\else",
    r"\ifnum#4=157 \textquotedblright\fi\fi\fi\fi\fi\fi}",
    "\n",
  ));
});
