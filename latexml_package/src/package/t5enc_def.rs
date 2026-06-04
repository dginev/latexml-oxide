use crate::prelude::*;
LoadDefinitions!({
  // Perl: t5enc.def.ltxml — Vietnamese T5 encoding uses many stacked
  // accents; LaTeXML pre-defines the common cases before loading the
  // raw t5enc.def so accent composition lands on glyphs we can render.
  DefAccent!("\\texthookabove", '\u{0309}', "'");
  Let!("\\h", "\\texthookabove");
  DefPrimitive!("\\Acircumflex", "\u{00C2}");
  DefPrimitive!("\\Abreve", "\u{0102}");
  DefPrimitive!("\\Ecircumflex", "\u{00CA}");
  DefPrimitive!("\\Ocircumflex", "\u{00D4}");
  DefPrimitive!("\\Ohorn", "\u{01A0}");
  DefPrimitive!("\\Uhorn", "\u{01AF}");
  DefPrimitive!("\\acircumflex", "\u{00E2}");
  DefPrimitive!("\\abreve", "\u{0103}");
  DefPrimitive!("\\ecircumflex", "\u{00EA}");
  DefPrimitive!("\\ocircumflex", "\u{00F4}");
  DefPrimitive!("\\ohorn", "\u{01A1}");
  DefPrimitive!("\\uhorn", "\u{01B0}");
  DefAccent!("\\k", '\u{0328}', "\u{02DB}");
  // vntex's `\textviet{…}` (`\DeclareTextFontCommand{\textviet}{\viet}`)
  // selects the Vietnamese font and typesets its argument. Font
  // selection is typesetting-only for our XML output, so the command is
  // a content passthrough. vntex.sty isn't installed in TeX Live, so
  // neither Perl nor our raw-load defines it — define it here (with the
  // rest of the Vietnamese command set) so author names like
  // `C\textviet{\uhorn\`{\ohorn}}ng` survive. Witness 2005.09299
  // (`\usepackage[vietnamese,english]{babel}`).
  DefMacro!("\\textviet{}", "#1");
  InputDefinitions!("t5enc", extension => Some("def".into()), noltxml => true);
});
