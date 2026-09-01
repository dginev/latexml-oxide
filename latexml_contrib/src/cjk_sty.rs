use latexml_package::prelude::*;

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
  def_macro_noop("\\CJKaddEncHook{}{}")?;
  def_macro_noop("\\CJK@loadBinding{}")?;
  def_macro_noop("\\CJK@envStart{}{}{}")?;
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
    r"\xdef\CJK@uniPunct{30, fe, ff}", "\n",
    r"\def\CJK@punctchar#1#2#3#4{",
    r"\ifnum#4=148 \textemdash\else",
    r"\ifnum#4=166 \textellipsis\else",
    r"\ifnum#4=152 \textquoteleft\else",
    r"\ifnum#4=153 \textquoteright\else",
    r"\ifnum#4=156 \textquotedblleft\else",
    r"\ifnum#4=157 \textquotedblright\fi\fi\fi\fi\fi\fi}", "\n",
  ));
});
