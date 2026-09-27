use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: revsymb.sty.ltxml — REVTeX symbol definitions

  DefMath!("\\lambdabar", "\u{03BB}\u{0304}");
  DefConstructor!("\\mathbb{}", "#1", bounded => true, require_math => true,
    font => {family => "blackboard", series => "medium", shape => "upright"});
  DefMacro!("\\Bbb{}", "\\mathbb{#1}");

  // revsymb4-2.sty:136-154 `\biglb` … `\Biggrb`: `\REV@boldopen`/`\REV@boldclose` set the sized
  // delimiter that follows in poor-man's bold (`\REV@pmb`) as an opening or closing atom. Perl
  // (revsymb.sty.ltxml:31-54): a bounded constructor over the delimiter in the big sizes, bold,
  // with the delimiter's role. Guard: `perfect_kernel_batch56::revsymb_delimiters_are_bold`.
  DefConstructor!("\\biglb TeXDelimiter", "#1", bounded => true, font => sub[_f] { Ok(bold_delimiter_font("big")) },
    after_construct => sub[document, _whatsit] { augment_delimiter_properties(document, "OPEN")?; });
  DefConstructor!("\\bigrb TeXDelimiter", "#1", bounded => true, font => sub[_f] { Ok(bold_delimiter_font("big")) },
    after_construct => sub[document, _whatsit] { augment_delimiter_properties(document, "CLOSE")?; });
  DefConstructor!("\\Biglb TeXDelimiter", "#1", bounded => true, font => sub[_f] { Ok(bold_delimiter_font("Big")) },
    after_construct => sub[document, _whatsit] { augment_delimiter_properties(document, "OPEN")?; });
  DefConstructor!("\\Bigrb TeXDelimiter", "#1", bounded => true, font => sub[_f] { Ok(bold_delimiter_font("Big")) },
    after_construct => sub[document, _whatsit] { augment_delimiter_properties(document, "CLOSE")?; });
  DefConstructor!("\\bigglb TeXDelimiter", "#1", bounded => true, font => sub[_f] { Ok(bold_delimiter_font("bigg")) },
    after_construct => sub[document, _whatsit] { augment_delimiter_properties(document, "OPEN")?; });
  DefConstructor!("\\biggrb TeXDelimiter", "#1", bounded => true, font => sub[_f] { Ok(bold_delimiter_font("bigg")) },
    after_construct => sub[document, _whatsit] { augment_delimiter_properties(document, "CLOSE")?; });
  DefConstructor!("\\Bigglb TeXDelimiter", "#1", bounded => true, font => sub[_f] { Ok(bold_delimiter_font("Bigg")) },
    after_construct => sub[document, _whatsit] { augment_delimiter_properties(document, "OPEN")?; });
  DefConstructor!("\\Biggrb TeXDelimiter", "#1", bounded => true, font => sub[_f] { Ok(bold_delimiter_font("Bigg")) },
    after_construct => sub[document, _whatsit] { augment_delimiter_properties(document, "CLOSE")?; });

  DefMath!("\\gtrsim", "\u{2273}", role => "RELOP", meaning => "greater-than-or-equivalent-to");
  DefMath!("\\lesssim", "\u{2272}", role => "RELOP", meaning => "less-than-or-similar-to");
  Let!("\\agt", "\\gtrsim");
  Let!("\\alt", "\\lesssim");

  DefMath!("\\precsim", "\u{227E}", role => "RELOP", meaning => "precedes-or-equivalent-to");
  DefMath!("\\succsim", "\u{227F}", role => "RELOP", meaning => "succeeds-or-equivalent-to");
  Let!("\\altprecsim", "\\precsim");
  Let!("\\altsuccsim", "\\succsim");

  DefMath!("\\overcirc{}", "\u{030A}", operator_role => "OVERACCENT");
  DefMath!("\\dddot{}", "\u{02D9}\u{02D9}\u{02D9}", operator_role => "OVERACCENT");
  Let!("\\overdots", "\\dddot");
  DefMath!("\\triangleq", "\u{225C}", role => "RELOP");
  Let!("\\corresponds", "\\triangleq");

  // revsymb4-2.sty defines `\REV@<sym>` fallbacks (L57-68) and installs them via
  // `\@ifxundefined\<sym>{\let\<sym>\REV@<sym>}` (L156-163) — for documents that
  // don't load amssymb. This binding already defines the symbols directly, but
  // in `rawstyles`/ar5iv mode (INCLUDE_STYLES, cortex_worker `--standalone`'s
  // ar5iv profile) the raw revsymb4-2.sty fallback can still fire and reference
  // an undefined `\REV@lesssim` (witness 2106.00028: `\lesssim` without amssymb →
  // `undefined:\REV@lesssim`). Alias the `\REV@*` names to our symbols so the
  // fallback resolves. (Perl LaTeXML's `latexml --preload=ar5iv.sty` is clean
  // here; this restores ar5iv-profile parity.)
  Let!("\\REV@lesssim", "\\lesssim");
  Let!("\\REV@gtrsim", "\\gtrsim");
  Let!("\\REV@triangleq", "\\triangleq");
  Let!("\\REV@dddot", "\\dddot");

  DefMath!("\\loarrow{}", "\u{20D6}", operator_role => "OVERACCENT");
  DefMath!("\\roarrow{}", "\u{20D7}", operator_role => "OVERACCENT");
  DefConstructor!("\\openone", "1",
    font => {family => "blackboard", series => "medium", shape => "upright"});
  DefMath!("\\overstar{}", "\u{0359}", operator_role => "OVERACCENT");
  DefMath!("\\tensor{}", "\u{20E1}", operator_role => "OVERACCENT");
});

/// Perl's `font => { size => $size, series => 'bold', forcebold => 1 }`: the symbolic size read at
/// digestion, against the document's nominal size (as `math_common`'s `\big`).
fn bold_delimiter_font(size: &str) -> Font {
  Font {
    series: Some(Cow::Borrowed("bold")),
    forcebold: Some(true),
    ..symbolic_font_size(size)
  }
}
