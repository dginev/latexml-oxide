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
});
