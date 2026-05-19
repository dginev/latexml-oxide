//! pifont.sty binding — Pi font symbols (dingbats)
use crate::prelude::*;

/// DEP-20 helper for empty-body `DefPrimitive!("\\cs[opt-spec]", None);` stubs.
/// Mirrors `def_macro_noop` but routes through `def_primitive` so the CS
/// is registered as a digestion-time primitive rather than an expandable
/// macro. Body=None is treated as a no-op primitive (no Box emitted).
fn def_primitive_noop(proto: &str) -> Result<()> {
  let (cs_tok, params) = parse_prototype(proto, true)?;
  def_primitive(cs_tok, params, None, PrimitiveOptions::default())?;
  Ok(())
}


#[rustfmt::skip]
LoadDefinitions!({
  // Perl: pifont.sty.ltxml — uses pzd fontmap

  // \Pifont{font-name} — switch to the named Pi font. Used by packages
  // like adforn.sty which redeclares \pzd via `\Pifont{paadr}`. For
  // XML/HTML output the font-family change has no semantic effect; the
  // resulting `\char N` produces a literal codepoint regardless of the
  // declared font. Stub as no-op so downstream `\char` / `\Pisymbol`
  // calls still resolve correctly. Witness: 2502.16764 (adforn.sty).
  DefMacro!("\\Pifont{}", "");

  // \Pisymbol{font}{code} — decode a codepoint from a Pi font
  DefPrimitive!("\\Pisymbol{}{Number}", sub[(pifont, code)] {
    let font_name = pifont.unwrap().to_string();
    let code_val = code.value_of();
    let (glyph, font) = font_decode(code_val as i32, Some(&font_name), None);
    // Perl: Box($glyph, $font, ...) — undef glyph produces empty box
    let sym = match glyph {
      Some(ch) => arena::pin(ch.to_string()),
      None => arena::pin_static(""),
    };
    Ok(Digested::from(Tbox::new(
      sym,
      font,
      None,
      Tokens::new(vec![]),
      SymHashMap::default(),
    )))
  });

  // \lx@Picountersymbol{font}{counter}{codebase} — decode counter-offset symbol
  DefPrimitive!("\\lx@Picountersymbol{}{}{Number}", sub[(pifont, counter, codebase)] {
    let font_name = pifont.unwrap().to_string();
    let counter_name = counter.unwrap().to_string();
    let base = codebase.value_of();
    let counter_val = lookup_register(&s!("\\c@{counter_name}"), vec![])?
      .map(|rv| rv.value_of())
      .unwrap_or(0);
    let code = base + counter_val - 1;
    let (glyph, font) = font_decode(code as i32, Some(&font_name), None);
    // Perl: Box($glyph, $font, ...) — undef glyph produces empty box
    let sym = match glyph {
      Some(ch) => arena::pin(ch.to_string()),
      None => arena::pin_static(""),
    };
    Ok(Digested::from(Tbox::new(
      sym,
      font,
      None,
      Tokens::new(vec![]),
      SymHashMap::default(),
    )))
  });

  DefMacro!("\\Pilist{}{}", "\\list{\\Pisymbol{#1}{#2}}{}");
  DefMacro!("\\endPilist", "\\endlist");

  // \lx@defpiautolabel{font}{base} — define pi font auto-labels for enumerate
  DefMacro!("\\lx@defpiautolabel{}{}", sub[(font, base)] {
    let font_str = font.unwrap().to_string();
    let base_str = base.unwrap().to_string();
    let level = (lookup_int("enumlevel").max(0) + 1) as i64;
    let postfix = roman_aux(level);
    // DefMacroI for \theenumX, \p@enumX, \labelenumX
    let the_body = s!("\\lx@Picountersymbol{{{font_str}}}{{enum{postfix}}}{{{base_str}}}");
    let the_tokens = latexml_core::mouth::tokenize_internal(&the_body);
    def_macro(T_CS!(s!("\\theenum{postfix}")), None, the_tokens, None)?;
    let empty_tokens = Tokens::new(vec![]);
    def_macro(T_CS!(s!("\\p@enum{postfix}")), None, empty_tokens, None)?;
    let label_body = s!("\\theenum{postfix}");
    let label_tokens = latexml_core::mouth::tokenize_internal(&label_body);
    def_macro(T_CS!(s!("\\labelenum{postfix}")), None, label_tokens, None)?;
    Ok(Tokens::new(vec![]))
  });

  DefMacro!("\\Piautolist{}{}", "\\lx@defpiautolabel{#1}{#2}\\enumerate");
  DefMacro!("\\endPiautolist", "\\endenumerate");

  // Don't know what to do with these.
  def_primitive_noop("\\Piline{}{Number}")?;
  def_primitive_noop("\\Pifill{}{Number}")?;

  // Dingbats shortcuts using pzd encoding
  DefMacro!("\\ding{}", "\\Pisymbol{pzd}{#1}");

  DefMacro!("\\dinglist", "\\Pilist{pzd}");
  DefMacro!("\\enddinglist", "\\endPilist");
  DefMacro!("\\dingautolist", "\\Piautolist{pzd}");
  DefMacro!("\\enddingautolist", "\\endPiautolist");

  // Don't know what to do with these.
  def_primitive_noop("\\dingline{Number}")?;
  def_primitive_noop("\\dingfill{Number}")?;
});
