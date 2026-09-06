use latexml_package::prelude::*;

fn bind_cjk_utf8_octets() -> Result<()> {
  let cjk_at_at_at = T_CS!("\\CJK@@@");
  if lookup_definition(&cjk_at_at_at)?.is_none() {
    def_macro_noop("\\CJK@@@")?;
    def_macro_noop("\\CJK@X{}")?;
    def_macro_noop("\\CJK@XX{}{}")?;
    def_macro_noop("\\CJK@XXp{}{}")?;
    def_macro_noop("\\CJK@XXX{}{}{}")?;
    def_macro_noop("\\CJK@XXXp{}{}{}{}")?;
    def_macro_noop("\\CJK@XXXX{}{}{}{}")?;
    def_macro_noop("\\CJK@XXXXp{}{}{}{}{}")?;
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
  DefMacro!("\\CJKfamily{}", "#1");
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
  // CJK.sty:915-1012, 1049-1075 + UTF8.bdg:
  // Faithfully implement CJK's active-byte decoder definitions and UTF8 binding.
  // Downstream packages (such as cjkutf8-ko.sty:150-154's "protect utf8 octets" loop)
  // expand active bytes 0x80..0xF4 via `\unexpanded\expandafter{~}`; without these
  // bindings, they hit inputenc's `\@inpenc@undefined` 117 times (cjk-ko-doc fatal).
  //
  // NOTE: We bind the active-byte MEANINGS only, and intentionally leave catcodes
  // 0x80..0xFE as Catcode::OTHER (from utf8_def.rs). In latexml-oxide, input is
  // Unicode codepoints; setting catcode ACTIVE would cause accented Latin characters
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
