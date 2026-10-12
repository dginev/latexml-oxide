//! rccol.sty — right-centered numeric columns (`R{<int>}{<frac>}`), formatted by fltpoint. No Perl binding.
use latexml_package::prelude::*;

LoadDefinitions!({
  // The cell code (`\rc@begin` … `\rc@end`, `\rc@write`, fltpoint's arithmetic) runs raw.
  InputDefinitions!("rccol", noltxml => true, extension => Some(Cow::Borrowed("sty")));
  // rccol.sty:46-69 `\NC@rewrite@R` rewrites `R[-][<in>][<out>]{<int>}{<frac>}` by assignments and an `\edef`
  // (`\rc@digits{…}`, `\def\rc@withsign{…}`, `\edef\rc@rewrite@scratch{\the\@temptokena>{…}c<{…}}`), run by
  // array.sty's column-type search (array.sty:351-362). The template reader expands a column type and does not run
  // it, so the rewrite is the `\edef`'s result, as tokens: `>{<in sign><sign box>\rc@begin}c<{\rc@end<out
  // sign>\rc@write{<int>}{<frac>}}`. Read as raw tokens it was the cell code taken for column letters, and, with
  // natbib's `\appdef` leaving `\@temptokena` filled, `Fatal:Timeout:PushbackLimit` (1101.1265, 0812.2309,
  // 2609.37147; repro alignment-bindings/rccol_with_natbib_pushback_loop).
  DefMacro!("\\NC@rewrite@R OptionalMatch:- [][]{}{}", sub[(minus, input, output, int, frac)] {
    let mut toks = vec![T_OTHER!(">"), T_BEGIN!()];
    if let Some(input) = input.as_ref() {
      // `\rc@digits{\rcDecimalSignInput{#1}}` (rccol.sty:58)
      toks.push(T_CS!("\\rcDecimalSignInput"));
      toks.push(T_BEGIN!());
      toks.extend(input.unlist_ref().iter().copied());
      toks.push(T_END!());
    }
    if minus.is_some() {
      // `\def\rc@withsign{{\setbox0\hbox{$-$}\hskip\wd0}}` (rccol.sty:52)
      toks.extend(TokenizeInternal!(r"{\setbox0\hbox{$-$}\hskip\wd0}").unlist());
    }
    toks.push(T_CS!("\\rc@begin"));
    toks.push(T_END!());
    toks.push(T_LETTER!("c"));
    toks.push(T_OTHER!("<"));
    toks.push(T_BEGIN!());
    toks.push(T_CS!("\\rc@end"));
    // `\rc@preothertoks{\rcDecimalSignOutput{…}}`: the second optional, else the first (rccol.sty:59-62)
    if let Some(sign) = output.as_ref().or(input.as_ref()) {
      toks.push(T_CS!("\\rcDecimalSignOutput"));
      toks.push(T_BEGIN!());
      toks.extend(sign.unlist_ref().iter().copied());
      toks.push(T_END!());
    }
    toks.push(T_CS!("\\rc@write"));
    toks.push(T_BEGIN!());
    toks.extend(int.unlist_ref().iter().copied());
    toks.push(T_END!());
    toks.push(T_BEGIN!());
    toks.extend(frac.unlist_ref().iter().copied());
    toks.push(T_END!());
    toks.push(T_END!());
    Ok(Tokens::new(toks))
  });
});
