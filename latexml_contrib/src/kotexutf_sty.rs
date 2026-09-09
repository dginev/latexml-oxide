//! kotexutf.sty — ko.TeX for pdfTeX (the UTF-8 byte path).
//!
//! The real package runs under the pdfTeX byte mouth (KERNEL_CAPABILITIES
//! K10, `inputenc_sty::enable_pdftex_byte_mouth`): kotexutf-core.tex:100-136
//! makes every UTF-8 lead byte an active reader of its continuation bytes,
//! and kotexutf.sty's josa commands (`\은`/`\를`/`\가`…, the byte-model
//! `\DeclareRobustCommand*\^^ec[2]` family) are control symbols over the
//! lead byte that read the next two bytes. Every non-ASCII character then
//! funnels through kotexutf-core.tex:213 `\protected\def\unihangulchar#1`
//! (the code point as a number) → `\unihangulchar@@` (line-break/josa
//! lookahead) → a glyph in an LUC subfont. Bindings outrank raw: the funnel
//! records the code point for the josa logic (kotexutf.sty:358 `\makejosa`
//! reads `\@josa`, set at kotexutf-core.tex:134) and emits the character
//! itself. Witnesses: kotex-utf-doc (27 josa errors), kotex-doc, cjk-ko-doc
//! via dhucs.sty:44. Guard:
//! `perfect_kernel_batch56::kotexutf_runs_under_the_byte_mouth`.
use latexml_package::prelude::*;

LoadDefinitions!({
  inputenc_sty::enable_pdftex_byte_mouth()?;
  InputDefinitions!("kotexutf", noltxml => true, extension => Some(Cow::Borrowed("sty")));
  // kotexutf-core.tex:213-217 + :134: keep the code point for `\makejosa`,
  // emit the character.
  RawTeX!(
    r"\protected\def\unihangulchar#1{\unih@ngulpoint#1\relax\global\@josa\unih@ngulpoint\lx@kotex@char{#1}}"
  );
  DefPrimitive!("\\lx@kotex@char{Number}", sub[(n)] {
    if let Some(ch) = u32::try_from(n.value_of()).ok().and_then(char::from_u32) {
      unread(Tokens!(Token { text: pin_char(ch), code: Catcode::OTHER,
        #[cfg(feature = "token-locators")] loc: 0 }));
    }
  });
});
